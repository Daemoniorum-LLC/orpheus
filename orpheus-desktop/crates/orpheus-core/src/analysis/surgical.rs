//! Surgical Planner - Targeted rewrite prescriptions
//!
//! Generates specific, actionable prescriptions for rewriting
//! sections that deviate from the composer's established voice.

use serde::{Deserialize, Serialize};

use crate::tab::TabDocument;
use super::anomaly::{AnomalyDetector, AnomalyReport, RegionDeviation};
use super::fingerprint::ComposerFingerprint;
use super::harmony::{HarmonicAnalyzer, HarmonicAnalysis};
use super::types::AnalysisRegion;

/// A surgical rewrite prescription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewritePrescription {
    /// Target region (measure range)
    pub region: AnalysisRegion,
    /// Priority (higher = more urgent)
    pub priority: u8,
    /// Severity label
    pub severity: Severity,
    /// Primary diagnosis
    pub diagnosis: String,
    /// Specific action items
    pub actions: Vec<ActionItem>,
    /// Target metrics to achieve
    pub targets: TargetMetrics,
    /// Reference measures (examples of "good" writing)
    pub reference_measures: Vec<usize>,
}

/// Action severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Minor adjustment needed
    Minor,
    /// Moderate rewrite needed
    Moderate,
    /// Major rewrite needed
    Major,
    /// Complete rewrite recommended
    Critical,
}

impl Severity {
    /// From anomaly score
    pub fn from_score(score: f32) -> Self {
        if score < 3.0 {
            Self::Minor
        } else if score < 6.0 {
            Self::Moderate
        } else if score < 10.0 {
            Self::Major
        } else {
            Self::Critical
        }
    }

    /// Get color for UI
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Self::Minor => (255, 206, 86),    // Yellow
            Self::Moderate => (255, 159, 64), // Orange
            Self::Major => (255, 99, 132),    // Red
            Self::Critical => (153, 102, 255), // Purple
        }
    }

    /// Get label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Minor => "Minor",
            Self::Moderate => "Moderate",
            Self::Major => "Major",
            Self::Critical => "Critical",
        }
    }
}

/// A specific action item in a prescription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    /// Action type
    pub action_type: ActionType,
    /// Description
    pub description: String,
    /// Rationale
    pub rationale: String,
    /// Specific examples or suggestions
    pub examples: Vec<String>,
}

/// Type of action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    /// Increase textural variety
    IncreaseVariance,
    /// Add harmonic tension
    AddTension,
    /// Adjust density
    AdjustDensity,
    /// Change root motion
    AdjustRootMotion,
    /// Add interval variety
    AddIntervals,
    /// Expand register usage
    ExpandRegister,
    /// Break repetitive patterns
    BreakRepetition,
    /// Match stylistic conventions
    MatchStyle,
}

impl ActionType {
    /// Get icon for UI
    pub fn icon(&self) -> &'static str {
        match self {
            Self::IncreaseVariance => "📊",
            Self::AddTension => "⚡",
            Self::AdjustDensity => "📈",
            Self::AdjustRootMotion => "🔄",
            Self::AddIntervals => "🎵",
            Self::ExpandRegister => "📏",
            Self::BreakRepetition => "🔀",
            Self::MatchStyle => "🎯",
        }
    }
}

/// Target metrics for a region
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetMetrics {
    /// Target fret variance
    pub variance_target: Option<f32>,
    /// Target dissonance level
    pub dissonance_target: Option<f32>,
    /// Target density
    pub density_target: Option<f32>,
    /// Suggested root notes to emphasize
    pub preferred_roots: Vec<u8>,
    /// Suggested intervals to incorporate
    pub preferred_intervals: Vec<u8>,
}

/// Surgical planner for rewrite prescriptions
#[derive(Debug)]
pub struct SurgicalPlanner {
    /// Baseline fingerprint
    fingerprint: ComposerFingerprint,
    /// Harmonic analysis
    harmony: Option<HarmonicAnalysis>,
}

impl SurgicalPlanner {
    /// Create planner from fingerprint
    pub fn new(fingerprint: ComposerFingerprint) -> Self {
        Self {
            fingerprint,
            harmony: None,
        }
    }

    /// Add harmonic analysis for richer prescriptions
    pub fn with_harmony(mut self, harmony: HarmonicAnalysis) -> Self {
        self.harmony = Some(harmony);
        self
    }

    /// Generate prescriptions from an anomaly report
    pub fn plan_from_report(&self, report: &AnomalyReport, doc: &TabDocument) -> SurgicalPlan {
        let prescriptions: Vec<RewritePrescription> = report
            .regions
            .iter()
            .map(|region| self.prescribe_region(region, report))
            .collect();

        let reference_measures = self.find_reference_measures(doc, &report.baseline);

        SurgicalPlan {
            fingerprint: self.fingerprint.clone(),
            prescriptions,
            reference_measures,
            total_measures_affected: report.total_anomalous_measures(),
        }
    }

    /// Generate prescription for a single region
    fn prescribe_region(
        &self,
        region: &AnalysisRegion,
        report: &AnomalyReport,
    ) -> RewritePrescription {
        let deviation = report.region_deviation(region);
        let mut actions = Vec::new();
        let severity = Severity::from_score(region.score);

        // Analyze specific issues and generate actions
        if deviation.has_variance_collapse() {
            actions.push(ActionItem {
                action_type: ActionType::IncreaseVariance,
                description: format!(
                    "Increase fret variance from {:.1} to {:.1}+",
                    region.stats.fret_variance,
                    self.fingerprint.fret_variance * 0.8
                ),
                rationale: "Section is too formulaic/repetitive compared to your style".to_string(),
                examples: vec![
                    "Add register shifts between phrases".to_string(),
                    "Vary fret positions instead of staying in one zone".to_string(),
                    "Use the full neck, not just the low frets".to_string(),
                ],
            });

            actions.push(ActionItem {
                action_type: ActionType::BreakRepetition,
                description: "Break repetitive patterns".to_string(),
                rationale: "Your compositional voice has more textural exploration".to_string(),
                examples: vec![
                    "Interrupt patterns with unexpected melodic fragments".to_string(),
                    "Use octave displacement".to_string(),
                    "Add chromatic passing tones".to_string(),
                ],
            });
        }

        if deviation.lacks_tension() {
            actions.push(ActionItem {
                action_type: ActionType::AddTension,
                description: format!(
                    "Increase harmonic tension from {:.2} to {:.2}+",
                    region.stats.dissonance,
                    self.fingerprint.dissonance * 0.8
                ),
                rationale: "Section is too 'clean' for your style".to_string(),
                examples: self.tension_examples(),
            });
        }

        if deviation.density_delta.abs() > 50.0 {
            let action_type = if deviation.density_delta < 0.0 {
                ActionType::AdjustDensity
            } else {
                ActionType::AdjustDensity
            };

            actions.push(ActionItem {
                action_type,
                description: format!(
                    "Adjust density from {:.2} to {:.2} notes/beat",
                    region.stats.density,
                    self.fingerprint.density
                ),
                rationale: format!(
                    "Density is {:+.0}% from your baseline",
                    deviation.density_delta
                ),
                examples: vec![
                    if deviation.density_delta < 0.0 {
                        "Add more notes per beat".to_string()
                    } else {
                        "Reduce note density for clarity".to_string()
                    },
                ],
            });
        }

        // Add style-matching action if we have harmonic analysis
        if let Some(ref harmony) = self.harmony {
            actions.push(self.create_style_action(harmony));
        }

        // Generate target metrics
        let targets = TargetMetrics {
            variance_target: Some(self.fingerprint.fret_variance * 0.8),
            dissonance_target: Some(self.fingerprint.dissonance * 0.8),
            density_target: Some(self.fingerprint.density),
            preferred_roots: self.fingerprint.dominant_roots(3)
                .into_iter()
                .map(|(r, _)| r)
                .collect(),
            preferred_intervals: self.fingerprint.dominant_intervals(3)
                .into_iter()
                .map(|(i, _)| i)
                .collect(),
        };

        let diagnosis = self.generate_diagnosis(&deviation);

        RewritePrescription {
            region: region.clone(),
            priority: self.calculate_priority(&deviation),
            severity,
            diagnosis,
            actions,
            targets,
            reference_measures: Vec::new(), // Filled in later
        }
    }

    /// Generate tension examples based on fingerprint
    fn tension_examples(&self) -> Vec<String> {
        let mut examples = Vec::new();

        if self.fingerprint.has_m7_signature() {
            examples.push("Stack M7 intervals for your signature semitone tension".to_string());
        }

        examples.push("Add tritone relationships".to_string());
        examples.push("Use minor 2nd intervals".to_string());

        if self.fingerprint.has_chromatic_oscillation() {
            examples.push("Maintain E↔F oscillation as primary root motion".to_string());
        }

        examples
    }

    /// Create style-matching action from harmonic analysis
    fn create_style_action(&self, harmony: &HarmonicAnalysis) -> ActionItem {
        let mut examples = Vec::new();

        if harmony.has_ef_oscillation() {
            examples.push("Use E↔F root oscillation pattern".to_string());
        }

        if harmony.has_m7_signature() {
            examples.push("Stack M7 intervals (semitone clusters)".to_string());
        }

        let motion = harmony.motion_breakdown();
        if motion.is_chromatic_dominant() {
            examples.push(format!(
                "Maintain chromatic root motion ({:.0}% of progressions)",
                motion.chromatic_pct
            ));
        }

        if examples.is_empty() {
            examples.push("Match interval vocabulary from reference sections".to_string());
        }

        ActionItem {
            action_type: ActionType::MatchStyle,
            description: "Match your established harmonic language".to_string(),
            rationale: "Section should sound like it belongs with the rest of the piece".to_string(),
            examples,
        }
    }

    /// Generate primary diagnosis
    fn generate_diagnosis(&self, deviation: &RegionDeviation) -> String {
        let mut issues = Vec::new();

        if deviation.has_variance_collapse() {
            issues.push("formulaic/repetitive writing");
        }
        if deviation.lacks_tension() {
            issues.push("missing harmonic tension");
        }
        if deviation.density_delta.abs() > 50.0 {
            issues.push("unusual note density");
        }

        if issues.is_empty() {
            "Stylistic inconsistency detected".to_string()
        } else {
            format!("Primary issues: {}", issues.join(", "))
        }
    }

    /// Calculate priority (1-10)
    fn calculate_priority(&self, deviation: &RegionDeviation) -> u8 {
        let mut priority = 5;

        if deviation.has_variance_collapse() {
            priority += 2;
        }
        if deviation.lacks_tension() {
            priority += 2;
        }
        if deviation.variance_delta < -80.0 {
            priority += 1;
        }

        priority.min(10)
    }

    /// Find reference measures (highest variance, most "you")
    fn find_reference_measures(
        &self,
        doc: &TabDocument,
        fingerprint: &ComposerFingerprint,
    ) -> Vec<ReferenceSection> {
        // Find measures with highest variance (most textural exploration)
        let mut measure_scores: Vec<(usize, f32)> = Vec::new();

        for measure in &doc.measures {
            let mut frets: Vec<u8> = Vec::new();
            for tm in &measure.track_beats {
                for beat in &tm.beats {
                    for note in &beat.notes {
                        frets.push(note.fret);
                    }
                }
            }

            if frets.len() >= 4 {
                let mean = frets.iter().map(|f| *f as f32).sum::<f32>() / frets.len() as f32;
                let variance = frets
                    .iter()
                    .map(|f| (*f as f32 - mean).powi(2))
                    .sum::<f32>()
                    / frets.len() as f32;
                measure_scores.push((measure.number, variance));
            }
        }

        measure_scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top measures, excluding those in problem regions
        let excluded: std::collections::HashSet<usize> = fingerprint
            .source_measures
            .iter()
            .copied()
            .collect();

        measure_scores
            .into_iter()
            .filter(|(m, _)| excluded.contains(m))
            .take(10)
            .map(|(measure, variance)| ReferenceSection {
                measure,
                variance,
                description: format!("High textural exploration (σ²={:.1})", variance),
            })
            .collect()
    }
}

/// A reference section for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceSection {
    /// Measure number
    pub measure: usize,
    /// Variance score
    pub variance: f32,
    /// Description
    pub description: String,
}

/// Complete surgical plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurgicalPlan {
    /// Source fingerprint
    pub fingerprint: ComposerFingerprint,
    /// Prescriptions for each problem region
    pub prescriptions: Vec<RewritePrescription>,
    /// Reference measures (examples of good writing)
    pub reference_measures: Vec<ReferenceSection>,
    /// Total measures affected
    pub total_measures_affected: usize,
}

impl SurgicalPlan {
    /// Check if plan is empty (no rewrites needed)
    pub fn is_empty(&self) -> bool {
        self.prescriptions.is_empty()
    }

    /// Get critical prescriptions
    pub fn critical(&self) -> impl Iterator<Item = &RewritePrescription> {
        self.prescriptions
            .iter()
            .filter(|p| p.severity == Severity::Critical)
    }

    /// Get prescriptions sorted by priority
    pub fn by_priority(&self) -> Vec<&RewritePrescription> {
        let mut sorted: Vec<_> = self.prescriptions.iter().collect();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        sorted
    }

    /// Generate summary text
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "SURGICAL REWRITE PLAN: {} measures need attention",
            self.total_measures_affected
        ));
        lines.push(format!(
            "Prescriptions: {} total",
            self.prescriptions.len()
        ));

        let critical_count = self.prescriptions.iter()
            .filter(|p| p.severity == Severity::Critical)
            .count();
        let major_count = self.prescriptions.iter()
            .filter(|p| p.severity == Severity::Major)
            .count();

        if critical_count > 0 {
            lines.push(format!("  ⚠️  {} critical", critical_count));
        }
        if major_count > 0 {
            lines.push(format!("  🔶 {} major", major_count));
        }

        lines.push("\nYour baseline metrics:".to_string());
        lines.push(format!("  Fret variance: {:.1}", self.fingerprint.fret_variance));
        lines.push(format!("  Density: {:.2} notes/beat", self.fingerprint.density));
        lines.push(format!("  Dissonance: {:.2}", self.fingerprint.dissonance));

        for (i, prescription) in self.prescriptions.iter().take(3).enumerate() {
            lines.push(format!(
                "\n{}. Measures {}-{} [{}]",
                i + 1,
                prescription.region.start,
                prescription.region.end,
                prescription.severity.label()
            ));
            lines.push(format!("   {}", prescription.diagnosis));
            for action in &prescription.actions {
                lines.push(format!("   {} {}", action.action_type.icon(), action.description));
            }
        }

        if !self.reference_measures.is_empty() {
            lines.push("\nReference measures (your best sections):".to_string());
            for reference in self.reference_measures.iter().take(5) {
                lines.push(format!("  Measure {}: {}", reference.measure, reference.description));
            }
        }

        lines.join("\n")
    }
}

/// Convenience function to create a complete surgical plan
pub fn create_surgical_plan(
    doc: &TabDocument,
    exclude_ranges: &[(usize, usize)],
) -> SurgicalPlan {
    // Calculate fingerprint from non-excluded measures
    let fingerprint = ComposerFingerprint::excluding_ranges(doc, exclude_ranges);

    // Run anomaly detection
    let detector = AnomalyDetector::new(fingerprint.clone());
    let report = detector.analyze(doc);

    // Run harmonic analysis
    let analyzer = HarmonicAnalyzer::new();
    let harmony = analyzer.analyze(doc);

    // Generate plan
    let planner = SurgicalPlanner::new(fingerprint).with_harmony(harmony);
    planner.plan_from_report(&report, doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_from_score() {
        assert_eq!(Severity::from_score(1.0), Severity::Minor);
        assert_eq!(Severity::from_score(5.0), Severity::Moderate);
        assert_eq!(Severity::from_score(8.0), Severity::Major);
        assert_eq!(Severity::from_score(15.0), Severity::Critical);
    }

    #[test]
    fn test_action_type_icons() {
        assert!(!ActionType::IncreaseVariance.icon().is_empty());
        assert!(!ActionType::AddTension.icon().is_empty());
    }

    #[test]
    fn test_planner_creation() {
        let fingerprint = ComposerFingerprint {
            fret_variance: 100.0,
            density: 2.0,
            dissonance: 2.0,
            ..Default::default()
        };

        let planner = SurgicalPlanner::new(fingerprint);
        assert!(planner.harmony.is_none());
    }

    #[test]
    fn test_empty_plan() {
        let plan = SurgicalPlan {
            fingerprint: ComposerFingerprint::default(),
            prescriptions: Vec::new(),
            reference_measures: Vec::new(),
            total_measures_affected: 0,
        };

        assert!(plan.is_empty());
        assert_eq!(plan.critical().count(), 0);
    }
}
