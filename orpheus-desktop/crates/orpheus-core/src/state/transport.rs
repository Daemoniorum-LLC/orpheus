//! Transport state - playback, recording, timeline position

use serde::{Deserialize, Serialize};

/// Time position in the project
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TimePosition {
    /// Position in samples
    pub samples: u64,
    /// Sample rate for conversion
    pub sample_rate: u32,
}

impl TimePosition {
    pub fn new(samples: u64, sample_rate: u32) -> Self {
        Self { samples, sample_rate }
    }

    /// Create from seconds
    pub fn from_seconds(seconds: f64, sample_rate: u32) -> Self {
        Self {
            samples: (seconds * sample_rate as f64) as u64,
            sample_rate,
        }
    }

    /// Get position in seconds
    pub fn as_seconds(&self) -> f64 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.samples as f64 / self.sample_rate as f64
    }

    /// Get position in beats given tempo
    pub fn as_beats(&self, tempo: f64) -> f64 {
        let seconds = self.as_seconds();
        seconds * tempo / 60.0
    }

    /// Format as MM:SS.mmm
    pub fn format_time(&self) -> String {
        let total_seconds = self.as_seconds();
        let minutes = (total_seconds / 60.0) as u32;
        let seconds = total_seconds % 60.0;
        format!("{:02}:{:05.2}", minutes, seconds)
    }

    /// Format as Bars:Beats (given tempo and time signature)
    pub fn format_bars(&self, tempo: f64, beats_per_bar: u32) -> String {
        let total_beats = self.as_beats(tempo);
        let bars = (total_beats / beats_per_bar as f64) as u32 + 1;
        let beats = (total_beats % beats_per_bar as f64) as u32 + 1;
        format!("{}:{}", bars, beats)
    }
}

/// Time signature
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self {
            numerator: 4,
            denominator: 4,
        }
    }
}

impl TimeSignature {
    pub fn new(numerator: u8, denominator: u8) -> Self {
        Self { numerator, denominator }
    }
}

/// Transport state - playback and recording control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportState {
    /// Currently playing
    pub is_playing: bool,
    /// Currently recording
    pub is_recording: bool,
    /// Current playhead position
    pub position: TimePosition,
    /// Tempo in BPM
    pub tempo: f64,
    /// Time signature
    pub time_signature: TimeSignature,
    /// Loop enabled
    pub loop_enabled: bool,
    /// Loop start position
    pub loop_start: TimePosition,
    /// Loop end position
    pub loop_end: TimePosition,
    /// Metronome enabled
    pub metronome_enabled: bool,
    /// Count-in bars before recording
    pub count_in_bars: u8,
    /// Sample rate
    pub sample_rate: u32,
}

impl Default for TransportState {
    fn default() -> Self {
        let sample_rate = 48000;
        Self {
            is_playing: false,
            is_recording: false,
            position: TimePosition::new(0, sample_rate),
            tempo: 120.0,
            time_signature: TimeSignature::default(),
            loop_enabled: false,
            loop_start: TimePosition::new(0, sample_rate),
            loop_end: TimePosition::from_seconds(8.0, sample_rate),
            metronome_enabled: false,
            count_in_bars: 1,
            sample_rate,
        }
    }
}

impl TransportState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start playback
    pub fn play(&mut self) {
        self.is_playing = true;
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.is_recording = false;
    }

    /// Pause playback (keeps position)
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// Start recording
    pub fn record(&mut self) {
        self.is_recording = true;
        self.is_playing = true;
    }

    /// Go to start
    pub fn rewind(&mut self) {
        self.position = TimePosition::new(0, self.sample_rate);
    }

    /// Set position in seconds
    pub fn seek(&mut self, seconds: f64) {
        self.position = TimePosition::from_seconds(seconds, self.sample_rate);
    }

    /// Advance position by given number of samples
    pub fn advance(&mut self, samples: u64) {
        self.position.samples += samples;

        // Handle looping
        if self.loop_enabled && self.position.samples >= self.loop_end.samples {
            self.position.samples = self.loop_start.samples;
        }
    }

    /// Get current bar number (1-indexed)
    pub fn current_bar(&self) -> u32 {
        let beats = self.position.as_beats(self.tempo);
        (beats / self.time_signature.numerator as f64) as u32 + 1
    }

    /// Get current beat within bar (1-indexed)
    pub fn current_beat(&self) -> u32 {
        let beats = self.position.as_beats(self.tempo);
        (beats % self.time_signature.numerator as f64) as u32 + 1
    }
}
