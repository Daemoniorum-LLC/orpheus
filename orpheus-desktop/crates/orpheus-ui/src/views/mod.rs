//! Mode views

pub mod arrange;
pub mod compose;
pub mod record;
pub mod mix;
pub mod master;
pub mod release;
pub mod distribute;
pub mod practice;
pub mod piano_roll;
pub mod tab_editor;

// Re-exports
pub use arrange::{ArrangeView, ArrangeViewState, ArrangeViewAction, TrackDisplay, ClipDisplay, SnapMode, Selection};
pub use compose::{ComposeView, ComposeViewState, BeatInspector};
pub use mix::{MixView, MixViewState, ChannelStrip};
pub use practice::{PracticeView, PracticeViewState, PracticeSession, PracticeAction};
pub use record::{RecordView, RecordViewState, AudioInput, RecordTrack, RecordingState};
pub use master::{
    MasterView, MasterViewState, MasterProcessor, ProcessorType, ExportFormat, ExportSettings,
    CompressionMode, CompressionSettings,
};
pub use release::{
    ReleaseView, ReleaseViewState, ArtworkProject, VisualizerProject,
    ArtworkStyle, VisualizerType, SocialPlatform, MerchType,
};
pub use distribute::{
    DistributeView, DistributeViewState, ReleaseMetadata, CoverArt,
    DistributionPlatform, ReleaseChecklist, DistributeTab,
};
pub use piano_roll::{
    PianoRollView, PianoRollState, PianoRollNote, PianoRollAction,
    PianoRollTool, SnapResolution,
};
pub use tab_editor::{
    TabEditorView, TabEditorState, TabEditorAction, TabCursor, TabSelection,
    EditorMode, ActiveTool, InputResult, PendingTabAction,
    TabPlaybackEvent, TabPlaybackEventType,
};
