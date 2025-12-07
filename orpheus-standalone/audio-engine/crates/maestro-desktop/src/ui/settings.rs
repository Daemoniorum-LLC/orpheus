use crate::app::MaestroApp;
use eframe::egui;

pub fn settings_window(_app: &mut MaestroApp, ui: &mut egui::Ui) {
    ui.heading("Settings");
    ui.label("Audio device, buffer size, sample rate");
}
