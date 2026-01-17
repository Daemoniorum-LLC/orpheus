//! Master mode view - mastering chain and final mixdown

use egui::{Color32, Ui, ScrollArea, Pos2, Rect, Vec2, Stroke};
use crate::theme::Theme;
use crate::widgets::{LevelMeter, Knob};
use std::path::PathBuf;

// ============================================================================
// Source Track Integration - Workflow from Mix to Master
// ============================================================================

/// A source track from the mix that feeds into the master bus
#[derive(Clone)]
pub struct SourceTrack {
    /// Track name (from mix)
    pub name: String,
    /// Track ID for linking
    pub id: u64,
    /// Track color
    pub color: Color32,
    /// Current level (dB)
    pub level_db: f32,
    /// Peak level (dB)
    pub peak_db: f32,
    /// Is muted
    pub muted: bool,
    /// Is solo'd
    pub solo: bool,
    /// Fader level (dB, 0 = unity)
    pub fader_db: f32,
    /// Pan position (-1 to 1)
    pub pan: f32,
    /// Include in stem export
    pub export_stem: bool,
}

impl SourceTrack {
    pub fn new(name: impl Into<String>, id: u64, color: Color32) -> Self {
        Self {
            name: name.into(),
            id,
            color,
            level_db: -60.0,
            peak_db: -60.0,
            muted: false,
            solo: false,
            fader_db: 0.0,
            pan: 0.0,
            export_stem: true,
        }
    }
}

/// Mix bus source configuration
#[derive(Clone)]
pub struct MixBusSource {
    /// Source tracks from the mixer
    pub tracks: Vec<SourceTrack>,
    /// Master bus level (pre-mastering)
    pub bus_level_db: f32,
    /// Master bus peak
    pub bus_peak_db: f32,
    /// Show source tracks panel
    pub show_sources: bool,
}

impl Default for MixBusSource {
    fn default() -> Self {
        // Create demo source tracks
        let tracks = vec![
            SourceTrack::new("Vocals", 1, Color32::from_rgb(231, 76, 60)),
            SourceTrack::new("Guitar", 2, Color32::from_rgb(46, 204, 113)),
            SourceTrack::new("Bass", 3, Color32::from_rgb(155, 89, 182)),
            SourceTrack::new("Drums", 4, Color32::from_rgb(52, 152, 219)),
            SourceTrack::new("Keys", 5, Color32::from_rgb(241, 196, 15)),
        ];

        Self {
            tracks,
            bus_level_db: -60.0,
            bus_peak_db: -60.0,
            show_sources: false,
        }
    }
}

// ============================================================================
// Reference Track System - A/B Comparison
// ============================================================================

/// A reference track for A/B comparison
#[derive(Clone)]
pub struct ReferenceTrack {
    /// File path
    pub path: PathBuf,
    /// Display name
    pub name: String,
    /// Duration in seconds
    pub duration_secs: f32,
    /// Sample rate
    pub sample_rate: u32,
    /// Integrated LUFS (pre-analyzed)
    pub lufs: f32,
    /// True peak (pre-analyzed)
    pub true_peak_db: f32,
    /// Gain offset for level-matched comparison (dB)
    pub gain_offset_db: f32,
    /// Is this reference currently active for A/B
    pub is_active: bool,
}

impl ReferenceTrack {
    pub fn new(path: PathBuf, name: impl Into<String>) -> Self {
        Self {
            path,
            name: name.into(),
            duration_secs: 0.0,
            sample_rate: 44100,
            lufs: -14.0,
            true_peak_db: -1.0,
            gain_offset_db: 0.0,
            is_active: false,
        }
    }
}

/// Reference track management state
#[derive(Clone, Default)]
pub struct ReferenceState {
    /// Loaded reference tracks
    pub tracks: Vec<ReferenceTrack>,
    /// Currently selected reference index
    pub selected: Option<usize>,
    /// A/B mode enabled (listening to reference)
    pub ab_active: bool,
    /// Level match references to master
    pub level_match: bool,
    /// Show reference panel
    pub show_panel: bool,
}

impl ReferenceState {
    /// Add a reference track
    pub fn add_reference(&mut self, path: PathBuf) {
        let name = path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Reference".to_string());
        let track = ReferenceTrack::new(path, name);
        self.tracks.push(track);
        if self.selected.is_none() {
            self.selected = Some(self.tracks.len() - 1);
        }
    }

    /// Remove reference at index
    pub fn remove_reference(&mut self, idx: usize) {
        if idx < self.tracks.len() {
            self.tracks.remove(idx);
            if let Some(sel) = self.selected {
                if sel >= self.tracks.len() && !self.tracks.is_empty() {
                    self.selected = Some(self.tracks.len() - 1);
                } else if self.tracks.is_empty() {
                    self.selected = None;
                }
            }
        }
    }

    /// Toggle A/B comparison
    pub fn toggle_ab(&mut self) {
        if !self.tracks.is_empty() {
            self.ab_active = !self.ab_active;
        }
    }
}

// ============================================================================
// Mastering Presets - Platform-specific Templates
// ============================================================================

/// Target platform for mastering preset
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MasteringTarget {
    /// Spotify / Apple Music / YouTube
    Streaming,
    /// CD / Lossless download
    CD,
    /// SoundCloud / Bandcamp
    Indie,
    /// Vinyl mastering
    Vinyl,
    /// Broadcast (TV/Radio)
    Broadcast,
    /// Club / DJ
    Club,
    /// Custom settings
    Custom,
}

impl MasteringTarget {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Streaming => "Streaming",
            Self::CD => "CD Quality",
            Self::Indie => "Indie Platforms",
            Self::Vinyl => "Vinyl",
            Self::Broadcast => "Broadcast",
            Self::Club => "Club / DJ",
            Self::Custom => "Custom",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Streaming => "Optimized for Spotify, Apple Music, YouTube (-14 LUFS)",
            Self::CD => "Full dynamic range for CD/lossless (-9 to -12 LUFS)",
            Self::Indie => "Balanced for SoundCloud, Bandcamp (-12 LUFS)",
            Self::Vinyl => "Low-end control, stereo considerations for vinyl cutting",
            Self::Broadcast => "Broadcast-safe levels (-24 LUFS, -2 dBTP)",
            Self::Club => "Punchy, loud for club systems (-8 to -10 LUFS)",
            Self::Custom => "Your custom mastering settings",
        }
    }

    pub fn target_lufs(&self) -> f32 {
        match self {
            Self::Streaming => -14.0,
            Self::CD => -10.0,
            Self::Indie => -12.0,
            Self::Vinyl => -12.0,
            Self::Broadcast => -24.0,
            Self::Club => -9.0,
            Self::Custom => -14.0,
        }
    }

    pub fn true_peak_ceiling(&self) -> f32 {
        match self {
            Self::Streaming => -1.0,
            Self::CD => -0.3,
            Self::Indie => -1.0,
            Self::Vinyl => -1.0,
            Self::Broadcast => -2.0,
            Self::Club => -0.3,
            Self::Custom => -1.0,
        }
    }
}

/// A saved mastering preset
#[derive(Clone)]
pub struct MasteringPreset {
    /// Preset name
    pub name: String,
    /// Target platform
    pub target: MasteringTarget,
    /// Chain configuration (processor types and params)
    pub chain: Vec<(ProcessorType, ProcessorParams)>,
    /// Target LUFS
    pub target_lufs: f32,
    /// True peak ceiling
    pub true_peak_ceiling: f32,
    /// Genre hint
    pub genre: Option<String>,
}

impl MasteringPreset {
    /// Create preset for streaming platforms
    pub fn streaming() -> Self {
        Self {
            name: "Streaming Master".to_string(),
            target: MasteringTarget::Streaming,
            chain: vec![
                (ProcessorType::Equalizer, ProcessorParams {
                    low_gain: 1.0,
                    mid_gain: 0.5,
                    high_gain: 1.5,
                    ..Default::default()
                }),
                (ProcessorType::Compressor, ProcessorParams {
                    threshold: -12.0,
                    ratio: 3.0,
                    attack: 15.0,
                    release: 150.0,
                    makeup: 2.0,
                    ..Default::default()
                }),
                (ProcessorType::StereoImager, ProcessorParams {
                    width: 1.1,
                    balance: 0.0,
                    ..Default::default()
                }),
                (ProcessorType::Limiter, ProcessorParams {
                    ceiling: -1.0,
                    lookahead: 5.0,
                    ..Default::default()
                }),
            ],
            target_lufs: -14.0,
            true_peak_ceiling: -1.0,
            genre: None,
        }
    }

    /// Create preset for CD mastering
    pub fn cd_quality() -> Self {
        Self {
            name: "CD Master".to_string(),
            target: MasteringTarget::CD,
            chain: vec![
                (ProcessorType::Equalizer, ProcessorParams::default()),
                (ProcessorType::Compressor, ProcessorParams {
                    threshold: -15.0,
                    ratio: 2.5,
                    attack: 20.0,
                    release: 200.0,
                    makeup: 1.0,
                    ..Default::default()
                }),
                (ProcessorType::Limiter, ProcessorParams {
                    ceiling: -0.3,
                    lookahead: 3.0,
                    ..Default::default()
                }),
            ],
            target_lufs: -10.0,
            true_peak_ceiling: -0.3,
            genre: None,
        }
    }

    /// Create preset for club/DJ
    pub fn club_dj() -> Self {
        Self {
            name: "Club / DJ".to_string(),
            target: MasteringTarget::Club,
            chain: vec![
                (ProcessorType::Equalizer, ProcessorParams {
                    low_gain: 2.0,
                    mid_gain: 0.0,
                    high_gain: 1.0,
                    low_freq: 80.0,
                    ..Default::default()
                }),
                (ProcessorType::Compressor, ProcessorParams {
                    threshold: -8.0,
                    ratio: 4.0,
                    attack: 5.0,
                    release: 100.0,
                    makeup: 3.0,
                    ..Default::default()
                }),
                (ProcessorType::Saturator, ProcessorParams {
                    drive: 3.0,
                    mix: 0.3,
                    ..Default::default()
                }),
                (ProcessorType::Limiter, ProcessorParams {
                    ceiling: -0.3,
                    lookahead: 2.0,
                    ..Default::default()
                }),
            ],
            target_lufs: -9.0,
            true_peak_ceiling: -0.3,
            genre: Some("Electronic / Dance".to_string()),
        }
    }

    /// Create preset for broadcast
    pub fn broadcast() -> Self {
        Self {
            name: "Broadcast".to_string(),
            target: MasteringTarget::Broadcast,
            chain: vec![
                (ProcessorType::Equalizer, ProcessorParams {
                    low_gain: -1.0,
                    mid_gain: 0.5,
                    high_gain: 0.0,
                    ..Default::default()
                }),
                (ProcessorType::Compressor, ProcessorParams {
                    threshold: -18.0,
                    ratio: 2.0,
                    attack: 30.0,
                    release: 250.0,
                    makeup: 0.0,
                    ..Default::default()
                }),
                (ProcessorType::Limiter, ProcessorParams {
                    ceiling: -2.0,
                    lookahead: 5.0,
                    ..Default::default()
                }),
            ],
            target_lufs: -24.0,
            true_peak_ceiling: -2.0,
            genre: None,
        }
    }

    /// Get all built-in presets
    pub fn builtins() -> Vec<Self> {
        vec![
            Self::streaming(),
            Self::cd_quality(),
            Self::club_dj(),
            Self::broadcast(),
        ]
    }
}

/// Preset management state
#[derive(Clone)]
pub struct PresetState {
    /// Available presets
    pub presets: Vec<MasteringPreset>,
    /// Currently applied preset index
    pub current: Option<usize>,
    /// Show preset panel
    pub show_panel: bool,
}

impl Default for PresetState {
    fn default() -> Self {
        Self {
            presets: MasteringPreset::builtins(),
            current: None,
            show_panel: false,
        }
    }
}

// ============================================================================
// Enhanced Metering - Correlation, Phase, Spectrum
// ============================================================================

/// Enhanced metering data beyond basic LUFS
#[derive(Clone)]
pub struct EnhancedMetering {
    /// Stereo correlation (-1 to 1, 1 = mono compatible)
    pub correlation: f32,
    /// Phase (degrees, 0 = in phase)
    pub phase_degrees: f32,
    /// Balance (L/R difference in dB)
    pub balance_db: f32,
    /// Spectrum bands (31-band for visualization)
    pub spectrum: [f32; 31],
    /// Mid level (dB)
    pub mid_level_db: f32,
    /// Side level (dB)
    pub side_level_db: f32,
    /// Crest factor (peak/RMS ratio)
    pub crest_factor: f32,
    /// RMS level left
    pub rms_l: f32,
    /// RMS level right
    pub rms_r: f32,
    /// K-system metering mode (K-12, K-14, K-20)
    pub k_mode: KMeteringMode,
}

impl Default for EnhancedMetering {
    fn default() -> Self {
        Self {
            correlation: 1.0,
            phase_degrees: 0.0,
            balance_db: 0.0,
            spectrum: [0.0; 31],
            mid_level_db: -60.0,
            side_level_db: -60.0,
            crest_factor: 0.0,
            rms_l: -60.0,
            rms_r: -60.0,
            k_mode: KMeteringMode::K14,
        }
    }
}

/// K-System metering mode
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KMeteringMode {
    K12,
    K14,
    K20,
}

impl KMeteringMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::K12 => "K-12 (Broadcast)",
            Self::K14 => "K-14 (Streaming)",
            Self::K20 => "K-20 (Film/Classical)",
        }
    }

    pub fn headroom(&self) -> f32 {
        match self {
            Self::K12 => 12.0,
            Self::K14 => 14.0,
            Self::K20 => 20.0,
        }
    }
}

// ============================================================================
// Stem Export - Individual Track Export
// ============================================================================

/// Stem export configuration
#[derive(Clone)]
pub struct StemExportConfig {
    /// Export individual stems
    pub export_stems: bool,
    /// Export master alongside stems
    pub export_master: bool,
    /// Create stems subfolder
    pub create_subfolder: bool,
    /// Stem naming pattern
    pub naming_pattern: StemNamingPattern,
    /// Apply master processing to stems
    pub process_stems: bool,
}

impl Default for StemExportConfig {
    fn default() -> Self {
        Self {
            export_stems: false,
            export_master: true,
            create_subfolder: true,
            naming_pattern: StemNamingPattern::TrackNumber,
            process_stems: false,
        }
    }
}

/// Stem file naming pattern
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StemNamingPattern {
    /// 01_TrackName.wav
    TrackNumber,
    /// TrackName.wav
    TrackName,
    /// ProjectName_TrackName.wav
    ProjectTrack,
}

impl StemNamingPattern {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TrackNumber => "01_TrackName",
            Self::TrackName => "TrackName",
            Self::ProjectTrack => "Project_TrackName",
        }
    }
}

// ============================================================================
// Auto Gain Staging - Level Analysis & Adjustment
// ============================================================================

/// Auto gain staging analysis and control
#[derive(Clone)]
pub struct AutoGainStaging {
    /// Analysis complete
    pub analyzed: bool,
    /// Recommended input gain adjustment
    pub recommended_gain_db: f32,
    /// Pre-master peak level
    pub pre_master_peak: f32,
    /// Pre-master RMS level
    pub pre_master_rms: f32,
    /// Pre-master LUFS
    pub pre_master_lufs: f32,
    /// Headroom after staging
    pub staged_headroom: f32,
    /// Auto-apply staging
    pub auto_apply: bool,
    /// Current staging gain (applied)
    pub applied_gain_db: f32,
}

impl Default for AutoGainStaging {
    fn default() -> Self {
        Self {
            analyzed: false,
            recommended_gain_db: 0.0,
            pre_master_peak: -60.0,
            pre_master_rms: -60.0,
            pre_master_lufs: -60.0,
            staged_headroom: 18.0,
            auto_apply: false,
            applied_gain_db: 0.0,
        }
    }
}

impl AutoGainStaging {
    /// Analyze input and calculate recommended gain
    pub fn analyze(&mut self, peak_db: f32, rms_db: f32, lufs: f32) {
        self.pre_master_peak = peak_db;
        self.pre_master_rms = rms_db;
        self.pre_master_lufs = lufs;

        // Target: -18 dBFS RMS for optimal headroom before mastering
        let target_rms = -18.0;
        self.recommended_gain_db = target_rms - rms_db;

        // Clamp to prevent clipping
        let max_gain = -peak_db - 3.0; // Leave 3dB headroom
        self.recommended_gain_db = self.recommended_gain_db.min(max_gain);

        self.staged_headroom = -peak_db - self.recommended_gain_db;
        self.analyzed = true;
    }

    /// Apply recommended gain
    pub fn apply(&mut self) {
        self.applied_gain_db = self.recommended_gain_db;
    }

    /// Reset staging
    pub fn reset(&mut self) {
        self.applied_gain_db = 0.0;
    }
}

// ============================================================================
// Batch Export - Multiple Format Export
// ============================================================================

/// Batch export job
#[derive(Clone)]
pub struct BatchExportJob {
    /// Format
    pub format: ExportFormat,
    /// Sample rate
    pub sample_rate: u32,
    /// Output filename suffix
    pub suffix: String,
    /// Include in batch
    pub enabled: bool,
    /// Progress (0-1)
    pub progress: f32,
    /// Status
    pub status: BatchJobStatus,
}

impl BatchExportJob {
    pub fn new(format: ExportFormat, sample_rate: u32, suffix: impl Into<String>) -> Self {
        Self {
            format,
            sample_rate,
            suffix: suffix.into(),
            enabled: true,
            progress: 0.0,
            status: BatchJobStatus::Pending,
        }
    }
}

/// Batch job status
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BatchJobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Batch export configuration
#[derive(Clone)]
pub struct BatchExportConfig {
    /// Export jobs
    pub jobs: Vec<BatchExportJob>,
    /// Output directory
    pub output_dir: Option<PathBuf>,
    /// Base filename
    pub base_name: String,
    /// Is batch export in progress
    pub is_running: bool,
    /// Current job index
    pub current_job: usize,
    /// Show batch panel
    pub show_panel: bool,
}

impl Default for BatchExportConfig {
    fn default() -> Self {
        // Default batch jobs for common scenarios
        let jobs = vec![
            BatchExportJob::new(ExportFormat::Wav24, 44100, "master"),
            BatchExportJob::new(ExportFormat::Mp3_320, 44100, "mp3"),
            BatchExportJob::new(ExportFormat::Flac, 44100, "flac"),
        ];

        Self {
            jobs,
            output_dir: None,
            base_name: "master".to_string(),
            is_running: false,
            current_job: 0,
            show_panel: false,
        }
    }
}

impl BatchExportConfig {
    /// Add common streaming package
    pub fn add_streaming_package(&mut self) {
        self.jobs.push(BatchExportJob::new(ExportFormat::Wav24, 44100, "master_44k"));
        self.jobs.push(BatchExportJob::new(ExportFormat::Wav24, 48000, "master_48k"));
        self.jobs.push(BatchExportJob::new(ExportFormat::Mp3_320, 44100, "streaming"));
    }

    /// Get enabled jobs count
    pub fn enabled_count(&self) -> usize {
        self.jobs.iter().filter(|j| j.enabled).count()
    }

    /// Get total progress
    pub fn total_progress(&self) -> f32 {
        let enabled: Vec<_> = self.jobs.iter().filter(|j| j.enabled).collect();
        if enabled.is_empty() {
            return 0.0;
        }
        enabled.iter().map(|j| j.progress).sum::<f32>() / enabled.len() as f32
    }
}

/// Mastering processor type
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcessorType {
    Equalizer,
    Compressor,
    Limiter,
    StereoImager,
    Saturator,
}

impl ProcessorType {
    pub fn name(&self) -> &'static str {
        match self {
            ProcessorType::Equalizer => "EQ",
            ProcessorType::Compressor => "Compressor",
            ProcessorType::Limiter => "Limiter",
            ProcessorType::StereoImager => "Stereo Imager",
            ProcessorType::Saturator => "Saturator",
        }
    }
}

/// A processor in the mastering chain
#[derive(Clone)]
pub struct MasterProcessor {
    /// Processor type
    pub processor_type: ProcessorType,
    /// Is enabled
    pub enabled: bool,
    /// Bypass
    pub bypassed: bool,
    /// Processor parameters (generic for now)
    pub params: ProcessorParams,
}

/// Generic processor parameters
#[derive(Clone)]
pub struct ProcessorParams {
    // EQ params
    pub low_gain: f32,
    pub mid_gain: f32,
    pub high_gain: f32,
    pub low_freq: f32,
    pub high_freq: f32,

    // Compressor params
    pub threshold: f32,
    pub ratio: f32,
    pub attack: f32,
    pub release: f32,
    pub makeup: f32,

    // Limiter params
    pub ceiling: f32,
    pub lookahead: f32,

    // Stereo params
    pub width: f32,
    pub balance: f32,

    // Saturator params
    pub drive: f32,
    pub mix: f32,
}

impl Default for ProcessorParams {
    fn default() -> Self {
        Self {
            low_gain: 0.0,
            mid_gain: 0.0,
            high_gain: 0.0,
            low_freq: 200.0,
            high_freq: 4000.0,
            threshold: -10.0,
            ratio: 4.0,
            attack: 10.0,
            release: 100.0,
            makeup: 0.0,
            ceiling: -0.3,
            lookahead: 5.0,
            width: 1.0,
            balance: 0.0,
            drive: 0.0,
            mix: 1.0,
        }
    }
}

impl MasterProcessor {
    pub fn new(processor_type: ProcessorType) -> Self {
        Self {
            processor_type,
            enabled: true,
            bypassed: false,
            params: ProcessorParams::default(),
        }
    }
}

/// Loudness metering values
#[derive(Clone, Default)]
pub struct LoudnessMetering {
    /// Integrated LUFS
    pub integrated_lufs: f32,
    /// Short-term LUFS (3 sec)
    pub short_term_lufs: f32,
    /// Momentary LUFS (400ms)
    pub momentary_lufs: f32,
    /// True peak left
    pub true_peak_l: f32,
    /// True peak right
    pub true_peak_r: f32,
    /// Loudness range (LRA)
    pub loudness_range: f32,
}

/// Export format
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Wav16,
    Wav24,
    Wav32Float,
    Flac,
    Mp3_320,
    Mp3V0,
    Ogg,
}

impl ExportFormat {
    pub fn name(&self) -> &'static str {
        match self {
            ExportFormat::Wav16 => "WAV 16-bit",
            ExportFormat::Wav24 => "WAV 24-bit",
            ExportFormat::Wav32Float => "WAV 32-bit Float",
            ExportFormat::Flac => "FLAC",
            ExportFormat::Mp3_320 => "MP3 320kbps",
            ExportFormat::Mp3V0 => "MP3 V0 VBR",
            ExportFormat::Ogg => "OGG Vorbis",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Wav16 | ExportFormat::Wav24 | ExportFormat::Wav32Float => "wav",
            ExportFormat::Flac => "flac",
            ExportFormat::Mp3_320 | ExportFormat::Mp3V0 => "mp3",
            ExportFormat::Ogg => "ogg",
        }
    }

    /// Returns true if this format supports zstd compression
    pub fn supports_compression(&self) -> bool {
        matches!(self, ExportFormat::Wav16 | ExportFormat::Wav24 | ExportFormat::Wav32Float | ExportFormat::Flac)
    }

    pub fn description(&self) -> &'static str {
        match self {
            ExportFormat::Wav16 => "Lossless, CD quality (16-bit)",
            ExportFormat::Wav24 => "Lossless, studio quality (24-bit)",
            ExportFormat::Wav32Float => "Lossless, highest precision (32-bit float)",
            ExportFormat::Flac => "Lossless compression, smaller files",
            ExportFormat::Mp3_320 => "Lossy, highest MP3 quality (320 kbps CBR)",
            ExportFormat::Mp3V0 => "Lossy, variable bitrate (~245 kbps avg)",
            ExportFormat::Ogg => "Lossy, open format, good quality/size ratio",
        }
    }
}

/// Compression mode for exported audio files
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum CompressionMode {
    /// No compression (raw format)
    #[default]
    None,
    /// Standard zstd compression
    Standard,
    /// Dictionary-based compression (better ratio)
    Dictionary,
}

impl CompressionMode {
    pub fn name(&self) -> &'static str {
        match self {
            CompressionMode::None => "None",
            CompressionMode::Standard => "Standard (zstd)",
            CompressionMode::Dictionary => "Dictionary (zstd)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CompressionMode::None => "No compression, larger file size",
            CompressionMode::Standard => "Standard zstd compression, ~60-70% reduction",
            CompressionMode::Dictionary => "Dictionary compression, ~70-80% reduction",
        }
    }
}

/// Compression settings for export
#[derive(Clone)]
pub struct CompressionSettings {
    /// Compression mode
    pub mode: CompressionMode,
    /// Compression level (1-22, higher = better ratio, slower)
    pub level: i32,
    /// Path to custom dictionary (if using Dictionary mode)
    pub dictionary_path: Option<PathBuf>,
    /// Use built-in dictionary when available
    pub use_builtin_dictionary: bool,
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            mode: CompressionMode::None,
            level: 3, // Default zstd level, good balance of speed/ratio
            dictionary_path: None,
            use_builtin_dictionary: true,
        }
    }
}

impl CompressionSettings {
    /// Get the effective file extension based on compression mode
    pub fn extension_suffix(&self) -> &'static str {
        match self.mode {
            CompressionMode::None => "",
            CompressionMode::Standard | CompressionMode::Dictionary => ".zst",
        }
    }
}

/// Export settings
#[derive(Clone)]
pub struct ExportSettings {
    /// Output format
    pub format: ExportFormat,
    /// Sample rate
    pub sample_rate: u32,
    /// Apply dither (for bit depth reduction)
    pub dither: bool,
    /// Normalize to target LUFS
    pub normalize: bool,
    /// Target LUFS
    pub target_lufs: f32,
    /// Limit to true peak
    pub limit_true_peak: bool,
    /// True peak ceiling
    pub true_peak_ceiling: f32,
    /// Compression settings
    pub compression: CompressionSettings,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: ExportFormat::Wav24,
            sample_rate: 44100,
            dither: true,
            normalize: false,
            target_lufs: -14.0,
            limit_true_peak: true,
            true_peak_ceiling: -1.0,
            compression: CompressionSettings::default(),
        }
    }
}

impl ExportSettings {
    /// Get the full output filename with compression extension if applicable
    pub fn output_filename(&self, base_name: &str) -> String {
        let ext = self.format.extension();
        let suffix = self.compression.extension_suffix();
        format!("{}.{}{}", base_name, ext, suffix)
    }
}

/// State for the master view
pub struct MasterViewState {
    /// Mastering chain
    pub chain: Vec<MasterProcessor>,
    /// Selected processor index
    pub selected_processor: Option<usize>,
    /// Loudness metering
    pub metering: LoudnessMetering,
    /// Current level dB (left)
    pub level_l: f32,
    /// Current level dB (right)
    pub level_r: f32,
    /// Peak level (left)
    pub peak_l: f32,
    /// Peak level (right)
    pub peak_r: f32,
    /// Is playing
    pub is_playing: bool,
    /// Playback position (0-1)
    pub position: f32,
    /// Total duration
    pub duration_secs: f32,
    /// Export settings
    pub export_settings: ExportSettings,
    /// Show export panel
    pub show_export: bool,
    /// Show chain panel
    pub show_chain: bool,

    // === New workflow integration fields ===
    /// Mix bus source (tracks from mixer)
    pub mix_source: MixBusSource,
    /// Reference tracks for A/B comparison
    pub reference: ReferenceState,
    /// Mastering presets
    pub presets: PresetState,
    /// Enhanced metering (correlation, phase, spectrum)
    pub enhanced_metering: EnhancedMetering,
    /// Stem export configuration
    pub stem_config: StemExportConfig,
    /// Auto gain staging
    pub gain_staging: AutoGainStaging,
    /// Batch export configuration
    pub batch_export: BatchExportConfig,
    /// Show enhanced metering panel
    pub show_enhanced_metering: bool,
    /// Currently active panel tab
    pub active_panel: MasterPanelTab,
}

/// Active panel tab in master view
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MasterPanelTab {
    Chain,
    Sources,
    Reference,
    Presets,
}

impl Default for MasterViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl MasterViewState {
    pub fn new() -> Self {
        // Default mastering chain
        let chain = vec![
            MasterProcessor::new(ProcessorType::Equalizer),
            MasterProcessor::new(ProcessorType::Compressor),
            MasterProcessor::new(ProcessorType::StereoImager),
            MasterProcessor::new(ProcessorType::Limiter),
        ];

        Self {
            chain,
            selected_processor: Some(0),
            metering: LoudnessMetering::default(),
            level_l: -60.0,
            level_r: -60.0,
            peak_l: -60.0,
            peak_r: -60.0,
            is_playing: false,
            position: 0.0,
            duration_secs: 180.0, // 3 minutes default
            export_settings: ExportSettings::default(),
            show_export: false,
            show_chain: true,
            // New workflow fields
            mix_source: MixBusSource::default(),
            reference: ReferenceState::default(),
            presets: PresetState::default(),
            enhanced_metering: EnhancedMetering::default(),
            stem_config: StemExportConfig::default(),
            gain_staging: AutoGainStaging::default(),
            batch_export: BatchExportConfig::default(),
            show_enhanced_metering: false,
            active_panel: MasterPanelTab::Chain,
        }
    }

    /// Apply a mastering preset
    pub fn apply_preset(&mut self, preset: &MasteringPreset) {
        // Clear existing chain
        self.chain.clear();

        // Apply preset chain
        for (ptype, params) in &preset.chain {
            let mut processor = MasterProcessor::new(*ptype);
            processor.params = params.clone();
            self.chain.push(processor);
        }

        // Update export settings
        self.export_settings.target_lufs = preset.target_lufs;
        self.export_settings.true_peak_ceiling = preset.true_peak_ceiling;
        self.export_settings.normalize = true;
        self.export_settings.limit_true_peak = true;

        // Select first processor
        self.selected_processor = if self.chain.is_empty() { None } else { Some(0) };
    }

    /// Run auto gain staging analysis
    pub fn analyze_gain_staging(&mut self) {
        // Simulated analysis - in real implementation would analyze actual audio
        let peak = self.peak_l.max(self.peak_r);
        let rms = (self.level_l + self.level_r) / 2.0 - 3.0; // Approximate RMS
        let lufs = self.metering.integrated_lufs;
        self.gain_staging.analyze(peak, rms, lufs);
    }

    /// Toggle reference A/B comparison
    pub fn toggle_reference_ab(&mut self) {
        self.reference.toggle_ab();
    }

    /// Update simulated levels and metering
    pub fn update_levels(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as f32 / 1000.0)
            .unwrap_or(0.0);

        if self.is_playing {
            // Simulated levels with some variation
            let base = -12.0;
            self.level_l = (base + (t * 2.3).sin() * 6.0 + (t * 5.1).sin() * 3.0).clamp(-60.0, 0.0);
            self.level_r = (base + (t * 2.1).sin() * 6.0 + (t * 4.9).sin() * 3.0).clamp(-60.0, 0.0);

            self.peak_l = self.peak_l.max(self.level_l) - 0.05;
            self.peak_r = self.peak_r.max(self.level_r) - 0.05;

            // Simulated LUFS
            self.metering.momentary_lufs = -14.0 + (t * 0.7).sin() * 4.0;
            self.metering.short_term_lufs = -14.0 + (t * 0.2).sin() * 2.0;
            self.metering.integrated_lufs = -14.2;
            self.metering.loudness_range = 6.5;
            self.metering.true_peak_l = self.level_l + 0.5;
            self.metering.true_peak_r = self.level_r + 0.5;

            // === Enhanced metering simulation ===
            // Correlation (simulated - real would use sample-by-sample correlation)
            self.enhanced_metering.correlation = 0.85 + (t * 0.3).sin() * 0.1;

            // Phase (simulated)
            self.enhanced_metering.phase_degrees = (t * 0.5).sin() * 15.0;

            // Balance
            self.enhanced_metering.balance_db = self.level_l - self.level_r;

            // Mid/Side levels
            let mid = (self.level_l + self.level_r) / 2.0;
            let side = (self.level_l - self.level_r).abs() / 2.0 - 6.0;
            self.enhanced_metering.mid_level_db = mid;
            self.enhanced_metering.side_level_db = side.max(-60.0);

            // RMS (simulated - about 3-6dB below peak)
            self.enhanced_metering.rms_l = self.level_l - 4.0 - (t * 1.5).sin().abs() * 2.0;
            self.enhanced_metering.rms_r = self.level_r - 4.0 - (t * 1.7).sin().abs() * 2.0;

            // Crest factor (peak / RMS)
            self.enhanced_metering.crest_factor =
                (self.peak_l.max(self.peak_r) - (self.enhanced_metering.rms_l + self.enhanced_metering.rms_r) / 2.0).max(0.0);

            // Spectrum (31 bands simulated)
            for i in 0..31 {
                let freq_factor = (i as f32 / 31.0) * 4.0;
                let band_base = -24.0 - (freq_factor - 1.5).abs() * 3.0; // Peak around 1.5 (mid frequencies)
                let variation = (t * (0.5 + i as f32 * 0.1)).sin() * 6.0;
                self.enhanced_metering.spectrum[i] = (band_base + variation).clamp(-60.0, 0.0);
            }

            // === Source track levels simulation ===
            for (i, track) in self.mix_source.tracks.iter_mut().enumerate() {
                if !track.muted {
                    let track_base = -18.0 + (i as f32 * 2.0);
                    let track_var = (t * (1.0 + i as f32 * 0.2)).sin() * 8.0;
                    track.level_db = (track_base + track_var + track.fader_db).clamp(-60.0, 0.0);
                    track.peak_db = track.peak_db.max(track.level_db) - 0.05;
                } else {
                    track.level_db = (track.level_db - 2.0).max(-60.0);
                    track.peak_db = (track.peak_db - 0.3).max(-60.0);
                }
            }

            // Mix bus level (sum of all tracks)
            let active_tracks: Vec<_> = self.mix_source.tracks.iter()
                .filter(|t| !t.muted)
                .collect();
            if !active_tracks.is_empty() {
                let sum_linear: f32 = active_tracks.iter()
                    .map(|t| 10.0f32.powf(t.level_db / 20.0))
                    .sum();
                self.mix_source.bus_level_db = 20.0 * sum_linear.log10();
                self.mix_source.bus_peak_db = self.mix_source.bus_peak_db.max(self.mix_source.bus_level_db) - 0.05;
            }
        } else {
            self.level_l = (self.level_l - 1.0).max(-60.0);
            self.level_r = (self.level_r - 1.0).max(-60.0);
            self.peak_l = (self.peak_l - 0.2).max(-60.0);
            self.peak_r = (self.peak_r - 0.2).max(-60.0);

            // Decay source tracks too
            for track in &mut self.mix_source.tracks {
                track.level_db = (track.level_db - 1.0).max(-60.0);
                track.peak_db = (track.peak_db - 0.2).max(-60.0);
            }
            self.mix_source.bus_level_db = (self.mix_source.bus_level_db - 1.0).max(-60.0);
            self.mix_source.bus_peak_db = (self.mix_source.bus_peak_db - 0.2).max(-60.0);
        }
    }

    /// Format position as time string
    pub fn format_position(&self) -> String {
        let secs = self.position * self.duration_secs;
        let mins = (secs / 60.0) as u32;
        let secs = secs % 60.0;
        format!("{}:{:05.2}", mins, secs)
    }

    /// Format duration
    pub fn format_duration(&self) -> String {
        let mins = (self.duration_secs / 60.0) as u32;
        let secs = self.duration_secs % 60.0;
        format!("{}:{:05.2}", mins, secs)
    }

    /// Toggle playback
    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
    }

    /// Reset peaks
    pub fn reset_peaks(&mut self) {
        self.peak_l = -60.0;
        self.peak_r = -60.0;
    }

    /// Add processor to chain
    pub fn add_processor(&mut self, processor_type: ProcessorType) {
        self.chain.push(MasterProcessor::new(processor_type));
        self.selected_processor = Some(self.chain.len() - 1);
    }

    /// Remove processor at index
    pub fn remove_processor(&mut self, index: usize) {
        if index < self.chain.len() {
            self.chain.remove(index);
            if let Some(selected) = self.selected_processor {
                if selected >= self.chain.len() && !self.chain.is_empty() {
                    self.selected_processor = Some(self.chain.len() - 1);
                } else if self.chain.is_empty() {
                    self.selected_processor = None;
                }
            }
        }
    }
}

/// Master mode view component
pub struct MasterView<'a> {
    state: &'a mut MasterViewState,
    theme: &'a Theme,
}

impl<'a> MasterView<'a> {
    pub fn new(state: &'a mut MasterViewState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        // Update levels
        self.state.update_levels();
        if self.state.is_playing {
            ui.ctx().request_repaint();
        }

        // Handle keyboard shortcuts
        self.handle_keyboard(ui);

        // Main layout
        ui.vertical(|ui| {
            // Transport bar with new features
            self.show_transport_bar(ui);

            ui.separator();

            // A/B reference indicator
            if self.state.reference.ab_active {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("▶ Listening to Reference")
                        .color(Color32::from_rgb(241, 196, 15))
                        .strong());
                    if let Some(idx) = self.state.reference.selected {
                        if let Some(track) = self.state.reference.tracks.get(idx) {
                            ui.label(egui::RichText::new(format!("({})", track.name))
                                .color(self.theme.palette.text_secondary));
                        }
                    }
                    if ui.small_button("Back to Master (B)").clicked() {
                        self.state.reference.ab_active = false;
                    }
                });
                ui.separator();
            }

            // Main content
            ui.horizontal(|ui| {
                // Left panel with tabs
                if self.state.show_chain || self.state.mix_source.show_sources ||
                   self.state.reference.show_panel || self.state.presets.show_panel
                {
                    ui.allocate_ui_with_layout(
                        Vec2::new(220.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            // Tab bar for left panel
                            ui.horizontal(|ui| {
                                if ui.selectable_label(self.state.active_panel == MasterPanelTab::Chain, "Chain")
                                    .on_hover_text("Mastering chain (Tab)")
                                    .clicked()
                                {
                                    self.state.active_panel = MasterPanelTab::Chain;
                                }
                                if ui.selectable_label(self.state.active_panel == MasterPanelTab::Sources, "Sources")
                                    .on_hover_text("Source tracks (S)")
                                    .clicked()
                                {
                                    self.state.active_panel = MasterPanelTab::Sources;
                                }
                                if ui.selectable_label(self.state.active_panel == MasterPanelTab::Reference, "Ref")
                                    .on_hover_text("Reference tracks (R)")
                                    .clicked()
                                {
                                    self.state.active_panel = MasterPanelTab::Reference;
                                }
                                if ui.selectable_label(self.state.active_panel == MasterPanelTab::Presets, "Presets")
                                    .on_hover_text("Mastering presets (P)")
                                    .clicked()
                                {
                                    self.state.active_panel = MasterPanelTab::Presets;
                                }
                            });
                            ui.separator();

                            // Show active panel content
                            ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    match self.state.active_panel {
                                        MasterPanelTab::Chain => self.show_chain_panel(ui),
                                        MasterPanelTab::Sources => self.show_source_panel(ui),
                                        MasterPanelTab::Reference => self.show_reference_panel(ui),
                                        MasterPanelTab::Presets => self.show_preset_panel(ui),
                                    }
                                });
                        },
                    );
                    ui.separator();
                }

                // Center area - processor details and waveform
                ui.vertical(|ui| {
                    // Processor detail panel
                    self.show_processor_detail(ui);

                    ui.separator();

                    // Waveform overview
                    self.show_waveform_overview(ui);
                });

                ui.separator();

                // Right side - metering panels
                ui.vertical(|ui| {
                    // Basic metering
                    ui.allocate_ui_with_layout(
                        Vec2::new(180.0, if self.state.show_enhanced_metering { ui.available_height() / 2.0 } else { ui.available_height() }),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            self.show_metering_panel(ui);
                        },
                    );

                    // Enhanced metering (if visible)
                    if self.state.show_enhanced_metering {
                        ui.separator();
                        ui.allocate_ui_with_layout(
                            Vec2::new(180.0, ui.available_height()),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                self.show_enhanced_metering_panel(ui);
                            },
                        );
                    }
                });
            });

            // Export panel (bottom, if visible)
            if self.state.show_export {
                ui.separator();
                self.show_export_panel(ui);
            }

            // Batch export panel (if visible)
            if self.state.batch_export.show_panel {
                ui.separator();
                self.show_batch_export_panel(ui);
            }

            // Status bar with keyboard hints
            ui.separator();
            ui.horizontal(|ui| {
                // Current preset info
                if let Some(idx) = self.state.presets.current {
                    if let Some(preset) = self.state.presets.presets.get(idx) {
                        ui.label(egui::RichText::new(format!("Preset: {}", preset.name))
                            .small()
                            .color(self.theme.palette.text_secondary));
                        ui.separator();
                    }
                }

                // Keyboard shortcuts hint
                ui.label(egui::RichText::new("B: A/B  M: Meters  E: Export  P: Presets  R: Refs  S: Sources")
                    .small()
                    .color(self.theme.palette.text_secondary));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Batch export button
                    let batch_color = if self.state.batch_export.show_panel {
                        self.theme.palette.accent
                    } else {
                        self.theme.palette.text_secondary
                    };
                    if ui.add(egui::Button::new(egui::RichText::new("Batch").color(batch_color).small())
                        .min_size(Vec2::new(50.0, 20.0)))
                        .on_hover_text("Batch export to multiple formats")
                        .clicked()
                    {
                        self.state.batch_export.show_panel = !self.state.batch_export.show_panel;
                    }

                    // Enhanced metering toggle
                    let meter_color = if self.state.show_enhanced_metering {
                        self.theme.palette.accent
                    } else {
                        self.theme.palette.text_secondary
                    };
                    if ui.add(egui::Button::new(egui::RichText::new("📊").color(meter_color))
                        .min_size(Vec2::new(24.0, 20.0)))
                        .on_hover_text("Enhanced metering (M)")
                        .clicked()
                    {
                        self.state.show_enhanced_metering = !self.state.show_enhanced_metering;
                    }
                });
            });
        });
    }

    fn show_transport_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Play/Stop
            let play_text = if self.state.is_playing { "Stop" } else { "Play" };
            let play_color = if self.state.is_playing {
                self.theme.palette.error
            } else {
                self.theme.palette.success
            };

            if ui.add(egui::Button::new(
                egui::RichText::new(play_text).color(play_color).size(14.0)
            ).min_size(Vec2::new(60.0, 30.0))).on_hover_text("Play/Stop (Space)").clicked() {
                self.state.toggle_play();
            }

            // Reset button
            if ui.button("Reset").on_hover_text("Return to start (Enter)").clicked() {
                self.state.position = 0.0;
            }

            ui.separator();

            // Position display
            ui.label(
                egui::RichText::new(format!(
                    "{} / {}",
                    self.state.format_position(),
                    self.state.format_duration()
                ))
                .monospace()
                .size(14.0)
            );

            // Position slider
            ui.add(egui::Slider::new(&mut self.state.position, 0.0..=1.0)
                .show_value(false))
                .on_hover_text("Playback position (drag to seek)");

            ui.separator();

            // Peak reset
            if ui.button("Reset Peaks").on_hover_text("Clear peak hold indicators").clicked() {
                self.state.reset_peaks();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Export button
                let export_color = if self.state.show_export {
                    self.theme.palette.accent
                } else {
                    self.theme.palette.text_primary
                };
                if ui.add(egui::Button::new(
                    egui::RichText::new("Export").color(export_color).size(13.0)
                ).min_size(Vec2::new(70.0, 28.0))).on_hover_text("Export master audio").clicked() {
                    self.state.show_export = !self.state.show_export;
                }

                // Toggle chain panel
                ui.toggle_value(&mut self.state.show_chain, "Chain")
                    .on_hover_text("Show/hide mastering processor chain");
            });
        });
    }

    fn show_chain_panel(&mut self, ui: &mut Ui) {
        ui.heading("Mastering Chain");
        ui.separator();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut remove_idx = None;

                for (idx, processor) in self.state.chain.iter_mut().enumerate() {
                    let is_selected = self.state.selected_processor == Some(idx);

                    let bg = if is_selected {
                        self.theme.palette.bg_tertiary
                    } else {
                        self.theme.palette.bg_secondary
                    };

                    egui::Frame::none()
                        .fill(bg)
                        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Enable checkbox
                                ui.checkbox(&mut processor.enabled, "");

                                // Processor name (clickable)
                                let name_color = if processor.enabled && !processor.bypassed {
                                    self.theme.palette.text_primary
                                } else {
                                    self.theme.palette.text_secondary
                                };

                                if ui.add(egui::Label::new(
                                    egui::RichText::new(processor.processor_type.name())
                                        .color(name_color)
                                        .strong()
                                ).sense(egui::Sense::click())).clicked() {
                                    self.state.selected_processor = Some(idx);
                                }
                            });

                            ui.horizontal(|ui| {
                                // Bypass button
                                let bypass_color = if processor.bypassed {
                                    Color32::from_rgb(241, 196, 15)
                                } else {
                                    self.theme.palette.text_secondary
                                };
                                if ui.add(egui::Button::new(
                                    egui::RichText::new("BYP").size(10.0).color(bypass_color)
                                ).min_size(Vec2::new(30.0, 18.0))).clicked() {
                                    processor.bypassed = !processor.bypassed;
                                }

                                // Remove button
                                if ui.add(egui::Button::new(
                                    egui::RichText::new("X").size(10.0).color(self.theme.palette.error)
                                ).min_size(Vec2::new(18.0, 18.0))).clicked() {
                                    remove_idx = Some(idx);
                                }
                            });
                        });

                    ui.add_space(4.0);
                }

                if let Some(idx) = remove_idx {
                    self.state.remove_processor(idx);
                }

                ui.separator();

                // Add processor dropdown
                ui.menu_button("+ Add Processor", |ui| {
                    for ptype in [
                        ProcessorType::Equalizer,
                        ProcessorType::Compressor,
                        ProcessorType::Limiter,
                        ProcessorType::StereoImager,
                        ProcessorType::Saturator,
                    ] {
                        if ui.button(ptype.name()).clicked() {
                            self.state.add_processor(ptype);
                            ui.close_menu();
                        }
                    }
                });
            });
    }

    fn show_processor_detail(&mut self, ui: &mut Ui) {
        let Some(selected_idx) = self.state.selected_processor else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a processor from the chain");
            });
            return;
        };

        let Some(processor) = self.state.chain.get_mut(selected_idx) else {
            return;
        };

        let processor_type = processor.processor_type;
        let bypassed = processor.bypassed;

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading(processor_type.name());

                if bypassed {
                    ui.label(
                        egui::RichText::new("(Bypassed)")
                            .color(Color32::from_rgb(241, 196, 15))
                    );
                }
            });

            ui.separator();

            // Draw processor-specific controls
            match processor_type {
                ProcessorType::Equalizer => {
                    draw_eq_controls(ui, &mut processor.params);
                }
                ProcessorType::Compressor => {
                    draw_compressor_controls(ui, &mut processor.params);
                }
                ProcessorType::Limiter => {
                    draw_limiter_controls(ui, &mut processor.params);
                }
                ProcessorType::StereoImager => {
                    draw_stereo_controls(ui, &mut processor.params);
                }
                ProcessorType::Saturator => {
                    draw_saturator_controls(ui, &mut processor.params);
                }
            }
        });
    }

    fn show_waveform_overview(&mut self, ui: &mut Ui) {
        ui.label("Waveform");

        let desired_height = 80.0;
        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), desired_height),
            egui::Sense::click_and_drag(),
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background
            painter.rect_filled(rect, 2.0, self.theme.palette.bg_secondary);

            // Draw fake waveform
            let center_y = rect.center().y;
            let amplitude = rect.height() / 2.5;

            // Left channel (top half)
            for i in 0..100 {
                let x = rect.left() + (i as f32 / 100.0) * rect.width();
                let phase = i as f32 * 0.15;
                let env = 0.5 + 0.5 * ((i as f32 / 30.0).sin() * 0.5 + 0.5);
                let y1 = center_y - amplitude * env * phase.sin().abs();
                let y2 = center_y;

                painter.line_segment(
                    [Pos2::new(x, y1), Pos2::new(x, y2)],
                    egui::Stroke::new(1.0, self.theme.palette.accent.linear_multiply(0.7)),
                );
            }

            // Right channel (bottom half)
            for i in 0..100 {
                let x = rect.left() + (i as f32 / 100.0) * rect.width();
                let phase = i as f32 * 0.15 + 0.3;
                let env = 0.5 + 0.5 * ((i as f32 / 28.0).sin() * 0.5 + 0.5);
                let y1 = center_y;
                let y2 = center_y + amplitude * env * phase.sin().abs();

                painter.line_segment(
                    [Pos2::new(x, y1), Pos2::new(x, y2)],
                    egui::Stroke::new(1.0, self.theme.palette.success.linear_multiply(0.7)),
                );
            }

            // Center line
            painter.line_segment(
                [Pos2::new(rect.left(), center_y), Pos2::new(rect.right(), center_y)],
                egui::Stroke::new(1.0, self.theme.palette.text_secondary.linear_multiply(0.3)),
            );

            // Playhead
            let playhead_x = rect.left() + self.state.position * rect.width();
            painter.line_segment(
                [Pos2::new(playhead_x, rect.top()), Pos2::new(playhead_x, rect.bottom())],
                egui::Stroke::new(2.0, Color32::WHITE),
            );
        }
    }

    fn show_metering_panel(&mut self, ui: &mut Ui) {
        ui.heading("Metering");
        ui.separator();

        // Level meters
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("L");
                LevelMeter::new(self.state.level_l)
                    .peak(self.state.peak_l)
                    .width(16.0)
                    .height(150.0)
                    .show(ui);
            });

            ui.vertical(|ui| {
                ui.label("R");
                LevelMeter::new(self.state.level_r)
                    .peak(self.state.peak_r)
                    .width(16.0)
                    .height(150.0)
                    .show(ui);
            });

            ui.add_space(8.0);

            // LUFS meter
            ui.vertical(|ui| {
                ui.label("LUFS");
                self.draw_lufs_meter(ui);
            });
        });

        ui.separator();

        // Loudness readings
        ui.group(|ui| {
            ui.label(egui::RichText::new("Loudness").strong());

            ui.horizontal(|ui| {
                ui.label("Integrated:");
                ui.label(
                    egui::RichText::new(format!("{:.1} LUFS", self.state.metering.integrated_lufs))
                        .monospace()
                );
            });

            ui.horizontal(|ui| {
                ui.label("Short-term:");
                ui.label(
                    egui::RichText::new(format!("{:.1} LUFS", self.state.metering.short_term_lufs))
                        .monospace()
                );
            });

            ui.horizontal(|ui| {
                ui.label("Momentary:");
                ui.label(
                    egui::RichText::new(format!("{:.1} LUFS", self.state.metering.momentary_lufs))
                        .monospace()
                );
            });

            ui.horizontal(|ui| {
                ui.label("LRA:");
                ui.label(
                    egui::RichText::new(format!("{:.1} LU", self.state.metering.loudness_range))
                        .monospace()
                );
            });
        });

        ui.add_space(4.0);

        // True peak
        ui.group(|ui| {
            ui.label(egui::RichText::new("True Peak").strong());

            let peak_color = |db: f32| {
                if db > -1.0 {
                    self.theme.palette.error
                } else if db > -3.0 {
                    Color32::from_rgb(241, 196, 15)
                } else {
                    self.theme.palette.text_primary
                }
            };

            ui.horizontal(|ui| {
                ui.label("L:");
                ui.label(
                    egui::RichText::new(format!("{:.1} dBTP", self.state.metering.true_peak_l))
                        .monospace()
                        .color(peak_color(self.state.metering.true_peak_l))
                );
            });

            ui.horizontal(|ui| {
                ui.label("R:");
                ui.label(
                    egui::RichText::new(format!("{:.1} dBTP", self.state.metering.true_peak_r))
                        .monospace()
                        .color(peak_color(self.state.metering.true_peak_r))
                );
            });
        });
    }

    fn draw_lufs_meter(&self, ui: &mut Ui) {
        let height = 150.0;
        let width = 24.0;

        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background
            painter.rect_filled(rect, 2.0, Color32::from_gray(20));

            // LUFS range: typically -24 to 0
            let lufs = self.state.metering.short_term_lufs;
            let normalized = ((lufs + 24.0) / 24.0).clamp(0.0, 1.0);
            let level_height = rect.height() * normalized;

            let level_rect = Rect::from_min_size(
                Pos2::new(rect.left() + 2.0, rect.bottom() - level_height),
                Vec2::new(rect.width() - 4.0, level_height),
            );

            // Color based on loudness
            let color = if lufs > -9.0 {
                Color32::from_rgb(234, 67, 53)  // Too loud
            } else if lufs > -14.0 {
                Color32::from_rgb(251, 188, 4)  // Optimal
            } else {
                Color32::from_rgb(52, 168, 83)  // Quiet
            };

            painter.rect_filled(level_rect, 0.0, color);

            // Target line at -14 LUFS
            let target_y = rect.bottom() - (rect.height() * ((-14.0 + 24.0) / 24.0));
            painter.line_segment(
                [Pos2::new(rect.left(), target_y), Pos2::new(rect.right(), target_y)],
                egui::Stroke::new(2.0, Color32::WHITE),
            );

            // Scale markers
            for lufs_mark in [-24, -18, -14, -9, -6, 0] {
                let y = rect.bottom() - (rect.height() * ((lufs_mark as f32 + 24.0) / 24.0));
                painter.line_segment(
                    [Pos2::new(rect.left(), y), Pos2::new(rect.left() + 4.0, y)],
                    egui::Stroke::new(1.0, Color32::from_gray(80)),
                );
            }
        }
    }

    fn show_export_panel(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Export Settings");

            ui.horizontal(|ui| {
                // Format selection
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Format").strong());

                    egui::ComboBox::from_id_salt("export_format")
                        .selected_text(self.state.export_settings.format.name())
                        .show_ui(ui, |ui| {
                            for fmt in [
                                ExportFormat::Wav16,
                                ExportFormat::Wav24,
                                ExportFormat::Wav32Float,
                                ExportFormat::Flac,
                                ExportFormat::Mp3_320,
                                ExportFormat::Mp3V0,
                                ExportFormat::Ogg,
                            ] {
                                ui.selectable_value(
                                    &mut self.state.export_settings.format,
                                    fmt,
                                    fmt.name(),
                                ).on_hover_text(fmt.description());
                            }
                        }).response.on_hover_text("Audio file format for export");

                    // Sample rate
                    egui::ComboBox::from_id_salt("sample_rate")
                        .selected_text(format!("{} Hz", self.state.export_settings.sample_rate))
                        .show_ui(ui, |ui| {
                            for rate in [44100, 48000, 88200, 96000] {
                                ui.selectable_value(
                                    &mut self.state.export_settings.sample_rate,
                                    rate,
                                    format!("{} Hz", rate),
                                );
                            }
                        }).response.on_hover_text("Sample rate (44.1kHz for CD, 48kHz for video)");

                    ui.checkbox(&mut self.state.export_settings.dither, "Apply Dither")
                        .on_hover_text("Add noise shaping to reduce quantization artifacts");
                });

                ui.separator();

                // Normalization settings
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Normalization").strong());

                    ui.checkbox(&mut self.state.export_settings.normalize, "Normalize to LUFS")
                        .on_hover_text("Adjust loudness to meet streaming platform standards");

                    if self.state.export_settings.normalize {
                        ui.horizontal(|ui| {
                            ui.label("Target:");
                            ui.add(egui::DragValue::new(&mut self.state.export_settings.target_lufs)
                                .range(-24.0..=-6.0)
                                .suffix(" LUFS")
                                .speed(0.5))
                                .on_hover_text("Target loudness (-14 LUFS for Spotify/YouTube)");
                        });
                    }

                    ui.checkbox(&mut self.state.export_settings.limit_true_peak, "Limit True Peak")
                        .on_hover_text("Prevent inter-sample peaks that cause distortion");

                    if self.state.export_settings.limit_true_peak {
                        ui.horizontal(|ui| {
                            ui.label("Ceiling:");
                            ui.add(egui::DragValue::new(&mut self.state.export_settings.true_peak_ceiling)
                                .range(-6.0..=0.0)
                                .suffix(" dBTP")
                                .speed(0.1))
                                .on_hover_text("Maximum true peak level (-1 dBTP recommended)");
                        });
                    }
                });

                ui.separator();

                // Compression settings (only for supported formats)
                let format_supports_compression = self.state.export_settings.format.supports_compression();
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Compression").strong());
                        if !format_supports_compression {
                            ui.label(
                                egui::RichText::new("(not available for this format)")
                                    .small()
                                    .color(self.theme.palette.text_secondary)
                            );
                        }
                    });

                    ui.add_enabled_ui(format_supports_compression, |ui| {
                        // Compression mode selector
                        egui::ComboBox::from_id_salt("compression_mode")
                            .selected_text(self.state.export_settings.compression.mode.name())
                            .show_ui(ui, |ui| {
                                for mode in [CompressionMode::None, CompressionMode::Standard, CompressionMode::Dictionary] {
                                    ui.selectable_value(
                                        &mut self.state.export_settings.compression.mode,
                                        mode,
                                        mode.name(),
                                    ).on_hover_text(mode.description());
                                }
                            });

                        // Show additional options only when compression is enabled
                        if self.state.export_settings.compression.mode != CompressionMode::None {
                            ui.horizontal(|ui| {
                                ui.label("Level:");
                                ui.add(egui::Slider::new(&mut self.state.export_settings.compression.level, 1..=19)
                                    .clamp_to_range(true))
                                    .on_hover_text("Compression level (higher = smaller file, slower encode)");
                                ui.label(
                                    egui::RichText::new(if self.state.export_settings.compression.level <= 3 {
                                        "(fast)"
                                    } else if self.state.export_settings.compression.level <= 9 {
                                        "(balanced)"
                                    } else {
                                        "(max ratio)"
                                    })
                                    .small()
                                    .color(self.theme.palette.text_secondary)
                                );
                            });

                            // Dictionary options
                            if self.state.export_settings.compression.mode == CompressionMode::Dictionary {
                                ui.checkbox(
                                    &mut self.state.export_settings.compression.use_builtin_dictionary,
                                    "Use built-in dictionary"
                                ).on_hover_text("Use Orpheus built-in dictionary optimized for audio project data");

                                if !self.state.export_settings.compression.use_builtin_dictionary {
                                    ui.horizontal(|ui| {
                                        ui.label("Custom:");
                                        let path_text = self.state.export_settings.compression.dictionary_path
                                            .as_ref()
                                            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
                                            .unwrap_or_else(|| "(none)".to_string());
                                        ui.label(egui::RichText::new(path_text).monospace());
                                        if ui.button("Browse...").clicked() {
                                            // Would open file dialog
                                        }
                                    });
                                }
                            }
                        }
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(
                        egui::RichText::new("Export Now").size(14.0).strong()
                    ).min_size(Vec2::new(100.0, 36.0)).fill(self.theme.palette.accent)).clicked() {
                        // Would trigger actual export
                    }

                    ui.label(
                        egui::RichText::new(format!(
                            "Output: {}",
                            self.state.export_settings.output_filename("master")
                        ))
                        .color(self.theme.palette.text_secondary)
                    );
                });
            });
        });
    }
}

// Standalone functions for processor controls to avoid borrow conflicts

fn draw_eq_controls(ui: &mut Ui, params: &mut ProcessorParams) {
    ui.horizontal(|ui| {
        // Low band
        ui.vertical(|ui| {
            ui.label("Low");
            let label = format!("{:.1} dB", params.low_gain);
            Knob::new(&mut params.low_gain)
                .range(-12.0, 12.0)
                .default_value(0.0)
                .size(50.0)
                .label(&label)
                .show(ui);

            ui.horizontal(|ui| {
                ui.label("Freq:");
                ui.add(egui::DragValue::new(&mut params.low_freq)
                    .range(20.0..=500.0)
                    .suffix(" Hz")
                    .speed(5.0));
            });
        });

        ui.separator();

        // Mid band
        ui.vertical(|ui| {
            ui.label("Mid");
            let label = format!("{:.1} dB", params.mid_gain);
            Knob::new(&mut params.mid_gain)
                .range(-12.0, 12.0)
                .default_value(0.0)
                .size(50.0)
                .label(&label)
                .show(ui);
        });

        ui.separator();

        // High band
        ui.vertical(|ui| {
            ui.label("High");
            let label = format!("{:.1} dB", params.high_gain);
            Knob::new(&mut params.high_gain)
                .range(-12.0, 12.0)
                .default_value(0.0)
                .size(50.0)
                .label(&label)
                .show(ui);

            ui.horizontal(|ui| {
                ui.label("Freq:");
                ui.add(egui::DragValue::new(&mut params.high_freq)
                    .range(1000.0..=16000.0)
                    .suffix(" Hz")
                    .speed(50.0));
            });
        });
    });
}

fn draw_compressor_controls(ui: &mut Ui, params: &mut ProcessorParams) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("Threshold");
            let label = format!("{:.1} dB", params.threshold);
            Knob::new(&mut params.threshold)
                .range(-40.0, 0.0)
                .default_value(-10.0)
                .size(50.0)
                .label(&label)
                .show(ui);
        });

        ui.vertical(|ui| {
            ui.label("Ratio");
            let label = format!("{:.1}:1", params.ratio);
            Knob::new(&mut params.ratio)
                .range(1.0, 20.0)
                .default_value(4.0)
                .size(50.0)
                .label(&label)
                .show(ui);
        });

        ui.vertical(|ui| {
            ui.label("Attack");
            let label = format!("{:.1} ms", params.attack);
            Knob::new(&mut params.attack)
                .range(0.1, 100.0)
                .default_value(10.0)
                .size(50.0)
                .label(&label)
                .show(ui);
        });

        ui.vertical(|ui| {
            ui.label("Release");
            let label = format!("{:.0} ms", params.release);
            Knob::new(&mut params.release)
                .range(10.0, 1000.0)
                .default_value(100.0)
                .size(50.0)
                .label(&label)
                .show(ui);
        });

        ui.vertical(|ui| {
            ui.label("Makeup");
            let label = format!("{:.1} dB", params.makeup);
            Knob::new(&mut params.makeup)
                .range(0.0, 24.0)
                .default_value(0.0)
                .size(50.0)
                .label(&label)
                .show(ui);
        });
    });
}

fn draw_limiter_controls(ui: &mut Ui, params: &mut ProcessorParams) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("Ceiling");
            let label = format!("{:.1} dB", params.ceiling);
            Knob::new(&mut params.ceiling)
                .range(-6.0, 0.0)
                .default_value(-0.3)
                .size(60.0)
                .label(&label)
                .show(ui);
        });

        ui.vertical(|ui| {
            ui.label("Lookahead");
            let label = format!("{:.1} ms", params.lookahead);
            Knob::new(&mut params.lookahead)
                .range(0.0, 10.0)
                .default_value(5.0)
                .size(60.0)
                .label(&label)
                .show(ui);
        });
    });

    ui.add_space(8.0);
    ui.label(
        egui::RichText::new("True peak limiting prevents intersample clipping")
            .size(11.0)
            .color(Color32::from_gray(128))
    );
}

fn draw_stereo_controls(ui: &mut Ui, params: &mut ProcessorParams) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("Width");
            let label = format!("{:.0}%", params.width * 100.0);
            Knob::new(&mut params.width)
                .range(0.0, 2.0)
                .default_value(1.0)
                .size(60.0)
                .label(&label)
                .show(ui);
        });

        ui.vertical(|ui| {
            ui.label("Balance");
            let balance_label = if params.balance < 0.0 {
                format!("L {:.0}%", params.balance.abs() * 100.0)
            } else if params.balance > 0.0 {
                format!("R {:.0}%", params.balance * 100.0)
            } else {
                "Center".to_string()
            };
            Knob::new(&mut params.balance)
                .range(-1.0, 1.0)
                .default_value(0.0)
                .size(60.0)
                .label(&balance_label)
                .show(ui);
        });
    });
}

fn draw_saturator_controls(ui: &mut Ui, params: &mut ProcessorParams) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("Drive");
            let label = format!("{:.1} dB", params.drive);
            Knob::new(&mut params.drive)
                .range(0.0, 24.0)
                .default_value(0.0)
                .size(60.0)
                .label(&label)
                .show(ui);
        });

        ui.vertical(|ui| {
            ui.label("Mix");
            let label = format!("{:.0}%", params.mix * 100.0);
            Knob::new(&mut params.mix)
                .range(0.0, 1.0)
                .default_value(1.0)
                .size(60.0)
                .label(&label)
                .show(ui);
        });
    });
}

// ============================================================================
// Additional UI Panel Methods for MasterView
// ============================================================================

impl<'a> MasterView<'a> {
    /// Show source tracks panel (from mix)
    fn show_source_panel(&mut self, ui: &mut Ui) {
        ui.heading("Source Tracks");
        ui.label(egui::RichText::new("From Mix Bus").small().color(self.theme.palette.text_secondary));
        ui.separator();

        // Mix bus meter
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Mix Bus").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("{:.1} dB", self.state.mix_source.bus_level_db))
                        .monospace()
                        .small());
                });
            });

            // Simple bus meter
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 12.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 2.0, self.theme.palette.bg_tertiary);
            let level = ((self.state.mix_source.bus_level_db + 60.0) / 60.0).clamp(0.0, 1.0);
            let fill_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width() * level, rect.height()));
            let color = if self.state.mix_source.bus_level_db > -3.0 {
                Color32::from_rgb(231, 76, 60)
            } else if self.state.mix_source.bus_level_db > -12.0 {
                Color32::from_rgb(241, 196, 15)
            } else {
                Color32::from_rgb(46, 204, 113)
            };
            ui.painter().rect_filled(fill_rect, 2.0, color);
        });

        ui.add_space(8.0);

        // Individual tracks
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(200.0)
            .show(ui, |ui| {
                for track in &mut self.state.mix_source.tracks {
                    ui.horizontal(|ui| {
                        // Color indicator
                        let (color_rect, _) = ui.allocate_exact_size(Vec2::new(4.0, 20.0), egui::Sense::hover());
                        ui.painter().rect_filled(color_rect, 2.0, track.color);

                        // Track name and level
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                // Mute
                                let mute_color = if track.muted {
                                    Color32::from_rgb(231, 76, 60)
                                } else {
                                    self.theme.palette.text_secondary
                                };
                                if ui.add(egui::Button::new(
                                    egui::RichText::new("M").size(9.0).color(mute_color)
                                ).min_size(Vec2::new(18.0, 16.0))).clicked() {
                                    track.muted = !track.muted;
                                }

                                // Solo
                                let solo_color = if track.solo {
                                    Color32::from_rgb(241, 196, 15)
                                } else {
                                    self.theme.palette.text_secondary
                                };
                                if ui.add(egui::Button::new(
                                    egui::RichText::new("S").size(9.0).color(solo_color)
                                ).min_size(Vec2::new(18.0, 16.0))).clicked() {
                                    track.solo = !track.solo;
                                }

                                ui.label(&track.name);

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Stem export toggle
                                    ui.checkbox(&mut track.export_stem, "")
                                        .on_hover_text("Include in stem export");

                                    ui.label(egui::RichText::new(format!("{:.1}", track.level_db))
                                        .monospace()
                                        .small());
                                });
                            });
                        });
                    });
                    ui.add_space(2.0);
                }
            });

        ui.separator();

        // Gain staging section
        ui.collapsing("Auto Gain Staging", |ui| {
            if !self.state.gain_staging.analyzed {
                if ui.button("Analyze Levels").clicked() {
                    self.state.analyze_gain_staging();
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label("Pre-master Peak:");
                    ui.label(egui::RichText::new(format!("{:.1} dB", self.state.gain_staging.pre_master_peak))
                        .monospace());
                });
                ui.horizontal(|ui| {
                    ui.label("Pre-master RMS:");
                    ui.label(egui::RichText::new(format!("{:.1} dB", self.state.gain_staging.pre_master_rms))
                        .monospace());
                });
                ui.horizontal(|ui| {
                    ui.label("Recommended Gain:");
                    let gain_color = if self.state.gain_staging.recommended_gain_db.abs() > 6.0 {
                        Color32::from_rgb(241, 196, 15)
                    } else {
                        self.theme.palette.text_primary
                    };
                    ui.label(egui::RichText::new(format!("{:+.1} dB", self.state.gain_staging.recommended_gain_db))
                        .monospace()
                        .color(gain_color));
                });

                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        self.state.gain_staging.apply();
                    }
                    if ui.button("Reset").clicked() {
                        self.state.gain_staging.reset();
                    }
                });

                if self.state.gain_staging.applied_gain_db != 0.0 {
                    ui.label(egui::RichText::new(format!(
                        "Applied: {:+.1} dB",
                        self.state.gain_staging.applied_gain_db
                    )).small().color(Color32::from_rgb(46, 204, 113)));
                }
            }
        });
    }

    /// Show reference tracks panel
    fn show_reference_panel(&mut self, ui: &mut Ui) {
        ui.heading("Reference Tracks");
        ui.separator();

        // A/B toggle
        let ab_active = self.state.reference.ab_active;
        let ab_color = if ab_active {
            Color32::from_rgb(241, 196, 15)
        } else {
            self.theme.palette.text_secondary
        };

        ui.horizontal(|ui| {
            if ui.add(egui::Button::new(
                egui::RichText::new(if ab_active { "B (Reference)" } else { "A (Master)" })
                    .color(ab_color)
                    .strong()
            ).min_size(Vec2::new(100.0, 28.0))).on_hover_text("Toggle A/B comparison (B key)").clicked() {
                self.state.reference.toggle_ab();
            }

            ui.checkbox(&mut self.state.reference.level_match, "Level Match")
                .on_hover_text("Match reference loudness to master");
        });

        ui.separator();

        // Reference track list
        if self.state.reference.tracks.is_empty() {
            ui.label(egui::RichText::new("No reference tracks loaded")
                .color(self.theme.palette.text_secondary)
                .italics());
            ui.label(egui::RichText::new("Drop audio files here or click Add")
                .small()
                .color(self.theme.palette.text_secondary));
        } else {
            let mut remove_idx = None;

            for (idx, track) in self.state.reference.tracks.iter().enumerate() {
                let is_selected = self.state.reference.selected == Some(idx);
                let bg = if is_selected {
                    self.theme.palette.bg_tertiary
                } else {
                    self.theme.palette.bg_secondary
                };

                egui::Frame::none()
                    .fill(bg)
                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                    .rounding(4.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.add(egui::Label::new(&track.name).sense(egui::Sense::click())).clicked() {
                                self.state.reference.selected = Some(idx);
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("X").clicked() {
                                    remove_idx = Some(idx);
                                }
                                ui.label(egui::RichText::new(format!("{:.1} LUFS", track.lufs))
                                    .small()
                                    .monospace());
                            });
                        });
                    });
                ui.add_space(2.0);
            }

            if let Some(idx) = remove_idx {
                self.state.reference.remove_reference(idx);
            }
        }

        ui.separator();

        // Add reference button
        if ui.button("+ Add Reference").on_hover_text("Load a reference track").clicked() {
            // Would open file dialog - for now add a demo track
            self.state.reference.add_reference(PathBuf::from("reference_master.wav"));
        }

        // Show selected reference details
        if let Some(idx) = self.state.reference.selected {
            if let Some(track) = self.state.reference.tracks.get_mut(idx) {
                ui.separator();
                ui.label(egui::RichText::new("Reference Details").strong());

                egui::Grid::new("ref_details")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("LUFS:");
                        ui.label(egui::RichText::new(format!("{:.1}", track.lufs)).monospace());
                        ui.end_row();

                        ui.label("True Peak:");
                        ui.label(egui::RichText::new(format!("{:.1} dBTP", track.true_peak_db)).monospace());
                        ui.end_row();

                        ui.label("Gain Offset:");
                        ui.add(egui::DragValue::new(&mut track.gain_offset_db)
                            .range(-24.0..=24.0)
                            .suffix(" dB")
                            .speed(0.1));
                        ui.end_row();
                    });
            }
        }
    }

    /// Show mastering presets panel
    fn show_preset_panel(&mut self, ui: &mut Ui) {
        ui.heading("Mastering Presets");
        ui.separator();

        // Quick platform targets
        ui.label(egui::RichText::new("Target Platform").strong());
        ui.horizontal_wrapped(|ui| {
            for target in [
                MasteringTarget::Streaming,
                MasteringTarget::CD,
                MasteringTarget::Club,
                MasteringTarget::Broadcast,
            ] {
                if ui.selectable_label(false, target.name())
                    .on_hover_text(target.description())
                    .clicked()
                {
                    // Find and apply matching preset
                    if let Some(preset) = self.state.presets.presets.iter()
                        .find(|p| p.target == target)
                        .cloned()
                    {
                        self.state.apply_preset(&preset);
                    }
                }
            }
        });

        ui.separator();

        // Preset list
        ui.label(egui::RichText::new("Saved Presets").strong());

        // Collect preset info to avoid borrow issues
        let preset_info: Vec<_> = self.state.presets.presets.iter()
            .enumerate()
            .map(|(idx, p)| (idx, p.name.clone(), p.target_lufs, p.clone()))
            .collect();
        let current_preset = self.state.presets.current;

        let mut apply_preset: Option<(usize, MasteringPreset)> = None;

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(150.0)
            .show(ui, |ui| {
                for (idx, name, lufs, preset) in &preset_info {
                    let is_current = current_preset == Some(*idx);

                    ui.horizontal(|ui| {
                        if ui.selectable_label(is_current, name).clicked() {
                            apply_preset = Some((*idx, preset.clone()));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(format!("{:.0} LUFS", lufs))
                                .small()
                                .monospace());
                        });
                    });
                }
            });

        // Apply preset outside the borrow
        if let Some((idx, preset)) = apply_preset {
            self.state.apply_preset(&preset);
            self.state.presets.current = Some(idx);
        }

        ui.separator();

        // Current settings summary
        ui.label(egui::RichText::new("Current Settings").strong());
        egui::Grid::new("current_settings")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label("Target LUFS:");
                ui.label(egui::RichText::new(format!("{:.1}", self.state.export_settings.target_lufs))
                    .monospace());
                ui.end_row();

                ui.label("True Peak Ceiling:");
                ui.label(egui::RichText::new(format!("{:.1} dBTP", self.state.export_settings.true_peak_ceiling))
                    .monospace());
                ui.end_row();

                ui.label("Chain:");
                ui.label(egui::RichText::new(format!("{} processors", self.state.chain.len()))
                    .monospace());
                ui.end_row();
            });

        // Save preset button
        ui.separator();
        if ui.button("Save as New Preset...").clicked() {
            // Would open save dialog
        }
    }

    /// Show enhanced metering panel
    fn show_enhanced_metering_panel(&mut self, ui: &mut Ui) {
        ui.heading("Enhanced Metering");
        ui.separator();

        // Correlation meter
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Correlation").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let corr = self.state.enhanced_metering.correlation;
                    let corr_color = if corr > 0.5 {
                        Color32::from_rgb(46, 204, 113)
                    } else if corr > 0.0 {
                        Color32::from_rgb(241, 196, 15)
                    } else {
                        Color32::from_rgb(231, 76, 60)
                    };
                    ui.label(egui::RichText::new(format!("{:.2}", corr))
                        .monospace()
                        .color(corr_color));
                });
            });

            // Correlation meter visualization
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 16.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 2.0, self.theme.palette.bg_tertiary);

            // Center marker
            let center_x = rect.center().x;
            ui.painter().vline(center_x, rect.y_range(), Stroke::new(1.0, Color32::WHITE));

            // Correlation indicator
            let corr = self.state.enhanced_metering.correlation;
            let indicator_x = center_x + (corr * rect.width() / 2.0);
            let indicator_rect = Rect::from_center_size(
                Pos2::new(indicator_x, rect.center().y),
                Vec2::new(6.0, rect.height() - 4.0)
            );
            let indicator_color = if corr > 0.5 {
                Color32::from_rgb(46, 204, 113)
            } else if corr > 0.0 {
                Color32::from_rgb(241, 196, 15)
            } else {
                Color32::from_rgb(231, 76, 60)
            };
            ui.painter().rect_filled(indicator_rect, 2.0, indicator_color);
        });

        ui.add_space(4.0);

        // Mid/Side levels
        ui.group(|ui| {
            ui.label(egui::RichText::new("Mid/Side").strong());

            ui.horizontal(|ui| {
                ui.label("Mid:");
                ui.label(egui::RichText::new(format!("{:.1} dB", self.state.enhanced_metering.mid_level_db))
                    .monospace());

                ui.separator();

                ui.label("Side:");
                ui.label(egui::RichText::new(format!("{:.1} dB", self.state.enhanced_metering.side_level_db))
                    .monospace());
            });
        });

        ui.add_space(4.0);

        // Spectrum analyzer
        ui.group(|ui| {
            ui.label(egui::RichText::new("Spectrum").strong());

            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 60.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 2.0, self.theme.palette.bg_tertiary);

            // Draw spectrum bars
            let num_bands = 31;
            let bar_width = rect.width() / num_bands as f32;

            for (i, &level) in self.state.enhanced_metering.spectrum.iter().enumerate() {
                let normalized = ((level + 60.0) / 60.0).clamp(0.0, 1.0);
                let bar_height = rect.height() * normalized;

                let bar_rect = Rect::from_min_size(
                    Pos2::new(rect.left() + i as f32 * bar_width + 1.0, rect.bottom() - bar_height),
                    Vec2::new(bar_width - 2.0, bar_height)
                );

                // Color gradient based on frequency
                let hue = i as f32 / num_bands as f32;
                let color = Color32::from_rgb(
                    (80.0 + hue * 150.0) as u8,
                    (200.0 - hue * 100.0) as u8,
                    (100.0 + hue * 50.0) as u8,
                );

                ui.painter().rect_filled(bar_rect, 0.0, color);
            }
        });

        ui.add_space(4.0);

        // Additional metrics
        ui.group(|ui| {
            ui.label(egui::RichText::new("Dynamics").strong());

            egui::Grid::new("dynamics_grid")
                .num_columns(2)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Crest Factor:");
                    ui.label(egui::RichText::new(format!("{:.1} dB", self.state.enhanced_metering.crest_factor))
                        .monospace());
                    ui.end_row();

                    ui.label("RMS L/R:");
                    ui.label(egui::RichText::new(format!("{:.1} / {:.1} dB",
                        self.state.enhanced_metering.rms_l,
                        self.state.enhanced_metering.rms_r
                    )).monospace());
                    ui.end_row();

                    ui.label("Balance:");
                    let balance = self.state.enhanced_metering.balance_db;
                    let balance_text = if balance.abs() < 0.5 {
                        "Center".to_string()
                    } else if balance < 0.0 {
                        format!("L {:.1} dB", balance.abs())
                    } else {
                        format!("R {:.1} dB", balance)
                    };
                    ui.label(egui::RichText::new(balance_text).monospace());
                    ui.end_row();
                });
        });

        // K-System mode selector
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("K-System:");
            egui::ComboBox::from_id_salt("k_mode")
                .selected_text(self.state.enhanced_metering.k_mode.name())
                .show_ui(ui, |ui| {
                    for mode in [KMeteringMode::K12, KMeteringMode::K14, KMeteringMode::K20] {
                        ui.selectable_value(&mut self.state.enhanced_metering.k_mode, mode, mode.name());
                    }
                });
        });
    }

    /// Show batch export panel
    fn show_batch_export_panel(&mut self, ui: &mut Ui) {
        ui.heading("Batch Export");
        ui.separator();

        // Output directory
        ui.horizontal(|ui| {
            ui.label("Output Folder:");
            let dir_text = self.state.batch_export.output_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(default)".to_string());
            ui.label(egui::RichText::new(dir_text).monospace().small());
            if ui.small_button("Browse...").clicked() {
                // Would open folder dialog
            }
        });

        ui.horizontal(|ui| {
            ui.label("Base Name:");
            ui.text_edit_singleline(&mut self.state.batch_export.base_name);
        });

        ui.separator();

        // Export jobs
        ui.label(egui::RichText::new("Export Jobs").strong());

        let mut remove_idx = None;

        for (idx, job) in self.state.batch_export.jobs.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.checkbox(&mut job.enabled, "");

                ui.label(job.format.name());
                ui.label(egui::RichText::new(format!("{}Hz", job.sample_rate))
                    .small()
                    .monospace());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("X").clicked() {
                        remove_idx = Some(idx);
                    }

                    // Status indicator
                    match job.status {
                        BatchJobStatus::Pending => {
                            ui.label(egui::RichText::new("○").color(self.theme.palette.text_secondary));
                        }
                        BatchJobStatus::Running => {
                            ui.label(egui::RichText::new("◐").color(Color32::from_rgb(241, 196, 15)));
                        }
                        BatchJobStatus::Completed => {
                            ui.label(egui::RichText::new("✓").color(Color32::from_rgb(46, 204, 113)));
                        }
                        BatchJobStatus::Failed => {
                            ui.label(egui::RichText::new("✕").color(Color32::from_rgb(231, 76, 60)));
                        }
                    }
                });
            });
        }

        if let Some(idx) = remove_idx {
            self.state.batch_export.jobs.remove(idx);
        }

        ui.separator();

        // Add job menu
        ui.menu_button("+ Add Format", |ui| {
            for (format, sample_rate, suffix) in [
                (ExportFormat::Wav24, 44100, "wav24"),
                (ExportFormat::Wav24, 48000, "wav24_48k"),
                (ExportFormat::Wav16, 44100, "wav16"),
                (ExportFormat::Flac, 44100, "flac"),
                (ExportFormat::Mp3_320, 44100, "mp3"),
                (ExportFormat::Ogg, 44100, "ogg"),
            ] {
                if ui.button(format!("{} @ {}Hz", format.name(), sample_rate)).clicked() {
                    self.state.batch_export.jobs.push(
                        BatchExportJob::new(format, sample_rate, suffix)
                    );
                    ui.close_menu();
                }
            }
        });

        // Stem export toggle
        ui.separator();
        ui.checkbox(&mut self.state.stem_config.export_stems, "Export Stems")
            .on_hover_text("Export individual track stems alongside master");

        if self.state.stem_config.export_stems {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.checkbox(&mut self.state.stem_config.process_stems, "Apply processing to stems");
            });

            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.label("Naming:");
                egui::ComboBox::from_id_salt("stem_naming")
                    .selected_text(self.state.stem_config.naming_pattern.name())
                    .show_ui(ui, |ui| {
                        for pattern in [StemNamingPattern::TrackNumber, StemNamingPattern::TrackName, StemNamingPattern::ProjectTrack] {
                            ui.selectable_value(&mut self.state.stem_config.naming_pattern, pattern, pattern.name());
                        }
                    });
            });

            // Show which stems will be exported
            let stem_count = self.state.mix_source.tracks.iter().filter(|t| t.export_stem).count();
            ui.label(egui::RichText::new(format!("{} stems selected", stem_count))
                .small()
                .color(self.theme.palette.text_secondary));
        }

        ui.separator();

        // Export progress / button
        if self.state.batch_export.is_running {
            let progress = self.state.batch_export.total_progress();
            ui.add(egui::ProgressBar::new(progress).show_percentage());

            if ui.button("Cancel").clicked() {
                self.state.batch_export.is_running = false;
            }
        } else {
            let enabled_count = self.state.batch_export.enabled_count();
            if ui.add_enabled(enabled_count > 0, egui::Button::new(
                egui::RichText::new(format!("Export {} Format{}", enabled_count, if enabled_count == 1 { "" } else { "s" }))
                    .strong()
            ).min_size(Vec2::new(0.0, 32.0))).clicked() {
                // Would start batch export
                self.state.batch_export.is_running = true;
            }
        }
    }

    /// Handle keyboard shortcuts
    fn handle_keyboard(&mut self, ui: &mut Ui) {
        ui.input(|i| {
            // Space -> toggle playback
            if i.key_pressed(egui::Key::Space) {
                self.state.toggle_play();
            }

            // B -> toggle A/B reference
            if i.key_pressed(egui::Key::B) {
                self.state.toggle_reference_ab();
            }

            // M -> toggle enhanced metering
            if i.key_pressed(egui::Key::M) && !i.modifiers.ctrl {
                self.state.show_enhanced_metering = !self.state.show_enhanced_metering;
            }

            // E -> toggle export panel
            if i.key_pressed(egui::Key::E) && !i.modifiers.ctrl {
                self.state.show_export = !self.state.show_export;
            }

            // P -> toggle presets panel
            if i.key_pressed(egui::Key::P) && !i.modifiers.ctrl {
                self.state.presets.show_panel = !self.state.presets.show_panel;
            }

            // R -> toggle reference panel
            if i.key_pressed(egui::Key::R) && !i.modifiers.ctrl {
                self.state.reference.show_panel = !self.state.reference.show_panel;
            }

            // S -> toggle sources panel
            if i.key_pressed(egui::Key::S) && !i.modifiers.ctrl {
                self.state.mix_source.show_sources = !self.state.mix_source.show_sources;
            }

            // Tab -> cycle through left panel tabs
            if i.key_pressed(egui::Key::Tab) && !i.modifiers.ctrl {
                self.state.active_panel = match self.state.active_panel {
                    MasterPanelTab::Chain => MasterPanelTab::Sources,
                    MasterPanelTab::Sources => MasterPanelTab::Reference,
                    MasterPanelTab::Reference => MasterPanelTab::Presets,
                    MasterPanelTab::Presets => MasterPanelTab::Chain,
                };
            }

            // Escape -> stop playback
            if i.key_pressed(egui::Key::Escape) {
                self.state.is_playing = false;
            }

            // Enter -> reset to start
            if i.key_pressed(egui::Key::Enter) {
                self.state.position = 0.0;
            }

            // Backspace -> reset peaks
            if i.key_pressed(egui::Key::Backspace) {
                self.state.reset_peaks();
            }
        });
    }
}
