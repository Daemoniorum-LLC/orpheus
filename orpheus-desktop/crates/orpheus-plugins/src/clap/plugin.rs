//! CLAP Plugin Loader
//!
//! Handles loading CLAP plugins from shared libraries.

use super::ffi::*;
use super::host::ClapHost;
use crate::{Error, Result};
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// CLAP plugin loader
pub struct ClapPluginLoader {
    /// Loaded library (kept alive for plugin lifetime)
    #[allow(dead_code)]
    library: Library,
    /// Plugin entry point
    entry: *const clap_plugin_entry,
    /// Plugin factory
    factory: *const clap_plugin_factory,
    /// Path to the plugin
    #[allow(dead_code)]
    path: PathBuf,
    /// Whether init was called
    initialized: bool,
}

impl ClapPluginLoader {
    /// Load a CLAP plugin from a path
    ///
    /// # Safety
    /// This function loads and executes code from a shared library.
    /// Only load trusted plugins.
    pub fn load(path: &Path) -> Result<Self> {
        info!(path = %path.display(), "Loading CLAP plugin");

        // Load the shared library
        let library = unsafe {
            Library::new(path).map_err(|e| {
                Error::LoadError(format!("Failed to load library {}: {}", path.display(), e))
            })?
        };

        // Find the clap_entry symbol
        let entry: *const clap_plugin_entry = unsafe {
            let symbol: Symbol<*const clap_plugin_entry> = library
                .get(b"clap_entry\0")
                .map_err(|e| Error::LoadError(format!("Missing clap_entry symbol: {}", e)))?;
            *symbol
        };

        if entry.is_null() {
            return Err(Error::LoadError("clap_entry is null".to_string()));
        }

        // Verify CLAP version compatibility
        let version = unsafe { (*entry).clap_version };
        if version.major != CLAP_VERSION.major {
            return Err(Error::LoadError(format!(
                "Incompatible CLAP version: plugin has {}.{}.{}, host requires {}.x.x",
                version.major, version.minor, version.revision, CLAP_VERSION.major
            )));
        }

        debug!(
            version = format!("{}.{}.{}", version.major, version.minor, version.revision),
            "CLAP version compatible"
        );

        // Initialize the plugin
        let path_cstr = CString::new(path.to_string_lossy().as_bytes())
            .map_err(|_| Error::LoadError("Invalid path".to_string()))?;

        let init_fn = unsafe { (*entry).init };
        if let Some(init) = init_fn {
            let success = unsafe { init(path_cstr.as_ptr()) };
            if !success {
                return Err(Error::InitializationError(
                    "Plugin init() returned false".to_string(),
                ));
            }
        }

        // Get the plugin factory
        let factory_id = CString::new(CLAP_PLUGIN_FACTORY_ID)
            .map_err(|_| Error::LoadError("Invalid factory ID".to_string()))?;

        let factory = unsafe {
            let get_factory = (*entry)
                .get_factory
                .ok_or_else(|| Error::LoadError("Missing get_factory".to_string()))?;
            get_factory(factory_id.as_ptr()) as *const clap_plugin_factory
        };

        if factory.is_null() {
            // Deinit before returning error
            if let Some(deinit) = unsafe { (*entry).deinit } {
                unsafe { deinit() };
            }
            return Err(Error::LoadError("Failed to get plugin factory".to_string()));
        }

        Ok(Self {
            library,
            entry,
            factory,
            path: path.to_path_buf(),
            initialized: true,
        })
    }

    /// Get the number of plugins in this bundle
    pub fn plugin_count(&self) -> u32 {
        unsafe {
            if let Some(get_count) = (*self.factory).get_plugin_count {
                get_count(self.factory)
            } else {
                0
            }
        }
    }

    /// Get descriptor for a plugin at index
    pub fn get_descriptor(&self, index: u32) -> Option<PluginDescriptor> {
        unsafe {
            let get_desc = (*self.factory).get_plugin_descriptor?;
            let desc = get_desc(self.factory, index);
            if desc.is_null() {
                return None;
            }

            Some(PluginDescriptor::from_raw(desc))
        }
    }

    /// Get all plugin descriptors
    pub fn descriptors(&self) -> Vec<PluginDescriptor> {
        let count = self.plugin_count();
        (0..count).filter_map(|i| self.get_descriptor(i)).collect()
    }

    /// Create a plugin instance
    pub fn create_instance(
        &self,
        plugin_id: &str,
        host: &ClapHost,
    ) -> Result<ClapPluginInstance> {
        let plugin_id_cstr = CString::new(plugin_id)
            .map_err(|_| Error::LoadError("Invalid plugin ID".to_string()))?;

        let plugin = unsafe {
            let create = (*self.factory)
                .create_plugin
                .ok_or_else(|| Error::LoadError("Missing create_plugin".to_string()))?;
            create(self.factory, host.as_ptr(), plugin_id_cstr.as_ptr())
        };

        if plugin.is_null() {
            return Err(Error::LoadError(format!(
                "Failed to create plugin: {}",
                plugin_id
            )));
        }

        // Initialize the plugin
        let success = unsafe {
            if let Some(init) = (*plugin).init {
                init(plugin)
            } else {
                true // No init function is okay
            }
        };

        if !success {
            // Destroy on failure
            unsafe {
                if let Some(destroy) = (*plugin).destroy {
                    destroy(plugin);
                }
            }
            return Err(Error::InitializationError(
                "Plugin init() returned false".to_string(),
            ));
        }

        info!(plugin_id = plugin_id, "Created CLAP plugin instance");

        Ok(ClapPluginInstance {
            plugin,
            plugin_id: plugin_id.to_string(),
            activated: false,
            processing: false,
        })
    }
}

impl Drop for ClapPluginLoader {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                if let Some(deinit) = (*self.entry).deinit {
                    deinit();
                }
            }
        }
    }
}

/// Plugin descriptor information
#[derive(Debug, Clone)]
pub struct PluginDescriptor {
    /// Unique plugin ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Vendor name
    pub vendor: String,
    /// Plugin URL
    pub url: String,
    /// Manual URL
    pub manual_url: String,
    /// Support URL
    pub support_url: String,
    /// Version string
    pub version: String,
    /// Description
    pub description: String,
    /// Feature tags
    pub features: Vec<String>,
}

impl PluginDescriptor {
    /// Create from raw descriptor pointer
    ///
    /// # Safety
    /// The pointer must be valid and point to a properly initialized descriptor.
    unsafe fn from_raw(raw: *const clap_plugin_descriptor) -> Self {
        fn str_from_ptr(ptr: *const std::ffi::c_char) -> String {
            if ptr.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
            }
        }

        fn features_from_ptr(ptr: *const *const std::ffi::c_char) -> Vec<String> {
            if ptr.is_null() {
                return Vec::new();
            }
            let mut features = Vec::new();
            let mut current = ptr;
            unsafe {
                while !(*current).is_null() {
                    features.push(CStr::from_ptr(*current).to_string_lossy().into_owned());
                    current = current.add(1);
                }
            }
            features
        }

        Self {
            id: str_from_ptr((*raw).id),
            name: str_from_ptr((*raw).name),
            vendor: str_from_ptr((*raw).vendor),
            url: str_from_ptr((*raw).url),
            manual_url: str_from_ptr((*raw).manual_url),
            support_url: str_from_ptr((*raw).support_url),
            version: str_from_ptr((*raw).version),
            description: str_from_ptr((*raw).description),
            features: features_from_ptr((*raw).features),
        }
    }
}

/// CLAP plugin instance
pub struct ClapPluginInstance {
    /// Raw plugin pointer
    plugin: *const clap_plugin,
    /// Plugin ID
    plugin_id: String,
    /// Whether the plugin is activated
    activated: bool,
    /// Whether processing is started
    processing: bool,
}

impl ClapPluginInstance {
    /// Get the plugin ID
    pub fn id(&self) -> &str {
        &self.plugin_id
    }

    /// Activate the plugin for processing
    pub fn activate(&mut self, sample_rate: f64, min_frames: u32, max_frames: u32) -> Result<()> {
        if self.activated {
            return Ok(());
        }

        let success = unsafe {
            if let Some(activate) = (*self.plugin).activate {
                activate(self.plugin, sample_rate, min_frames, max_frames)
            } else {
                true
            }
        };

        if success {
            self.activated = true;
            debug!(
                plugin_id = self.plugin_id,
                sample_rate = sample_rate,
                "Plugin activated"
            );
            Ok(())
        } else {
            Err(Error::InitializationError(
                "Plugin activation failed".to_string(),
            ))
        }
    }

    /// Deactivate the plugin
    pub fn deactivate(&mut self) {
        if !self.activated {
            return;
        }

        if self.processing {
            self.stop_processing();
        }

        unsafe {
            if let Some(deactivate) = (*self.plugin).deactivate {
                deactivate(self.plugin);
            }
        }
        self.activated = false;
        debug!(plugin_id = self.plugin_id, "Plugin deactivated");
    }

    /// Start audio processing
    pub fn start_processing(&mut self) -> Result<()> {
        if !self.activated {
            return Err(Error::ProcessingError(
                "Plugin must be activated first".to_string(),
            ));
        }

        if self.processing {
            return Ok(());
        }

        let success = unsafe {
            if let Some(start) = (*self.plugin).start_processing {
                start(self.plugin)
            } else {
                true
            }
        };

        if success {
            self.processing = true;
            Ok(())
        } else {
            Err(Error::ProcessingError(
                "Failed to start processing".to_string(),
            ))
        }
    }

    /// Stop audio processing
    pub fn stop_processing(&mut self) {
        if !self.processing {
            return;
        }

        unsafe {
            if let Some(stop) = (*self.plugin).stop_processing {
                stop(self.plugin);
            }
        }
        self.processing = false;
    }

    /// Process audio
    ///
    /// # Safety
    /// The process struct must be properly initialized with valid buffer pointers.
    pub unsafe fn process(&mut self, process: *const clap_process) -> clap_process_status {
        if !self.processing {
            return clap_process_status::CLAP_PROCESS_ERROR;
        }

        if let Some(process_fn) = (*self.plugin).process {
            process_fn(self.plugin, process)
        } else {
            clap_process_status::CLAP_PROCESS_CONTINUE
        }
    }

    /// Reset plugin state
    pub fn reset(&mut self) {
        unsafe {
            if let Some(reset) = (*self.plugin).reset {
                reset(self.plugin);
            }
        }
    }

    /// Call main thread callback (for GUI updates etc.)
    pub fn on_main_thread(&mut self) {
        unsafe {
            if let Some(callback) = (*self.plugin).on_main_thread {
                callback(self.plugin);
            }
        }
    }

    /// Check if plugin is activated
    pub fn is_activated(&self) -> bool {
        self.activated
    }

    /// Check if plugin is processing
    pub fn is_processing(&self) -> bool {
        self.processing
    }

    /// Get raw plugin pointer for extension access
    pub fn as_ptr(&self) -> *const clap_plugin {
        self.plugin
    }
}

impl Drop for ClapPluginInstance {
    fn drop(&mut self) {
        // Ensure proper shutdown sequence
        if self.processing {
            self.stop_processing();
        }
        if self.activated {
            self.deactivate();
        }

        // Destroy the plugin
        unsafe {
            if let Some(destroy) = (*self.plugin).destroy {
                destroy(self.plugin);
            }
        }
    }
}

// Safety: ClapPluginInstance is not thread-safe by default
// The plugin pointer is only accessed from a single thread

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_descriptor_default() {
        // Test with empty strings
        let desc = PluginDescriptor {
            id: String::new(),
            name: "Test".to_string(),
            vendor: "Vendor".to_string(),
            url: String::new(),
            manual_url: String::new(),
            support_url: String::new(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            features: vec!["instrument".to_string()],
        };

        assert_eq!(desc.name, "Test");
        assert_eq!(desc.version, "1.0.0");
        assert_eq!(desc.features.len(), 1);
    }

    #[test]
    fn test_loader_nonexistent_path() {
        let result = ClapPluginLoader::load(Path::new("/nonexistent/plugin.clap"));
        assert!(result.is_err());
    }

    #[test]
    fn test_loader_invalid_library() {
        // Try to load a non-library file
        let result = ClapPluginLoader::load(Path::new("/etc/passwd"));
        assert!(result.is_err());
    }
}
