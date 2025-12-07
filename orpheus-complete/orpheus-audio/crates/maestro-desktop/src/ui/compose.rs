use crate::app::MaestroApp;
use eframe::egui;

pub fn compose_view(_app: &mut MaestroApp, ui: &mut egui::Ui) {
    ui.heading("Compose View");
    ui.label("Tab editor and score view");
}
