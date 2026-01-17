//! Guitar synthesizer using multiple Karplus-Strong strings

use crate::karplus_strong::KarplusStrong;
use crate::{midi_to_freq, STANDARD_TUNING};

/// Guitar configuration
#[derive(Debug, Clone)]
pub struct GuitarConfig {
    /// Sample rate
    pub sample_rate: u32,
    /// Number of strings
    pub num_strings: usize,
    /// Tuning (MIDI note numbers for open strings)
    pub tuning: Vec<u8>,
    /// Feedback (sustain) - 0.99 to 0.9999
    pub feedback: f32,
    /// Damping (brightness) - 0.0 to 1.0
    pub damping: f32,
    /// Output gain
    pub gain: f32,
    /// Stereo width (0.0 = mono, 1.0 = full stereo)
    pub stereo_width: f32,
}

impl Default for GuitarConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            num_strings: 6,
            tuning: STANDARD_TUNING.to_vec(),
            feedback: 0.996,
            damping: 0.5,
            gain: 0.8,
            stereo_width: 0.5,
        }
    }
}

impl GuitarConfig {
    /// Configuration for acoustic guitar sound
    pub fn acoustic() -> Self {
        Self {
            feedback: 0.997,
            damping: 0.4,
            gain: 0.85,
            ..Default::default()
        }
    }

    /// Configuration for electric guitar (clean)
    pub fn electric_clean() -> Self {
        Self {
            feedback: 0.998,
            damping: 0.3,
            gain: 0.7,
            ..Default::default()
        }
    }

    /// Configuration for nylon string guitar
    pub fn nylon() -> Self {
        Self {
            feedback: 0.994,
            damping: 0.6,
            gain: 0.8,
            ..Default::default()
        }
    }

    /// Configuration for bass guitar
    pub fn bass() -> Self {
        Self {
            num_strings: 4,
            tuning: vec![28, 33, 38, 43], // E1, A1, D2, G2
            feedback: 0.998,
            damping: 0.6,
            gain: 0.9,
            ..Default::default()
        }
    }

    /// Configuration for 7-string metal guitar
    pub fn seven_string_metal() -> Self {
        Self {
            num_strings: 7,
            tuning: crate::SEVEN_STRING_TUNING.to_vec(),
            feedback: 0.995,  // Tighter, less sustain
            damping: 0.35,     // Brighter for cut
            gain: 0.85,
            stereo_width: 0.6,
            ..Default::default()
        }
    }

    /// Configuration for 8-string djent/progressive metal
    pub fn eight_string_metal() -> Self {
        Self {
            num_strings: 8,
            tuning: crate::EIGHT_STRING_TUNING.to_vec(),
            feedback: 0.994,   // Very tight for low frequencies
            damping: 0.3,
            gain: 0.9,
            stereo_width: 0.7,
            ..Default::default()
        }
    }

    /// Configuration optimized for palm muting (technical death metal)
    pub fn tech_death() -> Self {
        Self {
            num_strings: 7,
            tuning: crate::SEVEN_STRING_DROP_A.to_vec(),
            feedback: 0.993,   // Very tight response
            damping: 0.4,
            gain: 0.85,
            stereo_width: 0.5,
            ..Default::default()
        }
    }

    /// Drop tuning configuration builder
    pub fn with_drop_tuning(tuning: &[u8]) -> Self {
        Self {
            num_strings: tuning.len(),
            tuning: tuning.to_vec(),
            feedback: 0.995,
            damping: 0.4,
            gain: 0.85,
            stereo_width: 0.5,
            ..Default::default()
        }
    }
}

/// State of a single string
pub struct StringState {
    /// The string synthesizer
    synth: KarplusStrong,
    /// Open string MIDI note
    open_note: u8,
    /// Current fret being played (None = not playing)
    current_fret: Option<u8>,
    /// Pan position (-1.0 = left, 1.0 = right)
    pan: f32,
}

impl StringState {
    fn new(sample_rate: u32, open_note: u8, pan: f32) -> Self {
        Self {
            synth: KarplusStrong::new(sample_rate),
            open_note,
            current_fret: None,
            pan,
        }
    }

    /// Get the frequency for a fret on this string
    fn fret_frequency(&self, fret: u8) -> f32 {
        midi_to_freq(self.open_note + fret)
    }
}

/// Multi-string guitar synthesizer
pub struct GuitarSynth {
    /// Individual strings
    strings: Vec<StringState>,
    /// Configuration
    config: GuitarConfig,
}

impl GuitarSynth {
    /// Create a new guitar synthesizer
    pub fn new(config: GuitarConfig) -> Self {
        let num_strings = config.num_strings;
        let mut strings = Vec::with_capacity(num_strings);

        for (i, &open_note) in config.tuning.iter().take(num_strings).enumerate() {
            // Pan strings across stereo field
            // String 1 (highest) slightly left, String 6 (lowest) slightly right
            let pan = if num_strings > 1 {
                let t = i as f32 / (num_strings - 1) as f32;
                (t * 2.0 - 1.0) * config.stereo_width
            } else {
                0.0
            };

            let mut string_state = StringState::new(config.sample_rate, open_note, pan);
            string_state.synth.set_feedback(config.feedback);
            string_state.synth.set_damping(config.damping);
            strings.push(string_state);
        }

        Self { strings, config }
    }

    /// Create with default configuration
    pub fn default_guitar() -> Self {
        Self::new(GuitarConfig::default())
    }

    /// Pluck a string at the given fret
    /// string: 1-indexed (1 = highest pitch string)
    /// fret: 0 = open string
    /// velocity: 0.0 - 1.0
    pub fn pluck(&mut self, string: u8, fret: u8, velocity: f32) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            let freq = self.strings[idx].fret_frequency(fret);
            self.strings[idx].synth.pluck_with_velocity(freq, velocity);
            self.strings[idx].current_fret = Some(fret);
        }
    }

    /// Mute a specific string
    pub fn mute_string(&mut self, string: u8) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            self.strings[idx].synth.stop();
            self.strings[idx].current_fret = None;
        }
    }

    /// Mute all strings
    pub fn mute_all(&mut self) {
        for string in &mut self.strings {
            string.synth.stop();
            string.current_fret = None;
        }
    }

    /// Pluck a chord (multiple notes at once)
    pub fn pluck_chord(&mut self, notes: &[(u8, u8, f32)]) {
        // notes: [(string, fret, velocity), ...]
        for &(string, fret, velocity) in notes {
            self.pluck(string, fret, velocity);
        }
    }

    /// Generate mono output
    pub fn next_sample_mono(&mut self) -> f32 {
        let mut output = 0.0;
        for string in &mut self.strings {
            output += string.synth.next_sample();
        }
        output * self.config.gain / self.strings.len().max(1) as f32
    }

    /// Generate stereo output (left, right)
    pub fn next_sample_stereo(&mut self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for string in &mut self.strings {
            let sample = string.synth.next_sample();
            // Pan law: equal power panning
            let pan_angle = (string.pan + 1.0) * std::f32::consts::FRAC_PI_4;
            left += sample * pan_angle.cos();
            right += sample * pan_angle.sin();
        }

        let scale = self.config.gain / self.strings.len().max(1) as f32;
        (left * scale, right * scale)
    }

    /// Fill a mono buffer
    pub fn fill_buffer_mono(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.next_sample_mono();
        }
    }

    /// Fill a stereo interleaved buffer (L, R, L, R, ...)
    pub fn fill_buffer_stereo(&mut self, output: &mut [f32]) {
        for chunk in output.chunks_exact_mut(2) {
            let (l, r) = self.next_sample_stereo();
            chunk[0] = l;
            chunk[1] = r;
        }
    }

    /// Check if any string is producing sound
    pub fn is_active(&self) -> bool {
        self.strings.iter().any(|s| s.synth.is_active())
    }

    /// Get the configuration
    pub fn config(&self) -> &GuitarConfig {
        &self.config
    }

    /// Update configuration parameters
    pub fn set_feedback(&mut self, feedback: f32) {
        self.config.feedback = feedback;
        for string in &mut self.strings {
            string.synth.set_feedback(feedback);
        }
    }

    /// Update damping
    pub fn set_damping(&mut self, damping: f32) {
        self.config.damping = damping;
        for string in &mut self.strings {
            string.synth.set_damping(damping);
        }
    }

    /// Update gain
    pub fn set_gain(&mut self, gain: f32) {
        self.config.gain = gain.clamp(0.0, 2.0);
    }
}

/// Metal techniques
impl GuitarSynth {
    /// Palm mute a string (essential for metal rhythm playing)
    /// Dampens the string for a tight, percussive sound
    pub fn palm_mute(&mut self, string: u8, fret: u8, velocity: f32) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            // Save original settings
            let original_feedback = self.config.feedback;
            let original_damping = self.config.damping;

            // Palm mute: much shorter sustain, darker tone
            self.strings[idx].synth.set_feedback(0.95); // Very short sustain
            self.strings[idx].synth.set_damping(0.85);   // Dark, muted tone
            self.strings[idx].synth.set_noise_blend(0.5); // Less attack noise

            let freq = self.strings[idx].fret_frequency(fret);
            self.strings[idx].synth.pluck_with_velocity(freq, velocity * 0.85);
            self.strings[idx].current_fret = Some(fret);

            // Restore original settings (next note will use normal settings)
            self.strings[idx].synth.set_feedback(original_feedback);
            self.strings[idx].synth.set_damping(original_damping);
            self.strings[idx].synth.set_noise_blend(0.7);
        }
    }

    /// Palm mute a chord (multiple palm-muted notes)
    pub fn palm_mute_chord(&mut self, notes: &[(u8, u8, f32)]) {
        for &(string, fret, velocity) in notes {
            self.palm_mute(string, fret, velocity);
        }
    }

    /// Tremolo pick - rapid retriggering with minimal attack
    /// Used for fast alternate picking passages
    pub fn tremolo_pick(&mut self, string: u8, fret: u8, velocity: f32) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            // Tremolo: quick attack, maintains energy from previous note
            self.strings[idx].synth.set_noise_blend(0.4); // Less pluck noise

            let freq = self.strings[idx].fret_frequency(fret);
            // Don't fully reset the string - adds to existing vibration
            self.strings[idx].synth.pluck_with_velocity(freq, velocity * 0.6);
            self.strings[idx].current_fret = Some(fret);

            self.strings[idx].synth.set_noise_blend(0.7);
        }
    }

    /// Pinch harmonic - squealing high overtone
    pub fn pinch_harmonic(&mut self, string: u8, fret: u8, velocity: f32) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            // Pinch harmonics produce notes at harmonic frequencies
            // Typically 2nd, 3rd, or 4th harmonic depending on pick position
            let base_freq = self.strings[idx].fret_frequency(fret);
            let harmonic_freq = base_freq * 3.0; // 3rd harmonic is common

            // Very bright, squealy tone
            self.strings[idx].synth.set_damping(0.1);
            self.strings[idx].synth.set_feedback(0.997);
            self.strings[idx].synth.set_noise_blend(0.3);

            self.strings[idx].synth.pluck_with_velocity(harmonic_freq, velocity * 0.9);
            self.strings[idx].current_fret = Some(fret);

            // Restore
            self.strings[idx].synth.set_damping(self.config.damping);
            self.strings[idx].synth.set_feedback(self.config.feedback);
            self.strings[idx].synth.set_noise_blend(0.7);
        }
    }

    /// Natural harmonic at a node point
    pub fn natural_harmonic(&mut self, string: u8, fret: u8, velocity: f32) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            let base_freq = midi_to_freq(self.strings[idx].open_note);

            // Common harmonic positions and their multipliers
            let multiplier = match fret {
                12 => 2.0,  // Octave
                7 | 19 => 3.0,  // Perfect 5th + octave
                5 | 24 => 4.0,  // 2 octaves
                4 | 9 | 16 => 5.0,  // Major 3rd + 2 octaves
                _ => 2.0,
            };

            let harmonic_freq = base_freq * multiplier;

            // Pure, bell-like tone
            self.strings[idx].synth.set_damping(0.2);
            self.strings[idx].synth.set_feedback(0.998);
            self.strings[idx].synth.set_noise_blend(0.2);

            self.strings[idx].synth.pluck_with_velocity(harmonic_freq, velocity * 0.7);
            self.strings[idx].current_fret = Some(fret);

            self.strings[idx].synth.set_damping(self.config.damping);
            self.strings[idx].synth.set_feedback(self.config.feedback);
            self.strings[idx].synth.set_noise_blend(0.7);
        }
    }
}

/// Hammer-on/pull-off simulation
impl GuitarSynth {
    /// Simulate hammer-on (pluck with less attack noise)
    pub fn hammer_on(&mut self, string: u8, fret: u8, velocity: f32) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            // Reduce noise blend for hammer-on sound
            let original_blend = 0.7; // Default noise blend
            self.strings[idx].synth.set_noise_blend(0.3);

            let freq = self.strings[idx].fret_frequency(fret);
            self.strings[idx].synth.pluck_with_velocity(freq, velocity * 0.7);
            self.strings[idx].current_fret = Some(fret);

            // Restore noise blend
            self.strings[idx].synth.set_noise_blend(original_blend);
        }
    }

    /// Simulate pull-off
    pub fn pull_off(&mut self, string: u8, fret: u8, velocity: f32) {
        // Pull-off is similar to hammer-on but with slightly more attack
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            self.strings[idx].synth.set_noise_blend(0.4);

            let freq = self.strings[idx].fret_frequency(fret);
            self.strings[idx].synth.pluck_with_velocity(freq, velocity * 0.6);
            self.strings[idx].current_fret = Some(fret);

            self.strings[idx].synth.set_noise_blend(0.7);
        }
    }

    /// Simulate a slide to a new fret with real pitch sliding
    pub fn slide_to(&mut self, string: u8, target_fret: u8, duration_samples: usize) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            if self.strings[idx].synth.is_active() {
                let target_freq = self.strings[idx].fret_frequency(target_fret);
                self.strings[idx].synth.slide_to(target_freq, duration_samples);
                self.strings[idx].current_fret = Some(target_fret);
            }
        }
    }
}

/// Pitch articulations (bends, vibrato, slides)
impl GuitarSynth {
    /// Bend a string by semitones (e.g., 1.0 = half step, 2.0 = whole step)
    pub fn bend(&mut self, string: u8, semitones: f32) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            self.strings[idx].synth.bend_semitones(semitones);
        }
    }

    /// Bend a string to a specific fret (smooth pitch change)
    pub fn bend_to_fret(&mut self, string: u8, target_fret: u8) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            let target_freq = self.strings[idx].fret_frequency(target_fret);
            self.strings[idx].synth.bend_to(target_freq);
        }
    }

    /// Set bend speed for a string (how fast bends reach target)
    pub fn set_bend_speed(&mut self, string: u8, rate: f64) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            self.strings[idx].synth.set_bend_rate(rate);
        }
    }

    /// Set bend speed for all strings
    pub fn set_global_bend_speed(&mut self, rate: f64) {
        for string in &mut self.strings {
            string.synth.set_bend_rate(rate);
        }
    }

    /// Release bend (return to fretted pitch)
    pub fn release_bend(&mut self, string: u8) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            if let Some(fret) = self.strings[idx].current_fret {
                let freq = self.strings[idx].fret_frequency(fret);
                self.strings[idx].synth.bend_to(freq);
            }
        }
    }

    /// Enable vibrato on a string
    pub fn enable_vibrato(&mut self, string: u8, depth_semitones: f32, rate_hz: f32) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            self.strings[idx].synth.set_vibrato(depth_semitones, rate_hz);
        }
    }

    /// Enable default vibrato on a string
    pub fn enable_default_vibrato(&mut self, string: u8) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            self.strings[idx].synth.enable_vibrato();
        }
    }

    /// Disable vibrato on a string
    pub fn disable_vibrato(&mut self, string: u8) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            self.strings[idx].synth.disable_vibrato();
        }
    }

    /// Enable vibrato on all active strings
    pub fn enable_global_vibrato(&mut self, depth_semitones: f32, rate_hz: f32) {
        for string in &mut self.strings {
            string.synth.set_vibrato(depth_semitones, rate_hz);
        }
    }

    /// Disable vibrato on all strings
    pub fn disable_global_vibrato(&mut self) {
        for string in &mut self.strings {
            string.synth.disable_vibrato();
        }
    }

    /// Pre-bend: Bend before attack, then release
    /// Common technique for blues/rock solos
    pub fn pre_bend(&mut self, string: u8, fret: u8, bend_semitones: f32, velocity: f32) {
        let idx = (string as usize).saturating_sub(1);
        if idx < self.strings.len() {
            // Calculate bent frequency
            let base_freq = self.strings[idx].fret_frequency(fret);
            let bent_freq = base_freq * 2.0_f32.powf(bend_semitones / 12.0);

            // Set fast bend rate for release
            self.strings[idx].synth.set_bend_rate(0.005);

            // Pluck at bent pitch
            self.strings[idx].synth.pluck_with_velocity(bent_freq, velocity);
            self.strings[idx].current_fret = Some(fret);

            // Set target to base pitch (will release down)
            self.strings[idx].synth.bend_to(base_freq);
        }
    }

    /// Dive bomb: Rapid downward pitch dive using whammy bar
    pub fn dive_bomb(&mut self, semitones_down: f32) {
        for string in &mut self.strings {
            if string.synth.is_active() {
                string.synth.set_bend_rate(0.002); // Slow for dramatic effect
                string.synth.bend_semitones(-semitones_down);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guitar_creation() {
        let guitar = GuitarSynth::default_guitar();
        assert_eq!(guitar.strings.len(), 6);
    }

    #[test]
    fn test_pluck() {
        let mut guitar = GuitarSynth::default_guitar();
        guitar.pluck(1, 0, 0.8); // Open high E

        // Generate samples
        let mut buffer = vec![0.0; 1000];
        guitar.fill_buffer_mono(&mut buffer);

        // Should have output
        let max = buffer.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }

    #[test]
    fn test_chord() {
        let mut guitar = GuitarSynth::default_guitar();

        // E major chord
        guitar.pluck_chord(&[
            (6, 0, 0.8), // E2
            (5, 2, 0.8), // B2
            (4, 2, 0.8), // E3
            (3, 1, 0.8), // G#3
            (2, 0, 0.8), // B3
            (1, 0, 0.8), // E4
        ]);

        assert!(guitar.is_active());
    }

    #[test]
    fn test_stereo() {
        let mut guitar = GuitarSynth::default_guitar();
        guitar.pluck(6, 0, 0.8); // Low E (should be panned right)
        guitar.pluck(1, 0, 0.8); // High E (should be panned left)

        // Generate some samples
        let mut buffer = vec![0.0; 100];
        guitar.fill_buffer_stereo(&mut buffer);

        // Should have output in stereo
        let max_l = buffer.iter().step_by(2).map(|s| s.abs()).fold(0.0_f32, f32::max);
        let max_r = buffer.iter().skip(1).step_by(2).map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max_l > 0.0 || max_r > 0.0);
    }
}
