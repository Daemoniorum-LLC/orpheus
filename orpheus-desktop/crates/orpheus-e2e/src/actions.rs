//! Test actions that can be performed in E2E scenarios

use orpheus_core::tab::{Technique, BaseDuration, BendAmount, TapType, SlideDirection};
use std::time::Duration;

/// Actions that can be performed during a test
#[derive(Debug, Clone)]
pub enum TestAction {
    // === Project Operations ===
    /// Create a new project
    NewProject(String),
    /// Open an existing project file
    OpenProject(String),
    /// Save the current project
    SaveProject,
    /// Close the current project
    CloseProject,

    // === Track Operations ===
    Track(TrackAction),

    // === Tab Editor Operations ===
    TabEditor(TabEditorAction),

    // === Transport Operations ===
    /// Start playback
    Play,
    /// Stop playback
    Stop,
    /// Toggle play/pause
    TogglePlayback,
    /// Rewind to start
    Rewind,
    /// Set tempo
    SetTempo(f64),

    // === View Operations ===
    /// Open the tab editor panel
    OpenTabEditor,
    /// Open the piano roll panel
    OpenPianoRoll,
    /// Open the mixer panel
    OpenMixer,
    /// Close a specific panel
    ClosePanel(String),

    // === Navigation ===
    /// Press keyboard shortcut
    PressKey(egui::Key, egui::Modifiers),
    /// Click at position
    Click(f32, f32),
    /// Drag from position to position
    Drag(f32, f32, f32, f32),

    // === Timing ===
    /// Wait for specified duration
    Wait(Duration),

    // === Undo/Redo ===
    Undo,
    Redo,
}

/// Track-related actions
#[derive(Debug, Clone)]
pub enum TrackAction {
    /// Add a new audio track
    AddAudioTrack(String),
    /// Add a new MIDI track
    AddMidiTrack(String),
    /// Add a new guitar tab track
    AddTabTrack(String),
    /// Add a drum track
    AddDrumTrack(String),
    /// Delete a track by index
    DeleteTrack(usize),
    /// Select a track by index
    SelectTrack(usize),
    /// Mute a track
    MuteTrack(usize),
    /// Solo a track
    SoloTrack(usize),
    /// Arm track for recording
    ArmTrack(usize),
    /// Rename a track
    RenameTrack(usize, String),
}

/// Tab editor specific actions
#[derive(Debug, Clone)]
pub enum TabEditorAction {
    // === Mode ===
    /// Switch to Insert mode
    EnterInsertMode,
    /// Switch to Normal mode
    EnterNormalMode,
    /// Switch to Visual mode
    EnterVisualMode,

    // === Cursor Movement ===
    /// Move cursor
    MoveCursor(CursorMove),
    /// Jump to specific position
    JumpTo { measure: usize, beat: usize, string: u8 },

    // === Note Entry ===
    /// Enter a note (string 1-8, fret 0-24+)
    EnterNote { string: u8, fret: u8 },
    /// Enter a rest
    EnterRest,
    /// Delete note at cursor
    DeleteNote,
    /// Clear all notes in selection
    ClearSelection,

    // === Techniques ===
    /// Apply technique to current note
    ApplyTechnique(Technique),
    /// Set active tool
    SetTool(ToolType),
    /// Clear active tool
    ClearTool,

    // === Duration ===
    /// Set note duration
    SetDuration(BaseDuration),
    /// Toggle dotted
    ToggleDotted,
    /// Toggle triplet
    ToggleTriplet,

    // === Measure Operations ===
    /// Add a new measure
    AddMeasure,
    /// Delete current measure
    DeleteMeasure,
    /// Insert measure before current
    InsertMeasure,
    /// Set time signature
    SetTimeSignature(u8, u8),

    // === Selection ===
    /// Select range
    SelectRange {
        start_measure: usize,
        start_beat: usize,
        end_measure: usize,
        end_beat: usize,
    },
    /// Select all
    SelectAll,
    /// Deselect all
    DeselectAll,

    // === Clipboard ===
    /// Copy selection
    Copy,
    /// Cut selection
    Cut,
    /// Paste at cursor
    Paste,
}

/// Tool types for the tab editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    None,
    HammerOn,
    PullOff,
    Slide(SlideDirection),
    Bend(BendAmount),
    Tap(TapType),
    NaturalHarmonic,
    PinchHarmonic,
    PalmMute,
    Vibrato,
    LetRing,
}

/// Cursor movement directions
#[derive(Debug, Clone, Copy)]
pub enum CursorMove {
    Up,
    Down,
    Left,
    Right,
    /// Move to next measure
    NextMeasure,
    /// Move to previous measure
    PrevMeasure,
    /// Move to start of measure
    MeasureStart,
    /// Move to end of measure
    MeasureEnd,
    /// Move to start of track
    TrackStart,
    /// Move to end of track
    TrackEnd,
}

impl From<TrackAction> for TestAction {
    fn from(action: TrackAction) -> Self {
        TestAction::Track(action)
    }
}

impl From<TabEditorAction> for TestAction {
    fn from(action: TabEditorAction) -> Self {
        TestAction::TabEditor(action)
    }
}
