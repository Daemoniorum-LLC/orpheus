//! Clip playback engine
//!
//! Handles playback of MIDI clips on the timeline.

use uuid::Uuid;
use std::collections::HashMap;
use crate::project::{Clip, ClipContent, MidiNote, Track};

/// A scheduled MIDI event for playback
#[derive(Debug, Clone)]
pub struct ScheduledMidiEvent {
    /// Track ID this event belongs to
    pub track_id: Uuid,
    /// MIDI note number
    pub note: u8,
    /// Velocity (0 = note off)
    pub velocity: u8,
    /// Sample position when this should trigger
    pub sample_position: u64,
    /// Is this a note on event
    pub is_note_on: bool,
}

/// Clip playback state for a single track
#[derive(Debug)]
struct TrackPlaybackState {
    /// Currently active notes (note -> end sample position)
    active_notes: HashMap<u8, u64>,
}

impl TrackPlaybackState {
    fn new() -> Self {
        Self {
            active_notes: HashMap::new(),
        }
    }
}

/// Clip playback engine
#[derive(Debug)]
pub struct ClipPlaybackEngine {
    /// Sample rate
    sample_rate: u32,
    /// Tempo in BPM
    tempo: f64,
    /// Ticks per quarter note (PPQN)
    ppqn: u32,
    /// Current playback position in samples
    position: u64,
    /// Is playing
    is_playing: bool,
    /// Playback state per track
    track_states: HashMap<Uuid, TrackPlaybackState>,
    /// Cached events (sorted by sample position)
    scheduled_events: Vec<ScheduledMidiEvent>,
    /// Index of next event to process
    next_event_index: usize,
}

impl ClipPlaybackEngine {
    /// Create a new clip playback engine
    pub fn new(sample_rate: u32, tempo: f64) -> Self {
        Self {
            sample_rate,
            tempo,
            ppqn: 480, // Standard PPQN
            position: 0,
            is_playing: false,
            track_states: HashMap::new(),
            scheduled_events: Vec::new(),
            next_event_index: 0,
        }
    }

    /// Set tempo
    pub fn set_tempo(&mut self, tempo: f64) {
        self.tempo = tempo;
    }

    /// Convert ticks to samples
    fn ticks_to_samples(&self, ticks: u64) -> u64 {
        // ticks / ppqn = beats
        // beats / (tempo/60) = seconds
        // seconds * sample_rate = samples
        let beats = ticks as f64 / self.ppqn as f64;
        let seconds = beats * 60.0 / self.tempo;
        (seconds * self.sample_rate as f64) as u64
    }

    /// Load clips from tracks and schedule events
    pub fn load_clips(&mut self, tracks: &[&Track]) {
        self.scheduled_events.clear();
        self.track_states.clear();
        self.next_event_index = 0;

        for track in tracks {
            self.track_states.insert(track.id, TrackPlaybackState::new());

            for clip in &track.clips {
                if let ClipContent::Midi { notes } = &clip.content {
                    // Schedule events for each note in the clip
                    for note in notes {
                        // Convert tick positions to samples, relative to clip start
                        let note_start_samples = self.ticks_to_samples(note.start);
                        let note_end_samples = self.ticks_to_samples(note.start + note.duration);

                        // Add clip start offset
                        let absolute_start = clip.start + note_start_samples;
                        let absolute_end = clip.start + note_end_samples;

                        // Schedule note on
                        self.scheduled_events.push(ScheduledMidiEvent {
                            track_id: track.id,
                            note: note.note,
                            velocity: note.velocity,
                            sample_position: absolute_start,
                            is_note_on: true,
                        });

                        // Schedule note off
                        self.scheduled_events.push(ScheduledMidiEvent {
                            track_id: track.id,
                            note: note.note,
                            velocity: 0,
                            sample_position: absolute_end,
                            is_note_on: false,
                        });
                    }
                }
            }
        }

        // Sort by sample position
        self.scheduled_events.sort_by_key(|e| e.sample_position);
    }

    /// Start playback
    pub fn play(&mut self) {
        self.is_playing = true;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// Stop and rewind
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.position = 0;
        self.next_event_index = 0;
        self.track_states.values_mut().for_each(|s| s.active_notes.clear());
    }

    /// Seek to position
    pub fn seek(&mut self, position: u64) {
        self.position = position;

        // Find the first event at or after this position
        self.next_event_index = self.scheduled_events
            .iter()
            .position(|e| e.sample_position >= position)
            .unwrap_or(self.scheduled_events.len());

        // Clear active notes when seeking
        self.track_states.values_mut().for_each(|s| s.active_notes.clear());
    }

    /// Check if playing
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Get current position
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Process a frame and get events that should trigger
    ///
    /// Returns events that should trigger within [position, position + frame_size)
    pub fn process_frame(&mut self, frame_size: u64) -> Vec<ScheduledMidiEvent> {
        if !self.is_playing {
            return Vec::new();
        }

        let frame_end = self.position + frame_size;
        let mut events = Vec::new();

        // Collect all events in this frame
        while self.next_event_index < self.scheduled_events.len() {
            let event = &self.scheduled_events[self.next_event_index];
            if event.sample_position >= frame_end {
                break;
            }

            events.push(event.clone());
            self.next_event_index += 1;
        }

        // Advance position
        self.position = frame_end;

        events
    }

    /// Get count of scheduled events (for debugging)
    pub fn event_count(&self) -> usize {
        self.scheduled_events.len()
    }

    /// Get count of remaining events
    pub fn remaining_events(&self) -> usize {
        self.scheduled_events.len().saturating_sub(self.next_event_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{TrackType, Clip};

    #[test]
    fn test_clip_playback() {
        let mut engine = ClipPlaybackEngine::new(48000, 120.0);

        // Create a track with a clip
        let mut track = Track::new("Test", TrackType::Midi);
        track.clips.push(Clip {
            id: Uuid::new_v4(),
            name: "Test Clip".into(),
            start: 0,
            length: 48000, // 1 second
            content: ClipContent::Midi {
                notes: vec![
                    MidiNote {
                        note: 60,
                        velocity: 100,
                        start: 0,      // At clip start
                        duration: 480, // Quarter note
                    },
                ],
            },
        });

        engine.load_clips(&[&track]);
        assert_eq!(engine.event_count(), 2); // note on + note off

        engine.play();

        // Process first frame - should get note on
        let events = engine.process_frame(128);
        assert_eq!(events.len(), 1);
        assert!(events[0].is_note_on);
        assert_eq!(events[0].note, 60);
    }

    #[test]
    fn test_ticks_to_samples() {
        let engine = ClipPlaybackEngine::new(48000, 120.0);

        // At 120 BPM, 480 ticks (1 beat) = 0.5 seconds = 24000 samples
        let samples = engine.ticks_to_samples(480);
        assert_eq!(samples, 24000);

        // Half a beat = 240 ticks = 12000 samples
        let samples = engine.ticks_to_samples(240);
        assert_eq!(samples, 12000);
    }
}
