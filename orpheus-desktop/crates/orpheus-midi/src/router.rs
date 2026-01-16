//! MIDI routing to synths
//!
//! Routes MIDI channel messages to appropriate synthesizers.

use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, trace};

use crate::message::{Channel, MidiMessage, Note, Velocity};

/// Synth target types for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SynthTarget {
    /// Guitar synth (Karplus-Strong)
    Guitar,
    /// Piano synth (additive)
    Piano,
    /// Bass synth (subtractive)
    Bass,
    /// Drum machine
    Drums,
    /// Custom synth by ID
    Custom(u32),
}

/// A routed MIDI event ready for synth processing
#[derive(Debug, Clone)]
pub struct RoutedMidiEvent {
    /// Target synth
    pub target: SynthTarget,
    /// The MIDI message
    pub message: MidiMessage,
    /// Optional string number for guitar (1-based)
    pub string: Option<u8>,
    /// Timestamp from input (microseconds)
    pub timestamp: u64,
}

/// Channel to synth mapping
#[derive(Debug, Clone)]
pub struct ChannelMapping {
    /// MIDI channel (0-15)
    pub channel: Channel,
    /// Target synth
    pub target: SynthTarget,
    /// Optional string assignment for guitar
    pub string: Option<u8>,
}

/// MIDI router configuration
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Channel mappings
    pub mappings: Vec<ChannelMapping>,
    /// Default target for unmapped channels
    pub default_target: SynthTarget,
    /// Whether to pass through unmapped channels
    pub pass_unmapped: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            mappings: vec![
                // Guitar on channels 1-6 (MIDI 0-5), one per string
                ChannelMapping { channel: 0, target: SynthTarget::Guitar, string: Some(1) },
                ChannelMapping { channel: 1, target: SynthTarget::Guitar, string: Some(2) },
                ChannelMapping { channel: 2, target: SynthTarget::Guitar, string: Some(3) },
                ChannelMapping { channel: 3, target: SynthTarget::Guitar, string: Some(4) },
                ChannelMapping { channel: 4, target: SynthTarget::Guitar, string: Some(5) },
                ChannelMapping { channel: 5, target: SynthTarget::Guitar, string: Some(6) },
                // Piano on channel 7
                ChannelMapping { channel: 6, target: SynthTarget::Piano, string: None },
                // Bass on channel 8
                ChannelMapping { channel: 7, target: SynthTarget::Bass, string: None },
                // Drums on channel 10 (MIDI convention)
                ChannelMapping { channel: 9, target: SynthTarget::Drums, string: None },
            ],
            default_target: SynthTarget::Guitar,
            pass_unmapped: true,
        }
    }
}

/// MIDI router
pub struct MidiRouter {
    /// Configuration
    config: Arc<RwLock<RouterConfig>>,
    /// Channel to target lookup (for fast routing)
    channel_map: Arc<RwLock<HashMap<Channel, (SynthTarget, Option<u8>)>>>,
    /// Output sender
    output_tx: Sender<RoutedMidiEvent>,
    /// Output receiver
    output_rx: Receiver<RoutedMidiEvent>,
    /// Statistics
    stats: Arc<RwLock<RouterStats>>,
}

/// Router statistics
#[derive(Debug, Default, Clone)]
pub struct RouterStats {
    /// Total messages routed
    pub messages_routed: u64,
    /// Messages dropped (unmapped and pass_unmapped=false)
    pub messages_dropped: u64,
    /// Note on count
    pub note_on_count: u64,
    /// Note off count
    pub note_off_count: u64,
    /// Control change count
    pub cc_count: u64,
}

impl MidiRouter {
    /// Create a new router with default config
    pub fn new() -> Self {
        Self::with_config(RouterConfig::default())
    }

    /// Create a router with custom config
    pub fn with_config(config: RouterConfig) -> Self {
        let (tx, rx) = bounded(4096);
        let router = Self {
            config: Arc::new(RwLock::new(config.clone())),
            channel_map: Arc::new(RwLock::new(HashMap::new())),
            output_tx: tx,
            output_rx: rx,
            stats: Arc::new(RwLock::new(RouterStats::default())),
        };
        router.rebuild_channel_map();
        router
    }

    /// Rebuild the channel lookup map from config
    fn rebuild_channel_map(&self) {
        let config = self.config.read();
        let mut map = self.channel_map.write();
        map.clear();
        for mapping in &config.mappings {
            map.insert(mapping.channel, (mapping.target, mapping.string));
        }
    }

    /// Update configuration
    pub fn set_config(&self, config: RouterConfig) {
        *self.config.write() = config;
        self.rebuild_channel_map();
    }

    /// Get current configuration
    pub fn config(&self) -> RouterConfig {
        self.config.read().clone()
    }

    /// Route a MIDI message
    pub fn route(&self, message: MidiMessage, timestamp: u64) {
        let channel = match message.channel() {
            Some(ch) => ch,
            None => {
                // System messages - broadcast to all or ignore
                trace!("Ignoring system message: {:?}", message);
                return;
            }
        };

        let channel_map = self.channel_map.read();
        let config = self.config.read();

        let (target, string) = if let Some(&(target, string)) = channel_map.get(&channel) {
            (target, string)
        } else if config.pass_unmapped {
            (config.default_target, None)
        } else {
            self.stats.write().messages_dropped += 1;
            return;
        };

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.messages_routed += 1;
            match &message {
                MidiMessage::NoteOn { .. } => stats.note_on_count += 1,
                MidiMessage::NoteOff { .. } => stats.note_off_count += 1,
                MidiMessage::ControlChange { .. } => stats.cc_count += 1,
                _ => {}
            }
        }

        let event = RoutedMidiEvent {
            target,
            message,
            string,
            timestamp,
        };

        if self.output_tx.send(event).is_err() {
            debug!("Router output queue full");
        }
    }

    /// Route multiple messages
    pub fn route_batch(&self, messages: impl IntoIterator<Item = (MidiMessage, u64)>) {
        for (msg, ts) in messages {
            self.route(msg, ts);
        }
    }

    /// Try to receive a routed event
    pub fn try_recv(&self) -> Option<RoutedMidiEvent> {
        self.output_rx.try_recv().ok()
    }

    /// Receive a routed event (blocking)
    pub fn recv(&self) -> Option<RoutedMidiEvent> {
        self.output_rx.recv().ok()
    }

    /// Get the output receiver
    pub fn receiver(&self) -> &Receiver<RoutedMidiEvent> {
        &self.output_rx
    }

    /// Drain all pending events
    pub fn drain(&self) -> Vec<RoutedMidiEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.try_recv() {
            events.push(event);
        }
        events
    }

    /// Get statistics
    pub fn stats(&self) -> RouterStats {
        self.stats.read().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.write() = RouterStats::default();
    }

    /// Add a channel mapping
    pub fn add_mapping(&self, channel: Channel, target: SynthTarget, string: Option<u8>) {
        {
            let mut config = self.config.write();
            config.mappings.push(ChannelMapping { channel, target, string });
        }
        self.rebuild_channel_map();
    }

    /// Remove a channel mapping
    pub fn remove_mapping(&self, channel: Channel) {
        {
            let mut config = self.config.write();
            config.mappings.retain(|m| m.channel != channel);
        }
        self.rebuild_channel_map();
    }

    /// Set the default target
    pub fn set_default_target(&self, target: SynthTarget) {
        self.config.write().default_target = target;
    }
}

impl Default for MidiRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Guitar-specific MIDI routing helpers
pub mod guitar {
    use super::*;

    /// Map a MIDI note to guitar string and fret
    ///
    /// Uses standard tuning: E2(40), A2(45), D3(50), G3(55), B3(59), E4(64)
    pub fn note_to_string_fret(note: Note) -> Option<(u8, u8)> {
        const OPEN_STRINGS: [Note; 6] = [40, 45, 50, 55, 59, 64]; // E2, A2, D3, G3, B3, E4
        const MAX_FRET: u8 = 24;

        // Find the lowest string that can play this note
        for (string_idx, &open_note) in OPEN_STRINGS.iter().enumerate().rev() {
            if note >= open_note && note <= open_note + MAX_FRET {
                let fret = note - open_note;
                return Some(((string_idx + 1) as u8, fret));
            }
        }
        None
    }

    /// Map MIDI velocity to guitar pluck intensity
    pub fn velocity_to_intensity(velocity: Velocity) -> f32 {
        (velocity as f32 / 127.0).powf(0.8) // Slightly compress dynamics
    }

    /// Check if a MIDI note is within guitar range (E2 to D6)
    pub fn is_guitar_range(note: Note) -> bool {
        note >= 40 && note <= 86 // E2 to D6 (24 frets on high E)
    }
}

/// Drum-specific MIDI routing helpers
pub mod drums {
    use super::*;

    /// Standard GM drum note mappings
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DrumNote {
        Kick,
        Snare,
        SideStick,
        ClosedHiHat,
        OpenHiHat,
        PedalHiHat,
        CrashCymbal,
        RideCymbal,
        LowTom,
        MidTom,
        HighTom,
        FloorTom,
        Other(Note),
    }

    /// Map MIDI note to drum type (GM standard)
    pub fn note_to_drum(note: Note) -> DrumNote {
        match note {
            35 | 36 => DrumNote::Kick,
            38 | 40 => DrumNote::Snare,
            37 => DrumNote::SideStick,
            42 => DrumNote::ClosedHiHat,
            46 => DrumNote::OpenHiHat,
            44 => DrumNote::PedalHiHat,
            49 | 57 => DrumNote::CrashCymbal,
            51 | 59 => DrumNote::RideCymbal,
            41 => DrumNote::FloorTom,
            43 => DrumNote::LowTom,
            45 | 47 => DrumNote::MidTom,
            48 | 50 => DrumNote::HighTom,
            _ => DrumNote::Other(note),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = MidiRouter::new();
        assert_eq!(router.stats().messages_routed, 0);
    }

    #[test]
    fn test_route_note_on() {
        let router = MidiRouter::new();

        let msg = MidiMessage::NoteOn {
            channel: 0,
            note: 64,
            velocity: 100,
        };
        router.route(msg, 0);

        let event = router.try_recv();
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.target, SynthTarget::Guitar);
        assert_eq!(event.string, Some(1)); // Channel 0 = String 1
    }

    #[test]
    fn test_route_to_piano() {
        let router = MidiRouter::new();

        let msg = MidiMessage::NoteOn {
            channel: 6, // Piano channel
            note: 60,
            velocity: 80,
        };
        router.route(msg, 12345);

        let event = router.try_recv().unwrap();
        assert_eq!(event.target, SynthTarget::Piano);
        assert_eq!(event.string, None);
        assert_eq!(event.timestamp, 12345);
    }

    #[test]
    fn test_route_to_drums() {
        let router = MidiRouter::new();

        let msg = MidiMessage::NoteOn {
            channel: 9, // Drum channel (MIDI convention)
            note: 36, // Kick
            velocity: 127,
        };
        router.route(msg, 0);

        let event = router.try_recv().unwrap();
        assert_eq!(event.target, SynthTarget::Drums);
    }

    #[test]
    fn test_unmapped_channel_uses_default() {
        let router = MidiRouter::new();

        let msg = MidiMessage::NoteOn {
            channel: 15, // Unmapped
            note: 60,
            velocity: 100,
        };
        router.route(msg, 0);

        let event = router.try_recv().unwrap();
        // Default is Guitar
        assert_eq!(event.target, SynthTarget::Guitar);
    }

    #[test]
    fn test_unmapped_dropped_when_disabled() {
        let config = RouterConfig {
            mappings: vec![],
            default_target: SynthTarget::Guitar,
            pass_unmapped: false,
        };
        let router = MidiRouter::with_config(config);

        let msg = MidiMessage::NoteOn {
            channel: 0,
            note: 60,
            velocity: 100,
        };
        router.route(msg, 0);

        assert!(router.try_recv().is_none());
        assert_eq!(router.stats().messages_dropped, 1);
    }

    #[test]
    fn test_route_batch() {
        let router = MidiRouter::new();

        let messages = vec![
            (MidiMessage::NoteOn { channel: 0, note: 60, velocity: 100 }, 0),
            (MidiMessage::NoteOn { channel: 0, note: 64, velocity: 100 }, 1000),
            (MidiMessage::NoteOff { channel: 0, note: 60, velocity: 0 }, 2000),
        ];
        router.route_batch(messages);

        let events = router.drain();
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_stats_tracking() {
        let router = MidiRouter::new();

        router.route(MidiMessage::NoteOn { channel: 0, note: 60, velocity: 100 }, 0);
        router.route(MidiMessage::NoteOff { channel: 0, note: 60, velocity: 0 }, 1000);
        router.route(MidiMessage::ControlChange { channel: 0, control: 7, value: 100 }, 2000);

        let stats = router.stats();
        assert_eq!(stats.messages_routed, 3);
        assert_eq!(stats.note_on_count, 1);
        assert_eq!(stats.note_off_count, 1);
        assert_eq!(stats.cc_count, 1);
    }

    #[test]
    fn test_add_mapping() {
        let router = MidiRouter::new();
        router.add_mapping(15, SynthTarget::Custom(42), None);

        router.route(MidiMessage::NoteOn { channel: 15, note: 60, velocity: 100 }, 0);

        let event = router.try_recv().unwrap();
        assert_eq!(event.target, SynthTarget::Custom(42));
    }

    #[test]
    fn test_remove_mapping() {
        let router = MidiRouter::new();
        router.remove_mapping(0);

        router.route(MidiMessage::NoteOn { channel: 0, note: 60, velocity: 100 }, 0);

        let event = router.try_recv().unwrap();
        // Falls back to default
        assert_eq!(event.target, SynthTarget::Guitar);
        assert_eq!(event.string, None); // Mapping was removed
    }

    #[test]
    fn test_guitar_note_mapping() {
        // E2 (open low E string)
        assert_eq!(guitar::note_to_string_fret(40), Some((1, 0)));
        // A2 (open A string)
        assert_eq!(guitar::note_to_string_fret(45), Some((2, 0)));
        // A3 (5th fret low E or open A)
        assert_eq!(guitar::note_to_string_fret(57), Some((4, 2))); // G string, 2nd fret
        // E4 (open high E)
        assert_eq!(guitar::note_to_string_fret(64), Some((6, 0)));
        // Below guitar range
        assert_eq!(guitar::note_to_string_fret(30), None);
    }

    #[test]
    fn test_guitar_range() {
        assert!(!guitar::is_guitar_range(39)); // Below E2
        assert!(guitar::is_guitar_range(40));  // E2
        assert!(guitar::is_guitar_range(64));  // E4
        assert!(guitar::is_guitar_range(86));  // D6 (24th fret high E)
        assert!(!guitar::is_guitar_range(87)); // Above range
    }

    #[test]
    fn test_velocity_to_intensity() {
        assert!((guitar::velocity_to_intensity(0) - 0.0).abs() < 0.01);
        assert!((guitar::velocity_to_intensity(127) - 1.0).abs() < 0.01);
        // Mid velocity should be slightly above 0.5 due to compression
        let mid = guitar::velocity_to_intensity(64);
        assert!(mid > 0.4 && mid < 0.6);
    }

    #[test]
    fn test_drum_note_mapping() {
        assert_eq!(drums::note_to_drum(36), drums::DrumNote::Kick);
        assert_eq!(drums::note_to_drum(38), drums::DrumNote::Snare);
        assert_eq!(drums::note_to_drum(42), drums::DrumNote::ClosedHiHat);
        assert_eq!(drums::note_to_drum(46), drums::DrumNote::OpenHiHat);
        assert_eq!(drums::note_to_drum(99), drums::DrumNote::Other(99));
    }

    #[test]
    fn test_system_messages_ignored() {
        let router = MidiRouter::new();

        router.route(MidiMessage::TimingClock, 0);
        router.route(MidiMessage::Start, 0);
        router.route(MidiMessage::Stop, 0);

        assert!(router.try_recv().is_none());
        assert_eq!(router.stats().messages_routed, 0);
    }
}
