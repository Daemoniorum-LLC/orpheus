//! Main application state and logic

use eframe::egui;
use egui_dock::DockArea;
use tracing::{debug, info};

use orpheus_core::{
    AddTrackCommand, AppState, BoxedCommand, CommandContext, CommandHistory, CommandRegistry,
    EventBus, MixerState, Project, SelectionState, Settings, TrackType, TransportState,
};
use orpheus_export::{ExportSettings, ExportFormat, export_project, ExportResult};
use orpheus_ui::{
    layout::{DockLayout, DockTab, OrpheusTabViewer, ViewStates},
    toolbar::{ModeTabs, TransportAction, TransportBar},
    Theme, OrpheusTheme,
    PluginBrowserPanel, PluginBrowserState, PluginBrowserAction,
    ScannedPlugin, LoadedPlugin, PanelPluginFormat,
    VirtualKeyboardPanel, VirtualKeyboardState, VirtualKeyboardAction,
    SynthPresetPanel, SynthPresetState, SynthPresetAction,
    BassPresetType as UiBassPresetType, GuitarPresetType as UiGuitarPresetType,
    DrumPresetType as UiDrumPresetType, PresetType,
    MidiInputPanel, MidiInputState, MidiInputAction, MidiDevice, midi_message_color,
    PendingTabAction, TabPlaybackEventType, TabEditorState,
    WelcomeDialog, WelcomeState, WelcomeAction,
    PreferencesDialog, PreferencesState, PreferencesAction, ThemeMode,
    PracticeAction,
    AboutDialog, AboutState,
};
use orpheus_synth::NoteEventType;

use crate::audio_engine::{AudioEngine, EngineState, AudioStatus, PluginFormat, RecordedClipInfo};
use crate::screenshot::ScreenshotManager;

/// Main Orpheus application
pub struct OrpheusApp {
    // Theme
    theme: Theme,

    // State
    app_state: AppState,
    transport: TransportState,
    mixer: MixerState,
    selection: SelectionState,

    // Project
    project: Project,

    // Commands
    command_registry: CommandRegistry,
    command_history: CommandHistory,

    // Events
    event_bus: EventBus,

    // Settings
    settings: Settings,

    // UI
    dock_layout: DockLayout,
    view_states: ViewStates,

    // Audio engine
    audio_engine: AudioEngine,

    // Plugin browser state
    plugin_browser: PluginBrowserState,
    show_plugin_browser: bool,

    // Virtual keyboard state
    virtual_keyboard: VirtualKeyboardState,
    show_virtual_keyboard: bool,

    // Synth preset browser state
    synth_presets: SynthPresetState,
    show_synth_presets: bool,

    // MIDI input state
    midi_input: MidiInputState,
    show_midi_input: bool,

    // Frame counter for animations
    frame_count: u64,

    // Metering
    level_left: f32,
    level_right: f32,
    cpu_load: f32,

    // Export state
    export_state: ExportState,

    // Screenshot manager
    screenshot: ScreenshotManager,

    // Confirmation dialog state
    confirm_dialog: ConfirmDialogState,

    // Notification system
    notifications: NotificationState,

    // Welcome dialog state
    welcome_state: WelcomeState,

    // Preferences dialog state
    preferences_state: PreferencesState,

    // About dialog state
    about_state: AboutState,
}

/// State for project export
#[derive(Default)]
struct ExportState {
    /// Whether export is in progress
    is_exporting: bool,
    /// Export progress (0.0 - 1.0)
    progress: f32,
    /// Last export result
    last_result: Option<ExportResult>,
    /// Last export error message
    last_error: Option<String>,
    /// Show export result dialog
    show_result: bool,
}

/// State for confirmation dialogs
#[derive(Default)]
struct ConfirmDialogState {
    /// Whether dialog is visible
    visible: bool,
    /// Dialog title
    title: String,
    /// Dialog message
    message: String,
    /// Action to perform if confirmed
    pending_action: Option<PendingAction>,
}

/// Pending action to execute after confirmation
#[derive(Clone)]
enum PendingAction {
    NewProject,
    OpenProject,
    CloseApp,
}

/// Notification message
struct Notification {
    message: String,
    level: NotificationLevel,
    created_at: std::time::Instant,
}

/// Notification severity level
#[derive(Clone, Copy, PartialEq)]
enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationLevel {
    fn color(&self) -> egui::Color32 {
        match self {
            NotificationLevel::Info => egui::Color32::from_rgb(100, 150, 200),
            NotificationLevel::Success => egui::Color32::from_rgb(52, 168, 83),
            NotificationLevel::Warning => egui::Color32::from_rgb(251, 188, 4),
            NotificationLevel::Error => egui::Color32::from_rgb(234, 67, 53),
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            NotificationLevel::Info => "ℹ",
            NotificationLevel::Success => "✓",
            NotificationLevel::Warning => "⚠",
            NotificationLevel::Error => "✕",
        }
    }
}

/// State for notification system
#[derive(Default)]
struct NotificationState {
    notifications: Vec<Notification>,
}

impl NotificationState {
    fn push(&mut self, message: impl Into<String>, level: NotificationLevel) {
        self.notifications.push(Notification {
            message: message.into(),
            level,
            created_at: std::time::Instant::now(),
        });
    }

    fn info(&mut self, message: impl Into<String>) {
        self.push(message, NotificationLevel::Info);
    }

    fn success(&mut self, message: impl Into<String>) {
        self.push(message, NotificationLevel::Success);
    }

    fn warning(&mut self, message: impl Into<String>) {
        self.push(message, NotificationLevel::Warning);
    }

    fn error(&mut self, message: impl Into<String>) {
        self.push(message, NotificationLevel::Error);
    }

    fn cleanup(&mut self) {
        // Remove notifications older than 5 seconds
        let cutoff = std::time::Duration::from_secs(5);
        self.notifications.retain(|n| n.created_at.elapsed() < cutoff);
    }
}

impl OrpheusApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        info!("Initializing Orpheus application");

        // Load settings
        let settings = Settings::load();

        // Create default project with some tracks
        let mut project = Project::new("Untitled Project");
        let mut mixer = MixerState::new();

        // Add default tracks
        for i in 1..=4 {
            let track = orpheus_core::Track::audio(format!("Track {}", i));
            let track_id = project.add_track(track);
            mixer.add_channel(format!("Track {}", i));
            debug!("Added track {} with id {}", i, track_id);
        }

        // Apply theme based on settings
        let theme = if settings.ui.high_contrast {
            OrpheusTheme::high_contrast()
        } else if settings.ui.theme == "light" {
            OrpheusTheme::light()
        } else {
            OrpheusTheme::dark()
        };
        theme.apply(&cc.egui_ctx);

        // Initialize welcome dialog - show if first run or if setting is enabled
        let mut welcome_state = WelcomeState::new();
        welcome_state.visible = settings.ui.show_welcome_on_startup;
        welcome_state.show_on_startup = settings.ui.show_welcome_on_startup;

        // Initialize preferences dialog with settings values
        let mut preferences_state = PreferencesState::new();
        preferences_state.sync_from_settings(&settings);

        // Initialize audio engine
        let transport = TransportState::new();
        let audio_engine = AudioEngine::new(transport.sample_rate);
        info!("Audio engine initialized at {} Hz", transport.sample_rate);

        Self {
            theme,
            app_state: AppState::new(),
            transport,
            mixer,
            selection: SelectionState::new(),
            project,
            command_registry: CommandRegistry::new(),
            command_history: CommandHistory::new(),
            event_bus: EventBus::new(),
            settings,
            dock_layout: DockLayout::new(),
            view_states: ViewStates::new(),
            audio_engine,
            plugin_browser: PluginBrowserState::new(),
            show_plugin_browser: true,
            virtual_keyboard: VirtualKeyboardState::new(),
            show_virtual_keyboard: true,
            synth_presets: SynthPresetState::new(),
            show_synth_presets: false, // Start hidden, show via menu
            midi_input: MidiInputState::new(),
            show_midi_input: false, // Start hidden, show via menu
            frame_count: 0,
            level_left: 0.0,
            level_right: 0.0,
            cpu_load: 0.0,
            export_state: ExportState::default(),
            screenshot: ScreenshotManager::new(),
            confirm_dialog: ConfirmDialogState::default(),
            notifications: NotificationState::default(),
            welcome_state,
            preferences_state,
            about_state: AboutState::new(),
        }
    }

    /// Handle keyboard shortcuts
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // Don't process shortcuts if a text field or other widget wants keyboard input
        let wants_keyboard = ctx.wants_keyboard_input();
        let modifiers = ctx.input(|i| i.modifiers);

        ctx.input(|i| {
            // File shortcuts (Ctrl+key) - always work even during text input
            if modifiers.ctrl && i.key_pressed(egui::Key::N) {
                self.request_new_project();
            }
            if modifiers.ctrl && i.key_pressed(egui::Key::O) {
                self.request_open_project();
            }
            if modifiers.ctrl && i.key_pressed(egui::Key::S) {
                if modifiers.shift {
                    self.save_project_as();
                } else {
                    self.save_project();
                }
            }

            // Edit shortcuts (Ctrl+key) - always work
            if modifiers.ctrl && i.key_pressed(egui::Key::Z) {
                if modifiers.shift {
                    self.redo();
                } else {
                    self.undo();
                }
            }

            // Track shortcuts (Ctrl+key) - always work
            if modifiers.ctrl && i.key_pressed(egui::Key::T) {
                let num = self.project.track_count() + 1;
                if modifiers.shift {
                    self.add_track(&format!("MIDI {}", num), TrackType::Midi);
                } else {
                    self.add_track(&format!("Audio {}", num), TrackType::Audio);
                }
            }

            // Non-modifier shortcuts - only when not typing in text fields
            if !wants_keyboard {
                // Transport shortcuts
                if i.key_pressed(egui::Key::Space) {
                    self.toggle_playback();
                }
                if i.key_pressed(egui::Key::Enter) {
                    self.stop_and_rewind();
                }

                // View shortcuts
                if i.key_pressed(egui::Key::P) && !modifiers.ctrl {
                    self.open_piano_roll();
                }
            }

            // Screenshot (F12) - always works
            if i.key_pressed(egui::Key::F12) {
                self.screenshot.request_screenshot(ctx);
            }
        });

        // Handle dropped files
        self.handle_dropped_files(ctx);
    }

    /// Handle files dropped onto the application window
    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped_files: Vec<_> = ctx.input(|i| i.raw.dropped_files.clone());

        for file in dropped_files {
            if let Some(path) = &file.path {
                let path_str = path.to_string_lossy();
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

                match ext.as_str() {
                    // Guitar Pro files - import to tab editor
                    "gp3" | "gp4" | "gp5" | "gpx" | "gp" => {
                        info!("Dropped GP file: {}", path_str);
                        match orpheus_file::guitar_pro::parse_file(path) {
                            Ok(gp_file) => {
                                // Convert GP file to TabDocument and load into editor
                                let tab_doc = orpheus_file::guitar_pro::convert_gp_to_tab(&gp_file);
                                self.view_states.tab_editor = TabEditorState::from_document(tab_doc);
                                self.view_states.tab_editor.file_path = Some(path.clone());
                                self.notifications.success(format!(
                                    "Imported: {}",
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                ));
                            }
                            Err(e) => {
                                self.notifications.error(format!("Failed to import GP file: {}", e));
                            }
                        }
                    }
                    // Orpheus project files
                    "orph" | "orpheus" => {
                        info!("Dropped project file: {}", path_str);
                        self.load_project_from_path(path.clone());
                    }
                    // MIDI files
                    "mid" | "midi" => {
                        info!("Dropped MIDI file: {}", path_str);
                        self.notifications.info("MIDI import coming soon");
                    }
                    // Audio files
                    "wav" | "mp3" | "flac" | "ogg" | "aiff" => {
                        info!("Dropped audio file: {}", path_str);
                        self.notifications.info("Audio import coming soon");
                    }
                    _ => {
                        self.notifications.warning(format!(
                            "Unsupported file type: .{}",
                            ext
                        ));
                    }
                }
            }
        }
    }

    /// Request new project - shows confirmation if unsaved changes
    fn request_new_project(&mut self) {
        if self.app_state.is_dirty {
            self.confirm_dialog = ConfirmDialogState {
                visible: true,
                title: "Unsaved Changes".to_string(),
                message: "You have unsaved changes. Do you want to save before creating a new project?".to_string(),
                pending_action: Some(PendingAction::NewProject),
            };
        } else {
            self.new_project();
        }
    }

    fn new_project(&mut self) {
        info!("Creating new project");
        self.project = Project::new("Untitled Project");
        self.mixer = MixerState::new();
        self.app_state.project_path = None;
        self.app_state.is_dirty = false;
        self.command_history.clear();
        // Reset tab editor with a fresh guitar track
        self.view_states.tab_editor = TabEditorState::with_guitar_track();
        self.notifications.success("New project created");
    }

    /// Request open project - shows confirmation if unsaved changes
    fn request_open_project(&mut self) {
        if self.app_state.is_dirty {
            self.confirm_dialog = ConfirmDialogState {
                visible: true,
                title: "Unsaved Changes".to_string(),
                message: "You have unsaved changes. Do you want to save before opening another project?".to_string(),
                pending_action: Some(PendingAction::OpenProject),
            };
        } else {
            self.open_project();
        }
    }

    fn open_project(&mut self) {
        info!("Opening project dialog");

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Orpheus Project", &["orpheus", "json"])
            .pick_file()
        {
            self.load_project_from_path(path);
        }
    }

    fn load_project_from_path(&mut self, path: std::path::PathBuf) {
        match Project::load(&path) {
            Ok(project) => {
                // Reset mixer for new project
                self.mixer = MixerState::new();
                for track in project.tracks_ordered() {
                    self.mixer.add_channel(track.name.clone());
                }

                self.project = project;
                self.app_state.project_path = Some(path.clone());
                self.app_state.is_dirty = false;
                self.command_history.clear();
                info!("Loaded project: {}", self.project.metadata.name);
                self.notifications.success(format!("Opened: {}", path.file_name().unwrap_or_default().to_string_lossy()));
            }
            Err(e) => {
                tracing::error!("Failed to load project: {}", e);
                self.notifications.error(format!("Failed to open project: {}", e));
            }
        }
    }

    fn save_project(&mut self) {
        info!("Saving project");

        if let Some(path) = self.app_state.project_path.clone() {
            self.save_project_to_path(&path);
        } else {
            self.save_project_as();
        }
    }

    fn save_project_as(&mut self) {
        let default_name = format!("{}.orpheus", self.project.metadata.name);

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Orpheus Project", &["orpheus"])
            .set_file_name(&default_name)
            .save_file()
        {
            self.save_project_to_path(&path);
            self.app_state.project_path = Some(path);
        }
    }

    fn save_project_to_path(&mut self, path: &std::path::Path) {
        self.project.touch();

        match self.project.save(path) {
            Ok(()) => {
                self.app_state.is_dirty = false;
                info!("Saved project to {:?}", path);
                self.notifications.success(format!("Saved: {}", path.file_name().unwrap_or_default().to_string_lossy()));
            }
            Err(e) => {
                tracing::error!("Failed to save project: {}", e);
                self.notifications.error(format!("Failed to save: {}", e));
            }
        }
    }

    /// Export project as audio file (WAV)
    fn export_audio(&mut self) {
        self.export_audio_with_format(ExportFormat::Wav24);
    }

    /// Export project with specific format
    fn export_audio_with_format(&mut self, format: ExportFormat) {
        let default_name = format!("{}.{}", self.project.metadata.name, format.extension());

        // Build file filters based on format
        let (filter_name, filter_ext) = match format {
            ExportFormat::Wav16 | ExportFormat::Wav24 | ExportFormat::Wav32Float => ("WAV Audio", "wav"),
            ExportFormat::Flac16 | ExportFormat::Flac24 => ("FLAC Audio", "flac"),
            ExportFormat::Aiff16 | ExportFormat::Aiff24 => ("AIFF Audio", "aiff"),
            _ => ("Audio File", format.extension()),
        };

        if let Some(path) = rfd::FileDialog::new()
            .add_filter(filter_name, &[filter_ext])
            .set_file_name(&default_name)
            .save_file()
        {
            self.export_to_path(&path, format);
        }
    }

    /// Export project to a specific path with format
    fn export_to_path(&mut self, path: &std::path::Path, format: ExportFormat) {
        info!("Exporting project to {:?} as {:?}", path, format);

        self.export_state.is_exporting = true;
        self.export_state.progress = 0.0;
        self.export_state.last_error = None;
        self.export_state.last_result = None;

        // Create export settings
        let mut settings = ExportSettings::high_quality();
        settings.format = format;
        settings.sample_rate = self.transport.sample_rate;

        // Perform export
        match export_project(&self.project, path, &settings) {
            Ok(result) => {
                info!("Export complete: {} ({:.2}s, {})",
                      result.path.display(),
                      result.duration_secs,
                      result.file_size_human());
                self.export_state.last_result = Some(result);
                self.export_state.show_result = true;
            }
            Err(e) => {
                tracing::error!("Export failed: {}", e);
                self.export_state.last_error = Some(e.to_string());
                self.export_state.show_result = true;
            }
        }

        self.export_state.is_exporting = false;
        self.export_state.progress = 1.0;
    }

    /// Render export progress/result dialog
    fn render_export_dialog(&mut self, ctx: &egui::Context) {
        // Show progress dialog while exporting
        if self.export_state.is_exporting {
            egui::Window::new("Exporting...")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.add_space(8.0);
                        ui.label("Exporting audio, please wait...");
                    });
                    ui.add_space(8.0);

                    // Progress bar
                    let progress = self.export_state.progress;
                    ui.add(egui::ProgressBar::new(progress)
                        .show_percentage()
                        .animate(true));

                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("This may take a moment for large projects").small().color(egui::Color32::GRAY));
                });
            return;
        }

        if !self.export_state.show_result {
            return;
        }

        let title = if self.export_state.last_error.is_some() {
            "Export Failed"
        } else {
            "Export Complete"
        };

        egui::Window::new(title)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                if let Some(ref error) = self.export_state.last_error {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("✕").color(egui::Color32::from_rgb(234, 67, 53)).size(20.0));
                        ui.add_space(8.0);
                        ui.colored_label(egui::Color32::from_rgb(234, 67, 53), "Export Failed");
                    });
                    ui.separator();
                    ui.label(error);
                } else if let Some(ref result) = self.export_state.last_result {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("✓").color(egui::Color32::from_rgb(52, 168, 83)).size(20.0));
                        ui.add_space(8.0);
                        ui.colored_label(egui::Color32::from_rgb(52, 168, 83), "Export Successful");
                    });
                    ui.separator();

                    egui::Grid::new("export_result")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("File:");
                            ui.label(result.path.file_name().unwrap_or_default().to_string_lossy().to_string());
                            ui.end_row();

                            ui.label("Format:");
                            ui.label(result.format.description());
                            ui.end_row();

                            ui.label("Duration:");
                            ui.label(result.duration_human());
                            ui.end_row();

                            ui.label("Size:");
                            ui.label(result.file_size_human());
                            ui.end_row();

                            ui.label("Sample Rate:");
                            ui.label(format!("{} Hz", result.sample_rate));
                            ui.end_row();

                            ui.label("Channels:");
                            ui.label(if result.channels == 2 { "Stereo" } else { "Mono" });
                            ui.end_row();
                        });
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        self.export_state.show_result = false;
                    }

                    if self.export_state.last_result.is_some() {
                        if ui.button("Show in Folder").clicked() {
                            if let Some(ref result) = self.export_state.last_result {
                                if let Some(parent) = result.path.parent() {
                                    let _ = open::that(parent);
                                }
                            }
                        }
                    }
                });
            });
    }

    /// Render confirmation dialog for unsaved changes
    fn render_confirm_dialog(&mut self, ctx: &egui::Context) {
        if !self.confirm_dialog.visible {
            return;
        }

        let mut should_close = false;
        let mut action = None;

        egui::Window::new(&self.confirm_dialog.title)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(&self.confirm_dialog.message);
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.save_project();
                        action = self.confirm_dialog.pending_action.clone();
                        should_close = true;
                    }
                    if ui.button("Don't Save").clicked() {
                        action = self.confirm_dialog.pending_action.clone();
                        should_close = true;
                    }
                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }
                });
            });

        if should_close {
            self.confirm_dialog.visible = false;
            self.confirm_dialog.pending_action = None;

            // Execute the pending action if not cancelled
            if let Some(pending) = action {
                match pending {
                    PendingAction::NewProject => self.new_project(),
                    PendingAction::OpenProject => self.open_project(),
                    PendingAction::CloseApp => {
                        // Would need to handle app close via viewport command
                    }
                }
            }
        }
    }

    /// Render notification toasts
    fn render_notifications(&mut self, ctx: &egui::Context) {
        // Cleanup old notifications
        self.notifications.cleanup();

        if self.notifications.notifications.is_empty() {
            return;
        }

        // Render notifications in top-right corner
        let toast_width = 300.0;
        let toast_height = 50.0;
        let margin = 12.0;
        let spacing = 8.0;

        for (i, notification) in self.notifications.notifications.iter().enumerate() {
            let y_offset = margin + (i as f32 * (toast_height + spacing));

            egui::Area::new(egui::Id::new(format!("notification_{}", i)))
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-margin, y_offset))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(40, 40, 50))
                        .rounding(egui::Rounding::same(8.0))
                        .stroke(egui::Stroke::new(1.0, notification.level.color()))
                        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.set_min_width(toast_width - 24.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(notification.level.icon())
                                        .color(notification.level.color())
                                        .size(16.0),
                                );
                                ui.add_space(8.0);
                                ui.label(&notification.message);
                            });
                        });
                });
        }
    }

    /// Render welcome dialog
    fn render_welcome_dialog(&mut self, ctx: &egui::Context) {
        let action = WelcomeDialog::new(&mut self.welcome_state).show(ctx);
        if let Some(action) = action {
            self.handle_welcome_action(action);
        }
    }

    /// Handle welcome dialog actions
    fn handle_welcome_action(&mut self, action: WelcomeAction) {
        match action {
            WelcomeAction::Close => {
                // Dialog already hidden by WelcomeDialog
            }
            WelcomeAction::DontShowAgain => {
                self.settings.ui.show_welcome_on_startup = false;
                if let Err(e) = self.settings.save() {
                    tracing::error!("Failed to save settings: {}", e);
                }
            }
            WelcomeAction::OpenPreferences => {
                self.preferences_state.visible = true;
            }
            WelcomeAction::NewProject => {
                self.new_project();
            }
        }
    }

    /// Render preferences dialog
    fn render_preferences_dialog(&mut self, ctx: &egui::Context) {
        let action = PreferencesDialog::new(&mut self.preferences_state).show(ctx);
        if let Some(action) = action {
            self.handle_preferences_action(action, ctx);
        }
    }

    /// Render about dialog
    fn render_about_dialog(&mut self, ctx: &egui::Context) {
        AboutDialog::new(&mut self.about_state).show(ctx);
    }

    /// Handle preferences dialog actions
    fn handle_preferences_action(&mut self, action: PreferencesAction, ctx: &egui::Context) {
        match action {
            PreferencesAction::Close => {
                // Dialog already hidden
            }
            PreferencesAction::Apply => {
                // Sync preferences to settings and save
                self.preferences_state.sync_to_settings(&mut self.settings);
                if let Err(e) = self.settings.save() {
                    tracing::error!("Failed to save settings: {}", e);
                    self.notifications.error("Failed to save preferences");
                } else {
                    self.notifications.success("Preferences saved");
                }
                // Sync welcome dialog setting
                self.welcome_state.show_on_startup = self.settings.ui.show_welcome_on_startup;
            }
            PreferencesAction::ResetDefaults => {
                self.preferences_state = PreferencesState::new();
                self.notifications.info("Settings reset to defaults");
            }
            PreferencesAction::ThemeChanged(mode) => {
                // Apply theme immediately
                let theme = match mode {
                    ThemeMode::Dark => OrpheusTheme::dark(),
                    ThemeMode::Light => OrpheusTheme::light(),
                    ThemeMode::HighContrast => OrpheusTheme::high_contrast(),
                };
                theme.apply(ctx);
                self.theme = theme;
            }
            PreferencesAction::ScaleChanged(scale) => {
                ctx.set_pixels_per_point(scale);
            }
        }
    }

    /// Open welcome dialog manually (from Help menu)
    fn show_welcome(&mut self) {
        self.welcome_state.reset();
        self.welcome_state.visible = true;
    }

    /// Open preferences dialog
    fn show_preferences(&mut self) {
        self.preferences_state.visible = true;
    }

    /// Open about dialog
    fn show_about(&mut self) {
        self.about_state.show();
    }

    fn undo(&mut self) {
        let mut ctx = CommandContext {
            project: &mut self.project,
            state: &mut self.app_state,
            transport: &mut self.transport,
            mixer: &mut self.mixer,
        };

        match self.command_history.undo(&mut ctx) {
            Ok(true) => {
                self.app_state.mark_dirty();
                debug!("Undo successful");
            }
            Ok(false) => {
                debug!("Nothing to undo");
            }
            Err(e) => {
                tracing::error!("Undo failed: {}", e);
            }
        }
    }

    fn redo(&mut self) {
        let mut ctx = CommandContext {
            project: &mut self.project,
            state: &mut self.app_state,
            transport: &mut self.transport,
            mixer: &mut self.mixer,
        };

        match self.command_history.redo(&mut ctx) {
            Ok(true) => {
                self.app_state.mark_dirty();
                debug!("Redo successful");
            }
            Ok(false) => {
                debug!("Nothing to redo");
            }
            Err(e) => {
                tracing::error!("Redo failed: {}", e);
            }
        }
    }

    /// Execute a command and add it to history
    fn execute_command(&mut self, command: BoxedCommand) {
        let mut ctx = CommandContext {
            project: &mut self.project,
            state: &mut self.app_state,
            transport: &mut self.transport,
            mixer: &mut self.mixer,
        };

        match self.command_history.execute(command, &mut ctx) {
            Ok(()) => {
                self.app_state.mark_dirty();
            }
            Err(e) => {
                tracing::error!("Command execution failed: {}", e);
            }
        }
    }

    /// Add a new track to the project
    fn add_track(&mut self, name: &str, track_type: TrackType) {
        let command = AddTrackCommand::new(name, track_type);
        self.execute_command(Box::new(command));
        info!("Added {} track: {}",
            match track_type {
                TrackType::Audio => "audio",
                TrackType::Midi => "MIDI",
                TrackType::Aux => "aux",
                TrackType::Master => "master",
            },
            name
        );
    }

    fn toggle_playback(&mut self) {
        if self.transport.is_playing {
            self.transport.pause();
            self.audio_engine.pause();
            debug!("Playback paused");
        } else {
            // Load clips before starting playback
            self.load_clips_to_engine();
            self.transport.play();
            self.audio_engine.play();
            debug!("Playback started");
        }
    }

    /// Load all clips from project to audio engine for playback
    fn load_clips_to_engine(&mut self) {
        let tracks: Vec<orpheus_core::Track> = self.project.tracks.values().cloned().collect();
        let total_clips: usize = tracks.iter().map(|t| t.clips.len()).sum();
        if total_clips > 0 {
            self.audio_engine.load_clips(tracks);
            debug!("Loaded {} clips to engine", total_clips);
        }
    }

    fn stop_and_rewind(&mut self) {
        self.transport.stop();
        self.transport.rewind();
        self.audio_engine.stop();
        debug!("Stopped and rewound");
    }

    /// Open the Piano Roll editor in the dock
    fn open_piano_roll(&mut self) {
        // Check if Piano Roll tab already exists and get its location
        let piano_roll_location: Option<(egui_dock::SurfaceIndex, egui_dock::NodeIndex)> =
            self.dock_layout.state.iter_all_tabs()
                .find(|(_, tab)| **tab == DockTab::PianoRoll)
                .map(|((surface_idx, node_idx), _)| (surface_idx, node_idx));

        if let Some((surface_idx, node_idx)) = piano_roll_location {
            // Focus on the existing Piano Roll tab
            self.dock_layout.state.set_focused_node_and_surface((surface_idx, node_idx));
            info!("Focused existing Piano Roll tab");
        } else {
            // Add Piano Roll as a new tab in the main area
            let surface = self.dock_layout.state.main_surface_mut();
            surface.push_to_focused_leaf(DockTab::PianoRoll);
            info!("Opened Piano Roll editor");
        }
    }

    /// Helper to render a menu item with right-aligned shortcut
    fn menu_item_with_shortcut(ui: &mut egui::Ui, label: &str, shortcut: &str) -> bool {
        let response = ui.horizontal(|ui| {
            let resp = ui.add(egui::Label::new(label).sense(egui::Sense::click()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(shortcut).color(egui::Color32::GRAY));
            });
            resp
        });

        // Make the whole row clickable
        let inner = response.inner;
        let rect = response.response.rect;
        let id = ui.make_persistent_id(label);

        // Handle hover and click on full row
        let sense = egui::Sense::click();
        let row_response = ui.interact(rect, id, sense);

        inner.clicked() || row_response.clicked()
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        // Consistent menu item width
        let menu_width = 220.0;

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // File menu
                ui.menu_button("File", |ui| {
                    ui.set_min_width(menu_width);
                    if Self::menu_item_with_shortcut(ui, "New Project", "Ctrl+N") {
                        self.request_new_project();
                        ui.close_menu();
                    }
                    if Self::menu_item_with_shortcut(ui, "Open...", "Ctrl+O") {
                        self.request_open_project();
                        ui.close_menu();
                    }
                    ui.separator();
                    if Self::menu_item_with_shortcut(ui, "Save", "Ctrl+S") {
                        self.save_project();
                        ui.close_menu();
                    }
                    if Self::menu_item_with_shortcut(ui, "Save As...", "Ctrl+Shift+S") {
                        self.save_project_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.menu_button("Export Audio", |ui| {
                        if ui.button("WAV 24-bit (High Quality)").clicked() {
                            self.export_audio_with_format(ExportFormat::Wav24);
                            ui.close_menu();
                        }
                        if ui.button("WAV 16-bit (CD Quality)").clicked() {
                            self.export_audio_with_format(ExportFormat::Wav16);
                            ui.close_menu();
                        }
                        if ui.button("WAV 32-bit Float").clicked() {
                            self.export_audio_with_format(ExportFormat::Wav32Float);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("FLAC 24-bit (Lossless)").clicked() {
                            self.export_audio_with_format(ExportFormat::Flac24);
                            ui.close_menu();
                        }
                        if ui.button("FLAC 16-bit (Lossless)").clicked() {
                            self.export_audio_with_format(ExportFormat::Flac16);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("AIFF 24-bit").clicked() {
                            self.export_audio_with_format(ExportFormat::Aiff24);
                            ui.close_menu();
                        }
                        if ui.button("AIFF 16-bit").clicked() {
                            self.export_audio_with_format(ExportFormat::Aiff16);
                            ui.close_menu();
                        }
                    });
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // Edit menu
                ui.menu_button("Edit", |ui| {
                    let undo_text = if let Some(name) = self.command_history.undo_name() {
                        format!("Undo {}  Ctrl+Z", name)
                    } else {
                        "Undo  Ctrl+Z".to_string()
                    };
                    if ui
                        .add_enabled(self.command_history.can_undo(), egui::Button::new(undo_text))
                        .clicked()
                    {
                        self.undo();
                        ui.close_menu();
                    }

                    let redo_text = if let Some(name) = self.command_history.redo_name() {
                        format!("Redo {}  Ctrl+Shift+Z", name)
                    } else {
                        "Redo  Ctrl+Shift+Z".to_string()
                    };
                    if ui
                        .add_enabled(self.command_history.can_redo(), egui::Button::new(redo_text))
                        .clicked()
                    {
                        self.redo();
                        ui.close_menu();
                    }
                });

                // Track menu
                ui.menu_button("Track", |ui| {
                    ui.set_min_width(menu_width);
                    if Self::menu_item_with_shortcut(ui, "Add Audio Track", "Ctrl+T") {
                        let num = self.project.track_count() + 1;
                        self.add_track(&format!("Audio {}", num), TrackType::Audio);
                        ui.close_menu();
                    }
                    if Self::menu_item_with_shortcut(ui, "Add MIDI Track", "Ctrl+Shift+T") {
                        let num = self.project.track_count() + 1;
                        self.add_track(&format!("MIDI {}", num), TrackType::Midi);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Add Aux Track").clicked() {
                        let num = self.project.track_count() + 1;
                        self.add_track(&format!("Aux {}", num), TrackType::Aux);
                        ui.close_menu();
                    }
                });

                // View menu
                ui.menu_button("View", |ui| {
                    ui.set_min_width(menu_width);
                    if Self::menu_item_with_shortcut(ui, "Toggle Left Panel", "Ctrl+B") {
                        self.app_state.view.left_panel_visible =
                            !self.app_state.view.left_panel_visible;
                        ui.close_menu();
                    }
                    if ui.button("Toggle Right Panel").clicked() {
                        self.app_state.view.right_panel_visible =
                            !self.app_state.view.right_panel_visible;
                        ui.close_menu();
                    }
                    ui.separator();
                    let plugin_text = if self.show_plugin_browser { "Hide Plugin Browser" } else { "Show Plugin Browser" };
                    if ui.button(plugin_text).clicked() {
                        self.show_plugin_browser = !self.show_plugin_browser;
                        ui.close_menu();
                    }
                    let keyboard_text = if self.show_virtual_keyboard { "Hide Virtual Keyboard" } else { "Show Virtual Keyboard" };
                    if ui.button(keyboard_text).clicked() {
                        self.show_virtual_keyboard = !self.show_virtual_keyboard;
                        ui.close_menu();
                    }
                    let preset_text = if self.show_synth_presets { "Hide Synth Presets" } else { "Show Synth Presets" };
                    if ui.button(preset_text).clicked() {
                        self.show_synth_presets = !self.show_synth_presets;
                        ui.close_menu();
                    }
                    let midi_text = if self.show_midi_input { "Hide MIDI Input" } else { "Show MIDI Input" };
                    if ui.button(midi_text).clicked() {
                        self.show_midi_input = !self.show_midi_input;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Piano Roll        P").clicked() {
                        self.open_piano_roll();
                        ui.close_menu();
                    }
                });

                // Help menu
                ui.menu_button("Help", |ui| {
                    if ui.button("Welcome Guide").clicked() {
                        self.show_welcome();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Preferences...").clicked() {
                        self.show_preferences();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("About Orpheus").clicked() {
                        self.show_about();
                        ui.close_menu();
                    }
                });

                // Right-aligned project info
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let title = if self.app_state.is_dirty {
                        format!("{}*", self.project.metadata.name)
                    } else {
                        self.project.metadata.name.clone()
                    };
                    ui.label(egui::RichText::new(title).small());
                });
            });
        });
    }

    /// Render transport bar
    fn render_transport(&mut self, ctx: &egui::Context) {
        let audio_connected = self.audio_engine.is_connected();

        egui::TopBottomPanel::top("transport_bar")
            .frame(egui::Frame::none().fill(self.theme.palette.bg_tertiary).inner_margin(8.0))
            .show(ctx, |ui| {
                let action =
                    TransportBar::show(ui, &mut self.transport, audio_connected);

                match action {
                    TransportAction::Rewind => self.transport.rewind(),
                    TransportAction::Stop => self.stop_and_rewind(),
                    TransportAction::PlayPause => self.toggle_playback(),
                    TransportAction::Record => {
                        if self.transport.is_recording {
                            self.audio_engine.stop_recording();
                        } else {
                            // First, arm all tracks that have record_armed set
                            for (_, track) in &self.project.tracks {
                                if track.record_armed {
                                    self.audio_engine.arm_track(track.id);
                                }
                            }
                            self.audio_engine.start_recording();
                        }
                    }
                    TransportAction::MetronomeToggle(enabled) => {
                        self.audio_engine.set_metronome_enabled(enabled);
                    }
                    TransportAction::None => {}
                }
            });
    }

    /// Render mode tabs
    fn render_mode_tabs(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("mode_tabs")
            .frame(egui::Frame::none().fill(self.theme.palette.bg_secondary).inner_margin(4.0))
            .show(ctx, |ui| {
                if let Some(new_mode) =
                    ModeTabs::show(ui, self.app_state.mode, self.theme.accent())
                {
                    self.app_state.mode = new_mode;
                    info!("Switched to {:?} mode", new_mode);
                }
            });
    }

    /// Render main dock area
    fn render_dock(&mut self, ctx: &egui::Context) {
        let mode = self.app_state.mode;
        let theme = &self.theme;
        let view_states = &mut self.view_states;
        let dock_state = &mut self.dock_layout.state;

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(theme.palette.bg_primary))
            .show(ctx, |ui| {
                let mut tab_viewer = OrpheusTabViewer {
                    mode,
                    view_states,
                    theme,
                };

                DockArea::new(dock_state)
                    .style(DockLayout::style())
                    .show_inside(ui, &mut tab_viewer);
            });
    }

    /// Render status bar
    fn render_status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::none().fill(self.theme.palette.bg_tertiary).inner_margin(4.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Mode indicator
                    ui.label(
                        egui::RichText::new(format!(
                            "{} {}",
                            self.app_state.mode.icon(),
                            self.app_state.mode.display_name()
                        ))
                        .small(),
                    );

                    ui.separator();

                    // Track count
                    ui.label(
                        egui::RichText::new(format!("{} tracks", self.project.track_count()))
                            .small(),
                    );

                    ui.separator();

                    // Sample rate
                    ui.label(
                        egui::RichText::new(format!("{} Hz", self.transport.sample_rate)).small(),
                    );

                    ui.separator();

                    // Audio engine status
                    let engine_state = self.audio_engine.playback_state();
                    let state_text = match engine_state {
                        EngineState::Stopped => "Stopped",
                        EngineState::Playing => "Playing",
                        EngineState::Paused => "Paused",
                    };
                    ui.label(egui::RichText::new(state_text).small());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Audio levels
                        let level_db_l = if self.level_left > 0.0 {
                            20.0 * self.level_left.log10()
                        } else {
                            -60.0
                        };
                        let level_db_r = if self.level_right > 0.0 {
                            20.0 * self.level_right.log10()
                        } else {
                            -60.0
                        };

                        ui.label(
                            egui::RichText::new(format!(
                                "L: {:.1} dB | R: {:.1} dB | CPU: {:.0}%",
                                level_db_l.max(-60.0),
                                level_db_r.max(-60.0),
                                self.cpu_load * 100.0
                            ))
                            .small(),
                        );
                    });
                });
            });
    }

    /// Poll audio engine status and update local state
    fn poll_audio_engine(&mut self) {
        // Get levels from engine
        let (left, right) = self.audio_engine.levels();
        self.level_left = left;
        self.level_right = right;

        // Get state
        let state = self.audio_engine.state();
        self.cpu_load = state.cpu_load;

        // Sync transport position from engine when playing
        if self.transport.is_playing {
            self.transport.position.samples = self.audio_engine.position();

            // Update arrange view playhead position
            let playhead_secs = self.transport.position.as_seconds();
            self.view_states.arrange.playhead_secs = playhead_secs;
            self.view_states.arrange.is_playing = true;
            self.view_states.arrange.is_recording = self.transport.is_recording;
        } else {
            self.view_states.arrange.is_playing = false;
            self.view_states.arrange.is_recording = false;
        }

        // Process any status messages
        while let Some(status) = self.audio_engine.try_recv_status() {
            match status {
                AudioStatus::Ready => {
                    info!("Audio engine ready");
                }
                AudioStatus::Error(msg) => {
                    tracing::error!("Audio engine error: {}", msg);
                }
                AudioStatus::ShuttingDown => {
                    info!("Audio engine shutting down");
                }
                AudioStatus::PluginScanComplete(plugins) => {
                    info!("Plugin scan complete: {} plugins found", plugins.len());
                    self.plugin_browser.scanning = false;
                    self.plugin_browser.scanned_plugins = plugins.into_iter().map(|p| {
                        ScannedPlugin {
                            name: p.name,
                            path: p.path,
                            format: match p.format {
                                PluginFormat::Vst3 => PanelPluginFormat::Vst3,
                                PluginFormat::Clap => PanelPluginFormat::Clap,
                            },
                            vendor: if p.vendor.is_empty() { None } else { Some(p.vendor) },
                            category: Some(format!("{:?}", p.category)),
                        }
                    }).collect();
                }
                AudioStatus::PluginLoaded { id, name } => {
                    info!("Plugin loaded: {} ({})", name, id);
                }
                AudioStatus::PluginUnloaded(id) => {
                    info!("Plugin unloaded: {}", id);
                }
                AudioStatus::PluginLoadError(msg) => {
                    tracing::error!("Plugin load error: {}", msg);
                    self.plugin_browser.last_error = Some(msg);
                }
                AudioStatus::LoadedPlugins(plugins) => {
                    self.plugin_browser.loaded_plugins = plugins.into_iter().map(|p| {
                        LoadedPlugin {
                            id: p.id,
                            name: p.name,
                            format: match p.format {
                                PluginFormat::Vst3 => PanelPluginFormat::Vst3,
                                PluginFormat::Clap => PanelPluginFormat::Clap,
                            },
                            active: p.active,
                            bypassed: p.bypassed,
                            parameters: Vec::new(), // Parameters will be populated when plugin is expanded
                            expanded: false,
                        }
                    }).collect();
                }
                // MIDI status
                AudioStatus::MidiDevicesAvailable(devices) => {
                    self.midi_input.scanning = false;
                    self.midi_input.available_devices = devices.into_iter().map(|d| {
                        MidiDevice {
                            index: d.index,
                            name: d.name,
                        }
                    }).collect();
                }
                AudioStatus::MidiDeviceConnected { index, name } => {
                    self.midi_input.set_connected(index, &name);
                }
                AudioStatus::MidiDeviceDisconnected => {
                    self.midi_input.set_disconnected();
                }
                AudioStatus::MidiDeviceError(msg) => {
                    self.midi_input.scanning = false;
                    self.midi_input.set_error(msg);
                }
                AudioStatus::MidiMessageReceived { description, msg_type } => {
                    let color = midi_message_color(&msg_type);

                    // Update last note for visual feedback
                    if msg_type == "NoteOn" {
                        // Parse note from description (e.g., "Note On: C4 vel: 100")
                        if let Some(note_str) = description.split(':').nth(1) {
                            if let Some(vel_str) = note_str.split("vel:").nth(1) {
                                if let Ok(vel) = vel_str.trim().parse::<u8>() {
                                    // Extract note number from the note name
                                    // For now, just store 60 as a placeholder
                                    self.midi_input.last_note = Some((60, vel));
                                }
                            }
                        }
                    }

                    self.midi_input.add_activity(description, color);
                }
                // Recording status
                AudioStatus::RecordingStarted => {
                    info!("Recording started");
                    self.transport.is_recording = true;
                    self.transport.is_playing = true;
                }
                AudioStatus::RecordingStopped(clips) => {
                    info!("Recording stopped with {} clips", clips.len());
                    self.transport.is_recording = false;
                    self.transport.is_playing = false;

                    // Add recorded clips to their respective tracks
                    for clip_info in clips {
                        if let Some(track) = self.project.tracks.get_mut(&clip_info.track_id) {
                            let note_count = clip_info.notes.len();
                            let clip = orpheus_core::Clip {
                                id: clip_info.clip_id,
                                name: clip_info.clip_name,
                                start: clip_info.start,
                                length: clip_info.length,
                                content: orpheus_core::ClipContent::Midi { notes: clip_info.notes },
                            };
                            info!("Added clip {} with {} notes to track {}",
                                clip.id, note_count, clip_info.track_id);
                            track.clips.push(clip);
                        }
                    }
                }
                AudioStatus::TrackArmed { track_id, armed } => {
                    if let Some(track) = self.project.tracks.get_mut(&track_id) {
                        track.record_armed = armed;
                        info!("Track {} armed: {}", track_id, armed);
                    }
                }
                AudioStatus::RecordingEvents { track_id, count } => {
                    debug!("Track {} has {} recorded events", track_id, count);
                }
                // Tab playback position updates
                AudioStatus::CurrentBeat(beat) => {
                    self.view_states.tab_editor.update_playhead(beat);
                }
                _ => {}
            }
        }
    }

    /// Handle plugin browser actions
    fn handle_plugin_action(&mut self, action: PluginBrowserAction) {
        match action {
            PluginBrowserAction::ScanPlugins => {
                self.plugin_browser.scanning = true;
                self.plugin_browser.last_error = None;
                self.audio_engine.scan_plugins();
            }
            PluginBrowserAction::LoadPlugin(path) => {
                self.audio_engine.load_plugin(path);
            }
            PluginBrowserAction::UnloadPlugin(id) => {
                self.audio_engine.unload_plugin(id);
            }
            PluginBrowserAction::ToggleBypass(id) => {
                // Find current bypass state and toggle it
                if let Some(plugin) = self.plugin_browser.loaded_plugins.iter().find(|p| p.id == id) {
                    self.audio_engine.set_plugin_bypass(id, !plugin.bypassed);
                }
            }
            PluginBrowserAction::TogglePluginsEnabled => {
                self.plugin_browser.plugins_enabled = !self.plugin_browser.plugins_enabled;
                self.audio_engine.set_plugins_enabled(self.plugin_browser.plugins_enabled);
            }
            PluginBrowserAction::SetParameter { plugin_id, param_id, value } => {
                // Send parameter change to audio engine
                self.audio_engine.set_plugin_parameter(plugin_id, param_id, value);
            }
            PluginBrowserAction::ToggleExpanded(id) => {
                // Toggle is handled directly in UI, but could be used for state persistence
                if let Some(plugin) = self.plugin_browser.loaded_plugins.iter_mut().find(|p| p.id == id) {
                    plugin.expanded = !plugin.expanded;
                }
            }
            PluginBrowserAction::ResetParameter { plugin_id, param_id } => {
                // Find default value and send to audio engine
                if let Some(plugin) = self.plugin_browser.loaded_plugins.iter().find(|p| p.id == plugin_id) {
                    if let Some(param) = plugin.parameters.iter().find(|p| p.id == param_id) {
                        self.audio_engine.set_plugin_parameter(plugin_id, param_id, param.default);
                    }
                }
            }
        }
    }

    /// Render plugin browser in right panel
    fn render_plugin_browser(&mut self, ctx: &egui::Context) {
        if !self.show_plugin_browser {
            return;
        }

        egui::SidePanel::right("plugin_browser")
            .default_width(280.0)
            .min_width(200.0)
            .max_width(400.0)
            .resizable(true)
            .frame(egui::Frame::none().fill(self.theme.palette.bg_secondary).inner_margin(8.0))
            .show(ctx, |ui| {
                let action = PluginBrowserPanel::new(&mut self.plugin_browser).show(ui);
                if let Some(action) = action {
                    self.handle_plugin_action(action);
                }
            });
    }

    /// Handle virtual keyboard actions
    fn handle_keyboard_action(&mut self, action: VirtualKeyboardAction) {
        use crate::audio_engine::DrumType;

        match action {
            VirtualKeyboardAction::NoteOn(note, velocity) => {
                // Piano note on
                debug!("Piano note on: {} velocity: {}", note, velocity);
                self.audio_engine.piano_note_on(note, velocity);

                // Also record if recording is active
                if self.transport.is_recording {
                    let vel_u8 = (velocity * 127.0) as u8;
                    self.audio_engine.record_virtual_note(note, vel_u8, true);
                }
            }
            VirtualKeyboardAction::NoteOff(note) => {
                debug!("Piano note off: {}", note);
                self.audio_engine.piano_note_off(note);

                // Also record if recording is active
                if self.transport.is_recording {
                    self.audio_engine.record_virtual_note(note, 0, false);
                }
            }
            VirtualKeyboardAction::DrumHit(drum, velocity) => {
                debug!("Drum hit: {:?} velocity: {}", drum, velocity);
                // Map UI drum type to audio engine drum type
                let drum_type = match drum {
                    orpheus_ui::KeyboardDrumType::Kick => DrumType::Kick,
                    orpheus_ui::KeyboardDrumType::Snare => DrumType::Snare,
                    orpheus_ui::KeyboardDrumType::HiHatClosed => DrumType::ClosedHiHat,
                    orpheus_ui::KeyboardDrumType::HiHatOpen => DrumType::OpenHiHat,
                    orpheus_ui::KeyboardDrumType::Crash => DrumType::Crash,
                    orpheus_ui::KeyboardDrumType::Ride => DrumType::Ride,
                    orpheus_ui::KeyboardDrumType::Tom1 => DrumType::HighTom,
                    orpheus_ui::KeyboardDrumType::Tom2 => DrumType::MidTom,
                    orpheus_ui::KeyboardDrumType::Tom3 => DrumType::LowTom,
                };
                self.audio_engine.drum_trigger(drum_type, velocity);
            }
            VirtualKeyboardAction::GuitarPluck(string, fret, velocity) => {
                debug!("Guitar pluck: string {} fret {} velocity: {}", string, fret, velocity);
                self.audio_engine.guitar_pluck(string, fret, velocity);
            }
            VirtualKeyboardAction::BassNoteOn(note, velocity) => {
                debug!("Bass note on: {} velocity: {}", note, velocity);
                self.audio_engine.bass_note_on(note, velocity);
            }
            VirtualKeyboardAction::BassNoteOff(note) => {
                debug!("Bass note off: {}", note);
                self.audio_engine.bass_note_off(note);
            }
        }
    }

    /// Process virtual keyboard input and render
    fn process_and_render_virtual_keyboard(&mut self, ctx: &egui::Context) {
        // Only process keyboard input if the virtual keyboard is enabled
        if self.virtual_keyboard.enabled && self.show_virtual_keyboard {
            let mut panel = VirtualKeyboardPanel::new(&mut self.virtual_keyboard);
            let actions = panel.process_input(ctx);
            for action in actions {
                self.handle_keyboard_action(action);
            }
        }
    }

    /// Render virtual keyboard in bottom panel
    fn render_virtual_keyboard(&mut self, ctx: &egui::Context) {
        if !self.show_virtual_keyboard {
            return;
        }

        egui::TopBottomPanel::bottom("virtual_keyboard")
            .min_height(100.0)
            .max_height(180.0)
            .resizable(true)
            .frame(egui::Frame::none().fill(self.theme.palette.bg_secondary).inner_margin(8.0))
            .show(ctx, |ui| {
                let mut panel = VirtualKeyboardPanel::new(&mut self.virtual_keyboard);
                let actions = panel.show(ui);
                for action in actions {
                    self.handle_keyboard_action(action);
                }
            });
    }

    /// Handle synth preset actions
    fn handle_synth_preset_action(&mut self, action: SynthPresetAction) {
        use crate::audio_engine::{BassPresetType, GuitarConfigType};

        match action {
            SynthPresetAction::SelectPreset(preset_type) => {
                match preset_type {
                    PresetType::Piano(_) => {
                        // Piano doesn't have preset variations in the synth engine yet
                        debug!("Piano preset selected");
                    }
                    PresetType::Bass(bass_preset) => {
                        let engine_preset = match bass_preset {
                            UiBassPresetType::SynthBass => BassPresetType::SynthBass,
                            UiBassPresetType::SubBass => BassPresetType::SubBass,
                            UiBassPresetType::FunkBass => BassPresetType::FunkBass,
                            UiBassPresetType::Bass808 => BassPresetType::Bass808,
                            UiBassPresetType::WobbleBass => BassPresetType::WobbleBass,
                            UiBassPresetType::MoogBass => BassPresetType::MoogBass,
                            UiBassPresetType::ReeseBass => BassPresetType::ReeseBass,
                        };
                        self.audio_engine.set_bass_preset(engine_preset);
                        info!("Bass preset changed to {:?}", bass_preset);
                    }
                    PresetType::Guitar(guitar_preset) => {
                        let engine_config = match guitar_preset {
                            UiGuitarPresetType::Acoustic => GuitarConfigType::Standard6,
                            UiGuitarPresetType::ElectricClean => GuitarConfigType::Standard6,
                            UiGuitarPresetType::Nylon => GuitarConfigType::Standard6,
                            UiGuitarPresetType::Bass => GuitarConfigType::Bass,
                            UiGuitarPresetType::SevenStringMetal => GuitarConfigType::SevenStringMetal,
                            UiGuitarPresetType::EightStringMetal => GuitarConfigType::EightStringMetal,
                            UiGuitarPresetType::TechDeath => GuitarConfigType::TechDeath,
                        };
                        self.audio_engine.send(crate::audio_engine::AudioCommand::SetGuitarConfig(engine_config)).ok();
                        info!("Guitar preset changed to {:?}", guitar_preset);
                    }
                    PresetType::Drums(drum_preset) => {
                        // Drum machine configuration
                        debug!("Drum preset selected: {:?}", drum_preset);
                        // Drum presets would configure the drum machine
                    }
                }
            }
            SynthPresetAction::ToggleFavorite(name) => {
                self.synth_presets.toggle_favorite(&name);
                debug!("Toggled favorite: {}", name);
            }
        }
    }

    /// Render synth preset browser in left panel
    fn render_synth_presets(&mut self, ctx: &egui::Context) {
        if !self.show_synth_presets {
            return;
        }

        egui::SidePanel::left("synth_presets")
            .default_width(250.0)
            .min_width(200.0)
            .max_width(350.0)
            .resizable(true)
            .frame(egui::Frame::none().fill(self.theme.palette.bg_secondary).inner_margin(8.0))
            .show(ctx, |ui| {
                let action = SynthPresetPanel::new(&mut self.synth_presets).show(ui);
                if let Some(action) = action {
                    self.handle_synth_preset_action(action);
                }
            });
    }

    /// Handle MIDI input actions
    fn handle_midi_action(&mut self, action: MidiInputAction) {
        match action {
            MidiInputAction::ScanDevices => {
                self.midi_input.scanning = true;
                self.audio_engine.scan_midi_devices();
            }
            MidiInputAction::Connect(index) => {
                self.audio_engine.connect_midi_device(index);
            }
            MidiInputAction::Disconnect => {
                self.audio_engine.disconnect_midi_device();
            }
            MidiInputAction::ToggleLearnMode => {
                self.midi_input.learn_mode = !self.midi_input.learn_mode;
            }
            MidiInputAction::SetChannelFilter(channel) => {
                self.midi_input.channel_filter = channel;
                self.audio_engine.set_midi_channel_filter(channel);
            }
            MidiInputAction::ClearActivity => {
                self.midi_input.activity.clear();
            }
        }
    }

    /// Render MIDI input panel (shares left panel with synth presets)
    fn render_midi_input(&mut self, ctx: &egui::Context) {
        if !self.show_midi_input {
            return;
        }

        // Use left panel if synth presets aren't showing, otherwise use a window
        if self.show_synth_presets {
            // Show as floating window since synth presets are using left panel
            egui::Window::new("MIDI Input")
                .default_width(280.0)
                .default_height(400.0)
                .resizable(true)
                .show(ctx, |ui| {
                    let action = MidiInputPanel::new(&mut self.midi_input).show(ui);
                    if let Some(action) = action {
                        self.handle_midi_action(action);
                    }
                });
        } else {
            // Use left panel
            egui::SidePanel::left("midi_input")
                .default_width(280.0)
                .min_width(200.0)
                .max_width(350.0)
                .resizable(true)
                .frame(egui::Frame::none().fill(self.theme.palette.bg_secondary).inner_margin(8.0))
                .show(ctx, |ui| {
                    let action = MidiInputPanel::new(&mut self.midi_input).show(ui);
                    if let Some(action) = action {
                        self.handle_midi_action(action);
                    }
                });
        }
    }

    /// Poll practice view for pending audio actions
    fn poll_practice_actions(&mut self) {
        let actions = self.view_states.practice.drain_actions();
        for action in actions {
            match action {
                PracticeAction::Play { tempo, loop_start, loop_end } => {
                    info!("Practice play at {} BPM", tempo);
                    self.audio_engine.set_tempo(tempo as f64);

                    // Set loop region if provided
                    if let (Some(start), Some(end)) = (loop_start, loop_end) {
                        self.audio_engine.set_loop(start, end);
                        info!("Practice loop: {} - {} samples", start, end);
                    }

                    // Start the metronome playback
                    self.audio_engine.play();
                    self.transport.is_playing = true;
                }
                PracticeAction::Stop => {
                    info!("Practice stop");
                    self.audio_engine.stop();
                    self.audio_engine.clear_loop();
                    self.transport.is_playing = false;
                }
                PracticeAction::Pause => {
                    info!("Practice pause");
                    self.audio_engine.pause();
                    self.transport.is_playing = false;
                }
                PracticeAction::SetMetronome(enabled) => {
                    debug!("Practice metronome: {}", enabled);
                    self.audio_engine.set_metronome_enabled(enabled);
                }
                PracticeAction::SetMetronomeVolume(volume) => {
                    debug!("Practice metronome volume: {}", volume);
                    self.audio_engine.set_metronome_volume(volume);
                }
                PracticeAction::SeekToMeasure(measure) => {
                    // Convert measure to samples (simplified)
                    let tempo = self.view_states.practice.session.effective_tempo();
                    let samples_per_beat = 44100 * 60 / tempo as u64;
                    let time_sig = self.view_states.practice.gp_file.as_ref()
                        .map(|f| f.time_signature.numerator)
                        .unwrap_or(4);
                    let samples_per_measure = samples_per_beat * time_sig as u64;
                    let sample_pos = (measure as u64 - 1) * samples_per_measure;
                    self.audio_engine.seek(sample_pos);
                }
            }
        }
    }

    /// Poll tab editor for pending audio preview actions
    fn poll_tab_editor_actions(&mut self) {
        let actions = self.view_states.tab_editor.drain_actions();
        for action in actions {
            match action {
                PendingTabAction::PreviewNote { string, fret, velocity } => {
                    // Convert velocity from 0-127 to 0.0-1.0
                    let velocity_f = velocity as f32 / 127.0;
                    debug!("Tab preview: string {} fret {} vel {}", string, fret, velocity_f);
                    self.audio_engine.guitar_pluck(string, fret, velocity_f);
                }
                PendingTabAction::SetMetronome(enabled) => {
                    debug!("Tab metronome: {}", enabled);
                    self.audio_engine.set_metronome_enabled(enabled);
                }
                PendingTabAction::SetMetronomeVolume(volume) => {
                    debug!("Tab metronome volume: {}", volume);
                    self.audio_engine.set_metronome_volume(volume);
                }
                PendingTabAction::SetTempo(bpm) => {
                    debug!("Tab tempo: {} BPM", bpm);
                    self.audio_engine.set_tempo(bpm);
                }
                PendingTabAction::StartPlayback { events, tempo, loop_region } => {
                    info!("Tab playback: {} events at {} BPM", events.len(), tempo);

                    // Convert TabPlaybackEvent to audio engine NoteEvent format
                    let note_events: Vec<crate::audio_engine::TabNoteEvent> = events
                        .into_iter()
                        .map(|e| crate::audio_engine::TabNoteEvent {
                            string: e.string,
                            fret: e.fret,
                            velocity: e.velocity,
                            sample_pos: e.sample_pos,
                            event_type: match e.event_type {
                                TabPlaybackEventType::NoteOn => NoteEventType::NoteOn,
                                TabPlaybackEventType::NoteOff => NoteEventType::NoteOff,
                                TabPlaybackEventType::HammerOn => NoteEventType::HammerOn,
                                TabPlaybackEventType::PullOff => NoteEventType::PullOff,
                                TabPlaybackEventType::Slide => NoteEventType::Slide,
                            },
                        })
                        .collect();

                    // Set tempo first
                    self.audio_engine.set_tempo(tempo);

                    // Load tab events
                    self.audio_engine.load_tab(note_events);

                    // Set loop region if practice mode is enabled
                    if let Some((start, end)) = loop_region {
                        self.audio_engine.set_loop(start, end);
                        info!("Loop region set: {} - {} samples", start, end);
                    } else {
                        self.audio_engine.clear_loop();
                    }

                    // Start playback
                    self.audio_engine.play();
                }
                PendingTabAction::StopPlayback => {
                    info!("Tab playback stopped");
                    self.audio_engine.stop();
                    self.audio_engine.clear_tab();
                }
                PendingTabAction::PausePlayback => {
                    info!("Tab playback paused");
                    self.audio_engine.pause();
                }
                PendingTabAction::SeekTo(beat) => {
                    // Convert beat to samples (assuming 44100 Hz sample rate)
                    let samples_per_beat = (44100.0 * 60.0 / self.view_states.tab_editor.tempo) as u64;
                    let sample_pos = (beat * samples_per_beat as f64) as u64;
                    self.audio_engine.seek(sample_pos);
                }
                PendingTabAction::ExportPdf => {
                    info!("Tab export PDF requested");
                    self.export_tab_pdf();
                }
                PendingTabAction::ExportAudio => {
                    info!("Tab export audio requested");
                    self.export_tab_audio();
                }
            }
        }
    }

    /// Export tab document to audio (WAV)
    fn export_tab_audio(&mut self) {
        use orpheus_export::{export_tab, ExportSettings};

        // Get document and tempo
        let document = &self.view_states.tab_editor.document;
        let tempo = self.view_states.tab_editor.tempo;

        // Generate default filename from title
        let default_name = if document.metadata.title.is_empty() {
            "untitled.wav".to_string()
        } else {
            format!("{}.wav", document.metadata.title.replace(' ', "_"))
        };

        // Use CD quality settings (16-bit, 44.1kHz stereo)
        let settings = ExportSettings::cd_quality();

        // Export to current directory (simple fallback - no dialog support needed)
        let path = std::path::PathBuf::from(&default_name);
        match export_tab(document, tempo, &path, &settings) {
            Ok(result) => {
                info!("Exported audio to {:?}: {} ({:.2}s)",
                    result.path,
                    result.file_size_human(),
                    result.duration_secs);
            }
            Err(e) => {
                tracing::error!("Audio export failed: {}", e);
            }
        }
    }

    /// Export tab document to PDF
    fn export_tab_pdf(&mut self) {
        use orpheus_file::{export_pdf, PdfExportOptions};

        // Get document reference
        let document = &self.view_states.tab_editor.document;

        // Generate default filename from title
        let default_name = if document.metadata.title.is_empty() {
            "untitled.pdf".to_string()
        } else {
            format!("{}.pdf", document.metadata.title.replace(' ', "_"))
        };

        // Use file dialog to get save path
        #[cfg(feature = "dialogs")]
        {
            use orpheus_file::SaveDialog;

            let path = SaveDialog::new()
                .add_filter("PDF files", &["pdf"])
                .set_filename(&default_name)
                .save();

            if let Some(path) = path {
                let options = PdfExportOptions::default();
                match export_pdf(document, &path, &options) {
                    Ok(result) => {
                        info!("Exported PDF: {} pages, {} measures",
                            result.page_count, result.measure_count);
                    }
                    Err(e) => {
                        tracing::error!("PDF export failed: {}", e);
                    }
                }
            }
        }

        // Fallback for non-dialog builds: save to current directory
        #[cfg(not(feature = "dialogs"))]
        {
            let path = std::path::PathBuf::from(&default_name);
            let options = PdfExportOptions::default();
            match export_pdf(document, &path, &options) {
                Ok(result) => {
                    info!("Exported PDF to {:?}: {} pages, {} measures",
                        path, result.page_count, result.measure_count);
                }
                Err(e) => {
                    tracing::error!("PDF export failed: {}", e);
                }
            }
        }
    }

}

impl eframe::App for OrpheusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;

        // Apply theme
        self.theme.apply(ctx);

        // Handle shortcuts
        self.handle_shortcuts(ctx);

        // Handle screenshot events
        self.screenshot.handle_events(ctx);

        // Poll audio engine for status updates
        self.poll_audio_engine();

        // Request repaint when playing for smooth updates
        if self.transport.is_playing {
            ctx.request_repaint();
        }

        // Process virtual keyboard input (before other UI to capture keys)
        self.process_and_render_virtual_keyboard(ctx);

        // Render UI
        self.render_menu_bar(ctx);
        self.render_transport(ctx);
        self.render_mode_tabs(ctx);
        self.render_synth_presets(ctx);   // Left panel - must be before dock
        self.render_midi_input(ctx);      // Left panel or window - must be before dock
        self.render_plugin_browser(ctx);  // Right panel - must be before dock
        self.render_virtual_keyboard(ctx); // Bottom panel - must be before dock
        self.render_dock(ctx);
        self.render_status_bar(ctx);

        // Poll tab editor for audio preview actions
        self.poll_tab_editor_actions();

        // Poll practice mode for audio actions
        self.poll_practice_actions();

        // Modal dialogs
        self.render_export_dialog(ctx);
        self.render_confirm_dialog(ctx);
        self.render_welcome_dialog(ctx);
        self.render_preferences_dialog(ctx);
        self.render_about_dialog(ctx);

        // Notification toasts (always on top)
        self.render_notifications(ctx);
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // Save settings on exit
        if let Err(e) = self.settings.save() {
            tracing::error!("Failed to save settings: {}", e);
        }
    }
}
