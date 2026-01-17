//! Fretboard and grid rendering for tab editor
//!
//! Handles drawing the tablature grid, fret numbers, technique symbols.

use egui::{Color32, Pos2, Rect, Rounding, Stroke, Ui, Vec2};
use orpheus_core::tab::{Technique, BendAmount, WhammyTechnique, TapType, PalmMuteIntensity, SlideDirection};

use crate::theme::Theme;
use super::state::{TabEditorState, TabCursor, TabSelection, EditorMode, ActiveTool};

/// Rendering constants
pub const TOOLBAR_HEIGHT: f32 = 36.0;
pub const STRING_HEIGHT: f32 = 24.0;
pub const MIN_STRING_HEIGHT: f32 = 16.0;
pub const MAX_STRING_HEIGHT: f32 = 40.0;
pub const FRET_WIDTH: f32 = 40.0;
pub const MEASURE_HEADER_HEIGHT: f32 = 20.0;
pub const STATUS_HEIGHT: f32 = 24.0;
pub const TRACK_LABEL_WIDTH: f32 = 80.0;

/// Get technique symbol for display
pub fn technique_symbol(technique: &Technique) -> &'static str {
    match technique {
        Technique::HammerOn => "H",
        Technique::PullOff => "P",
        Technique::LegatoSlide(_) => "/",
        Technique::ShiftSlide(_) => "S",
        Technique::SlideIn(_) => "\\",
        Technique::SlideOut(_) => "/",
        Technique::Bend(_) => "b",
        Technique::PreBend(_) => "pb",
        Technique::PreBendRelease(_) => "pbr",
        Technique::UnisonBend(_) => "ub",
        Technique::NaturalHarmonic => "NH",
        Technique::PinchHarmonic => "PH",
        Technique::ArtificialHarmonic(_) => "AH",
        Technique::TapHarmonic(_) => "TH",
        Technique::SemiHarmonic => "SH",
        Technique::FeedbackHarmonic => "FH",
        Technique::Tap(TapType::RightHand) => "T",
        Technique::Tap(TapType::LeftHand) => "LT",
        Technique::Tap(TapType::TwoHand) => "2T",
        Technique::Tap(TapType::EightFinger) => "8T",
        Technique::TremoloPicking => "~",
        Technique::Vibrato(_) => "v",
        Technique::WideVibrato => "V",
        Technique::LetRing => "LR",
        Technique::DeadNote => "X",
        Technique::Staccato => ".",
        Technique::Accent => ">",
        Technique::Marcato => ">>",
        Technique::PalmMute(_) => "PM",
        Technique::WhammyBar(_) => "w",
        Technique::SweepPicking(_) => "sw",
        Technique::EconomyPicking => "ep",
        Technique::HybridPicking => "hp",
        Technique::ChickenPicking => "cp",
        Technique::PickScrape => "ps",
        Technique::Rake => "rk",
        Technique::Slap => "S!",
        Technique::Pop => "P!",
        Technique::Trill(_) => "tr",
        Technique::GraceNote(_) => "g",
        Technique::BehindNutBend(_) => "bn",
        Technique::WhammyPedal(_) => "wp",
        Technique::StringSkip => "ss",
    }
}

/// Get active tool indicator string
pub fn tool_indicator(tool: &ActiveTool) -> String {
    match tool {
        ActiveTool::None => String::new(),
        ActiveTool::HammerOn => "H-ON".to_string(),
        ActiveTool::PullOff => "P-OFF".to_string(),
        ActiveTool::Slide(dir) => format!("SLD {:?}", dir),
        ActiveTool::Bend(amt) => format!("BEND {:?}", amt),
        ActiveTool::Tap(t) => format!("TAP {:?}", t),
        ActiveTool::Harmonic(h) => format!("HARM {:?}", h),
        ActiveTool::Vibrato => "VIB".to_string(),
        ActiveTool::Whammy(w) => format!("WHAM {:?}", w),
        ActiveTool::PalmMute(i) => format!("PM {:?}", i),
        ActiveTool::LetRing => "L.RING".to_string(),
    }
}

/// Fret number color based on position
pub fn fret_color(fret: u8, theme: &Theme) -> Color32 {
    // Inlay positions (3, 5, 7, 9, 12, 15, 17, 19, 21, 24)
    let is_inlay = matches!(fret, 3 | 5 | 7 | 9 | 15 | 17 | 19 | 21);
    let is_double_inlay = matches!(fret, 12 | 24);

    if is_double_inlay {
        theme.palette.accent
    } else if is_inlay {
        theme.palette.accent.gamma_multiply(0.7)
    } else {
        theme.text_primary()
    }
}

/// Draw a single note cell
pub fn draw_note_cell(
    ui: &mut Ui,
    rect: Rect,
    fret: Option<u8>,
    techniques: &[Technique],
    is_cursor: bool,
    is_selected: bool,
    is_rest: bool,
    theme: &Theme,
) {
    let painter = ui.painter();

    // Background
    let bg = if is_cursor {
        theme.palette.accent.gamma_multiply(0.3)
    } else if is_selected {
        theme.palette.selection
    } else {
        Color32::TRANSPARENT
    };

    if bg != Color32::TRANSPARENT {
        painter.rect_filled(rect, Rounding::same(2.0), bg);
    }

    // Cursor border
    if is_cursor {
        painter.rect_stroke(rect.shrink(1.0), Rounding::same(2.0), Stroke::new(2.0, theme.palette.accent));
    }

    // Content
    if let Some(f) = fret {
        // Fret number
        let text = format!("{}", f);
        let color = fret_color(f, theme);

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &text,
            egui::FontId::monospace(14.0),
            color,
        );

        // Technique indicators (small, above or beside)
        if !techniques.is_empty() {
            let tech_text: String = techniques.iter()
                .take(2) // Show max 2 techniques
                .map(technique_symbol)
                .collect::<Vec<_>>()
                .join("");

            painter.text(
                Pos2::new(rect.right() - 2.0, rect.top() + 2.0),
                egui::Align2::RIGHT_TOP,
                &tech_text,
                egui::FontId::proportional(8.0),
                theme.palette.accent,
            );
        }
    } else if is_rest {
        // Rest symbol
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "-",
            egui::FontId::monospace(14.0),
            theme.text_secondary(),
        );
    }
}

/// Draw string lines for the fretboard
pub fn draw_string_lines(
    ui: &mut Ui,
    rect: Rect,
    string_count: u8,
    string_height: f32,
    theme: &Theme,
) {
    let painter = ui.painter();

    for i in 0..string_count {
        let y = rect.min.y + (i as f32 + 0.5) * string_height;

        // String line
        painter.line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
            Stroke::new(1.0, theme.border().gamma_multiply(0.5)),
        );
    }
}

/// Draw measure bar lines
pub fn draw_measure_lines(
    ui: &mut Ui,
    rect: Rect,
    beats_per_measure: usize,
    beat_width: f32,
    theme: &Theme,
) {
    let painter = ui.painter();

    // Start bar line
    painter.line_segment(
        [Pos2::new(rect.min.x, rect.min.y), Pos2::new(rect.min.x, rect.max.y)],
        Stroke::new(2.0, theme.bar_grid()),
    );

    // Beat lines
    for beat in 1..beats_per_measure {
        let x = rect.min.x + beat as f32 * beat_width;
        painter.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            Stroke::new(1.0, theme.beat_grid().gamma_multiply(0.3)),
        );
    }

    // End bar line
    painter.line_segment(
        [Pos2::new(rect.max.x, rect.min.y), Pos2::new(rect.max.x, rect.max.y)],
        Stroke::new(2.0, theme.bar_grid()),
    );
}

/// Draw measure header (number, time signature, etc.)
pub fn draw_measure_header(
    ui: &mut Ui,
    rect: Rect,
    measure_number: usize,
    time_sig: Option<(u8, u8)>,
    has_marker: bool,
    theme: &Theme,
) {
    let painter = ui.painter();

    // Background
    painter.rect_filled(rect, Rounding::ZERO, theme.surface_bg());

    // Measure number
    painter.text(
        Pos2::new(rect.min.x + 4.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        format!("{}", measure_number + 1),
        egui::FontId::proportional(10.0),
        theme.text_secondary(),
    );

    // Time signature if provided
    if let Some((num, denom)) = time_sig {
        painter.text(
            Pos2::new(rect.center().x, rect.center().y),
            egui::Align2::CENTER_CENTER,
            format!("{}/{}", num, denom),
            egui::FontId::proportional(9.0),
            theme.text_muted(),
        );
    }

    // Marker indicator
    if has_marker {
        painter.circle_filled(
            Pos2::new(rect.max.x - 8.0, rect.center().y),
            4.0,
            theme.palette.accent,
        );
    }

    // Bottom border
    painter.line_segment(
        [Pos2::new(rect.min.x, rect.max.y - 1.0), Pos2::new(rect.max.x, rect.max.y - 1.0)],
        Stroke::new(1.0, theme.border()),
    );
}

/// Draw track label on the left side
pub fn draw_track_label(
    ui: &mut Ui,
    rect: Rect,
    name: &str,
    instrument_info: &str,
    string_count: u8,
    string_height: f32,
    theme: &Theme,
) {
    let painter = ui.painter();

    // Background
    painter.rect_filled(rect, Rounding::ZERO, theme.surface_bg());

    // Track name
    painter.text(
        Pos2::new(rect.min.x + 4.0, rect.min.y + 4.0),
        egui::Align2::LEFT_TOP,
        name,
        egui::FontId::proportional(11.0),
        theme.text_primary(),
    );

    // Instrument info
    painter.text(
        Pos2::new(rect.min.x + 4.0, rect.min.y + 18.0),
        egui::Align2::LEFT_TOP,
        instrument_info,
        egui::FontId::proportional(9.0),
        theme.text_secondary(),
    );

    // String labels (tuning)
    let tuning_labels = ["e", "B", "G", "D", "A", "E", "B", "F#", "C#"];
    for i in 0..string_count {
        let y = rect.min.y + MEASURE_HEADER_HEIGHT + (i as f32 + 0.5) * string_height;
        let label = tuning_labels.get(i as usize).unwrap_or(&"?");

        painter.text(
            Pos2::new(rect.max.x - 4.0, y),
            egui::Align2::RIGHT_CENTER,
            *label,
            egui::FontId::monospace(10.0),
            theme.text_muted(),
        );
    }

    // Right border
    painter.line_segment(
        [Pos2::new(rect.max.x - 1.0, rect.min.y), Pos2::new(rect.max.x - 1.0, rect.max.y)],
        Stroke::new(1.0, theme.border()),
    );
}

/// Draw the playhead indicator
pub fn draw_playhead(
    ui: &mut Ui,
    rect: Rect,
    beat_position: f64,
    beat_width: f32,
    theme: &Theme,
) {
    let x = rect.min.x + (beat_position as f32 * beat_width);

    if x >= rect.min.x && x <= rect.max.x {
        ui.painter().line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            Stroke::new(2.0, theme.playhead()),
        );

        // Triangle at top
        let points = vec![
            Pos2::new(x - 6.0, rect.min.y),
            Pos2::new(x + 6.0, rect.min.y),
            Pos2::new(x, rect.min.y + 8.0),
        ];
        ui.painter().add(egui::Shape::convex_polygon(
            points,
            theme.playhead(),
            Stroke::NONE,
        ));
    }
}

/// Draw loop region markers and highlight
///
/// Shows start/end markers and a semi-transparent highlight over the loop region.
pub fn draw_loop_region(
    ui: &mut Ui,
    rect: Rect,
    loop_start_measure: usize,
    loop_end_measure: usize,
    beats_per_measure: usize,
    beat_width: f32,
    theme: &Theme,
    is_practice_mode: bool,
) {
    if !is_practice_mode {
        return;
    }

    let measure_width = beats_per_measure as f32 * beat_width;
    let loop_start_x = rect.min.x + loop_start_measure as f32 * measure_width;
    let loop_end_x = rect.min.x + (loop_end_measure + 1) as f32 * measure_width;

    // Clip to visible rect
    let start_x = loop_start_x.max(rect.min.x);
    let end_x = loop_end_x.min(rect.max.x);

    if start_x >= end_x {
        return; // Loop region not visible
    }

    // Semi-transparent highlight over loop region
    let loop_rect = Rect::from_min_max(
        Pos2::new(start_x, rect.min.y),
        Pos2::new(end_x, rect.max.y),
    );
    let highlight_color = theme.palette.accent.gamma_multiply(0.1);
    ui.painter().rect_filled(loop_rect, Rounding::ZERO, highlight_color);

    // Loop start marker (left bracket)
    if loop_start_x >= rect.min.x && loop_start_x <= rect.max.x {
        let marker_color = theme.palette.success;

        // Vertical line
        ui.painter().line_segment(
            [Pos2::new(loop_start_x, rect.min.y), Pos2::new(loop_start_x, rect.max.y)],
            Stroke::new(3.0, marker_color),
        );

        // Top bracket arm
        ui.painter().line_segment(
            [Pos2::new(loop_start_x, rect.min.y), Pos2::new(loop_start_x + 12.0, rect.min.y)],
            Stroke::new(3.0, marker_color),
        );

        // Bottom bracket arm
        ui.painter().line_segment(
            [Pos2::new(loop_start_x, rect.max.y), Pos2::new(loop_start_x + 12.0, rect.max.y)],
            Stroke::new(3.0, marker_color),
        );

        // "L" label
        ui.painter().text(
            Pos2::new(loop_start_x + 4.0, rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            "L",
            egui::FontId::monospace(10.0),
            marker_color,
        );
    }

    // Loop end marker (right bracket)
    if loop_end_x >= rect.min.x && loop_end_x <= rect.max.x {
        let marker_color = theme.palette.error;

        // Vertical line
        ui.painter().line_segment(
            [Pos2::new(loop_end_x, rect.min.y), Pos2::new(loop_end_x, rect.max.y)],
            Stroke::new(3.0, marker_color),
        );

        // Top bracket arm
        ui.painter().line_segment(
            [Pos2::new(loop_end_x - 12.0, rect.min.y), Pos2::new(loop_end_x, rect.min.y)],
            Stroke::new(3.0, marker_color),
        );

        // Bottom bracket arm
        ui.painter().line_segment(
            [Pos2::new(loop_end_x - 12.0, rect.max.y), Pos2::new(loop_end_x, rect.max.y)],
            Stroke::new(3.0, marker_color),
        );

        // "R" label
        ui.painter().text(
            Pos2::new(loop_end_x - 10.0, rect.min.y + 12.0),
            egui::Align2::RIGHT_TOP,
            "R",
            egui::FontId::monospace(10.0),
            marker_color,
        );
    }
}

/// Draw count-in overlay
///
/// Shows a large beat number in the center of the editor during count-in.
pub fn draw_count_in_overlay(
    ui: &mut Ui,
    rect: Rect,
    current_beat: u8,
    total_beats: u8,
    theme: &Theme,
) {
    if current_beat == 0 {
        return;
    }

    // Semi-transparent background overlay
    let overlay_color = Color32::from_rgba_unmultiplied(0, 0, 0, 180);
    ui.painter().rect_filled(rect, Rounding::ZERO, overlay_color);

    // Large beat number
    let beat_text = format!("{}", current_beat);
    let font_size = (rect.height() * 0.4).min(200.0);

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        &beat_text,
        egui::FontId::proportional(font_size),
        theme.palette.accent,
    );

    // Smaller "Count-in" label above
    ui.painter().text(
        Pos2::new(rect.center().x, rect.center().y - font_size * 0.5 - 20.0),
        egui::Align2::CENTER_CENTER,
        "Count-in",
        egui::FontId::proportional(16.0),
        theme.text_secondary(),
    );

    // Beat progress indicator (dots)
    let dot_radius = 8.0;
    let dot_spacing = 24.0;
    let total_width = (total_beats as f32 - 1.0) * dot_spacing;
    let start_x = rect.center().x - total_width / 2.0;
    let dot_y = rect.center().y + font_size * 0.5 + 30.0;

    for i in 0..total_beats {
        let dot_x = start_x + i as f32 * dot_spacing;
        let dot_color = if i + 1 <= current_beat {
            theme.palette.accent
        } else {
            theme.text_secondary().gamma_multiply(0.3)
        };

        ui.painter().circle_filled(
            Pos2::new(dot_x, dot_y),
            dot_radius,
            dot_color,
        );
    }
}

/// Draw the status bar at the bottom
pub fn draw_status_bar(
    ui: &mut Ui,
    rect: Rect,
    state: &TabEditorState,
    theme: &Theme,
) {
    let painter = ui.painter();

    // Background
    painter.rect_filled(rect, Rounding::ZERO, theme.surface_bg());

    // Top border
    painter.line_segment(
        [Pos2::new(rect.min.x, rect.min.y), Pos2::new(rect.max.x, rect.min.y)],
        Stroke::new(1.0, theme.border()),
    );

    // Mode indicator
    let mode_text = match state.mode {
        EditorMode::Normal => "NORMAL",
        EditorMode::Insert => "INSERT",
        EditorMode::Visual => "VISUAL",
        EditorMode::Command => "COMMAND",
    };

    let mode_color = match state.mode {
        EditorMode::Normal => theme.text_secondary(),
        EditorMode::Insert => theme.palette.success,
        EditorMode::Visual => theme.palette.accent,
        EditorMode::Command => theme.palette.warning,
    };

    painter.text(
        Pos2::new(rect.min.x + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        mode_text,
        egui::FontId::monospace(11.0),
        mode_color,
    );

    // Active tool
    let tool_text = tool_indicator(&state.active_tool);
    if !tool_text.is_empty() {
        painter.text(
            Pos2::new(rect.min.x + 80.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &tool_text,
            egui::FontId::monospace(10.0),
            theme.palette.accent,
        );
    }

    // Duration
    let duration_text = format!("{}", state.current_duration.base.name());
    painter.text(
        Pos2::new(rect.min.x + 160.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &duration_text,
        egui::FontId::monospace(10.0),
        theme.text_secondary(),
    );

    // Cursor position
    let pos_text = format!(
        "M{} B{} S{}",
        state.cursor.measure + 1,
        state.cursor.beat + 1,
        state.cursor.string
    );

    painter.text(
        Pos2::new(rect.center().x, rect.center().y),
        egui::Align2::CENTER_CENTER,
        &pos_text,
        egui::FontId::monospace(10.0),
        theme.text_secondary(),
    );

    // Fret buffer (if digits pending)
    if !state.fret_buffer.digits.is_empty() {
        painter.text(
            Pos2::new(rect.center().x + 100.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &format!("Fret: {}_", state.fret_buffer.digits),
            egui::FontId::monospace(10.0),
            theme.palette.warning,
        );
    }

    // Status message
    painter.text(
        Pos2::new(rect.max.x - 8.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        &state.status,
        egui::FontId::proportional(10.0),
        theme.text_muted(),
    );

    // Help hint
    painter.text(
        Pos2::new(rect.max.x - 200.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        "F1: Help",
        egui::FontId::proportional(9.0),
        theme.text_muted().gamma_multiply(0.5),
    );
}

/// Draw prominent fret buffer overlay near cursor position
///
/// Shows pending multi-digit fret entry (like "1_" when typing 12)
/// positioned above the cursor for immediate visibility.
pub fn draw_fret_buffer_overlay(
    ui: &mut Ui,
    cursor_screen_pos: Pos2,
    digits: &str,
    theme: &Theme,
) {
    if digits.is_empty() {
        return;
    }

    let painter = ui.painter();

    // Position above cursor
    let overlay_pos = Pos2::new(cursor_screen_pos.x, cursor_screen_pos.y - 30.0);

    // Background pill
    let text = format!("{}▏", digits);
    let font = egui::FontId::monospace(18.0);
    let text_size = painter.layout_no_wrap(text.clone(), font.clone(), Color32::WHITE).size();
    let padding = Vec2::new(8.0, 4.0);
    let bg_rect = Rect::from_center_size(overlay_pos, text_size + padding * 2.0);

    painter.rect_filled(bg_rect, Rounding::same(6.0), theme.palette.warning.gamma_multiply(0.9));
    painter.rect_stroke(bg_rect, Rounding::same(6.0), Stroke::new(2.0, Color32::WHITE));

    painter.text(
        overlay_pos,
        egui::Align2::CENTER_CENTER,
        &text,
        font,
        Color32::WHITE,
    );
}

/// Draw prominent mode indicator in corner
///
/// Large colored badge showing current editor mode for visibility.
pub fn draw_mode_indicator(
    ui: &mut Ui,
    rect: Rect,
    mode: EditorMode,
    theme: &Theme,
) {
    let painter = ui.painter();

    let (text, color, bg_color) = match mode {
        EditorMode::Normal => ("NORMAL", Color32::WHITE, theme.text_secondary()),
        EditorMode::Insert => ("INSERT", Color32::WHITE, theme.palette.success),
        EditorMode::Visual => ("VISUAL", Color32::WHITE, theme.palette.accent),
        EditorMode::Command => ("COMMAND", Color32::WHITE, theme.palette.warning),
    };

    // Position in top-left corner of main area
    let badge_size = Vec2::new(80.0, 24.0);
    let badge_pos = Pos2::new(rect.min.x + 8.0, rect.min.y + 8.0);
    let badge_rect = Rect::from_min_size(badge_pos, badge_size);

    painter.rect_filled(badge_rect, Rounding::same(4.0), bg_color);
    painter.text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::monospace(11.0),
        color,
    );
}

/// Draw technique shortcuts help overlay
///
/// Shows all available Shift+key technique shortcuts.
/// Toggle with F1 or ?
pub fn draw_technique_help_overlay(
    ui: &mut Ui,
    rect: Rect,
    theme: &Theme,
) {
    let painter = ui.painter();

    // Semi-transparent background
    painter.rect_filled(rect, Rounding::ZERO, Color32::from_black_alpha(200));

    // Help panel
    let panel_width = 400.0;
    let panel_height = 380.0;
    let panel_rect = Rect::from_center_size(
        rect.center(),
        Vec2::new(panel_width, panel_height),
    );

    painter.rect_filled(panel_rect, Rounding::same(8.0), theme.panel_bg());
    painter.rect_stroke(panel_rect, Rounding::same(8.0), Stroke::new(1.0, theme.border()));

    // Title
    painter.text(
        Pos2::new(panel_rect.center().x, panel_rect.min.y + 24.0),
        egui::Align2::CENTER_CENTER,
        "🎸 Technique Shortcuts",
        egui::FontId::proportional(16.0),
        theme.text_primary(),
    );

    // Subtitle
    painter.text(
        Pos2::new(panel_rect.center().x, panel_rect.min.y + 46.0),
        egui::Align2::CENTER_CENTER,
        "Press Shift + key to toggle technique mode",
        egui::FontId::proportional(11.0),
        theme.text_muted(),
    );

    // Shortcuts list
    let shortcuts = [
        ("Shift+H", "Hammer-on"),
        ("Shift+P", "Pull-off"),
        ("Shift+S", "Slide up"),
        ("Shift+B", "Bend (cycle: ½→1→1½→2)"),
        ("Shift+T", "Tap (right hand)"),
        ("Shift+N", "Natural harmonic"),
        ("Shift+I", "Pinch harmonic"),
        ("Shift+M", "Palm mute (cycle intensity)"),
        ("Shift+V", "Vibrato"),
        ("Shift+W", "Whammy dive bomb"),
        ("Shift+L", "Let ring"),
        ("Shift+X", "Dead note (muted)"),
        ("Shift+G", "Ghost note toggle"),
    ];

    let start_y = panel_rect.min.y + 70.0;
    let line_height = 22.0;
    let key_x = panel_rect.min.x + 30.0;
    let desc_x = panel_rect.min.x + 140.0;

    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let y = start_y + i as f32 * line_height;

        // Key badge
        let key_rect = Rect::from_min_size(
            Pos2::new(key_x, y - 8.0),
            Vec2::new(90.0, 18.0),
        );
        painter.rect_filled(key_rect, Rounding::same(3.0), theme.surface_bg());
        painter.text(
            key_rect.center(),
            egui::Align2::CENTER_CENTER,
            *key,
            egui::FontId::monospace(10.0),
            theme.palette.accent,
        );

        // Description
        painter.text(
            Pos2::new(desc_x, y),
            egui::Align2::LEFT_CENTER,
            *desc,
            egui::FontId::proportional(12.0),
            theme.text_primary(),
        );
    }

    // Footer
    painter.text(
        Pos2::new(panel_rect.center().x, panel_rect.max.y - 20.0),
        egui::Align2::CENTER_CENTER,
        "Press F1 or ? to close",
        egui::FontId::proportional(10.0),
        theme.text_muted(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technique_symbols() {
        assert_eq!(technique_symbol(&Technique::HammerOn), "H");
        assert_eq!(technique_symbol(&Technique::PullOff), "P");
        assert_eq!(technique_symbol(&Technique::NaturalHarmonic), "NH");
        assert_eq!(technique_symbol(&Technique::PinchHarmonic), "PH");
    }
}
