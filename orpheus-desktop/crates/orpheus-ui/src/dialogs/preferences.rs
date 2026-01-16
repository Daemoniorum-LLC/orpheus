//! Preferences dialog for application settings
//!
//! Allows users to customize theme, audio settings, keyboard shortcuts, and more.

use egui::{Color32, RichText, Ui, Vec2};

/// Preferences dialog state
pub struct PreferencesState {
    /// Whether dialog is visible
    pub visible: bool,
    /// Current tab
    pub current_tab: PreferencesTab,
    /// General settings
    pub general: GeneralSettings,
    /// Audio settings
    pub audio: AudioSettings,
    /// Theme settings
    pub theme: ThemeSettings,
    /// Editor settings
    pub editor: EditorSettings,
}

impl Default for PreferencesState {
    fn default() -> Self {
        Self::new()
    }
}

impl PreferencesState {
    pub fn new() -> Self {
        Self {
            visible: false,
            current_tab: PreferencesTab::General,
            general: GeneralSettings::default(),
            audio: AudioSettings::default(),
            theme: ThemeSettings::default(),
            editor: EditorSettings::default(),
        }
    }

    /// Sync preferences from core Settings struct
    pub fn sync_from_settings(&mut self, settings: &orpheus_core::Settings) {
        // General settings
        self.general.show_welcome = settings.ui.show_welcome_on_startup;
        self.general.auto_save_interval = settings.project.auto_save_interval;
        self.general.recent_projects_count = settings.project.max_recent;

        // Audio settings
        self.audio.sample_rate = settings.audio.sample_rate;
        self.audio.buffer_size = settings.audio.buffer_size;
        self.audio.input_device = settings.audio.input_device.clone();
        self.audio.output_device = settings.audio.output_device.clone();
        self.audio.midi_input = settings.midi.input_device.clone();

        // Theme settings
        self.theme.mode = if settings.ui.high_contrast {
            ThemeMode::HighContrast
        } else if settings.ui.theme == "light" {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        self.theme.ui_scale = settings.ui.scale;
    }

    /// Sync preferences to core Settings struct
    pub fn sync_to_settings(&self, settings: &mut orpheus_core::Settings) {
        // General settings
        settings.ui.show_welcome_on_startup = self.general.show_welcome;
        settings.project.auto_save_interval = self.general.auto_save_interval;
        settings.project.max_recent = self.general.recent_projects_count;

        // Audio settings
        settings.audio.sample_rate = self.audio.sample_rate;
        settings.audio.buffer_size = self.audio.buffer_size;
        settings.audio.input_device = self.audio.input_device.clone();
        settings.audio.output_device = self.audio.output_device.clone();
        settings.midi.input_device = self.audio.midi_input.clone();

        // Theme settings
        match self.theme.mode {
            ThemeMode::Dark => {
                settings.ui.theme = "dark".to_string();
                settings.ui.high_contrast = false;
            }
            ThemeMode::Light => {
                settings.ui.theme = "light".to_string();
                settings.ui.high_contrast = false;
            }
            ThemeMode::HighContrast => {
                settings.ui.theme = "high_contrast".to_string();
                settings.ui.high_contrast = true;
            }
        }
        settings.ui.scale = self.theme.ui_scale;
    }
}

/// Preferences tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferencesTab {
    General,
    Audio,
    Theme,
    Editor,
    Shortcuts,
}

impl PreferencesTab {
    pub fn name(&self) -> &'static str {
        match self {
            PreferencesTab::General => "General",
            PreferencesTab::Audio => "Audio",
            PreferencesTab::Theme => "Theme",
            PreferencesTab::Editor => "Editor",
            PreferencesTab::Shortcuts => "Shortcuts",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PreferencesTab::General => "⚙",
            PreferencesTab::Audio => "🔊",
            PreferencesTab::Theme => "🎨",
            PreferencesTab::Editor => "✏",
            PreferencesTab::Shortcuts => "⌨",
        }
    }
}

/// General settings
#[derive(Debug, Clone)]
pub struct GeneralSettings {
    /// Show welcome on startup
    pub show_welcome: bool,
    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u32,
    /// Recent projects count
    pub recent_projects_count: usize,
    /// Confirm before exit
    pub confirm_exit: bool,
    /// Check for updates
    pub check_updates: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            show_welcome: true,
            auto_save_interval: 300, // 5 minutes
            recent_projects_count: 10,
            confirm_exit: true,
            check_updates: true,
        }
    }
}

/// Audio settings
#[derive(Debug, Clone)]
pub struct AudioSettings {
    /// Sample rate
    pub sample_rate: u32,
    /// Buffer size
    pub buffer_size: u32,
    /// Input device
    pub input_device: Option<String>,
    /// Output device
    pub output_device: Option<String>,
    /// MIDI input device
    pub midi_input: Option<String>,
    /// Enable audio monitoring
    pub monitoring_enabled: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            buffer_size: 256,
            input_device: None,
            output_device: None,
            midi_input: None,
            monitoring_enabled: false,
        }
    }
}

/// Theme settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
    HighContrast,
}

impl ThemeMode {
    pub fn name(&self) -> &'static str {
        match self {
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
            ThemeMode::HighContrast => "High Contrast",
        }
    }
}

/// Theme settings
#[derive(Debug, Clone)]
pub struct ThemeSettings {
    /// Selected theme mode
    pub mode: ThemeMode,
    /// Custom accent color
    pub custom_accent: Option<Color32>,
    /// Use rounded controls
    pub rounded_controls: bool,
    /// UI scale factor
    pub ui_scale: f32,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Dark,
            custom_accent: None,
            rounded_controls: true,
            ui_scale: 1.0,
        }
    }
}

/// Editor settings
#[derive(Debug, Clone)]
pub struct EditorSettings {
    /// Default note duration
    pub default_duration: String,
    /// Snap to grid
    pub snap_to_grid: bool,
    /// Show beat numbers
    pub show_beat_numbers: bool,
    /// Show technique hints
    pub show_technique_hints: bool,
    /// Auto-insert mode
    pub auto_insert_mode: bool,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            default_duration: "quarter".to_string(),
            snap_to_grid: true,
            show_beat_numbers: true,
            show_technique_hints: true,
            auto_insert_mode: false,
        }
    }
}

/// Actions from preferences dialog
#[derive(Debug, Clone)]
pub enum PreferencesAction {
    /// Close dialog
    Close,
    /// Apply changes
    Apply,
    /// Reset to defaults
    ResetDefaults,
    /// Theme changed
    ThemeChanged(ThemeMode),
    /// UI scale changed
    ScaleChanged(f32),
}

/// Preferences dialog component
pub struct PreferencesDialog<'a> {
    state: &'a mut PreferencesState,
}

impl<'a> PreferencesDialog<'a> {
    pub fn new(state: &'a mut PreferencesState) -> Self {
        Self { state }
    }

    /// Show the preferences dialog
    pub fn show(&mut self, ctx: &egui::Context) -> Option<PreferencesAction> {
        if !self.state.visible {
            return None;
        }

        let mut action = None;

        let mut open = true;
        egui::Window::new("Preferences")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .collapsible(false)
            .open(&mut open)
            .fixed_size([600.0, 450.0])
            .show(ctx, |ui| {
                action = self.show_content(ui);
            });

        if !open {
            self.state.visible = false;
            action = Some(PreferencesAction::Close);
        }

        action
    }

    fn show_content(&mut self, ui: &mut Ui) -> Option<PreferencesAction> {
        let mut action = None;

        ui.horizontal(|ui| {
            // Sidebar tabs
            ui.vertical(|ui| {
                ui.set_min_width(120.0);
                for tab in [
                    PreferencesTab::General,
                    PreferencesTab::Audio,
                    PreferencesTab::Theme,
                    PreferencesTab::Editor,
                    PreferencesTab::Shortcuts,
                ] {
                    let selected = self.state.current_tab == tab;
                    if ui.selectable_label(selected, format!("{} {}", tab.icon(), tab.name())).clicked() {
                        self.state.current_tab = tab;
                    }
                }
            });

            ui.separator();

            // Content area
            ui.vertical(|ui| {
                ui.set_min_width(440.0);

                match self.state.current_tab {
                    PreferencesTab::General => self.show_general(ui),
                    PreferencesTab::Audio => self.show_audio(ui),
                    PreferencesTab::Theme => action = self.show_theme(ui),
                    PreferencesTab::Editor => self.show_editor(ui),
                    PreferencesTab::Shortcuts => self.show_shortcuts(ui),
                }
            });
        });

        ui.separator();

        // Bottom buttons
        ui.horizontal(|ui| {
            if ui.button("Reset to Defaults")
                .on_hover_text("Restore all settings to factory defaults")
                .clicked()
            {
                action = Some(PreferencesAction::ResetDefaults);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close")
                    .on_hover_text("Close without saving changes")
                    .clicked()
                {
                    self.state.visible = false;
                    action = Some(PreferencesAction::Close);
                }

                if ui.button("Apply")
                    .on_hover_text("Save and apply settings")
                    .clicked()
                {
                    action = Some(PreferencesAction::Apply);
                }
            });
        });

        action
    }

    fn show_general(&mut self, ui: &mut Ui) {
        ui.heading("General Settings");
        ui.add_space(8.0);

        egui::Grid::new("general_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Show welcome on startup:");
                ui.checkbox(&mut self.state.general.show_welcome, "");
                ui.end_row();

                ui.label("Confirm before exit:");
                ui.checkbox(&mut self.state.general.confirm_exit, "");
                ui.end_row();

                ui.label("Check for updates:");
                ui.checkbox(&mut self.state.general.check_updates, "");
                ui.end_row();

                ui.label("Auto-save interval:");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.state.general.auto_save_interval)
                        .range(0..=600)
                        .suffix(" sec"));
                    if self.state.general.auto_save_interval == 0 {
                        ui.label(RichText::new("(disabled)").small().weak());
                    }
                });
                ui.end_row();

                ui.label("Recent projects:");
                ui.add(egui::DragValue::new(&mut self.state.general.recent_projects_count)
                    .range(0..=20));
                ui.end_row();
            });
    }

    fn show_audio(&mut self, ui: &mut Ui) {
        ui.heading("Audio Settings");
        ui.add_space(8.0);

        egui::Grid::new("audio_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Sample Rate:");
                egui::ComboBox::from_id_salt("sample_rate")
                    .selected_text(format!("{} Hz", self.state.audio.sample_rate))
                    .show_ui(ui, |ui| {
                        for rate in [44100, 48000, 88200, 96000] {
                            ui.selectable_value(&mut self.state.audio.sample_rate, rate, format!("{} Hz", rate));
                        }
                    }).response.on_hover_text("Audio sample rate (higher = better quality, more CPU)");
                ui.end_row();

                ui.label("Buffer Size:");
                egui::ComboBox::from_id_salt("buffer_size")
                    .selected_text(format!("{} samples", self.state.audio.buffer_size))
                    .show_ui(ui, |ui| {
                        for size in [64, 128, 256, 512, 1024, 2048] {
                            let latency = (size as f64 / self.state.audio.sample_rate as f64) * 1000.0;
                            ui.selectable_value(
                                &mut self.state.audio.buffer_size,
                                size,
                                format!("{} samples ({:.1}ms)", size, latency)
                            );
                        }
                    }).response.on_hover_text("Smaller = lower latency, higher CPU; larger = more stable");
                ui.end_row();

                ui.label("Input monitoring:");
                ui.checkbox(&mut self.state.audio.monitoring_enabled, "Enable")
                    .on_hover_text("Hear input signal through headphones while recording");
                ui.end_row();
            });

        ui.add_space(12.0);
        ui.label(RichText::new("Note: Changing audio settings may require restarting the audio engine.").small().weak());
    }

    fn show_theme(&mut self, ui: &mut Ui) -> Option<PreferencesAction> {
        let mut action = None;

        ui.heading("Theme Settings");
        ui.add_space(8.0);

        egui::Grid::new("theme_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Theme:");
                let old_mode = self.state.theme.mode;
                egui::ComboBox::from_id_salt("theme_mode")
                    .selected_text(self.state.theme.mode.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.state.theme.mode, ThemeMode::Dark, "Dark");
                        ui.selectable_value(&mut self.state.theme.mode, ThemeMode::Light, "Light");
                        ui.selectable_value(&mut self.state.theme.mode, ThemeMode::HighContrast, "High Contrast");
                    });
                if self.state.theme.mode != old_mode {
                    action = Some(PreferencesAction::ThemeChanged(self.state.theme.mode));
                }
                ui.end_row();

                ui.label("UI Scale:");
                let old_scale = self.state.theme.ui_scale;
                ui.add(egui::Slider::new(&mut self.state.theme.ui_scale, 0.75..=2.0)
                    .step_by(0.25)
                    .suffix("x"));
                if (self.state.theme.ui_scale - old_scale).abs() > 0.01 {
                    action = Some(PreferencesAction::ScaleChanged(self.state.theme.ui_scale));
                }
                ui.end_row();

                ui.label("Rounded controls:");
                ui.checkbox(&mut self.state.theme.rounded_controls, "Enable");
                ui.end_row();
            });

        ui.add_space(12.0);

        // Theme preview
        ui.group(|ui| {
            ui.label(RichText::new("Preview").strong());
            ui.horizontal(|ui| {
                for (label, color) in [
                    ("Accent", Color32::from_rgb(26, 123, 93)),
                    ("Success", Color32::from_rgb(52, 168, 83)),
                    ("Warning", Color32::from_rgb(251, 188, 4)),
                    ("Error", Color32::from_rgb(234, 67, 53)),
                ] {
                    ui.vertical(|ui| {
                        let (_, rect) = ui.allocate_space(Vec2::new(40.0, 30.0));
                        ui.painter().rect_filled(rect, 4.0, color);
                        ui.label(RichText::new(label).small());
                    });
                }
            });
        });

        action
    }

    fn show_editor(&mut self, ui: &mut Ui) {
        ui.heading("Editor Settings");
        ui.add_space(8.0);

        egui::Grid::new("editor_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Snap to grid:");
                ui.checkbox(&mut self.state.editor.snap_to_grid, "Enable")
                    .on_hover_text("Automatically align notes to grid divisions");
                ui.end_row();

                ui.label("Show beat numbers:");
                ui.checkbox(&mut self.state.editor.show_beat_numbers, "Enable")
                    .on_hover_text("Display beat numbers above the staff");
                ui.end_row();

                ui.label("Show technique hints:");
                ui.checkbox(&mut self.state.editor.show_technique_hints, "Enable")
                    .on_hover_text("Show technique symbols (H, P, B, etc.) on notes");
                ui.end_row();

                ui.label("Auto insert mode:");
                ui.checkbox(&mut self.state.editor.auto_insert_mode, "Enable")
                    .on_hover_text("Automatically advance cursor after entering a note");
                ui.end_row();

                ui.label("Default duration:");
                egui::ComboBox::from_id_salt("default_duration")
                    .selected_text(&self.state.editor.default_duration)
                    .show_ui(ui, |ui| {
                        for dur in ["whole", "half", "quarter", "eighth", "sixteenth"] {
                            ui.selectable_value(&mut self.state.editor.default_duration, dur.to_string(), dur);
                        }
                    }).response.on_hover_text("Note duration for new notes");
                ui.end_row();
            });
    }

    fn show_shortcuts(&mut self, ui: &mut Ui) {
        ui.heading("Keyboard Shortcuts");
        ui.add_space(8.0);

        ui.label(RichText::new("Common shortcuts (read-only in this version)").weak());
        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                egui::Grid::new("shortcuts_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .show(ui, |ui| {
                        self.shortcut_row(ui, "Space", "Play / Pause");
                        self.shortcut_row(ui, "Enter", "Stop / Rewind");
                        self.shortcut_row(ui, "R", "Record");
                        self.shortcut_row(ui, "L", "Toggle Loop");
                        self.shortcut_row(ui, "M", "Toggle Metronome");
                        self.shortcut_row(ui, "Ctrl+N", "New Project");
                        self.shortcut_row(ui, "Ctrl+O", "Open Project");
                        self.shortcut_row(ui, "Ctrl+S", "Save Project");
                        self.shortcut_row(ui, "Ctrl+Shift+S", "Save As");
                        self.shortcut_row(ui, "Ctrl+Z", "Undo");
                        self.shortcut_row(ui, "Ctrl+Shift+Z", "Redo");
                        self.shortcut_row(ui, "Ctrl+T", "Add Track");
                        self.shortcut_row(ui, "Delete", "Delete Selection");

                        ui.add_space(8.0);
                        ui.end_row();
                        ui.label(RichText::new("Tab Editor").strong());
                        ui.end_row();

                        self.shortcut_row(ui, "i", "Enter Insert Mode");
                        self.shortcut_row(ui, "Esc", "Enter Normal Mode");
                        self.shortcut_row(ui, "v", "Enter Visual Mode");
                        self.shortcut_row(ui, "W/H/Q/E/S", "Whole/Half/Quarter/Eighth/Sixteenth");
                        self.shortcut_row(ui, "Shift+H", "Hammer-on");
                        self.shortcut_row(ui, "Shift+P", "Pull-off");
                        self.shortcut_row(ui, "Shift+S", "Slide");
                        self.shortcut_row(ui, "Shift+B", "Bend");
                    });
            });
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
    fn test_preferences_state_default() {
        let state = PreferencesState::new();
        assert!(!state.visible);
        assert_eq!(state.current_tab, PreferencesTab::General);
    }

    #[test]
    fn test_general_settings_default() {
        let settings = GeneralSettings::default();
        assert!(settings.show_welcome);
        assert_eq!(settings.auto_save_interval, 300);
        assert!(settings.confirm_exit);
    }

    #[test]
    fn test_audio_settings_default() {
        let settings = AudioSettings::default();
        assert_eq!(settings.sample_rate, 44100);
        assert_eq!(settings.buffer_size, 256);
    }

    #[test]
    fn test_theme_settings_default() {
        let settings = ThemeSettings::default();
        assert_eq!(settings.mode, ThemeMode::Dark);
        assert!(settings.rounded_controls);
        assert!((settings.ui_scale - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_editor_settings_default() {
        let settings = EditorSettings::default();
        assert!(settings.snap_to_grid);
        assert!(settings.show_beat_numbers);
    }

    #[test]
    fn test_preferences_tab_names() {
        assert_eq!(PreferencesTab::General.name(), "General");
        assert_eq!(PreferencesTab::Audio.name(), "Audio");
        assert_eq!(PreferencesTab::Theme.name(), "Theme");
    }
}
