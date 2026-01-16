//! Rotary knob widget

use egui::{Color32, Pos2, Response, Sense, Ui, Vec2};
use std::f32::consts::PI;

/// Rotary knob control
pub struct Knob<'a> {
    value: &'a mut f32,
    min: f32,
    max: f32,
    default: f32,
    size: f32,
    label: Option<&'a str>,
}

impl<'a> Knob<'a> {
    pub fn new(value: &'a mut f32) -> Self {
        Self {
            value,
            min: 0.0,
            max: 1.0,
            default: 0.5,
            size: 40.0,
            label: None,
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn default_value(mut self, default: f32) -> Self {
        self.default = default;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let label_height = if self.label.is_some() { 16.0 } else { 0.0 };
        let desired_size = Vec2::new(self.size, self.size + label_height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        // Handle drag
        if response.dragged() {
            let delta = response.drag_delta();
            let sensitivity = (self.max - self.min) / 150.0;
            *self.value = (*self.value - delta.y * sensitivity).clamp(self.min, self.max);
        }

        // Double-click to reset
        if response.double_clicked() {
            *self.value = self.default;
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            let knob_center = Pos2::new(
                rect.center().x,
                rect.top() + self.size / 2.0,
            );
            let radius = self.size / 2.0 - 2.0;

            // Background arc
            let arc_radius = radius - 4.0;
            let start_angle = 0.75 * PI;
            let end_angle = 2.25 * PI;

            // Draw background arc
            Self::draw_arc(
                painter,
                knob_center,
                arc_radius,
                start_angle,
                end_angle,
                Color32::from_gray(40),
                3.0,
            );

            // Value arc
            let value_normalized = (*self.value - self.min) / (self.max - self.min);
            let value_angle = start_angle + value_normalized * (end_angle - start_angle);

            Self::draw_arc(
                painter,
                knob_center,
                arc_radius,
                start_angle,
                value_angle,
                Color32::from_rgb(26, 123, 93),
                3.0,
            );

            // Knob body
            painter.circle_filled(knob_center, radius - 6.0, Color32::from_gray(50));
            painter.circle_stroke(
                knob_center,
                radius - 6.0,
                egui::Stroke::new(1.0, Color32::from_gray(70)),
            );

            // Indicator line
            let indicator_inner = radius - 14.0;
            let indicator_outer = radius - 8.0;
            let indicator_start = Pos2::new(
                knob_center.x + value_angle.cos() * indicator_inner,
                knob_center.y + value_angle.sin() * indicator_inner,
            );
            let indicator_end = Pos2::new(
                knob_center.x + value_angle.cos() * indicator_outer,
                knob_center.y + value_angle.sin() * indicator_outer,
            );
            painter.line_segment(
                [indicator_start, indicator_end],
                egui::Stroke::new(2.0, Color32::WHITE),
            );

            // Label
            if let Some(label) = self.label {
                painter.text(
                    Pos2::new(rect.center().x, rect.bottom() - 4.0),
                    egui::Align2::CENTER_BOTTOM,
                    label,
                    egui::FontId::proportional(10.0),
                    Color32::GRAY,
                );
            }

            // Value tooltip on hover
            if response.hovered() {
                let value_text = format!("{:.1}", *self.value);
                painter.text(
                    knob_center,
                    egui::Align2::CENTER_CENTER,
                    &value_text,
                    egui::FontId::proportional(9.0),
                    Color32::WHITE,
                );
            }
        }

        response
    }

    fn draw_arc(
        painter: &egui::Painter,
        center: Pos2,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        color: Color32,
        stroke_width: f32,
    ) {
        let segments = 32;
        let angle_step = (end_angle - start_angle) / segments as f32;

        for i in 0..segments {
            let a1 = start_angle + i as f32 * angle_step;
            let a2 = start_angle + (i + 1) as f32 * angle_step;

            let p1 = Pos2::new(center.x + a1.cos() * radius, center.y + a1.sin() * radius);
            let p2 = Pos2::new(center.x + a2.cos() * radius, center.y + a2.sin() * radius);

            painter.line_segment([p1, p2], egui::Stroke::new(stroke_width, color));
        }
    }
}
