//! # Orpheus Core
//!
//! Core types, state management, commands, and events for the Orpheus desktop application.

pub mod state;
pub mod commands;
pub mod compression;
pub mod events;
pub mod project;
pub mod config;
pub mod recording;
pub mod playback;
pub mod tab;
pub mod analysis;

pub use state::*;
pub use recording::{RecordingState, TrackRecordingBuffer, RecordedMidiEvent};
pub use playback::{ClipPlaybackEngine, ScheduledMidiEvent};
pub use commands::{
    // Track commands
    AddTrackCommand, BoxedCommand, Command, CommandContext, CommandHistory,
    CommandRegistry, MoveTrackCommand, RemoveTrackCommand, RenameTrackCommand,
    // Clip commands
    AddClipCommand, DeleteClipCommand, DuplicateClipCommand, MoveClipCommand,
    RenameClipCommand, TrimClipCommand,
};
pub use events::{AppEvent, EventBus};
pub use project::*;
pub use config::Settings;
pub use compression::{
    CompressionDictionary, CompressionError, CompressionStats,
    DictionaryTrainer, BuiltinDictionary,
};

// Tab notation types
pub use tab::{
    // Document
    TabDocument, TabMetadata, TabTrack, TabMeasure, TrackMeasure, TabBeat, TabNote,
    SectionMarker, RepeatMarker, KeySignature,
    // Instruments
    Instrument, StringedConfig, StringedType, MultiscaleConfig, TremoloType,
    DrumKit, DrumMidiMap, KeysConfig, Fingering,
    // Techniques
    Technique, BendData, BendAmount, BendPoint, WhammyTechnique, DiveDepth,
    WhammyPoint, SlideDirection, TapType, VibratoStyle, PalmMuteIntensity,
    SweepDirection, TrillData, GraceNoteData, BeatEffect, TempoChange,
    // Drums
    DrumHit, DrumPiece, DrumArticulation, DrumPattern, FlamData, DragData,
    // Rhythm
    RhythmValue, BaseDuration, Tuplet, TimeSignature, TempoMap, TempoEvent,
    MetricModulation, Polyrhythm,
    // Notation
    TextExpression, RehearsalMark, BarlineType, Ottava, Spanner, Clef,
    Accidental, ArticulationSymbol, Ornament, TremoloNotation, TremoloStrokes,
    NavigationSymbol, LayoutHints,
};

// Compositional analysis types
pub use analysis::{
    // Core types
    pitch_name, interval_name, MeasureStats, AnalysisRegion, PitchClass, Interval,
    IntervalClass, Distribution, RootMotion, ChordQuality, TrackAnalysis, TrackRole,
    // Fingerprinting
    ComposerFingerprint, FingerprintDeviation,
    // Anomaly detection
    AnomalyDetector, AnomalyConfig, AnomalyReport, RegionDeviation,
    // Harmonic analysis
    HarmonicAnalyzer, HarmonicAnalysis, ExtractedChord, TritoneStats, MotionBreakdown,
    // Surgical planning
    SurgicalPlanner, SurgicalPlan, RewritePrescription, Severity, ActionItem,
    ActionType, TargetMetrics, ReferenceSection, create_surgical_plan,
};

/// Result type for orpheus-core operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for orpheus-core
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Project error: {0}")]
    Project(String),

    #[error("Track not found: {0}")]
    TrackNotFound(uuid::Uuid),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}
