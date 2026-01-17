//! Common UI widgets for consistent UX across all views
//!
//! This module provides reusable components that ensure visual and
//! behavioral consistency throughout the Orpheus application.

use egui::{Ui, Color32, Response, RichText, Vec2};
use crate::theme::Theme;
use crate::layout::constants::*;

// ============================================================================
// Button Variants
// ============================================================================

/// Button style variants for consistent styling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Primary action button - accent colored, prominent
    #[default]
    Primary,
    /// Secondary action button - subtle, gray
    Secondary,
    /// Tertiary/ghost button - minimal styling
    Tertiary,
    /// Danger/destructive action - red
    Danger,
    /// Success action - green
    Success,
    /// Warning action - yellow/orange
    Warning,
}

impl ButtonVariant {
    /// Get the fill color for this variant
    pub fn fill_color(&self, theme: &Theme, selected: bool) -> Color32 {
        let base = match self {
            Self::Primary => theme.palette.accent,
            Self::Secondary => theme.surface_bg(),
            Self::Tertiary => Color32::TRANSPARENT,
            Self::Danger => theme.palette.error,
            Self::Success => theme.palette.success,
            Self::Warning => theme.palette.warning,
        };

        if selected {
            base.gamma_multiply(0.8)
        } else {
            base
        }
    }

    /// Get the text color for this variant
    pub fn text_color(&self, theme: &Theme, selected: bool) -> Color32 {
        match self {
            Self::Primary | Self::Danger | Self::Success => Color32::WHITE,
            Self::Secondary => {
                if selected {
                    theme.palette.accent
                } else {
                    theme.text_primary()
                }
            }
            Self::Tertiary => theme.text_secondary(),
            Self::Warning => Color32::BLACK,
        }
    }
}

/// Create a styled button with consistent sizing and colors
pub fn styled_button(
    ui: &mut Ui,
    text: &str,
    variant: ButtonVariant,
    theme: &Theme,
    selected: bool,
) -> Response {
    let fill = variant.fill_color(theme, selected);
    let text_color = variant.text_color(theme, selected);

    let btn = egui::Button::new(
        RichText::new(text).color(text_color)
    )
    .fill(fill)
    .min_size(BUTTON_SIZE_MD);

    ui.add(btn)
}

/// Create a small styled button
pub fn styled_button_small(
    ui: &mut Ui,
    text: &str,
    variant: ButtonVariant,
    theme: &Theme,
) -> Response {
    let fill = variant.fill_color(theme, false);
    let text_color = variant.text_color(theme, false);

    let btn = egui::Button::new(
        RichText::new(text).color(text_color).small()
    )
    .fill(fill)
    .min_size(BUTTON_SIZE_SM);

    ui.add(btn)
}

/// Create a large primary action button
pub fn primary_action_button(
    ui: &mut Ui,
    icon: &str,
    text: &str,
    theme: &Theme,
) -> Response {
    let btn = egui::Button::new(
        RichText::new(format!("{} {}", icon, text))
            .color(Color32::WHITE)
    )
    .fill(theme.palette.accent)
    .min_size(BUTTON_SIZE_LG);

    ui.add(btn)
}

/// Create an icon-only button
pub fn icon_button(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
    ui.add(
        egui::Button::new(icon)
            .min_size(BUTTON_SIZE_ICON)
    ).on_hover_text(tooltip)
}

// ============================================================================
// Tab Bar Component
// ============================================================================

/// Tab bar with numbered keyboard shortcuts
pub struct TabBar<'a, T: PartialEq + Copy> {
    tabs: &'a [T],
    current: &'a mut T,
    theme: &'a Theme,
    get_label: Box<dyn Fn(&T) -> String + 'a>,
}

impl<'a, T: PartialEq + Copy> TabBar<'a, T> {
    pub fn new(
        tabs: &'a [T],
        current: &'a mut T,
        theme: &'a Theme,
        get_label: impl Fn(&T) -> String + 'a,
    ) -> Self {
        Self {
            tabs,
            current,
            theme,
            get_label: Box::new(get_label),
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for (idx, tab) in self.tabs.iter().enumerate() {
                let selected = *self.current == *tab;
                let label = (self.get_label)(tab);
                let text = format!("{} {}", idx + 1, label);

                let btn = egui::Button::new(
                    RichText::new(&text)
                        .color(if selected {
                            self.theme.palette.accent
                        } else {
                            self.theme.text_primary()
                        })
                )
                .fill(if selected {
                    self.theme.palette.accent.gamma_multiply(0.2)
                } else {
                    Color32::TRANSPARENT
                })
                .min_size(Vec2::new(TAB_BUTTON_MIN_WIDTH, TAB_BAR_HEIGHT));

                if ui.add(btn).on_hover_text(format!("Press {} to switch", idx + 1)).clicked() {
                    *self.current = *tab;
                }
            }
        });
    }
}

/// Handle tab switching via number keys (1-9)
pub fn handle_tab_keys<T: Copy>(ui: &mut Ui, tabs: &[T], current: &mut T) -> bool {
    let mut changed = false;

    ui.input(|i| {
        let keys = [
            egui::Key::Num1, egui::Key::Num2, egui::Key::Num3,
            egui::Key::Num4, egui::Key::Num5, egui::Key::Num6,
            egui::Key::Num7, egui::Key::Num8, egui::Key::Num9,
        ];

        for (idx, key) in keys.iter().enumerate() {
            if idx < tabs.len() && i.key_pressed(*key) {
                *current = tabs[idx];
                changed = true;
                break;
            }
        }
    });

    changed
}

// ============================================================================
// Status Bar Component
// ============================================================================

/// Unified status bar for all views
pub struct StatusBar<'a> {
    theme: &'a Theme,
    left_items: Vec<String>,
    right_items: Vec<String>,
    shortcuts: Vec<(&'a str, &'a str)>,
}

impl<'a> StatusBar<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            theme,
            left_items: Vec::new(),
            right_items: Vec::new(),
            shortcuts: Vec::new(),
        }
    }

    /// Add an item to the left side
    pub fn left(mut self, item: impl Into<String>) -> Self {
        self.left_items.push(item.into());
        self
    }

    /// Add an item to the right side
    pub fn right(mut self, item: impl Into<String>) -> Self {
        self.right_items.push(item.into());
        self
    }

    /// Add a keyboard shortcut hint
    pub fn shortcut(mut self, key: &'a str, action: &'a str) -> Self {
        self.shortcuts.push((key, action));
        self
    }

    /// Add common playback shortcuts
    pub fn with_playback_shortcuts(self) -> Self {
        self.shortcut("Space", "Play/Pause")
            .shortcut("Esc", "Stop")
    }

    /// Add common editing shortcuts
    pub fn with_edit_shortcuts(self) -> Self {
        self.shortcut("Ctrl+Z", "Undo")
            .shortcut("Del", "Delete")
    }

    /// Show the status bar
    pub fn show(&self, ui: &mut Ui) {
        ui.add_space(SPACING_XS);
        ui.separator();

        ui.horizontal(|ui| {
            ui.set_height(STATUS_BAR_HEIGHT);

            // Left items
            for (i, item) in self.left_items.iter().enumerate() {
                if i > 0 {
                    ui.label(
                        RichText::new("|")
                            .color(self.theme.text_muted())
                            .small()
                    );
                }
                ui.label(
                    RichText::new(item)
                        .color(self.theme.text_secondary())
                        .small()
                );
            }

            // Spacer
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Shortcuts (rightmost)
                if !self.shortcuts.is_empty() {
                    let hints: Vec<String> = self.shortcuts
                        .iter()
                        .map(|(k, a)| format!("{}: {}", k, a))
                        .collect();

                    ui.label(
                        RichText::new(hints.join(" | "))
                            .color(self.theme.text_muted())
                            .small()
                    );
                }

                // Right items
                for item in self.right_items.iter().rev() {
                    ui.label(
                        RichText::new(item)
                            .color(self.theme.text_secondary())
                            .small()
                    );
                }
            });
        });
    }
}

// ============================================================================
// Progress Indicator
// ============================================================================

/// Show a progress indicator with optional label
pub fn progress_indicator(ui: &mut Ui, progress: f32, label: Option<&str>) {
    ui.horizontal(|ui| {
        if let Some(text) = label {
            ui.label(RichText::new(text).small());
        }
        ui.add(egui::ProgressBar::new(progress).show_percentage());
    });
}

/// Show a spinner with label for indeterminate progress
pub fn loading_spinner(ui: &mut Ui, label: &str) {
    ui.horizontal(|ui| {
        ui.spinner();
        ui.label(RichText::new(label).small());
    });
}

// ============================================================================
// Section Headers
// ============================================================================

/// Collapsible section with consistent styling
pub fn collapsible_section(
    ui: &mut Ui,
    title: &str,
    default_open: bool,
    add_contents: impl FnOnce(&mut Ui),
) {
    egui::CollapsingHeader::new(title)
        .default_open(default_open)
        .show(ui, add_contents);
}

/// Group with heading
pub fn section_group(
    ui: &mut Ui,
    title: &str,
    add_contents: impl FnOnce(&mut Ui),
) {
    ui.group(|ui| {
        ui.heading(title);
        ui.add_space(SPACING_SM);
        add_contents(ui);
    });
}

// ============================================================================
// Form Helpers
// ============================================================================

/// Standard form grid layout
pub fn form_grid(
    ui: &mut Ui,
    id: &str,
    add_contents: impl FnOnce(&mut Ui),
) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing(GRID_FORM_SPACING)
        .show(ui, add_contents);
}

/// Form row with label and widget
pub fn form_row(ui: &mut Ui, label: &str, add_widget: impl FnOnce(&mut Ui)) {
    ui.label(label);
    add_widget(ui);
    ui.end_row();
}

// ============================================================================
// Validation Display
// ============================================================================

/// Validation status icons
pub fn validation_icon(valid: bool) -> &'static str {
    if valid { "✓" } else { "○" }
}

/// Validation status with color
pub fn validation_label(ui: &mut Ui, valid: bool, text: &str, theme: &Theme) {
    let icon = validation_icon(valid);
    let color = if valid {
        theme.palette.success
    } else {
        theme.text_secondary()
    };

    ui.horizontal(|ui| {
        ui.label(RichText::new(icon).color(color));
        ui.label(text);
    });
}

// ============================================================================
// Keyboard Shortcut Handling
// ============================================================================

/// Global playback control - returns true if state changed
pub fn handle_global_playback(ui: &mut Ui, is_playing: &mut bool) -> bool {
    let mut changed = false;

    ui.input(|i| {
        if i.key_pressed(egui::Key::Space) {
            *is_playing = !*is_playing;
            changed = true;
        }
        if i.key_pressed(egui::Key::Escape) {
            *is_playing = false;
            changed = true;
        }
    });

    changed
}

/// Handle common navigation keys
pub struct NavigationResult {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}

pub fn handle_navigation_keys(ui: &mut Ui) -> NavigationResult {
    let mut result = NavigationResult {
        left: false,
        right: false,
        up: false,
        down: false,
    };

    ui.input(|i| {
        result.left = i.key_pressed(egui::Key::ArrowLeft);
        result.right = i.key_pressed(egui::Key::ArrowRight);
        result.up = i.key_pressed(egui::Key::ArrowUp);
        result.down = i.key_pressed(egui::Key::ArrowDown);
    });

    result
}
