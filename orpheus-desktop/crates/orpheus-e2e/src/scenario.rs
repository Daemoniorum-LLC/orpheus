//! Scenario builder for fluent E2E test definition
//!
//! Provides a fluent API for defining test scenarios:
//!
//! ```rust,ignore
//! ScenarioBuilder::new("Tab editor workflow")
//!     .given_new_project()
//!     .when_open_tab_editor()
//!     .and_enter_note(6, 0)
//!     .then_note_exists(6, 0)
//!     .run();
//! ```

use crate::{TestAction, TabEditorAction, TrackAction, CursorMove, Assertion, TestHarness};
use orpheus_core::tab::{Technique, BaseDuration, BendAmount, TapType, SlideDirection};
use std::time::{Duration, Instant};
use tracing::{info, error};

/// Result of running a scenario
#[derive(Debug)]
pub struct ScenarioResult {
    /// Scenario name
    pub name: String,
    /// Whether all steps passed
    pub passed: bool,
    /// Total duration
    pub duration: Duration,
    /// Individual step results
    pub steps: Vec<StepResult>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Result of a single step
#[derive(Debug)]
pub struct StepResult {
    /// Step description
    pub description: String,
    /// Whether step passed
    pub passed: bool,
    /// Step duration
    pub duration: Duration,
    /// Error if failed
    pub error: Option<String>,
}

/// Builder for creating and running test scenarios
pub struct ScenarioBuilder {
    name: String,
    setup_actions: Vec<(String, TestAction)>,
    test_actions: Vec<(String, TestAction)>,
    assertions: Vec<(String, Assertion)>,
    timeout: Duration,
}

impl ScenarioBuilder {
    /// Create a new scenario with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            setup_actions: Vec::new(),
            test_actions: Vec::new(),
            assertions: Vec::new(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Set timeout for the scenario
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    // ==================== Setup Actions (Given) ====================

    /// Given: Create a new project
    pub fn given_new_project(mut self) -> Self {
        self.setup_actions.push((
            "Create new project".to_string(),
            TestAction::NewProject("Test Project".to_string()),
        ));
        self
    }

    /// Given: Create a new project with name
    pub fn given_project_named(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.setup_actions.push((
            format!("Create project '{}'", name),
            TestAction::NewProject(name),
        ));
        self
    }

    /// Given: Add an audio track
    pub fn given_audio_track(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.setup_actions.push((
            format!("Add audio track '{}'", name),
            TestAction::Track(TrackAction::AddAudioTrack(name)),
        ));
        self
    }

    /// Given: Add a tab track
    pub fn given_tab_track(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.setup_actions.push((
            format!("Add tab track '{}'", name),
            TestAction::Track(TrackAction::AddTabTrack(name)),
        ));
        self
    }

    // ==================== Test Actions (When/And) ====================

    /// When: Open the tab editor
    pub fn when_open_tab_editor(mut self) -> Self {
        self.test_actions.push((
            "Open tab editor".to_string(),
            TestAction::OpenTabEditor,
        ));
        self
    }

    /// And: Enter insert mode
    pub fn and_enter_insert_mode(mut self) -> Self {
        self.test_actions.push((
            "Enter insert mode".to_string(),
            TestAction::TabEditor(TabEditorAction::EnterInsertMode),
        ));
        self
    }

    /// And: Enter normal mode
    pub fn and_enter_normal_mode(mut self) -> Self {
        self.test_actions.push((
            "Enter normal mode".to_string(),
            TestAction::TabEditor(TabEditorAction::EnterNormalMode),
        ));
        self
    }

    /// And: Enter a note
    pub fn and_enter_note(mut self, string: u8, fret: u8) -> Self {
        self.test_actions.push((
            format!("Enter note: string {} fret {}", string, fret),
            TestAction::TabEditor(TabEditorAction::EnterNote { string, fret }),
        ));
        self
    }

    /// And: Enter a rest
    pub fn and_enter_rest(mut self) -> Self {
        self.test_actions.push((
            "Enter rest".to_string(),
            TestAction::TabEditor(TabEditorAction::EnterRest),
        ));
        self
    }

    /// And: Delete note at cursor
    pub fn and_delete_note(mut self) -> Self {
        self.test_actions.push((
            "Delete note".to_string(),
            TestAction::TabEditor(TabEditorAction::DeleteNote),
        ));
        self
    }

    /// And: Move cursor
    pub fn and_move_cursor(mut self, direction: CursorMove) -> Self {
        self.test_actions.push((
            format!("Move cursor {:?}", direction),
            TestAction::TabEditor(TabEditorAction::MoveCursor(direction)),
        ));
        self
    }

    /// And: Move cursor right
    pub fn and_move_right(self) -> Self {
        self.and_move_cursor(CursorMove::Right)
    }

    /// And: Move cursor left
    pub fn and_move_left(self) -> Self {
        self.and_move_cursor(CursorMove::Left)
    }

    /// And: Move cursor up
    pub fn and_move_up(self) -> Self {
        self.and_move_cursor(CursorMove::Up)
    }

    /// And: Move cursor down
    pub fn and_move_down(self) -> Self {
        self.and_move_cursor(CursorMove::Down)
    }

    /// And: Set duration
    pub fn and_set_duration(mut self, duration: BaseDuration) -> Self {
        self.test_actions.push((
            format!("Set duration to {:?}", duration),
            TestAction::TabEditor(TabEditorAction::SetDuration(duration)),
        ));
        self
    }

    /// And: Toggle dotted
    pub fn and_toggle_dotted(mut self) -> Self {
        self.test_actions.push((
            "Toggle dotted".to_string(),
            TestAction::TabEditor(TabEditorAction::ToggleDotted),
        ));
        self
    }

    /// And: Add a measure
    pub fn and_add_measure(mut self) -> Self {
        self.test_actions.push((
            "Add measure".to_string(),
            TestAction::TabEditor(TabEditorAction::AddMeasure),
        ));
        self
    }

    /// And: Apply a technique
    pub fn and_apply_technique(mut self, technique: Technique) -> Self {
        self.test_actions.push((
            format!("Apply technique {:?}", technique),
            TestAction::TabEditor(TabEditorAction::ApplyTechnique(technique)),
        ));
        self
    }

    /// And: Undo
    pub fn and_undo(mut self) -> Self {
        self.test_actions.push((
            "Undo".to_string(),
            TestAction::Undo,
        ));
        self
    }

    /// And: Redo
    pub fn and_redo(mut self) -> Self {
        self.test_actions.push((
            "Redo".to_string(),
            TestAction::Redo,
        ));
        self
    }

    /// And: Wait for duration
    pub fn and_wait(mut self, duration: Duration) -> Self {
        self.test_actions.push((
            format!("Wait {:?}", duration),
            TestAction::Wait(duration),
        ));
        self
    }

    // ==================== Assertions (Then) ====================

    /// Then: Tab editor should be open
    pub fn then_tab_editor_is_open(mut self) -> Self {
        self.assertions.push((
            "Tab editor is open".to_string(),
            Assertion::TabEditorOpen,
        ));
        self
    }

    /// Then: Note should exist at position
    pub fn then_note_exists(mut self, string: u8, fret: u8) -> Self {
        self.assertions.push((
            format!("Note exists at string {} fret {}", string, fret),
            Assertion::NoteExists {
                measure: 0,
                beat: 0,
                string,
                fret,
            },
        ));
        self
    }

    /// Then: Note should exist at specific position
    pub fn then_note_at(mut self, measure: usize, beat: usize, string: u8, fret: u8) -> Self {
        self.assertions.push((
            format!("Note at M{} B{} S{} = fret {}", measure + 1, beat + 1, string, fret),
            Assertion::NoteExists { measure, beat, string, fret },
        ));
        self
    }

    /// Then: No note at position
    pub fn then_no_note_at(mut self, measure: usize, beat: usize, string: u8) -> Self {
        self.assertions.push((
            format!("No note at M{} B{} S{}", measure + 1, beat + 1, string),
            Assertion::NoNoteAt { measure, beat, string },
        ));
        self
    }

    /// Then: Cursor should be at position
    pub fn then_cursor_at(mut self, measure: usize, beat: usize, string: u8) -> Self {
        self.assertions.push((
            format!("Cursor at M{} B{} S{}", measure + 1, beat + 1, string),
            Assertion::CursorAt { measure, beat, string },
        ));
        self
    }

    /// Then: Measure count should be
    pub fn then_measure_count_is(mut self, count: usize) -> Self {
        self.assertions.push((
            format!("Measure count is {}", count),
            Assertion::MeasureCount(count),
        ));
        self
    }

    /// Then: Note count should be
    pub fn then_note_count_is(mut self, count: usize) -> Self {
        self.assertions.push((
            format!("Note count is {}", count),
            Assertion::TotalNoteCount(count),
        ));
        self
    }

    /// Then: Track count should be
    pub fn then_track_count_is(mut self, count: usize) -> Self {
        self.assertions.push((
            format!("Track count is {}", count),
            Assertion::TrackCount(count),
        ));
        self
    }

    /// Then: Project should be dirty
    pub fn then_project_is_dirty(mut self) -> Self {
        self.assertions.push((
            "Project has unsaved changes".to_string(),
            Assertion::IsDirty,
        ));
        self
    }

    /// Then: No errors
    pub fn then_no_errors(mut self) -> Self {
        self.assertions.push((
            "No errors".to_string(),
            Assertion::NoErrors,
        ));
        self
    }

    // ==================== Run ====================

    /// Run the scenario and return results
    pub fn run(self) -> ScenarioResult {
        info!("Running scenario: {}", self.name);
        let start = Instant::now();
        let mut harness = TestHarness::new();
        let mut steps = Vec::new();
        let mut all_passed = true;

        // Run setup actions
        for (desc, action) in &self.setup_actions {
            let step_start = Instant::now();
            let result = harness.execute(action);
            let step_passed = result.is_ok();
            let error = result.err();

            if !step_passed {
                all_passed = false;
                error!("Setup failed: {} - {:?}", desc, error);
            }

            steps.push(StepResult {
                description: format!("Setup: {}", desc),
                passed: step_passed,
                duration: step_start.elapsed(),
                error,
            });

            if !step_passed {
                break;
            }
        }

        // Run test actions
        if all_passed {
            for (desc, action) in &self.test_actions {
                let step_start = Instant::now();
                let result = harness.execute(action);
                let step_passed = result.is_ok();
                let error = result.err();

                if !step_passed {
                    all_passed = false;
                    error!("Action failed: {} - {:?}", desc, error);
                }

                steps.push(StepResult {
                    description: format!("Action: {}", desc),
                    passed: step_passed,
                    duration: step_start.elapsed(),
                    error,
                });

                if !step_passed {
                    break;
                }
            }
        }

        // Run assertions
        if all_passed {
            for (desc, assertion) in &self.assertions {
                let step_start = Instant::now();
                let result = harness.check(assertion);
                let step_passed = result.is_ok();
                let error = result.err();

                if !step_passed {
                    all_passed = false;
                    error!("Assertion failed: {} - {:?}", desc, error);
                }

                steps.push(StepResult {
                    description: format!("Assert: {}", desc),
                    passed: step_passed,
                    duration: step_start.elapsed(),
                    error,
                });
            }
        }

        let error = if all_passed {
            None
        } else {
            steps.iter()
                .find(|s| !s.passed)
                .and_then(|s| s.error.clone())
        };

        let result = ScenarioResult {
            name: self.name,
            passed: all_passed,
            duration: start.elapsed(),
            steps,
            error,
        };

        if result.passed {
            info!("Scenario PASSED in {:?}", result.duration);
        } else {
            error!("Scenario FAILED: {:?}", result.error);
        }

        result
    }

    /// Run and assert the scenario passes
    pub fn run_and_assert(self) {
        let result = self.run();
        if !result.passed {
            panic!(
                "Scenario '{}' failed: {}\n\nSteps:\n{}",
                result.name,
                result.error.unwrap_or_default(),
                result.steps.iter()
                    .map(|s| format!(
                        "  {} {} ({})",
                        if s.passed { "✓" } else { "✗" },
                        s.description,
                        if let Some(e) = &s.error { e } else { "ok" }
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_builder_creation() {
        let scenario = ScenarioBuilder::new("Test scenario");
        assert_eq!(scenario.name, "Test scenario");
    }

    #[test]
    fn test_basic_scenario() {
        let result = ScenarioBuilder::new("Open tab editor")
            .given_new_project()
            .when_open_tab_editor()
            .then_tab_editor_is_open()
            .run();

        assert!(result.passed);
    }

    #[test]
    fn test_note_entry_scenario() {
        let result = ScenarioBuilder::new("Enter a note")
            .given_new_project()
            .when_open_tab_editor()
            .and_enter_insert_mode()
            .and_enter_note(6, 0)
            .then_note_count_is(1)
            .run();

        assert!(result.passed, "Failed: {:?}", result.error);
    }
}
