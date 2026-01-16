//! Karplus-Strong string synthesis algorithm
//!
//! A physical modeling approach to string sound synthesis.
//! The algorithm simulates a vibrating string by:
//! 1. Initializing a delay line with noise (the "pluck")
//! 2. Repeatedly averaging adjacent samples with feedback
//! 3. The delay line length determines the fundamental frequency

use rand::Rng;

/// Karplus-Strong string synthesizer
pub struct KarplusStrong {
    /// Delay line buffer
    buffer: Vec<f32>,
    /// Current read position (fractional for pitch bending)
    read_pos: f64,
    /// Sample rate in Hz
    sample_rate: u32,
    /// Feedback amount (0.0 - 1.0, controls decay)
    feedback: f32,
    /// Damping factor (0.0 - 1.0, controls brightness)
    damping: f32,
    /// Blend between noise and sawtooth for pluck character
    noise_blend: f32,
    /// Current amplitude
    amplitude: f32,
    /// Previous sample (for lowpass filtering)
    prev_sample: f32,
    /// Is string currently sounding?
    active: bool,
    /// Current effective period (for pitch bending)
    current_period: f64,
    /// Target period (when bending)
    target_period: f64,
    /// Pitch bend rate (how fast to reach target, 0-1 per sample)
    bend_rate: f64,
    /// Vibrato depth in semitones
    vibrato_depth: f32,
    /// Vibrato rate in Hz
    vibrato_rate: f32,
    /// Vibrato phase
    vibrato_phase: f64,
    /// Peak tracker for decay detection (rolling max over one period)
    peak_tracker: f32,
    /// Samples since last peak reset
    peak_samples: usize,
}

impl KarplusStrong {
    /// Create a new Karplus-Strong synthesizer
    pub fn new(sample_rate: u32) -> Self {
        Self {
            buffer: Vec::new(),
            read_pos: 0.0,
            sample_rate,
            feedback: 0.996,     // Slightly less than 1 for decay
            damping: 0.5,       // Mid-range brightness
            noise_blend: 0.7,   // Mostly noise with some harmonic content
            amplitude: 1.0,
            prev_sample: 0.0,
            active: false,
            current_period: 100.0,
            target_period: 100.0,
            bend_rate: 0.001,    // Default: slow bends
            vibrato_depth: 0.0,  // No vibrato by default
            vibrato_rate: 5.0,   // 5 Hz vibrato when enabled
            vibrato_phase: 0.0,
            peak_tracker: 0.0,
            peak_samples: 0,
        }
    }

    /// Set the feedback amount (affects sustain/decay)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.9, 0.9999);
    }

    /// Set the damping factor (affects brightness)
    /// Lower values = brighter sound, higher = darker
    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping.clamp(0.0, 1.0);
    }

    /// Set noise blend (0.0 = pure harmonic, 1.0 = pure noise)
    pub fn set_noise_blend(&mut self, blend: f32) {
        self.noise_blend = blend.clamp(0.0, 1.0);
    }

    /// Set amplitude
    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude.clamp(0.0, 1.0);
    }

    /// Set pitch bend rate (how fast to reach target pitch)
    /// Values: 0.0001 (very slow) to 0.1 (nearly instant)
    pub fn set_bend_rate(&mut self, rate: f64) {
        self.bend_rate = rate.clamp(0.0001, 0.1);
    }

    /// Set vibrato parameters
    pub fn set_vibrato(&mut self, depth_semitones: f32, rate_hz: f32) {
        self.vibrato_depth = depth_semitones.clamp(0.0, 2.0);
        self.vibrato_rate = rate_hz.clamp(0.1, 15.0);
    }

    /// Enable vibrato with default settings
    pub fn enable_vibrato(&mut self) {
        self.vibrato_depth = 0.3;  // Subtle vibrato
        self.vibrato_rate = 5.0;   // 5 Hz
    }

    /// Disable vibrato
    pub fn disable_vibrato(&mut self) {
        self.vibrato_depth = 0.0;
    }

    /// Bend pitch to a new frequency
    pub fn bend_to(&mut self, target_frequency: f32) {
        self.target_period = self.sample_rate as f64 / target_frequency as f64;
    }

    /// Bend pitch by semitones relative to current
    pub fn bend_semitones(&mut self, semitones: f32) {
        let ratio = 2.0_f64.powf(semitones as f64 / 12.0);
        self.target_period = self.current_period / ratio;
    }

    /// Slide to a frequency over a given duration
    pub fn slide_to(&mut self, target_frequency: f32, duration_samples: usize) {
        self.target_period = self.sample_rate as f64 / target_frequency as f64;
        // Calculate rate needed to reach target in given duration
        let period_diff = (self.target_period - self.current_period).abs();
        if duration_samples > 0 && period_diff > 0.0 {
            self.bend_rate = period_diff / (self.current_period * duration_samples as f64);
        }
    }

    /// Reset pitch bend (return to plucked frequency)
    pub fn reset_bend(&mut self) {
        // Target is already set by pluck, just ensure we're at it
        self.current_period = self.target_period;
    }

    /// Pluck the string at the given frequency
    pub fn pluck(&mut self, frequency: f32) {
        // Calculate period (fractional for precision)
        let period = self.sample_rate as f64 / frequency as f64;
        self.current_period = period;
        self.target_period = period;

        // Buffer needs to be large enough for lowest possible pitch during bends
        // Use 2x the initial period to allow for downward bends
        let buffer_size = ((period * 2.0) as usize).max(2).min(4096);

        // Resize buffer if needed
        self.buffer.resize(buffer_size, 0.0);

        // Fill buffer with initial excitation
        let mut rng = rand::thread_rng();

        for i in 0..buffer_size {
            // Mix noise with a gentle triangle wave for more harmonic content
            let noise = rng.gen::<f32>() * 2.0 - 1.0;
            let phase = i as f32 / buffer_size as f32;
            let triangle = if phase < 0.5 {
                phase * 4.0 - 1.0
            } else {
                3.0 - phase * 4.0
            };

            self.buffer[i] = self.noise_blend * noise + (1.0 - self.noise_blend) * triangle;
        }

        // Apply initial window to soften attack
        for i in 0..buffer_size.min(10) {
            let window = i as f32 / 10.0;
            self.buffer[i] *= window;
        }

        self.read_pos = 0.0;
        self.prev_sample = 0.0;
        self.vibrato_phase = 0.0;
        self.peak_tracker = 1.0;  // Initial excitation has high energy
        self.peak_samples = 0;
        self.active = true;
    }

    /// Pluck with velocity (affects both amplitude and brightness)
    pub fn pluck_with_velocity(&mut self, frequency: f32, velocity: f32) {
        // Higher velocity = brighter sound and higher amplitude
        let vel = velocity.clamp(0.0, 1.0);

        // Temporarily adjust damping based on velocity
        let original_damping = self.damping;
        self.damping = self.damping * (1.5 - vel * 0.5); // Less damping for higher velocity
        self.damping = self.damping.clamp(0.1, 0.9);

        self.set_amplitude(vel);
        self.pluck(frequency);

        // Restore original damping
        self.damping = original_damping;
    }

    /// Stop the string (mute)
    pub fn stop(&mut self) {
        self.active = false;
        // Quick fade out to avoid clicks
        for sample in &mut self.buffer {
            *sample *= 0.9;
        }
    }

    /// Check if string is still producing sound
    pub fn is_active(&self) -> bool {
        self.active && self.amplitude > 0.001
    }

    /// Generate the next sample
    pub fn next_sample(&mut self) -> f32 {
        if !self.active || self.buffer.is_empty() {
            return 0.0;
        }

        let buffer_len = self.buffer.len();

        // Update pitch bend (smooth interpolation toward target)
        if (self.current_period - self.target_period).abs() > 0.01 {
            let diff = self.target_period - self.current_period;
            self.current_period += diff * self.bend_rate;
        }

        // Apply vibrato modulation
        let effective_period = if self.vibrato_depth > 0.0 {
            // Vibrato oscillation
            self.vibrato_phase += self.vibrato_rate as f64 / self.sample_rate as f64;
            if self.vibrato_phase >= 1.0 {
                self.vibrato_phase -= 1.0;
            }
            let vibrato_mod = (self.vibrato_phase * std::f64::consts::TAU).sin();
            // Convert depth from semitones to period ratio
            let depth_ratio = 2.0_f64.powf(self.vibrato_depth as f64 * vibrato_mod / 12.0);
            self.current_period * depth_ratio
        } else {
            self.current_period
        };

        // Read from fractional position using linear interpolation
        let read_int = self.read_pos as usize % buffer_len;
        let read_frac = self.read_pos - self.read_pos.floor();
        let next_int = (read_int + 1) % buffer_len;

        // Linear interpolation for smooth pitch changes
        let current = self.buffer[read_int] * (1.0 - read_frac as f32)
                    + self.buffer[next_int] * read_frac as f32;

        // Calculate write position (one period behind read)
        let write_pos = (self.read_pos - effective_period).rem_euclid(buffer_len as f64);
        let write_int = write_pos as usize % buffer_len;

        // Karplus-Strong averaging with damping (lowpass filter)
        // Read next sample for averaging
        let next_sample = self.buffer[(read_int + 1) % buffer_len];
        let averaged = (current + next_sample) * 0.5;
        let filtered = self.damping * self.prev_sample + (1.0 - self.damping) * averaged;

        // Apply feedback and store back at write position
        self.buffer[write_int] = filtered * self.feedback;

        // Advance read position by 1 sample
        self.read_pos = (self.read_pos + 1.0).rem_euclid(buffer_len as f64);
        self.prev_sample = filtered;

        // Track peak amplitude over one period for decay detection
        let sample_abs = filtered.abs();
        if sample_abs > self.peak_tracker {
            self.peak_tracker = sample_abs;
        }
        self.peak_samples += 1;

        // Check decay after each period completes
        let period_samples = self.current_period as usize;
        if self.peak_samples >= period_samples {
            // Check if peak over last period was below threshold
            if self.peak_tracker < 0.0001 {
                self.active = false;
            }
            // Reset for next period
            self.peak_tracker = 0.0;
            self.peak_samples = 0;
        }

        current * self.amplitude
    }

    /// Fill a buffer with samples
    pub fn fill_buffer(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.next_sample();
        }
    }

    /// Add samples to a buffer (mixing)
    pub fn add_to_buffer(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample += self.next_sample();
        }
    }
}

/// Extended string with sympathetic resonance
pub struct ResonantString {
    /// Primary string
    primary: KarplusStrong,
    /// Resonance buffer (captures harmonics from other strings)
    resonance_buffer: Vec<f32>,
    /// Resonance amount
    resonance: f32,
}

impl ResonantString {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            primary: KarplusStrong::new(sample_rate),
            resonance_buffer: vec![0.0; 4096],
            resonance: 0.02, // Subtle resonance
        }
    }

    pub fn pluck(&mut self, frequency: f32, velocity: f32) {
        self.primary.pluck_with_velocity(frequency, velocity);
    }

    pub fn excite_resonance(&mut self, sample: f32, frequency: f32) {
        // Add energy to resonance buffer at harmonic frequencies
        let period = (self.primary.sample_rate as f32 / frequency) as usize;
        if period > 0 && period < self.resonance_buffer.len() {
            let idx = period % self.resonance_buffer.len();
            self.resonance_buffer[idx] += sample * self.resonance;
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.primary.next_sample()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pluck() {
        let mut ks = KarplusStrong::new(44100);
        ks.pluck(440.0); // A4

        // Should be active after pluck
        assert!(ks.is_active());

        // Generate some samples
        let mut samples = vec![0.0; 1000];
        ks.fill_buffer(&mut samples);

        // Should have non-zero output
        let max = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }

    #[test]
    fn test_decay() {
        let mut ks = KarplusStrong::new(44100);
        ks.set_feedback(0.99); // Fast decay for testing
        ks.pluck(440.0);

        // Generate lots of samples until decay
        for _ in 0..100 {
            let mut buffer = vec![0.0; 4410]; // 0.1 seconds
            ks.fill_buffer(&mut buffer);
        }

        // Should have decayed to near silence
        assert!(!ks.is_active() || ks.next_sample().abs() < 0.001);
    }

    #[test]
    fn test_pitch_bend() {
        let mut ks = KarplusStrong::new(44100);
        ks.pluck(440.0); // A4
        ks.set_bend_rate(0.01); // Medium bend speed

        // Bend up a whole step (to B4)
        let b4_freq = 440.0 * 2.0_f32.powf(2.0 / 12.0);
        ks.bend_to(b4_freq);

        // Generate samples while bending
        let mut samples = vec![0.0; 4410];
        ks.fill_buffer(&mut samples);

        // Should still be active
        assert!(ks.is_active());
    }

    #[test]
    fn test_vibrato() {
        let mut ks = KarplusStrong::new(44100);
        ks.pluck(440.0);
        ks.set_vibrato(0.5, 5.0); // 0.5 semitone depth, 5 Hz rate

        // Generate samples with vibrato
        let mut samples = vec![0.0; 44100]; // 1 second
        ks.fill_buffer(&mut samples);

        // Should have output
        let max = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }

    #[test]
    fn test_slide() {
        let mut ks = KarplusStrong::new(44100);
        ks.pluck(440.0); // A4

        // Slide to E5 over 0.5 seconds
        let e5_freq = 440.0 * 2.0_f32.powf(7.0 / 12.0);
        ks.slide_to(e5_freq, 22050);

        // Generate samples during slide
        let mut samples = vec![0.0; 22050];
        ks.fill_buffer(&mut samples);

        // Should still be active
        assert!(ks.is_active());
    }
}
