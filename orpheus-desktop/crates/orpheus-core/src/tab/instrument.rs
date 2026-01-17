//! Instrument definitions for extended range and virtuoso playing
//!
//! Supports 4-12 string instruments, multi-scale necks, custom tunings.

use serde::{Deserialize, Serialize};

use super::drums::DrumKit;

/// Instrument configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instrument {
    /// Stringed instrument (guitar, bass, etc.)
    StringedInstrument(StringedConfig),
    /// Drum kit
    Drums(DrumKit),
    /// Keyboard/piano
    Keys(KeysConfig),
}

impl Instrument {
    /// Standard 6-string guitar in E standard
    pub fn guitar_standard() -> Self {
        Self::StringedInstrument(StringedConfig::guitar_6_standard())
    }

    /// 7-string guitar in B standard
    pub fn guitar_7_string() -> Self {
        Self::StringedInstrument(StringedConfig::guitar_7_standard())
    }

    /// 8-string guitar in F# standard
    pub fn guitar_8_string() -> Self {
        Self::StringedInstrument(StringedConfig::guitar_8_standard())
    }

    /// 9-string guitar
    pub fn guitar_9_string() -> Self {
        Self::StringedInstrument(StringedConfig::guitar_9_standard())
    }

    /// Standard 4-string bass
    pub fn bass_standard() -> Self {
        Self::StringedInstrument(StringedConfig::bass_4_standard())
    }

    /// 5-string bass
    pub fn bass_5_string() -> Self {
        Self::StringedInstrument(StringedConfig::bass_5_standard())
    }

    /// 6-string bass
    pub fn bass_6_string() -> Self {
        Self::StringedInstrument(StringedConfig::bass_6_standard())
    }
}

/// Configuration for stringed instruments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringedConfig {
    /// Instrument type
    pub instrument_type: StringedType,
    /// Number of strings (4-12)
    pub string_count: u8,
    /// Tuning - MIDI note for each string (lowest to highest pitch when played open)
    /// Index 0 = lowest string, last = highest
    pub tuning: Vec<u8>,
    /// Number of frets (typically 22-30)
    pub fret_count: u8,
    /// Multi-scale configuration (if any)
    pub multiscale: Option<MultiscaleConfig>,
    /// Capo position (0 = no capo)
    pub capo: u8,
    /// Scale length in inches (for reference)
    pub scale_length: f32,
    /// Has tremolo/whammy bar
    pub has_tremolo: bool,
    /// Tremolo type
    pub tremolo_type: Option<TremoloType>,
}

impl StringedConfig {
    // ===== GUITAR CONFIGURATIONS =====

    /// 6-string guitar in E standard (E2 A2 D3 G3 B3 E4)
    pub fn guitar_6_standard() -> Self {
        Self {
            instrument_type: StringedType::ElectricGuitar,
            string_count: 6,
            tuning: vec![40, 45, 50, 55, 59, 64], // E A D G B E
            fret_count: 24,
            multiscale: None,
            capo: 0,
            scale_length: 25.5,
            has_tremolo: true,
            tremolo_type: Some(TremoloType::FloydRose),
        }
    }

    /// 6-string in Drop D (D2 A2 D3 G3 B3 E4)
    pub fn guitar_6_drop_d() -> Self {
        let mut config = Self::guitar_6_standard();
        config.tuning[0] = 38; // D2
        config
    }

    /// 6-string in D standard (D2 G2 C3 F3 A3 D4)
    pub fn guitar_6_d_standard() -> Self {
        Self {
            instrument_type: StringedType::ElectricGuitar,
            string_count: 6,
            tuning: vec![38, 43, 48, 53, 57, 62],
            fret_count: 24,
            multiscale: None,
            capo: 0,
            scale_length: 25.5,
            has_tremolo: true,
            tremolo_type: Some(TremoloType::FloydRose),
        }
    }

    /// 7-string guitar in B standard (B1 E2 A2 D3 G3 B3 E4)
    pub fn guitar_7_standard() -> Self {
        Self {
            instrument_type: StringedType::ElectricGuitar,
            string_count: 7,
            tuning: vec![35, 40, 45, 50, 55, 59, 64],
            fret_count: 24,
            multiscale: None,
            capo: 0,
            scale_length: 26.5,
            has_tremolo: true,
            tremolo_type: Some(TremoloType::FloydRose),
        }
    }

    /// 7-string in Drop A (A1 E2 A2 D3 G3 B3 E4)
    pub fn guitar_7_drop_a() -> Self {
        let mut config = Self::guitar_7_standard();
        config.tuning[0] = 33; // A1
        config
    }

    /// 8-string guitar in F# standard (F#1 B1 E2 A2 D3 G3 B3 E4)
    pub fn guitar_8_standard() -> Self {
        Self {
            instrument_type: StringedType::ElectricGuitar,
            string_count: 8,
            tuning: vec![30, 35, 40, 45, 50, 55, 59, 64],
            fret_count: 24,
            multiscale: Some(MultiscaleConfig {
                bass_scale: 28.0,
                treble_scale: 25.5,
                perpendicular_fret: 12,
            }),
            capo: 0,
            scale_length: 27.0,
            has_tremolo: true,
            tremolo_type: Some(TremoloType::Hipshot),
        }
    }

    /// 8-string in Drop E (E1 B1 E2 A2 D3 G3 B3 E4)
    pub fn guitar_8_drop_e() -> Self {
        let mut config = Self::guitar_8_standard();
        config.tuning[0] = 28; // E1
        config
    }

    /// 9-string guitar (C#1 F#1 B1 E2 A2 D3 G3 B3 E4)
    pub fn guitar_9_standard() -> Self {
        Self {
            instrument_type: StringedType::ElectricGuitar,
            string_count: 9,
            tuning: vec![25, 30, 35, 40, 45, 50, 55, 59, 64],
            fret_count: 24,
            multiscale: Some(MultiscaleConfig {
                bass_scale: 30.0,
                treble_scale: 25.5,
                perpendicular_fret: 12,
            }),
            capo: 0,
            scale_length: 28.0,
            has_tremolo: false,
            tremolo_type: None,
        }
    }

    // ===== BASS CONFIGURATIONS =====

    /// 4-string bass in E standard (E1 A1 D2 G2)
    pub fn bass_4_standard() -> Self {
        Self {
            instrument_type: StringedType::Bass,
            string_count: 4,
            tuning: vec![28, 33, 38, 43],
            fret_count: 24,
            multiscale: None,
            capo: 0,
            scale_length: 34.0,
            has_tremolo: false,
            tremolo_type: None,
        }
    }

    /// 4-string bass in Drop D (D1 A1 D2 G2)
    pub fn bass_4_drop_d() -> Self {
        let mut config = Self::bass_4_standard();
        config.tuning[0] = 26;
        config
    }

    /// 5-string bass (B0 E1 A1 D2 G2)
    pub fn bass_5_standard() -> Self {
        Self {
            instrument_type: StringedType::Bass,
            string_count: 5,
            tuning: vec![23, 28, 33, 38, 43],
            fret_count: 24,
            multiscale: None,
            capo: 0,
            scale_length: 35.0,
            has_tremolo: false,
            tremolo_type: None,
        }
    }

    /// 6-string bass (B0 E1 A1 D2 G2 C3)
    pub fn bass_6_standard() -> Self {
        Self {
            instrument_type: StringedType::Bass,
            string_count: 6,
            tuning: vec![23, 28, 33, 38, 43, 48],
            fret_count: 24,
            multiscale: None,
            capo: 0,
            scale_length: 35.0,
            has_tremolo: false,
            tremolo_type: None,
        }
    }

    // ===== UTILITY METHODS =====

    /// Create custom tuning
    pub fn with_tuning(mut self, tuning: Vec<u8>) -> Self {
        self.string_count = tuning.len() as u8;
        self.tuning = tuning;
        self
    }

    /// Set capo position
    pub fn with_capo(mut self, fret: u8) -> Self {
        self.capo = fret;
        self
    }

    /// Configure as multi-scale
    pub fn with_multiscale(mut self, bass: f32, treble: f32, perp_fret: u8) -> Self {
        self.multiscale = Some(MultiscaleConfig {
            bass_scale: bass,
            treble_scale: treble,
            perpendicular_fret: perp_fret,
        });
        self
    }

    /// Get MIDI note for string and fret
    pub fn midi_note(&self, string: u8, fret: u8) -> u8 {
        let string_idx = (self.string_count - string) as usize;
        if string_idx >= self.tuning.len() {
            return 60; // Middle C fallback
        }
        self.tuning[string_idx].saturating_add(fret).saturating_add(self.capo)
    }

    /// Get note name for string and fret
    pub fn note_name(&self, string: u8, fret: u8) -> String {
        let midi = self.midi_note(string, fret);
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let note = note_names[(midi % 12) as usize];
        let octave = (midi / 12) as i32 - 1;
        format!("{}{}", note, octave)
    }

    /// Check if fret is valid
    pub fn is_valid_fret(&self, fret: u8) -> bool {
        fret <= self.fret_count
    }

    /// Check if string is valid
    pub fn is_valid_string(&self, string: u8) -> bool {
        string >= 1 && string <= self.string_count
    }
}

/// Type of stringed instrument
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StringedType {
    ElectricGuitar,
    AcousticGuitar,
    ClassicalGuitar,
    Bass,
    Banjo,
    Mandolin,
    Ukulele,
    Chapman,    // Chapman Stick
    Other,
}

/// Multi-scale (fanned fret) configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MultiscaleConfig {
    /// Scale length for bass side (inches)
    pub bass_scale: f32,
    /// Scale length for treble side (inches)
    pub treble_scale: f32,
    /// Fret where lines are perpendicular (typically 7-12)
    pub perpendicular_fret: u8,
}

impl MultiscaleConfig {
    /// Common Strandberg/Dingwall config
    pub fn standard() -> Self {
        Self {
            bass_scale: 28.0,
            treble_scale: 25.5,
            perpendicular_fret: 12,
        }
    }

    /// Calculate fret angle at a given fret position
    pub fn fret_angle(&self, fret: u8) -> f32 {
        if fret == self.perpendicular_fret {
            return 0.0;
        }
        // Simplified calculation - actual is more complex
        let diff = self.bass_scale - self.treble_scale;
        let offset = fret as i32 - self.perpendicular_fret as i32;
        (offset as f32 * diff * 0.5).atan().to_degrees()
    }
}

/// Tremolo/vibrato system type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TremoloType {
    /// Fender-style vintage tremolo
    Vintage,
    /// Floyd Rose double-locking
    FloydRose,
    /// Ibanez Edge/Zero
    Edge,
    /// Kahler
    Kahler,
    /// Bigsby
    Bigsby,
    /// Hipshot
    Hipshot,
    /// Evertune (technically not tremolo but pitch stable)
    Evertune,
    /// Steinberger TransTrem
    TransTrem,
    /// Other/custom
    Other,
}

impl TremoloType {
    /// Can this tremolo do dive bombs?
    pub fn can_dive(&self) -> bool {
        matches!(self, Self::FloydRose | Self::Edge | Self::Kahler | Self::Hipshot)
    }

    /// Can this tremolo do pull-ups (raise pitch)?
    pub fn can_pullup(&self) -> bool {
        matches!(self, Self::FloydRose | Self::Edge | Self::Kahler | Self::Hipshot)
    }

    /// Is this a locking tremolo?
    pub fn is_locking(&self) -> bool {
        matches!(self, Self::FloydRose | Self::Edge | Self::Kahler)
    }
}

/// Keyboard/piano configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeysConfig {
    /// Number of keys
    pub key_count: u8,
    /// Lowest MIDI note
    pub lowest_note: u8,
    /// Is weighted/hammer action
    pub weighted: bool,
}

impl Default for KeysConfig {
    fn default() -> Self {
        Self {
            key_count: 88,
            lowest_note: 21, // A0
            weighted: true,
        }
    }
}

/// Fingering notation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fingering {
    /// Open (no finger)
    Open,
    /// Thumb (T or p)
    Thumb,
    /// Index finger (1 or i)
    Index,
    /// Middle finger (2 or m)
    Middle,
    /// Ring finger (3 or a)
    Ring,
    /// Pinky (4 or c)
    Pinky,
}

impl Fingering {
    pub fn left_hand_symbol(&self) -> &'static str {
        match self {
            Self::Open => "0",
            Self::Thumb => "T",
            Self::Index => "1",
            Self::Middle => "2",
            Self::Ring => "3",
            Self::Pinky => "4",
        }
    }

    pub fn right_hand_symbol(&self) -> &'static str {
        match self {
            Self::Open => "-",
            Self::Thumb => "p",
            Self::Index => "i",
            Self::Middle => "m",
            Self::Ring => "a",
            Self::Pinky => "c",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_tunings() {
        let guitar = StringedConfig::guitar_6_standard();
        assert_eq!(guitar.string_count, 6);
        assert_eq!(guitar.tuning, vec![40, 45, 50, 55, 59, 64]);

        let bass = StringedConfig::bass_4_standard();
        assert_eq!(bass.string_count, 4);
        assert_eq!(bass.tuning, vec![28, 33, 38, 43]);
    }

    #[test]
    fn test_midi_note_calculation() {
        let guitar = StringedConfig::guitar_6_standard();

        // Open 6th string = E2 = MIDI 40
        assert_eq!(guitar.midi_note(6, 0), 40);

        // 12th fret 6th string = E3 = MIDI 52
        assert_eq!(guitar.midi_note(6, 12), 52);

        // Open 1st string = E4 = MIDI 64
        assert_eq!(guitar.midi_note(1, 0), 64);
    }

    #[test]
    fn test_note_names() {
        let guitar = StringedConfig::guitar_6_standard();

        assert_eq!(guitar.note_name(6, 0), "E2");
        assert_eq!(guitar.note_name(6, 5), "A2");
        assert_eq!(guitar.note_name(1, 0), "E4");
    }

    #[test]
    fn test_extended_range() {
        let eight_string = StringedConfig::guitar_8_standard();
        assert_eq!(eight_string.string_count, 8);
        assert!(eight_string.multiscale.is_some());

        // Lowest string open = F#1 = MIDI 30
        assert_eq!(eight_string.midi_note(8, 0), 30);
    }

    #[test]
    fn test_custom_tuning() {
        let custom = StringedConfig::guitar_6_standard()
            .with_tuning(vec![36, 43, 48, 53, 57, 62]); // Drop C

        assert_eq!(custom.midi_note(6, 0), 36); // C2
    }
}
