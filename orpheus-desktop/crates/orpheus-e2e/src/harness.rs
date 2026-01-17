//! Test harness for running E2E tests
//!
//! Provides the runtime environment for executing test scenarios.

use crate::{TestAction, Assertion, TestState, TabEditorTestState};
use orpheus_core::tab::{TabDocument, TabTrack, Technique};
use orpheus_ui::{TabEditorState, EditorMode, TabCursor};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tracing::{debug, info, warn};

/// Test harness for running E2E scenarios
pub struct TestHarness {
    /// Current test state
    state: TestState,
    /// Tab editor state (for direct manipulation)
    tab_editor: Option<TabEditorState>,
    /// Temp directory for test files
    temp_dir: TempDir,
    /// Collected errors
    errors: Vec<String>,
    /// Start time
    start_time: Instant,
}

impl TestHarness {
    /// Create a new test harness
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        info!("Created test harness with temp dir: {:?}", temp_dir.path());

        Self {
            state: TestState::new(),
            tab_editor: None,
            temp_dir,
            errors: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// Get current test state
    pub fn state(&self) -> &TestState {
        &self.state
    }

    /// Get mutable reference to test state
    pub fn state_mut(&mut self) -> &mut TestState {
        &mut self.state
    }

    /// Get temp directory path
    pub fn temp_dir(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Execute a test action
    pub fn execute(&mut self, action: &TestAction) -> Result<(), String> {
        debug!("Executing action: {:?}", std::mem::discriminant(action));

        match action {
            TestAction::NewProject(name) => {
                self.state.project.name = name.clone();
                self.state.project.track_count = 0;
                self.state.project.track_names.clear();
                self.state.project.is_dirty = false;
                Ok(())
            }

            TestAction::OpenTabEditor => {
                let mut editor = TabEditorState::with_guitar_track();
                self.state.tab_editor = Some(TestState::from_tab_editor(&editor));
                self.tab_editor = Some(editor);
                Ok(())
            }

            TestAction::TabEditor(tab_action) => {
                self.execute_tab_action(tab_action)
            }

            TestAction::Track(track_action) => {
                self.execute_track_action(track_action)
            }

            TestAction::Undo => {
                if let Some(ref mut editor) = self.tab_editor {
                    editor.undo();
                    self.sync_tab_editor_state();
                }
                Ok(())
            }

            TestAction::Redo => {
                if let Some(ref mut editor) = self.tab_editor {
                    editor.redo();
                    self.sync_tab_editor_state();
                }
                Ok(())
            }

            TestAction::Wait(duration) => {
                std::thread::sleep(*duration);
                Ok(())
            }

            _ => {
                warn!("Action not yet implemented: {:?}", std::mem::discriminant(action));
                Ok(())
            }
        }
    }

    /// Execute a tab editor action
    fn execute_tab_action(&mut self, action: &crate::TabEditorAction) -> Result<(), String> {
        use crate::TabEditorAction;

        let editor = self.tab_editor.as_mut()
            .ok_or_else(|| "Tab editor not open".to_string())?;

        match action {
            TabEditorAction::EnterInsertMode => {
                editor.mode = EditorMode::Insert;
            }
            TabEditorAction::EnterNormalMode => {
                editor.mode = EditorMode::Normal;
            }
            TabEditorAction::EnterVisualMode => {
                editor.mode = EditorMode::Visual;
            }

            TabEditorAction::MoveCursor(cursor_move) => {
                use crate::CursorMove;
                match cursor_move {
                    CursorMove::Up => editor.move_up(),
                    CursorMove::Down => editor.move_down(),
                    CursorMove::Left => editor.move_left(),
                    CursorMove::Right => editor.move_right(),
                    CursorMove::NextMeasure => editor.next_measure(),
                    CursorMove::PrevMeasure => editor.prev_measure(),
                    _ => {}
                }
            }

            TabEditorAction::JumpTo { measure, beat, string } => {
                editor.cursor.measure = *measure;
                editor.cursor.beat = *beat;
                editor.cursor.string = *string;
            }

            TabEditorAction::EnterNote { string, fret } => {
                // Set cursor to string
                editor.cursor.string = *string;
                // Enter the note
                editor.enter_note(*fret);
            }

            TabEditorAction::EnterRest => {
                editor.enter_rest();
            }

            TabEditorAction::DeleteNote => {
                editor.delete_note();
            }

            TabEditorAction::ApplyTechnique(_technique) => {
                // This would apply to the note at cursor
                // For now, we'll just update the active tool
            }

            TabEditorAction::SetDuration(duration) => {
                editor.set_duration(*duration);
            }

            TabEditorAction::ToggleDotted => {
                editor.toggle_dotted();
            }

            TabEditorAction::ToggleTriplet => {
                editor.toggle_triplet();
            }

            TabEditorAction::AddMeasure => {
                editor.add_measure();
            }

            TabEditorAction::Copy => {
                editor.copy();
            }

            TabEditorAction::Cut => {
                editor.cut();
            }

            TabEditorAction::Paste => {
                editor.paste();
            }

            TabEditorAction::SelectAll => {
                editor.select_all();
            }

            TabEditorAction::DeselectAll => {
                editor.clear_selection();
            }

            _ => {
                warn!("Tab action not yet implemented: {:?}", action);
            }
        }

        self.sync_tab_editor_state();
        Ok(())
    }

    /// Execute a track action
    fn execute_track_action(&mut self, action: &crate::TrackAction) -> Result<(), String> {
        use crate::TrackAction;

        match action {
            TrackAction::AddAudioTrack(name) => {
                self.state.project.track_count += 1;
                self.state.project.track_names.push(name.clone());
                self.state.project.is_dirty = true;
            }
            TrackAction::AddMidiTrack(name) => {
                self.state.project.track_count += 1;
                self.state.project.track_names.push(name.clone());
                self.state.project.is_dirty = true;
            }
            TrackAction::AddTabTrack(name) => {
                self.state.project.track_count += 1;
                self.state.project.track_names.push(name.clone());
                self.state.project.is_dirty = true;

                // Also add to tab editor if open
                if let Some(ref mut editor) = self.tab_editor {
                    let track = TabTrack::guitar(name);
                    editor.document.add_track(track);
                    self.sync_tab_editor_state();
                }
            }
            _ => {
                warn!("Track action not yet implemented: {:?}", action);
            }
        }

        Ok(())
    }

    /// Sync tab editor state to test state
    fn sync_tab_editor_state(&mut self) {
        if let Some(ref editor) = self.tab_editor {
            self.state.tab_editor = Some(TestState::from_tab_editor(editor));
        }
    }

    /// Check an assertion
    pub fn check(&self, assertion: &Assertion) -> Result<(), String> {
        debug!("Checking assertion: {}", assertion.description());

        match assertion {
            Assertion::ProjectName(expected) => {
                if self.state.project.name != *expected {
                    return Err(format!(
                        "Expected project name '{}', got '{}'",
                        expected, self.state.project.name
                    ));
                }
            }

            Assertion::TrackCount(expected) => {
                if self.state.project.track_count != *expected {
                    return Err(format!(
                        "Expected {} tracks, got {}",
                        expected, self.state.project.track_count
                    ));
                }
            }

            Assertion::IsDirty => {
                if !self.state.project.is_dirty {
                    return Err("Expected project to have unsaved changes".to_string());
                }
            }

            Assertion::IsClean => {
                if self.state.project.is_dirty {
                    return Err("Expected project to have no unsaved changes".to_string());
                }
            }

            Assertion::TabEditorOpen => {
                if self.state.tab_editor.is_none() {
                    return Err("Expected tab editor to be open".to_string());
                }
            }

            Assertion::TabEditorMode(expected) => {
                let tab = self.state.tab_editor.as_ref()
                    .ok_or_else(|| "Tab editor not open".to_string())?;

                let actual = match tab.mode {
                    crate::state::EditorModeSnapshot::Normal => crate::assertions::TabEditorModeAssertion::Normal,
                    crate::state::EditorModeSnapshot::Insert => crate::assertions::TabEditorModeAssertion::Insert,
                    crate::state::EditorModeSnapshot::Visual => crate::assertions::TabEditorModeAssertion::Visual,
                    crate::state::EditorModeSnapshot::Command => crate::assertions::TabEditorModeAssertion::Command,
                };

                if actual != *expected {
                    return Err(format!("Expected mode {:?}, got {:?}", expected, actual));
                }
            }

            Assertion::CursorAt { measure, beat, string } => {
                let tab = self.state.tab_editor.as_ref()
                    .ok_or_else(|| "Tab editor not open".to_string())?;

                if tab.cursor.measure != *measure || tab.cursor.beat != *beat || tab.cursor.string != *string {
                    return Err(format!(
                        "Expected cursor at M{} B{} S{}, got M{} B{} S{}",
                        measure + 1, beat + 1, string,
                        tab.cursor.measure + 1, tab.cursor.beat + 1, tab.cursor.string
                    ));
                }
            }

            Assertion::NoteExists { measure, beat, string, fret } => {
                if let Some(note) = self.state.get_note(*measure, *beat, *string) {
                    if note.fret != *fret {
                        return Err(format!(
                            "Expected fret {} at M{} B{} S{}, got {}",
                            fret, measure + 1, beat + 1, string, note.fret
                        ));
                    }
                } else {
                    return Err(format!(
                        "No note found at M{} B{} S{}",
                        measure + 1, beat + 1, string
                    ));
                }
            }

            Assertion::NoNoteAt { measure, beat, string } => {
                if self.state.note_exists(*measure, *beat, *string) {
                    return Err(format!(
                        "Expected no note at M{} B{} S{}, but found one",
                        measure + 1, beat + 1, string
                    ));
                }
            }

            Assertion::MeasureCount(expected) => {
                let tab = self.state.tab_editor.as_ref()
                    .ok_or_else(|| "Tab editor not open".to_string())?;

                if tab.document.measure_count != *expected {
                    return Err(format!(
                        "Expected {} measures, got {}",
                        expected, tab.document.measure_count
                    ));
                }
            }

            Assertion::TotalNoteCount(expected) => {
                let actual = self.state.total_note_count();
                if actual != *expected {
                    return Err(format!(
                        "Expected {} notes, got {}",
                        expected, actual
                    ));
                }
            }

            Assertion::NoErrors => {
                if !self.errors.is_empty() {
                    return Err(format!("Errors occurred: {:?}", self.errors));
                }
            }

            _ => {
                warn!("Assertion not yet implemented: {}", assertion.description());
            }
        }

        Ok(())
    }

    /// Get elapsed time since harness creation
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.state.errors.push(self.errors.last().unwrap().clone());
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_creation() {
        let harness = TestHarness::new();
        assert!(harness.temp_dir().exists());
    }

    #[test]
    fn test_new_project_action() {
        let mut harness = TestHarness::new();
        harness.execute(&TestAction::NewProject("Test".to_string())).unwrap();
        assert_eq!(harness.state().project.name, "Test");
    }

    #[test]
    fn test_open_tab_editor() {
        let mut harness = TestHarness::new();
        harness.execute(&TestAction::OpenTabEditor).unwrap();
        assert!(harness.state().tab_editor.is_some());
    }
}
