//! Configuration and settings

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Audio settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Sample rate
    pub sample_rate: u32,
    /// Buffer size in samples
    pub buffer_size: u32,
    /// Input device name
    pub input_device: Option<String>,
    /// Output device name
    pub output_device: Option<String>,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 512,
            input_device: None,
            output_device: None,
        }
    }
}

/// UI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    /// Theme (dark, light, high_contrast)
    pub theme: String,
    /// UI scale factor
    pub scale: f32,
    /// Show tooltips
    pub show_tooltips: bool,
    /// Meter refresh rate (Hz)
    pub meter_refresh_rate: u32,
    /// Show welcome dialog on startup
    #[serde(default = "default_show_welcome")]
    pub show_welcome_on_startup: bool,
    /// Use high contrast mode for accessibility
    #[serde(default)]
    pub high_contrast: bool,
}

fn default_show_welcome() -> bool {
    true
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            scale: 1.0,
            show_tooltips: true,
            meter_refresh_rate: 30,
            show_welcome_on_startup: true,
            high_contrast: false,
        }
    }
}

/// Project settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Default project location
    pub default_location: PathBuf,
    /// Recent projects
    pub recent_projects: Vec<PathBuf>,
    /// Maximum recent projects
    pub max_recent: usize,
    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u32,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            default_location: dirs::document_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("Orpheus Projects"),
            recent_projects: Vec::new(),
            max_recent: 10,
            auto_save_interval: 60,
        }
    }
}

/// MIDI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiSettings {
    /// Input device name
    pub input_device: Option<String>,
    /// Output device name
    pub output_device: Option<String>,
    /// Enable MIDI clock sync
    pub sync_enabled: bool,
}

impl Default for MidiSettings {
    fn default() -> Self {
        Self {
            input_device: None,
            output_device: None,
            sync_enabled: false,
        }
    }
}

/// Complete application settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Audio settings
    pub audio: AudioSettings,
    /// UI settings
    pub ui: UiSettings,
    /// Project settings
    pub project: ProjectSettings,
    /// MIDI settings
    pub midi: MidiSettings,
}

impl Settings {
    /// Load settings from default location
    pub fn load() -> Self {
        let path = Self::settings_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(settings) => return settings,
                    Err(e) => {
                        tracing::warn!("Failed to parse settings: {}", e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read settings: {}", e);
                }
            }
        }
        Self::default()
    }

    /// Save settings to default location
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::settings_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, content)
    }

    /// Get settings file path
    fn settings_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("orpheus")
            .join("settings.toml")
    }

    /// Add a recent project
    pub fn add_recent_project(&mut self, path: PathBuf) {
        // Remove if already exists
        self.project.recent_projects.retain(|p| p != &path);
        // Add to front
        self.project.recent_projects.insert(0, path);
        // Trim to max
        self.project.recent_projects.truncate(self.project.max_recent);
    }
}

/// Helper for dirs crate
mod dirs {
    use std::path::PathBuf;

    pub fn document_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Documents"))
    }

    pub fn config_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".config"))
    }
}
