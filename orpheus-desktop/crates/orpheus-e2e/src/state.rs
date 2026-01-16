//! Test state management
//!
//! Captures and manages application state for testing purposes.

use orpheus_core::tab::{TabDocument, TabTrack, TabMeasure, TabNote, Technique};
use orpheus_ui::{TabEditorState, EditorMode, TabCursor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete test state snapshot
#[derive(Debug, Clone, Default)]
pub struct TestState {
    /// Project state
    pub project: ProjectTestState,
    /// Tab editor state (if open)
    pub tab_editor: Option<TabEditorTestState>,
    /// Transport state
    pub transport: TransportTestState,
    /// Panel visibility
    pub panels: HashMap<String, bool>,
    /// Recent errors
    pub errors: Vec<String>,
}

/// Project state for testing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectTestState {
    /// Project name
    pub name: String,
    /// Number of tracks
    pub track_count: usize,
    /// Track names
    pub track_names: Vec<String>,
    /// Whether project has unsaved changes
    pub is_dirty: bool,
    /// Project file path (if saved)
    pub file_path: Option<String>,
}

/// Tab editor state for testing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TabEditorTestState {
    /// Current mode
    pub mode: EditorModeSnapshot,
    /// Cursor position
    pub cursor: CursorSnapshot,
    /// Document state
    pub document: DocumentSnapshot,
    /// Whether there's an active selection
    pub has_selection: bool,
    /// Active tool name
    pub active_tool: Option<String>,
    /// Is modified
    pub is_modified: bool,
}

/// Editor mode snapshot
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorModeSnapshot {
    #[default]
    Normal,
    Insert,
    Visual,
    Command,
}

/// Cursor position snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CursorSnapshot {
    pub track: usize,
    pub measure: usize,
    pub beat: usize,
    pub string: u8,
}

/// Document state snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentSnapshot {
    /// Document title
    pub title: String,
    /// Number of tracks
    pub track_count: usize,
    /// Number of measures
    pub measure_count: usize,
    /// Notes per track (track_index -> notes)
    pub notes: Vec<Vec<NoteSnapshot>>,
}

/// Note snapshot for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSnapshot {
    pub measure: usize,
    pub beat: usize,
    pub string: u8,
    pub fret: u8,
    pub techniques: Vec<String>,
}

/// Transport state snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransportTestState {
    pub is_playing: bool,
    pub playhead_beats: f64,
    pub tempo: f64,
    pub time_signature: (u8, u8),
}

impl TestState {
    /// Create empty test state
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract state from tab editor
    pub fn from_tab_editor(state: &TabEditorState) -> TabEditorTestState {
        let mode = match state.mode {
            EditorMode::Normal => EditorModeSnapshot::Normal,
            EditorMode::Insert => EditorModeSnapshot::Insert,
            EditorMode::Visual => EditorModeSnapshot::Visual,
            EditorMode::Command => EditorModeSnapshot::Command,
        };

        let cursor = CursorSnapshot {
            track: state.cursor.track,
            measure: state.cursor.measure,
            beat: state.cursor.beat,
            string: state.cursor.string,
        };

        let mut notes = Vec::new();
        for (track_idx, track) in state.document.tracks.iter().enumerate() {
            let mut track_notes = Vec::new();
            for (measure_idx, measure) in state.document.measures.iter().enumerate() {
                for track_beats in &measure.track_beats {
                    if track_beats.track_id == track.id {
                        for (beat_idx, beat) in track_beats.beats.iter().enumerate() {
                            for note in &beat.notes {
                                track_notes.push(NoteSnapshot {
                                    measure: measure_idx,
                                    beat: beat_idx,
                                    string: note.string,
                                    fret: note.fret,
                                    techniques: note.techniques.iter()
                                        .map(|t| format!("{:?}", t))
                                        .collect(),
                                });
                            }
                        }
                    }
                }
            }
            notes.push(track_notes);
        }

        let document = DocumentSnapshot {
            title: state.document.metadata.title.clone(),
            track_count: state.document.tracks.len(),
            measure_count: state.document.measures.len(),
            notes,
        };

        TabEditorTestState {
            mode,
            cursor,
            document,
            has_selection: state.selection.is_some(),
            active_tool: if state.active_tool.is_none() {
                None
            } else {
                Some(format!("{:?}", state.active_tool))
            },
            is_modified: state.is_modified,
        }
    }

    /// Check if a note exists at position
    pub fn note_exists(&self, measure: usize, beat: usize, string: u8) -> bool {
        if let Some(tab) = &self.tab_editor {
            for track_notes in &tab.document.notes {
                for note in track_notes {
                    if note.measure == measure && note.beat == beat && note.string == string {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get note at position
    pub fn get_note(&self, measure: usize, beat: usize, string: u8) -> Option<&NoteSnapshot> {
        if let Some(tab) = &self.tab_editor {
            for track_notes in &tab.document.notes {
                for note in track_notes {
                    if note.measure == measure && note.beat == beat && note.string == string {
                        return Some(note);
                    }
                }
            }
        }
        None
    }

    /// Count total notes
    pub fn total_note_count(&self) -> usize {
        if let Some(tab) = &self.tab_editor {
            tab.document.notes.iter().map(|t| t.len()).sum()
        } else {
            0
        }
    }
}

impl From<EditorMode> for EditorModeSnapshot {
    fn from(mode: EditorMode) -> Self {
        match mode {
            EditorMode::Normal => EditorModeSnapshot::Normal,
            EditorMode::Insert => EditorModeSnapshot::Insert,
            EditorMode::Visual => EditorModeSnapshot::Visual,
            EditorMode::Command => EditorModeSnapshot::Command,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_state() {
        let state = TestState::new();
        assert!(state.tab_editor.is_none());
        assert_eq!(state.project.track_count, 0);
    }

    #[test]
    fn test_editor_mode_conversion() {
        assert_eq!(EditorModeSnapshot::from(EditorMode::Normal), EditorModeSnapshot::Normal);
        assert_eq!(EditorModeSnapshot::from(EditorMode::Insert), EditorModeSnapshot::Insert);
    }
}
