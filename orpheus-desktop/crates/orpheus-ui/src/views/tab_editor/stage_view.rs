//! Stage View - Conductor's Podium for instrument arrangement
//!
//! A revolutionary approach to multi-track instrument setup:
//! - View your ensemble from the conductor's perspective
//! - Drag instruments from a palette onto stage positions
//! - Pan is automatically set based on left/right position
//! - Click any instrument "chair" for detailed configuration
//!
//! This replaces the tedious one-at-a-time instrument addition workflow
//! found in competitors like Guitar Pro.

use egui::{
    Color32, Pos2, Rect, Response, RichText, Sense, Stroke, Ui, Vec2,
    CursorIcon, Id,
};
use orpheus_core::tab::{
    Instrument, StringedConfig, StringedType, TabTrack, DrumKit,
};
use uuid::Uuid;

/// Position on the stage (from conductor's view)
#[derive(Debug, Clone, Copy)]
pub struct StagePosition {
    /// X position (-1.0 = far left, 1.0 = far right)
    pub x: f32,
    /// Y position (0.0 = front of stage, 1.0 = back)
    pub depth: f32,
}

impl StagePosition {
    pub fn new(x: f32, depth: f32) -> Self {
        Self {
            x: x.clamp(-1.0, 1.0),
            depth: depth.clamp(0.0, 1.0),
        }
    }

    /// Convert to pan value (-1.0 left, 1.0 right)
    pub fn to_pan(&self) -> f32 {
        self.x
    }

    /// Position for typical orchestra seating
    pub fn orchestra_position(section: OrchestraSection) -> Self {
        match section {
            OrchestraSection::LeadGuitar => Self::new(-0.6, 0.3),
            OrchestraSection::RhythmGuitar => Self::new(0.6, 0.3),
            OrchestraSection::Bass => Self::new(-0.3, 0.5),
            OrchestraSection::Drums => Self::new(0.0, 0.8),
            OrchestraSection::Keys => Self::new(0.4, 0.5),
            OrchestraSection::Vocals => Self::new(0.0, 0.1),
        }
    }

    /// Position for traditional rock band
    pub fn band_position(role: BandRole) -> Self {
        match role {
            BandRole::LeadGuitar => Self::new(-0.5, 0.3),
            BandRole::RhythmGuitar => Self::new(0.5, 0.3),
            BandRole::Bass => Self::new(-0.7, 0.4),
            BandRole::Drums => Self::new(0.0, 0.7),
            BandRole::Keys => Self::new(0.7, 0.4),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OrchestraSection {
    LeadGuitar,
    RhythmGuitar,
    Bass,
    Drums,
    Keys,
    Vocals,
}

#[derive(Debug, Clone, Copy)]
pub enum BandRole {
    LeadGuitar,
    RhythmGuitar,
    Bass,
    Drums,
    Keys,
}

/// An instrument template that can be dragged onto the stage
#[derive(Debug, Clone)]
pub struct InstrumentTemplate {
    pub name: &'static str,
    pub icon: &'static str,
    pub category: InstrumentCategory,
    pub create: fn() -> Instrument,
    pub description: &'static str,
    pub default_color: (u8, u8, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentCategory {
    Guitar,
    Bass,
    Drums,
    Keys,
}

impl InstrumentCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Guitar => "Guitars",
            Self::Bass => "Bass",
            Self::Drums => "Drums & Percussion",
            Self::Keys => "Keys & Synths",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Guitar => "🎸",
            Self::Bass => "🎸",
            Self::Drums => "🥁",
            Self::Keys => "🎹",
        }
    }
}

/// All available instrument templates
pub fn instrument_templates() -> Vec<InstrumentTemplate> {
    vec![
        // Guitars
        InstrumentTemplate {
            name: "6-String Guitar",
            icon: "🎸",
            category: InstrumentCategory::Guitar,
            create: || Instrument::guitar_standard(),
            description: "Standard 6-string in E tuning",
            default_color: (52, 152, 219),
        },
        InstrumentTemplate {
            name: "6-String Drop D",
            icon: "🎸",
            category: InstrumentCategory::Guitar,
            create: || Instrument::StringedInstrument(StringedConfig::guitar_6_drop_d()),
            description: "Drop D tuning for heavier riffs",
            default_color: (155, 89, 182),
        },
        InstrumentTemplate {
            name: "7-String Guitar",
            icon: "🎸",
            category: InstrumentCategory::Guitar,
            create: || Instrument::guitar_7_string(),
            description: "Extended range in B standard",
            default_color: (231, 76, 60),
        },
        InstrumentTemplate {
            name: "8-String Guitar",
            icon: "🎸",
            category: InstrumentCategory::Guitar,
            create: || Instrument::guitar_8_string(),
            description: "Djent machine in F# standard",
            default_color: (44, 62, 80),
        },
        InstrumentTemplate {
            name: "9-String Guitar",
            icon: "🎸",
            category: InstrumentCategory::Guitar,
            create: || Instrument::guitar_9_string(),
            description: "Maximum brutality in C# standard",
            default_color: (22, 31, 40),
        },
        InstrumentTemplate {
            name: "Acoustic Guitar",
            icon: "🪕",
            category: InstrumentCategory::Guitar,
            create: || Instrument::StringedInstrument(StringedConfig {
                instrument_type: StringedType::AcousticGuitar,
                ..StringedConfig::guitar_6_standard()
            }),
            description: "Steel string acoustic",
            default_color: (211, 84, 0),
        },

        // Bass
        InstrumentTemplate {
            name: "4-String Bass",
            icon: "🎸",
            category: InstrumentCategory::Bass,
            create: || Instrument::bass_standard(),
            description: "Standard bass in E tuning",
            default_color: (39, 174, 96),
        },
        InstrumentTemplate {
            name: "5-String Bass",
            icon: "🎸",
            category: InstrumentCategory::Bass,
            create: || Instrument::bass_5_string(),
            description: "Low B for extra depth",
            default_color: (22, 160, 133),
        },
        InstrumentTemplate {
            name: "6-String Bass",
            icon: "🎸",
            category: InstrumentCategory::Bass,
            create: || Instrument::bass_6_string(),
            description: "Extended range bass",
            default_color: (41, 128, 185),
        },

        // Drums
        InstrumentTemplate {
            name: "Standard Drums",
            icon: "🥁",
            category: InstrumentCategory::Drums,
            create: || Instrument::Drums(DrumKit::standard()),
            description: "Rock/pop drum kit",
            default_color: (241, 196, 15),
        },
        InstrumentTemplate {
            name: "Metal Drums",
            icon: "🥁",
            category: InstrumentCategory::Drums,
            create: || Instrument::Drums(DrumKit::metal()),
            description: "Double kick, extra toms",
            default_color: (192, 57, 43),
        },

        // Keys
        InstrumentTemplate {
            name: "Piano",
            icon: "🎹",
            category: InstrumentCategory::Keys,
            create: || Instrument::Keys(orpheus_core::tab::KeysConfig::default()),
            description: "88-key grand piano",
            default_color: (149, 165, 166),
        },
    ]
}

/// State for the stage view
#[derive(Debug, Clone)]
pub struct StageViewState {
    /// Currently dragging an instrument
    pub dragging: Option<DragState>,
    /// Show the instrument palette
    pub show_palette: bool,
    /// Selected category in palette
    pub selected_category: InstrumentCategory,
    /// Hovered stage position (for preview)
    pub hover_position: Option<StagePosition>,
    /// Selected placed instrument for configuration
    pub selected_track: Option<Uuid>,
    /// Show configuration panel
    pub show_config: bool,
    /// Stage layout preset
    pub layout: StageLayout,
}

impl Default for StageViewState {
    fn default() -> Self {
        Self {
            dragging: None,
            show_palette: true,
            selected_category: InstrumentCategory::Guitar,
            hover_position: None,
            selected_track: None,
            show_config: false,
            layout: StageLayout::RockBand,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DragState {
    pub template_idx: usize,
    pub offset: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageLayout {
    RockBand,
    Orchestra,
    Custom,
}

impl StageLayout {
    pub fn name(&self) -> &'static str {
        match self {
            Self::RockBand => "Rock Band",
            Self::Orchestra => "Orchestra",
            Self::Custom => "Custom",
        }
    }
}

/// Actions from the stage view
#[derive(Debug, Clone)]
pub enum StageViewAction {
    /// Add a new track at the given stage position
    AddTrack {
        template_idx: usize,
        position: StagePosition,
    },
    /// Move an existing track to a new position
    MoveTrack {
        track_id: Uuid,
        position: StagePosition,
    },
    /// Remove a track
    RemoveTrack(Uuid),
    /// Select track for configuration
    SelectTrack(Uuid),
    /// Update track configuration
    UpdateTrack {
        track_id: Uuid,
        name: Option<String>,
        pan: Option<f32>,
        volume: Option<f32>,
        color: Option<(u8, u8, u8)>,
    },
    /// Close the stage view
    Close,
}

/// The stage view widget
pub struct StageView<'a> {
    pub state: &'a mut StageViewState,
    pub tracks: &'a mut Vec<TabTrack>,
    pub theme: &'a crate::theme::OrpheusTheme,
}

impl<'a> StageView<'a> {
    pub fn new(
        state: &'a mut StageViewState,
        tracks: &'a mut Vec<TabTrack>,
        theme: &'a crate::theme::OrpheusTheme,
    ) -> Self {
        Self { state, tracks, theme }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Option<StageViewAction> {
        let mut action = None;

        // Header
        ui.horizontal(|ui| {
            ui.heading("🎭 Stage View");
            ui.separator();
            ui.label("Drag instruments onto the stage to position them");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕ Close").on_hover_text("Return to tab editor").clicked() {
                    action = Some(StageViewAction::Close);
                }

                ui.separator();

                // Layout preset selector
                egui::ComboBox::from_id_salt("stage_layout")
                    .selected_text(self.state.layout.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.state.layout, StageLayout::RockBand, "Rock Band")
                            .on_hover_text("Traditional rock/metal band positioning");
                        ui.selectable_value(&mut self.state.layout, StageLayout::Orchestra, "Orchestra")
                            .on_hover_text("Orchestra pit positioning");
                        ui.selectable_value(&mut self.state.layout, StageLayout::Custom, "Custom")
                            .on_hover_text("Free-form positioning");
                    }).response.on_hover_text("Stage layout preset");
            });
        });

        ui.separator();

        // Main content area
        ui.horizontal(|ui| {
            // Left: Instrument palette
            ui.vertical(|ui| {
                ui.set_min_width(180.0);
                ui.set_max_width(200.0);

                ui.heading("Instruments");
                ui.separator();

                // Category tabs
                ui.horizontal(|ui| {
                    for cat in [
                        InstrumentCategory::Guitar,
                        InstrumentCategory::Bass,
                        InstrumentCategory::Drums,
                        InstrumentCategory::Keys,
                    ] {
                        let selected = self.state.selected_category == cat;
                        if ui.selectable_label(selected, cat.icon())
                            .on_hover_text(cat.name())
                            .clicked()
                        {
                            self.state.selected_category = cat;
                        }
                    }
                });

                ui.separator();

                // Instrument list for selected category
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        let templates = instrument_templates();
                        for (idx, template) in templates.iter().enumerate() {
                            if template.category != self.state.selected_category {
                                continue;
                            }

                            let response = self.show_palette_item(ui, idx, template);

                            if response.drag_started() {
                                self.state.dragging = Some(DragState {
                                    template_idx: idx,
                                    offset: Vec2::ZERO,
                                });
                            }
                        }
                    });
            });

            ui.separator();

            // Center: Stage canvas
            ui.vertical(|ui| {
                ui.set_min_width(500.0);

                if let Some(stage_action) = self.show_stage_canvas(ui) {
                    action = Some(stage_action);
                }
            });

            ui.separator();

            // Right: Track configuration (if track selected)
            if let Some(track_id) = self.state.selected_track {
                ui.vertical(|ui| {
                    ui.set_min_width(200.0);
                    ui.set_max_width(250.0);

                    if let Some(config_action) = self.show_track_config(ui, track_id) {
                        action = Some(config_action);
                    }
                });
            }
        });

        action
    }

    fn show_palette_item(&self, ui: &mut Ui, idx: usize, template: &InstrumentTemplate) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(170.0, 50.0), Sense::drag());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let is_hovered = response.hovered();

            // Background
            let bg_color = if is_hovered {
                Color32::from_gray(50)
            } else {
                Color32::from_gray(35)
            };
            painter.rect_filled(rect, 4.0, bg_color);

            // Icon
            let icon_pos = rect.left_center() + Vec2::new(20.0, 0.0);
            painter.text(
                icon_pos,
                egui::Align2::CENTER_CENTER,
                template.icon,
                egui::FontId::proportional(20.0),
                Color32::WHITE,
            );

            // Name and description
            let text_left = rect.left() + 45.0;
            painter.text(
                Pos2::new(text_left, rect.top() + 15.0),
                egui::Align2::LEFT_CENTER,
                template.name,
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );
            painter.text(
                Pos2::new(text_left, rect.top() + 32.0),
                egui::Align2::LEFT_CENTER,
                template.description,
                egui::FontId::proportional(9.0),
                Color32::GRAY,
            );

            // Color indicator
            let color = Color32::from_rgb(
                template.default_color.0,
                template.default_color.1,
                template.default_color.2,
            );
            painter.rect_filled(
                Rect::from_min_size(Pos2::new(rect.right() - 12.0, rect.top() + 4.0), Vec2::new(8.0, 42.0)),
                2.0,
                color,
            );

            // Drag hint
            if is_hovered {
                ui.ctx().set_cursor_icon(CursorIcon::Grab);
            }
        }

        response.on_hover_text("Drag onto stage to add")
    }

    fn show_stage_canvas(&mut self, ui: &mut Ui) -> Option<StageViewAction> {
        let mut action = None;

        // Stage area
        let available = ui.available_size();
        let stage_size = Vec2::new(available.x.min(600.0), 400.0);
        let (rect, response) = ui.allocate_exact_size(stage_size, Sense::click_and_drag());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Stage background (gradient from front to back)
            painter.rect_filled(rect, 8.0, Color32::from_gray(25));

            // Stage floor lines (perspective)
            let line_color = Color32::from_gray(40);
            for i in 0..10 {
                let t = i as f32 / 10.0;
                let y = rect.top() + rect.height() * t;
                painter.line_segment(
                    [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                    Stroke::new(1.0, line_color),
                );
            }

            // Center line
            painter.line_segment(
                [
                    Pos2::new(rect.center().x, rect.top()),
                    Pos2::new(rect.center().x, rect.bottom()),
                ],
                Stroke::new(1.0, Color32::from_gray(50)),
            );

            // Labels
            painter.text(
                Pos2::new(rect.left() + 30.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                "← LEFT",
                egui::FontId::proportional(10.0),
                Color32::from_gray(80),
            );
            painter.text(
                Pos2::new(rect.right() - 30.0, rect.center().y),
                egui::Align2::RIGHT_CENTER,
                "RIGHT →",
                egui::FontId::proportional(10.0),
                Color32::from_gray(80),
            );
            painter.text(
                Pos2::new(rect.center().x, rect.top() + 15.0),
                egui::Align2::CENTER_CENTER,
                "🎤 FRONT (Audience)",
                egui::FontId::proportional(10.0),
                Color32::from_gray(80),
            );
            painter.text(
                Pos2::new(rect.center().x, rect.bottom() - 15.0),
                egui::Align2::CENTER_CENTER,
                "BACK",
                egui::FontId::proportional(10.0),
                Color32::from_gray(80),
            );

            // Draw placed instruments
            for track in self.tracks.iter() {
                let pos = self.stage_position_to_screen(rect, track.pan, 0.5);
                let is_selected = self.state.selected_track == Some(track.id);

                self.draw_placed_instrument(painter, pos, track, is_selected);
            }

            // Handle drop
            if let Some(drag_state) = &self.state.dragging {
                if response.drag_stopped() {
                    if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                        if rect.contains(pointer_pos) {
                            let stage_pos = self.screen_to_stage_position(rect, pointer_pos);
                            action = Some(StageViewAction::AddTrack {
                                template_idx: drag_state.template_idx,
                                position: stage_pos,
                            });
                        }
                    }
                    self.state.dragging = None;
                } else if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                    // Draw drag preview
                    let templates = instrument_templates();
                    if let Some(template) = templates.get(drag_state.template_idx) {
                        painter.text(
                            pointer_pos,
                            egui::Align2::CENTER_CENTER,
                            template.icon,
                            egui::FontId::proportional(32.0),
                            Color32::from_white_alpha(180),
                        );

                        // Show pan value preview
                        if rect.contains(pointer_pos) {
                            let stage_pos = self.screen_to_stage_position(rect, pointer_pos);
                            let pan_text = format!("Pan: {:.0}%", stage_pos.x * 100.0);
                            painter.text(
                                pointer_pos + Vec2::new(0.0, 25.0),
                                egui::Align2::CENTER_CENTER,
                                &pan_text,
                                egui::FontId::proportional(11.0),
                                Color32::WHITE,
                            );
                        }
                    }
                }
            }

            // Handle click on tracks
            if response.clicked() {
                if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                    let mut clicked_track = None;
                    for track in self.tracks.iter() {
                        let pos = self.stage_position_to_screen(rect, track.pan, 0.5);
                        let track_rect = Rect::from_center_size(pos, Vec2::splat(50.0));
                        if track_rect.contains(pointer_pos) {
                            clicked_track = Some(track.id);
                            break;
                        }
                    }
                    if let Some(id) = clicked_track {
                        self.state.selected_track = Some(id);
                        action = Some(StageViewAction::SelectTrack(id));
                    } else {
                        self.state.selected_track = None;
                    }
                }
            }
        }

        // Quick add buttons
        ui.horizontal(|ui| {
            ui.label("Quick Add:");
            if ui.button("🎸 Guitar").on_hover_text("Add 6-string guitar").clicked() {
                action = Some(StageViewAction::AddTrack {
                    template_idx: 0,
                    position: self.next_available_position(),
                });
            }
            if ui.button("🎸 Bass").on_hover_text("Add 4-string bass").clicked() {
                action = Some(StageViewAction::AddTrack {
                    template_idx: 6,
                    position: StagePosition::band_position(BandRole::Bass),
                });
            }
            if ui.button("🥁 Drums").on_hover_text("Add drum kit").clicked() {
                action = Some(StageViewAction::AddTrack {
                    template_idx: 9,
                    position: StagePosition::band_position(BandRole::Drums),
                });
            }
            if ui.button("🎹 Keys").on_hover_text("Add piano/keys").clicked() {
                action = Some(StageViewAction::AddTrack {
                    template_idx: 11,
                    position: StagePosition::band_position(BandRole::Keys),
                });
            }
        });

        action
    }

    fn draw_placed_instrument(&self, painter: &egui::Painter, pos: Pos2, track: &TabTrack, selected: bool) {
        let icon = match &track.instrument {
            Instrument::StringedInstrument(config) => {
                if config.instrument_type == StringedType::Bass {
                    "🎸"
                } else if config.instrument_type == StringedType::AcousticGuitar {
                    "🪕"
                } else {
                    "🎸"
                }
            }
            Instrument::Drums(_) => "🥁",
            Instrument::Keys(_) => "🎹",
        };

        let color = Color32::from_rgb(track.color.0, track.color.1, track.color.2);

        // Circle background
        let bg_color = if selected {
            Color32::from_rgb(52, 73, 94)
        } else {
            Color32::from_gray(45)
        };
        painter.circle_filled(pos, 28.0, bg_color);
        painter.circle_stroke(pos, 28.0, Stroke::new(2.0, color));

        if selected {
            painter.circle_stroke(pos, 32.0, Stroke::new(2.0, Color32::WHITE));
        }

        // Icon
        painter.text(
            pos - Vec2::new(0.0, 2.0),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(24.0),
            Color32::WHITE,
        );

        // Name below
        painter.text(
            pos + Vec2::new(0.0, 38.0),
            egui::Align2::CENTER_CENTER,
            &track.name,
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );

        // Pan indicator
        let pan_text = format!("{:.0}%", track.pan * 100.0);
        painter.text(
            pos + Vec2::new(0.0, 50.0),
            egui::Align2::CENTER_CENTER,
            &pan_text,
            egui::FontId::proportional(8.0),
            Color32::GRAY,
        );
    }

    fn show_track_config(&mut self, ui: &mut Ui, track_id: Uuid) -> Option<StageViewAction> {
        let mut action = None;

        let track = self.tracks.iter_mut().find(|t| t.id == track_id)?;

        ui.heading("Track Config");
        ui.separator();

        // Name
        ui.horizontal(|ui| {
            ui.label("Name:");
            let mut name = track.name.clone();
            if ui.text_edit_singleline(&mut name).changed() {
                action = Some(StageViewAction::UpdateTrack {
                    track_id,
                    name: Some(name),
                    pan: None,
                    volume: None,
                    color: None,
                });
            }
        });

        ui.add_space(8.0);

        // Instrument info
        let (icon, info) = match &track.instrument {
            Instrument::StringedInstrument(config) => {
                (
                    if config.instrument_type == StringedType::Bass { "🎸" } else { "🎸" },
                    format!("{}-string, {} frets", config.string_count, config.fret_count),
                )
            }
            Instrument::Drums(_) => ("🥁", "Drum Kit".to_string()),
            Instrument::Keys(config) => ("🎹", format!("{} keys", config.key_count)),
        };

        ui.horizontal(|ui| {
            ui.label(icon);
            ui.label(&info);
        });

        ui.add_space(8.0);
        ui.separator();

        // Pan slider
        ui.label("Pan Position:");
        let mut pan = track.pan;
        let pan_slider = egui::Slider::new(&mut pan, -1.0..=1.0)
            .suffix("")
            .custom_formatter(|v, _| {
                if v.abs() < 0.05 {
                    "C".to_string()
                } else if v < 0.0 {
                    format!("{:.0}L", v.abs() * 100.0)
                } else {
                    format!("{:.0}R", v * 100.0)
                }
            });
        if ui.add(pan_slider).on_hover_text("Left/right position in stereo field").changed() {
            action = Some(StageViewAction::UpdateTrack {
                track_id,
                name: None,
                pan: Some(pan),
                volume: None,
                color: None,
            });
        }

        ui.add_space(8.0);

        // Volume slider
        ui.label("Volume:");
        let mut volume = track.volume;
        let vol_slider = egui::Slider::new(&mut volume, 0.0..=1.0)
            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0));
        if ui.add(vol_slider).on_hover_text("Track volume").changed() {
            action = Some(StageViewAction::UpdateTrack {
                track_id,
                name: None,
                pan: None,
                volume: Some(volume),
                color: None,
            });
        }

        ui.add_space(8.0);

        // Color picker
        ui.label("Track Color:");
        let mut color = [
            track.color.0 as f32 / 255.0,
            track.color.1 as f32 / 255.0,
            track.color.2 as f32 / 255.0,
        ];
        if ui.color_edit_button_rgb(&mut color).changed() {
            action = Some(StageViewAction::UpdateTrack {
                track_id,
                name: None,
                pan: None,
                volume: None,
                color: Some((
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                )),
            });
        }

        ui.add_space(16.0);
        ui.separator();

        // Remove button
        if ui.button("🗑 Remove Track")
            .on_hover_text("Remove this track from the project")
            .clicked()
        {
            action = Some(StageViewAction::RemoveTrack(track_id));
            self.state.selected_track = None;
        }

        action
    }

    fn stage_position_to_screen(&self, rect: Rect, pan: f32, depth: f32) -> Pos2 {
        let x = rect.center().x + (pan * rect.width() * 0.45);
        let y = rect.top() + (depth * rect.height() * 0.8) + rect.height() * 0.1;
        Pos2::new(x, y)
    }

    fn screen_to_stage_position(&self, rect: Rect, screen_pos: Pos2) -> StagePosition {
        let x = (screen_pos.x - rect.center().x) / (rect.width() * 0.45);
        let y = (screen_pos.y - rect.top() - rect.height() * 0.1) / (rect.height() * 0.8);
        StagePosition::new(x, y)
    }

    fn next_available_position(&self) -> StagePosition {
        // Find a position that doesn't overlap with existing tracks
        let positions = [
            StagePosition::new(-0.5, 0.3),
            StagePosition::new(0.5, 0.3),
            StagePosition::new(-0.3, 0.5),
            StagePosition::new(0.3, 0.5),
            StagePosition::new(0.0, 0.4),
            StagePosition::new(-0.7, 0.4),
            StagePosition::new(0.7, 0.4),
        ];

        for pos in positions {
            let occupied = self.tracks.iter().any(|t| (t.pan - pos.x).abs() < 0.15);
            if !occupied {
                return pos;
            }
        }

        // Default fallback
        StagePosition::new(0.0, 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_position() {
        let pos = StagePosition::new(-0.5, 0.3);
        assert_eq!(pos.to_pan(), -0.5);

        let clamped = StagePosition::new(-2.0, 1.5);
        assert_eq!(clamped.x, -1.0);
        assert_eq!(clamped.depth, 1.0);
    }

    #[test]
    fn test_instrument_templates() {
        let templates = instrument_templates();
        assert!(!templates.is_empty());

        // Check we have all categories
        let has_guitar = templates.iter().any(|t| t.category == InstrumentCategory::Guitar);
        let has_bass = templates.iter().any(|t| t.category == InstrumentCategory::Bass);
        let has_drums = templates.iter().any(|t| t.category == InstrumentCategory::Drums);
        let has_keys = templates.iter().any(|t| t.category == InstrumentCategory::Keys);

        assert!(has_guitar);
        assert!(has_bass);
        assert!(has_drums);
        assert!(has_keys);
    }
}
