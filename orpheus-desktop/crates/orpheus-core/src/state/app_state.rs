//! Global application state

use serde::{Deserialize, Serialize};

/// Production modes available in Orpheus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ProductionMode {
    /// Timeline and arrangement editing
    Arrange,
    /// Tablature and notation editing
    #[default]
    Compose,
    /// Audio and MIDI recording
    Record,
    /// Mixing console
    Mix,
    /// Mastering chain
    Master,
    /// Release preparation - artwork, visualizers, promo materials
    Release,
    /// Practice and learning
    Practice,
    /// Distribution and export
    Distribute,
}

impl ProductionMode {
    /// Get display name for the mode
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Arrange => "Arrange",
            Self::Compose => "Compose",
            Self::Record => "Record",
            Self::Mix => "Mix",
            Self::Master => "Master",
            Self::Release => "Release",
            Self::Practice => "Practice",
            Self::Distribute => "Distribute",
        }
    }

    /// Get icon for the mode
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Arrange => "\u{1F4CB}",   // Clipboard (timeline)
            Self::Compose => "\u{1F3B5}",   // Musical note
            Self::Record => "\u{23FA}",     // Record button
            Self::Mix => "\u{1F39A}",       // Sliders
            Self::Master => "\u{1F4BF}",    // CD
            Self::Release => "\u{1F3A8}",   // Artist palette
            Self::Practice => "\u{1F3B8}",  // Guitar
            Self::Distribute => "\u{1F4E4}", // Outbox
        }
    }

    /// Get description for the mode (for tooltips)
    pub fn description(&self) -> &'static str {
        match self {
            Self::Arrange => "Arrange tracks and clips on the timeline",
            Self::Compose => "Edit tablature and musical notation",
            Self::Record => "Record audio and MIDI from inputs",
            Self::Mix => "Adjust volume, panning, and effects",
            Self::Master => "Finalize with mastering and export",
            Self::Release => "Create artwork, visualizers, and promo materials",
            Self::Practice => "Practice with tempo control and looping",
            Self::Distribute => "Publish and share your music",
        }
    }

    /// Get all modes in order
    pub fn all() -> &'static [ProductionMode] {
        &[
            Self::Arrange,
            Self::Compose,
            Self::Record,
            Self::Mix,
            Self::Master,
            Self::Release,
            Self::Practice,
            Self::Distribute,
        ]
    }
}

/// View state for UI panels and layout
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewState {
    /// Left panel visible
    pub left_panel_visible: bool,
    /// Right panel visible
    pub right_panel_visible: bool,
    /// Bottom panel visible
    pub bottom_panel_visible: bool,
    /// Left panel width
    pub left_panel_width: f32,
    /// Right panel width
    pub right_panel_width: f32,
    /// Bottom panel height
    pub bottom_panel_height: f32,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            left_panel_visible: true,
            right_panel_visible: true,
            bottom_panel_visible: false,
            left_panel_width: 250.0,
            right_panel_width: 300.0,
            bottom_panel_height: 200.0,
        }
    }
}

/// Global application state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppState {
    /// Current production mode
    pub mode: ProductionMode,
    /// View/layout state
    pub view: ViewState,
    /// Whether project has unsaved changes
    pub is_dirty: bool,
    /// Current project file path
    pub project_path: Option<std::path::PathBuf>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: ProductionMode::Compose,
            view: ViewState::new(),
            is_dirty: false,
            project_path: None,
        }
    }

    /// Mark project as modified
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Clear dirty flag (after save)
    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
    }
}
