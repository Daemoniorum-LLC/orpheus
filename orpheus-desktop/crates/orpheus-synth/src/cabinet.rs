//! Cabinet simulation module
//!
//! Models guitar cabinet frequency response and speaker characteristics
//! using filter networks and short synthetic impulse responses.

use std::f32::consts::PI;

/// Biquad filter coefficients
#[derive(Debug, Clone, Copy)]
struct BiquadCoeffs {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl Default for BiquadCoeffs {
    fn default() -> Self {
        Self { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0 }
    }
}

impl BiquadCoeffs {
    /// Low-pass filter (12dB/octave)
    fn lowpass(freq: f32, q: f32, sample_rate: f32) -> Self {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = (1.0 - cos_omega) / 2.0;
        let b1 = 1.0 - cos_omega;
        let b2 = (1.0 - cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// High-pass filter (12dB/octave)
    fn highpass(freq: f32, q: f32, sample_rate: f32) -> Self {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = (1.0 + cos_omega) / 2.0;
        let b1 = -(1.0 + cos_omega);
        let b2 = (1.0 + cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Peaking EQ filter
    fn peaking(freq: f32, q: f32, gain_db: f32, sample_rate: f32) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Low shelf filter
    fn low_shelf(freq: f32, gain_db: f32, sample_rate: f32) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / 2.0 * ((a + 1.0 / a) * (1.0 / 0.9 - 1.0) + 2.0).sqrt();

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_omega + 2.0 * alpha * a.sqrt());
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_omega);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_omega - 2.0 * alpha * a.sqrt());
        let a0 = (a + 1.0) + (a - 1.0) * cos_omega + 2.0 * alpha * a.sqrt();
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega);
        let a2 = (a + 1.0) + (a - 1.0) * cos_omega - 2.0 * alpha * a.sqrt();

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

/// Speaker resonance model
#[derive(Debug, Clone)]
struct SpeakerResonance {
    /// Resonance filter coefficients
    coeffs: BiquadCoeffs,
    /// Filter state
    state: [f32; 2],
}

impl SpeakerResonance {
    fn new(freq: f32, q: f32, gain_db: f32, sample_rate: f32) -> Self {
        Self {
            coeffs: BiquadCoeffs::peaking(freq, q, gain_db, sample_rate),
            state: [0.0; 2],
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let c = &self.coeffs;
        let output = c.b0 * input + self.state[0];
        self.state[0] = c.b1 * input - c.a1 * output + self.state[1];
        self.state[1] = c.b2 * input - c.a2 * output;
        output
    }

    fn reset(&mut self) {
        self.state = [0.0; 2];
    }
}

/// Short convolution for speaker "color"
/// Uses time-domain convolution for short impulses (< 256 samples)
#[derive(Debug, Clone)]
struct ShortConvolver {
    /// Impulse response
    ir: Vec<f32>,
    /// Delay line for input
    buffer: Vec<f32>,
    /// Write position
    write_pos: usize,
}

impl ShortConvolver {
    fn new(ir: Vec<f32>) -> Self {
        let len = ir.len();
        Self {
            ir,
            buffer: vec![0.0; len],
            write_pos: 0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        // Write input to circular buffer
        self.buffer[self.write_pos] = input;

        // Time-domain convolution
        let mut output = 0.0;
        let len = self.ir.len();
        for i in 0..len {
            let read_pos = (self.write_pos + len - i) % len;
            output += self.buffer[read_pos] * self.ir[i];
        }

        // Advance write position
        self.write_pos = (self.write_pos + 1) % len;

        output
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

/// Cabinet type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CabinetType {
    /// 4x12 closed back with Celestion Vintage 30 speakers
    /// Tight low end, aggressive mids, smooth highs
    Cab4x12V30,
    /// 4x12 closed back with Celestion Greenback speakers
    /// Classic rock tone, mid-focused, vintage character
    Cab4x12Greenback,
    /// 2x12 open back combo style
    /// Open, airy sound with less bass focus
    Cab2x12Open,
    /// 1x12 closed back for tighter response
    /// Direct, focused tone
    Cab1x12Closed,
    /// Direct injection (minimal coloring)
    /// Mostly high-cut filtering only
    DirectInject,
}

impl Default for CabinetType {
    fn default() -> Self {
        Self::Cab4x12V30
    }
}

/// Cabinet simulator
#[derive(Debug, Clone)]
pub struct CabinetSimulator {
    /// Cabinet type
    cab_type: CabinetType,
    /// Sample rate
    sample_rate: f32,
    /// High-pass filter (remove sub frequencies)
    highpass: BiquadCoeffs,
    highpass_state: [f32; 2],
    /// Low-pass filter (speaker rolloff)
    lowpass: BiquadCoeffs,
    lowpass_state: [f32; 2],
    /// Speaker resonances
    resonances: Vec<SpeakerResonance>,
    /// Short convolver for speaker breakup/color
    convolver: Option<ShortConvolver>,
    /// Output level
    output_level: f32,
    /// Microphone position (0.0 = center/bright, 1.0 = edge/dark)
    mic_position: f32,
    /// Mic position filter
    mic_filter: BiquadCoeffs,
    mic_state: [f32; 2],
}

impl CabinetSimulator {
    /// Create a new cabinet simulator
    pub fn new(sample_rate: f32) -> Self {
        let mut sim = Self {
            cab_type: CabinetType::default(),
            sample_rate,
            highpass: BiquadCoeffs::default(),
            highpass_state: [0.0; 2],
            lowpass: BiquadCoeffs::default(),
            lowpass_state: [0.0; 2],
            resonances: Vec::new(),
            convolver: None,
            output_level: 1.0,
            mic_position: 0.3, // Slightly off-center default
            mic_filter: BiquadCoeffs::default(),
            mic_state: [0.0; 2],
        };
        sim.set_cabinet_type(CabinetType::Cab4x12V30);
        sim
    }

    /// Set cabinet type
    pub fn set_cabinet_type(&mut self, cab_type: CabinetType) {
        self.cab_type = cab_type;

        match cab_type {
            CabinetType::Cab4x12V30 => self.configure_4x12_v30(),
            CabinetType::Cab4x12Greenback => self.configure_4x12_greenback(),
            CabinetType::Cab2x12Open => self.configure_2x12_open(),
            CabinetType::Cab1x12Closed => self.configure_1x12_closed(),
            CabinetType::DirectInject => self.configure_direct(),
        }

        self.update_mic_position();
    }

    /// Set microphone position (0.0 = center/bright, 1.0 = edge/dark)
    pub fn set_mic_position(&mut self, position: f32) {
        self.mic_position = position.clamp(0.0, 1.0);
        self.update_mic_position();
    }

    /// Set output level
    pub fn set_output_level(&mut self, level: f32) {
        self.output_level = level.clamp(0.0, 2.0);
    }

    fn update_mic_position(&mut self) {
        // Mic position affects high frequency content
        // Center = bright (less filtering)
        // Edge = dark (more high cut)
        let cutoff = 8000.0 - self.mic_position * 5000.0; // 8kHz to 3kHz
        self.mic_filter = BiquadCoeffs::lowpass(cutoff, 0.707, self.sample_rate);
        self.mic_state = [0.0; 2];
    }

    fn configure_4x12_v30(&mut self) {
        let sr = self.sample_rate;

        // High-pass: remove sub bass
        self.highpass = BiquadCoeffs::highpass(70.0, 0.707, sr);
        self.highpass_state = [0.0; 2];

        // Low-pass: speaker rolloff
        self.lowpass = BiquadCoeffs::lowpass(5500.0, 0.6, sr);
        self.lowpass_state = [0.0; 2];

        // V30 characteristics:
        // - Tight low end around 100Hz
        // - Strong upper-mid presence 2-4kHz
        // - Smooth high frequency rolloff
        self.resonances = vec![
            SpeakerResonance::new(100.0, 1.5, 3.0, sr),   // Bass punch
            SpeakerResonance::new(400.0, 2.0, -2.0, sr),  // Slight mid scoop
            SpeakerResonance::new(2500.0, 1.8, 4.0, sr),  // V30 presence peak
            SpeakerResonance::new(3500.0, 2.0, 2.0, sr),  // Upper presence
        ];

        // Short synthetic IR for speaker breakup character
        self.convolver = Some(ShortConvolver::new(Self::generate_v30_ir(sr)));
    }

    fn configure_4x12_greenback(&mut self) {
        let sr = self.sample_rate;

        // High-pass
        self.highpass = BiquadCoeffs::highpass(80.0, 0.707, sr);
        self.highpass_state = [0.0; 2];

        // Low-pass: earlier rolloff for vintage sound
        self.lowpass = BiquadCoeffs::lowpass(4500.0, 0.5, sr);
        self.lowpass_state = [0.0; 2];

        // Greenback characteristics:
        // - Looser low end
        // - Strong mids
        // - Earlier high rolloff (darker)
        self.resonances = vec![
            SpeakerResonance::new(120.0, 1.2, 4.0, sr),   // Looser bass
            SpeakerResonance::new(800.0, 1.5, 3.0, sr),   // Mid honk
            SpeakerResonance::new(2000.0, 2.0, 2.0, sr),  // Presence
        ];

        self.convolver = Some(ShortConvolver::new(Self::generate_greenback_ir(sr)));
    }

    fn configure_2x12_open(&mut self) {
        let sr = self.sample_rate;

        // High-pass: less sub removal
        self.highpass = BiquadCoeffs::highpass(60.0, 0.707, sr);
        self.highpass_state = [0.0; 2];

        // Low-pass
        self.lowpass = BiquadCoeffs::lowpass(6000.0, 0.6, sr);
        self.lowpass_state = [0.0; 2];

        // Open back characteristics:
        // - Less bass (cancellation from back wave)
        // - Airy, three-dimensional sound
        self.resonances = vec![
            SpeakerResonance::new(150.0, 1.0, -2.0, sr),  // Bass reduction
            SpeakerResonance::new(600.0, 1.5, 2.0, sr),   // Mid emphasis
            SpeakerResonance::new(3000.0, 2.0, 3.0, sr),  // Air/presence
        ];

        self.convolver = Some(ShortConvolver::new(Self::generate_open_back_ir(sr)));
    }

    fn configure_1x12_closed(&mut self) {
        let sr = self.sample_rate;

        // High-pass
        self.highpass = BiquadCoeffs::highpass(90.0, 0.707, sr);
        self.highpass_state = [0.0; 2];

        // Low-pass
        self.lowpass = BiquadCoeffs::lowpass(5000.0, 0.6, sr);
        self.lowpass_state = [0.0; 2];

        // 1x12 closed characteristics:
        // - Focused, direct sound
        // - Less bass extension
        // - Tighter response
        self.resonances = vec![
            SpeakerResonance::new(130.0, 2.0, 2.0, sr),   // Tight bass
            SpeakerResonance::new(500.0, 1.5, 1.0, sr),   // Mid focus
            SpeakerResonance::new(2200.0, 2.0, 2.5, sr),  // Presence
        ];

        self.convolver = Some(ShortConvolver::new(Self::generate_1x12_ir(sr)));
    }

    fn configure_direct(&mut self) {
        let sr = self.sample_rate;

        // Minimal filtering - just tame the extremes
        self.highpass = BiquadCoeffs::highpass(40.0, 0.707, sr);
        self.highpass_state = [0.0; 2];

        self.lowpass = BiquadCoeffs::lowpass(12000.0, 0.707, sr);
        self.lowpass_state = [0.0; 2];

        // No resonances for DI
        self.resonances.clear();
        self.convolver = None;
    }

    /// Generate synthetic V30-style impulse response
    fn generate_v30_ir(sample_rate: f32) -> Vec<f32> {
        let len = (sample_rate * 0.003) as usize; // 3ms IR
        let mut ir = vec![0.0; len];

        // Main impulse with slight asymmetry (speaker cone behavior)
        ir[0] = 1.0;
        ir[1] = 0.6;
        ir[2] = -0.3;
        ir[3] = 0.15;
        ir[4] = -0.08;

        // Add some early reflections (cabinet resonance)
        if len > 20 {
            ir[12] = 0.12;
            ir[18] = -0.08;
            ir[25] = 0.05;
        }

        // Decay tail
        for i in 5..len {
            let decay = (-3.0 * i as f32 / len as f32).exp();
            ir[i] += decay * 0.02 * ((i as f32 * 0.3).sin());
        }

        // Normalize
        let max = ir.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
        if max > 0.0 {
            for sample in &mut ir {
                *sample /= max;
            }
        }

        ir
    }

    /// Generate synthetic Greenback-style impulse response
    fn generate_greenback_ir(sample_rate: f32) -> Vec<f32> {
        let len = (sample_rate * 0.004) as usize; // 4ms IR (longer tail)
        let mut ir = vec![0.0; len];

        // Softer attack (vintage speaker)
        ir[0] = 0.8;
        ir[1] = 1.0;
        ir[2] = 0.5;
        ir[3] = -0.2;
        ir[4] = 0.1;
        ir[5] = -0.05;

        // More pronounced cabinet resonance
        if len > 30 {
            ir[15] = 0.15;
            ir[22] = -0.1;
            ir[30] = 0.08;
        }

        // Longer decay (looser construction)
        for i in 6..len {
            let decay = (-2.5 * i as f32 / len as f32).exp();
            ir[i] += decay * 0.03 * ((i as f32 * 0.25).sin());
        }

        let max = ir.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
        if max > 0.0 {
            for sample in &mut ir {
                *sample /= max;
            }
        }

        ir
    }

    /// Generate synthetic open-back impulse response
    fn generate_open_back_ir(sample_rate: f32) -> Vec<f32> {
        let len = (sample_rate * 0.005) as usize; // 5ms (room interaction)
        let mut ir = vec![0.0; len];

        // Initial impulse with cancellation notch
        ir[0] = 0.9;
        ir[1] = 0.4;
        ir[2] = -0.5; // Back wave cancellation
        ir[3] = 0.3;
        ir[4] = -0.15;

        // Room reflections (open back interacts more with environment)
        if len > 40 {
            ir[20] = 0.1;
            ir[28] = -0.12;
            ir[35] = 0.08;
            ir[45] = -0.05;
        }

        for i in 5..len {
            let decay = (-2.0 * i as f32 / len as f32).exp();
            ir[i] += decay * 0.025 * ((i as f32 * 0.2).sin());
        }

        let max = ir.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
        if max > 0.0 {
            for sample in &mut ir {
                *sample /= max;
            }
        }

        ir
    }

    /// Generate synthetic 1x12 closed impulse response
    fn generate_1x12_ir(sample_rate: f32) -> Vec<f32> {
        let len = (sample_rate * 0.002) as usize; // 2ms (tight response)
        let mut ir = vec![0.0; len];

        // Quick, tight response
        ir[0] = 1.0;
        ir[1] = 0.4;
        ir[2] = -0.2;
        ir[3] = 0.08;

        // Minimal cabinet resonance
        if len > 15 {
            ir[10] = 0.05;
            ir[15] = -0.03;
        }

        for i in 4..len {
            let decay = (-4.0 * i as f32 / len as f32).exp();
            ir[i] += decay * 0.015 * ((i as f32 * 0.35).sin());
        }

        let max = ir.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
        if max > 0.0 {
            for sample in &mut ir {
                *sample /= max;
            }
        }

        ir
    }

    /// Process a single sample through the cabinet
    pub fn process(&mut self, input: f32) -> f32 {
        // High-pass filter
        let hp_out = {
            let c = &self.highpass;
            let out = c.b0 * input + self.highpass_state[0];
            self.highpass_state[0] = c.b1 * input - c.a1 * out + self.highpass_state[1];
            self.highpass_state[1] = c.b2 * input - c.a2 * out;
            out
        };

        // Process through speaker resonances
        let mut signal = hp_out;
        for resonance in &mut self.resonances {
            signal = resonance.process(signal);
        }

        // Short convolution for speaker color
        if let Some(ref mut conv) = self.convolver {
            signal = conv.process(signal);
        }

        // Low-pass filter (speaker rolloff)
        let lp_out = {
            let c = &self.lowpass;
            let out = c.b0 * signal + self.lowpass_state[0];
            self.lowpass_state[0] = c.b1 * signal - c.a1 * out + self.lowpass_state[1];
            self.lowpass_state[1] = c.b2 * signal - c.a2 * out;
            out
        };

        // Mic position filter
        let mic_out = {
            let c = &self.mic_filter;
            let out = c.b0 * lp_out + self.mic_state[0];
            self.mic_state[0] = c.b1 * lp_out - c.a1 * out + self.mic_state[1];
            self.mic_state[1] = c.b2 * lp_out - c.a2 * out;
            out
        };

        mic_out * self.output_level
    }

    /// Reset all filter states
    pub fn reset(&mut self) {
        self.highpass_state = [0.0; 2];
        self.lowpass_state = [0.0; 2];
        self.mic_state = [0.0; 2];
        for resonance in &mut self.resonances {
            resonance.reset();
        }
        if let Some(ref mut conv) = self.convolver {
            conv.reset();
        }
    }

    /// Get cabinet type
    pub fn cabinet_type(&self) -> CabinetType {
        self.cab_type
    }

    /// Get current mic position
    pub fn mic_position(&self) -> f32 {
        self.mic_position
    }
}

/// Cabinet preset combining cabinet type with settings
#[derive(Debug, Clone)]
pub struct CabinetPreset {
    /// Cabinet type
    pub cab_type: CabinetType,
    /// Mic position
    pub mic_position: f32,
    /// Output level
    pub output_level: f32,
}

impl CabinetPreset {
    /// Modern metal (4x12 V30, slightly off-center)
    pub fn modern_metal() -> Self {
        Self {
            cab_type: CabinetType::Cab4x12V30,
            mic_position: 0.25,
            output_level: 1.0,
        }
    }

    /// Classic rock (4x12 Greenback, edge position)
    pub fn classic_rock() -> Self {
        Self {
            cab_type: CabinetType::Cab4x12Greenback,
            mic_position: 0.5,
            output_level: 1.0,
        }
    }

    /// Clean jazz (2x12 open, centered)
    pub fn clean_jazz() -> Self {
        Self {
            cab_type: CabinetType::Cab2x12Open,
            mic_position: 0.2,
            output_level: 0.9,
        }
    }

    /// Djent/progressive (4x12 V30, very tight/centered)
    pub fn djent() -> Self {
        Self {
            cab_type: CabinetType::Cab4x12V30,
            mic_position: 0.1, // Very close to center for maximum clarity
            output_level: 1.1,
        }
    }

    /// Vintage blues (open back, edge mic)
    pub fn vintage_blues() -> Self {
        Self {
            cab_type: CabinetType::Cab2x12Open,
            mic_position: 0.6,
            output_level: 0.95,
        }
    }

    /// Apply preset to cabinet simulator
    pub fn apply_to(&self, cab: &mut CabinetSimulator) {
        cab.set_cabinet_type(self.cab_type);
        cab.set_mic_position(self.mic_position);
        cab.set_output_level(self.output_level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cabinet_creation() {
        let cab = CabinetSimulator::new(44100.0);
        assert_eq!(cab.cabinet_type(), CabinetType::Cab4x12V30);
    }

    #[test]
    fn test_cabinet_types() {
        let mut cab = CabinetSimulator::new(44100.0);

        for cab_type in [
            CabinetType::Cab4x12V30,
            CabinetType::Cab4x12Greenback,
            CabinetType::Cab2x12Open,
            CabinetType::Cab1x12Closed,
            CabinetType::DirectInject,
        ] {
            cab.set_cabinet_type(cab_type);
            assert_eq!(cab.cabinet_type(), cab_type);
        }
    }

    #[test]
    fn test_cabinet_processing() {
        let mut cab = CabinetSimulator::new(44100.0);

        // Process some samples
        let mut has_output = false;
        for i in 0..1000 {
            let input = if i < 10 { 1.0 } else { 0.0 }; // Impulse
            let output = cab.process(input);
            if output.abs() > 0.001 {
                has_output = true;
            }
        }

        assert!(has_output, "Cabinet should produce output");
    }

    #[test]
    fn test_mic_position() {
        let mut cab = CabinetSimulator::new(44100.0);

        cab.set_mic_position(0.0); // Center
        assert!((cab.mic_position() - 0.0).abs() < 0.001);

        cab.set_mic_position(1.0); // Edge
        assert!((cab.mic_position() - 1.0).abs() < 0.001);

        // Clamp test
        cab.set_mic_position(2.0);
        assert!((cab.mic_position() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_presets() {
        let mut cab = CabinetSimulator::new(44100.0);

        CabinetPreset::modern_metal().apply_to(&mut cab);
        assert_eq!(cab.cabinet_type(), CabinetType::Cab4x12V30);

        CabinetPreset::classic_rock().apply_to(&mut cab);
        assert_eq!(cab.cabinet_type(), CabinetType::Cab4x12Greenback);

        CabinetPreset::clean_jazz().apply_to(&mut cab);
        assert_eq!(cab.cabinet_type(), CabinetType::Cab2x12Open);
    }

    #[test]
    fn test_ir_generation() {
        let ir = CabinetSimulator::generate_v30_ir(44100.0);
        assert!(!ir.is_empty());
        assert!(ir.iter().any(|&x| x != 0.0));

        // Check normalization
        let max = ir.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
        assert!((max - 1.0).abs() < 0.01);
    }
}
