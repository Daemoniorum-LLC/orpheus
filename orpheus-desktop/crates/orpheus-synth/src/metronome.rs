//! Metronome synthesis
//!
//! Simple click sound generator for tempo reference during recording.

/// Metronome click generator
#[derive(Debug)]
pub struct Metronome {
    sample_rate: u32,
    /// Samples remaining in current click
    samples_remaining: usize,
    /// Current amplitude
    amplitude: f32,
    /// Decay rate per sample
    decay: f32,
    /// Is this the downbeat (beat 1)?
    is_downbeat: bool,
    /// Base frequency for click
    frequency: f32,
    /// Phase accumulator
    phase: f32,
    /// Phase increment
    phase_inc: f32,
    /// Volume (0.0 - 1.0)
    volume: f32,
}

impl Metronome {
    /// Create a new metronome
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            samples_remaining: 0,
            amplitude: 0.0,
            decay: 0.0,
            is_downbeat: false,
            frequency: 1000.0,
            phase: 0.0,
            phase_inc: 0.0,
            volume: 0.6,
        }
    }

    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Trigger a click
    ///
    /// - `is_downbeat`: true for beat 1 (higher pitch), false for other beats
    pub fn trigger(&mut self, is_downbeat: bool) {
        self.is_downbeat = is_downbeat;

        // Duration: ~30ms for normal clicks, ~50ms for downbeat
        let duration_ms = if is_downbeat { 50.0 } else { 30.0 };
        self.samples_remaining = (self.sample_rate as f32 * duration_ms / 1000.0) as usize;

        // Frequency: higher for downbeat
        self.frequency = if is_downbeat { 1500.0 } else { 1000.0 };
        self.phase_inc = self.frequency * 2.0 * std::f32::consts::PI / self.sample_rate as f32;

        // Start at full amplitude
        self.amplitude = if is_downbeat { 0.8 } else { 0.5 };

        // Decay to silence over the duration
        self.decay = self.amplitude / self.samples_remaining as f32;

        // Reset phase
        self.phase = 0.0;
    }

    /// Generate next sample
    pub fn next_sample(&mut self) -> f32 {
        if self.samples_remaining == 0 {
            return 0.0;
        }

        // Generate sine wave with current amplitude
        let sample = self.amplitude * self.phase.sin() * self.volume;

        // Advance phase
        self.phase += self.phase_inc;
        if self.phase > 2.0 * std::f32::consts::PI {
            self.phase -= 2.0 * std::f32::consts::PI;
        }

        // Apply decay
        self.amplitude -= self.decay;
        if self.amplitude < 0.0 {
            self.amplitude = 0.0;
        }

        self.samples_remaining -= 1;
        sample
    }

    /// Check if currently producing sound
    pub fn is_active(&self) -> bool {
        self.samples_remaining > 0
    }
}

/// Metronome timing helper
#[derive(Debug)]
pub struct MetronomeTiming {
    /// Sample rate
    sample_rate: u32,
    /// Current tempo in BPM
    tempo: f64,
    /// Time signature numerator (beats per bar)
    beats_per_bar: u8,
    /// Samples since last beat
    samples_since_beat: u64,
    /// Samples per beat
    samples_per_beat: u64,
    /// Current beat number (1-indexed, 1-4 for 4/4)
    current_beat: u8,
    /// Whether metronome is enabled
    enabled: bool,
}

impl MetronomeTiming {
    /// Create new metronome timing
    pub fn new(sample_rate: u32, tempo: f64, beats_per_bar: u8) -> Self {
        let samples_per_beat = (sample_rate as f64 * 60.0 / tempo) as u64;
        Self {
            sample_rate,
            tempo,
            beats_per_bar,
            samples_since_beat: 0,
            samples_per_beat,
            current_beat: 1,
            enabled: false,
        }
    }

    /// Enable/disable metronome
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if metronome is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set tempo
    pub fn set_tempo(&mut self, tempo: f64) {
        self.tempo = tempo;
        self.samples_per_beat = (self.sample_rate as f64 * 60.0 / tempo) as u64;
    }

    /// Set time signature
    pub fn set_beats_per_bar(&mut self, beats: u8) {
        self.beats_per_bar = beats;
    }

    /// Reset to beat 1
    pub fn reset(&mut self) {
        self.samples_since_beat = 0;
        self.current_beat = 1;
    }

    /// Advance by a number of samples and check if a beat should trigger
    ///
    /// Returns `Some(true)` for downbeat (beat 1), `Some(false)` for other beats, `None` for no beat
    pub fn advance(&mut self, samples: u64) -> Option<bool> {
        if !self.enabled {
            return None;
        }

        self.samples_since_beat += samples;

        if self.samples_since_beat >= self.samples_per_beat {
            self.samples_since_beat -= self.samples_per_beat;

            // Advance to next beat first
            self.current_beat += 1;
            if self.current_beat > self.beats_per_bar {
                self.current_beat = 1;
            }

            // The click is for the beat we just entered
            let is_downbeat = self.current_beat == 1;

            Some(is_downbeat)
        } else {
            None
        }
    }

    /// Get current beat (1-indexed)
    pub fn current_beat(&self) -> u8 {
        self.current_beat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metronome_click() {
        let mut metro = Metronome::new(48000);

        // Should be silent initially
        assert!(!metro.is_active());
        assert_eq!(metro.next_sample(), 0.0);

        // Trigger a click
        metro.trigger(true);
        assert!(metro.is_active());

        // Process all samples and check we get some non-zero audio
        let mut had_nonzero = false;
        while metro.is_active() {
            let sample = metro.next_sample();
            if sample != 0.0 {
                had_nonzero = true;
            }
        }
        assert!(had_nonzero, "Click should produce non-zero audio");

        // Should be silent again
        assert!(!metro.is_active());
    }

    #[test]
    fn test_metronome_timing() {
        let mut timing = MetronomeTiming::new(48000, 120.0, 4);

        // At 120 BPM, one beat = 0.5 seconds = 24000 samples
        assert_eq!(timing.samples_per_beat, 24000);

        timing.set_enabled(true);

        // Advance by half a beat - no trigger
        assert!(timing.advance(12000).is_none());

        // Advance by another half beat - should trigger beat 2
        let result = timing.advance(12000);
        assert_eq!(result, Some(false)); // Not downbeat
        assert_eq!(timing.current_beat(), 2);

        // Advance through beats 2, 3, 4 and back to 1
        timing.advance(24000); // beat 3
        timing.advance(24000); // beat 4
        let result = timing.advance(24000); // beat 1 (downbeat)
        assert_eq!(result, Some(true)); // Downbeat!
    }
}
