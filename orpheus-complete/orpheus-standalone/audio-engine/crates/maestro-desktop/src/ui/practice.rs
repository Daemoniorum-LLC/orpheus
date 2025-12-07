use crate::app::MaestroApp;
use eframe::egui;

pub fn practice_view(_app: &mut MaestroApp, ui: &mut egui::Ui) {
    ui.heading("Practice View");
    ui.label("Speed trainer and metronome");
}
