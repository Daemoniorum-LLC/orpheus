/// Main application state and logic

use crate::ui;
use eframe::egui;
use maestro_proto::audio::audio_processor_client::AudioProcessorClient;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Channel;
use tracing::{error, info};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Compose,
    Mix,
    Master,
    Practice,
}

pub struct MaestroApp {
    // Mode
    current_mode: Mode,

    // Transport
    is_playing: bool,
    current_bpm: f32,
    current_position: f32, // In beats

    // Tracks
    track_count: usize,
    track_volumes: Vec<f32>,
    track_pans: Vec<f32>,
    track_mutes: Vec<bool>,
    track_solos: Vec<bool>,
    track_names: Vec<String>,

    // Effects
    eq_bands: Vec<(f32, f32, f32)>, // freq, gain, q
    compressor_enabled: bool,
    compressor_threshold: f32,
    compressor_ratio: f32,

    // Audio client (optional - may not be connected)
    audio_client: Option<Arc<RwLock<AudioProcessorClient<Channel>>>>,

    // UI state
    show_settings: bool,
    show_effects: bool,
}

impl MaestroApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure fonts
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "monospace".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/JetBrainsMono-Regular.ttf"))
                .unwrap_or(egui::FontData::default()),
        );

        cc.egui_ctx.set_fonts(fonts);

        // Try to connect to audio service
        let audio_client = match Self::connect_audio_service() {
            Ok(client) => Some(Arc::new(RwLock::new(client))),
            Err(e) => {
                error!("Failed to connect to audio service: {}", e);
                None
            }
        };

        Self {
            current_mode: Mode::Compose,
            is_playing: false,
            current_bpm: 120.0,
            current_position: 0.0,

            track_count: 8,
            track_volumes: vec![0.75; 8],
            track_pans: vec![0.5; 8],
            track_mutes: vec![false; 8],
            track_solos: vec![false; 8],
            track_names: vec![
                "Lead Guitar".to_string(),
                "Rhythm Guitar".to_string(),
                "Bass".to_string(),
                "Drums".to_string(),
                "Keys".to_string(),
                "Vocals".to_string(),
                "Synth".to_string(),
                "FX".to_string(),
            ],

            eq_bands: vec![
                (100.0, 0.0, 1.0),
                (400.0, 0.0, 1.0),
                (2000.0, 0.0, 1.0),
                (8000.0, 0.0, 1.0),
            ],

            compressor_enabled: false,
            compressor_threshold: -20.0,
            compressor_ratio: 4.0,

            audio_client,

            show_settings: false,
            show_effects: true,
        }
    }

    fn connect_audio_service() -> Result<AudioProcessorClient<Channel>, Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let client = AudioProcessorClient::connect("http://localhost:50051").await?;
            info!("Connected to audio service");
            Ok(client)
        })
    }

    fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
        info!("Playback: {}", if self.is_playing { "started" } else { "stopped" });
    }

    fn stop(&mut self) {
        self.is_playing = false;
        self.current_position = 0.0;
        info!("Playback stopped and reset");
    }
}

impl eframe::App for MaestroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaint for 60 FPS
        ctx.request_repaint();

        // Update position if playing
        if self.is_playing {
            self.current_position += self.current_bpm / 60.0 / 60.0; // Increment by BPM/60/60 per frame
        }

        // Top panel - Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project").clicked() {
                        info!("New project");
                    }
                    if ui.button("Open...").clicked() {
                        info!("Open project");
                    }
                    if ui.button("Save").clicked() {
                        info!("Save project");
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_effects, "Effects Panel");
                    ui.checkbox(&mut self.show_settings, "Settings");
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Documentation").clicked() {
                        info!("Open documentation");
                    }
                    if ui.button("About").clicked() {
                        info!("Show about");
                    }
                });
            });
        });

        // Top panel - Transport controls
        egui::TopBottomPanel::top("transport").show(ctx, |ui| {
            ui::transport_panel(self, ui);
        });

        // Top panel - Mode selector
        egui::TopBottomPanel::top("modes").show(ctx, |ui| {
            ui::mode_selector(self, ui);
        });

        // Left panel - Track list
        egui::SidePanel::left("tracks")
            .min_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui::track_list(self, ui);
            });

        // Right panel - Effects
        if self.show_effects {
            egui::SidePanel::right("effects")
                .min_width(350.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui::effects_panel(self, ui);
                });
        }

        // Central panel - Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_mode {
                Mode::Compose => ui::compose_view(self, ui),
                Mode::Mix => ui::mixer_view(self, ui),
                Mode::Master => ui::master_view(self, ui),
                Mode::Practice => ui::practice_view(self, ui),
            }
        });

        // Settings window
        if self.show_settings {
            egui::Window::new("Settings")
                .open(&mut self.show_settings)
                .resizable(true)
                .default_width(600.0)
                .show(ctx, |ui| {
                    ui::settings_window(self, ui);
                });
        }
    }
}
