//! Test assertions for verifying state

use orpheus_core::tab::Technique;
use std::path::PathBuf;

/// Assertions for verifying test state
#[derive(Debug, Clone)]
pub enum Assertion {
    // === Project Assertions ===
    /// Project has expected name
    ProjectName(String),
    /// Project has expected number of tracks
    TrackCount(usize),
    /// Project is dirty (has unsaved changes)
    IsDirty,
    /// Project is clean (no unsaved changes)
    IsClean,

    // === Track Assertions ===
    /// Track exists with name
    TrackExists(String),
    /// Track at index has name
    TrackNameAt(usize, String),
    /// Track is muted
    TrackMuted(usize),
    /// Track is soloed
    TrackSoloed(usize),
    /// Track is armed for recording
    TrackArmed(usize),

    // === Tab Editor Assertions ===
    /// Tab editor is open
    TabEditorOpen,
    /// Tab editor is in specific mode
    TabEditorMode(TabEditorModeAssertion),
    /// Cursor is at position
    CursorAt { measure: usize, beat: usize, string: u8 },
    /// Note exists at position
    NoteExists { measure: usize, beat: usize, string: u8, fret: u8 },
    /// No note at position
    NoNoteAt { measure: usize, beat: usize, string: u8 },
    /// Note has technique
    NoteHasTechnique { measure: usize, beat: usize, string: u8, technique: Technique },
    /// Measure count
    MeasureCount(usize),
    /// Total note count
    TotalNoteCount(usize),
    /// Selection exists
    HasSelection,
    /// No selection
    NoSelection,

    // === Transport Assertions ===
    /// Playback is active
    IsPlaying,
    /// Playback is stopped
    IsStopped,
    /// Playhead is at position (in beats)
    PlayheadAt(f64),
    /// Tempo is set to value
    TempoIs(f64),

    // === Panel Assertions ===
    /// Panel is visible
    PanelVisible(String),
    /// Panel is hidden
    PanelHidden(String),

    // === File Assertions ===
    /// File exists at path
    FileExists(PathBuf),
    /// File does not exist
    FileNotExists(PathBuf),

    // === Error Assertions ===
    /// No errors occurred
    NoErrors,
    /// Error message contains text
    ErrorContains(String),
}

/// Tab editor mode assertion variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabEditorModeAssertion {
    Normal,
    Insert,
    Visual,
    Command,
}

impl Assertion {
    /// Get a description of the assertion
    pub fn description(&self) -> String {
        match self {
            Assertion::ProjectName(name) => format!("Project name is '{}'", name),
            Assertion::TrackCount(n) => format!("Track count is {}", n),
            Assertion::IsDirty => "Project has unsaved changes".to_string(),
            Assertion::IsClean => "Project has no unsaved changes".to_string(),
            Assertion::TrackExists(name) => format!("Track '{}' exists", name),
            Assertion::TrackNameAt(idx, name) => format!("Track {} is named '{}'", idx, name),
            Assertion::TrackMuted(idx) => format!("Track {} is muted", idx),
            Assertion::TrackSoloed(idx) => format!("Track {} is soloed", idx),
            Assertion::TrackArmed(idx) => format!("Track {} is armed", idx),
            Assertion::TabEditorOpen => "Tab editor is open".to_string(),
            Assertion::TabEditorMode(mode) => format!("Tab editor mode is {:?}", mode),
            Assertion::CursorAt { measure, beat, string } => {
                format!("Cursor at M{} B{} S{}", measure + 1, beat + 1, string)
            }
            Assertion::NoteExists { measure, beat, string, fret } => {
                format!("Note at M{} B{} S{} = fret {}", measure + 1, beat + 1, string, fret)
            }
            Assertion::NoNoteAt { measure, beat, string } => {
                format!("No note at M{} B{} S{}", measure + 1, beat + 1, string)
            }
            Assertion::NoteHasTechnique { measure, beat, string, technique } => {
                format!("Note at M{} B{} S{} has {:?}", measure + 1, beat + 1, string, technique)
            }
            Assertion::MeasureCount(n) => format!("Measure count is {}", n),
            Assertion::TotalNoteCount(n) => format!("Total note count is {}", n),
            Assertion::HasSelection => "Selection exists".to_string(),
            Assertion::NoSelection => "No selection".to_string(),
            Assertion::IsPlaying => "Playback is active".to_string(),
            Assertion::IsStopped => "Playback is stopped".to_string(),
            Assertion::PlayheadAt(pos) => format!("Playhead at beat {:.2}", pos),
            Assertion::TempoIs(bpm) => format!("Tempo is {:.1} BPM", bpm),
            Assertion::PanelVisible(name) => format!("Panel '{}' is visible", name),
            Assertion::PanelHidden(name) => format!("Panel '{}' is hidden", name),
            Assertion::FileExists(path) => format!("File exists: {}", path.display()),
            Assertion::FileNotExists(path) => format!("File does not exist: {}", path.display()),
            Assertion::NoErrors => "No errors occurred".to_string(),
            Assertion::ErrorContains(text) => format!("Error contains '{}'", text),
        }
    }
}
