//! Dialog components for Orpheus
//!
//! Modal dialogs for onboarding, preferences, and other settings.

pub mod welcome;
pub mod preferences;
pub mod about;

pub use welcome::{WelcomeDialog, WelcomeState, WelcomeAction};
pub use preferences::{
    PreferencesDialog, PreferencesState, PreferencesAction, PreferencesTab, ThemeMode,
};
pub use about::{AboutDialog, AboutState};
