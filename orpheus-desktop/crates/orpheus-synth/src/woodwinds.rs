//! Woodwind instrument synthesizer using physical modeling
//!
//! Implements realistic woodwind synthesis with:
//! - Air jet oscillator (flute family)
//! - Single reed oscillator (clarinet, saxophone)
//! - Double reed oscillator (oboe, bassoon)
//! - Bore resonator with tone hole modeling
//! - Key click and breath noise

use std::f32::consts::PI;

/// Woodwind instrument types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WoodwindInstrument {
    // Flute family (air jet)
    Piccolo,
    Flute,
    AltoFlute,
    BassFlute,

    // Single reed (clarinet family)
    EbClarinet,
    BbClarinet,
    BassClarinet,
    ContrabassClarinet,

    // Single reed (saxophone family)
    SopraninoSax,
    SopranoSax,
    AltoSax,
    TenorSax,
    BaritoneSax,
    BassSax,

    // Double reed
    Oboe,
    EnglishHorn,
    Bassoon,
    Contrabassoon,
}

impl WoodwindInstrument {
    /// Get the excitation type for this instrument
    pub fn excitation_type(&self) -> ExcitationType {
        match self {
            Self::Piccolo | Self::Flute | Self::AltoFlute | Self::BassFlute => {
                ExcitationType::AirJet
            }
            Self::EbClarinet
            | Self::BbClarinet
            | Self::BassClarinet
            | Self::ContrabassClarinet
            | Self::SopraninoSax
            | Self::SopranoSax
            | Self::AltoSax
            | Self::TenorSax
            | Self::BaritoneSax
            | Self::BassSax => ExcitationType::SingleReed,
            Self::Oboe | Self::EnglishHorn | Self::Bassoon | Self::Contrabassoon => {
                ExcitationType::DoubleReed
            }
        }
    }

    /// Get the bore type for this instrument
    pub fn bore_type(&self) -> BoreType {
        match self {
            // Cylindrical bore
            Self::Piccolo
            | Self::Flute
            | Self::AltoFlute
            | Self::BassFlute
            | Self::EbClarinet
            | Self::BbClarinet
            | Self::BassClarinet
            | Self::ContrabassClarinet => BoreType::Cylindrical,
            // Conical bore
            Self::SopraninoSax
            | Self::SopranoSax
            | Self::AltoSax
            | Self::TenorSax
            | Self::BaritoneSax
            | Self::BassSax
            | Self::Oboe
            | Self::EnglishHorn
            | Self::Bassoon
            | Self::Contrabassoon => BoreType::Conical,
        }
    }

    /// Get base frequency range (lowest note in Hz)
    pub fn lowest_frequency(&self) -> f32 {
        match self {
            Self::Piccolo => 587.33,        // D5
            Self::Flute => 261.63,          // C4
            Self::AltoFlute => 196.0,       // G3
            Self::BassFlute => 130.81,      // C3
            Self::EbClarinet => 207.65,     // Ab3
            Self::BbClarinet => 146.83,     // D3
            Self::BassClarinet => 73.42,    // D2
            Self::ContrabassClarinet => 36.71, // D1
            Self::SopraninoSax => 311.13,   // Eb4
            Self::SopranoSax => 207.65,     // Ab3
            Self::AltoSax => 138.59,        // Db3
            Self::TenorSax => 103.83,       // Ab2
            Self::BaritoneSax => 69.3,      // Db2
            Self::BassSax => 46.25,         // Ab1
            Self::Oboe => 233.08,           // Bb3
            Self::EnglishHorn => 164.81,    // E3
            Self::Bassoon => 58.27,         // Bb1
            Self::Contrabassoon => 29.14,   // Bb0
        }
    }

    /// Get characteristic brightness (0.0 = dark, 1.0 = bright)
    pub fn brightness(&self) -> f32 {
        match self {
            Self::Piccolo => 0.95,
            Self::Flute => 0.75,
            Self::AltoFlute => 0.55,
            Self::BassFlute => 0.4,
            Self::EbClarinet => 0.8,
            Self::BbClarinet => 0.6,
            Self::BassClarinet => 0.35,
            Self::ContrabassClarinet => 0.25,
            Self::SopraninoSax => 0.85,
            Self::SopranoSax => 0.75,
            Self::AltoSax => 0.65,
            Self::TenorSax => 0.55,
            Self::BaritoneSax => 0.45,
            Self::BassSax => 0.3,
            Self::Oboe => 0.85,
            Self::EnglishHorn => 0.6,
            Self::Bassoon => 0.5,
            Self::Contrabassoon => 0.3,
        }
    }

    /// Get characteristic reed/embouchure stiffness
    pub fn stiffness(&self) -> f32 {
        match self {
            Self::Piccolo | Self::Flute | Self::AltoFlute | Self::BassFlute => 0.3,
            Self::EbClarinet => 0.7,
            Self::BbClarinet => 0.65,
            Self::BassClarinet => 0.55,
            Self::ContrabassClarinet => 0.5,
            Self::SopraninoSax => 0.6,
            Self::SopranoSax => 0.55,
            Self::AltoSax => 0.5,
            Self::TenorSax => 0.45,
            Self::BaritoneSax => 0.4,
            Self::BassSax => 0.35,
            Self::Oboe => 0.85,
            Self::EnglishHorn => 0.75,
            Self::Bassoon => 0.7,
            Self::Contrabassoon => 0.6,
        }
    }
}

/// Type of excitation mechanism
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExcitationType {
    AirJet,
    SingleReed,
    DoubleReed,
}

/// Type of bore
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoreType {
    Cylindrical,
    Conical,
}

/// Woodwind articulation styles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WoodwindArticulation {
    Normal,
    Legato,
    Staccato,
    Tenuto,
    Accent,
    Sforzando,
    // Flute-specific
    DoubleToguing,
    TripleTonguing,
    FlutterTongue,
    // Reed-specific
    Slap,
    Growl,
    Subtone,
    // Extended techniques
    Multiphonic,
    Harmonic,
    Glissando,
    Trill,
    Tremolo,
    BendUp,
    BendDown,
}

/// Air jet oscillator for flute family
#[derive(Debug, Clone)]
pub struct AirJetOscillator {
    /// Jet delay line
    jet_delay: Vec<f32>,
    jet_write_pos: usize,
    /// Jet parameters
    jet_velocity: f32,
    jet_length: f32,
    /// Edge tone frequency
    edge_freq: f32,
    /// Noise component
    noise_amount: f32,
    /// Low pass for jet smoothing
    lp_state: f32,
    /// Sample rate
    sample_rate: f32,
}

impl AirJetOscillator {
    pub fn new(sample_rate: f32) -> Self {
        let max_delay = (sample_rate * 0.01) as usize; // 10ms max
        Self {
            jet_delay: vec![0.0; max_delay],
            jet_write_pos: 0,
            jet_velocity: 0.5,
            jet_length: 0.005, // 5mm typical
            edge_freq: 440.0,
            noise_amount: 0.15,
            lp_state: 0.0,
            sample_rate,
        }
    }

    pub fn set_parameters(&mut self, velocity: f32, freq: f32) {
        self.jet_velocity = velocity.clamp(0.0, 1.0);
        self.edge_freq = freq;
        // Jet length affects turbulence
        self.jet_length = 0.003 + velocity * 0.004;
    }

    pub fn process(&mut self, bore_feedback: f32) -> f32 {
        // Calculate jet delay based on velocity
        let jet_delay_samples =
            (self.jet_length / (self.jet_velocity * 100.0 + 10.0) * self.sample_rate) as usize;
        let jet_delay_samples = jet_delay_samples.clamp(1, self.jet_delay.len() - 1);

        // Read from jet delay
        let read_pos = (self.jet_write_pos + self.jet_delay.len() - jet_delay_samples)
            % self.jet_delay.len();
        let delayed_jet = self.jet_delay[read_pos];

        // Generate turbulent noise
        let noise = (rand_simple() * 2.0 - 1.0) * self.noise_amount * self.jet_velocity;

        // Jet deflection based on bore feedback
        let jet_deflection = (bore_feedback * 2.0).tanh();

        // Air jet model: nonlinear interaction at the edge
        let jet_signal = (delayed_jet + jet_deflection + noise).tanh();

        // Write to delay line
        self.jet_delay[self.jet_write_pos] = jet_signal * self.jet_velocity;
        self.jet_write_pos = (self.jet_write_pos + 1) % self.jet_delay.len();

        // Low pass smoothing
        let cutoff = 0.3 + self.jet_velocity * 0.5;
        self.lp_state += cutoff * (jet_signal - self.lp_state);

        self.lp_state
    }
}

/// Single reed oscillator for clarinet and saxophone
#[derive(Debug, Clone)]
pub struct SingleReedOscillator {
    /// Reed displacement
    reed_state: f32,
    reed_velocity: f32,
    /// Reed parameters
    reed_stiffness: f32,
    reed_damping: f32,
    reed_mass: f32,
    /// Mouth pressure
    mouth_pressure: f32,
    /// Reed table (nonlinear characteristic)
    reed_table_offset: f32,
    reed_table_slope: f32,
    /// Sample rate
    sample_rate: f32,
}

impl SingleReedOscillator {
    pub fn new(sample_rate: f32, stiffness: f32) -> Self {
        Self {
            reed_state: 0.0,
            reed_velocity: 0.0,
            reed_stiffness: stiffness,
            reed_damping: 0.1,
            reed_mass: 0.01,
            mouth_pressure: 0.5,
            reed_table_offset: 0.7,
            reed_table_slope: -0.44,
            sample_rate,
        }
    }

    pub fn set_parameters(&mut self, pressure: f32, stiffness: f32) {
        self.mouth_pressure = pressure.clamp(0.0, 1.0);
        self.reed_stiffness = stiffness.clamp(0.1, 1.0);
    }

    pub fn process(&mut self, bore_pressure: f32) -> f32 {
        // Pressure difference across reed
        let delta_pressure = self.mouth_pressure - bore_pressure;

        // Reed dynamics (simplified mass-spring-damper)
        let reed_force = delta_pressure - self.reed_stiffness * self.reed_state
            - self.reed_damping * self.reed_velocity;

        // Update reed state
        let dt = 1.0 / self.sample_rate;
        self.reed_velocity += reed_force / self.reed_mass * dt;
        self.reed_state += self.reed_velocity * dt;

        // Clamp reed (can't go past closed or too open)
        self.reed_state = self.reed_state.clamp(-1.0, 1.0);

        // Reed table: nonlinear flow characteristic
        let reed_opening = self.reed_table_offset + self.reed_table_slope * delta_pressure;
        let reed_opening = reed_opening.clamp(0.0, 1.0);

        // Flow through reed (Bernoulli-based)
        let flow = reed_opening * delta_pressure.abs().sqrt() * delta_pressure.signum();

        flow * (1.0 - self.reed_state.abs() * 0.5)
    }
}

/// Double reed oscillator for oboe and bassoon
#[derive(Debug, Clone)]
pub struct DoubleReedOscillator {
    /// Two coupled reed states
    reed1_state: f32,
    reed2_state: f32,
    reed1_velocity: f32,
    reed2_velocity: f32,
    /// Reed parameters
    reed_stiffness: f32,
    reed_coupling: f32,
    reed_damping: f32,
    /// Mouth pressure
    mouth_pressure: f32,
    /// Sample rate
    sample_rate: f32,
}

impl DoubleReedOscillator {
    pub fn new(sample_rate: f32, stiffness: f32) -> Self {
        Self {
            reed1_state: 0.0,
            reed2_state: 0.0,
            reed1_velocity: 0.0,
            reed2_velocity: 0.0,
            reed_stiffness: stiffness,
            reed_coupling: 0.5,
            reed_damping: 0.15,
            mouth_pressure: 0.5,
            sample_rate,
        }
    }

    pub fn set_parameters(&mut self, pressure: f32, stiffness: f32) {
        self.mouth_pressure = pressure.clamp(0.0, 1.0);
        self.reed_stiffness = stiffness.clamp(0.1, 1.0);
    }

    pub fn process(&mut self, bore_pressure: f32) -> f32 {
        let delta_pressure = self.mouth_pressure - bore_pressure;
        let dt = 1.0 / self.sample_rate;

        // Coupled reed dynamics
        let coupling_force = self.reed_coupling * (self.reed2_state - self.reed1_state);

        // Reed 1 forces
        let force1 = delta_pressure - self.reed_stiffness * self.reed1_state
            - self.reed_damping * self.reed1_velocity
            + coupling_force;

        // Reed 2 forces (opposite coupling)
        let force2 = delta_pressure - self.reed_stiffness * self.reed2_state
            - self.reed_damping * self.reed2_velocity
            - coupling_force;

        // Update states
        self.reed1_velocity += force1 * dt * 100.0;
        self.reed2_velocity += force2 * dt * 100.0;
        self.reed1_state += self.reed1_velocity * dt;
        self.reed2_state += self.reed2_velocity * dt;

        // Clamp reeds
        self.reed1_state = self.reed1_state.clamp(-1.0, 1.0);
        self.reed2_state = self.reed2_state.clamp(-1.0, 1.0);

        // Gap between reeds determines flow
        let gap = 0.5 + 0.5 * (self.reed1_state - self.reed2_state);
        let gap = gap.clamp(0.0, 1.0);

        // Flow through gap (more complex than single reed)
        let flow = gap * gap * delta_pressure.abs().sqrt() * delta_pressure.signum();

        // Double reed has more harmonics
        flow * (1.0 + 0.3 * (self.reed1_state * self.reed2_state).sin())
    }
}

/// Bore resonator with tone holes
#[derive(Debug, Clone)]
pub struct BoreResonator {
    /// Primary delay line (bore)
    delay_line: Vec<f32>,
    write_pos: usize,
    /// Tone hole states
    tone_holes: Vec<ToneHole>,
    /// Bore type
    bore_type: BoreType,
    /// Bell reflection coefficient
    bell_reflection: f32,
    /// Current delay length
    delay_samples: f32,
    /// Lowpass filter state
    lp_state: f32,
    /// Sample rate
    sample_rate: f32,
}

/// Individual tone hole model
#[derive(Debug, Clone)]
pub struct ToneHole {
    /// Position along bore (0.0 = mouthpiece, 1.0 = bell)
    position: f32,
    /// Size of hole (affects radiation)
    size: f32,
    /// Open state (0.0 = closed, 1.0 = open)
    open: f32,
    /// Transmission coefficient
    transmission: f32,
}

impl BoreResonator {
    pub fn new(sample_rate: f32, bore_type: BoreType) -> Self {
        let max_delay = (sample_rate / 20.0) as usize; // Down to 20 Hz

        // Create typical tone hole layout
        let tone_holes = (0..12)
            .map(|i| ToneHole {
                position: 0.1 + (i as f32) * 0.07,
                size: 0.3 + (i as f32) * 0.02,
                open: 0.0,
                transmission: 0.9,
            })
            .collect();

        Self {
            delay_line: vec![0.0; max_delay],
            write_pos: 0,
            tone_holes,
            bore_type,
            bell_reflection: -0.7,
            delay_samples: 100.0,
            lp_state: 0.0,
            sample_rate,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        // For cylindrical bore (clarinet), acts as closed pipe (odd harmonics)
        // For conical bore (sax, oboe), acts as open pipe (all harmonics)
        let wavelength = self.sample_rate / freq;
        self.delay_samples = match self.bore_type {
            BoreType::Cylindrical => wavelength / 2.0, // Quarter wave, but round trip
            BoreType::Conical => wavelength / 2.0,
        };
        self.delay_samples = self.delay_samples.clamp(2.0, self.delay_line.len() as f32 - 1.0);
    }

    pub fn set_fingering(&mut self, fingering: &[bool]) {
        for (i, hole) in self.tone_holes.iter_mut().enumerate() {
            if i < fingering.len() {
                hole.open = if fingering[i] { 1.0 } else { 0.0 };
            }
        }
        self.update_effective_length();
    }

    fn update_effective_length(&mut self) {
        // Find first open hole to determine effective length
        let mut effective_pos = 1.0;
        for hole in &self.tone_holes {
            if hole.open > 0.5 {
                effective_pos = hole.position;
                break;
            }
        }
        // Adjust delay based on effective length
        self.delay_samples *= effective_pos;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        // Fractional delay interpolation
        let delay_int = self.delay_samples as usize;
        let delay_frac = self.delay_samples - delay_int as f32;

        let read_pos1 =
            (self.write_pos + self.delay_line.len() - delay_int) % self.delay_line.len();
        let read_pos2 = (read_pos1 + self.delay_line.len() - 1) % self.delay_line.len();

        let delayed = self.delay_line[read_pos1] * (1.0 - delay_frac)
            + self.delay_line[read_pos2] * delay_frac;

        // Tone hole radiation (simplified)
        let mut radiated = 0.0;
        for hole in &self.tone_holes {
            if hole.open > 0.5 {
                radiated += delayed * hole.size * hole.open * 0.1;
            }
        }

        // Bell reflection
        let reflected = delayed * self.bell_reflection;

        // Bore losses (frequency dependent)
        let cutoff = match self.bore_type {
            BoreType::Cylindrical => 0.85,
            BoreType::Conical => 0.9,
        };
        self.lp_state += cutoff * (reflected - self.lp_state);

        // Write new sample
        self.delay_line[self.write_pos] = input + self.lp_state;
        self.write_pos = (self.write_pos + 1) % self.delay_line.len();

        // Output is sum of bell radiation and tone hole radiation
        delayed * 0.7 + radiated
    }

    pub fn get_feedback(&self) -> f32 {
        let delay_int = self.delay_samples as usize;
        let read_pos =
            (self.write_pos + self.delay_line.len() - delay_int / 2) % self.delay_line.len();
        self.delay_line[read_pos]
    }
}

/// Envelope generator for woodwind dynamics
#[derive(Debug, Clone)]
pub struct WoodwindEnvelope {
    state: EnvelopeState,
    value: f32,
    attack_rate: f32,
    decay_rate: f32,
    sustain_level: f32,
    release_rate: f32,
    target: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl WoodwindEnvelope {
    pub fn new(sample_rate: f32, articulation: WoodwindArticulation) -> Self {
        let (attack, decay, sustain, release) = match articulation {
            WoodwindArticulation::Normal => (0.05, 0.1, 0.8, 0.15),
            WoodwindArticulation::Legato => (0.08, 0.05, 0.85, 0.2),
            WoodwindArticulation::Staccato => (0.02, 0.05, 0.7, 0.05),
            WoodwindArticulation::Tenuto => (0.06, 0.1, 0.85, 0.2),
            WoodwindArticulation::Accent => (0.01, 0.08, 0.75, 0.1),
            WoodwindArticulation::Sforzando => (0.005, 0.15, 0.6, 0.1),
            WoodwindArticulation::DoubleToguing | WoodwindArticulation::TripleTonguing => {
                (0.01, 0.02, 0.75, 0.02)
            }
            WoodwindArticulation::FlutterTongue => (0.03, 0.05, 0.8, 0.1),
            WoodwindArticulation::Slap => (0.002, 0.02, 0.3, 0.05),
            WoodwindArticulation::Growl => (0.04, 0.1, 0.85, 0.15),
            WoodwindArticulation::Subtone => (0.1, 0.15, 0.5, 0.25),
            WoodwindArticulation::Multiphonic => (0.08, 0.1, 0.7, 0.2),
            WoodwindArticulation::Harmonic => (0.06, 0.08, 0.6, 0.15),
            WoodwindArticulation::Glissando
            | WoodwindArticulation::Trill
            | WoodwindArticulation::Tremolo => (0.04, 0.08, 0.8, 0.12),
            WoodwindArticulation::BendUp | WoodwindArticulation::BendDown => (0.05, 0.1, 0.8, 0.15),
        };

        Self {
            state: EnvelopeState::Idle,
            value: 0.0,
            attack_rate: 1.0 / (attack * sample_rate),
            decay_rate: 1.0 / (decay * sample_rate),
            sustain_level: sustain,
            release_rate: 1.0 / (release * sample_rate),
            target: 0.0,
        }
    }

    pub fn note_on(&mut self, velocity: f32) {
        self.target = velocity;
        self.state = EnvelopeState::Attack;
    }

    pub fn note_off(&mut self) {
        self.state = EnvelopeState::Release;
    }

    pub fn process(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Idle => {}
            EnvelopeState::Attack => {
                self.value += self.attack_rate * self.target;
                if self.value >= self.target {
                    self.value = self.target;
                    self.state = EnvelopeState::Decay;
                }
            }
            EnvelopeState::Decay => {
                let target = self.sustain_level * self.target;
                self.value -= self.decay_rate * (self.value - target);
                if self.value <= target + 0.001 {
                    self.state = EnvelopeState::Sustain;
                }
            }
            EnvelopeState::Sustain => {
                // Hold at sustain level
            }
            EnvelopeState::Release => {
                self.value -= self.release_rate * self.value;
                if self.value < 0.001 {
                    self.value = 0.0;
                    self.state = EnvelopeState::Idle;
                }
            }
        }
        self.value
    }

    pub fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle
    }
}

/// Key click generator for realistic attacks
#[derive(Debug, Clone)]
pub struct KeyClick {
    /// Click samples
    click_buffer: Vec<f32>,
    /// Playback position
    position: usize,
    /// Is playing
    active: bool,
    /// Volume
    volume: f32,
}

impl KeyClick {
    pub fn new(sample_rate: f32) -> Self {
        // Generate a short click sound
        let click_duration = (sample_rate * 0.008) as usize; // 8ms
        let mut click_buffer = vec![0.0; click_duration];

        for (i, sample) in click_buffer.iter_mut().enumerate() {
            let t = i as f32 / sample_rate;
            // High frequency transient with quick decay
            let env = (-t * 500.0).exp();
            let noise = rand_simple() * 2.0 - 1.0;
            let click = (t * 8000.0 * PI).sin() * 0.3 + noise * 0.7;
            *sample = click * env;
        }

        Self {
            click_buffer,
            position: 0,
            active: false,
            volume: 0.1,
        }
    }

    pub fn trigger(&mut self, volume: f32) {
        self.position = 0;
        self.active = true;
        self.volume = volume * 0.15;
    }

    pub fn process(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        if self.position >= self.click_buffer.len() {
            self.active = false;
            return 0.0;
        }

        let sample = self.click_buffer[self.position] * self.volume;
        self.position += 1;
        sample
    }
}

/// Breath noise generator
#[derive(Debug, Clone)]
pub struct BreathNoise {
    /// Filter state
    bp_state1: f32,
    bp_state2: f32,
    /// Filter coefficients
    center_freq: f32,
    bandwidth: f32,
    /// Amount of noise
    amount: f32,
    /// Sample rate
    sample_rate: f32,
}

impl BreathNoise {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            bp_state1: 0.0,
            bp_state2: 0.0,
            center_freq: 2000.0,
            bandwidth: 1000.0,
            amount: 0.1,
            sample_rate,
        }
    }

    pub fn set_parameters(&mut self, freq: f32, amount: f32) {
        self.center_freq = freq.clamp(500.0, 8000.0);
        self.amount = amount.clamp(0.0, 0.5);
    }

    pub fn process(&mut self, breath_pressure: f32) -> f32 {
        if self.amount < 0.001 {
            return 0.0;
        }

        // Generate white noise
        let noise = rand_simple() * 2.0 - 1.0;

        // Bandpass filter for breath character
        let w0 = 2.0 * PI * self.center_freq / self.sample_rate;
        let q = self.center_freq / self.bandwidth;
        let alpha = w0.sin() / (2.0 * q);

        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;

        // Apply filter
        let output = (b0 / a0) * noise + self.bp_state1;
        self.bp_state1 = (b1 / a0) * noise - (a1 / a0) * output + self.bp_state2;
        self.bp_state2 = (b2 / a0) * noise - (a2 / a0) * output;

        output * self.amount * breath_pressure
    }
}

/// Complete woodwind synthesizer
#[derive(Debug, Clone)]
pub struct WoodwindSynth {
    /// Instrument type
    instrument: WoodwindInstrument,
    /// Excitation source (one of these will be used)
    air_jet: Option<AirJetOscillator>,
    single_reed: Option<SingleReedOscillator>,
    double_reed: Option<DoubleReedOscillator>,
    /// Bore resonator
    bore: BoreResonator,
    /// Envelope
    envelope: WoodwindEnvelope,
    /// Key click
    key_click: KeyClick,
    /// Breath noise
    breath_noise: BreathNoise,
    /// Current frequency
    frequency: f32,
    /// Breath pressure (0-1)
    breath_pressure: f32,
    /// Vibrato parameters
    vibrato_rate: f32,
    vibrato_depth: f32,
    vibrato_phase: f32,
    /// Sample rate
    sample_rate: f32,
    /// Articulation
    articulation: WoodwindArticulation,
}

impl WoodwindSynth {
    pub fn new(instrument: WoodwindInstrument, sample_rate: f32) -> Self {
        let excitation_type = instrument.excitation_type();
        let bore_type = instrument.bore_type();
        let stiffness = instrument.stiffness();

        let air_jet = if excitation_type == ExcitationType::AirJet {
            Some(AirJetOscillator::new(sample_rate))
        } else {
            None
        };

        let single_reed = if excitation_type == ExcitationType::SingleReed {
            Some(SingleReedOscillator::new(sample_rate, stiffness))
        } else {
            None
        };

        let double_reed = if excitation_type == ExcitationType::DoubleReed {
            Some(DoubleReedOscillator::new(sample_rate, stiffness))
        } else {
            None
        };

        Self {
            instrument,
            air_jet,
            single_reed,
            double_reed,
            bore: BoreResonator::new(sample_rate, bore_type),
            envelope: WoodwindEnvelope::new(sample_rate, WoodwindArticulation::Normal),
            key_click: KeyClick::new(sample_rate),
            breath_noise: BreathNoise::new(sample_rate),
            frequency: 440.0,
            breath_pressure: 0.0,
            vibrato_rate: 5.0,
            vibrato_depth: 0.02,
            vibrato_phase: 0.0,
            sample_rate,
            articulation: WoodwindArticulation::Normal,
        }
    }

    /// Factory method for flute
    pub fn flute(sample_rate: f32) -> Self {
        Self::new(WoodwindInstrument::Flute, sample_rate)
    }

    /// Factory method for clarinet
    pub fn clarinet(sample_rate: f32) -> Self {
        Self::new(WoodwindInstrument::BbClarinet, sample_rate)
    }

    /// Factory method for saxophone
    pub fn alto_sax(sample_rate: f32) -> Self {
        Self::new(WoodwindInstrument::AltoSax, sample_rate)
    }

    /// Factory method for oboe
    pub fn oboe(sample_rate: f32) -> Self {
        Self::new(WoodwindInstrument::Oboe, sample_rate)
    }

    /// Factory method for bassoon
    pub fn bassoon(sample_rate: f32) -> Self {
        Self::new(WoodwindInstrument::Bassoon, sample_rate)
    }

    pub fn set_articulation(&mut self, articulation: WoodwindArticulation) {
        self.articulation = articulation;
        self.envelope = WoodwindEnvelope::new(self.sample_rate, articulation);
    }

    pub fn note_on(&mut self, frequency: f32, velocity: f32) {
        self.frequency = frequency;
        self.breath_pressure = velocity;
        self.bore.set_frequency(frequency);

        // Trigger key click for non-legato
        if self.articulation != WoodwindArticulation::Legato {
            self.key_click.trigger(velocity * 0.5);
        }

        // Set excitation parameters
        if let Some(ref mut air_jet) = self.air_jet {
            air_jet.set_parameters(velocity, frequency);
        }
        if let Some(ref mut single_reed) = self.single_reed {
            single_reed.set_parameters(velocity, self.instrument.stiffness());
        }
        if let Some(ref mut double_reed) = self.double_reed {
            double_reed.set_parameters(velocity, self.instrument.stiffness());
        }

        // Set breath noise based on instrument
        let noise_amount = match self.instrument.excitation_type() {
            ExcitationType::AirJet => velocity * 0.15,
            ExcitationType::SingleReed => velocity * 0.05,
            ExcitationType::DoubleReed => velocity * 0.03,
        };
        self.breath_noise.set_parameters(2000.0 + frequency, noise_amount);

        self.envelope.note_on(velocity);
    }

    pub fn note_off(&mut self) {
        self.envelope.note_off();
    }

    pub fn set_vibrato(&mut self, rate: f32, depth: f32) {
        self.vibrato_rate = rate;
        self.vibrato_depth = depth;
    }

    pub fn set_fingering(&mut self, fingering: &[bool]) {
        self.bore.set_fingering(fingering);
    }

    pub fn process(&mut self) -> f32 {
        if !self.envelope.is_active() {
            return 0.0;
        }

        let env = self.envelope.process();

        // Apply vibrato
        self.vibrato_phase += self.vibrato_rate / self.sample_rate;
        if self.vibrato_phase >= 1.0 {
            self.vibrato_phase -= 1.0;
        }
        let vibrato = 1.0 + self.vibrato_depth * (self.vibrato_phase * 2.0 * PI).sin();
        let freq = self.frequency * vibrato;
        self.bore.set_frequency(freq);

        // Get bore feedback
        let bore_feedback = self.bore.get_feedback();

        // Generate excitation
        let excitation = if let Some(ref mut air_jet) = self.air_jet {
            air_jet.process(bore_feedback)
        } else if let Some(ref mut single_reed) = self.single_reed {
            single_reed.process(bore_feedback)
        } else if let Some(ref mut double_reed) = self.double_reed {
            double_reed.process(bore_feedback)
        } else {
            0.0
        };

        // Process through bore
        let bore_output = self.bore.process(excitation * env);

        // Add key click and breath noise
        let click = self.key_click.process();
        let breath = self.breath_noise.process(env);

        // Apply instrument brightness
        let brightness = self.instrument.brightness();
        let output = bore_output * brightness + bore_output * (1.0 - brightness) * 0.5;

        // Handle special articulations
        let output = match self.articulation {
            WoodwindArticulation::FlutterTongue => {
                let flutter = (self.vibrato_phase * 25.0 * 2.0 * PI).sin();
                output * (0.7 + 0.3 * flutter)
            }
            WoodwindArticulation::Growl => {
                let growl = (self.vibrato_phase * 30.0 * 2.0 * PI).sin();
                output * (0.8 + 0.2 * growl) + output * growl * 0.1
            }
            WoodwindArticulation::Subtone => output * 0.5,
            _ => output,
        };

        (output + click + breath) * env
    }

    pub fn is_active(&self) -> bool {
        self.envelope.is_active()
    }
}

/// Woodwind section for ensemble playing
#[derive(Debug, Clone)]
pub struct WoodwindSection {
    instruments: Vec<WoodwindSynth>,
    detune: Vec<f32>,
    pan: Vec<f32>,
}

impl WoodwindSection {
    pub fn new(instrument: WoodwindInstrument, count: usize, sample_rate: f32) -> Self {
        let instruments: Vec<_> = (0..count)
            .map(|_| WoodwindSynth::new(instrument, sample_rate))
            .collect();

        // Slight detuning for ensemble effect
        let detune: Vec<_> = (0..count)
            .map(|i| 1.0 + (i as f32 - count as f32 / 2.0) * 0.002)
            .collect();

        // Spread across stereo field
        let pan: Vec<_> = (0..count)
            .map(|i| (i as f32 / (count - 1).max(1) as f32) * 2.0 - 1.0)
            .collect();

        Self {
            instruments,
            detune,
            pan,
        }
    }

    /// Create a flute section
    pub fn flutes(count: usize, sample_rate: f32) -> Self {
        Self::new(WoodwindInstrument::Flute, count, sample_rate)
    }

    /// Create a clarinet section
    pub fn clarinets(count: usize, sample_rate: f32) -> Self {
        Self::new(WoodwindInstrument::BbClarinet, count, sample_rate)
    }

    /// Create an oboe section
    pub fn oboes(count: usize, sample_rate: f32) -> Self {
        Self::new(WoodwindInstrument::Oboe, count, sample_rate)
    }

    /// Create a bassoon section
    pub fn bassoons(count: usize, sample_rate: f32) -> Self {
        Self::new(WoodwindInstrument::Bassoon, count, sample_rate)
    }

    pub fn note_on(&mut self, frequency: f32, velocity: f32) {
        for (i, inst) in self.instruments.iter_mut().enumerate() {
            inst.note_on(frequency * self.detune[i], velocity);
        }
    }

    pub fn note_off(&mut self) {
        for inst in &mut self.instruments {
            inst.note_off();
        }
    }

    pub fn set_articulation(&mut self, articulation: WoodwindArticulation) {
        for inst in &mut self.instruments {
            inst.set_articulation(articulation);
        }
    }

    pub fn process(&mut self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for (i, inst) in self.instruments.iter_mut().enumerate() {
            let sample = inst.process();
            let pan = self.pan[i];

            // Equal power panning
            let left_gain = ((1.0 - pan) * 0.5 * PI / 2.0).cos();
            let right_gain = ((1.0 + pan) * 0.5 * PI / 2.0).cos();

            left += sample * left_gain;
            right += sample * right_gain;
        }

        let count = self.instruments.len() as f32;
        (left / count.sqrt(), right / count.sqrt())
    }

    pub fn is_active(&self) -> bool {
        self.instruments.iter().any(|i| i.is_active())
    }
}

/// Simple random number generator for noise
fn rand_simple() -> f32 {
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u32> = const { Cell::new(12345) };
    }
    SEED.with(|seed| {
        let mut s = seed.get();
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        seed.set(s);
        (s as f32) / (u32::MAX as f32)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_woodwind_creation() {
        let sample_rate = 44100.0;

        let flute = WoodwindSynth::flute(sample_rate);
        assert!(flute.air_jet.is_some());
        assert!(flute.single_reed.is_none());
        assert!(flute.double_reed.is_none());

        let clarinet = WoodwindSynth::clarinet(sample_rate);
        assert!(clarinet.air_jet.is_none());
        assert!(clarinet.single_reed.is_some());
        assert!(clarinet.double_reed.is_none());

        let oboe = WoodwindSynth::oboe(sample_rate);
        assert!(oboe.air_jet.is_none());
        assert!(oboe.single_reed.is_none());
        assert!(oboe.double_reed.is_some());
    }

    #[test]
    fn test_woodwind_sound() {
        let sample_rate = 44100.0;
        let mut flute = WoodwindSynth::flute(sample_rate);

        flute.note_on(440.0, 0.8);

        // Generate samples
        let mut samples = Vec::new();
        for _ in 0..4410 {
            samples.push(flute.process());
        }

        // Should produce non-zero output
        let max_amp = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_amp > 0.01);

        flute.note_off();

        // Should fade out
        for _ in 0..44100 {
            flute.process();
        }
        assert!(!flute.is_active());
    }

    #[test]
    fn test_woodwind_section() {
        let sample_rate = 44100.0;
        let mut clarinets = WoodwindSection::clarinets(3, sample_rate);

        clarinets.note_on(440.0, 0.7);

        // Generate stereo samples
        let mut left_samples = Vec::new();
        let mut right_samples = Vec::new();
        for _ in 0..4410 {
            let (l, r) = clarinets.process();
            left_samples.push(l);
            right_samples.push(r);
        }

        // Both channels should have output
        let max_left = left_samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        let max_right = right_samples
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);
        assert!(max_left > 0.01);
        assert!(max_right > 0.01);
    }

    #[test]
    fn test_articulations() {
        let sample_rate = 44100.0;
        let mut sax = WoodwindSynth::alto_sax(sample_rate);

        let articulations = [
            WoodwindArticulation::Normal,
            WoodwindArticulation::Staccato,
            WoodwindArticulation::Legato,
            WoodwindArticulation::FlutterTongue,
            WoodwindArticulation::Growl,
        ];

        for art in articulations {
            sax.set_articulation(art);
            sax.note_on(440.0, 0.8);

            for _ in 0..1000 {
                sax.process();
            }

            sax.note_off();

            // Let it release
            for _ in 0..10000 {
                sax.process();
            }
        }
    }

    #[test]
    fn test_all_instruments() {
        let sample_rate = 44100.0;
        let instruments = [
            WoodwindInstrument::Piccolo,
            WoodwindInstrument::Flute,
            WoodwindInstrument::BbClarinet,
            WoodwindInstrument::AltoSax,
            WoodwindInstrument::Oboe,
            WoodwindInstrument::Bassoon,
        ];

        for inst in instruments {
            let mut synth = WoodwindSynth::new(inst, sample_rate);
            synth.note_on(inst.lowest_frequency() * 1.5, 0.7);

            let mut had_sound = false;
            for _ in 0..4410 {
                if synth.process().abs() > 0.001 {
                    had_sound = true;
                }
            }
            assert!(had_sound, "{:?} should produce sound", inst);
        }
    }
}
