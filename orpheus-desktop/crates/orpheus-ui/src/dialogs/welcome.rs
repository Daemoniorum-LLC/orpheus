//! Welcome/onboarding dialog for first-time users
//!
//! Shows a multi-step introduction to Orpheus features and shortcuts.

use egui::{Color32, RichText, Ui, Vec2};

/// Welcome dialog state
pub struct WelcomeState {
    /// Current step (0-indexed)
    pub current_step: usize,
    /// Whether to show on startup
    pub show_on_startup: bool,
    /// Whether dialog is visible
    pub visible: bool,
}

impl Default for WelcomeState {
    fn default() -> Self {
        Self::new()
    }
}

impl WelcomeState {
    pub fn new() -> Self {
        Self {
            current_step: 0,
            show_on_startup: true,
            visible: true,
        }
    }

    /// Reset to first step
    pub fn reset(&mut self) {
        self.current_step = 0;
    }

    /// Total number of steps
    pub fn total_steps(&self) -> usize {
        5
    }

    /// Check if on last step
    pub fn is_last_step(&self) -> bool {
        self.current_step >= self.total_steps() - 1
    }

    /// Go to next step
    pub fn next(&mut self) {
        if !self.is_last_step() {
            self.current_step += 1;
        }
    }

    /// Go to previous step
    pub fn prev(&mut self) {
        if self.current_step > 0 {
            self.current_step -= 1;
        }
    }
}

/// Actions from welcome dialog
#[derive(Debug, Clone)]
pub enum WelcomeAction {
    /// Close the dialog
    Close,
    /// Don't show on startup anymore
    DontShowAgain,
    /// Open preferences
    OpenPreferences,
    /// Start a new project
    NewProject,
}

/// Welcome dialog component
pub struct WelcomeDialog<'a> {
    state: &'a mut WelcomeState,
}

impl<'a> WelcomeDialog<'a> {
    pub fn new(state: &'a mut WelcomeState) -> Self {
        Self { state }
    }

    /// Show the welcome dialog
    pub fn show(&mut self, ctx: &egui::Context) -> Option<WelcomeAction> {
        if !self.state.visible {
            return None;
        }

        let mut action = None;

        egui::Window::new("Welcome to Orpheus")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .collapsible(false)
            .fixed_size([500.0, 400.0])
            .show(ctx, |ui| {
                action = self.show_content(ui);
            });

        action
    }

    fn show_content(&mut self, ui: &mut Ui) -> Option<WelcomeAction> {
        let mut action = None;

        ui.vertical_centered(|ui| {
            // Step indicator
            ui.horizontal(|ui| {
                for i in 0..self.state.total_steps() {
                    let color = if i == self.state.current_step {
                        Color32::from_rgb(26, 123, 93)
                    } else if i < self.state.current_step {
                        Color32::from_rgb(100, 100, 100)
                    } else {
                        Color32::from_rgb(60, 60, 60)
                    };
                    ui.add(egui::Label::new(RichText::new("●").color(color)));
                }
            });

            ui.add_space(16.0);

            // Step content
            match self.state.current_step {
                0 => self.show_step_intro(ui),
                1 => self.show_step_modes(ui),
                2 => self.show_step_tab_editor(ui),
                3 => self.show_step_shortcuts(ui),
                4 => self.show_step_finish(ui, &mut action),
                _ => {}
            }

            ui.add_space(16.0);
            ui.separator();

            // Navigation
            ui.horizontal(|ui| {
                // Don't show on startup checkbox
                if ui.checkbox(&mut self.state.show_on_startup, "Show on startup").changed() {
                    if !self.state.show_on_startup {
                        action = Some(WelcomeAction::DontShowAgain);
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.state.is_last_step() {
                        if ui.button("Get Started").clicked() {
                            self.state.visible = false;
                            action = Some(WelcomeAction::Close);
                        }
                    } else {
                        if ui.button("Next →").clicked() {
                            self.state.next();
                        }
                    }

                    if self.state.current_step > 0 {
                        if ui.button("← Back").clicked() {
                            self.state.prev();
                        }
                    }

                    if ui.button("Skip").on_hover_text("Skip the welcome tour").clicked() {
                        self.state.visible = false;
                        action = Some(WelcomeAction::Close);
                    }
                });
            });
        });

        action
    }

    fn show_step_intro(&self, ui: &mut Ui) {
        ui.heading("Welcome to Orpheus");
        ui.add_space(8.0);
        ui.label(
            "Orpheus is a professional music production DAW designed for guitarists, \
            bassists, and composers. Let's take a quick tour of the key features."
        );
        ui.add_space(16.0);

        ui.horizontal(|ui| {
            self.feature_card(ui, "🎸", "Tab Editor", "Create guitar and bass tablature");
            self.feature_card(ui, "🎹", "Piano Roll", "MIDI editing and composition");
            self.feature_card(ui, "🎚️", "Mixer", "Professional mixing tools");
        });
    }

    fn show_step_modes(&self, ui: &mut Ui) {
        ui.heading("Working Modes");
        ui.add_space(8.0);
        ui.label("Orpheus has different modes for different tasks:");
        ui.add_space(12.0);

        egui::Grid::new("modes_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                self.mode_row(ui, "Arrange", "Organize tracks, clips, and project structure");
                self.mode_row(ui, "Compose", "Write music with the tab editor or piano roll");
                self.mode_row(ui, "Record", "Record audio and MIDI performances");
                self.mode_row(ui, "Mix", "Balance levels, add effects, and shape your sound");
                self.mode_row(ui, "Master", "Finalize your mix and export");
                self.mode_row(ui, "Practice", "Learn and rehearse with tempo control");
            });
    }

    fn show_step_tab_editor(&self, ui: &mut Ui) {
        ui.heading("Tab Editor Modes");
        ui.add_space(8.0);
        ui.label("The tab editor uses vim-style modal editing:");
        ui.add_space(12.0);

        egui::Grid::new("tab_modes_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label(RichText::new("INSERT").strong().color(Color32::from_rgb(100, 180, 255)));
                ui.label("Type fret numbers directly");
                ui.end_row();

                ui.label(RichText::new("NORMAL").strong().color(Color32::from_rgb(100, 255, 100)));
                ui.label("Navigate and edit with shortcuts");
                ui.end_row();

                ui.label(RichText::new("VISUAL").strong().color(Color32::from_rgb(255, 180, 100)));
                ui.label("Select and manipulate ranges");
                ui.end_row();
            });

        ui.add_space(12.0);
        ui.label(RichText::new("Press 'i' for Insert, 'Esc' for Normal, 'v' for Visual").small().weak());
    }

    fn show_step_shortcuts(&self, ui: &mut Ui) {
        ui.heading("Essential Shortcuts");
        ui.add_space(8.0);

        egui::Grid::new("shortcuts_grid")
            .num_columns(2)
            .spacing([20.0, 6.0])
            .show(ui, |ui| {
                self.shortcut_row(ui, "Space", "Play / Pause");
                self.shortcut_row(ui, "Enter", "Stop / Rewind");
                self.shortcut_row(ui, "R", "Record");
                self.shortcut_row(ui, "Ctrl+N", "New Project");
                self.shortcut_row(ui, "Ctrl+O", "Open Project");
                self.shortcut_row(ui, "Ctrl+S", "Save Project");
                self.shortcut_row(ui, "Ctrl+Z", "Undo");
                self.shortcut_row(ui, "Ctrl+Shift+Z", "Redo");
                self.shortcut_row(ui, "L", "Toggle Loop");
                self.shortcut_row(ui, "M", "Toggle Metronome");
            });

        ui.add_space(8.0);
        ui.label(RichText::new("Tab Editor: W/H/Q/E/S for note durations, Shift+H/P for hammer-on/pull-off").small().weak());
    }

    fn show_step_finish(&mut self, ui: &mut Ui, action: &mut Option<WelcomeAction>) {
        ui.heading("You're Ready!");
        ui.add_space(8.0);
        ui.label("You now know the basics of Orpheus. Here are some ways to get started:");
        ui.add_space(16.0);

        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("📄 New Project").min_size(Vec2::new(140.0, 40.0))).clicked() {
                self.state.visible = false;
                *action = Some(WelcomeAction::NewProject);
            }

            ui.add_space(8.0);

            if ui.add(egui::Button::new("⚙ Preferences").min_size(Vec2::new(140.0, 40.0))).clicked() {
                *action = Some(WelcomeAction::OpenPreferences);
            }
        });

        ui.add_space(16.0);
        ui.label(RichText::new("You can always access this guide from Help → Welcome").small().weak());
    }

    fn feature_card(&self, ui: &mut Ui, icon: &str, title: &str, desc: &str) {
        ui.group(|ui| {
            ui.set_min_width(140.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(icon).size(32.0));
                ui.add_space(4.0);
                ui.label(RichText::new(title).strong());
                ui.label(RichText::new(desc).small().weak());
            });
        });
    }

    fn mode_row(&self, ui: &mut Ui, name: &str, desc: &str) {
        ui.label(RichText::new(name).strong());
        ui.label(desc);
        ui.end_row();
    }

    fn shortcut_row(&self, ui: &mut Ui, key: &str, desc: &str) {
        ui.label(RichText::new(key).monospace().strong().color(Color32::from_rgb(150, 200, 255)));
        ui.label(desc);
        ui.end_row();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome_state_default() {
        let state = WelcomeState::new();
        assert_eq!(state.current_step, 0);
        assert!(state.show_on_startup);
        assert!(state.visible);
    }

    #[test]
    fn test_welcome_navigation() {
        let mut state = WelcomeState::new();

        state.next();
        assert_eq!(state.current_step, 1);

        state.next();
        state.next();
        assert_eq!(state.current_step, 3);

        state.prev();
        assert_eq!(state.current_step, 2);

        state.reset();
        assert_eq!(state.current_step, 0);
    }

    #[test]
    fn test_welcome_last_step() {
        let mut state = WelcomeState::new();
        assert!(!state.is_last_step());

        // Navigate to last step
        for _ in 0..state.total_steps() {
            state.next();
        }
        assert!(state.is_last_step());

        // Should not go beyond last step
        let last = state.current_step;
        state.next();
        assert_eq!(state.current_step, last);
    }
}
