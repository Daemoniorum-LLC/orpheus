//! Orchestral percussion synthesizer using physical and modal synthesis
//!
//! Implements realistic percussion synthesis with:
//! - Timpani (membrane vibration with air coupling)
//! - Mallet instruments (modal synthesis: xylophone, marimba, vibraphone, glockenspiel)
//! - Metallic percussion (cymbals, triangle, tam-tam, tubular bells)
//! - Drums (bass drum, snare drum)

use std::f32::consts::PI;

/// Percussion instrument categories
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PercussionType {
    Timpani,
    MalletPitched,
    MalletMetallic,
    Cymbal,
    Drum,
}

/// Specific percussion instruments
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PercussionInstrument {
    // Timpani (pitched drums)
    Timpani20,  // 20" - highest
    Timpani23,  // 23"
    Timpani26,  // 26"
    Timpani29,  // 29"
    Timpani32,  // 32" - lowest

    // Mallet pitched (wood/synthetic bars)
    Xylophone,
    Marimba,
    Vibraphone,
    Glockenspiel,

    // Mallet metallic
    TubularBells,
    Crotales,

    // Cymbals
    CrashCymbal,
    SuspendedCymbal,
    HiHat,
    SizzleCymbal,
    ChineseCymbal,
    RideCymbal,
    SplashCymbal,

    // Gongs
    TamTam,
    Gong,

    // Other metallic
    Triangle,
    FingerCymbals,
    Anvil,
    Bell,
    Chimes,

    // Drums
    BassDrum,
    ConcertSnareDrum,
    TenorDrum,
    TomTom,
    Bongos,
    Congas,
    Timbales,

    // Auxiliary
    Tambourine,
    Castanets,
    WoodBlock,
    TempleBlocks,
    Claves,
    Maracas,
    Guiro,
    Cabasa,
    Shaker,
    Sleighbells,
    Whip,
    Ratchet,
    Slapstick,
    Vibraslap,
    Windchimes,
}

impl PercussionInstrument {
    /// Get the percussion type category
    pub fn category(&self) -> PercussionType {
        match self {
            Self::Timpani20
            | Self::Timpani23
            | Self::Timpani26
            | Self::Timpani29
            | Self::Timpani32 => PercussionType::Timpani,

            Self::Xylophone | Self::Marimba | Self::Vibraphone | Self::Glockenspiel => {
                PercussionType::MalletPitched
            }

            Self::TubularBells | Self::Crotales => PercussionType::MalletMetallic,

            Self::CrashCymbal
            | Self::SuspendedCymbal
            | Self::HiHat
            | Self::SizzleCymbal
            | Self::ChineseCymbal
            | Self::RideCymbal
            | Self::SplashCymbal
            | Self::TamTam
            | Self::Gong
            | Self::Triangle
            | Self::FingerCymbals
            | Self::Anvil
            | Self::Bell
            | Self::Chimes => PercussionType::Cymbal,

            _ => PercussionType::Drum,
        }
    }

    /// Get decay time in seconds
    pub fn decay_time(&self) -> f32 {
        match self {
            Self::Timpani20 | Self::Timpani23 => 2.0,
            Self::Timpani26 | Self::Timpani29 => 2.5,
            Self::Timpani32 => 3.0,
            Self::Xylophone => 0.8,
            Self::Marimba => 1.5,
            Self::Vibraphone => 4.0,
            Self::Glockenspiel => 3.0,
            Self::TubularBells => 6.0,
            Self::Crotales => 5.0,
            Self::CrashCymbal => 3.0,
            Self::SuspendedCymbal => 8.0,
            Self::HiHat => 0.3,
            Self::SizzleCymbal => 5.0,
            Self::RideCymbal => 4.0,
            Self::SplashCymbal => 1.5,
            Self::ChineseCymbal => 2.5,
            Self::TamTam => 10.0,
            Self::Gong => 12.0,
            Self::Triangle => 3.0,
            Self::FingerCymbals => 2.0,
            Self::Anvil => 0.5,
            Self::Bell => 5.0,
            Self::Chimes => 4.0,
            Self::BassDrum => 1.0,
            Self::ConcertSnareDrum => 0.3,
            Self::TenorDrum => 0.5,
            Self::TomTom => 0.6,
            Self::Bongos => 0.4,
            Self::Congas => 0.5,
            Self::Timbales => 0.4,
            Self::Tambourine => 1.0,
            Self::Castanets => 0.1,
            Self::WoodBlock => 0.15,
            Self::TempleBlocks => 0.3,
            Self::Claves => 0.1,
            Self::Maracas => 0.2,
            Self::Guiro => 0.5,
            Self::Cabasa => 0.3,
            Self::Shaker => 0.2,
            Self::Sleighbells => 1.5,
            Self::Whip => 0.05,
            Self::Ratchet => 0.5,
            Self::Slapstick => 0.05,
            Self::Vibraslap => 1.0,
            Self::Windchimes => 3.0,
        }
    }

    /// Get characteristic brightness (affects harmonic content)
    pub fn brightness(&self) -> f32 {
        match self {
            Self::Glockenspiel | Self::Crotales | Self::Triangle => 0.95,
            Self::Xylophone | Self::TubularBells => 0.85,
            Self::Vibraphone | Self::CrashCymbal | Self::SplashCymbal => 0.75,
            Self::Marimba | Self::SuspendedCymbal | Self::RideCymbal => 0.6,
            Self::Timpani20 | Self::Timpani23 => 0.5,
            Self::Timpani26 | Self::ChineseCymbal => 0.45,
            Self::Timpani29 | Self::TamTam => 0.4,
            Self::Timpani32 | Self::Gong | Self::BassDrum => 0.3,
            _ => 0.5,
        }
    }
}

/// Mallet/stick types affect attack and tone
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MalletType {
    // Timpani mallets
    TimpaniSoft,
    TimpaniMedium,
    TimpaniHard,
    TimpaniWood,

    // Keyboard percussion
    YarnSoft,
    YarnMedium,
    YarnHard,
    RubberSoft,
    RubberMedium,
    RubberHard,
    Plastic,
    Brass,

    // Drum sticks
    DrumstickWood,
    DrumstickNylon,
    Brush,
    Rods,
    Mallet,

    // Cymbal/gong
    CymbalMallet,
    GongMallet,
    SuperBall,
    Bow,

    // Hand
    Hand,
    Fingers,
}

impl MalletType {
    /// Get attack hardness (affects transient and harmonics)
    pub fn hardness(&self) -> f32 {
        match self {
            Self::TimpaniSoft | Self::YarnSoft | Self::RubberSoft | Self::Brush => 0.2,
            Self::TimpaniMedium | Self::YarnMedium | Self::RubberMedium | Self::Hand => 0.4,
            Self::TimpaniHard | Self::YarnHard | Self::RubberHard | Self::Mallet => 0.6,
            Self::Plastic | Self::DrumstickNylon | Self::CymbalMallet | Self::GongMallet => 0.7,
            Self::TimpaniWood | Self::DrumstickWood | Self::Rods => 0.8,
            Self::Brass | Self::Fingers => 0.9,
            Self::SuperBall => 0.3,
            Self::Bow => 0.1,
        }
    }

    /// Get attack sharpness (affects envelope)
    pub fn attack_time(&self) -> f32 {
        match self {
            Self::Bow => 0.1,
            Self::SuperBall => 0.05,
            Self::TimpaniSoft | Self::YarnSoft | Self::Brush => 0.008,
            Self::TimpaniMedium | Self::YarnMedium | Self::RubberSoft => 0.005,
            Self::TimpaniHard | Self::YarnHard | Self::RubberMedium | Self::Hand => 0.003,
            Self::TimpaniWood
            | Self::RubberHard
            | Self::Mallet
            | Self::CymbalMallet
            | Self::GongMallet => 0.002,
            Self::Plastic | Self::DrumstickNylon | Self::Rods | Self::Fingers => 0.001,
            Self::DrumstickWood | Self::Brass => 0.0005,
        }
    }
}

/// Timpani synthesizer using membrane vibration model
#[derive(Debug, Clone)]
pub struct TimpaniSynth {
    /// Modal frequencies and amplitudes
    modes: Vec<TimpaniMode>,
    /// Membrane tension (affects pitch)
    tension: f32,
    /// Head diameter in inches
    diameter: f32,
    /// Current frequency
    frequency: f32,
    /// Envelope
    envelope: PercussionEnvelope,
    /// Mallet type
    mallet: MalletType,
    /// Strike position (0 = center, 1 = edge)
    strike_position: f32,
    /// Sample rate
    sample_rate: f32,
}

/// Single timpani mode
#[derive(Debug, Clone)]
struct TimpaniMode {
    /// Frequency ratio to fundamental
    ratio: f32,
    /// Current phase
    phase: f32,
    /// Amplitude
    amplitude: f32,
    /// Decay rate
    decay: f32,
    /// Current level
    level: f32,
}

impl TimpaniSynth {
    pub fn new(diameter: f32, sample_rate: f32) -> Self {
        // Timpani has inharmonic modes due to air coupling
        // Approximate mode ratios for a kettle drum
        let mode_ratios = [
            (1.0, 1.0),      // Fundamental (01 mode)
            (1.504, 0.8),    // 11 mode
            (1.741, 0.5),    // 21 mode
            (2.0, 0.6),      // 02 mode
            (2.295, 0.35),   // 31 mode
            (2.653, 0.25),   // 12 mode
            (2.917, 0.2),    // 41 mode
            (3.156, 0.15),   // 22 mode
            (3.501, 0.1),    // 03 mode
        ];

        let modes = mode_ratios
            .iter()
            .map(|(ratio, amp)| TimpaniMode {
                ratio: *ratio,
                phase: 0.0,
                amplitude: *amp,
                decay: 0.0,
                level: 0.0,
            })
            .collect();

        Self {
            modes,
            tension: 0.5,
            diameter,
            frequency: 110.0,
            envelope: PercussionEnvelope::new(sample_rate, 0.003, 2.5),
            mallet: MalletType::TimpaniMedium,
            strike_position: 0.3,
            sample_rate,
        }
    }

    /// Create 20" timpani (highest)
    pub fn timpani_20(sample_rate: f32) -> Self {
        Self::new(20.0, sample_rate)
    }

    /// Create 26" timpani (middle)
    pub fn timpani_26(sample_rate: f32) -> Self {
        Self::new(26.0, sample_rate)
    }

    /// Create 32" timpani (lowest)
    pub fn timpani_32(sample_rate: f32) -> Self {
        Self::new(32.0, sample_rate)
    }

    pub fn set_mallet(&mut self, mallet: MalletType) {
        self.mallet = mallet;
        self.envelope = PercussionEnvelope::new(
            self.sample_rate,
            mallet.attack_time(),
            2.0 + self.diameter * 0.05,
        );
    }

    pub fn set_strike_position(&mut self, position: f32) {
        self.strike_position = position.clamp(0.0, 1.0);
    }

    pub fn note_on(&mut self, frequency: f32, velocity: f32) {
        self.frequency = frequency;

        // Calculate decay rates based on frequency and mallet
        let base_decay = 2.0 + (1.0 - self.mallet.hardness()) * 2.0;

        for (i, mode) in self.modes.iter_mut().enumerate() {
            // Phase reset with slight randomization
            mode.phase = rand_simple() * 0.1;

            // Strike position affects which modes are excited
            // Center strike excites symmetric modes, edge excites asymmetric
            let strike_factor = if i % 2 == 0 {
                1.0 - self.strike_position * 0.5
            } else {
                self.strike_position
            };

            // Higher modes decay faster
            mode.decay = base_decay * (1.0 + i as f32 * 0.3);
            mode.level = mode.amplitude * strike_factor * velocity;
        }

        self.envelope.trigger(velocity);
    }

    pub fn process(&mut self) -> f32 {
        if !self.envelope.is_active() {
            return 0.0;
        }

        let env = self.envelope.process();
        let mut output = 0.0;

        for mode in &mut self.modes {
            if mode.level < 0.0001 {
                continue;
            }

            // Calculate mode frequency
            let freq = self.frequency * mode.ratio;

            // Update phase
            mode.phase += freq / self.sample_rate;
            if mode.phase >= 1.0 {
                mode.phase -= 1.0;
            }

            // Generate sine with slight nonlinearity for realism
            let sine = (mode.phase * 2.0 * PI).sin();
            let nonlin = sine + 0.1 * (sine * 2.0).sin(); // Slight harmonic distortion

            output += nonlin * mode.level;

            // Apply per-mode decay
            mode.level *= 1.0 - (mode.decay / self.sample_rate);
        }

        // Apply mallet brightness
        let brightness = self.mallet.hardness();
        output *= 0.5 + brightness * 0.5;

        output * env * 0.3
    }

    pub fn is_active(&self) -> bool {
        self.envelope.is_active()
    }
}

/// Modal synthesis for mallet percussion (xylophone, marimba, vibraphone, etc.)
#[derive(Debug, Clone)]
pub struct MalletSynth {
    /// Instrument type
    instrument: PercussionInstrument,
    /// Modal resonators
    modes: Vec<ModalResonator>,
    /// Current frequency
    frequency: f32,
    /// Envelope
    envelope: PercussionEnvelope,
    /// Mallet type
    mallet: MalletType,
    /// Tremolo (for vibraphone motor)
    tremolo_phase: f32,
    tremolo_rate: f32,
    tremolo_depth: f32,
    tremolo_enabled: bool,
    /// Sample rate
    sample_rate: f32,
}

/// Modal resonator for bar instruments
#[derive(Debug, Clone)]
struct ModalResonator {
    /// Frequency ratio to fundamental
    ratio: f32,
    /// Two-pole resonator state
    y1: f32,
    y2: f32,
    /// Coefficients
    a1: f32,
    a2: f32,
    b0: f32,
    /// Amplitude
    amplitude: f32,
    /// Frequency
    freq: f32,
    /// Q factor (resonance)
    q: f32,
}

impl ModalResonator {
    fn new(ratio: f32, amplitude: f32, q: f32) -> Self {
        Self {
            ratio,
            y1: 0.0,
            y2: 0.0,
            a1: 0.0,
            a2: 0.0,
            b0: 1.0,
            amplitude,
            freq: 440.0,
            q,
        }
    }

    fn set_frequency(&mut self, fundamental: f32, sample_rate: f32) {
        self.freq = fundamental * self.ratio;
        let w0 = 2.0 * PI * self.freq / sample_rate;
        // Q represents number of cycles to decay to 1/e
        // Calculate decay rate per sample
        let decay_samples = self.q * sample_rate / self.freq;
        let r = (-1.0 / decay_samples).exp();
        // Decaying sinusoid: y[n] = 2*r*cos(w)*y[n-1] - r^2*y[n-2]
        self.a1 = 2.0 * r * w0.cos();
        self.a2 = -r * r;
    }

    fn excite(&mut self, impulse: f32) {
        self.y1 += impulse * self.amplitude;
    }

    fn process(&mut self) -> f32 {
        let output = self.y1;
        let new_y = self.a1 * self.y1 + self.a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = new_y;
        output
    }
}

impl MalletSynth {
    pub fn new(instrument: PercussionInstrument, sample_rate: f32) -> Self {
        let modes = match instrument {
            PercussionInstrument::Xylophone => {
                // Xylophone has inharmonic ratios due to bar shape
                vec![
                    ModalResonator::new(1.0, 1.0, 200.0),
                    ModalResonator::new(3.0, 0.5, 180.0),
                    ModalResonator::new(6.0, 0.25, 150.0),
                    ModalResonator::new(10.3, 0.1, 120.0),
                ]
            }
            PercussionInstrument::Marimba => {
                // Marimba has tuned overtones
                vec![
                    ModalResonator::new(1.0, 1.0, 300.0),
                    ModalResonator::new(4.0, 0.6, 250.0),
                    ModalResonator::new(9.2, 0.3, 200.0),
                    ModalResonator::new(15.8, 0.15, 150.0),
                ]
            }
            PercussionInstrument::Vibraphone => {
                // Vibraphone has aluminum bars
                vec![
                    ModalResonator::new(1.0, 1.0, 500.0),
                    ModalResonator::new(4.0, 0.4, 400.0),
                    ModalResonator::new(10.0, 0.15, 300.0),
                    ModalResonator::new(20.0, 0.05, 200.0),
                ]
            }
            PercussionInstrument::Glockenspiel => {
                // Glockenspiel has steel bars
                vec![
                    ModalResonator::new(1.0, 1.0, 800.0),
                    ModalResonator::new(2.76, 0.3, 600.0),
                    ModalResonator::new(5.40, 0.15, 400.0),
                    ModalResonator::new(8.93, 0.05, 300.0),
                ]
            }
            PercussionInstrument::TubularBells => {
                // Tubular bells have complex inharmonic spectrum
                vec![
                    ModalResonator::new(1.0, 1.0, 1000.0),
                    ModalResonator::new(2.0, 0.8, 800.0),
                    ModalResonator::new(2.5, 0.4, 600.0),
                    ModalResonator::new(3.0, 0.3, 500.0),
                    ModalResonator::new(4.2, 0.2, 400.0),
                    ModalResonator::new(5.4, 0.1, 300.0),
                ]
            }
            PercussionInstrument::Crotales => {
                // Crotales (antique cymbals)
                vec![
                    ModalResonator::new(1.0, 1.0, 1200.0),
                    ModalResonator::new(3.0, 0.3, 900.0),
                    ModalResonator::new(6.0, 0.1, 600.0),
                ]
            }
            _ => {
                vec![ModalResonator::new(1.0, 1.0, 200.0)]
            }
        };

        let decay = instrument.decay_time();

        Self {
            instrument,
            modes,
            frequency: 440.0,
            envelope: PercussionEnvelope::new(sample_rate, 0.002, decay),
            mallet: MalletType::YarnMedium,
            tremolo_phase: 0.0,
            tremolo_rate: 6.0, // Hz
            tremolo_depth: 0.5,
            tremolo_enabled: false,
            sample_rate,
        }
    }

    /// Factory for xylophone
    pub fn xylophone(sample_rate: f32) -> Self {
        let mut synth = Self::new(PercussionInstrument::Xylophone, sample_rate);
        synth.mallet = MalletType::RubberHard;
        synth
    }

    /// Factory for marimba
    pub fn marimba(sample_rate: f32) -> Self {
        let mut synth = Self::new(PercussionInstrument::Marimba, sample_rate);
        synth.mallet = MalletType::YarnMedium;
        synth
    }

    /// Factory for vibraphone
    pub fn vibraphone(sample_rate: f32) -> Self {
        let mut synth = Self::new(PercussionInstrument::Vibraphone, sample_rate);
        synth.mallet = MalletType::YarnSoft;
        synth.tremolo_enabled = true;
        synth
    }

    /// Factory for glockenspiel
    pub fn glockenspiel(sample_rate: f32) -> Self {
        let mut synth = Self::new(PercussionInstrument::Glockenspiel, sample_rate);
        synth.mallet = MalletType::Brass;
        synth
    }

    /// Factory for tubular bells
    pub fn tubular_bells(sample_rate: f32) -> Self {
        let mut synth = Self::new(PercussionInstrument::TubularBells, sample_rate);
        synth.mallet = MalletType::RubberHard;
        synth
    }

    pub fn set_mallet(&mut self, mallet: MalletType) {
        self.mallet = mallet;
        self.envelope = PercussionEnvelope::new(
            self.sample_rate,
            mallet.attack_time(),
            self.instrument.decay_time(),
        );
    }

    pub fn set_tremolo(&mut self, enabled: bool, rate: f32, depth: f32) {
        self.tremolo_enabled = enabled;
        self.tremolo_rate = rate;
        self.tremolo_depth = depth.clamp(0.0, 1.0);
    }

    pub fn note_on(&mut self, frequency: f32, velocity: f32) {
        self.frequency = frequency;

        // Update mode frequencies
        for mode in &mut self.modes {
            mode.set_frequency(frequency, self.sample_rate);
        }

        // Excite modes based on mallet hardness
        let hardness = self.mallet.hardness();
        for (i, mode) in self.modes.iter_mut().enumerate() {
            // Harder mallets excite more high frequency modes
            let excitation = velocity * (1.0 - i as f32 * 0.2 * (1.0 - hardness));
            mode.excite(excitation.max(0.0));
        }

        self.envelope.trigger(velocity);
    }

    pub fn note_off(&mut self) {
        // For most mallet instruments, note off means damping
        self.envelope.release();
    }

    pub fn process(&mut self) -> f32 {
        if !self.envelope.is_active() {
            return 0.0;
        }

        let env = self.envelope.process();
        let mut output = 0.0;

        for mode in &mut self.modes {
            output += mode.process();
        }

        // Apply tremolo for vibraphone
        if self.tremolo_enabled {
            self.tremolo_phase += self.tremolo_rate / self.sample_rate;
            if self.tremolo_phase >= 1.0 {
                self.tremolo_phase -= 1.0;
            }
            let tremolo = 1.0 - self.tremolo_depth * 0.5 * (1.0 + (self.tremolo_phase * 2.0 * PI).sin());
            output *= tremolo;
        }

        output * env * 0.2
    }

    pub fn is_active(&self) -> bool {
        self.envelope.is_active()
    }
}

/// Cymbal synthesizer using stochastic modal synthesis
#[derive(Debug, Clone)]
pub struct CymbalSynth {
    /// Instrument type
    instrument: PercussionInstrument,
    /// Modal bank (many inharmonic modes)
    modes: Vec<CymbalMode>,
    /// Noise component
    noise_filter: BandpassNoise,
    /// Envelope
    envelope: PercussionEnvelope,
    /// Choke state
    choked: bool,
    choke_rate: f32,
    /// Sizzle amount (for sizzle cymbal)
    sizzle: f32,
    /// Sample rate
    sample_rate: f32,
}

#[derive(Debug, Clone)]
struct CymbalMode {
    freq: f32,
    phase: f32,
    amplitude: f32,
    decay: f32,
    level: f32,
}

#[derive(Debug, Clone)]
struct BandpassNoise {
    state1: f32,
    state2: f32,
    center: f32,
    bandwidth: f32,
    sample_rate: f32,
}

impl BandpassNoise {
    fn new(center: f32, bandwidth: f32, sample_rate: f32) -> Self {
        Self {
            state1: 0.0,
            state2: 0.0,
            center,
            bandwidth,
            sample_rate,
        }
    }

    fn process(&mut self) -> f32 {
        let noise = rand_simple() * 2.0 - 1.0;

        let w0 = 2.0 * PI * self.center / self.sample_rate;
        let q = self.center / self.bandwidth;
        let alpha = w0.sin() / (2.0 * q);

        let b0 = alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;

        let output = (b0 / a0) * noise + self.state1;
        self.state1 = -(a1 / a0) * output + self.state2;
        self.state2 = (b0 / a0) * noise - (a2 / a0) * output;

        output
    }
}

impl CymbalSynth {
    pub fn new(instrument: PercussionInstrument, sample_rate: f32) -> Self {
        // Generate many inharmonic modes for cymbal sound
        let mode_count = match instrument {
            PercussionInstrument::Triangle => 8,
            PercussionInstrument::FingerCymbals | PercussionInstrument::SplashCymbal => 20,
            PercussionInstrument::HiHat => 25,
            PercussionInstrument::CrashCymbal | PercussionInstrument::RideCymbal => 40,
            PercussionInstrument::SuspendedCymbal | PercussionInstrument::ChineseCymbal => 50,
            PercussionInstrument::TamTam | PercussionInstrument::Gong => 80,
            _ => 30,
        };

        let base_freq = match instrument {
            PercussionInstrument::Triangle => 2000.0,
            PercussionInstrument::FingerCymbals => 3000.0,
            PercussionInstrument::HiHat => 400.0,
            PercussionInstrument::SplashCymbal => 800.0,
            PercussionInstrument::CrashCymbal => 500.0,
            PercussionInstrument::RideCymbal => 350.0,
            PercussionInstrument::SuspendedCymbal => 300.0,
            PercussionInstrument::ChineseCymbal => 450.0,
            PercussionInstrument::TamTam => 80.0,
            PercussionInstrument::Gong => 100.0,
            _ => 500.0,
        };

        let modes: Vec<CymbalMode> = (0..mode_count)
            .map(|i| {
                // Inharmonic frequency distribution
                let ratio = 1.0 + (i as f32).powf(1.4) * 0.15;
                let freq = base_freq * ratio * (1.0 + rand_simple() * 0.05);
                CymbalMode {
                    freq,
                    phase: rand_simple(),
                    amplitude: 1.0 / (1.0 + i as f32 * 0.3),
                    decay: 0.0,
                    level: 0.0,
                }
            })
            .collect();

        let noise_center = base_freq * 2.0;
        let noise_bw = base_freq * 3.0;

        Self {
            instrument,
            modes,
            noise_filter: BandpassNoise::new(noise_center, noise_bw, sample_rate),
            envelope: PercussionEnvelope::new(sample_rate, 0.001, instrument.decay_time()),
            choked: false,
            choke_rate: 0.0,
            sizzle: 0.0,
            sample_rate,
        }
    }

    /// Factory for crash cymbal
    pub fn crash(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::CrashCymbal, sample_rate)
    }

    /// Factory for suspended cymbal
    pub fn suspended(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::SuspendedCymbal, sample_rate)
    }

    /// Factory for hi-hat
    pub fn hi_hat(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::HiHat, sample_rate)
    }

    /// Factory for ride cymbal
    pub fn ride(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::RideCymbal, sample_rate)
    }

    /// Factory for triangle
    pub fn triangle(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::Triangle, sample_rate)
    }

    /// Factory for tam-tam
    pub fn tam_tam(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::TamTam, sample_rate)
    }

    pub fn set_sizzle(&mut self, amount: f32) {
        self.sizzle = amount.clamp(0.0, 1.0);
    }

    pub fn strike(&mut self, velocity: f32) {
        self.choked = false;
        self.choke_rate = 0.0;

        let decay_time = self.instrument.decay_time();

        for mode in &mut self.modes {
            mode.phase = rand_simple();
            // Random variation in decay
            mode.decay = decay_time * (0.7 + rand_simple() * 0.6);
            mode.level = mode.amplitude * velocity;
        }

        self.envelope.trigger(velocity);
    }

    pub fn choke(&mut self) {
        self.choked = true;
        self.choke_rate = 0.995; // Quick damping
    }

    pub fn process(&mut self) -> f32 {
        if !self.envelope.is_active() {
            return 0.0;
        }

        let env = self.envelope.process();
        let mut output = 0.0;

        for mode in &mut self.modes {
            if mode.level < 0.0001 {
                continue;
            }

            mode.phase += mode.freq / self.sample_rate;
            if mode.phase >= 1.0 {
                mode.phase -= 1.0;
            }

            output += (mode.phase * 2.0 * PI).sin() * mode.level;

            // Natural decay
            mode.level *= 1.0 - (1.0 / (mode.decay * self.sample_rate));

            // Choke damping
            if self.choked {
                mode.level *= self.choke_rate;
            }
        }

        // Add noise component
        let noise = self.noise_filter.process() * env * 0.3;

        // Sizzle effect (high frequency flutter)
        let sizzle = if self.sizzle > 0.0 {
            let sizzle_noise = rand_simple() * 2.0 - 1.0;
            sizzle_noise * self.sizzle * 0.1 * env
        } else {
            0.0
        };

        (output * 0.15 + noise + sizzle) * env
    }

    pub fn is_active(&self) -> bool {
        self.envelope.is_active()
    }
}

/// Drum synthesizer for bass drum, snare, etc.
#[derive(Debug, Clone)]
pub struct DrumSynth {
    /// Instrument type
    instrument: PercussionInstrument,
    /// Membrane oscillator
    membrane_phase: f32,
    membrane_freq: f32,
    membrane_decay: f32,
    /// Pitch envelope (drums have pitch drop on attack)
    pitch_env: f32,
    pitch_env_decay: f32,
    pitch_env_amount: f32,
    /// Shell resonance
    shell_freq: f32,
    shell_state: f32,
    /// Snare buzz (for snare drum)
    snare_enabled: bool,
    snare_amount: f32,
    snare_state: f32,
    /// Noise filter for attack
    noise_state1: f32,
    noise_state2: f32,
    /// Envelope
    envelope: PercussionEnvelope,
    /// Sample rate
    sample_rate: f32,
}

impl DrumSynth {
    pub fn new(instrument: PercussionInstrument, sample_rate: f32) -> Self {
        let (membrane_freq, shell_freq, pitch_amount, snare) = match instrument {
            PercussionInstrument::BassDrum => (60.0, 80.0, 0.5, false),
            PercussionInstrument::ConcertSnareDrum => (200.0, 350.0, 0.2, true),
            PercussionInstrument::TenorDrum => (120.0, 200.0, 0.3, false),
            PercussionInstrument::TomTom => (150.0, 250.0, 0.25, false),
            PercussionInstrument::Bongos => (300.0, 500.0, 0.15, false),
            PercussionInstrument::Congas => (180.0, 300.0, 0.2, false),
            PercussionInstrument::Timbales => (250.0, 400.0, 0.15, false),
            _ => (200.0, 350.0, 0.2, false),
        };

        Self {
            instrument,
            membrane_phase: 0.0,
            membrane_freq,
            membrane_decay: instrument.decay_time(),
            pitch_env: 0.0,
            pitch_env_decay: 0.01,
            pitch_env_amount: pitch_amount,
            shell_freq,
            shell_state: 0.0,
            snare_enabled: snare,
            snare_amount: if snare { 0.4 } else { 0.0 },
            snare_state: 0.0,
            noise_state1: 0.0,
            noise_state2: 0.0,
            envelope: PercussionEnvelope::new(sample_rate, 0.001, instrument.decay_time()),
            sample_rate,
        }
    }

    /// Factory for bass drum
    pub fn bass_drum(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::BassDrum, sample_rate)
    }

    /// Factory for concert snare
    pub fn snare_drum(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::ConcertSnareDrum, sample_rate)
    }

    /// Factory for tom-tom
    pub fn tom_tom(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::TomTom, sample_rate)
    }

    pub fn set_snare(&mut self, amount: f32) {
        self.snare_amount = amount.clamp(0.0, 1.0);
        self.snare_enabled = amount > 0.0;
    }

    pub fn set_tuning(&mut self, frequency: f32) {
        self.membrane_freq = frequency;
    }

    pub fn strike(&mut self, velocity: f32) {
        self.membrane_phase = 0.0;
        self.pitch_env = 1.0;
        self.envelope.trigger(velocity);
    }

    pub fn process(&mut self) -> f32 {
        if !self.envelope.is_active() {
            return 0.0;
        }

        let env = self.envelope.process();

        // Pitch envelope (drums drop in pitch after attack)
        let current_freq = self.membrane_freq * (1.0 + self.pitch_env * self.pitch_env_amount);
        self.pitch_env *= 1.0 - self.pitch_env_decay;

        // Membrane oscillation
        self.membrane_phase += current_freq / self.sample_rate;
        if self.membrane_phase >= 1.0 {
            self.membrane_phase -= 1.0;
        }

        let membrane = (self.membrane_phase * 2.0 * PI).sin();

        // Shell resonance (filtered)
        let shell_input = membrane * 0.3;
        let shell_w = 2.0 * PI * self.shell_freq / self.sample_rate;
        self.shell_state = self.shell_state * 0.95 + shell_input * shell_w.sin();

        // Attack noise
        let noise = rand_simple() * 2.0 - 1.0;
        let attack_noise = self.pitch_env * noise * 0.3;

        // Snare buzz
        let snare = if self.snare_enabled && env > 0.1 {
            let snare_input = membrane.abs() * self.snare_amount;
            let snare_noise = rand_simple() * 2.0 - 1.0;

            // Bandpass filter for snare character
            let snare_w = 2.0 * PI * 1500.0 / self.sample_rate;
            self.snare_state = self.snare_state * 0.9 + snare_noise * snare_input * snare_w;
            self.snare_state * 0.5
        } else {
            0.0
        };

        (membrane * 0.6 + self.shell_state * 0.2 + attack_noise + snare) * env * 0.4
    }

    pub fn is_active(&self) -> bool {
        self.envelope.is_active()
    }
}

/// Auxiliary percussion (tambourine, woodblock, etc.)
#[derive(Debug, Clone)]
pub struct AuxPercussionSynth {
    instrument: PercussionInstrument,
    /// Modal resonators
    modes: Vec<f32>,
    mode_phases: Vec<f32>,
    mode_decays: Vec<f32>,
    mode_levels: Vec<f32>,
    /// Noise component
    noise_level: f32,
    noise_filter_state: f32,
    /// Jingle decay (for tambourine, sleighbells)
    jingle_phase: f32,
    jingle_rate: f32,
    /// Envelope
    envelope: PercussionEnvelope,
    sample_rate: f32,
}

impl AuxPercussionSynth {
    pub fn new(instrument: PercussionInstrument, sample_rate: f32) -> Self {
        let (modes, noise_level, jingle_rate) = match instrument {
            PercussionInstrument::Tambourine => {
                (vec![800.0, 1200.0, 2000.0, 3500.0], 0.4, 15.0)
            }
            PercussionInstrument::Castanets => {
                (vec![2000.0, 3500.0, 5000.0], 0.2, 0.0)
            }
            PercussionInstrument::WoodBlock => {
                (vec![800.0, 1600.0, 2400.0], 0.1, 0.0)
            }
            PercussionInstrument::TempleBlocks => {
                (vec![400.0, 800.0, 1200.0, 1600.0], 0.1, 0.0)
            }
            PercussionInstrument::Claves => {
                (vec![2500.0, 4000.0], 0.05, 0.0)
            }
            PercussionInstrument::Maracas => {
                (vec![], 0.8, 0.0)
            }
            PercussionInstrument::Shaker => {
                (vec![], 0.9, 0.0)
            }
            PercussionInstrument::Sleighbells => {
                (vec![3000.0, 4500.0, 6000.0, 8000.0], 0.3, 20.0)
            }
            PercussionInstrument::Whip => {
                (vec![], 0.95, 0.0)
            }
            PercussionInstrument::Slapstick => {
                (vec![500.0, 1000.0], 0.5, 0.0)
            }
            PercussionInstrument::Windchimes => {
                (vec![1000.0, 1500.0, 2000.0, 2500.0, 3000.0, 3500.0, 4000.0], 0.1, 0.0)
            }
            _ => (vec![1000.0], 0.3, 0.0),
        };

        let mode_count = modes.len();

        Self {
            instrument,
            modes,
            mode_phases: vec![0.0; mode_count],
            mode_decays: vec![instrument.decay_time(); mode_count],
            mode_levels: vec![0.0; mode_count],
            noise_level,
            noise_filter_state: 0.0,
            jingle_phase: 0.0,
            jingle_rate,
            envelope: PercussionEnvelope::new(sample_rate, 0.001, instrument.decay_time()),
            sample_rate,
        }
    }

    /// Factory for tambourine
    pub fn tambourine(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::Tambourine, sample_rate)
    }

    /// Factory for woodblock
    pub fn woodblock(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::WoodBlock, sample_rate)
    }

    /// Factory for claves
    pub fn claves(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::Claves, sample_rate)
    }

    /// Factory for shaker
    pub fn shaker(sample_rate: f32) -> Self {
        Self::new(PercussionInstrument::Shaker, sample_rate)
    }

    pub fn strike(&mut self, velocity: f32) {
        for (i, phase) in self.mode_phases.iter_mut().enumerate() {
            *phase = rand_simple() * 0.1;
            self.mode_levels[i] = velocity * (1.0 - i as f32 * 0.15).max(0.2);
        }

        self.envelope.trigger(velocity);
    }

    pub fn process(&mut self) -> f32 {
        if !self.envelope.is_active() {
            return 0.0;
        }

        let env = self.envelope.process();
        let mut output = 0.0;

        // Modal resonance
        for i in 0..self.modes.len() {
            if self.mode_levels[i] < 0.001 {
                continue;
            }

            self.mode_phases[i] += self.modes[i] / self.sample_rate;
            if self.mode_phases[i] >= 1.0 {
                self.mode_phases[i] -= 1.0;
            }

            output += (self.mode_phases[i] * 2.0 * PI).sin() * self.mode_levels[i];
            self.mode_levels[i] *= 1.0 - (1.0 / (self.mode_decays[i] * self.sample_rate));
        }

        // Noise component
        if self.noise_level > 0.0 {
            let noise = rand_simple() * 2.0 - 1.0;
            // High pass filter for crisp noise
            let filtered = noise - self.noise_filter_state;
            self.noise_filter_state = noise * 0.95;
            output += filtered * self.noise_level * env;
        }

        // Jingle modulation (tambourine, sleighbells)
        if self.jingle_rate > 0.0 {
            self.jingle_phase += self.jingle_rate / self.sample_rate;
            if self.jingle_phase >= 1.0 {
                self.jingle_phase -= 1.0;
            }
            let jingle = (self.jingle_phase * 2.0 * PI).sin().abs();
            output *= 0.7 + jingle * 0.3;
        }

        output * env * 0.3
    }

    pub fn is_active(&self) -> bool {
        self.envelope.is_active()
    }
}

/// Percussion envelope generator
#[derive(Debug, Clone)]
pub struct PercussionEnvelope {
    value: f32,
    target: f32,
    attack_rate: f32,
    decay_rate: f32,
    state: EnvState,
    sample_rate: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EnvState {
    Idle,
    Attack,
    Decay,
    Release,
}

impl PercussionEnvelope {
    pub fn new(sample_rate: f32, attack_time: f32, decay_time: f32) -> Self {
        Self {
            value: 0.0,
            target: 0.0,
            attack_rate: 1.0 / (attack_time * sample_rate).max(1.0),
            decay_rate: 1.0 / (decay_time * sample_rate).max(1.0),
            state: EnvState::Idle,
            sample_rate,
        }
    }

    pub fn trigger(&mut self, velocity: f32) {
        self.target = velocity;
        self.state = EnvState::Attack;
    }

    pub fn release(&mut self) {
        self.state = EnvState::Release;
    }

    pub fn process(&mut self) -> f32 {
        match self.state {
            EnvState::Idle => {}
            EnvState::Attack => {
                self.value += self.attack_rate * self.target;
                if self.value >= self.target {
                    self.value = self.target;
                    self.state = EnvState::Decay;
                }
            }
            EnvState::Decay => {
                self.value -= self.decay_rate * self.value;
                if self.value < 0.001 {
                    self.value = 0.0;
                    self.state = EnvState::Idle;
                }
            }
            EnvState::Release => {
                self.value *= 0.99; // Quick release
                if self.value < 0.001 {
                    self.value = 0.0;
                    self.state = EnvState::Idle;
                }
            }
        }
        self.value
    }

    pub fn is_active(&self) -> bool {
        self.state != EnvState::Idle || self.value > 0.001
    }
}

/// Simple random number generator
fn rand_simple() -> f32 {
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u32> = const { Cell::new(98765) };
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
    fn test_timpani() {
        let sample_rate = 44100.0;
        let mut timpani = TimpaniSynth::timpani_26(sample_rate);

        timpani.note_on(110.0, 0.8);

        let mut samples = Vec::new();
        for _ in 0..4410 {
            samples.push(timpani.process());
        }

        let max_amp = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_amp > 0.01, "Timpani should produce sound");
    }

    #[test]
    fn test_mallet_instruments() {
        let sample_rate = 44100.0;

        let instruments = [
            MalletSynth::xylophone(sample_rate),
            MalletSynth::marimba(sample_rate),
            MalletSynth::vibraphone(sample_rate),
            MalletSynth::glockenspiel(sample_rate),
            MalletSynth::tubular_bells(sample_rate),
        ];

        for mut inst in instruments {
            inst.note_on(440.0, 0.8);

            let mut had_sound = false;
            for _ in 0..4410 {
                if inst.process().abs() > 0.001 {
                    had_sound = true;
                }
            }
            assert!(had_sound, "Mallet instrument should produce sound");
        }
    }

    #[test]
    fn test_cymbals() {
        let sample_rate = 44100.0;

        let mut crash = CymbalSynth::crash(sample_rate);
        crash.strike(0.9);

        let mut samples = Vec::new();
        for _ in 0..4410 {
            samples.push(crash.process());
        }

        let max_amp = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_amp > 0.01, "Crash cymbal should produce sound");

        // Test choke
        crash.strike(0.9);
        for _ in 0..1000 {
            crash.process();
        }
        crash.choke();
        for _ in 0..1000 {
            crash.process();
        }
    }

    #[test]
    fn test_drums() {
        let sample_rate = 44100.0;

        let mut bass = DrumSynth::bass_drum(sample_rate);
        bass.strike(0.9);

        let mut had_sound = false;
        for _ in 0..4410 {
            if bass.process().abs() > 0.01 {
                had_sound = true;
            }
        }
        assert!(had_sound, "Bass drum should produce sound");

        let mut snare = DrumSynth::snare_drum(sample_rate);
        snare.strike(0.8);

        for _ in 0..4410 {
            snare.process();
        }
    }

    #[test]
    fn test_aux_percussion() {
        let sample_rate = 44100.0;

        let instruments = [
            AuxPercussionSynth::tambourine(sample_rate),
            AuxPercussionSynth::woodblock(sample_rate),
            AuxPercussionSynth::claves(sample_rate),
            AuxPercussionSynth::shaker(sample_rate),
        ];

        for mut inst in instruments {
            inst.strike(0.8);

            let mut had_sound = false;
            for _ in 0..4410 {
                if inst.process().abs() > 0.001 {
                    had_sound = true;
                }
            }
            assert!(had_sound, "Auxiliary percussion should produce sound");
        }
    }

    #[test]
    fn test_triangle() {
        let sample_rate = 44100.0;
        let mut triangle = CymbalSynth::triangle(sample_rate);

        triangle.strike(0.7);

        let mut samples = Vec::new();
        for _ in 0..4410 {
            samples.push(triangle.process());
        }

        let max_amp = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_amp > 0.01, "Triangle should produce sound");
    }

    #[test]
    fn test_mallet_types() {
        let sample_rate = 44100.0;
        let mut timpani = TimpaniSynth::timpani_26(sample_rate);

        let mallets = [
            MalletType::TimpaniSoft,
            MalletType::TimpaniMedium,
            MalletType::TimpaniHard,
            MalletType::TimpaniWood,
        ];

        for mallet in mallets {
            timpani.set_mallet(mallet);
            timpani.note_on(110.0, 0.8);

            for _ in 0..2000 {
                timpani.process();
            }
        }
    }
}
