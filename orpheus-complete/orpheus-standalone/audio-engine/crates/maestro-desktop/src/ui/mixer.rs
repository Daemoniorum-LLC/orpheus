use crate::app::MaestroApp;
use eframe::egui;

pub fn mixer_view(app: &mut MaestroApp, ui: &mut egui::Ui) {
    ui.heading("Mixer View");
    ui.horizontal(|ui| {
        for i in 0..app.track_count {
            ui.vertical(|ui| {
                ui.label(&app.track_names[i]);
                ui.add(egui::Slider::new(&mut app.track_volumes[i], 0.0..=1.0).vertical());
                ui.label(format!("{:.0}%", app.track_volumes[i] * 100.0));
                ui.add(egui::Slider::new(&mut app.track_pans[i], 0.0..=1.0).text("Pan"));
            });
        }
    });
}
