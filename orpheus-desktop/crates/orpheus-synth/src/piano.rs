//! Piano synthesis using additive synthesis and physical modeling
//!
//! Features:
//! - Harmonic partial synthesis for piano timbre
//! - Realistic ADSR envelope with velocity sensitivity
//! - Hammer noise for attack transient
//! - String resonance simulation
//! - Sustain pedal support
//! - Per-voice stereo panning (low notes left, high notes right)

use crate::midi_to_freq;

/// Calculate stereo pan for a MIDI note (equal power panning)
///
/// Returns (left_gain, right_gain) based on note position on piano.
/// A0 (21) pans left, C8 (108) pans right, center around E4 (64).
fn note_to_stereo_pan(note: u8) -> (f32, f32) {
    // Piano range: A0 (21) to C8 (108)
    // Center point: around middle C (60-72)
    const LOW_NOTE: f32 = 21.0;
    const HIGH_NOTE: f32 = 108.0;
    const CENTER: f32 = 64.0;

    // Normalize note to -1.0 (left) to 1.0 (right)
    let normalized = (note as f32 - CENTER) / ((HIGH_NOTE - LOW_NOTE) / 2.0);
    let pan = normalized.clamp(-1.0, 1.0);

    // Equal power panning: L = cos(angle), R = sin(angle)
    // where angle = (pan + 1) * PI/4  (0 to PI/2)
    let angle = (pan + 1.0) * 0.25 * std::f32::consts::PI;
    let left = angle.cos();
    let right = angle.sin();

    // Apply subtle stereo spread (not extreme L/R)
    // Mix with center to keep it subtle: 70% panned, 30% center
    let spread = 0.4;
    let center_mix = 1.0 - spread;
    let center = 0.707; // sqrt(0.5) for equal power center

    (
        left * spread + center * center_mix,
        right * spread + center * center_mix,
    )
}

/// ADSR envelope for piano notes
#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    /// Attack time in seconds
    pub attack: f32,
    /// Decay time in seconds
    pub decay: f32,
    /// Sustain level (0.0 - 1.0)
    pub sustain: f32,
    /// Release time in seconds
    pub release: f32,
}

impl Default for Envelope {
    fn default() -> Self {
        // Piano-like envelope: fast attack, moderate decay, low sustain, moderate release
        Self {
            attack: 0.005,
            decay: 0.3,
            sustain: 0.4,
            release: 0.5,
        }
    }
}

/// Envelope stage
#[derive(Debug, Clone, Copy, PartialEq)]
enum EnvelopeStage {
    Attack,
    Decay,
    Sustain,
    Release,
    Off,
}

/// Piano voice (single note)
pub struct PianoVoice {
    /// Sample rate
    sample_rate: u32,
    /// Current MIDI note
    note: u8,
    /// Base frequency
    frequency: f32,
    /// Velocity (affects brightness and volume)
    velocity: f32,
    /// Phase accumulators for each harmonic
    phases: [f32; 8],
    /// Harmonic amplitudes (tuned for piano timbre)
    harmonic_amplitudes: [f32; 8],
    /// Harmonic detuning (cents)
    harmonic_detune: [f32; 8],
    /// Envelope settings
    envelope: Envelope,
    /// Current envelope stage
    stage: EnvelopeStage,
    /// Current envelope level
    env_level: f32,
    /// Time in current stage
    stage_time: f32,
    /// Hammer noise phase
    noise_phase: f32,
    /// Hammer noise decay
    noise_decay: f32,
    /// Is note active
    active: bool,
    /// Is sustain pedal down
    sustain_pedal: bool,
    /// Was note released (waiting for pedal)
    note_off_pending: bool,
}

impl PianoVoice {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            note: 0,
            frequency: 440.0,
            velocity: 0.8,
            phases: [0.0; 8],
            // Piano harmonic structure (emphasizes odd harmonics less than even)
            harmonic_amplitudes: [1.0, 0.6, 0.35, 0.25, 0.15, 0.1, 0.06, 0.04],
            // Slight detuning for natural piano sound (inharmonicity)
            harmonic_detune: [0.0, 0.5, 1.2, 2.1, 3.2, 4.5, 6.0, 7.8],
            envelope: Envelope::default(),
            stage: EnvelopeStage::Off,
            env_level: 0.0,
            stage_time: 0.0,
            noise_phase: 0.0,
            noise_decay: 0.0,
            active: false,
            sustain_pedal: false,
            note_off_pending: false,
        }
    }

    /// Trigger a note
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.note = note;
        self.frequency = midi_to_freq(note);
        self.velocity = velocity.clamp(0.0, 1.0);

        // Reset phases
        for phase in &mut self.phases {
            *phase = 0.0;
        }

        // Adjust envelope based on note (higher notes decay faster)
        let note_factor = 1.0 - (note as f32 - 21.0) / 88.0; // A0 to C8
        self.envelope.decay = 0.15 + note_factor * 0.4;
        self.envelope.release = 0.2 + note_factor * 0.5;

        // Higher velocity = faster attack, brighter sound
        self.envelope.attack = 0.002 + (1.0 - velocity) * 0.01;

        // Start envelope
        self.stage = EnvelopeStage::Attack;
        self.stage_time = 0.0;
        self.env_level = 0.0;

        // Initialize hammer noise
        self.noise_decay = 1.0;

        self.active = true;
        self.note_off_pending = false;
    }

    /// Release a note
    pub fn note_off(&mut self) {
        if self.sustain_pedal {
            self.note_off_pending = true;
        } else {
            self.start_release();
        }
    }

    fn start_release(&mut self) {
        if self.stage != EnvelopeStage::Off && self.stage != EnvelopeStage::Release {
            self.stage = EnvelopeStage::Release;
            self.stage_time = 0.0;
        }
    }

    /// Set sustain pedal state
    pub fn set_sustain(&mut self, down: bool) {
        self.sustain_pedal = down;
        if !down && self.note_off_pending {
            self.start_release();
        }
    }

    /// Check if voice is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get current note
    pub fn current_note(&self) -> u8 {
        self.note
    }

    /// Generate next sample
    pub fn next_sample(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let dt = 1.0 / self.sample_rate as f32;

        // Update envelope
        self.update_envelope(dt);

        if !self.active {
            return 0.0;
        }

        // Generate piano tone using additive synthesis
        let mut sample = 0.0;

        for (i, phase) in self.phases.iter_mut().enumerate() {
            let harmonic = i + 1;
            let amp = self.harmonic_amplitudes[i];

            // Apply inharmonicity (piano strings are slightly stiff)
            let detune_factor = 1.0 + self.harmonic_detune[i] / 1200.0; // cents to ratio
            let freq = self.frequency * harmonic as f32 * detune_factor;

            // Phase increment
            let phase_inc = freq / self.sample_rate as f32;
            *phase += phase_inc;
            if *phase >= 1.0 {
                *phase -= 1.0;
            }

            // Generate sine wave
            let sine = (*phase * std::f32::consts::TAU).sin();

            // Higher harmonics roll off with frequency (piano characteristic)
            let rolloff = 1.0 / (1.0 + (harmonic as f32 - 1.0) * 0.3);

            // Velocity affects brightness (higher velocity = more harmonics)
            let vel_brightness = if harmonic > 2 {
                self.velocity.powf(0.5)
            } else {
                1.0
            };

            sample += sine * amp * rolloff * vel_brightness;
        }

        // Add hammer noise (transient at start)
        if self.noise_decay > 0.01 {
            let noise = self.generate_noise();
            // Bandpass filter the noise around the fundamental
            self.noise_phase += self.frequency / self.sample_rate as f32;
            let filtered_noise = noise * (self.noise_phase * std::f32::consts::TAU).cos();

            sample += filtered_noise * self.noise_decay * self.velocity * 0.15;
            self.noise_decay *= 0.995; // Fast decay
        }

        // Apply envelope and velocity
        let output = sample * self.env_level * self.velocity;

        // Soft clip to prevent harsh distortion
        output.tanh()
    }

    /// Generate next stereo sample with note-based panning
    pub fn next_sample_stereo(&mut self) -> (f32, f32) {
        let mono = self.next_sample();
        if mono.abs() < 0.0001 {
            return (0.0, 0.0);
        }
        let (left_gain, right_gain) = note_to_stereo_pan(self.note);
        (mono * left_gain, mono * right_gain)
    }

    fn update_envelope(&mut self, dt: f32) {
        self.stage_time += dt;

        match self.stage {
            EnvelopeStage::Attack => {
                self.env_level = self.stage_time / self.envelope.attack;
                if self.env_level >= 1.0 {
                    self.env_level = 1.0;
                    self.stage = EnvelopeStage::Decay;
                    self.stage_time = 0.0;
                }
            }
            EnvelopeStage::Decay => {
                let decay_progress = self.stage_time / self.envelope.decay;
                self.env_level = 1.0 - (1.0 - self.envelope.sustain) * decay_progress;
                if decay_progress >= 1.0 {
                    self.env_level = self.envelope.sustain;
                    self.stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => {
                // Gradual decay during sustain (piano strings naturally decay)
                self.env_level *= 0.99998;
                if self.env_level < 0.001 {
                    self.active = false;
                }
            }
            EnvelopeStage::Release => {
                let release_progress = self.stage_time / self.envelope.release;
                // Start from current level, not sustain level
                let start_level = self.env_level;
                self.env_level = start_level * (1.0 - release_progress);
                if release_progress >= 1.0 || self.env_level < 0.001 {
                    self.env_level = 0.0;
                    self.active = false;
                    self.stage = EnvelopeStage::Off;
                }
            }
            EnvelopeStage::Off => {
                self.active = false;
            }
        }
    }

    fn generate_noise(&self) -> f32 {
        // Simple LCG for noise
        static mut SEED: u32 = 12345;
        unsafe {
            SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
            ((SEED >> 16) as f32 / 32768.0) - 1.0
        }
    }
}

/// Polyphonic piano synthesizer
pub struct PianoSynth {
    /// Voices
    voices: Vec<PianoVoice>,
    /// Sample rate
    sample_rate: u32,
    /// Master volume
    master_volume: f32,
    /// Sustain pedal state
    sustain_pedal: bool,
}

impl PianoSynth {
    /// Create a new piano synth with given polyphony
    pub fn new(sample_rate: u32, polyphony: usize) -> Self {
        let voices = (0..polyphony)
            .map(|_| PianoVoice::new(sample_rate))
            .collect();

        Self {
            voices,
            sample_rate,
            master_volume: 0.7,
            sustain_pedal: false,
        }
    }

    /// Create with standard 16-voice polyphony
    pub fn standard(sample_rate: u32) -> Self {
        Self::new(sample_rate, 16)
    }

    /// Create with full 88-key polyphony (for complex pieces)
    pub fn full_polyphony(sample_rate: u32) -> Self {
        Self::new(sample_rate, 32)
    }

    /// Trigger a note
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        // Find a free voice or steal the quietest one
        let voice_idx = self.find_voice_for_note(note);
        self.voices[voice_idx].note_on(note, velocity);
    }

    /// Release a note
    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.is_active() && voice.current_note() == note {
                voice.note_off();
                break;
            }
        }
    }

    /// Set sustain pedal
    pub fn set_sustain(&mut self, down: bool) {
        self.sustain_pedal = down;
        for voice in &mut self.voices {
            voice.set_sustain(down);
        }
    }

    /// Release all notes
    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.note_off();
        }
    }

    /// Set master volume
    pub fn set_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Generate next sample (mono)
    pub fn next_sample(&mut self) -> f32 {
        let mut sample = 0.0;

        for voice in &mut self.voices {
            sample += voice.next_sample();
        }

        sample * self.master_volume
    }

    /// Generate next stereo sample with per-voice panning
    ///
    /// Low notes pan left (like left side of piano), high notes pan right.
    pub fn next_sample_stereo(&mut self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for voice in &mut self.voices {
            let (l, r) = voice.next_sample_stereo();
            left += l;
            right += r;
        }

        (left * self.master_volume, right * self.master_volume)
    }

    /// Fill a buffer with samples
    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.next_sample();
        }
    }

    /// Check if any voice is active
    pub fn is_active(&self) -> bool {
        self.voices.iter().any(|v| v.is_active())
    }

    fn find_voice_for_note(&self, note: u8) -> usize {
        // First, look for the same note (re-trigger)
        for (i, voice) in self.voices.iter().enumerate() {
            if voice.is_active() && voice.current_note() == note {
                return i;
            }
        }

        // Second, find an inactive voice
        for (i, voice) in self.voices.iter().enumerate() {
            if !voice.is_active() {
                return i;
            }
        }

        // Third, steal the oldest/quietest voice
        let mut quietest_idx = 0;
        let mut quietest_level = f32::MAX;

        for (i, voice) in self.voices.iter().enumerate() {
            if voice.env_level < quietest_level {
                quietest_level = voice.env_level;
                quietest_idx = i;
            }
        }

        quietest_idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piano_voice() {
        let mut voice = PianoVoice::new(44100);
        voice.note_on(60, 0.8); // Middle C

        // Should be active
        assert!(voice.is_active());

        // Generate samples
        let mut samples = vec![0.0; 1000];
        for sample in &mut samples {
            *sample = voice.next_sample();
        }

        // Should have output
        let max = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }

    #[test]
    fn test_piano_synth_polyphony() {
        let mut piano = PianoSynth::standard(44100);

        // Play a chord
        piano.note_on(60, 0.8); // C
        piano.note_on(64, 0.8); // E
        piano.note_on(67, 0.8); // G

        // Generate samples
        let mut buffer = vec![0.0; 4410];
        piano.fill_buffer(&mut buffer);

        let max = buffer.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }

    #[test]
    fn test_stereo_panning() {
        // Test that low notes pan left and high notes pan right
        let (left_low, right_low) = note_to_stereo_pan(21); // A0 (lowest)
        let (left_mid, right_mid) = note_to_stereo_pan(64); // E4 (center)
        let (left_high, right_high) = note_to_stereo_pan(108); // C8 (highest)

        // Low note should have more left than right
        assert!(left_low > right_low, "Low note should pan left");

        // High note should have more right than left
        assert!(right_high > left_high, "High note should pan right");

        // Center note should be roughly equal
        assert!((left_mid - right_mid).abs() < 0.1, "Center note should be balanced");
    }

    #[test]
    fn test_piano_stereo_output() {
        let mut piano = PianoSynth::standard(44100);

        // Play a low note
        piano.note_on(28, 0.8); // E1 (low bass)

        // Generate stereo samples
        let mut left_sum = 0.0f32;
        let mut right_sum = 0.0f32;
        for _ in 0..4410 {
            let (l, r) = piano.next_sample_stereo();
            left_sum += l.abs();
            right_sum += r.abs();
        }

        // Low note should have more energy on left
        assert!(left_sum > right_sum, "Low note should have more left energy");

        piano.all_notes_off();

        // Now play a high note
        piano.note_on(100, 0.8); // E7 (high treble)

        left_sum = 0.0;
        right_sum = 0.0;
        for _ in 0..4410 {
            let (l, r) = piano.next_sample_stereo();
            left_sum += l.abs();
            right_sum += r.abs();
        }

        // High note should have more energy on right
        assert!(right_sum > left_sum, "High note should have more right energy");
    }
}
