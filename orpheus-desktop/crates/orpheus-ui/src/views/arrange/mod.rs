//! Arrange mode view - timeline and clip arrangement
//!
//! The main arrangement view for composing and arranging tracks.

use egui::{Color32, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use uuid::Uuid;

use crate::theme::Theme;

/// Status bar height
const STATUS_BAR_HEIGHT: f32 = 24.0;

/// Pixels per second at zoom level 1.0
const BASE_PIXELS_PER_SECOND: f32 = 100.0;

/// Actions that can be returned from the arrange view
#[derive(Debug, Clone, PartialEq)]
pub enum ArrangeViewAction {
    /// No action
    None,
    /// Request playback start
    Play,
    /// Request playback stop
    Stop,
    /// Request playback pause
    Pause,
    /// Toggle playback (play/pause)
    TogglePlayback,
    /// Toggle recording
    ToggleRecording,
    /// Seek to position (seconds)
    Seek(f64),
    /// Delete selected clips
    DeleteSelection,
    /// Duplicate selected clips
    DuplicateSelection,
}

/// Track header width
const TRACK_HEADER_WIDTH: f32 = 200.0;

/// Track height
const TRACK_HEIGHT: f32 = 80.0;

/// Timeline ruler height
const RULER_HEIGHT: f32 = 30.0;

/// Minimum track height
const MIN_TRACK_HEIGHT: f32 = 40.0;

/// Maximum track height
const MAX_TRACK_HEIGHT: f32 = 200.0;

/// Clip data for display
#[derive(Clone)]
pub struct ClipDisplay {
    /// Clip ID
    pub id: Uuid,
    /// Clip name
    pub name: String,
    /// Start time in seconds
    pub start_secs: f64,
    /// Length in seconds
    pub length_secs: f64,
    /// Clip color
    pub color: Color32,
    /// Is this an audio clip
    pub is_audio: bool,
    /// Waveform peaks (normalized -1 to 1) if audio
    pub waveform: Option<Vec<(f32, f32)>>,
    /// Is selected
    pub selected: bool,
}

/// Track display info
#[derive(Clone)]
pub struct TrackDisplay {
    /// Track ID
    pub id: Uuid,
    /// Track name
    pub name: String,
    /// Track color
    pub color: Color32,
    /// Is muted
    pub muted: bool,
    /// Is soloed
    pub solo: bool,
    /// Is armed for recording
    pub armed: bool,
    /// Current level dB (for meter)
    pub level_db: f32,
    /// Peak level dB
    pub peak_db: f32,
    /// Track height
    pub height: f32,
    /// Clips on this track
    pub clips: Vec<ClipDisplay>,
}

impl Default for TrackDisplay {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Track".to_string(),
            color: Color32::from_rgb(52, 152, 219),
            muted: false,
            solo: false,
            armed: false,
            level_db: -60.0,
            peak_db: -60.0,
            height: TRACK_HEIGHT,
            clips: Vec::new(),
        }
    }
}

impl TrackDisplay {
    pub fn new(name: impl Into<String>, color: Color32) -> Self {
        Self {
            name: name.into(),
            color,
            ..Default::default()
        }
    }
}

/// Selection state
#[derive(Clone, Default)]
pub struct Selection {
    /// Selected track IDs
    pub tracks: Vec<Uuid>,
    /// Selected clip IDs
    pub clips: Vec<Uuid>,
    /// Selection rectangle (for drag selection)
    pub rect: Option<Rect>,
}

/// Snap mode for clip positioning
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SnapMode {
    Off,
    Grid,
    Bar,
    Beat,
    Subdivision,
}

impl Default for SnapMode {
    fn default() -> Self {
        Self::Grid
    }
}

/// State for the arrange view
pub struct ArrangeViewState {
    /// All tracks to display
    pub tracks: Vec<TrackDisplay>,
    /// Horizontal zoom level (1.0 = 100 pixels per second)
    pub zoom_h: f32,
    /// Vertical zoom level (affects track height)
    pub zoom_v: f32,
    /// Horizontal scroll position in seconds
    pub scroll_h: f64,
    /// Vertical scroll position in pixels
    pub scroll_v: f32,
    /// Current playhead position in seconds
    pub playhead_secs: f64,
    /// Tempo BPM
    pub tempo: f64,
    /// Time signature numerator
    pub time_sig_num: u8,
    /// Time signature denominator
    pub time_sig_denom: u8,
    /// Is playing
    pub is_playing: bool,
    /// Is recording
    pub is_recording: bool,
    /// Loop enabled
    pub loop_enabled: bool,
    /// Loop start in seconds
    pub loop_start: f64,
    /// Loop end in seconds
    pub loop_end: f64,
    /// Current selection
    pub selection: Selection,
    /// Snap mode
    pub snap_mode: SnapMode,
    /// Show grid
    pub show_grid: bool,
    /// Project duration in seconds (for scrollbar)
    pub duration_secs: f64,
    /// Dragging state
    drag_state: Option<DragState>,
    /// Currently selected/focused track index
    pub selected_track: Option<usize>,
    /// Show context menu for track
    pub context_menu_track: Option<usize>,
    /// Context menu position
    pub context_menu_pos: Option<Pos2>,
}

/// Drag operation state
#[derive(Clone)]
enum DragState {
    /// Dragging a clip
    MoveClip {
        clip_id: Uuid,
        track_id: Uuid,
        start_pos: f64,
        offset: f64,
    },
    /// Resizing a clip from the right edge
    ResizeClipRight {
        clip_id: Uuid,
        track_id: Uuid,
        original_length: f64,
    },
    /// Resizing a clip from the left edge
    ResizeClipLeft {
        clip_id: Uuid,
        track_id: Uuid,
        original_start: f64,
        original_length: f64,
    },
    /// Selecting with a rectangle
    RectSelect {
        start: Pos2,
    },
    /// Moving the playhead
    MovePlayhead,
}

impl Default for ArrangeViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl ArrangeViewState {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            zoom_h: 1.0,
            zoom_v: 1.0,
            scroll_h: 0.0,
            scroll_v: 0.0,
            playhead_secs: 0.0,
            tempo: 120.0,
            time_sig_num: 4,
            time_sig_denom: 4,
            is_playing: false,
            is_recording: false,
            loop_enabled: false,
            loop_start: 0.0,
            loop_end: 8.0,
            selection: Selection::default(),
            snap_mode: SnapMode::Grid,
            show_grid: true,
            duration_secs: 60.0,
            drag_state: None,
            selected_track: None,
            context_menu_track: None,
            context_menu_pos: None,
        }
    }

    /// Select all clips
    pub fn select_all_clips(&mut self) {
        self.selection.clips.clear();
        for track in &self.tracks {
            for clip in &track.clips {
                self.selection.clips.push(clip.id);
            }
        }
        // Update clip selected flags
        for track in &mut self.tracks {
            for clip in &mut track.clips {
                clip.selected = true;
            }
        }
    }

    /// Deselect all clips
    pub fn deselect_all(&mut self) {
        self.selection.clips.clear();
        self.selection.tracks.clear();
        for track in &mut self.tracks {
            for clip in &mut track.clips {
                clip.selected = false;
            }
        }
    }

    /// Delete selected clips
    pub fn delete_selected_clips(&mut self) {
        for track in &mut self.tracks {
            track.clips.retain(|c| !self.selection.clips.contains(&c.id));
        }
        self.selection.clips.clear();
    }

    /// Move playhead by beats
    pub fn move_playhead_beats(&mut self, beats: i32) {
        let beat_secs = self.beat_duration_secs();
        self.playhead_secs = (self.playhead_secs + beats as f64 * beat_secs).max(0.0);
        // Clamp to duration
        if self.playhead_secs > self.duration_secs {
            self.playhead_secs = self.duration_secs;
        }
    }

    /// Move playhead by bars
    pub fn move_playhead_bars(&mut self, bars: i32) {
        let bar_secs = self.bar_duration_secs();
        self.playhead_secs = (self.playhead_secs + bars as f64 * bar_secs).max(0.0);
        // Clamp to duration
        if self.playhead_secs > self.duration_secs {
            self.playhead_secs = self.duration_secs;
        }
    }

    /// Go to start
    pub fn go_to_start(&mut self) {
        self.playhead_secs = 0.0;
        self.scroll_h = 0.0;
    }

    /// Go to end
    pub fn go_to_end(&mut self) {
        self.playhead_secs = self.duration_secs;
    }

    /// Toggle mute on selected track
    pub fn toggle_selected_track_mute(&mut self) {
        if let Some(idx) = self.selected_track {
            if let Some(track) = self.tracks.get_mut(idx) {
                track.muted = !track.muted;
            }
        }
    }

    /// Toggle solo on selected track
    pub fn toggle_selected_track_solo(&mut self) {
        if let Some(idx) = self.selected_track {
            if let Some(track) = self.tracks.get_mut(idx) {
                track.solo = !track.solo;
            }
        }
    }

    /// Navigate to previous track
    pub fn select_prev_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        match self.selected_track {
            Some(idx) if idx > 0 => self.selected_track = Some(idx - 1),
            None if !self.tracks.is_empty() => self.selected_track = Some(self.tracks.len() - 1),
            _ => {}
        }
    }

    /// Navigate to next track
    pub fn select_next_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        match self.selected_track {
            Some(idx) if idx < self.tracks.len() - 1 => self.selected_track = Some(idx + 1),
            None => self.selected_track = Some(0),
            _ => {}
        }
    }

    /// Get count of selected clips
    pub fn selected_clip_count(&self) -> usize {
        self.selection.clips.len()
    }

    /// Add a demo track with clips (for testing)
    pub fn add_demo_track(&mut self, name: &str, color: Color32) {
        let mut track = TrackDisplay::new(name, color);

        // Add a demo clip
        track.clips.push(ClipDisplay {
            id: Uuid::new_v4(),
            name: format!("{} Clip 1", name),
            start_secs: 0.0,
            length_secs: 4.0,
            color,
            is_audio: true,
            waveform: Some(generate_demo_waveform(400)),
            selected: false,
        });

        self.tracks.push(track);
    }

    /// Sync tracks from tab editor track names
    /// This ensures the arrange view shows the same tracks as the tab editor
    pub fn sync_from_tracks(&mut self, track_names: &[String]) {
        // Track colors for visual distinction
        let colors = [
            Color32::from_rgb(52, 152, 219),  // Blue
            Color32::from_rgb(46, 204, 113),  // Green
            Color32::from_rgb(155, 89, 182),  // Purple
            Color32::from_rgb(231, 76, 60),   // Red
            Color32::from_rgb(241, 196, 15),  // Yellow
            Color32::from_rgb(26, 123, 93),   // Teal
        ];

        // Only update if track count changed
        if self.tracks.len() != track_names.len() {
            self.tracks = track_names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let color = colors[i % colors.len()];
                    TrackDisplay::new(name.clone(), color)
                })
                .collect();
        } else {
            // Update names if they changed
            for (track, name) in self.tracks.iter_mut().zip(track_names.iter()) {
                if track.name != *name {
                    track.name = name.clone();
                }
            }
        }
    }

    /// Seconds to pixels conversion
    pub fn secs_to_pixels(&self, secs: f64) -> f32 {
        (secs as f32) * BASE_PIXELS_PER_SECOND * self.zoom_h
    }

    /// Pixels to seconds conversion
    pub fn pixels_to_secs(&self, pixels: f32) -> f64 {
        (pixels / (BASE_PIXELS_PER_SECOND * self.zoom_h)) as f64
    }

    /// Get beat duration in seconds
    pub fn beat_duration_secs(&self) -> f64 {
        60.0 / self.tempo
    }

    /// Get bar duration in seconds
    pub fn bar_duration_secs(&self) -> f64 {
        self.beat_duration_secs() * self.time_sig_num as f64
    }

    /// Snap time to grid based on snap mode
    pub fn snap_time(&self, time: f64) -> f64 {
        match self.snap_mode {
            SnapMode::Off => time,
            SnapMode::Grid => {
                // Snap to 16th note
                let subdivision = self.beat_duration_secs() / 4.0;
                (time / subdivision).round() * subdivision
            }
            SnapMode::Bar => {
                let bar = self.bar_duration_secs();
                (time / bar).round() * bar
            }
            SnapMode::Beat => {
                let beat = self.beat_duration_secs();
                (time / beat).round() * beat
            }
            SnapMode::Subdivision => {
                let sub = self.beat_duration_secs() / 4.0;
                (time / sub).round() * sub
            }
        }
    }
}

/// Generate demo waveform data
fn generate_demo_waveform(num_points: usize) -> Vec<(f32, f32)> {
    (0..num_points)
        .map(|i| {
            let t = i as f32 / num_points as f32;
            let base = (t * 20.0 * std::f32::consts::PI).sin() * 0.3;
            let env = (t * std::f32::consts::PI).sin();
            let noise = ((i * 7919) % 1000) as f32 / 2000.0 - 0.25;
            let val = (base + noise) * env;
            (val.min(0.0), val.max(0.0))
        })
        .collect()
}

/// Arrange view widget
pub struct ArrangeView;

impl ArrangeView {
    /// Draw the arrange view
    pub fn show(ui: &mut Ui, state: &mut ArrangeViewState, theme: &Theme) -> ArrangeViewAction {
        let available = ui.available_rect_before_wrap();

        // Background
        ui.painter().rect_filled(available, Rounding::ZERO, theme.panel_bg());

        // Draw toolbar
        let toolbar_rect = Rect::from_min_size(
            available.min,
            Vec2::new(available.width(), 32.0),
        );
        Self::draw_toolbar(ui, state, theme, toolbar_rect);

        // Status bar at bottom
        let status_rect = Rect::from_min_size(
            Pos2::new(available.min.x, available.max.y - STATUS_BAR_HEIGHT),
            Vec2::new(available.width(), STATUS_BAR_HEIGHT),
        );

        // Main area below toolbar, above status bar
        let main_rect = Rect::from_min_max(
            Pos2::new(available.min.x, toolbar_rect.max.y),
            Pos2::new(available.max.x, status_rect.min.y),
        );

        // Track header area (left side)
        let header_rect = Rect::from_min_size(
            main_rect.min,
            Vec2::new(TRACK_HEADER_WIDTH, main_rect.height()),
        );

        // Timeline area (right side)
        let timeline_rect = Rect::from_min_max(
            Pos2::new(header_rect.max.x, main_rect.min.y),
            main_rect.max,
        );

        // Ruler at top of timeline
        let ruler_rect = Rect::from_min_size(
            timeline_rect.min,
            Vec2::new(timeline_rect.width(), RULER_HEIGHT),
        );

        // Track lanes below ruler
        let lanes_rect = Rect::from_min_max(
            Pos2::new(timeline_rect.min.x, ruler_rect.max.y),
            timeline_rect.max,
        );

        // Draw components
        Self::draw_track_headers(ui, state, theme, header_rect);
        Self::draw_ruler(ui, state, theme, ruler_rect);
        Self::draw_track_lanes(ui, state, theme, lanes_rect);
        Self::draw_playhead(ui, state, theme, ruler_rect, lanes_rect);
        Self::draw_status_bar(ui, state, theme, status_rect);

        // Handle scrolling
        let scroll_response = ui.allocate_rect(lanes_rect, Sense::click_and_drag());
        if scroll_response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta);
            if scroll.x != 0.0 || scroll.y != 0.0 {
                state.scroll_h -= state.pixels_to_secs(scroll.x);
                state.scroll_h = state.scroll_h.max(0.0);
                state.scroll_v -= scroll.y;
                state.scroll_v = state.scroll_v.max(0.0);
            }

            // Zoom with Ctrl+scroll
            let zoom_delta = ui.input(|i| {
                if i.modifiers.ctrl {
                    i.smooth_scroll_delta.y * 0.01
                } else {
                    0.0
                }
            });
            if zoom_delta != 0.0 {
                state.zoom_h = (state.zoom_h + zoom_delta).clamp(0.1, 10.0);
            }
        }

        // Draw context menu if open
        Self::draw_context_menu(ui, state, theme);

        // Handle keyboard shortcuts
        Self::handle_keyboard(ui, state)
    }

    /// Draw the track context menu
    fn draw_context_menu(ui: &mut Ui, state: &mut ArrangeViewState, theme: &Theme) {
        if state.context_menu_track.is_none() {
            return;
        }

        let track_idx = state.context_menu_track.unwrap();

        // Create menu at stored position
        let menu_id = ui.make_persistent_id("track_context_menu");

        // Use egui Area for the popup
        egui::Area::new(menu_id)
            .fixed_pos(state.context_menu_pos.unwrap_or(Pos2::ZERO))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(theme.surface_bg())
                    .stroke(Stroke::new(1.0, theme.border()))
                    .show(ui, |ui| {
                        ui.set_min_width(150.0);

                        let track_name = state.tracks.get(track_idx)
                            .map(|t| t.name.clone())
                            .unwrap_or_default();

                        // Menu header
                        ui.label(egui::RichText::new(&track_name).strong().size(12.0));
                        ui.separator();

                        // Mute toggle
                        let is_muted = state.tracks.get(track_idx).map(|t| t.muted).unwrap_or(false);
                        let mute_text = if is_muted { "Unmute" } else { "Mute" };
                        if ui.button(mute_text).clicked() {
                            if let Some(track) = state.tracks.get_mut(track_idx) {
                                track.muted = !track.muted;
                            }
                            state.context_menu_track = None;
                        }

                        // Solo toggle
                        let is_solo = state.tracks.get(track_idx).map(|t| t.solo).unwrap_or(false);
                        let solo_text = if is_solo { "Un-Solo" } else { "Solo" };
                        if ui.button(solo_text).clicked() {
                            if let Some(track) = state.tracks.get_mut(track_idx) {
                                track.solo = !track.solo;
                            }
                            state.context_menu_track = None;
                        }

                        // Record arm toggle
                        let is_armed = state.tracks.get(track_idx).map(|t| t.armed).unwrap_or(false);
                        let arm_text = if is_armed { "Disarm" } else { "Arm for Recording" };
                        if ui.button(arm_text).clicked() {
                            if let Some(track) = state.tracks.get_mut(track_idx) {
                                track.armed = !track.armed;
                            }
                            state.context_menu_track = None;
                        }

                        ui.separator();

                        // Duplicate track
                        if ui.button("Duplicate Track").clicked() {
                            if let Some(track) = state.tracks.get(track_idx).cloned() {
                                let mut new_track = track;
                                new_track.id = Uuid::new_v4();
                                new_track.name = format!("{} (copy)", new_track.name);
                                // Insert after current track
                                if track_idx + 1 < state.tracks.len() {
                                    state.tracks.insert(track_idx + 1, new_track);
                                } else {
                                    state.tracks.push(new_track);
                                }
                            }
                            state.context_menu_track = None;
                        }

                        // Delete track
                        if ui.button(egui::RichText::new("Delete Track").color(Color32::from_rgb(231, 76, 60))).clicked() {
                            if state.tracks.len() > 1 {
                                state.tracks.remove(track_idx);
                                // Adjust selected track if needed
                                if let Some(sel) = state.selected_track {
                                    if sel >= state.tracks.len() {
                                        state.selected_track = Some(state.tracks.len().saturating_sub(1));
                                    } else if sel > track_idx {
                                        state.selected_track = Some(sel - 1);
                                    }
                                }
                            }
                            state.context_menu_track = None;
                        }

                        ui.separator();

                        // Color submenu (simple version)
                        ui.menu_button("Change Color", |ui| {
                            let colors = [
                                ("Blue", Color32::from_rgb(52, 152, 219)),
                                ("Green", Color32::from_rgb(46, 204, 113)),
                                ("Purple", Color32::from_rgb(155, 89, 182)),
                                ("Red", Color32::from_rgb(231, 76, 60)),
                                ("Yellow", Color32::from_rgb(241, 196, 15)),
                                ("Orange", Color32::from_rgb(230, 126, 34)),
                                ("Teal", Color32::from_rgb(26, 188, 156)),
                                ("Pink", Color32::from_rgb(233, 30, 99)),
                            ];

                            for (name, color) in colors {
                                ui.horizontal(|ui| {
                                    // Color swatch
                                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                                    ui.painter().rect_filled(rect, Rounding::same(2.0), color);
                                    if ui.button(name).clicked() {
                                        if let Some(track) = state.tracks.get_mut(track_idx) {
                                            track.color = color;
                                        }
                                        state.context_menu_track = None;
                                    }
                                });
                            }
                        });

                        ui.separator();

                        // Close menu
                        if ui.button("Cancel").clicked() {
                            state.context_menu_track = None;
                        }
                    });
            });

        // Close menu on click outside or escape
        ui.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                state.context_menu_track = None;
            }
            // Close on any click outside
            if i.pointer.any_click() && state.context_menu_pos.is_some() {
                // Check if click is outside menu area (approximate)
                if let Some(pos) = i.pointer.interact_pos() {
                    let menu_pos = state.context_menu_pos.unwrap();
                    let menu_rect = Rect::from_min_size(menu_pos, Vec2::new(160.0, 300.0));
                    if !menu_rect.contains(pos) {
                        state.context_menu_track = None;
                    }
                }
            }
        });
    }

    /// Handle keyboard shortcuts
    fn handle_keyboard(ui: &mut Ui, state: &mut ArrangeViewState) -> ArrangeViewAction {
        let mut action = ArrangeViewAction::None;

        ui.input(|i| {
            // Space: toggle playback
            if i.key_pressed(egui::Key::Space) {
                action = ArrangeViewAction::TogglePlayback;
            }

            // Escape: stop and deselect
            if i.key_pressed(egui::Key::Escape) {
                if state.is_playing || state.is_recording {
                    action = ArrangeViewAction::Stop;
                } else {
                    state.deselect_all();
                }
            }

            // Delete/Backspace: delete selected clips
            if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                if !state.selection.clips.is_empty() {
                    state.delete_selected_clips();
                    action = ArrangeViewAction::DeleteSelection;
                }
            }

            // Arrow keys: navigate
            if i.key_pressed(egui::Key::ArrowLeft) {
                if i.modifiers.shift {
                    // Shift+Left: move by bar
                    state.move_playhead_bars(-1);
                } else {
                    // Left: move by beat
                    state.move_playhead_beats(-1);
                }
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                if i.modifiers.shift {
                    // Shift+Right: move by bar
                    state.move_playhead_bars(1);
                } else {
                    // Right: move by beat
                    state.move_playhead_beats(1);
                }
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                // Up: select previous track
                state.select_prev_track();
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                // Down: select next track
                state.select_next_track();
            }

            // Home: go to start
            if i.key_pressed(egui::Key::Home) {
                state.go_to_start();
            }

            // End: go to end
            if i.key_pressed(egui::Key::End) {
                state.go_to_end();
            }

            // A: select all (Ctrl+A)
            if i.key_pressed(egui::Key::A) && i.modifiers.ctrl {
                state.select_all_clips();
            }

            // D: deselect all
            if i.key_pressed(egui::Key::D) && !i.modifiers.ctrl {
                state.deselect_all();
            }

            // L: toggle loop
            if i.key_pressed(egui::Key::L) && !i.modifiers.ctrl {
                state.loop_enabled = !state.loop_enabled;
            }

            // G: toggle grid
            if i.key_pressed(egui::Key::G) && !i.modifiers.ctrl {
                state.show_grid = !state.show_grid;
            }

            // +/=: zoom in
            if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                state.zoom_h = (state.zoom_h + 0.2).min(10.0);
            }

            // -: zoom out
            if i.key_pressed(egui::Key::Minus) {
                state.zoom_h = (state.zoom_h - 0.2).max(0.1);
            }

            // M: toggle mute on selected track
            if i.key_pressed(egui::Key::M) && !i.modifiers.ctrl {
                state.toggle_selected_track_mute();
            }

            // S: toggle solo on selected track
            if i.key_pressed(egui::Key::S) && !i.modifiers.ctrl && !i.modifiers.shift {
                state.toggle_selected_track_solo();
            }

            // R: toggle recording
            if i.key_pressed(egui::Key::R) && !i.modifiers.ctrl {
                action = ArrangeViewAction::ToggleRecording;
            }
        });

        action
    }

    /// Draw the status bar
    fn draw_status_bar(ui: &mut Ui, state: &ArrangeViewState, theme: &Theme, rect: Rect) {
        // Background
        ui.painter().rect_filled(rect, Rounding::ZERO, theme.surface_bg());

        // Top border
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(1.0, theme.border()),
        );

        let mut status_ui = ui.child_ui(rect.shrink(4.0), egui::Layout::left_to_right(egui::Align::Center), None);

        // Position display
        let bars = (state.playhead_secs / state.bar_duration_secs()).floor() as i32 + 1;
        let beat_in_bar = ((state.playhead_secs % state.bar_duration_secs()) / state.beat_duration_secs()).floor() as i32 + 1;
        let total_secs = state.playhead_secs;
        let mins = (total_secs / 60.0) as i32;
        let secs = total_secs % 60.0;

        status_ui.label(
            egui::RichText::new(format!("Position: {}:{} ({:02}:{:05.2})", bars, beat_in_bar, mins, secs))
                .color(theme.text_primary())
                .size(11.0)
        );

        status_ui.separator();

        // Tempo
        status_ui.label(
            egui::RichText::new(format!("{:.0} BPM", state.tempo))
                .color(theme.text_secondary())
                .size(11.0)
        );

        status_ui.separator();

        // Time signature
        status_ui.label(
            egui::RichText::new(format!("{}/{}", state.time_sig_num, state.time_sig_denom))
                .color(theme.text_secondary())
                .size(11.0)
        );

        status_ui.separator();

        // Snap mode
        let snap_text = match state.snap_mode {
            SnapMode::Off => "Snap: Off",
            SnapMode::Grid => "Snap: Grid",
            SnapMode::Bar => "Snap: Bar",
            SnapMode::Beat => "Snap: Beat",
            SnapMode::Subdivision => "Snap: 1/16",
        };
        status_ui.label(
            egui::RichText::new(snap_text)
                .color(theme.text_secondary())
                .size(11.0)
        );

        status_ui.separator();

        // Selection count
        let clip_count = state.selected_clip_count();
        if clip_count > 0 {
            status_ui.label(
                egui::RichText::new(format!("{} clip{} selected", clip_count, if clip_count == 1 { "" } else { "s" }))
                    .color(theme.accent())
                    .size(11.0)
            );
        }

        // Right-aligned section
        status_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Playback status
            let status = if state.is_recording {
                ("Recording", Color32::from_rgb(231, 76, 60))
            } else if state.is_playing {
                ("Playing", Color32::from_rgb(46, 204, 113))
            } else {
                ("Stopped", theme.text_secondary())
            };
            ui.label(egui::RichText::new(status.0).color(status.1).size(11.0));

            ui.separator();

            // Loop indicator
            if state.loop_enabled {
                ui.label(
                    egui::RichText::new("Loop")
                        .color(Color32::from_rgb(155, 89, 182))
                        .size(11.0)
                );
            }

            ui.separator();

            // Zoom level
            ui.label(
                egui::RichText::new(format!("Zoom: {:.0}%", state.zoom_h * 100.0))
                    .color(theme.text_secondary())
                    .size(11.0)
            );

            // Keyboard hints
            ui.separator();
            ui.label(
                egui::RichText::new("Space: Play | Arrows: Navigate | L: Loop | G: Grid")
                    .color(theme.text_secondary().gamma_multiply(0.7))
                    .size(10.0)
            );
        });
    }

    /// Draw the toolbar
    fn draw_toolbar(ui: &mut Ui, state: &mut ArrangeViewState, theme: &Theme, rect: Rect) {
        // Toolbar background
        ui.painter().rect_filled(
            rect,
            Rounding::ZERO,
            theme.surface_bg(),
        );

        // Toolbar content
        let mut toolbar_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center), None);

        toolbar_ui.add_space(8.0);

        // Zoom controls
        toolbar_ui.label("Zoom:");
        if toolbar_ui.small_button("−").on_hover_text("Zoom out (-)").clicked() {
            state.zoom_h = (state.zoom_h - 0.1).max(0.1);
        }
        toolbar_ui.label(format!("{:.0}%", state.zoom_h * 100.0));
        if toolbar_ui.small_button("+").on_hover_text("Zoom in (+)").clicked() {
            state.zoom_h = (state.zoom_h + 0.1).min(10.0);
        }

        toolbar_ui.separator();

        // Snap controls
        toolbar_ui.label("Snap:");
        egui::ComboBox::from_id_salt("snap_mode")
            .selected_text(match state.snap_mode {
                SnapMode::Off => "Off",
                SnapMode::Grid => "Grid",
                SnapMode::Bar => "Bar",
                SnapMode::Beat => "Beat",
                SnapMode::Subdivision => "1/16",
            })
            .show_ui(&mut toolbar_ui, |ui| {
                ui.selectable_value(&mut state.snap_mode, SnapMode::Off, "Off");
                ui.selectable_value(&mut state.snap_mode, SnapMode::Grid, "Grid");
                ui.selectable_value(&mut state.snap_mode, SnapMode::Bar, "Bar");
                ui.selectable_value(&mut state.snap_mode, SnapMode::Beat, "Beat");
                ui.selectable_value(&mut state.snap_mode, SnapMode::Subdivision, "1/16");
            }).response.on_hover_text("Snap clips to grid divisions");

        toolbar_ui.separator();

        // Grid toggle
        toolbar_ui.checkbox(&mut state.show_grid, "Grid")
            .on_hover_text("Show grid lines on timeline");

        toolbar_ui.separator();

        // Position display
        let bars = (state.playhead_secs / state.bar_duration_secs()).floor() as i32 + 1;
        let beat_in_bar = ((state.playhead_secs % state.bar_duration_secs()) / state.beat_duration_secs()).floor() as i32 + 1;
        toolbar_ui.label(format!("{}:{}", bars, beat_in_bar));
    }

    /// Draw track headers
    fn draw_track_headers(ui: &mut Ui, state: &mut ArrangeViewState, theme: &Theme, rect: Rect) {
        // Background
        ui.painter().rect_filled(rect, Rounding::ZERO, theme.surface_bg());

        // Empty header for ruler alignment
        let ruler_header = Rect::from_min_size(rect.min, Vec2::new(rect.width(), RULER_HEIGHT));
        ui.painter().rect_filled(ruler_header, Rounding::ZERO, theme.panel_bg());
        ui.painter().text(
            ruler_header.center(),
            egui::Align2::CENTER_CENTER,
            "Tracks",
            egui::FontId::proportional(12.0),
            theme.text_secondary(),
        );

        // Track right-click detection
        let mut context_click: Option<(usize, Pos2)> = None;
        let mut selected_click: Option<usize> = None;

        // Track headers
        let mut y = rect.min.y + RULER_HEIGHT - state.scroll_v;
        for (idx, track) in state.tracks.iter_mut().enumerate() {
            let header_height = track.height * state.zoom_v;
            if y + header_height < rect.min.y + RULER_HEIGHT {
                y += header_height;
                continue;
            }
            if y > rect.max.y {
                break;
            }

            let header_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, y),
                Vec2::new(rect.width(), header_height),
            );

            // Clip to visible area
            let visible_rect = header_rect.intersect(Rect::from_min_max(
                Pos2::new(rect.min.x, rect.min.y + RULER_HEIGHT),
                rect.max,
            ));

            if visible_rect.height() > 0.0 {
                let is_selected = state.selected_track == Some(idx);
                let response = Self::draw_track_header(ui, track, theme, header_rect, is_selected);

                // Left-click to select
                if response.clicked() {
                    selected_click = Some(idx);
                }

                // Right-click for context menu
                if response.secondary_clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        context_click = Some((idx, pos));
                    }
                }
            }

            y += header_height;
        }

        // Handle selection
        if let Some(idx) = selected_click {
            state.selected_track = Some(idx);
        }

        // Handle context menu
        if let Some((idx, pos)) = context_click {
            state.context_menu_track = Some(idx);
            state.context_menu_pos = Some(pos);
            state.selected_track = Some(idx);
        }

        // Separator line
        ui.painter().line_segment(
            [
                Pos2::new(rect.max.x - 1.0, rect.min.y),
                Pos2::new(rect.max.x - 1.0, rect.max.y),
            ],
            Stroke::new(1.0, theme.border()),
        );
    }

    /// Draw a single track header
    fn draw_track_header(
        ui: &mut Ui,
        track: &mut TrackDisplay,
        theme: &Theme,
        rect: Rect,
        is_selected: bool,
    ) -> egui::Response {
        // Background with selection highlight
        let bg = if is_selected {
            theme.surface_bg().gamma_multiply(1.3)
        } else if track.muted {
            theme.surface_bg().gamma_multiply(0.5)
        } else {
            theme.surface_bg()
        };
        ui.painter().rect_filled(rect, Rounding::ZERO, bg);

        // Selection border
        if is_selected {
            ui.painter().rect_stroke(
                rect,
                Rounding::ZERO,
                Stroke::new(2.0, theme.accent()),
            );
        }

        // Color bar
        ui.painter().rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(4.0, rect.height())),
            Rounding::ZERO,
            track.color,
        );

        // Track name
        ui.painter().text(
            Pos2::new(rect.min.x + 10.0, rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            &track.name,
            egui::FontId::proportional(13.0),
            if is_selected { theme.accent() } else { theme.text_primary() },
        );

        // Buttons row
        let button_y = rect.min.y + 28.0;
        let button_size = Vec2::splat(18.0);
        let mut button_x = rect.min.x + 10.0;

        // Mute button
        let mute_rect = Rect::from_min_size(Pos2::new(button_x, button_y), button_size);
        let mute_response = ui.allocate_rect(mute_rect, Sense::click());
        let mute_color = if track.muted {
            Color32::from_rgb(231, 76, 60)
        } else {
            theme.text_secondary()
        };
        ui.painter().rect_filled(mute_rect, Rounding::same(3.0), theme.panel_bg());
        ui.painter().text(
            mute_rect.center(),
            egui::Align2::CENTER_CENTER,
            "M",
            egui::FontId::proportional(10.0),
            mute_color,
        );
        if mute_response.clicked() {
            track.muted = !track.muted;
        }

        button_x += button_size.x + 4.0;

        // Solo button
        let solo_rect = Rect::from_min_size(Pos2::new(button_x, button_y), button_size);
        let solo_response = ui.allocate_rect(solo_rect, Sense::click());
        let solo_color = if track.solo {
            Color32::from_rgb(241, 196, 15)
        } else {
            theme.text_secondary()
        };
        ui.painter().rect_filled(solo_rect, Rounding::same(3.0), theme.panel_bg());
        ui.painter().text(
            solo_rect.center(),
            egui::Align2::CENTER_CENTER,
            "S",
            egui::FontId::proportional(10.0),
            solo_color,
        );
        if solo_response.clicked() {
            track.solo = !track.solo;
        }

        button_x += button_size.x + 4.0;

        // Arm button
        let arm_rect = Rect::from_min_size(Pos2::new(button_x, button_y), button_size);
        let arm_response = ui.allocate_rect(arm_rect, Sense::click());
        let arm_color = if track.armed {
            Color32::from_rgb(231, 76, 60)
        } else {
            theme.text_secondary()
        };
        ui.painter().rect_filled(arm_rect, Rounding::same(3.0), theme.panel_bg());
        ui.painter().text(
            arm_rect.center(),
            egui::Align2::CENTER_CENTER,
            "R",
            egui::FontId::proportional(10.0),
            arm_color,
        );
        if arm_response.clicked() {
            track.armed = !track.armed;
        }

        // Level meter (if track is tall enough)
        if rect.height() > 50.0 {
            let meter_rect = Rect::from_min_size(
                Pos2::new(rect.max.x - 30.0, rect.min.y + 8.0),
                Vec2::new(20.0, rect.height() - 16.0),
            );
            // Simple level indicator
            let level_normalized = ((track.level_db + 60.0) / 60.0).clamp(0.0, 1.0);
            let meter_height = meter_rect.height() * level_normalized;
            ui.painter().rect_filled(meter_rect, Rounding::same(2.0), theme.panel_bg());
            ui.painter().rect_filled(
                Rect::from_min_size(
                    Pos2::new(meter_rect.min.x, meter_rect.max.y - meter_height),
                    Vec2::new(meter_rect.width(), meter_height),
                ),
                Rounding::same(2.0),
                Color32::from_rgb(46, 204, 113),
            );
        }

        // Bottom border
        ui.painter().line_segment(
            [
                Pos2::new(rect.min.x, rect.max.y - 1.0),
                Pos2::new(rect.max.x, rect.max.y - 1.0),
            ],
            Stroke::new(1.0, theme.border()),
        );

        // Allocate rect for interaction
        ui.allocate_rect(rect, Sense::click())
    }

    /// Draw the timeline ruler
    fn draw_ruler(ui: &mut Ui, state: &ArrangeViewState, theme: &Theme, rect: Rect) {
        // Background
        ui.painter().rect_filled(rect, Rounding::ZERO, theme.panel_bg());

        let pixels_per_bar = state.secs_to_pixels(state.bar_duration_secs());
        let pixels_per_beat = state.secs_to_pixels(state.beat_duration_secs());

        // Calculate visible range
        let start_secs = state.scroll_h;
        let end_secs = start_secs + state.pixels_to_secs(rect.width());

        // Draw bars and beats
        let start_bar = (start_secs / state.bar_duration_secs()).floor() as i32;
        let end_bar = (end_secs / state.bar_duration_secs()).ceil() as i32;

        for bar in start_bar..=end_bar {
            let bar_secs = bar as f64 * state.bar_duration_secs();
            let x = rect.min.x + state.secs_to_pixels(bar_secs - state.scroll_h);

            if x < rect.min.x || x > rect.max.x {
                continue;
            }

            // Bar line
            ui.painter().line_segment(
                [
                    Pos2::new(x, rect.min.y),
                    Pos2::new(x, rect.max.y),
                ],
                Stroke::new(1.0, theme.text_secondary()),
            );

            // Bar number
            ui.painter().text(
                Pos2::new(x + 4.0, rect.min.y + 4.0),
                egui::Align2::LEFT_TOP,
                format!("{}", bar + 1),
                egui::FontId::proportional(11.0),
                theme.text_primary(),
            );

            // Beat subdivisions
            if pixels_per_beat > 15.0 {
                for beat in 1..state.time_sig_num {
                    let beat_x = x + (beat as f32 * pixels_per_beat);
                    if beat_x > rect.min.x && beat_x < rect.max.x {
                        ui.painter().line_segment(
                            [
                                Pos2::new(beat_x, rect.max.y - 8.0),
                                Pos2::new(beat_x, rect.max.y),
                            ],
                            Stroke::new(1.0, theme.text_secondary().gamma_multiply(0.5)),
                        );
                    }
                }
            }
        }

        // Loop region indicator
        if state.loop_enabled {
            let loop_start_x = rect.min.x + state.secs_to_pixels(state.loop_start - state.scroll_h);
            let loop_end_x = rect.min.x + state.secs_to_pixels(state.loop_end - state.scroll_h);

            if loop_end_x > rect.min.x && loop_start_x < rect.max.x {
                let loop_rect = Rect::from_min_max(
                    Pos2::new(loop_start_x.max(rect.min.x), rect.max.y - 4.0),
                    Pos2::new(loop_end_x.min(rect.max.x), rect.max.y),
                );
                ui.painter().rect_filled(
                    loop_rect,
                    Rounding::ZERO,
                    Color32::from_rgb(155, 89, 182).gamma_multiply(0.7),
                );
            }
        }

        // Bottom border
        ui.painter().line_segment(
            [
                Pos2::new(rect.min.x, rect.max.y - 1.0),
                Pos2::new(rect.max.x, rect.max.y - 1.0),
            ],
            Stroke::new(1.0, theme.border()),
        );
    }

    /// Draw track lanes with clips
    fn draw_track_lanes(ui: &mut Ui, state: &mut ArrangeViewState, theme: &Theme, rect: Rect) {
        // Background
        ui.painter().rect_filled(rect, Rounding::ZERO, theme.panel_bg());

        // Draw grid if enabled
        if state.show_grid {
            Self::draw_grid(ui, state, theme, rect);
        }

        // Extract values needed for drawing before borrowing tracks mutably
        let scroll_v = state.scroll_v;
        let scroll_h = state.scroll_h;
        let zoom_v = state.zoom_v;
        let zoom_h = state.zoom_h;

        // Draw tracks and clips
        let mut y = rect.min.y - scroll_v;
        for track in state.tracks.iter_mut() {
            let track_height = track.height * zoom_v;
            if y + track_height < rect.min.y {
                y += track_height;
                continue;
            }
            if y > rect.max.y {
                break;
            }

            let lane_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, y),
                Vec2::new(rect.width(), track_height),
            );

            // Clip to visible area
            let visible_rect = lane_rect.intersect(rect);
            if visible_rect.height() > 0.0 {
                Self::draw_track_lane(ui, track, theme, lane_rect, rect, scroll_h, zoom_h);
            }

            y += track_height;
        }
    }

    /// Draw the grid
    fn draw_grid(ui: &mut Ui, state: &ArrangeViewState, theme: &Theme, rect: Rect) {
        let pixels_per_bar = state.secs_to_pixels(state.bar_duration_secs());
        let pixels_per_beat = state.secs_to_pixels(state.beat_duration_secs());

        let start_secs = state.scroll_h;
        let end_secs = start_secs + state.pixels_to_secs(rect.width());

        let start_bar = (start_secs / state.bar_duration_secs()).floor() as i32;
        let end_bar = (end_secs / state.bar_duration_secs()).ceil() as i32;

        for bar in start_bar..=end_bar {
            let bar_secs = bar as f64 * state.bar_duration_secs();
            let x = rect.min.x + state.secs_to_pixels(bar_secs - state.scroll_h);

            if x < rect.min.x || x > rect.max.x {
                continue;
            }

            // Bar line (darker)
            ui.painter().line_segment(
                [
                    Pos2::new(x, rect.min.y),
                    Pos2::new(x, rect.max.y),
                ],
                Stroke::new(1.0, theme.border()),
            );

            // Beat lines (lighter)
            if pixels_per_beat > 10.0 {
                for beat in 1..state.time_sig_num {
                    let beat_x = x + (beat as f32 * pixels_per_beat);
                    if beat_x > rect.min.x && beat_x < rect.max.x {
                        ui.painter().line_segment(
                            [
                                Pos2::new(beat_x, rect.min.y),
                                Pos2::new(beat_x, rect.max.y),
                            ],
                            Stroke::new(1.0, theme.border().gamma_multiply(0.5)),
                        );
                    }
                }
            }
        }
    }

    /// Draw a single track lane
    fn draw_track_lane(
        ui: &mut Ui,
        track: &mut TrackDisplay,
        theme: &Theme,
        lane_rect: Rect,
        clip_rect: Rect,
        scroll_h: f64,
        zoom_h: f32,
    ) {
        // Helper for seconds to pixels conversion
        let secs_to_pixels = |secs: f64| -> f32 {
            (secs as f32) * BASE_PIXELS_PER_SECOND * zoom_h
        };

        // Draw clips
        for clip in &mut track.clips {
            let clip_x = clip_rect.min.x + secs_to_pixels(clip.start_secs - scroll_h);
            let clip_width = secs_to_pixels(clip.length_secs);

            // Skip if not visible
            if clip_x + clip_width < clip_rect.min.x || clip_x > clip_rect.max.x {
                continue;
            }

            let clip_display_rect = Rect::from_min_size(
                Pos2::new(clip_x, lane_rect.min.y + 2.0),
                Vec2::new(clip_width, lane_rect.height() - 4.0),
            );

            Self::draw_clip(ui, clip, theme, clip_display_rect, clip_rect);
        }

        // Bottom border
        ui.painter().line_segment(
            [
                Pos2::new(lane_rect.min.x, lane_rect.max.y - 1.0),
                Pos2::new(lane_rect.max.x, lane_rect.max.y - 1.0),
            ],
            Stroke::new(1.0, theme.border().gamma_multiply(0.5)),
        );
    }

    /// Draw a clip
    fn draw_clip(
        ui: &mut Ui,
        clip: &mut ClipDisplay,
        theme: &Theme,
        rect: Rect,
        clip_bounds: Rect,
    ) {
        // Clip to bounds
        let visible_rect = rect.intersect(clip_bounds);
        if visible_rect.width() <= 0.0 || visible_rect.height() <= 0.0 {
            return;
        }

        // Selection highlight
        let bg_color = if clip.selected {
            clip.color.gamma_multiply(1.2)
        } else {
            clip.color
        };

        // Clip background
        ui.painter().rect_filled(visible_rect, Rounding::same(4.0), bg_color.gamma_multiply(0.3));

        // Clip border
        let border_color = if clip.selected {
            Color32::WHITE
        } else {
            bg_color
        };
        ui.painter().rect_stroke(visible_rect, Rounding::same(4.0), Stroke::new(1.0, border_color));

        // Clip header
        let header_height = 16.0_f32.min(visible_rect.height() * 0.3);
        let header_rect = Rect::from_min_size(visible_rect.min, Vec2::new(visible_rect.width(), header_height));
        ui.painter().rect_filled(
            header_rect,
            Rounding {
                nw: 4.0,
                ne: 4.0,
                sw: 0.0,
                se: 0.0,
            },
            bg_color.gamma_multiply(0.6),
        );

        // Clip name
        if visible_rect.width() > 30.0 {
            let name_rect = header_rect.shrink(2.0);
            ui.painter().text(
                name_rect.left_center(),
                egui::Align2::LEFT_CENTER,
                &clip.name,
                egui::FontId::proportional(10.0),
                Color32::WHITE,
            );
        }

        // Draw waveform if audio clip
        if clip.is_audio {
            if let Some(ref waveform) = clip.waveform {
                let waveform_rect = Rect::from_min_max(
                    Pos2::new(visible_rect.min.x, visible_rect.min.y + header_height),
                    visible_rect.max,
                );

                if waveform_rect.height() > 10.0 {
                    Self::draw_waveform(ui, waveform, theme, waveform_rect, clip.color);
                }
            }
        }

        // Make clip interactive
        let response = ui.allocate_rect(visible_rect, Sense::click_and_drag());
        if response.clicked() {
            clip.selected = !clip.selected;
        }
    }

    /// Draw waveform in a clip
    fn draw_waveform(
        ui: &mut Ui,
        waveform: &[(f32, f32)],
        _theme: &Theme,
        rect: Rect,
        color: Color32,
    ) {
        if waveform.is_empty() {
            return;
        }

        let mid_y = rect.center().y;
        let height = rect.height() * 0.8;
        let num_points = waveform.len();
        let x_step = rect.width() / num_points as f32;

        let waveform_color = color.gamma_multiply(0.8);

        for (i, &(min, max)) in waveform.iter().enumerate() {
            let x = rect.min.x + i as f32 * x_step;
            let y_min = mid_y - min * height / 2.0;
            let y_max = mid_y - max * height / 2.0;

            ui.painter().line_segment(
                [Pos2::new(x, y_min), Pos2::new(x, y_max)],
                Stroke::new(1.0, waveform_color),
            );
        }
    }

    /// Draw the playhead
    fn draw_playhead(
        ui: &mut Ui,
        state: &ArrangeViewState,
        theme: &Theme,
        ruler_rect: Rect,
        lanes_rect: Rect,
    ) {
        let x = lanes_rect.min.x + state.secs_to_pixels(state.playhead_secs - state.scroll_h);

        if x < lanes_rect.min.x || x > lanes_rect.max.x {
            return;
        }

        let playhead_color = if state.is_recording {
            Color32::from_rgb(231, 76, 60)
        } else if state.is_playing {
            Color32::from_rgb(46, 204, 113)
        } else {
            Color32::WHITE
        };

        // Playhead line through lanes
        ui.painter().line_segment(
            [
                Pos2::new(x, ruler_rect.min.y),
                Pos2::new(x, lanes_rect.max.y),
            ],
            Stroke::new(2.0, playhead_color),
        );

        // Playhead triangle in ruler
        let triangle = [
            Pos2::new(x, ruler_rect.max.y - 8.0),
            Pos2::new(x - 6.0, ruler_rect.max.y),
            Pos2::new(x + 6.0, ruler_rect.max.y),
        ];
        ui.painter().add(egui::Shape::convex_polygon(
            triangle.to_vec(),
            playhead_color,
            Stroke::NONE,
        ));
    }
}
