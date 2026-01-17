//! Section Analysis for Tab Editor
//!
//! Provides automatic section detection and a UI for adjusting detected sections
//! before applying them to the document.

use egui::{Color32, Pos2, Rect, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use orpheus_core::tab::{SectionMarker, TabDocument, TabMeasure};
use uuid::Uuid;

use crate::theme::Theme;

/// A detected section with adjustable boundaries
#[derive(Debug, Clone)]
pub struct DetectedSection {
    /// Unique ID for UI tracking
    pub id: Uuid,
    /// Starting measure (0-indexed)
    pub start_measure: usize,
    /// Ending measure (0-indexed, inclusive)
    pub end_measure: usize,
    /// Suggested section name
    pub name: String,
    /// Section type for coloring
    pub section_type: SectionType,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Whether this section is selected in UI
    pub selected: bool,
}

impl DetectedSection {
    pub fn new(start: usize, end: usize, name: &str, section_type: SectionType, confidence: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            start_measure: start,
            end_measure: end,
            name: name.to_string(),
            section_type,
            confidence,
            selected: false,
        }
    }

    /// Get the color for this section type
    pub fn color(&self) -> Color32 {
        self.section_type.color()
    }

    /// Get the measure count
    pub fn measure_count(&self) -> usize {
        self.end_measure.saturating_sub(self.start_measure) + 1
    }
}

/// Common section types in music
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Intro,
    Verse,
    PreChorus,
    Chorus,
    Bridge,
    Solo,
    Breakdown,
    Outro,
    Interlude,
    Riff,
    Custom,
}

impl SectionType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Intro => "Intro",
            Self::Verse => "Verse",
            Self::PreChorus => "Pre-Chorus",
            Self::Chorus => "Chorus",
            Self::Bridge => "Bridge",
            Self::Solo => "Solo",
            Self::Breakdown => "Breakdown",
            Self::Outro => "Outro",
            Self::Interlude => "Interlude",
            Self::Riff => "Riff",
            Self::Custom => "Section",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            Self::Intro => Color32::from_rgb(52, 152, 219),     // Blue
            Self::Verse => Color32::from_rgb(46, 204, 113),     // Green
            Self::PreChorus => Color32::from_rgb(155, 89, 182), // Purple
            Self::Chorus => Color32::from_rgb(231, 76, 60),     // Red
            Self::Bridge => Color32::from_rgb(241, 196, 15),    // Yellow
            Self::Solo => Color32::from_rgb(230, 126, 34),      // Orange
            Self::Breakdown => Color32::from_rgb(44, 62, 80),   // Dark blue
            Self::Outro => Color32::from_rgb(149, 165, 166),    // Gray
            Self::Interlude => Color32::from_rgb(26, 188, 156), // Teal
            Self::Riff => Color32::from_rgb(192, 57, 43),       // Dark red
            Self::Custom => Color32::from_rgb(127, 140, 141),   // Silver
        }
    }

    pub fn all() -> &'static [SectionType] {
        &[
            Self::Intro,
            Self::Verse,
            Self::PreChorus,
            Self::Chorus,
            Self::Bridge,
            Self::Solo,
            Self::Breakdown,
            Self::Outro,
            Self::Interlude,
            Self::Riff,
            Self::Custom,
        ]
    }
}

/// State for the section analysis dialog
#[derive(Debug, Clone, Default)]
pub struct SectionAnalysisState {
    /// Whether the analysis dialog is open
    pub is_open: bool,
    /// Detected sections (before user confirmation)
    pub detected_sections: Vec<DetectedSection>,
    /// Currently selected section for editing
    pub selected_section: Option<Uuid>,
    /// Analysis has been performed
    pub analysis_done: bool,
    /// Sensitivity for pattern detection (0.0 - 1.0)
    pub sensitivity: f32,
    /// Minimum section length in measures
    pub min_section_length: usize,
}

impl SectionAnalysisState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            detected_sections: Vec::new(),
            selected_section: None,
            analysis_done: false,
            sensitivity: 0.5,
            min_section_length: 4,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.analysis_done = false;
        self.detected_sections.clear();
        self.selected_section = None;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
}

/// Analyze a document and detect sections
pub fn analyze_sections(document: &TabDocument, sensitivity: f32, min_length: usize) -> Vec<DetectedSection> {
    let mut sections = Vec::new();
    let measure_count = document.measures.len();

    if measure_count == 0 {
        return sections;
    }

    // Collect analysis features for each measure
    let features: Vec<MeasureFeatures> = document.measures.iter()
        .enumerate()
        .map(|(idx, m)| extract_features(document, m, idx))
        .collect();

    // Calculate song-wide statistics for relative comparison
    // This is crucial for tech death where absolute thresholds fail
    let song_stats = SongStatistics::from_features(&features);

    // Find section boundaries based on feature changes
    let mut boundaries = vec![0]; // Always start at measure 0

    for i in 1..features.len() {
        let change_score = calculate_change_score(&features[i - 1], &features[i], &song_stats);

        // Threshold based on sensitivity (higher sensitivity = more sections)
        let threshold = 1.0 - sensitivity;
        if change_score > threshold {
            boundaries.push(i);
        }
    }

    // Always end at last measure
    if boundaries.last() != Some(&(measure_count - 1)) {
        boundaries.push(measure_count);
    }

    // Merge sections that are too short
    let mut merged_boundaries = vec![boundaries[0]];
    for i in 1..boundaries.len() {
        let prev = *merged_boundaries.last().unwrap();
        let curr = boundaries[i];
        if curr - prev >= min_length {
            merged_boundaries.push(curr);
        }
    }
    // Ensure we have the end
    if *merged_boundaries.last().unwrap() != measure_count {
        merged_boundaries.push(measure_count);
    }

    // Create sections from boundaries
    for i in 0..merged_boundaries.len() - 1 {
        let start = merged_boundaries[i];
        let end = merged_boundaries[i + 1] - 1;

        // Determine section type based on position and features (relative to song stats)
        let section_type = guess_section_type(i, merged_boundaries.len() - 1, &features[start..=end.min(features.len() - 1)], &song_stats);
        let confidence = calculate_confidence(&features[start..=end.min(features.len() - 1)]);

        let name = if i == 0 && section_type == SectionType::Intro {
            "Intro".to_string()
        } else if i == merged_boundaries.len() - 2 && section_type == SectionType::Outro {
            "Outro".to_string()
        } else {
            format!("{} {}", section_type.name(), count_section_type(&sections, section_type) + 1)
        };

        sections.push(DetectedSection::new(start, end, &name, section_type, confidence));
    }

    sections
}

/// Song-wide statistics for relative comparison
/// Critical for genres like tech death where absolute thresholds are meaningless
#[derive(Debug, Clone)]
struct SongStatistics {
    /// Mean note density across all measures
    mean_density: f32,
    /// Standard deviation of note density
    std_density: f32,
    /// Mean fret position
    mean_fret: f32,
    /// Standard deviation of fret position
    std_fret: f32,
    /// Mean technique density
    mean_technique: f32,
    /// Whether this song is "technical" (consistently high density)
    is_technical: bool,
}

impl SongStatistics {
    fn from_features(features: &[MeasureFeatures]) -> Self {
        if features.is_empty() {
            return Self {
                mean_density: 0.0,
                std_density: 1.0,
                mean_fret: 0.0,
                std_fret: 1.0,
                mean_technique: 0.0,
                is_technical: false,
            };
        }

        let n = features.len() as f32;

        // Calculate means
        let mean_density: f32 = features.iter().map(|f| f.note_density).sum::<f32>() / n;
        let mean_fret: f32 = features.iter().map(|f| f.avg_fret).sum::<f32>() / n;
        let mean_technique: f32 = features.iter().map(|f| f.technique_density).sum::<f32>() / n;

        // Calculate standard deviations
        let var_density: f32 = features.iter()
            .map(|f| (f.note_density - mean_density).powi(2))
            .sum::<f32>() / n;
        let std_density = var_density.sqrt().max(0.1); // Avoid division by zero

        let var_fret: f32 = features.iter()
            .map(|f| (f.avg_fret - mean_fret).powi(2))
            .sum::<f32>() / n;
        let std_fret = var_fret.sqrt().max(0.1);

        // A song is "technical" if:
        // - High average density (> 2 notes per beat)
        // - High technique usage (> 0.3 techniques per note)
        // - Low relative variance (consistently brutal, not just one solo)
        let is_technical = mean_density > 2.0 && mean_technique > 0.3 && std_density / mean_density < 0.5;

        Self {
            mean_density,
            std_density,
            mean_fret,
            std_fret,
            mean_technique,
            is_technical,
        }
    }

    /// Get z-score for density (how many std devs from mean)
    fn density_zscore(&self, density: f32) -> f32 {
        (density - self.mean_density) / self.std_density
    }

    /// Get z-score for fret position
    fn fret_zscore(&self, fret: f32) -> f32 {
        (fret - self.mean_fret) / self.std_fret
    }
}

/// Features extracted from a measure for analysis
#[derive(Debug, Clone, Default)]
struct MeasureFeatures {
    /// Note density (notes per beat)
    note_density: f32,
    /// Average fret position
    avg_fret: f32,
    /// Has tempo change
    has_tempo_change: bool,
    /// Has time signature change
    has_time_sig_change: bool,
    /// Has repeat marker
    has_repeat: bool,
    /// Technique density
    technique_density: f32,
    /// Is mostly rest
    is_sparse: bool,
}

fn extract_features(document: &TabDocument, measure: &TabMeasure, _idx: usize) -> MeasureFeatures {
    let mut features = MeasureFeatures::default();

    features.has_tempo_change = measure.tempo.is_some();
    features.has_time_sig_change = measure.time_signature.is_some();
    features.has_repeat = measure.repeat != orpheus_core::tab::RepeatMarker::None;

    // Analyze notes across all tracks
    let mut total_notes = 0;
    let mut total_fret = 0u32;
    let mut total_techniques = 0;
    let mut total_beats = 0;
    let mut rest_beats = 0;

    for track_measure in &measure.track_beats {
        for beat in &track_measure.beats {
            total_beats += 1;
            if beat.is_rest || beat.notes.is_empty() {
                rest_beats += 1;
            }
            for note in &beat.notes {
                total_notes += 1;
                total_fret += note.fret as u32;
                total_techniques += note.techniques.len();
            }
        }
    }

    if total_beats > 0 {
        features.note_density = total_notes as f32 / total_beats as f32;
        features.is_sparse = rest_beats as f32 / total_beats as f32 > 0.5;
    }

    if total_notes > 0 {
        features.avg_fret = total_fret as f32 / total_notes as f32;
        features.technique_density = total_techniques as f32 / total_notes as f32;
    }

    features
}

fn calculate_change_score(prev: &MeasureFeatures, curr: &MeasureFeatures, stats: &SongStatistics) -> f32 {
    let mut score = 0.0;

    // Structural markers are strongest indicators (genre-independent)
    if curr.has_tempo_change { score += 0.9; }
    if curr.has_time_sig_change { score += 0.95; }
    if curr.has_repeat { score += 0.7; }

    // Use RELATIVE density change (z-score difference)
    // This works for tech death: a drop from 8 notes/beat to 4 is significant
    // even though 4 would be "high" in rock
    let prev_z = stats.density_zscore(prev.note_density);
    let curr_z = stats.density_zscore(curr.note_density);
    let density_z_change = (curr_z - prev_z).abs();
    score += (density_z_change * 0.3).min(0.5);

    // Register change using relative fret position
    let prev_fret_z = stats.fret_zscore(prev.avg_fret);
    let curr_fret_z = stats.fret_zscore(curr.avg_fret);
    let fret_z_change = (curr_fret_z - prev_fret_z).abs();
    if fret_z_change > 1.0 { score += 0.4; } // >1 std dev = significant register shift

    // Sparseness change (relative to song's sparseness, not absolute)
    if prev.is_sparse != curr.is_sparse { score += 0.5; }

    // Technique density change
    let tech_change = (curr.technique_density - prev.technique_density).abs();
    score += tech_change * 0.15; // Reduced weight - less meaningful in tech death

    score.min(1.0)
}

fn guess_section_type(section_idx: usize, total_sections: usize, features: &[MeasureFeatures], stats: &SongStatistics) -> SectionType {
    if features.is_empty() {
        return SectionType::Custom;
    }

    let n = features.len() as f32;

    // Calculate section averages
    let avg_density: f32 = features.iter().map(|f| f.note_density).sum::<f32>() / n;
    let avg_fret: f32 = features.iter().map(|f| f.avg_fret).sum::<f32>() / n;
    let avg_tech: f32 = features.iter().map(|f| f.technique_density).sum::<f32>() / n;

    // Calculate z-scores relative to song
    let density_z = stats.density_zscore(avg_density);
    let fret_z = stats.fret_zscore(avg_fret);

    // Check for structural intro/outro (sparse relative to song)
    // First section with below-average density = intro
    if section_idx == 0 && density_z < -0.5 {
        return SectionType::Intro;
    }

    // Last section with below-average density = outro
    if section_idx == total_sections - 1 && density_z < -0.5 {
        return SectionType::Outro;
    }

    // For technical songs, use different heuristics
    if stats.is_technical {
        // In tech death, "solo" means EXCEPTIONALLY high density/technique
        // (more than 1.5 std devs above the already-brutal mean)
        if density_z > 1.5 && avg_tech > stats.mean_technique * 1.3 {
            return SectionType::Solo;
        }

        // Breakdown = significantly LOWER density (the "breathing room")
        if density_z < -1.0 {
            return SectionType::Breakdown;
        }

        // Low register (chugging) = Riff
        if fret_z < -0.5 && density_z > 0.0 {
            return SectionType::Riff;
        }

        // High register melodic section = could be "chorus" equivalent
        if fret_z > 1.0 {
            return SectionType::Chorus;
        }

        // Bridge = moderate density with register change
        if density_z.abs() < 0.5 && fret_z.abs() > 0.8 {
            return SectionType::Bridge;
        }

        // Default for tech death: most sections are "verses" (main riff sections)
        return SectionType::Verse;
    }

    // Non-technical song heuristics (rock, pop, etc.)

    // Sparse section = breakdown
    if features.iter().all(|f| f.is_sparse) {
        return SectionType::Breakdown;
    }

    // High density + techniques = solo (works for non-tech songs)
    if density_z > 1.0 && avg_tech > 0.4 {
        return SectionType::Solo;
    }

    // High energy (above average density + higher frets) = chorus
    if density_z > 0.3 && fret_z > 0.5 {
        return SectionType::Chorus;
    }

    // Below average density = verse
    if density_z < 0.0 && density_z > -1.0 {
        return SectionType::Verse;
    }

    // Low fret, moderate-high density = riff
    if fret_z < -0.3 && density_z > 0.0 {
        return SectionType::Riff;
    }

    // Pre-chorus: building energy (slightly above verse density)
    if density_z > 0.0 && density_z < 0.5 && fret_z > 0.0 {
        return SectionType::PreChorus;
    }

    SectionType::Custom
}

fn calculate_confidence(features: &[MeasureFeatures]) -> f32 {
    if features.is_empty() {
        return 0.0;
    }

    // Higher confidence if the section has consistent features
    let densities: Vec<f32> = features.iter().map(|f| f.note_density).collect();
    let mean_density: f32 = densities.iter().sum::<f32>() / densities.len() as f32;
    let variance: f32 = densities.iter().map(|d| (d - mean_density).powi(2)).sum::<f32>() / densities.len() as f32;

    // Lower variance = higher confidence
    let consistency = 1.0 / (1.0 + variance);

    // Bonus for structural markers
    let has_markers = features.iter().any(|f| f.has_tempo_change || f.has_time_sig_change || f.has_repeat);
    let marker_bonus = if has_markers { 0.2 } else { 0.0 };

    (consistency * 0.8 + marker_bonus).min(1.0)
}

fn count_section_type(sections: &[DetectedSection], section_type: SectionType) -> usize {
    sections.iter().filter(|s| s.section_type == section_type).count()
}

/// Action from the section analysis dialog
#[derive(Debug, Clone)]
pub enum SectionAnalysisAction {
    /// Close the dialog without applying
    Cancel,
    /// Apply the sections to the document
    Apply(Vec<DetectedSection>),
    /// Run analysis with current settings
    Analyze,
    /// Jump to a section for preview
    JumpToSection(usize),
}

/// Section Analysis Dialog
pub struct SectionAnalysisDialog<'a> {
    state: &'a mut SectionAnalysisState,
    document: &'a TabDocument,
    theme: &'a Theme,
}

impl<'a> SectionAnalysisDialog<'a> {
    pub fn new(
        state: &'a mut SectionAnalysisState,
        document: &'a TabDocument,
        theme: &'a Theme,
    ) -> Self {
        Self { state, document, theme }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Option<SectionAnalysisAction> {
        let mut action = None;

        if !self.state.is_open {
            return None;
        }

        // Modal overlay
        let screen_rect = ui.ctx().screen_rect();
        ui.painter().rect_filled(screen_rect, Rounding::ZERO, Color32::from_black_alpha(180));

        // Dialog window
        let dialog_size = Vec2::new(700.0, 500.0);
        let dialog_rect = Rect::from_center_size(screen_rect.center(), dialog_size);

        let mut dialog_ui = ui.child_ui(dialog_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        let painter = dialog_ui.painter();

        // Dialog background
        painter.rect_filled(dialog_rect, Rounding::same(8.0), self.theme.panel_bg());
        painter.rect_stroke(dialog_rect, Rounding::same(8.0), Stroke::new(1.0, self.theme.border()));

        // Content area
        let content_rect = dialog_rect.shrink(16.0);
        let mut content_ui = dialog_ui.child_ui(content_rect, egui::Layout::top_down(egui::Align::LEFT), None);

        // Title
        content_ui.heading("🎼 Section Analysis");
        content_ui.add_space(8.0);
        content_ui.label("Automatically detect song sections and adjust before applying.");
        content_ui.add_space(16.0);

        // Analysis settings
        content_ui.horizontal(|ui| {
            ui.label("Sensitivity:");
            ui.add(egui::Slider::new(&mut self.state.sensitivity, 0.1..=0.9)
                .show_value(false))
                .on_hover_text("Higher = more sections detected");

            ui.add_space(16.0);

            ui.label("Min length:");
            ui.add(egui::DragValue::new(&mut self.state.min_section_length)
                .clamp_range(1..=16)
                .suffix(" bars"))
                .on_hover_text("Minimum measures per section");

            ui.add_space(16.0);

            if ui.button("🔍 Analyze")
                .on_hover_text("Run section detection with current settings")
                .clicked()
            {
                self.state.detected_sections = analyze_sections(
                    self.document,
                    self.state.sensitivity,
                    self.state.min_section_length,
                );
                self.state.analysis_done = true;
                action = Some(SectionAnalysisAction::Analyze);
            }
        });

        content_ui.add_space(16.0);
        content_ui.separator();
        content_ui.add_space(8.0);

        // Section list and timeline
        if self.state.analysis_done && !self.state.detected_sections.is_empty() {
            // Timeline visualization
            let timeline_height = 60.0;
            let (timeline_rect, _) = content_ui.allocate_exact_size(
                Vec2::new(content_rect.width() - 32.0, timeline_height),
                Sense::click(),
            );

            self.draw_timeline(&content_ui, timeline_rect, &mut action);

            content_ui.add_space(16.0);

            // Section list
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(&mut content_ui, |ui| {
                    self.show_section_list(ui, &mut action);
                });
        } else if self.state.analysis_done {
            content_ui.label("No sections detected. Try adjusting sensitivity or minimum length.");
        } else {
            content_ui.label("Click 'Analyze' to detect sections in your tab.");
        }

        content_ui.add_space(16.0);

        // Bottom buttons
        content_ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    self.state.close();
                    action = Some(SectionAnalysisAction::Cancel);
                }

                ui.add_space(8.0);

                let can_apply = self.state.analysis_done && !self.state.detected_sections.is_empty();
                if ui.add_enabled(can_apply, egui::Button::new("✓ Apply Sections"))
                    .on_hover_text("Add detected sections to document")
                    .clicked()
                {
                    action = Some(SectionAnalysisAction::Apply(self.state.detected_sections.clone()));
                    self.state.close();
                }
            });
        });

        action
    }

    fn draw_timeline(&self, ui: &Ui, rect: Rect, action: &mut Option<SectionAnalysisAction>) {
        let painter = ui.painter();
        let measure_count = self.document.measures.len().max(1);

        // Background
        painter.rect_filled(rect, Rounding::same(4.0), self.theme.surface_bg());

        // Draw sections as colored blocks
        for section in &self.state.detected_sections {
            let start_x = rect.left() + (section.start_measure as f32 / measure_count as f32) * rect.width();
            let end_x = rect.left() + ((section.end_measure + 1) as f32 / measure_count as f32) * rect.width();

            let section_rect = Rect::from_min_max(
                Pos2::new(start_x, rect.top()),
                Pos2::new(end_x, rect.bottom() - 20.0),
            );

            let color = section.color();
            painter.rect_filled(section_rect, Rounding::same(2.0), color.gamma_multiply(0.6));
            painter.rect_stroke(section_rect, Rounding::same(2.0), Stroke::new(1.0, color));

            // Section name (if fits)
            if section_rect.width() > 40.0 {
                painter.text(
                    section_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &section.name,
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );
            }
        }

        // Measure numbers
        let step = if measure_count > 32 { 8 } else if measure_count > 16 { 4 } else { 2 };
        for i in (0..=measure_count).step_by(step) {
            let x = rect.left() + (i as f32 / measure_count as f32) * rect.width();
            painter.text(
                Pos2::new(x, rect.bottom() - 8.0),
                egui::Align2::CENTER_CENTER,
                format!("{}", i + 1),
                egui::FontId::proportional(9.0),
                self.theme.text_muted(),
            );
        }
    }

    fn show_section_list(&mut self, ui: &mut Ui, action: &mut Option<SectionAnalysisAction>) {
        for section in &mut self.state.detected_sections {
            ui.horizontal(|ui| {
                // Color indicator
                let color = section.color();
                let (indicator_rect, _) = ui.allocate_exact_size(Vec2::new(16.0, 16.0), Sense::hover());
                ui.painter().rect_filled(indicator_rect, Rounding::same(2.0), color);

                // Section name (editable)
                ui.add(egui::TextEdit::singleline(&mut section.name).desired_width(120.0));

                // Section type selector
                egui::ComboBox::from_id_salt(section.id)
                    .selected_text(section.section_type.name())
                    .width(100.0)
                    .show_ui(ui, |ui| {
                        for st in SectionType::all() {
                            if ui.selectable_label(section.section_type == *st, st.name()).clicked() {
                                section.section_type = *st;
                            }
                        }
                    });

                // Start measure
                ui.label("M:");
                let mut start = (section.start_measure + 1) as i32;
                if ui.add(egui::DragValue::new(&mut start)
                    .clamp_range(1..=self.document.measures.len() as i32))
                    .on_hover_text("Start measure")
                    .changed()
                {
                    section.start_measure = (start - 1) as usize;
                }

                ui.label("→");

                let mut end = (section.end_measure + 1) as i32;
                if ui.add(egui::DragValue::new(&mut end)
                    .clamp_range(1..=self.document.measures.len() as i32))
                    .on_hover_text("End measure")
                    .changed()
                {
                    section.end_measure = (end - 1) as usize;
                }

                // Jump to section
                if ui.small_button("▶")
                    .on_hover_text("Jump to this section")
                    .clicked()
                {
                    *action = Some(SectionAnalysisAction::JumpToSection(section.start_measure));
                }

                // Confidence indicator
                let confidence_text = format!("{:.0}%", section.confidence * 100.0);
                ui.label(RichText::new(confidence_text).small().color(self.theme.text_muted()));
            });
        }
    }
}

/// Section Navigator widget for the toolbar
pub struct SectionNavigator<'a> {
    sections: &'a [SectionMarker],
    current_measure: usize,
    theme: &'a Theme,
}

impl<'a> SectionNavigator<'a> {
    pub fn new(sections: &'a [SectionMarker], current_measure: usize, theme: &'a Theme) -> Self {
        Self { sections, current_measure, theme }
    }

    /// Show the section navigator and return the measure to jump to (if any)
    pub fn show(&self, ui: &mut Ui) -> Option<usize> {
        let mut jump_to = None;

        if self.sections.is_empty() {
            return None;
        }

        // Find current section
        let current_section = self.sections.iter()
            .filter(|s| s.measure <= self.current_measure)
            .last();

        let current_name = current_section
            .map(|s| s.name.as_str())
            .unwrap_or("No Section");

        // Previous section button
        if ui.small_button("◀")
            .on_hover_text("Previous Section ([ key)")
            .clicked()
        {
            if let Some(prev) = self.find_previous_section() {
                jump_to = Some(prev.measure);
            }
        }

        // Section dropdown
        egui::ComboBox::from_id_salt("section_nav")
            .selected_text(current_name)
            .width(120.0)
            .show_ui(ui, |ui| {
                for section in self.sections {
                    let text = format!("M{}: {}", section.measure + 1, section.name);
                    let is_current = current_section.map(|s| s.measure) == Some(section.measure);
                    if ui.selectable_label(is_current, text).clicked() {
                        jump_to = Some(section.measure);
                    }
                }
            })
            .response
            .on_hover_text("Jump to section");

        // Next section button
        if ui.small_button("▶")
            .on_hover_text("Next Section (] key)")
            .clicked()
        {
            if let Some(next) = self.find_next_section() {
                jump_to = Some(next.measure);
            }
        }

        jump_to
    }

    fn find_previous_section(&self) -> Option<&SectionMarker> {
        self.sections.iter()
            .filter(|s| s.measure < self.current_measure)
            .last()
    }

    fn find_next_section(&self) -> Option<&SectionMarker> {
        self.sections.iter()
            .find(|s| s.measure > self.current_measure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_type_colors() {
        for st in SectionType::all() {
            let color = st.color();
            assert!(color.r() > 0 || color.g() > 0 || color.b() > 0);
        }
    }

    #[test]
    fn test_detected_section_measure_count() {
        let section = DetectedSection::new(0, 3, "Test", SectionType::Verse, 0.8);
        assert_eq!(section.measure_count(), 4);
    }
}
