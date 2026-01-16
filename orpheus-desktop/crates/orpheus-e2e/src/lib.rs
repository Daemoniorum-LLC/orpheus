//! # Orpheus E2E Testing Framework
//!
//! End-to-end testing framework for the Orpheus music production platform.
//! Based on Eidolon's ScenarioBuilder pattern for fluent test definition.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use orpheus_e2e::{ScenarioBuilder, TestAction};
//!
//! #[test]
//! fn test_tab_editor_workflow() {
//!     ScenarioBuilder::new("Tab editor note entry")
//!         .given_new_project()
//!         .when_open_tab_editor()
//!         .and_enter_note(6, 0)  // String 6, fret 0 (low E open)
//!         .and_apply_technique(Technique::PalmMute)
//!         .and_move_cursor_right()
//!         .and_enter_note(6, 3)  // String 6, fret 3
//!         .then_note_count_is(2)
//!         .then_note_exists(6, 0)
//!         .run();
//! }
//! ```

mod actions;
mod assertions;
mod harness;
mod scenario;
mod state;

pub use actions::{TestAction, CursorMove, TrackAction, TabEditorAction};
pub use assertions::Assertion;
pub use harness::TestHarness;
pub use scenario::{ScenarioBuilder, ScenarioResult, StepResult};
pub use state::{TestState, TabEditorTestState, ProjectTestState};
