//! About dialog

use egui::{Context, Color32, RichText};

/// State for the about dialog
#[derive(Default)]
pub struct AboutState {
    /// Whether the dialog is visible
    pub visible: bool,
}

impl AboutState {
    pub fn new() -> Self {
        Self { visible: false }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }
}

/// About dialog component
pub struct AboutDialog<'a> {
    state: &'a mut AboutState,
}

impl<'a> AboutDialog<'a> {
    pub fn new(state: &'a mut AboutState) -> Self {
        Self { state }
    }

    pub fn show(&mut self, ctx: &Context) {
        if !self.state.visible {
            return;
        }

        let mut open = true;
        egui::Window::new("About Orpheus")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);

                    // App name and version
                    ui.label(
                        RichText::new("Orpheus")
                            .size(28.0)
                            .strong()
                            .color(Color32::from_rgb(52, 152, 219))
                    );
                    ui.label(
                        RichText::new("Version 0.1.0")
                            .size(14.0)
                            .color(Color32::GRAY)
                    );

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Description
                    ui.label("A professional guitar-focused DAW");
                    ui.label("for composition, practice, and distribution.");

                    ui.add_space(16.0);

                    // Features
                    ui.group(|ui| {
                        ui.label(RichText::new("Features").strong());
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label("•");
                            ui.label("Guitar Pro file import (.gp3, .gp4, .gp5)");
                        });
                        ui.horizontal(|ui| {
                            ui.label("•");
                            ui.label("Tablature editor with vim-style navigation");
                        });
                        ui.horizontal(|ui| {
                            ui.label("•");
                            ui.label("Built-in guitar, bass, and drum synthesizers");
                        });
                        ui.horizontal(|ui| {
                            ui.label("•");
                            ui.label("Practice mode with tempo control and looping");
                        });
                        ui.horizontal(|ui| {
                            ui.label("•");
                            ui.label("Multi-track arrangement and mixing");
                        });
                    });

                    ui.add_space(16.0);

                    // Credits
                    ui.label(
                        RichText::new("Daemoniorum LLC")
                            .size(12.0)
                            .color(Color32::GRAY)
                    );

                    ui.add_space(8.0);

                    // Close button
                    if ui.button("Close").clicked() {
                        self.state.visible = false;
                    }

                    ui.add_space(8.0);
                });
            });

        if !open {
            self.state.visible = false;
        }
    }
}
