//! Core types for compositional analysis
//!
//! These types are shared across all analysis components.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics for a single measure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeasureStats {
    /// Measure number (1-indexed)
    pub number: usize,
    /// Note density (notes per beat)
    pub density: f32,
    /// Average fret position
    pub avg_fret: f32,
    /// Fret position variance (textural exploration)
    pub fret_variance: f32,
    /// Dissonance index (weighted interval score)
    pub dissonance: f32,
    /// String spread (unique strings used)
    pub string_spread: f32,
    /// Note count in this measure
    pub note_count: usize,
    /// Beat count in this measure
    pub beat_count: usize,
}

impl MeasureStats {
    /// Calculate deviation from a reference baseline
    pub fn deviation_from(&self, baseline: &MeasureStats) -> f32 {
        let mut score = 0.0;

        // Variance deviation (formulaic writing detection)
        if baseline.fret_variance > 0.0 {
            let var_ratio = self.fret_variance / baseline.fret_variance;
            if var_ratio < 0.3 {
                score += 2.0 * (1.0 - var_ratio);
            }
        }

        // Dissonance deviation (too clean for style)
        if baseline.dissonance > 0.0 {
            let dis_ratio = self.dissonance / baseline.dissonance;
            if dis_ratio < 0.5 {
                score += 1.5 * (1.0 - dis_ratio);
            }
        }

        // Density deviation
        if baseline.density > 0.0 && self.density > 0.0 {
            let den_ratio = self.density / baseline.density;
            if den_ratio < 0.5 || den_ratio > 2.0 {
                score += 1.0;
            }
        }

        // String spread deviation
        if self.string_spread < 2.0 && self.density > 0.5 {
            score += 1.0;
        }

        score
    }
}

/// A contiguous region of measures with shared characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRegion {
    /// Start measure (1-indexed)
    pub start: usize,
    /// End measure (1-indexed, inclusive)
    pub end: usize,
    /// Accumulated anomaly score for this region
    pub score: f32,
    /// Detected issues in this region
    pub issues: Vec<String>,
    /// Region statistics (aggregated from measures)
    pub stats: MeasureStats,
}

impl AnalysisRegion {
    /// Get the length of this region in measures
    pub fn length(&self) -> usize {
        self.end.saturating_sub(self.start) + 1
    }

    /// Check if a measure falls within this region
    pub fn contains(&self, measure: usize) -> bool {
        measure >= self.start && measure <= self.end
    }
}

/// Pitch class (0-11, C=0 through B=11)
pub type PitchClass = u8;

/// Pitch class name lookup
pub fn pitch_name(pc: PitchClass) -> &'static str {
    const NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    NAMES.get((pc % 12) as usize).unwrap_or(&"?")
}

/// Interval in semitones
pub type Interval = u8;

/// Interval name lookup
pub fn interval_name(semitones: Interval) -> &'static str {
    const NAMES: [&str; 13] = [
        "unison", "m2", "M2", "m3", "M3", "P4", "tritone", "P5", "m6", "M6", "m7", "M7", "octave",
    ];
    NAMES.get((semitones % 12) as usize).unwrap_or(&"?")
}

/// Interval classification for dissonance weighting
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntervalClass {
    /// Perfect consonance (unison, octave, P5)
    PerfectConsonance,
    /// Imperfect consonance (M3, m3, M6, m6)
    ImperfectConsonance,
    /// Mild dissonance (M2, m7)
    MildDissonance,
    /// Sharp dissonance (m2, M7)
    SharpDissonance,
    /// Tritone (ambiguous)
    Tritone,
}

impl IntervalClass {
    /// Classify an interval by semitone count
    pub fn classify(semitones: u8) -> Self {
        match semitones % 12 {
            0 => Self::PerfectConsonance,       // unison
            1 => Self::SharpDissonance,         // m2
            2 => Self::MildDissonance,          // M2
            3 => Self::ImperfectConsonance,     // m3
            4 => Self::ImperfectConsonance,     // M3
            5 => Self::PerfectConsonance,       // P4 (context dependent, treating as consonant)
            6 => Self::Tritone,                 // tritone
            7 => Self::PerfectConsonance,       // P5
            8 => Self::ImperfectConsonance,     // m6
            9 => Self::ImperfectConsonance,     // M6
            10 => Self::MildDissonance,         // m7
            11 => Self::SharpDissonance,        // M7
            _ => Self::PerfectConsonance,       // octave
        }
    }

    /// Get dissonance weight for this interval class
    pub fn dissonance_weight(&self) -> f32 {
        match self {
            Self::PerfectConsonance => 0.0,
            Self::ImperfectConsonance => 0.5,
            Self::MildDissonance => 1.5,
            Self::SharpDissonance => 3.0,
            Self::Tritone => 2.5,
        }
    }
}

/// Distribution of pitch classes or intervals
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Distribution {
    /// Counts per item
    pub counts: HashMap<u8, usize>,
    /// Total count
    pub total: usize,
}

impl Distribution {
    /// Add a count for an item
    pub fn add(&mut self, item: u8) {
        *self.counts.entry(item).or_insert(0) += 1;
        self.total += 1;
    }

    /// Get percentage for an item
    pub fn percentage(&self, item: u8) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        100.0 * self.counts.get(&item).copied().unwrap_or(0) as f32 / self.total as f32
    }

    /// Get top N items sorted by count
    pub fn top(&self, n: usize) -> Vec<(u8, f32)> {
        let mut items: Vec<_> = self
            .counts
            .iter()
            .map(|(&k, &v)| (k, 100.0 * v as f32 / self.total.max(1) as f32))
            .collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items.truncate(n);
        items
    }

    /// Get entropy (measure of uniformity)
    pub fn entropy(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        let mut entropy = 0.0;
        for &count in self.counts.values() {
            if count > 0 {
                let p = count as f32 / self.total as f32;
                entropy -= p * p.ln();
            }
        }
        entropy
    }
}

/// Root motion between consecutive chords
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RootMotion {
    /// Static (same root)
    Static,
    /// Chromatic (±1 semitone)
    Chromatic(i8),
    /// Minor third (±3 semitones)
    MinorThird(i8),
    /// Major third (±4 semitones)
    MajorThird(i8),
    /// Perfect fourth/fifth (±5/7 semitones)
    FourthFifth(i8),
    /// Tritone (±6 semitones)
    Tritone,
    /// Other
    Other(i8),
}

impl RootMotion {
    /// Classify a root motion by semitone interval
    pub fn classify(semitones: i8) -> Self {
        // Normalize to -6..+6 range
        let normalized = if semitones > 6 {
            semitones - 12
        } else if semitones < -6 {
            semitones + 12
        } else {
            semitones
        };

        match normalized.abs() {
            0 => Self::Static,
            1 => Self::Chromatic(normalized),
            3 => Self::MinorThird(normalized),
            4 => Self::MajorThird(normalized),
            5 | 7 => Self::FourthFifth(normalized),
            6 => Self::Tritone,
            _ => Self::Other(normalized),
        }
    }

    /// Get name for this motion type
    pub fn name(&self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Chromatic(_) => "chromatic",
            Self::MinorThird(_) => "minor 3rd",
            Self::MajorThird(_) => "major 3rd",
            Self::FourthFifth(_) => "4th/5th",
            Self::Tritone => "tritone",
            Self::Other(_) => "other",
        }
    }
}

/// Chord quality classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChordQuality {
    /// Has minor 3rd
    pub has_minor_third: bool,
    /// Has major 3rd
    pub has_major_third: bool,
    /// Has perfect 5th
    pub has_perfect_fifth: bool,
    /// Has tritone
    pub has_tritone: bool,
    /// Has minor 7th
    pub has_minor_seventh: bool,
    /// Has major 7th
    pub has_major_seventh: bool,
    /// Has flat 9 (or minor 2nd)
    pub has_flat_nine: bool,
    /// Raw intervals present
    pub intervals: Vec<u8>,
}

impl ChordQuality {
    /// Create from a set of intervals from root
    pub fn from_intervals(intervals: &[u8]) -> Self {
        Self {
            has_minor_third: intervals.contains(&3),
            has_major_third: intervals.contains(&4),
            has_perfect_fifth: intervals.contains(&7),
            has_tritone: intervals.contains(&6),
            has_minor_seventh: intervals.contains(&10),
            has_major_seventh: intervals.contains(&11),
            has_flat_nine: intervals.contains(&1) || intervals.contains(&13),
            intervals: intervals.to_vec(),
        }
    }

    /// Get descriptive name
    pub fn name(&self) -> String {
        // Power chord special case (just root + 5th)
        if self.is_power_chord() && !self.has_minor_seventh && !self.has_major_seventh {
            return "5".to_string();
        }

        let mut parts = Vec::new();

        if self.has_minor_third {
            parts.push("m3");
        }
        if self.has_major_third {
            parts.push("M3");
        }
        if self.has_tritone {
            parts.push("b5");
        }
        if self.has_perfect_fifth && (self.has_minor_third || self.has_major_third) {
            parts.push("P5");
        }
        if self.has_minor_seventh {
            parts.push("m7");
        }
        if self.has_major_seventh {
            parts.push("M7");
        }
        if self.has_flat_nine {
            parts.push("b9");
        }

        if parts.is_empty() {
            return format!("intervals:{:?}", self.intervals);
        }

        parts.join("+")
    }

    /// Check if this is a power chord (just root + 5th)
    pub fn is_power_chord(&self) -> bool {
        self.has_perfect_fifth
            && !self.has_minor_third
            && !self.has_major_third
            && !self.has_tritone
    }

    /// Calculate dissonance score
    pub fn dissonance_score(&self) -> f32 {
        let mut score = 0.0;
        if self.has_major_seventh {
            score += 3.0;
        }
        if self.has_flat_nine {
            score += 2.5;
        }
        if self.has_tritone {
            score += 2.0;
        }
        if self.has_minor_seventh {
            score += 1.0;
        }
        score
    }
}

/// Track analysis data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackAnalysis {
    /// Track name
    pub name: String,
    /// Total note count
    pub note_count: usize,
    /// Average fret position
    pub avg_fret: f32,
    /// Fret variance
    pub fret_variance: f32,
    /// Average string position
    pub avg_string: f32,
    /// Inferred role
    pub role: TrackRole,
}

/// Inferred compositional role of a track
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TrackRole {
    /// Lead voice (high register, high variance)
    Lead,
    /// Rhythm anchor (low register, consistent patterns)
    Rhythm,
    /// Upper harmony voice
    UpperHarmony,
    /// Bass foundation
    BassFoundation,
    /// Inner voice / filler
    #[default]
    InnerVoice,
    /// Drums
    Drums,
    /// Orchestral/synth (extreme variance)
    Orchestral,
}

impl TrackRole {
    /// Infer role from track statistics
    pub fn infer(avg_fret: f32, fret_variance: f32, avg_string: f32, is_drums: bool) -> Self {
        if is_drums {
            return Self::Drums;
        }

        if avg_fret > 12.0 && fret_variance > 20.0 {
            Self::Lead
        } else if avg_fret < 6.0 && fret_variance < 10.0 {
            Self::Rhythm
        } else if fret_variance > 50.0 {
            Self::Orchestral
        } else if avg_string < 2.5 {
            Self::UpperHarmony
        } else if avg_string > 4.5 {
            Self::BassFoundation
        } else {
            Self::InnerVoice
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Lead => "high register, melodic exploration",
            Self::Rhythm => "low register, foundational pulse",
            Self::UpperHarmony => "high strings, upper voice in texture",
            Self::BassFoundation => "low strings, bottom of harmonic stack",
            Self::InnerVoice => "mid-register, harmonic filler",
            Self::Drums => "percussion",
            Self::Orchestral => "extreme variance, likely non-guitar",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_classification() {
        assert_eq!(
            IntervalClass::classify(1),
            IntervalClass::SharpDissonance
        );
        assert_eq!(
            IntervalClass::classify(6),
            IntervalClass::Tritone
        );
        assert_eq!(
            IntervalClass::classify(7),
            IntervalClass::PerfectConsonance
        );
        assert_eq!(
            IntervalClass::classify(11),
            IntervalClass::SharpDissonance
        );
    }

    #[test]
    fn test_distribution() {
        let mut dist = Distribution::default();
        dist.add(0); // C
        dist.add(0); // C
        dist.add(4); // E
        dist.add(7); // G

        assert_eq!(dist.total, 4);
        assert_eq!(dist.percentage(0), 50.0);
        assert_eq!(dist.percentage(4), 25.0);

        let top = dist.top(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, 0); // C is most common
    }

    #[test]
    fn test_root_motion() {
        assert_eq!(RootMotion::classify(0), RootMotion::Static);
        assert_eq!(RootMotion::classify(1), RootMotion::Chromatic(1));
        assert_eq!(RootMotion::classify(-1), RootMotion::Chromatic(-1));
        assert_eq!(RootMotion::classify(6), RootMotion::Tritone);
    }

    #[test]
    fn test_chord_quality() {
        // Power chord
        let power = ChordQuality::from_intervals(&[7]);
        assert!(power.is_power_chord());
        assert_eq!(power.name(), "5");

        // Major 7th chord
        let maj7 = ChordQuality::from_intervals(&[4, 7, 11]);
        assert!(maj7.has_major_seventh);
        assert_eq!(maj7.dissonance_score(), 3.0);
    }

    #[test]
    fn test_track_role_inference() {
        assert_eq!(
            TrackRole::infer(15.0, 25.0, 2.0, false),
            TrackRole::Lead
        );
        assert_eq!(
            TrackRole::infer(4.0, 8.0, 5.0, false),
            TrackRole::Rhythm
        );
        assert_eq!(
            TrackRole::infer(8.0, 12.0, 4.0, true),
            TrackRole::Drums
        );
    }
}
