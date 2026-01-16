/// Effects panel UI

use crate::app::MaestroApp;
use eframe::egui;

pub fn effects_panel(app: &mut MaestroApp, ui: &mut egui::Ui) {
    ui.heading("Effects");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // EQ
        ui.collapsing("Parametric EQ", |ui| {
            for (i, (freq, gain, q)) in app.eq_bands.iter_mut().enumerate() {
                ui.group(|ui| {
                    ui.label(format!("Band {}", i + 1));
                    ui.add(egui::Slider::new(freq, 20.0..=20000.0).text("Freq").logarithmic(true));
                    ui.add(egui::Slider::new(gain, -12.0..=12.0).text("Gain"));
                    ui.add(egui::Slider::new(q, 0.1..=10.0).text("Q"));
                });
            }
        });

        ui.add_space(8.0);

        // Compressor
        ui.collapsing("Compressor", |ui| {
            ui.checkbox(&mut app.compressor_enabled, "Enabled");
            if app.compressor_enabled {
                ui.add(egui::Slider::new(&mut app.compressor_threshold, -60.0..=0.0).text("Threshold"));
                ui.add(egui::Slider::new(&mut app.compressor_ratio, 1.0..=20.0).text("Ratio"));
            }
        });
    });
}
