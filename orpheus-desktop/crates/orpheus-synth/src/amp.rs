//! Guitar Amplifier Simulation
//!
//! Models tube amplifier characteristics including:
//! - Preamp tube stages with asymmetric clipping
//! - Passive tonestack EQ (Fender/Marshall/Mesa voicings)
//! - Power amp compression and sag
//! - Output transformer coloration

use std::f32::consts::PI;

// ============================================================================
// Tube Stage - Models a single triode tube
// ============================================================================

/// A single triode tube stage with asymmetric soft clipping
#[derive(Debug, Clone)]
pub struct TubeStage {
    /// Input gain (drive)
    drive: f32,
    /// Bias point (-1.0 to 1.0, affects asymmetry)
    bias: f32,
    /// Output level
    level: f32,
    /// DC blocking highpass state
    dc_block_state: f32,
    /// DC blocking coefficient
    dc_block_coeff: f32,
    /// Sample rate
    sample_rate: f32,
}

impl TubeStage {
    pub fn new(sample_rate: f32) -> Self {
        // DC blocking at ~10Hz
        let dc_block_coeff = 1.0 - (2.0 * PI * 10.0 / sample_rate);

        Self {
            drive: 1.0,
            bias: 0.0,
            level: 1.0,
            dc_block_state: 0.0,
            dc_block_coeff,
            sample_rate,
        }
    }

    /// Set drive amount (1.0 = unity, higher = more saturation)
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.max(0.1);
    }

    /// Set bias point for asymmetric clipping
    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias.clamp(-0.5, 0.5);
    }

    /// Set output level
    pub fn set_level(&mut self, level: f32) {
        self.level = level.clamp(0.0, 2.0);
    }

    /// Process a single sample through the tube stage
    pub fn process(&mut self, input: f32) -> f32 {
        // Apply drive and bias
        let x = input * self.drive + self.bias;

        // Asymmetric tube saturation curve
        // Positive half: soft compression (like tube grid limiting)
        // Negative half: harder clipping (like plate saturation)
        let saturated = if x >= 0.0 {
            // Soft saturation for positive swing
            tube_soft_clip(x)
        } else {
            // Slightly harder for negative swing (asymmetric)
            -tube_hard_clip(-x * 1.2) * 0.9
        };

        // DC blocking highpass filter
        let output = saturated - self.dc_block_state;
        self.dc_block_state = saturated - output * self.dc_block_coeff;

        output * self.level
    }

    /// Reset internal state
    pub fn reset(&mut self) {
        self.dc_block_state = 0.0;
    }
}

/// Soft tube-like saturation (tanh approximation with harmonics)
fn tube_soft_clip(x: f32) -> f32 {
    // Modified tanh with even harmonic content
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

/// Harder clipping for negative swing
fn tube_hard_clip(x: f32) -> f32 {
    // Faster saturation curve
    if x < 1.0 {
        x - x * x * x / 3.0
    } else {
        2.0 / 3.0
    }
}

// ============================================================================
// Tonestack - Passive EQ modeling classic amp circuits
// ============================================================================

/// Tonestack voicing presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TonestackVoicing {
    /// Fender-style (scooped mids, bright)
    Fender,
    /// Marshall-style (mid-forward, crunchy)
    Marshall,
    /// Mesa/Boogie style (tight lows, aggressive mids)
    Mesa,
    /// Modern high-gain (extended lows, presence peak)
    Modern,
    /// Bypass (flat response)
    Bypass,
}

impl Default for TonestackVoicing {
    fn default() -> Self {
        Self::Marshall
    }
}

/// Passive tonestack EQ
/// Models the classic bass-mid-treble passive circuit found in tube amps
#[derive(Debug, Clone)]
pub struct Tonestack {
    /// Bass control (0.0 - 1.0)
    bass: f32,
    /// Mid control (0.0 - 1.0)
    mid: f32,
    /// Treble control (0.0 - 1.0)
    treble: f32,
    /// Current voicing
    voicing: TonestackVoicing,
    /// Sample rate
    sample_rate: f32,

    // Filter states (biquad sections)
    low_state: [f32; 2],
    mid_state: [f32; 2],
    high_state: [f32; 2],

    // Cached coefficients
    low_coeffs: BiquadCoeffs,
    mid_coeffs: BiquadCoeffs,
    high_coeffs: BiquadCoeffs,
}

#[derive(Debug, Clone, Copy, Default)]
struct BiquadCoeffs {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl Tonestack {
    pub fn new(sample_rate: f32) -> Self {
        let mut ts = Self {
            bass: 0.5,
            mid: 0.5,
            treble: 0.5,
            voicing: TonestackVoicing::default(),
            sample_rate,
            low_state: [0.0; 2],
            mid_state: [0.0; 2],
            high_state: [0.0; 2],
            low_coeffs: BiquadCoeffs::default(),
            mid_coeffs: BiquadCoeffs::default(),
            high_coeffs: BiquadCoeffs::default(),
        };
        ts.update_coefficients();
        ts
    }

    /// Set bass control (0.0 - 1.0)
    pub fn set_bass(&mut self, bass: f32) {
        self.bass = bass.clamp(0.0, 1.0);
        self.update_coefficients();
    }

    /// Set mid control (0.0 - 1.0)
    pub fn set_mid(&mut self, mid: f32) {
        self.mid = mid.clamp(0.0, 1.0);
        self.update_coefficients();
    }

    /// Set treble control (0.0 - 1.0)
    pub fn set_treble(&mut self, treble: f32) {
        self.treble = treble.clamp(0.0, 1.0);
        self.update_coefficients();
    }

    /// Set all controls at once
    pub fn set_controls(&mut self, bass: f32, mid: f32, treble: f32) {
        self.bass = bass.clamp(0.0, 1.0);
        self.mid = mid.clamp(0.0, 1.0);
        self.treble = treble.clamp(0.0, 1.0);
        self.update_coefficients();
    }

    /// Set voicing
    pub fn set_voicing(&mut self, voicing: TonestackVoicing) {
        self.voicing = voicing;
        self.update_coefficients();
    }

    /// Get voicing-specific frequency centers
    fn get_frequencies(&self) -> (f32, f32, f32) {
        match self.voicing {
            TonestackVoicing::Fender => (100.0, 800.0, 3000.0),
            TonestackVoicing::Marshall => (120.0, 650.0, 2500.0),
            TonestackVoicing::Mesa => (80.0, 500.0, 4000.0),
            TonestackVoicing::Modern => (60.0, 400.0, 5000.0),
            TonestackVoicing::Bypass => (100.0, 1000.0, 3000.0),
        }
    }

    /// Get voicing-specific Q values
    fn get_q_values(&self) -> (f32, f32, f32) {
        match self.voicing {
            TonestackVoicing::Fender => (0.7, 1.5, 0.8),
            TonestackVoicing::Marshall => (0.8, 2.0, 0.7),
            TonestackVoicing::Mesa => (1.0, 2.5, 0.6),
            TonestackVoicing::Modern => (1.2, 3.0, 0.5),
            TonestackVoicing::Bypass => (0.7, 0.7, 0.7),
        }
    }

    fn update_coefficients(&mut self) {
        if self.voicing == TonestackVoicing::Bypass {
            // Flat response
            self.low_coeffs = BiquadCoeffs { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0 };
            self.mid_coeffs = BiquadCoeffs { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0 };
            self.high_coeffs = BiquadCoeffs { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0 };
            return;
        }

        let (low_freq, mid_freq, high_freq) = self.get_frequencies();
        let (low_q, mid_q, high_q) = self.get_q_values();

        // Convert control positions to dB gain (-12 to +12 dB range)
        let bass_db = (self.bass - 0.5) * 24.0;
        let mid_db = (self.mid - 0.5) * 24.0;
        let treble_db = (self.treble - 0.5) * 24.0;

        // Calculate biquad coefficients for each band
        self.low_coeffs = calc_low_shelf(low_freq, low_q, bass_db, self.sample_rate);
        self.mid_coeffs = calc_peaking(mid_freq, mid_q, mid_db, self.sample_rate);
        self.high_coeffs = calc_high_shelf(high_freq, high_q, treble_db, self.sample_rate);
    }

    /// Process a single sample
    pub fn process(&mut self, input: f32) -> f32 {
        // Cascade three biquad filters (inline to avoid borrow conflicts)

        // Low shelf
        let low_out = {
            let c = &self.low_coeffs;
            let s = &mut self.low_state;
            let out = c.b0 * input + s[0];
            s[0] = c.b1 * input - c.a1 * out + s[1];
            s[1] = c.b2 * input - c.a2 * out;
            out
        };

        // Mid peak
        let mid_out = {
            let c = &self.mid_coeffs;
            let s = &mut self.mid_state;
            let out = c.b0 * low_out + s[0];
            s[0] = c.b1 * low_out - c.a1 * out + s[1];
            s[1] = c.b2 * low_out - c.a2 * out;
            out
        };

        // High shelf
        let high_out = {
            let c = &self.high_coeffs;
            let s = &mut self.high_state;
            let out = c.b0 * mid_out + s[0];
            s[0] = c.b1 * mid_out - c.a1 * out + s[1];
            s[1] = c.b2 * mid_out - c.a2 * out;
            out
        };

        high_out
    }

    /// Reset internal state
    pub fn reset(&mut self) {
        self.low_state = [0.0; 2];
        self.mid_state = [0.0; 2];
        self.high_state = [0.0; 2];
    }
}

// Biquad coefficient calculations
fn calc_low_shelf(freq: f32, q: f32, gain_db: f32, sample_rate: f32) -> BiquadCoeffs {
    let a = 10.0_f32.powf(gain_db / 40.0);
    let w0 = 2.0 * PI * freq / sample_rate;
    let cos_w0 = w0.cos();
    let sin_w0 = w0.sin();
    let alpha = sin_w0 / (2.0 * q);

    let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha;

    BiquadCoeffs {
        b0: (a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha)) / a0,
        b1: (2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0)) / a0,
        b2: (a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha)) / a0,
        a1: (-2.0 * ((a - 1.0) + (a + 1.0) * cos_w0)) / a0,
        a2: ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha) / a0,
    }
}

fn calc_high_shelf(freq: f32, q: f32, gain_db: f32, sample_rate: f32) -> BiquadCoeffs {
    let a = 10.0_f32.powf(gain_db / 40.0);
    let w0 = 2.0 * PI * freq / sample_rate;
    let cos_w0 = w0.cos();
    let sin_w0 = w0.sin();
    let alpha = sin_w0 / (2.0 * q);

    let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha;

    BiquadCoeffs {
        b0: (a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha)) / a0,
        b1: (-2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0)) / a0,
        b2: (a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha)) / a0,
        a1: (2.0 * ((a - 1.0) - (a + 1.0) * cos_w0)) / a0,
        a2: ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha) / a0,
    }
}

fn calc_peaking(freq: f32, q: f32, gain_db: f32, sample_rate: f32) -> BiquadCoeffs {
    let a = 10.0_f32.powf(gain_db / 40.0);
    let w0 = 2.0 * PI * freq / sample_rate;
    let cos_w0 = w0.cos();
    let sin_w0 = w0.sin();
    let alpha = sin_w0 / (2.0 * q);

    let a0 = 1.0 + alpha / a;

    BiquadCoeffs {
        b0: (1.0 + alpha * a) / a0,
        b1: (-2.0 * cos_w0) / a0,
        b2: (1.0 - alpha * a) / a0,
        a1: (-2.0 * cos_w0) / a0,
        a2: (1.0 - alpha / a) / a0,
    }
}

// ============================================================================
// Power Amp - Compression, sag, and output stage
// ============================================================================

/// Power amp simulation with compression and sag
#[derive(Debug, Clone)]
pub struct PowerAmp {
    /// Drive/gain into power section
    drive: f32,
    /// Sag amount (power supply droop)
    sag: f32,
    /// Presence control (negative feedback)
    presence: f32,
    /// Output level
    level: f32,
    /// Sample rate
    sample_rate: f32,

    // Envelope follower for sag
    envelope: f32,
    envelope_attack: f32,
    envelope_release: f32,

    // Presence filter state
    presence_state: [f32; 2],
    presence_coeffs: BiquadCoeffs,

    // Output transformer lowpass state
    transformer_state: f32,
    transformer_coeff: f32,
}

impl PowerAmp {
    pub fn new(sample_rate: f32) -> Self {
        // Presence peak around 4-5kHz
        let presence_coeffs = calc_high_shelf(4500.0, 0.7, 0.0, sample_rate);

        // Output transformer rolls off around 8kHz
        let transformer_coeff = (-2.0 * PI * 8000.0 / sample_rate).exp();

        // Envelope times
        let envelope_attack = (-2.0 * PI * 50.0 / sample_rate).exp();   // ~3ms attack
        let envelope_release = (-2.0 * PI * 5.0 / sample_rate).exp();   // ~30ms release

        Self {
            drive: 1.0,
            sag: 0.3,
            presence: 0.5,
            level: 1.0,
            sample_rate,
            envelope: 0.0,
            envelope_attack,
            envelope_release,
            presence_state: [0.0; 2],
            presence_coeffs,
            transformer_state: 0.0,
            transformer_coeff,
        }
    }

    /// Set drive into power section
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(0.5, 10.0);
    }

    /// Set sag amount (0.0 = none, 1.0 = maximum)
    pub fn set_sag(&mut self, sag: f32) {
        self.sag = sag.clamp(0.0, 1.0);
    }

    /// Set presence (negative feedback control)
    pub fn set_presence(&mut self, presence: f32) {
        self.presence = presence.clamp(0.0, 1.0);
        let gain_db = (presence - 0.5) * 12.0;
        self.presence_coeffs = calc_high_shelf(4500.0, 0.7, gain_db, self.sample_rate);
    }

    /// Set output level
    pub fn set_level(&mut self, level: f32) {
        self.level = level.clamp(0.0, 2.0);
    }

    /// Process a single sample
    pub fn process(&mut self, input: f32) -> f32 {
        // Apply drive
        let driven = input * self.drive;

        // Envelope follower for sag
        let abs_input = driven.abs();
        if abs_input > self.envelope {
            self.envelope = self.envelope_attack * self.envelope + (1.0 - self.envelope_attack) * abs_input;
        } else {
            self.envelope = self.envelope_release * self.envelope + (1.0 - self.envelope_release) * abs_input;
        }

        // Calculate sag reduction (power supply droop under load)
        let sag_amount = 1.0 - self.sag * (self.envelope * 0.5).min(0.3);

        // Apply sag and push-pull tube compression
        let sagged = driven * sag_amount;
        let compressed = power_tube_clip(sagged);

        // Presence filter (negative feedback adjustment)
        let with_presence = self.process_presence(compressed);

        // Output transformer lowpass (smooths harsh frequencies)
        self.transformer_state = self.transformer_coeff * self.transformer_state
            + (1.0 - self.transformer_coeff) * with_presence;

        self.transformer_state * self.level
    }

    fn process_presence(&mut self, input: f32) -> f32 {
        // Direct Form II Transposed biquad
        let coeffs = &self.presence_coeffs;
        let state = &mut self.presence_state;

        let output = coeffs.b0 * input + state[0];
        state[0] = coeffs.b1 * input - coeffs.a1 * output + state[1];
        state[1] = coeffs.b2 * input - coeffs.a2 * output;
        output
    }

    /// Reset internal state
    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.presence_state = [0.0; 2];
        self.transformer_state = 0.0;
    }
}

/// Push-pull power tube clipping (symmetric, rounded)
fn power_tube_clip(x: f32) -> f32 {
    // Soft symmetric clipping with gradual onset
    let x_abs = x.abs();
    if x_abs < 0.5 {
        x
    } else if x_abs < 1.0 {
        // Gradual compression zone
        x.signum() * (0.5 + (x_abs - 0.5) * 0.8)
    } else {
        // Soft clip zone
        x.signum() * (0.9 + 0.1 * (1.0 - (-3.0 * (x_abs - 1.0)).exp()))
    }
}

// ============================================================================
// Complete Amp Simulator
// ============================================================================

/// Amp channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmpChannel {
    /// Clean channel (minimal gain stages)
    Clean,
    /// Crunch channel (moderate gain)
    Crunch,
    /// High gain channel (multiple cascaded stages)
    HighGain,
    /// Lead channel (maximum saturation)
    Lead,
}

impl Default for AmpChannel {
    fn default() -> Self {
        Self::Crunch
    }
}

/// Complete guitar amplifier simulator
#[derive(Debug, Clone)]
pub struct AmpSimulator {
    /// Current channel
    channel: AmpChannel,
    /// Preamp gain (master input gain)
    gain: f32,
    /// Preamp tube stages
    stages: Vec<TubeStage>,
    /// Tonestack EQ
    tonestack: Tonestack,
    /// Power amp section
    power_amp: PowerAmp,
    /// Master volume
    master: f32,
    /// Sample rate
    sample_rate: f32,
}

impl AmpSimulator {
    /// Create a new amp simulator
    pub fn new(sample_rate: f32) -> Self {
        let mut amp = Self {
            channel: AmpChannel::default(),
            gain: 0.5,
            stages: Vec::new(),
            tonestack: Tonestack::new(sample_rate),
            power_amp: PowerAmp::new(sample_rate),
            master: 0.7,
            sample_rate,
        };
        amp.setup_channel(AmpChannel::Crunch);
        amp
    }

    /// Set channel and configure gain stages
    pub fn set_channel(&mut self, channel: AmpChannel) {
        self.channel = channel;
        self.setup_channel(channel);
    }

    fn setup_channel(&mut self, channel: AmpChannel) {
        self.stages.clear();

        match channel {
            AmpChannel::Clean => {
                // Single stage, low gain
                let mut stage = TubeStage::new(self.sample_rate);
                stage.set_drive(1.5);
                stage.set_level(0.8);
                self.stages.push(stage);
            }
            AmpChannel::Crunch => {
                // Two stages, moderate gain
                let mut stage1 = TubeStage::new(self.sample_rate);
                stage1.set_drive(3.0);
                stage1.set_bias(0.1);
                stage1.set_level(0.7);
                self.stages.push(stage1);

                let mut stage2 = TubeStage::new(self.sample_rate);
                stage2.set_drive(2.0);
                stage2.set_level(0.8);
                self.stages.push(stage2);
            }
            AmpChannel::HighGain => {
                // Three stages, high gain
                let mut stage1 = TubeStage::new(self.sample_rate);
                stage1.set_drive(5.0);
                stage1.set_bias(0.15);
                stage1.set_level(0.6);
                self.stages.push(stage1);

                let mut stage2 = TubeStage::new(self.sample_rate);
                stage2.set_drive(4.0);
                stage2.set_bias(0.1);
                stage2.set_level(0.6);
                self.stages.push(stage2);

                let mut stage3 = TubeStage::new(self.sample_rate);
                stage3.set_drive(3.0);
                stage3.set_level(0.7);
                self.stages.push(stage3);
            }
            AmpChannel::Lead => {
                // Four stages, maximum saturation
                for i in 0..4 {
                    let mut stage = TubeStage::new(self.sample_rate);
                    let drive = 6.0 - i as f32 * 0.5;
                    let bias = 0.2 - i as f32 * 0.03;
                    stage.set_drive(drive);
                    stage.set_bias(bias);
                    stage.set_level(0.55);
                    self.stages.push(stage);
                }
            }
        }
    }

    /// Set preamp gain (0.0 - 1.0)
    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 1.0);
    }

    /// Set tonestack controls
    pub fn set_eq(&mut self, bass: f32, mid: f32, treble: f32) {
        self.tonestack.set_controls(bass, mid, treble);
    }

    /// Set tonestack voicing
    pub fn set_voicing(&mut self, voicing: TonestackVoicing) {
        self.tonestack.set_voicing(voicing);
    }

    /// Set power amp presence
    pub fn set_presence(&mut self, presence: f32) {
        self.power_amp.set_presence(presence);
    }

    /// Set power amp sag
    pub fn set_sag(&mut self, sag: f32) {
        self.power_amp.set_sag(sag);
    }

    /// Set master volume (0.0 - 1.0)
    pub fn set_master(&mut self, master: f32) {
        self.master = master.clamp(0.0, 1.0);
    }

    /// Process a single sample through the full amp
    pub fn process(&mut self, input: f32) -> f32 {
        // Input gain stage
        let gained = input * (self.gain * 10.0 + 0.5);

        // Cascade through preamp tubes
        let mut signal = gained;
        for stage in &mut self.stages {
            signal = stage.process(signal);
        }

        // Tonestack EQ
        signal = self.tonestack.process(signal);

        // Power amp
        signal = self.power_amp.process(signal);

        // Master volume
        signal * self.master
    }

    /// Reset all internal state
    pub fn reset(&mut self) {
        for stage in &mut self.stages {
            stage.reset();
        }
        self.tonestack.reset();
        self.power_amp.reset();
    }
}

// ============================================================================
// Preset Configurations
// ============================================================================

/// Amp preset configurations
#[derive(Debug, Clone)]
pub struct AmpPreset {
    pub name: &'static str,
    pub channel: AmpChannel,
    pub gain: f32,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub presence: f32,
    pub voicing: TonestackVoicing,
    pub sag: f32,
    pub master: f32,
}

impl AmpPreset {
    /// Clean jazz tone
    pub fn clean_jazz() -> Self {
        Self {
            name: "Clean Jazz",
            channel: AmpChannel::Clean,
            gain: 0.3,
            bass: 0.4,
            mid: 0.6,
            treble: 0.5,
            presence: 0.4,
            voicing: TonestackVoicing::Fender,
            sag: 0.1,
            master: 0.7,
        }
    }

    /// Classic rock crunch
    pub fn classic_rock() -> Self {
        Self {
            name: "Classic Rock",
            channel: AmpChannel::Crunch,
            gain: 0.6,
            bass: 0.5,
            mid: 0.7,
            treble: 0.6,
            presence: 0.5,
            voicing: TonestackVoicing::Marshall,
            sag: 0.4,
            master: 0.6,
        }
    }

    /// Modern metal
    pub fn modern_metal() -> Self {
        Self {
            name: "Modern Metal",
            channel: AmpChannel::HighGain,
            gain: 0.8,
            bass: 0.6,
            mid: 0.4,
            treble: 0.7,
            presence: 0.7,
            voicing: TonestackVoicing::Mesa,
            sag: 0.2,
            master: 0.5,
        }
    }

    /// Djent/progressive
    pub fn djent() -> Self {
        Self {
            name: "Djent",
            channel: AmpChannel::HighGain,
            gain: 0.75,
            bass: 0.4,
            mid: 0.5,
            treble: 0.65,
            presence: 0.8,
            voicing: TonestackVoicing::Modern,
            sag: 0.1,
            master: 0.5,
        }
    }

    /// Technical death metal
    pub fn tech_death() -> Self {
        Self {
            name: "Tech Death",
            channel: AmpChannel::Lead,
            gain: 0.9,
            bass: 0.5,
            mid: 0.45,
            treble: 0.75,
            presence: 0.75,
            voicing: TonestackVoicing::Modern,
            sag: 0.15,
            master: 0.45,
        }
    }

    /// Apply preset to amp simulator
    pub fn apply_to(&self, amp: &mut AmpSimulator) {
        amp.set_channel(self.channel);
        amp.set_gain(self.gain);
        amp.set_eq(self.bass, self.mid, self.treble);
        amp.set_voicing(self.voicing);
        amp.set_presence(self.presence);
        amp.set_sag(self.sag);
        amp.set_master(self.master);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tube_stage() {
        let mut stage = TubeStage::new(44100.0);
        stage.set_drive(3.0);

        // Process some samples
        let output: Vec<f32> = (0..1000)
            .map(|i| {
                let input = (i as f32 * 0.01).sin() * 0.5;
                stage.process(input)
            })
            .collect();

        // Should produce non-zero output
        assert!(output.iter().any(|&s| s.abs() > 0.01));
    }

    #[test]
    fn test_tonestack() {
        let mut ts = Tonestack::new(44100.0);
        ts.set_controls(0.5, 0.5, 0.5);

        // Process impulse
        let output = ts.process(1.0);
        assert!(output.is_finite());
    }

    #[test]
    fn test_power_amp() {
        let mut pa = PowerAmp::new(44100.0);
        pa.set_drive(2.0);
        pa.set_sag(0.3);

        let output: Vec<f32> = (0..1000)
            .map(|i| {
                let input = (i as f32 * 0.02).sin() * 0.8;
                pa.process(input)
            })
            .collect();

        assert!(output.iter().any(|&s| s.abs() > 0.01));
    }

    #[test]
    fn test_amp_simulator() {
        let mut amp = AmpSimulator::new(44100.0);
        amp.set_channel(AmpChannel::HighGain);
        amp.set_gain(0.7);
        amp.set_eq(0.5, 0.5, 0.6);

        // Process a guitar-like signal
        let output: Vec<f32> = (0..44100)
            .map(|i| {
                let t = i as f32 / 44100.0;
                let input = (t * 440.0 * 2.0 * PI).sin() * 0.3 * (-t * 5.0).exp();
                amp.process(input)
            })
            .collect();

        // Should have audio content
        let rms: f32 = (output.iter().map(|s| s * s).sum::<f32>() / output.len() as f32).sqrt();
        assert!(rms > 0.001);
    }

    #[test]
    fn test_presets() {
        let mut amp = AmpSimulator::new(44100.0);

        for preset in [
            AmpPreset::clean_jazz(),
            AmpPreset::classic_rock(),
            AmpPreset::modern_metal(),
            AmpPreset::djent(),
            AmpPreset::tech_death(),
        ] {
            preset.apply_to(&mut amp);

            // Process some audio
            let output = amp.process(0.5);
            assert!(output.is_finite());
        }
    }
}
