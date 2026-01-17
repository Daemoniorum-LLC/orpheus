//! Practice mode view - learning and practice tools

use egui::{Color32, Pos2, Rect, Rounding, Stroke, Ui, ScrollArea, Vec2};
use crate::theme::Theme;
use crate::widgets::{Knob, TablatureView, TabState};
use orpheus_file::guitar_pro::GuitarProFile;

/// Status bar height
const STATUS_BAR_HEIGHT: f32 = 24.0;

/// Beat indicator size
const BEAT_INDICATOR_SIZE: f32 = 20.0;

/// Actions from the practice view that need to be handled by the main app
#[derive(Debug, Clone)]
pub enum PracticeAction {
    /// Start playback at effective tempo
    Play {
        tempo: u16,
        loop_start: Option<u64>,
        loop_end: Option<u64>,
    },
    /// Stop playback
    Stop,
    /// Pause playback
    Pause,
    /// Enable/disable metronome
    SetMetronome(bool),
    /// Set metronome volume
    SetMetronomeVolume(f32),
    /// Seek to specific measure
    SeekToMeasure(u16),
}

/// Practice session state
#[derive(Clone)]
pub struct PracticeSession {
    /// Current tempo (percentage of original)
    pub tempo_percent: f32,
    /// Original tempo from file
    pub original_tempo: u16,
    /// Is metronome enabled
    pub metronome_enabled: bool,
    /// Metronome volume (0.0 - 1.0)
    pub metronome_volume: f32,
    /// Count-in bars before playback
    pub count_in_bars: u8,
    /// Loop enabled
    pub loop_enabled: bool,
    /// Loop start measure (1-indexed)
    pub loop_start: u16,
    /// Loop end measure (1-indexed)
    pub loop_end: u16,
    /// Current playback position (measure)
    pub current_measure: u16,
    /// Current beat within measure
    pub current_beat: u8,
    /// Is playing
    pub is_playing: bool,
    /// Practice statistics
    pub stats: PracticeStats,
    /// Time since last beat (for visual pulse)
    pub beat_phase: f32,
    /// Time since last measure start
    pub measure_phase: f32,
    /// Count-in remaining beats (0 = not in count-in)
    pub count_in_remaining: u8,
    /// Is in count-in phase
    pub in_count_in: bool,
    /// Time signature numerator (beats per measure)
    pub time_sig_num: u8,
}

impl Default for PracticeSession {
    fn default() -> Self {
        Self {
            tempo_percent: 100.0,
            original_tempo: 120,
            metronome_enabled: true,
            metronome_volume: 0.7,
            count_in_bars: 1,
            loop_enabled: false,
            loop_start: 1,
            loop_end: 4,
            current_measure: 1,
            current_beat: 1,
            is_playing: false,
            stats: PracticeStats::default(),
            beat_phase: 0.0,
            measure_phase: 0.0,
            count_in_remaining: 0,
            in_count_in: false,
            time_sig_num: 4,
        }
    }
}

impl PracticeSession {
    /// Get effective tempo (original * percentage)
    pub fn effective_tempo(&self) -> u16 {
        ((self.original_tempo as f32 * self.tempo_percent / 100.0) as u16).max(20).min(300)
    }

    /// Get beat duration in seconds
    pub fn beat_duration(&self) -> f32 {
        60.0 / self.effective_tempo() as f32
    }

    /// Reset to beginning
    pub fn reset(&mut self) {
        self.current_measure = if self.loop_enabled { self.loop_start } else { 1 };
        self.current_beat = 1;
        self.beat_phase = 0.0;
        self.measure_phase = 0.0;
        self.in_count_in = false;
        self.count_in_remaining = 0;
    }

    /// Start playback with optional count-in
    pub fn start_play(&mut self) {
        self.is_playing = true;
        self.stats.practice_count += 1;

        if self.count_in_bars > 0 {
            self.in_count_in = true;
            self.count_in_remaining = self.count_in_bars * self.time_sig_num;
        } else {
            self.in_count_in = false;
            self.count_in_remaining = 0;
        }

        self.beat_phase = 0.0;
        self.measure_phase = 0.0;
    }

    /// Stop playback
    pub fn stop_play(&mut self) {
        self.is_playing = false;
        self.in_count_in = false;
        self.count_in_remaining = 0;
        self.beat_phase = 0.0;
    }

    /// Toggle playback
    pub fn toggle_play(&mut self) {
        if self.is_playing {
            self.stop_play();
        } else {
            self.start_play();
        }
    }

    /// Go to next measure
    pub fn next_measure(&mut self, total_measures: u16) {
        if self.loop_enabled && self.current_measure >= self.loop_end {
            self.current_measure = self.loop_start;
        } else if self.current_measure < total_measures {
            self.current_measure += 1;
        }
        self.current_beat = 1;
        self.beat_phase = 0.0;
        self.measure_phase = 0.0;
    }

    /// Go to previous measure
    pub fn prev_measure(&mut self) {
        if self.loop_enabled && self.current_measure <= self.loop_start {
            self.current_measure = self.loop_end;
        } else if self.current_measure > 1 {
            self.current_measure -= 1;
        }
        self.current_beat = 1;
        self.beat_phase = 0.0;
        self.measure_phase = 0.0;
    }

    /// Increase tempo
    pub fn tempo_up(&mut self, amount: f32) {
        self.tempo_percent = (self.tempo_percent + amount).min(150.0);
    }

    /// Decrease tempo
    pub fn tempo_down(&mut self, amount: f32) {
        self.tempo_percent = (self.tempo_percent - amount).max(25.0);
    }

    /// Get progress within current beat (0.0 to 1.0)
    pub fn beat_progress(&self) -> f32 {
        self.beat_phase / self.beat_duration()
    }

    /// Get progress within current measure (0.0 to 1.0)
    pub fn measure_progress(&self) -> f32 {
        let measure_duration = self.beat_duration() * self.time_sig_num as f32;
        self.measure_phase / measure_duration
    }
}

/// Practice statistics
#[derive(Clone, Default)]
pub struct PracticeStats {
    /// Number of times practiced
    pub practice_count: u32,
    /// Total practice time in seconds
    pub total_time_secs: u32,
    /// Current session time in seconds
    pub session_time_secs: u32,
}

/// State for the practice view
pub struct PracticeViewState {
    /// Loaded Guitar Pro file
    pub gp_file: Option<GuitarProFile>,
    /// Practice session
    pub session: PracticeSession,
    /// Tablature view state
    pub tab_state: TabState,
    /// Selected track index
    pub selected_track: usize,
    /// Show settings panel
    pub show_settings: bool,
    /// File path
    pub file_path: Option<String>,
    /// Pending actions for the main app to handle
    pending_actions: Vec<PracticeAction>,
}

impl Default for PracticeViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl PracticeViewState {
    pub fn new() -> Self {
        Self {
            gp_file: None,
            session: PracticeSession::default(),
            tab_state: TabState::new(),
            selected_track: 0,
            show_settings: true,
            file_path: None,
            pending_actions: Vec::new(),
        }
    }

    /// Queue an action for the main app to handle
    fn queue_action(&mut self, action: PracticeAction) {
        self.pending_actions.push(action);
    }

    /// Drain pending actions for the main app to process
    pub fn drain_actions(&mut self) -> Vec<PracticeAction> {
        std::mem::take(&mut self.pending_actions)
    }

    /// Load a Guitar Pro file
    pub fn load_file(&mut self, path: &str) {
        match orpheus_file::guitar_pro::parse_file(path) {
            Ok(file) => {
                self.session.original_tempo = file.tempo;
                self.session.loop_end = file.measures.len() as u16;
                self.gp_file = Some(file);
                self.file_path = Some(path.to_string());
                self.session.reset();
                self.tab_state = TabState::new();
            }
            Err(_) => {
                self.gp_file = None;
            }
        }
    }

    /// Update playback position (called each frame when playing)
    pub fn update_playback(&mut self, delta_time: f32) {
        if !self.session.is_playing {
            return;
        }

        let Some(file) = &self.gp_file else { return };

        // Calculate beat duration based on tempo
        let beat_duration = self.session.beat_duration();

        // Update timing statistics
        self.session.stats.session_time_secs = (self.session.stats.session_time_secs as f32 + delta_time) as u32;

        // Get time signature for current measure
        let time_sig = file.time_signature;
        let beats_per_measure = time_sig.numerator;
        self.session.time_sig_num = beats_per_measure;

        // Accumulate time since last beat
        self.session.beat_phase += delta_time;
        self.session.measure_phase += delta_time;

        // Check if we've completed a beat
        while self.session.beat_phase >= beat_duration {
            self.session.beat_phase -= beat_duration;

            // Handle count-in
            if self.session.in_count_in {
                self.session.count_in_remaining = self.session.count_in_remaining.saturating_sub(1);
                if self.session.count_in_remaining == 0 {
                    self.session.in_count_in = false;
                    self.session.current_beat = 1;
                    self.session.measure_phase = 0.0;
                }
                continue;
            }

            // Advance beat
            self.session.current_beat += 1;
            if self.session.current_beat > beats_per_measure {
                self.session.current_beat = 1;
                self.session.current_measure += 1;
                self.session.measure_phase = 0.0;

                // Handle looping
                if self.session.loop_enabled {
                    if self.session.current_measure > self.session.loop_end {
                        self.session.current_measure = self.session.loop_start;
                    }
                } else if self.session.current_measure > file.measures.len() as u16 {
                    self.session.current_measure = 1;
                }
            }
        }

        // Update tab state selection to follow playback
        if !self.session.in_count_in {
            self.tab_state.selected_measure = Some((self.session.current_measure - 1) as usize);
        }
    }

    /// Get total measures in the loaded file
    pub fn total_measures(&self) -> u16 {
        self.gp_file.as_ref().map(|f| f.measures.len() as u16).unwrap_or(1)
    }
}

/// Practice mode view component
pub struct PracticeView<'a> {
    state: &'a mut PracticeViewState,
    theme: &'a Theme,
}

impl<'a> PracticeView<'a> {
    pub fn new(state: &'a mut PracticeViewState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        // Handle file drops
        self.handle_file_drop(ui);

        // Handle keyboard shortcuts
        self.handle_keyboard(ui);

        let available = ui.available_rect_before_wrap();

        // Status bar at bottom
        let status_rect = Rect::from_min_size(
            Pos2::new(available.min.x, available.max.y - STATUS_BAR_HEIGHT),
            Vec2::new(available.width(), STATUS_BAR_HEIGHT),
        );

        // Main layout
        ui.vertical(|ui| {
            // Transport and tempo controls
            self.show_transport_bar(ui);

            ui.separator();

            // Beat indicator when playing
            if self.state.session.is_playing || self.state.session.in_count_in {
                self.show_beat_indicator(ui);
                ui.separator();
            }

            // Main content area (leave room for status bar)
            let content_height = ui.available_height() - STATUS_BAR_HEIGHT - 8.0;
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), content_height),
                egui::Layout::left_to_right(egui::Align::TOP),
                |ui| {
                    // Settings panel (collapsible)
                    if self.state.show_settings {
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(220.0, ui.available_height()),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                self.show_settings_panel(ui);
                            },
                        );
                        ui.separator();
                    }

                    // Tab view
                    ui.vertical(|ui| {
                        self.show_tab_area(ui);
                    });
                }
            );
        });

        // Draw status bar
        self.draw_status_bar(ui, status_rect);
    }

    /// Handle keyboard shortcuts
    fn handle_keyboard(&mut self, ui: &mut Ui) {
        ui.input(|i| {
            // Space: play/stop
            if i.key_pressed(egui::Key::Space) {
                self.state.session.toggle_play();

                // Queue action for audio engine
                if self.state.session.is_playing {
                    let loop_region = if self.state.session.loop_enabled {
                        let samples_per_beat = 44100 * 60 / self.state.session.effective_tempo() as u64;
                        let time_sig = self.state.session.time_sig_num as u64;
                        let samples_per_measure = samples_per_beat * time_sig;
                        Some((
                            (self.state.session.loop_start as u64 - 1) * samples_per_measure,
                            self.state.session.loop_end as u64 * samples_per_measure,
                        ))
                    } else {
                        None
                    };

                    self.state.queue_action(PracticeAction::Play {
                        tempo: self.state.session.effective_tempo(),
                        loop_start: loop_region.map(|(s, _)| s),
                        loop_end: loop_region.map(|(_, e)| e),
                    });
                } else {
                    self.state.queue_action(PracticeAction::Stop);
                }
            }

            // Escape: stop
            if i.key_pressed(egui::Key::Escape) {
                if self.state.session.is_playing {
                    self.state.session.stop_play();
                    self.state.queue_action(PracticeAction::Stop);
                }
            }

            // Enter: reset to start
            if i.key_pressed(egui::Key::Enter) {
                self.state.session.reset();
                self.state.queue_action(PracticeAction::SeekToMeasure(
                    if self.state.session.loop_enabled {
                        self.state.session.loop_start
                    } else {
                        1
                    }
                ));
            }

            // Left/Right: navigate measures
            if i.key_pressed(egui::Key::ArrowLeft) {
                self.state.session.prev_measure();
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                let total = self.state.total_measures();
                self.state.session.next_measure(total);
            }

            // Up/Down or +/-: adjust tempo
            if i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                let amount = if i.modifiers.shift { 10.0 } else { 5.0 };
                self.state.session.tempo_up(amount);
            }
            if i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::Minus) {
                let amount = if i.modifiers.shift { 10.0 } else { 5.0 };
                self.state.session.tempo_down(amount);
            }

            // M: toggle metronome
            if i.key_pressed(egui::Key::M) && !i.modifiers.ctrl {
                self.state.session.metronome_enabled = !self.state.session.metronome_enabled;
                self.state.queue_action(PracticeAction::SetMetronome(self.state.session.metronome_enabled));
            }

            // L: toggle loop
            if i.key_pressed(egui::Key::L) && !i.modifiers.ctrl {
                self.state.session.loop_enabled = !self.state.session.loop_enabled;
            }

            // Number keys 1-9: quick tempo presets (50%, 60%, ... 130%)
            for (key, percent) in [
                (egui::Key::Num1, 50.0),
                (egui::Key::Num2, 60.0),
                (egui::Key::Num3, 70.0),
                (egui::Key::Num4, 80.0),
                (egui::Key::Num5, 90.0),
                (egui::Key::Num6, 100.0),
                (egui::Key::Num7, 110.0),
                (egui::Key::Num8, 120.0),
                (egui::Key::Num9, 130.0),
            ] {
                if i.key_pressed(key) {
                    self.state.session.tempo_percent = percent;
                }
            }

            // Home: go to first measure
            if i.key_pressed(egui::Key::Home) {
                self.state.session.current_measure = if self.state.session.loop_enabled {
                    self.state.session.loop_start
                } else {
                    1
                };
                self.state.session.current_beat = 1;
            }

            // End: go to last measure
            if i.key_pressed(egui::Key::End) {
                let total = self.state.total_measures();
                self.state.session.current_measure = if self.state.session.loop_enabled {
                    self.state.session.loop_end
                } else {
                    total
                };
                self.state.session.current_beat = 1;
            }
        });
    }

    /// Show beat indicator with visual pulse
    fn show_beat_indicator(&self, ui: &mut Ui) {
        let session = &self.state.session;
        let time_sig = session.time_sig_num;

        // Calculate beat pulse (fades from 1.0 to 0.0 over beat duration)
        let pulse = 1.0 - session.beat_progress().min(1.0);

        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // Count-in display
            if session.in_count_in {
                let remaining_bars = session.count_in_remaining / time_sig.max(1);
                let remaining_beat = session.count_in_remaining % time_sig.max(1);

                ui.label(
                    egui::RichText::new(format!("Count-in: {} bar{}, beat {}",
                        remaining_bars + 1,
                        if remaining_bars == 0 { "" } else { "s" },
                        time_sig - remaining_beat
                    ))
                    .size(16.0)
                    .color(Color32::from_rgb(241, 196, 15))
                );
            } else {
                // Regular beat display
                ui.label(
                    egui::RichText::new(format!("Beat {} of {}", session.current_beat, time_sig))
                        .size(14.0)
                        .color(self.theme.text_secondary())
                );
            }

            ui.add_space(16.0);

            // Beat dots
            for i in 1..=time_sig {
                let is_current = i == session.current_beat && !session.in_count_in;
                let is_downbeat = i == 1;

                let (rect, _) = ui.allocate_exact_size(
                    Vec2::splat(BEAT_INDICATOR_SIZE),
                    egui::Sense::hover(),
                );

                let color = if is_current {
                    // Pulsing color for current beat
                    let alpha = (pulse * 0.5 + 0.5) * 255.0;
                    if is_downbeat {
                        Color32::from_rgba_unmultiplied(231, 76, 60, alpha as u8) // Red for downbeat
                    } else {
                        Color32::from_rgba_unmultiplied(46, 204, 113, alpha as u8) // Green for other beats
                    }
                } else if is_downbeat {
                    self.theme.text_secondary()
                } else {
                    self.theme.text_secondary().gamma_multiply(0.5)
                };

                let size = if is_current {
                    BEAT_INDICATOR_SIZE * (0.8 + pulse * 0.4)
                } else {
                    BEAT_INDICATOR_SIZE * 0.6
                };

                ui.painter().circle_filled(
                    rect.center(),
                    size / 2.0,
                    color,
                );

                // Beat number
                if is_downbeat || is_current {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        i.to_string(),
                        egui::FontId::proportional(10.0),
                        if is_current { Color32::WHITE } else { self.theme.text_primary() },
                    );
                }
            }

            // Measure progress bar
            ui.add_space(16.0);
            let progress = session.measure_progress();
            let progress_width = 100.0;
            let (progress_rect, _) = ui.allocate_exact_size(
                Vec2::new(progress_width, 8.0),
                egui::Sense::hover(),
            );

            ui.painter().rect_filled(
                progress_rect,
                Rounding::same(4.0),
                self.theme.panel_bg(),
            );

            let filled_width = progress_width * progress.min(1.0);
            let filled_rect = Rect::from_min_size(
                progress_rect.min,
                Vec2::new(filled_width, progress_rect.height()),
            );
            ui.painter().rect_filled(
                filled_rect,
                Rounding::same(4.0),
                if session.in_count_in {
                    Color32::from_rgb(241, 196, 15)
                } else {
                    self.theme.accent()
                },
            );
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

        // Position
        status_ui.label(
            egui::RichText::new(format!(
                "Measure {}/{} | Beat {}/{}",
                self.state.session.current_measure,
                self.state.total_measures(),
                self.state.session.current_beat,
                self.state.session.time_sig_num
            ))
            .color(self.theme.text_primary())
            .size(11.0)
        );

        status_ui.separator();

        // Tempo
        status_ui.label(
            egui::RichText::new(format!(
                "{:.0}% = {} BPM",
                self.state.session.tempo_percent,
                self.state.session.effective_tempo()
            ))
            .color(self.theme.accent())
            .size(11.0)
        );

        status_ui.separator();

        // Loop status
        if self.state.session.loop_enabled {
            status_ui.label(
                egui::RichText::new(format!(
                    "Loop: {}-{}",
                    self.state.session.loop_start,
                    self.state.session.loop_end
                ))
                .color(Color32::from_rgb(155, 89, 182))
                .size(11.0)
            );
            status_ui.separator();
        }

        // Metronome
        let metro_text = if self.state.session.metronome_enabled { "Metro: ON" } else { "Metro: OFF" };
        let metro_color = if self.state.session.metronome_enabled {
            self.theme.accent()
        } else {
            self.theme.text_secondary()
        };
        status_ui.label(
            egui::RichText::new(metro_text)
                .color(metro_color)
                .size(11.0)
        );

        // Right side: status and shortcuts
        status_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Playback status
            let status = if self.state.session.in_count_in {
                ("Count-in...", Color32::from_rgb(241, 196, 15))
            } else if self.state.session.is_playing {
                ("Playing", Color32::from_rgb(46, 204, 113))
            } else {
                ("Stopped", self.theme.text_secondary())
            };
            ui.label(egui::RichText::new(status.0).color(status.1).size(11.0));

            ui.separator();

            // Practice time
            let secs = self.state.session.stats.session_time_secs;
            ui.label(
                egui::RichText::new(format!("{}:{:02}", secs / 60, secs % 60))
                    .color(self.theme.text_secondary())
                    .size(11.0)
            );

            ui.separator();

            // Keyboard hints
            ui.label(
                egui::RichText::new("Space: Play | Arrows: Nav/Tempo | M: Metro | L: Loop")
                    .color(self.theme.text_secondary().gamma_multiply(0.7))
                    .size(10.0)
            );
        });
    }

    fn show_transport_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Play/Stop button - now queues actions for the audio engine
            let play_text = if self.state.session.is_playing { "Stop" } else { "Play" };
            let play_color = if self.state.session.is_playing {
                self.theme.palette.error
            } else {
                self.theme.palette.success
            };

            if ui.add(egui::Button::new(
                egui::RichText::new(play_text).color(play_color).size(14.0)
            ).min_size(egui::Vec2::new(60.0, 30.0))).clicked() {
                self.state.session.toggle_play();

                // Queue action for audio engine
                if self.state.session.is_playing {
                    let loop_region = if self.state.session.loop_enabled {
                        // Convert measures to sample positions (simplified)
                        let samples_per_beat = 44100 * 60 / self.state.session.effective_tempo() as u64;
                        let time_sig = self.state.gp_file.as_ref()
                            .map(|f| f.time_signature.numerator)
                            .unwrap_or(4);
                        let samples_per_measure = samples_per_beat * time_sig as u64;
                        Some((
                            (self.state.session.loop_start as u64 - 1) * samples_per_measure,
                            self.state.session.loop_end as u64 * samples_per_measure,
                        ))
                    } else {
                        None
                    };

                    self.state.queue_action(PracticeAction::Play {
                        tempo: self.state.session.effective_tempo(),
                        loop_start: loop_region.map(|(s, _)| s),
                        loop_end: loop_region.map(|(_, e)| e),
                    });
                } else {
                    self.state.queue_action(PracticeAction::Stop);
                }
            }

            // Reset button
            if ui.button("Reset").on_hover_text("Return to start (Enter)").clicked() {
                self.state.session.reset();
                self.state.queue_action(PracticeAction::SeekToMeasure(
                    if self.state.session.loop_enabled {
                        self.state.session.loop_start
                    } else {
                        1
                    }
                ));
            }

            ui.separator();

            // Position display
            if let Some(file) = &self.state.gp_file {
                ui.label(format!(
                    "Measure {}/{} - Beat {}",
                    self.state.session.current_measure,
                    file.measures.len(),
                    self.state.session.current_beat
                ));
            }

            ui.separator();

            // Tempo control
            ui.label("Tempo:");
            let tempo_slider = egui::Slider::new(&mut self.state.session.tempo_percent, 25.0..=150.0)
                .suffix("%")
                .show_value(true);
            if ui.add(tempo_slider)
                .on_hover_text("Playback speed as percentage of original tempo")
                .changed() && self.state.session.is_playing
            {
                // Tempo changed during playback - update audio engine
                self.state.queue_action(PracticeAction::Play {
                    tempo: self.state.session.effective_tempo(),
                    loop_start: None,
                    loop_end: None,
                });
            }

            ui.label(format!("= {} BPM", self.state.session.effective_tempo()));

            // Preset tempo buttons
            ui.separator();
            for &preset in &[50, 75, 100] {
                if ui.small_button(format!("{}%", preset)).clicked() {
                    self.state.session.tempo_percent = preset as f32;
                    if self.state.session.is_playing {
                        self.state.queue_action(PracticeAction::Play {
                            tempo: self.state.session.effective_tempo(),
                            loop_start: None,
                            loop_end: None,
                        });
                    }
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Toggle settings
                ui.toggle_value(&mut self.state.show_settings, "Settings")
                    .on_hover_text("Show/hide practice settings panel");

                // Metronome toggle
                let metro_color = if self.state.session.metronome_enabled {
                    self.theme.palette.accent
                } else {
                    self.theme.palette.text_secondary
                };
                if ui.add(egui::Button::new(
                    egui::RichText::new("Metro").color(metro_color)
                )).on_hover_text("Toggle metronome (M)").clicked() {
                    self.state.session.metronome_enabled = !self.state.session.metronome_enabled;
                    self.state.queue_action(PracticeAction::SetMetronome(self.state.session.metronome_enabled));
                }
            });
        });
    }

    fn show_settings_panel(&mut self, ui: &mut Ui) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.heading("Practice Settings");
                ui.separator();

                // Song info
                if let Some(file) = &self.state.gp_file {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Song Info").strong());
                        ui.label(&file.info.title);
                        ui.label(&file.info.artist);
                        ui.label(format!("Original Tempo: {} BPM", file.tempo));
                        ui.label(format!(
                            "Time Signature: {}/{}",
                            file.time_signature.numerator,
                            file.time_signature.denominator
                        ));
                    });

                    ui.add_space(8.0);

                    // Track selection
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Track").strong());
                        for (idx, track) in file.tracks.iter().enumerate() {
                            let selected = self.state.selected_track == idx;
                            if ui.selectable_label(selected, &track.name).clicked() {
                                self.state.selected_track = idx;
                                self.state.tab_state.selected_track = idx;
                            }
                        }
                    });

                    ui.add_space(8.0);
                }

                // Loop settings
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Loop").strong());

                    ui.checkbox(&mut self.state.session.loop_enabled, "Enable Loop");

                    if self.state.session.loop_enabled {
                        ui.horizontal(|ui| {
                            ui.label("Start:");
                            ui.add(egui::DragValue::new(&mut self.state.session.loop_start)
                                .range(1..=self.state.session.loop_end))
                                .on_hover_text("Loop start measure");
                        });

                        ui.horizontal(|ui| {
                            ui.label("End:");
                            let max = self.state.gp_file.as_ref()
                                .map(|f| f.measures.len() as u16)
                                .unwrap_or(100);
                            ui.add(egui::DragValue::new(&mut self.state.session.loop_end)
                                .range(self.state.session.loop_start..=max))
                                .on_hover_text("Loop end measure");
                        });

                        // Quick loop buttons for sections
                        if let Some(file) = &self.state.gp_file {
                            let markers: Vec<_> = file.measures.iter()
                                .filter(|m| m.marker.is_some())
                                .collect();

                            if !markers.is_empty() {
                                ui.separator();
                                ui.label("Quick Loop:");
                                for m in markers.iter().take(8) {
                                    let marker = m.marker.as_ref().unwrap();
                                    if ui.small_button(marker).clicked() {
                                        self.state.session.loop_start = m.number;
                                        // Find next marker or end
                                        let next = markers.iter()
                                            .find(|n| n.number > m.number)
                                            .map(|n| n.number - 1)
                                            .unwrap_or(file.measures.len() as u16);
                                        self.state.session.loop_end = next;
                                    }
                                }
                            }
                        }
                    }
                });

                ui.add_space(8.0);

                // Metronome settings
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Metronome").strong());

                    if ui.checkbox(&mut self.state.session.metronome_enabled, "Enable").changed() {
                        self.state.queue_action(PracticeAction::SetMetronome(self.state.session.metronome_enabled));
                    }

                    if self.state.session.metronome_enabled {
                        ui.horizontal(|ui| {
                            ui.label("Volume:");
                            let old_vol = self.state.session.metronome_volume;
                            Knob::new(&mut self.state.session.metronome_volume)
                                .range(0.0, 1.0)
                                .default_value(0.7)
                                .size(35.0)
                                .show(ui);
                            if (self.state.session.metronome_volume - old_vol).abs() > 0.01 {
                                self.state.queue_action(PracticeAction::SetMetronomeVolume(self.state.session.metronome_volume));
                            }
                        });
                    }

                    ui.horizontal(|ui| {
                        ui.label("Count-in:");
                        ui.add(egui::DragValue::new(&mut self.state.session.count_in_bars)
                            .range(0..=4)
                            .suffix(" bars"));
                    });
                });

                ui.add_space(8.0);

                // Practice statistics
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Statistics").strong());
                    ui.label(format!("Practice count: {}", self.state.session.stats.practice_count));
                    ui.label(format!(
                        "Session time: {}:{:02}",
                        self.state.session.stats.session_time_secs / 60,
                        self.state.session.stats.session_time_secs % 60
                    ));
                    ui.label(format!(
                        "Total time: {}:{:02}",
                        self.state.session.stats.total_time_secs / 60,
                        self.state.session.stats.total_time_secs % 60
                    ));
                });
            });
    }

    fn show_tab_area(&mut self, ui: &mut Ui) {
        if let Some(file) = &self.state.gp_file {
            // Highlight current measure if playing
            if self.state.session.is_playing {
                self.state.tab_state.selected_measure =
                    Some((self.state.session.current_measure - 1) as usize);
            }

            ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    TablatureView::new(&mut self.state.tab_state)
                        .with_file(file)
                        .show(ui);
                });
        } else {
            // Empty state
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Practice Mode");
                    ui.add_space(20.0);
                    ui.label("Load a Guitar Pro file to start practicing");
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("Drag and drop .gp3, .gp4, or .gp5 files")
                            .color(self.theme.palette.text_secondary)
                    );
                    ui.add_space(30.0);

                    // Features list
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Features:").strong());
                        ui.label("- Adjustable tempo (25% - 150%)");
                        ui.label("- Loop sections for focused practice");
                        ui.label("- Metronome with count-in");
                        ui.label("- Track selection");
                        ui.label("- Practice statistics");
                    });
                });
            });
        }
    }

    fn handle_file_drop(&mut self, ui: &mut Ui) {
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

                        if matches!(ext.as_str(), "gp" | "gp3" | "gp4" | "gp5") {
                            self.state.load_file(&path_str);
                            return;
                        }
                    }
                }
            }
        });
    }
}
