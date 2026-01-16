/// Mode selector UI

use crate::app::{MaestroApp, Mode};
use eframe::egui;

pub fn mode_selector(app: &mut MaestroApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.selectable_value(&mut app.current_mode, Mode::Compose, "✏️  Compose");
        ui.selectable_value(&mut app.current_mode, Mode::Mix, "🎚️  Mix");
        ui.selectable_value(&mut app.current_mode, Mode::Master, "✨ Master");
        ui.selectable_value(&mut app.current_mode, Mode::Practice, "🎸 Practice");
    });
}
