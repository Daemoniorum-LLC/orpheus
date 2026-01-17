//! Virtuoso guitar techniques
//!
//! Complete technique notation for technical death metal and beyond.

use serde::{Deserialize, Serialize};

/// All note-level techniques
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Technique {
    // ===== LEGATO TECHNIQUES =====
    /// Hammer-on to higher fret
    HammerOn,
    /// Pull-off to lower fret
    PullOff,
    /// Tapping with right hand (or specified hand)
    Tap(TapType),
    /// Legato slide to target fret
    LegatoSlide(SlideDirection),

    // ===== SLIDES =====
    /// Shift slide (re-pick at destination)
    ShiftSlide(SlideDirection),
    /// Slide into note from nowhere
    SlideIn(SlideDirection),
    /// Slide out of note to nowhere
    SlideOut(SlideDirection),

    // ===== BENDS =====
    /// Bend with curve points
    Bend(BendData),
    /// Pre-bend (bend before picking)
    PreBend(BendAmount),
    /// Pre-bend then release
    PreBendRelease(BendAmount),
    /// Unison bend (bend to match another string)
    UnisonBend(u8), // Target string

    // ===== VIBRATO =====
    /// Standard vibrato
    Vibrato(VibratoStyle),
    /// Wide vibrato
    WideVibrato,

    // ===== HARMONICS =====
    /// Natural harmonic at fret
    NaturalHarmonic,
    /// Artificial/harp harmonic (thumb touches string)
    ArtificialHarmonic(u8), // Fret above picked note
    /// Pinch harmonic (pick edge touches string)
    PinchHarmonic,
    /// Tap harmonic
    TapHarmonic(u8), // Fret tapped
    /// Semi-harmonic (partial harmonic)
    SemiHarmonic,
    /// Feedback harmonic (controlled feedback)
    FeedbackHarmonic,

    // ===== WHAMMY BAR / TREMOLO =====
    /// Whammy bar technique
    WhammyBar(WhammyTechnique),

    // ===== PICKING TECHNIQUES =====
    /// Palm mute
    PalmMute(PalmMuteIntensity),
    /// Pick scrape/scratch
    PickScrape,
    /// Rake (muted strings before target)
    Rake,
    /// Tremolo picking
    TremoloPicking,
    /// Sweep picking marker
    SweepPicking(SweepDirection),
    /// Economy picking marker
    EconomyPicking,
    /// Hybrid picking (pick + fingers)
    HybridPicking,
    /// Chicken picking
    ChickenPicking,

    // ===== FRETTING TECHNIQUES =====
    /// Let ring (sustain)
    LetRing,
    /// Staccato (short)
    Staccato,
    /// Accent (emphasized)
    Accent,
    /// Heavy accent / marcato
    Marcato,

    // ===== SPECIAL TECHNIQUES =====
    /// Trill between two frets
    Trill(TrillData),
    /// Grace note (quick note before main)
    GraceNote(GraceNoteData),
    /// Behind the nut bend (for headless or open headstock)
    BehindNutBend(i8), // Semitones
    /// Whammy pedal
    WhammyPedal(WhammyPedalData),
    /// String skip indication
    StringSkip,
    /// Dead note / muted
    DeadNote,
    /// Slap (bass)
    Slap,
    /// Pop (bass)
    Pop,
}

// ===== SLIDE TYPES =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlideDirection {
    Up,   // To higher fret
    Down, // To lower fret
}

// ===== TAP TYPES =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TapType {
    /// Right hand tap (standard)
    RightHand,
    /// Left hand tap
    LeftHand,
    /// Two-hand tapping
    TwoHand,
    /// Eight-finger tapping
    EightFinger,
}

// ===== BEND DATA =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BendData {
    /// Target bend amount
    pub amount: BendAmount,
    /// Bend curve points (for complex bends)
    pub curve: Vec<BendPoint>,
    /// Hold at peak
    pub hold: bool,
    /// Return to original pitch
    pub release: bool,
}

impl BendData {
    pub fn half_step() -> Self {
        Self {
            amount: BendAmount::Half,
            curve: vec![BendPoint::peak(BendAmount::Half)],
            hold: false,
            release: false,
        }
    }

    pub fn full() -> Self {
        Self {
            amount: BendAmount::Full,
            curve: vec![BendPoint::peak(BendAmount::Full)],
            hold: false,
            release: false,
        }
    }

    pub fn full_and_release() -> Self {
        Self {
            amount: BendAmount::Full,
            curve: vec![
                BendPoint::peak(BendAmount::Full),
                BendPoint::release(),
            ],
            hold: false,
            release: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BendAmount {
    /// Quarter step (micro-bend)
    Quarter,
    /// Half step (1 semitone)
    Half,
    /// Full step (2 semitones / whole tone)
    Full,
    /// 1.5 steps (3 semitones)
    OneAndHalf,
    /// 2 steps (4 semitones)
    Two,
    /// 2.5 steps (5 semitones)
    TwoAndHalf,
    /// 3 steps (6 semitones)
    Three,
    /// Custom in cents (100 = 1 semitone)
    Custom(i16),
}

impl BendAmount {
    /// Convert to cents (100 = 1 semitone)
    pub fn cents(&self) -> i16 {
        match self {
            Self::Quarter => 25,
            Self::Half => 100,
            Self::Full => 200,
            Self::OneAndHalf => 300,
            Self::Two => 400,
            Self::TwoAndHalf => 500,
            Self::Three => 600,
            Self::Custom(c) => *c,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BendPoint {
    /// Position in beat (0.0 - 1.0)
    pub position: f32,
    /// Bend amount at this point
    pub amount: BendAmount,
}

impl BendPoint {
    pub fn peak(amount: BendAmount) -> Self {
        Self { position: 0.5, amount }
    }

    pub fn release() -> Self {
        Self { position: 1.0, amount: BendAmount::Custom(0) }
    }
}

// ===== VIBRATO =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VibratoStyle {
    /// Standard guitar vibrato (wrist-based)
    Standard,
    /// Classical vibrato (finger-based)
    Classical,
    /// Wide/heavy vibrato
    Wide,
    /// Fast vibrato
    Fast,
    /// Slow vibrato
    Slow,
    /// Extreme (metal-style)
    Extreme,
}

// ===== WHAMMY BAR TECHNIQUES =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhammyTechnique {
    /// Dive bomb (pitch drops dramatically)
    DiveBomb(DiveDepth),
    /// Pull up (raise pitch)
    PullUp(BendAmount),
    /// Dip (quick down and back)
    Dip(BendAmount),
    /// Scoop (dip into note)
    Scoop(BendAmount),
    /// Flutter (rapid oscillation)
    Flutter,
    /// Gradual dive
    GradualDive(Vec<WhammyPoint>),
    /// Gradual rise
    GradualRise(Vec<WhammyPoint>),
    /// Harmonic squeal with bar
    HarmonicDive,
    /// Gargle (extreme flutter)
    Gargle,
    /// Custom whammy curve
    Custom(Vec<WhammyPoint>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiveDepth {
    /// Standard dive (down to slack)
    Slack,
    /// Octave down
    Octave,
    /// Two octaves down (if possible)
    TwoOctaves,
    /// Partial dive
    Partial(i16), // Cents
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WhammyPoint {
    /// Position in beat (0.0 - 1.0)
    pub position: f32,
    /// Pitch offset in cents (negative = down)
    pub cents: i16,
}

// ===== PALM MUTE =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PalmMuteIntensity {
    /// Light palm mute
    Light,
    /// Medium palm mute (typical metal)
    Medium,
    /// Heavy palm mute (djent-style)
    Heavy,
    /// Extreme (almost fully muted)
    Chug,
}

// ===== SWEEP PICKING =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SweepDirection {
    /// Downward sweep (high to low strings)
    Down,
    /// Upward sweep (low to high strings)
    Up,
    /// Economy (continues direction from previous)
    Economy,
}

// ===== TRILL =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrillData {
    /// Upper fret of trill
    pub upper_fret: u8,
    /// Trill speed
    pub speed: TrillSpeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrillSpeed {
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
    Free, // As fast as possible
}

// ===== GRACE NOTE =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraceNoteData {
    /// Fret of grace note
    pub fret: u8,
    /// Is on the beat (vs before)
    pub on_beat: bool,
    /// Duration type
    pub duration: GraceNoteDuration,
    /// Transition type
    pub transition: GraceNoteTransition,
    /// Velocity
    pub velocity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraceNoteDuration {
    ThirtySecond,
    Sixteenth,
    Eighth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraceNoteTransition {
    /// Normal pick
    None,
    /// Hammer-on to main note
    HammerOn,
    /// Pull-off to main note
    PullOff,
    /// Slide to main note
    Slide,
    /// Bend to main note
    Bend,
}

// ===== WHAMMY PEDAL =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhammyPedalData {
    /// Starting interval (semitones)
    pub start_interval: i8,
    /// Ending interval (semitones)
    pub end_interval: i8,
    /// Curve points for complex sweeps
    pub curve: Vec<WhammyPedalPoint>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WhammyPedalPoint {
    /// Position (0.0 - 1.0)
    pub position: f32,
    /// Interval in semitones
    pub interval: i8,
}

// ===== BEAT-LEVEL EFFECTS =====

/// Effects that apply to an entire beat/chord
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BeatEffect {
    /// Arpeggio direction
    pub arpeggio: Option<ArpeggioDirection>,
    /// Strum direction
    pub strum: Option<StrumDirection>,
    /// Is a brush stroke
    pub brush: bool,
    /// Rasgueado (flamenco)
    pub rasgueado: bool,
    /// Chord name (e.g., "Am7")
    pub chord_name: Option<String>,
    /// Chord diagram
    pub chord_diagram: Option<ChordDiagram>,
    /// Tempo change
    pub tempo_change: Option<TempoChange>,
    /// Dynamics marking
    pub dynamics: Option<Dynamics>,
    /// Fermata (pause/hold)
    pub fermata: bool,
}

/// Chord diagram for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordDiagram {
    pub name: String,
    pub base_fret: u8,
    /// Fret for each string (0 = open, 255 = muted)
    pub frets: Vec<u8>,
    /// Optional fingering
    pub fingering: Vec<Option<Fingering>>,
    /// Barre info: (start_string, end_string, fret)
    pub barre: Option<(u8, u8, u8)>,
}

use super::Fingering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArpeggioDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrumDirection {
    Down,
    Up,
}

/// Tempo change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempoChange {
    /// New tempo in BPM
    pub bpm: f64,
    /// Change type
    pub change_type: TempoChangeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TempoChangeType {
    /// Immediate change
    Immediate,
    /// Gradual accelerando
    Accelerando,
    /// Gradual ritardando
    Ritardando,
    /// Rallentando (slowing at end of phrase)
    Rallentando,
    /// A tempo (return to original)
    ATempo,
}

/// Dynamic markings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dynamics {
    /// Pianississimo (ppp)
    PPP,
    /// Pianissimo (pp)
    PP,
    /// Piano (p)
    P,
    /// Mezzo-piano (mp)
    MP,
    /// Mezzo-forte (mf)
    MF,
    /// Forte (f)
    F,
    /// Fortissimo (ff)
    FF,
    /// Fortississimo (fff)
    FFF,
    /// Sforzando (sudden accent)
    Sfz,
    /// Crescendo
    Crescendo,
    /// Decrescendo
    Decrescendo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bend_amounts() {
        assert_eq!(BendAmount::Half.cents(), 100);
        assert_eq!(BendAmount::Full.cents(), 200);
        assert_eq!(BendAmount::OneAndHalf.cents(), 300);
    }

    #[test]
    fn test_bend_data() {
        let bend = BendData::full_and_release();
        assert_eq!(bend.amount, BendAmount::Full);
        assert!(bend.release);
        assert_eq!(bend.curve.len(), 2);
    }

    #[test]
    fn test_whammy_techniques() {
        let dive = WhammyTechnique::DiveBomb(DiveDepth::Slack);
        match dive {
            WhammyTechnique::DiveBomb(DiveDepth::Slack) => {}
            _ => panic!("Expected dive bomb"),
        }
    }
}
