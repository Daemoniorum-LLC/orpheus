//! Advanced rhythm notation
//!
//! Tuplets, polyrhythms, odd time signatures, metric modulation.

use serde::{Deserialize, Serialize};

/// Complete rhythmic value with all modifiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmValue {
    /// Base duration
    pub base: BaseDuration,
    /// Number of dots (0, 1, 2, 3)
    pub dots: u8,
    /// Tuplet modifier (if any)
    pub tuplet: Option<Tuplet>,
    /// Is tied to next note
    pub tied: bool,
}

impl RhythmValue {
    pub fn new(base: BaseDuration) -> Self {
        Self {
            base,
            dots: 0,
            tuplet: None,
            tied: false,
        }
    }

    // Common durations
    pub fn whole() -> Self {
        Self::new(BaseDuration::Whole)
    }

    pub fn half() -> Self {
        Self::new(BaseDuration::Half)
    }

    pub fn quarter() -> Self {
        Self::new(BaseDuration::Quarter)
    }

    pub fn eighth() -> Self {
        Self::new(BaseDuration::Eighth)
    }

    pub fn sixteenth() -> Self {
        Self::new(BaseDuration::Sixteenth)
    }

    pub fn thirty_second() -> Self {
        Self::new(BaseDuration::ThirtySecond)
    }

    pub fn sixty_fourth() -> Self {
        Self::new(BaseDuration::SixtyFourth)
    }

    // Dotted variants
    pub fn dotted(mut self) -> Self {
        self.dots = 1;
        self
    }

    pub fn double_dotted(mut self) -> Self {
        self.dots = 2;
        self
    }

    pub fn triple_dotted(mut self) -> Self {
        self.dots = 3;
        self
    }

    // Tuplet variants
    pub fn triplet(mut self) -> Self {
        self.tuplet = Some(Tuplet::triplet());
        self
    }

    pub fn quintuplet(mut self) -> Self {
        self.tuplet = Some(Tuplet::quintuplet());
        self
    }

    pub fn septuplet(mut self) -> Self {
        self.tuplet = Some(Tuplet::septuplet());
        self
    }

    pub fn with_tuplet(mut self, tuplet: Tuplet) -> Self {
        self.tuplet = Some(tuplet);
        self
    }

    pub fn tied(mut self) -> Self {
        self.tied = true;
        self
    }

    /// Get duration in ticks (at 480 PPQN)
    pub fn ticks(&self) -> u64 {
        let base_ticks = self.base.ticks();

        // Apply dots: each dot adds half of the previous value
        let dotted_ticks = match self.dots {
            0 => base_ticks,
            1 => base_ticks + base_ticks / 2,
            2 => base_ticks + base_ticks / 2 + base_ticks / 4,
            3 => base_ticks + base_ticks / 2 + base_ticks / 4 + base_ticks / 8,
            _ => base_ticks,
        };

        // Apply tuplet
        if let Some(ref tuplet) = self.tuplet {
            (dotted_ticks as f64 * tuplet.ratio()) as u64
        } else {
            dotted_ticks
        }
    }

    /// Get display symbol
    pub fn symbol(&self) -> String {
        let base_sym = self.base.symbol();
        let dots = ".".repeat(self.dots as usize);

        if let Some(ref tuplet) = self.tuplet {
            format!("{}{}({})", base_sym, dots, tuplet.display())
        } else {
            format!("{}{}", base_sym, dots)
        }
    }
}

/// Base note duration without modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BaseDuration {
    /// Longa (4 whole notes) - used in early music
    Longa,
    /// Breve / double whole note
    Breve,
    /// Whole note (semibreve)
    Whole,
    /// Half note (minim)
    Half,
    /// Quarter note (crotchet)
    Quarter,
    /// Eighth note (quaver)
    Eighth,
    /// Sixteenth note (semiquaver)
    Sixteenth,
    /// 32nd note (demisemiquaver)
    ThirtySecond,
    /// 64th note (hemidemisemiquaver)
    SixtyFourth,
    /// 128th note
    OneHundredTwentyEighth,
    /// 256th note (extremely rare)
    TwoHundredFiftySixth,
}

impl BaseDuration {
    /// Duration in ticks at 480 PPQN
    pub fn ticks(&self) -> u64 {
        match self {
            Self::Longa => 480 * 16,
            Self::Breve => 480 * 8,
            Self::Whole => 480 * 4,
            Self::Half => 480 * 2,
            Self::Quarter => 480,
            Self::Eighth => 240,
            Self::Sixteenth => 120,
            Self::ThirtySecond => 60,
            Self::SixtyFourth => 30,
            Self::OneHundredTwentyEighth => 15,
            Self::TwoHundredFiftySixth => 7, // Rounded
        }
    }

    /// Display symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Longa => "𝅜",
            Self::Breve => "𝅝",
            Self::Whole => "𝅗𝅥",
            Self::Half => "𝅗𝅥",
            Self::Quarter => "♩",
            Self::Eighth => "♪",
            Self::Sixteenth => "𝅘𝅥𝅯",
            Self::ThirtySecond => "𝅘𝅥𝅰",
            Self::SixtyFourth => "𝅘𝅥𝅱",
            Self::OneHundredTwentyEighth => "128",
            Self::TwoHundredFiftySixth => "256",
        }
    }

    /// Short name (for UI)
    pub fn name(&self) -> &'static str {
        match self {
            Self::Longa => "Longa",
            Self::Breve => "Breve",
            Self::Whole => "Whole",
            Self::Half => "Half",
            Self::Quarter => "Quarter",
            Self::Eighth => "8th",
            Self::Sixteenth => "16th",
            Self::ThirtySecond => "32nd",
            Self::SixtyFourth => "64th",
            Self::OneHundredTwentyEighth => "128th",
            Self::TwoHundredFiftySixth => "256th",
        }
    }
}

/// Tuplet definition (n notes in the space of m)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tuplet {
    /// Number of notes played
    pub actual: u8,
    /// Space they occupy (in normal notes)
    pub normal: u8,
}

impl Tuplet {
    pub fn new(actual: u8, normal: u8) -> Self {
        Self { actual, normal }
    }

    /// Triplet (3 in the space of 2)
    pub fn triplet() -> Self {
        Self::new(3, 2)
    }

    /// Duplet (2 in the space of 3, used in compound time)
    pub fn duplet() -> Self {
        Self::new(2, 3)
    }

    /// Quintuplet (5 in space of 4)
    pub fn quintuplet() -> Self {
        Self::new(5, 4)
    }

    /// Sextuplet (6 in space of 4)
    pub fn sextuplet() -> Self {
        Self::new(6, 4)
    }

    /// Septuplet (7 in space of 4)
    pub fn septuplet() -> Self {
        Self::new(7, 4)
    }

    /// Nonuplet (9 in space of 8)
    pub fn nonuplet() -> Self {
        Self::new(9, 8)
    }

    /// Custom tuplet
    pub fn custom(actual: u8, normal: u8) -> Self {
        Self::new(actual, normal)
    }

    /// Duration ratio (multiply base ticks by this)
    pub fn ratio(&self) -> f64 {
        self.normal as f64 / self.actual as f64
    }

    /// Display text
    pub fn display(&self) -> String {
        if self.normal == 2 && self.actual == 3 {
            "3".to_string()
        } else {
            format!("{}:{}", self.actual, self.normal)
        }
    }
}

/// Time signature
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeSignature {
    /// Numerator (beats per measure)
    pub numerator: u8,
    /// Denominator (note value that gets one beat)
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

impl TimeSignature {
    pub fn new(numerator: u8, denominator: u8) -> Self {
        Self { numerator, denominator }
    }

    // Common time signatures
    pub fn common() -> Self {
        Self::new(4, 4)
    }

    pub fn cut() -> Self {
        Self::new(2, 2)
    }

    pub fn waltz() -> Self {
        Self::new(3, 4)
    }

    pub fn six_eight() -> Self {
        Self::new(6, 8)
    }

    // Odd meters
    pub fn five_four() -> Self {
        Self::new(5, 4)
    }

    pub fn seven_eight() -> Self {
        Self::new(7, 8)
    }

    pub fn eleven_eight() -> Self {
        Self::new(11, 8)
    }

    pub fn thirteen_eight() -> Self {
        Self::new(13, 8)
    }

    // Very odd (prog/tech death territory)
    pub fn fifteen_sixteen() -> Self {
        Self::new(15, 16)
    }

    pub fn seventeen_sixteen() -> Self {
        Self::new(17, 16)
    }

    /// Is this a compound meter? (groupings of 3)
    pub fn is_compound(&self) -> bool {
        self.numerator % 3 == 0 && self.numerator >= 6
    }

    /// Is this a simple meter? (groupings of 2 or 4)
    pub fn is_simple(&self) -> bool {
        !self.is_compound() && self.denominator <= 4
    }

    /// Is this an odd/asymmetric meter?
    pub fn is_odd(&self) -> bool {
        !matches!(self.numerator, 2 | 3 | 4 | 6 | 9 | 12)
    }

    /// Duration of one measure in ticks (at 480 PPQN where quarter = 480)
    pub fn measure_ticks(&self) -> u64 {
        let beat_ticks = match self.denominator {
            1 => 480 * 4,
            2 => 480 * 2,
            4 => 480,
            8 => 240,
            16 => 120,
            32 => 60,
            64 => 30,
            _ => 480,
        };
        beat_ticks * self.numerator as u64
    }

    /// Number of beats per measure
    pub fn beats(&self) -> u8 {
        self.numerator
    }

    /// Display string
    pub fn display(&self) -> String {
        format!("{}/{}", self.numerator, self.denominator)
    }

    /// Grouping pattern (for beaming) - returns beat divisions
    /// e.g., 7/8 might be [2, 2, 3] or [3, 2, 2]
    pub fn default_grouping(&self) -> Vec<u8> {
        match (self.numerator, self.denominator) {
            (4, 4) => vec![1, 1, 1, 1],
            (3, 4) => vec![1, 1, 1],
            (2, 4) => vec![1, 1],
            (6, 8) => vec![3, 3],
            (9, 8) => vec![3, 3, 3],
            (12, 8) => vec![3, 3, 3, 3],
            (5, 4) => vec![3, 2],  // Most common grouping
            (5, 8) => vec![3, 2],
            (7, 8) => vec![2, 2, 3], // Most common for rock/metal
            (11, 8) => vec![3, 3, 3, 2],
            (13, 8) => vec![3, 3, 3, 2, 2],
            (15, 16) => vec![4, 4, 4, 3],
            _ => vec![self.numerator],
        }
    }
}

/// Tempo map for tempo changes over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempoMap {
    /// Base tempo
    pub base_tempo: f64,
    /// Tempo events
    pub events: Vec<TempoEvent>,
}

impl TempoMap {
    pub fn new(bpm: f64) -> Self {
        Self {
            base_tempo: bpm,
            events: Vec::new(),
        }
    }

    /// Add a tempo change
    pub fn add_change(&mut self, measure: usize, beat: f64, tempo: f64) {
        self.events.push(TempoEvent {
            measure,
            beat,
            tempo,
            change_type: TempoEventType::Immediate,
        });
    }

    /// Add gradual tempo change
    pub fn add_gradual(&mut self, measure: usize, beat: f64, tempo: f64, event_type: TempoEventType) {
        self.events.push(TempoEvent {
            measure,
            beat,
            tempo,
            change_type: event_type,
        });
    }

    /// Get tempo at position
    pub fn tempo_at(&self, measure: usize, beat: f64) -> f64 {
        let mut current_tempo = self.base_tempo;

        for event in &self.events {
            if event.measure < measure || (event.measure == measure && event.beat <= beat) {
                current_tempo = event.tempo;
            }
        }

        current_tempo
    }
}

/// Tempo change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempoEvent {
    /// Measure number (0-indexed)
    pub measure: usize,
    /// Beat within measure (0-indexed, can be fractional)
    pub beat: f64,
    /// Target tempo
    pub tempo: f64,
    /// Type of change
    pub change_type: TempoEventType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TempoEventType {
    /// Immediate change
    Immediate,
    /// Accelerando (gradual speed up)
    Accelerando,
    /// Ritardando (gradual slow down)
    Ritardando,
    /// Rallentando (slowing at phrase end)
    Rallentando,
    /// A tempo (return to previous tempo)
    ATempo,
}

/// Metric modulation - tempo change via rhythmic relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricModulation {
    /// Old note value
    pub old_value: BaseDuration,
    /// New note value
    pub new_value: BaseDuration,
    /// Optional tuplet for old value
    pub old_tuplet: Option<Tuplet>,
    /// Optional tuplet for new value
    pub new_tuplet: Option<Tuplet>,
}

impl MetricModulation {
    /// Create modulation where old and new values are equal duration
    pub fn new(old_value: BaseDuration, new_value: BaseDuration) -> Self {
        Self {
            old_value,
            new_value,
            old_tuplet: None,
            new_tuplet: None,
        }
    }

    /// Classic "quarter = dotted quarter" feel change
    pub fn dotted_to_straight() -> Self {
        Self::new(BaseDuration::Quarter, BaseDuration::Quarter)
    }

    /// Calculate tempo ratio
    pub fn tempo_ratio(&self) -> f64 {
        let old_ticks = if let Some(ref t) = self.old_tuplet {
            self.old_value.ticks() as f64 * t.ratio()
        } else {
            self.old_value.ticks() as f64
        };

        let new_ticks = if let Some(ref t) = self.new_tuplet {
            self.new_value.ticks() as f64 * t.ratio()
        } else {
            self.new_value.ticks() as f64
        };

        old_ticks / new_ticks
    }
}

/// Polyrhythm notation (n against m)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polyrhythm {
    /// Top rhythm (primary voice)
    pub primary: PolyrhythmVoice,
    /// Bottom rhythm (secondary voice)
    pub secondary: PolyrhythmVoice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyrhythmVoice {
    /// Number of notes
    pub count: u8,
    /// Base duration of each note
    pub duration: BaseDuration,
}

impl Polyrhythm {
    /// 3 against 2
    pub fn three_against_two() -> Self {
        Self {
            primary: PolyrhythmVoice {
                count: 3,
                duration: BaseDuration::Eighth,
            },
            secondary: PolyrhythmVoice {
                count: 2,
                duration: BaseDuration::Eighth,
            },
        }
    }

    /// 4 against 3
    pub fn four_against_three() -> Self {
        Self {
            primary: PolyrhythmVoice {
                count: 4,
                duration: BaseDuration::Sixteenth,
            },
            secondary: PolyrhythmVoice {
                count: 3,
                duration: BaseDuration::Eighth,
            },
        }
    }

    /// 5 against 4
    pub fn five_against_four() -> Self {
        Self {
            primary: PolyrhythmVoice {
                count: 5,
                duration: BaseDuration::Sixteenth,
            },
            secondary: PolyrhythmVoice {
                count: 4,
                duration: BaseDuration::Sixteenth,
            },
        }
    }

    /// Custom polyrhythm
    pub fn custom(primary: u8, secondary: u8) -> Self {
        Self {
            primary: PolyrhythmVoice {
                count: primary,
                duration: BaseDuration::Sixteenth,
            },
            secondary: PolyrhythmVoice {
                count: secondary,
                duration: BaseDuration::Sixteenth,
            },
        }
    }

    /// Get display notation
    pub fn display(&self) -> String {
        format!("{}:{}", self.primary.count, self.secondary.count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rhythm_value_ticks() {
        // Quarter note = 480 ticks
        assert_eq!(RhythmValue::quarter().ticks(), 480);

        // Dotted quarter = 720 ticks
        assert_eq!(RhythmValue::quarter().dotted().ticks(), 720);

        // Eighth = 240 ticks
        assert_eq!(RhythmValue::eighth().ticks(), 240);

        // Triplet eighth = 160 ticks (240 * 2/3)
        assert_eq!(RhythmValue::eighth().triplet().ticks(), 160);
    }

    #[test]
    fn test_time_signature() {
        let four_four = TimeSignature::common();
        assert_eq!(four_four.measure_ticks(), 480 * 4);
        assert!(!four_four.is_odd());

        let seven_eight = TimeSignature::seven_eight();
        assert_eq!(seven_eight.measure_ticks(), 240 * 7);
        assert!(seven_eight.is_odd());
    }

    #[test]
    fn test_tuplets() {
        let triplet = Tuplet::triplet();
        assert!((triplet.ratio() - 0.666666).abs() < 0.001);

        let quintuplet = Tuplet::quintuplet();
        assert_eq!(quintuplet.ratio(), 0.8);
    }

    #[test]
    fn test_tempo_map() {
        let mut map = TempoMap::new(120.0);
        map.add_change(4, 0.0, 140.0);

        assert_eq!(map.tempo_at(0, 0.0), 120.0);
        assert_eq!(map.tempo_at(4, 0.0), 140.0);
        assert_eq!(map.tempo_at(10, 2.0), 140.0);
    }

    #[test]
    fn test_polyrhythm() {
        let poly = Polyrhythm::three_against_two();
        assert_eq!(poly.display(), "3:2");
    }
}
