//! Mix mode view - mixing console

use egui::{Color32, Pos2, Rect, Rounding, Stroke, Ui, ScrollArea, Vec2};
use crate::theme::Theme;
use crate::widgets::{LevelMeter, Knob};

/// Status bar height
const STATUS_BAR_HEIGHT: f32 = 24.0;

/// Channel strip state
#[derive(Clone)]
pub struct ChannelStrip {
    /// Channel name
    pub name: String,
    /// Channel color
    pub color: Color32,
    /// Fader level in dB (-inf to +6)
    pub fader_db: f32,
    /// Pan position (-1.0 to 1.0)
    pub pan: f32,
    /// Solo state
    pub solo: bool,
    /// Mute state
    pub mute: bool,
    /// Record arm state
    pub record_arm: bool,
    /// Current level (for meter)
    pub level_db: f32,
    /// Peak level (for meter)
    pub peak_db: f32,
    /// Input gain in dB
    pub input_gain: f32,
    /// Insert effect slots (name or empty)
    pub inserts: Vec<Option<String>>,
    /// Send levels (send name, level)
    pub sends: Vec<(String, f32)>,
}

impl Default for ChannelStrip {
    fn default() -> Self {
        Self {
            name: "Track".to_string(),
            color: Color32::from_rgb(100, 100, 120),
            fader_db: 0.0,
            pan: 0.0,
            solo: false,
            mute: false,
            record_arm: false,
            level_db: -60.0,
            peak_db: -60.0,
            input_gain: 0.0,
            inserts: vec![None; 4],  // 4 insert slots
            sends: vec![
                ("Reverb".to_string(), -12.0),
                ("Delay".to_string(), -24.0),
            ],
        }
    }
}

impl ChannelStrip {
    pub fn new(name: impl Into<String>, color: Color32) -> Self {
        Self {
            name: name.into(),
            color,
            ..Default::default()
        }
    }
}

/// State for the mix view
pub struct MixViewState {
    /// Channel strips
    pub channels: Vec<ChannelStrip>,
    /// Master channel
    pub master: ChannelStrip,
    /// Selected channel index
    pub selected_channel: Option<usize>,
    /// Show insert rack
    pub show_inserts: bool,
    /// Show sends
    pub show_sends: bool,
    /// Show EQ
    pub show_eq: bool,
}

impl Default for MixViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl MixViewState {
    pub fn new() -> Self {
        // Start with a basic master channel - tracks will be synced from tab editor
        let mut master = ChannelStrip::new("Master", Color32::from_rgb(26, 123, 93));
        master.inserts = vec![
            Some("Limiter".to_string()),
            Some("EQ".to_string()),
            None,
            None,
        ];

        Self {
            channels: vec![], // Will be populated from tab editor tracks
            master,
            selected_channel: None,
            show_inserts: true,
            show_sends: true,
            show_eq: false,
        }
    }

    /// Sync mixer channels with tab document tracks
    /// Colors are based on track index for visual consistency
    pub fn sync_from_tracks(&mut self, track_names: &[String]) {
        // Track colors palette
        let colors = [
            Color32::from_rgb(52, 152, 219),   // Blue
            Color32::from_rgb(46, 204, 113),   // Green
            Color32::from_rgb(155, 89, 182),   // Purple
            Color32::from_rgb(231, 76, 60),    // Red
            Color32::from_rgb(241, 196, 15),   // Yellow
            Color32::from_rgb(230, 126, 34),   // Orange
            Color32::from_rgb(26, 188, 156),   // Teal
            Color32::from_rgb(52, 73, 94),     // Dark blue
        ];

        // Only update if track count changed
        if self.channels.len() != track_names.len() {
            self.channels = track_names.iter().enumerate().map(|(i, name)| {
                let color = colors[i % colors.len()];
                ChannelStrip::new(name, color)
            }).collect();

            // Reset selection if it's now out of bounds
            if let Some(sel) = self.selected_channel {
                if sel >= self.channels.len() {
                    self.selected_channel = None;
                }
            }
        } else {
            // Just update names if count is the same
            for (i, name) in track_names.iter().enumerate() {
                if self.channels[i].name != *name {
                    self.channels[i].name = name.clone();
                }
            }
        }
    }

    /// Simulate audio levels (for demo purposes)
    pub fn update_levels(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as f32 / 1000.0)
            .unwrap_or(0.0);

        for (i, channel) in self.channels.iter_mut().enumerate() {
            if !channel.mute {
                // Simulate varying levels
                let base = -20.0 + (i as f32 * 0.5);
                let variation = ((t * (1.0 + i as f32 * 0.1)).sin() * 10.0) +
                               ((t * 2.3).sin() * 5.0);
                channel.level_db = (base + variation + channel.fader_db).clamp(-60.0, 6.0);
                channel.peak_db = channel.peak_db.max(channel.level_db) - 0.1;
            } else {
                channel.level_db = -60.0;
                channel.peak_db = (channel.peak_db - 0.5).max(-60.0);
            }
        }

        // Master level - sum of all channels (simplified)
        let sum = self.channels.iter()
            .filter(|c| !c.mute)
            .map(|c| c.level_db)
            .fold(-60.0_f32, |a, b| a.max(b));
        self.master.level_db = sum + self.master.fader_db;
        self.master.peak_db = self.master.peak_db.max(self.master.level_db) - 0.05;
    }

    /// Check if any channel is soloed
    pub fn any_soloed(&self) -> bool {
        self.channels.iter().any(|c| c.solo)
    }

    /// Select next channel
    pub fn select_next_channel(&mut self) {
        if self.channels.is_empty() {
            return;
        }
        match self.selected_channel {
            Some(idx) if idx < self.channels.len() - 1 => self.selected_channel = Some(idx + 1),
            None => self.selected_channel = Some(0),
            _ => {}
        }
    }

    /// Select previous channel
    pub fn select_prev_channel(&mut self) {
        if self.channels.is_empty() {
            return;
        }
        match self.selected_channel {
            Some(idx) if idx > 0 => self.selected_channel = Some(idx - 1),
            None if !self.channels.is_empty() => self.selected_channel = Some(self.channels.len() - 1),
            _ => {}
        }
    }

    /// Toggle solo on selected channel
    pub fn toggle_selected_solo(&mut self) {
        if let Some(idx) = self.selected_channel {
            if let Some(channel) = self.channels.get_mut(idx) {
                channel.solo = !channel.solo;
            }
        }
    }

    /// Toggle mute on selected channel
    pub fn toggle_selected_mute(&mut self) {
        if let Some(idx) = self.selected_channel {
            if let Some(channel) = self.channels.get_mut(idx) {
                channel.mute = !channel.mute;
            }
        }
    }

    /// Toggle record arm on selected channel
    pub fn toggle_selected_record(&mut self) {
        if let Some(idx) = self.selected_channel {
            if let Some(channel) = self.channels.get_mut(idx) {
                channel.record_arm = !channel.record_arm;
            }
        }
    }

    /// Adjust fader on selected channel
    pub fn adjust_selected_fader(&mut self, delta_db: f32) {
        if let Some(idx) = self.selected_channel {
            if let Some(channel) = self.channels.get_mut(idx) {
                channel.fader_db = (channel.fader_db + delta_db).clamp(-60.0, 6.0);
            }
        }
    }

    /// Reset fader on selected channel to 0 dB
    pub fn reset_selected_fader(&mut self) {
        if let Some(idx) = self.selected_channel {
            if let Some(channel) = self.channels.get_mut(idx) {
                channel.fader_db = 0.0;
            }
        }
    }

    /// Mute all channels
    pub fn mute_all(&mut self) {
        for channel in &mut self.channels {
            channel.mute = true;
        }
    }

    /// Unmute all channels
    pub fn unmute_all(&mut self) {
        for channel in &mut self.channels {
            channel.mute = false;
        }
    }

    /// Clear all solos
    pub fn clear_solos(&mut self) {
        for channel in &mut self.channels {
            channel.solo = false;
        }
    }

    /// Reset all peak meters
    pub fn reset_peaks(&mut self) {
        for channel in &mut self.channels {
            channel.peak_db = -60.0;
        }
        self.master.peak_db = -60.0;
    }
}

/// Mix mode view component
pub struct MixView<'a> {
    state: &'a mut MixViewState,
    theme: &'a Theme,
}

impl<'a> MixView<'a> {
    pub fn new(state: &'a mut MixViewState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        // Update levels for animation
        self.state.update_levels();
        ui.ctx().request_repaint(); // Keep updating

        let available = ui.available_rect_before_wrap();

        // Handle keyboard shortcuts
        self.handle_keyboard(ui);

        let any_soloed = self.state.any_soloed();

        // Toolbar
        ui.horizontal(|ui| {
            ui.heading("Mixer");
            ui.separator();

            ui.toggle_value(&mut self.state.show_inserts, "Inserts")
                .on_hover_text("Show/hide insert effect slots (I)");
            ui.toggle_value(&mut self.state.show_sends, "Sends")
                .on_hover_text("Show/hide aux send controls (E)");
            ui.toggle_value(&mut self.state.show_eq, "EQ")
                .on_hover_text("Show/hide EQ controls (Q)");

            ui.separator();

            // Channel navigation hint
            if let Some(idx) = self.state.selected_channel {
                ui.label(format!("Channel: {} ({}/{})",
                    self.state.channels.get(idx).map(|c| c.name.as_str()).unwrap_or("?"),
                    idx + 1,
                    self.state.channels.len()
                ));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Reset Peaks").on_hover_text("Clear all peak hold indicators (P)").clicked() {
                    self.state.reset_peaks();
                }
                if ui.button("Clear Solos").on_hover_text("Un-solo all channels (Escape)").clicked() {
                    self.state.clear_solos();
                }
            });
        });

        ui.separator();

        // Capture values before the loop to avoid borrow conflicts
        let show_inserts = self.state.show_inserts;
        let show_sends = self.state.show_sends;
        let selected_channel = self.state.selected_channel;
        let theme = self.theme;

        // Calculate status bar position
        let status_rect = Rect::from_min_size(
            Pos2::new(available.min.x, available.max.y - STATUS_BAR_HEIGHT),
            Vec2::new(available.width(), STATUS_BAR_HEIGHT),
        );

        // Main mixer area (leave room for status bar)
        let mixer_height = ui.available_height() - STATUS_BAR_HEIGHT - 4.0;
        let clicked_channel = ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), mixer_height),
            egui::Layout::left_to_right(egui::Align::TOP),
            |ui| {
                ScrollArea::horizontal()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Channel strips - collect clicked indices
                            let mut clicked_idx: Option<usize> = None;
                            for (idx, channel) in self.state.channels.iter_mut().enumerate() {
                                let is_selected = selected_channel == Some(idx);
                                let is_active = !any_soloed || channel.solo;

                                let response = draw_channel_strip(
                                    ui,
                                    channel,
                                    is_selected,
                                    is_active && !channel.mute,
                                    show_inserts,
                                    show_sends,
                                    theme,
                                );

                                // Track clicked channel
                                if response.clicked() {
                                    clicked_idx = Some(idx);
                                }
                            }

                            // Separator before master
                            ui.separator();

                            // Master channel
                            draw_master_strip(ui, &mut self.state.master, theme);

                            clicked_idx
                        }).inner
                    }).inner
            }
        ).inner;

        // Handle channel selection from click
        if let Some(idx) = clicked_channel {
            self.state.selected_channel = Some(idx);
        }

        // Draw status bar
        self.draw_status_bar(ui, status_rect);
    }

    /// Handle keyboard shortcuts
    fn handle_keyboard(&mut self, ui: &mut Ui) {
        ui.input(|i| {
            // Left/Right: navigate channels
            if i.key_pressed(egui::Key::ArrowLeft) {
                self.state.select_prev_channel();
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                self.state.select_next_channel();
            }

            // Up/Down: adjust fader
            if i.key_pressed(egui::Key::ArrowUp) {
                if i.modifiers.shift {
                    self.state.adjust_selected_fader(3.0); // Coarse
                } else {
                    self.state.adjust_selected_fader(0.5); // Fine
                }
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                if i.modifiers.shift {
                    self.state.adjust_selected_fader(-3.0);
                } else {
                    self.state.adjust_selected_fader(-0.5);
                }
            }

            // S: toggle solo
            if i.key_pressed(egui::Key::S) && !i.modifiers.ctrl {
                self.state.toggle_selected_solo();
            }

            // M: toggle mute
            if i.key_pressed(egui::Key::M) && !i.modifiers.ctrl {
                self.state.toggle_selected_mute();
            }

            // R: toggle record arm
            if i.key_pressed(egui::Key::R) && !i.modifiers.ctrl {
                self.state.toggle_selected_record();
            }

            // 0: reset fader to 0 dB
            if i.key_pressed(egui::Key::Num0) {
                self.state.reset_selected_fader();
            }

            // Escape: clear solos
            if i.key_pressed(egui::Key::Escape) {
                self.state.clear_solos();
            }

            // P: reset peaks
            if i.key_pressed(egui::Key::P) && !i.modifiers.ctrl {
                self.state.reset_peaks();
            }

            // I: toggle inserts
            if i.key_pressed(egui::Key::I) && !i.modifiers.ctrl {
                self.state.show_inserts = !self.state.show_inserts;
            }

            // E: toggle sends
            if i.key_pressed(egui::Key::E) && !i.modifiers.ctrl {
                self.state.show_sends = !self.state.show_sends;
            }

            // Q: toggle EQ
            if i.key_pressed(egui::Key::Q) && !i.modifiers.ctrl {
                self.state.show_eq = !self.state.show_eq;
            }

            // Home: select first channel
            if i.key_pressed(egui::Key::Home) {
                if !self.state.channels.is_empty() {
                    self.state.selected_channel = Some(0);
                }
            }

            // End: select last channel
            if i.key_pressed(egui::Key::End) {
                if !self.state.channels.is_empty() {
                    self.state.selected_channel = Some(self.state.channels.len() - 1);
                }
            }
        });
    }

    /// Draw the status bar
    fn draw_status_bar(&self, ui: &mut Ui, rect: Rect) {
        // Background
        ui.painter().rect_filled(rect, Rounding::ZERO, self.theme.surface_bg());

        // Top border
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(1.0, self.theme.border()),
        );

        let status_ui_rect = rect.shrink(4.0);
        let mut status_ui = ui.child_ui(status_ui_rect, egui::Layout::left_to_right(egui::Align::Center), None);

        // Channel count
        let channel_count = self.state.channels.len();
        let soloed_count = self.state.channels.iter().filter(|c| c.solo).count();
        let muted_count = self.state.channels.iter().filter(|c| c.mute).count();
        let armed_count = self.state.channels.iter().filter(|c| c.record_arm).count();

        status_ui.label(
            egui::RichText::new(format!("{} channels", channel_count))
                .color(self.theme.text_primary())
                .size(11.0)
        );

        status_ui.separator();

        // Selected channel info
        if let Some(idx) = self.state.selected_channel {
            if let Some(channel) = self.state.channels.get(idx) {
                status_ui.label(
                    egui::RichText::new(format!("{}: {:+.1} dB", channel.name, channel.fader_db))
                        .color(self.theme.accent())
                        .size(11.0)
                );
                status_ui.separator();
            }
        }

        // Solo/Mute/Arm counts
        if soloed_count > 0 {
            status_ui.label(
                egui::RichText::new(format!("{} soloed", soloed_count))
                    .color(Color32::from_rgb(241, 196, 15))
                    .size(11.0)
            );
            status_ui.separator();
        }

        if muted_count > 0 {
            status_ui.label(
                egui::RichText::new(format!("{} muted", muted_count))
                    .color(Color32::from_rgb(231, 76, 60))
                    .size(11.0)
            );
            status_ui.separator();
        }

        if armed_count > 0 {
            status_ui.label(
                egui::RichText::new(format!("{} armed", armed_count))
                    .color(Color32::from_rgb(231, 76, 60))
                    .size(11.0)
            );
        }

        // Right side: master level and shortcuts
        status_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Master level
            ui.label(
                egui::RichText::new(format!("Master: {:+.1} dB", self.state.master.fader_db))
                    .color(self.theme.text_primary())
                    .size(11.0)
            );

            ui.separator();

            // Keyboard hints
            ui.label(
                egui::RichText::new("Arrows: Nav/Fader | S: Solo | M: Mute | 0: Unity")
                    .color(self.theme.text_secondary().gamma_multiply(0.7))
                    .size(10.0)
            );
        });
    }
}

fn draw_channel_strip(
    ui: &mut Ui,
    channel: &mut ChannelStrip,
    is_selected: bool,
    is_active: bool,
    show_inserts: bool,
    show_sends: bool,
    theme: &Theme,
) -> egui::Response {
    let strip_width = 70.0;

    let bg_color = if is_selected {
        theme.palette.bg_tertiary
    } else {
        theme.palette.bg_secondary
    };

    // Selection highlight for selected channel
    let frame_stroke = if is_selected {
        egui::Stroke::new(2.0, theme.palette.accent)
    } else {
        egui::Stroke::NONE
    };

    let frame_response = egui::Frame::none()
            .fill(bg_color)
            .inner_margin(egui::Margin::symmetric(4.0, 8.0))
            .rounding(4.0)
            .stroke(frame_stroke)
            .show(ui, |ui| {
                ui.set_min_width(strip_width);
                ui.set_max_width(strip_width);

                ui.vertical(|ui| {
                    // Channel name with color indicator
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::Vec2::new(4.0, 16.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(rect, 2.0, channel.color);

                        ui.label(
                            egui::RichText::new(&channel.name)
                                .size(11.0)
                                .color(if is_active {
                                    theme.palette.text_primary
                                } else {
                                    theme.palette.text_secondary
                                }),
                        );
                    });

                    ui.add_space(4.0);

                    // Insert slots (if shown)
                    if show_inserts {
                        ui.group(|ui| {
                            ui.set_min_width(strip_width - 16.0);
                            ui.label(
                                egui::RichText::new("Inserts")
                                    .size(9.0)
                                    .color(theme.palette.text_secondary),
                            );

                            for insert in &channel.inserts {
                                let label = insert.as_deref().unwrap_or("---");
                                let color = if insert.is_some() {
                                    theme.palette.accent
                                } else {
                                    theme.palette.text_secondary
                                };

                                ui.label(
                                    egui::RichText::new(label)
                                        .size(9.0)
                                        .color(color),
                                );
                            }
                        });
                        ui.add_space(4.0);
                    }

                    // Sends (if shown)
                    if show_sends {
                        ui.group(|ui| {
                            ui.set_min_width(strip_width - 16.0);
                            ui.label(
                                egui::RichText::new("Sends")
                                    .size(9.0)
                                    .color(theme.palette.text_secondary),
                            );

                            for (name, level) in &channel.sends {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(name)
                                            .size(9.0)
                                            .color(theme.palette.text_secondary),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("{:.0}", level))
                                            .size(9.0)
                                            .color(theme.palette.accent),
                                    );
                                });
                            }
                        });
                        ui.add_space(4.0);
                    }

                    // Input gain knob
                    ui.horizontal(|ui| {
                        Knob::new(&mut channel.input_gain)
                            .range(-24.0, 24.0)
                            .default_value(0.0)
                            .size(30.0)
                            .label("Gain")
                            .show(ui);
                    });

                    ui.add_space(4.0);

                    // Pan knob
                    ui.horizontal(|ui| {
                        Knob::new(&mut channel.pan)
                            .range(-1.0, 1.0)
                            .default_value(0.0)
                            .size(30.0)
                            .label("Pan")
                            .show(ui);
                    });

                    ui.add_space(8.0);

                    // Meter and fader
                    ui.horizontal(|ui| {
                        // Level meter
                        let meter_level = if is_active { channel.level_db } else { -60.0 };
                        LevelMeter::new(meter_level)
                            .peak(channel.peak_db)
                            .width(12.0)
                            .height(140.0)
                            .show(ui);

                        ui.add_space(4.0);

                        // Fader
                        ui.vertical(|ui| {
                            // dB value
                            ui.label(
                                egui::RichText::new(format!("{:+.1}", channel.fader_db))
                                    .size(9.0)
                                    .color(theme.palette.text_secondary),
                            );

                            // Vertical slider for fader
                            let slider = egui::Slider::new(&mut channel.fader_db, -60.0..=6.0)
                                .vertical()
                                .show_value(false);
                            ui.add_sized([20.0, 140.0], slider)
                                .on_hover_text("Channel volume (drag vertically)");
                        });
                    });

                    ui.add_space(8.0);

                    // Solo/Mute/Record buttons
                    ui.horizontal(|ui| {
                        let solo_color = if channel.solo {
                            Color32::from_rgb(241, 196, 15)  // Yellow
                        } else {
                            theme.palette.text_secondary
                        };
                        if ui.add(egui::Button::new(
                            egui::RichText::new("S").size(11.0).color(solo_color)
                        ).min_size(egui::Vec2::new(18.0, 18.0)))
                            .on_hover_text("Solo - play only this track")
                            .clicked()
                        {
                            channel.solo = !channel.solo;
                        }

                        let mute_color = if channel.mute {
                            Color32::from_rgb(231, 76, 60)  // Red
                        } else {
                            theme.palette.text_secondary
                        };
                        if ui.add(egui::Button::new(
                            egui::RichText::new("M").size(11.0).color(mute_color)
                        ).min_size(egui::Vec2::new(18.0, 18.0)))
                            .on_hover_text("Mute - silence this track")
                            .clicked()
                        {
                            channel.mute = !channel.mute;
                        }

                        let rec_color = if channel.record_arm {
                            Color32::from_rgb(231, 76, 60)  // Red
                        } else {
                            theme.palette.text_secondary
                        };
                        if ui.add(egui::Button::new(
                            egui::RichText::new("R").size(11.0).color(rec_color)
                        ).min_size(egui::Vec2::new(18.0, 18.0)))
                            .on_hover_text("Record arm - enable recording on this track")
                            .clicked()
                        {
                            channel.record_arm = !channel.record_arm;
                        }
                    });
                });
            })
            .response
            .interact(egui::Sense::click());

    frame_response
}

fn draw_master_strip(ui: &mut Ui, master: &mut ChannelStrip, theme: &Theme) {
    let strip_width = 90.0;

    egui::Frame::none()
        .fill(theme.palette.bg_tertiary)
        .inner_margin(egui::Margin::symmetric(6.0, 8.0))
        .rounding(4.0)
        .stroke(egui::Stroke::new(1.0, theme.palette.accent))
        .show(ui, |ui| {
            ui.set_min_width(strip_width);
            ui.set_max_width(strip_width);

            ui.vertical(|ui| {
                // Master label
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(
                        egui::Vec2::new(4.0, 16.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 2.0, theme.palette.accent);

                    ui.label(
                        egui::RichText::new("MASTER")
                            .size(11.0)
                            .strong()
                            .color(theme.palette.text_primary),
                    );
                });

                ui.add_space(4.0);

                // Master inserts
                ui.group(|ui| {
                    ui.set_min_width(strip_width - 20.0);
                    ui.label(
                        egui::RichText::new("Inserts")
                            .size(9.0)
                            .color(theme.palette.text_secondary),
                    );

                    for insert in &master.inserts {
                        let label = insert.as_deref().unwrap_or("---");
                        let color = if insert.is_some() {
                            theme.palette.accent
                        } else {
                            theme.palette.text_secondary
                        };

                        ui.label(
                            egui::RichText::new(label)
                                .size(9.0)
                                .color(color),
                        );
                    }
                });

                ui.add_space(8.0);

                // Stereo meter and fader
                ui.horizontal(|ui| {
                    // Left meter
                    LevelMeter::new(master.level_db - 1.0)
                        .peak(master.peak_db - 1.0)
                        .width(14.0)
                        .height(180.0)
                        .show(ui);

                    // Right meter
                    LevelMeter::new(master.level_db)
                        .peak(master.peak_db)
                        .width(14.0)
                        .height(180.0)
                        .show(ui);

                    ui.add_space(4.0);

                    // Master fader
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{:+.1}", master.fader_db))
                                .size(10.0)
                                .color(theme.palette.text_primary),
                        );

                        let slider = egui::Slider::new(&mut master.fader_db, -60.0..=6.0)
                            .vertical()
                            .show_value(false);
                        ui.add_sized([24.0, 180.0], slider)
                            .on_hover_text("Master volume (drag vertically)");
                    });
                });

                ui.add_space(8.0);

                // Dim/Mono buttons
                ui.horizontal(|ui| {
                    if ui.button(egui::RichText::new("DIM").size(9.0))
                        .on_hover_text("Reduce master volume for reference listening")
                        .clicked()
                    {
                        // Toggle dim
                    }
                    if ui.button(egui::RichText::new("MONO").size(9.0))
                        .on_hover_text("Sum to mono for compatibility checking")
                        .clicked()
                    {
                        // Toggle mono
                    }
                });
            });
        });
}
