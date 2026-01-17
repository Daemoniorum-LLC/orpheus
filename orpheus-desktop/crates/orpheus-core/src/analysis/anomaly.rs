//! Anomaly Detection - Identifying stylistic deviations
//!
//! Detects sections that deviate from established compositional patterns,
//! flagging potential "foreign" contributions or stylistic inconsistencies.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::tab::{Instrument, TabDocument, TabMeasure, TabTrack};
use super::fingerprint::ComposerFingerprint;
use super::types::{AnalysisRegion, IntervalClass, MeasureStats};

/// Configuration for anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyConfig {
    /// Minimum anomaly score to flag a measure
    pub score_threshold: f32,
    /// Maximum gap between anomalous measures to group into region
    pub region_gap: usize,
    /// Minimum region length to report
    pub min_region_length: usize,
    /// Variance deviation threshold (percentage)
    pub variance_threshold: f32,
    /// Dissonance deviation threshold (percentage)
    pub dissonance_threshold: f32,
    /// Density deviation range (percentage)
    pub density_threshold: f32,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            score_threshold: 1.5,
            region_gap: 4,
            min_region_length: 5,
            variance_threshold: 0.3,  // 30% of baseline
            dissonance_threshold: 0.5, // 50% of baseline
            density_threshold: 0.5,   // 50% deviation
        }
    }
}

/// Anomaly detector for compositional analysis
#[derive(Debug, Clone)]
pub struct AnomalyDetector {
    /// Baseline fingerprint to compare against
    baseline: ComposerFingerprint,
    /// Detection configuration
    config: AnomalyConfig,
}

impl AnomalyDetector {
    /// Create a new detector with the given baseline fingerprint
    pub fn new(baseline: ComposerFingerprint) -> Self {
        Self {
            baseline,
            config: AnomalyConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(baseline: ComposerFingerprint, config: AnomalyConfig) -> Self {
        Self { baseline, config }
    }

    /// Get the baseline fingerprint
    pub fn baseline(&self) -> &ComposerFingerprint {
        &self.baseline
    }

    /// Analyze a document for anomalies
    pub fn analyze(&self, doc: &TabDocument) -> AnomalyReport {
        let measure_stats: Vec<MeasureStats> = doc
            .measures
            .iter()
            .map(|m| self.calculate_measure_stats(m, &doc.tracks))
            .collect();

        let baseline_stats = self.baseline.as_baseline_stats();
        let mut anomaly_scores: Vec<(usize, f32, Vec<String>)> = Vec::new();

        for stats in &measure_stats {
            let (score, reasons) = self.score_measure(stats, &baseline_stats);
            if score > self.config.score_threshold {
                anomaly_scores.push((stats.number, score, reasons));
            }
        }

        let regions = self.group_into_regions(anomaly_scores, &measure_stats);

        AnomalyReport {
            baseline: self.baseline.clone(),
            measure_stats,
            regions,
            config: self.config.clone(),
        }
    }

    /// Calculate statistics for a single measure
    fn calculate_measure_stats(&self, measure: &TabMeasure, tracks: &[TabTrack]) -> MeasureStats {
        let mut all_frets: Vec<u8> = Vec::new();
        let mut all_strings: Vec<u8> = Vec::new();
        let mut intervals: Vec<u8> = Vec::new();
        let mut note_count = 0usize;
        let mut beat_count = 0usize;

        // Build track lookup
        let track_map: std::collections::HashMap<Uuid, &TabTrack> = tracks
            .iter()
            .map(|t| (t.id, t))
            .collect();

        for tm in &measure.track_beats {
            let track = track_map.get(&tm.track_id);
            let tuning = track.and_then(|t| match &t.instrument {
                Instrument::StringedInstrument(s) => Some(&s.tuning),
                _ => None,
            });
            let string_count = track
                .and_then(|t| match &t.instrument {
                    Instrument::StringedInstrument(s) => Some(s.string_count),
                    _ => None,
                })
                .unwrap_or(6);

            for beat in &tm.beats {
                beat_count += 1;
                note_count += beat.notes.len();

                for note in &beat.notes {
                    all_frets.push(note.fret);
                    all_strings.push(note.string);
                }

                // Calculate intervals within chord
                if beat.notes.len() >= 2 {
                    if let Some(tuning) = tuning {
                        let mut pitches: Vec<u8> = beat
                            .notes
                            .iter()
                            .filter_map(|n| {
                                let string_idx = (string_count - n.string) as usize;
                                tuning.get(string_idx).map(|open| open + n.fret)
                            })
                            .collect();
                        pitches.sort();

                        for i in 0..pitches.len().saturating_sub(1) {
                            let interval =
                                ((pitches[i + 1] as i16 - pitches[i] as i16).abs() % 12) as u8;
                            if interval > 0 {
                                intervals.push(interval);
                            }
                        }
                    }
                }
            }
        }

        let density = if beat_count > 0 {
            note_count as f32 / beat_count as f32
        } else {
            0.0
        };

        let avg_fret = if !all_frets.is_empty() {
            all_frets.iter().map(|f| *f as f32).sum::<f32>() / all_frets.len() as f32
        } else {
            0.0
        };

        let fret_variance = if !all_frets.is_empty() {
            let mean = avg_fret;
            all_frets
                .iter()
                .map(|f| (*f as f32 - mean).powi(2))
                .sum::<f32>()
                / all_frets.len() as f32
        } else {
            0.0
        };

        // Dissonance score
        let dissonance = if !intervals.is_empty() {
            intervals
                .iter()
                .map(|i| IntervalClass::classify(*i).dissonance_weight())
                .sum::<f32>()
                / intervals.len() as f32
        } else {
            0.0
        };

        let unique_strings: HashSet<u8> = all_strings.iter().copied().collect();
        let string_spread = unique_strings.len() as f32;

        MeasureStats {
            number: measure.number,
            density,
            avg_fret,
            fret_variance,
            dissonance,
            string_spread,
            note_count,
            beat_count,
        }
    }

    /// Score a measure against baseline
    fn score_measure(
        &self,
        stats: &MeasureStats,
        baseline: &MeasureStats,
    ) -> (f32, Vec<String>) {
        let mut score = 0.0;
        let mut reasons = Vec::new();

        // Check for LOW variance (formulaic playing)
        if stats.density > 0.5 && baseline.fret_variance > 0.0 {
            let ratio = stats.fret_variance / baseline.fret_variance;
            if ratio < self.config.variance_threshold {
                score += 2.0 * (1.0 - ratio);
                reasons.push(format!(
                    "low variance ({:.1} vs {:.1})",
                    stats.fret_variance, baseline.fret_variance
                ));
            }
        }

        // Check for LOW dissonance (too "clean" for style)
        if stats.density > 0.5 && baseline.dissonance > 0.0 {
            let ratio = stats.dissonance / baseline.dissonance;
            if ratio < self.config.dissonance_threshold {
                score += 1.5 * (1.0 - ratio);
                reasons.push(format!(
                    "low dissonance ({:.2} vs {:.2})",
                    stats.dissonance, baseline.dissonance
                ));
            }
        }

        // Check for unusual density patterns
        if stats.density > 0.0 && baseline.density > 0.0 {
            let ratio = stats.density / baseline.density;
            if ratio < self.config.density_threshold || ratio > (1.0 / self.config.density_threshold) {
                score += 1.0;
                reasons.push(format!("unusual density ({:.2})", stats.density));
            }
        }

        // Check for limited string spread
        if stats.string_spread < 2.0 && stats.density > 0.5 {
            score += 1.0;
            reasons.push(format!("limited strings ({:.0})", stats.string_spread));
        }

        (score, reasons)
    }

    /// Group consecutive anomalous measures into regions
    fn group_into_regions(
        &self,
        anomaly_scores: Vec<(usize, f32, Vec<String>)>,
        measure_stats: &[MeasureStats],
    ) -> Vec<AnalysisRegion> {
        if anomaly_scores.is_empty() {
            return Vec::new();
        }

        let mut regions: Vec<AnalysisRegion> = Vec::new();
        let mut current_start = 0usize;
        let mut current_end = 0usize;
        let mut current_score = 0.0;
        let mut current_reasons: Vec<String> = Vec::new();

        for (measure, score, reasons) in anomaly_scores {
            if current_start == 0 {
                // First anomaly
                current_start = measure;
                current_end = measure;
                current_score = score;
                current_reasons = reasons;
            } else if measure <= current_end + self.config.region_gap {
                // Extend current region
                current_end = measure;
                current_score += score;
                for reason in reasons {
                    if !current_reasons.contains(&reason) {
                        current_reasons.push(reason);
                    }
                }
            } else {
                // Start new region (if current is long enough)
                if current_end - current_start + 1 >= self.config.min_region_length {
                    let stats = self.aggregate_region_stats(
                        current_start,
                        current_end,
                        measure_stats,
                    );
                    regions.push(AnalysisRegion {
                        start: current_start,
                        end: current_end,
                        score: current_score,
                        issues: current_reasons.clone(),
                        stats,
                    });
                }
                current_start = measure;
                current_end = measure;
                current_score = score;
                current_reasons = reasons;
            }
        }

        // Don't forget the last region
        if current_end - current_start + 1 >= self.config.min_region_length {
            let stats = self.aggregate_region_stats(current_start, current_end, measure_stats);
            regions.push(AnalysisRegion {
                start: current_start,
                end: current_end,
                score: current_score,
                issues: current_reasons,
                stats,
            });
        }

        // Sort by score descending
        regions.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        regions
    }

    /// Aggregate statistics for a region
    fn aggregate_region_stats(
        &self,
        start: usize,
        end: usize,
        measure_stats: &[MeasureStats],
    ) -> MeasureStats {
        let region_stats: Vec<_> = measure_stats
            .iter()
            .filter(|s| s.number >= start && s.number <= end)
            .collect();

        if region_stats.is_empty() {
            return MeasureStats::default();
        }

        let n = region_stats.len() as f32;
        MeasureStats {
            number: 0, // Not applicable for aggregates
            density: region_stats.iter().map(|s| s.density).sum::<f32>() / n,
            avg_fret: region_stats.iter().map(|s| s.avg_fret).sum::<f32>() / n,
            fret_variance: region_stats.iter().map(|s| s.fret_variance).sum::<f32>() / n,
            dissonance: region_stats.iter().map(|s| s.dissonance).sum::<f32>() / n,
            string_spread: region_stats.iter().map(|s| s.string_spread).sum::<f32>() / n,
            note_count: region_stats.iter().map(|s| s.note_count).sum(),
            beat_count: region_stats.iter().map(|s| s.beat_count).sum(),
        }
    }
}

/// Report from anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    /// Baseline fingerprint used
    pub baseline: ComposerFingerprint,
    /// Per-measure statistics
    pub measure_stats: Vec<MeasureStats>,
    /// Detected anomalous regions
    pub regions: Vec<AnalysisRegion>,
    /// Configuration used
    pub config: AnomalyConfig,
}

impl AnomalyReport {
    /// Check if any anomalies were detected
    pub fn has_anomalies(&self) -> bool {
        !self.regions.is_empty()
    }

    /// Get total measures flagged as anomalous
    pub fn total_anomalous_measures(&self) -> usize {
        self.regions.iter().map(|r| r.length()).sum()
    }

    /// Get the most severe region
    pub fn most_severe(&self) -> Option<&AnalysisRegion> {
        self.regions.first()
    }

    /// Check if a specific measure is in an anomalous region
    pub fn is_anomalous(&self, measure: usize) -> bool {
        self.regions.iter().any(|r| r.contains(measure))
    }

    /// Get deviation comparison for a region
    pub fn region_deviation(&self, region: &AnalysisRegion) -> RegionDeviation {
        let baseline = self.baseline.as_baseline_stats();

        let variance_delta = if baseline.fret_variance > 0.0 {
            (region.stats.fret_variance - baseline.fret_variance) / baseline.fret_variance * 100.0
        } else {
            0.0
        };

        let dissonance_delta = if baseline.dissonance > 0.01 {
            (region.stats.dissonance - baseline.dissonance) / baseline.dissonance * 100.0
        } else {
            0.0
        };

        let density_delta = if baseline.density > 0.0 {
            (region.stats.density - baseline.density) / baseline.density * 100.0
        } else {
            0.0
        };

        RegionDeviation {
            region: region.clone(),
            baseline: baseline.clone(),
            variance_delta,
            dissonance_delta,
            density_delta,
        }
    }

    /// Generate summary text
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        if self.regions.is_empty() {
            lines.push("No significant anomalies detected.".to_string());
        } else {
            lines.push(format!(
                "Detected {} anomalous region(s) ({} measures total):",
                self.regions.len(),
                self.total_anomalous_measures()
            ));

            for (i, region) in self.regions.iter().take(5).enumerate() {
                lines.push(format!(
                    "\n{}. Measures {}-{} (score: {:.1})",
                    i + 1,
                    region.start,
                    region.end,
                    region.score
                ));
                lines.push(format!("   Issues: {}", region.issues.join("; ")));

                let dev = self.region_deviation(region);
                lines.push(format!(
                    "   Variance: {:+.0}%, Dissonance: {:+.0}%, Density: {:+.0}%",
                    dev.variance_delta, dev.dissonance_delta, dev.density_delta
                ));
            }
        }

        lines.join("\n")
    }
}

/// Deviation metrics for a specific region
#[derive(Debug, Clone)]
pub struct RegionDeviation {
    pub region: AnalysisRegion,
    pub baseline: MeasureStats,
    pub variance_delta: f32,
    pub dissonance_delta: f32,
    pub density_delta: f32,
}

impl RegionDeviation {
    /// Check if this region has severe variance collapse
    pub fn has_variance_collapse(&self) -> bool {
        self.variance_delta < -50.0
    }

    /// Check if this region lacks harmonic tension
    pub fn lacks_tension(&self) -> bool {
        self.dissonance_delta < -50.0
    }

    /// Get prescription for fixing this region
    pub fn prescriptions(&self) -> Vec<String> {
        let mut prescriptions = Vec::new();

        if self.variance_delta < -50.0 {
            prescriptions.push(format!(
                "Increase fret variance from {:.1} → {:.1}+",
                self.region.stats.fret_variance,
                self.baseline.fret_variance * 0.8
            ));
            prescriptions.push("Break repetitive patterns, add register exploration".to_string());
        }

        if self.dissonance_delta < -50.0 {
            prescriptions.push("Add more dissonant intervals (M7, m2, tritone)".to_string());
            prescriptions.push("Increase harmonic tension to match style".to_string());
        }

        if self.density_delta.abs() > 50.0 {
            if self.density_delta < 0.0 {
                prescriptions.push("Increase note density".to_string());
            } else {
                prescriptions.push("Reduce note density to match style".to_string());
            }
        }

        prescriptions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let baseline = ComposerFingerprint {
            fret_variance: 100.0,
            density: 2.0,
            dissonance: 2.0,
            ..Default::default()
        };

        let detector = AnomalyDetector::new(baseline.clone());
        assert_eq!(detector.baseline().fret_variance, 100.0);
    }

    #[test]
    fn test_config_defaults() {
        let config = AnomalyConfig::default();
        assert_eq!(config.score_threshold, 1.5);
        assert_eq!(config.region_gap, 4);
        assert_eq!(config.min_region_length, 5);
    }

    #[test]
    fn test_measure_scoring() {
        let baseline = ComposerFingerprint {
            fret_variance: 100.0,
            density: 2.0,
            dissonance: 2.0,
            ..Default::default()
        };

        let detector = AnomalyDetector::new(baseline);
        let baseline_stats = detector.baseline().as_baseline_stats();

        // Low variance measure
        let low_var = MeasureStats {
            number: 1,
            fret_variance: 10.0, // 90% lower
            density: 2.0,
            dissonance: 2.0,
            string_spread: 3.0,
            ..Default::default()
        };

        let (score, reasons) = detector.score_measure(&low_var, &baseline_stats);
        assert!(score > 1.0);
        assert!(!reasons.is_empty());
    }

    #[test]
    fn test_region_deviation() {
        let baseline = ComposerFingerprint {
            fret_variance: 100.0,
            density: 2.0,
            dissonance: 2.0,
            ..Default::default()
        };

        let region = AnalysisRegion {
            start: 1,
            end: 10,
            score: 5.0,
            issues: vec!["low variance".to_string()],
            stats: MeasureStats {
                fret_variance: 20.0,
                dissonance: 0.5,
                density: 2.0,
                ..Default::default()
            },
        };

        let report = AnomalyReport {
            baseline,
            measure_stats: Vec::new(),
            regions: vec![region.clone()],
            config: AnomalyConfig::default(),
        };

        let deviation = report.region_deviation(&region);
        assert!(deviation.has_variance_collapse());
        assert!(deviation.lacks_tension());
        assert!(!deviation.prescriptions().is_empty());
    }
}
