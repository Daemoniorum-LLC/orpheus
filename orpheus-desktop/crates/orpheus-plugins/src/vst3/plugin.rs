//! VST3 Plugin Loader
//!
//! Handles loading VST3 plugins from shared libraries.

use super::com::*;
use super::ffi::*;
use super::host::Vst3Host;
use crate::{Error, Result};
use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};
use std::ptr;
use tracing::{debug, info};

/// VST3 plugin loader
pub struct Vst3PluginLoader {
    /// Loaded library (kept alive for plugin lifetime)
    #[allow(dead_code)]
    _library: Library,
    /// Plugin factory
    factory: *mut IPluginFactory,
    /// Path to the plugin
    #[allow(dead_code)]
    path: PathBuf,
    /// Whether module was initialized
    #[allow(dead_code)]
    initialized: bool,
}

impl Vst3PluginLoader {
    /// Load a VST3 plugin from a path
    ///
    /// # Safety
    /// This function loads and executes code from a shared library.
    /// Only load trusted plugins.
    pub fn load(path: &Path) -> Result<Self> {
        info!(path = %path.display(), "Loading VST3 plugin");

        // Determine the actual library path within the bundle
        let lib_path = Self::find_library_in_bundle(path)?;

        // Load the shared library
        let library = unsafe {
            Library::new(&lib_path).map_err(|e| {
                Error::LoadError(format!("Failed to load library {}: {}", lib_path.display(), e))
            })?
        };

        // Try to call InitModule (Linux) or bundleEntry (macOS)
        let initialized = unsafe {
            // Try Linux entry
            let linux_init: std::result::Result<Symbol<InitModuleFn>, _> =
                library.get(b"InitModule\0");
            let macos_init: std::result::Result<Symbol<InitModuleFn>, _> =
                library.get(b"bundleEntry\0");

            if let Ok(init_fn) = linux_init {
                init_fn()
            } else if let Ok(init_fn) = macos_init {
                init_fn()
            } else {
                true // No init function is okay
            }
        };

        if !initialized {
            return Err(Error::InitializationError(
                "Module initialization failed".to_string(),
            ));
        }

        // Get the plugin factory
        let factory = unsafe {
            let get_factory: Symbol<GetPluginFactoryFn> = library
                .get(b"GetPluginFactory\0")
                .map_err(|e| Error::LoadError(format!("Missing GetPluginFactory symbol: {}", e)))?;

            get_factory()
        };

        if factory.is_null() {
            // Try to exit the module
            Self::exit_module(&library);
            return Err(Error::LoadError("GetPluginFactory returned null".to_string()));
        }

        debug!(path = %path.display(), "VST3 plugin loaded successfully");

        Ok(Self {
            _library: library,
            factory,
            path: path.to_path_buf(),
            initialized: true,
        })
    }

    /// Find the actual library file within a VST3 bundle
    fn find_library_in_bundle(bundle_path: &Path) -> Result<PathBuf> {
        // VST3 bundles have platform-specific layouts:
        // Windows: MyPlugin.vst3/Contents/x86_64-win/MyPlugin.vst3
        // macOS: MyPlugin.vst3/Contents/MacOS/MyPlugin
        // Linux: MyPlugin.vst3/Contents/x86_64-linux/MyPlugin.so

        let bundle_name = bundle_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::LoadError("Invalid bundle path".to_string()))?;

        #[cfg(target_os = "linux")]
        {
            let lib_path = bundle_path
                .join("Contents")
                .join("x86_64-linux")
                .join(format!("{}.so", bundle_name));

            if lib_path.exists() {
                return Ok(lib_path);
            }

            // Try alternate paths
            let alt_path = bundle_path
                .join("Contents")
                .join("x86-linux")
                .join(format!("{}.so", bundle_name));

            if alt_path.exists() {
                return Ok(alt_path);
            }

            // Maybe it's just a .so file directly
            if bundle_path.is_file() {
                return Ok(bundle_path.to_path_buf());
            }

            Err(Error::LoadError(format!(
                "Cannot find library in VST3 bundle: {}",
                bundle_path.display()
            )))
        }

        #[cfg(target_os = "macos")]
        {
            let lib_path = bundle_path
                .join("Contents")
                .join("MacOS")
                .join(bundle_name);

            if lib_path.exists() {
                return Ok(lib_path);
            }

            Err(Error::LoadError(format!(
                "Cannot find library in VST3 bundle: {}",
                bundle_path.display()
            )))
        }

        #[cfg(target_os = "windows")]
        {
            let lib_path = bundle_path
                .join("Contents")
                .join("x86_64-win")
                .join(format!("{}.vst3", bundle_name));

            if lib_path.exists() {
                return Ok(lib_path);
            }

            // Try alternate path
            let alt_path = bundle_path
                .join("Contents")
                .join("x86-win")
                .join(format!("{}.vst3", bundle_name));

            if alt_path.exists() {
                return Ok(alt_path);
            }

            // Maybe it's a standalone DLL
            if bundle_path.is_file() {
                return Ok(bundle_path.to_path_buf());
            }

            Err(Error::LoadError(format!(
                "Cannot find library in VST3 bundle: {}",
                bundle_path.display()
            )))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(Error::UnsupportedFormat(
                "Unsupported platform".to_string(),
            ))
        }
    }

    /// Exit the module
    fn exit_module(library: &Library) {
        unsafe {
            let linux_exit: std::result::Result<Symbol<ExitModuleFn>, _> =
                library.get(b"ExitModule\0");
            let macos_exit: std::result::Result<Symbol<ExitModuleFn>, _> =
                library.get(b"bundleExit\0");

            if let Ok(exit_fn) = linux_exit {
                let _ = exit_fn();
            } else if let Ok(exit_fn) = macos_exit {
                let _ = exit_fn();
            }
        }
    }

    /// Get factory info
    pub fn get_factory_info(&self) -> Result<FactoryInfo> {
        let mut info = PFactoryInfo::default();

        let result = unsafe {
            let vtbl = &*(*self.factory).vtbl;
            (vtbl.getFactoryInfo)(self.factory as *mut _, &mut info)
        };

        if result != K_RESULT_OK {
            return Err(Error::LoadError("Failed to get factory info".to_string()));
        }

        Ok(FactoryInfo::from_raw(&info))
    }

    /// Get the number of plugin classes
    pub fn class_count(&self) -> i32 {
        unsafe {
            let vtbl = &*(*self.factory).vtbl;
            (vtbl.countClasses)(self.factory as *mut _)
        }
    }

    /// Get class info by index
    pub fn get_class_info(&self, index: i32) -> Result<ClassInfo> {
        let mut info = PClassInfo::default();

        let result = unsafe {
            let vtbl = &*(*self.factory).vtbl;
            (vtbl.getClassInfo)(self.factory as *mut _, index, &mut info)
        };

        if result != K_RESULT_OK {
            return Err(Error::LoadError(format!(
                "Failed to get class info at index {}",
                index
            )));
        }

        Ok(ClassInfo::from_raw(&info))
    }

    /// Get all class infos
    pub fn classes(&self) -> Vec<ClassInfo> {
        let count = self.class_count();
        (0..count)
            .filter_map(|i| self.get_class_info(i).ok())
            .collect()
    }

    /// Create a plugin instance
    pub fn create_instance(
        &self,
        cid: &TUID,
        _host: &Vst3Host,
    ) -> Result<Vst3PluginInstance> {
        let mut component: *mut c_void = ptr::null_mut();

        let result = unsafe {
            let vtbl = &*(*self.factory).vtbl;
            (vtbl.createInstance)(
                self.factory as *mut _,
                cid as *const _,
                &ICOMPONENT_IID as *const _,
                &mut component,
            )
        };

        if result != K_RESULT_OK || component.is_null() {
            return Err(Error::LoadError("Failed to create component".to_string()));
        }

        info!("Created VST3 plugin instance");

        Ok(Vst3PluginInstance {
            component: component as *mut IComponent,
            processor: ptr::null_mut(),
            initialized: false,
            active: false,
            processing: false,
        })
    }
}

impl Drop for Vst3PluginLoader {
    fn drop(&mut self) {
        // Release the factory
        if !self.factory.is_null() {
            unsafe {
                let unknown = self.factory as *const FUnknown;
                (*unknown).release();
            }
        }

        // Note: The library exit is called when _library is dropped
    }
}

/// Factory info (Rust-friendly version)
#[derive(Debug, Clone)]
pub struct FactoryInfo {
    /// Vendor name
    pub vendor: String,
    /// URL
    pub url: String,
    /// Email
    pub email: String,
    /// Flags
    pub flags: i32,
}

impl FactoryInfo {
    fn from_raw(raw: &PFactoryInfo) -> Self {
        Self {
            vendor: Self::string_from_bytes(&raw.vendor),
            url: Self::string_from_bytes(&raw.url),
            email: Self::string_from_bytes(&raw.email),
            flags: raw.flags,
        }
    }

    fn string_from_bytes(bytes: &[u8]) -> String {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    }
}

/// Class info (Rust-friendly version)
#[derive(Debug, Clone)]
pub struct ClassInfo {
    /// Class ID
    pub cid: TUID,
    /// Cardinality
    pub cardinality: i32,
    /// Category
    pub category: String,
    /// Name
    pub name: String,
}

impl ClassInfo {
    fn from_raw(raw: &PClassInfo) -> Self {
        Self {
            cid: raw.cid,
            cardinality: raw.cardinality,
            category: Self::string_from_bytes(&raw.category),
            name: Self::string_from_bytes(&raw.name),
        }
    }

    fn string_from_bytes(bytes: &[u8]) -> String {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    }
}

/// VST3 plugin instance
pub struct Vst3PluginInstance {
    /// Component interface
    component: *mut IComponent,
    /// Audio processor interface (if available)
    processor: *mut IAudioProcessor,
    /// Whether initialized
    initialized: bool,
    /// Whether active
    active: bool,
    /// Whether processing
    processing: bool,
}

use std::ffi::c_void;

impl Vst3PluginInstance {
    /// Initialize the plugin
    pub fn initialize(&mut self, host: &Vst3Host) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        let result = unsafe {
            let vtbl = &*(*self.component).vtbl;
            (vtbl.base.initialize)(self.component as *mut _, host.as_ptr())
        };

        if result != K_RESULT_OK {
            return Err(Error::InitializationError(
                "Component initialization failed".to_string(),
            ));
        }

        // Query for IAudioProcessor
        self.processor = unsafe {
            let unknown = self.component as *const FUnknown;
            let mut proc: *mut c_void = ptr::null_mut();
            let result = ((*(*unknown).vtbl).queryInterface)(
                unknown as *mut _,
                &IAUDIO_PROCESSOR_IID,
                &mut proc,
            );
            if result == K_RESULT_OK && !proc.is_null() {
                proc as *mut IAudioProcessor
            } else {
                ptr::null_mut()
            }
        };

        self.initialized = true;
        debug!("VST3 plugin initialized");
        Ok(())
    }

    /// Terminate the plugin
    pub fn terminate(&mut self) {
        if !self.initialized {
            return;
        }

        if self.processing {
            let _ = self.set_processing(false);
        }
        if self.active {
            let _ = self.set_active(false);
        }

        // Release processor if we have one
        if !self.processor.is_null() {
            unsafe {
                let unknown = self.processor as *const FUnknown;
                (*unknown).release();
            }
            self.processor = ptr::null_mut();
        }

        // Terminate component
        unsafe {
            let vtbl = &*(*self.component).vtbl;
            (vtbl.base.terminate)(self.component as *mut _);
        }

        self.initialized = false;
        debug!("VST3 plugin terminated");
    }

    /// Set active state
    pub fn set_active(&mut self, state: bool) -> Result<()> {
        if !self.initialized {
            return Err(Error::NotFound("Plugin not initialized".to_string()));
        }

        let result = unsafe {
            let vtbl = &*(*self.component).vtbl;
            (vtbl.setActive)(self.component as *mut _, state as u8)
        };

        if result != K_RESULT_OK {
            return Err(Error::ProcessingError("Failed to set active state".to_string()));
        }

        self.active = state;
        debug!(active = state, "VST3 plugin active state changed");
        Ok(())
    }

    /// Setup processing
    pub fn setup_processing(&mut self, setup: &ProcessSetup) -> Result<()> {
        if self.processor.is_null() {
            return Err(Error::NotFound("No audio processor".to_string()));
        }

        let result = unsafe {
            let vtbl = &*(*self.processor).vtbl;
            (vtbl.setupProcessing)(self.processor as *mut _, setup as *const _ as *mut _)
        };

        if result != K_RESULT_OK {
            return Err(Error::ProcessingError("Failed to setup processing".to_string()));
        }

        debug!(
            sample_rate = setup.sampleRate,
            block_size = setup.maxSamplesPerBlock,
            "VST3 processing setup"
        );
        Ok(())
    }

    /// Set processing state
    pub fn set_processing(&mut self, state: bool) -> Result<()> {
        if self.processor.is_null() {
            return Err(Error::NotFound("No audio processor".to_string()));
        }

        let result = unsafe {
            let vtbl = &*(*self.processor).vtbl;
            (vtbl.setProcessing)(self.processor as *mut _, state as u8)
        };

        if result != K_RESULT_OK {
            return Err(Error::ProcessingError("Failed to set processing state".to_string()));
        }

        self.processing = state;
        Ok(())
    }

    /// Process audio
    ///
    /// # Safety
    /// The process data must be properly initialized with valid buffer pointers.
    pub unsafe fn process(&mut self, data: &mut ProcessData) -> Result<()> {
        if !self.processing {
            return Err(Error::ProcessingError("Not in processing state".to_string()));
        }

        if self.processor.is_null() {
            return Err(Error::NotFound("No audio processor".to_string()));
        }

        let result = {
            let vtbl = &*(*self.processor).vtbl;
            (vtbl.process)(self.processor as *mut _, data as *mut _)
        };

        if result != K_RESULT_OK {
            return Err(Error::ProcessingError("Process failed".to_string()));
        }

        Ok(())
    }

    /// Get latency in samples
    pub fn get_latency(&self) -> u32 {
        if self.processor.is_null() {
            return 0;
        }

        unsafe {
            let vtbl = &*(*self.processor).vtbl;
            (vtbl.getLatencySamples)(self.processor as *mut _)
        }
    }

    /// Get tail samples
    pub fn get_tail_samples(&self) -> u32 {
        if self.processor.is_null() {
            return 0;
        }

        unsafe {
            let vtbl = &*(*self.processor).vtbl;
            (vtbl.getTailSamples)(self.processor as *mut _)
        }
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Check if active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Check if processing
    pub fn is_processing(&self) -> bool {
        self.processing
    }

    /// Check if has audio processor
    pub fn has_processor(&self) -> bool {
        !self.processor.is_null()
    }
}

impl Drop for Vst3PluginInstance {
    fn drop(&mut self) {
        // Ensure proper shutdown
        if self.initialized {
            self.terminate();
        }

        // Release component
        if !self.component.is_null() {
            unsafe {
                let unknown = self.component as *const FUnknown;
                (*unknown).release();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_info_from_raw() {
        let mut raw = PFactoryInfo::default();
        raw.vendor[..4].copy_from_slice(b"Test");
        raw.flags = 1;

        let info = FactoryInfo::from_raw(&raw);
        assert_eq!(info.vendor, "Test");
        assert_eq!(info.flags, 1);
    }

    #[test]
    fn test_class_info_from_raw() {
        let mut raw = PClassInfo::default();
        raw.name[..6].copy_from_slice(b"Plugin");
        raw.category[..5].copy_from_slice(b"Audio");
        raw.cardinality = 1;

        let info = ClassInfo::from_raw(&raw);
        assert_eq!(info.name, "Plugin");
        assert_eq!(info.category, "Audio");
        assert_eq!(info.cardinality, 1);
    }

    #[test]
    fn test_loader_nonexistent_path() {
        let result = Vst3PluginLoader::load(Path::new("/nonexistent/plugin.vst3"));
        assert!(result.is_err());
    }

    #[test]
    fn test_loader_invalid_file() {
        let result = Vst3PluginLoader::load(Path::new("/etc/passwd"));
        assert!(result.is_err());
    }
}
