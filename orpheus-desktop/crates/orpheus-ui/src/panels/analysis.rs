//! Compositional Analysis Panel
//!
//! Data-driven analysis tools for understanding compositions, detecting
//! stylistic anomalies, and generating rewrite prescriptions.

use egui::{Color32, ProgressBar, RichText, Sense, Ui, Vec2};
#[allow(unused_imports)] // AnalysisRegion used implicitly when iterating report.regions
use orpheus_core::analysis::{
    AnalysisRegion, AnomalyConfig, AnomalyDetector, AnomalyReport, ComposerFingerprint,
    HarmonicAnalyzer, HarmonicAnalysis, SurgicalPlan, SurgicalPlanner,
    Severity, pitch_name, interval_name,
};
use orpheus_core::tab::TabDocument;

/// Actions that can be triggered from the analysis panel
#[derive(Debug, Clone)]
pub enum AnalysisPanelAction {
    /// Run full analysis on the document
    RunAnalysis,
    /// Run analysis excluding specific measure ranges
    RunAnalysisExcluding(Vec<(usize, usize)>),
    /// Jump to a specific measure
    JumpToMeasure(usize),
    /// Jump to an anomalous region
    JumpToRegion(usize, usize),
    /// Mark a region as excluded from fingerprint calculation
    ExcludeRegion(usize, usize),
    /// Exclude a region and automatically re-run analysis
    ExcludeAndReanalyze(usize, usize),
    /// Clear all excluded regions
    ClearExclusions,
    /// Update anomaly detection sensitivity
    UpdateSensitivity(f32),
    /// Toggle showing anomaly highlights on timeline
    ToggleHighlights,
    /// Copy pattern from reference measure to target region
    CopyFromReference {
        /// Source reference measure
        source_measure: usize,
        /// Target start measure
        target_start: usize,
        /// Target end measure
        target_end: usize,
    },
    /// Apply auto-fix suggestions to a region
    AutoFixRegion {
        /// Start measure
        start: usize,
        /// End measure
        end: usize,
    },
    /// Set a region as a "golden" reference
    MarkAsReference(usize, usize),
    /// Start practice mode on a specific region
    PracticeRegion {
        /// Start measure (0-indexed)
        start: usize,
        /// End measure (0-indexed, inclusive)
        end: usize,
        /// Suggested starting tempo percentage
        start_tempo_percent: f32,
    },
    /// Export analysis report to clipboard
    ExportToClipboard,
    /// Add annotation to a specific measure
    AddAnnotation {
        measure: usize,
        text: String,
    },
    /// Clear annotation from a measure
    ClearAnnotation(usize),
    /// Undo last exclusion change
    UndoExclusion,
    /// Redo last undone exclusion change
    RedoExclusion,
}

/// State for the analysis panel
#[derive(Debug, Clone)]
pub struct AnalysisPanelState {
    /// Whether analysis has been run
    pub analysis_done: bool,
    /// Analysis in progress
    pub analyzing: bool,
    /// Analysis progress (0.0 - 1.0)
    pub progress: f32,
    /// Composer fingerprint from good sections
    pub fingerprint: Option<ComposerFingerprint>,
    /// Anomaly detection report
    pub anomaly_report: Option<AnomalyReport>,
    /// Harmonic analysis
    pub harmony: Option<HarmonicAnalysis>,
    /// Surgical rewrite plan
    pub surgical_plan: Option<SurgicalPlan>,
    /// Excluded measure ranges (for fingerprint calculation)
    pub excluded_ranges: Vec<(usize, usize)>,
    /// Anomaly detection sensitivity (0.0 - 1.0)
    pub sensitivity: f32,
    /// Minimum region length
    pub min_region_length: usize,
    /// Show anomaly highlights on timeline
    pub show_highlights: bool,
    /// Currently selected tab in panel
    pub active_tab: AnalysisTab,
    /// Last error message
    pub last_error: Option<String>,
    /// Currently selected prescription index for applying fixes
    pub selected_prescription: Option<usize>,
    /// User-marked reference regions (golden sections)
    pub reference_regions: Vec<(usize, usize)>,
    /// Enable automatic real-time analysis
    pub auto_analyze: bool,
    /// Timestamp of last document change (for debouncing)
    pub last_change_time_ms: u64,
    /// Debounce delay in milliseconds
    pub debounce_ms: u64,
    /// Analysis results are stale (document changed since last analysis)
    pub needs_analysis: bool,
    /// User annotations per measure
    pub annotations: std::collections::HashMap<usize, String>,
    /// Pending text to copy to clipboard (handled by UI layer)
    pub pending_clipboard: Option<String>,
    /// Undo stack for exclusion changes
    exclusion_undo_stack: Vec<Vec<(usize, usize)>>,
    /// Redo stack for exclusion changes
    exclusion_redo_stack: Vec<Vec<(usize, usize)>>,
}

/// Serializable analysis state for persistence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct PersistedAnalysisState {
    /// Excluded measure ranges
    pub excluded_ranges: Vec<(usize, usize)>,
    /// User-marked reference regions
    pub reference_regions: Vec<(usize, usize)>,
    /// Annotations keyed by measure index
    pub annotations: std::collections::HashMap<usize, String>,
    /// Sensitivity setting
    pub sensitivity: f32,
    /// Minimum region length
    pub min_region_length: usize,
    /// Auto-analyze enabled
    pub auto_analyze: bool,
}

impl PersistedAnalysisState {
    /// Create from current analysis panel state
    pub fn from_panel_state(state: &AnalysisPanelState) -> Self {
        Self {
            excluded_ranges: state.excluded_ranges.clone(),
            reference_regions: state.reference_regions.clone(),
            annotations: state.annotations.clone(),
            sensitivity: state.sensitivity,
            min_region_length: state.min_region_length,
            auto_analyze: state.auto_analyze,
        }
    }

    /// Apply to analysis panel state
    pub fn apply_to(&self, state: &mut AnalysisPanelState) {
        state.excluded_ranges = self.excluded_ranges.clone();
        state.reference_regions = self.reference_regions.clone();
        state.annotations = self.annotations.clone();
        state.sensitivity = self.sensitivity;
        state.min_region_length = self.min_region_length;
        state.auto_analyze = self.auto_analyze;
    }
}

/// Analysis panel tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisTab {
    /// Fingerprint overview
    Fingerprint,
    /// Stylistic departures from baseline
    Departures,
    /// Harmonic analysis
    Harmony,
    /// Compositional insights and suggestions
    Insights,
    /// Reference section comparison
    Comparison,
}

impl Default for AnalysisPanelState {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisPanelState {
    pub fn new() -> Self {
        Self {
            analysis_done: false,
            analyzing: false,
            progress: 0.0,
            fingerprint: None,
            anomaly_report: None,
            harmony: None,
            surgical_plan: None,
            excluded_ranges: Vec::new(),
            sensitivity: 0.5,
            min_region_length: 5,
            show_highlights: true,
            active_tab: AnalysisTab::Fingerprint,
            last_error: None,
            selected_prescription: None,
            reference_regions: Vec::new(),
            auto_analyze: true,
            last_change_time_ms: 0,
            debounce_ms: 1500, // 1.5 seconds debounce
            needs_analysis: false,
            annotations: std::collections::HashMap::new(),
            pending_clipboard: None,
            exclusion_undo_stack: Vec::new(),
            exclusion_redo_stack: Vec::new(),
        }
    }

    /// Push current exclusions to undo stack before making changes
    fn push_exclusion_undo(&mut self) {
        self.exclusion_undo_stack.push(self.excluded_ranges.clone());
        self.exclusion_redo_stack.clear(); // Clear redo on new change
        // Limit undo stack size
        if self.exclusion_undo_stack.len() > 50 {
            self.exclusion_undo_stack.remove(0);
        }
    }

    /// Undo last exclusion change
    pub fn undo_exclusion(&mut self) -> bool {
        if let Some(prev) = self.exclusion_undo_stack.pop() {
            self.exclusion_redo_stack.push(self.excluded_ranges.clone());
            self.excluded_ranges = prev;
            true
        } else {
            false
        }
    }

    /// Redo last undone exclusion change
    pub fn redo_exclusion(&mut self) -> bool {
        if let Some(next) = self.exclusion_redo_stack.pop() {
            self.exclusion_undo_stack.push(self.excluded_ranges.clone());
            self.excluded_ranges = next;
            true
        } else {
            false
        }
    }

    /// Check if undo is available
    pub fn can_undo_exclusion(&self) -> bool {
        !self.exclusion_undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo_exclusion(&self) -> bool {
        !self.exclusion_redo_stack.is_empty()
    }

    /// Add or update an annotation for a measure
    pub fn add_annotation(&mut self, measure: usize, text: String) {
        if text.is_empty() {
            self.annotations.remove(&measure);
        } else {
            self.annotations.insert(measure, text);
        }
    }

    /// Generate a markdown report of the analysis
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# Compositional Analysis Report\n\n");

        if let Some(ref fp) = self.fingerprint {
            report.push_str("## Fingerprint\n\n");
            report.push_str(&format!("- Fret Variance: {:.1}\n", fp.fret_variance));
            report.push_str(&format!("- Note Density: {:.2} notes/beat\n", fp.density));
            report.push_str(&format!("- Dissonance Index: {:.2}\n", fp.dissonance));
            report.push_str(&format!("- Total Notes: {}\n", fp.total_notes));
            report.push_str(&format!("- Measures Analyzed: {}\n\n", fp.source_measures.len()));
        }

        if let Some(ref ar) = self.anomaly_report {
            report.push_str("## Stylistic Departures\n\n");
            if ar.regions.is_empty() {
                report.push_str("Your voice is consistent throughout - no significant departures detected.\n\n");
            } else {
                for region in &ar.regions {
                    let severity = Severity::from_score(region.score);
                    report.push_str(&format!(
                        "### Measures {}-{} [{}]\n",
                        region.start + 1, region.end + 1, severity.label()
                    ));
                    for issue in &region.issues {
                        report.push_str(&format!("- {}\n", issue));
                    }
                    report.push('\n');
                }
            }
        }

        if !self.annotations.is_empty() {
            report.push_str("## Annotations\n\n");
            let mut measures: Vec<_> = self.annotations.keys().collect();
            measures.sort();
            for measure in measures {
                if let Some(text) = self.annotations.get(measure) {
                    report.push_str(&format!("- **M{}**: {}\n", measure + 1, text));
                }
            }
        }

        report
    }

    /// Mark analysis as stale due to document change
    pub fn mark_stale(&mut self, current_time_ms: u64) {
        self.needs_analysis = true;
        self.last_change_time_ms = current_time_ms;
    }

    /// Check if debounce period has elapsed and analysis should run
    pub fn should_run_auto_analysis(&self, current_time_ms: u64) -> bool {
        self.auto_analyze
            && self.needs_analysis
            && !self.analyzing
            && current_time_ms >= self.last_change_time_ms + self.debounce_ms
    }

    /// Clear stale flag after analysis completes
    pub fn analysis_completed(&mut self) {
        self.needs_analysis = false;
        self.analysis_done = true;
        self.analyzing = false;
    }

    /// Add a user-defined reference region
    pub fn add_reference(&mut self, start: usize, end: usize) {
        if !self.reference_regions.iter().any(|(s, e)| *s == start && *e == end) {
            self.reference_regions.push((start, end));
        }
    }

    /// Run analysis on a document
    pub fn analyze(&mut self, document: &TabDocument) {
        // Calculate fingerprint (excluding marked regions)
        let fingerprint = if self.excluded_ranges.is_empty() {
            ComposerFingerprint::from_document(document)
        } else {
            ComposerFingerprint::excluding_ranges(document, &self.excluded_ranges)
        };

        // Run anomaly detection
        let config = AnomalyConfig {
            score_threshold: 1.5 - (self.sensitivity * 0.5),
            region_gap: 4,
            min_region_length: self.min_region_length,
            ..Default::default()
        };
        let detector = AnomalyDetector::with_config(fingerprint.clone(), config);
        let anomaly_report = detector.analyze(document);

        // Run harmonic analysis
        let analyzer = HarmonicAnalyzer::new();
        let harmony = analyzer.analyze(document);

        // Generate surgical plan
        let planner = SurgicalPlanner::new(fingerprint.clone()).with_harmony(harmony.clone());
        let surgical_plan = planner.plan_from_report(&anomaly_report, document);

        self.fingerprint = Some(fingerprint);
        self.anomaly_report = Some(anomaly_report);
        self.harmony = Some(harmony);
        self.surgical_plan = Some(surgical_plan);
        self.analysis_done = true;
        self.analyzing = false;
        self.progress = 1.0;
    }

    /// Add a region to exclude from fingerprint
    pub fn exclude_region(&mut self, start: usize, end: usize) {
        if !self.excluded_ranges.iter().any(|(s, e)| *s == start && *e == end) {
            self.push_exclusion_undo();
            self.excluded_ranges.push((start, end));
        }
    }

    /// Clear all exclusions
    pub fn clear_exclusions(&mut self) {
        if !self.excluded_ranges.is_empty() {
            self.push_exclusion_undo();
            self.excluded_ranges.clear();
        }
    }

    /// Check if a measure is in an anomalous region
    pub fn is_anomalous(&self, measure: usize) -> bool {
        self.anomaly_report.as_ref()
            .map(|r| r.is_anomalous(measure))
            .unwrap_or(false)
    }

    /// Get severity color for a region
    pub fn severity_color(severity: Severity) -> Color32 {
        match severity {
            Severity::Minor => Color32::from_rgb(255, 206, 86),
            Severity::Moderate => Color32::from_rgb(255, 159, 64),
            Severity::Major => Color32::from_rgb(255, 99, 132),
            Severity::Critical => Color32::from_rgb(153, 102, 255),
        }
    }
}

/// Compositional Analysis Panel component
pub struct AnalysisPanel<'a> {
    state: &'a mut AnalysisPanelState,
    #[allow(dead_code)] // Reserved for future inline re-analysis
    document: &'a TabDocument,
}

impl<'a> AnalysisPanel<'a> {
    pub fn new(state: &'a mut AnalysisPanelState, document: &'a TabDocument) -> Self {
        Self { state, document }
    }

    /// Show the analysis panel and return any actions triggered
    pub fn show(&mut self, ui: &mut Ui) -> Option<AnalysisPanelAction> {
        let mut action = None;

        // Auto-run analysis on first open if enabled and not yet done
        if self.state.auto_analyze && !self.state.analysis_done && !self.state.analyzing {
            action = Some(AnalysisPanelAction::RunAnalysis);
        }

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading("🎼 Compositional Analysis");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(if self.state.show_highlights { "🔍 Highlights On" } else { "🔍 Highlights Off" })
                        .on_hover_text("Toggle departure highlighting on timeline")
                        .clicked()
                    {
                        self.state.show_highlights = !self.state.show_highlights;
                        action = Some(AnalysisPanelAction::ToggleHighlights);
                    }
                });
            });
            ui.separator();

            // Error display
            let mut clear_error = false;
            if let Some(ref error) = self.state.last_error {
                let error_msg = error.clone();
                ui.horizontal(|ui| {
                    ui.colored_label(Color32::from_rgb(231, 76, 60), format!("⚠ {}", error_msg));
                });
                if ui.small_button("✕").clicked() {
                    clear_error = true;
                }
                ui.add_space(4.0);
            }
            if clear_error {
                self.state.last_error = None;
            }

            // Analysis controls
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Sensitivity:");
                    let old_sens = self.state.sensitivity;
                    ui.add(egui::Slider::new(&mut self.state.sensitivity, 0.1..=0.9)
                        .show_value(false))
                        .on_hover_text("Higher = detect more subtle deviations");

                    if self.state.sensitivity != old_sens {
                        action = Some(AnalysisPanelAction::UpdateSensitivity(self.state.sensitivity));
                    }

                    ui.add_space(8.0);

                    ui.label("Min region:");
                    ui.add(egui::DragValue::new(&mut self.state.min_region_length)
                        .range(2..=16)
                        .suffix(" bars"));
                });

                ui.horizontal(|ui| {
                    // Excluded regions summary
                    // Exclusion undo/redo buttons
                    if self.state.can_undo_exclusion() {
                        if ui.small_button("↶").on_hover_text("Undo exclusion").clicked() {
                            action = Some(AnalysisPanelAction::UndoExclusion);
                        }
                    }
                    if self.state.can_redo_exclusion() {
                        if ui.small_button("↷").on_hover_text("Redo exclusion").clicked() {
                            action = Some(AnalysisPanelAction::RedoExclusion);
                        }
                    }

                    if !self.state.excluded_ranges.is_empty() {
                        let excluded_text = self.state.excluded_ranges.iter()
                            .map(|(s, e)| format!("{}-{}", s, e))
                            .collect::<Vec<_>>()
                            .join(", ");
                        ui.label(RichText::new(format!("Excluded: {}", excluded_text)).small());
                        if ui.small_button("✕").on_hover_text("Clear exclusions").clicked() {
                            action = Some(AnalysisPanelAction::ClearExclusions);
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.state.analyzing {
                            ui.add(ProgressBar::new(self.state.progress));
                            ui.spinner();
                        } else {
                            // Stale indicator
                            if self.state.needs_analysis && self.state.analysis_done {
                                ui.label(RichText::new("⟳").color(Color32::from_rgb(241, 196, 15)))
                                    .on_hover_text("Analysis outdated - document changed");
                            }
                            if ui.button("▶ Run").on_hover_text("Run Analysis").clicked() {
                                action = Some(if self.state.excluded_ranges.is_empty() {
                                    AnalysisPanelAction::RunAnalysis
                                } else {
                                    AnalysisPanelAction::RunAnalysisExcluding(self.state.excluded_ranges.clone())
                                });
                            }
                        }
                        // Export button
                        if self.state.analysis_done {
                            if ui.small_button("📋").on_hover_text("Copy report to clipboard").clicked() {
                                action = Some(AnalysisPanelAction::ExportToClipboard);
                            }
                        }

                        // Auto-analyze toggle
                        let auto_text = if self.state.auto_analyze { "🔄" } else { "⏸" };
                        let auto_tip = if self.state.auto_analyze {
                            "Auto-analyze ON (click to disable)"
                        } else {
                            "Auto-analyze OFF (click to enable)"
                        };
                        if ui.small_button(auto_text).on_hover_text(auto_tip).clicked() {
                            self.state.auto_analyze = !self.state.auto_analyze;
                        }
                    });
                });
            });

            ui.add_space(8.0);

            // Tab bar
            ui.horizontal(|ui| {
                let tabs = [
                    (AnalysisTab::Fingerprint, "📊 Fingerprint"),
                    (AnalysisTab::Departures, "🔀 Departures"),
                    (AnalysisTab::Harmony, "🎵 Harmony"),
                    (AnalysisTab::Insights, "💡 Insights"),
                    (AnalysisTab::Comparison, "🔍 Compare"),
                ];

                for (tab, label) in tabs {
                    let is_selected = self.state.active_tab == tab;
                    if ui.selectable_label(is_selected, label).clicked() {
                        self.state.active_tab = tab;
                    }
                }
            });

            ui.separator();

            // Tab content
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    match self.state.active_tab {
                        AnalysisTab::Fingerprint => self.show_fingerprint(ui, &mut action),
                        AnalysisTab::Departures => self.show_departures(ui, &mut action),
                        AnalysisTab::Harmony => self.show_harmony(ui),
                        AnalysisTab::Insights => self.show_insights(ui, &mut action),
                        AnalysisTab::Comparison => self.show_comparison(ui, &mut action),
                    }
                });
        });

        action
    }

    /// Show fingerprint tab
    fn show_fingerprint(&self, ui: &mut Ui, _action: &mut Option<AnalysisPanelAction>) {
        if let Some(ref fp) = self.state.fingerprint {
            ui.group(|ui| {
                ui.label(RichText::new("Baseline Metrics").strong());
                ui.add_space(4.0);

                // Metrics grid
                egui::Grid::new("fingerprint_metrics")
                    .num_columns(2)
                    .spacing([16.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Fret Variance:");
                        ui.label(RichText::new(format!("{:.1}", fp.fret_variance)).monospace());
                        ui.end_row();

                        ui.label("Note Density:");
                        ui.label(RichText::new(format!("{:.2} notes/beat", fp.density)).monospace());
                        ui.end_row();

                        ui.label("Dissonance Index:");
                        ui.label(RichText::new(format!("{:.2}", fp.dissonance)).monospace());
                        ui.end_row();

                        ui.label("Measures Analyzed:");
                        ui.label(RichText::new(format!("{}", fp.source_measures.len())).monospace());
                        ui.end_row();

                        ui.label("Total Notes:");
                        ui.label(RichText::new(format!("{}", fp.total_notes)).monospace());
                        ui.end_row();
                    });
            });

            ui.add_space(8.0);

            // Root distribution
            ui.group(|ui| {
                ui.label(RichText::new("Root Distribution").strong());
                ui.add_space(4.0);

                for (root, pct) in fp.dominant_roots(5) {
                    ui.horizontal(|ui| {
                        let name = pitch_name(root);
                        ui.label(format!("{:>2}:", name));
                        let bar_width = (pct * 2.0).min(200.0);
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, 14.0), Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(52, 152, 219));
                        ui.label(format!("{:.1}%", pct));
                    });
                }
            });

            ui.add_space(8.0);

            // Interval vocabulary
            ui.group(|ui| {
                ui.label(RichText::new("Interval Vocabulary").strong());
                ui.add_space(4.0);

                for (interval, pct) in fp.dominant_intervals(5) {
                    ui.horizontal(|ui| {
                        let name = interval_name(interval);
                        ui.label(format!("{:>8}:", name));
                        let bar_width = (pct * 2.0).min(200.0);
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, 14.0), Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(155, 89, 182));
                        ui.label(format!("{:.1}%", pct));
                    });
                }
            });

            ui.add_space(8.0);

            // Signature patterns
            ui.group(|ui| {
                ui.label(RichText::new("Detected Patterns").strong());
                ui.add_space(4.0);

                if fp.has_m7_signature() {
                    ui.label(RichText::new("✓ M7 Signature (semitone tension stacking)")
                        .color(Color32::from_rgb(46, 204, 113)));
                }

                if fp.has_chromatic_oscillation() {
                    ui.label(RichText::new("✓ E↔F Oscillation (bitonal root motion)")
                        .color(Color32::from_rgb(46, 204, 113)));
                }

                if !fp.has_m7_signature() && !fp.has_chromatic_oscillation() {
                    ui.label(RichText::new("No distinctive patterns detected").color(Color32::GRAY));
                }
            });

            ui.add_space(8.0);

            // Density waveform visualization
            self.draw_density_waveform(ui, fp);
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Run analysis to see fingerprint data");
            });
        }
    }

    /// Show departures tab - sections that diverge from baseline style
    fn show_departures(&self, ui: &mut Ui, action: &mut Option<AnalysisPanelAction>) {
        if let Some(ref report) = self.state.anomaly_report {
            if report.regions.is_empty() {
                ui.group(|ui| {
                    ui.label(RichText::new("✓ Consistent voice throughout")
                        .color(Color32::from_rgb(46, 204, 113)));
                    ui.label("No significant departures from your baseline style detected.");
                });
            } else {
                ui.label(format!(
                    "Found {} section(s) departing from baseline ({} measures)",
                    report.regions.len(),
                    report.total_anomalous_measures()
                ));
                ui.label(RichText::new("These may be intentional contrasts or areas to revisit.")
                    .small().italics().color(Color32::GRAY));

                ui.add_space(8.0);

                for region in report.regions.iter() {
                    let deviation = report.region_deviation(region);
                    let severity = Severity::from_score(region.score);
                    let color = AnalysisPanelState::severity_color(severity);

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // Divergence indicator
                            let (indicator_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
                            ui.painter().circle_filled(indicator_rect.center(), 4.0, color);

                            ui.label(RichText::new(format!(
                                "Measures {}-{}", region.start + 1, region.end + 1
                            )).strong());

                            ui.label(format!("[{}]", Self::divergence_label(severity)));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("▶").on_hover_text("Jump to section").clicked() {
                                    *action = Some(AnalysisPanelAction::JumpToRegion(region.start, region.end));
                                }
                                // Practice button - tempo based on divergence
                                let start_tempo = match severity {
                                    Severity::Critical => 40.0,
                                    Severity::Major => 50.0,
                                    Severity::Moderate => 60.0,
                                    Severity::Minor => 75.0,
                                };
                                if ui.small_button("🎯").on_hover_text(format!("Practice at {}% tempo", start_tempo as u8)).clicked() {
                                    *action = Some(AnalysisPanelAction::PracticeRegion {
                                        start: region.start,
                                        end: region.end,
                                        start_tempo_percent: start_tempo,
                                    });
                                }
                                if ui.small_button("📌").on_hover_text("Exclude from baseline (will re-analyze)").clicked() {
                                    *action = Some(AnalysisPanelAction::ExcludeAndReanalyze(region.start, region.end));
                                }
                            });
                        });

                        // Observations about this section
                        ui.label(RichText::new(region.issues.join("; ")).small().color(Color32::GRAY));

                        // Deviation metrics
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!(
                                "Variance: {:+.0}%  Dissonance: {:+.0}%  Density: {:+.0}%",
                                deviation.variance_delta,
                                deviation.dissonance_delta,
                                deviation.density_delta
                            )).small());
                        });
                    });

                    ui.add_space(4.0);
                }
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Run analysis to explore your compositional voice");
            });
        }
    }

    /// Get user-friendly divergence label (non-judgmental)
    fn divergence_label(severity: Severity) -> &'static str {
        match severity {
            Severity::Critical => "Strong departure",
            Severity::Major => "Notable shift",
            Severity::Moderate => "Moderate variation",
            Severity::Minor => "Subtle departure",
        }
    }

    /// Show harmony tab
    fn show_harmony(&self, ui: &mut Ui) {
        if let Some(ref harmony) = self.state.harmony {
            ui.group(|ui| {
                ui.label(RichText::new("Tonal Analysis").strong());
                ui.add_space(4.0);

                ui.label(format!("Total chords analyzed: {}", harmony.chords.len()));

                if let Some(center) = harmony.primary_tonal_center() {
                    ui.label(format!(
                        "Tonal center: {} (clarity: {:.0}%)",
                        pitch_name(center),
                        harmony.tonal_clarity() * 100.0
                    ));
                }

                if harmony.is_tonally_ambiguous() {
                    ui.label(RichText::new("⚠ HIGHLY AMBIGUOUS: Multiple competing centers")
                        .color(Color32::from_rgb(241, 196, 15)));
                }

                if harmony.is_chromatically_saturated() {
                    ui.label(RichText::new("✓ Chromatic saturation (full 12-tone usage)")
                        .color(Color32::from_rgb(155, 89, 182)));
                }
            });

            ui.add_space(8.0);

            // Motion breakdown
            ui.group(|ui| {
                ui.label(RichText::new("Root Motion Patterns").strong());
                ui.add_space(4.0);

                let motion = harmony.motion_breakdown();
                egui::Grid::new("motion_breakdown")
                    .num_columns(2)
                    .spacing([16.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Static (pedal):");
                        ui.label(format!("{:.1}%", motion.static_pct));
                        ui.end_row();

                        ui.label("Chromatic (±1 st):");
                        ui.label(format!("{:.1}%", motion.chromatic_pct));
                        ui.end_row();

                        ui.label("Tritone (±6 st):");
                        ui.label(format!("{:.1}%", motion.tritone_pct));
                        ui.end_row();

                        ui.label("4th/5th:");
                        ui.label(format!("{:.1}%", motion.fourth_fifth_pct));
                        ui.end_row();
                    });

                if motion.is_chromatic_dominant() {
                    ui.label(RichText::new("→ Predominantly chromatic motion")
                        .color(Color32::from_rgb(230, 126, 34)));
                }
            });

            ui.add_space(8.0);

            // Tritone usage
            ui.group(|ui| {
                ui.label(RichText::new("Tritone Usage").strong());
                ui.add_space(4.0);

                ui.label(format!(
                    "{} chords ({:.1}%)",
                    harmony.tritone_stats.count,
                    harmony.tritone_stats.percentage
                ));
            });

            ui.add_space(8.0);

            // Detected patterns
            if harmony.has_ef_oscillation() || harmony.has_m7_signature() {
                ui.group(|ui| {
                    ui.label(RichText::new("Signature Patterns").strong());
                    ui.add_space(4.0);

                    if harmony.has_ef_oscillation() {
                        ui.label(RichText::new("✓ E↔F bitonal oscillation")
                            .color(Color32::from_rgb(46, 204, 113)));
                    }

                    if harmony.has_m7_signature() {
                        ui.label(RichText::new("✓ M7 interval signature")
                            .color(Color32::from_rgb(46, 204, 113)));
                    }
                });
            }

            // Progressions
            if !harmony.progression_patterns.is_empty() {
                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Recurring Progressions").strong());
                    ui.add_space(4.0);

                    for (pattern, count) in harmony.progression_patterns.iter().take(5) {
                        ui.label(format!("{}: {} times", pattern, count));
                    }
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Run analysis to see harmonic data");
            });
        }
    }

    /// Show insights tab - suggestions for exploration
    fn show_insights(&self, ui: &mut Ui, action: &mut Option<AnalysisPanelAction>) {
        if let Some(ref plan) = self.state.surgical_plan {
            if plan.is_empty() {
                ui.group(|ui| {
                    ui.label(RichText::new("✓ Strong compositional consistency")
                        .color(Color32::from_rgb(46, 204, 113)));
                    ui.label("Your voice is clear and unified throughout.");
                });
            } else {
                ui.label(RichText::new(format!(
                    "Insights for {} measures worth exploring",
                    plan.total_measures_affected
                )).strong());
                ui.label(RichText::new("Consider these observations - they may inspire refinement or reveal intentional choices.")
                    .small().italics().color(Color32::GRAY));

                ui.add_space(8.0);

                for (i, prescription) in plan.prescriptions.iter().take(5).enumerate() {
                    let color = AnalysisPanelState::severity_color(prescription.severity);
                    let is_selected = self.state.selected_prescription == Some(i);

                    let frame_stroke = if is_selected {
                        egui::Stroke::new(2.0, AnalysisPanelState::severity_color(prescription.severity))
                    } else {
                        egui::Stroke::NONE
                    };

                    egui::Frame::none()
                        .stroke(frame_stroke)
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let (indicator_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
                                ui.painter().circle_filled(indicator_rect.center(), 4.0, color);

                                ui.label(RichText::new(format!(
                                    "{}. Measures {}-{}",
                                    i + 1,
                                    prescription.region.start + 1,
                                    prescription.region.end + 1
                                )).strong());

                                ui.label(format!("[{}]", Self::divergence_label(prescription.severity)));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Explore button
                                    if ui.small_button("🔍 Try")
                                        .on_hover_text("Apply adjustments (use Ctrl+Z to undo)")
                                        .clicked()
                                    {
                                        *action = Some(AnalysisPanelAction::AutoFixRegion {
                                            start: prescription.region.start,
                                            end: prescription.region.end,
                                        });
                                    }
                                    // Practice button
                                    let start_tempo = match prescription.severity {
                                        Severity::Critical => 40.0,
                                        Severity::Major => 50.0,
                                        Severity::Moderate => 60.0,
                                        Severity::Minor => 75.0,
                                    };
                                    if ui.small_button("🎯")
                                        .on_hover_text(format!("Practice at {}% tempo", start_tempo as u8))
                                        .clicked()
                                    {
                                        *action = Some(AnalysisPanelAction::PracticeRegion {
                                            start: prescription.region.start,
                                            end: prescription.region.end,
                                            start_tempo_percent: start_tempo,
                                        });
                                    }
                                    if ui.small_button("▶").on_hover_text("Jump to section").clicked() {
                                        *action = Some(AnalysisPanelAction::JumpToMeasure(prescription.region.start));
                                    }
                                });
                            });

                            ui.label(RichText::new(&prescription.diagnosis).color(Color32::from_rgb(155, 89, 182)));

                            ui.add_space(4.0);

                            for action_item in &prescription.actions {
                                ui.horizontal(|ui| {
                                    ui.label(action_item.action_type.icon());
                                    ui.label(&action_item.description);
                                });
                            }

                            // Target metrics (optional context)
                            if let Some(target_var) = prescription.targets.variance_target {
                                ui.add_space(4.0);
                                ui.label(RichText::new(format!(
                                    "Baseline variance: {:.1}+",
                                    target_var
                                )).small().color(Color32::GRAY));
                            }

                            // Reference measure copy buttons
                            if !prescription.reference_measures.is_empty() {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Draw from:").small());
                                    for &ref_measure in prescription.reference_measures.iter().take(3) {
                                        if ui.small_button(format!("M{}", ref_measure + 1))
                                            .on_hover_text(format!("Apply patterns from measure {}", ref_measure + 1))
                                            .clicked()
                                        {
                                            *action = Some(AnalysisPanelAction::CopyFromReference {
                                                source_measure: ref_measure,
                                                target_start: prescription.region.start,
                                                target_end: prescription.region.end,
                                            });
                                        }
                                    }
                                });
                            }
                        });

                    ui.add_space(4.0);
                }

                // Reference measures (global)
                if !plan.reference_measures.is_empty() {
                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Your Signature Sections").strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("+ Mark Selection")
                                    .on_hover_text("Mark selection (or current measure) as signature section")
                                    .clicked()
                                {
                                    *action = Some(AnalysisPanelAction::MarkAsReference(0, 0));
                                }
                            });
                        });
                        ui.add_space(4.0);

                        for reference in plan.reference_measures.iter().take(5) {
                            ui.horizontal(|ui| {
                                if ui.small_button(format!("▶ M{}", reference.measure + 1))
                                    .on_hover_text("Jump to this section")
                                    .clicked()
                                {
                                    *action = Some(AnalysisPanelAction::JumpToMeasure(reference.measure));
                                }
                                ui.label(RichText::new(&reference.description).small());
                            });
                        }
                    });
                }
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Run analysis to discover insights");
            });
        }
    }

    /// Show reference comparison tab
    fn show_comparison(&self, ui: &mut Ui, action: &mut Option<AnalysisPanelAction>) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Reference Sections").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("+ Mark Selection")
                        .on_hover_text("Mark selection (or current measure) as reference")
                        .clicked()
                    {
                        *action = Some(AnalysisPanelAction::MarkAsReference(0, 0));
                    }
                });
            });
            ui.add_space(4.0);

            if self.state.reference_regions.is_empty() {
                ui.label(RichText::new("No reference sections marked")
                    .italics()
                    .color(Color32::GRAY));
                ui.label("Mark your best-written sections as references to compare against other parts.");
            } else {
                for (i, (start, end)) in self.state.reference_regions.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let color = Color32::from_rgb(46, 204, 113); // Green for reference
                        let (indicator_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
                        ui.painter().circle_filled(indicator_rect.center(), 4.0, color);

                        ui.label(RichText::new(format!("Reference {}: M{}-{}", i + 1, start + 1, end + 1)).strong());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("▶").on_hover_text("Jump to reference").clicked() {
                                *action = Some(AnalysisPanelAction::JumpToMeasure(*start));
                            }
                        });
                    });
                }
            }
        });

        ui.add_space(8.0);

        // Comparison with anomalous regions
        if let (Some(ref fingerprint), Some(ref report)) =
            (&self.state.fingerprint, &self.state.anomaly_report)
        {
            if !report.regions.is_empty() {
                ui.group(|ui| {
                    ui.label(RichText::new("How Departures Compare to Your Baseline").strong());
                    ui.add_space(4.0);

                    ui.label(RichText::new("Baseline (reference fingerprint):").small().strong());
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!(
                            "Variance: {:.1}  Dissonance: {:.1}  Density: {:.1}",
                            fingerprint.fret_variance,
                            fingerprint.dissonance,
                            fingerprint.density
                        )).small());
                    });

                    ui.add_space(8.0);

                    for region in report.regions.iter().take(5) {
                        let deviation = report.region_deviation(region);
                        let severity = Severity::from_score(region.score);
                        let color = AnalysisPanelState::severity_color(severity);

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let (indicator_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
                                ui.painter().circle_filled(indicator_rect.center(), 4.0, color);

                                ui.label(RichText::new(format!(
                                    "M{}-{}", region.start + 1, region.end + 1
                                )).strong());

                                ui.label(format!("[{}]", severity.label()));
                            });

                            // Detailed comparison
                            ui.add_space(4.0);
                            self.draw_comparison_bar(ui, "Variance", deviation.variance_delta, -50.0, 100.0);
                            self.draw_comparison_bar(ui, "Dissonance", deviation.dissonance_delta, -30.0, 50.0);
                            self.draw_comparison_bar(ui, "Density", deviation.density_delta, -50.0, 50.0);

                            ui.horizontal(|ui| {
                                if ui.small_button("▶").on_hover_text("Jump to region").clicked() {
                                    *action = Some(AnalysisPanelAction::JumpToRegion(region.start, region.end));
                                }
                                if ui.small_button("🎯").on_hover_text("Practice this section").clicked() {
                                    let start_tempo = match severity {
                                        Severity::Critical => 40.0,
                                        Severity::Major => 50.0,
                                        Severity::Moderate => 60.0,
                                        Severity::Minor => 75.0,
                                    };
                                    *action = Some(AnalysisPanelAction::PracticeRegion {
                                        start: region.start,
                                        end: region.end,
                                        start_tempo_percent: start_tempo,
                                    });
                                }
                            });
                        });
                        ui.add_space(4.0);
                    }
                });
            } else {
                ui.group(|ui| {
                    ui.label(RichText::new("✓ Unified voice - all sections align with your baseline")
                        .color(Color32::from_rgb(46, 204, 113)));
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Run analysis to compare sections");
            });
        }
    }

    /// Draw density waveform visualization
    fn draw_density_waveform(&self, ui: &mut Ui, fp: &ComposerFingerprint) {
        ui.group(|ui| {
            ui.label(RichText::new("Density Waveform").strong());
            ui.add_space(4.0);

            // Get per-track density data
            if fp.track_analysis.is_empty() {
                ui.label(RichText::new("No track data available").color(Color32::GRAY));
                return;
            }

            let waveform_height = 60.0;
            let available_width = ui.available_width() - 16.0;

            // Calculate density per track from note count
            let max_notes = fp.track_analysis.iter()
                .map(|t| t.note_count as f32)
                .fold(0.0f32, |a, b| a.max(b))
                .max(1.0);

            let (rect, _response) = ui.allocate_exact_size(
                Vec2::new(available_width, waveform_height),
                Sense::hover()
            );

            // Background
            ui.painter().rect_filled(rect, 4.0, Color32::from_gray(30));

            // Draw baseline reference line (average notes per track)
            let avg_notes = fp.total_notes as f32 / fp.track_analysis.len().max(1) as f32;
            let baseline_y = rect.max.y - (avg_notes / max_notes * waveform_height).min(waveform_height);
            ui.painter().hline(
                rect.x_range(),
                baseline_y,
                egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(46, 204, 113, 100))
            );

            // Draw track note counts as bars
            if !fp.track_analysis.is_empty() {
                let bar_width = available_width / fp.track_analysis.len() as f32;

                for (i, track) in fp.track_analysis.iter().enumerate() {
                    let x = rect.min.x + i as f32 * bar_width;
                    let notes = track.note_count as f32;
                    let height = (notes / max_notes * waveform_height).min(waveform_height);
                    let y = rect.max.y - height;

                    // Color based on how much above/below average
                    let ratio = notes / avg_notes.max(1.0);
                    let color = if ratio > 1.2 {
                        Color32::from_rgb(231, 76, 60) // Red - above baseline
                    } else if ratio < 0.8 {
                        Color32::from_rgb(52, 152, 219) // Blue - below baseline
                    } else {
                        Color32::from_rgb(46, 204, 113) // Green - near baseline
                    };

                    let bar_rect = egui::Rect::from_min_max(
                        egui::Pos2::new(x + 1.0, y),
                        egui::Pos2::new(x + bar_width - 1.0, rect.max.y)
                    );
                    ui.painter().rect_filled(bar_rect, 2.0, color);
                }
            }

            // Legend
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("●").color(Color32::from_rgb(46, 204, 113)).small());
                ui.label(RichText::new("Baseline").small());
                ui.add_space(8.0);
                ui.label(RichText::new("●").color(Color32::from_rgb(231, 76, 60)).small());
                ui.label(RichText::new("Dense").small());
                ui.add_space(8.0);
                ui.label(RichText::new("●").color(Color32::from_rgb(52, 152, 219)).small());
                ui.label(RichText::new("Sparse").small());
            });
        });
    }

    /// Draw a comparison bar showing deviation from baseline
    fn draw_comparison_bar(&self, ui: &mut Ui, label: &str, value: f32, min: f32, max: f32) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{:12}", label)).small().monospace());

            let bar_width = 100.0;
            let bar_height = 8.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), Sense::hover());

            // Background
            ui.painter().rect_filled(rect, 2.0, Color32::from_gray(40));

            // Center line
            let center_x = rect.center().x;
            ui.painter().vline(center_x, rect.y_range(), egui::Stroke::new(1.0, Color32::GRAY));

            // Value bar
            let normalized = value.clamp(min, max);
            let bar_fraction = (normalized - min) / (max - min);
            let value_x = rect.min.x + bar_width * bar_fraction;

            let bar_color = if value > 0.0 {
                Color32::from_rgb(231, 76, 60) // Red for above baseline
            } else {
                Color32::from_rgb(52, 152, 219) // Blue for below baseline
            };

            if value > 0.0 {
                let bar_rect = egui::Rect::from_min_max(
                    egui::Pos2::new(center_x, rect.min.y),
                    egui::Pos2::new(value_x, rect.max.y),
                );
                ui.painter().rect_filled(bar_rect, 0.0, bar_color);
            } else {
                let bar_rect = egui::Rect::from_min_max(
                    egui::Pos2::new(value_x, rect.min.y),
                    egui::Pos2::new(center_x, rect.max.y),
                );
                ui.painter().rect_filled(bar_rect, 0.0, bar_color);
            }

            ui.label(RichText::new(format!("{:+.0}%", value)).small());
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_panel_state_default() {
        let state = AnalysisPanelState::new();
        assert!(!state.analysis_done);
        assert!(state.fingerprint.is_none());
        assert!(state.excluded_ranges.is_empty());
    }

    #[test]
    fn test_exclude_region() {
        let mut state = AnalysisPanelState::new();
        state.exclude_region(10, 20);
        state.exclude_region(30, 40);

        assert_eq!(state.excluded_ranges.len(), 2);
        assert!(state.excluded_ranges.contains(&(10, 20)));
        assert!(state.excluded_ranges.contains(&(30, 40)));

        // Duplicate should not be added
        state.exclude_region(10, 20);
        assert_eq!(state.excluded_ranges.len(), 2);
    }

    #[test]
    fn test_clear_exclusions() {
        let mut state = AnalysisPanelState::new();
        state.exclude_region(10, 20);
        state.clear_exclusions();
        assert!(state.excluded_ranges.is_empty());
    }

    #[test]
    fn test_severity_color() {
        let minor = AnalysisPanelState::severity_color(Severity::Minor);
        let critical = AnalysisPanelState::severity_color(Severity::Critical);
        assert_ne!(minor, critical);
    }
}
