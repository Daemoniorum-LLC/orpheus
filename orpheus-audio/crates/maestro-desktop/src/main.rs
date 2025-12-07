/// Maestro Desktop
/// Native audio workstation with egui GUI

mod app;
mod ui;

use app::MaestroApp;
use eframe::NativeOptions;
use tracing::Level;
use tracing_subscriber;

fn main() -> Result<(), eframe::Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([1200.0, 700.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .unwrap_or_default(),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Maestro - Professional Audio Workstation",
        options,
        Box::new(|cc| Box::new(MaestroApp::new(cc))),
    )
}
