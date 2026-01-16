//! Plugin scanner for discovering installed plugins
//!
//! Scans standard plugin directories and extracts metadata from plugins.

use crate::{Error, PluginCategory, PluginFormat, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Metadata for a discovered plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for this plugin instance
    pub id: Uuid,
    /// Plugin name
    pub name: String,
    /// Vendor/manufacturer name
    pub vendor: String,
    /// Plugin version string
    pub version: String,
    /// Plugin format (VST3, CLAP)
    pub format: PluginFormat,
    /// Plugin category
    pub category: PluginCategory,
    /// Path to plugin file/bundle
    pub path: PathBuf,
    /// Number of audio inputs
    pub num_inputs: u32,
    /// Number of audio outputs
    pub num_outputs: u32,
    /// Number of MIDI inputs
    pub num_midi_inputs: u32,
    /// Number of MIDI outputs
    pub num_midi_outputs: u32,
    /// Whether plugin has custom editor GUI
    pub has_editor: bool,
    /// Plugin description (if available)
    pub description: Option<String>,
    /// Plugin URL (if available)
    pub url: Option<String>,
    /// VST3/CLAP specific identifier
    pub plugin_id: Option<String>,
}

impl PluginMetadata {
    /// Create new metadata with minimal info
    pub fn new(name: impl Into<String>, path: PathBuf, format: PluginFormat) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            vendor: String::new(),
            version: String::from("1.0.0"),
            format,
            category: PluginCategory::default(),
            path,
            num_inputs: 2,
            num_outputs: 2,
            num_midi_inputs: 0,
            num_midi_outputs: 0,
            has_editor: false,
            description: None,
            url: None,
            plugin_id: None,
        }
    }

    /// Check if this is an instrument plugin
    pub fn is_instrument(&self) -> bool {
        self.category.is_instrument()
    }

    /// Check if this is an effect plugin
    pub fn is_effect(&self) -> bool {
        self.category.is_effect()
    }

    /// Check if plugin supports MIDI input
    pub fn has_midi_input(&self) -> bool {
        self.num_midi_inputs > 0 || self.is_instrument()
    }
}

/// Result of a plugin scan operation
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    /// Successfully scanned plugins
    pub plugins: Vec<PluginMetadata>,
    /// Paths that failed to scan with error messages
    pub errors: Vec<(PathBuf, String)>,
    /// Directories that were scanned
    pub scanned_dirs: Vec<PathBuf>,
}

impl ScanResult {
    /// Create new empty result
    pub fn new() -> Self {
        Self::default()
    }

    /// Total number of plugins found
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Number of scan errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get plugins of a specific format
    pub fn by_format(&self, format: PluginFormat) -> Vec<&PluginMetadata> {
        self.plugins.iter().filter(|p| p.format == format).collect()
    }

    /// Get plugins of a specific category
    pub fn by_category(&self, category: PluginCategory) -> Vec<&PluginMetadata> {
        self.plugins
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }

    /// Get instrument plugins
    pub fn instruments(&self) -> Vec<&PluginMetadata> {
        self.plugins.iter().filter(|p| p.is_instrument()).collect()
    }

    /// Get effect plugins
    pub fn effects(&self) -> Vec<&PluginMetadata> {
        self.plugins.iter().filter(|p| p.is_effect()).collect()
    }

    /// Merge another scan result into this one
    pub fn merge(&mut self, other: ScanResult) {
        self.plugins.extend(other.plugins);
        self.errors.extend(other.errors);
        self.scanned_dirs.extend(other.scanned_dirs);
    }
}

/// Plugin scanner for discovering installed plugins
#[derive(Debug, Clone)]
pub struct PluginScanner {
    /// Additional search paths (beyond defaults)
    additional_paths: Vec<PathBuf>,
    /// Formats to scan for
    formats: Vec<PluginFormat>,
}

impl Default for PluginScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginScanner {
    /// Create a new scanner with default settings
    pub fn new() -> Self {
        Self {
            additional_paths: Vec::new(),
            formats: PluginFormat::all().to_vec(),
        }
    }

    /// Add an additional search path
    pub fn add_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.additional_paths.push(path.into());
        self
    }

    /// Set which formats to scan for
    pub fn set_formats(&mut self, formats: Vec<PluginFormat>) -> &mut Self {
        self.formats = formats;
        self
    }

    /// Get default plugin paths for the current platform
    pub fn default_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "linux")]
        {
            // VST3 paths
            if let Some(home) = dirs::home_dir() {
                paths.push(home.join(".vst3"));
            }
            paths.push(PathBuf::from("/usr/lib/vst3"));
            paths.push(PathBuf::from("/usr/local/lib/vst3"));

            // CLAP paths
            if let Some(home) = dirs::home_dir() {
                paths.push(home.join(".clap"));
            }
            paths.push(PathBuf::from("/usr/lib/clap"));
            paths.push(PathBuf::from("/usr/local/lib/clap"));
        }

        #[cfg(target_os = "macos")]
        {
            // VST3 paths
            if let Some(home) = dirs::home_dir() {
                paths.push(home.join("Library/Audio/Plug-Ins/VST3"));
            }
            paths.push(PathBuf::from("/Library/Audio/Plug-Ins/VST3"));

            // CLAP paths
            if let Some(home) = dirs::home_dir() {
                paths.push(home.join("Library/Audio/Plug-Ins/CLAP"));
            }
            paths.push(PathBuf::from("/Library/Audio/Plug-Ins/CLAP"));
        }

        #[cfg(target_os = "windows")]
        {
            // VST3 paths
            if let Some(common) = dirs::data_dir() {
                paths.push(common.join("VST3"));
            }
            paths.push(PathBuf::from("C:\\Program Files\\Common Files\\VST3"));

            // CLAP paths
            if let Some(common) = dirs::data_dir() {
                paths.push(common.join("CLAP"));
            }
            paths.push(PathBuf::from("C:\\Program Files\\Common Files\\CLAP"));
        }

        // Add any additional paths
        paths.extend(self.additional_paths.iter().cloned());

        paths
    }

    /// Scan all default paths for plugins
    pub fn scan_all(&self) -> Result<ScanResult> {
        let paths = self.default_paths();
        let mut result = ScanResult::new();

        for path in &paths {
            if path.exists() {
                debug!("Scanning plugin directory: {}", path.display());
                match self.scan_directory(path) {
                    Ok(dir_result) => {
                        result.merge(dir_result);
                    }
                    Err(e) => {
                        warn!("Failed to scan {}: {}", path.display(), e);
                        result.errors.push((path.clone(), e.to_string()));
                    }
                }
            }
        }

        info!(
            "Plugin scan complete: {} plugins found, {} errors",
            result.count(),
            result.error_count()
        );

        Ok(result)
    }

    /// Scan a specific directory for plugins
    pub fn scan_directory(&self, dir: &Path) -> Result<ScanResult> {
        let mut result = ScanResult::new();
        result.scanned_dirs.push(dir.to_path_buf());

        if !dir.exists() {
            return Ok(result);
        }

        let entries = std::fs::read_dir(dir)?;

        for entry in entries.flatten() {
            let path = entry.path();

            // Check if this is a plugin bundle
            if let Some(format) = PluginFormat::from_path(&path) {
                if self.formats.contains(&format) {
                    match self.scan_plugin(&path) {
                        Ok(metadata) => {
                            debug!("Found plugin: {} ({})", metadata.name, format);
                            result.plugins.push(metadata);
                        }
                        Err(e) => {
                            warn!("Failed to scan plugin {}: {}", path.display(), e);
                            result.errors.push((path, e.to_string()));
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Scan a specific plugin file/bundle
    pub fn scan_plugin(&self, path: &Path) -> Result<PluginMetadata> {
        if !path.exists() {
            return Err(Error::NotFound(path.display().to_string()));
        }

        let format = PluginFormat::from_path(path)
            .ok_or_else(|| Error::UnsupportedFormat(path.display().to_string()))?;

        // Extract name from filename
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Plugin")
            .to_string();

        let mut metadata = PluginMetadata::new(name, path.to_path_buf(), format);

        // Try to extract more metadata based on format
        match format {
            PluginFormat::Vst3 => self.extract_vst3_metadata(path, &mut metadata)?,
            PluginFormat::Clap => self.extract_clap_metadata(path, &mut metadata)?,
        }

        Ok(metadata)
    }

    /// Extract VST3-specific metadata
    fn extract_vst3_metadata(&self, path: &Path, metadata: &mut PluginMetadata) -> Result<()> {
        // VST3 bundles have a specific structure:
        // Plugin.vst3/
        // ├── Contents/
        // │   ├── x86_64-linux/Plugin.so (Linux)
        // │   ├── MacOS/Plugin (macOS)
        // │   └── x86_64-win/Plugin.vst3 (Windows)
        // ├── moduleinfo.json (optional)
        // └── plugin.vst3 (Windows single-file)

        // Check for moduleinfo.json
        let moduleinfo_path = path.join("Contents").join("moduleinfo.json");
        if moduleinfo_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&moduleinfo_path) {
                // Parse moduleinfo.json for metadata
                // This is a simplified parse - real implementation would use serde_json
                if content.contains("\"name\"") {
                    debug!("Found moduleinfo.json for VST3: {}", path.display());
                }
            }
        }

        // Check for Resources directory with info
        let resources_path = path.join("Contents").join("Resources");
        if resources_path.exists() {
            metadata.has_editor = true;
        }

        Ok(())
    }

    /// Extract CLAP-specific metadata
    fn extract_clap_metadata(&self, path: &Path, _metadata: &mut PluginMetadata) -> Result<()> {
        // CLAP plugins are typically single shared library files
        // Metadata is extracted by loading the plugin and calling clap_entry

        // For now, we just mark common assumptions
        if path.is_file() {
            // Single file CLAP plugin
            debug!("Found CLAP plugin file: {}", path.display());
        } else if path.is_dir() {
            // CLAP bundle
            debug!("Found CLAP plugin bundle: {}", path.display());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_plugin_metadata_new() {
        let path = PathBuf::from("/path/to/plugin.vst3");
        let metadata = PluginMetadata::new("Test Plugin", path.clone(), PluginFormat::Vst3);

        assert_eq!(metadata.name, "Test Plugin");
        assert_eq!(metadata.path, path);
        assert_eq!(metadata.format, PluginFormat::Vst3);
        assert_eq!(metadata.num_inputs, 2);
        assert_eq!(metadata.num_outputs, 2);
    }

    #[test]
    fn test_plugin_metadata_is_instrument() {
        let mut metadata =
            PluginMetadata::new("Synth", PathBuf::from("synth.vst3"), PluginFormat::Vst3);
        metadata.category = PluginCategory::Instrument;

        assert!(metadata.is_instrument());
        assert!(!metadata.is_effect());

        metadata.category = PluginCategory::Effect;
        assert!(!metadata.is_instrument());
        assert!(metadata.is_effect());
    }

    #[test]
    fn test_plugin_metadata_has_midi_input() {
        let mut metadata =
            PluginMetadata::new("Effect", PathBuf::from("effect.vst3"), PluginFormat::Vst3);

        assert!(!metadata.has_midi_input());

        metadata.num_midi_inputs = 1;
        assert!(metadata.has_midi_input());

        metadata.num_midi_inputs = 0;
        metadata.category = PluginCategory::Instrument;
        assert!(metadata.has_midi_input());
    }

    #[test]
    fn test_scan_result_new() {
        let result = ScanResult::new();
        assert_eq!(result.count(), 0);
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_scan_result_by_format() {
        let mut result = ScanResult::new();
        result.plugins.push(PluginMetadata::new(
            "VST Plugin",
            PathBuf::from("plugin.vst3"),
            PluginFormat::Vst3,
        ));
        result.plugins.push(PluginMetadata::new(
            "CLAP Plugin",
            PathBuf::from("plugin.clap"),
            PluginFormat::Clap,
        ));

        let vst3_plugins = result.by_format(PluginFormat::Vst3);
        assert_eq!(vst3_plugins.len(), 1);
        assert_eq!(vst3_plugins[0].name, "VST Plugin");

        let clap_plugins = result.by_format(PluginFormat::Clap);
        assert_eq!(clap_plugins.len(), 1);
        assert_eq!(clap_plugins[0].name, "CLAP Plugin");
    }

    #[test]
    fn test_scan_result_by_category() {
        let mut result = ScanResult::new();

        let mut effect = PluginMetadata::new(
            "EQ Plugin",
            PathBuf::from("eq.vst3"),
            PluginFormat::Vst3,
        );
        effect.category = PluginCategory::Eq;

        let mut instrument = PluginMetadata::new(
            "Synth Plugin",
            PathBuf::from("synth.vst3"),
            PluginFormat::Vst3,
        );
        instrument.category = PluginCategory::Instrument;

        result.plugins.push(effect);
        result.plugins.push(instrument);

        let eq_plugins = result.by_category(PluginCategory::Eq);
        assert_eq!(eq_plugins.len(), 1);

        let instruments = result.instruments();
        assert_eq!(instruments.len(), 1);
        assert_eq!(instruments[0].name, "Synth Plugin");

        let effects = result.effects();
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].name, "EQ Plugin");
    }

    #[test]
    fn test_scan_result_merge() {
        let mut result1 = ScanResult::new();
        result1.plugins.push(PluginMetadata::new(
            "Plugin 1",
            PathBuf::from("p1.vst3"),
            PluginFormat::Vst3,
        ));

        let mut result2 = ScanResult::new();
        result2.plugins.push(PluginMetadata::new(
            "Plugin 2",
            PathBuf::from("p2.vst3"),
            PluginFormat::Vst3,
        ));
        result2
            .errors
            .push((PathBuf::from("bad.vst3"), "Failed".to_string()));

        result1.merge(result2);

        assert_eq!(result1.count(), 2);
        assert_eq!(result1.error_count(), 1);
    }

    #[test]
    fn test_plugin_scanner_new() {
        let scanner = PluginScanner::new();
        assert_eq!(scanner.formats.len(), 2);
    }

    #[test]
    fn test_plugin_scanner_default_paths() {
        let scanner = PluginScanner::new();
        let paths = scanner.default_paths();

        // Should have at least some default paths
        assert!(!paths.is_empty());

        // Check that paths include expected patterns
        #[cfg(target_os = "linux")]
        {
            assert!(paths.iter().any(|p| p.to_string_lossy().contains(".vst3")));
        }
    }

    #[test]
    fn test_plugin_scanner_add_path() {
        let mut scanner = PluginScanner::new();
        let custom_path = PathBuf::from("/custom/plugin/path");
        scanner.add_path(&custom_path);

        let paths = scanner.default_paths();
        assert!(paths.contains(&custom_path));
    }

    #[test]
    fn test_plugin_scanner_set_formats() {
        let mut scanner = PluginScanner::new();
        scanner.set_formats(vec![PluginFormat::Vst3]);

        assert_eq!(scanner.formats.len(), 1);
        assert!(scanner.formats.contains(&PluginFormat::Vst3));
        assert!(!scanner.formats.contains(&PluginFormat::Clap));
    }

    #[test]
    fn test_plugin_scanner_scan_empty_directory() {
        let temp = tempdir().unwrap();
        let scanner = PluginScanner::new();

        let result = scanner.scan_directory(temp.path()).unwrap();
        assert_eq!(result.count(), 0);
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_plugin_scanner_scan_nonexistent_directory() {
        let scanner = PluginScanner::new();
        let result = scanner.scan_directory(Path::new("/nonexistent/path")).unwrap();

        assert_eq!(result.count(), 0);
    }

    #[test]
    fn test_plugin_scanner_scan_directory_with_mock_plugins() {
        let temp = tempdir().unwrap();

        // Create mock plugin directories
        let vst3_dir = temp.path().join("TestPlugin.vst3");
        std::fs::create_dir(&vst3_dir).unwrap();

        let clap_dir = temp.path().join("TestPlugin.clap");
        std::fs::create_dir(&clap_dir).unwrap();

        let scanner = PluginScanner::new();
        let result = scanner.scan_directory(temp.path()).unwrap();

        assert_eq!(result.count(), 2);
        assert!(result.by_format(PluginFormat::Vst3).len() >= 1);
        assert!(result.by_format(PluginFormat::Clap).len() >= 1);
    }

    #[test]
    fn test_plugin_scanner_scan_plugin_not_found() {
        let scanner = PluginScanner::new();
        let result = scanner.scan_plugin(Path::new("/nonexistent/plugin.vst3"));

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotFound(_)));
    }

    #[test]
    fn test_plugin_scanner_scan_plugin_unsupported_format() {
        let temp = tempdir().unwrap();
        let dll_path = temp.path().join("plugin.dll");
        std::fs::write(&dll_path, b"fake dll").unwrap();

        let scanner = PluginScanner::new();
        let result = scanner.scan_plugin(&dll_path);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::UnsupportedFormat(_)));
    }

    #[test]
    fn test_plugin_scanner_scan_mock_vst3() {
        let temp = tempdir().unwrap();
        let vst3_path = temp.path().join("MyPlugin.vst3");
        std::fs::create_dir(&vst3_path).unwrap();

        let scanner = PluginScanner::new();
        let result = scanner.scan_plugin(&vst3_path).unwrap();

        assert_eq!(result.name, "MyPlugin");
        assert_eq!(result.format, PluginFormat::Vst3);
        assert_eq!(result.path, vst3_path);
    }
}
