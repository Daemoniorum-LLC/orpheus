//! Orpheus Desktop Application
//!
//! Professional music production platform for Nyx/Linux.

use anyhow::Result;
use eframe::egui;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

mod app;
mod audio_engine;
mod screenshot;

use app::OrpheusApp;

fn main() -> Result<()> {
    // Initialize logging
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,orpheus=debug")),
        )
        .init();

    info!("Starting Orpheus Desktop v{}", env!("CARGO_PKG_VERSION"));

    // Configure native options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Orpheus - Professional Music Production")
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([1200.0, 700.0])
            .with_icon(load_icon()),
        centered: true,
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Orpheus",
        options,
        Box::new(|cc| Ok(Box::new(OrpheusApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))?;

    Ok(())
}

/// Load application icon
fn load_icon() -> egui::IconData {
    // Create a simple placeholder icon (green circle)
    let size = 64;
    let mut rgba = vec![0u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let dx = x as f32 - size as f32 / 2.0;
            let dy = y as f32 - size as f32 / 2.0;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < size as f32 / 2.0 - 2.0 {
                // Phthalo Green
                rgba[idx] = 26;      // R
                rgba[idx + 1] = 123; // G
                rgba[idx + 2] = 93;  // B
                rgba[idx + 3] = 255; // A
            } else {
                rgba[idx + 3] = 0; // Transparent
            }
        }
    }

    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}
