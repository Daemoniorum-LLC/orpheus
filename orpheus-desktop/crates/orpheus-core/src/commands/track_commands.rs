//! Track-related commands for undo/redo

use std::any::Any;
use uuid::Uuid;

use super::{Command, CommandContext};
use crate::{Result, Track, TrackType};

/// Command to add a new track
pub struct AddTrackCommand {
    /// Name for the new track
    name: String,
    /// Type of track to add
    track_type: TrackType,
    /// ID of the created track (set after execution)
    created_id: Option<Uuid>,
}

impl AddTrackCommand {
    pub fn new(name: impl Into<String>, track_type: TrackType) -> Self {
        Self {
            name: name.into(),
            track_type,
            created_id: None,
        }
    }

    pub fn audio(name: impl Into<String>) -> Self {
        Self::new(name, TrackType::Audio)
    }

    pub fn midi(name: impl Into<String>) -> Self {
        Self::new(name, TrackType::Midi)
    }
}

impl Command for AddTrackCommand {
    fn name(&self) -> &str {
        "Add Track"
    }

    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()> {
        let track = Track::new(&self.name, self.track_type);
        let id = ctx.project.add_track(track);
        ctx.mixer.add_channel(&self.name);
        self.created_id = Some(id);
        Ok(())
    }

    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(id) = self.created_id {
            ctx.project.remove_track(id);
            // Note: We'd need to track mixer channel index for proper undo
            // For now this is a simplified implementation
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Command to remove a track
pub struct RemoveTrackCommand {
    /// ID of the track to remove
    track_id: Uuid,
    /// Stored track data for undo
    removed_track: Option<Track>,
    /// Track's original position in order
    original_index: Option<usize>,
}

impl RemoveTrackCommand {
    pub fn new(track_id: Uuid) -> Self {
        Self {
            track_id,
            removed_track: None,
            original_index: None,
        }
    }
}

impl Command for RemoveTrackCommand {
    fn name(&self) -> &str {
        "Remove Track"
    }

    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()> {
        // Find the track's position before removal
        self.original_index = ctx.project.track_order
            .iter()
            .position(|&id| id == self.track_id);

        // Remove and store the track
        if let Some(track) = ctx.project.remove_track(self.track_id) {
            self.removed_track = Some(track);
        }
        Ok(())
    }

    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(track) = self.removed_track.take() {
            // Re-add the track
            ctx.project.tracks.insert(self.track_id, track.clone());

            // Restore to original position if possible
            if let Some(idx) = self.original_index {
                if idx <= ctx.project.track_order.len() {
                    ctx.project.track_order.insert(idx, self.track_id);
                } else {
                    ctx.project.track_order.push(self.track_id);
                }
            } else {
                ctx.project.track_order.push(self.track_id);
            }

            // Store it back for potential redo
            self.removed_track = Some(track);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Command to rename a track
pub struct RenameTrackCommand {
    /// ID of the track to rename
    track_id: Uuid,
    /// New name for the track
    new_name: String,
    /// Previous name (stored for undo)
    old_name: Option<String>,
}

impl RenameTrackCommand {
    pub fn new(track_id: Uuid, new_name: impl Into<String>) -> Self {
        Self {
            track_id,
            new_name: new_name.into(),
            old_name: None,
        }
    }
}

impl Command for RenameTrackCommand {
    fn name(&self) -> &str {
        "Rename Track"
    }

    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(track) = ctx.project.get_track_mut(self.track_id) {
            self.old_name = Some(track.name.clone());
            track.name = self.new_name.clone();
        }
        Ok(())
    }

    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(old_name) = &self.old_name {
            if let Some(track) = ctx.project.get_track_mut(self.track_id) {
                track.name = old_name.clone();
            }
        }
        Ok(())
    }

    fn can_merge(&self, other: &dyn Command) -> bool {
        // Merge consecutive renames of the same track
        if let Some(other) = other.as_any().downcast_ref::<RenameTrackCommand>() {
            return self.track_id == other.track_id;
        }
        false
    }

    fn merge(&mut self, other: Box<dyn Command>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<RenameTrackCommand>() {
            if self.track_id == other.track_id {
                // Keep our old_name, use their new_name
                self.new_name = other.new_name.clone();
                return true;
            }
        }
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Command to change track order
pub struct MoveTrackCommand {
    /// ID of the track to move
    track_id: Uuid,
    /// New position index
    new_index: usize,
    /// Original position (stored for undo)
    old_index: Option<usize>,
}

impl MoveTrackCommand {
    pub fn new(track_id: Uuid, new_index: usize) -> Self {
        Self {
            track_id,
            new_index,
            old_index: None,
        }
    }
}

impl Command for MoveTrackCommand {
    fn name(&self) -> &str {
        "Move Track"
    }

    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(old_idx) = ctx.project.track_order
            .iter()
            .position(|&id| id == self.track_id)
        {
            self.old_index = Some(old_idx);
            ctx.project.track_order.remove(old_idx);

            let insert_idx = self.new_index.min(ctx.project.track_order.len());
            ctx.project.track_order.insert(insert_idx, self.track_id);

            // Update track order values
            for (i, &id) in ctx.project.track_order.iter().enumerate() {
                if let Some(track) = ctx.project.tracks.get_mut(&id) {
                    track.order = i;
                }
            }
        }
        Ok(())
    }

    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(old_idx) = self.old_index {
            if let Some(current_idx) = ctx.project.track_order
                .iter()
                .position(|&id| id == self.track_id)
            {
                ctx.project.track_order.remove(current_idx);
                let insert_idx = old_idx.min(ctx.project.track_order.len());
                ctx.project.track_order.insert(insert_idx, self.track_id);

                // Update track order values
                for (i, &id) in ctx.project.track_order.iter().enumerate() {
                    if let Some(track) = ctx.project.tracks.get_mut(&id) {
                        track.order = i;
                    }
                }
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppState, MixerState, Project, TransportState};

    fn create_test_context() -> (Project, AppState, TransportState, MixerState) {
        (
            Project::new("Test"),
            AppState::new(),
            TransportState::new(),
            MixerState::new(),
        )
    }

    #[test]
    fn test_add_track_command() {
        let (mut project, mut state, mut transport, mut mixer) = create_test_context();
        let mut ctx = CommandContext {
            project: &mut project,
            state: &mut state,
            transport: &mut transport,
            mixer: &mut mixer,
        };

        let mut cmd = AddTrackCommand::audio("Test Track");
        assert!(cmd.execute(&mut ctx).is_ok());
        assert_eq!(ctx.project.track_count(), 1);
        assert!(cmd.created_id.is_some());

        // Undo
        assert!(cmd.undo(&mut ctx).is_ok());
        assert_eq!(ctx.project.track_count(), 0);
    }

    #[test]
    fn test_rename_track_command() {
        let (mut project, mut state, mut transport, mut mixer) = create_test_context();

        // Add a track first
        let track = Track::audio("Original Name");
        let track_id = project.add_track(track);

        let mut ctx = CommandContext {
            project: &mut project,
            state: &mut state,
            transport: &mut transport,
            mixer: &mut mixer,
        };

        let mut cmd = RenameTrackCommand::new(track_id, "New Name");
        assert!(cmd.execute(&mut ctx).is_ok());
        assert_eq!(ctx.project.get_track(track_id).unwrap().name, "New Name");

        // Undo
        assert!(cmd.undo(&mut ctx).is_ok());
        assert_eq!(ctx.project.get_track(track_id).unwrap().name, "Original Name");
    }
}
