//! Synth Preset Browser Panel
//!
//! Allows browsing and selecting presets for built-in synthesizers:
//! Piano, Bass, Guitar, and Drums.

use egui::{Color32, RichText, Rounding, Stroke, Ui, Vec2};

/// Synth category for preset browsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynthCategory {
    Piano,
    Bass,
    Guitar,
    Drums,
}

impl SynthCategory {
    pub fn all() -> &'static [SynthCategory] {
        &[
            SynthCategory::Piano,
            SynthCategory::Bass,
            SynthCategory::Guitar,
            SynthCategory::Drums,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            SynthCategory::Piano => "Piano",
            SynthCategory::Bass => "Bass",
            SynthCategory::Guitar => "Guitar",
            SynthCategory::Drums => "Drums",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SynthCategory::Piano => "🎹",
            SynthCategory::Bass => "🎸",
            SynthCategory::Guitar => "🎸",
            SynthCategory::Drums => "🥁",
        }
    }

    pub fn presets(&self) -> Vec<SynthPreset> {
        match self {
            SynthCategory::Piano => vec![
                SynthPreset::new("Grand Piano", "Bright, full sound", PianoPresetType::Standard),
                SynthPreset::new("Studio Piano", "Warm, intimate", PianoPresetType::Standard),
                SynthPreset::new("Honky Tonk", "Detuned, vintage", PianoPresetType::Standard),
                SynthPreset::new("Electric Piano", "Rhodes-style", PianoPresetType::Standard),
            ],
            SynthCategory::Bass => vec![
                SynthPreset::new("Synth Bass", "Classic analog", BassPresetType::SynthBass),
                SynthPreset::new("Sub Bass", "Deep low end", BassPresetType::SubBass),
                SynthPreset::new("Funk Bass", "Bright & punchy", BassPresetType::FunkBass),
                SynthPreset::new("808 Bass", "Hip-hop style", BassPresetType::Bass808),
                SynthPreset::new("Wobble Bass", "Dubstep", BassPresetType::WobbleBass),
                SynthPreset::new("Moog Bass", "Fat & warm", BassPresetType::MoogBass),
                SynthPreset::new("Reese Bass", "Detuned saws", BassPresetType::ReeseBass),
            ],
            SynthCategory::Guitar => vec![
                SynthPreset::new("Acoustic", "Steel string", GuitarPresetType::Acoustic),
                SynthPreset::new("Electric Clean", "Sparkly clean", GuitarPresetType::ElectricClean),
                SynthPreset::new("Nylon", "Classical", GuitarPresetType::Nylon),
                SynthPreset::new("Bass Guitar", "4-string", GuitarPresetType::Bass),
                SynthPreset::new("7-String Metal", "Drop A tuning", GuitarPresetType::SevenStringMetal),
                SynthPreset::new("8-String Metal", "Extended range", GuitarPresetType::EightStringMetal),
                SynthPreset::new("Tech Death", "Brutal tones", GuitarPresetType::TechDeath),
            ],
            SynthCategory::Drums => vec![
                SynthPreset::new("Rock Kit", "Classic rock", DrumPresetType::Default),
                SynthPreset::new("Metal Kit", "Punchy & tight", DrumPresetType::Metal),
                SynthPreset::new("Electronic", "808/909 style", DrumPresetType::Default),
            ],
        }
    }
}

/// Piano preset types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PianoPresetType {
    Standard,
}

/// Bass preset types (matches orpheus_synth::BassPreset)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BassPresetType {
    SynthBass,
    SubBass,
    FunkBass,
    Bass808,
    WobbleBass,
    MoogBass,
    ReeseBass,
}

/// Guitar preset types (matches GuitarConfig constructors)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuitarPresetType {
    Acoustic,
    ElectricClean,
    Nylon,
    Bass,
    SevenStringMetal,
    EightStringMetal,
    TechDeath,
}

/// Drum preset types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumPresetType {
    Default,
    Metal,
}

/// Preset type that can be any synth category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetType {
    Piano(PianoPresetType),
    Bass(BassPresetType),
    Guitar(GuitarPresetType),
    Drums(DrumPresetType),
}

/// A synth preset
#[derive(Debug, Clone)]
pub struct SynthPreset {
    pub name: String,
    pub description: String,
    pub preset_type: PresetType,
}

impl SynthPreset {
    fn new(name: &str, description: &str, preset: impl Into<PresetType>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            preset_type: preset.into(),
        }
    }
}

impl From<PianoPresetType> for PresetType {
    fn from(p: PianoPresetType) -> Self {
        PresetType::Piano(p)
    }
}

impl From<BassPresetType> for PresetType {
    fn from(p: BassPresetType) -> Self {
        PresetType::Bass(p)
    }
}

impl From<GuitarPresetType> for PresetType {
    fn from(p: GuitarPresetType) -> Self {
        PresetType::Guitar(p)
    }
}

impl From<DrumPresetType> for PresetType {
    fn from(p: DrumPresetType) -> Self {
        PresetType::Drums(p)
    }
}

/// State for the synth preset browser
#[derive(Debug, Clone)]
pub struct SynthPresetState {
    /// Currently selected category
    pub selected_category: SynthCategory,
    /// Currently selected preset name per category
    pub selected_presets: [Option<String>; 4],
    /// Search filter
    pub search_filter: String,
    /// Show favorites only
    pub favorites_only: bool,
    /// Favorite preset names
    pub favorites: Vec<String>,
}

impl SynthPresetState {
    pub fn new() -> Self {
        Self {
            selected_category: SynthCategory::Piano,
            selected_presets: [
                Some("Grand Piano".to_string()),
                Some("Synth Bass".to_string()),
                Some("Acoustic".to_string()),
                Some("Rock Kit".to_string()),
            ],
            search_filter: String::new(),
            favorites_only: false,
            favorites: Vec::new(),
        }
    }

    /// Get the selected preset for current category
    pub fn current_preset(&self) -> Option<&String> {
        let idx = self.selected_category as usize;
        self.selected_presets[idx].as_ref()
    }

    /// Set the selected preset for current category
    pub fn set_preset(&mut self, name: String) {
        let idx = self.selected_category as usize;
        self.selected_presets[idx] = Some(name);
    }

    /// Toggle favorite status for a preset
    pub fn toggle_favorite(&mut self, name: &str) {
        if let Some(pos) = self.favorites.iter().position(|f| f == name) {
            self.favorites.remove(pos);
        } else {
            self.favorites.push(name.to_string());
        }
    }

    /// Check if a preset is a favorite
    pub fn is_favorite(&self, name: &str) -> bool {
        self.favorites.iter().any(|f| f == name)
    }
}

impl Default for SynthPresetState {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions that can be triggered by the synth preset browser
#[derive(Debug, Clone)]
pub enum SynthPresetAction {
    /// Select a preset (category, preset_type)
    SelectPreset(PresetType),
    /// Toggle favorite status
    ToggleFavorite(String),
}

/// Synth preset browser panel
pub struct SynthPresetPanel<'a> {
    state: &'a mut SynthPresetState,
}

impl<'a> SynthPresetPanel<'a> {
    pub fn new(state: &'a mut SynthPresetState) -> Self {
        Self { state }
    }

    /// Show the panel and return any action
    pub fn show(&mut self, ui: &mut Ui) -> Option<SynthPresetAction> {
        let mut action = None;

        // Header
        ui.horizontal(|ui| {
            ui.heading("🎛 Synth Presets");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Favorites toggle
                let fav_icon = if self.state.favorites_only { "★" } else { "☆" };
                if ui.selectable_label(self.state.favorites_only, fav_icon).clicked() {
                    self.state.favorites_only = !self.state.favorites_only;
                }
            });
        });

        ui.separator();

        // Category tabs
        ui.horizontal(|ui| {
            for category in SynthCategory::all() {
                let selected = self.state.selected_category == *category;
                let text = format!("{} {}", category.icon(), category.name());

                if ui.selectable_label(selected, text).clicked() {
                    self.state.selected_category = *category;
                }
            }
        });

        ui.separator();

        // Search bar
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(
                egui::TextEdit::singleline(&mut self.state.search_filter)
                    .hint_text("Search presets...")
                    .desired_width(ui.available_width() - 40.0)
            );
            if !self.state.search_filter.is_empty() {
                if ui.small_button("✕").on_hover_text("Clear search").clicked() {
                    self.state.search_filter.clear();
                }
            }
        });

        ui.add_space(4.0);

        // Preset list
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let presets = self.state.selected_category.presets();
                let filter = self.state.search_filter.to_lowercase();
                let current_preset = self.state.current_preset().cloned();

                // Count matching presets for empty state
                let matching_count = presets.iter().filter(|p| {
                    let matches_filter = filter.is_empty() || p.name.to_lowercase().contains(&filter);
                    let matches_favorites = !self.state.favorites_only || self.state.is_favorite(&p.name);
                    matches_filter && matches_favorites
                }).count();

                if matching_count == 0 {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        if self.state.favorites_only && filter.is_empty() {
                            ui.label(RichText::new("No Favorites").strong().size(16.0));
                            ui.add_space(8.0);
                            ui.label(RichText::new("Click the ☆ icon on any preset to add it to favorites").weak());
                        } else if !filter.is_empty() {
                            ui.label(RichText::new("No Matching Presets").strong().size(16.0));
                            ui.add_space(8.0);
                            ui.label(RichText::new(format!("No presets match '{}'", self.state.search_filter)).weak());
                        }
                    });
                    return;
                }

                for preset in presets {
                    // Apply filters
                    if !filter.is_empty() && !preset.name.to_lowercase().contains(&filter) {
                        continue;
                    }
                    if self.state.favorites_only && !self.state.is_favorite(&preset.name) {
                        continue;
                    }

                    let is_selected = current_preset.as_ref() == Some(&preset.name);
                    let is_favorite = self.state.is_favorite(&preset.name);

                    // Preset card
                    let response = ui.allocate_ui(Vec2::new(ui.available_width(), 50.0), |ui| {
                        let rect = ui.available_rect_before_wrap();

                        // Background
                        let bg_color = if is_selected {
                            Color32::from_rgb(60, 80, 120)
                        } else if ui.rect_contains_pointer(rect) {
                            Color32::from_gray(50)
                        } else {
                            Color32::from_gray(35)
                        };

                        ui.painter().rect_filled(rect, Rounding::same(4.0), bg_color);

                        if is_selected {
                            ui.painter().rect_stroke(
                                rect,
                                Rounding::same(4.0),
                                Stroke::new(1.0, Color32::from_rgb(100, 150, 220)),
                            );
                        }

                        ui.horizontal(|ui| {
                            ui.add_space(8.0);

                            // Favorite star
                            let star = if is_favorite { "★" } else { "☆" };
                            let star_color = if is_favorite {
                                Color32::from_rgb(255, 200, 50)
                            } else {
                                Color32::from_gray(100)
                            };
                            if ui.add(egui::Label::new(RichText::new(star).color(star_color)).sense(egui::Sense::click())).clicked() {
                                action = Some(SynthPresetAction::ToggleFavorite(preset.name.clone()));
                            }

                            ui.add_space(8.0);

                            // Preset info
                            ui.vertical(|ui| {
                                ui.add_space(4.0);
                                ui.label(RichText::new(&preset.name).strong());
                                ui.label(RichText::new(&preset.description).small().weak());
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if is_selected {
                                    ui.label(RichText::new("✓").color(Color32::from_rgb(100, 200, 100)));
                                }
                            });
                        });

                        // Click to select
                        ui.interact(rect, ui.id().with(&preset.name), egui::Sense::click())
                    });

                    if response.inner.clicked() {
                        self.state.set_preset(preset.name.clone());
                        action = Some(SynthPresetAction::SelectPreset(preset.preset_type.clone()));
                    }

                    ui.add_space(2.0);
                }
            });

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_state_new() {
        let state = SynthPresetState::new();
        assert_eq!(state.selected_category, SynthCategory::Piano);
        assert_eq!(state.current_preset(), Some(&"Grand Piano".to_string()));
    }

    #[test]
    fn test_set_preset() {
        let mut state = SynthPresetState::new();
        state.selected_category = SynthCategory::Bass;
        state.set_preset("Moog Bass".to_string());
        assert_eq!(state.current_preset(), Some(&"Moog Bass".to_string()));
    }

    #[test]
    fn test_favorites() {
        let mut state = SynthPresetState::new();
        assert!(!state.is_favorite("Grand Piano"));

        state.toggle_favorite("Grand Piano");
        assert!(state.is_favorite("Grand Piano"));

        state.toggle_favorite("Grand Piano");
        assert!(!state.is_favorite("Grand Piano"));
    }

    #[test]
    fn test_category_presets() {
        let bass_presets = SynthCategory::Bass.presets();
        assert!(bass_presets.len() >= 7);
        assert!(bass_presets.iter().any(|p| p.name == "Synth Bass"));

        let guitar_presets = SynthCategory::Guitar.presets();
        assert!(guitar_presets.iter().any(|p| p.name == "Tech Death"));
    }
}
