//! Level meter widget with accessibility support

use egui::{Color32, Pos2, Rect, Response, Sense, Ui, Vec2};

/// Audio level meter with color-blind friendly indicators
pub struct LevelMeter {
    /// Current level in dB
    level_db: f32,
    /// Peak level in dB
    peak_db: f32,
    /// Meter height
    height: f32,
    /// Meter width
    width: f32,
    /// Show peak hold
    show_peak: bool,
    /// Show status icons for accessibility
    show_status_icons: bool,
}

impl LevelMeter {
    pub fn new(level_db: f32) -> Self {
        Self {
            level_db,
            peak_db: level_db,
            height: 120.0,
            width: 16.0,
            show_peak: true,
            show_status_icons: true,
        }
    }

    pub fn peak(mut self, peak_db: f32) -> Self {
        self.peak_db = peak_db;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn show_peak(mut self, show: bool) -> Self {
        self.show_peak = show;
        self
    }

    pub fn show_status_icons(mut self, show: bool) -> Self {
        self.show_status_icons = show;
        self
    }

    /// Get the current level status for accessibility
    fn level_status(&self) -> LevelStatus {
        if self.level_db > -3.0 {
            LevelStatus::Clipping
        } else if self.level_db > -6.0 {
            LevelStatus::Hot
        } else if self.level_db > -12.0 {
            LevelStatus::Warm
        } else {
            LevelStatus::Safe
        }
    }

    /// Show the meter
    pub fn show(self, ui: &mut Ui) -> Response {
        let icon_space = if self.show_status_icons { 14.0 } else { 0.0 };
        let desired_size = Vec2::new(self.width, self.height + icon_space);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let meter_rect = Rect::from_min_size(
                rect.min + Vec2::new(0.0, icon_space),
                Vec2::new(rect.width(), rect.height() - icon_space),
            );

            // Background
            painter.rect_filled(meter_rect, 2.0, Color32::from_gray(25));

            // Calculate level position (0dB at top, -60dB at bottom)
            let level_normalized = ((self.level_db + 60.0) / 60.0).clamp(0.0, 1.0);
            let level_height = meter_rect.height() * level_normalized;

            // Gradient colors based on level
            let level_rect = Rect::from_min_size(
                Pos2::new(meter_rect.left() + 1.0, meter_rect.bottom() - level_height),
                Vec2::new(meter_rect.width() - 2.0, level_height),
            );

            // Color based on level
            let status = self.level_status();
            let color = status.color();

            painter.rect_filled(level_rect, 0.0, color);

            // Peak hold indicator
            if self.show_peak {
                let peak_normalized = ((self.peak_db + 60.0) / 60.0).clamp(0.0, 1.0);
                let peak_y = meter_rect.bottom() - (meter_rect.height() * peak_normalized);

                let peak_color = if self.peak_db > -3.0 {
                    Color32::from_rgb(234, 67, 53)
                } else {
                    Color32::WHITE
                };

                painter.line_segment(
                    [
                        Pos2::new(meter_rect.left() + 1.0, peak_y),
                        Pos2::new(meter_rect.right() - 1.0, peak_y),
                    ],
                    egui::Stroke::new(2.0, peak_color),
                );
            }

            // Scale markers
            let marker_color = Color32::from_gray(60);
            for db in [-48, -36, -24, -12, -6, -3, 0].iter() {
                let y = meter_rect.bottom() - (meter_rect.height() * ((*db as f32 + 60.0) / 60.0));
                painter.line_segment(
                    [Pos2::new(meter_rect.left(), y), Pos2::new(meter_rect.left() + 3.0, y)],
                    egui::Stroke::new(1.0, marker_color),
                );
            }

            // Status icon for accessibility (colorblind support)
            if self.show_status_icons && status != LevelStatus::Safe {
                let icon = status.icon();
                let icon_pos = Pos2::new(rect.center().x, rect.top() + 7.0);
                painter.text(
                    icon_pos,
                    egui::Align2::CENTER_CENTER,
                    icon,
                    egui::FontId::proportional(10.0),
                    status.color(),
                );
            }
        }

        // Hover tooltip for accessibility
        let status = self.level_status();
        response.on_hover_text(format!(
            "{:.1} dB ({})",
            self.level_db,
            status.description()
        ))
    }
}

/// Level status for accessibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LevelStatus {
    Safe,
    Warm,
    Hot,
    Clipping,
}

impl LevelStatus {
    fn color(&self) -> Color32 {
        match self {
            LevelStatus::Safe => Color32::from_rgb(52, 168, 83),     // Green
            LevelStatus::Warm => Color32::from_rgb(251, 188, 4),     // Yellow
            LevelStatus::Hot => Color32::from_rgb(255, 152, 0),      // Orange
            LevelStatus::Clipping => Color32::from_rgb(234, 67, 53), // Red
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            LevelStatus::Safe => "",
            LevelStatus::Warm => "~",      // Wave pattern for caution
            LevelStatus::Hot => "▲",       // Triangle for warning
            LevelStatus::Clipping => "!",  // Exclamation for danger
        }
    }

    fn description(&self) -> &'static str {
        match self {
            LevelStatus::Safe => "Safe level",
            LevelStatus::Warm => "Warm - approaching optimal",
            LevelStatus::Hot => "Hot - reduce gain",
            LevelStatus::Clipping => "Clipping! Reduce gain immediately",
        }
    }
}
