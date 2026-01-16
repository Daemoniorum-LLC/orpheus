//! Unified audio engine connecting synthesis to real-time output
//!
//! The `AudioEngine` provides a high-level interface for audio playback,
//! combining the `PlaybackEngine` (synthesis and scheduling) with
//! `AudioOutput` (real-time cpal output).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;
use tracing::{debug, error, info, warn};

use crate::playback::{PlaybackEngine, PlaybackState, NoteEvent};
use crate::guitar::GuitarConfig;
use crate::Error;

#[cfg(feature = "audio-output")]
use crate::output::AudioOutput;

/// Audio engine configuration
#[derive(Debug, Clone)]
pub struct AudioEngineConfig {
    /// Sample rate (default: 48000)
    pub sample_rate: u32,
    /// Buffer size in samples (default: 512)
    pub buffer_size: usize,
    /// Number of channels (default: 2 for stereo)
    pub channels: u16,
    /// Master volume (0.0 - 1.0, default: 0.8)
    pub master_volume: f32,
    /// Initial tempo in BPM (default: 120.0)
    pub tempo: f64,
}

impl Default for AudioEngineConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 512,
            channels: 2,
            master_volume: 0.8,
            tempo: 120.0,
        }
    }
}

/// Messages sent to the audio thread
#[derive(Debug)]
enum AudioCommand {
    Play,
    Pause,
    Stop,
    Seek(f64),
    SetTempo(f64),
    SetVolume(f32),
    ScheduleEvent(NoteEvent),
    ScheduleEvents(Vec<NoteEvent>),
    ClearEvents,
    SetLoop(f64, f64),
    ClearLoop,
    Shutdown,
}

/// Unified audio engine for real-time playback
pub struct AudioEngine {
    /// Command sender to audio thread
    command_tx: Sender<AudioCommand>,
    /// Audio processing thread handle
    #[allow(dead_code)]
    audio_thread: Option<JoinHandle<()>>,
    /// Current playback state (shared with audio thread)
    is_playing: Arc<AtomicBool>,
    /// Current position in samples (shared with audio thread)
    position_samples: Arc<AtomicU64>,
    /// Sample rate
    sample_rate: u32,
    /// Whether the engine is running
    running: Arc<AtomicBool>,
}

impl AudioEngine {
    /// Create a new audio engine with default configuration
    #[cfg(feature = "audio-output")]
    pub fn new() -> Result<Self, Error> {
        Self::with_config(AudioEngineConfig::default())
    }

    /// Create with custom configuration
    #[cfg(feature = "audio-output")]
    pub fn with_config(config: AudioEngineConfig) -> Result<Self, Error> {
        let (command_tx, command_rx) = bounded::<AudioCommand>(256);
        let is_playing = Arc::new(AtomicBool::new(false));
        let position_samples = Arc::new(AtomicU64::new(0));
        let running = Arc::new(AtomicBool::new(true));

        let is_playing_clone = Arc::clone(&is_playing);
        let position_clone = Arc::clone(&position_samples);
        let running_clone = Arc::clone(&running);
        let sample_rate = config.sample_rate;

        // Spawn audio processing thread
        let audio_thread = thread::Builder::new()
            .name("orpheus-audio".to_string())
            .spawn(move || {
                if let Err(e) = run_audio_thread(
                    config,
                    command_rx,
                    is_playing_clone,
                    position_clone,
                    running_clone,
                ) {
                    error!("Audio thread error: {}", e);
                }
            })
            .map_err(|e| Error::AudioOutput(format!("Failed to spawn audio thread: {}", e)))?;

        info!("Audio engine started");

        Ok(Self {
            command_tx,
            audio_thread: Some(audio_thread),
            is_playing,
            position_samples,
            sample_rate,
            running,
        })
    }

    /// Create a mock engine for testing (no audio output)
    ///
    /// The mock engine accepts all commands but doesn't process them.
    /// Useful for testing the API interface without any audio processing.
    pub fn mock() -> Self {
        let (command_tx, command_rx) = bounded::<AudioCommand>(256);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        // Spawn a thread to drain commands (prevents channel from blocking)
        let audio_thread = thread::Builder::new()
            .name("orpheus-mock".to_string())
            .spawn(move || {
                while running_clone.load(Ordering::Relaxed) {
                    match command_rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(AudioCommand::Shutdown) => break,
                        Ok(_) => {} // Discard other commands
                        Err(_) => {} // Timeout, continue
                    }
                }
            })
            .ok();

        Self {
            command_tx,
            audio_thread,
            is_playing: Arc::new(AtomicBool::new(false)),
            position_samples: Arc::new(AtomicU64::new(0)),
            sample_rate: 48000,
            running,
        }
    }

    /// Create a simulation engine for CI testing
    ///
    /// This runs the full audio processing pipeline (synthesis, scheduling,
    /// playback control) but discards the audio output instead of sending
    /// it to real hardware. Useful for testing in environments without audio.
    #[cfg(feature = "simulation")]
    pub fn new_simulation() -> Result<Self, Error> {
        Self::with_config_simulation(AudioEngineConfig::default())
    }

    /// Create simulation engine with custom configuration
    #[cfg(feature = "simulation")]
    pub fn with_config_simulation(config: AudioEngineConfig) -> Result<Self, Error> {
        let (command_tx, command_rx) = bounded::<AudioCommand>(256);
        let is_playing = Arc::new(AtomicBool::new(false));
        let position_samples = Arc::new(AtomicU64::new(0));
        let running = Arc::new(AtomicBool::new(true));

        let is_playing_clone = Arc::clone(&is_playing);
        let position_clone = Arc::clone(&position_samples);
        let running_clone = Arc::clone(&running);
        let sample_rate = config.sample_rate;

        // Spawn simulation thread (no real audio output)
        let audio_thread = thread::Builder::new()
            .name("orpheus-audio-sim".to_string())
            .spawn(move || {
                if let Err(e) = run_audio_thread_simulation(
                    config,
                    command_rx,
                    is_playing_clone,
                    position_clone,
                    running_clone,
                ) {
                    error!("Simulation thread error: {}", e);
                }
            })
            .map_err(|e| Error::AudioOutput(format!("Failed to spawn simulation thread: {}", e)))?;

        info!("Audio engine started (simulation mode)");

        Ok(Self {
            command_tx,
            audio_thread: Some(audio_thread),
            is_playing,
            position_samples,
            sample_rate,
            running,
        })
    }

    /// Start playback
    pub fn play(&self) -> Result<(), Error> {
        self.send_command(AudioCommand::Play)
    }

    /// Pause playback
    pub fn pause(&self) -> Result<(), Error> {
        self.send_command(AudioCommand::Pause)
    }

    /// Stop playback and reset position
    pub fn stop(&self) -> Result<(), Error> {
        self.send_command(AudioCommand::Stop)
    }

    /// Seek to position in seconds
    pub fn seek(&self, position_secs: f64) -> Result<(), Error> {
        self.send_command(AudioCommand::Seek(position_secs))
    }

    /// Set tempo in BPM
    pub fn set_tempo(&self, bpm: f64) -> Result<(), Error> {
        self.send_command(AudioCommand::SetTempo(bpm))
    }

    /// Set master volume (0.0 - 1.0)
    pub fn set_volume(&self, volume: f32) -> Result<(), Error> {
        self.send_command(AudioCommand::SetVolume(volume))
    }

    /// Schedule a note event
    pub fn schedule_event(&self, event: NoteEvent) -> Result<(), Error> {
        self.send_command(AudioCommand::ScheduleEvent(event))
    }

    /// Schedule multiple note events
    pub fn schedule_events(&self, events: Vec<NoteEvent>) -> Result<(), Error> {
        self.send_command(AudioCommand::ScheduleEvents(events))
    }

    /// Clear all scheduled events
    pub fn clear_events(&self) -> Result<(), Error> {
        self.send_command(AudioCommand::ClearEvents)
    }

    /// Set loop points in seconds
    pub fn set_loop(&self, start: f64, end: f64) -> Result<(), Error> {
        self.send_command(AudioCommand::SetLoop(start, end))
    }

    /// Clear loop points
    pub fn clear_loop(&self) -> Result<(), Error> {
        self.send_command(AudioCommand::ClearLoop)
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Relaxed)
    }

    /// Get current position in seconds
    pub fn position_secs(&self) -> f64 {
        let samples = self.position_samples.load(Ordering::Relaxed);
        samples as f64 / self.sample_rate as f64
    }

    /// Get current position in samples
    pub fn position_samples(&self) -> u64 {
        self.position_samples.load(Ordering::Relaxed)
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Check if engine is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn send_command(&self, cmd: AudioCommand) -> Result<(), Error> {
        self.command_tx
            .send(cmd)
            .map_err(|_| Error::AudioOutput("Audio thread not responding".into()))
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        // Signal shutdown
        let _ = self.command_tx.send(AudioCommand::Shutdown);
        self.running.store(false, Ordering::SeqCst);

        // Wait for audio thread to finish
        if let Some(handle) = self.audio_thread.take() {
            let _ = handle.join();
        }

        info!("Audio engine stopped");
    }
}

/// Run the audio processing thread
#[cfg(feature = "audio-output")]
fn run_audio_thread(
    config: AudioEngineConfig,
    command_rx: Receiver<AudioCommand>,
    is_playing: Arc<AtomicBool>,
    position_samples: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
) -> Result<(), Error> {
    // Create audio output
    let output = AudioOutput::with_buffer_size(config.buffer_size * 4)?;
    let actual_sample_rate = output.sample_rate();

    info!(
        "Audio output initialized: {} Hz, {} channels",
        actual_sample_rate,
        output.channels()
    );

    // Create playback engine
    let guitar_config = GuitarConfig {
        sample_rate: actual_sample_rate,
        ..Default::default()
    };
    let mut playback = PlaybackEngine::with_config(actual_sample_rate, guitar_config);
    playback.set_tempo(config.tempo);
    playback.set_volume(config.master_volume);

    // Audio generation buffer
    let buffer_size = config.buffer_size * 2; // Stereo
    let mut audio_buffer = vec![0.0f32; buffer_size];

    // Main processing loop
    while running.load(Ordering::Relaxed) {
        // Process pending commands (non-blocking)
        while let Ok(cmd) = command_rx.try_recv() {
            match cmd {
                AudioCommand::Play => {
                    playback.play();
                    is_playing.store(true, Ordering::Relaxed);
                    debug!("Playback started");
                }
                AudioCommand::Pause => {
                    playback.pause();
                    is_playing.store(false, Ordering::Relaxed);
                    debug!("Playback paused");
                }
                AudioCommand::Stop => {
                    playback.stop();
                    is_playing.store(false, Ordering::Relaxed);
                    position_samples.store(0, Ordering::Relaxed);
                    debug!("Playback stopped");
                }
                AudioCommand::Seek(pos) => {
                    playback.seek(pos);
                    let samples = (pos * actual_sample_rate as f64) as u64;
                    position_samples.store(samples, Ordering::Relaxed);
                }
                AudioCommand::SetTempo(bpm) => {
                    playback.set_tempo(bpm);
                }
                AudioCommand::SetVolume(vol) => {
                    playback.set_volume(vol);
                    let _ = output.set_volume(vol);
                }
                AudioCommand::ScheduleEvent(event) => {
                    playback.schedule_event(event);
                }
                AudioCommand::ScheduleEvents(events) => {
                    playback.schedule_events(events);
                }
                AudioCommand::ClearEvents => {
                    playback.clear_events();
                }
                AudioCommand::SetLoop(start, end) => {
                    playback.set_loop(start, end);
                }
                AudioCommand::ClearLoop => {
                    playback.clear_loop();
                }
                AudioCommand::Shutdown => {
                    info!("Audio thread shutting down");
                    return Ok(());
                }
            }
        }

        // Generate audio if playing
        if playback.state() == PlaybackState::Playing {
            playback.process_stereo(&mut audio_buffer);

            // Update shared position
            position_samples.store(playback.position_samples(), Ordering::Relaxed);

            // Send to output
            if let Err(e) = output.send_samples(audio_buffer.clone()) {
                warn!("Failed to send audio samples: {}", e);
            }
        } else {
            // Sleep briefly when not playing to avoid spinning
            thread::sleep(Duration::from_millis(10));
        }
    }

    Ok(())
}

/// Run the audio thread in simulation mode (no real audio output)
///
/// This function runs the full audio processing pipeline but discards
/// the output instead of sending it to hardware. Used for CI testing.
#[cfg(feature = "simulation")]
fn run_audio_thread_simulation(
    config: AudioEngineConfig,
    command_rx: Receiver<AudioCommand>,
    is_playing: Arc<AtomicBool>,
    position_samples: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
) -> Result<(), Error> {
    let sample_rate = config.sample_rate;

    info!(
        "Audio simulation initialized: {} Hz (no output)",
        sample_rate
    );

    // Create playback engine
    let guitar_config = GuitarConfig {
        sample_rate,
        ..Default::default()
    };
    let mut playback = PlaybackEngine::with_config(sample_rate, guitar_config);
    playback.set_tempo(config.tempo);
    playback.set_volume(config.master_volume);

    // Audio generation buffer (discarded after processing)
    let buffer_size = config.buffer_size * 2; // Stereo
    let mut audio_buffer = vec![0.0f32; buffer_size];

    // Main processing loop
    while running.load(Ordering::Relaxed) {
        // Process pending commands (non-blocking)
        while let Ok(cmd) = command_rx.try_recv() {
            match cmd {
                AudioCommand::Play => {
                    playback.play();
                    is_playing.store(true, Ordering::Relaxed);
                    debug!("Playback started (simulation)");
                }
                AudioCommand::Pause => {
                    playback.pause();
                    is_playing.store(false, Ordering::Relaxed);
                    debug!("Playback paused (simulation)");
                }
                AudioCommand::Stop => {
                    playback.stop();
                    is_playing.store(false, Ordering::Relaxed);
                    position_samples.store(0, Ordering::Relaxed);
                    debug!("Playback stopped (simulation)");
                }
                AudioCommand::Seek(pos) => {
                    playback.seek(pos);
                    let samples = (pos * sample_rate as f64) as u64;
                    position_samples.store(samples, Ordering::Relaxed);
                }
                AudioCommand::SetTempo(bpm) => {
                    playback.set_tempo(bpm);
                }
                AudioCommand::SetVolume(vol) => {
                    playback.set_volume(vol);
                }
                AudioCommand::ScheduleEvent(event) => {
                    playback.schedule_event(event);
                }
                AudioCommand::ScheduleEvents(events) => {
                    playback.schedule_events(events);
                }
                AudioCommand::ClearEvents => {
                    playback.clear_events();
                }
                AudioCommand::SetLoop(start, end) => {
                    playback.set_loop(start, end);
                }
                AudioCommand::ClearLoop => {
                    playback.clear_loop();
                }
                AudioCommand::Shutdown => {
                    info!("Simulation thread shutting down");
                    return Ok(());
                }
            }
        }

        // Generate audio if playing (output discarded in simulation)
        if playback.state() == PlaybackState::Playing {
            playback.process_stereo(&mut audio_buffer);

            // Update shared position
            position_samples.store(playback.position_samples(), Ordering::Relaxed);

            // In simulation mode, we process audio at roughly real-time rate
            // to properly test timing-dependent behavior
            let buffer_duration_ms = (config.buffer_size as f64 / sample_rate as f64 * 1000.0) as u64;
            thread::sleep(Duration::from_millis(buffer_duration_ms.max(1)));
        } else {
            // Sleep briefly when not playing to avoid spinning
            thread::sleep(Duration::from_millis(10));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playback::NoteEventType;

    #[test]
    fn test_audio_engine_config_default() {
        let config = AudioEngineConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.buffer_size, 512);
        assert_eq!(config.channels, 2);
        assert!((config.master_volume - 0.8).abs() < 0.001);
        assert!((config.tempo - 120.0).abs() < 0.001);
    }

    #[test]
    fn test_mock_engine_creation() {
        let engine = AudioEngine::mock();
        assert!(!engine.is_playing());
        assert_eq!(engine.position_secs(), 0.0);
        assert_eq!(engine.sample_rate(), 48000);
    }

    #[test]
    fn test_mock_engine_commands() {
        let engine = AudioEngine::mock();

        // Commands should not error on mock engine
        assert!(engine.play().is_ok());
        assert!(engine.pause().is_ok());
        assert!(engine.stop().is_ok());
        assert!(engine.seek(5.0).is_ok());
        assert!(engine.set_tempo(140.0).is_ok());
        assert!(engine.set_volume(0.5).is_ok());
    }

    #[test]
    fn test_schedule_event() {
        let engine = AudioEngine::mock();

        let event = NoteEvent {
            string: 1,
            fret: 5,
            velocity: 0.8,
            sample_pos: 0,
            event_type: NoteEventType::NoteOn,
        };

        assert!(engine.schedule_event(event).is_ok());
    }

    #[test]
    fn test_schedule_events_batch() {
        let engine = AudioEngine::mock();

        let events = vec![
            NoteEvent {
                string: 1,
                fret: 0,
                velocity: 0.8,
                sample_pos: 0,
                event_type: NoteEventType::NoteOn,
            },
            NoteEvent {
                string: 2,
                fret: 2,
                velocity: 0.7,
                sample_pos: 22050, // 0.5 seconds at 44100
                event_type: NoteEventType::NoteOn,
            },
            NoteEvent {
                string: 3,
                fret: 2,
                velocity: 0.6,
                sample_pos: 44100, // 1 second at 44100
                event_type: NoteEventType::NoteOn,
            },
        ];

        assert!(engine.schedule_events(events).is_ok());
    }

    #[test]
    fn test_loop_points() {
        let engine = AudioEngine::mock();

        assert!(engine.set_loop(1.0, 5.0).is_ok());
        assert!(engine.clear_loop().is_ok());
    }

    #[test]
    fn test_clear_events() {
        let engine = AudioEngine::mock();

        let event = NoteEvent {
            string: 1,
            fret: 5,
            velocity: 0.8,
            sample_pos: 0,
            event_type: NoteEventType::NoteOn,
        };

        engine.schedule_event(event).unwrap();
        assert!(engine.clear_events().is_ok());
    }

    // Integration test with real audio output (requires audio device)
    #[test]
    #[ignore] // Run with `cargo test -- --ignored` on machine with audio
    #[cfg(feature = "audio-output")]
    fn test_real_audio_engine() {
        let engine = AudioEngine::new().expect("Failed to create audio engine");

        // Schedule a simple note
        let event = NoteEvent {
            string: 1,
            fret: 0, // Open high E
            velocity: 0.8,
            sample_pos: 0,
            event_type: NoteEventType::NoteOn,
        };

        engine.schedule_event(event).unwrap();
        engine.play().unwrap();

        assert!(engine.is_playing());

        // Let it play briefly
        std::thread::sleep(Duration::from_millis(500));

        assert!(engine.position_secs() > 0.0);

        engine.stop().unwrap();
        assert!(!engine.is_playing());
    }

    #[test]
    #[ignore]
    #[cfg(feature = "audio-output")]
    fn test_audio_engine_seek() {
        let engine = AudioEngine::new().expect("Failed to create audio engine");

        engine.seek(2.5).unwrap();
        std::thread::sleep(Duration::from_millis(50)); // Allow command to process

        // Position should be approximately 2.5 seconds
        let pos = engine.position_secs();
        assert!((pos - 2.5).abs() < 0.1);
    }

    #[test]
    #[ignore]
    #[cfg(feature = "audio-output")]
    fn test_audio_engine_tempo_change() {
        let engine = AudioEngine::new().expect("Failed to create audio engine");

        // Schedule notes at beat positions
        let sample_rate = engine.sample_rate();

        // At 120 BPM, 1 beat = 0.5 seconds = 24000 samples at 48kHz
        let events = vec![
            NoteEvent {
                string: 1,
                fret: 0,
                velocity: 0.8,
                sample_pos: 0,
                event_type: NoteEventType::NoteOn,
            },
            NoteEvent {
                string: 1,
                fret: 2,
                velocity: 0.8,
                sample_pos: 24000,
                event_type: NoteEventType::NoteOn,
            },
        ];

        engine.schedule_events(events).unwrap();

        // Change to double tempo
        engine.set_tempo(240.0).unwrap();

        engine.play().unwrap();
        std::thread::sleep(Duration::from_secs(1));
        engine.stop().unwrap();
    }

    // Simulation tests - run full audio pipeline without real hardware
    // These tests can run in CI environments without audio devices

    #[test]
    #[cfg(feature = "simulation")]
    fn test_simulation_engine_creation() {
        let engine = AudioEngine::new_simulation().expect("Failed to create simulation engine");
        assert!(!engine.is_playing());
        assert_eq!(engine.position_secs(), 0.0);
        assert_eq!(engine.sample_rate(), 48000);
        assert!(engine.is_running());
    }

    #[test]
    #[cfg(feature = "simulation")]
    fn test_simulation_play_stop() {
        let engine = AudioEngine::new_simulation().expect("Failed to create simulation engine");

        // Schedule a note
        let event = NoteEvent {
            string: 1,
            fret: 0,
            velocity: 0.8,
            sample_pos: 0,
            event_type: NoteEventType::NoteOn,
        };

        engine.schedule_event(event).unwrap();
        engine.play().unwrap();

        // Give time for the command to be processed
        std::thread::sleep(Duration::from_millis(50));

        assert!(engine.is_playing());

        // Let it run briefly
        std::thread::sleep(Duration::from_millis(200));

        // Position should have advanced
        assert!(engine.position_secs() > 0.0);

        engine.stop().unwrap();
        std::thread::sleep(Duration::from_millis(50));
        assert!(!engine.is_playing());
    }

    #[test]
    #[cfg(feature = "simulation")]
    fn test_simulation_seek() {
        let engine = AudioEngine::new_simulation().expect("Failed to create simulation engine");

        engine.seek(2.5).unwrap();
        std::thread::sleep(Duration::from_millis(50)); // Allow command to process

        // Position should be approximately 2.5 seconds
        let pos = engine.position_secs();
        assert!((pos - 2.5).abs() < 0.1, "Position was {} but expected ~2.5", pos);
    }

    #[test]
    #[cfg(feature = "simulation")]
    fn test_simulation_tempo_change() {
        let engine = AudioEngine::new_simulation().expect("Failed to create simulation engine");

        // Schedule notes
        let events = vec![
            NoteEvent {
                string: 1,
                fret: 0,
                velocity: 0.8,
                sample_pos: 0,
                event_type: NoteEventType::NoteOn,
            },
            NoteEvent {
                string: 1,
                fret: 2,
                velocity: 0.8,
                sample_pos: 24000, // 0.5 seconds at 48kHz
                event_type: NoteEventType::NoteOn,
            },
        ];

        engine.schedule_events(events).unwrap();

        // Change to double tempo
        engine.set_tempo(240.0).unwrap();

        engine.play().unwrap();
        std::thread::sleep(Duration::from_millis(100));

        // Verify it's playing
        assert!(engine.is_playing());

        engine.stop().unwrap();
        std::thread::sleep(Duration::from_millis(50));
        assert!(!engine.is_playing());
    }

    #[test]
    #[cfg(feature = "simulation")]
    fn test_simulation_pause_resume() {
        let engine = AudioEngine::new_simulation().expect("Failed to create simulation engine");

        let event = NoteEvent {
            string: 1,
            fret: 5,
            velocity: 0.8,
            sample_pos: 0,
            event_type: NoteEventType::NoteOn,
        };

        engine.schedule_event(event).unwrap();
        engine.play().unwrap();
        std::thread::sleep(Duration::from_millis(100));

        assert!(engine.is_playing());
        let pos_before_pause = engine.position_secs();

        engine.pause().unwrap();
        std::thread::sleep(Duration::from_millis(50));

        assert!(!engine.is_playing());

        // Resume playback
        engine.play().unwrap();
        std::thread::sleep(Duration::from_millis(100));

        assert!(engine.is_playing());
        assert!(engine.position_secs() > pos_before_pause);

        engine.stop().unwrap();
    }

    #[test]
    #[cfg(feature = "simulation")]
    fn test_simulation_loop_points() {
        let engine = AudioEngine::new_simulation().expect("Failed to create simulation engine");

        // Set loop from 1.0 to 3.0 seconds
        engine.set_loop(1.0, 3.0).unwrap();
        std::thread::sleep(Duration::from_millis(50));

        // Clear loop
        engine.clear_loop().unwrap();
        std::thread::sleep(Duration::from_millis(50));

        // Should still be operational
        assert!(engine.is_running());
    }
}
