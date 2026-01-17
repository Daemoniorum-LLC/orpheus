//! Composer Fingerprint - Statistical voice capture
//!
//! Captures the statistical "voice" of a composer from a body of work,
//! enabling comparison against sections to detect foreign contributions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::tab::{Instrument, TabDocument, TabMeasure, TabTrack};
use super::types::{Distribution, IntervalClass, MeasureStats, TrackAnalysis, TrackRole};

/// A statistical fingerprint of a composer's style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposerFingerprint {
    /// Source measures used to calculate fingerprint
    pub source_measures: Vec<usize>,
    /// Overall fret variance (textural exploration metric)
    pub fret_variance: f32,
    /// Note density (notes per beat)
    pub density: f32,
    /// Dissonance index (weighted interval tension)
    pub dissonance: f32,
    /// Root note distribution (pitch class percentages)
    pub root_distribution: Distribution,
    /// Interval vocabulary distribution
    pub interval_distribution: Distribution,
    /// Average fret position
    pub avg_fret: f32,
    /// Average string spread
    pub avg_string_spread: f32,
    /// Per-track analysis
    pub track_analysis: Vec<TrackAnalysis>,
    /// Total notes analyzed
    pub total_notes: usize,
    /// Total beats analyzed
    pub total_beats: usize,
}

impl Default for ComposerFingerprint {
    fn default() -> Self {
        Self {
            source_measures: Vec::new(),
            fret_variance: 0.0,
            density: 0.0,
            dissonance: 0.0,
            root_distribution: Distribution::default(),
            interval_distribution: Distribution::default(),
            avg_fret: 0.0,
            avg_string_spread: 0.0,
            track_analysis: Vec::new(),
            total_notes: 0,
            total_beats: 0,
        }
    }
}

impl ComposerFingerprint {
    /// Calculate fingerprint from an entire document
    pub fn from_document(doc: &TabDocument) -> Self {
        let measures: Vec<usize> = doc.measures.iter().map(|m| m.number).collect();
        Self::from_measures(doc, &measures)
    }

    /// Calculate fingerprint from specific measures
    pub fn from_measures(doc: &TabDocument, measure_numbers: &[usize]) -> Self {
        let measure_set: std::collections::HashSet<_> = measure_numbers.iter().collect();
        let measures: Vec<_> = doc
            .measures
            .iter()
            .filter(|m| measure_set.contains(&m.number))
            .collect();

        Self::calculate(&measures, &doc.tracks)
    }

    /// Calculate fingerprint from measures, excluding specific ranges
    pub fn excluding_ranges(doc: &TabDocument, exclude: &[(usize, usize)]) -> Self {
        let measures: Vec<usize> = doc
            .measures
            .iter()
            .filter_map(|m| {
                let in_excluded = exclude.iter().any(|(start, end)| {
                    m.number >= *start && m.number <= *end
                });
                if in_excluded { None } else { Some(m.number) }
            })
            .collect();

        Self::from_measures(doc, &measures)
    }

    /// Core calculation from measure slice
    fn calculate(measures: &[&TabMeasure], tracks: &[TabTrack]) -> Self {
        let mut all_frets: Vec<u8> = Vec::new();
        let mut all_strings: Vec<u8> = Vec::new();
        let mut note_count = 0usize;
        let mut beat_count = 0usize;
        let mut root_dist = Distribution::default();
        let mut interval_dist = Distribution::default();
        let mut track_data: HashMap<Uuid, (Vec<u8>, Vec<u8>)> = HashMap::new();
        let source_measures: Vec<usize> = measures.iter().map(|m| m.number).collect();

        // Build track ID to config map
        let track_configs: HashMap<Uuid, &TabTrack> = tracks.iter()
            .map(|t| (t.id, t))
            .collect();

        for measure in measures {
            for tm in &measure.track_beats {
                let track = track_configs.get(&tm.track_id);
                let tuning = track.and_then(|t| match &t.instrument {
                    Instrument::StringedInstrument(s) => Some(&s.tuning),
                    _ => None,
                });
                let string_count = track.and_then(|t| match &t.instrument {
                    Instrument::StringedInstrument(s) => Some(s.string_count),
                    _ => None,
                }).unwrap_or(6);

                for beat in &tm.beats {
                    beat_count += 1;
                    note_count += beat.notes.len();

                    for note in &beat.notes {
                        all_frets.push(note.fret);
                        all_strings.push(note.string);

                        // Track per-track data
                        let entry = track_data.entry(tm.track_id).or_insert_with(|| (Vec::new(), Vec::new()));
                        entry.0.push(note.fret);
                        entry.1.push(note.string);
                    }

                    // Analyze chords for root and intervals
                    if beat.notes.len() >= 2 {
                        if let Some(tuning) = tuning {
                            let mut pitches: Vec<u8> = beat
                                .notes
                                .iter()
                                .filter_map(|n| {
                                    // Convert string number to tuning index
                                    // String 1 = highest pitch = last in tuning array
                                    let string_idx = (string_count - n.string) as usize;
                                    tuning.get(string_idx).map(|open| open + n.fret)
                                })
                                .collect();
                            pitches.sort();
                            pitches.dedup();

                            if let Some(&root) = pitches.first() {
                                root_dist.add(root % 12);
                            }

                            // Calculate intervals within chord
                            for i in 0..pitches.len().saturating_sub(1) {
                                let interval = ((pitches[i + 1] as i16 - pitches[i] as i16).abs() % 12) as u8;
                                if interval > 0 {
                                    interval_dist.add(interval);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Calculate variance
        let (fret_variance, avg_fret) = if !all_frets.is_empty() {
            let mean = all_frets.iter().map(|f| *f as f32).sum::<f32>() / all_frets.len() as f32;
            let variance = all_frets
                .iter()
                .map(|f| (*f as f32 - mean).powi(2))
                .sum::<f32>()
                / all_frets.len() as f32;
            (variance, mean)
        } else {
            (0.0, 0.0)
        };

        // Calculate density
        let density = if beat_count > 0 {
            note_count as f32 / beat_count as f32
        } else {
            0.0
        };

        // Calculate dissonance (weighted interval score)
        let dissonance = if interval_dist.total > 0 {
            interval_dist
                .counts
                .iter()
                .map(|(&interval, &count)| {
                    let class = IntervalClass::classify(interval);
                    class.dissonance_weight() * count as f32
                })
                .sum::<f32>()
                / interval_dist.total as f32
        } else {
            0.0
        };

        // Calculate average string spread per measure
        let mut spreads: Vec<f32> = Vec::new();
        for measure in measures {
            let unique_strings: std::collections::HashSet<u8> = measure
                .track_beats
                .iter()
                .flat_map(|tm| &tm.beats)
                .flat_map(|b| &b.notes)
                .map(|n| n.string)
                .collect();
            spreads.push(unique_strings.len() as f32);
        }
        let avg_string_spread = if !spreads.is_empty() {
            spreads.iter().sum::<f32>() / spreads.len() as f32
        } else {
            0.0
        };

        // Calculate per-track analysis
        let track_analysis: Vec<TrackAnalysis> = tracks
            .iter()
            .filter_map(|track| {
                let (frets, strings) = track_data.get(&track.id)?;
                if frets.is_empty() {
                    return None;
                }

                let avg_fret = frets.iter().map(|f| *f as f32).sum::<f32>() / frets.len() as f32;
                let avg_string = strings.iter().map(|s| *s as f32).sum::<f32>() / strings.len() as f32;
                let fret_variance = {
                    let mean = avg_fret;
                    frets.iter().map(|f| (*f as f32 - mean).powi(2)).sum::<f32>() / frets.len() as f32
                };

                let is_drums = matches!(track.instrument, Instrument::Drums(_));
                let role = TrackRole::infer(avg_fret, fret_variance, avg_string, is_drums);

                Some(TrackAnalysis {
                    name: track.name.clone(),
                    note_count: frets.len(),
                    avg_fret,
                    fret_variance,
                    avg_string,
                    role,
                })
            })
            .collect();

        Self {
            source_measures,
            fret_variance,
            density,
            dissonance,
            root_distribution: root_dist,
            interval_distribution: interval_dist,
            avg_fret,
            avg_string_spread,
            track_analysis,
            total_notes: note_count,
            total_beats: beat_count,
        }
    }

    /// Calculate deviation of this fingerprint from another
    pub fn deviation_from(&self, baseline: &ComposerFingerprint) -> FingerprintDeviation {
        let variance_delta = if baseline.fret_variance > 0.0 {
            (self.fret_variance - baseline.fret_variance) / baseline.fret_variance * 100.0
        } else {
            0.0
        };

        let density_delta = if baseline.density > 0.0 {
            (self.density - baseline.density) / baseline.density * 100.0
        } else {
            0.0
        };

        let dissonance_delta = if baseline.dissonance > 0.01 {
            (self.dissonance - baseline.dissonance) / baseline.dissonance * 100.0
        } else {
            0.0
        };

        FingerprintDeviation {
            variance_delta,
            density_delta,
            dissonance_delta,
            is_significant: variance_delta.abs() > 50.0
                || density_delta.abs() > 50.0
                || dissonance_delta.abs() > 50.0,
        }
    }

    /// Get the dominant root notes (top N)
    pub fn dominant_roots(&self, n: usize) -> Vec<(u8, f32)> {
        self.root_distribution.top(n)
    }

    /// Get the dominant intervals (top N)
    pub fn dominant_intervals(&self, n: usize) -> Vec<(u8, f32)> {
        self.interval_distribution.top(n)
    }

    /// Check if M7 intervals are signature (>40% of vocabulary)
    pub fn has_m7_signature(&self) -> bool {
        self.interval_distribution.percentage(11) > 40.0
    }

    /// Check for chromatic root motion (E↔F oscillation pattern)
    pub fn has_chromatic_oscillation(&self) -> bool {
        let e_pct = self.root_distribution.percentage(4); // E
        let f_pct = self.root_distribution.percentage(5); // F
        e_pct > 30.0 && f_pct > 30.0 && (e_pct - f_pct).abs() < 15.0
    }

    /// Get aggregate baseline stats for comparison
    pub fn as_baseline_stats(&self) -> MeasureStats {
        MeasureStats {
            number: 0,
            density: self.density,
            avg_fret: self.avg_fret,
            fret_variance: self.fret_variance,
            dissonance: self.dissonance,
            string_spread: self.avg_string_spread,
            note_count: self.total_notes,
            beat_count: self.total_beats,
        }
    }

    /// Generate a textual summary of the fingerprint
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Measures analyzed: {}", self.source_measures.len()));
        lines.push(format!("Total notes: {}", self.total_notes));
        lines.push(format!("Fret variance: {:.1}", self.fret_variance));
        lines.push(format!("Note density: {:.2} notes/beat", self.density));
        lines.push(format!("Dissonance index: {:.2}", self.dissonance));

        lines.push("\nDominant roots:".to_string());
        for (root, pct) in self.dominant_roots(4) {
            lines.push(format!("  {}: {:.1}%", super::types::pitch_name(root), pct));
        }

        lines.push("\nInterval vocabulary:".to_string());
        for (interval, pct) in self.dominant_intervals(5) {
            lines.push(format!(
                "  {}: {:.1}%",
                super::types::interval_name(interval),
                pct
            ));
        }

        if self.has_m7_signature() {
            lines.push("\n→ M7 signature detected (high semitone tension)".to_string());
        }
        if self.has_chromatic_oscillation() {
            lines.push("→ Chromatic oscillation pattern (E↔F)".to_string());
        }

        lines.join("\n")
    }
}

/// Deviation metrics between fingerprints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintDeviation {
    /// Variance change percentage
    pub variance_delta: f32,
    /// Density change percentage
    pub density_delta: f32,
    /// Dissonance change percentage
    pub dissonance_delta: f32,
    /// Whether the deviation is considered significant
    pub is_significant: bool,
}

impl FingerprintDeviation {
    /// Get primary issue description
    pub fn primary_issue(&self) -> Option<&'static str> {
        if self.variance_delta < -50.0 {
            Some("LOW VARIANCE: Formulaic/repetitive writing")
        } else if self.variance_delta > 100.0 {
            Some("HIGH VARIANCE: Unstable texture")
        } else if self.dissonance_delta < -50.0 {
            Some("LOW DISSONANCE: Missing harmonic tension")
        } else if self.dissonance_delta > 100.0 {
            Some("HIGH DISSONANCE: Excessive tension")
        } else if self.density_delta < -50.0 {
            Some("LOW DENSITY: Sparse note activity")
        } else if self.density_delta > 100.0 {
            Some("HIGH DENSITY: Excessive note activity")
        } else {
            None
        }
    }

    /// Get all detected issues
    pub fn all_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.variance_delta < -50.0 {
            issues.push(format!(
                "Variance {:.0}% below baseline (formulaic)",
                self.variance_delta.abs()
            ));
        }
        if self.dissonance_delta < -50.0 {
            issues.push(format!(
                "Dissonance {:.0}% below baseline (too clean)",
                self.dissonance_delta.abs()
            ));
        }
        if self.density_delta.abs() > 50.0 {
            issues.push(format!(
                "Density {:+.0}% from baseline",
                self.density_delta
            ));
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_default() {
        let fp = ComposerFingerprint::default();
        assert_eq!(fp.total_notes, 0);
        assert_eq!(fp.fret_variance, 0.0);
    }

    #[test]
    fn test_deviation_calculation() {
        let baseline = ComposerFingerprint {
            fret_variance: 100.0,
            density: 2.0,
            dissonance: 2.0,
            ..Default::default()
        };

        let problem = ComposerFingerprint {
            fret_variance: 20.0, // 80% lower
            density: 2.0,
            dissonance: 0.5, // 75% lower
            ..Default::default()
        };

        let deviation = problem.deviation_from(&baseline);
        assert!(deviation.variance_delta < -70.0);
        assert!(deviation.dissonance_delta < -70.0);
        assert!(deviation.is_significant);
        assert!(deviation.primary_issue().is_some());
    }

    #[test]
    fn test_m7_signature_detection() {
        let mut fp = ComposerFingerprint::default();
        fp.interval_distribution.total = 100;
        fp.interval_distribution.counts.insert(11, 50); // 50% M7
        assert!(fp.has_m7_signature());

        fp.interval_distribution.counts.insert(11, 30); // 30% M7
        assert!(!fp.has_m7_signature());
    }

    #[test]
    fn test_chromatic_oscillation_detection() {
        let mut fp = ComposerFingerprint::default();
        fp.root_distribution.total = 100;
        fp.root_distribution.counts.insert(4, 40); // 40% E
        fp.root_distribution.counts.insert(5, 38); // 38% F
        assert!(fp.has_chromatic_oscillation());

        fp.root_distribution.counts.insert(4, 50); // 50% E
        fp.root_distribution.counts.insert(5, 10); // 10% F
        assert!(!fp.has_chromatic_oscillation());
    }
}
