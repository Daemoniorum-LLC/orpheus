//! Harmonic Analyzer - Tonal structure and chord language
//!
//! Analyzes harmonic language including root motion patterns,
//! chord vocabulary, tonal center detection, and interval usage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::tab::{Instrument, TabDocument, TabTrack};
use super::types::{
    pitch_name, interval_name, ChordQuality, Distribution, PitchClass,
};

/// A chord extracted from simultaneous notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedChord {
    /// Measure number
    pub measure: usize,
    /// Beat index within measure
    pub beat_index: usize,
    /// MIDI pitches (sorted low to high)
    pub pitches: Vec<u8>,
    /// Pitch classes present
    pub pitch_classes: Vec<PitchClass>,
    /// Root pitch class (lowest note)
    pub root: PitchClass,
    /// Intervals from root
    pub intervals: Vec<u8>,
    /// Chord quality
    pub quality: ChordQuality,
}

impl ExtractedChord {
    /// Get chord name (root + quality)
    pub fn name(&self) -> String {
        format!("{}{}", pitch_name(self.root), self.quality.name())
    }

    /// Check if chord contains tritone
    pub fn has_tritone(&self) -> bool {
        self.intervals.contains(&6)
    }

    /// Get bass note pitch class
    pub fn bass(&self) -> PitchClass {
        self.root
    }
}

/// Complete harmonic analysis of a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonicAnalysis {
    /// All extracted chords
    pub chords: Vec<ExtractedChord>,
    /// Root note distribution
    pub root_distribution: Distribution,
    /// Interval vocabulary distribution
    pub interval_distribution: Distribution,
    /// Chord quality distribution
    pub quality_distribution: HashMap<String, usize>,
    /// Root motion patterns
    pub root_motions: HashMap<i8, usize>,
    /// Detected tonal centers (weighted by structural importance)
    pub tonal_centers: Vec<(PitchClass, f32)>,
    /// Tritone usage statistics
    pub tritone_stats: TritoneStats,
    /// Progression patterns (3-chord sequences)
    pub progression_patterns: Vec<(String, usize)>,
}

/// Tritone usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TritoneStats {
    /// Total chords containing tritone
    pub count: usize,
    /// Percentage of all chords
    pub percentage: f32,
    /// Root distribution of tritone chords
    pub root_distribution: Distribution,
}

/// Harmonic analyzer
#[derive(Debug)]
pub struct HarmonicAnalyzer {
    /// Default tuning if track tuning unavailable
    default_tuning: Vec<u8>,
}

impl Default for HarmonicAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl HarmonicAnalyzer {
    /// Create a new harmonic analyzer
    pub fn new() -> Self {
        Self {
            // Standard E tuning
            default_tuning: vec![40, 45, 50, 55, 59, 64],
        }
    }

    /// Set default tuning
    pub fn with_default_tuning(mut self, tuning: Vec<u8>) -> Self {
        self.default_tuning = tuning;
        self
    }

    /// Analyze a document
    pub fn analyze(&self, doc: &TabDocument) -> HarmonicAnalysis {
        let chords = self.extract_chords(doc);
        let root_distribution = self.calculate_root_distribution(&chords);
        let interval_distribution = self.calculate_interval_distribution(&chords);
        let quality_distribution = self.calculate_quality_distribution(&chords);
        let root_motions = self.calculate_root_motions(&chords);
        let tonal_centers = self.detect_tonal_centers(&chords);
        let tritone_stats = self.calculate_tritone_stats(&chords);
        let progression_patterns = self.find_progression_patterns(&chords);

        HarmonicAnalysis {
            chords,
            root_distribution,
            interval_distribution,
            quality_distribution,
            root_motions,
            tonal_centers,
            tritone_stats,
            progression_patterns,
        }
    }

    /// Extract all chords from document
    fn extract_chords(&self, doc: &TabDocument) -> Vec<ExtractedChord> {
        let mut chords = Vec::new();

        // Build track lookup
        let track_map: HashMap<Uuid, &TabTrack> = doc
            .tracks
            .iter()
            .map(|t| (t.id, t))
            .collect();

        for measure in &doc.measures {
            for tm in &measure.track_beats {
                let track = track_map.get(&tm.track_id);
                let (tuning, string_count) = track
                    .and_then(|t| match &t.instrument {
                        Instrument::StringedInstrument(s) => {
                            Some((&s.tuning, s.string_count))
                        }
                        _ => None,
                    })
                    .unwrap_or((&self.default_tuning, 6));

                for (beat_idx, beat) in tm.beats.iter().enumerate() {
                    if beat.notes.len() >= 2 {
                        let mut pitches: Vec<u8> = beat
                            .notes
                            .iter()
                            .filter_map(|n| {
                                let string_idx = (string_count - n.string) as usize;
                                tuning.get(string_idx).map(|open| open + n.fret)
                            })
                            .collect();
                        pitches.sort();
                        pitches.dedup();

                        if pitches.len() >= 2 {
                            let pitch_classes: Vec<PitchClass> =
                                pitches.iter().map(|p| p % 12).collect();

                            let root = pitches[0] % 12;

                            let intervals: Vec<u8> = pitches
                                .iter()
                                .map(|p| ((*p as i16 - pitches[0] as i16).abs() % 12) as u8)
                                .filter(|i| *i > 0)
                                .collect();

                            let quality = ChordQuality::from_intervals(&intervals);

                            chords.push(ExtractedChord {
                                measure: measure.number,
                                beat_index: beat_idx,
                                pitches,
                                pitch_classes,
                                root,
                                intervals,
                                quality,
                            });
                        }
                    }
                }
            }
        }

        chords
    }

    /// Calculate root note distribution
    fn calculate_root_distribution(&self, chords: &[ExtractedChord]) -> Distribution {
        let mut dist = Distribution::default();
        for chord in chords {
            dist.add(chord.root);
        }
        dist
    }

    /// Calculate interval vocabulary distribution
    fn calculate_interval_distribution(&self, chords: &[ExtractedChord]) -> Distribution {
        let mut dist = Distribution::default();
        for chord in chords {
            for interval in &chord.intervals {
                dist.add(*interval);
            }
        }
        dist
    }

    /// Calculate chord quality distribution
    fn calculate_quality_distribution(&self, chords: &[ExtractedChord]) -> HashMap<String, usize> {
        let mut dist = HashMap::new();
        for chord in chords {
            let name = chord.quality.name();
            *dist.entry(name).or_insert(0) += 1;
        }
        dist
    }

    /// Calculate root motion patterns
    fn calculate_root_motions(&self, chords: &[ExtractedChord]) -> HashMap<i8, usize> {
        let mut motions = HashMap::new();

        for i in 1..chords.len() {
            // Only consider adjacent or nearly-adjacent chords
            if chords[i].measure <= chords[i - 1].measure + 1 {
                let motion = (chords[i].root as i8 - chords[i - 1].root as i8 + 18) % 12 - 6;
                // Normalize to -6 to +5 range
                let normalized = if motion > 6 {
                    motion - 12
                } else if motion < -6 {
                    motion + 12
                } else {
                    motion
                };
                *motions.entry(normalized).or_insert(0) += 1;
            }
        }

        motions
    }

    /// Detect tonal centers
    fn detect_tonal_centers(&self, chords: &[ExtractedChord]) -> Vec<(PitchClass, f32)> {
        let mut weights: HashMap<PitchClass, f32> = HashMap::new();

        for chord in chords {
            // Root gets more weight
            *weights.entry(chord.root).or_insert(0.0) += 2.0;
            // Other pitch classes get less
            for pc in &chord.pitch_classes {
                if *pc != chord.root {
                    *weights.entry(*pc).or_insert(0.0) += 0.5;
                }
            }
        }

        let mut centers: Vec<_> = weights.into_iter().collect();
        centers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        centers
    }

    /// Calculate tritone usage statistics
    fn calculate_tritone_stats(&self, chords: &[ExtractedChord]) -> TritoneStats {
        let tritone_chords: Vec<_> = chords.iter().filter(|c| c.has_tritone()).collect();

        let count = tritone_chords.len();
        let percentage = if !chords.is_empty() {
            100.0 * count as f32 / chords.len() as f32
        } else {
            0.0
        };

        let mut root_distribution = Distribution::default();
        for chord in &tritone_chords {
            root_distribution.add(chord.root);
        }

        TritoneStats {
            count,
            percentage,
            root_distribution,
        }
    }

    /// Find recurring progression patterns (3-chord sequences)
    fn find_progression_patterns(&self, chords: &[ExtractedChord]) -> Vec<(String, usize)> {
        let mut trigrams: HashMap<(PitchClass, PitchClass, PitchClass), usize> = HashMap::new();

        for i in 2..chords.len() {
            // Only consider chords within 2 measures of each other
            if chords[i].measure <= chords[i - 2].measure + 2 {
                let seq = (chords[i - 2].root, chords[i - 1].root, chords[i].root);
                *trigrams.entry(seq).or_insert(0) += 1;
            }
        }

        let mut patterns: Vec<_> = trigrams
            .into_iter()
            .filter(|(_, count)| *count >= 3)
            .map(|((a, b, c), count)| {
                let name = format!(
                    "{}→{}→{}",
                    pitch_name(a),
                    pitch_name(b),
                    pitch_name(c)
                );
                (name, count)
            })
            .collect();

        patterns.sort_by(|a, b| b.1.cmp(&a.1));
        patterns
    }
}

impl HarmonicAnalysis {
    /// Get primary tonal center (if clear)
    pub fn primary_tonal_center(&self) -> Option<PitchClass> {
        self.tonal_centers.first().map(|(pc, _)| *pc)
    }

    /// Check for tonal ambiguity
    pub fn is_tonally_ambiguous(&self) -> bool {
        if self.tonal_centers.len() < 2 {
            return false;
        }
        let ratio = self.tonal_centers[1].1 / self.tonal_centers[0].1;
        ratio > 0.85
    }

    /// Get tonal clarity score (0.0 = ambiguous, 1.0 = clear)
    pub fn tonal_clarity(&self) -> f32 {
        if self.tonal_centers.len() < 2 {
            return 1.0;
        }
        let ratio = self.tonal_centers[1].1 / self.tonal_centers[0].1;
        1.0 - ratio
    }

    /// Check for chromatic saturation (using nearly all 12 pitch classes)
    pub fn is_chromatically_saturated(&self) -> bool {
        self.root_distribution.counts.len() >= 10
    }

    /// Get motion type breakdown
    pub fn motion_breakdown(&self) -> MotionBreakdown {
        let total: usize = self.root_motions.values().sum();
        if total == 0 {
            return MotionBreakdown::default();
        }

        let static_count = self.root_motions.get(&0).copied().unwrap_or(0);
        let chromatic_count = self.root_motions.get(&1).copied().unwrap_or(0)
            + self.root_motions.get(&-1).copied().unwrap_or(0);
        let tritone_count = self.root_motions.get(&6).copied().unwrap_or(0);
        let fourth_fifth_count = self.root_motions.get(&5).copied().unwrap_or(0)
            + self.root_motions.get(&-5).copied().unwrap_or(0)
            + self.root_motions.get(&7).copied().unwrap_or(0);

        MotionBreakdown {
            static_pct: 100.0 * static_count as f32 / total as f32,
            chromatic_pct: 100.0 * chromatic_count as f32 / total as f32,
            tritone_pct: 100.0 * tritone_count as f32 / total as f32,
            fourth_fifth_pct: 100.0 * fourth_fifth_count as f32 / total as f32,
            total_motions: total,
        }
    }

    /// Check for E↔F oscillation pattern
    pub fn has_ef_oscillation(&self) -> bool {
        let e_pct = self.root_distribution.percentage(4);
        let f_pct = self.root_distribution.percentage(5);
        e_pct > 30.0 && f_pct > 30.0 && (e_pct - f_pct).abs() < 15.0
    }

    /// Check for M7 signature
    pub fn has_m7_signature(&self) -> bool {
        self.interval_distribution.percentage(11) > 40.0
    }

    /// Generate summary text
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("Total chords analyzed: {}", self.chords.len()));

        lines.push("\nRoot distribution:".to_string());
        for (root, pct) in self.root_distribution.top(5) {
            lines.push(format!("  {}: {:.1}%", pitch_name(root), pct));
        }

        lines.push("\nInterval vocabulary:".to_string());
        for (interval, pct) in self.interval_distribution.top(5) {
            lines.push(format!("  {}: {:.1}%", interval_name(interval), pct));
        }

        if let Some(center) = self.primary_tonal_center() {
            lines.push(format!(
                "\nTonal center: {} (clarity: {:.2})",
                pitch_name(center),
                self.tonal_clarity()
            ));
        }

        if self.is_tonally_ambiguous() {
            lines.push("→ HIGHLY AMBIGUOUS: Multiple competing centers".to_string());
        }

        if self.is_chromatically_saturated() {
            lines.push("→ CHROMATIC SATURATION: Using nearly full chromatic set".to_string());
        }

        let motion = self.motion_breakdown();
        lines.push("\nMotion breakdown:".to_string());
        lines.push(format!("  Static (pedal): {:.1}%", motion.static_pct));
        lines.push(format!("  Chromatic (±1): {:.1}%", motion.chromatic_pct));
        lines.push(format!("  Tritone (±6): {:.1}%", motion.tritone_pct));
        lines.push(format!("  4th/5th: {:.1}%", motion.fourth_fifth_pct));

        lines.push(format!(
            "\nTritone usage: {} chords ({:.1}%)",
            self.tritone_stats.count, self.tritone_stats.percentage
        ));

        if self.has_ef_oscillation() {
            lines.push("\n→ E↔F oscillation pattern detected".to_string());
        }
        if self.has_m7_signature() {
            lines.push("→ M7 signature detected (high semitone tension)".to_string());
        }

        if !self.progression_patterns.is_empty() {
            lines.push("\nRecurring progressions:".to_string());
            for (pattern, count) in self.progression_patterns.iter().take(5) {
                lines.push(format!("  {}: {} times", pattern, count));
            }
        }

        lines.join("\n")
    }
}

/// Root motion type breakdown
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MotionBreakdown {
    /// Static (pedal) motion percentage
    pub static_pct: f32,
    /// Chromatic (±1 semitone) motion percentage
    pub chromatic_pct: f32,
    /// Tritone (±6 semitone) motion percentage
    pub tritone_pct: f32,
    /// Fourth/fifth motion percentage
    pub fourth_fifth_pct: f32,
    /// Total motion count
    pub total_motions: usize,
}

impl MotionBreakdown {
    /// Check if predominantly chromatic
    pub fn is_chromatic_dominant(&self) -> bool {
        self.chromatic_pct > self.fourth_fifth_pct && self.chromatic_pct > 40.0
    }

    /// Check if predominantly functional (4th/5th motion)
    pub fn is_functional(&self) -> bool {
        self.fourth_fifth_pct > 40.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = HarmonicAnalyzer::new();
        assert_eq!(analyzer.default_tuning.len(), 6);
    }

    #[test]
    fn test_chord_quality() {
        // Power chord
        let power = ChordQuality::from_intervals(&[7]);
        assert!(power.is_power_chord());

        // Minor 7th chord
        let min7 = ChordQuality::from_intervals(&[3, 7, 10]);
        assert!(min7.has_minor_third);
        assert!(min7.has_minor_seventh);
    }

    #[test]
    fn test_motion_breakdown() {
        let mut analysis = HarmonicAnalysis {
            chords: Vec::new(),
            root_distribution: Distribution::default(),
            interval_distribution: Distribution::default(),
            quality_distribution: HashMap::new(),
            root_motions: HashMap::new(),
            tonal_centers: Vec::new(),
            tritone_stats: TritoneStats::default(),
            progression_patterns: Vec::new(),
        };

        analysis.root_motions.insert(1, 40);  // Chromatic
        analysis.root_motions.insert(-1, 30); // Chromatic
        analysis.root_motions.insert(5, 20);  // Fourth
        analysis.root_motions.insert(0, 10);  // Static

        let motion = analysis.motion_breakdown();
        assert!(motion.chromatic_pct > 60.0);
        assert!(motion.is_chromatic_dominant());
    }

    #[test]
    fn test_tonal_clarity() {
        let mut analysis = HarmonicAnalysis {
            chords: Vec::new(),
            root_distribution: Distribution::default(),
            interval_distribution: Distribution::default(),
            quality_distribution: HashMap::new(),
            root_motions: HashMap::new(),
            tonal_centers: vec![(4, 100.0), (5, 90.0)], // E and F nearly equal
            tritone_stats: TritoneStats::default(),
            progression_patterns: Vec::new(),
        };

        assert!(analysis.is_tonally_ambiguous());
        assert!(analysis.tonal_clarity() < 0.15);
    }
}
