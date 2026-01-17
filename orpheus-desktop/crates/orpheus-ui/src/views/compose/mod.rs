//! Compose mode view - tablature and notation editing

use egui::{Color32, ScrollArea, Ui, RichText};
use orpheus_file::guitar_pro::GuitarProFile;
use orpheus_file::{FileFilter, OpenDialog};
use crate::widgets::{TabState, TablatureView};
use crate::theme::Theme;
use crate::layout::constants::*;

/// State for the compose view
pub struct ComposeViewState {
    /// Loaded Guitar Pro file
    pub gp_file: Option<GuitarProFile>,
    /// Tablature view state
    pub tab_state: TabState,
    /// File path (for display)
    pub file_path: Option<String>,
    /// Error message if file failed to load
    pub load_error: Option<String>,
}

impl Default for ComposeViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl ComposeViewState {
    pub fn new() -> Self {
        Self {
            gp_file: None,
            tab_state: TabState::new(),
            file_path: None,
            load_error: None,
        }
    }

    /// Load a Guitar Pro file
    pub fn load_file(&mut self, path: &str) {
        match orpheus_file::guitar_pro::parse_file(path) {
            Ok(file) => {
                self.gp_file = Some(file);
                self.file_path = Some(path.to_string());
                self.load_error = None;
                self.tab_state = TabState::new(); // Reset state
            }
            Err(e) => {
                self.load_error = Some(format!("Failed to load file: {:?}", e));
                self.gp_file = None;
            }
        }
    }

    /// Clear the current file
    pub fn clear_file(&mut self) {
        self.gp_file = None;
        self.file_path = None;
        self.load_error = None;
    }
}

/// Compose mode view component
pub struct ComposeView<'a> {
    state: &'a mut ComposeViewState,
    theme: &'a Theme,
}

impl<'a> ComposeView<'a> {
    pub fn new(state: &'a mut ComposeViewState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    /// Show the compose view
    pub fn show(&mut self, ui: &mut Ui) {
        // Handle file drops
        self.handle_file_drop(ui);

        // Handle keyboard shortcuts
        self.handle_keyboard(ui);

        // Main vertical layout (content + status bar)
        ui.vertical(|ui| {
            // Main horizontal layout
            let available_height = ui.available_height() - STATUS_BAR_HEIGHT - SPACING_SM;

            ui.allocate_ui_with_layout(
                egui::Vec2::new(ui.available_width(), available_height),
                egui::Layout::left_to_right(egui::Align::TOP),
                |ui| {
                    // Left panel: Track list (fixed width)
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(PANEL_WIDTH_STANDARD, ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            self.show_track_panel(ui);
                        },
                    );

                    ui.separator();

                    // Main area: Tablature view
                    ui.vertical(|ui| {
                        // Toolbar
                        self.show_toolbar(ui);

                        ui.separator();

                        // Tab view with scrolling
                        ScrollArea::both()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                if let Some(file) = &self.state.gp_file {
                                    TablatureView::new(&mut self.state.tab_state)
                                        .with_file(file)
                                        .show(ui);
                                } else {
                                    TablatureView::new(&mut self.state.tab_state).show(ui);
                                }
                            });
                    });
                },
            );

            // Status bar
            self.show_status_bar(ui);
        });

        // Show error if any
        let mut clear_error = false;
        if let Some(error) = &self.state.load_error {
            let error = error.clone();
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(&error);
                    if ui.button("OK").clicked() {
                        clear_error = true;
                    }
                });
        }
        if clear_error {
            self.state.load_error = None;
        }
    }

    /// Handle keyboard shortcuts
    fn handle_keyboard(&mut self, ui: &mut Ui) {
        ui.input(|i| {
            // Zoom controls
            if i.key_pressed(egui::Key::Equals) || i.key_pressed(egui::Key::Plus) {
                self.state.tab_state.zoom = (self.state.tab_state.zoom + 0.1).min(2.0);
            }
            if i.key_pressed(egui::Key::Minus) {
                self.state.tab_state.zoom = (self.state.tab_state.zoom - 0.1).max(0.5);
            }

            // Navigation with arrow keys
            if i.key_pressed(egui::Key::ArrowUp) {
                // Previous track
                if self.state.tab_state.selected_track > 0 {
                    self.state.tab_state.selected_track -= 1;
                }
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                // Next track
                if let Some(file) = &self.state.gp_file {
                    if self.state.tab_state.selected_track < file.tracks.len().saturating_sub(1) {
                        self.state.tab_state.selected_track += 1;
                    }
                }
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                // Previous measure
                if let Some(idx) = self.state.tab_state.selected_measure {
                    if idx > 0 {
                        self.state.tab_state.selected_measure = Some(idx - 1);
                    }
                }
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                // Next measure
                if let Some(file) = &self.state.gp_file {
                    if let Some(idx) = self.state.tab_state.selected_measure {
                        if idx < file.measures.len().saturating_sub(1) {
                            self.state.tab_state.selected_measure = Some(idx + 1);
                        }
                    } else {
                        self.state.tab_state.selected_measure = Some(0);
                    }
                }
            }

            // Open file with Ctrl+O
            if i.modifiers.ctrl && i.key_pressed(egui::Key::O) {
                if let Some(paths) = OpenDialog::new()
                    .title("Open Guitar Pro File")
                    .filter(FileFilter::guitar_pro())
                    .show()
                {
                    if let Some(path) = paths.first() {
                        self.state.load_file(&path.to_string_lossy());
                    }
                }
            }

            // Home/End for first/last measure
            if i.key_pressed(egui::Key::Home) {
                self.state.tab_state.selected_measure = Some(0);
            }
            if i.key_pressed(egui::Key::End) {
                if let Some(file) = &self.state.gp_file {
                    self.state.tab_state.selected_measure = Some(file.measures.len().saturating_sub(1));
                }
            }

            // Escape to deselect
            if i.key_pressed(egui::Key::Escape) {
                self.state.tab_state.selected_measure = None;
                self.state.tab_state.selected_beat = None;
            }
        });
    }

    /// Show the status bar
    fn show_status_bar(&self, ui: &mut Ui) {
        ui.add_space(SPACING_XS);
        ui.separator();

        ui.horizontal(|ui| {
            ui.set_height(STATUS_BAR_HEIGHT);

            // File info
            if let Some(file) = &self.state.gp_file {
                ui.label(
                    RichText::new(format!("{} tracks", file.tracks.len()))
                        .color(self.theme.text_secondary())
                        .small()
                );
                ui.label(RichText::new("|").color(self.theme.text_muted()).small());
                ui.label(
                    RichText::new(format!("{} measures", file.measures.len()))
                        .color(self.theme.text_secondary())
                        .small()
                );

                // Selected measure info
                if let Some(idx) = self.state.tab_state.selected_measure {
                    ui.label(RichText::new("|").color(self.theme.text_muted()).small());
                    ui.label(
                        RichText::new(format!("Measure {}", idx + 1))
                            .color(self.theme.accent())
                            .small()
                    );
                }
            } else {
                ui.label(
                    RichText::new("No file loaded - Open a Guitar Pro file or drag and drop")
                        .color(self.theme.text_secondary())
                        .small()
                );
            }

            // Zoom display
            ui.label(RichText::new("|").color(self.theme.text_muted()).small());
            ui.label(
                RichText::new(format!("Zoom: {:.0}%", self.state.tab_state.zoom * 100.0))
                    .color(self.theme.text_secondary())
                    .small()
            );

            // Right side: keyboard hints
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("Arrows: Navigate | +/-: Zoom | Ctrl+O: Open | Home/End: First/Last")
                        .color(self.theme.text_muted())
                        .small()
                );
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // File info
            if let Some(file) = &self.state.gp_file {
                ui.label(
                    egui::RichText::new(&file.info.title)
                        .strong()
                        .color(self.theme.palette.text_primary),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(&file.info.artist)
                        .color(self.theme.palette.text_secondary),
                );
                ui.separator();
                ui.label(format!("{} BPM", file.tempo));
                ui.label(format!(
                    "{}/{}",
                    file.time_signature.numerator,
                    file.time_signature.denominator
                ));
            } else {
                ui.label("No file loaded");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Zoom controls
                if ui.button("-").on_hover_text("Zoom out (-)").clicked() {
                    self.state.tab_state.zoom = (self.state.tab_state.zoom - 0.1).max(0.5);
                }
                ui.label(format!("{:.0}%", self.state.tab_state.zoom * 100.0));
                if ui.button("+").on_hover_text("Zoom in (+)").clicked() {
                    self.state.tab_state.zoom = (self.state.tab_state.zoom + 0.1).min(2.0);
                }
                ui.separator();

                // Open file button
                if ui.button("Open File...").on_hover_text("Open a Guitar Pro file (Ctrl+O)").clicked() {
                    // Show file open dialog
                    if let Some(paths) = OpenDialog::new()
                        .title("Open Guitar Pro File")
                        .filter(FileFilter::guitar_pro())
                        .show()
                    {
                        if let Some(path) = paths.first() {
                            self.state.load_file(&path.to_string_lossy());
                        }
                    }
                }

                if self.state.gp_file.is_some() && ui.button("Close").on_hover_text("Close current file").clicked() {
                    self.state.clear_file();
                }
            });
        });
    }

    fn show_track_panel(&mut self, ui: &mut Ui) {
        ui.heading("Tracks");
        ui.separator();

        if let Some(file) = &self.state.gp_file {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (idx, track) in file.tracks.iter().enumerate() {
                        let selected = self.state.tab_state.selected_track == idx;

                        let bg_color = if selected {
                            self.theme.palette.accent.gamma_multiply(0.3)
                        } else {
                            Color32::TRANSPARENT
                        };

                        egui::Frame::none()
                            .fill(bg_color)
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                            .rounding(4.0)
                            .show(ui, |ui| {
                                let response = ui.horizontal(|ui| {
                                    // Track color indicator
                                    let (r, g, b) = track.color;
                                    let color = Color32::from_rgb(r, g, b);
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::Vec2::new(4.0, 24.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 2.0, color);

                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(&track.name)
                                                    .strong()
                                                    .color(if selected {
                                                        self.theme.palette.text_primary
                                                    } else {
                                                        self.theme.palette.text_secondary
                                                    }),
                                            );
                                        });

                                        ui.horizontal(|ui| {
                                            let icon = if track.is_drums { "D" } else { "G" };
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{} {} strings",
                                                    icon,
                                                    track.strings
                                                ))
                                                .small()
                                                .color(self.theme.palette.text_secondary),
                                            );
                                        });
                                    });
                                });

                                // Handle track selection
                                if response.response.interact(egui::Sense::click()).clicked() {
                                    self.state.tab_state.selected_track = idx;
                                    self.state.tab_state.selected_measure = None;
                                    self.state.tab_state.selected_beat = None;
                                }
                            });

                        ui.add_space(2.0);
                    }
                });
        } else {
            ui.label(
                egui::RichText::new("Open a Guitar Pro file to see tracks")
                    .color(self.theme.palette.text_secondary),
            );
        }

        ui.separator();

        // Song info section
        if let Some(file) = &self.state.gp_file {
            ui.collapsing("Song Info", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Title:");
                    ui.label(&file.info.title);
                });
                ui.horizontal(|ui| {
                    ui.label("Artist:");
                    ui.label(&file.info.artist);
                });
                ui.horizontal(|ui| {
                    ui.label("Album:");
                    ui.label(&file.info.album);
                });
                ui.horizontal(|ui| {
                    ui.label("Tab Author:");
                    ui.label(&file.info.tab_author);
                });
                ui.horizontal(|ui| {
                    ui.label("Measures:");
                    ui.label(format!("{}", file.measures.len()));
                });
            });

            // Section markers
            let markers: Vec<_> = file
                .measures
                .iter()
                .filter(|m| m.marker.is_some())
                .collect();

            if !markers.is_empty() {
                ui.collapsing("Sections", |ui| {
                    for measure in markers.iter().take(20) {
                        let marker = measure.marker.as_ref().unwrap();
                        let response = ui.horizontal(|ui| {
                            ui.label(format!("M{}: {}", measure.number, marker));
                        });

                        // Click to jump to measure
                        if response.response.interact(egui::Sense::click()).clicked() {
                            // Find measure index
                            if let Some(idx) = file
                                .measures
                                .iter()
                                .position(|m| m.number == measure.number)
                            {
                                self.state.tab_state.selected_measure = Some(idx);
                            }
                        }
                    }
                });
            }
        }
    }

    fn handle_file_drop(&mut self, ui: &mut Ui) {
        // Check for dropped files
        ui.ctx().input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        let path_str = path.to_string_lossy();
                        let ext = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e.to_lowercase())
                            .unwrap_or_default();

                        if matches!(ext.as_str(), "gp" | "gp3" | "gp4" | "gp5" | "gpx") {
                            self.state.load_file(&path_str);
                            return;
                        }
                    }
                }
            }
        });
    }
}

/// Inspector panel for selected notes/beats
pub struct BeatInspector<'a> {
    state: &'a ComposeViewState,
    theme: &'a Theme,
}

impl<'a> BeatInspector<'a> {
    pub fn new(state: &'a ComposeViewState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    pub fn show(&self, ui: &mut Ui) {
        ui.heading("Inspector");
        ui.separator();

        let Some(file) = &self.state.gp_file else {
            ui.label("No file loaded");
            return;
        };

        let Some(measure_idx) = self.state.tab_state.selected_measure else {
            ui.label("Select a measure to inspect");
            return;
        };

        let measure = &file.measures[measure_idx];
        let track_idx = self.state.tab_state.selected_track;

        // Measure info
        ui.group(|ui| {
            ui.label(
                egui::RichText::new(format!("Measure {}", measure.number))
                    .strong()
                    .color(self.theme.palette.text_primary),
            );

            if let Some(marker) = &measure.marker {
                ui.label(format!("Section: {}", marker));
            }

            if measure.repeat_start {
                ui.label("Repeat Start");
            }

            if measure.repeat_end > 0 {
                ui.label(format!("Repeat End (x{})", measure.repeat_end));
            }
        });

        // Beat info
        if let Some(beat_idx) = self.state.tab_state.selected_beat {
            if let Some(track_beats) = measure.beats.get(track_idx) {
                if let Some(beat) = track_beats.beats.get(beat_idx) {
                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Beat {}", beat_idx + 1))
                                .strong()
                                .color(self.theme.palette.text_primary),
                        );

                        ui.label(format!("Duration: {:?}", beat.duration));

                        if beat.is_rest {
                            ui.label("Rest");
                        }

                        if beat.dotted {
                            ui.label("Dotted");
                        }

                        if let Some(tuplet) = beat.tuplet {
                            ui.label(format!("Tuplet: {}", tuplet));
                        }

                        if let Some(text) = &beat.text {
                            ui.label(format!("Text: {}", text));
                        }

                        // Notes
                        if !beat.notes.is_empty() {
                            ui.separator();
                            ui.label(
                                egui::RichText::new("Notes:")
                                    .color(self.theme.palette.text_secondary),
                            );

                            for note in &beat.notes {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "String {}: Fret {}",
                                        note.string, note.fret
                                    ));

                                    if note.tied {
                                        ui.label("(tied)");
                                    }

                                    if note.ghost {
                                        ui.label("(ghost)");
                                    }
                                });

                                // Note effects
                                let effects = &note.effects;
                                let mut effect_strs = Vec::new();

                                if effects.hammer_on {
                                    effect_strs.push("Hammer-on");
                                }
                                if effects.pull_off {
                                    effect_strs.push("Pull-off");
                                }
                                if effects.vibrato {
                                    effect_strs.push("Vibrato");
                                }
                                if effects.slide.is_some() {
                                    effect_strs.push("Slide");
                                }
                                if effects.bend.is_some() {
                                    effect_strs.push("Bend");
                                }
                                if effects.harmonic.is_some() {
                                    effect_strs.push("Harmonic");
                                }

                                if !effect_strs.is_empty() {
                                    ui.horizontal(|ui| {
                                        ui.add_space(16.0);
                                        ui.label(
                                            egui::RichText::new(effect_strs.join(", "))
                                                .small()
                                                .color(self.theme.palette.accent),
                                        );
                                    });
                                }
                            }
                        }
                    });
                }
            }
        }
    }
}
