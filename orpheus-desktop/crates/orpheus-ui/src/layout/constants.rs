//! Layout constants for consistent spacing and sizing across all views
//!
//! This module defines standard spacing, sizing, and layout constants
//! to ensure UX consistency throughout the Orpheus application.

use egui::{Vec2, Color32};

// ============================================================================
// Spacing Constants
// ============================================================================

/// Extra small spacing (4px) - tight inline elements
pub const SPACING_XS: f32 = 4.0;

/// Small spacing (8px) - between related elements
pub const SPACING_SM: f32 = 8.0;

/// Medium spacing (12px) - standard gap between elements
pub const SPACING_MD: f32 = 12.0;

/// Large spacing (16px) - between sections
pub const SPACING_LG: f32 = 16.0;

/// Extra large spacing (20px) - major section breaks
pub const SPACING_XL: f32 = 20.0;

/// XXL spacing (24px) - page-level spacing
pub const SPACING_XXL: f32 = 24.0;

// ============================================================================
// Grid Layout Constants
// ============================================================================

/// Standard form grid spacing [horizontal, vertical]
pub const GRID_FORM_SPACING: [f32; 2] = [12.0, 8.0];

/// Standard grid spacing for button/card grids
pub const GRID_CARD_SPACING: [f32; 2] = [8.0, 8.0];

/// Compact grid spacing
pub const GRID_COMPACT_SPACING: [f32; 2] = [4.0, 4.0];

/// Wide grid spacing for large items
pub const GRID_WIDE_SPACING: [f32; 2] = [16.0, 12.0];

// ============================================================================
// Panel Widths
// ============================================================================

/// Narrow sidebar width (120px)
pub const PANEL_WIDTH_NARROW: f32 = 120.0;

/// Standard sidebar width (200px)
pub const PANEL_WIDTH_STANDARD: f32 = 200.0;

/// Medium panel width (280px)
pub const PANEL_WIDTH_MEDIUM: f32 = 280.0;

/// Wide panel width (350px)
pub const PANEL_WIDTH_WIDE: f32 = 350.0;

/// Extra wide panel (450px)
pub const PANEL_WIDTH_EXTRA_WIDE: f32 = 450.0;

// ============================================================================
// Button Sizes
// ============================================================================

/// Small button minimum size
pub const BUTTON_SIZE_SM: Vec2 = Vec2::new(60.0, 24.0);

/// Medium button minimum size
pub const BUTTON_SIZE_MD: Vec2 = Vec2::new(100.0, 30.0);

/// Large button minimum size
pub const BUTTON_SIZE_LG: Vec2 = Vec2::new(150.0, 36.0);

/// Extra large button (primary actions)
pub const BUTTON_SIZE_XL: Vec2 = Vec2::new(200.0, 40.0);

/// Icon button size (square)
pub const BUTTON_SIZE_ICON: Vec2 = Vec2::new(28.0, 28.0);

// ============================================================================
// Input Field Widths
// ============================================================================

/// Short input field width (100px) - numbers, codes
pub const INPUT_WIDTH_SHORT: f32 = 100.0;

/// Medium input field width (200px) - names, titles
pub const INPUT_WIDTH_MEDIUM: f32 = 200.0;

/// Standard input field width (300px) - descriptions
pub const INPUT_WIDTH_STANDARD: f32 = 300.0;

/// Wide input field width (400px) - long text
pub const INPUT_WIDTH_WIDE: f32 = 400.0;

/// Label width for form layouts
pub const LABEL_WIDTH: f32 = 120.0;

// ============================================================================
// Track & Timeline Constants
// ============================================================================

/// Standard track height
pub const TRACK_HEIGHT: f32 = 80.0;

/// Minimum track height
pub const TRACK_HEIGHT_MIN: f32 = 40.0;

/// Maximum track height
pub const TRACK_HEIGHT_MAX: f32 = 200.0;

/// Track header/label width
pub const TRACK_HEADER_WIDTH: f32 = 200.0;

/// Timeline ruler height
pub const TIMELINE_RULER_HEIGHT: f32 = 24.0;

// ============================================================================
// Status Bar Constants
// ============================================================================

/// Status bar height
pub const STATUS_BAR_HEIGHT: f32 = 24.0;

/// Status bar padding
pub const STATUS_BAR_PADDING: f32 = 8.0;

// ============================================================================
// Tab Bar Constants
// ============================================================================

/// Tab bar height
pub const TAB_BAR_HEIGHT: f32 = 32.0;

/// Tab button minimum width
pub const TAB_BUTTON_MIN_WIDTH: f32 = 80.0;

// ============================================================================
// Corner Radius
// ============================================================================

/// Small corner radius (2px)
pub const RADIUS_SM: f32 = 2.0;

/// Medium corner radius (4px)
pub const RADIUS_MD: f32 = 4.0;

/// Large corner radius (8px)
pub const RADIUS_LG: f32 = 8.0;

/// Extra large corner radius (12px)
pub const RADIUS_XL: f32 = 12.0;

// ============================================================================
// Keyboard Shortcut Display
// ============================================================================

/// Format keyboard shortcut hint for status bar
pub fn format_shortcut(key: &str, action: &str) -> String {
    format!("{}: {}", key, action)
}

/// Format multiple shortcuts for status bar
pub fn format_shortcuts(shortcuts: &[(&str, &str)]) -> String {
    shortcuts
        .iter()
        .map(|(key, action)| format_shortcut(key, action))
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Common keyboard shortcuts used across views
pub mod shortcuts {
    pub const PLAY_PAUSE: (&str, &str) = ("Space", "Play/Pause");
    pub const STOP: (&str, &str) = ("Escape", "Stop");
    pub const MUTE: (&str, &str) = ("M", "Mute");
    pub const SOLO: (&str, &str) = ("S", "Solo");
    pub const RECORD: (&str, &str) = ("R", "Record");
    pub const LOOP: (&str, &str) = ("L", "Loop");
    pub const GRID: (&str, &str) = ("G", "Grid");
    pub const ZOOM_IN: (&str, &str) = ("+", "Zoom In");
    pub const ZOOM_OUT: (&str, &str) = ("-", "Zoom Out");
    pub const UNDO: (&str, &str) = ("Ctrl+Z", "Undo");
    pub const REDO: (&str, &str) = ("Ctrl+Shift+Z", "Redo");
    pub const SAVE: (&str, &str) = ("Ctrl+S", "Save");
    pub const EXPORT: (&str, &str) = ("E", "Export");
    pub const GENERATE: (&str, &str) = ("G", "Generate");
    pub const TAB_NAV: (&str, &str) = ("1-6", "Tabs");
    pub const NAV_ARROWS: (&str, &str) = ("Arrows", "Navigate");
    pub const DELETE: (&str, &str) = ("Del", "Delete");
    pub const SELECT_ALL: (&str, &str) = ("Ctrl+A", "Select All");
}

// ============================================================================
// Track Colors (Persistent)
// ============================================================================

/// Standard track color palette (8 colors)
pub const TRACK_COLORS: [Color32; 8] = [
    Color32::from_rgb(231, 76, 60),   // Red
    Color32::from_rgb(241, 196, 15),  // Yellow
    Color32::from_rgb(46, 204, 113),  // Green
    Color32::from_rgb(52, 152, 219),  // Blue
    Color32::from_rgb(155, 89, 182),  // Purple
    Color32::from_rgb(230, 126, 34),  // Orange
    Color32::from_rgb(26, 188, 156),  // Teal
    Color32::from_rgb(236, 240, 241), // Light Gray
];

/// Get track color by index (cycles through palette)
pub fn track_color(index: usize) -> Color32 {
    TRACK_COLORS[index % TRACK_COLORS.len()]
}

// ============================================================================
// Animation Durations
// ============================================================================

/// Fast animation (100ms)
pub const ANIM_FAST_MS: u64 = 100;

/// Normal animation (200ms)
pub const ANIM_NORMAL_MS: u64 = 200;

/// Slow animation (300ms)
pub const ANIM_SLOW_MS: u64 = 300;

// ============================================================================
// Responsive Breakpoints
// ============================================================================

/// Compact width threshold
pub const BREAKPOINT_COMPACT: f32 = 800.0;

/// Standard width threshold
pub const BREAKPOINT_STANDARD: f32 = 1200.0;

/// Wide width threshold
pub const BREAKPOINT_WIDE: f32 = 1600.0;

/// Calculate number of columns based on available width
pub fn responsive_columns(available_width: f32, min_columns: usize, max_columns: usize) -> usize {
    if available_width < BREAKPOINT_COMPACT {
        min_columns
    } else if available_width < BREAKPOINT_STANDARD {
        ((max_columns + min_columns) / 2).max(min_columns)
    } else {
        max_columns
    }
}
