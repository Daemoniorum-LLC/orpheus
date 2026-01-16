# Orpheus Native Desktop Application Architecture

## Overview

**orpheus-desktop** is a native Nyx/Linux desktop application for the Orpheus music production platform, built entirely in Rust using egui/wgpu for rendering and integrating with the existing Maestro audio engine via gRPC.

### Design Goals

1. **Native Performance**: < 10ms audio latency, 60 FPS UI rendering
2. **Unified Experience**: All 6 production modes in a single application
3. **Platform Integration**: Native Nyx/Linux look and feel via daemoniorum-egui
4. **Modular Architecture**: Plugin-ready, extensible crate structure
5. **Offline-First**: Full functionality without network connection
6. **AI-Integrated**: Seamless persona assistance across all modes

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ORPHEUS DESKTOP APPLICATION                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                     UI LAYER (egui + wgpu)                              │ │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │ │
│  │  │  Transport  │ │ Mode Tabs   │ │  Toolbar    │ │  Menu Bar   │       │ │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘       │ │
│  │  ┌───────────────────────────────────────────────────────────────────┐ │ │
│  │  │                      MODE VIEWS (egui_dock)                        │ │ │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐     │ │ │
│  │  │  │ Compose │ │ Record  │ │   Mix   │ │ Master  │ │Practice │     │ │ │
│  │  │  │  View   │ │  View   │ │  View   │ │  View   │ │  View   │     │ │ │
│  │  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘     │ │ │
│  │  └───────────────────────────────────────────────────────────────────┘ │ │
│  │  ┌─────────────────────┐ ┌──────────────────────────────────────────┐ │ │
│  │  │    Panel System     │ │           Overlay System                  │ │ │
│  │  │  - Track List       │ │  - Command Palette                        │ │ │
│  │  │  - Inspector        │ │  - AI Chat                                │ │ │
│  │  │  - Browser          │ │  - Settings                               │ │ │
│  │  │  - AI Assistant     │ │  - File Dialogs                           │ │ │
│  │  └─────────────────────┘ └──────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                    APPLICATION CORE                                     │ │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │ │
│  │  │   State     │ │  Commands   │ │   Events    │ │  Shortcuts  │       │ │
│  │  │  Manager    │ │   System    │ │    Bus      │ │   Handler   │       │ │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘       │ │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │ │
│  │  │  Undo/Redo  │ │  Project    │ │   Theme     │ │    Config   │       │ │
│  │  │   History   │ │  Manager    │ │   Engine    │ │   Loader    │       │ │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘       │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                    SERVICE LAYER                                        │ │
│  │  ┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────┐   │ │
│  │  │    Audio Client     │ │     AI Client       │ │  File Service   │   │ │
│  │  │  (gRPC to Maestro)  │ │ (Leviathan Agent)   │ │  (.maestro I/O) │   │ │
│  │  └─────────────────────┘ └─────────────────────┘ └─────────────────┘   │ │
│  │  ┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────┐   │ │
│  │  │   MIDI Service      │ │   Plugin Host       │ │  Export Service │   │ │
│  │  │  (Input/Output)     │ │   (VST3/CLAP)       │ │  (WAV/MP3/FLAC) │   │ │
│  │  └─────────────────────┘ └─────────────────────┘ └─────────────────────┘│ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
                    ▼               ▼               ▼
            ┌───────────────┐ ┌───────────┐ ┌──────────────┐
            │ Maestro Audio │ │ Leviathan │ │   System     │
            │    Server     │ │ AI Server │ │  Services    │
            │  (gRPC:50051) │ │           │ │ (ALSA, MIDI) │
            └───────────────┘ └───────────┘ └──────────────┘
                    │
                    ▼
            ┌───────────────┐
            │  JUCE Engine  │
            │  (C++ DSP)    │
            └───────────────┘
```

---

## Crate Structure

```
orpheus-desktop/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── orpheus-app/              # Main application binary
│   │   ├── src/
│   │   │   ├── main.rs           # Entry point, eframe setup
│   │   │   ├── app.rs            # OrpheusApp struct, eframe::App impl
│   │   │   ├── startup.rs        # Initialization, service connections
│   │   │   └── shutdown.rs       # Graceful cleanup
│   │   └── Cargo.toml
│   │
│   ├── orpheus-core/             # Core types and state management
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── state/            # Application state
│   │   │   │   ├── mod.rs
│   │   │   │   ├── app_state.rs  # Global app state
│   │   │   │   ├── project.rs    # Project state (tracks, timeline)
│   │   │   │   ├── transport.rs  # Playback state
│   │   │   │   ├── mixer.rs      # Mixer state
│   │   │   │   └── selection.rs  # Selection state
│   │   │   ├── commands/         # Command pattern
│   │   │   │   ├── mod.rs
│   │   │   │   ├── command.rs    # Command trait
│   │   │   │   ├── history.rs    # Undo/redo stack
│   │   │   │   └── registry.rs   # Command dispatch
│   │   │   ├── events/           # Event bus
│   │   │   │   ├── mod.rs
│   │   │   │   ├── event.rs      # Event types
│   │   │   │   └── bus.rs        # Pub/sub system
│   │   │   ├── project/          # Project model
│   │   │   │   ├── mod.rs
│   │   │   │   ├── track.rs      # Track types
│   │   │   │   ├── clip.rs       # Audio/MIDI clips
│   │   │   │   ├── timeline.rs   # Timeline model
│   │   │   │   └── automation.rs # Automation curves
│   │   │   └── config/           # Configuration
│   │   │       ├── mod.rs
│   │   │       ├── settings.rs   # User settings
│   │   │       └── shortcuts.rs  # Key bindings
│   │   └── Cargo.toml
│   │
│   ├── orpheus-ui/               # UI components and views
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── layout/           # Layout system
│   │   │   │   ├── mod.rs
│   │   │   │   ├── dock.rs       # Docking system
│   │   │   │   └── panels.rs     # Panel manager
│   │   │   ├── toolbar/          # Toolbar components
│   │   │   │   ├── mod.rs
│   │   │   │   ├── transport.rs  # Transport controls
│   │   │   │   ├── mode_tabs.rs  # Mode switcher
│   │   │   │   └── tools.rs      # Tool palette
│   │   │   ├── views/            # Mode views
│   │   │   │   ├── mod.rs
│   │   │   │   ├── compose/      # Compose mode
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── tab_editor.rs
│   │   │   │   │   ├── score_view.rs
│   │   │   │   │   ├── fretboard.rs
│   │   │   │   │   └── chord_palette.rs
│   │   │   │   ├── record/       # Record mode
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── arrangement.rs
│   │   │   │   │   ├── timeline.rs
│   │   │   │   │   └── waveform.rs
│   │   │   │   ├── mix/          # Mix mode
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── mixer.rs
│   │   │   │   │   ├── channel_strip.rs
│   │   │   │   │   ├── eq_view.rs
│   │   │   │   │   └── routing.rs
│   │   │   │   ├── master/       # Master mode
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── mastering_chain.rs
│   │   │   │   │   ├── meters.rs
│   │   │   │   │   └── loudness.rs
│   │   │   │   └── practice/     # Practice mode
│   │   │   │       ├── mod.rs
│   │   │   │       ├── speed_trainer.rs
│   │   │   │       ├── loop_section.rs
│   │   │   │       └── performance.rs
│   │   │   ├── panels/           # Dockable panels
│   │   │   │   ├── mod.rs
│   │   │   │   ├── tracks.rs     # Track list
│   │   │   │   ├── inspector.rs  # Properties
│   │   │   │   ├── browser.rs    # File browser
│   │   │   │   ├── ai_chat.rs    # AI assistant
│   │   │   │   └── effects.rs    # Effects rack
│   │   │   ├── overlays/         # Modal overlays
│   │   │   │   ├── mod.rs
│   │   │   │   ├── command_palette.rs
│   │   │   │   ├── settings.rs
│   │   │   │   └── export.rs
│   │   │   └── widgets/          # Custom widgets
│   │   │       ├── mod.rs
│   │   │       ├── knob.rs       # Rotary knob
│   │   │       ├── fader.rs      # Volume fader
│   │   │       ├── meter.rs      # Level meters
│   │   │       ├── waveform.rs   # Waveform display
│   │   │       ├── piano_roll.rs # MIDI editor
│   │   │       └── automation.rs # Automation editor
│   │   └── Cargo.toml
│   │
│   ├── orpheus-audio/            # Audio service client
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── client.rs         # gRPC client to Maestro
│   │   │   ├── playback.rs       # Playback engine
│   │   │   ├── recording.rs      # Recording engine
│   │   │   └── monitoring.rs     # Input monitoring
│   │   └── Cargo.toml
│   │
│   ├── orpheus-midi/             # MIDI handling
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── input.rs          # MIDI input handling
│   │   │   ├── output.rs         # MIDI output
│   │   │   ├── mapping.rs        # Controller mapping
│   │   │   └── devices.rs        # Device enumeration
│   │   └── Cargo.toml
│   │
│   ├── orpheus-file/             # File format handling
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── maestro.rs        # .maestro format
│   │   │   ├── guitar_pro.rs     # GP import
│   │   │   ├── midi_file.rs      # MIDI import/export
│   │   │   └── audio.rs          # Audio file I/O
│   │   └── Cargo.toml
│   │
│   ├── orpheus-ai/               # AI integration
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── client.rs         # Leviathan client
│   │   │   ├── personas.rs       # Persona selection
│   │   │   ├── chat.rs           # Chat interface
│   │   │   └── suggestions.rs    # Context-aware suggestions
│   │   └── Cargo.toml
│   │
│   ├── orpheus-plugins/          # Plugin hosting
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── scanner.rs        # Plugin discovery
│   │   │   ├── host.rs           # VST3/CLAP host
│   │   │   └── ui.rs             # Plugin UI hosting
│   │   └── Cargo.toml
│   │
│   └── orpheus-export/           # Export functionality
│       ├── src/
│       │   ├── lib.rs
│       │   ├── render.rs         # Offline render
│       │   ├── formats.rs        # WAV/MP3/FLAC/OGG
│       │   └── stems.rs          # Stem export
│       └── Cargo.toml
│
├── assets/                       # Application assets
│   ├── icons/
│   ├── fonts/
│   └── themes/
│
└── proto/                        # Additional proto definitions
    └── orpheus.proto
```

---

## Core Components

### 1. OrpheusApp (Main Application)

```rust
// orpheus-app/src/app.rs

pub struct OrpheusApp {
    // Core state
    state: AppState,
    project: Option<Project>,

    // UI systems
    theme: Theme,
    dock_state: DockState<Tab>,
    panel_manager: PanelManager,

    // Services
    audio_client: AudioClient,
    midi_service: MidiService,
    ai_client: AiClient,

    // Commands
    command_registry: CommandRegistry,
    command_history: CommandHistory,

    // Events
    event_bus: EventBus,

    // Overlays
    command_palette_open: bool,
    settings_open: bool,
    export_dialog: Option<ExportDialog>,

    // Runtime
    tokio_runtime: tokio::runtime::Runtime,
}

impl eframe::App for OrpheusApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 1. Apply theme
        self.theme.apply(ctx);

        // 2. Handle keyboard shortcuts
        self.handle_shortcuts(ctx);

        // 3. Poll async services
        self.poll_services();

        // 4. Process events
        self.process_events();

        // 5. Render UI
        self.render_menu_bar(ctx);
        self.render_toolbar(ctx);
        self.render_dock(ctx);
        self.render_overlays(ctx);

        // 6. Request continuous repaint for audio visualization
        if self.state.transport.is_playing {
            ctx.request_repaint();
        }
    }
}
```

### 2. State Management

```rust
// orpheus-core/src/state/app_state.rs

#[derive(Default)]
pub struct AppState {
    pub mode: ProductionMode,
    pub transport: TransportState,
    pub mixer: MixerState,
    pub selection: SelectionState,
    pub view: ViewState,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProductionMode {
    Compose,
    Record,
    Mix,
    Master,
    Practice,
    Distribute,
}

pub struct TransportState {
    pub is_playing: bool,
    pub is_recording: bool,
    pub position: TimePosition,
    pub tempo: f64,
    pub time_signature: TimeSignature,
    pub loop_enabled: bool,
    pub loop_start: TimePosition,
    pub loop_end: TimePosition,
}

pub struct MixerState {
    pub master_volume: f32,
    pub master_pan: f32,
    pub solo_mode: SoloMode,
    pub channels: Vec<ChannelState>,
}
```

### 3. Command System

```rust
// orpheus-core/src/commands/command.rs

pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()>;
    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()>;
    fn merge(&mut self, other: &dyn Command) -> bool { false }
}

pub struct CommandContext<'a> {
    pub project: &'a mut Project,
    pub state: &'a mut AppState,
    pub audio: &'a AudioClient,
}

// Example commands
pub struct SetTrackVolumeCommand {
    track_id: TrackId,
    old_volume: f32,
    new_volume: f32,
}

impl Command for SetTrackVolumeCommand {
    fn name(&self) -> &str { "Set Track Volume" }

    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()> {
        ctx.project.tracks.get_mut(&self.track_id)?.volume = self.new_volume;
        Ok(())
    }

    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()> {
        ctx.project.tracks.get_mut(&self.track_id)?.volume = self.old_volume;
        Ok(())
    }

    fn merge(&mut self, other: &dyn Command) -> bool {
        if let Some(cmd) = other.downcast_ref::<SetTrackVolumeCommand>() {
            if cmd.track_id == self.track_id {
                self.new_volume = cmd.new_volume;
                return true;
            }
        }
        false
    }
}
```

### 4. Event System

```rust
// orpheus-core/src/events/event.rs

#[derive(Clone)]
pub enum AppEvent {
    // Transport
    PlaybackStarted,
    PlaybackStopped,
    PositionChanged(TimePosition),
    TempoChanged(f64),

    // Project
    ProjectLoaded(PathBuf),
    ProjectSaved(PathBuf),
    ProjectModified,

    // Tracks
    TrackAdded(TrackId),
    TrackRemoved(TrackId),
    TrackSelected(TrackId),

    // Audio
    AudioServiceConnected,
    AudioServiceDisconnected,
    LatencyChanged(Duration),

    // AI
    AiResponseReceived(String),
    AiSuggestion(Suggestion),

    // UI
    ModeChanged(ProductionMode),
    PanelToggled(PanelId, bool),
}
```

---

## Mode Views

### Compose Mode

```rust
// orpheus-ui/src/views/compose/mod.rs

pub struct ComposeView {
    tab_editor: TabEditor,
    score_view: ScoreView,
    fretboard: FretboardView,
    chord_palette: ChordPalette,

    // State
    selected_track: Option<TrackId>,
    edit_mode: EditMode,
    note_value: NoteValue,
    show_tablature: bool,
    show_notation: bool,
}

impl ComposeView {
    pub fn show(&mut self, ui: &mut egui::Ui, project: &mut Project, events: &EventBus) {
        // Horizontal split: main editor + sidebar
        egui::SidePanel::right("compose_sidebar")
            .default_width(250.0)
            .show_inside(ui, |ui| {
                self.chord_palette.show(ui);
                ui.separator();
                self.fretboard.show(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // Vertical split: tabs/notation + piano roll
            let available = ui.available_rect_before_wrap();
            let split_y = available.height() * 0.6;

            ui.allocate_ui_at_rect(
                egui::Rect::from_min_size(available.min, egui::vec2(available.width(), split_y)),
                |ui| {
                    if self.show_tablature {
                        self.tab_editor.show(ui, project);
                    }
                    if self.show_notation {
                        self.score_view.show(ui, project);
                    }
                }
            );
        });
    }
}
```

### Mix Mode

```rust
// orpheus-ui/src/views/mix/mixer.rs

pub struct MixerView {
    channel_strips: Vec<ChannelStrip>,
    master_strip: MasterStrip,
    routing_matrix: RoutingMatrix,

    // UI state
    show_routing: bool,
    meter_mode: MeterMode,
    fader_size: FaderSize,
}

impl MixerView {
    pub fn show(&mut self, ui: &mut egui::Ui, mixer: &mut MixerState) {
        egui::TopBottomPanel::bottom("routing_panel")
            .show_animated_inside(ui, self.show_routing, |ui| {
                self.routing_matrix.show(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Channel strips
                    for (i, channel) in mixer.channels.iter_mut().enumerate() {
                        self.channel_strips[i].show(ui, channel);
                        ui.separator();
                    }

                    // Master strip
                    ui.separator();
                    self.master_strip.show(ui, &mut mixer.master_volume, &mut mixer.master_pan);
                });
            });
        });
    }
}
```

---

## Custom Widgets

### Audio Meter

```rust
// orpheus-ui/src/widgets/meter.rs

pub struct AudioMeter {
    peak_hold_time: Duration,
    fall_speed: f32,
    show_peak_hold: bool,
    orientation: Orientation,
}

impl AudioMeter {
    pub fn show(&self, ui: &mut egui::Ui, level_db: f32, peak_db: f32) -> Response {
        let desired_size = match self.orientation {
            Orientation::Vertical => egui::vec2(20.0, 150.0),
            Orientation::Horizontal => egui::vec2(150.0, 20.0),
        };

        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background
            painter.rect_filled(rect, 2.0, Color32::from_gray(30));

            // Level gradient
            let level_normalized = db_to_linear(level_db);
            let level_rect = self.calculate_level_rect(rect, level_normalized);

            let gradient = self.create_gradient(level_normalized);
            painter.rect_filled(level_rect, 0.0, gradient);

            // Peak hold indicator
            if self.show_peak_hold {
                let peak_normalized = db_to_linear(peak_db);
                let peak_y = self.calculate_peak_position(rect, peak_normalized);
                painter.line_segment(
                    [pos2(rect.left(), peak_y), pos2(rect.right(), peak_y)],
                    Stroke::new(2.0, Color32::WHITE),
                );
            }

            // Scale markers (-60, -30, -12, -6, -3, 0, +3 dB)
            self.draw_scale(painter, rect);
        }

        response
    }
}
```

### Rotary Knob

```rust
// orpheus-ui/src/widgets/knob.rs

pub struct Knob {
    value: f32,
    min: f32,
    max: f32,
    default: f32,
    label: String,
    format: KnobFormat,
    size: f32,
}

impl Knob {
    pub fn show(&mut self, ui: &mut egui::Ui) -> Response {
        let desired_size = egui::vec2(self.size, self.size + 20.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        // Handle input
        if response.dragged() {
            let delta = response.drag_delta();
            let sensitivity = (self.max - self.min) / 200.0;
            self.value = (self.value - delta.y * sensitivity).clamp(self.min, self.max);
        }

        if response.double_clicked() {
            self.value = self.default;
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let knob_rect = egui::Rect::from_center_size(
                rect.center() - egui::vec2(0.0, 10.0),
                egui::vec2(self.size, self.size),
            );

            // Draw knob body
            painter.circle_filled(knob_rect.center(), self.size / 2.0, Color32::from_gray(60));
            painter.circle_stroke(knob_rect.center(), self.size / 2.0, Stroke::new(2.0, Color32::from_gray(80)));

            // Draw indicator
            let angle = self.value_to_angle();
            let indicator_length = self.size / 2.0 - 4.0;
            let indicator_end = knob_rect.center() + egui::vec2(
                angle.cos() * indicator_length,
                angle.sin() * indicator_length,
            );
            painter.line_segment(
                [knob_rect.center(), indicator_end],
                Stroke::new(3.0, Color32::WHITE),
            );

            // Draw label
            painter.text(
                pos2(rect.center().x, rect.bottom() - 5.0),
                Align2::CENTER_BOTTOM,
                &self.label,
                FontId::proportional(11.0),
                Color32::GRAY,
            );

            // Draw value on hover
            if response.hovered() {
                let value_text = self.format_value();
                painter.text(
                    knob_rect.center(),
                    Align2::CENTER_CENTER,
                    &value_text,
                    FontId::proportional(10.0),
                    Color32::WHITE,
                );
            }
        }

        response
    }
}
```

---

## Service Integration

### Audio Client

```rust
// orpheus-audio/src/client.rs

pub struct AudioClient {
    client: Option<AudioProcessorClient<Channel>>,
    runtime: Arc<tokio::runtime::Runtime>,
    connected: Arc<AtomicBool>,
    latency: Arc<AtomicU32>,
}

impl AudioClient {
    pub async fn connect(address: &str) -> Result<Self> {
        let client = AudioProcessorClient::connect(address.to_string()).await?;

        Ok(Self {
            client: Some(client),
            runtime: Arc::new(tokio::runtime::Runtime::new()?),
            connected: Arc::new(AtomicBool::new(true)),
            latency: Arc::new(AtomicU32::new(0)),
        })
    }

    pub fn process_audio(&self, buffer: AudioBuffer) -> Result<AudioBuffer> {
        let client = self.client.as_ref().ok_or(Error::NotConnected)?;

        self.runtime.block_on(async {
            let response = client.clone().process_audio(buffer.into_proto()).await?;
            Ok(AudioBuffer::from_proto(response.into_inner()))
        })
    }

    pub fn update_effect(&self, effect_id: u32, param_id: u32, value: f32) -> Result<()> {
        let client = self.client.as_ref().ok_or(Error::NotConnected)?;

        self.runtime.block_on(async {
            client.clone().update_effect(EffectUpdate {
                effect_id,
                parameter_id: param_id,
                value,
            }).await?;
            Ok(())
        })
    }
}
```

### AI Client

```rust
// orpheus-ai/src/client.rs

pub struct AiClient {
    client: Option<LeviathanClient>,
    active_persona: Persona,
    chat_history: Vec<ChatMessage>,
    pending_responses: mpsc::Receiver<AiResponse>,
}

impl AiClient {
    pub fn send_message(&mut self, message: &str, context: AiContext) -> Result<()> {
        let request = ChatRequest {
            persona: self.active_persona.clone(),
            message: message.to_string(),
            context: context.into(),
            history: self.chat_history.clone(),
        };

        // Non-blocking send
        self.client.as_mut()
            .ok_or(Error::NotConnected)?
            .send_message_async(request);

        self.chat_history.push(ChatMessage::user(message));
        Ok(())
    }

    pub fn poll_response(&mut self) -> Option<AiResponse> {
        match self.pending_responses.try_recv() {
            Ok(response) => {
                self.chat_history.push(ChatMessage::assistant(&response.content));
                Some(response)
            }
            Err(_) => None,
        }
    }

    pub fn set_persona(&mut self, persona: Persona) {
        self.active_persona = persona;
        self.chat_history.clear();
    }
}

#[derive(Clone)]
pub enum Persona {
    MusicComposer,
    MusicTheoryTutor,
    MixingEngineer,
    MasteringEngineer,
    GuitarCoach,
    SessionAssistant,
    ProductionTutor,
}
```

---

## Dependencies

```toml
# orpheus-app/Cargo.toml

[dependencies]
# UI Framework
eframe = { version = "0.29", features = ["wgpu"] }
egui = "0.29"
egui_dock = "0.14"
egui-wgpu = "0.29"

# Shared components
daemoniorum-egui = { path = "../../nyx/daemoniorum-egui" }

# Async runtime
tokio = { version = "1.42", features = ["full"] }

# gRPC
tonic = "0.12"
prost = "0.13"

# Audio
cpal = "0.15"  # Audio device access
hound = "3.5"  # WAV files
symphonia = "0.5"  # Audio decoding

# MIDI
midir = "0.10"
midly = "0.5"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Utilities
uuid = { version = "1.11", features = ["v4"] }
chrono = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "2.0"
anyhow = "1.0"

# File handling
rfd = "0.15"  # Native file dialogs
notify = "7.0"  # File watching
```

---

## Build Configuration

```toml
# Cargo.toml (workspace root)

[workspace]
resolver = "2"
members = [
    "crates/orpheus-app",
    "crates/orpheus-core",
    "crates/orpheus-ui",
    "crates/orpheus-audio",
    "crates/orpheus-midi",
    "crates/orpheus-file",
    "crates/orpheus-ai",
    "crates/orpheus-plugins",
    "crates/orpheus-export",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "MIT OR Apache-2.0"
repository = "https://github.com/Daemoniorum-LLC/orpheus"

[workspace.dependencies]
# Centralized dependency versions
egui = "0.29"
eframe = "0.29"
tokio = { version = "1.42", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"
tracing = "0.1"

[profile.release]
lto = true
codegen-units = 1
opt-level = 3
strip = true

[profile.release-debug]
inherits = "release"
debug = true
strip = false
```

---

## Platform Integration

### Nyx-Specific Features

```rust
// orpheus-app/src/platform/nyx.rs

#[cfg(target_os = "nyx")]
pub fn integrate_with_nyx() -> Result<()> {
    // Register with Grimoire for persona management
    grimoire_client::register_app("orpheus", AppManifest {
        name: "Orpheus",
        version: env!("CARGO_PKG_VERSION"),
        personas: vec!["music-composer", "mixing-engineer", "guitar-coach"],
        capabilities: vec!["audio", "midi", "file-system"],
    })?;

    // Request capabilities from Guardian
    guardian_client::request_capabilities(&[
        Capability::AudioDevice,
        Capability::MidiDevice,
        Capability::FileSystem(FileAccess::UserMusic),
    ])?;

    // Register with Vesper for audio routing
    vesper_client::register_audio_app("orpheus", AudioProfile {
        sample_rate: 48000,
        buffer_size: 512,
        channels: 2,
    })?;

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn integrate_with_linux() -> Result<()> {
    // Standard Linux audio via PipeWire/ALSA
    // XDG desktop integration
    Ok(())
}
```

### Desktop Entry

```desktop
# orpheus.desktop
[Desktop Entry]
Name=Orpheus
Comment=Professional Music Production
Exec=orpheus %F
Icon=orpheus
Type=Application
Categories=AudioVideo;Audio;Sequencer;Midi;
MimeType=application/x-maestro;audio/midi;
Keywords=music;audio;daw;guitar;tab;
```

---

## Performance Targets

| Metric | Target | Implementation |
|--------|--------|----------------|
| UI Frame Rate | 60 FPS | egui request_repaint, wgpu backend |
| Audio Latency | < 10ms | gRPC streaming, JUCE engine |
| Startup Time | < 2s | Lazy service connections |
| Memory (Idle) | < 200MB | Lazy loading, asset streaming |
| Memory (Project) | < 1GB | Efficient buffer management |
| File Save | < 500ms | Async I/O, compression |

---

## Implementation Phases

### Phase 1: Core Application (4-6 weeks)
- [ ] Workspace setup with all crates
- [ ] Basic eframe application with theming
- [ ] Transport controls and playback state
- [ ] Audio client connection to Maestro
- [ ] File menu (new, open, save)

### Phase 2: Compose Mode (4-6 weeks)
- [ ] Tab editor with note input
- [ ] Score view rendering
- [ ] Fretboard visualization
- [ ] Chord palette
- [ ] Guitar Pro import

### Phase 3: Mix Mode (3-4 weeks)
- [ ] Mixer console with channel strips
- [ ] EQ and compressor UI
- [ ] Level meters
- [ ] Routing matrix

### Phase 4: Record Mode (3-4 weeks)
- [ ] Waveform display
- [ ] Timeline view
- [ ] Recording functionality
- [ ] MIDI input handling

### Phase 5: Master & Practice (2-3 weeks)
- [ ] Mastering chain UI
- [ ] LUFS metering
- [ ] Speed trainer
- [ ] Loop section controls

### Phase 6: AI Integration (2-3 weeks)
- [ ] AI chat panel
- [ ] Persona switching
- [ ] Context-aware suggestions
- [ ] Leviathan client integration

### Phase 7: Polish & Platform (2-3 weeks)
- [ ] Command palette
- [ ] Keyboard shortcuts
- [ ] Settings persistence
- [ ] Nyx/Linux integration
- [ ] Plugin hosting (VST3)

---

## Summary

This architecture provides:

1. **Modular Crate Structure**: Clean separation of concerns across 9 crates
2. **Native Performance**: Rust + egui/wgpu for 60 FPS rendering
3. **Professional Audio**: Integration with existing Maestro gRPC engine
4. **AI-First Design**: Built-in Leviathan persona integration
5. **Platform Flexibility**: Works on both Nyx OS and standard Linux
6. **Extensibility**: Plugin hosting and command system for future growth

The design leverages existing infrastructure (daemoniorum-egui, maestro-audio) while providing a complete native application experience for professional music production.
