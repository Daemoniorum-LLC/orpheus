//! Tab Editor State
//!
//! Manages cursor position, selection, clipboard, and edit mode.

use orpheus_core::tab::{
    TabDocument, TabTrack, TabMeasure, TabBeat, TabNote, TrackMeasure,
    RhythmValue, BaseDuration, Technique, Instrument, StringedConfig,
    BendData, BendAmount, WhammyTechnique, DiveDepth, SlideDirection,
    PalmMuteIntensity, TapType,
};
use std::path::PathBuf;
use uuid::Uuid;

use super::stage_view::StageViewState;
use super::section_analysis::SectionAnalysisState;
use super::track_groups::TrackGroupsPanelState;
use crate::panels::{AnalysisPanelState, PersistedAnalysisState};
use serde::{Serialize, Deserialize};

/// Maestro file format - contains document and analysis state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaestroFile {
    /// File format version
    pub version: u32,
    /// The tab document
    pub document: TabDocument,
    /// Persisted analysis state (optional for backwards compatibility)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<PersistedAnalysisState>,
}

impl MaestroFile {
    /// Current file format version
    pub const CURRENT_VERSION: u32 = 2;

    /// Create new maestro file from document and analysis state
    pub fn new(document: TabDocument, analysis: Option<PersistedAnalysisState>) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            document,
            analysis,
        }
    }
}

/// Editor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Normal navigation mode
    Normal,
    /// Note entry mode (numbers enter frets)
    Insert,
    /// Visual selection mode
    Visual,
    /// Command mode (for complex operations)
    Command,
}

impl Default for EditorMode {
    fn default() -> Self {
        Self::Insert // Start in insert mode for quick entry
    }
}

/// Cursor position in the tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabCursor {
    /// Current track index
    pub track: usize,
    /// Current measure index
    pub measure: usize,
    /// Current beat index within measure
    pub beat: usize,
    /// Current string (1 = highest pitch)
    pub string: u8,
}

impl Default for TabCursor {
    fn default() -> Self {
        Self {
            track: 0,
            measure: 0,
            beat: 0,
            string: 1,
        }
    }
}

impl TabCursor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Move cursor down (to lower pitch string)
    pub fn down(&mut self, max_strings: u8) {
        if self.string < max_strings {
            self.string += 1;
        }
    }

    /// Move cursor up (to higher pitch string)
    pub fn up(&mut self) {
        if self.string > 1 {
            self.string -= 1;
        }
    }

    /// Move cursor right (next beat)
    pub fn right(&mut self, max_beats: usize) {
        if self.beat < max_beats.saturating_sub(1) {
            self.beat += 1;
        }
    }

    /// Move cursor left (previous beat)
    pub fn left(&mut self) {
        if self.beat > 0 {
            self.beat -= 1;
        }
    }
}

/// Selection range
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabSelection {
    /// Start position
    pub start: TabCursor,
    /// End position
    pub end: TabCursor,
}

impl TabSelection {
    pub fn new(cursor: TabCursor) -> Self {
        Self {
            start: cursor,
            end: cursor,
        }
    }

    /// Get normalized range (start before end)
    pub fn normalized(&self) -> (TabCursor, TabCursor) {
        if self.start.measure < self.end.measure
            || (self.start.measure == self.end.measure && self.start.beat <= self.end.beat)
        {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    /// Is a position within the selection?
    pub fn contains(&self, measure: usize, beat: usize, string: u8) -> bool {
        let (start, end) = self.normalized();

        if measure < start.measure || measure > end.measure {
            return false;
        }

        if measure == start.measure && beat < start.beat {
            return false;
        }

        if measure == end.measure && beat > end.beat {
            return false;
        }

        // Check string range
        let min_string = start.string.min(end.string);
        let max_string = start.string.max(end.string);
        string >= min_string && string <= max_string
    }
}

/// A group of tracks that can be collapsed/expanded
///
/// Track groups allow organizing multiple tracks into logical units
/// (e.g., "Rhythm Section", "Leads", "Orchestral") for easier navigation
/// in complex multi-track compositions.
#[derive(Debug, Clone)]
pub struct TrackGroup {
    /// Unique identifier
    pub id: Uuid,
    /// Display name (e.g., "Rhythm Section", "Leads")
    pub name: String,
    /// Group color for visual distinction
    pub color: (u8, u8, u8),
    /// Indices of tracks in this group
    pub track_indices: Vec<usize>,
    /// Whether this group is collapsed (hidden from view)
    pub collapsed: bool,
}

impl TrackGroup {
    pub fn new(name: impl Into<String>, color: (u8, u8, u8)) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            color,
            track_indices: Vec::new(),
            collapsed: false,
        }
    }

    /// Add a track to this group
    pub fn add_track(&mut self, track_idx: usize) {
        if !self.track_indices.contains(&track_idx) {
            self.track_indices.push(track_idx);
        }
    }

    /// Remove a track from this group
    pub fn remove_track(&mut self, track_idx: usize) {
        self.track_indices.retain(|&idx| idx != track_idx);
    }

    /// Toggle collapsed state
    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Check if a track is in this group
    pub fn contains_track(&self, track_idx: usize) -> bool {
        self.track_indices.contains(&track_idx)
    }
}

/// Clipboard contents
#[derive(Debug, Clone, Default)]
pub struct TabClipboard {
    /// Copied beats
    pub beats: Vec<TabBeat>,
    /// Source track info for paste matching
    pub source_strings: u8,
}

/// Fret input buffer (for multi-digit frets like 12, 24)
#[derive(Debug, Clone, Default)]
pub struct FretBuffer {
    /// Buffered digits
    pub digits: String,
    /// Time of last digit entry (for timeout)
    pub last_entry_ms: u64,
}

impl FretBuffer {
    /// Buffer timeout in milliseconds
    const TIMEOUT_MS: u64 = 500;

    pub fn new() -> Self {
        Self::default()
    }

    /// Add a digit to the buffer
    pub fn push(&mut self, digit: char, current_time_ms: u64) {
        // Clear if timed out
        if current_time_ms - self.last_entry_ms > Self::TIMEOUT_MS {
            self.digits.clear();
        }

        self.digits.push(digit);
        self.last_entry_ms = current_time_ms;
    }

    /// Get buffered fret number and clear
    pub fn take(&mut self) -> Option<u8> {
        if self.digits.is_empty() {
            return None;
        }

        let result = self.digits.parse::<u8>().ok();
        self.digits.clear();
        result
    }

    /// Peek at current value without clearing
    pub fn peek(&self) -> Option<u8> {
        if self.digits.is_empty() {
            return None;
        }
        self.digits.parse::<u8>().ok()
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.digits.clear();
    }

    /// Check if should auto-commit (single digit or timed out)
    pub fn should_commit(&self, current_time_ms: u64, max_fret: u8) -> bool {
        if self.digits.is_empty() {
            return false;
        }

        // Auto-commit if we have 2 digits
        if self.digits.len() >= 2 {
            return true;
        }

        // Auto-commit if single digit >= 3 and max fret < 30
        // (can't be first digit of valid fret)
        if self.digits.len() == 1 {
            if let Ok(d) = self.digits.parse::<u8>() {
                if d >= 3 && max_fret < 30 {
                    return true;
                }
                // Also commit 0 immediately (open string)
                if d == 0 {
                    return true;
                }
            }
        }

        // Timeout
        current_time_ms - self.last_entry_ms > Self::TIMEOUT_MS
    }
}

/// Current tool/technique being applied
#[derive(Debug, Clone)]
pub enum ActiveTool {
    /// No special tool
    None,
    /// Palm mute (next notes will have PM)
    PalmMute(PalmMuteIntensity),
    /// Let ring
    LetRing,
    /// Bend tool
    Bend(BendAmount),
    /// Slide tool
    Slide(SlideDirection),
    /// Hammer-on
    HammerOn,
    /// Pull-off
    PullOff,
    /// Tap
    Tap(TapType),
    /// Vibrato
    Vibrato,
    /// Harmonic
    Harmonic(HarmonicTool),
    /// Whammy
    Whammy(WhammyTechnique),
}

impl ActiveTool {
    /// Check if tool is "none"
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarmonicTool {
    Natural,
    Pinch,
    Artificial,
    Tap,
}

/// Tab note event for playback (matches orpheus_synth::NoteEvent structure)
#[derive(Debug, Clone)]
pub struct TabPlaybackEvent {
    /// String number (1-indexed)
    pub string: u8,
    /// Fret number
    pub fret: u8,
    /// Velocity (0.0 - 1.0)
    pub velocity: f32,
    /// Sample position when this should trigger
    pub sample_pos: u64,
    /// Event type
    pub event_type: TabPlaybackEventType,
}

/// Type of playback event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabPlaybackEventType {
    /// Normal note on
    NoteOn,
    /// Note off (mute)
    NoteOff,
    /// Hammer-on
    HammerOn,
    /// Pull-off
    PullOff,
    /// Slide
    Slide,
}

/// Pending action from tab editor (for audio preview, etc.)
#[derive(Debug, Clone)]
pub enum PendingTabAction {
    /// Preview a note (for audio feedback)
    PreviewNote {
        /// String number (1-indexed, 1 = highest pitch)
        string: u8,
        /// Fret number
        fret: u8,
        /// Velocity (0-127)
        velocity: u8,
    },
    /// Enable/disable metronome
    SetMetronome(bool),
    /// Set metronome volume
    SetMetronomeVolume(f32),
    /// Sync tempo to audio engine
    SetTempo(f64),
    /// Start playback with tab events
    StartPlayback {
        /// Events to play
        events: Vec<TabPlaybackEvent>,
        /// Tempo in BPM
        tempo: f64,
        /// Loop region (start, end) in samples, None if no loop
        loop_region: Option<(u64, u64)>,
    },
    /// Stop playback
    StopPlayback,
    /// Pause playback
    PausePlayback,
    /// Seek to position (beat number)
    SeekTo(f64),
    /// Export to PDF
    ExportPdf,
    /// Export to audio (WAV)
    ExportAudio,
}

/// Practice mode settings for the tab editor
#[derive(Debug, Clone)]
pub struct PracticeMode {
    /// Practice mode enabled
    pub enabled: bool,
    /// Loop enabled
    pub loop_enabled: bool,
    /// Loop start measure (0-indexed)
    pub loop_start: usize,
    /// Loop end measure (0-indexed, inclusive)
    pub loop_end: usize,
    /// Tempo ramp enabled
    pub tempo_ramp_enabled: bool,
    /// Starting tempo percentage (0-100)
    pub start_tempo_percent: f32,
    /// Target tempo percentage (0-100)
    pub target_tempo_percent: f32,
    /// Current tempo percentage (0-100)
    pub current_tempo_percent: f32,
    /// Tempo increment per loop iteration (percentage points)
    pub tempo_increment: f32,
    /// Count-in beats before loop starts
    pub count_in_beats: u8,
    /// Number of loop iterations completed
    pub loop_count: u32,
}

impl Default for PracticeMode {
    fn default() -> Self {
        Self {
            enabled: false,
            loop_enabled: false,
            loop_start: 0,
            loop_end: 0,
            tempo_ramp_enabled: false,
            start_tempo_percent: 50.0,
            target_tempo_percent: 100.0,
            current_tempo_percent: 100.0,
            tempo_increment: 5.0,
            count_in_beats: 4,
            loop_count: 0,
        }
    }
}

impl PracticeMode {
    /// Get effective tempo based on original tempo and current percentage
    pub fn effective_tempo(&self, original_bpm: f64) -> f64 {
        original_bpm * (self.current_tempo_percent as f64 / 100.0)
    }

    /// Advance to next tempo level (called after each loop iteration)
    /// Returns true if target tempo reached
    pub fn advance_tempo(&mut self) -> bool {
        if !self.tempo_ramp_enabled {
            return true;
        }

        self.current_tempo_percent += self.tempo_increment;
        self.loop_count += 1;

        if self.current_tempo_percent >= self.target_tempo_percent {
            self.current_tempo_percent = self.target_tempo_percent;
            true
        } else {
            false
        }
    }

    /// Reset practice mode (for starting over)
    pub fn reset(&mut self) {
        self.current_tempo_percent = if self.tempo_ramp_enabled {
            self.start_tempo_percent
        } else {
            100.0
        };
        self.loop_count = 0;
    }

    /// Set loop region from selection
    pub fn set_loop_from_selection(&mut self, start_measure: usize, end_measure: usize) {
        self.loop_start = start_measure;
        self.loop_end = end_measure;
        self.loop_enabled = true;
    }
}

/// Tab Editor State
pub struct TabEditorState {
    /// The document being edited
    pub document: TabDocument,
    /// Current file path (None if new/unsaved)
    pub file_path: Option<PathBuf>,
    /// Current cursor position
    pub cursor: TabCursor,
    /// Selection (if in visual mode)
    pub selection: Option<TabSelection>,
    /// Current editor mode
    pub mode: EditorMode,
    /// Clipboard
    pub clipboard: TabClipboard,
    /// Fret input buffer
    pub fret_buffer: FretBuffer,
    /// Current duration for new notes
    pub current_duration: RhythmValue,
    /// Current velocity for new notes
    pub current_velocity: u8,
    /// Active tool
    pub active_tool: ActiveTool,
    /// Show fret numbers (vs note names)
    pub show_frets: bool,
    /// Show all tracks stacked (multi-track view)
    pub show_all_tracks: bool,
    /// Horizontal zoom (pixels per beat)
    pub zoom_h: f32,
    /// Vertical zoom (string height)
    pub zoom_v: f32,
    /// Horizontal scroll (beat offset)
    pub scroll_x: f32,
    /// Metronome tempo
    pub tempo: f64,
    /// Metronome enabled
    pub metronome_enabled: bool,
    /// Metronome volume (0.0 - 1.0)
    pub metronome_volume: f32,
    /// Is playing
    pub is_playing: bool,
    /// Playback position (beat)
    pub playhead: f64,
    /// Is modified
    pub is_modified: bool,
    /// Undo stack
    undo_stack: Vec<UndoEntry>,
    /// Redo stack
    redo_stack: Vec<UndoEntry>,
    /// Status message
    pub status: String,
    /// Pending actions for app.rs to process (e.g., audio preview)
    pub pending_actions: Vec<PendingTabAction>,
    /// Practice mode settings
    pub practice: PracticeMode,
    /// Count-in state: is counting in before playback
    pub is_counting_in: bool,
    /// Count-in current beat (1-indexed, e.g., 1, 2, 3, 4)
    pub count_in_beat: u8,
    /// Timestamp when count-in started (for timing)
    pub count_in_start_time: Option<f64>,
    /// Show stage view for instrument arrangement
    pub show_stage_view: bool,
    /// Stage view state
    pub stage_view: StageViewState,
    /// Section analysis state
    pub section_analysis: SectionAnalysisState,
    /// Track groups for organizing tracks
    pub track_groups: Vec<TrackGroup>,
    /// Currently focused group (if Some, only show tracks in that group)
    pub focused_group: Option<usize>,
    /// Show track group panel
    pub show_track_groups: bool,
    /// Compositional analysis panel state
    pub analysis_panel: AnalysisPanelState,
    /// Show analysis panel
    pub show_analysis_panel: bool,
    /// Track groups panel state
    pub track_groups_panel: TrackGroupsPanelState,
    /// Show technique shortcuts help overlay (toggle with F1 or ?)
    pub show_technique_help: bool,
}

#[derive(Debug, Clone)]
struct UndoEntry {
    description: String,
    document_snapshot: TabDocument,
    cursor: TabCursor,
}

impl Default for TabEditorState {
    fn default() -> Self {
        Self::new()
    }
}

impl TabEditorState {
    pub fn new() -> Self {
        Self {
            document: TabDocument::new(),
            file_path: None,
            cursor: TabCursor::new(),
            selection: None,
            mode: EditorMode::Insert,
            clipboard: TabClipboard::default(),
            fret_buffer: FretBuffer::new(),
            current_duration: RhythmValue::eighth(),
            current_velocity: 100,
            active_tool: ActiveTool::None,
            show_frets: true,
            show_all_tracks: false,
            zoom_h: 40.0,
            zoom_v: 20.0,
            scroll_x: 0.0,
            tempo: 120.0,
            metronome_enabled: false,
            metronome_volume: 0.7,
            is_playing: false,
            playhead: 0.0,
            is_modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            status: "Ready".to_string(),
            pending_actions: Vec::new(),
            practice: PracticeMode::default(),
            is_counting_in: false,
            count_in_beat: 0,
            count_in_start_time: None,
            show_stage_view: false,
            stage_view: StageViewState::default(),
            section_analysis: SectionAnalysisState::new(),
            track_groups: Vec::new(),
            focused_group: None,
            show_track_groups: false,
            analysis_panel: AnalysisPanelState::new(),
            show_analysis_panel: false,
            track_groups_panel: TrackGroupsPanelState::default(),
            show_technique_help: false,
        }
    }

    /// Create with a new document with one track
    pub fn with_guitar_track() -> Self {
        let mut state = Self::new();
        let track = TabTrack::guitar("Guitar 1");
        state.document.add_track(track);
        state.document.add_measure();
        state.ensure_beats();
        state
    }

    /// Create from a TabDocument
    pub fn from_document(document: TabDocument) -> Self {
        let tempo = document.tempo_map.base_tempo;
        Self {
            document,
            file_path: None,
            cursor: TabCursor::new(),
            selection: None,
            mode: EditorMode::Normal,
            clipboard: TabClipboard::default(),
            fret_buffer: FretBuffer::new(),
            current_duration: RhythmValue::eighth(),
            current_velocity: 100,
            active_tool: ActiveTool::None,
            show_frets: true,
            show_all_tracks: false,
            zoom_h: 40.0,
            zoom_v: 20.0,
            scroll_x: 0.0,
            tempo,
            metronome_enabled: false,
            metronome_volume: 0.7,
            is_playing: false,
            playhead: 0.0,
            is_modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            status: "File loaded".to_string(),
            pending_actions: Vec::new(),
            practice: PracticeMode::default(),
            is_counting_in: false,
            count_in_beat: 0,
            count_in_start_time: None,
            show_stage_view: false,
            stage_view: StageViewState::default(),
            section_analysis: SectionAnalysisState::new(),
            track_groups: Vec::new(),
            focused_group: None,
            show_track_groups: false,
            analysis_panel: AnalysisPanelState::new(),
            show_analysis_panel: false,
            track_groups_panel: TrackGroupsPanelState::default(),
            show_technique_help: false,
        }
    }

    /// Load from a Guitar Pro file
    pub fn from_gp_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, String> {
        use orpheus_file::guitar_pro::{parse_file, convert_gp_to_tab};

        let gp_file = parse_file(path).map_err(|e| format!("Failed to parse GP file: {}", e))?;
        let document = convert_gp_to_tab(&gp_file);

        if document.tracks.is_empty() {
            return Err("GP file contains no tracks".to_string());
        }

        Ok(Self::from_document(document))
    }

    /// Ensure current measure has beats for all tracks
    pub fn ensure_beats(&mut self) {
        if self.cursor.measure >= self.document.measures.len() {
            return;
        }

        let measure = &mut self.document.measures[self.cursor.measure];

        for track in &self.document.tracks {
            if !measure.track_beats.iter().any(|tb| tb.track_id == track.id) {
                measure.track_beats.push(TrackMeasure {
                    track_id: track.id,
                    beats: vec![TabBeat::rest(self.current_duration.clone())],
                });
            }
        }
    }

    /// Get current track
    pub fn current_track(&self) -> Option<&TabTrack> {
        self.document.tracks.get(self.cursor.track)
    }

    /// Get current track's string count
    pub fn string_count(&self) -> u8 {
        self.current_track()
            .and_then(|t| match &t.instrument {
                Instrument::StringedInstrument(s) => Some(s.string_count),
                _ => None,
            })
            .unwrap_or(6)
    }

    /// Get current track's fret count
    pub fn fret_count(&self) -> u8 {
        self.current_track()
            .and_then(|t| match &t.instrument {
                Instrument::StringedInstrument(s) => Some(s.fret_count),
                _ => None,
            })
            .unwrap_or(24)
    }

    /// Get current measure
    pub fn current_measure(&self) -> Option<&TabMeasure> {
        self.document.measures.get(self.cursor.measure)
    }

    /// Get current measure mutably
    pub fn current_measure_mut(&mut self) -> Option<&mut TabMeasure> {
        self.document.measures.get_mut(self.cursor.measure)
    }

    /// Get current beat for current track
    pub fn current_beat(&self) -> Option<&TabBeat> {
        let track_id = self.current_track()?.id;
        let measure = self.current_measure()?;
        let track_beats = measure.track_beats.iter().find(|tb| tb.track_id == track_id)?;
        track_beats.beats.get(self.cursor.beat)
    }

    /// Get beats count in current measure for current track
    pub fn beats_in_measure(&self) -> usize {
        let Some(track) = self.current_track() else { return 0 };
        let Some(measure) = self.current_measure() else { return 0 };
        measure
            .track_beats
            .iter()
            .find(|tb| tb.track_id == track.id)
            .map(|tb| tb.beats.len())
            .unwrap_or(0)
    }

    /// Save undo state
    fn save_undo(&mut self, description: &str) {
        self.undo_stack.push(UndoEntry {
            description: description.to_string(),
            document_snapshot: self.document.clone(),
            cursor: self.cursor,
        });
        self.redo_stack.clear();
        self.is_modified = true;

        // Limit undo stack size
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
    }

    /// Undo last action
    pub fn undo(&mut self) {
        if let Some(entry) = self.undo_stack.pop() {
            // Save current state to redo
            self.redo_stack.push(UndoEntry {
                description: entry.description.clone(),
                document_snapshot: self.document.clone(),
                cursor: self.cursor,
            });

            self.document = entry.document_snapshot;
            self.cursor = entry.cursor;
            self.status = format!("Undo: {}", entry.description);
        }
    }

    /// Redo last undone action
    pub fn redo(&mut self) {
        if let Some(entry) = self.redo_stack.pop() {
            self.undo_stack.push(UndoEntry {
                description: entry.description.clone(),
                document_snapshot: self.document.clone(),
                cursor: self.cursor,
            });

            self.document = entry.document_snapshot;
            self.cursor = entry.cursor;
            self.status = format!("Redo: {}", entry.description);
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get undo stack count
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack count
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Enter a note at cursor position
    pub fn enter_note(&mut self, fret: u8) {
        if fret > self.fret_count() {
            self.status = format!("Fret {} exceeds max {}", fret, self.fret_count());
            return;
        }

        self.save_undo("Enter note");

        let string = self.cursor.string;
        let current_duration = self.current_duration.clone();
        let velocity = self.current_velocity;

        // Get or create the beat
        let track_id = match self.current_track() {
            Some(t) => t.id,
            None => return,
        };

        self.ensure_beats();

        if let Some(measure) = self.document.measures.get_mut(self.cursor.measure) {
            if let Some(track_beats) = measure.track_beats.iter_mut().find(|tb| tb.track_id == track_id) {
                // Ensure we have enough beats
                while track_beats.beats.len() <= self.cursor.beat {
                    track_beats.beats.push(TabBeat::rest(current_duration.clone()));
                }

                let beat = &mut track_beats.beats[self.cursor.beat];

                // Create note
                let mut note = TabNote::new(string, fret);
                note.velocity = velocity;

                // Apply active tool
                match &self.active_tool {
                    ActiveTool::None => {}
                    ActiveTool::PalmMute(_intensity) => {
                        beat.effects.arpeggio = None; // PM on beat
                        // Store in beat effects - would need to extend BeatEffect
                    }
                    ActiveTool::HammerOn => {
                        note.techniques.push(Technique::HammerOn);
                    }
                    ActiveTool::PullOff => {
                        note.techniques.push(Technique::PullOff);
                    }
                    ActiveTool::Slide(dir) => {
                        note.techniques.push(Technique::LegatoSlide(*dir));
                    }
                    ActiveTool::Bend(amount) => {
                        note.techniques.push(Technique::Bend(BendData {
                            amount: *amount,
                            curve: vec![],
                            hold: false,
                            release: false,
                        }));
                    }
                    ActiveTool::Tap(tap_type) => {
                        note.techniques.push(Technique::Tap(*tap_type));
                    }
                    ActiveTool::Harmonic(h) => {
                        match h {
                            HarmonicTool::Natural => note.techniques.push(Technique::NaturalHarmonic),
                            HarmonicTool::Pinch => note.techniques.push(Technique::PinchHarmonic),
                            HarmonicTool::Artificial => note.techniques.push(Technique::ArtificialHarmonic(12)),
                            HarmonicTool::Tap => note.techniques.push(Technique::TapHarmonic(fret + 12)),
                        }
                    }
                    ActiveTool::Vibrato => {
                        note.techniques.push(Technique::Vibrato(orpheus_core::tab::VibratoStyle::Standard));
                    }
                    ActiveTool::Whammy(whammy) => {
                        note.techniques.push(Technique::WhammyBar(whammy.clone()));
                    }
                    ActiveTool::LetRing => {
                        note.techniques.push(Technique::LetRing);
                    }
                }

                // Remove any existing note on this string
                beat.notes.retain(|n| n.string != string);
                beat.notes.push(note);
                beat.is_rest = false;
                beat.rhythm = current_duration;

                self.status = format!("Note: string {} fret {}", string, fret);
            }
        }

        // Auto-advance cursor
        self.cursor.right(self.beats_in_measure() + 1);
    }

    /// Delete note at cursor
    pub fn delete_note(&mut self) {
        self.save_undo("Delete note");

        let string = self.cursor.string;
        let track_id = match self.current_track() {
            Some(t) => t.id,
            None => return,
        };

        if let Some(measure) = self.document.measures.get_mut(self.cursor.measure) {
            if let Some(track_beats) = measure.track_beats.iter_mut().find(|tb| tb.track_id == track_id) {
                if let Some(beat) = track_beats.beats.get_mut(self.cursor.beat) {
                    beat.notes.retain(|n| n.string != string);
                    if beat.notes.is_empty() {
                        beat.is_rest = true;
                    }
                    self.status = "Note deleted".to_string();
                }
            }
        }
    }

    /// Insert a new beat at cursor
    pub fn insert_beat(&mut self) {
        self.save_undo("Insert beat");

        let track_id = match self.current_track() {
            Some(t) => t.id,
            None => return,
        };

        if let Some(measure) = self.document.measures.get_mut(self.cursor.measure) {
            if let Some(track_beats) = measure.track_beats.iter_mut().find(|tb| tb.track_id == track_id) {
                let new_beat = TabBeat::rest(self.current_duration.clone());
                if self.cursor.beat < track_beats.beats.len() {
                    track_beats.beats.insert(self.cursor.beat, new_beat);
                } else {
                    track_beats.beats.push(new_beat);
                }
                self.status = "Beat inserted".to_string();
            }
        }
    }

    /// Delete beat at cursor
    pub fn delete_beat(&mut self) {
        self.save_undo("Delete beat");

        let track_id = match self.current_track() {
            Some(t) => t.id,
            None => return,
        };

        if let Some(measure) = self.document.measures.get_mut(self.cursor.measure) {
            if let Some(track_beats) = measure.track_beats.iter_mut().find(|tb| tb.track_id == track_id) {
                if track_beats.beats.len() > 1 && self.cursor.beat < track_beats.beats.len() {
                    track_beats.beats.remove(self.cursor.beat);
                    if self.cursor.beat >= track_beats.beats.len() {
                        self.cursor.beat = track_beats.beats.len().saturating_sub(1);
                    }
                    self.status = "Beat deleted".to_string();
                }
            }
        }
    }

    /// Add a new measure
    pub fn add_measure(&mut self) {
        self.save_undo("Add measure");
        self.document.add_measure();
        self.ensure_beats();
        self.status = format!("Measure {} added", self.document.measures.len());
    }

    /// Copy selection to clipboard
    pub fn copy(&mut self) {
        let track_id = match self.current_track() {
            Some(t) => t.id,
            None => return,
        };

        // Get beats to copy based on selection or current beat
        let (start_measure, start_beat, end_measure, end_beat) = if let Some(ref sel) = self.selection {
            let (start, end) = sel.normalized();
            (start.measure, start.beat, end.measure, end.beat)
        } else {
            // Copy just the current beat
            (self.cursor.measure, self.cursor.beat, self.cursor.measure, self.cursor.beat)
        };

        let mut copied_beats = Vec::new();

        for m_idx in start_measure..=end_measure {
            if let Some(measure) = self.document.measures.get(m_idx) {
                if let Some(track_beats) = measure.track_beats.iter().find(|tb| tb.track_id == track_id) {
                    let beat_start = if m_idx == start_measure { start_beat } else { 0 };
                    let beat_end = if m_idx == end_measure {
                        end_beat + 1
                    } else {
                        track_beats.beats.len()
                    };

                    for b_idx in beat_start..beat_end.min(track_beats.beats.len()) {
                        copied_beats.push(track_beats.beats[b_idx].clone());
                    }
                }
            }
        }

        let count = copied_beats.len();
        self.clipboard = TabClipboard {
            beats: copied_beats,
            source_strings: self.string_count(),
        };
        self.status = format!("Copied {} beat(s)", count);
    }

    /// Cut selection to clipboard
    pub fn cut(&mut self) {
        self.copy();

        if self.clipboard.beats.is_empty() {
            return;
        }

        self.save_undo("Cut");

        let track_id = match self.current_track() {
            Some(t) => t.id,
            None => return,
        };

        // Delete the selection or current beat
        let (start_measure, start_beat, end_measure, end_beat) = if let Some(ref sel) = self.selection {
            let (start, end) = sel.normalized();
            (start.measure, start.beat, end.measure, end.beat)
        } else {
            (self.cursor.measure, self.cursor.beat, self.cursor.measure, self.cursor.beat)
        };

        // Delete beats in reverse order to maintain indices
        for m_idx in (start_measure..=end_measure).rev() {
            if let Some(measure) = self.document.measures.get_mut(m_idx) {
                if let Some(track_beats) = measure.track_beats.iter_mut().find(|tb| tb.track_id == track_id) {
                    let beat_start = if m_idx == start_measure { start_beat } else { 0 };
                    let beat_end = if m_idx == end_measure {
                        end_beat + 1
                    } else {
                        track_beats.beats.len()
                    };

                    // Mark beats as rests (don't remove them to preserve structure)
                    for b_idx in beat_start..beat_end.min(track_beats.beats.len()) {
                        track_beats.beats[b_idx].notes.clear();
                        track_beats.beats[b_idx].is_rest = true;
                    }
                }
            }
        }

        self.selection = None;
        self.mode = EditorMode::Normal;
        self.is_modified = true;
        self.status = "Cut".to_string();
    }

    /// Paste from clipboard
    pub fn paste(&mut self) {
        if self.clipboard.beats.is_empty() {
            self.status = "Clipboard empty".to_string();
            return;
        }

        self.save_undo("Paste");

        let track_id = match self.current_track() {
            Some(t) => t.id,
            None => return,
        };

        self.ensure_beats();

        let mut paste_measure = self.cursor.measure;
        let mut paste_beat = self.cursor.beat;

        for beat in &self.clipboard.beats {
            // Make sure measure exists
            while paste_measure >= self.document.measures.len() {
                self.document.add_measure();
            }

            if let Some(measure) = self.document.measures.get_mut(paste_measure) {
                if let Some(track_beats) = measure.track_beats.iter_mut().find(|tb| tb.track_id == track_id) {
                    // Ensure enough beats in measure
                    let current_duration = self.current_duration.clone();
                    while track_beats.beats.len() <= paste_beat {
                        track_beats.beats.push(TabBeat::rest(current_duration.clone()));
                    }

                    // Paste the beat (clone notes)
                    track_beats.beats[paste_beat] = beat.clone();
                }
            }

            // Advance to next beat
            paste_beat += 1;
            let beats_per_measure = self.time_signature_for_measure(paste_measure).numerator as usize;
            if paste_beat >= beats_per_measure {
                paste_beat = 0;
                paste_measure += 1;
            }
        }

        self.is_modified = true;
        self.status = format!("Pasted {} beat(s)", self.clipboard.beats.len());
    }

    /// Set duration for new notes
    pub fn set_duration(&mut self, base: BaseDuration) {
        self.current_duration = RhythmValue::new(base);
        self.status = format!("Duration: {}", base.name());
    }

    /// Toggle dotted duration
    pub fn toggle_dotted(&mut self) {
        if self.current_duration.dots == 0 {
            self.current_duration.dots = 1;
            self.status = "Dotted".to_string();
        } else {
            self.current_duration.dots = 0;
            self.status = "Normal".to_string();
        }
    }

    /// Toggle triplet
    pub fn toggle_triplet(&mut self) {
        if self.current_duration.tuplet.is_some() {
            self.current_duration.tuplet = None;
            self.status = "Normal".to_string();
        } else {
            self.current_duration = self.current_duration.clone().triplet();
            self.status = "Triplet".to_string();
        }
    }

    // ==================== Tempo & Time Signature ====================

    /// Set tempo (BPM)
    pub fn set_tempo(&mut self, bpm: f64) {
        let clamped = bpm.clamp(20.0, 400.0);
        self.tempo = clamped;
        self.document.tempo_map.base_tempo = clamped;
        self.is_modified = true;
        self.status = format!("Tempo: {:.0} BPM", clamped);
        // Queue action for audio engine
        self.pending_actions.push(PendingTabAction::SetTempo(clamped));
    }

    /// Toggle metronome on/off
    pub fn toggle_metronome(&mut self) {
        self.metronome_enabled = !self.metronome_enabled;
        self.status = if self.metronome_enabled {
            "Metronome ON".to_string()
        } else {
            "Metronome OFF".to_string()
        };
        self.pending_actions.push(PendingTabAction::SetMetronome(self.metronome_enabled));
    }

    /// Set metronome volume (0.0-1.0)
    pub fn set_metronome_volume(&mut self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        self.metronome_volume = clamped;
        self.pending_actions.push(PendingTabAction::SetMetronomeVolume(clamped));
    }

    // ==================== Practice Mode ====================

    /// Toggle practice mode
    pub fn toggle_practice_mode(&mut self) {
        self.practice.enabled = !self.practice.enabled;
        if self.practice.enabled {
            // Set loop region to selected measures or current measure
            if let Some(selection) = &self.selection {
                self.practice.set_loop_from_selection(selection.start.measure, selection.end.measure);
            } else {
                // Default to current measure only
                self.practice.loop_start = self.cursor.measure;
                self.practice.loop_end = self.cursor.measure;
            }
            self.practice.reset();
            self.metronome_enabled = true;
            self.pending_actions.push(PendingTabAction::SetMetronome(true));
            self.status = "Practice mode ON".to_string();
        } else {
            self.status = "Practice mode OFF".to_string();
        }
    }

    /// Quick toggle practice mode on current measure with sensible defaults
    ///
    /// Single-key shortcut (Ctrl+P) for fast workflow:
    /// - If practice mode off: enable on current measure at 75% tempo
    /// - If practice mode on: toggle off
    pub fn quick_practice_toggle(&mut self) {
        if self.practice.enabled {
            self.practice.enabled = false;
            self.practice.tempo_ramp_enabled = false;
            self.pending_actions.push(PendingTabAction::SetTempo(self.tempo));
            self.status = "Practice OFF".to_string();
        } else {
            self.practice.enabled = true;
            self.practice.loop_enabled = true;
            self.practice.loop_start = self.cursor.measure;
            self.practice.loop_end = self.cursor.measure;
            self.practice.tempo_ramp_enabled = true;
            self.practice.start_tempo_percent = 75.0;
            self.practice.target_tempo_percent = 100.0;
            self.practice.tempo_increment = 5.0;
            self.practice.reset();
            self.metronome_enabled = true;
            self.pending_actions.push(PendingTabAction::SetMetronome(true));
            let effective = self.practice.effective_tempo(self.tempo);
            self.pending_actions.push(PendingTabAction::SetTempo(effective));
            self.status = format!("Practice M{} @ 75%→100%", self.cursor.measure + 1);
        }
    }

    /// Set loop region for practice
    pub fn set_practice_loop(&mut self, start: usize, end: usize) {
        self.practice.loop_start = start.min(self.document.measures.len().saturating_sub(1));
        self.practice.loop_end = end.min(self.document.measures.len().saturating_sub(1));
        if self.practice.loop_end < self.practice.loop_start {
            std::mem::swap(&mut self.practice.loop_start, &mut self.practice.loop_end);
        }
        self.practice.loop_enabled = true;
        self.status = format!("Loop: measures {} - {}", self.practice.loop_start + 1, self.practice.loop_end + 1);
    }

    /// Set loop region from visual selection
    pub fn set_loop_from_selection(&mut self) {
        if let Some(selection) = &self.selection {
            self.set_practice_loop(selection.start.measure, selection.end.measure);
        }
    }

    /// Enable tempo ramping for practice
    pub fn enable_tempo_ramp(&mut self, start_percent: f32, target_percent: f32, increment: f32) {
        self.practice.tempo_ramp_enabled = true;
        self.practice.start_tempo_percent = start_percent.clamp(20.0, 100.0);
        self.practice.target_tempo_percent = target_percent.clamp(50.0, 150.0);
        self.practice.tempo_increment = increment.clamp(1.0, 20.0);
        self.practice.reset();

        // Apply starting tempo
        let effective_tempo = self.practice.effective_tempo(self.tempo);
        self.pending_actions.push(PendingTabAction::SetTempo(effective_tempo));
        self.status = format!("Tempo ramp: {}% → {}% (+{}%/loop)",
            self.practice.start_tempo_percent as u32,
            self.practice.target_tempo_percent as u32,
            self.practice.tempo_increment as u32);
    }

    /// Disable tempo ramping
    pub fn disable_tempo_ramp(&mut self) {
        self.practice.tempo_ramp_enabled = false;
        self.practice.current_tempo_percent = 100.0;
        // Restore original tempo
        self.pending_actions.push(PendingTabAction::SetTempo(self.tempo));
        self.status = "Tempo ramp disabled".to_string();
    }

    /// Called when loop iteration completes
    pub fn on_loop_complete(&mut self) {
        if self.practice.tempo_ramp_enabled {
            let reached_target = self.practice.advance_tempo();
            let effective_tempo = self.practice.effective_tempo(self.tempo);
            self.pending_actions.push(PendingTabAction::SetTempo(effective_tempo));

            if reached_target {
                self.status = format!("Target tempo reached! ({:.0}%)", self.practice.current_tempo_percent);
            } else {
                self.status = format!("Tempo: {:.0}% (loop #{})",
                    self.practice.current_tempo_percent,
                    self.practice.loop_count);
            }
        }
    }

    /// Get time signature for current measure
    pub fn current_time_signature(&self) -> orpheus_core::tab::TimeSignature {
        self.current_measure()
            .and_then(|m| m.time_signature)
            .unwrap_or_default()
    }

    /// Get time signature for a specific measure
    ///
    /// Returns the measure's time signature if set, otherwise inherits from previous
    /// measures or falls back to the document default (4/4).
    pub fn time_signature_for_measure(&self, measure_idx: usize) -> orpheus_core::tab::TimeSignature {
        // First check if this measure has an explicit time signature
        if let Some(measure) = self.document.measures.get(measure_idx) {
            if let Some(ts) = measure.time_signature {
                return ts;
            }
        }

        // Otherwise, look backwards for the most recent time signature
        for i in (0..measure_idx).rev() {
            if let Some(measure) = self.document.measures.get(i) {
                if let Some(ts) = measure.time_signature {
                    return ts;
                }
            }
        }

        // Fall back to default 4/4
        orpheus_core::tab::TimeSignature::default()
    }

    /// Get the document's default time signature (from first measure or 4/4)
    pub fn default_time_signature(&self) -> orpheus_core::tab::TimeSignature {
        self.document.measures.first()
            .and_then(|m| m.time_signature)
            .unwrap_or_default()
    }

    /// Get beats per measure for layout calculations
    ///
    /// Uses the document's default time signature.
    pub fn beats_per_measure(&self) -> usize {
        self.default_time_signature().numerator as usize
    }

    /// Set time signature for current measure
    pub fn set_time_signature(&mut self, numerator: u8, denominator: u8) {
        self.save_undo("Set time signature");

        if let Some(measure) = self.document.measures.get_mut(self.cursor.measure) {
            measure.time_signature = Some(orpheus_core::tab::TimeSignature::new(numerator, denominator));
        }
        self.is_modified = true;
        self.status = format!("Time signature: {}/{}", numerator, denominator);
    }

    /// Set time signature for all measures
    pub fn set_global_time_signature(&mut self, numerator: u8, denominator: u8) {
        self.save_undo("Set global time signature");

        let ts = orpheus_core::tab::TimeSignature::new(numerator, denominator);
        for measure in &mut self.document.measures {
            measure.time_signature = Some(ts);
        }
        self.is_modified = true;
        self.status = format!("Global time signature: {}/{}", numerator, denominator);
    }

    // ==================== Multi-track View ====================

    /// Toggle multi-track view (show all tracks vs single track)
    pub fn toggle_multi_track_view(&mut self) {
        self.show_all_tracks = !self.show_all_tracks;
        self.status = if self.show_all_tracks {
            "Multi-track view".to_string()
        } else {
            "Single track view".to_string()
        };
    }

    /// Switch to next track
    pub fn next_track(&mut self) {
        if self.document.tracks.is_empty() {
            return;
        }
        self.cursor.track = (self.cursor.track + 1) % self.document.tracks.len();
        self.cursor.beat = 0;
        if let Some(track) = self.current_track() {
            self.status = format!("Track: {}", track.name);
        }
    }

    /// Switch to previous track
    pub fn prev_track(&mut self) {
        if self.document.tracks.is_empty() {
            return;
        }
        if self.cursor.track == 0 {
            self.cursor.track = self.document.tracks.len() - 1;
        } else {
            self.cursor.track -= 1;
        }
        self.cursor.beat = 0;
        if let Some(track) = self.current_track() {
            self.status = format!("Track: {}", track.name);
        }
    }

    /// Select a specific track by index
    pub fn select_track(&mut self, index: usize) {
        if index < self.document.tracks.len() {
            self.cursor.track = index;
            self.cursor.beat = 0;
            if let Some(track) = self.current_track() {
                self.status = format!("Track: {}", track.name);
            }
        }
    }

    /// Get track count
    pub fn track_count(&self) -> usize {
        self.document.tracks.len()
    }

    /// Get track by index
    pub fn get_track(&self, index: usize) -> Option<&TabTrack> {
        self.document.tracks.get(index)
    }

    // ==================== Track Groups ====================

    /// Create a new track group
    pub fn create_track_group(&mut self, name: impl Into<String>, color: (u8, u8, u8)) -> usize {
        let group = TrackGroup::new(name, color);
        self.track_groups.push(group);
        let idx = self.track_groups.len() - 1;
        self.status = format!("Created group: {}", self.track_groups[idx].name);
        idx
    }

    /// Add a track to a group
    pub fn add_track_to_group(&mut self, track_idx: usize, group_idx: usize) {
        // Remove from any existing group first
        for group in &mut self.track_groups {
            group.remove_track(track_idx);
        }
        // Add to new group
        if let Some(group) = self.track_groups.get_mut(group_idx) {
            group.add_track(track_idx);
            if let Some(track) = self.document.tracks.get(track_idx) {
                self.status = format!("Added '{}' to group '{}'", track.name, group.name);
            }
        }
    }

    /// Remove a track from its group
    pub fn remove_track_from_group(&mut self, track_idx: usize) {
        for group in &mut self.track_groups {
            group.remove_track(track_idx);
        }
    }

    /// Toggle a group's collapsed state
    pub fn toggle_group_collapsed(&mut self, group_idx: usize) {
        if let Some(group) = self.track_groups.get_mut(group_idx) {
            group.toggle_collapsed();
            self.status = if group.collapsed {
                format!("Collapsed: {}", group.name)
            } else {
                format!("Expanded: {}", group.name)
            };
        }
    }

    /// Focus on a specific group (only show its tracks)
    pub fn focus_group(&mut self, group_idx: Option<usize>) {
        self.focused_group = group_idx;
        if let Some(idx) = group_idx {
            if let Some(group) = self.track_groups.get(idx) {
                self.status = format!("Focused: {}", group.name);
                // Switch to first track in group if current track not in group
                if !group.contains_track(self.cursor.track) {
                    if let Some(&first_track) = group.track_indices.first() {
                        self.cursor.track = first_track;
                    }
                }
            }
        } else {
            self.status = "Showing all tracks".to_string();
        }
    }

    /// Cycle focus to next group (or all tracks if at end)
    pub fn next_group(&mut self) {
        if self.track_groups.is_empty() {
            return;
        }
        self.focused_group = match self.focused_group {
            None => Some(0),
            Some(idx) if idx + 1 < self.track_groups.len() => Some(idx + 1),
            Some(_) => None,
        };
        self.focus_group(self.focused_group);
    }

    /// Cycle focus to previous group (or all tracks if at start)
    pub fn prev_group(&mut self) {
        if self.track_groups.is_empty() {
            return;
        }
        self.focused_group = match self.focused_group {
            None => Some(self.track_groups.len() - 1),
            Some(0) => None,
            Some(idx) => Some(idx - 1),
        };
        self.focus_group(self.focused_group);
    }

    /// Delete a track group (tracks remain, just ungrouped)
    pub fn delete_track_group(&mut self, group_idx: usize) {
        if group_idx < self.track_groups.len() {
            let name = self.track_groups[group_idx].name.clone();
            self.track_groups.remove(group_idx);
            // Update focused_group if needed
            if self.focused_group == Some(group_idx) {
                self.focused_group = None;
            } else if let Some(focused) = self.focused_group {
                if focused > group_idx {
                    self.focused_group = Some(focused - 1);
                }
            }
            self.status = format!("Deleted group: {}", name);
        }
    }

    /// Get the group a track belongs to (if any)
    pub fn get_track_group(&self, track_idx: usize) -> Option<&TrackGroup> {
        self.track_groups.iter().find(|g| g.contains_track(track_idx))
    }

    /// Check if a track should be visible based on group focus
    pub fn is_track_visible(&self, track_idx: usize) -> bool {
        match self.focused_group {
            None => {
                // No focus - check if track's group is collapsed
                if let Some(group) = self.get_track_group(track_idx) {
                    !group.collapsed
                } else {
                    true // Ungrouped tracks always visible
                }
            }
            Some(group_idx) => {
                // Focused on a group - only show tracks in that group
                self.track_groups
                    .get(group_idx)
                    .map(|g| g.contains_track(track_idx))
                    .unwrap_or(false)
            }
        }
    }

    /// Get visible tracks based on current group focus
    pub fn visible_tracks(&self) -> Vec<usize> {
        (0..self.document.tracks.len())
            .filter(|&idx| self.is_track_visible(idx))
            .collect()
    }

    /// Toggle track group panel visibility
    pub fn toggle_track_groups_panel(&mut self) {
        self.show_track_groups = !self.show_track_groups;
    }

    /// Toggle analysis panel visibility
    pub fn toggle_analysis_panel(&mut self) {
        self.show_analysis_panel = !self.show_analysis_panel;
    }

    /// Run compositional analysis on the current document
    pub fn run_analysis(&mut self) {
        self.analysis_panel.analyze(&self.document);
    }

    /// Copy measure pattern from source to target region
    pub fn copy_measure_pattern(&mut self, source_measure: usize, target_start: usize, target_end: usize) {
        // Save undo state
        self.save_undo("Copy measure pattern");

        // Get source measure data
        if source_measure >= self.document.measures.len() {
            return;
        }

        let source = self.document.measures[source_measure].clone();

        // Copy to each target measure
        for target_idx in target_start..=target_end {
            if target_idx >= self.document.measures.len() {
                break;
            }

            // Copy track beats from source, preserving the measure structure
            self.document.measures[target_idx].track_beats = source.track_beats.clone();
        }
    }

    /// Apply automatic fixes to a region based on analysis
    /// Uses deterministic improvements based on musical patterns
    pub fn auto_fix_region(&mut self, start: usize, end: usize) {
        // Save undo state
        self.save_undo("Auto-fix region");

        // Apply fixes to each measure in range
        for measure_idx in start..=end {
            if measure_idx >= self.document.measures.len() {
                break;
            }

            // For each track in the measure
            for track_measure in &mut self.document.measures[measure_idx].track_beats {
                for (beat_idx, beat) in track_measure.beats.iter_mut().enumerate() {
                    for note in &mut beat.notes {
                        // Add let ring to notes on beat 1 and 3 (common in many styles)
                        if beat_idx % 2 == 0 && note.techniques.is_empty() {
                            note.techniques.push(Technique::LetRing);
                        }

                        // Suggest position shifts for very high frets
                        if note.fret > 12 && note.string < 5 {
                            // Could play same note on higher string, lower fret
                            // This is a hint - actual fret would need calculation
                            note.fret = note.fret.saturating_sub(5);
                            note.string = note.string.saturating_add(1);
                        }
                    }
                }
            }
        }
    }

    // ==================== Section Markers ====================

    /// Add a quick section marker at the current cursor position
    ///
    /// Uses auto-incrementing names: "Section 1", "Section 2", etc.
    /// For more control, use the Section Analysis dialog.
    pub fn add_quick_section_marker(&mut self) {
        use orpheus_core::tab::SectionMarker;

        let measure = self.cursor.measure;

        // Check if there's already a marker at this measure
        if self.document.markers.iter().any(|m| m.measure == measure) {
            self.status = format!("Section marker already exists at M{}", measure + 1);
            return;
        }

        // Auto-generate name
        let count = self.document.markers.len() + 1;
        let name = format!("Section {}", count);

        self.document.markers.push(SectionMarker::new(measure, &name));
        self.document.markers.sort_by_key(|m| m.measure);
        self.is_modified = true;
        self.status = format!("Added section marker at M{}", measure + 1);
    }

    /// Remove section marker at current cursor position
    pub fn remove_section_marker(&mut self) {
        let measure = self.cursor.measure;
        let original_len = self.document.markers.len();
        self.document.markers.retain(|m| m.measure != measure);

        if self.document.markers.len() < original_len {
            self.is_modified = true;
            self.status = format!("Removed section marker at M{}", measure + 1);
        }
    }

    // ==================== Navigation ====================

    /// Move cursor up (higher pitch string)
    pub fn move_up(&mut self) {
        self.cursor.up();
    }

    /// Move cursor down (lower pitch string)
    pub fn move_down(&mut self) {
        self.cursor.down(self.string_count());
    }

    /// Move cursor left (previous beat)
    pub fn move_left(&mut self) {
        self.cursor.left();
    }

    /// Move cursor right (next beat)
    pub fn move_right(&mut self) {
        let max_beats = self.beats_in_measure();
        self.cursor.right(max_beats);
    }

    /// Move to next measure
    pub fn next_measure(&mut self) {
        if self.cursor.measure < self.document.measures.len().saturating_sub(1) {
            self.cursor.measure += 1;
            self.cursor.beat = 0;
            self.ensure_beats();
        }
    }

    /// Move to previous measure
    pub fn prev_measure(&mut self) {
        if self.cursor.measure > 0 {
            self.cursor.measure -= 1;
            self.cursor.beat = 0;
        }
    }

    /// Move to start of measure
    pub fn start_of_measure(&mut self) {
        self.cursor.beat = 0;
    }

    /// Move to end of measure
    pub fn end_of_measure(&mut self) {
        let max_beats = self.beats_in_measure();
        self.cursor.beat = max_beats.saturating_sub(1);
    }

    // ==================== Selection ====================

    /// Select all in current measure
    pub fn select_all(&mut self) {
        self.mode = EditorMode::Visual;
        self.selection = Some(TabSelection {
            start: TabCursor {
                track: self.cursor.track,
                measure: self.cursor.measure,
                beat: 0,
                string: 1,
            },
            end: TabCursor {
                track: self.cursor.track,
                measure: self.cursor.measure,
                beat: self.beats_in_measure().saturating_sub(1),
                string: self.string_count(),
            },
        });
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
        if self.mode == EditorMode::Visual {
            self.mode = EditorMode::Normal;
        }
    }

    /// Enter a dead (muted) note at cursor position
    ///
    /// Dead notes are shown as "X" on the tab and produce a muted sound.
    pub fn enter_dead_note(&mut self) {
        self.save_undo("Enter dead note");

        let string = self.cursor.string;
        let current_duration = self.current_duration.clone();
        let velocity = self.current_velocity;

        // Get or create the beat
        let track_id = match self.current_track() {
            Some(t) => t.id,
            None => return,
        };

        self.ensure_beats();

        if let Some(measure) = self.document.measures.get_mut(self.cursor.measure) {
            if let Some(track_beats) = measure.track_beats.iter_mut().find(|tb| tb.track_id == track_id) {
                // Ensure we have enough beats
                while track_beats.beats.len() <= self.cursor.beat {
                    track_beats.beats.push(TabBeat::rest(current_duration.clone()));
                }

                let beat = &mut track_beats.beats[self.cursor.beat];

                // Create dead note (fret 0, dead = true)
                let mut note = TabNote::new(string, 0).dead();
                note.velocity = velocity;

                // Remove any existing note on this string
                beat.notes.retain(|n| n.string != string);
                beat.notes.push(note);
                beat.is_rest = false;
                beat.rhythm = current_duration;

                self.status = format!("Dead note: string {}", string);
            }
        }

        // Auto-advance cursor
        self.cursor.right(self.beats_in_measure() + 1);
    }

    /// Toggle ghost note on current note at cursor position
    ///
    /// Ghost notes are shown in parentheses and played more quietly.
    /// Returns true if there was a note to toggle, false otherwise.
    pub fn toggle_ghost_note(&mut self) -> bool {
        let string = self.cursor.string;

        let track_id = match self.current_track() {
            Some(t) => t.id,
            None => return false,
        };

        // First check if there's a note at cursor (read-only)
        let has_note = self.document.measures.get(self.cursor.measure)
            .and_then(|m| m.track_beats.iter().find(|tb| tb.track_id == track_id))
            .and_then(|tb| tb.beats.get(self.cursor.beat))
            .and_then(|b| b.notes.iter().find(|n| n.string == string))
            .is_some();

        if !has_note {
            return false;
        }

        // Save undo before modification
        self.save_undo("Toggle ghost note");

        // Now modify the note
        if let Some(measure) = self.document.measures.get_mut(self.cursor.measure) {
            if let Some(track_beats) = measure.track_beats.iter_mut().find(|tb| tb.track_id == track_id) {
                if let Some(beat) = track_beats.beats.get_mut(self.cursor.beat) {
                    if let Some(note) = beat.notes.iter_mut().find(|n| n.string == string) {
                        note.ghost = !note.ghost;
                        self.is_modified = true;
                        self.status = if note.ghost {
                            format!("Ghost note ON: string {} fret {}", string, note.fret)
                        } else {
                            format!("Ghost note OFF: string {} fret {}", string, note.fret)
                        };
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Enter a rest at cursor position
    pub fn enter_rest(&mut self) {
        self.save_undo("Enter rest");

        let track_id = match self.current_track() {
            Some(t) => t.id,
            None => return,
        };

        let current_duration = self.current_duration.clone();
        self.ensure_beats();

        if let Some(measure) = self.document.measures.get_mut(self.cursor.measure) {
            if let Some(track_beats) = measure.track_beats.iter_mut().find(|tb| tb.track_id == track_id) {
                // Ensure we have enough beats
                while track_beats.beats.len() <= self.cursor.beat {
                    track_beats.beats.push(TabBeat::rest(current_duration.clone()));
                }

                let beat = &mut track_beats.beats[self.cursor.beat];
                beat.notes.clear();
                beat.is_rest = true;
                beat.rhythm = current_duration;

                self.status = "Rest entered".to_string();
            }
        }

        // Auto-advance cursor
        self.cursor.right(self.beats_in_measure() + 1);
    }

    // ==================== Note Preview ====================

    /// Get InputResult for navigation - NoteFocused if there's a note, Handled otherwise
    ///
    /// This is imported from input module, so we need to use the full path
    pub fn navigation_result(&self) -> super::input::InputResult {
        if let Some((string, fret, velocity)) = self.get_note_at_cursor() {
            super::input::InputResult::NoteFocused { string, fret, velocity }
        } else {
            super::input::InputResult::Handled
        }
    }

    /// Get note info at cursor position for audio preview
    ///
    /// Returns (string, fret, velocity) if there's a note at cursor
    pub fn get_note_at_cursor(&self) -> Option<(u8, u8, u8)> {
        let track_id = self.current_track()?.id;

        let measure = self.document.measures.get(self.cursor.measure)?;
        let track_beats = measure.track_beats.iter()
            .find(|tb| tb.track_id == track_id)?;
        let beat = track_beats.beats.get(self.cursor.beat)?;

        // Find note on current string
        let note = beat.notes.iter()
            .find(|n| n.string == self.cursor.string)?;

        Some((note.string, note.fret, note.velocity))
    }

    /// Get note info for a specific position
    pub fn get_note_at(&self, measure: usize, beat: usize, string: u8) -> Option<(u8, u8, u8)> {
        let track_id = self.current_track()?.id;

        let measure = self.document.measures.get(measure)?;
        let track_beats = measure.track_beats.iter()
            .find(|tb| tb.track_id == track_id)?;
        let beat = track_beats.beats.get(beat)?;

        // Find note on specified string
        let note = beat.notes.iter()
            .find(|n| n.string == string)?;

        Some((note.string, note.fret, note.velocity))
    }

    // ==================== File Operations ====================

    /// Save document to current file path, or return false if no path set
    pub fn save(&mut self) -> bool {
        let Some(path) = self.file_path.clone() else {
            self.status = "No file path set - use Save As".to_string();
            return false;
        };

        match self.save_to_path(&path) {
            Ok(()) => {
                self.is_modified = false;
                self.status = format!("Saved: {}", path.display());
                true
            }
            Err(e) => {
                self.status = format!("Save failed: {}", e);
                false
            }
        }
    }

    /// Save document to a specific path
    pub fn save_to_path(&self, path: &std::path::Path) -> Result<(), String> {
        // Create maestro file with analysis state
        let analysis = PersistedAnalysisState::from_panel_state(&self.analysis_panel);
        let maestro_file = MaestroFile::new(self.document.clone(), Some(analysis));

        let json = serde_json::to_string_pretty(&maestro_file)
            .map_err(|e| format!("Serialization error: {}", e))?;

        std::fs::write(path, json)
            .map_err(|e| format!("Write error: {}", e))?;

        tracing::info!("Saved tab document to: {:?}", path);
        Ok(())
    }

    /// Set file path and save
    pub fn save_as(&mut self, path: PathBuf) -> bool {
        match self.save_to_path(&path) {
            Ok(()) => {
                self.file_path = Some(path.clone());
                self.is_modified = false;
                self.status = format!("Saved: {}", path.display());
                true
            }
            Err(e) => {
                self.status = format!("Save failed: {}", e);
                false
            }
        }
    }

    /// Load document from a file path
    pub fn load_from_path(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Read error: {}", e))?;

        // Try to parse as new MaestroFile format first
        if let Ok(maestro_file) = serde_json::from_str::<MaestroFile>(&content) {
            let mut state = Self::from_document(maestro_file.document);
            state.file_path = Some(path.to_path_buf());
            state.status = format!("Loaded: {}", path.display());

            // Restore analysis state if present
            if let Some(analysis) = maestro_file.analysis {
                analysis.apply_to(&mut state.analysis_panel);
            }

            tracing::info!("Loaded maestro file (v{}) from: {:?}", maestro_file.version, path);
            return Ok(state);
        }

        // Fall back to legacy format (raw TabDocument)
        let document: TabDocument = serde_json::from_str(&content)
            .map_err(|e| format!("Parse error: {}", e))?;

        let mut state = Self::from_document(document);
        state.file_path = Some(path.to_path_buf());
        state.status = format!("Loaded (legacy): {}", path.display());

        tracing::info!("Loaded legacy tab document from: {:?}", path);
        Ok(state)
    }

    /// Check if document has unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.is_modified
    }

    /// Get display title (filename or "Untitled")
    pub fn title(&self) -> String {
        if let Some(path) = &self.file_path {
            let name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled");
            if self.is_modified {
                format!("{}*", name)
            } else {
                name.to_string()
            }
        } else if self.is_modified {
            "Untitled*".to_string()
        } else {
            "Untitled".to_string()
        }
    }

    /// Drain pending actions for processing by app.rs
    pub fn drain_actions(&mut self) -> Vec<PendingTabAction> {
        std::mem::take(&mut self.pending_actions)
    }

    /// Queue a preview note action
    pub fn queue_preview(&mut self, string: u8, fret: u8, velocity: u8) {
        self.pending_actions.push(PendingTabAction::PreviewNote {
            string,
            fret,
            velocity,
        });
    }

    /// Queue a PDF export action
    pub fn queue_export_pdf(&mut self) {
        self.pending_actions.push(PendingTabAction::ExportPdf);
    }

    /// Queue an audio export action
    pub fn queue_export_audio(&mut self) {
        self.pending_actions.push(PendingTabAction::ExportAudio);
    }

    // ==================== Playback ====================

    /// Convert document to playback events
    pub fn document_to_events(&self, sample_rate: u32) -> Vec<TabPlaybackEvent> {
        let mut events = Vec::new();

        // PPQN = 480 (ticks per quarter note/beat)
        const PPQN: f64 = 480.0;

        // Samples per tick at current tempo
        // At tempo BPM: 1 beat (quarter) = 60/BPM seconds = 60 * sample_rate / BPM samples
        // 1 beat = 480 ticks, so samples_per_tick = (60 * sample_rate / BPM) / 480
        let samples_per_tick = (60.0 * sample_rate as f64 / self.tempo) / PPQN;

        // Track current position in ticks
        let mut current_tick: u64 = 0;

        for measure in &self.document.measures {
            // Get time signature for this measure
            let time_sig = measure.time_signature.unwrap_or_default();

            // Process each track's beats in this measure
            for track_measure in &measure.track_beats {
                let mut tick_offset: u64 = 0;

                for beat in &track_measure.beats {
                    // Get rhythm duration in ticks
                    let beat_ticks = beat.rhythm.ticks();

                    // Skip rests
                    if !beat.is_rest {
                        // Convert tick position to sample position
                        let sample_pos = ((current_tick + tick_offset) as f64 * samples_per_tick) as u64;

                        // Add note events for each note in the beat
                        for note in &beat.notes {
                            // Determine event type based on techniques
                            let event_type = if note.techniques.iter().any(|t| matches!(t, Technique::HammerOn)) {
                                TabPlaybackEventType::HammerOn
                            } else if note.techniques.iter().any(|t| matches!(t, Technique::PullOff)) {
                                TabPlaybackEventType::PullOff
                            } else if note.techniques.iter().any(|t| matches!(t, Technique::LegatoSlide(_) | Technique::ShiftSlide(_))) {
                                TabPlaybackEventType::Slide
                            } else {
                                TabPlaybackEventType::NoteOn
                            };

                            events.push(TabPlaybackEvent {
                                string: note.string,
                                fret: note.fret,
                                velocity: note.velocity as f32 / 127.0,
                                sample_pos,
                                event_type,
                            });

                            // Add note off event (after note duration)
                            let note_off_sample = sample_pos + (beat_ticks as f64 * samples_per_tick) as u64;

                            // Only add note off if note doesn't have let ring
                            if !note.techniques.iter().any(|t| matches!(t, Technique::LetRing)) {
                                events.push(TabPlaybackEvent {
                                    string: note.string,
                                    fret: 0, // Not used for note off
                                    velocity: 0.0,
                                    sample_pos: note_off_sample,
                                    event_type: TabPlaybackEventType::NoteOff,
                                });
                            }
                        }
                    }

                    tick_offset += beat_ticks;
                }
            }

            // Advance to next measure (use measure_ticks from time signature)
            current_tick += time_sig.measure_ticks();
        }

        // Sort by sample position
        events.sort_by_key(|e| e.sample_pos);

        events
    }

    /// Start playback from current position
    ///
    /// If practice mode is enabled with count-in, starts count-in first.
    /// Otherwise starts playback immediately.
    pub fn start_playback(&mut self) {
        // If practice mode is enabled and count-in beats > 0, start count-in
        if self.practice.enabled && self.practice.count_in_beats > 0 {
            self.start_count_in();
        } else {
            self.start_playback_immediate();
        }
    }

    /// Start count-in before playback
    fn start_count_in(&mut self) {
        self.is_counting_in = true;
        self.count_in_beat = 1;
        self.count_in_start_time = None; // Will be set on first update
        self.status = format!("Count-in: {}/{}", self.count_in_beat, self.practice.count_in_beats);

        // Enable metronome during count-in if not already enabled
        if !self.metronome_enabled {
            self.pending_actions.push(PendingTabAction::SetMetronome(true));
        }
    }

    /// Update count-in state (called each frame)
    ///
    /// Returns true if count-in is complete and playback should start.
    pub fn update_count_in(&mut self, current_time: f64) -> bool {
        if !self.is_counting_in {
            return false;
        }

        // Initialize start time on first update
        if self.count_in_start_time.is_none() {
            self.count_in_start_time = Some(current_time);
            return false;
        }

        let start_time = self.count_in_start_time.unwrap();
        let elapsed = current_time - start_time;

        // Get effective tempo for count-in timing
        let effective_tempo = if self.practice.tempo_ramp_enabled {
            self.practice.effective_tempo(self.tempo)
        } else {
            self.tempo
        };

        // Calculate beat duration in seconds
        let beat_duration = 60.0 / effective_tempo;

        // Calculate current beat based on elapsed time
        let current_beat = (elapsed / beat_duration).floor() as u8 + 1;

        if current_beat > self.practice.count_in_beats {
            // Count-in complete
            self.is_counting_in = false;
            self.count_in_beat = 0;
            self.count_in_start_time = None;

            // Start actual playback
            self.start_playback_immediate();
            return true;
        } else if current_beat != self.count_in_beat {
            // Beat changed
            self.count_in_beat = current_beat;
            self.status = format!("Count-in: {}/{}", self.count_in_beat, self.practice.count_in_beats);
        }

        false
    }

    /// Start playback immediately (without count-in)
    fn start_playback_immediate(&mut self) {
        // Default sample rate - will be overridden by audio engine
        const SAMPLE_RATE: u32 = 44100;

        let events = self.document_to_events(SAMPLE_RATE);

        // Calculate loop region if practice mode is enabled
        let loop_region = if self.practice.enabled && self.practice.loop_enabled {
            // Get time signature and calculate measure duration
            let time_sig = self.current_time_signature();
            let measure_ticks = time_sig.measure_ticks();

            // Samples per tick at current tempo
            const PPQN: f64 = 480.0;
            let samples_per_tick = (60.0 * SAMPLE_RATE as f64 / self.tempo) / PPQN;

            let start_samples = (self.practice.loop_start as u64 * measure_ticks) as f64 * samples_per_tick;
            let end_samples = ((self.practice.loop_end as u64 + 1) * measure_ticks) as f64 * samples_per_tick;

            Some((start_samples as u64, end_samples as u64))
        } else {
            None
        };

        // Get effective tempo (with practice mode adjustment)
        let effective_tempo = if self.practice.enabled && self.practice.tempo_ramp_enabled {
            self.practice.effective_tempo(self.tempo)
        } else {
            self.tempo
        };

        self.is_playing = true;
        self.pending_actions.push(PendingTabAction::StartPlayback {
            events,
            tempo: effective_tempo,
            loop_region,
        });
        self.status = "Playback started".to_string();
    }

    /// Stop playback
    pub fn stop_playback(&mut self) {
        self.is_playing = false;
        self.playhead = 0.0;

        // Cancel count-in if active
        if self.is_counting_in {
            self.is_counting_in = false;
            self.count_in_beat = 0;
            self.count_in_start_time = None;
        }

        self.pending_actions.push(PendingTabAction::StopPlayback);
        self.status = "Playback stopped".to_string();
    }

    /// Toggle playback (play/stop)
    pub fn toggle_playback(&mut self) {
        if self.is_playing {
            self.stop_playback();
        } else {
            self.start_playback();
        }
    }

    /// Pause playback
    pub fn pause_playback(&mut self) {
        if self.is_playing {
            self.is_playing = false;
            self.pending_actions.push(PendingTabAction::PausePlayback);
            self.status = "Playback paused".to_string();
        }
    }

    /// Update playhead position from audio engine feedback
    pub fn update_playhead(&mut self, beat: f64) {
        self.playhead = beat;
    }

    /// Get measure and beat for current playhead position
    pub fn playhead_position(&self) -> (usize, usize) {
        let time_sig = self.current_time_signature();
        let beats_per_measure = time_sig.numerator as f64;

        let measure = (self.playhead / beats_per_measure).floor() as usize;
        let beat_in_measure = (self.playhead % beats_per_measure).floor() as usize;

        (measure, beat_in_measure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_movement() {
        let mut cursor = TabCursor::new();
        assert_eq!(cursor.string, 1);

        cursor.down(6);
        assert_eq!(cursor.string, 2);

        cursor.up();
        assert_eq!(cursor.string, 1);

        cursor.up(); // Should not go below 1
        assert_eq!(cursor.string, 1);
    }

    #[test]
    fn test_fret_buffer() {
        let mut buffer = FretBuffer::new();

        buffer.push('1', 0);
        assert_eq!(buffer.peek(), Some(1));

        buffer.push('2', 100);
        assert_eq!(buffer.peek(), Some(12));

        assert_eq!(buffer.take(), Some(12));
        assert!(buffer.digits.is_empty());
    }

    #[test]
    fn test_editor_state() {
        let state = TabEditorState::with_guitar_track();
        assert_eq!(state.document.tracks.len(), 1);
        assert_eq!(state.string_count(), 6);
    }

    #[test]
    fn test_time_signature_methods() {
        let mut state = TabEditorState::with_guitar_track();

        // Default should be 4/4
        assert_eq!(state.beats_per_measure(), 4);
        assert_eq!(state.default_time_signature().numerator, 4);
        assert_eq!(state.default_time_signature().denominator, 4);

        // Set time signature for first measure
        state.cursor.measure = 0;
        state.set_time_signature(3, 4); // Waltz time

        // Now default should be 3/4
        assert_eq!(state.beats_per_measure(), 3);
        assert_eq!(state.time_signature_for_measure(0).numerator, 3);

        // Add more measures and check inheritance
        state.add_measure();
        state.add_measure();

        // Measure 1 and 2 should inherit from measure 0
        assert_eq!(state.time_signature_for_measure(1).numerator, 3);
        assert_eq!(state.time_signature_for_measure(2).numerator, 3);

        // Set different time sig for measure 2
        state.cursor.measure = 2;
        state.set_time_signature(6, 8);

        // Measure 2 should now be 6/8
        assert_eq!(state.time_signature_for_measure(2).numerator, 6);
        // Measure 1 still inherits from measure 0
        assert_eq!(state.time_signature_for_measure(1).numerator, 3);
    }
}
