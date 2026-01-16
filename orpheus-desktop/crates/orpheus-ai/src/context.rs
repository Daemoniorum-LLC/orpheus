//! Composition Context
//!
//! Musical context types for AI-assisted composition.

use serde::{Deserialize, Serialize};

/// Musical key signature
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Key {
    // Major keys
    #[default]
    CMajor,
    GMajor,
    DMajor,
    AMajor,
    EMajor,
    BMajor,
    FSharpMajor,
    CSharpMajor,
    FMajor,
    BFlatMajor,
    EFlatMajor,
    AFlatMajor,
    DFlatMajor,
    GFlatMajor,

    // Minor keys
    AMinor,
    EMinor,
    BMinor,
    FSharpMinor,
    CSharpMinor,
    GSharpMinor,
    DSharpMinor,
    DMinor,
    GMinor,
    CMinor,
    FMinor,
    BFlatMinor,
    EFlatMinor,
}

impl Key {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Key::CMajor => "C Major",
            Key::GMajor => "G Major",
            Key::DMajor => "D Major",
            Key::AMajor => "A Major",
            Key::EMajor => "E Major",
            Key::BMajor => "B Major",
            Key::FSharpMajor => "F# Major",
            Key::CSharpMajor => "C# Major",
            Key::FMajor => "F Major",
            Key::BFlatMajor => "Bb Major",
            Key::EFlatMajor => "Eb Major",
            Key::AFlatMajor => "Ab Major",
            Key::DFlatMajor => "Db Major",
            Key::GFlatMajor => "Gb Major",
            Key::AMinor => "A Minor",
            Key::EMinor => "E Minor",
            Key::BMinor => "B Minor",
            Key::FSharpMinor => "F# Minor",
            Key::CSharpMinor => "C# Minor",
            Key::GSharpMinor => "G# Minor",
            Key::DSharpMinor => "D# Minor",
            Key::DMinor => "D Minor",
            Key::GMinor => "G Minor",
            Key::CMinor => "C Minor",
            Key::FMinor => "F Minor",
            Key::BFlatMinor => "Bb Minor",
            Key::EFlatMinor => "Eb Minor",
        }
    }

    /// Check if major key
    pub fn is_major(&self) -> bool {
        matches!(
            self,
            Key::CMajor
                | Key::GMajor
                | Key::DMajor
                | Key::AMajor
                | Key::EMajor
                | Key::BMajor
                | Key::FSharpMajor
                | Key::CSharpMajor
                | Key::FMajor
                | Key::BFlatMajor
                | Key::EFlatMajor
                | Key::AFlatMajor
                | Key::DFlatMajor
                | Key::GFlatMajor
        )
    }

    /// Check if minor key
    pub fn is_minor(&self) -> bool {
        !self.is_major()
    }

    /// Get relative key
    pub fn relative(&self) -> Key {
        match self {
            Key::CMajor => Key::AMinor,
            Key::GMajor => Key::EMinor,
            Key::DMajor => Key::BMinor,
            Key::AMajor => Key::FSharpMinor,
            Key::EMajor => Key::CSharpMinor,
            Key::BMajor => Key::GSharpMinor,
            Key::FSharpMajor => Key::DSharpMinor,
            Key::CSharpMajor => Key::GSharpMinor, // Enharmonic
            Key::FMajor => Key::DMinor,
            Key::BFlatMajor => Key::GMinor,
            Key::EFlatMajor => Key::CMinor,
            Key::AFlatMajor => Key::FMinor,
            Key::DFlatMajor => Key::BFlatMinor,
            Key::GFlatMajor => Key::EFlatMinor,
            Key::AMinor => Key::CMajor,
            Key::EMinor => Key::GMajor,
            Key::BMinor => Key::DMajor,
            Key::FSharpMinor => Key::AMajor,
            Key::CSharpMinor => Key::EMajor,
            Key::GSharpMinor => Key::BMajor,
            Key::DSharpMinor => Key::FSharpMajor,
            Key::DMinor => Key::FMajor,
            Key::GMinor => Key::BFlatMajor,
            Key::CMinor => Key::EFlatMajor,
            Key::FMinor => Key::AFlatMajor,
            Key::BFlatMinor => Key::DFlatMajor,
            Key::EFlatMinor => Key::GFlatMajor,
        }
    }
}


/// Chord type/quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChordType {
    /// Major triad
    Major,
    /// Minor triad
    Minor,
    /// Diminished triad
    Diminished,
    /// Augmented triad
    Augmented,
    /// Dominant 7th
    Dominant7,
    /// Major 7th
    Major7,
    /// Minor 7th
    Minor7,
    /// Half-diminished 7th
    HalfDiminished7,
    /// Diminished 7th
    Diminished7,
    /// Suspended 2nd
    Sus2,
    /// Suspended 4th
    Sus4,
    /// Add9
    Add9,
    /// Power chord (5th)
    Power,
}

impl ChordType {
    /// Get display suffix
    pub fn suffix(&self) -> &'static str {
        match self {
            ChordType::Major => "",
            ChordType::Minor => "m",
            ChordType::Diminished => "dim",
            ChordType::Augmented => "aug",
            ChordType::Dominant7 => "7",
            ChordType::Major7 => "maj7",
            ChordType::Minor7 => "m7",
            ChordType::HalfDiminished7 => "ø7",
            ChordType::Diminished7 => "dim7",
            ChordType::Sus2 => "sus2",
            ChordType::Sus4 => "sus4",
            ChordType::Add9 => "add9",
            ChordType::Power => "5",
        }
    }
}

/// A chord in the composition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chord {
    /// Root note name (e.g., "C", "F#", "Bb")
    pub root: String,
    /// Chord type/quality
    pub chord_type: ChordType,
    /// Bass note if different from root (e.g., "G" for C/G)
    pub bass: Option<String>,
}

impl Chord {
    /// Create a new chord
    pub fn new(root: impl Into<String>, chord_type: ChordType) -> Self {
        Self {
            root: root.into(),
            chord_type,
            bass: None,
        }
    }

    /// Create a chord with a bass note
    pub fn with_bass(mut self, bass: impl Into<String>) -> Self {
        self.bass = Some(bass.into());
        self
    }

    /// Get display name
    pub fn display_name(&self) -> String {
        let base = format!("{}{}", self.root, self.chord_type.suffix());
        if let Some(ref bass) = self.bass {
            format!("{}/{}", base, bass)
        } else {
            base
        }
    }

    /// Common chord constructors
    pub fn c_major() -> Self {
        Self::new("C", ChordType::Major)
    }

    pub fn g_major() -> Self {
        Self::new("G", ChordType::Major)
    }

    pub fn d_major() -> Self {
        Self::new("D", ChordType::Major)
    }

    pub fn a_minor() -> Self {
        Self::new("A", ChordType::Minor)
    }

    pub fn e_minor() -> Self {
        Self::new("E", ChordType::Minor)
    }
}

/// Time signature
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSignature {
    /// Numerator (beats per measure)
    pub numerator: u8,
    /// Denominator (note value that gets one beat)
    pub denominator: u8,
}

impl TimeSignature {
    /// Create a new time signature
    pub fn new(numerator: u8, denominator: u8) -> Self {
        Self { numerator, denominator }
    }

    /// Common time signatures
    pub fn common() -> Self {
        Self::new(4, 4)
    }

    pub fn waltz() -> Self {
        Self::new(3, 4)
    }

    pub fn cut_time() -> Self {
        Self::new(2, 2)
    }

    pub fn six_eight() -> Self {
        Self::new(6, 8)
    }

    /// Get display string
    pub fn display(&self) -> String {
        format!("{}/{}", self.numerator, self.denominator)
    }
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self::common()
    }
}

/// Genre/style hint for the AI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Genre {
    Rock,
    Pop,
    Jazz,
    Blues,
    Metal,
    Folk,
    Classical,
    Electronic,
    Country,
    RnB,
    HipHop,
    Funk,
    Reggae,
    Indie,
    Progressive,
}

impl Genre {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Genre::Rock => "Rock",
            Genre::Pop => "Pop",
            Genre::Jazz => "Jazz",
            Genre::Blues => "Blues",
            Genre::Metal => "Metal",
            Genre::Folk => "Folk",
            Genre::Classical => "Classical",
            Genre::Electronic => "Electronic",
            Genre::Country => "Country",
            Genre::RnB => "R&B",
            Genre::HipHop => "Hip Hop",
            Genre::Funk => "Funk",
            Genre::Reggae => "Reggae",
            Genre::Indie => "Indie",
            Genre::Progressive => "Progressive",
        }
    }
}

/// Composition context for AI suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionContext {
    /// Current key signature
    pub key: Key,
    /// Current tempo in BPM
    pub tempo: f64,
    /// Time signature
    pub time_signature: TimeSignature,
    /// Current chord (if any)
    pub current_chord: Option<Chord>,
    /// Previous chords in progression
    pub chord_history: Vec<Chord>,
    /// Genre hint
    pub genre: Option<Genre>,
    /// Current position in bars
    pub position_bars: f64,
    /// Current section (verse, chorus, etc.)
    pub section: Option<String>,
    /// Additional notes/context
    pub notes: Option<String>,
}

impl Default for CompositionContext {
    fn default() -> Self {
        Self {
            key: Key::CMajor,
            tempo: 120.0,
            time_signature: TimeSignature::common(),
            current_chord: None,
            chord_history: Vec::new(),
            genre: None,
            position_bars: 0.0,
            section: None,
            notes: None,
        }
    }
}

impl CompositionContext {
    /// Create a new composition context
    pub fn new(key: Key, tempo: f64) -> Self {
        Self {
            key,
            tempo,
            ..Default::default()
        }
    }

    /// Set time signature
    pub fn with_time_signature(mut self, time_sig: TimeSignature) -> Self {
        self.time_signature = time_sig;
        self
    }

    /// Set genre
    pub fn with_genre(mut self, genre: Genre) -> Self {
        self.genre = Some(genre);
        self
    }

    /// Set current chord
    pub fn with_current_chord(mut self, chord: Chord) -> Self {
        self.current_chord = Some(chord);
        self
    }

    /// Add chord to history
    pub fn add_chord(&mut self, chord: Chord) {
        // Move current chord to history if exists
        if let Some(current) = self.current_chord.take() {
            self.chord_history.push(current);
        }
        self.current_chord = Some(chord);
    }

    /// Get last N chords from history
    pub fn recent_chords(&self, n: usize) -> Vec<&Chord> {
        let start = self.chord_history.len().saturating_sub(n);
        self.chord_history[start..].iter().collect()
    }

    /// Convert to JSON for AI prompt
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "key": self.key.display_name(),
            "tempo": self.tempo,
            "time_signature": self.time_signature.display(),
            "current_chord": self.current_chord.as_ref().map(|c| c.display_name()),
            "recent_chords": self.recent_chords(4).iter().map(|c| c.display_name()).collect::<Vec<_>>(),
            "genre": self.genre.map(|g| g.display_name()),
            "position_bars": self.position_bars,
            "section": self.section,
        })
    }
}

/// Suggestion type from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    /// Next chord in progression
    NextChord(Chord),
    /// Melody notes
    MelodyNotes(Vec<u8>),
    /// Rhythm pattern
    RhythmPattern(String),
    /// Key modulation
    KeyModulation(Key),
    /// Tempo change
    TempoChange(f64),
    /// General text advice
    Advice(String),
}

/// An AI suggestion for composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionSuggestion {
    /// Suggestion type
    pub suggestion_type: SuggestionType,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
    /// Explanation text
    pub explanation: String,
    /// Alternative suggestions
    pub alternatives: Vec<SuggestionType>,
}

impl CompositionSuggestion {
    /// Create a chord suggestion
    pub fn chord(chord: Chord, confidence: f64, explanation: impl Into<String>) -> Self {
        Self {
            suggestion_type: SuggestionType::NextChord(chord),
            confidence,
            explanation: explanation.into(),
            alternatives: Vec::new(),
        }
    }

    /// Create an advice suggestion
    pub fn advice(text: impl Into<String>) -> Self {
        Self {
            suggestion_type: SuggestionType::Advice(text.into()),
            confidence: 1.0,
            explanation: String::new(),
            alternatives: Vec::new(),
        }
    }

    /// Add alternative
    pub fn with_alternative(mut self, alt: SuggestionType) -> Self {
        self.alternatives.push(alt);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_properties() {
        assert!(Key::CMajor.is_major());
        assert!(!Key::CMajor.is_minor());
        assert!(Key::AMinor.is_minor());
        assert!(!Key::AMinor.is_major());
    }

    #[test]
    fn test_key_relative() {
        assert_eq!(Key::CMajor.relative(), Key::AMinor);
        assert_eq!(Key::AMinor.relative(), Key::CMajor);
        assert_eq!(Key::GMajor.relative(), Key::EMinor);
        assert_eq!(Key::EMinor.relative(), Key::GMajor);
    }

    #[test]
    fn test_chord_display() {
        let c = Chord::new("C", ChordType::Major);
        assert_eq!(c.display_name(), "C");

        let am = Chord::new("A", ChordType::Minor);
        assert_eq!(am.display_name(), "Am");

        let g7 = Chord::new("G", ChordType::Dominant7);
        assert_eq!(g7.display_name(), "G7");

        let cmaj7 = Chord::new("C", ChordType::Major7);
        assert_eq!(cmaj7.display_name(), "Cmaj7");

        let c_over_g = Chord::new("C", ChordType::Major).with_bass("G");
        assert_eq!(c_over_g.display_name(), "C/G");
    }

    #[test]
    fn test_time_signature() {
        let common = TimeSignature::common();
        assert_eq!(common.numerator, 4);
        assert_eq!(common.denominator, 4);
        assert_eq!(common.display(), "4/4");

        let waltz = TimeSignature::waltz();
        assert_eq!(waltz.display(), "3/4");
    }

    #[test]
    fn test_composition_context() {
        let mut ctx = CompositionContext::new(Key::GMajor, 100.0)
            .with_genre(Genre::Rock)
            .with_time_signature(TimeSignature::common())
            .with_current_chord(Chord::g_major());

        assert_eq!(ctx.key, Key::GMajor);
        assert!((ctx.tempo - 100.0).abs() < 0.01);
        assert_eq!(ctx.genre, Some(Genre::Rock));
        assert!(ctx.current_chord.is_some());

        // Add some chords
        ctx.add_chord(Chord::c_major());
        ctx.add_chord(Chord::d_major());

        // G should now be in history
        assert_eq!(ctx.chord_history.len(), 2);
        assert_eq!(ctx.current_chord.as_ref().unwrap().root, "D");

        // Recent chords
        let recent = ctx.recent_chords(2);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_composition_context_to_json() {
        let ctx = CompositionContext::new(Key::AMajor, 140.0)
            .with_genre(Genre::Blues)
            .with_current_chord(Chord::new("A", ChordType::Dominant7));

        let json = ctx.to_json();
        assert_eq!(json["key"], "A Major");
        assert_eq!(json["tempo"], 140.0);
        assert_eq!(json["genre"], "Blues");
        assert_eq!(json["current_chord"], "A7");
    }

    #[test]
    fn test_composition_suggestion() {
        let suggestion = CompositionSuggestion::chord(
            Chord::new("D", ChordType::Major),
            0.85,
            "D major provides a strong resolution from G7",
        )
        .with_alternative(SuggestionType::NextChord(Chord::new("E", ChordType::Minor)));

        assert!((suggestion.confidence - 0.85).abs() < 0.01);
        assert_eq!(suggestion.alternatives.len(), 1);
    }

    #[test]
    fn test_genre_display() {
        assert_eq!(Genre::Jazz.display_name(), "Jazz");
        assert_eq!(Genre::RnB.display_name(), "R&B");
        assert_eq!(Genre::HipHop.display_name(), "Hip Hop");
    }
}
