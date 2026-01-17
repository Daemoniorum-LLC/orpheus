//! Custom widgets for Orpheus
//!
//! This module provides Orpheus-specific widgets (LevelMeter, Knob, TablatureView)
//! as well as re-exports from daemoniorum-egui for shared widgets (Button, Card, Dialog, etc.)

mod meter;
mod knob;
mod tablature;
pub mod common;

// Orpheus-specific widgets
pub use meter::LevelMeter;
pub use knob::Knob;
pub use tablature::{TablatureView, TabConfig, TabState, EditMode, NoteDuration};

// Common UX components
pub use common::{
    ButtonVariant, styled_button, styled_button_small, primary_action_button, icon_button,
    StatusBar, progress_indicator, loading_spinner,
    collapsible_section, section_group, form_grid, form_row,
    validation_icon, validation_label,
    handle_global_playback, handle_navigation_keys, handle_tab_keys, NavigationResult,
};

// Re-export shared daemoniorum-egui widgets
pub use daemoniorum_egui::widgets::{
    Badge, Card, CardStyle, ConfirmDialog, ConfirmDialogResponse, Dialog, EnhancedButton,
    LoadingIndicator, LoadingStyle, SelectableList, SelectableListResponse, StatusMessageWidget,
    TabBar, TabBarResponse, TabItem, TabStyle,
};

// Re-export icons for consistent iconography
pub use daemoniorum_egui::icons::Icon;
