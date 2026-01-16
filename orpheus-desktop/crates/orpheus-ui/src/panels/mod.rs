//! Dockable panels

pub mod plugin_browser;
pub mod virtual_keyboard;
pub mod synth_presets;
pub mod midi_input;
pub mod compression;
pub mod analysis;

pub use plugin_browser::{
    PluginBrowserPanel, PluginBrowserState, PluginBrowserAction,
    ScannedPlugin, LoadedPlugin, PluginFormat as PanelPluginFormat,
    PluginParameter,
};

pub use virtual_keyboard::{
    VirtualKeyboardPanel, VirtualKeyboardState, VirtualKeyboardAction,
    KeyboardInstrument, DrumType as KeyboardDrumType,
};

pub use synth_presets::{
    SynthPresetPanel, SynthPresetState, SynthPresetAction,
    SynthCategory, SynthPreset, PresetType,
    PianoPresetType, BassPresetType, GuitarPresetType, DrumPresetType,
};

pub use midi_input::{
    MidiInputPanel, MidiInputState, MidiInputAction,
    MidiDevice, MidiActivityEntry, midi_message_color,
};

pub use compression::{
    CompressionPanel, CompressionPanelState, CompressionPanelAction,
    CompressionStatsDisplay,
};

pub use analysis::{
    AnalysisPanel, AnalysisPanelState, AnalysisPanelAction,
    AnalysisTab, PersistedAnalysisState,
};
