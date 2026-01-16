/// Track list UI

use crate::app::MaestroApp;
use eframe::egui;

pub fn track_list(app: &mut MaestroApp, ui: &mut egui::Ui) {
    ui.heading("Tracks");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for i in 0..app.track_count {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}", i + 1));
                    ui.text_edit_singleline(&mut app.track_names[i]);
                });
                ui.horizontal(|ui| {
                    if ui.small_button("S").clicked() {
                        app.track_solos[i] = !app.track_solos[i];
                    }
                    if ui.small_button("M").clicked() {
                        app.track_mutes[i] = !app.track_mutes[i];
                    }
                    ui.label(if app.track_mutes[i] { "🔇" } else { "🔊" });
                });
            });
            ui.add_space(4.0);
        }
    });

    if ui.button("+ Add Track").clicked() {
        app.track_count += 1;
        app.track_volumes.push(0.75);
        app.track_pans.push(0.5);
        app.track_mutes.push(false);
        app.track_solos.push(false);
        app.track_names.push(format!("Track {}", app.track_count));
    }
}
