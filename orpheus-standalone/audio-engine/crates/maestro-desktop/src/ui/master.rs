use crate::app::MaestroApp;
use eframe::egui;

pub fn master_view(_app: &mut MaestroApp, ui: &mut egui::Ui) {
    ui.heading("Master View");
    ui.label("Mastering chain and LUFS meters");
}
