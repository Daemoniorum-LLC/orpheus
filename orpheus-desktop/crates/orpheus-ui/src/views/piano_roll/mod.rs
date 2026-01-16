//! Piano Roll view - MIDI note editing
//!
//! A visual note editor for MIDI clips with piano keyboard, grid, and velocity editing.

use egui::{Color32, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use uuid::Uuid;

use crate::theme::Theme;

/// MIDI note range constants
const MIN_NOTE: u8 = 0;
const MAX_NOTE: u8 = 127;
const DEFAULT_VIEW_LOW: u8 = 36;  // C2
const DEFAULT_VIEW_HIGH: u8 = 84; // C6

/// Piano keyboard width
const KEYBOARD_WIDTH: f32 = 60.0;

/// Note height in pixels
const NOTE_HEIGHT: f32 = 16.0;

/// Minimum note height (zoomed out)
const MIN_NOTE_HEIGHT: f32 = 6.0;

/// Maximum note height (zoomed in)
const MAX_NOTE_HEIGHT: f32 = 32.0;

/// Toolbar height
const TOOLBAR_HEIGHT: f32 = 32.0;

/// Velocity lane height
const VELOCITY_HEIGHT: f32 = 60.0;

/// Pixels per beat at zoom 1.0
const BASE_PIXELS_PER_BEAT: f32 = 40.0;

/// A MIDI note in the piano roll
#[derive(Clone, Debug)]
pub struct PianoRollNote {
    /// Unique note ID
    pub id: Uuid,
    /// MIDI note number (0-127)
    pub note: u8,
    /// Start time in ticks (480 PPQN)
    pub start_ticks: u64,
    /// Duration in ticks
    pub duration_ticks: u64,
    /// Velocity (0-127)
    pub velocity: u8,
    /// Is selected
    pub selected: bool,
    /// Is being dragged
    pub dragging: bool,
}

impl PianoRollNote {
    pub fn new(note: u8, start_ticks: u64, duration_ticks: u64, velocity: u8) -> Self {
        Self {
            id: Uuid::new_v4(),
            note,
            start_ticks,
            duration_ticks,
            velocity,
            selected: false,
            dragging: false,
        }
    }
}

/// Snap resolution for grid
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SnapResolution {
    Off,
    /// Whole note
    Bar,
    /// Quarter note
    Beat,
    /// Eighth note
    Eighth,
    /// Sixteenth note
    Sixteenth,
    /// Thirty-second note
    ThirtySecond,
    /// Triplet eighth
    TripletEighth,
    /// Triplet sixteenth
    TripletSixteenth,
}

impl SnapResolution {
    /// Get snap resolution in ticks (at 480 PPQN)
    pub fn ticks(&self, time_sig_num: u8) -> u64 {
        match self {
            Self::Off => 1,
            Self::Bar => 480 * time_sig_num as u64,
            Self::Beat => 480,
            Self::Eighth => 240,
            Self::Sixteenth => 120,
            Self::ThirtySecond => 60,
            Self::TripletEighth => 160,
            Self::TripletSixteenth => 80,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Bar => "1 Bar",
            Self::Beat => "1/4",
            Self::Eighth => "1/8",
            Self::Sixteenth => "1/16",
            Self::ThirtySecond => "1/32",
            Self::TripletEighth => "1/8T",
            Self::TripletSixteenth => "1/16T",
        }
    }
}

impl Default for SnapResolution {
    fn default() -> Self {
        Self::Sixteenth
    }
}

/// Piano roll tool mode
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PianoRollTool {
    /// Select and move notes
    Select,
    /// Draw new notes
    Draw,
    /// Erase notes
    Erase,
    /// Edit velocity
    Velocity,
}

impl Default for PianoRollTool {
    fn default() -> Self {
        Self::Select
    }
}

/// State for the piano roll view
pub struct PianoRollState {
    /// Notes in the piano roll
    pub notes: Vec<PianoRollNote>,
    /// Clip ID being edited (if any)
    pub clip_id: Option<Uuid>,
    /// Clip name
    pub clip_name: String,
    /// Horizontal zoom (pixels per beat)
    pub zoom_h: f32,
    /// Vertical zoom (note height multiplier)
    pub zoom_v: f32,
    /// Horizontal scroll in ticks
    pub scroll_h: u64,
    /// Vertical scroll (note number at top)
    pub scroll_v: u8,
    /// View low note
    pub view_low: u8,
    /// View high note
    pub view_high: u8,
    /// Current snap resolution
    pub snap: SnapResolution,
    /// Current tool
    pub tool: PianoRollTool,
    /// Tempo BPM
    pub tempo: f64,
    /// Time signature numerator
    pub time_sig_num: u8,
    /// Ticks per quarter note (PPQN)
    pub ppqn: u32,
    /// Show velocity lane
    pub show_velocity: bool,
    /// Grid visibility
    pub show_grid: bool,
    /// Playhead position in ticks
    pub playhead_ticks: u64,
    /// Default note length in ticks
    pub default_note_length: u64,
    /// Default velocity
    pub default_velocity: u8,
    /// Drag state
    drag_state: Option<DragState>,
    /// Is modified
    pub is_modified: bool,
}

/// Drag operation state
#[derive(Clone)]
enum DragState {
    /// Moving notes
    MoveNotes {
        start_pos: Pos2,
        original_positions: Vec<(Uuid, u64, u8)>, // (id, start_ticks, note)
    },
    /// Resizing note from right edge
    ResizeNote {
        note_id: Uuid,
        original_duration: u64,
    },
    /// Drawing a new note
    DrawNote {
        note: u8,
        start_ticks: u64,
    },
    /// Adjusting velocity
    VelocityAdjust {
        note_id: Uuid,
        start_velocity: u8,
    },
    /// Rectangle selection
    RectSelect {
        start: Pos2,
    },
}

impl Default for PianoRollState {
    fn default() -> Self {
        Self::new()
    }
}

impl PianoRollState {
    pub fn new() -> Self {
        Self {
            notes: Vec::new(),
            clip_id: None,
            clip_name: "No Clip".to_string(),
            zoom_h: 1.0,
            zoom_v: 1.0,
            scroll_h: 0,
            scroll_v: DEFAULT_VIEW_HIGH,
            view_low: DEFAULT_VIEW_LOW,
            view_high: DEFAULT_VIEW_HIGH,
            snap: SnapResolution::Sixteenth,
            tool: PianoRollTool::Select,
            tempo: 120.0,
            time_sig_num: 4,
            ppqn: 480,
            show_velocity: true,
            show_grid: true,
            playhead_ticks: 0,
            default_note_length: 480, // Quarter note
            default_velocity: 100,
            drag_state: None,
            is_modified: false,
        }
    }

    /// Load notes from a clip
    pub fn load_clip(&mut self, clip_id: Uuid, name: &str, notes: Vec<PianoRollNote>) {
        self.clip_id = Some(clip_id);
        self.clip_name = name.to_string();
        self.notes = notes;
        self.is_modified = false;

        // Auto-scroll to show notes
        if !self.notes.is_empty() {
            let min_note = self.notes.iter().map(|n| n.note).min().unwrap_or(60);
            let max_note = self.notes.iter().map(|n| n.note).max().unwrap_or(72);
            self.view_low = min_note.saturating_sub(4);
            self.view_high = (max_note + 4).min(127);
            self.scroll_v = self.view_high;
        }
    }

    /// Clear the piano roll
    pub fn clear(&mut self) {
        self.clip_id = None;
        self.clip_name = "No Clip".to_string();
        self.notes.clear();
        self.is_modified = false;
    }

    /// Get selected notes
    pub fn selected_notes(&self) -> Vec<&PianoRollNote> {
        self.notes.iter().filter(|n| n.selected).collect()
    }

    /// Select all notes
    pub fn select_all(&mut self) {
        for note in &mut self.notes {
            note.selected = true;
        }
    }

    /// Deselect all notes
    pub fn deselect_all(&mut self) {
        for note in &mut self.notes {
            note.selected = false;
        }
    }

    /// Delete selected notes
    pub fn delete_selected(&mut self) {
        self.notes.retain(|n| !n.selected);
        self.is_modified = true;
    }

    /// Snap ticks to grid
    pub fn snap_ticks(&self, ticks: u64) -> u64 {
        if self.snap == SnapResolution::Off {
            return ticks;
        }
        let snap_ticks = self.snap.ticks(self.time_sig_num);
        ((ticks + snap_ticks / 2) / snap_ticks) * snap_ticks
    }

    /// Convert ticks to pixels
    pub fn ticks_to_pixels(&self, ticks: u64) -> f32 {
        let beats = ticks as f32 / self.ppqn as f32;
        beats * BASE_PIXELS_PER_BEAT * self.zoom_h
    }

    /// Convert pixels to ticks
    pub fn pixels_to_ticks(&self, pixels: f32) -> u64 {
        let beats = pixels / (BASE_PIXELS_PER_BEAT * self.zoom_h);
        (beats * self.ppqn as f32) as u64
    }

    /// Get note height based on zoom
    pub fn note_height(&self) -> f32 {
        (NOTE_HEIGHT * self.zoom_v).clamp(MIN_NOTE_HEIGHT, MAX_NOTE_HEIGHT)
    }

    /// Get pixels per beat
    pub fn pixels_per_beat(&self) -> f32 {
        BASE_PIXELS_PER_BEAT * self.zoom_h
    }
}

/// Piano Roll action result
#[derive(Clone, Debug)]
pub enum PianoRollAction {
    /// No action
    None,
    /// Note added
    NoteAdded(PianoRollNote),
    /// Notes deleted
    NotesDeleted(Vec<Uuid>),
    /// Notes moved
    NotesMoved(Vec<(Uuid, u64, u8)>), // (id, new_start, new_note)
    /// Note resized
    NoteResized(Uuid, u64), // (id, new_duration)
    /// Velocity changed
    VelocityChanged(Uuid, u8), // (id, new_velocity)
    /// Request to close piano roll
    Close,
    /// Request to save changes
    Save,
}

/// Piano roll view component
pub struct PianoRollView<'a> {
    state: &'a mut PianoRollState,
    theme: &'a Theme,
}

impl<'a> PianoRollView<'a> {
    pub fn new(state: &'a mut PianoRollState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    /// Show the piano roll and return any actions
    pub fn show(&mut self, ui: &mut Ui) -> PianoRollAction {
        let mut action = PianoRollAction::None;

        // Handle keyboard shortcuts
        self.handle_shortcuts(ui, &mut action);

        let available = ui.available_rect_before_wrap();

        // Background
        ui.painter().rect_filled(available, Rounding::ZERO, self.theme.panel_bg());

        // Toolbar
        let toolbar_rect = Rect::from_min_size(
            available.min,
            Vec2::new(available.width(), TOOLBAR_HEIGHT),
        );
        self.draw_toolbar(ui, toolbar_rect, &mut action);

        // Main area
        let main_rect = Rect::from_min_max(
            Pos2::new(available.min.x, toolbar_rect.max.y),
            if self.state.show_velocity {
                Pos2::new(available.max.x, available.max.y - VELOCITY_HEIGHT)
            } else {
                available.max
            },
        );

        // Keyboard area (left)
        let keyboard_rect = Rect::from_min_size(
            main_rect.min,
            Vec2::new(KEYBOARD_WIDTH, main_rect.height()),
        );

        // Note grid area (right)
        let grid_rect = Rect::from_min_max(
            Pos2::new(keyboard_rect.max.x, main_rect.min.y),
            main_rect.max,
        );

        // Draw components
        self.draw_keyboard(ui, keyboard_rect);
        self.draw_grid(ui, grid_rect);
        self.draw_notes(ui, grid_rect, &mut action);
        self.draw_playhead(ui, grid_rect);

        // Velocity lane
        if self.state.show_velocity {
            let velocity_rect = Rect::from_min_max(
                Pos2::new(keyboard_rect.max.x, main_rect.max.y),
                available.max,
            );
            self.draw_velocity_lane(ui, velocity_rect, &mut action);
        }

        // Handle mouse input for note creation/selection
        self.handle_grid_input(ui, grid_rect, &mut action);

        // Handle scrolling
        self.handle_scroll(ui, grid_rect);

        action
    }

    /// Handle keyboard shortcuts
    fn handle_shortcuts(&mut self, ui: &mut Ui, action: &mut PianoRollAction) {
        ui.ctx().input(|i| {
            // Delete selected notes
            if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                let deleted: Vec<Uuid> = self.state.notes.iter()
                    .filter(|n| n.selected)
                    .map(|n| n.id)
                    .collect();
                if !deleted.is_empty() {
                    self.state.delete_selected();
                    *action = PianoRollAction::NotesDeleted(deleted);
                }
            }

            // Select all
            if i.modifiers.ctrl && i.key_pressed(egui::Key::A) {
                self.state.select_all();
            }

            // Tool shortcuts
            if i.key_pressed(egui::Key::S) && !i.modifiers.ctrl {
                self.state.tool = PianoRollTool::Select;
            }
            if i.key_pressed(egui::Key::D) && !i.modifiers.ctrl {
                self.state.tool = PianoRollTool::Draw;
            }
            if i.key_pressed(egui::Key::E) && !i.modifiers.ctrl {
                self.state.tool = PianoRollTool::Erase;
            }

            // Escape to deselect
            if i.key_pressed(egui::Key::Escape) {
                self.state.deselect_all();
            }
        });
    }

    /// Draw the toolbar
    fn draw_toolbar(&mut self, ui: &mut Ui, rect: Rect, action: &mut PianoRollAction) {
        // Background
        ui.painter().rect_filled(rect, Rounding::ZERO, self.theme.surface_bg());

        let mut toolbar_ui = ui.child_ui(rect.shrink(4.0), egui::Layout::left_to_right(egui::Align::Center), None);

        // Clip name
        toolbar_ui.label(
            egui::RichText::new(&self.state.clip_name)
                .strong()
                .color(self.theme.text_primary())
        );

        toolbar_ui.separator();

        // Tool selection
        toolbar_ui.label("Tool:");
        if toolbar_ui.selectable_label(self.state.tool == PianoRollTool::Select, "Select").clicked() {
            self.state.tool = PianoRollTool::Select;
        }
        if toolbar_ui.selectable_label(self.state.tool == PianoRollTool::Draw, "Draw").clicked() {
            self.state.tool = PianoRollTool::Draw;
        }
        if toolbar_ui.selectable_label(self.state.tool == PianoRollTool::Erase, "Erase").clicked() {
            self.state.tool = PianoRollTool::Erase;
        }

        toolbar_ui.separator();

        // Snap resolution
        toolbar_ui.label("Snap:");
        egui::ComboBox::from_id_salt("snap_resolution")
            .selected_text(self.state.snap.label())
            .show_ui(&mut toolbar_ui, |ui| {
                ui.selectable_value(&mut self.state.snap, SnapResolution::Off, "Off");
                ui.selectable_value(&mut self.state.snap, SnapResolution::Bar, "1 Bar");
                ui.selectable_value(&mut self.state.snap, SnapResolution::Beat, "1/4");
                ui.selectable_value(&mut self.state.snap, SnapResolution::Eighth, "1/8");
                ui.selectable_value(&mut self.state.snap, SnapResolution::Sixteenth, "1/16");
                ui.selectable_value(&mut self.state.snap, SnapResolution::ThirtySecond, "1/32");
            }).response.on_hover_text("Snap notes to grid divisions");

        toolbar_ui.separator();

        // Zoom controls
        toolbar_ui.label("H:");
        if toolbar_ui.small_button("-").on_hover_text("Zoom out horizontally").clicked() {
            self.state.zoom_h = (self.state.zoom_h - 0.2).max(0.2);
        }
        toolbar_ui.label(format!("{:.0}%", self.state.zoom_h * 100.0));
        if toolbar_ui.small_button("+").on_hover_text("Zoom in horizontally").clicked() {
            self.state.zoom_h = (self.state.zoom_h + 0.2).min(4.0);
        }

        toolbar_ui.label("V:");
        if toolbar_ui.small_button("-").on_hover_text("Zoom out vertically").clicked() {
            self.state.zoom_v = (self.state.zoom_v - 0.2).max(0.4);
        }
        toolbar_ui.label(format!("{:.0}%", self.state.zoom_v * 100.0));
        if toolbar_ui.small_button("+").on_hover_text("Zoom in vertically").clicked() {
            self.state.zoom_v = (self.state.zoom_v + 0.2).min(2.0);
        }

        toolbar_ui.separator();

        // Velocity lane toggle
        toolbar_ui.checkbox(&mut self.state.show_velocity, "Velocity")
            .on_hover_text("Show velocity editor lane below the piano roll");

        // Right side - close/save buttons
        toolbar_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Close").on_hover_text("Close piano roll editor (Esc)").clicked() {
                *action = PianoRollAction::Close;
            }
            if self.state.is_modified {
                if ui.button("Save").on_hover_text("Save changes (Ctrl+S)").clicked() {
                    *action = PianoRollAction::Save;
                }
            }
        });

        // Bottom border
        ui.painter().line_segment(
            [Pos2::new(rect.min.x, rect.max.y - 1.0), Pos2::new(rect.max.x, rect.max.y - 1.0)],
            Stroke::new(1.0, self.theme.border()),
        );
    }

    /// Draw the piano keyboard
    fn draw_keyboard(&mut self, ui: &mut Ui, rect: Rect) {
        // Background
        ui.painter().rect_filled(rect, Rounding::ZERO, self.theme.surface_bg());

        let note_height = self.state.note_height();
        let visible_notes = (rect.height() / note_height) as u8;

        // Draw keys from top to bottom
        for i in 0..=visible_notes {
            let note = self.state.scroll_v.saturating_sub(i);
            if note < MIN_NOTE {
                break;
            }

            let y = rect.min.y + i as f32 * note_height;
            let key_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, y),
                Vec2::new(rect.width(), note_height),
            );

            // Determine if black key
            let note_in_octave = note % 12;
            let is_black = matches!(note_in_octave, 1 | 3 | 6 | 8 | 10);

            // Key colors
            let key_color = if is_black {
                Color32::from_rgb(40, 40, 40)
            } else {
                Color32::from_rgb(240, 240, 240)
            };

            let text_color = if is_black {
                Color32::from_rgb(200, 200, 200)
            } else {
                Color32::from_rgb(60, 60, 60)
            };

            // Draw key
            ui.painter().rect_filled(key_rect, Rounding::ZERO, key_color);

            // Key label (C notes only, or if zoomed in enough)
            if note_in_octave == 0 || note_height > 18.0 {
                let octave = (note / 12) as i32 - 1;
                let note_name = match note_in_octave {
                    0 => "C",
                    1 => "C#",
                    2 => "D",
                    3 => "D#",
                    4 => "E",
                    5 => "F",
                    6 => "F#",
                    7 => "G",
                    8 => "G#",
                    9 => "A",
                    10 => "A#",
                    11 => "B",
                    _ => "",
                };

                let label = if note_in_octave == 0 {
                    format!("C{}", octave)
                } else if note_height > 18.0 {
                    note_name.to_string()
                } else {
                    continue;
                };

                ui.painter().text(
                    Pos2::new(key_rect.min.x + 4.0, key_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(10.0),
                    text_color,
                );
            }

            // Key border
            ui.painter().line_segment(
                [Pos2::new(rect.min.x, y + note_height - 1.0), Pos2::new(rect.max.x, y + note_height - 1.0)],
                Stroke::new(1.0, Color32::from_rgb(100, 100, 100)),
            );
        }

        // Right border
        ui.painter().line_segment(
            [Pos2::new(rect.max.x - 1.0, rect.min.y), Pos2::new(rect.max.x - 1.0, rect.max.y)],
            Stroke::new(1.0, self.theme.border()),
        );
    }

    /// Draw the note grid
    fn draw_grid(&mut self, ui: &mut Ui, rect: Rect) {
        // Background
        ui.painter().rect_filled(rect, Rounding::ZERO, self.theme.panel_bg());

        if !self.state.show_grid {
            return;
        }

        let note_height = self.state.note_height();
        let pixels_per_beat = self.state.pixels_per_beat();
        let snap_ticks = self.state.snap.ticks(self.state.time_sig_num);

        // Horizontal lines (note rows)
        let visible_notes = (rect.height() / note_height) as u8;
        for i in 0..=visible_notes {
            let note = self.state.scroll_v.saturating_sub(i);
            if note < MIN_NOTE {
                break;
            }

            let y = rect.min.y + i as f32 * note_height;

            // C notes get darker line
            let note_in_octave = note % 12;
            let line_color = if note_in_octave == 0 {
                self.theme.border()
            } else if matches!(note_in_octave, 1 | 3 | 6 | 8 | 10) {
                // Black key row - subtle background
                ui.painter().rect_filled(
                    Rect::from_min_size(Pos2::new(rect.min.x, y), Vec2::new(rect.width(), note_height)),
                    Rounding::ZERO,
                    Color32::from_rgba_unmultiplied(0, 0, 0, 20),
                );
                self.theme.border().gamma_multiply(0.3)
            } else {
                self.theme.border().gamma_multiply(0.3)
            };

            ui.painter().line_segment(
                [Pos2::new(rect.min.x, y + note_height), Pos2::new(rect.max.x, y + note_height)],
                Stroke::new(1.0, line_color),
            );
        }

        // Vertical lines (beats/bars)
        let start_ticks = self.state.scroll_h;
        let visible_ticks = self.state.pixels_to_ticks(rect.width());
        let end_ticks = start_ticks + visible_ticks;

        let bar_ticks = self.state.ppqn as u64 * self.state.time_sig_num as u64;
        let beat_ticks = self.state.ppqn as u64;

        // Bar lines
        let start_bar = start_ticks / bar_ticks;
        let end_bar = end_ticks / bar_ticks + 1;

        for bar in start_bar..=end_bar {
            let ticks = bar * bar_ticks;
            let x = rect.min.x + self.state.ticks_to_pixels(ticks.saturating_sub(start_ticks));

            if x >= rect.min.x && x <= rect.max.x {
                ui.painter().line_segment(
                    [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                    Stroke::new(1.0, self.theme.text_secondary().gamma_multiply(0.5)),
                );
            }

            // Beat lines
            if pixels_per_beat > 10.0 {
                for beat in 1..self.state.time_sig_num {
                    let beat_x = x + self.state.ticks_to_pixels(beat as u64 * beat_ticks);
                    if beat_x >= rect.min.x && beat_x <= rect.max.x {
                        ui.painter().line_segment(
                            [Pos2::new(beat_x, rect.min.y), Pos2::new(beat_x, rect.max.y)],
                            Stroke::new(1.0, self.theme.border().gamma_multiply(0.5)),
                        );
                    }
                }
            }
        }
    }

    /// Draw notes
    fn draw_notes(&mut self, ui: &mut Ui, rect: Rect, _action: &mut PianoRollAction) {
        let note_height = self.state.note_height();
        let start_ticks = self.state.scroll_h;

        // Clone notes to avoid borrow issues
        let notes: Vec<_> = self.state.notes.iter().cloned().collect();

        for note in &notes {
            // Calculate note position
            let note_y_offset = self.state.scroll_v.saturating_sub(note.note) as f32 * note_height;
            let y = rect.min.y + note_y_offset;

            // Skip if not visible vertically
            if y + note_height < rect.min.y || y > rect.max.y {
                continue;
            }

            // Calculate horizontal position
            let x = rect.min.x + self.state.ticks_to_pixels(note.start_ticks.saturating_sub(start_ticks));
            let width = self.state.ticks_to_pixels(note.duration_ticks);

            // Skip if not visible horizontally
            if x + width < rect.min.x || x > rect.max.x {
                continue;
            }

            let note_rect = Rect::from_min_size(
                Pos2::new(x.max(rect.min.x), y),
                Vec2::new((x + width).min(rect.max.x) - x.max(rect.min.x), note_height - 1.0),
            );

            // Note color based on velocity
            let velocity_factor = note.velocity as f32 / 127.0;
            let base_color = Color32::from_rgb(
                (80.0 + 100.0 * velocity_factor) as u8,
                (150.0 + 50.0 * velocity_factor) as u8,
                (220.0) as u8,
            );

            let bg_color = if note.selected {
                base_color.gamma_multiply(1.3)
            } else {
                base_color
            };

            // Draw note
            ui.painter().rect_filled(note_rect, Rounding::same(2.0), bg_color);

            // Selection border
            if note.selected {
                ui.painter().rect_stroke(
                    note_rect,
                    Rounding::same(2.0),
                    Stroke::new(2.0, Color32::WHITE),
                );
            }

            // Note name (if wide enough)
            if note_rect.width() > 30.0 && note_height > 12.0 {
                let note_in_octave = note.note % 12;
                let octave = (note.note / 12) as i32 - 1;
                let note_name = match note_in_octave {
                    0 => "C", 1 => "C#", 2 => "D", 3 => "D#", 4 => "E",
                    5 => "F", 6 => "F#", 7 => "G", 8 => "G#", 9 => "A",
                    10 => "A#", 11 => "B", _ => "",
                };
                ui.painter().text(
                    Pos2::new(note_rect.min.x + 4.0, note_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    format!("{}{}", note_name, octave),
                    egui::FontId::proportional(9.0),
                    Color32::WHITE,
                );
            }
        }
    }

    /// Draw the playhead
    fn draw_playhead(&mut self, ui: &mut Ui, rect: Rect) {
        let x = rect.min.x + self.state.ticks_to_pixels(
            self.state.playhead_ticks.saturating_sub(self.state.scroll_h)
        );

        if x >= rect.min.x && x <= rect.max.x {
            ui.painter().line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(2.0, Color32::from_rgb(231, 76, 60)),
            );
        }
    }

    /// Draw velocity lane
    fn draw_velocity_lane(&mut self, ui: &mut Ui, rect: Rect, _action: &mut PianoRollAction) {
        // Background
        ui.painter().rect_filled(rect, Rounding::ZERO, self.theme.surface_bg());

        // Top border
        ui.painter().line_segment(
            [Pos2::new(rect.min.x, rect.min.y), Pos2::new(rect.max.x, rect.min.y)],
            Stroke::new(1.0, self.theme.border()),
        );

        // Label
        ui.painter().text(
            Pos2::new(rect.min.x - KEYBOARD_WIDTH + 4.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Vel",
            egui::FontId::proportional(10.0),
            self.theme.text_secondary(),
        );

        let start_ticks = self.state.scroll_h;

        // Draw velocity bars for each note
        for note in &self.state.notes {
            let x = rect.min.x + self.state.ticks_to_pixels(note.start_ticks.saturating_sub(start_ticks));

            if x < rect.min.x || x > rect.max.x {
                continue;
            }

            let bar_width = 8.0_f32.max(self.state.ticks_to_pixels(note.duration_ticks) / 4.0);
            let bar_height = (note.velocity as f32 / 127.0) * (rect.height() - 4.0);

            let bar_rect = Rect::from_min_size(
                Pos2::new(x, rect.max.y - bar_height - 2.0),
                Vec2::new(bar_width, bar_height),
            );

            // Color based on velocity
            let velocity_color = Color32::from_rgb(
                (50.0 + 150.0 * (note.velocity as f32 / 127.0)) as u8,
                180,
                220,
            );

            let color = if note.selected {
                velocity_color.gamma_multiply(1.3)
            } else {
                velocity_color
            };

            ui.painter().rect_filled(bar_rect, Rounding::same(2.0), color);
        }
    }

    /// Handle grid input for note creation/selection
    fn handle_grid_input(&mut self, ui: &mut Ui, rect: Rect, action: &mut PianoRollAction) {
        let response = ui.allocate_rect(rect, Sense::click_and_drag());

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let (note, ticks) = self.pos_to_note_and_ticks(pos, rect);

                match self.state.tool {
                    PianoRollTool::Select => {
                        // Click to select/deselect note
                        if !ui.ctx().input(|i| i.modifiers.shift) {
                            self.state.deselect_all();
                        }

                        // Check if clicking on a note
                        let clicked_note = self.find_note_at(note, ticks);
                        if let Some(id) = clicked_note {
                            if let Some(n) = self.state.notes.iter_mut().find(|n| n.id == id) {
                                n.selected = !n.selected;
                            }
                        }
                    }
                    PianoRollTool::Draw => {
                        // Create a new note
                        let snapped_ticks = self.state.snap_ticks(ticks);
                        let new_note = PianoRollNote::new(
                            note,
                            snapped_ticks,
                            self.state.default_note_length,
                            self.state.default_velocity,
                        );
                        *action = PianoRollAction::NoteAdded(new_note.clone());
                        self.state.notes.push(new_note);
                        self.state.is_modified = true;
                    }
                    PianoRollTool::Erase => {
                        // Erase note at position
                        if let Some(id) = self.find_note_at(note, ticks) {
                            self.state.notes.retain(|n| n.id != id);
                            *action = PianoRollAction::NotesDeleted(vec![id]);
                            self.state.is_modified = true;
                        }
                    }
                    PianoRollTool::Velocity => {
                        // Would handle velocity adjustment
                    }
                }
            }
        }

        // Double-click to create note in select mode
        if response.double_clicked() && self.state.tool == PianoRollTool::Select {
            if let Some(pos) = response.interact_pointer_pos() {
                let (note, ticks) = self.pos_to_note_and_ticks(pos, rect);

                // Only create if not clicking on existing note
                if self.find_note_at(note, ticks).is_none() {
                    let snapped_ticks = self.state.snap_ticks(ticks);
                    let new_note = PianoRollNote::new(
                        note,
                        snapped_ticks,
                        self.state.default_note_length,
                        self.state.default_velocity,
                    );
                    *action = PianoRollAction::NoteAdded(new_note.clone());
                    self.state.notes.push(new_note);
                    self.state.is_modified = true;
                }
            }
        }
    }

    /// Convert screen position to note number and ticks
    fn pos_to_note_and_ticks(&self, pos: Pos2, rect: Rect) -> (u8, u64) {
        let note_height = self.state.note_height();
        let y_offset = (pos.y - rect.min.y) / note_height;
        let note = self.state.scroll_v.saturating_sub(y_offset as u8);

        let x_offset = pos.x - rect.min.x;
        let ticks = self.state.pixels_to_ticks(x_offset) + self.state.scroll_h;

        (note.clamp(MIN_NOTE, MAX_NOTE), ticks)
    }

    /// Find note at position
    fn find_note_at(&self, note: u8, ticks: u64) -> Option<Uuid> {
        self.state.notes.iter()
            .find(|n| {
                n.note == note &&
                ticks >= n.start_ticks &&
                ticks < n.start_ticks + n.duration_ticks
            })
            .map(|n| n.id)
    }

    /// Handle scrolling
    fn handle_scroll(&mut self, ui: &mut Ui, rect: Rect) {
        let response = ui.allocate_rect(rect, Sense::hover());

        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta);

            // Horizontal scroll
            if scroll.x != 0.0 {
                let ticks_delta = self.state.pixels_to_ticks(scroll.x.abs());
                if scroll.x > 0.0 {
                    self.state.scroll_h = self.state.scroll_h.saturating_sub(ticks_delta);
                } else {
                    self.state.scroll_h += ticks_delta;
                }
            }

            // Vertical scroll
            if scroll.y != 0.0 {
                let note_delta = (scroll.y / self.state.note_height()).abs() as u8;
                if scroll.y > 0.0 {
                    self.state.scroll_v = (self.state.scroll_v + note_delta).min(MAX_NOTE);
                } else {
                    self.state.scroll_v = self.state.scroll_v.saturating_sub(note_delta);
                }
            }

            // Zoom with Ctrl+scroll
            ui.input(|i| {
                if i.modifiers.ctrl && i.smooth_scroll_delta.y != 0.0 {
                    let zoom_delta = i.smooth_scroll_delta.y * 0.01;
                    self.state.zoom_h = (self.state.zoom_h + zoom_delta).clamp(0.2, 4.0);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_resolution_ticks() {
        assert_eq!(SnapResolution::Beat.ticks(4), 480);
        assert_eq!(SnapResolution::Eighth.ticks(4), 240);
        assert_eq!(SnapResolution::Sixteenth.ticks(4), 120);
        assert_eq!(SnapResolution::Bar.ticks(4), 1920);
    }

    #[test]
    fn test_piano_roll_state_snap() {
        let state = PianoRollState::new();
        assert_eq!(state.snap_ticks(0), 0);
        assert_eq!(state.snap_ticks(60), 120);  // Rounds to nearest 16th
        assert_eq!(state.snap_ticks(100), 120);
        assert_eq!(state.snap_ticks(480), 480);
    }

    #[test]
    fn test_note_creation() {
        let note = PianoRollNote::new(60, 0, 480, 100);
        assert_eq!(note.note, 60);
        assert_eq!(note.start_ticks, 0);
        assert_eq!(note.duration_ticks, 480);
        assert_eq!(note.velocity, 100);
        assert!(!note.selected);
    }
}
