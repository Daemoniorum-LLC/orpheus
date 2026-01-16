//! # Compositional Analysis Suite
//!
//! Data-driven analysis tools for understanding and shaping compositions.
//!
//! ## Philosophy
//!
//! Music is data. This module treats compositions as analyzable structures,
//! extracting patterns that reveal the composer's voice and flagging deviations
//! that break consistency.
//!
//! ## Core Components
//!
//! - [`ComposerFingerprint`] - Captures a composer's statistical voice
//! - [`AnomalyDetector`] - Identifies sections that deviate from established patterns
//! - [`HarmonicAnalyzer`] - Maps tonal structure, root motion, interval vocabulary
//! - [`SurgicalPlanner`] - Generates targeted rewrite prescriptions
//!
//! ## Usage
//!
//! ### Quick Analysis
//!
//! ```ignore
//! use orpheus_core::analysis::{ComposerFingerprint, AnomalyDetector};
//!
//! // Calculate fingerprint from a document
//! let fingerprint = ComposerFingerprint::from_document(&document);
//!
//! // Detect anomalies against that fingerprint
//! let detector = AnomalyDetector::new(fingerprint);
//! let report = detector.analyze(&document);
//!
//! for region in &report.regions {
//!     println!("Measures {}-{}: {}", region.start, region.end, region.issues.join(", "));
//! }
//! ```
//!
//! ### Excluding Known Problem Sections
//!
//! ```ignore
//! use orpheus_core::analysis::{ComposerFingerprint, create_surgical_plan};
//!
//! // Calculate fingerprint excluding known problem sections
//! let exclude = &[(72, 144), (220, 263)];
//! let fingerprint = ComposerFingerprint::excluding_ranges(&document, exclude);
//!
//! // Or get a complete surgical plan
//! let plan = create_surgical_plan(&document, exclude);
//! println!("{}", plan.summary());
//! ```
//!
//! ### Harmonic Analysis
//!
//! ```ignore
//! use orpheus_core::analysis::HarmonicAnalyzer;
//!
//! let analyzer = HarmonicAnalyzer::new();
//! let harmony = analyzer.analyze(&document);
//!
//! // Check for signature patterns
//! if harmony.has_ef_oscillation() {
//!     println!("E↔F bitonal oscillation detected");
//! }
//! if harmony.has_m7_signature() {
//!     println!("M7 signature detected (high semitone tension)");
//! }
//!
//! println!("Tonal clarity: {:.2}", harmony.tonal_clarity());
//! ```
//!
//! ## Key Concepts
//!
//! ### Fingerprint Metrics
//!
//! - **Fret Variance**: Measure of textural exploration. High variance indicates
//!   use of the full fretboard; low variance suggests repetitive patterns.
//!
//! - **Density**: Notes per beat. Indicates rhythmic activity level.
//!
//! - **Dissonance Index**: Weighted score of interval tension. Higher values
//!   indicate more dissonant interval choices (m2, M7, tritone).
//!
//! ### Anomaly Detection
//!
//! Anomalies are detected by comparing measure-level statistics against a baseline
//! fingerprint. Regions with significantly lower variance or dissonance are flagged
//! as potentially "foreign" to the composer's voice.
//!
//! ### Harmonic Patterns
//!
//! - **E↔F Oscillation**: Bitonal root motion pattern common in certain styles,
//!   derived from Indian Raga traditions.
//!
//! - **M7 Signature**: Predominant use of major 7th intervals (semitone clusters)
//!   creating characteristic tension.
//!
//! - **Chromatic Motion**: Root movement by semitone, as opposed to functional
//!   fourth/fifth motion.

mod types;
mod fingerprint;
mod anomaly;
mod harmony;
mod surgical;

// Re-export core types
pub use types::{
    pitch_name, interval_name,
    MeasureStats, AnalysisRegion, PitchClass, Interval,
    IntervalClass, Distribution, RootMotion, ChordQuality,
    TrackAnalysis, TrackRole,
};

// Re-export fingerprint types
pub use fingerprint::{
    ComposerFingerprint, FingerprintDeviation,
};

// Re-export anomaly detection types
pub use anomaly::{
    AnomalyDetector, AnomalyConfig, AnomalyReport, RegionDeviation,
};

// Re-export harmonic analysis types
pub use harmony::{
    HarmonicAnalyzer, HarmonicAnalysis, ExtractedChord,
    TritoneStats, MotionBreakdown,
};

// Re-export surgical planning types
pub use surgical::{
    SurgicalPlanner, SurgicalPlan, RewritePrescription,
    Severity, ActionItem, ActionType, TargetMetrics, ReferenceSection,
    create_surgical_plan,
};
