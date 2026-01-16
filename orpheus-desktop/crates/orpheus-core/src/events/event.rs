//! Application events

use crate::ProductionMode;
use std::path::PathBuf;
use uuid::Uuid;

/// Application events
#[derive(Debug, Clone)]
pub enum AppEvent {
    // === Project Events ===
    /// New project created
    ProjectCreated,
    /// Project loaded from file
    ProjectLoaded(PathBuf),
    /// Project saved to file
    ProjectSaved(PathBuf),
    /// Project modified (dirty flag set)
    ProjectModified,
    /// Project closed
    ProjectClosed,

    // === Transport Events ===
    /// Playback started
    PlaybackStarted,
    /// Playback stopped
    PlaybackStopped,
    /// Playback paused
    PlaybackPaused,
    /// Recording started
    RecordingStarted,
    /// Recording stopped
    RecordingStopped,
    /// Playhead position changed
    PositionChanged(u64),
    /// Tempo changed
    TempoChanged(f64),
    /// Loop enabled/disabled
    LoopToggled(bool),

    // === Track Events ===
    /// Track added
    TrackAdded(Uuid),
    /// Track removed
    TrackRemoved(Uuid),
    /// Track selected
    TrackSelected(Uuid),
    /// Track renamed
    TrackRenamed(Uuid, String),
    /// Track volume changed
    TrackVolumeChanged(Uuid, f32),
    /// Track muted/unmuted
    TrackMuteToggled(Uuid, bool),
    /// Track soloed/unsoloed
    TrackSoloToggled(Uuid, bool),

    // === Mode Events ===
    /// Production mode changed
    ModeChanged(ProductionMode),

    // === View Events ===
    /// Panel visibility toggled
    PanelToggled(String, bool),
    /// Zoom level changed
    ZoomChanged(f32),

    // === Audio Service Events ===
    /// Connected to audio service
    AudioServiceConnected,
    /// Disconnected from audio service
    AudioServiceDisconnected,
    /// Audio latency updated
    LatencyChanged(u32),

    // === AI Events ===
    /// AI response received
    AiResponseReceived(String),
    /// AI processing started
    AiProcessingStarted,
    /// AI processing completed
    AiProcessingCompleted,

    // === Error Events ===
    /// Error occurred
    Error(String),
    /// Warning
    Warning(String),
    /// Info message
    Info(String),
}
