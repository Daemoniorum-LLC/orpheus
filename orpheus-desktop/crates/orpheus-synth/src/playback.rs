//! Playback engine for real-time audio generation

use crate::guitar::{GuitarSynth, GuitarConfig};
use std::collections::VecDeque;

/// Note event for playback
#[derive(Debug, Clone)]
pub struct NoteEvent {
    /// String number (1-indexed)
    pub string: u8,
    /// Fret number
    pub fret: u8,
    /// Velocity (0.0 - 1.0)
    pub velocity: f32,
    /// Sample position when this should trigger
    pub sample_pos: u64,
    /// Event type
    pub event_type: NoteEventType,
}

/// Type of note event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteEventType {
    /// Normal note on
    NoteOn,
    /// Note off (mute)
    NoteOff,
    /// Hammer-on
    HammerOn,
    /// Pull-off
    PullOff,
    /// Slide
    Slide,
}

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// Not playing
    Stopped,
    /// Playing
    Playing,
    /// Paused
    Paused,
}

/// Audio playback engine
pub struct PlaybackEngine {
    /// Guitar synthesizer
    synth: GuitarSynth,
    /// Sample rate
    sample_rate: u32,
    /// Current sample position
    sample_position: u64,
    /// Scheduled note events
    event_queue: VecDeque<NoteEvent>,
    /// Current playback state
    state: PlaybackState,
    /// Tempo in BPM
    tempo: f64,
    /// Master volume (0.0 - 1.0)
    master_volume: f32,
    /// Loop start position (in samples)
    loop_start: Option<u64>,
    /// Loop end position (in samples)
    loop_end: Option<u64>,
    /// Output buffer for double-buffering
    output_buffer: Vec<f32>,
}

impl PlaybackEngine {
    /// Create a new playback engine
    pub fn new(sample_rate: u32) -> Self {
        let config = GuitarConfig {
            sample_rate,
            ..Default::default()
        };

        Self {
            synth: GuitarSynth::new(config),
            sample_rate,
            sample_position: 0,
            event_queue: VecDeque::new(),
            state: PlaybackState::Stopped,
            tempo: 120.0,
            master_volume: 0.8,
            loop_start: None,
            loop_end: None,
            output_buffer: Vec::new(),
        }
    }

    /// Create with custom guitar configuration
    pub fn with_config(sample_rate: u32, guitar_config: GuitarConfig) -> Self {
        let mut config = guitar_config;
        config.sample_rate = sample_rate;

        Self {
            synth: GuitarSynth::new(config),
            sample_rate,
            sample_position: 0,
            event_queue: VecDeque::new(),
            state: PlaybackState::Stopped,
            tempo: 120.0,
            master_volume: 0.8,
            loop_start: None,
            loop_end: None,
            output_buffer: Vec::new(),
        }
    }

    /// Start playback
    pub fn play(&mut self) {
        self.state = PlaybackState::Playing;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    /// Stop playback and reset position
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.sample_position = 0;
        self.synth.mute_all();
    }

    /// Get current playback state
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Get current position in seconds
    pub fn position_secs(&self) -> f64 {
        self.sample_position as f64 / self.sample_rate as f64
    }

    /// Get current position in samples
    pub fn position_samples(&self) -> u64 {
        self.sample_position
    }

    /// Seek to a position in seconds
    pub fn seek(&mut self, position_secs: f64) {
        self.sample_position = (position_secs * self.sample_rate as f64) as u64;
        self.synth.mute_all();
    }

    /// Set tempo
    pub fn set_tempo(&mut self, bpm: f64) {
        self.tempo = bpm.clamp(20.0, 300.0);
    }

    /// Get tempo
    pub fn tempo(&self) -> f64 {
        self.tempo
    }

    /// Set master volume
    pub fn set_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set loop points
    pub fn set_loop(&mut self, start_secs: f64, end_secs: f64) {
        self.loop_start = Some((start_secs * self.sample_rate as f64) as u64);
        self.loop_end = Some((end_secs * self.sample_rate as f64) as u64);
    }

    /// Clear loop points
    pub fn clear_loop(&mut self) {
        self.loop_start = None;
        self.loop_end = None;
    }

    /// Schedule a note event
    pub fn schedule_event(&mut self, event: NoteEvent) {
        // Insert in sorted order by sample position
        let pos = self.event_queue
            .iter()
            .position(|e| e.sample_pos > event.sample_pos)
            .unwrap_or(self.event_queue.len());
        self.event_queue.insert(pos, event);
    }

    /// Schedule multiple events
    pub fn schedule_events(&mut self, events: impl IntoIterator<Item = NoteEvent>) {
        for event in events {
            self.schedule_event(event);
        }
    }

    /// Clear all scheduled events
    pub fn clear_events(&mut self) {
        self.event_queue.clear();
    }

    /// Convert beats to samples at current tempo
    pub fn beats_to_samples(&self, beats: f64) -> u64 {
        let seconds_per_beat = 60.0 / self.tempo;
        (beats * seconds_per_beat * self.sample_rate as f64) as u64
    }

    /// Convert samples to beats at current tempo
    pub fn samples_to_beats(&self, samples: u64) -> f64 {
        let seconds = samples as f64 / self.sample_rate as f64;
        seconds * self.tempo / 60.0
    }

    /// Process audio and fill output buffer (mono)
    pub fn process_mono(&mut self, output: &mut [f32]) {
        if self.state != PlaybackState::Playing {
            // Fill with silence when not playing
            output.fill(0.0);
            return;
        }

        for sample in output.iter_mut() {
            // Process pending events at this sample position
            self.process_events_at_position();

            // Generate audio
            *sample = self.synth.next_sample_mono() * self.master_volume;

            // Advance position
            self.sample_position += 1;

            // Handle looping
            if let (Some(start), Some(end)) = (self.loop_start, self.loop_end) {
                if self.sample_position >= end {
                    self.sample_position = start;
                    // Re-queue events that fall within the loop
                    // (This is a simple implementation - proper looping would need more work)
                }
            }
        }
    }

    /// Process audio and fill output buffer (stereo interleaved)
    pub fn process_stereo(&mut self, output: &mut [f32]) {
        if self.state != PlaybackState::Playing {
            output.fill(0.0);
            return;
        }

        for chunk in output.chunks_exact_mut(2) {
            self.process_events_at_position();

            let (l, r) = self.synth.next_sample_stereo();
            chunk[0] = l * self.master_volume;
            chunk[1] = r * self.master_volume;

            self.sample_position += 1;

            if let (Some(start), Some(end)) = (self.loop_start, self.loop_end) {
                if self.sample_position >= end {
                    self.sample_position = start;
                }
            }
        }
    }

    /// Process events at current sample position
    fn process_events_at_position(&mut self) {
        while let Some(event) = self.event_queue.front() {
            if event.sample_pos > self.sample_position {
                break;
            }

            let event = self.event_queue.pop_front().unwrap();
            self.process_event(&event);
        }
    }

    /// Process a single note event
    fn process_event(&mut self, event: &NoteEvent) {
        match event.event_type {
            NoteEventType::NoteOn => {
                self.synth.pluck(event.string, event.fret, event.velocity);
            }
            NoteEventType::NoteOff => {
                self.synth.mute_string(event.string);
            }
            NoteEventType::HammerOn => {
                self.synth.hammer_on(event.string, event.fret, event.velocity);
            }
            NoteEventType::PullOff => {
                self.synth.pull_off(event.string, event.fret, event.velocity);
            }
            NoteEventType::Slide => {
                self.synth.slide_to(event.string, event.fret, 0);
            }
        }
    }

    /// Render a section to a buffer (offline rendering)
    pub fn render_to_buffer(&mut self, duration_secs: f64, stereo: bool) -> Vec<f32> {
        let num_samples = (duration_secs * self.sample_rate as f64) as usize;
        let buffer_size = if stereo { num_samples * 2 } else { num_samples };
        let mut buffer = vec![0.0; buffer_size];

        // Save and set state
        let original_state = self.state;
        self.state = PlaybackState::Playing;

        if stereo {
            self.process_stereo(&mut buffer);
        } else {
            self.process_mono(&mut buffer);
        }

        // Restore state
        self.state = original_state;

        buffer
    }

    /// Get access to the guitar synth for direct manipulation
    pub fn synth(&self) -> &GuitarSynth {
        &self.synth
    }

    /// Get mutable access to the guitar synth
    pub fn synth_mut(&mut self) -> &mut GuitarSynth {
        &mut self.synth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_engine() {
        let mut engine = PlaybackEngine::new(44100);
        engine.set_tempo(120.0);

        // Schedule some notes
        engine.schedule_event(NoteEvent {
            string: 1,
            fret: 0,
            velocity: 0.8,
            sample_pos: 0,
            event_type: NoteEventType::NoteOn,
        });

        engine.play();

        let mut buffer = vec![0.0; 4410]; // 0.1 seconds
        engine.process_mono(&mut buffer);

        // Should have output
        let max = buffer.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }

    #[test]
    fn test_tempo_conversion() {
        let engine = PlaybackEngine::new(44100);
        // At 120 BPM, 1 beat = 0.5 seconds = 22050 samples
        let samples = engine.beats_to_samples(1.0);
        assert_eq!(samples, 22050);
    }
}
