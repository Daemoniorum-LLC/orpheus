//! Instrument routing and multi-synthesizer playback engine
//!
//! Provides:
//! - Synthesizer trait for unified instrument interface
//! - MIDI program to instrument type mapping
//! - Multi-track playback with per-track synth routing
//! - Orchestra-aware rendering with spatial positioning

use std::collections::HashMap;

use crate::{
    bass::BassSynth,
    brass::{BrassArticulation, BrassInstrument, BrassMute, BrassSection, BrassSynth},
    choir::{Choir, ChoirSection, VoiceType, Vowel},
    drums::{DrumMachine, DrumType},
    guitar::{GuitarConfig, GuitarSynth},
    orchestra::{ConcertHall, Listener, Orchestra, OrchestraSection, Position, SeatingArrangement},
    orchestral_percussion::{
        AuxPercussionSynth, CymbalSynth, MalletSynth, MalletType, PercussionInstrument,
        TimpaniSynth,
    },
    piano::PianoSynth,
    strings::{StringArticulation, StringInstrument, StringSection, StringSynth},
    woodwinds::{WoodwindArticulation, WoodwindInstrument, WoodwindSection, WoodwindSynth},
};

/// Unified synthesizer trait for all instruments
pub trait Synthesizer: Send {
    /// Trigger a note on
    fn note_on(&mut self, note: u8, velocity: f32);

    /// Trigger a note off
    fn note_off(&mut self, note: u8);

    /// Process a block of samples, returning stereo output
    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>);

    /// Get the instrument name for display
    fn name(&self) -> &str;

    /// Check if any voices are currently active
    fn is_active(&self) -> bool;

    /// Set master volume (0.0 - 1.0)
    fn set_volume(&mut self, volume: f32);

    /// Apply pitch bend (-1.0 to 1.0, in semitones)
    fn pitch_bend(&mut self, _semitones: f32) {}

    /// Apply modulation (0.0 - 1.0)
    fn modulation(&mut self, _amount: f32) {}

    /// Apply sustain pedal
    fn sustain(&mut self, _on: bool) {}
}

/// Instrument type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstrumentCategory {
    Guitar,
    Bass,
    Piano,
    Strings,
    Brass,
    Woodwind,
    Percussion,
    Choir,
    Synth,
    Other,
}

/// Specific instrument types with configuration
#[derive(Debug, Clone)]
pub enum InstrumentType {
    // Guitar family
    AcousticGuitar(GuitarConfig),
    ElectricGuitar(GuitarConfig),
    ClassicalGuitar(GuitarConfig),
    TwelveString(GuitarConfig),

    // Bass
    ElectricBass,
    AcousticBass,
    SynthBass,

    // Piano/Keys
    GrandPiano,
    ElectricPiano,
    Harpsichord,
    Organ,

    // Strings (orchestral)
    Violin,
    Viola,
    Cello,
    DoubleBass,
    StringSection, // Full section

    // Brass
    Trumpet,
    FrenchHorn,
    Trombone,
    Tuba,
    BrassSection,

    // Woodwinds
    Flute,
    Clarinet,
    Oboe,
    Bassoon,
    Saxophone(SaxType),
    WoodwindSection,

    // Percussion
    Timpani,
    Xylophone,
    Marimba,
    Vibraphone,
    Glockenspiel,
    TubularBells,
    Cymbals,
    Triangle,
    DrumKit,

    // Choir
    SopranoVoice,
    AltoVoice,
    TenorVoice,
    BassVoice,
    FullChoir,

    // Generic fallback
    GenericMidi(u8), // MIDI program number
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaxType {
    Soprano,
    Alto,
    Tenor,
    Baritone,
}

impl InstrumentType {
    /// Map MIDI program number (0-127) to instrument type
    pub fn from_midi_program(program: u8) -> Self {
        match program {
            // Piano
            0..=7 => Self::GrandPiano,

            // Chromatic Percussion
            8..=15 => Self::Vibraphone,

            // Organ
            16..=23 => Self::Organ,

            // Guitar
            24 => Self::ClassicalGuitar(GuitarConfig::default()),
            25 => Self::AcousticGuitar(GuitarConfig::default()),
            26..=31 => Self::ElectricGuitar(GuitarConfig::default()),

            // Bass
            32..=39 => Self::ElectricBass,

            // Strings
            40 => Self::Violin,
            41 => Self::Viola,
            42 => Self::Cello,
            43 => Self::DoubleBass,
            44 => Self::StringSection, // Tremolo strings
            45 => Self::StringSection, // Pizzicato
            46 => Self::StringSection, // Orchestral harp (use strings)
            47 => Self::Timpani,

            // Ensemble
            48..=51 => Self::StringSection,
            52..=55 => Self::FullChoir,

            // Brass
            56 => Self::Trumpet,
            57 => Self::Trombone,
            58 => Self::Tuba,
            59 => Self::FrenchHorn, // Muted trumpet → horn
            60 => Self::FrenchHorn,
            61..=63 => Self::BrassSection,

            // Reed
            64 => Self::Saxophone(SaxType::Soprano),
            65 => Self::Saxophone(SaxType::Alto),
            66 => Self::Saxophone(SaxType::Tenor),
            67 => Self::Saxophone(SaxType::Baritone),
            68 => Self::Oboe,
            69 => Self::Oboe, // English Horn
            70 => Self::Bassoon,
            71 => Self::Clarinet,

            // Pipe
            72 => Self::Flute, // Piccolo
            73 => Self::Flute,
            74 => Self::Flute, // Recorder
            75 => Self::Flute, // Pan flute
            76..=79 => Self::Flute,

            // Synth Lead
            80..=87 => Self::GenericMidi(program),

            // Synth Pad
            88..=95 => Self::StringSection,

            // Synth Effects
            96..=103 => Self::GenericMidi(program),

            // Ethnic
            104..=111 => Self::GenericMidi(program),

            // Percussive
            112..=119 => Self::DrumKit,

            // Sound Effects
            120..=127 => Self::GenericMidi(program),

            // Out of range (MIDI only uses 0-127)
            128..=255 => Self::GenericMidi(program),
        }
    }

    /// Get the orchestra section for this instrument (if applicable)
    pub fn orchestra_section(&self) -> Option<OrchestraSection> {
        match self {
            Self::Violin => Some(OrchestraSection::Violin1),
            Self::Viola => Some(OrchestraSection::Viola),
            Self::Cello => Some(OrchestraSection::Cello),
            Self::DoubleBass => Some(OrchestraSection::DoubleBass),
            Self::StringSection => Some(OrchestraSection::Violin1),

            Self::Trumpet => Some(OrchestraSection::Trumpet),
            Self::FrenchHorn => Some(OrchestraSection::FrenchHorn),
            Self::Trombone => Some(OrchestraSection::Trombone),
            Self::Tuba => Some(OrchestraSection::Tuba),
            Self::BrassSection => Some(OrchestraSection::Trumpet),

            Self::Flute => Some(OrchestraSection::Flute),
            Self::Clarinet => Some(OrchestraSection::Clarinet),
            Self::Oboe => Some(OrchestraSection::Oboe),
            Self::Bassoon => Some(OrchestraSection::Bassoon),
            Self::Saxophone(_) => Some(OrchestraSection::Clarinet),
            Self::WoodwindSection => Some(OrchestraSection::Flute),

            Self::Timpani => Some(OrchestraSection::Timpani),
            Self::Xylophone
            | Self::Marimba
            | Self::Vibraphone
            | Self::Glockenspiel
            | Self::TubularBells
            | Self::Cymbals
            | Self::Triangle => Some(OrchestraSection::Percussion),

            Self::SopranoVoice => Some(OrchestraSection::Soprano),
            Self::AltoVoice => Some(OrchestraSection::Alto),
            Self::TenorVoice => Some(OrchestraSection::Tenor),
            Self::BassVoice => Some(OrchestraSection::Bass),
            Self::FullChoir => Some(OrchestraSection::Soprano),

            _ => None,
        }
    }

    /// Get category
    pub fn category(&self) -> InstrumentCategory {
        match self {
            Self::AcousticGuitar(_)
            | Self::ElectricGuitar(_)
            | Self::ClassicalGuitar(_)
            | Self::TwelveString(_) => InstrumentCategory::Guitar,

            Self::ElectricBass | Self::AcousticBass | Self::SynthBass => InstrumentCategory::Bass,

            Self::GrandPiano | Self::ElectricPiano | Self::Harpsichord | Self::Organ => {
                InstrumentCategory::Piano
            }

            Self::Violin
            | Self::Viola
            | Self::Cello
            | Self::DoubleBass
            | Self::StringSection => InstrumentCategory::Strings,

            Self::Trumpet
            | Self::FrenchHorn
            | Self::Trombone
            | Self::Tuba
            | Self::BrassSection => InstrumentCategory::Brass,

            Self::Flute
            | Self::Clarinet
            | Self::Oboe
            | Self::Bassoon
            | Self::Saxophone(_)
            | Self::WoodwindSection => InstrumentCategory::Woodwind,

            Self::Timpani
            | Self::Xylophone
            | Self::Marimba
            | Self::Vibraphone
            | Self::Glockenspiel
            | Self::TubularBells
            | Self::Cymbals
            | Self::Triangle
            | Self::DrumKit => InstrumentCategory::Percussion,

            Self::SopranoVoice
            | Self::AltoVoice
            | Self::TenorVoice
            | Self::BassVoice
            | Self::FullChoir => InstrumentCategory::Choir,

            Self::GenericMidi(_) => InstrumentCategory::Other,
        }
    }
}

/// Factory for creating synthesizers from instrument types
pub struct SynthFactory {
    sample_rate: f32,
}

impl SynthFactory {
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }

    /// Create a synthesizer for the given instrument type
    pub fn create(&self, instrument: &InstrumentType) -> Box<dyn Synthesizer> {
        match instrument {
            // Guitar
            InstrumentType::AcousticGuitar(config)
            | InstrumentType::ElectricGuitar(config)
            | InstrumentType::ClassicalGuitar(config)
            | InstrumentType::TwelveString(config) => {
                Box::new(GuitarSynthWrapper::new(config.clone(), self.sample_rate as u32))
            }

            // Bass
            InstrumentType::ElectricBass
            | InstrumentType::AcousticBass
            | InstrumentType::SynthBass => {
                Box::new(BassSynthWrapper::new(self.sample_rate as u32))
            }

            // Piano
            InstrumentType::GrandPiano
            | InstrumentType::ElectricPiano
            | InstrumentType::Harpsichord => {
                Box::new(PianoSynthWrapper::new(self.sample_rate as u32))
            }

            InstrumentType::Organ => Box::new(PianoSynthWrapper::new(self.sample_rate as u32)),

            // Strings
            InstrumentType::Violin => Box::new(StringSynthWrapper::new(
                StringInstrument::Violin,
                self.sample_rate,
            )),
            InstrumentType::Viola => Box::new(StringSynthWrapper::new(
                StringInstrument::Viola,
                self.sample_rate,
            )),
            InstrumentType::Cello => Box::new(StringSynthWrapper::new(
                StringInstrument::Cello,
                self.sample_rate,
            )),
            InstrumentType::DoubleBass => Box::new(StringSynthWrapper::new(
                StringInstrument::DoubleBass,
                self.sample_rate,
            )),
            InstrumentType::StringSection => Box::new(StringSectionWrapper::new(self.sample_rate)),

            // Brass
            InstrumentType::Trumpet => Box::new(BrassSynthWrapper::new(
                BrassInstrument::Trumpet,
                self.sample_rate,
            )),
            InstrumentType::FrenchHorn => Box::new(BrassSynthWrapper::new(
                BrassInstrument::FrenchHorn,
                self.sample_rate,
            )),
            InstrumentType::Trombone => Box::new(BrassSynthWrapper::new(
                BrassInstrument::Trombone,
                self.sample_rate,
            )),
            InstrumentType::Tuba => Box::new(BrassSynthWrapper::new(
                BrassInstrument::Tuba,
                self.sample_rate,
            )),
            InstrumentType::BrassSection => {
                Box::new(BrassSectionWrapper::new(self.sample_rate))
            }

            // Woodwinds
            InstrumentType::Flute => Box::new(WoodwindSynthWrapper::new(
                WoodwindInstrument::Flute,
                self.sample_rate,
            )),
            InstrumentType::Clarinet => Box::new(WoodwindSynthWrapper::new(
                WoodwindInstrument::BbClarinet,
                self.sample_rate,
            )),
            InstrumentType::Oboe => Box::new(WoodwindSynthWrapper::new(
                WoodwindInstrument::Oboe,
                self.sample_rate,
            )),
            InstrumentType::Bassoon => Box::new(WoodwindSynthWrapper::new(
                WoodwindInstrument::Bassoon,
                self.sample_rate,
            )),
            InstrumentType::Saxophone(sax_type) => {
                let instrument = match sax_type {
                    SaxType::Soprano => WoodwindInstrument::SopranoSax,
                    SaxType::Alto => WoodwindInstrument::AltoSax,
                    SaxType::Tenor => WoodwindInstrument::TenorSax,
                    SaxType::Baritone => WoodwindInstrument::BaritoneSax,
                };
                Box::new(WoodwindSynthWrapper::new(instrument, self.sample_rate))
            }
            InstrumentType::WoodwindSection => {
                Box::new(WoodwindSectionWrapper::new(self.sample_rate))
            }

            // Percussion
            InstrumentType::Timpani => Box::new(TimpaniWrapper::new(self.sample_rate)),
            InstrumentType::Xylophone => Box::new(MalletWrapper::new(
                PercussionInstrument::Xylophone,
                self.sample_rate,
            )),
            InstrumentType::Marimba => Box::new(MalletWrapper::new(
                PercussionInstrument::Marimba,
                self.sample_rate,
            )),
            InstrumentType::Vibraphone => Box::new(MalletWrapper::new(
                PercussionInstrument::Vibraphone,
                self.sample_rate,
            )),
            InstrumentType::Glockenspiel => Box::new(MalletWrapper::new(
                PercussionInstrument::Glockenspiel,
                self.sample_rate,
            )),
            InstrumentType::TubularBells => Box::new(MalletWrapper::new(
                PercussionInstrument::TubularBells,
                self.sample_rate,
            )),
            InstrumentType::Cymbals => Box::new(CymbalWrapper::new(self.sample_rate)),
            InstrumentType::Triangle => Box::new(TriangleWrapper::new(self.sample_rate)),
            InstrumentType::DrumKit => Box::new(DrumKitWrapper::new(self.sample_rate as u32)),

            // Choir
            InstrumentType::SopranoVoice => {
                Box::new(ChoirVoiceWrapper::new(VoiceType::Soprano, self.sample_rate))
            }
            InstrumentType::AltoVoice => {
                Box::new(ChoirVoiceWrapper::new(VoiceType::Alto, self.sample_rate))
            }
            InstrumentType::TenorVoice => {
                Box::new(ChoirVoiceWrapper::new(VoiceType::Tenor, self.sample_rate))
            }
            InstrumentType::BassVoice => {
                Box::new(ChoirVoiceWrapper::new(VoiceType::Bass, self.sample_rate))
            }
            InstrumentType::FullChoir => Box::new(FullChoirWrapper::new(self.sample_rate)),

            // Generic fallback
            InstrumentType::GenericMidi(_) => {
                Box::new(PianoSynthWrapper::new(self.sample_rate as u32))
            }
        }
    }
}

// ============================================================================
// Synthesizer Wrappers - Adapt each synth to the Synthesizer trait
// ============================================================================

/// Guitar synth wrapper
struct GuitarSynthWrapper {
    synth: GuitarSynth,
    volume: f32,
    name: String,
}

impl GuitarSynthWrapper {
    fn new(mut config: GuitarConfig, sample_rate: u32) -> Self {
        config.sample_rate = sample_rate;
        Self {
            synth: GuitarSynth::new(config),
            volume: 1.0,
            name: "Guitar".to_string(),
        }
    }
}

impl Synthesizer for GuitarSynthWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        // Map MIDI note to string/fret
        // Standard tuning: E2=40, A2=45, D3=50, G3=55, B3=59, E4=64
        let tuning = [40u8, 45, 50, 55, 59, 64];

        for (string, &open_note) in tuning.iter().enumerate() {
            if note >= open_note && note <= open_note + 24 {
                let fret = note - open_note;
                self.synth.pluck((string + 1) as u8, fret, velocity);
                return;
            }
        }

        // Fallback: use highest string
        if note > 64 {
            self.synth.pluck(6, (note - 64).min(24), velocity);
        }
    }

    fn note_off(&mut self, note: u8) {
        // Guitar strings ring out naturally, no explicit off needed
        let _ = note;
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let (l, r) = self.synth.next_sample_stereo();
            left.push(l * self.volume);
            right.push(r * self.volume);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.synth.is_active()
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Bass synth wrapper
struct BassSynthWrapper {
    synth: BassSynth,
    volume: f32,
}

impl BassSynthWrapper {
    fn new(sample_rate: u32) -> Self {
        Self {
            synth: BassSynth::new(sample_rate, 4),
            volume: 1.0,
        }
    }
}

impl Synthesizer for BassSynthWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        self.synth.note_on(note, velocity);
    }

    fn note_off(&mut self, note: u8) {
        self.synth.note_off(note);
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let sample = self.synth.next_sample() * self.volume;
            left.push(sample);
            right.push(sample);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        "Bass"
    }

    fn is_active(&self) -> bool {
        true // Bass synth doesn't track active state
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Piano synth wrapper
struct PianoSynthWrapper {
    synth: PianoSynth,
    volume: f32,
}

impl PianoSynthWrapper {
    fn new(sample_rate: u32) -> Self {
        Self {
            synth: PianoSynth::new(sample_rate, 16),
            volume: 1.0,
        }
    }
}

impl Synthesizer for PianoSynthWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        self.synth.note_on(note, velocity);
    }

    fn note_off(&mut self, note: u8) {
        self.synth.note_off(note);
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let sample = self.synth.next_sample() * self.volume;
            // Slight stereo spread based on note
            left.push(sample);
            right.push(sample);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        "Piano"
    }

    fn is_active(&self) -> bool {
        self.synth.is_active()
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    fn sustain(&mut self, on: bool) {
        self.synth.set_sustain(on);
    }
}

/// String synth wrapper (single instrument)
struct StringSynthWrapper {
    synth: StringSynth,
    volume: f32,
    name: String,
    current_note: Option<u8>,
}

impl StringSynthWrapper {
    fn new(instrument: StringInstrument, sample_rate: f32) -> Self {
        Self {
            synth: StringSynth::new(instrument, sample_rate as u32),
            volume: 1.0,
            name: format!("{:?}", instrument),
            current_note: None,
        }
    }
}

impl Synthesizer for StringSynthWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        self.current_note = Some(note);
        self.synth.note_on(note, velocity);
    }

    fn note_off(&mut self, note: u8) {
        self.synth.note_off(note);
        if self.current_note == Some(note) {
            self.current_note = None;
        }
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let (l, r) = self.synth.next_sample_stereo();
            left.push(l * self.volume);
            right.push(r * self.volume);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.synth.is_active()
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// String section wrapper
struct StringSectionWrapper {
    section: StringSection,
    volume: f32,
    current_note: Option<u8>,
}

impl StringSectionWrapper {
    fn new(sample_rate: f32) -> Self {
        Self {
            section: StringSection::violins(8, sample_rate as u32),
            volume: 1.0,
            current_note: None,
        }
    }
}

impl Synthesizer for StringSectionWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        self.current_note = Some(note);
        self.section.note_on(note, velocity);
    }

    fn note_off(&mut self, note: u8) {
        self.section.note_off(note);
        if self.current_note == Some(note) {
            self.current_note = None;
        }
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let (l, r) = self.section.next_sample_stereo();
            left.push(l * self.volume);
            right.push(r * self.volume);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        "String Section"
    }

    fn is_active(&self) -> bool {
        self.section.is_active()
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Brass synth wrapper
struct BrassSynthWrapper {
    synth: BrassSynth,
    volume: f32,
    name: String,
}

impl BrassSynthWrapper {
    fn new(instrument: BrassInstrument, sample_rate: f32) -> Self {
        Self {
            synth: BrassSynth::new(sample_rate, instrument),
            volume: 1.0,
            name: format!("{:?}", instrument),
        }
    }
}

impl Synthesizer for BrassSynthWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        let freq = midi_to_freq(note);
        self.synth.play(freq, velocity);
    }

    fn note_off(&mut self, _note: u8) {
        self.synth.release();
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let sample = self.synth.next_sample() * self.volume;
            left.push(sample);
            right.push(sample);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.synth.is_active()
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Brass section wrapper
struct BrassSectionWrapper {
    section: BrassSection,
    volume: f32,
}

impl BrassSectionWrapper {
    fn new(sample_rate: f32) -> Self {
        Self {
            section: BrassSection::trumpets(3, sample_rate),
            volume: 1.0,
        }
    }
}

impl Synthesizer for BrassSectionWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        let freq = midi_to_freq(note);
        self.section.play(freq, velocity);
    }

    fn note_off(&mut self, _note: u8) {
        self.section.release();
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let (l, r) = self.section.next_sample_stereo();
            left.push(l * self.volume);
            right.push(r * self.volume);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        "Brass Section"
    }

    fn is_active(&self) -> bool {
        self.section.is_active()
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Woodwind synth wrapper
struct WoodwindSynthWrapper {
    synth: WoodwindSynth,
    volume: f32,
    name: String,
}

impl WoodwindSynthWrapper {
    fn new(instrument: WoodwindInstrument, sample_rate: f32) -> Self {
        Self {
            synth: WoodwindSynth::new(instrument, sample_rate),
            volume: 1.0,
            name: format!("{:?}", instrument),
        }
    }
}

impl Synthesizer for WoodwindSynthWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        let freq = midi_to_freq(note);
        self.synth.note_on(freq, velocity);
    }

    fn note_off(&mut self, _note: u8) {
        self.synth.note_off();
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let sample = self.synth.process() * self.volume;
            left.push(sample);
            right.push(sample);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.synth.is_active()
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Woodwind section wrapper
struct WoodwindSectionWrapper {
    section: WoodwindSection,
    volume: f32,
}

impl WoodwindSectionWrapper {
    fn new(sample_rate: f32) -> Self {
        Self {
            section: WoodwindSection::flutes(3, sample_rate),
            volume: 1.0,
        }
    }
}

impl Synthesizer for WoodwindSectionWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        let freq = midi_to_freq(note);
        self.section.note_on(freq, velocity);
    }

    fn note_off(&mut self, _note: u8) {
        self.section.note_off();
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let (l, r) = self.section.process();
            left.push(l * self.volume);
            right.push(r * self.volume);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        "Woodwind Section"
    }

    fn is_active(&self) -> bool {
        self.section.is_active()
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Timpani wrapper
struct TimpaniWrapper {
    synths: Vec<TimpaniSynth>,
    active_notes: HashMap<u8, usize>,
    volume: f32,
}

impl TimpaniWrapper {
    fn new(sample_rate: f32) -> Self {
        // Create 4 timpani of different sizes
        Self {
            synths: vec![
                TimpaniSynth::timpani_20(sample_rate),
                TimpaniSynth::timpani_26(sample_rate),
                TimpaniSynth::timpani_26(sample_rate),
                TimpaniSynth::timpani_32(sample_rate),
            ],
            active_notes: HashMap::new(),
            volume: 1.0,
        }
    }
}

impl Synthesizer for TimpaniWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        // Find an available timpani
        for (i, synth) in self.synths.iter_mut().enumerate() {
            if !synth.is_active() {
                let freq = midi_to_freq(note);
                synth.note_on(freq, velocity);
                self.active_notes.insert(note, i);
                return;
            }
        }
        // All busy, use first one
        let freq = midi_to_freq(note);
        self.synths[0].note_on(freq, velocity);
        self.active_notes.insert(note, 0);
    }

    fn note_off(&mut self, _note: u8) {
        // Timpani rings out naturally
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let mut sample = 0.0;
            for synth in &mut self.synths {
                sample += synth.process();
            }
            sample *= self.volume;
            left.push(sample);
            right.push(sample);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        "Timpani"
    }

    fn is_active(&self) -> bool {
        self.synths.iter().any(|s| s.is_active())
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Mallet percussion wrapper
struct MalletWrapper {
    synths: Vec<MalletSynth>,
    instrument: PercussionInstrument,
    active_notes: HashMap<u8, usize>,
    volume: f32,
}

impl MalletWrapper {
    fn new(instrument: PercussionInstrument, sample_rate: f32) -> Self {
        Self {
            synths: (0..8)
                .map(|_| MalletSynth::new(instrument, sample_rate))
                .collect(),
            instrument,
            active_notes: HashMap::new(),
            volume: 1.0,
        }
    }
}

impl Synthesizer for MalletWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        for (i, synth) in self.synths.iter_mut().enumerate() {
            if !synth.is_active() {
                let freq = midi_to_freq(note);
                synth.note_on(freq, velocity);
                self.active_notes.insert(note, i);
                return;
            }
        }
        let freq = midi_to_freq(note);
        self.synths[0].note_on(freq, velocity);
    }

    fn note_off(&mut self, note: u8) {
        if let Some(&idx) = self.active_notes.get(&note) {
            self.synths[idx].note_off();
            self.active_notes.remove(&note);
        }
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let mut sample = 0.0;
            for synth in &mut self.synths {
                sample += synth.process();
            }
            sample *= self.volume;
            left.push(sample);
            right.push(sample);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        match self.instrument {
            PercussionInstrument::Xylophone => "Xylophone",
            PercussionInstrument::Marimba => "Marimba",
            PercussionInstrument::Vibraphone => "Vibraphone",
            PercussionInstrument::Glockenspiel => "Glockenspiel",
            PercussionInstrument::TubularBells => "Tubular Bells",
            _ => "Mallet Percussion",
        }
    }

    fn is_active(&self) -> bool {
        self.synths.iter().any(|s| s.is_active())
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Cymbal wrapper
struct CymbalWrapper {
    synth: CymbalSynth,
    volume: f32,
}

impl CymbalWrapper {
    fn new(sample_rate: f32) -> Self {
        Self {
            synth: CymbalSynth::crash(sample_rate),
            volume: 1.0,
        }
    }
}

impl Synthesizer for CymbalWrapper {
    fn note_on(&mut self, _note: u8, velocity: f32) {
        self.synth.strike(velocity);
    }

    fn note_off(&mut self, _note: u8) {
        self.synth.choke();
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let sample = self.synth.process() * self.volume;
            left.push(sample);
            right.push(sample);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        "Cymbals"
    }

    fn is_active(&self) -> bool {
        self.synth.is_active()
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Triangle wrapper
struct TriangleWrapper {
    synth: CymbalSynth,
    volume: f32,
}

impl TriangleWrapper {
    fn new(sample_rate: f32) -> Self {
        Self {
            synth: CymbalSynth::triangle(sample_rate),
            volume: 1.0,
        }
    }
}

impl Synthesizer for TriangleWrapper {
    fn note_on(&mut self, _note: u8, velocity: f32) {
        self.synth.strike(velocity);
    }

    fn note_off(&mut self, _note: u8) {}

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let sample = self.synth.process() * self.volume;
            left.push(sample);
            right.push(sample);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        "Triangle"
    }

    fn is_active(&self) -> bool {
        self.synth.is_active()
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Drum kit wrapper
struct DrumKitWrapper {
    machine: DrumMachine,
    volume: f32,
}

impl DrumKitWrapper {
    fn new(sample_rate: u32) -> Self {
        Self {
            machine: DrumMachine::new(sample_rate, 16),
            volume: 1.0,
        }
    }
}

impl Synthesizer for DrumKitWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        // Map MIDI drum notes to drum types
        let drum = match note {
            35 | 36 => DrumType::Kick,
            38 | 40 => DrumType::Snare,
            42 | 44 => DrumType::ClosedHiHat,
            46 => DrumType::OpenHiHat,
            41 | 43 => DrumType::FloorTom,
            45 | 47 => DrumType::MidTom,
            48 | 50 => DrumType::HighTom,
            49 | 57 => DrumType::Crash,
            51 | 59 => DrumType::Ride,
            _ => DrumType::Snare,
        };

        self.machine.trigger(drum, velocity);
    }

    fn note_off(&mut self, _note: u8) {}

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let sample = self.machine.next_sample() * self.volume;
            // Mono drum output
            left.push(sample);
            right.push(sample);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        "Drum Kit"
    }

    fn is_active(&self) -> bool {
        true
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Single choir voice wrapper
struct ChoirVoiceWrapper {
    section: ChoirSection,
    volume: f32,
    name: String,
}

impl ChoirVoiceWrapper {
    fn new(voice_type: VoiceType, sample_rate: f32) -> Self {
        Self {
            section: ChoirSection::new(4, sample_rate, voice_type),
            volume: 1.0,
            name: format!("{:?}", voice_type),
        }
    }
}

impl Synthesizer for ChoirVoiceWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        let freq = midi_to_freq(note);
        self.section.set_vowel(Vowel::A);
        self.section.sing(freq, velocity);
    }

    fn note_off(&mut self, _note: u8) {
        self.section.release();
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let (l, r) = self.section.next_sample_stereo();
            left.push(l * self.volume);
            right.push(r * self.volume);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.section.is_active()
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

/// Full choir wrapper
struct FullChoirWrapper {
    choir: Choir,
    volume: f32,
}

impl FullChoirWrapper {
    fn new(sample_rate: f32) -> Self {
        Self {
            choir: Choir::chamber(sample_rate),
            volume: 1.0,
        }
    }
}

impl Synthesizer for FullChoirWrapper {
    fn note_on(&mut self, note: u8, velocity: f32) {
        let freq = midi_to_freq(note);
        self.choir.set_vowel(Vowel::A);
        // Have all sections sing the note (simplified for basic usage)
        self.choir.sopranos.sing(freq, velocity);
        self.choir.altos.sing(freq * 0.5, velocity);
        self.choir.tenors.sing(freq * 0.5, velocity);
        self.choir.basses.sing(freq * 0.25, velocity);
    }

    fn note_off(&mut self, _note: u8) {
        self.choir.release_all();
    }

    fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(num_samples);
        let mut right = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let (l, r) = self.choir.next_sample_stereo();
            left.push(l * self.volume);
            right.push(r * self.volume);
        }

        (left, right)
    }

    fn name(&self) -> &str {
        "Full Choir"
    }

    fn is_active(&self) -> bool {
        self.choir.is_active()
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Convert MIDI note to frequency
fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

// ============================================================================
// Multi-track playback engine
// ============================================================================

/// Track in the multi-instrument engine
pub struct InstrumentTrack {
    pub id: u32,
    pub name: String,
    pub instrument_type: InstrumentType,
    pub synth: Box<dyn Synthesizer>,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
}

/// Multi-instrument playback engine
pub struct MultiInstrumentEngine {
    tracks: Vec<InstrumentTrack>,
    sample_rate: f32,
    factory: SynthFactory,
    master_volume: f32,
    has_solo: bool,
}

impl MultiInstrumentEngine {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            tracks: Vec::new(),
            sample_rate,
            factory: SynthFactory::new(sample_rate),
            master_volume: 1.0,
            has_solo: false,
        }
    }

    /// Add a track with the specified instrument
    pub fn add_track(&mut self, id: u32, name: String, instrument: InstrumentType) -> usize {
        let synth = self.factory.create(&instrument);
        let track = InstrumentTrack {
            id,
            name,
            instrument_type: instrument,
            synth,
            volume: 1.0,
            pan: 0.0,
            muted: false,
            solo: false,
        };
        self.tracks.push(track);
        self.tracks.len() - 1
    }

    /// Add a track from MIDI program number
    pub fn add_track_from_midi(&mut self, id: u32, name: String, program: u8) -> usize {
        let instrument = InstrumentType::from_midi_program(program);
        self.add_track(id, name, instrument)
    }

    /// Get track by index
    pub fn get_track(&self, index: usize) -> Option<&InstrumentTrack> {
        self.tracks.get(index)
    }

    /// Get mutable track by index
    pub fn get_track_mut(&mut self, index: usize) -> Option<&mut InstrumentTrack> {
        self.tracks.get_mut(index)
    }

    /// Find track by ID
    pub fn find_track(&self, id: u32) -> Option<usize> {
        self.tracks.iter().position(|t| t.id == id)
    }

    /// Trigger note on for a track
    pub fn note_on(&mut self, track_index: usize, note: u8, velocity: f32) {
        if let Some(track) = self.tracks.get_mut(track_index) {
            track.synth.note_on(note, velocity);
        }
    }

    /// Trigger note off for a track
    pub fn note_off(&mut self, track_index: usize, note: u8) {
        if let Some(track) = self.tracks.get_mut(track_index) {
            track.synth.note_off(note);
        }
    }

    /// Set track volume
    pub fn set_track_volume(&mut self, track_index: usize, volume: f32) {
        if let Some(track) = self.tracks.get_mut(track_index) {
            track.volume = volume;
            track.synth.set_volume(volume);
        }
    }

    /// Set track pan
    pub fn set_track_pan(&mut self, track_index: usize, pan: f32) {
        if let Some(track) = self.tracks.get_mut(track_index) {
            track.pan = pan.clamp(-1.0, 1.0);
        }
    }

    /// Mute/unmute track
    pub fn set_track_muted(&mut self, track_index: usize, muted: bool) {
        if let Some(track) = self.tracks.get_mut(track_index) {
            track.muted = muted;
        }
    }

    /// Solo/unsolo track
    pub fn set_track_solo(&mut self, track_index: usize, solo: bool) {
        if let Some(track) = self.tracks.get_mut(track_index) {
            track.solo = solo;
        }
        self.has_solo = self.tracks.iter().any(|t| t.solo);
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 2.0);
    }

    /// Process all tracks and mix to stereo
    pub fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = vec![0.0; num_samples];
        let mut right = vec![0.0; num_samples];

        for track in &mut self.tracks {
            // Skip muted tracks
            if track.muted {
                continue;
            }

            // Skip non-solo tracks if any track is solo'd
            if self.has_solo && !track.solo {
                continue;
            }

            // Generate audio from this track
            let (track_left, track_right) = track.synth.process(num_samples);

            // Apply panning and mix
            let pan = track.pan;
            let left_gain = ((1.0 - pan) * 0.5).sqrt();
            let right_gain = ((1.0 + pan) * 0.5).sqrt();

            for i in 0..num_samples {
                left[i] += track_left[i] * left_gain * track.volume;
                right[i] += track_right[i] * right_gain * track.volume;
            }
        }

        // Apply master volume
        for i in 0..num_samples {
            left[i] *= self.master_volume;
            right[i] *= self.master_volume;
        }

        (left, right)
    }

    /// Get track count
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Check if any tracks are active
    pub fn is_active(&self) -> bool {
        self.tracks.iter().any(|t| t.synth.is_active())
    }

    /// Clear all tracks
    pub fn clear(&mut self) {
        self.tracks.clear();
        self.has_solo = false;
    }

    /// Remove track by ID
    pub fn remove_track(&mut self, id: u32) {
        if let Some(idx) = self.find_track(id) {
            self.tracks.remove(idx);
            self.has_solo = self.tracks.iter().any(|t| t.solo);
        }
    }

    /// Get track by ID
    pub fn get_track_by_id(&self, id: u32) -> Option<&InstrumentTrack> {
        self.tracks.iter().find(|t| t.id == id)
    }

    /// Get mutable track by ID
    pub fn get_track_by_id_mut(&mut self, id: u32) -> Option<&mut InstrumentTrack> {
        self.tracks.iter_mut().find(|t| t.id == id)
    }

    /// Trigger note on for a track by ID
    pub fn note_on_by_id(&mut self, id: u32, note: u8, velocity: f32) {
        if let Some(track) = self.get_track_by_id_mut(id) {
            track.synth.note_on(note, velocity);
        }
    }

    /// Trigger note off for a track by ID
    pub fn note_off_by_id(&mut self, id: u32, note: u8) {
        if let Some(track) = self.get_track_by_id_mut(id) {
            track.synth.note_off(note);
        }
    }

    /// Set track volume by ID
    pub fn set_track_volume_by_id(&mut self, id: u32, volume: f32) {
        if let Some(track) = self.get_track_by_id_mut(id) {
            track.volume = volume;
            track.synth.set_volume(volume);
        }
    }

    /// Mute track by ID
    pub fn mute_track_by_id(&mut self, id: u32, muted: bool) {
        if let Some(track) = self.get_track_by_id_mut(id) {
            track.muted = muted;
        }
    }

    /// Solo track by ID
    pub fn solo_track_by_id(&mut self, id: u32, solo: bool) {
        if let Some(track) = self.get_track_by_id_mut(id) {
            track.solo = solo;
        }
        self.has_solo = self.tracks.iter().any(|t| t.solo);
    }

    /// Process a single track and return its audio (for spatial rendering)
    pub fn process_track(&mut self, id: u32, num_samples: usize) -> Option<(Vec<f32>, Vec<f32>)> {
        let track = self.tracks.iter_mut().find(|t| t.id == id)?;

        // Skip if muted or (has_solo and not solo'd)
        if track.muted || (self.has_solo && !track.solo) {
            return Some((vec![0.0; num_samples], vec![0.0; num_samples]));
        }

        Some(track.synth.process(num_samples))
    }

    /// Get all tracks
    pub fn tracks(&self) -> &[InstrumentTrack] {
        &self.tracks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_program_mapping() {
        // Piano
        assert!(matches!(
            InstrumentType::from_midi_program(0),
            InstrumentType::GrandPiano
        ));

        // Violin
        assert!(matches!(
            InstrumentType::from_midi_program(40),
            InstrumentType::Violin
        ));

        // Trumpet
        assert!(matches!(
            InstrumentType::from_midi_program(56),
            InstrumentType::Trumpet
        ));

        // Flute
        assert!(matches!(
            InstrumentType::from_midi_program(73),
            InstrumentType::Flute
        ));
    }

    #[test]
    fn test_synth_factory() {
        let factory = SynthFactory::new(44100.0);

        let violin = factory.create(&InstrumentType::Violin);
        assert_eq!(violin.name(), "Violin");

        let trumpet = factory.create(&InstrumentType::Trumpet);
        assert_eq!(trumpet.name(), "Trumpet");

        let piano = factory.create(&InstrumentType::GrandPiano);
        assert_eq!(piano.name(), "Piano");
    }

    #[test]
    fn test_multi_instrument_engine() {
        let mut engine = MultiInstrumentEngine::new(44100.0);

        // Add tracks
        let violin_idx = engine.add_track(1, "Violin I".to_string(), InstrumentType::Violin);
        let trumpet_idx = engine.add_track(2, "Trumpet".to_string(), InstrumentType::Trumpet);

        assert_eq!(engine.track_count(), 2);

        // Trigger notes
        engine.note_on(violin_idx, 69, 0.8); // A4
        engine.note_on(trumpet_idx, 72, 0.7); // C5

        // Process audio
        let (left, right) = engine.process(1024);
        assert_eq!(left.len(), 1024);
        assert_eq!(right.len(), 1024);

        // Should have produced sound
        let max_amp = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_amp > 0.001);
    }

    #[test]
    fn test_track_mute_solo() {
        let mut engine = MultiInstrumentEngine::new(44100.0);

        let t1 = engine.add_track(1, "Track 1".to_string(), InstrumentType::GrandPiano);
        let t2 = engine.add_track(2, "Track 2".to_string(), InstrumentType::Violin);

        // Mute track 1
        engine.set_track_muted(t1, true);
        engine.note_on(t1, 60, 0.8);
        engine.note_on(t2, 60, 0.8);

        let (left, _) = engine.process(512);
        let amp1 = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

        // Solo track 2
        engine.set_track_muted(t1, false);
        engine.set_track_solo(t2, true);

        // Only track 2 should be heard
        assert!(engine.has_solo);
    }

    #[test]
    fn test_instrument_categories() {
        assert_eq!(
            InstrumentType::Violin.category(),
            InstrumentCategory::Strings
        );
        assert_eq!(
            InstrumentType::Trumpet.category(),
            InstrumentCategory::Brass
        );
        assert_eq!(
            InstrumentType::Flute.category(),
            InstrumentCategory::Woodwind
        );
        assert_eq!(
            InstrumentType::Timpani.category(),
            InstrumentCategory::Percussion
        );
        assert_eq!(
            InstrumentType::FullChoir.category(),
            InstrumentCategory::Choir
        );
    }

    #[test]
    fn test_orchestra_section_mapping() {
        assert_eq!(
            InstrumentType::Violin.orchestra_section(),
            Some(OrchestraSection::Violin1)
        );
        assert_eq!(
            InstrumentType::Trumpet.orchestra_section(),
            Some(OrchestraSection::Trumpet)
        );
        assert_eq!(
            InstrumentType::Timpani.orchestra_section(),
            Some(OrchestraSection::Timpani)
        );

        // Non-orchestral instruments
        assert_eq!(InstrumentType::ElectricBass.orchestra_section(), None);
    }
}
