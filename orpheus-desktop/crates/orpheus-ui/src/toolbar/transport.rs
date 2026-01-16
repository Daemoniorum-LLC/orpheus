//! Transport controls

use egui::{Color32, RichText, Ui};
use orpheus_core::TransportState;

/// Transport control bar
pub struct TransportBar;

impl TransportBar {
    /// Show transport controls
    pub fn show(ui: &mut Ui, transport: &mut TransportState, connected: bool) -> TransportAction {
        let mut action = TransportAction::None;

        ui.horizontal(|ui| {
            // Rewind button
            if ui.button("\u{23EE}").on_hover_text("Rewind (Enter)").clicked() {
                action = TransportAction::Rewind;
            }

            // Stop button
            let stop_color = if !transport.is_playing {
                Color32::WHITE
            } else {
                Color32::GRAY
            };
            if ui.button(RichText::new("\u{23F9}").color(stop_color))
                .on_hover_text("Stop (Enter)")
                .clicked()
            {
                action = TransportAction::Stop;
            }

            // Play/Pause button
            let play_icon = if transport.is_playing { "\u{23F8}" } else { "\u{25B6}" };
            let play_color = if transport.is_playing {
                Color32::from_rgb(52, 168, 83)
            } else {
                Color32::WHITE
            };
            if ui.button(RichText::new(play_icon).color(play_color))
                .on_hover_text(if transport.is_playing { "Pause (Space)" } else { "Play (Space)" })
                .clicked()
            {
                action = TransportAction::PlayPause;
            }

            // Record button
            let rec_color = if transport.is_recording {
                Color32::from_rgb(234, 67, 53)
            } else {
                Color32::GRAY
            };
            if ui.button(RichText::new("\u{23FA}").color(rec_color))
                .on_hover_text("Record (R)")
                .clicked()
            {
                action = TransportAction::Record;
            }

            ui.separator();

            // Position display
            let pos_text = transport.position.format_time();
            ui.monospace(RichText::new(&pos_text).size(16.0));

            ui.separator();

            // Bar:Beat display
            let bar_beat = transport.position.format_bars(transport.tempo, transport.time_signature.numerator as u32);
            ui.monospace(&bar_beat);

            ui.separator();

            // Tempo
            ui.label("BPM:");
            let mut tempo = transport.tempo as f32;
            if ui.add(egui::DragValue::new(&mut tempo)
                .range(40.0..=240.0)
                .speed(0.5))
                .on_hover_text("Tempo in beats per minute (drag to adjust)")
                .changed()
            {
                transport.tempo = tempo as f64;
            }

            ui.separator();

            // Loop toggle
            let loop_text = if transport.loop_enabled { "\u{1F501}" } else { "\u{27A1}" };
            if ui.selectable_label(transport.loop_enabled, loop_text)
                .on_hover_text("Toggle Loop (L)")
                .clicked()
            {
                transport.loop_enabled = !transport.loop_enabled;
            }

            // Metronome toggle
            let metro_text = "\u{1F514}";
            if ui.selectable_label(transport.metronome_enabled, metro_text)
                .on_hover_text("Toggle Metronome (M)")
                .clicked()
            {
                transport.metronome_enabled = !transport.metronome_enabled;
                action = TransportAction::MetronomeToggle(transport.metronome_enabled);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Connection status
                let (status_color, status_text) = if connected {
                    (Color32::from_rgb(52, 168, 83), "Connected")
                } else {
                    (Color32::from_rgb(234, 67, 53), "Disconnected")
                };
                ui.label(RichText::new("\u{25CF}").color(status_color).size(10.0));
                ui.label(RichText::new(status_text).size(11.0).color(Color32::GRAY));
            });
        });

        action
    }
}

/// Actions from transport controls
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransportAction {
    None,
    Rewind,
    Stop,
    PlayPause,
    Record,
    MetronomeToggle(bool),
}
