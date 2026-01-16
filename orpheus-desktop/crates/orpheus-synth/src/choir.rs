//! Choir/Vocal Synthesis Engine
//!
//! Realistic vocal synthesis using formant filtering.
//! Unlike typical MIDI choir sounds that use simple samples,
//! this engine models the human voice through:
//! - Glottal excitation (voiced/unvoiced)
//! - Formant resonance (vowel characteristics)
//! - Voice type characteristics (soprano, alto, tenor, bass)
//! - Natural vibrato and breath simulation
//! - Smooth vowel transitions

use std::f32::consts::PI;

/// Voice type classifications
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoiceType {
    Soprano,
    MezzoSoprano,
    Alto,
    Tenor,
    Baritone,
    Bass,
}

impl VoiceType {
    /// Get the typical frequency range for this voice type (min, max in Hz)
    pub fn frequency_range(&self) -> (f32, f32) {
        match self {
            VoiceType::Soprano => (261.6, 1046.5),      // C4 - C6
            VoiceType::MezzoSoprano => (220.0, 880.0),  // A3 - A5
            VoiceType::Alto => (174.6, 698.5),          // F3 - F5
            VoiceType::Tenor => (130.8, 523.3),         // C3 - C5
            VoiceType::Baritone => (110.0, 392.0),      // A2 - G4
            VoiceType::Bass => (82.4, 329.6),           // E2 - E4
        }
    }

    /// Get formant scaling factor (higher voices have shifted formants)
    pub fn formant_scale(&self) -> f32 {
        match self {
            VoiceType::Soprano => 1.15,
            VoiceType::MezzoSoprano => 1.08,
            VoiceType::Alto => 1.0,
            VoiceType::Tenor => 0.95,
            VoiceType::Baritone => 0.9,
            VoiceType::Bass => 0.85,
        }
    }

    /// Get default vibrato characteristics (depth in semitones, rate in Hz)
    pub fn default_vibrato(&self) -> (f32, f32) {
        match self {
            VoiceType::Soprano => (0.4, 5.5),
            VoiceType::MezzoSoprano => (0.35, 5.3),
            VoiceType::Alto => (0.3, 5.0),
            VoiceType::Tenor => (0.35, 5.2),
            VoiceType::Baritone => (0.3, 4.8),
            VoiceType::Bass => (0.25, 4.5),
        }
    }
}

/// Vowel sounds with their characteristic formant frequencies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Vowel {
    // Primary vowels (IPA)
    A,  // "ah" as in "father"
    E,  // "eh" as in "bed"
    I,  // "ee" as in "meet"
    O,  // "oh" as in "boat"
    U,  // "oo" as in "boot"
    // Extended vowels for more expression
    Ae, // "a" as in "cat"
    Uh, // schwa, as in "the"
    Aw, // "aw" as in "caught"
    Oe, // German ö
    // Choir specific
    Mm, // humming (closed mouth)
}

impl Vowel {
    /// Get formant frequencies (F1, F2, F3, F4, F5) in Hz
    /// Based on acoustic phonetics research for adult male voice
    pub fn formants(&self) -> [f32; 5] {
        match self {
            Vowel::A => [730.0, 1090.0, 2440.0, 3400.0, 4200.0],
            Vowel::E => [530.0, 1840.0, 2480.0, 3520.0, 4200.0],
            Vowel::I => [270.0, 2290.0, 3010.0, 3800.0, 4500.0],
            Vowel::O => [570.0, 840.0, 2410.0, 3400.0, 4200.0],
            Vowel::U => [300.0, 870.0, 2240.0, 3200.0, 4100.0],
            Vowel::Ae => [660.0, 1720.0, 2410.0, 3400.0, 4200.0],
            Vowel::Uh => [500.0, 1500.0, 2500.0, 3500.0, 4200.0],
            Vowel::Aw => [570.0, 840.0, 2410.0, 3400.0, 4200.0],
            Vowel::Oe => [400.0, 1600.0, 2400.0, 3300.0, 4200.0],
            Vowel::Mm => [250.0, 2500.0, 3000.0, 3500.0, 4200.0],
        }
    }

    /// Get formant bandwidths (narrower = more resonant)
    pub fn bandwidths(&self) -> [f32; 5] {
        match self {
            Vowel::I | Vowel::E => [60.0, 90.0, 150.0, 200.0, 250.0],
            Vowel::A | Vowel::Ae => [80.0, 100.0, 120.0, 180.0, 250.0],
            Vowel::O | Vowel::Aw => [70.0, 80.0, 140.0, 200.0, 250.0],
            Vowel::U => [50.0, 70.0, 130.0, 200.0, 250.0],
            Vowel::Uh => [70.0, 100.0, 140.0, 200.0, 250.0],
            Vowel::Oe => [60.0, 90.0, 140.0, 200.0, 250.0],
            Vowel::Mm => [40.0, 200.0, 200.0, 200.0, 250.0],
        }
    }

    /// Get relative amplitudes for each formant (dB, converted to linear)
    pub fn amplitudes(&self) -> [f32; 5] {
        match self {
            Vowel::A => [1.0, 0.5, 0.25, 0.1, 0.05],
            Vowel::E => [1.0, 0.7, 0.3, 0.12, 0.05],
            Vowel::I => [1.0, 0.5, 0.35, 0.15, 0.06],
            Vowel::O => [1.0, 0.4, 0.2, 0.08, 0.04],
            Vowel::U => [1.0, 0.3, 0.15, 0.06, 0.03],
            Vowel::Ae => [1.0, 0.6, 0.28, 0.1, 0.05],
            Vowel::Uh => [1.0, 0.5, 0.25, 0.1, 0.05],
            Vowel::Aw => [1.0, 0.4, 0.2, 0.08, 0.04],
            Vowel::Oe => [1.0, 0.55, 0.25, 0.1, 0.05],
            Vowel::Mm => [0.3, 0.1, 0.15, 0.1, 0.05],
        }
    }
}

/// Consonant classification by manner of articulation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConsonantType {
    /// Complete closure then release (p, b, t, d, k, g)
    Plosive,
    /// Turbulent airflow through constriction (f, v, s, z, sh, th)
    Fricative,
    /// Air through nasal cavity (m, n, ng)
    Nasal,
    /// Vowel-like but with constriction (l, r)
    Liquid,
    /// Gliding transition (w, y)
    Glide,
    /// Plosive + fricative combination (ch, j)
    Affricate,
}

/// Individual consonant sounds
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Consonant {
    // Plosives (stops)
    P,  // voiceless bilabial
    B,  // voiced bilabial
    T,  // voiceless alveolar
    D,  // voiced alveolar
    K,  // voiceless velar
    G,  // voiced velar

    // Fricatives
    F,  // voiceless labiodental
    V,  // voiced labiodental
    Th, // voiceless dental (θ as in "think")
    Dh, // voiced dental (ð as in "this")
    S,  // voiceless alveolar
    Z,  // voiced alveolar
    Sh, // voiceless postalveolar (ʃ)
    Zh, // voiced postalveolar (ʒ as in "measure")
    H,  // voiceless glottal

    // Nasals
    M,  // bilabial
    N,  // alveolar
    Ng, // velar (ŋ as in "sing")

    // Liquids
    L,  // lateral
    R,  // rhotic

    // Glides (semivowels)
    W,  // labio-velar
    Y,  // palatal

    // Affricates
    Ch, // voiceless (tʃ as in "church")
    J,  // voiced (dʒ as in "judge")
}

impl Consonant {
    /// Get the consonant type
    pub fn consonant_type(&self) -> ConsonantType {
        match self {
            Consonant::P | Consonant::B | Consonant::T |
            Consonant::D | Consonant::K | Consonant::G => ConsonantType::Plosive,

            Consonant::F | Consonant::V | Consonant::Th | Consonant::Dh |
            Consonant::S | Consonant::Z | Consonant::Sh | Consonant::Zh |
            Consonant::H => ConsonantType::Fricative,

            Consonant::M | Consonant::N | Consonant::Ng => ConsonantType::Nasal,

            Consonant::L | Consonant::R => ConsonantType::Liquid,

            Consonant::W | Consonant::Y => ConsonantType::Glide,

            Consonant::Ch | Consonant::J => ConsonantType::Affricate,
        }
    }

    /// Is this consonant voiced?
    pub fn is_voiced(&self) -> bool {
        match self {
            Consonant::B | Consonant::D | Consonant::G |
            Consonant::V | Consonant::Dh | Consonant::Z | Consonant::Zh |
            Consonant::M | Consonant::N | Consonant::Ng |
            Consonant::L | Consonant::R | Consonant::W | Consonant::Y |
            Consonant::J => true,

            Consonant::P | Consonant::T | Consonant::K |
            Consonant::F | Consonant::Th | Consonant::S | Consonant::Sh |
            Consonant::H | Consonant::Ch => false,
        }
    }

    /// Get the duration in milliseconds for this consonant
    pub fn duration_ms(&self) -> f32 {
        match self.consonant_type() {
            ConsonantType::Plosive => 80.0,   // Brief burst
            ConsonantType::Fricative => 120.0, // Sustained friction
            ConsonantType::Nasal => 100.0,    // Similar to vowels
            ConsonantType::Liquid => 80.0,    // Quick transition
            ConsonantType::Glide => 60.0,     // Very brief
            ConsonantType::Affricate => 140.0, // Plosive + fricative
        }
    }

    /// Get the noise filter center frequency (Hz) for fricatives
    pub fn noise_frequency(&self) -> f32 {
        match self {
            // High frequency sibilants
            Consonant::S | Consonant::Z => 6500.0,
            Consonant::Sh | Consonant::Zh => 3500.0,

            // Lower frequency fricatives
            Consonant::F | Consonant::V => 8000.0,
            Consonant::Th | Consonant::Dh => 5500.0,
            Consonant::H => 2000.0,

            // Affricates (use fricative portion)
            Consonant::Ch => 3500.0,
            Consonant::J => 3500.0,

            // Non-fricatives - not used but provide reasonable default
            _ => 4000.0,
        }
    }

    /// Get the noise filter bandwidth for fricatives
    pub fn noise_bandwidth(&self) -> f32 {
        match self {
            Consonant::S | Consonant::Z => 4000.0,   // Wide, bright
            Consonant::Sh | Consonant::Zh => 2500.0, // Narrower
            Consonant::F | Consonant::V => 6000.0,   // Very wide
            Consonant::Th | Consonant::Dh => 4000.0,
            Consonant::H => 4000.0,                  // Breath-like
            Consonant::Ch | Consonant::J => 2500.0,
            _ => 3000.0,
        }
    }

    /// Get the burst frequency for plosives (Hz)
    pub fn burst_frequency(&self) -> f32 {
        match self {
            // Bilabial - lower burst
            Consonant::P | Consonant::B => 800.0,
            // Alveolar - mid burst
            Consonant::T | Consonant::D => 3500.0,
            // Velar - higher burst, varies with vowel context
            Consonant::K | Consonant::G => 2000.0,
            // Affricates
            Consonant::Ch | Consonant::J => 3500.0,
            _ => 2000.0,
        }
    }

    /// Get formant transitions for this consonant going into a vowel
    /// Returns (F1_offset, F2_offset, F3_offset) in Hz
    pub fn formant_transitions(&self) -> (f32, f32, f32) {
        match self {
            // Bilabials - low F2 locus
            Consonant::P | Consonant::B | Consonant::M | Consonant::W =>
                (-100.0, -500.0, -200.0),

            // Alveolars - F2 around 1700-1800 Hz
            Consonant::T | Consonant::D | Consonant::N | Consonant::L |
            Consonant::S | Consonant::Z =>
                (-50.0, 0.0, -300.0),

            // Velars - F2 varies, F3 lowered
            Consonant::K | Consonant::G | Consonant::Ng =>
                (-50.0, 200.0, -500.0),

            // Palatals - high F2
            Consonant::Sh | Consonant::Zh | Consonant::Y | Consonant::Ch | Consonant::J =>
                (-30.0, 400.0, -100.0),

            // Labiodentals
            Consonant::F | Consonant::V =>
                (-80.0, -300.0, -100.0),

            // Dentals
            Consonant::Th | Consonant::Dh =>
                (-60.0, -100.0, -200.0),

            // R has distinctive lowered F3
            Consonant::R =>
                (-50.0, 100.0, -600.0),

            // Glottal - minimal transition
            Consonant::H =>
                (0.0, 0.0, 0.0),
        }
    }
}

/// Consonant sound generator
pub struct ConsonantGenerator {
    sample_rate: f32,
    noise_state: u32,
    // Noise filter (bandpass for fricatives)
    noise_filter: FormantFilter,
    // Burst envelope
    burst_phase: f32,
    burst_duration: f32,
    // Current consonant state
    active: bool,
    consonant: Option<Consonant>,
    samples_elapsed: usize,
    total_samples: usize,
    // For voiced consonants
    voicing_amount: f32,
}

impl ConsonantGenerator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            noise_state: 0x12345678,
            noise_filter: FormantFilter::new(sample_rate),
            burst_phase: 0.0,
            burst_duration: 0.0,
            active: false,
            consonant: None,
            samples_elapsed: 0,
            total_samples: 0,
            voicing_amount: 0.0,
        }
    }

    /// Start generating a consonant
    pub fn start(&mut self, consonant: Consonant) {
        self.consonant = Some(consonant);
        self.active = true;
        self.samples_elapsed = 0;
        self.total_samples = (consonant.duration_ms() * self.sample_rate / 1000.0) as usize;
        self.burst_phase = 0.0;
        self.burst_duration = 0.01 * self.sample_rate; // 10ms burst

        // Set up noise filter for fricatives
        if matches!(consonant.consonant_type(), ConsonantType::Fricative | ConsonantType::Affricate) {
            self.noise_filter.set_params(consonant.noise_frequency(), consonant.noise_bandwidth());
        }

        self.voicing_amount = if consonant.is_voiced() { 0.3 } else { 0.0 };
    }

    /// Check if consonant is still active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the current consonant
    pub fn current(&self) -> Option<Consonant> {
        self.consonant
    }

    /// Get progress through consonant (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.total_samples == 0 {
            1.0
        } else {
            (self.samples_elapsed as f32 / self.total_samples as f32).min(1.0)
        }
    }

    /// Generate white noise sample
    fn noise(&mut self) -> f32 {
        let mut x = self.noise_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.noise_state = x;
        (x as f32) / (u32::MAX as f32) * 2.0 - 1.0
    }

    /// Generate next sample (returns consonant audio + voicing amount for glottal mixing)
    pub fn next_sample(&mut self, glottal_input: f32) -> f32 {
        if !self.active {
            return 0.0;
        }

        let consonant = match self.consonant {
            Some(c) => c,
            None => return 0.0,
        };

        self.samples_elapsed += 1;
        if self.samples_elapsed >= self.total_samples {
            self.active = false;
        }

        let progress = self.progress();

        // Envelope: attack and release shaping
        let envelope = if progress < 0.1 {
            progress / 0.1 // Quick attack
        } else if progress > 0.8 {
            (1.0 - progress) / 0.2 // Release
        } else {
            1.0
        };

        let output = match consonant.consonant_type() {
            ConsonantType::Plosive => {
                // Plosives: silence (closure) then burst
                if progress < 0.4 {
                    // Closure phase - silence or very quiet
                    0.0
                } else if progress < 0.6 {
                    // Burst phase - noise burst at characteristic frequency
                    let burst_env = ((progress - 0.4) / 0.2 * PI).sin();
                    let noise = self.noise();
                    self.noise_filter.set_params(consonant.burst_frequency(), 1500.0);
                    self.noise_filter.process(noise) * burst_env * 0.8
                } else {
                    // Aspiration/release - blend to vowel
                    let release_env = 1.0 - (progress - 0.6) / 0.4;
                    let noise = self.noise();
                    self.noise_filter.process(noise) * release_env * 0.3
                }
            }

            ConsonantType::Fricative => {
                // Continuous filtered noise
                let noise = self.noise();
                let filtered = self.noise_filter.process(noise);

                // Add voicing for voiced fricatives
                let voiced = if consonant.is_voiced() {
                    glottal_input * self.voicing_amount
                } else {
                    0.0
                };

                (filtered * 0.5 + voiced) * envelope
            }

            ConsonantType::Nasal => {
                // Nasals use glottal source with nasal formants
                // Low frequency emphasis, anti-resonance around 1000 Hz
                let nasal_color = glottal_input * 0.7;

                // Add nasal resonance (simplified)
                nasal_color * envelope
            }

            ConsonantType::Liquid => {
                // Liquids are vowel-like but with specific formant patterns
                // L: lateral - side airflow
                // R: rhotic - bunched tongue
                glottal_input * envelope * 0.6
            }

            ConsonantType::Glide => {
                // Very short, vowel-like transitions
                glottal_input * envelope * 0.5
            }

            ConsonantType::Affricate => {
                // Plosive onset + fricative release
                if progress < 0.3 {
                    // Plosive portion
                    if progress < 0.15 {
                        0.0 // Closure
                    } else {
                        let burst_env = ((progress - 0.15) / 0.15 * PI).sin();
                        let noise = self.noise();
                        self.noise_filter.set_params(consonant.burst_frequency(), 1500.0);
                        self.noise_filter.process(noise) * burst_env * 0.6
                    }
                } else {
                    // Fricative portion
                    let fric_env = if progress > 0.8 {
                        (1.0 - progress) / 0.2
                    } else {
                        1.0
                    };
                    let noise = self.noise();
                    self.noise_filter.set_params(consonant.noise_frequency(), consonant.noise_bandwidth());
                    let filtered = self.noise_filter.process(noise);

                    let voiced = if consonant.is_voiced() {
                        glottal_input * self.voicing_amount
                    } else {
                        0.0
                    };

                    (filtered * 0.4 + voiced) * fric_env
                }
            }
        };

        output
    }

    /// Reset the generator
    pub fn reset(&mut self) {
        self.active = false;
        self.consonant = None;
        self.samples_elapsed = 0;
        self.noise_filter.reset();
    }
}

/// A phoneme: either a vowel, consonant, or silence
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phoneme {
    Vowel(Vowel),
    Consonant(Consonant),
    Silence,
}

/// A syllable composed of optional onset consonant(s), vowel nucleus, and optional coda consonant(s)
#[derive(Debug, Clone)]
pub struct Syllable {
    pub onset: Vec<Consonant>,   // Initial consonant(s), e.g., "str" in "string"
    pub nucleus: Vowel,           // Vowel core
    pub coda: Vec<Consonant>,    // Final consonant(s), e.g., "ng" in "sing"
}

impl Syllable {
    pub fn new(nucleus: Vowel) -> Self {
        Self {
            onset: Vec::new(),
            nucleus,
            coda: Vec::new(),
        }
    }

    pub fn with_onset(mut self, consonants: &[Consonant]) -> Self {
        self.onset = consonants.to_vec();
        self
    }

    pub fn with_coda(mut self, consonants: &[Consonant]) -> Self {
        self.coda = consonants.to_vec();
        self
    }

    /// Common syllables for choral music
    pub fn la() -> Self {
        Self::new(Vowel::A).with_onset(&[Consonant::L])
    }

    pub fn ah() -> Self {
        Self::new(Vowel::A)
    }

    pub fn oh() -> Self {
        Self::new(Vowel::O)
    }

    pub fn alleluia() -> Vec<Self> {
        vec![
            Self::new(Vowel::A),                           // "A"
            Self::new(Vowel::E).with_onset(&[Consonant::L]), // "lle"
            Self::new(Vowel::U).with_onset(&[Consonant::L]), // "lu"
            Self::new(Vowel::A).with_onset(&[Consonant::Y]), // "ia"
        ]
    }

    pub fn gloria() -> Vec<Self> {
        vec![
            Self::new(Vowel::O).with_onset(&[Consonant::G]).with_coda(&[Consonant::L]), // "Glor"
            Self::new(Vowel::I),                           // "i"
            Self::new(Vowel::A),                           // "a"
        ]
    }

    pub fn amen() -> Vec<Self> {
        vec![
            Self::new(Vowel::A),                           // "A"
            Self::new(Vowel::E).with_onset(&[Consonant::M]).with_coda(&[Consonant::N]), // "men"
        ]
    }

    pub fn kyrie() -> Vec<Self> {
        vec![
            Self::new(Vowel::I).with_onset(&[Consonant::K]), // "Ky"
            Self::new(Vowel::I).with_onset(&[Consonant::R]), // "ri"
            Self::new(Vowel::E),                           // "e"
        ]
    }

    pub fn eleison() -> Vec<Self> {
        vec![
            Self::new(Vowel::E),                           // "e"
            Self::new(Vowel::E).with_onset(&[Consonant::L]), // "lei"
            Self::new(Vowel::O).with_onset(&[Consonant::S]), // "son"
            Self::new(Vowel::Uh).with_coda(&[Consonant::N]), // (final n)
        ]
    }
}

/// Glottal source - generates the basic vocal cord vibration
pub struct GlottalSource {
    sample_rate: f32,
    phase: f64,
    frequency: f32,
    // LF model parameters
    open_quotient: f32,    // Ratio of open phase (0.4-0.7 typical)
    speed_quotient: f32,   // Asymmetry of opening/closing
    // Jitter and shimmer for naturalness
    jitter: f32,           // Frequency perturbation
    shimmer: f32,          // Amplitude perturbation
    // Breathiness
    breathiness: f32,
    noise_phase: u32,
    // Vibrato
    vibrato_depth: f32,
    vibrato_rate: f32,
    vibrato_phase: f64,
}

impl GlottalSource {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            frequency: 220.0,
            open_quotient: 0.6,
            speed_quotient: 2.0,
            jitter: 0.003,
            shimmer: 0.02,
            breathiness: 0.05,
            noise_phase: 0,
            vibrato_depth: 0.0,
            vibrato_rate: 5.0,
            vibrato_phase: 0.0,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(50.0, 2000.0);
    }

    pub fn set_vibrato(&mut self, depth_semitones: f32, rate_hz: f32) {
        self.vibrato_depth = depth_semitones.clamp(0.0, 1.0);
        self.vibrato_rate = rate_hz.clamp(0.1, 10.0);
    }

    pub fn set_breathiness(&mut self, amount: f32) {
        self.breathiness = amount.clamp(0.0, 0.5);
    }

    pub fn set_tension(&mut self, tension: f32) {
        // Higher tension = shorter open phase, brighter sound
        let t = tension.clamp(0.0, 1.0);
        self.open_quotient = 0.7 - t * 0.3;
        self.speed_quotient = 1.5 + t * 2.0;
    }

    /// Generate next glottal pulse sample using LF model approximation
    pub fn next_sample(&mut self) -> f32 {
        // Apply vibrato
        let vibrato = if self.vibrato_depth > 0.0 {
            self.vibrato_phase += self.vibrato_rate as f64 / self.sample_rate as f64;
            if self.vibrato_phase >= 1.0 {
                self.vibrato_phase -= 1.0;
            }
            let vib_mod = (self.vibrato_phase * std::f64::consts::TAU).sin() as f32;
            2.0_f32.powf(self.vibrato_depth * vib_mod / 12.0)
        } else {
            1.0
        };

        // Add jitter (frequency perturbation)
        let jitter_mod = if self.jitter > 0.0 {
            1.0 + (Self::noise_sample(&mut self.noise_phase) * 2.0 - 1.0) * self.jitter
        } else {
            1.0
        };

        let freq = self.frequency * vibrato * jitter_mod;
        let period = self.sample_rate / freq;

        // Advance phase
        self.phase += 1.0 / period as f64;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        let t = self.phase as f32;

        // LF (Liljencrants-Fant) model approximation
        // Opening phase: exponential growth
        // Closing phase: exponential decay (return phase)
        let glottal = if t < self.open_quotient {
            // Open phase - asymmetric sinusoid
            let normalized_t = t / self.open_quotient;
            let opening_phase = normalized_t / self.speed_quotient;
            let closing_phase = normalized_t * (1.0 - 1.0 / self.speed_quotient);

            if normalized_t < 1.0 / self.speed_quotient {
                // Opening
                (opening_phase * PI).sin()
            } else {
                // Closing
                (closing_phase * PI + PI / self.speed_quotient).sin()
            }
        } else {
            // Closed phase - return to equilibrium
            let normalized_t = (t - self.open_quotient) / (1.0 - self.open_quotient);
            -0.2 * (-normalized_t * 5.0).exp()
        };

        // Add shimmer (amplitude perturbation)
        let shimmer_mod = if self.shimmer > 0.0 {
            1.0 + (Self::noise_sample(&mut self.noise_phase) * 2.0 - 1.0) * self.shimmer
        } else {
            1.0
        };

        // Mix in breathiness (aspiration noise)
        let noise = Self::noise_sample(&mut self.noise_phase) * 2.0 - 1.0;
        let breath = noise * self.breathiness;

        glottal * shimmer_mod + breath
    }

    /// Simple noise generator
    fn noise_sample(state: &mut u32) -> f32 {
        // xorshift32
        let mut x = *state;
        if x == 0 { x = 0x12345678; }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *state = x;
        (x as f32) / (u32::MAX as f32)
    }
}

/// Biquad filter for formant resonance
#[derive(Clone)]
pub struct FormantFilter {
    // Coefficients
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    // State
    x1: f32, x2: f32,
    y1: f32, y2: f32,
    // Parameters for morphing
    frequency: f32,
    bandwidth: f32,
    sample_rate: f32,
}

impl FormantFilter {
    pub fn new(sample_rate: f32) -> Self {
        let mut filter = Self {
            b0: 1.0, b1: 0.0, b2: 0.0,
            a1: 0.0, a2: 0.0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
            frequency: 1000.0,
            bandwidth: 100.0,
            sample_rate,
        };
        filter.update_coefficients();
        filter
    }

    pub fn set_params(&mut self, frequency: f32, bandwidth: f32) {
        self.frequency = frequency.clamp(50.0, self.sample_rate * 0.45);
        self.bandwidth = bandwidth.clamp(20.0, 500.0);
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        // Bandpass filter design for formant
        let omega = 2.0 * PI * self.frequency / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();

        // Q from bandwidth
        let q = self.frequency / self.bandwidth;
        let alpha = sin_omega / (2.0 * q);

        let a0 = 1.0 + alpha;

        self.b0 = (alpha * q) / a0;  // Scale for unity gain at peak
        self.b1 = 0.0;
        self.b2 = -(alpha * q) / a0;
        self.a1 = (-2.0 * cos_omega) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
                   - self.a1 * self.y1 - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

/// Formant filter bank - 5 parallel bandpass filters
pub struct FormantBank {
    filters: [FormantFilter; 5],
    amplitudes: [f32; 5],
    // For smooth transitions
    target_freqs: [f32; 5],
    target_bandwidths: [f32; 5],
    target_amplitudes: [f32; 5],
    current_freqs: [f32; 5],
    current_bandwidths: [f32; 5],
    current_amplitudes: [f32; 5],
    morph_rate: f32,
}

impl FormantBank {
    pub fn new(sample_rate: f32) -> Self {
        let filters = [
            FormantFilter::new(sample_rate),
            FormantFilter::new(sample_rate),
            FormantFilter::new(sample_rate),
            FormantFilter::new(sample_rate),
            FormantFilter::new(sample_rate),
        ];

        let initial_freqs = [500.0, 1500.0, 2500.0, 3500.0, 4200.0];
        let initial_bw = [80.0, 100.0, 120.0, 150.0, 200.0];
        let initial_amp = [1.0, 0.5, 0.25, 0.1, 0.05];

        Self {
            filters,
            amplitudes: initial_amp,
            target_freqs: initial_freqs,
            target_bandwidths: initial_bw,
            target_amplitudes: initial_amp,
            current_freqs: initial_freqs,
            current_bandwidths: initial_bw,
            current_amplitudes: initial_amp,
            morph_rate: 0.001,
        }
    }

    /// Set target vowel (formants will morph toward this)
    pub fn set_vowel(&mut self, vowel: Vowel, voice_scale: f32) {
        let formants = vowel.formants();
        let bandwidths = vowel.bandwidths();
        let amplitudes = vowel.amplitudes();

        for i in 0..5 {
            self.target_freqs[i] = formants[i] * voice_scale;
            self.target_bandwidths[i] = bandwidths[i];
            self.target_amplitudes[i] = amplitudes[i];
        }
    }

    /// Set the morph rate (how fast vowels transition)
    pub fn set_morph_rate(&mut self, rate: f32) {
        self.morph_rate = rate.clamp(0.0001, 0.1);
    }

    /// Process a sample through the formant bank
    pub fn process(&mut self, input: f32) -> f32 {
        // Interpolate toward target formants
        for i in 0..5 {
            self.current_freqs[i] += (self.target_freqs[i] - self.current_freqs[i]) * self.morph_rate;
            self.current_bandwidths[i] += (self.target_bandwidths[i] - self.current_bandwidths[i]) * self.morph_rate;
            self.current_amplitudes[i] += (self.target_amplitudes[i] - self.current_amplitudes[i]) * self.morph_rate;

            self.filters[i].set_params(self.current_freqs[i], self.current_bandwidths[i]);
            self.amplitudes[i] = self.current_amplitudes[i];
        }

        // Sum parallel filters
        let mut output = 0.0;
        for i in 0..5 {
            output += self.filters[i].process(input) * self.amplitudes[i];
        }

        // Normalize
        output * 0.3
    }

    pub fn reset(&mut self) {
        for filter in &mut self.filters {
            filter.reset();
        }
    }
}

/// Attack/release envelope for natural note shaping
pub struct VocalEnvelope {
    sample_rate: f32,
    state: EnvelopeState,
    level: f32,
    attack_rate: f32,
    decay_rate: f32,
    sustain_level: f32,
    release_rate: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl VocalEnvelope {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            state: EnvelopeState::Idle,
            level: 0.0,
            attack_rate: 1.0 / (0.05 * sample_rate),   // 50ms attack
            decay_rate: 1.0 / (0.1 * sample_rate),     // 100ms decay
            sustain_level: 0.8,
            release_rate: 1.0 / (0.15 * sample_rate),  // 150ms release
        }
    }

    pub fn set_attack(&mut self, seconds: f32) {
        self.attack_rate = 1.0 / (seconds.max(0.001) * self.sample_rate);
    }

    pub fn set_release(&mut self, seconds: f32) {
        self.release_rate = 1.0 / (seconds.max(0.001) * self.sample_rate);
    }

    pub fn trigger(&mut self) {
        self.state = EnvelopeState::Attack;
    }

    pub fn release(&mut self) {
        if self.state != EnvelopeState::Idle {
            self.state = EnvelopeState::Release;
        }
    }

    pub fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle || self.level > 0.001
    }

    pub fn next_sample(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Idle => {}
            EnvelopeState::Attack => {
                self.level += self.attack_rate;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.state = EnvelopeState::Decay;
                }
            }
            EnvelopeState::Decay => {
                self.level -= self.decay_rate;
                if self.level <= self.sustain_level {
                    self.level = self.sustain_level;
                    self.state = EnvelopeState::Sustain;
                }
            }
            EnvelopeState::Sustain => {
                // Hold at sustain level
            }
            EnvelopeState::Release => {
                self.level -= self.release_rate;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.state = EnvelopeState::Idle;
                }
            }
        }
        self.level
    }
}

/// Complete single voice synthesizer
pub struct VocalSynth {
    glottal: GlottalSource,
    formants: FormantBank,
    envelope: VocalEnvelope,
    voice_type: VoiceType,
    current_vowel: Vowel,
    amplitude: f32,
    // Legato handling
    legato: bool,
    target_frequency: f32,
    current_frequency: f32,
    portamento_rate: f32,
}

impl VocalSynth {
    pub fn new(sample_rate: f32, voice_type: VoiceType) -> Self {
        let mut glottal = GlottalSource::new(sample_rate);
        let (vib_depth, vib_rate) = voice_type.default_vibrato();
        glottal.set_vibrato(vib_depth, vib_rate);

        let mut formants = FormantBank::new(sample_rate);
        formants.set_vowel(Vowel::A, voice_type.formant_scale());

        Self {
            glottal,
            formants,
            envelope: VocalEnvelope::new(sample_rate),
            voice_type,
            current_vowel: Vowel::A,
            amplitude: 0.8,
            legato: false,
            target_frequency: 220.0,
            current_frequency: 220.0,
            portamento_rate: 0.005,
        }
    }

    /// Factory methods for voice types
    pub fn soprano(sample_rate: f32) -> Self {
        Self::new(sample_rate, VoiceType::Soprano)
    }

    pub fn alto(sample_rate: f32) -> Self {
        Self::new(sample_rate, VoiceType::Alto)
    }

    pub fn tenor(sample_rate: f32) -> Self {
        Self::new(sample_rate, VoiceType::Tenor)
    }

    pub fn bass(sample_rate: f32) -> Self {
        Self::new(sample_rate, VoiceType::Bass)
    }

    /// Start singing a note
    pub fn sing(&mut self, frequency: f32, velocity: f32) {
        self.target_frequency = frequency;
        self.amplitude = velocity.clamp(0.0, 1.0);

        if !self.legato || !self.envelope.is_active() {
            self.current_frequency = frequency;
            self.glottal.set_frequency(frequency);
            self.envelope.trigger();
        }
    }

    /// Stop singing
    pub fn release(&mut self) {
        self.envelope.release();
    }

    /// Set the vowel sound
    pub fn set_vowel(&mut self, vowel: Vowel) {
        self.current_vowel = vowel;
        self.formants.set_vowel(vowel, self.voice_type.formant_scale());
    }

    /// Morph between current vowel and target vowel
    pub fn set_vowel_morph_rate(&mut self, rate: f32) {
        self.formants.set_morph_rate(rate);
    }

    /// Set vibrato parameters
    pub fn set_vibrato(&mut self, depth_semitones: f32, rate_hz: f32) {
        self.glottal.set_vibrato(depth_semitones, rate_hz);
    }

    /// Set breathiness (0.0 = clean, 0.5 = very breathy)
    pub fn set_breathiness(&mut self, amount: f32) {
        self.glottal.set_breathiness(amount);
    }

    /// Set tension (0.0 = relaxed/warm, 1.0 = tense/bright)
    pub fn set_tension(&mut self, tension: f32) {
        self.glottal.set_tension(tension);
    }

    /// Enable legato mode (notes connect smoothly)
    pub fn set_legato(&mut self, enabled: bool) {
        self.legato = enabled;
    }

    /// Set portamento time (for legato slides)
    pub fn set_portamento(&mut self, rate: f32) {
        self.portamento_rate = rate.clamp(0.001, 0.1);
    }

    /// Check if voice is active
    pub fn is_active(&self) -> bool {
        self.envelope.is_active()
    }

    /// Generate next sample
    pub fn next_sample(&mut self) -> f32 {
        if !self.envelope.is_active() {
            return 0.0;
        }

        // Portamento (smooth pitch transition for legato)
        if (self.current_frequency - self.target_frequency).abs() > 0.1 {
            self.current_frequency += (self.target_frequency - self.current_frequency) * self.portamento_rate;
            self.glottal.set_frequency(self.current_frequency);
        }

        // Generate glottal source
        let glottal = self.glottal.next_sample();

        // Filter through formants
        let filtered = self.formants.process(glottal);

        // Apply envelope and amplitude
        let env = self.envelope.next_sample();

        filtered * env * self.amplitude
    }

    /// Fill buffer with samples
    pub fn fill_buffer(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.next_sample();
        }
    }

    /// Add to buffer (mixing)
    pub fn add_to_buffer(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample += self.next_sample();
        }
    }
}

/// Choir section - multiple voices with natural variation
pub struct ChoirSection {
    voices: Vec<VocalSynth>,
    detune: Vec<f32>,      // Cents of detuning per voice
    pan: Vec<f32>,          // Stereo position (-1 to 1)
    timing_offset: Vec<f32>, // Attack timing variation (samples)
    section_type: VoiceType,
}

impl ChoirSection {
    /// Create a choir section with specified number of voices
    pub fn new(count: usize, sample_rate: f32, voice_type: VoiceType) -> Self {
        let mut voices = Vec::with_capacity(count);
        let mut detune = Vec::with_capacity(count);
        let mut pan = Vec::with_capacity(count);
        let mut timing_offset = Vec::with_capacity(count);

        for i in 0..count {
            let mut voice = VocalSynth::new(sample_rate, voice_type);

            // Add natural variation
            let variation = (i as f32 / count as f32) * 2.0 - 1.0;

            // Slight vibrato variation
            let (base_depth, base_rate) = voice_type.default_vibrato();
            voice.set_vibrato(
                base_depth * (0.8 + variation.abs() * 0.4),
                base_rate * (0.9 + variation.abs() * 0.2),
            );

            // Slight breathiness variation
            voice.set_breathiness(0.03 + variation.abs() * 0.04);

            voices.push(voice);

            // Spread detuning (-15 to +15 cents typical for choir)
            detune.push(variation * 15.0);

            // Spread pan across stereo field
            pan.push(variation * 0.7);

            // Random-ish timing offset (up to 30ms)
            timing_offset.push(variation.abs() * sample_rate * 0.03);
        }

        Self {
            voices,
            detune,
            pan,
            timing_offset,
            section_type: voice_type,
        }
    }

    /// Factory methods for common sections
    pub fn sopranos(count: usize, sample_rate: f32) -> Self {
        Self::new(count, sample_rate, VoiceType::Soprano)
    }

    pub fn altos(count: usize, sample_rate: f32) -> Self {
        Self::new(count, sample_rate, VoiceType::Alto)
    }

    pub fn tenors(count: usize, sample_rate: f32) -> Self {
        Self::new(count, sample_rate, VoiceType::Tenor)
    }

    pub fn basses(count: usize, sample_rate: f32) -> Self {
        Self::new(count, sample_rate, VoiceType::Bass)
    }

    /// Start all voices singing a note
    pub fn sing(&mut self, frequency: f32, velocity: f32) {
        for (i, voice) in self.voices.iter_mut().enumerate() {
            // Apply detuning (convert cents to frequency ratio)
            let detune_ratio = 2.0_f32.powf(self.detune[i] / 1200.0);
            let detuned_freq = frequency * detune_ratio;

            voice.sing(detuned_freq, velocity);
        }
    }

    /// Release all voices
    pub fn release(&mut self) {
        for voice in &mut self.voices {
            voice.release();
        }
    }

    /// Set vowel for all voices
    pub fn set_vowel(&mut self, vowel: Vowel) {
        for voice in &mut self.voices {
            voice.set_vowel(vowel);
        }
    }

    /// Set vowel morph rate for all voices
    pub fn set_vowel_morph_rate(&mut self, rate: f32) {
        for voice in &mut self.voices {
            voice.set_vowel_morph_rate(rate);
        }
    }

    /// Enable/disable legato for all voices
    pub fn set_legato(&mut self, enabled: bool) {
        for voice in &mut self.voices {
            voice.set_legato(enabled);
        }
    }

    /// Check if any voice is active
    pub fn is_active(&self) -> bool {
        self.voices.iter().any(|v| v.is_active())
    }

    /// Generate stereo output
    pub fn next_sample_stereo(&mut self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;
        let num_voices = self.voices.len() as f32;

        for (i, voice) in self.voices.iter_mut().enumerate() {
            let sample = voice.next_sample();
            let pan = self.pan[i];

            // Equal power panning
            let angle = (pan + 1.0) * 0.25 * PI;
            left += sample * angle.cos() / num_voices.sqrt();
            right += sample * angle.sin() / num_voices.sqrt();
        }

        (left, right)
    }

    /// Generate mono output (summed)
    pub fn next_sample(&mut self) -> f32 {
        let (l, r) = self.next_sample_stereo();
        (l + r) * 0.5
    }

    /// Fill stereo buffers
    pub fn fill_buffer_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        let len = left.len().min(right.len());
        for i in 0..len {
            let (l, r) = self.next_sample_stereo();
            left[i] = l;
            right[i] = r;
        }
    }
}

/// Full SATB (Soprano, Alto, Tenor, Bass) choir
pub struct Choir {
    pub sopranos: ChoirSection,
    pub altos: ChoirSection,
    pub tenors: ChoirSection,
    pub basses: ChoirSection,
}

impl Choir {
    /// Create a standard SATB choir
    /// sizes: (sopranos, altos, tenors, basses)
    pub fn new(sizes: (usize, usize, usize, usize), sample_rate: f32) -> Self {
        Self {
            sopranos: ChoirSection::sopranos(sizes.0, sample_rate),
            altos: ChoirSection::altos(sizes.1, sample_rate),
            tenors: ChoirSection::tenors(sizes.2, sample_rate),
            basses: ChoirSection::basses(sizes.3, sample_rate),
        }
    }

    /// Standard chamber choir (4-4-4-4)
    pub fn chamber(sample_rate: f32) -> Self {
        Self::new((4, 4, 4, 4), sample_rate)
    }

    /// Large concert choir (12-10-10-8)
    pub fn concert(sample_rate: f32) -> Self {
        Self::new((12, 10, 10, 8), sample_rate)
    }

    /// Set vowel for entire choir
    pub fn set_vowel(&mut self, vowel: Vowel) {
        self.sopranos.set_vowel(vowel);
        self.altos.set_vowel(vowel);
        self.tenors.set_vowel(vowel);
        self.basses.set_vowel(vowel);
    }

    /// Set vowel morph rate for entire choir
    pub fn set_vowel_morph_rate(&mut self, rate: f32) {
        self.sopranos.set_vowel_morph_rate(rate);
        self.altos.set_vowel_morph_rate(rate);
        self.tenors.set_vowel_morph_rate(rate);
        self.basses.set_vowel_morph_rate(rate);
    }

    /// Release all sections
    pub fn release_all(&mut self) {
        self.sopranos.release();
        self.altos.release();
        self.tenors.release();
        self.basses.release();
    }

    /// Check if any section is active
    pub fn is_active(&self) -> bool {
        self.sopranos.is_active() || self.altos.is_active()
            || self.tenors.is_active() || self.basses.is_active()
    }

    /// Generate stereo output (all sections combined)
    pub fn next_sample_stereo(&mut self) -> (f32, f32) {
        let (sl, sr) = self.sopranos.next_sample_stereo();
        let (al, ar) = self.altos.next_sample_stereo();
        let (tl, tr) = self.tenors.next_sample_stereo();
        let (bl, br) = self.basses.next_sample_stereo();

        // Mix all sections
        let left = (sl + al + tl + bl) * 0.4;
        let right = (sr + ar + tr + br) * 0.4;

        (left, right)
    }

    /// Fill stereo buffers
    pub fn fill_buffer_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        let len = left.len().min(right.len());
        for i in 0..len {
            let (l, r) = self.next_sample_stereo();
            left[i] = l;
            right[i] = r;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glottal_source() {
        let mut glottal = GlottalSource::new(44100.0);
        glottal.set_frequency(220.0);

        let mut samples = vec![0.0; 4410];
        for sample in &mut samples {
            *sample = glottal.next_sample();
        }

        // Should have non-zero output
        let max = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }

    #[test]
    fn test_formant_filter() {
        let mut filter = FormantFilter::new(44100.0);
        filter.set_params(500.0, 80.0);

        // Process some samples
        let mut output = 0.0;
        for i in 0..1000 {
            let input = if i < 10 { 1.0 } else { 0.0 }; // Impulse
            output = filter.process(input);
        }

        // Filter should produce output
        assert!(output.abs() < 1.0); // Should be decaying
    }

    #[test]
    fn test_vocal_synth() {
        let mut voice = VocalSynth::soprano(44100.0);
        voice.set_vowel(Vowel::A);
        voice.sing(440.0, 0.8);

        let mut samples = vec![0.0; 44100];
        voice.fill_buffer(&mut samples);

        // Should have output
        let max = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }

    #[test]
    fn test_choir_section() {
        let mut section = ChoirSection::sopranos(4, 44100.0);
        section.set_vowel(Vowel::A);
        section.sing(440.0, 0.8);

        let mut left = vec![0.0; 4410];
        let mut right = vec![0.0; 4410];
        section.fill_buffer_stereo(&mut left, &mut right);

        // Should have stereo output
        let max_l = left.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        let max_r = right.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max_l > 0.0);
        assert!(max_r > 0.0);
    }

    #[test]
    fn test_vowel_formants() {
        // Verify formant data is sensible
        for vowel in [Vowel::A, Vowel::E, Vowel::I, Vowel::O, Vowel::U] {
            let formants = vowel.formants();
            // F1 should be lowest
            assert!(formants[0] < formants[1]);
            // All should be positive
            assert!(formants.iter().all(|&f| f > 0.0));
        }
    }

    #[test]
    fn test_full_choir() {
        let mut choir = Choir::chamber(44100.0);
        choir.set_vowel(Vowel::A);

        choir.sopranos.sing(523.25, 0.7); // C5
        choir.altos.sing(392.0, 0.7);     // G4
        choir.tenors.sing(329.63, 0.7);   // E4
        choir.basses.sing(261.63, 0.7);   // C4

        let mut left = vec![0.0; 4410];
        let mut right = vec![0.0; 4410];
        choir.fill_buffer_stereo(&mut left, &mut right);

        // Should have output
        let max = left.iter().chain(right.iter()).map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }

    #[test]
    fn test_consonant_types() {
        // Test consonant classification
        assert_eq!(Consonant::P.consonant_type(), ConsonantType::Plosive);
        assert_eq!(Consonant::S.consonant_type(), ConsonantType::Fricative);
        assert_eq!(Consonant::M.consonant_type(), ConsonantType::Nasal);
        assert_eq!(Consonant::L.consonant_type(), ConsonantType::Liquid);
        assert_eq!(Consonant::W.consonant_type(), ConsonantType::Glide);
        assert_eq!(Consonant::Ch.consonant_type(), ConsonantType::Affricate);
    }

    #[test]
    fn test_consonant_voicing() {
        // Voiceless
        assert!(!Consonant::P.is_voiced());
        assert!(!Consonant::T.is_voiced());
        assert!(!Consonant::S.is_voiced());

        // Voiced
        assert!(Consonant::B.is_voiced());
        assert!(Consonant::D.is_voiced());
        assert!(Consonant::Z.is_voiced());
        assert!(Consonant::M.is_voiced());
    }

    #[test]
    fn test_consonant_generator_fricative() {
        let mut gen = ConsonantGenerator::new(44100.0);
        let mut glottal = GlottalSource::new(44100.0);
        glottal.set_frequency(220.0);

        gen.start(Consonant::S);
        assert!(gen.is_active());

        let mut samples = Vec::new();
        while gen.is_active() {
            let g = glottal.next_sample();
            samples.push(gen.next_sample(g));
        }

        // Should have produced output
        let max = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
        assert!(!gen.is_active());
    }

    #[test]
    fn test_consonant_generator_plosive() {
        let mut gen = ConsonantGenerator::new(44100.0);
        let mut glottal = GlottalSource::new(44100.0);
        glottal.set_frequency(220.0);

        gen.start(Consonant::T);
        assert!(gen.is_active());

        let mut samples = Vec::new();
        while gen.is_active() {
            let g = glottal.next_sample();
            samples.push(gen.next_sample(g));
        }

        // First part should be silent (closure phase)
        let closure_samples = (0.4 * samples.len() as f32) as usize;
        let closure_max = samples[..closure_samples].iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(closure_max < 0.01, "Closure phase should be silent");

        // Should have burst after closure
        let burst_max = samples[closure_samples..].iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(burst_max > 0.0, "Should have burst after closure");
    }

    #[test]
    fn test_syllable_construction() {
        let syllable = Syllable::new(Vowel::A)
            .with_onset(&[Consonant::S, Consonant::T])
            .with_coda(&[Consonant::R]);

        assert_eq!(syllable.onset.len(), 2);
        assert_eq!(syllable.nucleus, Vowel::A);
        assert_eq!(syllable.coda.len(), 1);
    }

    #[test]
    fn test_common_syllables() {
        let alleluia = Syllable::alleluia();
        assert_eq!(alleluia.len(), 4);

        let amen = Syllable::amen();
        assert_eq!(amen.len(), 2);

        let kyrie = Syllable::kyrie();
        assert_eq!(kyrie.len(), 3);
    }

    #[test]
    fn test_formant_transitions() {
        // Bilabials should have low F2 locus
        let (_, f2, _) = Consonant::P.formant_transitions();
        assert!(f2 < 0.0, "Bilabials should lower F2");

        // R should have distinctively lowered F3
        let (_, _, f3) = Consonant::R.formant_transitions();
        assert!(f3 < -400.0, "R should significantly lower F3");
    }
}
