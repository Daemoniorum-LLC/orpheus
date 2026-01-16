/// Transport controls UI

use crate::app::MaestroApp;
use eframe::egui::{self, RichText};

pub fn transport_panel(app: &mut MaestroApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading(RichText::new("🎸 Maestro").size(18.0).strong());

        ui.separator();

        // Play/Pause button
        let play_text = if app.is_playing { "⏸ Pause" } else { "▶ Play" };
        if ui.button(RichText::new(play_text).size(16.0)).clicked() {
            app.toggle_play();
        }

        // Stop button
        if ui.button(RichText::new("⏹ Stop").size(16.0)).clicked() {
            app.stop();
        }

        // Record button
        if ui.button(RichText::new("⏺ Record").size(16.0)).clicked() {
            // TODO: Start recording
        }

        ui.separator();

        // Position display
        let beats = app.current_position as i32;
        let measures = beats / 4 + 1;
        let beat_in_measure = (beats % 4) + 1;
        ui.label(RichText::new(format!("{}:{}", measures, beat_in_measure))
            .size(24.0)
            .monospace());

        ui.separator();

        // BPM control
        ui.label("BPM:");
        ui.add(
            egui::Slider::new(&mut app.current_bpm, 40.0..=240.0)
                .min_decimals(0)
                .max_decimals(0),
        );
        ui.label(RichText::new(format!("{:.0}", app.current_bpm)).monospace());

        ui.separator();

        // Audio service status
        let status_text = if app.audio_client.is_some() {
            RichText::new("🟢 Audio Service Connected").color(egui::Color32::GREEN)
        } else {
            RichText::new("🔴 Audio Service Offline").color(egui::Color32::RED)
        };
        ui.label(status_text);
    });
}
