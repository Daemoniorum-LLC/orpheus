//! # Orpheus Plugins
//!
//! Plugin hosting for Orpheus - VST3/CLAP support.
//!
//! This crate provides:
//! - Plugin discovery and scanning from standard locations
//! - Plugin metadata extraction
//! - Plugin host for loading and running plugins
//! - Audio processing integration
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use orpheus_plugins::{PluginScanner, PluginHost, HostConfig};
//!
//! // Scan for available plugins
//! let scanner = PluginScanner::new();
//! let plugins = scanner.scan_all()?;
//!
//! // Load a plugin
//! let host = PluginHost::new(HostConfig::default());
//! let instance = host.load(&plugins[0].path)?;
//! ```

pub mod clap;
pub mod format;
pub mod host;
pub mod scanner;
pub mod vst3;

// Re-exports
pub use format::{PluginFormat, PluginCategory};
pub use host::{HostConfig, PluginHost, PluginInstance, PluginState};
pub use scanner::{PluginMetadata, PluginScanner, ScanResult};

/// Result type for plugin operations
pub type Result<T> = std::result::Result<T, Error>;

/// Plugin error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Plugin not found at specified path
    #[error("Plugin not found: {0}")]
    NotFound(String),

    /// Failed to load plugin
    #[error("Plugin load error: {0}")]
    LoadError(String),

    /// Plugin format not supported
    #[error("Plugin format not supported: {0}")]
    UnsupportedFormat(String),

    /// Plugin initialization failed
    #[error("Plugin initialization failed: {0}")]
    InitializationError(String),

    /// Audio processing error
    #[error("Audio processing error: {0}")]
    ProcessingError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Parameter error
    #[error("Parameter error: {0}")]
    ParameterError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::NotFound("test.vst3".to_string());
        assert!(err.to_string().contains("test.vst3"));

        let err = Error::LoadError("failed to load".to_string());
        assert!(err.to_string().contains("failed to load"));

        let err = Error::UnsupportedFormat("lv2".to_string());
        assert!(err.to_string().contains("lv2"));
    }
}
