//! Plugin browser panel for loading and managing VST3/CLAP plugins

use egui::{Color32, ScrollArea, Ui};
use std::path::PathBuf;
use uuid::Uuid;

/// Plugin format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PluginFormat {
    Vst3,
    Clap,
}

impl std::fmt::Display for PluginFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginFormat::Vst3 => write!(f, "VST3"),
            PluginFormat::Clap => write!(f, "CLAP"),
        }
    }
}

/// Information about a scanned plugin
#[derive(Debug, Clone)]
pub struct ScannedPlugin {
    pub name: String,
    pub path: PathBuf,
    pub format: PluginFormat,
    pub vendor: Option<String>,
    pub category: Option<String>,
}

/// A plugin parameter with current value and metadata
#[derive(Debug, Clone)]
pub struct PluginParameter {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub unit: Option<String>,
}

impl PluginParameter {
    /// Get the normalized value (0.0 to 1.0)
    pub fn normalized(&self) -> f32 {
        ((self.value - self.min) / (self.max - self.min)) as f32
    }

    /// Set from normalized value (0.0 to 1.0)
    pub fn set_normalized(&mut self, norm: f32) {
        self.value = self.min + (norm as f64) * (self.max - self.min);
    }

    /// Format the value for display
    pub fn display_value(&self) -> String {
        if let Some(ref unit) = self.unit {
            format!("{:.2} {}", self.value, unit)
        } else {
            format!("{:.2}", self.value)
        }
    }
}

/// Information about a loaded plugin instance
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    pub id: Uuid,
    pub name: String,
    pub format: PluginFormat,
    pub active: bool,
    pub bypassed: bool,
    /// Plugin parameters
    pub parameters: Vec<PluginParameter>,
    /// Whether the parameter panel is expanded
    pub expanded: bool,
}

/// Actions that can be triggered from the plugin browser
#[derive(Debug, Clone)]
pub enum PluginBrowserAction {
    /// Request to scan for plugins
    ScanPlugins,
    /// Load a plugin from path
    LoadPlugin(PathBuf),
    /// Unload a plugin by ID
    UnloadPlugin(Uuid),
    /// Toggle plugin bypass
    ToggleBypass(Uuid),
    /// Toggle plugins enabled globally
    TogglePluginsEnabled,
    /// Set a plugin parameter value
    SetParameter { plugin_id: Uuid, param_id: u32, value: f64 },
    /// Toggle plugin parameter panel expansion
    ToggleExpanded(Uuid),
    /// Reset parameter to default
    ResetParameter { plugin_id: Uuid, param_id: u32 },
}

/// State for the plugin browser panel
pub struct PluginBrowserState {
    /// Available plugins from last scan
    pub scanned_plugins: Vec<ScannedPlugin>,
    /// Currently loaded plugin instances
    pub loaded_plugins: Vec<LoadedPlugin>,
    /// Whether plugins are enabled globally
    pub plugins_enabled: bool,
    /// Whether a scan is in progress
    pub scanning: bool,
    /// Search filter
    pub search_filter: String,
    /// Selected format filter
    pub format_filter: Option<PluginFormat>,
    /// Last error message
    pub last_error: Option<String>,
}

impl Default for PluginBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginBrowserState {
    pub fn new() -> Self {
        Self {
            scanned_plugins: Vec::new(),
            loaded_plugins: Vec::new(),
            plugins_enabled: true,
            scanning: false,
            search_filter: String::new(),
            format_filter: None,
            last_error: None,
        }
    }

    /// Get filtered list of scanned plugins
    pub fn filtered_plugins(&self) -> Vec<&ScannedPlugin> {
        self.scanned_plugins
            .iter()
            .filter(|p| {
                // Format filter
                if let Some(fmt) = self.format_filter {
                    if p.format != fmt {
                        return false;
                    }
                }
                // Search filter
                if !self.search_filter.is_empty() {
                    let search = self.search_filter.to_lowercase();
                    let name_match = p.name.to_lowercase().contains(&search);
                    let vendor_match = p.vendor
                        .as_ref()
                        .map(|v| v.to_lowercase().contains(&search))
                        .unwrap_or(false);
                    if !name_match && !vendor_match {
                        return false;
                    }
                }
                true
            })
            .collect()
    }
}

/// Plugin browser panel component
pub struct PluginBrowserPanel<'a> {
    state: &'a mut PluginBrowserState,
}

impl<'a> PluginBrowserPanel<'a> {
    pub fn new(state: &'a mut PluginBrowserState) -> Self {
        Self { state }
    }

    /// Show the plugin browser and return any actions triggered
    pub fn show(&mut self, ui: &mut Ui) -> Option<PluginBrowserAction> {
        let mut action = None;

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading("Plugins");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Global enable toggle
                    let enable_text = if self.state.plugins_enabled { "ON" } else { "OFF" };
                    let enable_color = if self.state.plugins_enabled {
                        Color32::from_rgb(46, 204, 113)
                    } else {
                        Color32::from_rgb(231, 76, 60)
                    };

                    if ui.button(egui::RichText::new(enable_text).color(enable_color)).clicked() {
                        action = Some(PluginBrowserAction::TogglePluginsEnabled);
                    }

                    ui.label("Plugins:");
                });
            });

            ui.separator();

            // Scan button and status
            ui.horizontal(|ui| {
                let scan_text = if self.state.scanning { "Scanning..." } else { "Scan Plugins" };
                if ui.add_enabled(!self.state.scanning, egui::Button::new(scan_text)).clicked() {
                    action = Some(PluginBrowserAction::ScanPlugins);
                }

                ui.label(format!("{} found", self.state.scanned_plugins.len()));
            });

            // Error display
            if let Some(ref error) = self.state.last_error {
                ui.colored_label(Color32::from_rgb(231, 76, 60), error);
            }

            ui.add_space(8.0);

            // Loaded plugins section
            ui.collapsing("Loaded Plugins", |ui| {
                if self.state.loaded_plugins.is_empty() {
                    ui.label(egui::RichText::new("No plugins loaded").italics());
                } else {
                    // We need to collect parameter changes since we can't modify during iteration
                    let mut param_changes: Vec<(Uuid, u32, f64)> = Vec::new();

                    for plugin in &mut self.state.loaded_plugins {
                        let plugin_id = plugin.id;

                        // Plugin header
                        ui.horizontal(|ui| {
                            // Expand/collapse arrow
                            let arrow = if plugin.expanded { "▼" } else { "▶" };
                            if ui.small_button(arrow).clicked() {
                                plugin.expanded = !plugin.expanded;
                            }

                            // Format badge
                            let format_color = match plugin.format {
                                PluginFormat::Vst3 => Color32::from_rgb(52, 152, 219),
                                PluginFormat::Clap => Color32::from_rgb(155, 89, 182),
                            };
                            ui.colored_label(format_color, format!("[{}]", plugin.format));

                            // Plugin name
                            let name_color = if plugin.bypassed {
                                Color32::GRAY
                            } else {
                                Color32::WHITE
                            };
                            ui.colored_label(name_color, &plugin.name);

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Unload button
                                if ui.small_button("✕").on_hover_text("Remove plugin").clicked() {
                                    action = Some(PluginBrowserAction::UnloadPlugin(plugin_id));
                                }

                                // Bypass toggle
                                let bypass_text = if plugin.bypassed { "BYP" } else { "ON" };
                                let bypass_color = if plugin.bypassed {
                                    Color32::from_rgb(241, 196, 15)
                                } else {
                                    Color32::from_rgb(46, 204, 113)
                                };
                                if ui.button(egui::RichText::new(bypass_text).color(bypass_color).small()).clicked() {
                                    action = Some(PluginBrowserAction::ToggleBypass(plugin_id));
                                }
                            });
                        });

                        // Parameter controls (when expanded)
                        if plugin.expanded && !plugin.parameters.is_empty() {
                            ui.indent(plugin_id, |ui| {
                                ui.add_space(4.0);

                                for param in &mut plugin.parameters {
                                    ui.horizontal(|ui| {
                                        // Parameter name
                                        ui.label(egui::RichText::new(&param.name).small());

                                        // Value slider
                                        let mut norm = param.normalized();
                                        let slider = egui::Slider::new(&mut norm, 0.0..=1.0)
                                            .show_value(false)
                                            .clamp_to_range(true);

                                        if ui.add(slider).changed() {
                                            let old_norm = param.normalized();
                                            if (norm - old_norm).abs() > 0.001 {
                                                param.set_normalized(norm);
                                                param_changes.push((plugin_id, param.id, param.value));
                                            }
                                        }

                                        // Value display
                                        ui.label(egui::RichText::new(param.display_value()).small().monospace());

                                        // Reset button
                                        if ui.small_button("↺").on_hover_text("Reset to default").clicked() {
                                            param.value = param.default;
                                            param_changes.push((plugin_id, param.id, param.default));
                                        }
                                    });
                                }

                                ui.add_space(4.0);
                            });
                        } else if plugin.expanded && plugin.parameters.is_empty() {
                            ui.indent(plugin_id, |ui| {
                                ui.label(egui::RichText::new("No parameters available").small().italics());
                            });
                        }

                        ui.separator();
                    }

                    // Apply parameter changes
                    for (plugin_id, param_id, value) in param_changes {
                        if action.is_none() {
                            action = Some(PluginBrowserAction::SetParameter { plugin_id, param_id, value });
                        }
                    }
                }
            });

            ui.add_space(8.0);

            // Available plugins section
            ui.collapsing("Available Plugins", |ui| {
                // Search and filter
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.state.search_filter)
                            .hint_text("Search plugins...")
                            .desired_width(ui.available_width() - 30.0)
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Format:");
                    if ui.selectable_label(self.state.format_filter.is_none(), "All").clicked() {
                        self.state.format_filter = None;
                    }
                    if ui.selectable_label(self.state.format_filter == Some(PluginFormat::Vst3), "VST3").clicked() {
                        self.state.format_filter = Some(PluginFormat::Vst3);
                    }
                    if ui.selectable_label(self.state.format_filter == Some(PluginFormat::Clap), "CLAP").clicked() {
                        self.state.format_filter = Some(PluginFormat::Clap);
                    }
                });

                ui.add_space(4.0);

                // Plugin list
                let filtered = self.state.filtered_plugins();

                if filtered.is_empty() {
                    if self.state.scanned_plugins.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("🔌").size(32.0));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("No Plugins Found").strong());
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("Click 'Scan Plugins' above to search for VST3 and CLAP plugins").italics().color(Color32::GRAY));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("Standard plugin locations will be scanned automatically").small().color(Color32::GRAY));
                        });
                    } else {
                        ui.label(egui::RichText::new("No plugins match the current filter").italics());
                    }
                } else {
                    ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for plugin in filtered {
                                ui.horizontal(|ui| {
                                    // Format badge
                                    let format_color = match plugin.format {
                                        PluginFormat::Vst3 => Color32::from_rgb(52, 152, 219),
                                        PluginFormat::Clap => Color32::from_rgb(155, 89, 182),
                                    };
                                    ui.colored_label(format_color, format!("[{}]", plugin.format));

                                    // Plugin info
                                    ui.vertical(|ui| {
                                        ui.label(&plugin.name);
                                        if let Some(ref vendor) = plugin.vendor {
                                            ui.label(egui::RichText::new(vendor).small().color(Color32::GRAY));
                                        }
                                    });

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("Load").clicked() {
                                            action = Some(PluginBrowserAction::LoadPlugin(plugin.path.clone()));
                                        }
                                    });
                                });
                                ui.separator();
                            }
                        });
                }
            });
        });

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_browser_state_default() {
        let state = PluginBrowserState::new();
        assert!(state.scanned_plugins.is_empty());
        assert!(state.loaded_plugins.is_empty());
        assert!(state.plugins_enabled);
        assert!(!state.scanning);
    }

    #[test]
    fn test_filtered_plugins_no_filter() {
        let mut state = PluginBrowserState::new();
        state.scanned_plugins = vec![
            ScannedPlugin {
                name: "TestPlugin".to_string(),
                path: PathBuf::from("/test.vst3"),
                format: PluginFormat::Vst3,
                vendor: Some("Test Vendor".to_string()),
                category: None,
            },
        ];

        let filtered = state.filtered_plugins();
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filtered_plugins_format_filter() {
        let mut state = PluginBrowserState::new();
        state.scanned_plugins = vec![
            ScannedPlugin {
                name: "VST Plugin".to_string(),
                path: PathBuf::from("/test.vst3"),
                format: PluginFormat::Vst3,
                vendor: None,
                category: None,
            },
            ScannedPlugin {
                name: "CLAP Plugin".to_string(),
                path: PathBuf::from("/test.clap"),
                format: PluginFormat::Clap,
                vendor: None,
                category: None,
            },
        ];

        state.format_filter = Some(PluginFormat::Vst3);
        let filtered = state.filtered_plugins();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "VST Plugin");
    }

    #[test]
    fn test_filtered_plugins_search_filter() {
        let mut state = PluginBrowserState::new();
        state.scanned_plugins = vec![
            ScannedPlugin {
                name: "Compressor".to_string(),
                path: PathBuf::from("/comp.vst3"),
                format: PluginFormat::Vst3,
                vendor: Some("FabFilter".to_string()),
                category: None,
            },
            ScannedPlugin {
                name: "EQ".to_string(),
                path: PathBuf::from("/eq.vst3"),
                format: PluginFormat::Vst3,
                vendor: Some("Waves".to_string()),
                category: None,
            },
        ];

        state.search_filter = "fab".to_string();
        let filtered = state.filtered_plugins();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Compressor");
    }
}
