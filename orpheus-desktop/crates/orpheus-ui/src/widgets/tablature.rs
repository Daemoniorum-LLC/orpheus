//! Tablature view widget for guitar/bass notation
//!
//! Features:
//! - View and render Guitar Pro files
//! - Edit mode for note entry
//! - Playback integration with orpheus-synth

use egui::{Color32, FontId, Key, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use orpheus_file::guitar_pro::{Beat, Duration, GuitarProFile, Track, Note, NoteEffects};

/// Configuration for tablature rendering
#[derive(Clone)]
pub struct TabConfig {
    /// Height between string lines in pixels
    pub string_spacing: f32,
    /// Width of a beat unit (quarter note)
    pub beat_width: f32,
    /// Margin at top of tab
    pub top_margin: f32,
    /// Left margin for string labels
    pub left_margin: f32,
    /// Font size for fret numbers
    pub fret_font_size: f32,
    /// Font size for markers
    pub marker_font_size: f32,
    /// String line color
    pub string_color: Color32,
    /// Fret number color
    pub fret_color: Color32,
    /// Bar line color
    pub bar_color: Color32,
    /// Marker color
    pub marker_color: Color32,
    /// Selected beat highlight
    pub selection_color: Color32,
    /// Background color
    pub background_color: Color32,
    /// Cursor color (for edit mode)
    pub cursor_color: Color32,
    /// Edit string highlight
    pub edit_string_color: Color32,
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            string_spacing: 18.0,
            beat_width: 40.0,
            top_margin: 30.0,
            left_margin: 30.0,
            fret_font_size: 14.0,
            marker_font_size: 12.0,
            string_color: Color32::from_rgb(80, 80, 100),
            fret_color: Color32::from_rgb(230, 230, 240),
            bar_color: Color32::from_rgb(100, 100, 120),
            marker_color: Color32::from_rgb(26, 123, 93),  // Phthalo Green accent
            selection_color: Color32::from_rgba_premultiplied(26, 123, 93, 60),
            background_color: Color32::from_rgb(26, 26, 36),
            cursor_color: Color32::from_rgb(255, 200, 50),  // Golden cursor
            edit_string_color: Color32::from_rgba_premultiplied(255, 200, 50, 30),
        }
    }
}

/// Edit mode for tablature
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum EditMode {
    /// View only
    #[default]
    View,
    /// Edit mode - can modify notes
    Edit,
    /// Insert mode - adds new beats
    Insert,
}

/// Duration for new notes
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NoteDuration {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

impl NoteDuration {
    pub fn to_duration(&self) -> Duration {
        match self {
            Self::Whole => Duration::Whole,
            Self::Half => Duration::Half,
            Self::Quarter => Duration::Quarter,
            Self::Eighth => Duration::Eighth,
            Self::Sixteenth => Duration::Sixteenth,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Whole => "W",
            Self::Half => "H",
            Self::Quarter => "Q",
            Self::Eighth => "8",
            Self::Sixteenth => "16",
        }
    }
}

impl Default for NoteDuration {
    fn default() -> Self {
        Self::Quarter
    }
}

/// State for tablature view
#[derive(Default)]
pub struct TabState {
    /// Currently selected measure index
    pub selected_measure: Option<usize>,
    /// Currently selected beat index within measure
    pub selected_beat: Option<usize>,
    /// Currently selected string (1-indexed, 1 = highest)
    pub selected_string: u8,
    /// Currently selected track index
    pub selected_track: usize,
    /// Scroll offset X
    pub scroll_x: f32,
    /// Zoom level (1.0 = 100%)
    pub zoom: f32,
    /// Edit mode
    pub edit_mode: EditMode,
    /// Current note duration for new notes
    pub note_duration: NoteDuration,
    /// Fret input buffer (for multi-digit frets like "12")
    pub fret_input_buffer: String,
    /// Is playing
    pub is_playing: bool,
    /// Playback position (beat index)
    pub playback_beat: Option<usize>,
    /// Show note preview
    pub show_preview: bool,
}

impl TabState {
    pub fn new() -> Self {
        Self {
            selected_measure: None,
            selected_beat: None,
            selected_string: 1,
            selected_track: 0,
            scroll_x: 0.0,
            zoom: 1.0,
            edit_mode: EditMode::View,
            note_duration: NoteDuration::Quarter,
            fret_input_buffer: String::new(),
            is_playing: false,
            playback_beat: None,
            show_preview: true,
        }
    }

    /// Toggle edit mode
    pub fn toggle_edit_mode(&mut self) {
        self.edit_mode = match self.edit_mode {
            EditMode::View => EditMode::Edit,
            EditMode::Edit => EditMode::View,
            EditMode::Insert => EditMode::View,
        };
        self.fret_input_buffer.clear();
    }

    /// Cycle through note durations
    pub fn cycle_duration(&mut self, reverse: bool) {
        self.note_duration = if reverse {
            match self.note_duration {
                NoteDuration::Whole => NoteDuration::Sixteenth,
                NoteDuration::Half => NoteDuration::Whole,
                NoteDuration::Quarter => NoteDuration::Half,
                NoteDuration::Eighth => NoteDuration::Quarter,
                NoteDuration::Sixteenth => NoteDuration::Eighth,
            }
        } else {
            match self.note_duration {
                NoteDuration::Whole => NoteDuration::Half,
                NoteDuration::Half => NoteDuration::Quarter,
                NoteDuration::Quarter => NoteDuration::Eighth,
                NoteDuration::Eighth => NoteDuration::Sixteenth,
                NoteDuration::Sixteenth => NoteDuration::Whole,
            }
        };
    }

    /// Move cursor to next beat
    pub fn next_beat(&mut self, num_beats_in_measure: usize) {
        if let Some(beat) = self.selected_beat {
            if beat + 1 < num_beats_in_measure {
                self.selected_beat = Some(beat + 1);
            }
        }
        self.fret_input_buffer.clear();
    }

    /// Move cursor to previous beat
    pub fn prev_beat(&mut self) {
        if let Some(beat) = self.selected_beat {
            if beat > 0 {
                self.selected_beat = Some(beat - 1);
            }
        }
        self.fret_input_buffer.clear();
    }

    /// Move cursor up (lower string number = higher pitch)
    pub fn cursor_up(&mut self, num_strings: u8) {
        if self.selected_string > 1 {
            self.selected_string -= 1;
        }
        self.fret_input_buffer.clear();
    }

    /// Move cursor down (higher string number = lower pitch)
    pub fn cursor_down(&mut self, num_strings: u8) {
        if self.selected_string < num_strings {
            self.selected_string += 1;
        }
        self.fret_input_buffer.clear();
    }

    /// Input a fret digit
    pub fn input_fret_digit(&mut self, digit: char) -> Option<u8> {
        self.fret_input_buffer.push(digit);

        // Try to parse as fret number
        if let Ok(fret) = self.fret_input_buffer.parse::<u8>() {
            if fret <= 24 {
                // Valid fret, return it
                // Clear buffer if we have 2 digits or fret > 2
                if self.fret_input_buffer.len() >= 2 || fret > 2 {
                    self.fret_input_buffer.clear();
                }
                return Some(fret);
            }
        }

        // Invalid, clear buffer
        if self.fret_input_buffer.len() >= 2 {
            self.fret_input_buffer.clear();
        }
        None
    }
}

/// Tablature view widget
pub struct TablatureView<'a> {
    gp_file: Option<&'a GuitarProFile>,
    config: TabConfig,
    state: &'a mut TabState,
}

impl<'a> TablatureView<'a> {
    pub fn new(state: &'a mut TabState) -> Self {
        Self {
            gp_file: None,
            config: TabConfig::default(),
            state,
        }
    }

    pub fn with_file(mut self, file: &'a GuitarProFile) -> Self {
        self.gp_file = Some(file);
        self
    }

    pub fn with_config(mut self, config: TabConfig) -> Self {
        self.config = config;
        self
    }

    /// Show the tablature view
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let Some(file) = self.gp_file else {
            return self.show_empty_state(ui);
        };

        if file.tracks.is_empty() {
            return self.show_empty_state(ui);
        }

        let track_idx = self.state.selected_track.min(file.tracks.len() - 1);
        let track = &file.tracks[track_idx];

        // Calculate required size
        let num_strings = track.strings as usize;
        let total_beats: usize = file.measures.iter()
            .filter_map(|m| m.beats.get(track_idx))
            .map(|tb| tb.beats.len().max(1))
            .sum();

        let content_width = (total_beats as f32 * self.config.beat_width * self.state.zoom)
            + self.config.left_margin + 50.0;
        let content_height = (num_strings as f32 * self.config.string_spacing)
            + self.config.top_margin + 40.0;

        // Allocate space with scrolling
        let available = ui.available_size();
        let desired_size = Vec2::new(available.x.max(content_width), content_height);

        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);

            // Draw background
            painter.rect_filled(rect, 0.0, self.config.background_color);

            // Draw tablature
            self.draw_tablature(&painter, rect, file, track, track_idx);
        }

        // Handle interactions
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.handle_click(pos, rect, file, track_idx);
            }
        }

        response
    }

    fn show_empty_state(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::hover());

        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(rect, 0.0, self.config.background_color);

            let center = rect.center();
            ui.painter().text(
                center,
                egui::Align2::CENTER_CENTER,
                "Open a Guitar Pro file to view tablature\n\nDrag and drop .gp3, .gp4, or .gp5 files",
                FontId::proportional(16.0),
                Color32::from_rgb(140, 140, 160),
            );
        }

        response
    }

    fn draw_tablature(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        file: &GuitarProFile,
        track: &Track,
        track_idx: usize,
    ) {
        let cfg = &self.config;
        let num_strings = track.strings as usize;
        let zoom = self.state.zoom;

        // Starting position
        let start_x = rect.left() + cfg.left_margin - self.state.scroll_x;
        let start_y = rect.top() + cfg.top_margin;

        // Draw string labels (tuning)
        for (i, &note) in track.tuning.iter().enumerate().take(num_strings) {
            let y = start_y + (i as f32 * cfg.string_spacing);
            let note_name = midi_note_to_name(note);
            painter.text(
                Pos2::new(rect.left() + 10.0, y),
                egui::Align2::LEFT_CENTER,
                note_name,
                FontId::monospace(10.0),
                cfg.fret_color.linear_multiply(0.7),
            );
        }

        // Draw string lines
        let line_width = rect.width() - cfg.left_margin;
        for i in 0..num_strings {
            let y = start_y + (i as f32 * cfg.string_spacing);
            painter.line_segment(
                [Pos2::new(start_x, y), Pos2::new(start_x + line_width, y)],
                Stroke::new(1.0, cfg.string_color),
            );
        }

        // Draw measures and notes
        let mut x = start_x;

        for (measure_idx, measure) in file.measures.iter().enumerate() {
            // Draw marker if present
            if let Some(marker) = &measure.marker {
                painter.text(
                    Pos2::new(x + 5.0, start_y - 15.0),
                    egui::Align2::LEFT_BOTTOM,
                    marker,
                    FontId::proportional(cfg.marker_font_size),
                    cfg.marker_color,
                );
            }

            // Draw repeat start bracket
            if measure.repeat_start {
                painter.line_segment(
                    [
                        Pos2::new(x, start_y - 5.0),
                        Pos2::new(x, start_y + ((num_strings - 1) as f32 * cfg.string_spacing) + 5.0),
                    ],
                    Stroke::new(3.0, cfg.bar_color),
                );
                // Draw dots
                let dot_y1 = start_y + (1.5 * cfg.string_spacing);
                let dot_y2 = start_y + (3.5 * cfg.string_spacing);
                painter.circle_filled(Pos2::new(x + 8.0, dot_y1), 3.0, cfg.bar_color);
                painter.circle_filled(Pos2::new(x + 8.0, dot_y2), 3.0, cfg.bar_color);
            }

            // Get beats for this track in this measure
            let track_beats = measure.beats.get(track_idx);
            let beats: &[Beat] = track_beats.map(|tb| tb.beats.as_slice()).unwrap_or(&[]);

            // Calculate measure width based on beats
            let measure_beats = beats.len().max(4); // Minimum 4 beats per measure for spacing
            let measure_width = measure_beats as f32 * cfg.beat_width * zoom;

            // Highlight selected measure
            if self.state.selected_measure == Some(measure_idx) {
                let highlight_rect = Rect::from_min_size(
                    Pos2::new(x, start_y - 5.0),
                    Vec2::new(measure_width, (num_strings as f32 * cfg.string_spacing) + 10.0),
                );
                painter.rect_filled(highlight_rect, 0.0, cfg.selection_color);
            }

            // Draw beats
            let mut beat_x = x;
            for (beat_idx, beat) in beats.iter().enumerate() {
                // Calculate beat width based on duration
                let beat_width = duration_to_width(beat.duration, cfg.beat_width) * zoom;

                // Highlight selected beat
                if self.state.selected_measure == Some(measure_idx)
                    && self.state.selected_beat == Some(beat_idx)
                {
                    let beat_rect = Rect::from_min_size(
                        Pos2::new(beat_x, start_y - 3.0),
                        Vec2::new(beat_width, (num_strings as f32 * cfg.string_spacing) + 6.0),
                    );
                    painter.rect_stroke(beat_rect, 2.0, Stroke::new(2.0, cfg.marker_color));
                }

                if beat.is_rest {
                    // Draw rest symbol
                    let rest_y = start_y + ((num_strings as f32 - 1.0) / 2.0 * cfg.string_spacing);
                    painter.text(
                        Pos2::new(beat_x + beat_width / 2.0, rest_y),
                        egui::Align2::CENTER_CENTER,
                        "-",
                        FontId::monospace(cfg.fret_font_size),
                        cfg.fret_color.linear_multiply(0.5),
                    );
                } else {
                    // Draw notes
                    for note in &beat.notes {
                        let string_idx = (note.string as usize).saturating_sub(1);
                        if string_idx < num_strings {
                            let y = start_y + (string_idx as f32 * cfg.string_spacing);

                            // Draw fret number with background
                            let fret_text = if note.tied {
                                "─".to_string() // Tie indicator
                            } else if note.ghost {
                                format!("({})", note.fret)
                            } else {
                                note.fret.to_string()
                            };

                            // Background for fret number
                            let text_width = fret_text.len() as f32 * 8.0;
                            painter.rect_filled(
                                Rect::from_center_size(
                                    Pos2::new(beat_x + beat_width / 2.0, y),
                                    Vec2::new(text_width + 4.0, cfg.string_spacing - 2.0),
                                ),
                                2.0,
                                cfg.background_color,
                            );

                            // Fret number
                            painter.text(
                                Pos2::new(beat_x + beat_width / 2.0, y),
                                egui::Align2::CENTER_CENTER,
                                &fret_text,
                                FontId::monospace(cfg.fret_font_size),
                                cfg.fret_color,
                            );

                            // Draw effect indicators
                            if note.effects.hammer_on || note.effects.pull_off {
                                // H/P arc above
                                painter.text(
                                    Pos2::new(beat_x + beat_width / 2.0, y - cfg.string_spacing / 2.0 - 2.0),
                                    egui::Align2::CENTER_CENTER,
                                    if note.effects.hammer_on { "H" } else { "P" },
                                    FontId::monospace(8.0),
                                    cfg.marker_color,
                                );
                            }

                            if note.effects.slide.is_some() {
                                // Slide indicator
                                painter.text(
                                    Pos2::new(beat_x + beat_width - 3.0, y),
                                    egui::Align2::CENTER_CENTER,
                                    "/",
                                    FontId::monospace(10.0),
                                    cfg.marker_color,
                                );
                            }

                            if note.effects.bend.is_some() {
                                // Bend indicator
                                painter.text(
                                    Pos2::new(beat_x + beat_width / 2.0, y - cfg.string_spacing / 2.0 - 2.0),
                                    egui::Align2::CENTER_CENTER,
                                    "b",
                                    FontId::monospace(8.0),
                                    cfg.marker_color,
                                );
                            }
                        }
                    }
                }

                // Draw dotted note indicator
                if beat.dotted {
                    let dot_y = start_y + ((num_strings as f32 + 0.5) * cfg.string_spacing);
                    painter.circle_filled(
                        Pos2::new(beat_x + beat_width - 5.0, dot_y),
                        2.0,
                        cfg.fret_color,
                    );
                }

                // Draw beat text annotation
                if let Some(text) = &beat.text {
                    painter.text(
                        Pos2::new(beat_x + beat_width / 2.0, start_y - 5.0),
                        egui::Align2::CENTER_BOTTOM,
                        text,
                        FontId::proportional(9.0),
                        cfg.fret_color.linear_multiply(0.7),
                    );
                }

                beat_x += beat_width;
            }

            x += measure_width;

            // Draw bar line
            let bar_y1 = start_y;
            let bar_y2 = start_y + ((num_strings - 1) as f32 * cfg.string_spacing);

            // Draw repeat end
            if measure.repeat_end > 0 {
                painter.line_segment(
                    [Pos2::new(x - 3.0, bar_y1 - 5.0), Pos2::new(x - 3.0, bar_y2 + 5.0)],
                    Stroke::new(3.0, cfg.bar_color),
                );
                painter.line_segment(
                    [Pos2::new(x, bar_y1 - 5.0), Pos2::new(x, bar_y2 + 5.0)],
                    Stroke::new(1.0, cfg.bar_color),
                );
                // Draw dots and repeat count
                let dot_y1 = start_y + (1.5 * cfg.string_spacing);
                let dot_y2 = start_y + (3.5 * cfg.string_spacing);
                painter.circle_filled(Pos2::new(x - 11.0, dot_y1), 3.0, cfg.bar_color);
                painter.circle_filled(Pos2::new(x - 11.0, dot_y2), 3.0, cfg.bar_color);

                if measure.repeat_end > 1 {
                    painter.text(
                        Pos2::new(x + 5.0, bar_y2 + 10.0),
                        egui::Align2::LEFT_TOP,
                        format!("x{}", measure.repeat_end),
                        FontId::proportional(10.0),
                        cfg.fret_color,
                    );
                }
            } else {
                painter.line_segment(
                    [Pos2::new(x, bar_y1), Pos2::new(x, bar_y2)],
                    Stroke::new(1.0, cfg.bar_color),
                );
            }

            // Draw measure number
            painter.text(
                Pos2::new(x - measure_width / 2.0, bar_y2 + 15.0),
                egui::Align2::CENTER_TOP,
                format!("{}", measure.number),
                FontId::proportional(9.0),
                cfg.fret_color.linear_multiply(0.5),
            );
        }
    }

    fn handle_click(&mut self, pos: Pos2, rect: Rect, file: &GuitarProFile, track_idx: usize) {
        let cfg = &self.config;
        let zoom = self.state.zoom;

        let start_x = rect.left() + cfg.left_margin - self.state.scroll_x;
        let mut x = start_x;

        for (measure_idx, measure) in file.measures.iter().enumerate() {
            let track_beats = measure.beats.get(track_idx);
            let beats: &[Beat] = track_beats.map(|tb| tb.beats.as_slice()).unwrap_or(&[]);
            let measure_beats = beats.len().max(4);
            let measure_width = measure_beats as f32 * cfg.beat_width * zoom;

            if pos.x >= x && pos.x < x + measure_width {
                self.state.selected_measure = Some(measure_idx);

                // Find which beat was clicked
                let mut beat_x = x;
                for (beat_idx, beat) in beats.iter().enumerate() {
                    let beat_width = duration_to_width(beat.duration, cfg.beat_width) * zoom;
                    if pos.x >= beat_x && pos.x < beat_x + beat_width {
                        self.state.selected_beat = Some(beat_idx);
                        return;
                    }
                    beat_x += beat_width;
                }

                self.state.selected_beat = None;
                return;
            }

            x += measure_width;
        }

        // Clicked outside all measures
        self.state.selected_measure = None;
        self.state.selected_beat = None;
    }
}

/// Convert MIDI note number to note name
fn midi_note_to_name(note: u8) -> &'static str {
    const NOTES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    NOTES[(note % 12) as usize]
}

/// Convert duration to pixel width
fn duration_to_width(duration: Duration, beat_width: f32) -> f32 {
    match duration {
        Duration::Whole => beat_width * 4.0,
        Duration::Half => beat_width * 2.0,
        Duration::Quarter => beat_width,
        Duration::Eighth => beat_width / 2.0,
        Duration::Sixteenth => beat_width / 4.0,
        Duration::ThirtySecond => beat_width / 8.0,
        Duration::SixtyFourth => beat_width / 16.0,
    }
}
