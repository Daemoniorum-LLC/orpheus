//! Notation symbols and markings
//!
//! Additional notation elements for complete score representation.

use serde::{Deserialize, Serialize};

/// Text expression markings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextExpression {
    /// Tempo text (e.g., "Allegro", "Fast")
    Tempo(String),
    /// Dynamic text (e.g., "pp suddenly")
    Dynamic(String),
    /// Technique instruction (e.g., "with distortion")
    Technique(String),
    /// Performance instruction (e.g., "freely")
    Performance(String),
    /// Lyrics
    Lyric(String),
    /// Chord symbol (e.g., "Am7")
    ChordSymbol(String),
    /// Generic text
    Generic(String),
}

/// Rehearsal marks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RehearsalMark {
    /// Letter mark (A, B, C, ...)
    Letter(char),
    /// Number mark (1, 2, 3, ...)
    Number(u16),
    /// Custom text
    Custom(String),
}

impl RehearsalMark {
    pub fn letter(c: char) -> Self {
        Self::Letter(c.to_ascii_uppercase())
    }

    pub fn number(n: u16) -> Self {
        Self::Number(n)
    }

    pub fn display(&self) -> String {
        match self {
            Self::Letter(c) => format!("[{}]", c),
            Self::Number(n) => format!("[{}]", n),
            Self::Custom(s) => format!("[{}]", s),
        }
    }
}

/// Barline types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarlineType {
    /// Normal single barline
    Normal,
    /// Double barline (end of section)
    Double,
    /// Final barline (end of piece)
    Final,
    /// Repeat start
    RepeatStart,
    /// Repeat end
    RepeatEnd,
    /// Repeat both (end and start)
    RepeatBoth,
    /// Dashed (hidden in tab)
    Dashed,
    /// Dotted
    Dotted,
    /// None (no barline)
    None,
}

/// Ottava (octave transposition)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ottava {
    /// 8va (one octave up)
    OttavaUp,
    /// 8vb (one octave down)
    OttavaDown,
    /// 15ma (two octaves up)
    QuindicessimaUp,
    /// 15mb (two octaves down)
    QuindicessimaDown,
}

impl Ottava {
    pub fn transposition_semitones(&self) -> i8 {
        match self {
            Self::OttavaUp => 12,
            Self::OttavaDown => -12,
            Self::QuindicessimaUp => 24,
            Self::QuindicessimaDown => -24,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::OttavaUp => "8va",
            Self::OttavaDown => "8vb",
            Self::QuindicessimaUp => "15ma",
            Self::QuindicessimaDown => "15mb",
        }
    }
}

/// Spanners (lines that extend over multiple notes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Spanner {
    /// Let ring spanner
    LetRing {
        start_beat: usize,
        end_beat: usize,
    },
    /// Palm mute spanner
    PalmMute {
        start_beat: usize,
        end_beat: usize,
    },
    /// Crescendo
    Crescendo {
        start_beat: usize,
        end_beat: usize,
    },
    /// Decrescendo
    Decrescendo {
        start_beat: usize,
        end_beat: usize,
    },
    /// Trill line
    Trill {
        start_beat: usize,
        end_beat: usize,
        upper_note: u8,
    },
    /// Slide line
    Slide {
        start_beat: usize,
        end_beat: usize,
    },
    /// Hammer-on/Pull-off line
    Legato {
        start_beat: usize,
        end_beat: usize,
    },
    /// Ottava line
    Ottava {
        ottava_type: Ottava,
        start_beat: usize,
        end_beat: usize,
    },
    /// Pedal line
    Pedal {
        start_beat: usize,
        end_beat: usize,
    },
    /// 8va line for whammy
    WhammyLine {
        start_beat: usize,
        end_beat: usize,
        points: Vec<(f32, i16)>, // (position, cents)
    },
}

/// Clef types (mainly for standard notation view)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Clef {
    /// Treble clef (G clef)
    Treble,
    /// Bass clef (F clef)
    Bass,
    /// Tenor clef (C clef on 4th line)
    Tenor,
    /// Alto clef (C clef on 3rd line)
    Alto,
    /// Percussion clef
    Percussion,
    /// Tab "clef"
    Tab,
    /// 8va treble (octave up)
    Treble8va,
    /// 8vb treble (octave down)
    Treble8vb,
}

/// Accidental types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Accidental {
    /// Double flat
    DoubleFlat,
    /// Flat
    Flat,
    /// Natural
    Natural,
    /// Sharp
    Sharp,
    /// Double sharp
    DoubleSharp,
    /// Quarter tone flat
    QuarterFlat,
    /// Quarter tone sharp
    QuarterSharp,
}

impl Accidental {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::DoubleFlat => "𝄫",
            Self::Flat => "♭",
            Self::Natural => "♮",
            Self::Sharp => "♯",
            Self::DoubleSharp => "𝄪",
            Self::QuarterFlat => "𝄳",
            Self::QuarterSharp => "𝄲",
        }
    }

    pub fn semitone_offset(&self) -> i8 {
        match self {
            Self::DoubleFlat => -2,
            Self::Flat => -1,
            Self::Natural => 0,
            Self::Sharp => 1,
            Self::DoubleSharp => 2,
            Self::QuarterFlat => 0, // Microtonal
            Self::QuarterSharp => 0,
        }
    }
}

/// Articulation symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArticulationSymbol {
    /// Staccato (dot)
    Staccato,
    /// Staccatissimo (wedge)
    Staccatissimo,
    /// Tenuto (dash)
    Tenuto,
    /// Accent (>)
    Accent,
    /// Marcato (^)
    Marcato,
    /// Sforzando
    Sforzando,
    /// Fermata (pause)
    Fermata,
    /// Short fermata
    FermataShort,
    /// Long fermata
    FermataLong,
    /// Breath mark
    BreathMark,
    /// Caesura (railroad tracks)
    Caesura,
}

impl ArticulationSymbol {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Staccato => ".",
            Self::Staccatissimo => "▾",
            Self::Tenuto => "–",
            Self::Accent => ">",
            Self::Marcato => "^",
            Self::Sforzando => "sfz",
            Self::Fermata => "𝄐",
            Self::FermataShort => "𝄑",
            Self::FermataLong => "𝄒",
            Self::BreathMark => ",",
            Self::Caesura => "//",
        }
    }
}

/// Ornament symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ornament {
    /// Trill
    Trill,
    /// Mordent
    Mordent,
    /// Inverted mordent
    InvertedMordent,
    /// Turn
    Turn,
    /// Inverted turn
    InvertedTurn,
    /// Delayed turn
    DelayedTurn,
    /// Appoggiatura
    Appoggiatura,
    /// Acciaccatura
    Acciaccatura,
}

impl Ornament {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Trill => "tr",
            Self::Mordent => "𝆖",
            Self::InvertedMordent => "𝆗",
            Self::Turn => "𝄾",
            Self::InvertedTurn => "𝄿",
            Self::DelayedTurn => "~",
            Self::Appoggiatura => "⌐",
            Self::Acciaccatura => "𝅘𝅥",
        }
    }
}

/// Tremolo notation (strokes on stem)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TremoloNotation {
    /// Single note tremolo (repeated rapidly)
    Single(TremoloStrokes),
    /// Two-note tremolo (alternating between notes)
    TwoNote(TremoloStrokes),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TremoloStrokes {
    /// Eighth note speed (1 stroke)
    One,
    /// Sixteenth note speed (2 strokes)
    Two,
    /// 32nd note speed (3 strokes)
    Three,
    /// As fast as possible
    Buzz,
}

/// Coda/Segno navigation symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationSymbol {
    /// Segno (the sign)
    Segno,
    /// Coda
    Coda,
    /// D.C. (Da Capo - from the beginning)
    DaCapo,
    /// D.S. (Dal Segno - from the sign)
    DalSegno,
    /// D.C. al Fine (to end)
    DaCapoAlFine,
    /// D.C. al Coda
    DaCapoAlCoda,
    /// D.S. al Fine
    DalSegnoAlFine,
    /// D.S. al Coda
    DalSegnoAlCoda,
    /// To Coda (jump to coda)
    ToCoda,
    /// Fine (end)
    Fine,
}

impl NavigationSymbol {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Segno => "𝄋",
            Self::Coda => "𝄌",
            Self::DaCapo => "D.C.",
            Self::DalSegno => "D.S.",
            Self::DaCapoAlFine => "D.C. al Fine",
            Self::DaCapoAlCoda => "D.C. al Coda",
            Self::DalSegnoAlFine => "D.S. al Fine",
            Self::DalSegnoAlCoda => "D.S. al Coda",
            Self::ToCoda => "To Coda 𝄌",
            Self::Fine => "Fine",
        }
    }
}

/// Layout hints for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutHints {
    /// Force system break
    pub system_break: bool,
    /// Force page break
    pub page_break: bool,
    /// Staff spacing adjustment
    pub staff_spacing: Option<f32>,
    /// Hide staff
    pub hide_staff: bool,
    /// Custom measure width
    pub measure_width: Option<f32>,
}

impl Default for LayoutHints {
    fn default() -> Self {
        Self {
            system_break: false,
            page_break: false,
            staff_spacing: None,
            hide_staff: false,
            measure_width: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rehearsal_marks() {
        assert_eq!(RehearsalMark::letter('a').display(), "[A]");
        assert_eq!(RehearsalMark::number(1).display(), "[1]");
    }

    #[test]
    fn test_ottava() {
        assert_eq!(Ottava::OttavaUp.transposition_semitones(), 12);
        assert_eq!(Ottava::OttavaDown.transposition_semitones(), -12);
    }

    #[test]
    fn test_accidentals() {
        assert_eq!(Accidental::Sharp.semitone_offset(), 1);
        assert_eq!(Accidental::Flat.semitone_offset(), -1);
        assert_eq!(Accidental::DoubleSharp.semitone_offset(), 2);
    }
}
