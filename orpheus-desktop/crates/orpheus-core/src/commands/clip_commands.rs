//! Clip-related commands for undo/redo

use std::any::Any;
use uuid::Uuid;

use super::{Command, CommandContext};
use crate::{Clip, ClipContent, Result};

/// Command to add a new clip to a track
pub struct AddClipCommand {
    /// Track ID to add clip to
    track_id: Uuid,
    /// The clip to add
    clip: Clip,
    /// Whether the clip was successfully added
    added: bool,
}

impl AddClipCommand {
    pub fn new(track_id: Uuid, clip: Clip) -> Self {
        Self {
            track_id,
            clip,
            added: false,
        }
    }

    /// Create an audio clip command
    pub fn audio(
        track_id: Uuid,
        name: impl Into<String>,
        start: u64,
        length: u64,
        file_path: impl Into<String>,
    ) -> Self {
        let clip = Clip {
            id: Uuid::new_v4(),
            name: name.into(),
            start,
            length,
            content: ClipContent::Audio {
                file_path: file_path.into(),
                file_offset: 0,
            },
        };
        Self::new(track_id, clip)
    }

    /// Create an empty MIDI clip command
    pub fn midi(track_id: Uuid, name: impl Into<String>, start: u64, length: u64) -> Self {
        let clip = Clip {
            id: Uuid::new_v4(),
            name: name.into(),
            start,
            length,
            content: ClipContent::Midi { notes: Vec::new() },
        };
        Self::new(track_id, clip)
    }
}

impl Command for AddClipCommand {
    fn name(&self) -> &str {
        "Add Clip"
    }

    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(track) = ctx.project.get_track_mut(self.track_id) {
            track.clips.push(self.clip.clone());
            self.added = true;
        }
        Ok(())
    }

    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if self.added {
            if let Some(track) = ctx.project.get_track_mut(self.track_id) {
                track.clips.retain(|c| c.id != self.clip.id);
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

/// Command to delete a clip from a track
pub struct DeleteClipCommand {
    /// Track ID containing the clip
    track_id: Uuid,
    /// Clip ID to delete
    clip_id: Uuid,
    /// Stored clip for undo
    deleted_clip: Option<Clip>,
    /// Original index in clips array
    original_index: Option<usize>,
}

impl DeleteClipCommand {
    pub fn new(track_id: Uuid, clip_id: Uuid) -> Self {
        Self {
            track_id,
            clip_id,
            deleted_clip: None,
            original_index: None,
        }
    }
}

impl Command for DeleteClipCommand {
    fn name(&self) -> &str {
        "Delete Clip"
    }

    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(track) = ctx.project.get_track_mut(self.track_id) {
            if let Some(idx) = track.clips.iter().position(|c| c.id == self.clip_id) {
                self.original_index = Some(idx);
                self.deleted_clip = Some(track.clips.remove(idx));
            }
        }
        Ok(())
    }

    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(clip) = self.deleted_clip.take() {
            if let Some(track) = ctx.project.get_track_mut(self.track_id) {
                if let Some(idx) = self.original_index {
                    if idx <= track.clips.len() {
                        track.clips.insert(idx, clip.clone());
                    } else {
                        track.clips.push(clip.clone());
                    }
                } else {
                    track.clips.push(clip.clone());
                }
                // Store for potential redo
                self.deleted_clip = Some(clip);
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

/// Command to move a clip to a new position
pub struct MoveClipCommand {
    /// Track ID containing the clip
    track_id: Uuid,
    /// Clip ID to move
    clip_id: Uuid,
    /// New start position in samples
    new_start: u64,
    /// Old start position (stored for undo)
    old_start: Option<u64>,
}

impl MoveClipCommand {
    pub fn new(track_id: Uuid, clip_id: Uuid, new_start: u64) -> Self {
        Self {
            track_id,
            clip_id,
            new_start,
            old_start: None,
        }
    }
}

impl Command for MoveClipCommand {
    fn name(&self) -> &str {
        "Move Clip"
    }

    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(track) = ctx.project.get_track_mut(self.track_id) {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == self.clip_id) {
                self.old_start = Some(clip.start);
                clip.start = self.new_start;
            }
        }
        Ok(())
    }

    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(old_start) = self.old_start {
            if let Some(track) = ctx.project.get_track_mut(self.track_id) {
                if let Some(clip) = track.clips.iter_mut().find(|c| c.id == self.clip_id) {
                    clip.start = old_start;
                }
            }
        }
        Ok(())
    }

    fn can_merge(&self, other: &dyn Command) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<MoveClipCommand>() {
            return self.track_id == other.track_id && self.clip_id == other.clip_id;
        }
        false
    }

    fn merge(&mut self, other: Box<dyn Command>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<MoveClipCommand>() {
            if self.track_id == other.track_id && self.clip_id == other.clip_id {
                self.new_start = other.new_start;
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

/// Command to trim a clip (change start offset and/or length)
pub struct TrimClipCommand {
    /// Track ID containing the clip
    track_id: Uuid,
    /// Clip ID to trim
    clip_id: Uuid,
    /// New start position (if trimming from left)
    new_start: Option<u64>,
    /// New length
    new_length: Option<u64>,
    /// Old start position
    old_start: Option<u64>,
    /// Old length
    old_length: Option<u64>,
    /// Old file offset (for audio clips)
    old_file_offset: Option<u64>,
    /// New file offset (for audio clips)
    new_file_offset: Option<u64>,
}

impl TrimClipCommand {
    pub fn new(track_id: Uuid, clip_id: Uuid) -> Self {
        Self {
            track_id,
            clip_id,
            new_start: None,
            new_length: None,
            old_start: None,
            old_length: None,
            old_file_offset: None,
            new_file_offset: None,
        }
    }

    /// Set new start position (trim from left)
    pub fn with_start(mut self, start: u64) -> Self {
        self.new_start = Some(start);
        self
    }

    /// Set new length (trim from right)
    pub fn with_length(mut self, length: u64) -> Self {
        self.new_length = Some(length);
        self
    }

    /// Set new file offset (for audio clips trimmed from left)
    pub fn with_file_offset(mut self, offset: u64) -> Self {
        self.new_file_offset = Some(offset);
        self
    }
}

impl Command for TrimClipCommand {
    fn name(&self) -> &str {
        "Trim Clip"
    }

    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(track) = ctx.project.get_track_mut(self.track_id) {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == self.clip_id) {
                // Store old values
                self.old_start = Some(clip.start);
                self.old_length = Some(clip.length);

                // Apply new values
                if let Some(new_start) = self.new_start {
                    clip.start = new_start;
                }
                if let Some(new_length) = self.new_length {
                    clip.length = new_length;
                }

                // Handle audio file offset
                if let ClipContent::Audio { file_offset, .. } = &mut clip.content {
                    self.old_file_offset = Some(*file_offset);
                    if let Some(new_offset) = self.new_file_offset {
                        *file_offset = new_offset;
                    }
                }
            }
        }
        Ok(())
    }

    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(track) = ctx.project.get_track_mut(self.track_id) {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == self.clip_id) {
                if let Some(old_start) = self.old_start {
                    clip.start = old_start;
                }
                if let Some(old_length) = self.old_length {
                    clip.length = old_length;
                }
                if let ClipContent::Audio { file_offset, .. } = &mut clip.content {
                    if let Some(old_offset) = self.old_file_offset {
                        *file_offset = old_offset;
                    }
                }
            }
        }
        Ok(())
    }

    fn can_merge(&self, other: &dyn Command) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<TrimClipCommand>() {
            return self.track_id == other.track_id && self.clip_id == other.clip_id;
        }
        false
    }

    fn merge(&mut self, other: Box<dyn Command>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<TrimClipCommand>() {
            if self.track_id == other.track_id && self.clip_id == other.clip_id {
                if other.new_start.is_some() {
                    self.new_start = other.new_start;
                }
                if other.new_length.is_some() {
                    self.new_length = other.new_length;
                }
                if other.new_file_offset.is_some() {
                    self.new_file_offset = other.new_file_offset;
                }
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

/// Command to rename a clip
pub struct RenameClipCommand {
    /// Track ID containing the clip
    track_id: Uuid,
    /// Clip ID to rename
    clip_id: Uuid,
    /// New name
    new_name: String,
    /// Old name (stored for undo)
    old_name: Option<String>,
}

impl RenameClipCommand {
    pub fn new(track_id: Uuid, clip_id: Uuid, new_name: impl Into<String>) -> Self {
        Self {
            track_id,
            clip_id,
            new_name: new_name.into(),
            old_name: None,
        }
    }
}

impl Command for RenameClipCommand {
    fn name(&self) -> &str {
        "Rename Clip"
    }

    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(track) = ctx.project.get_track_mut(self.track_id) {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == self.clip_id) {
                self.old_name = Some(clip.name.clone());
                clip.name = self.new_name.clone();
            }
        }
        Ok(())
    }

    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(old_name) = &self.old_name {
            if let Some(track) = ctx.project.get_track_mut(self.track_id) {
                if let Some(clip) = track.clips.iter_mut().find(|c| c.id == self.clip_id) {
                    clip.name = old_name.clone();
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

/// Command to duplicate a clip
pub struct DuplicateClipCommand {
    /// Track ID containing the clip
    track_id: Uuid,
    /// Original clip ID
    original_clip_id: Uuid,
    /// Offset for the duplicate (added to original start)
    offset: u64,
    /// Created duplicate clip ID
    duplicate_id: Option<Uuid>,
}

impl DuplicateClipCommand {
    pub fn new(track_id: Uuid, original_clip_id: Uuid, offset: u64) -> Self {
        Self {
            track_id,
            original_clip_id,
            offset,
            duplicate_id: None,
        }
    }
}

impl Command for DuplicateClipCommand {
    fn name(&self) -> &str {
        "Duplicate Clip"
    }

    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(track) = ctx.project.get_track_mut(self.track_id) {
            if let Some(original) = track.clips.iter().find(|c| c.id == self.original_clip_id) {
                let mut duplicate = original.clone();
                duplicate.id = Uuid::new_v4();
                duplicate.name = format!("{} (copy)", original.name);
                duplicate.start = original.start + self.offset;
                self.duplicate_id = Some(duplicate.id);
                track.clips.push(duplicate);
            }
        }
        Ok(())
    }

    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()> {
        if let Some(dup_id) = self.duplicate_id {
            if let Some(track) = ctx.project.get_track_mut(self.track_id) {
                track.clips.retain(|c| c.id != dup_id);
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
    use crate::{AppState, MixerState, Project, Track, TransportState};

    fn create_test_context() -> (Project, AppState, TransportState, MixerState) {
        let mut project = Project::new("Test");
        let track = Track::audio("Test Track");
        project.add_track(track);
        (project, AppState::new(), TransportState::new(), MixerState::new())
    }

    #[test]
    fn test_add_clip_command() {
        let (mut project, mut state, mut transport, mut mixer) = create_test_context();
        let track_id = *project.track_order.first().unwrap();

        let mut ctx = CommandContext {
            project: &mut project,
            state: &mut state,
            transport: &mut transport,
            mixer: &mut mixer,
        };

        let mut cmd = AddClipCommand::audio(track_id, "Test Clip", 0, 48000, "/path/to/file.wav");
        assert!(cmd.execute(&mut ctx).is_ok());
        assert_eq!(ctx.project.get_track(track_id).unwrap().clips.len(), 1);

        // Undo
        assert!(cmd.undo(&mut ctx).is_ok());
        assert_eq!(ctx.project.get_track(track_id).unwrap().clips.len(), 0);
    }

    #[test]
    fn test_move_clip_command() {
        let (mut project, mut state, mut transport, mut mixer) = create_test_context();
        let track_id = *project.track_order.first().unwrap();

        // Add a clip first
        let clip = Clip {
            id: Uuid::new_v4(),
            name: "Test Clip".into(),
            start: 0,
            length: 48000,
            content: ClipContent::Audio {
                file_path: "/test.wav".into(),
                file_offset: 0,
            },
        };
        let clip_id = clip.id;
        project.get_track_mut(track_id).unwrap().clips.push(clip);

        let mut ctx = CommandContext {
            project: &mut project,
            state: &mut state,
            transport: &mut transport,
            mixer: &mut mixer,
        };

        let mut cmd = MoveClipCommand::new(track_id, clip_id, 96000);
        assert!(cmd.execute(&mut ctx).is_ok());

        let clip = &ctx.project.get_track(track_id).unwrap().clips[0];
        assert_eq!(clip.start, 96000);

        // Undo
        assert!(cmd.undo(&mut ctx).is_ok());
        let clip = &ctx.project.get_track(track_id).unwrap().clips[0];
        assert_eq!(clip.start, 0);
    }
}
