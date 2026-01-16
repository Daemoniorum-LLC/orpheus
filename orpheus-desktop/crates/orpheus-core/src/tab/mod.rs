//! Virtuoso-grade tablature system
//!
//! Comprehensive tab notation supporting extended range instruments,
//! advanced techniques, and complex rhythmic structures.

mod instrument;
mod technique;
mod rhythm;
mod drums;
mod notation;

pub use instrument::*;
pub use technique::*;
pub use rhythm::*;
pub use drums::*;
pub use notation::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A complete tablature document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabDocument {
    /// Unique document ID
    pub id: Uuid,
    /// Song metadata
    pub metadata: TabMetadata,
    /// Instrument tracks
    pub tracks: Vec<TabTrack>,
    /// Global tempo map (tempo changes over time)
    pub tempo_map: TempoMap,
    /// Measures
    pub measures: Vec<TabMeasure>,
    /// Markers/sections
    pub markers: Vec<SectionMarker>,
}

impl Default for TabDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl TabDocument {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            metadata: TabMetadata::default(),
            tracks: Vec::new(),
            tempo_map: TempoMap::new(120.0),
            measures: Vec::new(),
            markers: Vec::new(),
        }
    }

    /// Add a new track
    pub fn add_track(&mut self, track: TabTrack) -> Uuid {
        let id = track.id;
        self.tracks.push(track);
        id
    }

    /// Add a new measure
    pub fn add_measure(&mut self) -> usize {
        let num = self.measures.len() + 1;
        self.measures.push(TabMeasure::new(num));
        num - 1
    }

    /// Get total duration in ticks
    pub fn total_ticks(&self) -> u64 {
        self.measures.iter().map(|m| m.duration_ticks()).sum()
    }
}

/// Song metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TabMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: Option<u16>,
    pub transcriber: String,
    pub copyright: String,
    pub instructions: String,
    pub comments: Vec<String>,
    /// Genre tags
    pub tags: Vec<String>,
    /// Difficulty rating (1-10)
    pub difficulty: u8,
}

/// A track in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabTrack {
    pub id: Uuid,
    pub name: String,
    pub instrument: Instrument,
    /// Track color for UI
    pub color: (u8, u8, u8),
    /// Track volume (0.0 - 1.0)
    pub volume: f32,
    /// Track pan (-1.0 left, 1.0 right)
    pub pan: f32,
    /// Is muted
    pub muted: bool,
    /// Is soloed
    pub solo: bool,
}

impl TabTrack {
    pub fn new(name: &str, instrument: Instrument) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            instrument,
            color: (52, 152, 219),
            volume: 0.8,
            pan: 0.0,
            muted: false,
            solo: false,
        }
    }

    pub fn guitar(name: &str) -> Self {
        Self::new(name, Instrument::guitar_standard())
    }

    pub fn bass(name: &str) -> Self {
        Self::new(name, Instrument::bass_standard())
    }

    pub fn drums(name: &str) -> Self {
        Self::new(name, Instrument::Drums(DrumKit::standard()))
    }
}

/// A measure/bar in the tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabMeasure {
    /// Measure number (1-indexed for display)
    pub number: usize,
    /// Time signature (if changed from previous)
    pub time_signature: Option<TimeSignature>,
    /// Tempo change (if any)
    pub tempo: Option<TempoChange>,
    /// Key signature change (if any)
    pub key_signature: Option<KeySignature>,
    /// Beats per track
    pub track_beats: Vec<TrackMeasure>,
    /// Repeat markers
    pub repeat: RepeatMarker,
    /// Alternate endings
    pub endings: Vec<u8>,
}

impl TabMeasure {
    pub fn new(number: usize) -> Self {
        Self {
            number,
            time_signature: None,
            tempo: None,
            key_signature: None,
            track_beats: Vec::new(),
            repeat: RepeatMarker::None,
            endings: Vec::new(),
        }
    }

    /// Get duration in ticks based on time signature
    pub fn duration_ticks(&self) -> u64 {
        let ts = self.time_signature.unwrap_or_default();
        ts.measure_ticks()
    }
}

/// Beats for a single track in a measure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackMeasure {
    pub track_id: Uuid,
    pub beats: Vec<TabBeat>,
}

/// A single beat (rhythmic unit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabBeat {
    /// Beat ID for editing
    pub id: Uuid,
    /// Notes in this beat (can be a chord)
    pub notes: Vec<TabNote>,
    /// Rhythmic value
    pub rhythm: RhythmValue,
    /// Is this a rest?
    pub is_rest: bool,
    /// Beat-level effects
    pub effects: BeatEffect,
    /// Text annotation
    pub text: Option<String>,
    /// Voice (for multi-voice notation)
    pub voice: u8,
}

impl Default for TabBeat {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            notes: Vec::new(),
            rhythm: RhythmValue::quarter(),
            is_rest: false,
            effects: BeatEffect::default(),
            text: None,
            voice: 0,
        }
    }
}

impl TabBeat {
    pub fn rest(rhythm: RhythmValue) -> Self {
        Self {
            is_rest: true,
            rhythm,
            ..Default::default()
        }
    }

    pub fn note(string: u8, fret: u8, rhythm: RhythmValue) -> Self {
        Self {
            notes: vec![TabNote::new(string, fret)],
            rhythm,
            ..Default::default()
        }
    }

    pub fn chord(notes: Vec<(u8, u8)>, rhythm: RhythmValue) -> Self {
        Self {
            notes: notes.into_iter().map(|(s, f)| TabNote::new(s, f)).collect(),
            rhythm,
            ..Default::default()
        }
    }
}

/// A single note on a string
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabNote {
    /// String number (1 = highest pitch string for guitars)
    pub string: u8,
    /// Fret number (0 = open, can go to 30+)
    pub fret: u8,
    /// Velocity (0-127)
    pub velocity: u8,
    /// Is this tied to previous note?
    pub tied: bool,
    /// Is this a ghost note?
    pub ghost: bool,
    /// Is this a dead/muted note?
    pub dead: bool,
    /// Note techniques/effects
    pub techniques: Vec<Technique>,
    /// Left hand fingering
    pub left_finger: Option<Fingering>,
    /// Right hand fingering
    pub right_finger: Option<Fingering>,
}

impl TabNote {
    pub fn new(string: u8, fret: u8) -> Self {
        Self {
            string,
            fret,
            velocity: 100,
            tied: false,
            ghost: false,
            dead: false,
            techniques: Vec::new(),
            left_finger: None,
            right_finger: None,
        }
    }

    pub fn with_velocity(mut self, velocity: u8) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.ghost = true;
        self.velocity = 60;
        self
    }

    pub fn dead(mut self) -> Self {
        self.dead = true;
        self
    }

    pub fn with_technique(mut self, tech: Technique) -> Self {
        self.techniques.push(tech);
        self
    }

    /// Check if note has a specific technique type
    pub fn has_technique(&self, check: impl Fn(&Technique) -> bool) -> bool {
        self.techniques.iter().any(check)
    }
}

/// Repeat markers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMarker {
    None,
    Start,
    End(u8), // Number of times to repeat
    StartEnd(u8),
}

/// Section marker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionMarker {
    /// Measure number
    pub measure: usize,
    /// Section name
    pub name: String,
    /// Color for UI
    pub color: Option<(u8, u8, u8)>,
}

impl SectionMarker {
    pub fn new(measure: usize, name: &str) -> Self {
        Self {
            measure,
            name: name.to_string(),
            color: None,
        }
    }

    pub fn intro(measure: usize) -> Self {
        Self::new(measure, "Intro")
    }

    pub fn verse(measure: usize) -> Self {
        Self::new(measure, "Verse")
    }

    pub fn chorus(measure: usize) -> Self {
        Self::new(measure, "Chorus")
    }

    pub fn breakdown(measure: usize) -> Self {
        Self::new(measure, "Breakdown")
    }

    pub fn solo(measure: usize) -> Self {
        Self::new(measure, "Solo")
    }
}

/// Key signature
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KeySignature {
    /// Root note (0 = C, 1 = C#, etc.)
    pub root: u8,
    /// Is minor?
    pub minor: bool,
}

impl Default for KeySignature {
    fn default() -> Self {
        Self { root: 0, minor: false }
    }
}

impl KeySignature {
    pub fn new(root: u8, minor: bool) -> Self {
        Self { root, minor }
    }

    pub fn c_major() -> Self {
        Self::new(0, false)
    }

    pub fn a_minor() -> Self {
        Self::new(9, true)
    }

    pub fn name(&self) -> &'static str {
        let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        notes.get(self.root as usize).unwrap_or(&"C")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_document_creation() {
        let mut doc = TabDocument::new();
        assert!(doc.tracks.is_empty());

        let guitar = TabTrack::guitar("Lead Guitar");
        doc.add_track(guitar);
        assert_eq!(doc.tracks.len(), 1);

        doc.add_measure();
        assert_eq!(doc.measures.len(), 1);
    }

    #[test]
    fn test_beat_creation() {
        let beat = TabBeat::note(1, 12, RhythmValue::sixteenth());
        assert_eq!(beat.notes.len(), 1);
        assert_eq!(beat.notes[0].fret, 12);
        assert!(!beat.is_rest);

        let rest = TabBeat::rest(RhythmValue::quarter());
        assert!(rest.is_rest);
    }

    #[test]
    fn test_chord_creation() {
        // Power chord: root + fifth
        let chord = TabBeat::chord(
            vec![(6, 3), (5, 5), (4, 5)], // G power chord
            RhythmValue::eighth(),
        );
        assert_eq!(chord.notes.len(), 3);
    }
}
