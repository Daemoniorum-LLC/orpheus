//! Guitar Pro data types

use serde::{Deserialize, Serialize};

/// A parsed Guitar Pro file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuitarProFile {
    /// File format version
    pub version: GpVersion,
    /// Song metadata
    pub info: SongInfo,
    /// Tracks in the file
    pub tracks: Vec<Track>,
    /// Measures/bars
    pub measures: Vec<Measure>,
    /// Tempo in BPM
    pub tempo: u16,
    /// Time signature
    pub time_signature: TimeSignature,
    /// Key signature
    pub key_signature: KeySignature,
}

impl Default for GuitarProFile {
    fn default() -> Self {
        Self {
            version: GpVersion::Unknown,
            info: SongInfo::default(),
            tracks: Vec::new(),
            measures: Vec::new(),
            tempo: 120,
            time_signature: TimeSignature::default(),
            key_signature: KeySignature::default(),
        }
    }
}

/// Guitar Pro file version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpVersion {
    Gp3,
    Gp4,
    Gp5,
    Gp6,
    Gp7,
    Unknown,
}

impl GpVersion {
    pub fn from_header(header: &str) -> Self {
        if header.contains("v5.") {
            Self::Gp5
        } else if header.contains("v4.") {
            Self::Gp4
        } else if header.contains("v3.") {
            Self::Gp3
        } else {
            Self::Unknown
        }
    }
}

/// Song information/metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SongInfo {
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub album: String,
    pub author: String,
    pub copyright: String,
    pub tab_author: String,
    pub instructions: String,
    pub comments: Vec<String>,
}

/// A track in the file (instrument)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Track number (1-indexed)
    pub number: u8,
    /// Track name
    pub name: String,
    /// Number of strings (for string instruments)
    pub strings: u8,
    /// String tuning (MIDI note numbers)
    pub tuning: Vec<u8>,
    /// MIDI channel
    pub channel: u8,
    /// MIDI program (instrument)
    pub program: u8,
    /// Is this a drum track?
    pub is_drums: bool,
    /// Track volume (0-127)
    pub volume: u8,
    /// Track pan (0-127, 64 = center)
    pub pan: u8,
    /// Capo fret
    pub capo: u8,
    /// Track color (RGB)
    pub color: (u8, u8, u8),
}

impl Default for Track {
    fn default() -> Self {
        Self {
            number: 1,
            name: String::new(),
            strings: 6,
            tuning: vec![64, 59, 55, 50, 45, 40], // Standard guitar tuning E A D G B E
            channel: 0,
            program: 25, // Steel guitar
            is_drums: false,
            volume: 100,
            pan: 64,
            capo: 0,
            color: (255, 0, 0),
        }
    }
}

/// A measure/bar in the song
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measure {
    /// Measure number (1-indexed)
    pub number: u16,
    /// Time signature (if changed)
    pub time_signature: Option<TimeSignature>,
    /// Tempo (if changed)
    pub tempo: Option<u16>,
    /// Beats in this measure, per track
    pub beats: Vec<TrackBeats>,
    /// Is this a repeat start?
    pub repeat_start: bool,
    /// Number of repeat endings
    pub repeat_end: u8,
    /// Marker/section name
    pub marker: Option<String>,
}

impl Default for Measure {
    fn default() -> Self {
        Self {
            number: 1,
            time_signature: None,
            tempo: None,
            beats: Vec::new(),
            repeat_start: false,
            repeat_end: 0,
            marker: None,
        }
    }
}

/// Beats for a single track in a measure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackBeats {
    /// Track number
    pub track: u8,
    /// Beats/notes in this measure
    pub beats: Vec<Beat>,
}

/// A single beat (can contain multiple notes - chord)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beat {
    /// Notes in this beat
    pub notes: Vec<Note>,
    /// Beat duration
    pub duration: Duration,
    /// Is this a rest?
    pub is_rest: bool,
    /// Dotted note
    pub dotted: bool,
    /// Tuplet (e.g., 3 for triplet)
    pub tuplet: Option<u8>,
    /// Text annotation
    pub text: Option<String>,
    /// Beat effects
    pub effects: BeatEffects,
}

impl Default for Beat {
    fn default() -> Self {
        Self {
            notes: Vec::new(),
            duration: Duration::Quarter,
            is_rest: false,
            dotted: false,
            tuplet: None,
            text: None,
            effects: BeatEffects::default(),
        }
    }
}

/// A single note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// String number (1 = highest pitch string)
    pub string: u8,
    /// Fret number (0 = open string)
    pub fret: u8,
    /// Note velocity (0-127)
    pub velocity: u8,
    /// Tied to previous note
    pub tied: bool,
    /// Ghost note
    pub ghost: bool,
    /// Note effects
    pub effects: NoteEffects,
}

impl Default for Note {
    fn default() -> Self {
        Self {
            string: 1,
            fret: 0,
            velocity: 100,
            tied: false,
            ghost: false,
            effects: NoteEffects::default(),
        }
    }
}

/// Note duration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Duration {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
}

impl Duration {
    pub fn from_value(value: i8) -> Self {
        match value {
            -2 => Self::Whole,
            -1 => Self::Half,
            0 => Self::Quarter,
            1 => Self::Eighth,
            2 => Self::Sixteenth,
            3 => Self::ThirtySecond,
            4 => Self::SixtyFourth,
            _ => Self::Quarter,
        }
    }

    pub fn to_ticks(&self) -> u32 {
        match self {
            Self::Whole => 960 * 4,
            Self::Half => 960 * 2,
            Self::Quarter => 960,
            Self::Eighth => 480,
            Self::Sixteenth => 240,
            Self::ThirtySecond => 120,
            Self::SixtyFourth => 60,
        }
    }
}

/// Beat-level effects
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BeatEffects {
    pub staccato: bool,
    pub palm_mute: bool,
    pub let_ring: bool,
    pub fade_in: bool,
    pub stroke: Option<Stroke>,
    pub tremolo_bar: Option<Vec<BendPoint>>,
}

/// Stroke direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Stroke {
    Up(u8),   // Duration value
    Down(u8), // Duration value
}

/// Note-level effects
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoteEffects {
    pub hammer_on: bool,
    pub pull_off: bool,
    pub slide: Option<SlideType>,
    pub bend: Option<Vec<BendPoint>>,
    pub vibrato: bool,
    pub harmonic: Option<HarmonicType>,
    pub trill: Option<(u8, Duration)>, // (fret, speed)
    pub grace_note: Option<GraceNote>,
    pub left_hand_finger: Option<Finger>,
    pub right_hand_finger: Option<Finger>,
}

/// Slide types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SlideType {
    IntoFromAbove,
    IntoFromBelow,
    OutDownwards,
    OutUpwards,
    ShiftSlide,
    LegatoSlide,
}

/// Harmonic types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HarmonicType {
    Natural,
    Artificial,
    Pinch,
    Tap,
    Semi,
    Feedback,
}

/// Bend point for tremolo bar or string bends
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BendPoint {
    /// Position (0-60, represents 60ths of the beat)
    pub position: u8,
    /// Bend value in quarter tones (100 = 1 semitone)
    pub value: i16,
    /// Is this a vibrato point?
    pub vibrato: bool,
}

/// Grace note
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GraceNote {
    pub fret: u8,
    pub velocity: u8,
    pub duration: u8,
    pub on_beat: bool,
    pub dead: bool,
}

/// Finger notation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Finger {
    Open,
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

/// Time signature
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self {
            numerator: 4,
            denominator: 4,
        }
    }
}

/// Key signature
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct KeySignature {
    /// Number of sharps (positive) or flats (negative)
    pub key: i8,
    /// Is this a minor key?
    pub minor: bool,
}
