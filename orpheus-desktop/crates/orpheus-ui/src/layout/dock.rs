//! Dock layout for panels

use egui_dock::{DockState, TabViewer, NodeIndex, Style as DockStyle};
use egui::Color32;
use orpheus_core::ProductionMode;
use crate::theme::Theme;
use crate::views::{
    ArrangeView, ArrangeViewState, ArrangeViewAction, TrackDisplay,
    ComposeView, ComposeViewState,
    MixView, MixViewState,
    PracticeView, PracticeViewState,
    RecordView, RecordViewState,
    MasterView, MasterViewState,
    ReleaseView, ReleaseViewState,
    DistributeView, DistributeViewState,
    PianoRollView, PianoRollState,
    TabEditorView, TabEditorState, TabEditorAction,
};

/// Tab types for dock
#[derive(Debug, Clone, PartialEq)]
pub enum DockTab {
    /// Main view for current mode
    ModeView,
    /// Track list panel
    Tracks,
    /// Inspector/properties panel
    Inspector,
    /// Browser panel
    Browser,
    /// AI chat panel
    AiChat,
    /// Effects rack panel
    Effects,
    /// Mixer panel
    Mixer,
    /// Piano roll MIDI editor
    PianoRoll,
    /// Tab editor (tablature)
    TabEditor,
}

impl DockTab {
    pub fn title(&self) -> &'static str {
        match self {
            Self::ModeView => "Main View",
            Self::Tracks => "Tracks",
            Self::Inspector => "Inspector",
            Self::Browser => "Browser",
            Self::AiChat => "AI Assistant",
            Self::Effects => "Effects",
            Self::Mixer => "Mixer",
            Self::PianoRoll => "Piano Roll",
            Self::TabEditor => "Tab Editor",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::ModeView => "",
            Self::Tracks => "",
            Self::Inspector => "",
            Self::Browser => "",
            Self::AiChat => "",
            Self::Effects => "",
            Self::Mixer => "",
            Self::PianoRoll => "",
            Self::TabEditor => "",
        }
    }
}

/// All view states for the application
pub struct ViewStates {
    pub arrange: ArrangeViewState,
    pub compose: ComposeViewState,
    pub mix: MixViewState,
    pub practice: PracticeViewState,
    pub record: RecordViewState,
    pub master: MasterViewState,
    pub release: ReleaseViewState,
    pub distribute: DistributeViewState,
    pub piano_roll: PianoRollState,
    pub tab_editor: TabEditorState,
}

impl Default for ViewStates {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewStates {
    pub fn new() -> Self {
        // Start with clean states - tab editor has the source of truth for tracks
        // Arrange and Mix views sync from tab editor when displayed
        Self {
            arrange: ArrangeViewState::new(),
            compose: ComposeViewState::new(),
            mix: MixViewState::new(),
            practice: PracticeViewState::new(),
            record: RecordViewState::new(),
            master: MasterViewState::new(),
            release: ReleaseViewState::new(),
            distribute: DistributeViewState::new(),
            piano_roll: PianoRollState::new(),
            tab_editor: TabEditorState::with_guitar_track(),
        }
    }
}

/// Dock layout manager
pub struct DockLayout {
    pub state: DockState<DockTab>,
}

impl Default for DockLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl DockLayout {
    pub fn new() -> Self {
        // Create default layout with Tab Editor as the main view for new projects
        let mut state = DockState::new(vec![DockTab::ModeView]);

        // Get the main surface
        let surface = state.main_surface_mut();

        // Split left for tracks panel
        let [_main, _left] = surface.split_left(
            NodeIndex::root(),
            0.2,
            vec![DockTab::Tracks],
        );

        // Split right for inspector
        let [_main, _right] = surface.split_right(
            NodeIndex::root(),
            0.75,
            vec![DockTab::Inspector],
        );

        Self { state }
    }

    /// Get dock style
    pub fn style() -> DockStyle {
        let mut style = DockStyle::from_egui(&egui::Style::default());
        style.tab_bar.fill_tab_bar = true;
        style.tab_bar.height = 28.0;
        style
    }
}

/// Tab viewer for dock
pub struct OrpheusTabViewer<'a> {
    pub mode: ProductionMode,
    pub view_states: &'a mut ViewStates,
    pub theme: &'a Theme,
}

impl<'a> TabViewer for OrpheusTabViewer<'a> {
    type Tab = DockTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        let icon = tab.icon();
        if icon.is_empty() {
            tab.title().into()
        } else {
            format!("{} {}", icon, tab.title()).into()
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            DockTab::ModeView => {
                self.show_mode_view(ui);
            }
            DockTab::Tracks => {
                self.show_tracks_panel(ui);
            }
            DockTab::Inspector => {
                self.show_inspector_panel(ui);
            }
            DockTab::Browser => {
                self.show_browser_panel(ui);
            }
            DockTab::AiChat => {
                self.show_ai_chat_panel(ui);
            }
            DockTab::Effects => {
                self.show_effects_panel(ui);
            }
            DockTab::Mixer => {
                self.show_mixer_panel(ui);
            }
            DockTab::PianoRoll => {
                self.show_piano_roll(ui);
            }
            DockTab::TabEditor => {
                self.show_tab_editor(ui);
            }
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        true
    }
}

impl<'a> OrpheusTabViewer<'a> {
    fn show_mode_view(&mut self, ui: &mut egui::Ui) {
        match self.mode {
            ProductionMode::Arrange => {
                // Sync arrange tracks with tab editor tracks
                let track_names: Vec<String> = self.view_states.tab_editor
                    .document
                    .tracks
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                self.view_states.arrange.sync_from_tracks(&track_names);

                // Timeline/arrangement view for organizing clips and tracks
                // Action returned for future audio engine integration
                let _action = ArrangeView::show(ui, &mut self.view_states.arrange, self.theme);
            }
            ProductionMode::Compose => {
                // Show the tab editor (blank document editor) as the default compose view
                // ComposeView is for viewing imported Guitar Pro files
                self.show_tab_editor(ui);
            }
            ProductionMode::Mix => {
                // Sync mixer channels with tab editor tracks
                let track_names: Vec<String> = self.view_states.tab_editor
                    .document
                    .tracks
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                self.view_states.mix.sync_from_tracks(&track_names);

                let mut view = MixView::new(&mut self.view_states.mix, self.theme);
                view.show(ui);
            }
            ProductionMode::Practice => {
                let mut view = PracticeView::new(&mut self.view_states.practice, self.theme);
                view.show(ui);
            }
            ProductionMode::Record => {
                // Recording view with audio input, levels, and waveform display
                let mut view = RecordView::new(&mut self.view_states.record, self.theme);
                view.show(ui);
            }
            ProductionMode::Master => {
                let mut view = MasterView::new(&mut self.view_states.master, self.theme);
                view.show(ui);
            }
            ProductionMode::Release => {
                let mut view = ReleaseView::new(&mut self.view_states.release, self.theme);
                view.show(ui);
            }
            ProductionMode::Distribute => {
                let mut view = DistributeView::new(&mut self.view_states.distribute, self.theme);
                view.show(ui);
            }
        }
    }

    fn show_tracks_panel(&mut self, ui: &mut egui::Ui) {
        use orpheus_core::tab::Instrument;

        ui.heading("Tracks");
        ui.separator();

        let tracks = &self.view_states.tab_editor.document.tracks;
        let selected = self.view_states.tab_editor.cursor.track;

        if tracks.is_empty() {
            ui.label(
                egui::RichText::new("No tracks yet")
                    .italics()
                    .color(self.theme.text_secondary())
            );
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, track) in tracks.iter().enumerate() {
                    let is_selected = idx == selected;
                    let bg = if is_selected {
                        self.theme.palette.accent.linear_multiply(0.3)
                    } else {
                        egui::Color32::TRANSPARENT
                    };

                    // Get string count from instrument
                    let (icon, string_info) = match &track.instrument {
                        Instrument::StringedInstrument(config) => {
                            let icon = if config.string_count == 4 { "🎸" } else { "🎸" };
                            (icon, format!("{}str", config.string_count))
                        }
                        Instrument::Drums(_) => ("🥁", "Drums".to_string()),
                        Instrument::Keys(_) => ("🎹", "Keys".to_string()),
                    };

                    let response = egui::Frame::none()
                        .fill(bg)
                        .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(icon);
                                ui.label(&track.name);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        egui::RichText::new(string_info)
                                            .small()
                                            .color(self.theme.text_secondary())
                                    );
                                });
                            });
                        });

                    // Make track item clickable with hover feedback
                    let click_response = response.response.interact(egui::Sense::click());
                    if click_response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if click_response.clicked() {
                        self.view_states.tab_editor.cursor.track = idx;
                    }
                }
            });
        }

        ui.separator();
        if ui.button("+ Add Guitar Track").on_hover_text("Add a new 6-string guitar track").clicked() {
            let name = format!("Guitar {}", tracks.len() + 1);
            let track = orpheus_core::tab::TabTrack::guitar(&name);
            self.view_states.tab_editor.document.add_track(track);
        }
    }

    fn show_inspector_panel(&mut self, ui: &mut egui::Ui) {
        use orpheus_core::tab::Instrument;

        ui.heading("Inspector");
        ui.separator();

        let state = &self.view_states.tab_editor;
        let tracks = &state.document.tracks;

        if tracks.is_empty() {
            ui.label("No tracks to inspect");
            return;
        }

        // Show current track info
        if let Some(track) = tracks.get(state.cursor.track) {
            ui.group(|ui| {
                ui.label(egui::RichText::new(&track.name).strong());

                // Get string info from instrument
                if let Instrument::StringedInstrument(config) = &track.instrument {
                    ui.horizontal(|ui| {
                        ui.label("Strings:");
                        ui.label(format!("{}", config.string_count));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Tuning:");
                        let tuning_str: String = config.tuning.iter()
                            .map(|&midi| {
                                let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
                                note_names[(midi % 12) as usize].to_string()
                            })
                            .collect::<Vec<_>>()
                            .join(" ");
                        ui.label(tuning_str);
                    });
                }
            });

            ui.add_space(8.0);

            // Cursor position
            ui.group(|ui| {
                ui.label(egui::RichText::new("Cursor").strong());
                ui.horizontal(|ui| {
                    ui.label("Measure:");
                    ui.label(format!("{}", state.cursor.measure + 1));
                });
                ui.horizontal(|ui| {
                    ui.label("Beat:");
                    ui.label(format!("{}", state.cursor.beat + 1));
                });
                ui.horizontal(|ui| {
                    ui.label("String:");
                    ui.label(format!("{}", state.cursor.string + 1));
                });
            });

            ui.add_space(8.0);

            // Editor mode
            ui.group(|ui| {
                ui.label(egui::RichText::new("Mode").strong());
                let mode_str = match state.mode {
                    crate::views::tab_editor::EditorMode::Normal => "Normal",
                    crate::views::tab_editor::EditorMode::Insert => "Insert",
                    crate::views::tab_editor::EditorMode::Visual => "Visual",
                    crate::views::tab_editor::EditorMode::Command => "Command",
                };
                ui.label(mode_str);
            });
        }
    }

    fn show_browser_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Browser");
        ui.separator();

        ui.label("Quick access to samples and presets");
        ui.add_space(8.0);

        ui.collapsing("Recent Files", |ui| {
            ui.label(
                egui::RichText::new("No recent files")
                    .italics()
                    .color(self.theme.text_secondary())
            );
        });

        ui.collapsing("Guitar Pro Files", |ui| {
            ui.label("Drag & drop .gp files here");
        });

        ui.collapsing("Audio Samples", |ui| {
            ui.label(
                egui::RichText::new("Coming soon...")
                    .italics()
                    .color(self.theme.text_secondary())
            );
        });
    }

    fn show_ai_chat_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("AI Assistant");
        ui.separator();

        ui.label("AI-powered composition assistance");
        ui.add_space(8.0);

        ui.label(
            egui::RichText::new("Features coming soon:")
                .color(self.theme.text_secondary())
        );
        ui.horizontal(|ui| {
            ui.label("•");
            ui.label("Chord suggestions");
        });
        ui.horizontal(|ui| {
            ui.label("•");
            ui.label("Scale recommendations");
        });
        ui.horizontal(|ui| {
            ui.label("•");
            ui.label("Arrangement ideas");
        });
        ui.horizontal(|ui| {
            ui.label("•");
            ui.label("Technique guidance");
        });
    }

    fn show_effects_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Effects");
        ui.separator();

        ui.label("Track effects chain");
        ui.add_space(8.0);

        ui.label(
            egui::RichText::new("Available effects:")
                .color(self.theme.text_secondary())
        );
        ui.horizontal(|ui| {
            ui.label("•");
            ui.label("Amp simulation");
        });
        ui.horizontal(|ui| {
            ui.label("•");
            ui.label("Cabinet modeling");
        });
        ui.horizontal(|ui| {
            ui.label("•");
            ui.label("Reverb & delay");
        });
        ui.horizontal(|ui| {
            ui.label("•");
            ui.label("Compression & EQ");
        });

        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Full effect rack coming soon...")
                .italics()
                .color(self.theme.text_secondary())
        );
    }

    fn show_mixer_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Quick Mix");
        ui.separator();

        ui.label("For full mixing capabilities, use Mix mode");
        ui.add_space(8.0);

        // Show basic track levels from tab editor
        let tracks = &self.view_states.tab_editor.document.tracks;
        if tracks.is_empty() {
            ui.label(
                egui::RichText::new("No tracks")
                    .italics()
                    .color(self.theme.text_secondary())
            );
        } else {
            for track in tracks.iter() {
                ui.horizontal(|ui| {
                    ui.label(&track.name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("0 dB");
                    });
                });
            }
        }

        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Switch to Mix mode for channel strips")
                .italics()
                .small()
                .color(self.theme.palette.accent)
        );
    }

    fn show_piano_roll(&mut self, ui: &mut egui::Ui) {
        let mut view = PianoRollView::new(&mut self.view_states.piano_roll, self.theme);
        let _action = view.show(ui);
        // Actions are handled by the main app
    }

    fn show_tab_editor(&mut self, ui: &mut egui::Ui) {
        let mut view = TabEditorView::new(&mut self.view_states.tab_editor, self.theme);
        let action = view.show(ui);
        // Queue actions for app.rs to handle
        match action {
            TabEditorAction::PreviewNote { string, fret, velocity } => {
                self.view_states.tab_editor.queue_preview(string, fret, velocity);
            }
            TabEditorAction::ExportPdf => {
                self.view_states.tab_editor.queue_export_pdf();
            }
            TabEditorAction::ExportAudio => {
                self.view_states.tab_editor.queue_export_audio();
            }
            _ => {}
        }
    }
}
