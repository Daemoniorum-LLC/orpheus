//! Audio engine - bridges UI and synth
//!
//! Runs audio processing on a separate thread and communicates
//! with the UI via channels.
//!
//! When compiled with the `audio` feature, outputs to real audio devices via cpal.
//! Otherwise, operates in mock mode for UI testing.

use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tracing::{debug, info, warn};
use uuid::Uuid;

use orpheus_synth::{
    DrumMachine, DrumPattern, GuitarConfig, GuitarSynth,
    Distortion, NoiseGate, ThreeBandEq,
    PlaybackEngine, NoteEvent,
    PianoSynth, BassSynth, BassPreset,
    Metronome, MetronomeTiming,
};

use orpheus_core::{RecordingState, RecordedMidiEvent, ClipPlaybackEngine, ScheduledMidiEvent, Track};

use orpheus_plugins::{
    HostConfig, PluginHost, PluginInstance, PluginMetadata, PluginScanner, ScanResult,
};

#[cfg(feature = "audio")]
use orpheus_synth::{AudioOutput, AudioMessage};

// Re-export for app usage
pub use orpheus_synth::{NoteEvent as TabNoteEvent, TabSequencer, DrumType};
pub use orpheus_plugins::{PluginFormat, PluginCategory};

/// Commands sent from UI to audio engine
#[derive(Debug, Clone)]
pub enum AudioCommand {
    /// Start playback
    Play,
    /// Pause playback
    Pause,
    /// Stop and reset position
    Stop,
    /// Seek to position (in samples)
    Seek(u64),
    /// Set tempo (BPM)
    SetTempo(f64),
    /// Set master volume (0.0 - 1.0)
    SetVolume(f32),

    // Guitar commands
    /// Pluck guitar string (string 1-8, fret, velocity)
    GuitarPluck(u8, u8, f32),
    /// Palm mute guitar string
    GuitarPalmMute(u8, u8, f32),
    /// Mute all guitar strings
    GuitarMuteAll,
    /// Set guitar config
    SetGuitarConfig(GuitarConfigType),

    // Drum commands
    /// Trigger drum (type, velocity)
    DrumTrigger(DrumType, f32),
    /// Play drum pattern
    PlayDrumPattern(DrumPatternType),
    /// Stop drum pattern
    StopDrumPattern,

    // Piano commands
    /// Play piano note (note, velocity)
    PianoNoteOn(u8, f32),
    /// Release piano note
    PianoNoteOff(u8),
    /// Set piano sustain pedal
    PianoSustain(bool),
    /// Release all piano notes
    PianoAllNotesOff,

    // Bass commands
    /// Play bass note (note, velocity)
    BassNoteOn(u8, f32),
    /// Release bass note
    BassNoteOff(u8),
    /// Set bass preset
    SetBassPreset(BassPresetType),
    /// Release all bass notes
    BassAllNotesOff,

    // Effects
    /// Enable/disable distortion
    SetDistortion(bool),
    /// Set distortion drive
    SetDistortionDrive(f32),
    /// Enable/disable noise gate
    SetNoiseGate(bool),

    // Tab playback
    /// Load tablature events for playback
    LoadTab(Vec<NoteEvent>),
    /// Clear loaded tab
    ClearTab,
    /// Set loop region (start_samples, end_samples)
    SetLoop(u64, u64),
    /// Clear loop region
    ClearLoop,

    /// Render to buffer and return (for export)
    RenderToBuffer { duration_secs: f64, stereo: bool },

    // Plugin commands
    /// Scan for available plugins
    ScanPlugins,
    /// Load a plugin from path
    LoadPlugin(PathBuf),
    /// Unload a plugin by instance ID
    UnloadPlugin(Uuid),
    /// Activate a plugin
    ActivatePlugin(Uuid),
    /// Deactivate a plugin
    DeactivatePlugin(Uuid),
    /// Set plugin bypass state
    SetPluginBypass(Uuid, bool),
    /// Set plugin parameter (instance_id, param_id, normalized_value)
    SetPluginParameter(Uuid, u32, f64),
    /// Enable/disable plugin processing in the audio chain
    SetPluginsEnabled(bool),

    // MIDI input commands
    /// Scan for available MIDI input devices
    ScanMidiDevices,
    /// Connect to a MIDI input device by index
    ConnectMidiDevice(usize),
    /// Disconnect from current MIDI device
    DisconnectMidiDevice,
    /// Set MIDI channel filter (None = all channels)
    SetMidiChannelFilter(Option<u8>),

    // Recording commands
    /// Arm a track for recording
    ArmTrack(Uuid),
    /// Disarm a track
    DisarmTrack(Uuid),
    /// Start recording (playback + capture on armed tracks)
    StartRecording,
    /// Stop recording and get clips
    StopRecording,
    /// Route a virtual keyboard note to recording
    RecordVirtualNote { note: u8, velocity: u8, is_note_on: bool },

    // Metronome commands
    /// Enable/disable metronome
    SetMetronomeEnabled(bool),
    /// Set metronome volume (0.0 - 1.0)
    SetMetronomeVolume(f32),

    // Clip playback commands
    /// Load clips from tracks for playback
    LoadClips(Vec<Track>),
    /// Clear loaded clips
    ClearClips,

    /// Shutdown the engine
    Shutdown,
}

/// Guitar configuration presets
#[derive(Debug, Clone, Copy)]
pub enum GuitarConfigType {
    Standard6,
    SevenStringMetal,
    EightStringMetal,
    TechDeath,
    DropD,
    DropC,
    DropA,
    Bass,
}

/// Drum pattern presets
#[derive(Debug, Clone, Copy)]
pub enum DrumPatternType {
    Rock,
    Disco,
    Metal,
    BlastBeat,
    HammerBlast,
    GravityBlast,
    TechDeath,
    DoubleBass16ths,
    DBeat,
}

/// Bass synth preset types
#[derive(Debug, Clone, Copy)]
pub enum BassPresetType {
    SynthBass,
    SubBass,
    FunkBass,
    Bass808,
    WobbleBass,
    MoogBass,
    ReeseBass,
}

/// Status updates sent from engine to UI
#[derive(Debug, Clone)]
pub enum AudioStatus {
    /// Current playback position (samples)
    Position(u64),
    /// Current playback beat (for tab highlighting)
    CurrentBeat(f64),
    /// Playback state changed
    StateChanged(EngineState),
    /// Audio level (left, right) for metering
    Level(f32, f32),
    /// CPU load percentage
    CpuLoad(f32),
    /// Engine is ready
    Ready,
    /// Engine shutting down
    ShuttingDown,
    /// Rendered buffer ready
    RenderedBuffer(Vec<f32>),
    /// Error occurred
    Error(String),

    // Plugin status
    /// Plugin scan results available
    PluginScanComplete(Vec<PluginMetadata>),
    /// Plugin loaded successfully
    PluginLoaded { id: Uuid, name: String },
    /// Plugin unloaded
    PluginUnloaded(Uuid),
    /// Plugin load failed
    PluginLoadError(String),
    /// List of currently loaded plugins
    LoadedPlugins(Vec<LoadedPluginInfo>),

    // MIDI status
    /// Available MIDI devices
    MidiDevicesAvailable(Vec<MidiDeviceInfo>),
    /// MIDI device connected
    MidiDeviceConnected { index: usize, name: String },
    /// MIDI device disconnected
    MidiDeviceDisconnected,
    /// MIDI device error
    MidiDeviceError(String),
    /// MIDI message received (for activity display)
    MidiMessageReceived { description: String, msg_type: String },

    // Recording status
    /// Recording started
    RecordingStarted,
    /// Recording stopped with recorded clips
    RecordingStopped(Vec<RecordedClipInfo>),
    /// Track armed status changed
    TrackArmed { track_id: Uuid, armed: bool },
    /// Recording event count update (track_id, count)
    RecordingEvents { track_id: Uuid, count: usize },
}

/// Information about a recorded clip for UI
#[derive(Debug, Clone)]
pub struct RecordedClipInfo {
    pub track_id: Uuid,
    pub clip_id: Uuid,
    pub clip_name: String,
    pub start: u64,
    pub length: u64,
    pub notes: Vec<orpheus_core::MidiNote>,
}

/// Information about a MIDI device for UI display
#[derive(Debug, Clone)]
pub struct MidiDeviceInfo {
    pub index: usize,
    pub name: String,
}

/// Information about a loaded plugin for UI display
#[derive(Debug, Clone)]
pub struct LoadedPluginInfo {
    pub id: Uuid,
    pub name: String,
    pub format: PluginFormat,
    pub active: bool,
    pub bypassed: bool,
}

/// Engine playback state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineState {
    Stopped,
    Playing,
    Paused,
}

/// Shared engine state for UI access
pub struct EngineSharedState {
    /// Current position in samples
    pub position: u64,
    /// Current state
    pub state: EngineState,
    /// Left channel level
    pub level_left: f32,
    /// Right channel level
    pub level_right: f32,
    /// CPU load
    pub cpu_load: f32,
    /// Sample rate
    pub sample_rate: u32,
    /// Is connected/running
    pub connected: bool,
}

impl Default for EngineSharedState {
    fn default() -> Self {
        Self {
            position: 0,
            state: EngineState::Stopped,
            level_left: 0.0,
            level_right: 0.0,
            cpu_load: 0.0,
            sample_rate: 44100,
            connected: false,
        }
    }
}

/// Audio engine handle for UI
pub struct AudioEngine {
    /// Command sender
    command_tx: Sender<AudioCommand>,
    /// Status receiver
    status_rx: Receiver<AudioStatus>,
    /// Shared state (lock-free reads from UI)
    shared_state: Arc<Mutex<EngineSharedState>>,
    /// Thread handle
    thread_handle: Option<JoinHandle<()>>,
}

impl AudioEngine {
    /// Create and start a new audio engine
    pub fn new(sample_rate: u32) -> Self {
        let (command_tx, command_rx) = bounded::<AudioCommand>(64);
        let (status_tx, status_rx) = bounded::<AudioStatus>(64);
        let shared_state = Arc::new(Mutex::new(EngineSharedState {
            sample_rate,
            ..Default::default()
        }));

        let state_clone = Arc::clone(&shared_state);

        // Spawn audio thread
        let thread_handle = thread::Builder::new()
            .name("audio-engine".to_string())
            .spawn(move || {
                let mut runner = EngineRunner::new(sample_rate, command_rx, status_tx, state_clone);
                runner.run();
            })
            .expect("Failed to spawn audio thread");

        info!("Audio engine started at {} Hz", sample_rate);

        Self {
            command_tx,
            status_rx,
            shared_state,
            thread_handle: Some(thread_handle),
        }
    }

    /// Send a command to the engine
    pub fn send(&self, cmd: AudioCommand) -> Result<(), String> {
        self.command_tx
            .send(cmd)
            .map_err(|e| format!("Failed to send command: {}", e))
    }

    /// Try to receive a status update (non-blocking)
    pub fn try_recv_status(&self) -> Option<AudioStatus> {
        self.status_rx.try_recv().ok()
    }

    /// Get current shared state
    pub fn state(&self) -> EngineSharedState {
        self.shared_state.lock().clone()
    }

    /// Check if engine is connected/running
    pub fn is_connected(&self) -> bool {
        self.shared_state.lock().connected
    }

    /// Get current position
    pub fn position(&self) -> u64 {
        self.shared_state.lock().position
    }

    /// Get current levels (left, right)
    pub fn levels(&self) -> (f32, f32) {
        let state = self.shared_state.lock();
        (state.level_left, state.level_right)
    }

    /// Get playback state
    pub fn playback_state(&self) -> EngineState {
        self.shared_state.lock().state
    }

    // Convenience methods

    pub fn play(&self) {
        let _ = self.send(AudioCommand::Play);
    }

    pub fn pause(&self) {
        let _ = self.send(AudioCommand::Pause);
    }

    pub fn stop(&self) {
        let _ = self.send(AudioCommand::Stop);
    }

    pub fn seek(&self, position: u64) {
        let _ = self.send(AudioCommand::Seek(position));
    }

    pub fn set_tempo(&self, bpm: f64) {
        let _ = self.send(AudioCommand::SetTempo(bpm));
    }

    pub fn set_volume(&self, volume: f32) {
        let _ = self.send(AudioCommand::SetVolume(volume));
    }

    pub fn guitar_pluck(&self, string: u8, fret: u8, velocity: f32) {
        let _ = self.send(AudioCommand::GuitarPluck(string, fret, velocity));
    }

    pub fn guitar_palm_mute(&self, string: u8, fret: u8, velocity: f32) {
        let _ = self.send(AudioCommand::GuitarPalmMute(string, fret, velocity));
    }

    pub fn drum_trigger(&self, drum: DrumType, velocity: f32) {
        let _ = self.send(AudioCommand::DrumTrigger(drum, velocity));
    }

    pub fn play_drum_pattern(&self, pattern: DrumPatternType) {
        let _ = self.send(AudioCommand::PlayDrumPattern(pattern));
    }

    pub fn stop_drum_pattern(&self) {
        let _ = self.send(AudioCommand::StopDrumPattern);
    }

    // Piano methods
    pub fn piano_note_on(&self, note: u8, velocity: f32) {
        let _ = self.send(AudioCommand::PianoNoteOn(note, velocity));
    }

    pub fn piano_note_off(&self, note: u8) {
        let _ = self.send(AudioCommand::PianoNoteOff(note));
    }

    pub fn piano_sustain(&self, down: bool) {
        let _ = self.send(AudioCommand::PianoSustain(down));
    }

    pub fn piano_all_notes_off(&self) {
        let _ = self.send(AudioCommand::PianoAllNotesOff);
    }

    // Bass methods
    pub fn bass_note_on(&self, note: u8, velocity: f32) {
        let _ = self.send(AudioCommand::BassNoteOn(note, velocity));
    }

    pub fn bass_note_off(&self, note: u8) {
        let _ = self.send(AudioCommand::BassNoteOff(note));
    }

    pub fn set_bass_preset(&self, preset: BassPresetType) {
        let _ = self.send(AudioCommand::SetBassPreset(preset));
    }

    pub fn bass_all_notes_off(&self) {
        let _ = self.send(AudioCommand::BassAllNotesOff);
    }

    pub fn load_tab(&self, events: Vec<NoteEvent>) {
        let _ = self.send(AudioCommand::LoadTab(events));
    }

    pub fn clear_tab(&self) {
        let _ = self.send(AudioCommand::ClearTab);
    }

    /// Set loop region (in samples)
    pub fn set_loop(&self, start_samples: u64, end_samples: u64) {
        let _ = self.send(AudioCommand::SetLoop(start_samples, end_samples));
    }

    /// Clear loop region
    pub fn clear_loop(&self) {
        let _ = self.send(AudioCommand::ClearLoop);
    }

    pub fn shutdown(&self) {
        let _ = self.send(AudioCommand::Shutdown);
    }

    // Plugin methods
    pub fn scan_plugins(&self) {
        let _ = self.send(AudioCommand::ScanPlugins);
    }

    pub fn load_plugin(&self, path: PathBuf) {
        let _ = self.send(AudioCommand::LoadPlugin(path));
    }

    pub fn unload_plugin(&self, id: Uuid) {
        let _ = self.send(AudioCommand::UnloadPlugin(id));
    }

    pub fn activate_plugin(&self, id: Uuid) {
        let _ = self.send(AudioCommand::ActivatePlugin(id));
    }

    pub fn deactivate_plugin(&self, id: Uuid) {
        let _ = self.send(AudioCommand::DeactivatePlugin(id));
    }

    pub fn set_plugin_bypass(&self, id: Uuid, bypassed: bool) {
        let _ = self.send(AudioCommand::SetPluginBypass(id, bypassed));
    }

    pub fn set_plugin_parameter(&self, id: Uuid, param_id: u32, value: f64) {
        let _ = self.send(AudioCommand::SetPluginParameter(id, param_id, value));
    }

    pub fn set_plugins_enabled(&self, enabled: bool) {
        let _ = self.send(AudioCommand::SetPluginsEnabled(enabled));
    }

    // MIDI methods
    pub fn scan_midi_devices(&self) {
        let _ = self.send(AudioCommand::ScanMidiDevices);
    }

    pub fn connect_midi_device(&self, index: usize) {
        let _ = self.send(AudioCommand::ConnectMidiDevice(index));
    }

    pub fn disconnect_midi_device(&self) {
        let _ = self.send(AudioCommand::DisconnectMidiDevice);
    }

    pub fn set_midi_channel_filter(&self, channel: Option<u8>) {
        let _ = self.send(AudioCommand::SetMidiChannelFilter(channel));
    }

    // Recording methods
    pub fn arm_track(&self, track_id: Uuid) {
        let _ = self.send(AudioCommand::ArmTrack(track_id));
    }

    pub fn disarm_track(&self, track_id: Uuid) {
        let _ = self.send(AudioCommand::DisarmTrack(track_id));
    }

    pub fn start_recording(&self) {
        let _ = self.send(AudioCommand::StartRecording);
    }

    pub fn stop_recording(&self) {
        let _ = self.send(AudioCommand::StopRecording);
    }

    pub fn record_virtual_note(&self, note: u8, velocity: u8, is_note_on: bool) {
        let _ = self.send(AudioCommand::RecordVirtualNote { note, velocity, is_note_on });
    }

    // Metronome methods
    pub fn set_metronome_enabled(&self, enabled: bool) {
        let _ = self.send(AudioCommand::SetMetronomeEnabled(enabled));
    }

    pub fn set_metronome_volume(&self, volume: f32) {
        let _ = self.send(AudioCommand::SetMetronomeVolume(volume));
    }

    // Clip playback methods
    pub fn load_clips(&self, tracks: Vec<Track>) {
        let _ = self.send(AudioCommand::LoadClips(tracks));
    }

    pub fn clear_clips(&self) {
        let _ = self.send(AudioCommand::ClearClips);
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        info!("Shutting down audio engine");
        let _ = self.command_tx.send(AudioCommand::Shutdown);

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Clone for EngineSharedState {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            state: self.state,
            level_left: self.level_left,
            level_right: self.level_right,
            cpu_load: self.cpu_load,
            sample_rate: self.sample_rate,
            connected: self.connected,
        }
    }
}

/// Internal engine runner (runs on audio thread)
struct EngineRunner {
    sample_rate: u32,
    command_rx: Receiver<AudioCommand>,
    status_tx: Sender<AudioStatus>,
    shared_state: Arc<Mutex<EngineSharedState>>,

    // Synthesis
    guitar: GuitarSynth,
    drums: DrumMachine,
    piano: PianoSynth,
    bass: BassSynth,

    // Tab playback engine (when a tab is loaded)
    tab_engine: Option<PlaybackEngine>,
    tab_loaded: bool,

    // Effects
    distortion: Distortion,
    noise_gate: NoiseGate,
    eq: ThreeBandEq,
    distortion_enabled: bool,
    gate_enabled: bool,

    // State
    state: EngineState,
    position: u64,
    tempo: f64,
    master_volume: f32,

    // Pattern playback
    current_pattern: Option<DrumPattern>,
    pattern_step: usize,
    samples_per_step: usize,
    step_sample_counter: usize,

    // Metering
    peak_left: f32,
    peak_right: f32,

    // Plugin hosting
    plugin_host: PluginHost,
    plugin_chain: Vec<Arc<PluginInstance>>,
    plugins_enabled: bool,
    plugin_buffer_left: Vec<f32>,
    plugin_buffer_right: Vec<f32>,

    // Real audio output (when audio feature is enabled)
    #[cfg(feature = "audio")]
    audio_output: Option<AudioOutput>,

    // MIDI input
    midi_input: Option<orpheus_midi::MidiInputPort>,
    midi_channel_filter: Option<u8>,

    // Recording
    recording_state: RecordingState,
    armed_tracks: Vec<Uuid>,

    // Metronome
    metronome: Metronome,
    metronome_timing: MetronomeTiming,

    // Clip playback
    clip_playback: ClipPlaybackEngine,
}

impl EngineRunner {
    fn new(
        sample_rate: u32,
        command_rx: Receiver<AudioCommand>,
        status_tx: Sender<AudioStatus>,
        shared_state: Arc<Mutex<EngineSharedState>>,
    ) -> Self {
        // Try to initialize real audio output when feature is enabled
        #[cfg(feature = "audio")]
        let audio_output = match AudioOutput::new() {
            Ok(output) => {
                let actual_rate = output.sample_rate();
                info!("Audio output initialized: {} Hz, {} channels",
                      actual_rate, output.channels());
                Some(output)
            }
            Err(e) => {
                warn!("Failed to initialize audio output: {}. Running in mock mode.", e);
                None
            }
        };

        #[cfg(feature = "audio")]
        let actual_sample_rate = audio_output.as_ref()
            .map(|o| o.sample_rate())
            .unwrap_or(sample_rate);

        #[cfg(not(feature = "audio"))]
        let actual_sample_rate = sample_rate;

        Self {
            sample_rate: actual_sample_rate,
            command_rx,
            status_tx,
            shared_state,

            guitar: GuitarSynth::new(GuitarConfig::tech_death()),
            drums: DrumMachine::metal_machine(actual_sample_rate),
            piano: PianoSynth::new(actual_sample_rate, 16), // 16-voice polyphony
            bass: BassSynth::standard(actual_sample_rate),

            tab_engine: None,
            tab_loaded: false,

            distortion: Distortion::tech_death(),
            noise_gate: NoiseGate::tight(),
            eq: ThreeBandEq::modern_metal(),
            distortion_enabled: true,
            gate_enabled: true,

            state: EngineState::Stopped,
            position: 0,
            tempo: 120.0,
            master_volume: 0.8,

            current_pattern: None,
            pattern_step: 0,
            samples_per_step: 0,
            step_sample_counter: 0,

            peak_left: 0.0,
            peak_right: 0.0,

            // Initialize plugin host with matching sample rate and buffer size
            plugin_host: PluginHost::new(
                HostConfig::default()
                    .with_sample_rate(actual_sample_rate)
                    .with_buffer_size(128) // Match FRAME_SIZE
            ),
            plugin_chain: Vec::new(),
            plugins_enabled: true,
            plugin_buffer_left: vec![0.0; 128],
            plugin_buffer_right: vec![0.0; 128],

            #[cfg(feature = "audio")]
            audio_output,

            midi_input: None,
            midi_channel_filter: None,

            recording_state: RecordingState::new(),
            armed_tracks: Vec::new(),

            metronome: Metronome::new(actual_sample_rate),
            metronome_timing: MetronomeTiming::new(actual_sample_rate, 120.0, 4),

            clip_playback: ClipPlaybackEngine::new(actual_sample_rate, 120.0),
        }
    }

    fn run(&mut self) {
        info!("Audio engine runner started");

        // Mark as connected
        {
            let mut state = self.shared_state.lock();
            state.connected = true;
        }
        let _ = self.status_tx.send(AudioStatus::Ready);

        // Main loop
        loop {
            // Process commands
            match self.command_rx.try_recv() {
                Ok(cmd) => {
                    if !self.handle_command(cmd) {
                        break;
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    warn!("Command channel disconnected");
                    break;
                }
            }

            // Poll MIDI input
            self.poll_midi_input();

            // Generate audio if playing
            if self.state == EngineState::Playing {
                self.process_audio_frame();
            }

            // Small sleep to avoid spinning
            std::thread::sleep(std::time::Duration::from_micros(100));
        }

        // Mark as disconnected
        {
            let mut state = self.shared_state.lock();
            state.connected = false;
        }
        let _ = self.status_tx.send(AudioStatus::ShuttingDown);

        info!("Audio engine runner stopped");
    }

    fn handle_command(&mut self, cmd: AudioCommand) -> bool {
        match cmd {
            AudioCommand::Play => {
                self.state = EngineState::Playing;
                self.metronome_timing.reset();
                self.clip_playback.play();
                if let Some(ref mut engine) = self.tab_engine {
                    engine.play();
                }
                self.update_shared_state();
                debug!("Engine: Play");
            }
            AudioCommand::Pause => {
                self.state = EngineState::Paused;
                self.clip_playback.pause();
                if let Some(ref mut engine) = self.tab_engine {
                    engine.pause();
                }
                self.update_shared_state();
                debug!("Engine: Pause");
            }
            AudioCommand::Stop => {
                self.state = EngineState::Stopped;
                self.position = 0;
                self.guitar.mute_all();
                self.piano.all_notes_off();
                self.bass.all_notes_off();
                self.clip_playback.stop();
                if let Some(ref mut engine) = self.tab_engine {
                    engine.stop();
                }
                self.pattern_step = 0;
                self.step_sample_counter = 0;
                self.update_shared_state();
                debug!("Engine: Stop");
            }
            AudioCommand::Seek(pos) => {
                self.position = pos;
                self.clip_playback.seek(pos);
                self.update_shared_state();
                debug!("Engine: Seek to {}", pos);
            }
            AudioCommand::SetTempo(bpm) => {
                self.tempo = bpm;
                self.recalculate_step_timing();
                self.metronome_timing.set_tempo(bpm);
                self.clip_playback.set_tempo(bpm);
                debug!("Engine: Tempo = {} BPM", bpm);
            }
            AudioCommand::SetVolume(vol) => {
                self.master_volume = vol.clamp(0.0, 1.0);
            }
            AudioCommand::GuitarPluck(string, fret, vel) => {
                self.guitar.pluck(string, fret, vel);
            }
            AudioCommand::GuitarPalmMute(string, fret, vel) => {
                self.guitar.palm_mute(string, fret, vel);
            }
            AudioCommand::GuitarMuteAll => {
                self.guitar.mute_all();
            }
            AudioCommand::SetGuitarConfig(config_type) => {
                let config = match config_type {
                    GuitarConfigType::Standard6 => GuitarConfig::default(),
                    GuitarConfigType::SevenStringMetal => GuitarConfig::seven_string_metal(),
                    GuitarConfigType::EightStringMetal => GuitarConfig::eight_string_metal(),
                    GuitarConfigType::TechDeath => GuitarConfig::tech_death(),
                    GuitarConfigType::DropD => {
                        GuitarConfig::with_drop_tuning(&orpheus_synth::DROP_D_TUNING)
                    }
                    GuitarConfigType::DropC => {
                        GuitarConfig::with_drop_tuning(&orpheus_synth::DROP_C_TUNING)
                    }
                    GuitarConfigType::DropA => {
                        GuitarConfig::with_drop_tuning(&orpheus_synth::DROP_A_TUNING)
                    }
                    GuitarConfigType::Bass => GuitarConfig::bass(),
                };
                self.guitar = GuitarSynth::new(config);
                debug!("Engine: Guitar config changed");
            }
            AudioCommand::DrumTrigger(drum, vel) => {
                self.drums.trigger(drum, vel);
            }
            AudioCommand::PlayDrumPattern(pattern_type) => {
                let pattern = match pattern_type {
                    DrumPatternType::Rock => DrumPattern::rock_beat(),
                    DrumPatternType::Disco => DrumPattern::disco_beat(),
                    DrumPatternType::Metal => DrumPattern::metal_beat(),
                    DrumPatternType::BlastBeat => DrumPattern::blast_beat(),
                    DrumPatternType::HammerBlast => DrumPattern::hammer_blast(),
                    DrumPatternType::GravityBlast => DrumPattern::gravity_blast(),
                    DrumPatternType::TechDeath => DrumPattern::tech_death_beat(),
                    DrumPatternType::DoubleBass16ths => DrumPattern::double_bass_16ths(),
                    DrumPatternType::DBeat => DrumPattern::d_beat(),
                };
                self.current_pattern = Some(pattern);
                self.pattern_step = 0;
                self.step_sample_counter = 0;
                self.recalculate_step_timing();
                debug!("Engine: Playing drum pattern {:?}", pattern_type);
            }
            AudioCommand::StopDrumPattern => {
                self.current_pattern = None;
                debug!("Engine: Stopped drum pattern");
            }
            // Piano commands
            AudioCommand::PianoNoteOn(note, velocity) => {
                self.piano.note_on(note, velocity);
            }
            AudioCommand::PianoNoteOff(note) => {
                self.piano.note_off(note);
            }
            AudioCommand::PianoSustain(down) => {
                self.piano.set_sustain(down);
            }
            AudioCommand::PianoAllNotesOff => {
                self.piano.all_notes_off();
            }
            // Bass commands
            AudioCommand::BassNoteOn(note, velocity) => {
                self.bass.note_on(note, velocity);
            }
            AudioCommand::BassNoteOff(note) => {
                self.bass.note_off(note);
            }
            AudioCommand::SetBassPreset(preset_type) => {
                let preset = match preset_type {
                    BassPresetType::SynthBass => BassPreset::SynthBass,
                    BassPresetType::SubBass => BassPreset::SubBass,
                    BassPresetType::FunkBass => BassPreset::FunkBass,
                    BassPresetType::Bass808 => BassPreset::Bass808,
                    BassPresetType::WobbleBass => BassPreset::WobbleBass,
                    BassPresetType::MoogBass => BassPreset::MoogBass,
                    BassPresetType::ReeseBass => BassPreset::ReeseBass,
                };
                self.bass.set_preset(preset);
                debug!("Engine: Bass preset set to {:?}", preset_type);
            }
            AudioCommand::BassAllNotesOff => {
                self.bass.all_notes_off();
            }
            AudioCommand::SetDistortion(enabled) => {
                self.distortion_enabled = enabled;
            }
            AudioCommand::SetDistortionDrive(drive) => {
                self.distortion.set_drive(drive);
            }
            AudioCommand::SetNoiseGate(enabled) => {
                self.gate_enabled = enabled;
            }
            AudioCommand::LoadTab(events) => {
                // Create a new playback engine and load the events
                let mut engine = PlaybackEngine::new(self.sample_rate);
                engine.set_tempo(self.tempo);
                engine.schedule_events(events);
                self.tab_engine = Some(engine);
                self.tab_loaded = true;
                info!("Engine: Tab loaded");
            }
            AudioCommand::ClearTab => {
                self.tab_engine = None;
                self.tab_loaded = false;
                debug!("Engine: Tab cleared");
            }
            AudioCommand::SetLoop(start, end) => {
                if let Some(ref mut engine) = self.tab_engine {
                    let start_secs = start as f64 / self.sample_rate as f64;
                    let end_secs = end as f64 / self.sample_rate as f64;
                    engine.set_loop(start_secs, end_secs);
                    debug!("Engine: Loop set {:.2}s - {:.2}s", start_secs, end_secs);
                }
            }
            AudioCommand::ClearLoop => {
                if let Some(ref mut engine) = self.tab_engine {
                    engine.clear_loop();
                    debug!("Engine: Loop cleared");
                }
            }
            AudioCommand::RenderToBuffer {
                duration_secs,
                stereo,
            } => {
                let buffer = self.render_offline(duration_secs, stereo);
                let _ = self.status_tx.send(AudioStatus::RenderedBuffer(buffer));
            }

            // Plugin commands
            AudioCommand::ScanPlugins => {
                info!("Engine: Scanning for plugins");
                let scanner = PluginScanner::new();
                match scanner.scan_all() {
                    Ok(result) => {
                        info!("Found {} plugins", result.plugins.len());
                        let _ = self.status_tx.send(AudioStatus::PluginScanComplete(result.plugins));
                    }
                    Err(e) => {
                        warn!("Plugin scan error: {}", e);
                        let _ = self.status_tx.send(AudioStatus::PluginLoadError(format!("Scan failed: {}", e)));
                    }
                }
            }
            AudioCommand::LoadPlugin(path) => {
                info!("Engine: Loading plugin from {:?}", path);
                match self.plugin_host.load(&path) {
                    Ok(instance) => {
                        let id = instance.id();
                        let name = instance.name().to_string();

                        // Activate and add to chain
                        if let Err(e) = instance.activate() {
                            warn!("Failed to activate plugin: {}", e);
                        }

                        self.plugin_chain.push(instance);
                        info!("Loaded and activated plugin: {}", name);
                        let _ = self.status_tx.send(AudioStatus::PluginLoaded { id, name });
                        self.send_loaded_plugins_status();
                    }
                    Err(e) => {
                        warn!("Failed to load plugin: {}", e);
                        let _ = self.status_tx.send(AudioStatus::PluginLoadError(e.to_string()));
                    }
                }
            }
            AudioCommand::UnloadPlugin(id) => {
                info!("Engine: Unloading plugin {}", id);
                self.plugin_chain.retain(|p| p.id() != id);
                if let Err(e) = self.plugin_host.unload(id) {
                    warn!("Failed to unload plugin: {}", e);
                }
                let _ = self.status_tx.send(AudioStatus::PluginUnloaded(id));
                self.send_loaded_plugins_status();
            }
            AudioCommand::ActivatePlugin(id) => {
                if let Some(plugin) = self.plugin_chain.iter().find(|p| p.id() == id) {
                    if let Err(e) = plugin.activate() {
                        warn!("Failed to activate plugin: {}", e);
                    }
                }
                self.send_loaded_plugins_status();
            }
            AudioCommand::DeactivatePlugin(id) => {
                if let Some(plugin) = self.plugin_chain.iter().find(|p| p.id() == id) {
                    if let Err(e) = plugin.deactivate() {
                        warn!("Failed to deactivate plugin: {}", e);
                    }
                }
                self.send_loaded_plugins_status();
            }
            AudioCommand::SetPluginBypass(id, bypassed) => {
                if let Some(plugin) = self.plugin_chain.iter().find(|p| p.id() == id) {
                    plugin.set_bypassed(bypassed);
                    debug!("Plugin {} bypass: {}", id, bypassed);
                }
                self.send_loaded_plugins_status();
            }
            AudioCommand::SetPluginParameter(id, param_id, value) => {
                if let Some(plugin) = self.plugin_chain.iter().find(|p| p.id() == id) {
                    if let Err(e) = plugin.set_parameter(param_id, value) {
                        warn!("Failed to set parameter: {}", e);
                    }
                }
            }
            AudioCommand::SetPluginsEnabled(enabled) => {
                self.plugins_enabled = enabled;
                info!("Plugins enabled: {}", enabled);
            }

            // MIDI commands
            AudioCommand::ScanMidiDevices => {
                info!("Engine: Scanning for MIDI devices");
                match orpheus_midi::list_input_devices() {
                    Ok(devices) => {
                        let device_infos: Vec<MidiDeviceInfo> = devices
                            .into_iter()
                            .map(|d| MidiDeviceInfo {
                                index: d.index,
                                name: d.name,
                            })
                            .collect();
                        info!("Found {} MIDI devices", device_infos.len());
                        let _ = self.status_tx.send(AudioStatus::MidiDevicesAvailable(device_infos));
                    }
                    Err(e) => {
                        warn!("MIDI scan error: {}", e);
                        let _ = self.status_tx.send(AudioStatus::MidiDeviceError(e.to_string()));
                    }
                }
            }
            AudioCommand::ConnectMidiDevice(index) => {
                info!("Engine: Connecting to MIDI device {}", index);
                // Disconnect existing if any
                self.midi_input = None;

                match orpheus_midi::open_input(index) {
                    Ok(port) => {
                        let name = port.name().to_string();
                        info!("Connected to MIDI device: {}", name);
                        self.midi_input = Some(port);
                        let _ = self.status_tx.send(AudioStatus::MidiDeviceConnected { index, name });
                    }
                    Err(e) => {
                        warn!("MIDI connection error: {}", e);
                        let _ = self.status_tx.send(AudioStatus::MidiDeviceError(e.to_string()));
                    }
                }
            }
            AudioCommand::DisconnectMidiDevice => {
                info!("Engine: Disconnecting MIDI device");
                self.midi_input = None;
                let _ = self.status_tx.send(AudioStatus::MidiDeviceDisconnected);
            }
            AudioCommand::SetMidiChannelFilter(channel) => {
                self.midi_channel_filter = channel;
                debug!("MIDI channel filter set to {:?}", channel);
            }

            // Recording commands
            AudioCommand::ArmTrack(track_id) => {
                if !self.armed_tracks.contains(&track_id) {
                    self.armed_tracks.push(track_id);
                    info!("Engine: Armed track {}", track_id);
                    let _ = self.status_tx.send(AudioStatus::TrackArmed { track_id, armed: true });
                }
            }
            AudioCommand::DisarmTrack(track_id) => {
                self.armed_tracks.retain(|id| *id != track_id);
                info!("Engine: Disarmed track {}", track_id);
                let _ = self.status_tx.send(AudioStatus::TrackArmed { track_id, armed: false });
            }
            AudioCommand::StartRecording => {
                if !self.armed_tracks.is_empty() {
                    info!("Engine: Starting recording on {} armed tracks", self.armed_tracks.len());
                    self.recording_state.start(
                        self.armed_tracks.clone(),
                        self.position,
                        self.sample_rate,
                        self.tempo,
                    );
                    // Also start playback
                    self.state = EngineState::Playing;
                    if let Some(ref mut engine) = self.tab_engine {
                        engine.play();
                    }
                    self.update_shared_state();
                    let _ = self.status_tx.send(AudioStatus::RecordingStarted);
                } else {
                    warn!("Engine: Cannot start recording - no armed tracks");
                }
            }
            AudioCommand::StopRecording => {
                info!("Engine: Stopping recording");
                // Stop playback
                self.state = EngineState::Stopped;
                self.guitar.mute_all();
                self.piano.all_notes_off();
                self.bass.all_notes_off();
                if let Some(ref mut engine) = self.tab_engine {
                    engine.stop();
                }
                self.pattern_step = 0;
                self.step_sample_counter = 0;
                self.update_shared_state();

                // Finalize recording and get clips
                let clips = self.recording_state.stop();
                let clip_infos: Vec<RecordedClipInfo> = clips
                    .into_iter()
                    .map(|(track_id, clip)| {
                        let notes = match clip.content {
                            orpheus_core::ClipContent::Midi { notes } => notes,
                            _ => vec![],
                        };
                        RecordedClipInfo {
                            track_id,
                            clip_id: clip.id,
                            clip_name: clip.name,
                            start: clip.start,
                            length: clip.length,
                            notes,
                        }
                    })
                    .collect();
                info!("Engine: Recording produced {} clips", clip_infos.len());
                let _ = self.status_tx.send(AudioStatus::RecordingStopped(clip_infos));
            }
            AudioCommand::RecordVirtualNote { note, velocity, is_note_on } => {
                // Route virtual keyboard notes to recording on first armed track
                if self.recording_state.is_recording {
                    if let Some(&track_id) = self.armed_tracks.first() {
                        if is_note_on {
                            self.recording_state.record_note_on(track_id, note, velocity, self.position);
                        } else {
                            self.recording_state.record_note_off(track_id, note, self.position);
                        }
                    }
                }
            }

            // Metronome commands
            AudioCommand::SetMetronomeEnabled(enabled) => {
                self.metronome_timing.set_enabled(enabled);
                info!("Engine: Metronome enabled = {}", enabled);
            }
            AudioCommand::SetMetronomeVolume(volume) => {
                self.metronome.set_volume(volume);
                debug!("Engine: Metronome volume = {}", volume);
            }

            // Clip playback commands
            AudioCommand::LoadClips(tracks) => {
                let track_refs: Vec<&Track> = tracks.iter().collect();
                self.clip_playback.load_clips(&track_refs);
                info!("Engine: Loaded {} clips with {} events",
                    tracks.iter().map(|t| t.clips.len()).sum::<usize>(),
                    self.clip_playback.event_count());
            }
            AudioCommand::ClearClips => {
                self.clip_playback.stop();
                // Re-create to clear all state
                self.clip_playback = ClipPlaybackEngine::new(self.sample_rate, self.tempo);
                debug!("Engine: Cleared clips");
            }

            AudioCommand::Shutdown => {
                info!("Engine: Shutdown requested");
                return false;
            }
        }
        true
    }

    /// Poll MIDI input and route to synths
    fn poll_midi_input(&mut self) {
        if let Some(ref port) = self.midi_input {
            while let Some(timestamped) = port.try_recv() {
                let msg = &timestamped.message;

                // Apply channel filter if set
                if let Some(filter_ch) = self.midi_channel_filter {
                    if let Some(msg_ch) = msg.channel() {
                        if msg_ch + 1 != filter_ch {
                            continue; // Skip messages not on the filtered channel
                        }
                    }
                }

                // Route MIDI to synths
                match msg {
                    orpheus_midi::MidiMessage::NoteOn { note, velocity, .. } => {
                        let vel = *velocity as f32 / 127.0;
                        // Route to piano by default
                        self.piano.note_on(*note, vel);

                        // Record if recording is active
                        if self.recording_state.is_recording {
                            if let Some(&track_id) = self.armed_tracks.first() {
                                self.recording_state.record_note_on(track_id, *note, *velocity, self.position);
                            }
                        }

                        // Send to UI for activity display
                        let note_name = Self::note_to_name(*note);
                        let description = format!("Note On: {} vel: {}", note_name, velocity);
                        let _ = self.status_tx.send(AudioStatus::MidiMessageReceived {
                            description,
                            msg_type: "NoteOn".to_string(),
                        });
                    }
                    orpheus_midi::MidiMessage::NoteOff { note, .. } => {
                        self.piano.note_off(*note);

                        // Record if recording is active
                        if self.recording_state.is_recording {
                            if let Some(&track_id) = self.armed_tracks.first() {
                                self.recording_state.record_note_off(track_id, *note, self.position);
                            }
                        }

                        let note_name = Self::note_to_name(*note);
                        let description = format!("Note Off: {}", note_name);
                        let _ = self.status_tx.send(AudioStatus::MidiMessageReceived {
                            description,
                            msg_type: "NoteOff".to_string(),
                        });
                    }
                    orpheus_midi::MidiMessage::ControlChange { control, value, .. } => {
                        // Handle sustain pedal
                        if *control == 64 {
                            self.piano.set_sustain(*value >= 64);
                        }

                        let description = format!("CC {}: {}", control, value);
                        let _ = self.status_tx.send(AudioStatus::MidiMessageReceived {
                            description,
                            msg_type: "CC".to_string(),
                        });
                    }
                    orpheus_midi::MidiMessage::PitchBend { value, .. } => {
                        let description = format!("Pitch Bend: {}", value);
                        let _ = self.status_tx.send(AudioStatus::MidiMessageReceived {
                            description,
                            msg_type: "PitchBend".to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// Convert MIDI note to name
    fn note_to_name(note: u8) -> String {
        let names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let octave = (note / 12) as i8 - 1;
        let name = names[(note % 12) as usize];
        format!("{}{}", name, octave)
    }

    /// Send current loaded plugins status to UI
    fn send_loaded_plugins_status(&self) {
        let plugins: Vec<LoadedPluginInfo> = self.plugin_chain.iter().map(|p| {
            LoadedPluginInfo {
                id: p.id(),
                name: p.name().to_string(),
                format: p.format(),
                active: p.is_active(),
                bypassed: p.is_bypassed(),
            }
        }).collect();
        let _ = self.status_tx.send(AudioStatus::LoadedPlugins(plugins));
    }

    fn recalculate_step_timing(&mut self) {
        // Calculate samples per step based on tempo
        // Assuming 16th notes (4 steps per beat)
        let beats_per_second = self.tempo / 60.0;
        let steps_per_second = beats_per_second * 4.0; // 16th notes
        self.samples_per_step = (self.sample_rate as f64 / steps_per_second) as usize;
    }

    fn process_audio_frame(&mut self) {
        // Process a small batch of samples
        const FRAME_SIZE: usize = 128;
        let mut left_buffer = [0.0f32; FRAME_SIZE];
        let mut right_buffer = [0.0f32; FRAME_SIZE];

        // Process tab playback if loaded
        if let Some(ref mut tab_engine) = self.tab_engine {
            // Create interleaved stereo buffer
            let mut stereo_buffer = [0.0f32; FRAME_SIZE * 2];
            tab_engine.process_stereo(&mut stereo_buffer);

            // De-interleave to left/right
            for i in 0..FRAME_SIZE {
                left_buffer[i] = stereo_buffer[i * 2];
                right_buffer[i] = stereo_buffer[i * 2 + 1];
            }
        } else {
            // Generate guitar samples directly (for manual triggering)
            for i in 0..FRAME_SIZE {
                let (l, r) = self.guitar.next_sample_stereo();
                left_buffer[i] += l;
                right_buffer[i] += r;
            }
        }

        // Process drum pattern if active
        if let Some(ref pattern) = self.current_pattern {
            for i in 0..FRAME_SIZE {
                // Check if we need to advance to next step
                if self.step_sample_counter >= self.samples_per_step {
                    self.step_sample_counter = 0;
                    self.pattern_step = (self.pattern_step + 1) % pattern.steps;

                    // Trigger drums for this step
                    for drum_step in pattern.get_step(self.pattern_step) {
                        self.drums.trigger(drum_step.drum, drum_step.velocity);
                    }
                }

                self.step_sample_counter += 1;

                // Generate drum samples
                let drum_sample = self.drums.next_sample();
                left_buffer[i] += drum_sample;
                right_buffer[i] += drum_sample;
            }
        } else {
            // Still generate drum samples for manual triggering
            for i in 0..FRAME_SIZE {
                let drum_sample = self.drums.next_sample();
                left_buffer[i] += drum_sample;
                right_buffer[i] += drum_sample;
            }
        }

        // Generate piano samples (clean, no distortion)
        for i in 0..FRAME_SIZE {
            let (piano_l, piano_r) = self.piano.next_sample_stereo();
            left_buffer[i] += piano_l;
            right_buffer[i] += piano_r;
        }

        // Generate bass samples (clean, goes through own filter)
        for i in 0..FRAME_SIZE {
            let bass_sample = self.bass.next_sample();
            // Bass is mono, center panned
            left_buffer[i] += bass_sample * 0.8;
            right_buffer[i] += bass_sample * 0.8;
        }

        // Process clip playback events
        let clip_events = self.clip_playback.process_frame(FRAME_SIZE as u64);
        for event in clip_events {
            // Route clip notes to piano synth (all clips use piano for now)
            if event.is_note_on {
                let velocity = event.velocity as f32 / 127.0;
                self.piano.note_on(event.note, velocity);
            } else {
                self.piano.note_off(event.note);
            }
        }

        // Generate metronome clicks
        if self.metronome_timing.is_enabled() {
            // Check for beat triggers at the start of each frame
            if let Some(is_downbeat) = self.metronome_timing.advance(FRAME_SIZE as u64) {
                self.metronome.trigger(is_downbeat);
            }

            // Mix metronome samples
            for i in 0..FRAME_SIZE {
                let click = self.metronome.next_sample();
                left_buffer[i] += click;
                right_buffer[i] += click;
            }
        }

        // Apply effects (process mono mix for effects)
        for i in 0..FRAME_SIZE {
            let mono = (left_buffer[i] + right_buffer[i]) * 0.5;

            let mut processed = mono;

            if self.gate_enabled {
                processed = self.noise_gate.process(processed);
            }

            if self.distortion_enabled {
                processed = self.distortion.process(processed);
            }

            processed = self.eq.process(processed);

            // Split back to stereo (simple for now)
            left_buffer[i] = processed;
            right_buffer[i] = processed;
        }

        // Process through plugin chain if enabled
        if self.plugins_enabled && !self.plugin_chain.is_empty() {
            // Copy buffers for plugin processing
            self.plugin_buffer_left.copy_from_slice(&left_buffer);
            self.plugin_buffer_right.copy_from_slice(&right_buffer);

            // Process through each plugin in the chain
            for plugin in &self.plugin_chain {
                if plugin.is_active() && !plugin.is_bypassed() {
                    // Create input/output slices for plugin
                    let input_left = self.plugin_buffer_left.clone();
                    let input_right = self.plugin_buffer_right.clone();

                    // Process through plugin (stereo)
                    if let Err(e) = plugin.process(
                        &[&input_left, &input_right],
                        &mut [&mut self.plugin_buffer_left, &mut self.plugin_buffer_right],
                    ) {
                        // Only log occasionally to avoid spam
                        if self.position % 44100 == 0 {
                            debug!("Plugin processing error: {}", e);
                        }
                    }
                }
            }

            // Copy processed audio back to main buffers
            left_buffer.copy_from_slice(&self.plugin_buffer_left);
            right_buffer.copy_from_slice(&self.plugin_buffer_right);
        }

        // Apply master volume and calculate peaks
        let mut frame_peak_l = 0.0f32;
        let mut frame_peak_r = 0.0f32;

        for i in 0..FRAME_SIZE {
            left_buffer[i] *= self.master_volume;
            right_buffer[i] *= self.master_volume;

            frame_peak_l = frame_peak_l.max(left_buffer[i].abs());
            frame_peak_r = frame_peak_r.max(right_buffer[i].abs());
        }

        // Update peak meters (decay)
        self.peak_left = self.peak_left * 0.95 + frame_peak_l * 0.05;
        self.peak_right = self.peak_right * 0.95 + frame_peak_r * 0.05;

        // Send samples to real audio output when feature is enabled
        #[cfg(feature = "audio")]
        if let Some(ref audio_output) = self.audio_output {
            // Interleave stereo samples for output
            let mut interleaved = Vec::with_capacity(FRAME_SIZE * 2);
            for i in 0..FRAME_SIZE {
                interleaved.push(left_buffer[i]);
                interleaved.push(right_buffer[i]);
            }

            if let Err(e) = audio_output.send_samples(interleaved) {
                // Only warn occasionally to avoid log spam
                if self.position % 44100 == 0 {
                    warn!("Failed to send audio samples: {}", e);
                }
            }
        }

        // Advance position
        self.position += FRAME_SIZE as u64;

        // Update recording position if recording
        if self.recording_state.is_recording {
            self.recording_state.update_position(self.position);
        }

        // Report current beat position for tab highlighting
        if self.tab_loaded {
            if let Some(ref tab_engine) = self.tab_engine {
                let current_beat = tab_engine.samples_to_beats(tab_engine.position_samples());
                // Send beat position periodically for UI highlighting
                if self.position % 2048 == 0 {
                    let _ = self.status_tx.send(AudioStatus::CurrentBeat(current_beat));
                }
            }
        }

        // Update shared state periodically
        if self.position % 4096 == 0 {
            self.update_shared_state();
        }
    }

    fn render_offline(&mut self, duration_secs: f64, stereo: bool) -> Vec<f32> {
        let num_samples = (self.sample_rate as f64 * duration_secs) as usize;
        let buffer_size = if stereo { num_samples * 2 } else { num_samples };
        let mut buffer = vec![0.0f32; buffer_size];

        for i in 0..num_samples {
            let (l, r) = self.guitar.next_sample_stereo();
            let drum = self.drums.next_sample();

            let mut left = l + drum;
            let mut right = r + drum;

            // Apply effects
            let mono = (left + right) * 0.5;
            let mut processed = mono;

            if self.gate_enabled {
                processed = self.noise_gate.process(processed);
            }
            if self.distortion_enabled {
                processed = self.distortion.process(processed);
            }
            processed = self.eq.process(processed);

            left = processed * self.master_volume;
            right = processed * self.master_volume;

            if stereo {
                buffer[i * 2] = left;
                buffer[i * 2 + 1] = right;
            } else {
                buffer[i] = (left + right) * 0.5;
            }
        }

        buffer
    }

    fn update_shared_state(&mut self) {
        let mut state = self.shared_state.lock();
        state.position = self.position;
        state.state = self.state;
        state.level_left = self.peak_left;
        state.level_right = self.peak_right;
    }
}
