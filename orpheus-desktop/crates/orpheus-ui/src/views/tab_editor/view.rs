//! Tab Editor View
//!
//! Main UI component for the tablature editor with keyboard-driven workflow.

use egui::{Color32, Pos2, Rect, Rounding, ScrollArea, Sense, Stroke, Ui, Vec2};
use orpheus_core::tab::{Instrument, TabTrack, TrackMeasure};

use crate::theme::Theme;
use super::state::{TabEditorState, EditorMode, ActiveTool};
use super::input::InputResult;
use super::render::{
    TOOLBAR_HEIGHT, STRING_HEIGHT, FRET_WIDTH, MEASURE_HEADER_HEIGHT,
    STATUS_HEIGHT, TRACK_LABEL_WIDTH,
    draw_note_cell, draw_string_lines, draw_measure_lines, draw_measure_header,
    draw_track_label, draw_playhead, draw_loop_region, draw_count_in_overlay,
    draw_status_bar, tool_indicator, draw_mode_indicator, draw_technique_help_overlay,
    draw_fret_buffer_overlay,
};
use super::stage_view::{StageView, StageViewAction, StagePosition, instrument_templates};
use super::section_analysis::{SectionAnalysisDialog, SectionAnalysisAction, SectionNavigator, analyze_sections};
use super::track_groups::{TrackGroupsPanel, TrackGroupAction};
use crate::panels::{AnalysisPanel, AnalysisPanelAction};

/// Action result from the tab editor
#[derive(Debug, Clone, PartialEq)]
pub enum TabEditorAction {
    /// No action
    None,
    /// Editor closed
    Close,
    /// Open file requested
    Open,
    /// Save requested
    Save,
    /// Export MIDI requested
    ExportMidi,
    /// Export PDF requested
    ExportPdf,
    /// Export audio (WAV) requested
    ExportAudio,
    /// Play/stop toggled
    TogglePlayback,
    /// Document modified
    Modified,
    /// Preview a note (for audio feedback)
    /// Contains (string, fret, velocity)
    PreviewNote {
        /// String number (1-indexed, 1 = highest pitch)
        string: u8,
        /// Fret number
        fret: u8,
        /// Velocity (0-127)
        velocity: u8,
    },
    /// Switch to record mode with current tempo/time signature
    SwitchToRecordMode {
        /// Current tempo in BPM
        tempo: f64,
        /// Time signature numerator
        time_sig_num: u8,
        /// Time signature denominator
        time_sig_denom: u8,
        /// Track name (for creating recording track)
        track_name: String,
    },
    /// Import MIDI file requested
    ImportMidi,
}

/// Tab Editor View component
pub struct TabEditorView<'a> {
    state: &'a mut TabEditorState,
    theme: &'a Theme,
}

impl<'a> TabEditorView<'a> {
    pub fn new(state: &'a mut TabEditorState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    /// Show the tab editor and return any actions
    pub fn show(&mut self, ui: &mut Ui) -> TabEditorAction {
        let mut action = TabEditorAction::None;

        // Get current time for fret buffer timeout and count-in
        let current_time = ui.ctx().input(|i| i.time);
        let current_time_ms = (current_time * 1000.0) as u64;

        // Update count-in state if counting in
        if self.state.is_counting_in {
            self.state.update_count_in(current_time);
            // Request repaint for smooth count-in animation
            ui.ctx().request_repaint();
        }

        // Handle pending clipboard operations
        if let Some(text) = self.state.analysis_panel.pending_clipboard.take() {
            ui.ctx().copy_text(text);
        }

        // Handle keyboard input
        self.handle_keyboard(ui, &mut action, current_time_ms);

        // Show side panels FIRST so they properly adjust the available space
        // Track Groups Panel (left side panel)
        let mut track_group_action: Option<TrackGroupAction> = None;
        if self.state.show_track_groups {
            egui::SidePanel::left("track_groups_panel")
                .resizable(true)
                .default_width(250.0)
                .show_inside(ui, |ui| {
                    let mut panel = TrackGroupsPanel::new(
                        self.state,
                        self.theme,
                    );
                    track_group_action = panel.show(ui);
                });
        }

        // Analysis Panel (right side panel)
        let mut analysis_action_result: Option<AnalysisPanelAction> = None;
        if self.state.show_analysis_panel {
            egui::SidePanel::right("analysis_panel")
                .resizable(true)
                .default_width(320.0)
                .show_inside(ui, |ui| {
                    let mut panel = AnalysisPanel::new(
                        &mut self.state.analysis_panel,
                        &self.state.document,
                    );
                    analysis_action_result = panel.show(ui);
                });
        }

        // Now calculate the remaining available space after side panels
        let available = ui.available_rect_before_wrap();

        // Background
        ui.painter().rect_filled(available, Rounding::ZERO, self.theme.panel_bg());

        // Toolbar
        let toolbar_rect = Rect::from_min_size(
            available.min,
            Vec2::new(available.width(), TOOLBAR_HEIGHT),
        );
        self.draw_toolbar(ui, toolbar_rect, &mut action);

        // Status bar at bottom
        let status_rect = Rect::from_min_size(
            Pos2::new(available.min.x, available.max.y - STATUS_HEIGHT),
            Vec2::new(available.width(), STATUS_HEIGHT),
        );

        // Main editing area
        let main_rect = Rect::from_min_max(
            Pos2::new(available.min.x, toolbar_rect.max.y),
            Pos2::new(available.max.x, status_rect.min.y),
        );

        // Show either stage view or tab editor
        if self.state.show_stage_view {
            // Create a child UI for the stage view area
            let mut child_ui = ui.child_ui(main_rect, egui::Layout::top_down(egui::Align::LEFT), None);

            // Show stage view
            let mut stage_view = StageView::new(
                &mut self.state.stage_view,
                &mut self.state.document.tracks,
                self.theme,
            );

            if let Some(stage_action) = stage_view.show(&mut child_ui) {
                self.handle_stage_action(stage_action, &mut action);
            }
        } else {
            // Draw main tab content with scrolling
            self.draw_main_area(ui, main_rect, &mut action);

            // Draw count-in overlay (on top of everything except status bar)
            if self.state.is_counting_in {
                draw_count_in_overlay(
                    ui,
                    main_rect,
                    self.state.count_in_beat,
                    self.state.practice.count_in_beats,
                    self.theme,
                );
            }
        }

        // Draw status bar
        draw_status_bar(ui, status_rect, self.state, self.theme);

        // Draw mode indicator badge (top-left of main area)
        if !self.state.show_stage_view {
            draw_mode_indicator(ui, main_rect, self.state.mode, self.theme);
        }

        // Draw fret buffer overlay (prominent display when entering multi-digit frets)
        if !self.state.fret_buffer.digits.is_empty() {
            // Position at top-center of main area for high visibility
            let overlay_pos = Pos2::new(main_rect.center().x, main_rect.min.y + 50.0);
            draw_fret_buffer_overlay(ui, overlay_pos, &self.state.fret_buffer.digits, self.theme);
        }

        // Draw technique help overlay (if active)
        if self.state.show_technique_help {
            draw_technique_help_overlay(ui, main_rect, self.theme);
        }

        // Section Analysis Dialog (modal, shown on top)
        if self.state.section_analysis.is_open {
            let mut dialog = SectionAnalysisDialog::new(
                &mut self.state.section_analysis,
                &self.state.document,
                self.theme,
            );

            if let Some(section_action) = dialog.show(ui) {
                self.handle_section_action(section_action, &mut action);
            }
        }

        // Handle deferred actions from panels (after all UI is drawn)
        if let Some(group_action) = track_group_action {
            self.handle_track_group_action(group_action);
        }
        if let Some(analysis_action) = analysis_action_result {
            self.handle_analysis_action(analysis_action, &mut action);
        }

        // Mark analysis as stale when document is modified
        if action == TabEditorAction::Modified {
            self.state.analysis_panel.mark_stale(current_time_ms);
        }

        // Check for debounced auto-analysis
        if self.state.analysis_panel.should_run_auto_analysis(current_time_ms) {
            self.state.run_analysis();
            self.state.analysis_panel.analysis_completed();
        }

        action
    }

    /// Handle section analysis actions
    fn handle_section_action(&mut self, section_action: SectionAnalysisAction, _action: &mut TabEditorAction) {
        match section_action {
            SectionAnalysisAction::Cancel => {
                // Dialog already closed itself
            }
            SectionAnalysisAction::Analyze => {
                // Analysis already happened in dialog
            }
            SectionAnalysisAction::Apply(detected_sections) => {
                // Convert detected sections to SectionMarkers and apply
                use orpheus_core::tab::SectionMarker;

                self.state.document.markers.clear();
                for section in detected_sections {
                    let mut marker = SectionMarker::new(section.start_measure, &section.name);
                    let color = section.color();
                    marker.color = Some((color.r(), color.g(), color.b()));
                    self.state.document.markers.push(marker);
                }

                self.state.status = format!("Applied {} sections", self.state.document.markers.len());
            }
            SectionAnalysisAction::JumpToSection(measure) => {
                self.state.cursor.measure = measure;
                self.state.cursor.beat = 0;
            }
        }
    }

    /// Handle track group actions
    fn handle_track_group_action(&mut self, action: TrackGroupAction) {
        match action {
            TrackGroupAction::CreateGroup(name, color) => {
                self.state.create_track_group(name, color);
            }
            TrackGroupAction::DeleteGroup(idx) => {
                self.state.delete_track_group(idx);
            }
            TrackGroupAction::RenameGroup(idx, name) => {
                if let Some(group) = self.state.track_groups.get_mut(idx) {
                    group.name = name;
                }
            }
            TrackGroupAction::AddTrackToGroup(track_idx, group_idx) => {
                self.state.add_track_to_group(track_idx, group_idx);
            }
            TrackGroupAction::RemoveTrackFromGroup(track_idx) => {
                self.state.remove_track_from_group(track_idx);
            }
            TrackGroupAction::ToggleGroupCollapsed(idx) => {
                self.state.toggle_group_collapsed(idx);
            }
            TrackGroupAction::FocusGroup(idx) => {
                self.state.focus_group(idx);
            }
            TrackGroupAction::SelectTrack(idx) => {
                self.state.select_track(idx);
            }
        }
    }

    /// Handle analysis panel actions
    fn handle_analysis_action(&mut self, action: AnalysisPanelAction, tab_action: &mut TabEditorAction) {
        match action {
            AnalysisPanelAction::RunAnalysis => {
                self.state.run_analysis();
            }
            AnalysisPanelAction::RunAnalysisExcluding(ranges) => {
                self.state.analysis_panel.excluded_ranges = ranges;
                self.state.run_analysis();
            }
            AnalysisPanelAction::JumpToMeasure(measure) => {
                self.state.cursor.measure = measure;
                self.state.cursor.beat = 0;
            }
            AnalysisPanelAction::JumpToRegion(start, _end) => {
                self.state.cursor.measure = start;
                self.state.cursor.beat = 0;
            }
            AnalysisPanelAction::ExcludeRegion(start, end) => {
                self.state.analysis_panel.exclude_region(start, end);
            }
            AnalysisPanelAction::ExcludeAndReanalyze(start, end) => {
                self.state.analysis_panel.exclude_region(start, end);
                self.state.run_analysis();
                self.state.status = format!("Excluded M{}-{} and re-analyzed", start + 1, end + 1);
            }
            AnalysisPanelAction::ClearExclusions => {
                self.state.analysis_panel.clear_exclusions();
            }
            AnalysisPanelAction::UpdateSensitivity(sens) => {
                self.state.analysis_panel.sensitivity = sens;
            }
            AnalysisPanelAction::ToggleHighlights => {
                // Already handled in panel, this is just a notification
            }
            AnalysisPanelAction::CopyFromReference { source_measure, target_start, target_end } => {
                self.state.copy_measure_pattern(source_measure, target_start, target_end);
                self.state.status = format!("Copied pattern from M{} to M{}-{}", source_measure, target_start, target_end);
                *tab_action = TabEditorAction::Modified;
            }
            AnalysisPanelAction::AutoFixRegion { start, end } => {
                self.state.auto_fix_region(start, end);
                self.state.status = format!("Applied auto-fixes to M{}-{}", start, end);
                *tab_action = TabEditorAction::Modified;
            }
            AnalysisPanelAction::MarkAsReference(_, _) => {
                // Use selection range if available, otherwise current cursor position
                let (start, end) = if let Some(ref sel) = self.state.selection {
                    let start_m = sel.start.measure.min(sel.end.measure);
                    let end_m = sel.start.measure.max(sel.end.measure);
                    (start_m, end_m)
                } else {
                    (self.state.cursor.measure, self.state.cursor.measure)
                };
                self.state.analysis_panel.add_reference(start, end);
                if start == end {
                    self.state.status = format!("Marked M{} as reference", start + 1);
                } else {
                    self.state.status = format!("Marked M{}-{} as reference", start + 1, end + 1);
                }
            }
            AnalysisPanelAction::PracticeRegion { start, end, start_tempo_percent } => {
                // Enable practice mode on the specified region
                self.state.practice.enabled = true;
                self.state.practice.loop_enabled = true;
                self.state.practice.loop_start = start;
                self.state.practice.loop_end = end;
                self.state.practice.tempo_ramp_enabled = true;
                self.state.practice.start_tempo_percent = start_tempo_percent;
                self.state.practice.target_tempo_percent = 100.0;
                self.state.practice.tempo_increment = 5.0;
                self.state.practice.reset();
                // Move cursor to start of practice region
                self.state.cursor.measure = start;
                self.state.cursor.beat = 0;
                self.state.status = format!(
                    "Practice mode: M{}-{} at {}%",
                    start + 1, end + 1, start_tempo_percent as u8
                );
            }
            AnalysisPanelAction::ExportToClipboard => {
                let report = self.state.analysis_panel.generate_report();
                self.state.analysis_panel.pending_clipboard = Some(report);
                self.state.status = "Analysis report copied to clipboard".to_string();
            }
            AnalysisPanelAction::AddAnnotation { measure, text } => {
                self.state.analysis_panel.add_annotation(measure, text);
                self.state.status = format!("Added annotation to M{}", measure + 1);
            }
            AnalysisPanelAction::ClearAnnotation(measure) => {
                self.state.analysis_panel.annotations.remove(&measure);
                self.state.status = format!("Removed annotation from M{}", measure + 1);
            }
            AnalysisPanelAction::UndoExclusion => {
                if self.state.analysis_panel.undo_exclusion() {
                    self.state.status = "Undid exclusion change".to_string();
                }
            }
            AnalysisPanelAction::RedoExclusion => {
                if self.state.analysis_panel.redo_exclusion() {
                    self.state.status = "Redid exclusion change".to_string();
                }
            }
        }
    }

    /// Handle keyboard input
    fn handle_keyboard(&mut self, ui: &mut Ui, action: &mut TabEditorAction, current_time_ms: u64) {
        ui.ctx().input(|input| {
            for event in &input.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    let result = self.state.handle_key(*key, *modifiers, current_time_ms);
                    match result {
                        InputResult::Handled => {
                            *action = TabEditorAction::Modified;
                        }
                        InputResult::NoteEntered { string, fret, velocity } => {
                            *action = TabEditorAction::PreviewNote { string, fret, velocity };
                        }
                        InputResult::NoteFocused { string, fret, velocity } => {
                            *action = TabEditorAction::PreviewNote { string, fret, velocity };
                        }
                        InputResult::Open => {
                            *action = TabEditorAction::Open;
                        }
                        InputResult::Save => {
                            *action = TabEditorAction::Save;
                        }
                        InputResult::ExportMidi => {
                            *action = TabEditorAction::ExportMidi;
                        }
                        InputResult::Exit => {
                            *action = TabEditorAction::Close;
                        }
                        InputResult::TogglePlayback => {
                            *action = TabEditorAction::TogglePlayback;
                        }
                        InputResult::PrevSection => {
                            // Find previous section
                            if let Some(prev) = self.state.document.markers.iter()
                                .filter(|m| m.measure < self.state.cursor.measure)
                                .last()
                            {
                                self.state.cursor.measure = prev.measure;
                                self.state.cursor.beat = 0;
                                self.state.status = format!("→ {}", prev.name);
                            }
                        }
                        InputResult::NextSection => {
                            // Find next section
                            if let Some(next) = self.state.document.markers.iter()
                                .find(|m| m.measure > self.state.cursor.measure)
                            {
                                self.state.cursor.measure = next.measure;
                                self.state.cursor.beat = 0;
                                self.state.status = format!("→ {}", next.name);
                            }
                        }
                        InputResult::AddSectionMarker => {
                            self.state.add_quick_section_marker();
                        }
                        InputResult::QuickPractice => {
                            self.state.quick_practice_toggle();
                        }
                        InputResult::SwitchToRecordMode => {
                            // Get current tempo from tempo map
                            let tempo = self.state.document.tempo_map.tempo_at(0, 0.0);
                            // Get time signature from first measure, or use default 4/4
                            let (time_sig_num, time_sig_denom) = self.state.document.measures
                                .first()
                                .and_then(|m| m.time_signature.as_ref())
                                .map(|ts| (ts.numerator, ts.denominator))
                                .unwrap_or((4, 4));
                            let track_name = self.state.document.tracks
                                .first()
                                .map(|t| t.name.clone())
                                .unwrap_or_else(|| "Recording".to_string());
                            *action = TabEditorAction::SwitchToRecordMode {
                                tempo,
                                time_sig_num,
                                time_sig_denom,
                                track_name,
                            };
                        }
                        InputResult::ImportMidi => {
                            *action = TabEditorAction::ImportMidi;
                        }
                        InputResult::None => {}
                    }
                }
            }
        });
    }

    /// Draw the toolbar
    fn draw_toolbar(&mut self, ui: &mut Ui, rect: Rect, action: &mut TabEditorAction) {
        ui.painter().rect_filled(rect, Rounding::ZERO, self.theme.surface_bg());

        let mut toolbar_ui = ui.child_ui(
            rect.shrink(4.0),
            egui::Layout::left_to_right(egui::Align::Center),
            None,
        );

        // Document title (shows filename with * for unsaved changes)
        let title = self.state.title();
        let title_color = if self.state.is_modified {
            self.theme.palette.warning
        } else {
            self.theme.text_primary()
        };
        let title_tooltip = match &self.state.file_path {
            Some(path) => format!("File: {}", path.display()),
            None => "New document (unsaved)".to_string(),
        };
        toolbar_ui.label(
            egui::RichText::new(&title)
                .strong()
                .color(title_color),
        ).on_hover_text(title_tooltip);

        toolbar_ui.separator();

        // Undo/Redo buttons
        let undo_enabled = self.state.can_undo();
        let redo_enabled = self.state.can_redo();
        let undo_count = self.state.undo_count();
        let redo_count = self.state.redo_count();

        let undo_btn = toolbar_ui.add_enabled(undo_enabled, egui::Button::new("↶"));
        if undo_btn.on_hover_text(format!("Undo (Ctrl+Z) - {} actions available", undo_count)).clicked() {
            self.state.undo();
            *action = TabEditorAction::Modified;
        }

        let redo_btn = toolbar_ui.add_enabled(redo_enabled, egui::Button::new("↷"));
        if redo_btn.on_hover_text(format!("Redo (Ctrl+Y) - {} actions available", redo_count)).clicked() {
            self.state.redo();
            *action = TabEditorAction::Modified;
        }

        toolbar_ui.separator();

        // Mode indicator (clickable to switch)
        let mode_text = match self.state.mode {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Visual => "VISUAL",
            EditorMode::Command => "CMD",
        };

        let mode_color = match self.state.mode {
            EditorMode::Normal => self.theme.text_secondary(),
            EditorMode::Insert => self.theme.palette.success,
            EditorMode::Visual => self.theme.palette.accent,
            EditorMode::Command => self.theme.palette.warning,
        };

        let mode_tooltip = match self.state.mode {
            EditorMode::Normal => "NORMAL mode - navigate with arrows, press i to insert. Click to toggle mode.",
            EditorMode::Insert => "INSERT mode - type fret numbers, Esc for normal. Click to toggle mode.",
            EditorMode::Visual => "VISUAL mode - select regions with arrows. Click to toggle mode.",
            EditorMode::Command => "COMMAND mode - type commands. Click to toggle mode.",
        };
        if toolbar_ui.button(egui::RichText::new(mode_text).color(mode_color))
            .on_hover_text(mode_tooltip)
            .clicked()
        {
            // Toggle between Normal and Insert
            self.state.mode = match self.state.mode {
                EditorMode::Insert => EditorMode::Normal,
                _ => EditorMode::Insert,
            };
        }

        toolbar_ui.separator();

        // Duration buttons
        toolbar_ui.label("Duration:");
        let durations = [
            ("W", orpheus_core::tab::BaseDuration::Whole, "Whole note (W key)"),
            ("H", orpheus_core::tab::BaseDuration::Half, "Half note (H key)"),
            ("Q", orpheus_core::tab::BaseDuration::Quarter, "Quarter note (Q key)"),
            ("E", orpheus_core::tab::BaseDuration::Eighth, "Eighth note (E key)"),
            ("S", orpheus_core::tab::BaseDuration::Sixteenth, "Sixteenth note (S key)"),
            ("T", orpheus_core::tab::BaseDuration::ThirtySecond, "32nd note (T key)"),
        ];

        for (label, dur, tooltip) in &durations {
            let is_active = self.state.current_duration.base == *dur;
            let btn = toolbar_ui.selectable_label(is_active, *label)
                .on_hover_text(*tooltip);
            if btn.clicked() {
                self.state.set_duration(*dur);
            }
        }

        // Dotted toggle
        if toolbar_ui.selectable_label(self.state.current_duration.dots > 0, ".")
            .on_hover_text("Dotted note - adds 50% duration (. key)")
            .clicked()
        {
            self.state.toggle_dotted();
        }

        // Triplet toggle
        if toolbar_ui.selectable_label(self.state.current_duration.tuplet.is_some(), "3")
            .on_hover_text("Triplet - 3 notes in the space of 2 (Alt+3)")
            .clicked()
        {
            self.state.toggle_triplet();
        }

        toolbar_ui.separator();

        // Active tool indicator
        let tool_text = tool_indicator(&self.state.active_tool);
        if !tool_text.is_empty() {
            toolbar_ui.label(
                egui::RichText::new(&tool_text)
                    .color(self.theme.palette.accent)
                    .strong(),
            );
            if toolbar_ui.small_button("✕").on_hover_text("Clear active tool").clicked() {
                self.state.active_tool = ActiveTool::None;
            }
            toolbar_ui.separator();
        }

        // Zoom controls
        toolbar_ui.label("Zoom:");
        if toolbar_ui.small_button("-").on_hover_text("Zoom out - decrease beat width (Ctrl+-)").clicked() {
            self.state.zoom_h = (self.state.zoom_h - 5.0).max(20.0);
        }
        toolbar_ui.label(format!("{:.0}px", self.state.zoom_h))
            .on_hover_text("Pixels per beat - controls horizontal zoom level");
        if toolbar_ui.small_button("+").on_hover_text("Zoom in - increase beat width (Ctrl++)").clicked() {
            self.state.zoom_h = (self.state.zoom_h + 5.0).min(100.0);
        }

        // Right side - action buttons
        toolbar_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Close").on_hover_text("Close Editor (Esc)").clicked() {
                *action = TabEditorAction::Close;
            }

            // Save button - shows "Save" if file exists, "Save As" otherwise
            let save_text = if self.state.file_path.is_some() { "Save" } else { "Save As" };
            let save_hover = if self.state.file_path.is_some() {
                "Save File (Ctrl+S)"
            } else {
                "Save As (Ctrl+S)"
            };
            if ui.button(save_text).on_hover_text(save_hover).clicked() {
                *action = TabEditorAction::Save;
            }

            // Export buttons
            if ui.button("WAV").on_hover_text("Export as WAV Audio").clicked() {
                *action = TabEditorAction::ExportAudio;
            }
            if ui.button("PDF").on_hover_text("Export as PDF Tab Sheet").clicked() {
                *action = TabEditorAction::ExportPdf;
            }
            if ui.button("MIDI").on_hover_text("Export MIDI (Ctrl+E)").clicked() {
                *action = TabEditorAction::ExportMidi;
            }

            ui.separator();

            // Record Mode button - one-click transition to recording
            if ui.button(egui::RichText::new("⏺ Record").color(egui::Color32::from_rgb(231, 76, 60)))
                .on_hover_text("Switch to Record Mode (Ctrl+R)\nPreserves tempo and time signature")
                .clicked()
            {
                let tempo = self.state.document.tempo_map.tempo_at(0, 0.0);
                let (time_sig_num, time_sig_denom) = self.state.document.measures
                    .first()
                    .and_then(|m| m.time_signature.as_ref())
                    .map(|ts| (ts.numerator, ts.denominator))
                    .unwrap_or((4, 4));
                let track_name = self.state.document.tracks
                    .first()
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "Recording".to_string());
                *action = TabEditorAction::SwitchToRecordMode {
                    tempo,
                    time_sig_num,
                    time_sig_denom,
                    track_name,
                };
            }

            // Import MIDI button
            if ui.button("Import MIDI").on_hover_text("Import MIDI File (Ctrl+I)").clicked() {
                *action = TabEditorAction::ImportMidi;
            }

            // Open file button
            if ui.button("Open").on_hover_text("Open File (Ctrl+O)").clicked() {
                *action = TabEditorAction::Open;
            }

            // Multi-track toggle
            let multi_text = if self.state.show_all_tracks { "Single" } else { "Multi" };
            if ui.button(multi_text).on_hover_text("Toggle Multi-track View (Ctrl+T)").clicked() {
                self.state.toggle_multi_track_view();
            }

            // Stage View toggle
            let stage_icon = if self.state.show_stage_view { "📋 Tab" } else { "🎭 Stage" };
            let stage_tip = if self.state.show_stage_view {
                "Return to tab editor"
            } else {
                "Open Stage View to add and arrange instruments"
            };
            if ui.button(stage_icon).on_hover_text(stage_tip).clicked() {
                self.state.show_stage_view = !self.state.show_stage_view;
            }

            // Section Analysis button
            if ui.button("🎼 Sections")
                .on_hover_text("Analyze and create song sections")
                .clicked()
            {
                self.state.section_analysis.open();
            }

            // Track Groups button
            let groups_text = if self.state.show_track_groups { "📁 Groups ✓" } else { "📁 Groups" };
            if ui.button(groups_text)
                .on_hover_text("Organize tracks into collapsible groups")
                .clicked()
            {
                self.state.toggle_track_groups_panel();
            }

            // Analysis button
            let analysis_text = if self.state.show_analysis_panel { "🎵 Analysis ✓" } else { "🎵 Analysis" };
            if ui.button(analysis_text)
                .on_hover_text("Compositional analysis: fingerprints, anomalies, harmony")
                .clicked()
            {
                self.state.toggle_analysis_panel();
            }

            // Section Navigator (when sections exist)
            if !self.state.document.markers.is_empty() {
                ui.separator();
                let navigator = SectionNavigator::new(
                    &self.state.document.markers,
                    self.state.cursor.measure,
                    self.theme,
                );
                if let Some(measure) = navigator.show(ui) {
                    self.state.cursor.measure = measure;
                    self.state.cursor.beat = 0;
                }
            }

            // Track selector (when multiple tracks exist)
            if self.state.track_count() > 1 {
                ui.separator();
                let track_idx = self.state.cursor.track + 1;
                let track_total = self.state.track_count();

                if ui.button("<").on_hover_text("Previous Track (Shift+Ctrl+Tab)").clicked() {
                    self.state.prev_track();
                }
                if let Some(track) = self.state.current_track() {
                    let track_label = format!("{} ({}/{})", track.name, track_idx, track_total);
                    ui.label(&track_label)
                        .on_hover_text("Current track. Use < > buttons or Ctrl+Tab to switch tracks.");
                }
                if ui.button(">").on_hover_text("Next Track (Ctrl+Tab)").clicked() {
                    self.state.next_track();
                }
            }

            // Add measure button
            if ui.button("+M").on_hover_text("Add Measure (Ctrl+M)").clicked() {
                self.state.add_measure();
            }

            // Play button
            let play_text = if self.state.is_playing { "⏹ Stop" } else { "▶ Play" };
            let play_color = if self.state.is_playing {
                self.theme.palette.error
            } else {
                self.theme.palette.success
            };
            if ui.button(egui::RichText::new(play_text).color(play_color))
                .on_hover_text("Play/Stop (Space)")
                .clicked()
            {
                self.state.toggle_playback();
            }

            // Metronome toggle
            let metro_text = if self.state.metronome_enabled { "🔔" } else { "🔕" };
            let metro_color = if self.state.metronome_enabled {
                self.theme.palette.success
            } else {
                self.theme.text_secondary()
            };
            if ui.selectable_label(self.state.metronome_enabled, egui::RichText::new(metro_text).color(metro_color))
                .on_hover_text("Toggle Metronome")
                .clicked()
            {
                self.state.toggle_metronome();
            }

            // Metronome volume (only show when enabled)
            if self.state.metronome_enabled {
                let mut vol = self.state.metronome_volume;
                let vol_drag = egui::DragValue::new(&mut vol)
                    .speed(0.01)
                    .clamp_range(0.0..=1.0)
                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0));
                if ui.add(vol_drag).on_hover_text("Metronome Volume").changed() {
                    self.state.set_metronome_volume(vol);
                }
            }

            ui.separator();

            // Time signature selector
            let ts = self.state.current_time_signature();
            ui.menu_button(format!("{}/{}", ts.numerator, ts.denominator), |ui| {
                ui.set_min_width(80.0);
                // Common time signatures
                if ui.button("4/4 (Common)").clicked() {
                    self.state.set_global_time_signature(4, 4);
                    ui.close_menu();
                }
                if ui.button("3/4 (Waltz)").clicked() {
                    self.state.set_global_time_signature(3, 4);
                    ui.close_menu();
                }
                if ui.button("6/8").clicked() {
                    self.state.set_global_time_signature(6, 8);
                    ui.close_menu();
                }
                if ui.button("2/4").clicked() {
                    self.state.set_global_time_signature(2, 4);
                    ui.close_menu();
                }
                ui.separator();
                // Odd meters (prog/metal)
                if ui.button("5/4").clicked() {
                    self.state.set_global_time_signature(5, 4);
                    ui.close_menu();
                }
                if ui.button("7/8").clicked() {
                    self.state.set_global_time_signature(7, 8);
                    ui.close_menu();
                }
                if ui.button("9/8").clicked() {
                    self.state.set_global_time_signature(9, 8);
                    ui.close_menu();
                }
                if ui.button("11/8").clicked() {
                    self.state.set_global_time_signature(11, 8);
                    ui.close_menu();
                }
                ui.separator();
                // Technical death metal territory
                if ui.button("13/8").clicked() {
                    self.state.set_global_time_signature(13, 8);
                    ui.close_menu();
                }
                if ui.button("15/16").clicked() {
                    self.state.set_global_time_signature(15, 16);
                    ui.close_menu();
                }
            }).response.on_hover_text("Time Signature (affects all measures)");

            // Tempo control
            ui.label("BPM:");
            let mut tempo = self.state.tempo;
            let tempo_drag = egui::DragValue::new(&mut tempo)
                .speed(0.5)
                .clamp_range(20.0..=400.0);
            if ui.add(tempo_drag)
                .on_hover_text("Tempo in beats per minute")
                .changed()
            {
                self.state.set_tempo(tempo);
            }

            ui.separator();

            // Practice mode toggle and controls
            let practice_text = if self.state.practice.enabled { "🎯 Practice" } else { "Practice" };
            let practice_color = if self.state.practice.enabled {
                self.theme.palette.accent
            } else {
                self.theme.text_secondary()
            };

            ui.menu_button(egui::RichText::new(practice_text).color(practice_color), |ui| {
                ui.set_min_width(200.0);

                // Toggle practice mode
                if ui.button(if self.state.practice.enabled { "Disable Practice Mode" } else { "Enable Practice Mode" }).clicked() {
                    self.state.toggle_practice_mode();
                    ui.close_menu();
                }

                ui.separator();

                // Loop controls (always visible)
                ui.label(egui::RichText::new("Loop Region").strong());

                ui.horizontal(|ui| {
                    ui.label("Start:");
                    let mut start = (self.state.practice.loop_start + 1) as i32;
                    if ui.add(egui::DragValue::new(&mut start).clamp_range(1..=self.state.document.measures.len().max(1) as i32))
                        .on_hover_text("First measure of the loop region")
                        .changed()
                    {
                        let end = self.state.practice.loop_end;
                        self.state.set_practice_loop((start - 1) as usize, end);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("End:");
                    let mut end = (self.state.practice.loop_end + 1) as i32;
                    if ui.add(egui::DragValue::new(&mut end).clamp_range(1..=self.state.document.measures.len().max(1) as i32))
                        .on_hover_text("Last measure of the loop region (inclusive)")
                        .changed()
                    {
                        let start = self.state.practice.loop_start;
                        self.state.set_practice_loop(start, (end - 1) as usize);
                    }
                });

                if ui.button("Set from selection")
                    .on_hover_text("Set loop region to current visual selection")
                    .clicked()
                {
                    self.state.set_loop_from_selection();
                }

                ui.separator();

                // Tempo ramp controls
                ui.label(egui::RichText::new("Tempo Ramp").strong())
                    .on_hover_text("Gradually increase tempo with each loop iteration");

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.state.practice.tempo_ramp_enabled, "Enable")
                        .on_hover_text("Enable gradual tempo increase during practice");
                });

                if self.state.practice.tempo_ramp_enabled {
                    ui.horizontal(|ui| {
                        ui.label("Start:");
                        let mut start_pct = self.state.practice.start_tempo_percent;
                        if ui.add(egui::DragValue::new(&mut start_pct).clamp_range(20.0..=100.0).suffix("%"))
                            .on_hover_text("Starting tempo as percentage of original BPM")
                            .changed()
                        {
                            self.state.practice.start_tempo_percent = start_pct;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Target:");
                        let mut target_pct = self.state.practice.target_tempo_percent;
                        if ui.add(egui::DragValue::new(&mut target_pct).clamp_range(50.0..=150.0).suffix("%"))
                            .on_hover_text("Goal tempo as percentage of original BPM")
                            .changed()
                        {
                            self.state.practice.target_tempo_percent = target_pct;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Increment:");
                        let mut inc = self.state.practice.tempo_increment;
                        if ui.add(egui::DragValue::new(&mut inc).clamp_range(1.0..=20.0).suffix("%/loop"))
                            .on_hover_text("Percentage points to increase tempo after each loop")
                            .changed()
                        {
                            self.state.practice.tempo_increment = inc;
                        }
                    });

                    if ui.button("Apply Tempo Ramp")
                        .on_hover_text("Start practice with tempo ramping")
                        .clicked()
                    {
                        self.state.enable_tempo_ramp(
                            self.state.practice.start_tempo_percent,
                            self.state.practice.target_tempo_percent,
                            self.state.practice.tempo_increment,
                        );
                        ui.close_menu();
                    }
                }

                ui.separator();

                // Count-in settings
                ui.label(egui::RichText::new("Count-in").strong())
                    .on_hover_text("Metronome clicks before loop starts");
                ui.horizontal(|ui| {
                    ui.label("Beats:");
                    let mut count_in = self.state.practice.count_in_beats as i32;
                    if ui.add(egui::DragValue::new(&mut count_in).clamp_range(0..=8))
                        .on_hover_text("Number of beats to count before playing starts")
                        .changed()
                    {
                        self.state.practice.count_in_beats = count_in as u8;
                    }
                });

                // Quick count-in presets
                ui.horizontal(|ui| {
                    if ui.small_button("0").clicked() {
                        self.state.practice.count_in_beats = 0;
                    }
                    if ui.small_button("2").clicked() {
                        self.state.practice.count_in_beats = 2;
                    }
                    if ui.small_button("4").clicked() {
                        self.state.practice.count_in_beats = 4;
                    }
                    if ui.small_button("8").clicked() {
                        self.state.practice.count_in_beats = 8;
                    }
                });

                ui.separator();

                // Practice stats
                if self.state.practice.enabled {
                    ui.label(format!("Loop count: {}", self.state.practice.loop_count));
                    ui.label(format!("Current tempo: {:.0}%", self.state.practice.current_tempo_percent));
                }

                // Quick presets
                ui.separator();
                ui.label(egui::RichText::new("Quick Presets").strong());
                if ui.button("Slow Practice (50% → 100%)").clicked() {
                    self.state.enable_tempo_ramp(50.0, 100.0, 5.0);
                    ui.close_menu();
                }
                if ui.button("Speed Building (70% → 110%)").clicked() {
                    self.state.enable_tempo_ramp(70.0, 110.0, 5.0);
                    ui.close_menu();
                }
                if ui.button("Gradual Mastery (60% → 100%, +2%)").clicked() {
                    self.state.enable_tempo_ramp(60.0, 100.0, 2.0);
                    ui.close_menu();
                }
            }).response.on_hover_text("Practice mode with loop and tempo ramping");
        });

        // Bottom border
        ui.painter().line_segment(
            [Pos2::new(rect.min.x, rect.max.y - 1.0), Pos2::new(rect.max.x, rect.max.y - 1.0)],
            Stroke::new(1.0, self.theme.border()),
        );
    }

    /// Draw the main editing area
    fn draw_main_area(&mut self, ui: &mut Ui, rect: Rect, _action: &mut TabEditorAction) {
        // If no tracks, show empty state
        if self.state.document.tracks.is_empty() {
            self.draw_empty_state(ui, rect);
            return;
        }

        // Calculate dimensions
        let string_height = self.state.zoom_v;
        let beat_width = self.state.zoom_h;

        // Determine tracks to display
        // Use visible_tracks() which respects group visibility and focus
        let tracks_to_display: Vec<usize> = if self.state.show_all_tracks {
            self.state.visible_tracks()
        } else {
            vec![self.state.cursor.track]
        };

        // Track label area (fixed on left)
        let track_label_rect = Rect::from_min_size(
            rect.min,
            Vec2::new(TRACK_LABEL_WIDTH, rect.height()),
        );

        // Scrollable grid area
        let _grid_rect = Rect::from_min_max(
            Pos2::new(track_label_rect.max.x, rect.min.y),
            rect.max,
        );

        // Draw track labels for all visible tracks
        let mut label_y = track_label_rect.min.y;
        for &track_idx in &tracks_to_display {
            if let Some(track) = self.state.get_track(track_idx) {
                let track_string_count = match &track.instrument {
                    Instrument::StringedInstrument(s) => s.string_count,
                    Instrument::Drums(_) => 0,
                    Instrument::Keys(_) => 0,
                };
                let track_height = MEASURE_HEADER_HEIGHT + (track_string_count as f32 * string_height);

                let instrument_info = match &track.instrument {
                    Instrument::StringedInstrument(s) => format!("{}-string {:.0}\"", s.string_count, s.scale_length),
                    Instrument::Drums(_) => "Drums".to_string(),
                    Instrument::Keys(_) => "Keys".to_string(),
                };

                let is_active = track_idx == self.state.cursor.track;

                // Highlight active track label
                if is_active {
                    let highlight_rect = Rect::from_min_size(
                        Pos2::new(track_label_rect.min.x, label_y),
                        Vec2::new(TRACK_LABEL_WIDTH, track_height),
                    );
                    ui.painter().rect_filled(
                        highlight_rect,
                        Rounding::ZERO,
                        self.theme.palette.accent.gamma_multiply(0.1),
                    );
                }

                draw_track_label(
                    ui,
                    Rect::from_min_size(Pos2::new(track_label_rect.min.x, label_y), Vec2::new(TRACK_LABEL_WIDTH, track_height)),
                    &track.name,
                    &instrument_info,
                    track_string_count,
                    string_height,
                    self.theme,
                );

                label_y += track_height + 4.0; // Gap between tracks
            }
        }

        // For backward compatibility, still set these for single-track mode
        let string_count = self.state.string_count();
        let track_height = MEASURE_HEADER_HEIGHT + (string_count as f32 * string_height);

        // Scrollable content
        let scroll_id = ui.make_persistent_id("tab_editor_scroll");
        ScrollArea::horizontal()
            .id_salt(scroll_id)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show_viewport(ui, |ui, viewport| {
                // Calculate content size based on measures
                let measure_count = self.state.document.measures.len().max(4);
                let beats_per_measure = self.state.beats_per_measure();
                let measure_width = beats_per_measure as f32 * beat_width;
                let total_width = measure_count as f32 * measure_width;

                // Allocate space for content
                let content_size = Vec2::new(total_width, track_height);
                let (response, painter) = ui.allocate_painter(content_size, Sense::click_and_drag());
                let content_rect = response.rect;

                // Background
                painter.rect_filled(content_rect, Rounding::ZERO, self.theme.panel_bg());

                // Draw loop region markers (behind everything)
                if self.state.practice.enabled && self.state.practice.loop_enabled {
                    draw_loop_region(
                        ui,
                        content_rect,
                        self.state.practice.loop_start,
                        self.state.practice.loop_end,
                        beats_per_measure,
                        beat_width,
                        self.theme,
                        true,
                    );
                }

                // Draw measures
                self.draw_measures(ui, content_rect, string_height, beat_width, beats_per_measure);

                // Draw playhead if playing
                if self.state.is_playing {
                    draw_playhead(ui, content_rect, self.state.playhead, beat_width, self.theme);
                }

                // Handle click to move cursor
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.handle_click(pos, content_rect, string_height, beat_width, beats_per_measure);
                        // Preview note at clicked position if there is one
                        if let Some((string, fret, velocity)) = self.state.get_note_at_cursor() {
                            *_action = TabEditorAction::PreviewNote { string, fret, velocity };
                        }
                    }
                }
            });
    }

    /// Draw measures
    fn draw_measures(
        &self,
        ui: &mut Ui,
        rect: Rect,
        string_height: f32,
        beat_width: f32,
        beats_per_measure: usize,
    ) {
        let string_count = self.state.string_count();
        let measure_width = beats_per_measure as f32 * beat_width;
        let track_id = self.state.current_track().map(|t| t.id);

        for (measure_idx, measure) in self.state.document.measures.iter().enumerate() {
            let measure_x = rect.min.x + measure_idx as f32 * measure_width;

            // Full measure rect (header + grid)
            let full_measure_rect = Rect::from_min_size(
                Pos2::new(measure_x, rect.min.y),
                Vec2::new(measure_width, MEASURE_HEADER_HEIGHT + string_count as f32 * string_height),
            );

            // Draw anomaly highlight background (before other content)
            if self.state.analysis_panel.show_highlights {
                if let Some(ref report) = self.state.analysis_panel.anomaly_report {
                    if let Some(region) = report.regions.iter().find(|r| measure_idx >= r.start && measure_idx <= r.end) {
                        let severity = orpheus_core::analysis::Severity::from_score(region.score);
                        let color = crate::panels::AnalysisPanelState::severity_color(severity);
                        // Draw semi-transparent highlight
                        let highlight_color = Color32::from_rgba_unmultiplied(
                            color.r(), color.g(), color.b(), 30
                        );
                        ui.painter().rect_filled(full_measure_rect, Rounding::ZERO, highlight_color);
                    }
                }
            }

            // Measure header
            let header_rect = Rect::from_min_size(
                Pos2::new(measure_x, rect.min.y),
                Vec2::new(measure_width, MEASURE_HEADER_HEIGHT),
            );

            let time_sig = measure.time_signature.as_ref()
                .map(|ts| (ts.numerator, ts.denominator));

            // Check if this measure has a marker in the document
            let has_marker = self.state.document.markers.iter()
                .any(|m| m.measure == measure_idx);

            draw_measure_header(
                ui,
                header_rect,
                measure_idx,
                time_sig,
                has_marker,
                self.theme,
            );

            // Grid area for this measure
            let grid_rect = Rect::from_min_max(
                Pos2::new(measure_x, rect.min.y + MEASURE_HEADER_HEIGHT),
                Pos2::new(measure_x + measure_width, rect.min.y + MEASURE_HEADER_HEIGHT + string_count as f32 * string_height),
            );

            // Draw string lines
            draw_string_lines(ui, grid_rect, string_count, string_height, self.theme);

            // Draw measure lines
            draw_measure_lines(ui, grid_rect, beats_per_measure, beat_width, self.theme);

            // Draw notes for this track
            if let Some(tid) = track_id {
                if let Some(track_beats) = measure.track_beats.iter().find(|tb| tb.track_id == tid) {
                    self.draw_beats(ui, grid_rect, track_beats, measure_idx, string_height, beat_width);
                }
            }
        }
    }

    /// Draw beats for a track in a measure
    fn draw_beats(
        &self,
        ui: &mut Ui,
        rect: Rect,
        track_beats: &TrackMeasure,
        measure_idx: usize,
        string_height: f32,
        beat_width: f32,
    ) {
        let string_count = self.state.string_count();

        for (beat_idx, beat) in track_beats.beats.iter().enumerate() {
            let beat_x = rect.min.x + beat_idx as f32 * beat_width;

            // Draw notes on each string
            for string in 1..=string_count {
                let y = rect.min.y + (string - 1) as f32 * string_height;
                let cell_rect = Rect::from_min_size(
                    Pos2::new(beat_x + 2.0, y + 2.0),
                    Vec2::new(beat_width - 4.0, string_height - 4.0),
                );

                // Find note on this string
                let note = beat.notes.iter().find(|n| n.string == string);

                // Check if cursor is here
                let is_cursor = self.state.cursor.measure == measure_idx
                    && self.state.cursor.beat == beat_idx
                    && self.state.cursor.string == string;

                // Check if selected
                let is_selected = self.state.selection.as_ref()
                    .map(|sel| sel.contains(measure_idx, beat_idx, string))
                    .unwrap_or(false);

                draw_note_cell(
                    ui,
                    cell_rect,
                    note.map(|n| n.fret),
                    note.map(|n| n.techniques.as_slice()).unwrap_or(&[]),
                    is_cursor,
                    is_selected,
                    beat.is_rest,
                    self.theme,
                );
            }
        }
    }

    /// Draw empty state when no tracks exist
    fn draw_empty_state(&mut self, ui: &mut Ui, rect: Rect) {
        // Button rects
        let guitar_btn_rect = Rect::from_center_size(
            Pos2::new(rect.center().x - 70.0, rect.center().y + 60.0),
            Vec2::new(120.0, 32.0),
        );
        let stage_btn_rect = Rect::from_center_size(
            Pos2::new(rect.center().x + 70.0, rect.center().y + 60.0),
            Vec2::new(120.0, 32.0),
        );

        // Allocate buttons first (needs mutable borrow)
        let guitar_response = ui.allocate_rect(guitar_btn_rect, Sense::click());
        let stage_response = ui.allocate_rect(stage_btn_rect, Sense::click());

        // Compute hover states before getting painter
        let guitar_bg = if guitar_response.hovered() {
            self.theme.palette.accent
        } else {
            self.theme.surface_bg()
        };
        let stage_bg = if stage_response.hovered() {
            self.theme.palette.success
        } else {
            self.theme.surface_bg()
        };

        // Now get painter and draw everything (immutable borrow)
        let painter = ui.painter();

        // Empty state icon
        painter.text(
            Pos2::new(rect.center().x, rect.center().y - 60.0),
            egui::Align2::CENTER_CENTER,
            "🎸",
            egui::FontId::proportional(48.0),
            self.theme.text_secondary(),
        );

        // Title
        painter.text(
            Pos2::new(rect.center().x, rect.center().y - 10.0),
            egui::Align2::CENTER_CENTER,
            "No Instruments Yet",
            egui::FontId::proportional(18.0),
            self.theme.text_primary(),
        );

        // Subtitle
        painter.text(
            Pos2::new(rect.center().x, rect.center().y + 15.0),
            egui::Align2::CENTER_CENTER,
            "Add your first track to start composing",
            egui::FontId::proportional(13.0),
            self.theme.text_secondary(),
        );

        // Draw guitar button
        painter.rect_filled(guitar_btn_rect, Rounding::same(4.0), guitar_bg);
        painter.rect_stroke(guitar_btn_rect, Rounding::same(4.0), Stroke::new(1.0, self.theme.border()));
        painter.text(
            guitar_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "🎸 Add Guitar",
            egui::FontId::proportional(12.0),
            self.theme.text_primary(),
        );

        // Draw stage view button
        painter.rect_filled(stage_btn_rect, Rounding::same(4.0), stage_bg);
        painter.rect_stroke(stage_btn_rect, Rounding::same(4.0), Stroke::new(1.0, self.theme.border()));
        painter.text(
            stage_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "🎭 Stage View",
            egui::FontId::proportional(12.0),
            self.theme.text_primary(),
        );

        // Help text
        painter.text(
            Pos2::new(rect.center().x, rect.center().y + 100.0),
            egui::Align2::CENTER_CENTER,
            "Stage View lets you add and arrange multiple instruments at once",
            egui::FontId::proportional(11.0),
            self.theme.text_muted(),
        );

        // Handle clicks after painting
        if guitar_response.clicked() {
            let track = orpheus_core::tab::TabTrack::guitar("Guitar 1");
            self.state.document.add_track(track);
            self.state.document.add_measure();
            self.state.ensure_beats();
        }

        if stage_response.clicked() {
            self.state.show_stage_view = true;
        }
    }

    /// Handle click to move cursor
    fn handle_click(
        &mut self,
        pos: Pos2,
        rect: Rect,
        string_height: f32,
        beat_width: f32,
        beats_per_measure: usize,
    ) {
        let measure_width = beats_per_measure as f32 * beat_width;

        // Calculate measure
        let rel_x = pos.x - rect.min.x;
        let measure = (rel_x / measure_width) as usize;

        if measure >= self.state.document.measures.len() {
            return;
        }

        // Calculate beat within measure
        let beat_x = rel_x - (measure as f32 * measure_width);
        let beat = (beat_x / beat_width) as usize;

        // Calculate string (accounting for header)
        let rel_y = pos.y - rect.min.y - MEASURE_HEADER_HEIGHT;
        if rel_y < 0.0 {
            return; // Clicked on header
        }

        let string = ((rel_y / string_height) as u8).saturating_add(1);
        let string_count = self.state.string_count();

        if string > string_count {
            return;
        }

        // Update cursor
        self.state.cursor.measure = measure;
        self.state.cursor.beat = beat;
        self.state.cursor.string = string;

        // If in visual mode, extend selection
        if self.state.mode == EditorMode::Visual {
            if let Some(ref mut sel) = self.state.selection {
                sel.end = self.state.cursor;
            }
        }
    }

    /// Handle actions from the stage view
    fn handle_stage_action(&mut self, stage_action: StageViewAction, action: &mut TabEditorAction) {
        match stage_action {
            StageViewAction::AddTrack { template_idx, position } => {
                let templates = instrument_templates();
                if let Some(template) = templates.get(template_idx) {
                    let instrument = (template.create)();
                    let track_num = self.state.document.tracks.len() + 1;
                    let name = format!("{} {}", template.name, track_num);

                    let mut track = TabTrack::new(&name, instrument);
                    track.pan = position.to_pan();
                    track.color = template.default_color;

                    self.state.document.add_track(track);
                    self.state.ensure_beats();
                    self.state.is_modified = true;
                    *action = TabEditorAction::Modified;

                    self.state.status = format!("Added {} at pan {:.0}%", template.name, position.to_pan() * 100.0);
                }
            }
            StageViewAction::MoveTrack { track_id, position } => {
                if let Some(track) = self.state.document.tracks.iter_mut().find(|t| t.id == track_id) {
                    track.pan = position.to_pan();
                    self.state.is_modified = true;
                    *action = TabEditorAction::Modified;
                }
            }
            StageViewAction::RemoveTrack(track_id) => {
                self.state.document.tracks.retain(|t| t.id != track_id);
                self.state.is_modified = true;
                *action = TabEditorAction::Modified;
                self.state.status = "Track removed".to_string();
            }
            StageViewAction::SelectTrack(track_id) => {
                // Find track index and set cursor
                if let Some(idx) = self.state.document.tracks.iter().position(|t| t.id == track_id) {
                    self.state.cursor.track = idx;
                }
            }
            StageViewAction::UpdateTrack { track_id, name, pan, volume, color } => {
                if let Some(track) = self.state.document.tracks.iter_mut().find(|t| t.id == track_id) {
                    if let Some(n) = name {
                        track.name = n;
                    }
                    if let Some(p) = pan {
                        track.pan = p;
                    }
                    if let Some(v) = volume {
                        track.volume = v;
                    }
                    if let Some(c) = color {
                        track.color = c;
                    }
                    self.state.is_modified = true;
                    *action = TabEditorAction::Modified;
                }
            }
            StageViewAction::Close => {
                self.state.show_stage_view = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_editor_view_creation() {
        let state = TabEditorState::with_guitar_track();
        assert!(!state.document.tracks.is_empty());
    }
}
