//! Plugin format definitions
//!
//! Defines supported plugin formats (VST3, CLAP) and categories.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Supported plugin formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginFormat {
    /// VST3 format (Steinberg)
    Vst3,
    /// CLAP format (Clever Audio Plugin)
    Clap,
}

impl PluginFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Vst3 => "vst3",
            Self::Clap => "clap",
        }
    }

    /// Get display name for this format
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Vst3 => "VST3",
            Self::Clap => "CLAP",
        }
    }

    /// Detect format from file path
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "vst3" => Some(Self::Vst3),
            "clap" => Some(Self::Clap),
            _ => None,
        }
    }

    /// Check if path is a plugin bundle (directory with extension)
    pub fn is_bundle(path: &Path) -> bool {
        if !path.is_dir() {
            return false;
        }

        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e.to_lowercase().as_str(), "vst3" | "clap"))
            .unwrap_or(false)
    }

    /// Get all supported formats
    pub fn all() -> &'static [PluginFormat] {
        &[PluginFormat::Vst3, PluginFormat::Clap]
    }
}

impl std::fmt::Display for PluginFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Plugin categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PluginCategory {
    /// Audio effect (EQ, compressor, etc.)
    #[default]
    Effect,
    /// Virtual instrument
    Instrument,
    /// Analyzer (spectrum, meter)
    Analyzer,
    /// Spatial/surround processing
    Spatial,
    /// MIDI effect
    MidiEffect,
    /// Dynamics processor
    Dynamics,
    /// Equalizer
    Eq,
    /// Reverb/delay
    Reverb,
    /// Distortion/saturation
    Distortion,
    /// Modulation (chorus, flanger, phaser)
    Modulation,
    /// Filter
    Filter,
    /// Pitch correction/shifting
    Pitch,
    /// Mastering tool
    Mastering,
    /// Utility (gain, mono, etc.)
    Utility,
    /// Unknown/other
    Other,
}

impl PluginCategory {
    /// Get display name for this category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Effect => "Effect",
            Self::Instrument => "Instrument",
            Self::Analyzer => "Analyzer",
            Self::Spatial => "Spatial",
            Self::MidiEffect => "MIDI Effect",
            Self::Dynamics => "Dynamics",
            Self::Eq => "EQ",
            Self::Reverb => "Reverb/Delay",
            Self::Distortion => "Distortion",
            Self::Modulation => "Modulation",
            Self::Filter => "Filter",
            Self::Pitch => "Pitch",
            Self::Mastering => "Mastering",
            Self::Utility => "Utility",
            Self::Other => "Other",
        }
    }

    /// Parse category from VST3 category string
    pub fn from_vst3_category(category: &str) -> Self {
        let cat_lower = category.to_lowercase();
        if cat_lower.contains("instrument") || cat_lower.contains("synth") {
            Self::Instrument
        } else if cat_lower.contains("analyzer") {
            Self::Analyzer
        } else if cat_lower.contains("spatial") || cat_lower.contains("surround") {
            Self::Spatial
        } else if cat_lower.contains("dynamics") || cat_lower.contains("compressor") {
            Self::Dynamics
        } else if cat_lower.contains("eq") || cat_lower.contains("equalizer") {
            Self::Eq
        } else if cat_lower.contains("reverb") || cat_lower.contains("delay") {
            Self::Reverb
        } else if cat_lower.contains("distortion") || cat_lower.contains("saturation") {
            Self::Distortion
        } else if cat_lower.contains("modulation")
            || cat_lower.contains("chorus")
            || cat_lower.contains("flanger")
        {
            Self::Modulation
        } else if cat_lower.contains("filter") {
            Self::Filter
        } else if cat_lower.contains("pitch") {
            Self::Pitch
        } else if cat_lower.contains("mastering") {
            Self::Mastering
        } else if cat_lower.contains("utility") || cat_lower.contains("tools") {
            Self::Utility
        } else if cat_lower.contains("effect") || cat_lower.contains("fx") {
            Self::Effect
        } else {
            Self::Other
        }
    }

    /// Check if this is an instrument (generates audio)
    pub fn is_instrument(&self) -> bool {
        matches!(self, Self::Instrument)
    }

    /// Check if this is an effect (processes audio)
    pub fn is_effect(&self) -> bool {
        !matches!(self, Self::Instrument | Self::Analyzer | Self::MidiEffect)
    }
}

impl std::fmt::Display for PluginCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_format_extension() {
        assert_eq!(PluginFormat::Vst3.extension(), "vst3");
        assert_eq!(PluginFormat::Clap.extension(), "clap");
    }

    #[test]
    fn test_plugin_format_display_name() {
        assert_eq!(PluginFormat::Vst3.display_name(), "VST3");
        assert_eq!(PluginFormat::Clap.display_name(), "CLAP");
    }

    #[test]
    fn test_plugin_format_from_path() {
        assert_eq!(
            PluginFormat::from_path(Path::new("/path/to/plugin.vst3")),
            Some(PluginFormat::Vst3)
        );
        assert_eq!(
            PluginFormat::from_path(Path::new("/path/to/plugin.clap")),
            Some(PluginFormat::Clap)
        );
        assert_eq!(
            PluginFormat::from_path(Path::new("/path/to/plugin.VST3")),
            Some(PluginFormat::Vst3)
        );
        assert_eq!(
            PluginFormat::from_path(Path::new("/path/to/plugin.dll")),
            None
        );
        assert_eq!(PluginFormat::from_path(Path::new("/path/to/plugin")), None);
    }

    #[test]
    fn test_plugin_format_all() {
        let all = PluginFormat::all();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&PluginFormat::Vst3));
        assert!(all.contains(&PluginFormat::Clap));
    }

    #[test]
    fn test_plugin_format_display() {
        assert_eq!(format!("{}", PluginFormat::Vst3), "VST3");
        assert_eq!(format!("{}", PluginFormat::Clap), "CLAP");
    }

    #[test]
    fn test_plugin_category_display_name() {
        assert_eq!(PluginCategory::Effect.display_name(), "Effect");
        assert_eq!(PluginCategory::Instrument.display_name(), "Instrument");
        assert_eq!(PluginCategory::Dynamics.display_name(), "Dynamics");
    }

    #[test]
    fn test_plugin_category_from_vst3() {
        assert_eq!(
            PluginCategory::from_vst3_category("Fx|Dynamics"),
            PluginCategory::Dynamics
        );
        assert_eq!(
            PluginCategory::from_vst3_category("Instrument|Synth"),
            PluginCategory::Instrument
        );
        assert_eq!(
            PluginCategory::from_vst3_category("Fx|EQ"),
            PluginCategory::Eq
        );
        assert_eq!(
            PluginCategory::from_vst3_category("Fx|Reverb"),
            PluginCategory::Reverb
        );
        assert_eq!(
            PluginCategory::from_vst3_category("Unknown"),
            PluginCategory::Other
        );
    }

    #[test]
    fn test_plugin_category_is_instrument() {
        assert!(PluginCategory::Instrument.is_instrument());
        assert!(!PluginCategory::Effect.is_instrument());
        assert!(!PluginCategory::Dynamics.is_instrument());
    }

    #[test]
    fn test_plugin_category_is_effect() {
        assert!(PluginCategory::Effect.is_effect());
        assert!(PluginCategory::Dynamics.is_effect());
        assert!(PluginCategory::Eq.is_effect());
        assert!(!PluginCategory::Instrument.is_effect());
        assert!(!PluginCategory::Analyzer.is_effect());
    }

    #[test]
    fn test_plugin_category_default() {
        assert_eq!(PluginCategory::default(), PluginCategory::Effect);
    }
}
