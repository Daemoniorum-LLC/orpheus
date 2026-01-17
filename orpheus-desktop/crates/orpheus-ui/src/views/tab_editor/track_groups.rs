//! Track Groups Panel
//!
//! UI for organizing tracks into collapsible groups for complex compositions.

use egui::{Color32, RichText, Sense, Ui, Vec2};

use crate::theme::Theme;
use super::state::TabEditorState;

/// Actions that can be triggered from the track groups panel
#[derive(Debug, Clone)]
pub enum TrackGroupAction {
    /// Create a new group
    CreateGroup(String, (u8, u8, u8)),
    /// Delete a group (tracks remain, just ungrouped)
    DeleteGroup(usize),
    /// Rename a group
    RenameGroup(usize, String),
    /// Add track to group
    AddTrackToGroup(usize, usize),
    /// Remove track from its group
    RemoveTrackFromGroup(usize),
    /// Toggle group collapsed/expanded
    ToggleGroupCollapsed(usize),
    /// Focus on a group (only show its tracks)
    FocusGroup(Option<usize>),
    /// Select a track
    SelectTrack(usize),
}

/// Track Groups Panel state (stored separately from main state)
#[derive(Debug, Clone, Default)]
pub struct TrackGroupsPanelState {
    /// New group name input
    pub new_group_name: String,
    /// Selected color for new group
    pub new_group_color_idx: usize,
    /// Currently dragging track (for drag-drop)
    pub dragging_track: Option<usize>,
    /// Rename mode for group
    pub renaming_group: Option<usize>,
    /// Rename input buffer
    pub rename_buffer: String,
}

/// Predefined group colors
const GROUP_COLORS: [(u8, u8, u8); 8] = [
    (52, 152, 219),   // Blue
    (46, 204, 113),   // Green
    (155, 89, 182),   // Purple
    (241, 196, 15),   // Yellow
    (230, 126, 34),   // Orange
    (231, 76, 60),    // Red
    (26, 188, 156),   // Teal
    (149, 165, 166),  // Gray
];

/// Snapshot of a group for rendering (avoids borrow issues)
struct GroupSnapshot {
    idx: usize,
    name: String,
    color: (u8, u8, u8),
    collapsed: bool,
    track_indices: Vec<usize>,
    is_focused: bool,
}

/// Snapshot of a track for rendering
struct TrackSnapshot {
    idx: usize,
    name: String,
    is_current: bool,
}

/// Track Groups Panel component
pub struct TrackGroupsPanel<'a> {
    state: &'a mut TabEditorState,
    theme: &'a Theme,
}

impl<'a> TrackGroupsPanel<'a> {
    pub fn new(
        state: &'a mut TabEditorState,
        theme: &'a Theme,
    ) -> Self {
        Self { state, theme }
    }

    /// Show the track groups panel and return any actions triggered
    pub fn show(&mut self, ui: &mut Ui) -> Option<TrackGroupAction> {
        let mut action = None;

        // Collect data snapshots to avoid borrow issues
        let focused_group = self.state.focused_group;
        let current_track = self.state.cursor.track;
        let renaming_group = self.state.track_groups_panel.renaming_group;

        let groups: Vec<GroupSnapshot> = self.state.track_groups.iter().enumerate()
            .map(|(idx, g)| GroupSnapshot {
                idx,
                name: g.name.clone(),
                color: g.color,
                collapsed: g.collapsed,
                track_indices: g.track_indices.clone(),
                is_focused: focused_group == Some(idx),
            })
            .collect();

        let all_tracks: Vec<TrackSnapshot> = self.state.document.tracks.iter().enumerate()
            .map(|(idx, t)| TrackSnapshot {
                idx,
                name: t.name.clone(),
                is_current: idx == current_track,
            })
            .collect();

        // Find ungrouped tracks
        let ungrouped: Vec<usize> = (0..all_tracks.len())
            .filter(|&idx| !groups.iter().any(|g| g.track_indices.contains(&idx)))
            .collect();

        // Get group names for dropdown
        let group_names: Vec<(usize, String, (u8, u8, u8))> = groups.iter()
            .map(|g| (g.idx, g.name.clone(), g.color))
            .collect();

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading("Track Groups");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Focus controls
                    if focused_group.is_some() {
                        if ui.small_button("Show All").on_hover_text("Show all tracks").clicked() {
                            action = Some(TrackGroupAction::FocusGroup(None));
                        }
                    }
                });
            });
            ui.separator();

            // Create new group
            ui.group(|ui| {
                ui.label(RichText::new("New Group").strong());
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.state.track_groups_panel.new_group_name)
                        .on_hover_text("Enter group name");

                    // Color picker
                    let current_color = GROUP_COLORS[self.state.track_groups_panel.new_group_color_idx];
                    let color32 = Color32::from_rgb(current_color.0, current_color.1, current_color.2);
                    let (rect, response) = ui.allocate_exact_size(Vec2::new(20.0, 20.0), Sense::click());
                    ui.painter().rect_filled(rect, 4.0, color32);
                    if response.on_hover_text("Click to change color").clicked() {
                        self.state.track_groups_panel.new_group_color_idx =
                            (self.state.track_groups_panel.new_group_color_idx + 1) % GROUP_COLORS.len();
                    }

                    if ui.button("+ Create").clicked() && !self.state.track_groups_panel.new_group_name.is_empty() {
                        action = Some(TrackGroupAction::CreateGroup(
                            self.state.track_groups_panel.new_group_name.clone(),
                            current_color,
                        ));
                        self.state.track_groups_panel.new_group_name.clear();
                    }
                });
            });

            ui.add_space(8.0);

            // Existing groups
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(300.0)
                .show(ui, |ui| {
                    // Show each group
                    for group in &groups {
                        ui.group(|ui| {
                            // Group header
                            let color = Color32::from_rgb(group.color.0, group.color.1, group.color.2);

                            ui.horizontal(|ui| {
                                // Color indicator
                                let (indicator_rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), Sense::hover());
                                ui.painter().rect_filled(indicator_rect, 3.0, color);

                                // Collapse toggle
                                let collapse_text = if group.collapsed { "▶" } else { "▼" };
                                if ui.small_button(collapse_text)
                                    .on_hover_text(if group.collapsed { "Expand" } else { "Collapse" })
                                    .clicked()
                                {
                                    action = Some(TrackGroupAction::ToggleGroupCollapsed(group.idx));
                                }

                                // Group name (or rename input)
                                if renaming_group == Some(group.idx) {
                                    let response = ui.text_edit_singleline(&mut self.state.track_groups_panel.rename_buffer);
                                    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                        if !self.state.track_groups_panel.rename_buffer.is_empty() {
                                            action = Some(TrackGroupAction::RenameGroup(
                                                group.idx,
                                                self.state.track_groups_panel.rename_buffer.clone(),
                                            ));
                                        }
                                        self.state.track_groups_panel.renaming_group = None;
                                    }
                                } else {
                                    let name_text = if group.is_focused {
                                        RichText::new(&group.name).strong().color(self.theme.palette.accent)
                                    } else {
                                        RichText::new(&group.name).strong()
                                    };

                                    let response = ui.label(name_text);
                                    if response.double_clicked() {
                                        self.state.track_groups_panel.renaming_group = Some(group.idx);
                                        self.state.track_groups_panel.rename_buffer = group.name.clone();
                                    }
                                }

                                // Track count
                                ui.label(RichText::new(format!("({})", group.track_indices.len())).small().color(Color32::GRAY));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Delete button
                                    if ui.small_button("✕").on_hover_text("Delete group (tracks remain)").clicked() {
                                        action = Some(TrackGroupAction::DeleteGroup(group.idx));
                                    }

                                    // Focus button
                                    let focus_text = if group.is_focused { "⊙" } else { "○" };
                                    let focus_tip = if group.is_focused { "Currently focused" } else { "Focus on this group" };
                                    if ui.small_button(focus_text).on_hover_text(focus_tip).clicked() {
                                        action = Some(TrackGroupAction::FocusGroup(
                                            if group.is_focused { None } else { Some(group.idx) }
                                        ));
                                    }
                                });
                            });

                            // Show tracks in group if not collapsed
                            if !group.collapsed {
                                for &track_idx in &group.track_indices {
                                    if let Some(track) = all_tracks.get(track_idx) {
                                        Self::draw_track_item(
                                            ui,
                                            track,
                                            Some(group.idx),
                                            &group_names,
                                            &mut action,
                                            self.theme,
                                        );
                                    }
                                }

                                if group.track_indices.is_empty() {
                                    ui.label(RichText::new("(empty - drag tracks here)").italics().small().color(Color32::GRAY));
                                }
                            }
                        });
                        ui.add_space(4.0);
                    }

                    // Ungrouped tracks
                    if !ungrouped.is_empty() {
                        ui.add_space(8.0);
                        ui.label(RichText::new("Ungrouped Tracks").strong());
                        ui.separator();

                        for track_idx in ungrouped {
                            if let Some(track) = all_tracks.get(track_idx) {
                                Self::draw_track_item(
                                    ui,
                                    track,
                                    None,
                                    &group_names,
                                    &mut action,
                                    self.theme,
                                );
                            }
                        }
                    }
                });
        });

        action
    }

    /// Draw a track item (static method to avoid borrow issues)
    fn draw_track_item(
        ui: &mut Ui,
        track: &TrackSnapshot,
        current_group: Option<usize>,
        group_names: &[(usize, String, (u8, u8, u8))],
        action: &mut Option<TrackGroupAction>,
        theme: &Theme,
    ) {
        ui.horizontal(|ui| {
            ui.add_space(16.0); // Indent

            // Track indicator
            let text_color = if track.is_current {
                theme.palette.accent
            } else {
                theme.text_primary()
            };

            let response = ui.selectable_label(track.is_current, RichText::new(&track.name).color(text_color));
            if response.clicked() {
                *action = Some(TrackGroupAction::SelectTrack(track.idx));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Remove from group button
                if current_group.is_some() {
                    if ui.small_button("−").on_hover_text("Remove from group").clicked() {
                        *action = Some(TrackGroupAction::RemoveTrackFromGroup(track.idx));
                    }
                }

                // Add to group dropdown
                if !group_names.is_empty() {
                    egui::ComboBox::from_id_salt(format!("track_group_{}", track.idx))
                        .selected_text("→")
                        .width(30.0)
                        .show_ui(ui, |ui| {
                            for (idx, name, color) in group_names {
                                if current_group != Some(*idx) {
                                    let color = Color32::from_rgb(color.0, color.1, color.2);
                                    if ui.selectable_label(false, RichText::new(name).color(color)).clicked() {
                                        *action = Some(TrackGroupAction::AddTrackToGroup(track.idx, *idx));
                                    }
                                }
                            }
                        })
                        .response
                        .on_hover_text("Move to group");
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_state_default() {
        let state = TrackGroupsPanelState::default();
        assert!(state.new_group_name.is_empty());
        assert_eq!(state.new_group_color_idx, 0);
        assert!(state.dragging_track.is_none());
    }

    #[test]
    fn test_group_colors_count() {
        assert_eq!(GROUP_COLORS.len(), 8);
    }
}
