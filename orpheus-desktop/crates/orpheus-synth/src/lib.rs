//! # Orpheus Synth
//!
//! Audio synthesis engine for Orpheus.
//!
//! Features:
//! - Guitar synthesis using Karplus-Strong algorithm
//! - Amp simulation with tube stages and tonestack EQ
//! - Piano synthesis using additive synthesis
//! - Bass synthesis using subtractive synthesis
//! - Bowed string synthesis (violin, viola, cello, bass)
//! - Brass synthesis (trumpet, trombone, horn, tuba) with mutes
//! - Woodwind synthesis (flute, clarinet, oboe, bassoon, saxophone)
//! - Orchestral percussion (timpani, mallet instruments, cymbals, drums)
//! - Choir/vocal synthesis with formant filtering and consonants
//! - Orchestra mixer with spatial positioning and concert hall simulation
//! - Multi-instrument routing with MIDI program mapping
//! - Drum synthesis with common drum sounds
//! - Real-time audio output via cpal
//! - Pattern-based drum sequencing
//! - Dantalion integration for AI-generated audio (music, SFX, TTS)

pub mod karplus_strong;
pub mod guitar;
pub mod amp;
pub mod cabinet;
pub mod piano;
pub mod bass;
pub mod strings;
pub mod brass;
pub mod woodwinds;
pub mod orchestral_percussion;
pub mod choir;
pub mod orchestra;
pub mod instrument_router;
pub mod orchestra_renderer;
pub mod playback;
pub mod sequencer;
pub mod drums;
pub mod effects;
pub mod metronome;
pub mod wav;

#[cfg(feature = "audio-output")]
pub mod output;

// Engine is available when either audio-output or simulation is enabled
#[cfg(any(feature = "audio-output", feature = "simulation"))]
pub mod engine;

#[cfg(feature = "dantalion")]
pub mod dantalion;

pub use karplus_strong::KarplusStrong;
pub use guitar::{GuitarSynth, GuitarConfig, StringState};
pub use amp::{AmpSimulator, AmpChannel, AmpPreset, TubeStage, Tonestack, TonestackVoicing, PowerAmp};
pub use cabinet::{CabinetSimulator, CabinetType, CabinetPreset};
pub use piano::{PianoSynth, PianoVoice, Envelope as PianoEnvelope};
pub use bass::{BassSynth, BassVoice, BassPreset, Waveform};
pub use strings::{StringSynth, StringSection, StringInstrument, StringArticulation, BowedString};
pub use brass::{
    BrassSynth, BrassSection, BrassInstrument, BrassMute, BrassArticulation,
};
pub use woodwinds::{
    WoodwindSynth, WoodwindSection, WoodwindInstrument, WoodwindArticulation,
    ExcitationType, BoreType, AirJetOscillator, SingleReedOscillator, DoubleReedOscillator,
};
pub use orchestral_percussion::{
    PercussionType, PercussionInstrument, MalletType,
    TimpaniSynth, MalletSynth, CymbalSynth, DrumSynth as OrchestralDrumSynth,
    AuxPercussionSynth,
};
pub use choir::{
    VocalSynth, ChoirSection, Choir, VoiceType, Vowel, GlottalSource, FormantBank,
    Consonant, ConsonantType, ConsonantGenerator, Phoneme, Syllable,
};
pub use orchestra::{
    Position, Listener, HallType, ConcertHall, OrchestraSection, Musician,
    SeatingArrangement, SpatialProcessor, SectionMixer, Orchestra,
    SectionVisualization, OrchestraVisualization,
};
pub use instrument_router::{
    Synthesizer, InstrumentCategory, InstrumentType, SaxType,
    SynthFactory, InstrumentTrack, MultiInstrumentEngine,
};
pub use orchestra_renderer::{
    OrchestraRenderer, OrchestraRendererConfig, OrchestraRendererVisualization,
    TrackPosition, TrackVisualization,
};
pub use playback::{PlaybackEngine, PlaybackState, NoteEvent, NoteEventType};
pub use sequencer::{TabSequencer, SequencerEvent};
pub use drums::{DrumType, DrumVoice, DrumMachine, DrumPattern, DrumStep};
pub use effects::{Distortion, DistortionType, NoiseGate, ThreeBandEq, Delay, StereoDelay, Reverb};
pub use metronome::{Metronome, MetronomeTiming};
pub use wav::{WavWriter, write_wav, write_wav_mono};

#[cfg(feature = "audio-output")]
pub use output::{AudioOutput, AudioMessage, list_output_devices, default_output_device_name};

#[cfg(feature = "audio-output")]
pub use engine::{AudioEngine, AudioEngineConfig};

#[cfg(feature = "dantalion")]
pub use dantalion::{
    AudioType, AudioModel, AudioGenerationRequest, GenerationStatus, GenerationResponse,
    DantalionConfig, DantalionClient,
};

/// Result type for synth operations
pub type Result<T> = std::result::Result<T, Error>;

/// Synth error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid note: {0}")]
    InvalidNote(String),

    #[error("Buffer underrun")]
    BufferUnderrun,

    #[error("Invalid sample rate: {0}")]
    InvalidSampleRate(u32),

    #[error("Audio output error: {0}")]
    AudioOutput(String),
}

/// Standard sample rates
pub const SAMPLE_RATE_44100: u32 = 44100;
pub const SAMPLE_RATE_48000: u32 = 48000;

/// Concert pitch A4 frequency
pub const A4_FREQ: f32 = 440.0;

/// Convert MIDI note number to frequency
pub fn midi_to_freq(midi_note: u8) -> f32 {
    A4_FREQ * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0)
}

/// Convert frequency to period in samples
pub fn freq_to_period(freq: f32, sample_rate: u32) -> usize {
    (sample_rate as f32 / freq).round() as usize
}

/// Standard guitar tuning MIDI notes (E2, A2, D3, G3, B3, E4)
pub const STANDARD_TUNING: [u8; 6] = [40, 45, 50, 55, 59, 64];

/// Bass guitar tuning MIDI notes (E1, A1, D2, G2)
pub const BASS_TUNING: [u8; 4] = [28, 33, 38, 43];

/// 7-string guitar tuning (B1, E2, A2, D3, G3, B3, E4)
pub const SEVEN_STRING_TUNING: [u8; 7] = [35, 40, 45, 50, 55, 59, 64];

/// 8-string guitar tuning (F#1, B1, E2, A2, D3, G3, B3, E4)
pub const EIGHT_STRING_TUNING: [u8; 8] = [30, 35, 40, 45, 50, 55, 59, 64];

// Drop tunings for metal
/// Drop D tuning (D2, A2, D3, G3, B3, E4)
pub const DROP_D_TUNING: [u8; 6] = [38, 45, 50, 55, 59, 64];

/// Drop C tuning (C2, G2, C3, F3, A3, D4)
pub const DROP_C_TUNING: [u8; 6] = [36, 43, 48, 53, 57, 62];

/// Drop B tuning (B1, F#2, B2, E3, G#3, C#4)
pub const DROP_B_TUNING: [u8; 6] = [35, 42, 47, 52, 56, 61];

/// Drop A tuning (A1, E2, A2, D3, F#3, B3)
pub const DROP_A_TUNING: [u8; 6] = [33, 40, 45, 50, 54, 59];

/// 7-string Drop A tuning (A1, E2, A2, D3, G3, B3, E4)
pub const SEVEN_STRING_DROP_A: [u8; 7] = [33, 40, 45, 50, 55, 59, 64];

/// 8-string Drop E tuning (E1, B1, E2, A2, D3, G3, B3, E4)
pub const EIGHT_STRING_DROP_E: [u8; 8] = [28, 35, 40, 45, 50, 55, 59, 64];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_to_freq() {
        // A4 = 440 Hz
        assert!((midi_to_freq(69) - 440.0).abs() < 0.01);
        // A3 = 220 Hz
        assert!((midi_to_freq(57) - 220.0).abs() < 0.01);
        // E2 (low E guitar) ≈ 82.4 Hz
        assert!((midi_to_freq(40) - 82.41).abs() < 0.1);
    }

    #[test]
    fn test_freq_to_period() {
        // 440 Hz at 44100 sample rate = 100.23 samples
        let period = freq_to_period(440.0, 44100);
        assert_eq!(period, 100);
    }
}
