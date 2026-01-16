//! # Orpheus UI
//!
//! UI components and views for the Orpheus desktop application.

pub mod theme;
pub mod layout;
pub mod toolbar;
pub mod views;
pub mod panels;
pub mod widgets;
pub mod dialogs;

pub use theme::{Theme, OrpheusTheme, OrpheusColorPalette, ColorPalette};
pub use layout::{DockLayout, DockTab, OrpheusTabViewer, ViewStates};
pub use views::{
    ArrangeView, ArrangeViewState, TrackDisplay, ClipDisplay, SnapMode, Selection,
    ComposeView, ComposeViewState, BeatInspector,
    MixView, MixViewState, ChannelStrip,
    PracticeView, PracticeViewState, PracticeSession, PracticeAction,
    RecordView, RecordViewState, AudioInput, RecordTrack, RecordingState,
    MasterView, MasterViewState, MasterProcessor, ProcessorType, ExportFormat, ExportSettings,
    CompressionMode, CompressionSettings,
    PianoRollView, PianoRollState, PianoRollNote, PianoRollAction, PianoRollTool, SnapResolution,
    TabEditorView, TabEditorState, TabEditorAction, TabCursor, TabSelection, EditorMode, ActiveTool, PendingTabAction,
    TabPlaybackEvent, TabPlaybackEventType,
};
// Orpheus-specific widgets
pub use widgets::{TablatureView, TabState, TabConfig, EditMode, NoteDuration, LevelMeter, Knob};

// Shared widgets from daemoniorum-egui
pub use widgets::{
    Badge, Card, CardStyle, ConfirmDialog, ConfirmDialogResponse, Dialog, EnhancedButton,
    Icon, LoadingIndicator, LoadingStyle, SelectableList, SelectableListResponse,
    StatusMessageWidget, TabBar, TabBarResponse, TabItem, TabStyle,
};

// Plugin browser panel
pub use panels::{
    PluginBrowserPanel, PluginBrowserState, PluginBrowserAction,
    ScannedPlugin, LoadedPlugin, PanelPluginFormat,
};

// Virtual keyboard panel
pub use panels::{
    VirtualKeyboardPanel, VirtualKeyboardState, VirtualKeyboardAction,
    KeyboardInstrument, KeyboardDrumType,
};

// Synth preset browser panel
pub use panels::{
    SynthPresetPanel, SynthPresetState, SynthPresetAction,
    SynthCategory, SynthPreset, PresetType,
    PianoPresetType, BassPresetType, GuitarPresetType, DrumPresetType,
};

// MIDI input panel
pub use panels::{
    MidiInputPanel, MidiInputState, MidiInputAction,
    MidiDevice, MidiActivityEntry, midi_message_color,
};

// Compression settings panel
pub use panels::{
    CompressionPanel, CompressionPanelState, CompressionPanelAction,
    CompressionStatsDisplay,
};

// Dialogs
pub use dialogs::{
    WelcomeDialog, WelcomeState, WelcomeAction,
    PreferencesDialog, PreferencesState, PreferencesAction, PreferencesTab, ThemeMode,
    AboutDialog, AboutState,
};
