//! Record mode view - audio and MIDI recording

use egui::{Color32, Pos2, Rect, Ui, ScrollArea, Stroke, Vec2};
use crate::theme::Theme;
use crate::widgets::{LevelMeter, Knob};

// ============================================================================
// Signal Analysis - Real-time input monitoring and quality metrics
// ============================================================================

/// Real-time analysis data for an audio input
#[derive(Clone)]
pub struct InputAnalysis {
    /// Rolling waveform buffer (recent samples for visualization)
    pub waveform_buffer: Vec<f32>,
    /// Maximum buffer size
    pub buffer_size: usize,
    /// Current write position in buffer
    pub buffer_pos: usize,
    /// RMS level (root mean square)
    pub rms_db: f32,
    /// Peak level
    pub peak_db: f32,
    /// True peak (inter-sample peak approximation)
    pub true_peak_db: f32,
    /// Short-term LUFS (loudness units full scale)
    pub lufs_short: f32,
    /// Integrated LUFS (cumulative)
    pub lufs_integrated: f32,
    /// Loudness range (LRA)
    pub loudness_range: f32,
    /// Number of clipping events
    pub clip_count: u32,
    /// Time since last clip (seconds)
    pub time_since_clip: f32,
    /// Headroom (dB below 0)
    pub headroom_db: f32,
    /// Dynamic range (peak - RMS)
    pub dynamic_range_db: f32,
    /// DC offset
    pub dc_offset: f32,
    /// Frequency analysis bins (simplified spectrum)
    pub spectrum_bins: [f32; 8],
    /// Is signal present (above noise floor)
    pub signal_present: bool,
    /// Peak hold value
    pub peak_hold_db: f32,
    /// Peak hold decay timer
    pub peak_hold_time: f32,
}

impl Default for InputAnalysis {
    fn default() -> Self {
        Self::new(1024)
    }
}

impl InputAnalysis {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            waveform_buffer: vec![0.0; buffer_size],
            buffer_size,
            buffer_pos: 0,
            rms_db: -60.0,
            peak_db: -60.0,
            true_peak_db: -60.0,
            lufs_short: -60.0,
            lufs_integrated: -60.0,
            loudness_range: 0.0,
            clip_count: 0,
            time_since_clip: f32::MAX,
            headroom_db: 60.0,
            dynamic_range_db: 0.0,
            dc_offset: 0.0,
            spectrum_bins: [0.0; 8],
            signal_present: false,
            peak_hold_db: -60.0,
            peak_hold_time: 0.0,
        }
    }

    /// Reset all analysis data
    pub fn reset(&mut self) {
        self.waveform_buffer.fill(0.0);
        self.buffer_pos = 0;
        self.rms_db = -60.0;
        self.peak_db = -60.0;
        self.true_peak_db = -60.0;
        self.lufs_short = -60.0;
        self.lufs_integrated = -60.0;
        self.loudness_range = 0.0;
        self.clip_count = 0;
        self.time_since_clip = f32::MAX;
        self.headroom_db = 60.0;
        self.dynamic_range_db = 0.0;
        self.dc_offset = 0.0;
        self.spectrum_bins = [0.0; 8];
        self.signal_present = false;
        self.peak_hold_db = -60.0;
        self.peak_hold_time = 0.0;
    }

    /// Process a batch of audio samples
    pub fn process_samples(&mut self, samples: &[f32], dt: f32) {
        if samples.is_empty() {
            return;
        }

        // Update waveform buffer
        for &sample in samples {
            self.waveform_buffer[self.buffer_pos] = sample;
            self.buffer_pos = (self.buffer_pos + 1) % self.buffer_size;
        }

        // Calculate RMS
        let sum_squared: f32 = samples.iter().map(|s| s * s).sum();
        let rms = (sum_squared / samples.len() as f32).sqrt();
        self.rms_db = if rms > 0.0 { 20.0 * rms.log10() } else { -60.0 };

        // Calculate peak
        let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        self.peak_db = if peak > 0.0 { 20.0 * peak.log10() } else { -60.0 };

        // True peak approximation (4x oversampling simulation)
        let true_peak = self.estimate_true_peak(samples);
        self.true_peak_db = if true_peak > 0.0 { 20.0 * true_peak.log10() } else { -60.0 };

        // Update headroom
        self.headroom_db = -self.true_peak_db.max(self.peak_db);

        // Dynamic range
        self.dynamic_range_db = self.peak_db - self.rms_db;

        // Clipping detection
        if peak >= 0.99 {
            self.clip_count += 1;
            self.time_since_clip = 0.0;
        } else {
            self.time_since_clip += dt;
        }

        // Peak hold with decay
        if self.peak_db > self.peak_hold_db {
            self.peak_hold_db = self.peak_db;
            self.peak_hold_time = 2.0; // Hold for 2 seconds
        } else {
            self.peak_hold_time -= dt;
            if self.peak_hold_time <= 0.0 {
                self.peak_hold_db = (self.peak_hold_db - 20.0 * dt).max(self.peak_db);
            }
        }

        // Signal presence (above -50dB threshold)
        self.signal_present = self.rms_db > -50.0;

        // DC offset calculation
        let dc = samples.iter().sum::<f32>() / samples.len() as f32;
        self.dc_offset = self.dc_offset * 0.95 + dc * 0.05;

        // Simplified LUFS calculation (K-weighting approximation)
        // Real LUFS needs proper K-weighting filter, this is simplified
        let lufs_instant = -0.691 + 10.0 * (sum_squared / samples.len() as f32).log10();
        self.lufs_short = self.lufs_short * 0.9 + lufs_instant * 0.1;
        self.lufs_integrated = self.lufs_integrated * 0.99 + lufs_instant * 0.01;

        // Simplified spectrum analysis (8 bands)
        self.update_spectrum(samples);
    }

    /// Estimate inter-sample true peak
    fn estimate_true_peak(&self, samples: &[f32]) -> f32 {
        let mut max_peak = 0.0f32;
        for window in samples.windows(2) {
            let a = window[0];
            let b = window[1];
            max_peak = max_peak.max(a.abs()).max(b.abs());
            // Simple linear interpolation between samples
            let mid = (a + b) / 2.0;
            max_peak = max_peak.max(mid.abs());
        }
        max_peak
    }

    /// Update spectrum bins (simplified 8-band analysis)
    fn update_spectrum(&mut self, samples: &[f32]) {
        // Very simplified - just split into 8 RMS bands
        // Real implementation would use FFT
        let band_size = samples.len() / 8;
        if band_size == 0 {
            return;
        }

        for (i, band) in self.spectrum_bins.iter_mut().enumerate() {
            let start = i * band_size;
            let end = (start + band_size).min(samples.len());
            let band_samples = &samples[start..end];

            let rms: f32 = (band_samples.iter().map(|s| s * s).sum::<f32>()
                / band_samples.len() as f32).sqrt();

            // Smooth the spectrum
            *band = *band * 0.8 + rms * 0.2;
        }
    }

    /// Get waveform data for visualization
    pub fn get_waveform(&self, width: usize) -> Vec<(f32, f32)> {
        if width == 0 || self.buffer_size == 0 {
            return vec![];
        }

        let samples_per_pixel = self.buffer_size / width;
        let mut result = Vec::with_capacity(width);

        for i in 0..width {
            let start = i * samples_per_pixel;
            let end = (start + samples_per_pixel).min(self.buffer_size);

            let mut min = 0.0f32;
            let mut max = 0.0f32;

            for j in start..end {
                let idx = (self.buffer_pos + j) % self.buffer_size;
                let sample = self.waveform_buffer[idx];
                min = min.min(sample);
                max = max.max(sample);
            }

            result.push((min, max));
        }

        result
    }

    /// Check if there's a clipping warning
    pub fn has_clipping_warning(&self) -> bool {
        self.time_since_clip < 5.0
    }

    /// Check if headroom is critically low
    pub fn low_headroom(&self) -> bool {
        self.headroom_db < 3.0
    }

    /// Get quality rating (0-100)
    pub fn quality_score(&self) -> u8 {
        let mut score = 100u8;

        // Penalize clipping
        if self.clip_count > 0 {
            score = score.saturating_sub((self.clip_count * 5).min(50) as u8);
        }

        // Penalize low headroom
        if self.headroom_db < 3.0 {
            score = score.saturating_sub(20);
        } else if self.headroom_db < 6.0 {
            score = score.saturating_sub(10);
        }

        // Penalize high DC offset
        if self.dc_offset.abs() > 0.01 {
            score = score.saturating_sub(10);
        }

        score
    }
}

/// Analysis panel tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordAnalysisTab {
    /// Input signal analysis
    Input,
    /// Recording quality metrics
    Quality,
    /// Session health overview
    Session,
}

/// State for the record analysis panel
pub struct RecordAnalysisState {
    /// Per-input analysis data
    pub input_analysis: Vec<InputAnalysis>,
    /// Currently selected input for detailed view
    pub selected_input: usize,
    /// Active analysis tab
    pub active_tab: RecordAnalysisTab,
    /// Show analysis panel
    pub show_panel: bool,
    /// Show waveform overlay on tracks
    pub show_waveform_overlay: bool,
    /// Total recording time for this session
    pub total_record_time: f32,
    /// Session start time (for elapsed time display)
    pub session_start_time: f64,
    /// Total clips in session across all inputs
    pub total_clips: u32,
    /// Best input quality score in session
    pub best_quality: u8,
    /// Worst input quality score in session
    pub worst_quality: u8,
}

impl Default for RecordAnalysisState {
    fn default() -> Self {
        Self::new(4) // Default to 4 inputs
    }
}

impl RecordAnalysisState {
    pub fn new(num_inputs: usize) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        Self {
            input_analysis: (0..num_inputs).map(|_| InputAnalysis::new(1024)).collect(),
            selected_input: 0,
            active_tab: RecordAnalysisTab::Input,
            show_panel: false,
            show_waveform_overlay: true,
            total_record_time: 0.0,
            session_start_time: now,
            total_clips: 0,
            best_quality: 100,
            worst_quality: 100,
        }
    }

    /// Reset all analysis data
    pub fn reset(&mut self) {
        for analysis in &mut self.input_analysis {
            analysis.reset();
        }
        self.total_record_time = 0.0;
        self.total_clips = 0;
        self.best_quality = 100;
        self.worst_quality = 100;
    }

    /// Update session statistics from input analyses
    pub fn update_session_stats(&mut self) {
        self.total_clips = self.input_analysis.iter().map(|a| a.clip_count).sum();

        if !self.input_analysis.is_empty() {
            self.best_quality = self.input_analysis.iter()
                .map(|a| a.quality_score())
                .max()
                .unwrap_or(100);
            self.worst_quality = self.input_analysis.iter()
                .map(|a| a.quality_score())
                .min()
                .unwrap_or(100);
        }
    }

    /// Get overall session health status
    pub fn session_health(&self) -> SessionHealth {
        if self.total_clips > 10 {
            SessionHealth::Critical
        } else if self.total_clips > 0 || self.worst_quality < 70 {
            SessionHealth::Warning
        } else if self.worst_quality < 85 {
            SessionHealth::Caution
        } else {
            SessionHealth::Good
        }
    }

    /// Check if any input has a clipping warning
    pub fn any_clipping(&self) -> bool {
        self.input_analysis.iter().any(|a| a.has_clipping_warning())
    }

    /// Get the input with the most recent clip
    pub fn most_recent_clip_input(&self) -> Option<usize> {
        self.input_analysis.iter()
            .enumerate()
            .filter(|(_, a)| a.has_clipping_warning())
            .min_by(|(_, a), (_, b)| a.time_since_clip.partial_cmp(&b.time_since_clip).unwrap())
            .map(|(i, _)| i)
    }
}

/// Session health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionHealth {
    /// All good
    Good,
    /// Minor issues
    Caution,
    /// Significant issues
    Warning,
    /// Critical problems
    Critical,
}

impl SessionHealth {
    pub fn color(&self) -> Color32 {
        match self {
            Self::Good => Color32::from_rgb(46, 204, 113),
            Self::Caution => Color32::from_rgb(241, 196, 15),
            Self::Warning => Color32::from_rgb(230, 126, 34),
            Self::Critical => Color32::from_rgb(231, 76, 60),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Good => "Good",
            Self::Caution => "Caution",
            Self::Warning => "Warning",
            Self::Critical => "Critical",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Good => "✓",
            Self::Caution => "◐",
            Self::Warning => "⚠",
            Self::Critical => "✕",
        }
    }
}

/// Audio input configuration
#[derive(Clone)]
pub struct AudioInput {
    /// Input name
    pub name: String,
    /// Is this input enabled
    pub enabled: bool,
    /// Input gain in dB
    pub gain_db: f32,
    /// Current input level
    pub level_db: f32,
    /// Peak level
    pub peak_db: f32,
    /// Is mono
    pub is_mono: bool,
    /// Monitor enabled
    pub monitor: bool,
}

impl Default for AudioInput {
    fn default() -> Self {
        Self {
            name: "Input 1".to_string(),
            enabled: true,
            gain_db: 0.0,
            level_db: -60.0,
            peak_db: -60.0,
            is_mono: true,
            monitor: false,
        }
    }
}

impl AudioInput {
    pub fn new(name: impl Into<String>, is_mono: bool) -> Self {
        Self {
            name: name.into(),
            is_mono,
            ..Default::default()
        }
    }
}

/// Recording track
#[derive(Clone)]
pub struct RecordTrack {
    /// Track name
    pub name: String,
    /// Track color
    pub color: Color32,
    /// Selected input index
    pub input_index: usize,
    /// Is armed for recording
    pub armed: bool,
    /// Is muted
    pub muted: bool,
    /// Has recorded audio
    pub has_audio: bool,
    /// Recorded duration in seconds
    pub duration_secs: f32,
}

impl Default for RecordTrack {
    fn default() -> Self {
        Self {
            name: "Track 1".to_string(),
            color: Color32::from_rgb(52, 152, 219),
            input_index: 0,
            armed: false,
            muted: false,
            has_audio: false,
            duration_secs: 0.0,
        }
    }
}

impl RecordTrack {
    pub fn new(name: impl Into<String>, color: Color32) -> Self {
        Self {
            name: name.into(),
            color,
            ..Default::default()
        }
    }
}

/// Recording state
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RecordingState {
    Stopped,
    Armed,
    Recording,
    Paused,
    Playing,
}

/// Actions that can be returned from the record view
#[derive(Debug, Clone, PartialEq)]
pub enum RecordViewAction {
    /// No action
    None,
    /// Return to tab editor / composition view
    BackToComposition,
}

/// State for the record view
pub struct RecordViewState {
    /// Available audio inputs
    pub inputs: Vec<AudioInput>,
    /// Recording tracks
    pub tracks: Vec<RecordTrack>,
    /// Current recording state
    pub recording_state: RecordingState,
    /// Current position in seconds
    pub position_secs: f32,
    /// Tempo BPM
    pub tempo: u16,
    /// Time signature
    pub time_sig_num: u8,
    pub time_sig_denom: u8,
    /// Metronome enabled
    pub metronome: bool,
    /// Count-in enabled
    pub count_in: bool,
    /// Count-in bars
    pub count_in_bars: u8,
    /// Pre-roll enabled
    pub pre_roll: bool,
    /// Pre-roll bars
    pub pre_roll_bars: u8,
    /// Punch in enabled
    pub punch_in: bool,
    /// Punch in position (seconds)
    pub punch_in_pos: f32,
    /// Punch out enabled
    pub punch_out: bool,
    /// Punch out position (seconds)
    pub punch_out_pos: f32,
    /// Show input panel
    pub show_inputs: bool,
    /// Show template selector
    pub show_template_selector: bool,
    /// Currently selected template index (for highlighting)
    pub selected_template: Option<usize>,
    /// Track being renamed (index)
    pub renaming_track: Option<usize>,
    /// Rename buffer
    pub rename_buffer: String,
    /// Solo'd track index (None = no solo)
    pub solo_track: Option<usize>,
    /// Tap tempo timestamps (for averaging)
    pub tap_times: Vec<f64>,
    /// Last tap time
    pub last_tap_time: f64,
    /// Signal analysis state
    pub analysis: RecordAnalysisState,
}

impl Default for RecordViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordViewState {
    pub fn new() -> Self {
        // Create default inputs (simulated audio interface)
        let inputs = vec![
            AudioInput::new("Mic 1 (Mono)", true),
            AudioInput::new("Mic 2 (Mono)", true),
            AudioInput::new("Guitar DI", true),
            AudioInput::new("Stereo In L/R", false),
        ];

        // Create default tracks
        let tracks = vec![
            RecordTrack::new("Vocals", Color32::from_rgb(231, 76, 60)),
            RecordTrack::new("Guitar", Color32::from_rgb(26, 123, 93)),
            RecordTrack::new("Bass", Color32::from_rgb(155, 89, 182)),
        ];

        Self {
            inputs,
            tracks,
            recording_state: RecordingState::Stopped,
            position_secs: 0.0,
            tempo: 120,
            time_sig_num: 4,
            time_sig_denom: 4,
            metronome: true,
            count_in: true,
            count_in_bars: 1,
            pre_roll: false,
            pre_roll_bars: 2,
            punch_in: false,
            punch_in_pos: 0.0,
            punch_out: false,
            punch_out_pos: 10.0,
            show_inputs: true,
            show_template_selector: false,
            selected_template: None,
            renaming_track: None,
            rename_buffer: String::new(),
            solo_track: None,
            tap_times: Vec::new(),
            last_tap_time: 0.0,
            analysis: RecordAnalysisState::new(4), // 4 inputs
        }
    }

    /// Handle tap tempo - call when user taps
    pub fn tap_tempo(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        // Reset if more than 2 seconds since last tap
        if now - self.last_tap_time > 2.0 {
            self.tap_times.clear();
        }

        self.tap_times.push(now);
        self.last_tap_time = now;

        // Keep only last 8 taps
        if self.tap_times.len() > 8 {
            self.tap_times.remove(0);
        }

        // Calculate average BPM from intervals
        if self.tap_times.len() >= 2 {
            let mut total_interval = 0.0;
            for i in 1..self.tap_times.len() {
                total_interval += self.tap_times[i] - self.tap_times[i - 1];
            }
            let avg_interval = total_interval / (self.tap_times.len() - 1) as f64;
            if avg_interval > 0.0 {
                let bpm = (60.0 / avg_interval).round() as u16;
                self.tempo = bpm.clamp(20, 300);
            }
        }
    }

    /// Arm all tracks
    pub fn arm_all(&mut self) {
        for track in &mut self.tracks {
            track.armed = true;
        }
    }

    /// Disarm all tracks
    pub fn disarm_all(&mut self) {
        for track in &mut self.tracks {
            track.armed = false;
        }
    }

    /// Delete track at index
    pub fn delete_track(&mut self, index: usize) {
        if self.tracks.len() > 1 && index < self.tracks.len() {
            self.tracks.remove(index);
            // Clear solo if it was the solo'd track
            if self.solo_track == Some(index) {
                self.solo_track = None;
            } else if let Some(solo) = self.solo_track {
                if solo > index {
                    self.solo_track = Some(solo - 1);
                }
            }
        }
    }

    /// Toggle solo on track
    pub fn toggle_solo(&mut self, index: usize) {
        if self.solo_track == Some(index) {
            self.solo_track = None;
        } else {
            self.solo_track = Some(index);
        }
    }

    /// Start renaming a track
    pub fn start_rename(&mut self, index: usize) {
        if index < self.tracks.len() {
            self.renaming_track = Some(index);
            self.rename_buffer = self.tracks[index].name.clone();
        }
    }

    /// Finish renaming
    pub fn finish_rename(&mut self) {
        if let Some(index) = self.renaming_track {
            if index < self.tracks.len() && !self.rename_buffer.is_empty() {
                self.tracks[index].name = self.rename_buffer.clone();
            }
        }
        self.renaming_track = None;
        self.rename_buffer.clear();
    }

    /// Cancel renaming
    pub fn cancel_rename(&mut self) {
        self.renaming_track = None;
        self.rename_buffer.clear();
    }

    /// Toggle playback
    pub fn toggle_play(&mut self) {
        match self.recording_state {
            RecordingState::Stopped => {
                self.recording_state = RecordingState::Playing;
            }
            RecordingState::Playing => {
                self.recording_state = RecordingState::Stopped;
            }
            RecordingState::Recording => {
                self.recording_state = RecordingState::Stopped;
            }
            _ => {}
        }
    }

    /// Update simulated input levels and analysis data
    pub fn update_levels(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as f32 / 1000.0)
            .unwrap_or(0.0);

        let dt = 1.0 / 60.0; // Assume ~60 FPS update rate

        for (i, input) in self.inputs.iter_mut().enumerate() {
            if input.enabled {
                let base = -30.0 + (i as f32 * 3.0);
                let variation = ((t * (2.0 + i as f32 * 0.3)).sin() * 15.0)
                    + ((t * 4.7).sin() * 8.0);
                input.level_db = (base + variation + input.gain_db).clamp(-60.0, 0.0);
                input.peak_db = input.peak_db.max(input.level_db) - 0.1;

                // Generate simulated waveform samples for analysis
                if i < self.analysis.input_analysis.len() {
                    let analysis = &mut self.analysis.input_analysis[i];

                    // Generate ~64 samples per frame for visualization
                    let mut samples = Vec::with_capacity(64);
                    for j in 0..64 {
                        let sample_t = t + (j as f32 * 0.0001);
                        // Generate complex waveform with multiple harmonics
                        let freq1 = 110.0 * (1.0 + i as f32 * 0.5); // Base frequency varies per input
                        let freq2 = freq1 * 2.0; // First harmonic
                        let freq3 = freq1 * 3.0; // Second harmonic

                        let wave = (sample_t * freq1 * std::f32::consts::TAU).sin() * 0.5
                            + (sample_t * freq2 * std::f32::consts::TAU).sin() * 0.25
                            + (sample_t * freq3 * std::f32::consts::TAU).sin() * 0.1;

                        // Apply envelope based on input level
                        let level_linear = 10.0f32.powf(input.level_db / 20.0);
                        let sample = wave * level_linear;

                        // Occasional high transient to simulate clips (rare)
                        let transient = if (sample_t * 17.3).sin() > 0.99 && input.gain_db > 10.0 {
                            0.99
                        } else {
                            sample.clamp(-0.95, 0.95)
                        };

                        samples.push(transient);
                    }

                    analysis.process_samples(&samples, dt);
                }
            } else {
                input.level_db = -60.0;
                input.peak_db = (input.peak_db - 0.5).max(-60.0);
            }
        }

        // Update session stats
        self.analysis.update_session_stats();

        // Update total recording time if recording
        if self.recording_state == RecordingState::Recording {
            self.analysis.total_record_time += dt;
        }
    }

    /// Format position as time string
    pub fn format_position(&self) -> String {
        let mins = (self.position_secs / 60.0) as u32;
        let secs = self.position_secs % 60.0;
        format!("{}:{:05.2}", mins, secs)
    }

    /// Format position as bars:beats
    pub fn format_bars_beats(&self) -> String {
        let beat_duration = 60.0 / self.tempo as f32;
        let total_beats = (self.position_secs / beat_duration) as u32;
        let bar = total_beats / self.time_sig_num as u32 + 1;
        let beat = total_beats % self.time_sig_num as u32 + 1;
        format!("{}:{}", bar, beat)
    }

    /// Check if any track is armed
    pub fn any_armed(&self) -> bool {
        self.tracks.iter().any(|t| t.armed)
    }

    /// Toggle recording
    pub fn toggle_record(&mut self) {
        match self.recording_state {
            RecordingState::Stopped | RecordingState::Paused | RecordingState::Playing => {
                if self.any_armed() {
                    self.recording_state = RecordingState::Recording;
                }
            }
            RecordingState::Armed => {
                self.recording_state = RecordingState::Recording;
            }
            RecordingState::Recording => {
                self.recording_state = RecordingState::Stopped;
            }
        }
    }

    /// Stop recording
    pub fn stop(&mut self) {
        self.recording_state = RecordingState::Stopped;
        self.position_secs = 0.0;
    }
}

/// Record mode view component
pub struct RecordView<'a> {
    state: &'a mut RecordViewState,
    theme: &'a Theme,
}

impl<'a> RecordView<'a> {
    pub fn new(state: &'a mut RecordViewState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    pub fn show(&mut self, ui: &mut Ui) -> RecordViewAction {
        let mut action = RecordViewAction::None;

        // Update levels
        self.state.update_levels();
        ui.ctx().request_repaint();

        // Handle keyboard shortcuts
        action = self.handle_keyboard(ui, action);

        // Show analysis panel if enabled (as side panel)
        if self.state.analysis.show_panel {
            self.show_analysis_panel(ui);
        }

        // Main layout
        ui.vertical(|ui| {
            // Transport bar
            self.show_transport_bar(ui, &mut action);

            ui.separator();

            // Template selector (if shown)
            if self.state.show_template_selector {
                self.show_template_selector(ui);
                ui.separator();
            }

            // Clipping alert banner (shown prominently when recording)
            if self.state.recording_state == RecordingState::Recording && self.state.analysis.any_clipping() {
                ui.horizontal(|ui| {
                    let clip_color = Color32::from_rgb(231, 76, 60);
                    ui.label(egui::RichText::new("⚠ CLIPPING DETECTED")
                        .color(clip_color)
                        .strong());

                    if let Some(input_idx) = self.state.analysis.most_recent_clip_input() {
                        if let Some(input) = self.state.inputs.get(input_idx) {
                            ui.label(egui::RichText::new(format!("on {}", input.name))
                                .color(clip_color)
                                .small());
                        }
                    }

                    ui.label(egui::RichText::new("- Reduce input gain!")
                        .color(clip_color)
                        .small());
                });
                ui.separator();
            }

            // Main content
            ui.horizontal(|ui| {
                // Input panel (collapsible)
                if self.state.show_inputs {
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(180.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            self.show_input_panel(ui);
                        },
                    );
                    ui.separator();
                }

                // Track arrangement area
                ui.vertical(|ui| {
                    self.show_track_area(ui);
                });
            });

            // Status bar with analysis toggle
            ui.separator();
            self.show_record_status_bar(ui);
        });

        action
    }

    /// Show status bar with analysis toggle and session info
    fn show_record_status_bar(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Session health indicator
            let health = self.state.analysis.session_health();
            ui.label(egui::RichText::new(format!("{} {}", health.icon(), health.label()))
                .small()
                .color(health.color()));

            ui.separator();

            // Recording time
            let mins = (self.state.analysis.total_record_time / 60.0) as u32;
            let secs = self.state.analysis.total_record_time % 60.0;
            ui.label(egui::RichText::new(format!("Recorded: {}:{:05.2}", mins, secs))
                .small()
                .monospace());

            // Clip count
            if self.state.analysis.total_clips > 0 {
                ui.separator();
                ui.label(egui::RichText::new(format!("⚠ {} clips", self.state.analysis.total_clips))
                    .small()
                    .color(Color32::from_rgb(231, 76, 60)));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Keyboard shortcuts hint
                ui.label(egui::RichText::new("A: Analysis  W: Waveform  M: Metro  T: Tap")
                    .small()
                    .color(self.theme.palette.text_secondary));
            });
        });
    }

    /// Handle keyboard shortcuts
    fn handle_keyboard(&mut self, ui: &mut Ui, mut action: RecordViewAction) -> RecordViewAction {
        // Don't handle keys if renaming a track
        if self.state.renaming_track.is_some() {
            return action;
        }

        ui.input(|i| {
            // Space -> toggle play/record
            if i.key_pressed(egui::Key::Space) {
                if self.state.any_armed() {
                    self.state.toggle_record();
                } else {
                    self.state.toggle_play();
                }
            }

            // R -> toggle arm on first un-armed track (or disarm all if all armed)
            if i.key_pressed(egui::Key::R) && !i.modifiers.ctrl {
                if self.state.tracks.iter().all(|t| t.armed) {
                    self.state.disarm_all();
                } else {
                    // Arm first unarmed track
                    if let Some(track) = self.state.tracks.iter_mut().find(|t| !t.armed) {
                        track.armed = true;
                    }
                }
            }

            // Escape -> stop or back to composition
            if i.key_pressed(egui::Key::Escape) {
                if self.state.recording_state == RecordingState::Recording
                    || self.state.recording_state == RecordingState::Playing
                {
                    self.state.stop();
                } else {
                    action = RecordViewAction::BackToComposition;
                }
            }

            // T -> tap tempo
            if i.key_pressed(egui::Key::T) && !i.modifiers.ctrl {
                self.state.tap_tempo();
            }

            // M -> toggle metronome
            if i.key_pressed(egui::Key::M) && !i.modifiers.ctrl {
                self.state.metronome = !self.state.metronome;
            }

            // A -> toggle analysis panel
            if i.key_pressed(egui::Key::A) && !i.modifiers.ctrl {
                self.state.analysis.show_panel = !self.state.analysis.show_panel;
            }

            // W -> toggle waveform overlay
            if i.key_pressed(egui::Key::W) && !i.modifiers.ctrl {
                self.state.analysis.show_waveform_overlay = !self.state.analysis.show_waveform_overlay;
            }

            // 1/2/3 -> switch analysis tabs (when panel is open)
            if self.state.analysis.show_panel {
                if i.key_pressed(egui::Key::Num1) {
                    self.state.analysis.active_tab = RecordAnalysisTab::Input;
                }
                if i.key_pressed(egui::Key::Num2) {
                    self.state.analysis.active_tab = RecordAnalysisTab::Quality;
                }
                if i.key_pressed(egui::Key::Num3) {
                    self.state.analysis.active_tab = RecordAnalysisTab::Session;
                }

                // Left/Right arrow to cycle through inputs
                if i.key_pressed(egui::Key::ArrowLeft) {
                    let num_inputs = self.state.analysis.input_analysis.len();
                    if num_inputs > 0 {
                        self.state.analysis.selected_input =
                            (self.state.analysis.selected_input + num_inputs - 1) % num_inputs;
                    }
                }
                if i.key_pressed(egui::Key::ArrowRight) {
                    let num_inputs = self.state.analysis.input_analysis.len();
                    if num_inputs > 0 {
                        self.state.analysis.selected_input =
                            (self.state.analysis.selected_input + 1) % num_inputs;
                    }
                }
            }
        });

        action
    }

    /// Show template selector panel
    fn show_template_selector(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Quick Setup:").strong());

            let presets = SessionTemplate::presets();
            for (i, preset) in presets.iter().enumerate() {
                let is_selected = self.state.selected_template == Some(i);
                let btn = ui.selectable_label(is_selected, &preset.name);
                if btn.on_hover_text(&preset.description).clicked() {
                    // Apply template
                    *self.state = RecordViewState::from_template(preset);
                    self.state.selected_template = Some(i);
                    self.state.show_template_selector = false;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").on_hover_text("Close template selector").clicked() {
                    self.state.show_template_selector = false;
                }
            });
        });
    }

    fn show_transport_bar(&mut self, ui: &mut Ui, action: &mut RecordViewAction) {
        ui.horizontal(|ui| {
            // Back to composition button
            if ui.button("← Back")
                .on_hover_text("Return to Tab Editor (Esc)")
                .clicked()
            {
                *action = RecordViewAction::BackToComposition;
            }

            ui.separator();

            // Transport controls
            let is_recording = self.state.recording_state == RecordingState::Recording;
            let is_playing = self.state.recording_state == RecordingState::Playing;

            // Stop button
            if ui.add(egui::Button::new(
                egui::RichText::new("⏹").size(14.0)
            ).min_size(egui::Vec2::new(32.0, 28.0)))
                .on_hover_text("Stop (Space)")
                .clicked()
            {
                self.state.stop();
            }

            // Play button
            let play_icon = if is_playing { "⏸" } else { "▶" };
            let play_color = if is_playing {
                self.theme.palette.success
            } else {
                self.theme.palette.text_primary
            };
            if ui.add(egui::Button::new(
                egui::RichText::new(play_icon).color(play_color).size(14.0)
            ).min_size(egui::Vec2::new(32.0, 28.0)))
                .on_hover_text("Play/Pause (Space when no tracks armed)")
                .clicked()
            {
                self.state.toggle_play();
            }

            // Record button
            let rec_color = if is_recording {
                Color32::from_rgb(231, 76, 60)  // Pulsing red
            } else if self.state.any_armed() {
                Color32::from_rgb(200, 60, 60)  // Armed red
            } else {
                self.theme.palette.text_secondary
            };

            if ui.add(egui::Button::new(
                egui::RichText::new("⏺").color(rec_color).size(14.0)
            ).min_size(egui::Vec2::new(32.0, 28.0)))
                .on_hover_text("Record (Space when tracks armed)")
                .clicked()
            {
                self.state.toggle_record();
            }

            ui.separator();

            // Position display
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(self.state.format_position())
                        .monospace()
                        .size(16.0)
                        .color(self.theme.palette.text_primary)
                );
                ui.label(
                    egui::RichText::new(self.state.format_bars_beats())
                        .monospace()
                        .size(11.0)
                        .color(self.theme.palette.text_secondary)
                );
            });

            ui.separator();

            // Tempo with tap tempo
            ui.label("BPM:");
            ui.add(egui::DragValue::new(&mut self.state.tempo)
                .range(20..=300)
                .speed(1.0))
                .on_hover_text("Tempo in beats per minute");

            // Tap tempo button
            if ui.button("Tap")
                .on_hover_text("Tap to set tempo (T key)")
                .clicked()
            {
                self.state.tap_tempo();
            }

            // Time signature
            ui.label("Time:");
            ui.add(egui::DragValue::new(&mut self.state.time_sig_num)
                .range(1..=16))
                .on_hover_text("Beats per measure");
            ui.label("/");
            ui.add(egui::DragValue::new(&mut self.state.time_sig_denom)
                .range(1..=16))
                .on_hover_text("Beat unit (4 = quarter note)");

            ui.separator();

            // Metronome
            let metro_color = if self.state.metronome {
                self.theme.palette.accent
            } else {
                self.theme.palette.text_secondary
            };
            if ui.add(egui::Button::new(
                egui::RichText::new("🔔").color(metro_color)
            ))
                .on_hover_text("Toggle metronome (M key)")
                .clicked()
            {
                self.state.metronome = !self.state.metronome;
            }

            // Count-in
            ui.checkbox(&mut self.state.count_in, "Count-in")
                .on_hover_text("Play metronome clicks before recording starts");
            if self.state.count_in {
                ui.add(egui::DragValue::new(&mut self.state.count_in_bars)
                    .range(1..=4)
                    .suffix(" bars"))
                    .on_hover_text("Number of count-in bars");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Toggle inputs panel
                ui.toggle_value(&mut self.state.show_inputs, "Inputs")
                    .on_hover_text("Show/hide audio input panel");

                // Template selector toggle
                ui.toggle_value(&mut self.state.show_template_selector, "Templates")
                    .on_hover_text("Quick session setup templates");

                ui.separator();

                // Arm all / Disarm all
                if ui.small_button("Arm All")
                    .on_hover_text("Arm all tracks for recording")
                    .clicked()
                {
                    self.state.arm_all();
                }
                if ui.small_button("Disarm")
                    .on_hover_text("Disarm all tracks")
                    .clicked()
                {
                    self.state.disarm_all();
                }

                ui.separator();

                // Recording/Playing indicator
                if is_recording {
                    ui.label(
                        egui::RichText::new("⏺ RECORDING")
                            .color(Color32::from_rgb(231, 76, 60))
                            .strong()
                    );
                } else if is_playing {
                    ui.label(
                        egui::RichText::new("▶ PLAYING")
                            .color(self.theme.palette.success)
                            .strong()
                    );
                }
            });
        });
    }

    fn show_input_panel(&mut self, ui: &mut Ui) {
        ui.heading("Inputs");
        ui.separator();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for input in &mut self.state.inputs {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut input.enabled, "");
                            ui.label(
                                egui::RichText::new(&input.name)
                                    .color(if input.enabled {
                                        self.theme.palette.text_primary
                                    } else {
                                        self.theme.palette.text_secondary
                                    })
                            );
                        });

                        if input.enabled {
                            ui.horizontal(|ui| {
                                // Level meter
                                LevelMeter::new(input.level_db)
                                    .peak(input.peak_db)
                                    .width(10.0)
                                    .height(80.0)
                                    .show(ui);

                                ui.vertical(|ui| {
                                    // Gain knob
                                    Knob::new(&mut input.gain_db)
                                        .range(-24.0, 24.0)
                                        .default_value(0.0)
                                        .size(35.0)
                                        .label("Gain")
                                        .show(ui);

                                    // Monitor toggle
                                    let mon_color = if input.monitor {
                                        self.theme.palette.accent
                                    } else {
                                        self.theme.palette.text_secondary
                                    };
                                    if ui.add(egui::Button::new(
                                        egui::RichText::new("MON").size(10.0).color(mon_color)
                                    ).min_size(egui::Vec2::new(35.0, 20.0))).clicked() {
                                        input.monitor = !input.monitor;
                                    }
                                });
                            });

                            // Level readout
                            ui.label(
                                egui::RichText::new(format!("{:.1} dB", input.level_db))
                                    .size(10.0)
                                    .color(self.theme.palette.text_secondary)
                            );
                        }
                    });
                    ui.add_space(4.0);
                }

                ui.separator();

                // Peak reset
                if ui.button("Reset Peaks").on_hover_text("Clear peak hold indicators").clicked() {
                    for input in &mut self.state.inputs {
                        input.peak_db = -60.0;
                    }
                }
            });
    }

    fn show_track_area(&mut self, ui: &mut Ui) {
        // Track header
        ui.horizontal(|ui| {
            ui.heading("Tracks");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("+ Add Track").on_hover_text("Create a new recording track").clicked() {
                    let num = self.state.tracks.len() + 1;
                    let colors = [
                        Color32::from_rgb(231, 76, 60),
                        Color32::from_rgb(52, 152, 219),
                        Color32::from_rgb(46, 204, 113),
                        Color32::from_rgb(155, 89, 182),
                        Color32::from_rgb(241, 196, 15),
                    ];
                    let color = colors[(num - 1) % colors.len()];
                    self.state.tracks.push(RecordTrack::new(format!("Track {}", num), color));
                }
            });
        });

        ui.separator();

        // Track list with management actions
        let is_recording = self.state.recording_state == RecordingState::Recording;
        let inputs: Vec<_> = self.state.inputs.iter()
            .map(|i| i.name.clone())
            .collect();
        let theme = self.theme;
        let solo_track = self.state.solo_track;
        let track_count = self.state.tracks.len();
        let renaming_track = self.state.renaming_track;

        // Collect actions to apply after iteration
        let mut action_delete: Option<usize> = None;
        let mut action_solo: Option<usize> = None;
        let mut action_rename_start: Option<usize> = None;

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (idx, track) in self.state.tracks.iter_mut().enumerate() {
                    let is_solo = solo_track == Some(idx);
                    let is_renaming = renaming_track == Some(idx);

                    let track_action = draw_track_row_with_actions(
                        ui, track, &inputs, is_recording, theme,
                        idx, is_solo, is_renaming, track_count > 1,
                        &mut self.state.rename_buffer,
                    );

                    match track_action {
                        TrackRowAction::Delete => action_delete = Some(idx),
                        TrackRowAction::Solo => action_solo = Some(idx),
                        TrackRowAction::StartRename => action_rename_start = Some(idx),
                        TrackRowAction::FinishRename => {
                            if !self.state.rename_buffer.is_empty() {
                                track.name = self.state.rename_buffer.clone();
                            }
                            self.state.renaming_track = None;
                            self.state.rename_buffer.clear();
                        }
                        TrackRowAction::CancelRename => {
                            self.state.renaming_track = None;
                            self.state.rename_buffer.clear();
                        }
                        TrackRowAction::None => {}
                    }
                }
            });

        // Apply collected actions
        if let Some(idx) = action_delete {
            self.state.delete_track(idx);
        }
        if let Some(idx) = action_solo {
            self.state.toggle_solo(idx);
        }
        if let Some(idx) = action_rename_start {
            self.state.start_rename(idx);
        }
    }

    // ========================================================================
    // Analysis Panel UI
    // ========================================================================

    /// Show the signal analysis panel
    fn show_analysis_panel(&mut self, ui: &mut Ui) {
        let panel_width = 320.0;

        egui::SidePanel::right("record_analysis_panel")
            .default_width(panel_width)
            .min_width(280.0)
            .max_width(450.0)
            .show_inside(ui, |ui| {
                // Header with session health indicator
                ui.horizontal(|ui| {
                    ui.heading("📊 Analysis");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Session health badge
                        let health = self.state.analysis.session_health();
                        let health_text = format!("{} {}", health.icon(), health.label());
                        ui.label(egui::RichText::new(health_text)
                            .color(health.color())
                            .small());

                        // Close button
                        if ui.small_button("✕").on_hover_text("Close panel (A)").clicked() {
                            self.state.analysis.show_panel = false;
                        }
                    });
                });

                ui.separator();

                // Tab bar
                ui.horizontal(|ui| {
                    let tabs = [
                        (RecordAnalysisTab::Input, "📈 Input", "Input signal analysis (1)"),
                        (RecordAnalysisTab::Quality, "✓ Quality", "Recording quality (2)"),
                        (RecordAnalysisTab::Session, "📋 Session", "Session overview (3)"),
                    ];

                    for (tab, label, tooltip) in tabs {
                        let is_selected = self.state.analysis.active_tab == tab;
                        if ui.selectable_label(is_selected, label)
                            .on_hover_text(tooltip)
                            .clicked()
                        {
                            self.state.analysis.active_tab = tab;
                        }
                    }
                });

                ui.separator();

                // Tab content
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        match self.state.analysis.active_tab {
                            RecordAnalysisTab::Input => self.show_input_analysis_tab(ui),
                            RecordAnalysisTab::Quality => self.show_quality_tab(ui),
                            RecordAnalysisTab::Session => self.show_session_tab(ui),
                        }
                    });

                // Status bar
                ui.separator();
                self.show_analysis_status_bar(ui);
            });
    }

    /// Show input analysis tab
    fn show_input_analysis_tab(&self, ui: &mut Ui) {
        let selected = self.state.analysis.selected_input;
        let input_names: Vec<_> = self.state.inputs.iter().map(|i| i.name.clone()).collect();

        // Input selector
        ui.horizontal(|ui| {
            ui.label("Input:");
            for (i, name) in input_names.iter().enumerate() {
                let is_selected = i == selected;
                let btn = ui.selectable_label(is_selected, &**name);
                if btn.clicked() {
                    // Note: can't mutate here, would need to handle in parent
                }
            }
        });

        ui.add_space(8.0);

        // Get analysis for selected input
        if let Some(analysis) = self.state.analysis.input_analysis.get(selected) {
            // Waveform visualization
            ui.group(|ui| {
                ui.label(egui::RichText::new("Waveform").strong());
                self.draw_waveform(ui, analysis);
            });

            ui.add_space(8.0);

            // Level meters
            ui.group(|ui| {
                ui.label(egui::RichText::new("Levels").strong());
                ui.add_space(4.0);

                egui::Grid::new("level_meters")
                    .num_columns(3)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        // Peak level
                        ui.label("Peak:");
                        let peak_color = if analysis.peak_db > -3.0 {
                            Color32::from_rgb(231, 76, 60)
                        } else if analysis.peak_db > -6.0 {
                            Color32::from_rgb(241, 196, 15)
                        } else {
                            self.theme.palette.text_primary
                        };
                        ui.label(egui::RichText::new(format!("{:.1} dB", analysis.peak_db))
                            .color(peak_color)
                            .monospace());
                        self.draw_level_bar(ui, analysis.peak_db, 100.0);
                        ui.end_row();

                        // True peak
                        ui.label("True Peak:");
                        let tp_color = if analysis.true_peak_db > -1.0 {
                            Color32::from_rgb(231, 76, 60)
                        } else {
                            self.theme.palette.text_secondary
                        };
                        ui.label(egui::RichText::new(format!("{:.1} dB", analysis.true_peak_db))
                            .color(tp_color)
                            .monospace());
                        self.draw_level_bar(ui, analysis.true_peak_db, 100.0);
                        ui.end_row();

                        // RMS
                        ui.label("RMS:");
                        ui.label(egui::RichText::new(format!("{:.1} dB", analysis.rms_db))
                            .monospace());
                        self.draw_level_bar(ui, analysis.rms_db, 100.0);
                        ui.end_row();

                        // Headroom
                        ui.label("Headroom:");
                        let hr_color = if analysis.headroom_db < 3.0 {
                            Color32::from_rgb(231, 76, 60)
                        } else if analysis.headroom_db < 6.0 {
                            Color32::from_rgb(241, 196, 15)
                        } else {
                            Color32::from_rgb(46, 204, 113)
                        };
                        ui.label(egui::RichText::new(format!("{:.1} dB", analysis.headroom_db))
                            .color(hr_color)
                            .monospace());
                        ui.label("");
                        ui.end_row();
                    });
            });

            ui.add_space(8.0);

            // LUFS metering
            ui.group(|ui| {
                ui.label(egui::RichText::new("Loudness (LUFS)").strong());
                ui.add_space(4.0);

                egui::Grid::new("lufs_meters")
                    .num_columns(2)
                    .spacing([16.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Short-term:");
                        ui.label(egui::RichText::new(format!("{:.1} LUFS", analysis.lufs_short))
                            .monospace());
                        ui.end_row();

                        ui.label("Integrated:");
                        ui.label(egui::RichText::new(format!("{:.1} LUFS", analysis.lufs_integrated))
                            .monospace());
                        ui.end_row();

                        ui.label("Dynamic Range:");
                        ui.label(egui::RichText::new(format!("{:.1} dB", analysis.dynamic_range_db))
                            .monospace());
                        ui.end_row();
                    });
            });

            ui.add_space(8.0);

            // Spectrum analyzer
            ui.group(|ui| {
                ui.label(egui::RichText::new("Spectrum").strong());
                self.draw_spectrum(ui, analysis);
            });

            // Clipping alerts
            if analysis.clip_count > 0 {
                ui.add_space(8.0);
                ui.group(|ui| {
                    let recent = analysis.time_since_clip < 5.0;
                    let color = if recent {
                        Color32::from_rgb(231, 76, 60)
                    } else {
                        Color32::from_rgb(241, 196, 15)
                    };

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("⚠").color(color));
                        ui.label(egui::RichText::new(format!(
                            "{} clip{} detected",
                            analysis.clip_count,
                            if analysis.clip_count == 1 { "" } else { "s" }
                        )).color(color));
                    });

                    if recent {
                        ui.label(egui::RichText::new("Recent clipping! Consider reducing gain.")
                            .small()
                            .color(self.theme.palette.text_secondary));
                    }
                });
            }

            // DC offset warning
            if analysis.dc_offset.abs() > 0.005 {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(format!(
                    "⚠ DC offset: {:.3}",
                    analysis.dc_offset
                )).small().color(Color32::from_rgb(241, 196, 15)));
            }
        } else {
            ui.label("No analysis data available");
        }
    }

    /// Show quality metrics tab
    fn show_quality_tab(&self, ui: &mut Ui) {
        // Quality scores per input
        ui.group(|ui| {
            ui.label(egui::RichText::new("Input Quality Scores").strong());
            ui.add_space(4.0);

            for (i, analysis) in self.state.analysis.input_analysis.iter().enumerate() {
                let input_name = self.state.inputs.get(i)
                    .map(|inp| inp.name.as_str())
                    .unwrap_or("Unknown");

                let score = analysis.quality_score();
                let color = if score >= 90 {
                    Color32::from_rgb(46, 204, 113)
                } else if score >= 70 {
                    Color32::from_rgb(241, 196, 15)
                } else if score >= 50 {
                    Color32::from_rgb(230, 126, 34)
                } else {
                    Color32::from_rgb(231, 76, 60)
                };

                ui.horizontal(|ui| {
                    ui.label(input_name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("{}%", score))
                            .color(color)
                            .monospace());

                        // Progress bar
                        let bar_width = 80.0;
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, 12.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, self.theme.palette.bg_tertiary);
                        let fill_width = bar_width * (score as f32 / 100.0);
                        let fill_rect = Rect::from_min_size(
                            rect.min,
                            Vec2::new(fill_width, rect.height())
                        );
                        ui.painter().rect_filled(fill_rect, 2.0, color);
                    });
                });

                ui.add_space(4.0);
            }
        });

        ui.add_space(8.0);

        // Quality issues breakdown
        ui.group(|ui| {
            ui.label(egui::RichText::new("Issues Detected").strong());
            ui.add_space(4.0);

            let mut has_issues = false;

            for (i, analysis) in self.state.analysis.input_analysis.iter().enumerate() {
                let input_name = self.state.inputs.get(i)
                    .map(|inp| inp.name.as_str())
                    .unwrap_or("Unknown");

                let mut issues = Vec::new();

                if analysis.clip_count > 0 {
                    issues.push(format!("{} clips", analysis.clip_count));
                }
                if analysis.headroom_db < 3.0 {
                    issues.push("Low headroom".to_string());
                }
                if analysis.dc_offset.abs() > 0.01 {
                    issues.push("DC offset".to_string());
                }

                if !issues.is_empty() {
                    has_issues = true;
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("⚠")
                            .color(Color32::from_rgb(241, 196, 15)));
                        ui.label(format!("{}: {}", input_name, issues.join(", ")));
                    });
                }
            }

            if !has_issues {
                ui.label(egui::RichText::new("✓ No issues detected")
                    .color(Color32::from_rgb(46, 204, 113)));
            }
        });

        ui.add_space(8.0);

        // Recommendations
        ui.group(|ui| {
            ui.label(egui::RichText::new("Recommendations").strong());
            ui.add_space(4.0);

            let mut recommendations = Vec::new();

            for analysis in &self.state.analysis.input_analysis {
                if analysis.headroom_db < 6.0 && analysis.headroom_db >= 3.0 {
                    recommendations.push("Consider reducing input gain for more headroom");
                }
                if analysis.dynamic_range_db > 30.0 {
                    recommendations.push("High dynamic range - consider compression");
                }
                if analysis.dynamic_range_db < 6.0 && analysis.signal_present {
                    recommendations.push("Low dynamic range - signal may be over-compressed");
                }
            }

            recommendations.dedup();

            if recommendations.is_empty() {
                ui.label(egui::RichText::new("Recording levels look good!")
                    .color(self.theme.palette.text_secondary)
                    .italics());
            } else {
                for rec in recommendations.iter().take(3) {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("→")
                            .color(Color32::from_rgb(52, 152, 219)));
                        ui.label(*rec);
                    });
                }
            }
        });
    }

    /// Show session overview tab
    fn show_session_tab(&self, ui: &mut Ui) {
        let analysis = &self.state.analysis;

        // Session health summary
        ui.group(|ui| {
            let health = analysis.session_health();

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Session Health").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("{} {}", health.icon(), health.label()))
                        .color(health.color())
                        .strong());
                });
            });

            ui.add_space(8.0);

            egui::Grid::new("session_stats")
                .num_columns(2)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Recording Time:");
                    let mins = (analysis.total_record_time / 60.0) as u32;
                    let secs = analysis.total_record_time % 60.0;
                    ui.label(egui::RichText::new(format!("{}:{:05.2}", mins, secs)).monospace());
                    ui.end_row();

                    ui.label("Total Clips:");
                    let clip_color = if analysis.total_clips > 0 {
                        Color32::from_rgb(231, 76, 60)
                    } else {
                        Color32::from_rgb(46, 204, 113)
                    };
                    ui.label(egui::RichText::new(format!("{}", analysis.total_clips))
                        .color(clip_color)
                        .monospace());
                    ui.end_row();

                    ui.label("Best Quality:");
                    ui.label(egui::RichText::new(format!("{}%", analysis.best_quality)).monospace());
                    ui.end_row();

                    ui.label("Worst Quality:");
                    let wq_color = if analysis.worst_quality < 70 {
                        Color32::from_rgb(231, 76, 60)
                    } else if analysis.worst_quality < 85 {
                        Color32::from_rgb(241, 196, 15)
                    } else {
                        self.theme.palette.text_primary
                    };
                    ui.label(egui::RichText::new(format!("{}%", analysis.worst_quality))
                        .color(wq_color)
                        .monospace());
                    ui.end_row();

                    ui.label("Active Inputs:");
                    let active_count = self.state.inputs.iter().filter(|i| i.enabled).count();
                    ui.label(egui::RichText::new(format!("{}", active_count)).monospace());
                    ui.end_row();

                    ui.label("Armed Tracks:");
                    let armed_count = self.state.tracks.iter().filter(|t| t.armed).count();
                    ui.label(egui::RichText::new(format!("{}", armed_count)).monospace());
                    ui.end_row();
                });
        });

        ui.add_space(8.0);

        // Input status summary
        ui.group(|ui| {
            ui.label(egui::RichText::new("Input Status").strong());
            ui.add_space(4.0);

            for (i, input) in self.state.inputs.iter().enumerate() {
                let analysis_opt = self.state.analysis.input_analysis.get(i);

                ui.horizontal(|ui| {
                    // Status indicator
                    let (status_icon, status_color) = if !input.enabled {
                        ("○", self.theme.palette.text_secondary)
                    } else if let Some(a) = analysis_opt {
                        if a.has_clipping_warning() {
                            ("⚠", Color32::from_rgb(231, 76, 60))
                        } else if a.low_headroom() {
                            ("◐", Color32::from_rgb(241, 196, 15))
                        } else if a.signal_present {
                            ("●", Color32::from_rgb(46, 204, 113))
                        } else {
                            ("○", self.theme.palette.text_secondary)
                        }
                    } else {
                        ("○", self.theme.palette.text_secondary)
                    };

                    ui.label(egui::RichText::new(status_icon).color(status_color));
                    ui.label(&input.name);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(a) = analysis_opt {
                            if a.signal_present {
                                ui.label(egui::RichText::new(format!("{:.1} dB", a.peak_db))
                                    .small()
                                    .monospace());
                            }
                        }
                    });
                });
            }
        });

        ui.add_space(8.0);

        // Actions
        ui.group(|ui| {
            ui.label(egui::RichText::new("Actions").strong());
            ui.add_space(4.0);

            if ui.button("Reset Analysis").on_hover_text("Clear all analysis data").clicked() {
                // Would need to handle this through state mutation
            }

            if ui.button("Reset Peak Holds").on_hover_text("Clear peak hold indicators").clicked() {
                // Would need to handle through state
            }
        });
    }

    /// Draw waveform visualization
    fn draw_waveform(&self, ui: &mut Ui, analysis: &InputAnalysis) {
        let available_width = ui.available_width().max(100.0);
        let height = 60.0;

        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(available_width, height),
            egui::Sense::hover()
        );

        // Background
        ui.painter().rect_filled(rect, 4.0, self.theme.palette.bg_tertiary);

        // Center line
        let center_y = rect.center().y;
        ui.painter().hline(
            rect.x_range(),
            center_y,
            Stroke::new(1.0, Color32::from_gray(60))
        );

        // Get waveform data
        let waveform = analysis.get_waveform(available_width as usize);
        let scale = height / 2.0 * 0.9; // Leave some margin

        if !waveform.is_empty() {
            // Draw waveform as filled area
            for (i, (min, max)) in waveform.iter().enumerate() {
                let x = rect.left() + i as f32;
                let y_min = center_y - max * scale;
                let y_max = center_y - min * scale;

                // Color based on level
                let level = max.abs().max(min.abs());
                let color = if level > 0.9 {
                    Color32::from_rgb(231, 76, 60) // Clipping
                } else if level > 0.7 {
                    Color32::from_rgb(241, 196, 15) // Hot
                } else {
                    Color32::from_rgb(46, 204, 113) // Good
                };

                ui.painter().vline(
                    x,
                    egui::Rangef::new(y_min.min(y_max), y_min.max(y_max)),
                    Stroke::new(1.0, color)
                );
            }
        }

        // Clipping threshold lines
        let clip_y_top = center_y - scale * 0.95;
        let clip_y_bottom = center_y + scale * 0.95;
        ui.painter().hline(
            rect.x_range(),
            clip_y_top,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(231, 76, 60, 100))
        );
        ui.painter().hline(
            rect.x_range(),
            clip_y_bottom,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(231, 76, 60, 100))
        );
    }

    /// Draw spectrum analyzer visualization
    fn draw_spectrum(&self, ui: &mut Ui, analysis: &InputAnalysis) {
        let available_width = ui.available_width().max(100.0);
        let height = 50.0;

        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(available_width, height),
            egui::Sense::hover()
        );

        // Background
        ui.painter().rect_filled(rect, 4.0, self.theme.palette.bg_tertiary);

        // Draw spectrum bars
        let num_bins = analysis.spectrum_bins.len();
        let bar_width = available_width / num_bins as f32;
        let max_level = analysis.spectrum_bins.iter().fold(0.01f32, |a, &b| a.max(b));

        let band_labels = ["Sub", "Low", "L-Mid", "Mid", "H-Mid", "High", "Air", "Ultra"];

        for (i, &level) in analysis.spectrum_bins.iter().enumerate() {
            let x = rect.left() + i as f32 * bar_width;
            let normalized = (level / max_level).clamp(0.0, 1.0);
            let bar_height = normalized * height * 0.9;

            let bar_rect = Rect::from_min_size(
                Pos2::new(x + 2.0, rect.bottom() - bar_height),
                Vec2::new(bar_width - 4.0, bar_height)
            );

            // Color gradient based on frequency band
            let color = match i {
                0..=1 => Color32::from_rgb(52, 152, 219),  // Blue for low
                2..=4 => Color32::from_rgb(46, 204, 113),  // Green for mid
                _ => Color32::from_rgb(155, 89, 182),      // Purple for high
            };

            ui.painter().rect_filled(bar_rect, 2.0, color);
        }

        // Band labels
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            for label in band_labels.iter().take(num_bins) {
                ui.label(egui::RichText::new(*label).size(8.0).color(Color32::GRAY));
                ui.add_space(bar_width - 20.0);
            }
        });
    }

    /// Draw a level meter bar
    fn draw_level_bar(&self, ui: &mut Ui, level_db: f32, width: f32) {
        let height = 10.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());

        // Background
        ui.painter().rect_filled(rect, 2.0, self.theme.palette.bg_tertiary);

        // Convert dB to linear (0 = -60dB, 1 = 0dB)
        let normalized = ((level_db + 60.0) / 60.0).clamp(0.0, 1.0);
        let fill_width = width * normalized;

        // Color based on level
        let color = if level_db > -3.0 {
            Color32::from_rgb(231, 76, 60) // Red
        } else if level_db > -12.0 {
            Color32::from_rgb(241, 196, 15) // Yellow
        } else {
            Color32::from_rgb(46, 204, 113) // Green
        };

        let fill_rect = Rect::from_min_size(rect.min, Vec2::new(fill_width, height));
        ui.painter().rect_filled(fill_rect, 2.0, color);

        // Reference marks at -6dB and -12dB
        let mark_6db = width * (54.0 / 60.0);
        let mark_12db = width * (48.0 / 60.0);
        ui.painter().vline(rect.left() + mark_6db, rect.y_range(), Stroke::new(1.0, Color32::from_gray(80)));
        ui.painter().vline(rect.left() + mark_12db, rect.y_range(), Stroke::new(1.0, Color32::from_gray(80)));
    }

    /// Draw analysis status bar
    fn show_analysis_status_bar(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Waveform overlay toggle
            let overlay_icon = if self.state.analysis.show_waveform_overlay { "◉" } else { "○" };
            ui.label(egui::RichText::new(format!("{} Overlay", overlay_icon))
                .small()
                .color(if self.state.analysis.show_waveform_overlay {
                    self.theme.palette.accent
                } else {
                    self.theme.palette.text_secondary
                }))
                .on_hover_text("Toggle waveform overlay (W)");

            ui.separator();

            // Clipping indicator
            if self.state.analysis.any_clipping() {
                ui.label(egui::RichText::new("⚠ CLIP")
                    .small()
                    .color(Color32::from_rgb(231, 76, 60)));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Selected input indicator
                let input_name = self.state.inputs.get(self.state.analysis.selected_input)
                    .map(|i| i.name.as_str())
                    .unwrap_or("?");
                ui.label(egui::RichText::new(format!("← → {}", input_name))
                    .small()
                    .color(self.theme.palette.text_secondary));
            });
        });
    }
}

/// Actions from track row UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrackRowAction {
    None,
    Delete,
    Solo,
    StartRename,
    FinishRename,
    CancelRename,
}

/// Draw a single recording track row (standalone function to avoid borrow conflicts)
fn draw_track_row(
    ui: &mut Ui,
    track: &mut RecordTrack,
    inputs: &[String],
    is_recording: bool,
    theme: &Theme,
) {
    egui::Frame::none()
        .fill(theme.palette.bg_secondary)
        .inner_margin(egui::Margin::symmetric(8.0, 8.0))
        .rounding(4.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Color indicator
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(4.0, 50.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 2.0, track.color);

                ui.add_space(8.0);

                // Track info
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&track.name)
                                .strong()
                                .color(theme.palette.text_primary)
                        );

                        // Recording indicator
                        if track.armed && is_recording {
                            ui.label(
                                egui::RichText::new("REC")
                                    .color(Color32::from_rgb(231, 76, 60))
                                    .small()
                            );
                        }
                    });

                    // Input selector
                    ui.horizontal(|ui| {
                        ui.label("Input:");
                        egui::ComboBox::from_id_salt(&track.name)
                            .selected_text(&inputs[track.input_index.min(inputs.len() - 1)])
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                for (idx, input_name) in inputs.iter().enumerate() {
                                    ui.selectable_value(&mut track.input_index, idx, input_name);
                                }
                            });
                    });
                });

                ui.add_space(16.0);

                // Waveform area (placeholder)
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(ui.available_width() - 100.0, 50.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 2.0, theme.palette.bg_tertiary);

                if track.has_audio {
                    // Draw fake waveform
                    let center_y = rect.center().y;
                    let amplitude = rect.height() / 3.0;
                    let points: Vec<_> = (0..50).map(|i| {
                        let x = rect.left() + (i as f32 / 50.0) * rect.width();
                        let y = center_y + (i as f32 * 0.5).sin() * amplitude * (1.0 - (i as f32 / 100.0));
                        egui::Pos2::new(x, y)
                    }).collect();

                    for w in points.windows(2) {
                        ui.painter().line_segment(
                            [w[0], w[1]],
                            egui::Stroke::new(1.0, track.color),
                        );
                    }
                } else {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "No audio recorded",
                        egui::FontId::proportional(11.0),
                        theme.palette.text_secondary,
                    );
                }

                // Track controls
                ui.vertical(|ui| {
                    // Arm button
                    let arm_color = if track.armed {
                        Color32::from_rgb(231, 76, 60)
                    } else {
                        theme.palette.text_secondary
                    };
                    if ui.add(egui::Button::new(
                        egui::RichText::new("R").color(arm_color).size(12.0)
                    ).min_size(egui::Vec2::new(24.0, 24.0))).clicked() {
                        track.armed = !track.armed;
                    }

                    // Mute button
                    let mute_color = if track.muted {
                        Color32::from_rgb(231, 76, 60)
                    } else {
                        theme.palette.text_secondary
                    };
                    if ui.add(egui::Button::new(
                        egui::RichText::new("M").color(mute_color).size(12.0)
                    ).min_size(egui::Vec2::new(24.0, 24.0))).clicked() {
                        track.muted = !track.muted;
                    }
                });
            });
        });

    ui.add_space(4.0);
}

/// Draw a track row with full management controls
fn draw_track_row_with_actions(
    ui: &mut Ui,
    track: &mut RecordTrack,
    inputs: &[String],
    is_recording: bool,
    theme: &Theme,
    index: usize,
    is_solo: bool,
    is_renaming: bool,
    can_delete: bool,
    rename_buffer: &mut String,
) -> TrackRowAction {
    let mut action = TrackRowAction::None;

    egui::Frame::none()
        .fill(if is_solo {
            theme.palette.bg_tertiary
        } else {
            theme.palette.bg_secondary
        })
        .inner_margin(egui::Margin::symmetric(8.0, 8.0))
        .rounding(4.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Color indicator
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(4.0, 50.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 2.0, track.color);

                ui.add_space(8.0);

                // Track info
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        // Track name (editable if renaming)
                        if is_renaming {
                            let response = ui.add(
                                egui::TextEdit::singleline(rename_buffer)
                                    .desired_width(100.0)
                                    .font(egui::TextStyle::Body)
                            );
                            if response.lost_focus() {
                                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                    action = TrackRowAction::CancelRename;
                                } else {
                                    action = TrackRowAction::FinishRename;
                                }
                            }
                            response.request_focus();
                        } else {
                            let name_response = ui.label(
                                egui::RichText::new(&track.name)
                                    .strong()
                                    .color(theme.palette.text_primary)
                            );
                            // Double-click to rename
                            if name_response.double_clicked() {
                                action = TrackRowAction::StartRename;
                            }
                        }

                        // Recording indicator
                        if track.armed && is_recording {
                            ui.label(
                                egui::RichText::new("⏺")
                                    .color(Color32::from_rgb(231, 76, 60))
                                    .small()
                            );
                        }

                        // Solo indicator
                        if is_solo {
                            ui.label(
                                egui::RichText::new("SOLO")
                                    .color(theme.palette.warning)
                                    .small()
                            );
                        }
                    });

                    // Input selector
                    ui.horizontal(|ui| {
                        ui.label("Input:");
                        egui::ComboBox::from_id_salt(format!("track_input_{}", index))
                            .selected_text(&inputs[track.input_index.min(inputs.len() - 1)])
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                for (idx, input_name) in inputs.iter().enumerate() {
                                    ui.selectable_value(&mut track.input_index, idx, input_name);
                                }
                            });
                    });
                });

                ui.add_space(16.0);

                // Waveform area (placeholder)
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(ui.available_width() - 140.0, 50.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 2.0, theme.palette.bg_tertiary);

                if track.has_audio {
                    // Draw fake waveform
                    let center_y = rect.center().y;
                    let amplitude = rect.height() / 3.0;
                    let points: Vec<_> = (0..50).map(|i| {
                        let x = rect.left() + (i as f32 / 50.0) * rect.width();
                        let y = center_y + (i as f32 * 0.5).sin() * amplitude * (1.0 - (i as f32 / 100.0));
                        egui::Pos2::new(x, y)
                    }).collect();

                    for w in points.windows(2) {
                        ui.painter().line_segment(
                            [w[0], w[1]],
                            egui::Stroke::new(1.0, track.color),
                        );
                    }
                } else {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "No audio recorded",
                        egui::FontId::proportional(11.0),
                        theme.palette.text_secondary,
                    );
                }

                // Track controls
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        // Arm button
                        let arm_color = if track.armed {
                            Color32::from_rgb(231, 76, 60)
                        } else {
                            theme.palette.text_secondary
                        };
                        if ui.add(egui::Button::new(
                            egui::RichText::new("R").color(arm_color).size(11.0)
                        ).min_size(egui::Vec2::new(22.0, 22.0)))
                            .on_hover_text("Arm for recording")
                            .clicked()
                        {
                            track.armed = !track.armed;
                        }

                        // Mute button
                        let mute_color = if track.muted {
                            Color32::from_rgb(231, 76, 60)
                        } else {
                            theme.palette.text_secondary
                        };
                        if ui.add(egui::Button::new(
                            egui::RichText::new("M").color(mute_color).size(11.0)
                        ).min_size(egui::Vec2::new(22.0, 22.0)))
                            .on_hover_text("Mute track")
                            .clicked()
                        {
                            track.muted = !track.muted;
                        }

                        // Solo button
                        let solo_color = if is_solo {
                            theme.palette.warning
                        } else {
                            theme.palette.text_secondary
                        };
                        if ui.add(egui::Button::new(
                            egui::RichText::new("S").color(solo_color).size(11.0)
                        ).min_size(egui::Vec2::new(22.0, 22.0)))
                            .on_hover_text("Solo track")
                            .clicked()
                        {
                            action = TrackRowAction::Solo;
                        }
                    });

                    ui.horizontal(|ui| {
                        // Delete button (only if more than one track)
                        if can_delete {
                            if ui.add(egui::Button::new(
                                egui::RichText::new("✕").color(theme.palette.text_secondary).size(10.0)
                            ).min_size(egui::Vec2::new(22.0, 22.0)))
                                .on_hover_text("Delete track")
                                .clicked()
                            {
                                action = TrackRowAction::Delete;
                            }
                        }
                    });
                });
            });
        });

    ui.add_space(4.0);
    action
}

// ============================================================================
// Session Templates - Quick setup for common recording scenarios
// ============================================================================

/// A track template for session setup
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrackTemplate {
    /// Track name
    pub name: String,
    /// Track color RGB
    pub color: (u8, u8, u8),
    /// Preferred input (index or name)
    pub input_name: Option<String>,
    /// Auto-arm this track
    pub auto_arm: bool,
}

impl TrackTemplate {
    pub fn new(name: impl Into<String>, color: (u8, u8, u8)) -> Self {
        Self {
            name: name.into(),
            color,
            input_name: None,
            auto_arm: false,
        }
    }

    pub fn with_input(mut self, input: impl Into<String>) -> Self {
        self.input_name = Some(input.into());
        self
    }

    pub fn armed(mut self) -> Self {
        self.auto_arm = true;
        self
    }
}

/// Session template for quick recording setup
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionTemplate {
    /// Template name
    pub name: String,
    /// Description
    pub description: String,
    /// Tempo BPM
    pub tempo: u16,
    /// Time signature numerator
    pub time_sig_num: u8,
    /// Time signature denominator
    pub time_sig_denom: u8,
    /// Track templates
    pub tracks: Vec<TrackTemplate>,
    /// Enable metronome
    pub metronome: bool,
    /// Enable count-in
    pub count_in: bool,
    /// Count-in bars
    pub count_in_bars: u8,
    /// Pre-roll enabled
    pub pre_roll: bool,
    /// Pre-roll bars
    pub pre_roll_bars: u8,
}

impl Default for SessionTemplate {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            description: "Empty session".to_string(),
            tempo: 120,
            time_sig_num: 4,
            time_sig_denom: 4,
            tracks: Vec::new(),
            metronome: true,
            count_in: true,
            count_in_bars: 1,
            pre_roll: false,
            pre_roll_bars: 2,
        }
    }
}

impl SessionTemplate {
    /// Create a new template
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            ..Default::default()
        }
    }

    /// Set tempo
    pub fn with_tempo(mut self, tempo: u16) -> Self {
        self.tempo = tempo;
        self
    }

    /// Set time signature
    pub fn with_time_sig(mut self, num: u8, denom: u8) -> Self {
        self.time_sig_num = num;
        self.time_sig_denom = denom;
        self
    }

    /// Add a track
    pub fn with_track(mut self, track: TrackTemplate) -> Self {
        self.tracks.push(track);
        self
    }

    /// Configure metronome
    pub fn with_metronome(mut self, enabled: bool) -> Self {
        self.metronome = enabled;
        self
    }

    /// Configure count-in
    pub fn with_count_in(mut self, bars: u8) -> Self {
        self.count_in = bars > 0;
        self.count_in_bars = bars;
        self
    }

    // ========================================================================
    // Preset Templates
    // ========================================================================

    /// Solo guitar practice/recording session
    pub fn solo_guitar() -> Self {
        Self::new("Solo Guitar", "Single guitar with DI input")
            .with_tempo(120)
            .with_track(
                TrackTemplate::new("Guitar", (26, 123, 93))
                    .with_input("Guitar DI")
                    .armed()
            )
            .with_metronome(true)
            .with_count_in(1)
    }

    /// Acoustic singer-songwriter setup
    pub fn singer_songwriter() -> Self {
        Self::new("Singer-Songwriter", "Vocals + acoustic guitar")
            .with_tempo(100)
            .with_track(
                TrackTemplate::new("Vocals", (231, 76, 60))
                    .with_input("Mic 1")
                    .armed()
            )
            .with_track(
                TrackTemplate::new("Acoustic Guitar", (46, 204, 113))
                    .with_input("Mic 2")
            )
            .with_metronome(true)
            .with_count_in(2)
    }

    /// Full band recording template
    pub fn full_band() -> Self {
        Self::new("Full Band", "Drums, bass, guitars, vocals")
            .with_tempo(120)
            .with_track(TrackTemplate::new("Drums L", (155, 89, 182)))
            .with_track(TrackTemplate::new("Drums R", (142, 68, 173)))
            .with_track(
                TrackTemplate::new("Bass", (241, 196, 15))
                    .with_input("Guitar DI")
            )
            .with_track(TrackTemplate::new("Guitar L", (46, 204, 113)))
            .with_track(TrackTemplate::new("Guitar R", (39, 174, 96)))
            .with_track(
                TrackTemplate::new("Lead Vocals", (231, 76, 60))
                    .with_input("Mic 1")
            )
            .with_metronome(true)
            .with_count_in(2)
    }

    /// Podcast/voiceover template
    pub fn podcast() -> Self {
        Self::new("Podcast", "Multi-person podcast recording")
            .with_tempo(120)
            .with_track(
                TrackTemplate::new("Host", (52, 152, 219))
                    .with_input("Mic 1")
                    .armed()
            )
            .with_track(
                TrackTemplate::new("Guest 1", (46, 204, 113))
                    .with_input("Mic 2")
            )
            .with_metronome(false)
            .with_count_in(0)
    }

    /// Practice session with slow tempo
    pub fn practice_slow() -> Self {
        Self::new("Practice (Slow)", "Slow tempo for learning parts")
            .with_tempo(80)
            .with_track(
                TrackTemplate::new("Practice", (52, 152, 219))
                    .with_input("Guitar DI")
                    .armed()
            )
            .with_metronome(true)
            .with_count_in(2)
    }

    /// Get all preset templates
    pub fn presets() -> Vec<Self> {
        vec![
            Self::solo_guitar(),
            Self::singer_songwriter(),
            Self::full_band(),
            Self::podcast(),
            Self::practice_slow(),
        ]
    }

    /// Create from tab editor state (for transitioning from composition to recording)
    pub fn from_tab_editor(
        tempo: f64,
        time_sig_num: u8,
        time_sig_denom: u8,
        track_name: Option<&str>,
    ) -> Self {
        let mut template = Self::new(
            "From Composition",
            "Recording session created from tab editor"
        )
        .with_tempo(tempo as u16)
        .with_time_sig(time_sig_num, time_sig_denom)
        .with_metronome(true)
        .with_count_in(2);

        // Add a track for the composition
        if let Some(name) = track_name {
            template = template.with_track(
                TrackTemplate::new(name, (26, 123, 93))
                    .with_input("Guitar DI")
                    .armed()
            );
        }

        template
    }
}

impl RecordViewState {
    /// Create from a session template
    pub fn from_template(template: &SessionTemplate) -> Self {
        let mut state = Self::new();

        // Apply template settings
        state.tempo = template.tempo;
        state.time_sig_num = template.time_sig_num;
        state.time_sig_denom = template.time_sig_denom;
        state.metronome = template.metronome;
        state.count_in = template.count_in;
        state.count_in_bars = template.count_in_bars;
        state.pre_roll = template.pre_roll;
        state.pre_roll_bars = template.pre_roll_bars;

        // Create tracks from template
        let colors = [
            Color32::from_rgb(231, 76, 60),
            Color32::from_rgb(52, 152, 219),
            Color32::from_rgb(46, 204, 113),
            Color32::from_rgb(155, 89, 182),
            Color32::from_rgb(241, 196, 15),
        ];

        state.tracks.clear();
        for (i, track_template) in template.tracks.iter().enumerate() {
            let color = Color32::from_rgb(
                track_template.color.0,
                track_template.color.1,
                track_template.color.2,
            );

            // Find input index by name
            let input_index = track_template.input_name.as_ref()
                .and_then(|name| {
                    state.inputs.iter().position(|input| input.name.contains(name))
                })
                .unwrap_or(i % state.inputs.len());

            let mut track = RecordTrack::new(&track_template.name, color);
            track.input_index = input_index;
            track.armed = track_template.auto_arm;

            state.tracks.push(track);
        }

        // If no tracks in template, create a default one
        if state.tracks.is_empty() {
            let color = colors[0];
            let mut track = RecordTrack::new("Track 1", color);
            track.armed = true;
            state.tracks.push(track);
        }

        state
    }

    /// Apply settings from tab editor for seamless transition
    pub fn apply_from_tab_editor(
        &mut self,
        tempo: f64,
        time_sig_num: u8,
        time_sig_denom: u8,
    ) {
        self.tempo = tempo as u16;
        self.time_sig_num = time_sig_num;
        self.time_sig_denom = time_sig_denom;
    }
}
