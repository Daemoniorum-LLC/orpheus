//! Mixer state - channels, volumes, panning

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Solo mode behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SoloMode {
    /// Standard solo - mutes other tracks
    #[default]
    Standard,
    /// Solo-in-place - preserves routing
    InPlace,
    /// Exclusive solo - only one track can be soloed
    Exclusive,
}

/// Channel state for a single mixer channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    /// Unique channel ID
    pub id: Uuid,
    /// Channel name
    pub name: String,
    /// Volume (0.0 to 1.0, can exceed for gain)
    pub volume: f32,
    /// Pan (-1.0 left to 1.0 right)
    pub pan: f32,
    /// Muted
    pub mute: bool,
    /// Soloed
    pub solo: bool,
    /// Record armed
    pub armed: bool,
    /// Channel color for UI
    pub color: ChannelColor,
    /// Current peak level (for metering)
    #[serde(skip)]
    pub peak_level: f32,
    /// Current RMS level (for metering)
    #[serde(skip)]
    pub rms_level: f32,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::from("Track"),
            volume: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
            armed: false,
            color: ChannelColor::default(),
            peak_level: 0.0,
            rms_level: 0.0,
        }
    }
}

impl ChannelState {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn with_color(mut self, color: ChannelColor) -> Self {
        self.color = color;
        self
    }

    /// Convert volume to dB
    pub fn volume_db(&self) -> f32 {
        if self.volume <= 0.0 {
            f32::NEG_INFINITY
        } else {
            20.0 * self.volume.log10()
        }
    }

    /// Set volume from dB
    pub fn set_volume_db(&mut self, db: f32) {
        self.volume = 10.0_f32.powf(db / 20.0);
    }

    /// Convert peak level to dB
    pub fn peak_db(&self) -> f32 {
        if self.peak_level <= 0.0 {
            f32::NEG_INFINITY
        } else {
            20.0 * self.peak_level.log10()
        }
    }
}

/// Channel colors for visual identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ChannelColor {
    #[default]
    Blue,
    Green,
    Yellow,
    Orange,
    Red,
    Purple,
    Cyan,
    Pink,
}

impl ChannelColor {
    /// Get RGB values
    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Blue => (66, 133, 244),
            Self::Green => (52, 168, 83),
            Self::Yellow => (251, 188, 4),
            Self::Orange => (234, 134, 47),
            Self::Red => (234, 67, 53),
            Self::Purple => (142, 68, 173),
            Self::Cyan => (52, 211, 153),
            Self::Pink => (236, 72, 153),
        }
    }

    /// Get all colors
    pub fn all() -> &'static [ChannelColor] {
        &[
            Self::Blue,
            Self::Green,
            Self::Yellow,
            Self::Orange,
            Self::Red,
            Self::Purple,
            Self::Cyan,
            Self::Pink,
        ]
    }
}

/// Complete mixer state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerState {
    /// All channels
    pub channels: Vec<ChannelState>,
    /// Master volume
    pub master_volume: f32,
    /// Master pan
    pub master_pan: f32,
    /// Master mute
    pub master_mute: bool,
    /// Solo mode behavior
    pub solo_mode: SoloMode,
    /// Master peak level
    #[serde(skip)]
    pub master_peak: f32,
}

impl Default for MixerState {
    fn default() -> Self {
        Self {
            channels: Vec::new(),
            master_volume: 0.8,
            master_pan: 0.0,
            master_mute: false,
            solo_mode: SoloMode::Standard,
            master_peak: 0.0,
        }
    }
}

impl MixerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new channel
    pub fn add_channel(&mut self, name: impl Into<String>) -> Uuid {
        let color_index = self.channels.len() % ChannelColor::all().len();
        let channel = ChannelState::new(name)
            .with_color(ChannelColor::all()[color_index]);
        let id = channel.id;
        self.channels.push(channel);
        id
    }

    /// Remove a channel by ID
    pub fn remove_channel(&mut self, id: Uuid) -> Option<ChannelState> {
        if let Some(pos) = self.channels.iter().position(|c| c.id == id) {
            Some(self.channels.remove(pos))
        } else {
            None
        }
    }

    /// Get channel by ID
    pub fn get_channel(&self, id: Uuid) -> Option<&ChannelState> {
        self.channels.iter().find(|c| c.id == id)
    }

    /// Get mutable channel by ID
    pub fn get_channel_mut(&mut self, id: Uuid) -> Option<&mut ChannelState> {
        self.channels.iter_mut().find(|c| c.id == id)
    }

    /// Check if any channel is soloed
    pub fn any_soloed(&self) -> bool {
        self.channels.iter().any(|c| c.solo)
    }

    /// Get effective volume for a channel (considering solo/mute)
    pub fn effective_volume(&self, id: Uuid) -> f32 {
        let channel = match self.get_channel(id) {
            Some(c) => c,
            None => return 0.0,
        };

        if channel.mute {
            return 0.0;
        }

        if self.any_soloed() && !channel.solo {
            return 0.0;
        }

        channel.volume
    }
}
