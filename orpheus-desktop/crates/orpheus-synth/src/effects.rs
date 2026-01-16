//! Audio effects for metal tone shaping
//!
//! Includes distortion, saturation, and tone shaping for metal guitar tones.

/// Distortion type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistortionType {
    /// Soft clipping (tube-like warmth)
    Overdrive,
    /// Hard clipping (aggressive, djent-style)
    Distortion,
    /// Asymmetric clipping (modern high-gain)
    HighGain,
    /// Fuzz (extreme saturation)
    Fuzz,
}

/// Distortion/overdrive effect
pub struct Distortion {
    /// Distortion type
    dist_type: DistortionType,
    /// Input gain (drive) - 1.0 to 100.0
    drive: f32,
    /// Output level - 0.0 to 1.0
    level: f32,
    /// Tone control (0.0 = dark, 1.0 = bright)
    tone: f32,
    /// Mix (0.0 = dry, 1.0 = wet)
    mix: f32,
    /// Low-pass filter state
    lp_state: f32,
    /// High-pass filter state
    hp_state: f32,
}

impl Default for Distortion {
    fn default() -> Self {
        Self {
            dist_type: DistortionType::HighGain,
            drive: 20.0,
            level: 0.5,
            tone: 0.6,
            mix: 1.0,
            lp_state: 0.0,
            hp_state: 0.0,
        }
    }
}

impl Distortion {
    /// Create a new distortion effect
    pub fn new(dist_type: DistortionType) -> Self {
        Self {
            dist_type,
            ..Default::default()
        }
    }

    /// Create overdrive preset
    pub fn overdrive() -> Self {
        Self {
            dist_type: DistortionType::Overdrive,
            drive: 8.0,
            level: 0.6,
            tone: 0.5,
            mix: 1.0,
            ..Default::default()
        }
    }

    /// Create high-gain metal preset
    pub fn metal() -> Self {
        Self {
            dist_type: DistortionType::HighGain,
            drive: 40.0,
            level: 0.45,
            tone: 0.65,
            mix: 1.0,
            ..Default::default()
        }
    }

    /// Create djent/modern metal preset
    pub fn djent() -> Self {
        Self {
            dist_type: DistortionType::HighGain,
            drive: 50.0,
            level: 0.4,
            tone: 0.7,  // Brighter for clarity
            mix: 1.0,
            ..Default::default()
        }
    }

    /// Create technical death metal preset
    pub fn tech_death() -> Self {
        Self {
            dist_type: DistortionType::HighGain,
            drive: 35.0,
            level: 0.5,
            tone: 0.6,
            mix: 1.0,
            ..Default::default()
        }
    }

    /// Set drive amount (1.0 - 100.0)
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(1.0, 100.0);
    }

    /// Set output level (0.0 - 1.0)
    pub fn set_level(&mut self, level: f32) {
        self.level = level.clamp(0.0, 1.0);
    }

    /// Set tone (0.0 = dark, 1.0 = bright)
    pub fn set_tone(&mut self, tone: f32) {
        self.tone = tone.clamp(0.0, 1.0);
    }

    /// Set wet/dry mix
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Process a single sample
    pub fn process(&mut self, input: f32) -> f32 {
        // Apply input gain
        let driven = input * self.drive;

        // Apply distortion based on type
        let distorted = match self.dist_type {
            DistortionType::Overdrive => self.soft_clip(driven),
            DistortionType::Distortion => self.hard_clip(driven),
            DistortionType::HighGain => self.high_gain_clip(driven),
            DistortionType::Fuzz => self.fuzz_clip(driven),
        };

        // Apply tone control (simple one-pole lowpass)
        let cutoff = 0.1 + self.tone * 0.8; // 0.1 to 0.9
        self.lp_state = self.lp_state + cutoff * (distorted - self.lp_state);

        // High-pass to remove DC offset and mud
        let hp_cutoff = 0.01;
        self.hp_state = self.hp_state + hp_cutoff * (self.lp_state - self.hp_state);
        let filtered = self.lp_state - self.hp_state;

        // Apply output level and mix
        let wet = filtered * self.level;
        input * (1.0 - self.mix) + wet * self.mix
    }

    /// Process a buffer of samples
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    /// Process stereo buffer (interleaved L, R, L, R, ...)
    pub fn process_stereo(&mut self, buffer: &mut [f32]) {
        // Process each channel
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    // Clipping functions

    /// Soft clipping using tanh (tube-like)
    fn soft_clip(&self, x: f32) -> f32 {
        x.tanh()
    }

    /// Hard clipping
    fn hard_clip(&self, x: f32) -> f32 {
        x.clamp(-1.0, 1.0)
    }

    /// High-gain asymmetric clipping
    fn high_gain_clip(&self, x: f32) -> f32 {
        // Asymmetric soft clipping with more harmonics
        if x >= 0.0 {
            1.0 - (-x).exp()
        } else {
            -1.0 + x.exp()
        }
    }

    /// Fuzz distortion (extreme)
    fn fuzz_clip(&self, x: f32) -> f32 {
        // Square wave-ish clipping
        let sign = if x >= 0.0 { 1.0 } else { -1.0 };
        let abs_x = x.abs();

        if abs_x < 0.5 {
            sign * abs_x * 2.0
        } else {
            sign * (1.0 - 0.25 / abs_x)
        }
    }
}

/// Simple noise gate for tightening palm mutes
pub struct NoiseGate {
    /// Threshold (0.0 - 1.0)
    threshold: f32,
    /// Attack time in samples
    attack: usize,
    /// Release time in samples
    release: usize,
    /// Current envelope
    envelope: f32,
    /// Gate state (0.0 = closed, 1.0 = open)
    gate: f32,
    /// Sample counter for timing
    counter: usize,
}

impl Default for NoiseGate {
    fn default() -> Self {
        Self {
            threshold: 0.01,
            attack: 10,
            release: 100,
            envelope: 0.0,
            gate: 0.0,
            counter: 0,
        }
    }
}

impl NoiseGate {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            ..Default::default()
        }
    }

    /// Tight gate for metal (fast attack/release)
    pub fn tight() -> Self {
        Self {
            threshold: 0.02,
            attack: 5,
            release: 50,
            ..Default::default()
        }
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        // Update envelope follower
        let abs_input = input.abs();
        if abs_input > self.envelope {
            self.envelope = abs_input;
        } else {
            self.envelope *= 0.9995; // Slow decay
        }

        // Gate logic
        if self.envelope > self.threshold {
            // Open gate
            if self.gate < 1.0 {
                self.gate = (self.gate + 1.0 / self.attack as f32).min(1.0);
            }
        } else {
            // Close gate
            if self.gate > 0.0 {
                self.gate = (self.gate - 1.0 / self.release as f32).max(0.0);
            }
        }

        input * self.gate
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

/// Simple 3-band EQ for tone shaping
pub struct ThreeBandEq {
    /// Low frequency gain (bass)
    low_gain: f32,
    /// Mid frequency gain
    mid_gain: f32,
    /// High frequency gain (treble)
    high_gain: f32,
    /// Low band filter state
    low_state: f32,
    /// High band filter state
    high_state: f32,
}

impl Default for ThreeBandEq {
    fn default() -> Self {
        Self {
            low_gain: 1.0,
            mid_gain: 1.0,
            high_gain: 1.0,
            low_state: 0.0,
            high_state: 0.0,
        }
    }
}

impl ThreeBandEq {
    /// Metal EQ preset (scooped mids)
    pub fn metal_scoop() -> Self {
        Self {
            low_gain: 1.3,
            mid_gain: 0.7,
            high_gain: 1.2,
            ..Default::default()
        }
    }

    /// Modern metal EQ (tight lows, present mids)
    pub fn modern_metal() -> Self {
        Self {
            low_gain: 1.1,
            mid_gain: 1.1,
            high_gain: 1.15,
            ..Default::default()
        }
    }

    pub fn set_low(&mut self, gain: f32) {
        self.low_gain = gain.clamp(0.0, 2.0);
    }

    pub fn set_mid(&mut self, gain: f32) {
        self.mid_gain = gain.clamp(0.0, 2.0);
    }

    pub fn set_high(&mut self, gain: f32) {
        self.high_gain = gain.clamp(0.0, 2.0);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        // Simple crossover using one-pole filters
        let low_cutoff = 0.05;
        let high_cutoff = 0.3;

        // Extract low frequencies
        self.low_state = self.low_state + low_cutoff * (input - self.low_state);
        let low = self.low_state;

        // Extract high frequencies
        self.high_state = self.high_state + high_cutoff * (input - self.high_state);
        let high = input - self.high_state;

        // Mid is what's left
        let mid = input - low - high;

        // Apply gains and sum
        low * self.low_gain + mid * self.mid_gain + high * self.high_gain
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

/// Delay effect with feedback
pub struct Delay {
    /// Delay buffer (circular)
    buffer: Vec<f32>,
    /// Write position
    write_pos: usize,
    /// Delay time in samples
    delay_samples: usize,
    /// Feedback amount (0.0 - 1.0)
    feedback: f32,
    /// Wet/dry mix (0.0 - 1.0)
    mix: f32,
    /// Sample rate
    sample_rate: u32,
    /// Low-pass filter on feedback (darkens repeats)
    feedback_lp_state: f32,
    /// High-cut coefficient
    high_cut: f32,
}

impl Delay {
    /// Create a new delay effect
    pub fn new(sample_rate: u32) -> Self {
        // Max 2 seconds delay
        let max_delay = sample_rate as usize * 2;
        Self {
            buffer: vec![0.0; max_delay],
            write_pos: 0,
            delay_samples: (sample_rate as f32 * 0.3) as usize, // 300ms default
            feedback: 0.4,
            mix: 0.3,
            sample_rate,
            feedback_lp_state: 0.0,
            high_cut: 0.5, // Moderate darkening
        }
    }

    /// Set delay time in milliseconds
    pub fn set_delay_ms(&mut self, ms: f32) {
        let samples = (self.sample_rate as f32 * ms / 1000.0) as usize;
        self.delay_samples = samples.min(self.buffer.len() - 1).max(1);
    }

    /// Set delay time synced to tempo
    pub fn set_delay_tempo(&mut self, tempo: f64, note_division: f32) {
        // note_division: 1.0 = quarter, 0.5 = eighth, 0.25 = sixteenth
        let beat_duration_ms = 60000.0 / tempo;
        let delay_ms = (beat_duration_ms * note_division as f64) as f32;
        self.set_delay_ms(delay_ms);
    }

    /// Set feedback amount (0.0 - 1.0)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.95); // Prevent runaway
    }

    /// Set wet/dry mix (0.0 - 1.0)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Set high-cut filter amount (0.0 = bright, 1.0 = dark)
    pub fn set_high_cut(&mut self, amount: f32) {
        self.high_cut = amount.clamp(0.0, 0.95);
    }

    /// Process a single sample
    pub fn process(&mut self, input: f32) -> f32 {
        // Read from delay buffer
        let read_pos = (self.write_pos + self.buffer.len() - self.delay_samples) % self.buffer.len();
        let delayed = self.buffer[read_pos];

        // Apply high-cut filter to feedback (darken repeats)
        self.feedback_lp_state = self.feedback_lp_state + self.high_cut * (delayed - self.feedback_lp_state);
        let filtered_delayed = if self.high_cut > 0.0 {
            self.feedback_lp_state
        } else {
            delayed
        };

        // Write input + filtered feedback to buffer
        self.buffer[self.write_pos] = input + filtered_delayed * self.feedback;

        // Advance write position
        self.write_pos = (self.write_pos + 1) % self.buffer.len();

        // Mix dry and wet
        input * (1.0 - self.mix) + delayed * self.mix
    }

    /// Reset delay buffer
    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.feedback_lp_state = 0.0;
    }

    /// Slap-back preset (short, low feedback)
    pub fn slap_back(sample_rate: u32) -> Self {
        let mut delay = Self::new(sample_rate);
        delay.set_delay_ms(80.0);
        delay.set_feedback(0.2);
        delay.set_mix(0.25);
        delay.set_high_cut(0.3);
        delay
    }

    /// Analog-style tape delay preset
    pub fn tape_delay(sample_rate: u32) -> Self {
        let mut delay = Self::new(sample_rate);
        delay.set_delay_ms(350.0);
        delay.set_feedback(0.45);
        delay.set_mix(0.3);
        delay.set_high_cut(0.6); // Darker repeats
        delay
    }

    /// Clean digital delay preset
    pub fn clean_delay(sample_rate: u32) -> Self {
        let mut delay = Self::new(sample_rate);
        delay.set_delay_ms(300.0);
        delay.set_feedback(0.4);
        delay.set_mix(0.35);
        delay.set_high_cut(0.1); // Bright repeats
        delay
    }
}

/// Stereo delay with ping-pong option
pub struct StereoDelay {
    /// Left channel delay
    left: Delay,
    /// Right channel delay
    right: Delay,
    /// Ping-pong mode (alternates between channels)
    ping_pong: bool,
    /// Cross-feed amount for ping-pong
    cross_feed: f32,
}

impl StereoDelay {
    /// Create a new stereo delay
    pub fn new(sample_rate: u32) -> Self {
        Self {
            left: Delay::new(sample_rate),
            right: Delay::new(sample_rate),
            ping_pong: false,
            cross_feed: 0.0,
        }
    }

    /// Enable ping-pong mode
    pub fn set_ping_pong(&mut self, enabled: bool) {
        self.ping_pong = enabled;
        if enabled {
            // Offset right channel for ping-pong effect
            let left_delay = self.left.delay_samples;
            self.right.delay_samples = left_delay / 2;
            self.cross_feed = 0.7;
        } else {
            self.cross_feed = 0.0;
        }
    }

    /// Set delay time in milliseconds (both channels)
    pub fn set_delay_ms(&mut self, ms: f32) {
        self.left.set_delay_ms(ms);
        if self.ping_pong {
            self.right.set_delay_ms(ms / 2.0);
        } else {
            self.right.set_delay_ms(ms);
        }
    }

    /// Set feedback amount
    pub fn set_feedback(&mut self, feedback: f32) {
        self.left.set_feedback(feedback);
        self.right.set_feedback(feedback);
    }

    /// Set wet/dry mix
    pub fn set_mix(&mut self, mix: f32) {
        self.left.set_mix(mix);
        self.right.set_mix(mix);
    }

    /// Process stereo samples
    pub fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        if self.ping_pong {
            // Cross-feed for ping-pong
            let left_out = self.left.process(left_in);

            // Feed left delay output to right for ping-pong effect
            let right_out = self.right.process(right_in + left_out * self.cross_feed);

            (left_out, right_out)
        } else {
            let left_out = self.left.process(left_in);
            let right_out = self.right.process(right_in);
            (left_out, right_out)
        }
    }

    /// Reset both channels
    pub fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
    }
}

/// Comb filter for reverb
struct CombFilter {
    buffer: Vec<f32>,
    write_pos: usize,
    feedback: f32,
    damp: f32,
    damp_state: f32,
}

impl CombFilter {
    fn new(delay_samples: usize, feedback: f32, damp: f32) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            write_pos: 0,
            feedback,
            damp,
            damp_state: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.write_pos];

        // Damping filter (low-pass on feedback)
        self.damp_state = output * (1.0 - self.damp) + self.damp_state * self.damp;

        // Write input + filtered feedback
        self.buffer[self.write_pos] = input + self.damp_state * self.feedback;

        // Advance position
        self.write_pos = (self.write_pos + 1) % self.buffer.len();

        output
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.damp_state = 0.0;
    }
}

/// All-pass filter for reverb diffusion
struct AllPassFilter {
    buffer: Vec<f32>,
    write_pos: usize,
    feedback: f32,
}

impl AllPassFilter {
    fn new(delay_samples: usize, feedback: f32) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            write_pos: 0,
            feedback,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buffered = self.buffer[self.write_pos];
        let output = buffered - input;

        self.buffer[self.write_pos] = input + buffered * self.feedback;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();

        output
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
    }
}

/// Schroeder/Freeverb-style algorithmic reverb
pub struct Reverb {
    /// Comb filters (parallel)
    combs_l: Vec<CombFilter>,
    combs_r: Vec<CombFilter>,
    /// All-pass filters (series)
    allpasses_l: Vec<AllPassFilter>,
    allpasses_r: Vec<AllPassFilter>,
    /// Room size (affects comb feedback)
    room_size: f32,
    /// Damping (high frequency absorption)
    damping: f32,
    /// Wet/dry mix
    mix: f32,
    /// Stereo width
    width: f32,
    /// Pre-delay in samples
    predelay_buffer: Vec<f32>,
    predelay_pos: usize,
    predelay_samples: usize,
}

impl Reverb {
    /// Comb filter delays (in samples at 44100Hz)
    const COMB_TUNINGS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
    /// All-pass filter delays
    const ALLPASS_TUNINGS: [usize; 4] = [556, 441, 341, 225];
    /// Stereo spread
    const STEREO_SPREAD: usize = 23;

    /// Create a new reverb effect
    pub fn new(sample_rate: u32) -> Self {
        let scale = sample_rate as f32 / 44100.0;

        // Create comb filters with scaled delays
        let combs_l: Vec<CombFilter> = Self::COMB_TUNINGS.iter()
            .map(|&d| CombFilter::new((d as f32 * scale) as usize, 0.84, 0.2))
            .collect();

        let combs_r: Vec<CombFilter> = Self::COMB_TUNINGS.iter()
            .map(|&d| CombFilter::new(((d + Self::STEREO_SPREAD) as f32 * scale) as usize, 0.84, 0.2))
            .collect();

        // Create all-pass filters
        let allpasses_l: Vec<AllPassFilter> = Self::ALLPASS_TUNINGS.iter()
            .map(|&d| AllPassFilter::new((d as f32 * scale) as usize, 0.5))
            .collect();

        let allpasses_r: Vec<AllPassFilter> = Self::ALLPASS_TUNINGS.iter()
            .map(|&d| AllPassFilter::new(((d + Self::STEREO_SPREAD) as f32 * scale) as usize, 0.5))
            .collect();

        // Pre-delay buffer (up to 100ms)
        let predelay_max = (sample_rate as f32 * 0.1) as usize;

        Self {
            combs_l,
            combs_r,
            allpasses_l,
            allpasses_r,
            room_size: 0.5,
            damping: 0.5,
            mix: 0.3,
            width: 1.0,
            predelay_buffer: vec![0.0; predelay_max],
            predelay_pos: 0,
            predelay_samples: 0,
        }
    }

    /// Set room size (0.0 - 1.0)
    pub fn set_room_size(&mut self, size: f32) {
        self.room_size = size.clamp(0.0, 1.0);

        // Update comb filter feedback
        let feedback = 0.28 + size * 0.7; // 0.28 to 0.98
        for comb in &mut self.combs_l {
            comb.feedback = feedback;
        }
        for comb in &mut self.combs_r {
            comb.feedback = feedback;
        }
    }

    /// Set damping (0.0 = bright, 1.0 = dark)
    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping.clamp(0.0, 1.0);

        for comb in &mut self.combs_l {
            comb.damp = damping;
        }
        for comb in &mut self.combs_r {
            comb.damp = damping;
        }
    }

    /// Set wet/dry mix (0.0 - 1.0)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Set stereo width (0.0 = mono, 1.0 = full stereo)
    pub fn set_width(&mut self, width: f32) {
        self.width = width.clamp(0.0, 1.0);
    }

    /// Set pre-delay in milliseconds
    pub fn set_predelay_ms(&mut self, ms: f32, sample_rate: u32) {
        let samples = (sample_rate as f32 * ms / 1000.0) as usize;
        self.predelay_samples = samples.min(self.predelay_buffer.len() - 1);
    }

    /// Process stereo samples
    pub fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        // Apply pre-delay
        let predelayed = if self.predelay_samples > 0 {
            let read_pos = (self.predelay_pos + self.predelay_buffer.len() - self.predelay_samples)
                % self.predelay_buffer.len();
            let out = self.predelay_buffer[read_pos];
            self.predelay_buffer[self.predelay_pos] = (left_in + right_in) * 0.5;
            self.predelay_pos = (self.predelay_pos + 1) % self.predelay_buffer.len();
            out
        } else {
            (left_in + right_in) * 0.5
        };

        // Process through parallel comb filters
        let mut out_l = 0.0;
        let mut out_r = 0.0;

        for comb in &mut self.combs_l {
            out_l += comb.process(predelayed);
        }
        for comb in &mut self.combs_r {
            out_r += comb.process(predelayed);
        }

        // Process through series all-pass filters
        for allpass in &mut self.allpasses_l {
            out_l = allpass.process(out_l);
        }
        for allpass in &mut self.allpasses_r {
            out_r = allpass.process(out_r);
        }

        // Apply width
        let wet_l = out_l + out_r * (1.0 - self.width);
        let wet_r = out_r + out_l * (1.0 - self.width);

        // Scale wet signal
        let wet_l = wet_l * 0.015; // Reduce level
        let wet_r = wet_r * 0.015;

        // Mix dry and wet
        let mix_l = left_in * (1.0 - self.mix) + wet_l * self.mix;
        let mix_r = right_in * (1.0 - self.mix) + wet_r * self.mix;

        (mix_l, mix_r)
    }

    /// Process mono sample (returns mono)
    pub fn process_mono(&mut self, input: f32) -> f32 {
        let (l, r) = self.process(input, input);
        (l + r) * 0.5
    }

    /// Reset all buffers
    pub fn reset(&mut self) {
        for comb in &mut self.combs_l {
            comb.reset();
        }
        for comb in &mut self.combs_r {
            comb.reset();
        }
        for allpass in &mut self.allpasses_l {
            allpass.reset();
        }
        for allpass in &mut self.allpasses_r {
            allpass.reset();
        }
        self.predelay_buffer.fill(0.0);
    }

    /// Small room preset (tight, short decay)
    pub fn small_room(sample_rate: u32) -> Self {
        let mut reverb = Self::new(sample_rate);
        reverb.set_room_size(0.3);
        reverb.set_damping(0.6);
        reverb.set_mix(0.2);
        reverb.set_width(0.8);
        reverb
    }

    /// Medium room preset
    pub fn medium_room(sample_rate: u32) -> Self {
        let mut reverb = Self::new(sample_rate);
        reverb.set_room_size(0.5);
        reverb.set_damping(0.5);
        reverb.set_mix(0.25);
        reverb.set_width(1.0);
        reverb
    }

    /// Large hall preset
    pub fn hall(sample_rate: u32) -> Self {
        let mut reverb = Self::new(sample_rate);
        reverb.set_room_size(0.8);
        reverb.set_damping(0.3);
        reverb.set_mix(0.35);
        reverb.set_width(1.0);
        reverb.set_predelay_ms(25.0, sample_rate);
        reverb
    }

    /// Cathedral/ambient preset
    pub fn cathedral(sample_rate: u32) -> Self {
        let mut reverb = Self::new(sample_rate);
        reverb.set_room_size(0.95);
        reverb.set_damping(0.2);
        reverb.set_mix(0.4);
        reverb.set_width(1.0);
        reverb.set_predelay_ms(40.0, sample_rate);
        reverb
    }

    /// Plate reverb emulation
    pub fn plate(sample_rate: u32) -> Self {
        let mut reverb = Self::new(sample_rate);
        reverb.set_room_size(0.6);
        reverb.set_damping(0.1); // Bright
        reverb.set_mix(0.3);
        reverb.set_width(1.0);
        reverb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distortion() {
        let mut dist = Distortion::metal();

        // Test soft signal
        let output = dist.process(0.1);
        assert!(output.abs() < 1.0);

        // Test hard signal (should be clipped)
        let mut loud_dist = Distortion::metal();
        loud_dist.set_drive(80.0);
        let output = loud_dist.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_noise_gate() {
        let mut gate = NoiseGate::tight();

        // Below threshold should be gated
        for _ in 0..100 {
            let output = gate.process(0.001);
            // Gate should close
            assert!(output.abs() < 0.01);
        }
    }

    #[test]
    fn test_eq() {
        let mut eq = ThreeBandEq::metal_scoop();

        let input = 0.5;
        let output = eq.process(input);

        // Should have some output
        assert!(output != 0.0);
    }

    #[test]
    fn test_delay() {
        let mut delay = Delay::new(44100);
        delay.set_delay_ms(100.0);
        delay.set_feedback(0.5);
        delay.set_mix(0.5);

        // Process an impulse
        let mut has_delayed = false;
        for i in 0..10000 {
            let input = if i == 0 { 1.0 } else { 0.0 };
            let output = delay.process(input);

            // After delay time, we should hear the delayed signal
            if i > 4000 && output.abs() > 0.1 {
                has_delayed = true;
            }
        }

        assert!(has_delayed, "Delay should produce delayed output");
    }

    #[test]
    fn test_stereo_delay() {
        let mut delay = StereoDelay::new(44100);
        delay.set_delay_ms(200.0);
        delay.set_feedback(0.3);
        delay.set_mix(0.4);

        let (left, right) = delay.process(0.5, 0.5);
        // Initial output should be mostly dry
        assert!((left - 0.3).abs() < 0.2); // 0.5 * (1-0.4) = 0.3
    }

    #[test]
    fn test_reverb() {
        let mut reverb = Reverb::medium_room(44100);

        // Process an impulse
        let mut has_reverb = false;
        for i in 0..44100 {
            let input = if i == 0 { 1.0 } else { 0.0 };
            let (left, right) = reverb.process(input, input);

            // After initial impulse, reverb should continue
            if i > 1000 && (left.abs() > 0.0001 || right.abs() > 0.0001) {
                has_reverb = true;
                break;
            }
        }

        assert!(has_reverb, "Reverb should produce decay tail");
    }

    #[test]
    fn test_reverb_presets() {
        // Just verify presets don't panic
        let _ = Reverb::small_room(44100);
        let _ = Reverb::medium_room(44100);
        let _ = Reverb::hall(44100);
        let _ = Reverb::cathedral(44100);
        let _ = Reverb::plate(44100);
    }

    #[test]
    fn test_delay_presets() {
        // Verify delay presets work
        let mut slap = Delay::slap_back(44100);
        let _ = slap.process(0.5);

        let mut tape = Delay::tape_delay(44100);
        let _ = tape.process(0.5);

        let mut clean = Delay::clean_delay(44100);
        let _ = clean.process(0.5);
    }
}
