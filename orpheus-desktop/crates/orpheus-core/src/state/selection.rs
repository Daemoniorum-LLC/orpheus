//! Selection state - tracks, clips, regions

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Selection state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectionState {
    /// Selected track IDs
    pub tracks: Vec<Uuid>,
    /// Selected clip IDs
    pub clips: Vec<Uuid>,
    /// Time range selection (start, end in samples)
    pub time_range: Option<(u64, u64)>,
}

impl SelectionState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all selection
    pub fn clear(&mut self) {
        self.tracks.clear();
        self.clips.clear();
        self.time_range = None;
    }

    /// Select a single track (clears other track selection)
    pub fn select_track(&mut self, id: Uuid) {
        self.tracks.clear();
        self.tracks.push(id);
    }

    /// Add track to selection
    pub fn add_track(&mut self, id: Uuid) {
        if !self.tracks.contains(&id) {
            self.tracks.push(id);
        }
    }

    /// Toggle track selection
    pub fn toggle_track(&mut self, id: Uuid) {
        if let Some(pos) = self.tracks.iter().position(|&i| i == id) {
            self.tracks.remove(pos);
        } else {
            self.tracks.push(id);
        }
    }

    /// Check if track is selected
    pub fn is_track_selected(&self, id: Uuid) -> bool {
        self.tracks.contains(&id)
    }

    /// Select a single clip
    pub fn select_clip(&mut self, id: Uuid) {
        self.clips.clear();
        self.clips.push(id);
    }

    /// Add clip to selection
    pub fn add_clip(&mut self, id: Uuid) {
        if !self.clips.contains(&id) {
            self.clips.push(id);
        }
    }

    /// Set time range selection
    pub fn set_time_range(&mut self, start: u64, end: u64) {
        self.time_range = Some((start.min(end), start.max(end)));
    }

    /// Clear time range selection
    pub fn clear_time_range(&mut self) {
        self.time_range = None;
    }

    /// Get the first selected track (if any)
    pub fn primary_track(&self) -> Option<Uuid> {
        self.tracks.first().copied()
    }
}
