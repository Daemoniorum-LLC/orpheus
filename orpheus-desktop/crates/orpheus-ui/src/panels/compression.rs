//! Compression settings panel for project file compression and dictionary management

use egui::{Color32, ProgressBar, Ui, Vec2};
use std::path::PathBuf;

/// Actions that can be triggered from the compression panel
#[derive(Debug, Clone)]
pub enum CompressionPanelAction {
    /// Train a dictionary from the user's projects
    TrainDictionary {
        /// Paths to project files to train from
        project_paths: Vec<PathBuf>,
        /// Output path for the dictionary
        output_path: PathBuf,
    },
    /// Load a custom dictionary
    LoadDictionary(PathBuf),
    /// Unload the current custom dictionary
    UnloadDictionary,
    /// Set the default compression level for new projects
    SetDefaultLevel(i32),
    /// Toggle auto-compression for project saves
    SetAutoCompress(bool),
    /// Set whether to use dictionary compression by default
    SetUseDictionaryByDefault(bool),
    /// Browse for project files to train dictionary
    BrowseProjectsForTraining,
    /// Browse for dictionary output path
    BrowseDictionaryOutput,
}

/// Compression statistics for display
#[derive(Clone, Default)]
pub struct CompressionStatsDisplay {
    /// Original size in bytes
    pub original_size: u64,
    /// Compressed size in bytes
    pub compressed_size: u64,
    /// Compression ratio (0.0-1.0)
    pub ratio: f64,
    /// Size using dictionary compression
    pub dict_compressed_size: Option<u64>,
    /// Dictionary compression ratio
    pub dict_ratio: Option<f64>,
    /// Improvement from dictionary (percentage)
    pub dict_improvement: Option<f64>,
}

impl CompressionStatsDisplay {
    /// Format file size for display
    pub fn format_size(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Calculate space saved
    pub fn space_saved(&self) -> u64 {
        self.original_size.saturating_sub(self.compressed_size)
    }

    /// Calculate space saved percentage
    pub fn space_saved_percent(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            (1.0 - self.ratio) * 100.0
        }
    }
}

/// State for the compression settings panel
pub struct CompressionPanelState {
    /// Default compression level (1-19)
    pub default_level: i32,
    /// Whether to auto-compress on save
    pub auto_compress: bool,
    /// Whether to use dictionary compression by default
    pub use_dictionary_by_default: bool,
    /// Currently loaded custom dictionary path
    pub custom_dictionary_path: Option<PathBuf>,
    /// Is a dictionary currently loaded?
    pub dictionary_loaded: bool,
    /// Dictionary training in progress
    pub training_in_progress: bool,
    /// Training progress (0.0-1.0)
    pub training_progress: f32,
    /// Training status message
    pub training_status: String,
    /// Project paths selected for training
    pub selected_training_projects: Vec<PathBuf>,
    /// Output path for dictionary
    pub dictionary_output_path: Option<PathBuf>,
    /// Last compression statistics
    pub last_stats: Option<CompressionStatsDisplay>,
    /// Show advanced settings
    pub show_advanced: bool,
    /// Last error message
    pub last_error: Option<String>,
}

impl Default for CompressionPanelState {
    fn default() -> Self {
        Self::new()
    }
}

impl CompressionPanelState {
    pub fn new() -> Self {
        Self {
            default_level: 3,
            auto_compress: true,
            use_dictionary_by_default: true,
            custom_dictionary_path: None,
            dictionary_loaded: false,
            training_in_progress: false,
            training_progress: 0.0,
            training_status: String::new(),
            selected_training_projects: Vec::new(),
            dictionary_output_path: None,
            last_stats: None,
            show_advanced: false,
            last_error: None,
        }
    }

    /// Update training progress
    pub fn update_training_progress(&mut self, progress: f32, status: &str) {
        self.training_progress = progress;
        self.training_status = status.to_string();
    }

    /// Complete training
    pub fn complete_training(&mut self, success: bool, message: &str) {
        self.training_in_progress = false;
        self.training_progress = if success { 1.0 } else { 0.0 };
        self.training_status = message.to_string();
        if !success {
            self.last_error = Some(message.to_string());
        }
    }
}

/// Compression settings panel component
pub struct CompressionPanel<'a> {
    state: &'a mut CompressionPanelState,
}

impl<'a> CompressionPanel<'a> {
    pub fn new(state: &'a mut CompressionPanelState) -> Self {
        Self { state }
    }

    /// Show the compression settings panel and return any actions triggered
    pub fn show(&mut self, ui: &mut Ui) -> Option<CompressionPanelAction> {
        let mut action = None;

        ui.vertical(|ui| {
            ui.heading("Compression Settings");
            ui.separator();

            // Error display
            let mut clear_error = false;
            if let Some(ref error) = self.state.last_error {
                let error_text = error.clone();
                ui.horizontal(|ui| {
                    ui.colored_label(Color32::from_rgb(231, 76, 60), format!("⚠ {}", error_text));
                    if ui.small_button("✕").on_hover_text("Dismiss error").clicked() {
                        clear_error = true;
                    }
                });
                ui.add_space(4.0);
            }
            if clear_error {
                self.state.last_error = None;
            }

            // General settings section
            ui.group(|ui| {
                ui.label(egui::RichText::new("Project Compression").strong());
                ui.add_space(4.0);

                // Auto-compress toggle
                if ui.checkbox(&mut self.state.auto_compress, "Auto-compress on save").changed() {
                    action = Some(CompressionPanelAction::SetAutoCompress(self.state.auto_compress));
                }
                ui.label(
                    egui::RichText::new("Automatically compress project files when saving")
                        .small()
                        .color(Color32::GRAY)
                );

                ui.add_space(4.0);

                // Default compression level
                ui.horizontal(|ui| {
                    ui.label("Default level:");
                    let old_level = self.state.default_level;
                    ui.add(egui::Slider::new(&mut self.state.default_level, 1..=19)
                        .clamp_to_range(true));

                    let level_desc = if self.state.default_level <= 3 {
                        ("Fast", Color32::from_rgb(46, 204, 113))
                    } else if self.state.default_level <= 9 {
                        ("Balanced", Color32::from_rgb(241, 196, 15))
                    } else {
                        ("Max", Color32::from_rgb(231, 76, 60))
                    };
                    ui.colored_label(level_desc.1, level_desc.0);

                    if self.state.default_level != old_level {
                        action = Some(CompressionPanelAction::SetDefaultLevel(self.state.default_level));
                    }
                });
            });

            ui.add_space(8.0);

            // Dictionary section
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Dictionary Compression").strong());

                    let status_color = if self.state.dictionary_loaded {
                        Color32::from_rgb(46, 204, 113)
                    } else {
                        Color32::GRAY
                    };
                    let status_text = if self.state.dictionary_loaded { "Active" } else { "Inactive" };
                    ui.colored_label(status_color, format!("[{}]", status_text));
                });

                ui.add_space(4.0);

                // Use dictionary by default
                if ui.checkbox(&mut self.state.use_dictionary_by_default, "Use dictionary compression by default").changed() {
                    action = Some(CompressionPanelAction::SetUseDictionaryByDefault(self.state.use_dictionary_by_default));
                }
                ui.label(
                    egui::RichText::new("Dictionary compression provides 10-30% better ratios for structured data")
                        .small()
                        .color(Color32::GRAY)
                );

                ui.add_space(4.0);

                // Custom dictionary path
                ui.horizontal(|ui| {
                    ui.label("Dictionary:");
                    let dict_text = self.state.custom_dictionary_path
                        .as_ref()
                        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
                        .unwrap_or_else(|| "(using built-in)".to_string());
                    ui.label(egui::RichText::new(dict_text).monospace());

                    if ui.button("Load...").clicked() {
                        action = Some(CompressionPanelAction::LoadDictionary(PathBuf::new()));
                    }

                    if self.state.custom_dictionary_path.is_some() {
                        if ui.button("Unload").clicked() {
                            action = Some(CompressionPanelAction::UnloadDictionary);
                        }
                    }
                });
            });

            ui.add_space(8.0);

            // Dictionary training section
            ui.group(|ui| {
                ui.label(egui::RichText::new("Train Custom Dictionary").strong());
                ui.add_space(4.0);

                ui.label(
                    egui::RichText::new("Train a dictionary from your projects for better compression")
                        .small()
                        .color(Color32::GRAY)
                );

                ui.add_space(4.0);

                // Selected projects for training
                ui.horizontal(|ui| {
                    ui.label("Projects:");
                    if self.state.selected_training_projects.is_empty() {
                        ui.label(egui::RichText::new("(none selected)").color(Color32::GRAY));
                    } else {
                        ui.label(format!("{} projects selected", self.state.selected_training_projects.len()));
                    }
                    if ui.button("Browse...").clicked() {
                        action = Some(CompressionPanelAction::BrowseProjectsForTraining);
                    }
                });

                // Output path
                ui.horizontal(|ui| {
                    ui.label("Output:");
                    let output_text = self.state.dictionary_output_path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "(not set)".to_string());
                    ui.label(egui::RichText::new(output_text).monospace().small());
                    if ui.button("Browse...").clicked() {
                        action = Some(CompressionPanelAction::BrowseDictionaryOutput);
                    }
                });

                ui.add_space(4.0);

                // Training progress
                if self.state.training_in_progress {
                    ui.horizontal(|ui| {
                        ui.add(ProgressBar::new(self.state.training_progress)
                            .show_percentage());
                        ui.spinner();
                    });
                    ui.label(
                        egui::RichText::new(&self.state.training_status)
                            .small()
                            .color(Color32::GRAY)
                    );
                } else {
                    let can_train = !self.state.selected_training_projects.is_empty()
                        && self.state.dictionary_output_path.is_some();

                    if ui.add_enabled(
                        can_train,
                        egui::Button::new("Train Dictionary").min_size(Vec2::new(120.0, 28.0))
                    ).clicked() {
                        if let Some(output_path) = self.state.dictionary_output_path.clone() {
                            action = Some(CompressionPanelAction::TrainDictionary {
                                project_paths: self.state.selected_training_projects.clone(),
                                output_path,
                            });
                        }
                    }

                    if !can_train {
                        ui.label(
                            egui::RichText::new("Select projects and output path to train")
                                .small()
                                .color(Color32::GRAY)
                        );
                    }
                }
            });

            ui.add_space(8.0);

            // Statistics section
            if let Some(ref stats) = self.state.last_stats {
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Last Compression Statistics").strong());
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Original:");
                            ui.label(
                                egui::RichText::new(CompressionStatsDisplay::format_size(stats.original_size))
                                    .monospace()
                            );
                        });

                        ui.separator();

                        ui.vertical(|ui| {
                            ui.label("Compressed:");
                            ui.label(
                                egui::RichText::new(CompressionStatsDisplay::format_size(stats.compressed_size))
                                    .monospace()
                            );
                        });

                        ui.separator();

                        ui.vertical(|ui| {
                            ui.label("Saved:");
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} ({:.1}%)",
                                    CompressionStatsDisplay::format_size(stats.space_saved()),
                                    stats.space_saved_percent()
                                ))
                                .monospace()
                                .color(Color32::from_rgb(46, 204, 113))
                            );
                        });
                    });

                    // Dictionary comparison if available
                    if let (Some(dict_size), Some(improvement)) = (stats.dict_compressed_size, stats.dict_improvement) {
                        ui.add_space(4.0);
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Dictionary:");
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} ({:+.1}% vs standard)",
                                    CompressionStatsDisplay::format_size(dict_size),
                                    -improvement
                                ))
                                .monospace()
                                .color(if improvement > 0.0 {
                                    Color32::from_rgb(46, 204, 113)
                                } else {
                                    Color32::GRAY
                                })
                            );
                        });
                    }
                });
            }

            // Advanced settings (collapsible)
            ui.add_space(4.0);
            ui.collapsing("Advanced", |ui| {
                ui.label(
                    egui::RichText::new("Compression Level Guide")
                        .strong()
                );

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("1-3:");
                        ui.label("4-9:");
                        ui.label("10-19:");
                    });
                    ui.vertical(|ui| {
                        ui.label("Fast compression, good for quick saves");
                        ui.label("Balanced speed and ratio (recommended)");
                        ui.label("Maximum compression, slower");
                    });
                });

                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new("Dictionary Training")
                        .strong()
                );
                ui.label(
                    egui::RichText::new("Dictionaries are trained on sample data to learn common patterns. \
                        For best results, train on 10+ similar projects with at least 100KB total size.")
                        .small()
                        .color(Color32::GRAY)
                );
            });
        });

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_panel_state_default() {
        let state = CompressionPanelState::new();
        assert_eq!(state.default_level, 3);
        assert!(state.auto_compress);
        assert!(state.use_dictionary_by_default);
        assert!(state.custom_dictionary_path.is_none());
        assert!(!state.dictionary_loaded);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(CompressionStatsDisplay::format_size(500), "500 B");
        assert_eq!(CompressionStatsDisplay::format_size(1536), "1.5 KB");
        assert_eq!(CompressionStatsDisplay::format_size(1572864), "1.5 MB");
        assert_eq!(CompressionStatsDisplay::format_size(1610612736), "1.50 GB");
    }

    #[test]
    fn test_compression_stats() {
        let stats = CompressionStatsDisplay {
            original_size: 1000,
            compressed_size: 300,
            ratio: 0.3,
            dict_compressed_size: Some(250),
            dict_ratio: Some(0.25),
            dict_improvement: Some(16.67),
        };

        assert_eq!(stats.space_saved(), 700);
        assert!((stats.space_saved_percent() - 70.0).abs() < 0.01);
    }

    #[test]
    fn test_update_training_progress() {
        let mut state = CompressionPanelState::new();
        state.training_in_progress = true;

        state.update_training_progress(0.5, "Processing project 5 of 10...");
        assert!((state.training_progress - 0.5).abs() < 0.001);
        assert_eq!(state.training_status, "Processing project 5 of 10...");
    }

    #[test]
    fn test_complete_training() {
        let mut state = CompressionPanelState::new();
        state.training_in_progress = true;

        state.complete_training(true, "Dictionary trained successfully");
        assert!(!state.training_in_progress);
        assert!((state.training_progress - 1.0).abs() < 0.001);
        assert!(state.last_error.is_none());

        state.training_in_progress = true;
        state.complete_training(false, "Training failed: insufficient data");
        assert!(!state.training_in_progress);
        assert!(state.last_error.is_some());
    }
}
