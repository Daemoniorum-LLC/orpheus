//! Recording system for capturing MIDI and audio
//!
//! Handles buffering of recorded events and conversion to clips.

use uuid::Uuid;
use std::collections::HashMap;
use crate::project::{Clip, ClipContent, MidiNote};

/// A recorded MIDI event (before conversion to notes)
#[derive(Debug, Clone)]
pub struct RecordedMidiEvent {
    /// Note number
    pub note: u8,
    /// Velocity (0 for note off)
    pub velocity: u8,
    /// Sample position when event occurred
    pub sample_position: u64,
    /// Is note on (true) or note off (false)
    pub is_note_on: bool,
}

/// Recording buffer for a single track
#[derive(Debug, Clone)]
pub struct TrackRecordingBuffer {
    /// Track ID this buffer belongs to
    pub track_id: Uuid,
    /// Recording start position in samples
    pub start_position: u64,
    /// Current recording position in samples
    pub current_position: u64,
    /// Recorded MIDI events
    pub midi_events: Vec<RecordedMidiEvent>,
    /// Currently held notes (for note-off matching)
    pub held_notes: HashMap<u8, u64>, // note -> start sample position
    /// Sample rate
    pub sample_rate: u32,
    /// Tempo for tick conversion
    pub tempo: f64,
}

impl TrackRecordingBuffer {
    pub fn new(track_id: Uuid, start_position: u64, sample_rate: u32, tempo: f64) -> Self {
        Self {
            track_id,
            start_position,
            current_position: start_position,
            midi_events: Vec::new(),
            held_notes: HashMap::new(),
            sample_rate,
            tempo,
        }
    }

    /// Record a note on event
    pub fn note_on(&mut self, note: u8, velocity: u8, sample_position: u64) {
        self.midi_events.push(RecordedMidiEvent {
            note,
            velocity,
            sample_position,
            is_note_on: true,
        });
        self.held_notes.insert(note, sample_position);
        self.current_position = sample_position;
    }

    /// Record a note off event
    pub fn note_off(&mut self, note: u8, sample_position: u64) {
        self.midi_events.push(RecordedMidiEvent {
            note,
            velocity: 0,
            sample_position,
            is_note_on: false,
        });
        self.held_notes.remove(&note);
        self.current_position = sample_position;
    }

    /// Update current position (called during playback)
    pub fn update_position(&mut self, sample_position: u64) {
        self.current_position = sample_position;
    }

    /// Get the length of the recording in samples
    pub fn length(&self) -> u64 {
        if self.current_position > self.start_position {
            self.current_position - self.start_position
        } else {
            0
        }
    }

    /// Convert samples to ticks (for MIDI storage)
    fn samples_to_ticks(&self, samples: u64) -> u64 {
        // Use 480 PPQN (pulses per quarter note)
        const PPQN: f64 = 480.0;
        let beats_per_second = self.tempo / 60.0;
        let seconds = samples as f64 / self.sample_rate as f64;
        let beats = seconds * beats_per_second;
        (beats * PPQN) as u64
    }

    /// Finalize recording and create a clip
    pub fn finalize(mut self) -> Option<Clip> {
        let length = self.length();
        if length == 0 || self.midi_events.is_empty() {
            return None;
        }

        // Close any still-held notes
        for (&note, &_start) in &self.held_notes.clone() {
            self.midi_events.push(RecordedMidiEvent {
                note,
                velocity: 0,
                sample_position: self.current_position,
                is_note_on: false,
            });
        }

        // Convert events to notes
        let notes = self.convert_to_notes();

        if notes.is_empty() {
            return None;
        }

        Some(Clip {
            id: Uuid::new_v4(),
            name: format!("Recording {}", chrono::Utc::now().format("%H:%M:%S")),
            start: self.start_position,
            length,
            content: ClipContent::Midi { notes },
        })
    }

    /// Convert recorded events to MIDI notes
    fn convert_to_notes(&self) -> Vec<MidiNote> {
        let mut notes = Vec::new();
        let mut pending_notes: HashMap<u8, (u64, u8)> = HashMap::new(); // note -> (start_tick, velocity)

        for event in &self.midi_events {
            let tick = self.samples_to_ticks(event.sample_position - self.start_position);

            if event.is_note_on && event.velocity > 0 {
                pending_notes.insert(event.note, (tick, event.velocity));
            } else {
                // Note off
                if let Some((start_tick, velocity)) = pending_notes.remove(&event.note) {
                    let duration = if tick > start_tick { tick - start_tick } else { 1 };
                    notes.push(MidiNote {
                        note: event.note,
                        velocity,
                        start: start_tick,
                        duration,
                    });
                }
            }
        }

        // Handle any remaining pending notes (held to the end)
        let end_tick = self.samples_to_ticks(self.length());
        for (note, (start_tick, velocity)) in pending_notes {
            let duration = if end_tick > start_tick { end_tick - start_tick } else { 1 };
            notes.push(MidiNote {
                note,
                velocity,
                start: start_tick,
                duration,
            });
        }

        // Sort by start time
        notes.sort_by_key(|n| n.start);
        notes
    }
}

/// Recording state manager
#[derive(Debug, Default)]
pub struct RecordingState {
    /// Active recording buffers (one per armed track)
    pub buffers: HashMap<Uuid, TrackRecordingBuffer>,
    /// Is recording active
    pub is_recording: bool,
}

impl RecordingState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start recording on armed tracks
    pub fn start(&mut self, armed_track_ids: Vec<Uuid>, start_position: u64, sample_rate: u32, tempo: f64) {
        self.buffers.clear();
        for track_id in armed_track_ids {
            self.buffers.insert(
                track_id,
                TrackRecordingBuffer::new(track_id, start_position, sample_rate, tempo),
            );
        }
        self.is_recording = true;
    }

    /// Stop recording and get clips
    pub fn stop(&mut self) -> Vec<(Uuid, Clip)> {
        self.is_recording = false;
        let buffers = std::mem::take(&mut self.buffers);

        buffers
            .into_iter()
            .filter_map(|(track_id, buffer)| {
                buffer.finalize().map(|clip| (track_id, clip))
            })
            .collect()
    }

    /// Record a note on to a specific track
    pub fn record_note_on(&mut self, track_id: Uuid, note: u8, velocity: u8, sample_position: u64) {
        if let Some(buffer) = self.buffers.get_mut(&track_id) {
            buffer.note_on(note, velocity, sample_position);
        }
    }

    /// Record a note off to a specific track
    pub fn record_note_off(&mut self, track_id: Uuid, note: u8, sample_position: u64) {
        if let Some(buffer) = self.buffers.get_mut(&track_id) {
            buffer.note_off(note, sample_position);
        }
    }

    /// Update position for all buffers
    pub fn update_position(&mut self, sample_position: u64) {
        for buffer in self.buffers.values_mut() {
            buffer.update_position(sample_position);
        }
    }

    /// Check if a track is being recorded
    pub fn is_track_recording(&self, track_id: Uuid) -> bool {
        self.is_recording && self.buffers.contains_key(&track_id)
    }

    /// Get number of events recorded on a track
    pub fn event_count(&self, track_id: Uuid) -> usize {
        self.buffers
            .get(&track_id)
            .map(|b| b.midi_events.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_buffer() {
        let track_id = Uuid::new_v4();
        let mut buffer = TrackRecordingBuffer::new(track_id, 0, 48000, 120.0);

        // Record a note
        buffer.note_on(60, 100, 0);
        buffer.note_off(60, 24000); // Half second

        let clip = buffer.finalize().expect("Should create clip");
        assert_eq!(clip.start, 0);
        assert_eq!(clip.length, 24000);

        if let ClipContent::Midi { notes } = &clip.content {
            assert_eq!(notes.len(), 1);
            assert_eq!(notes[0].note, 60);
            assert_eq!(notes[0].velocity, 100);
        } else {
            panic!("Expected MIDI content");
        }
    }

    #[test]
    fn test_recording_state() {
        let mut state = RecordingState::new();
        let track_id = Uuid::new_v4();

        state.start(vec![track_id], 0, 48000, 120.0);
        assert!(state.is_recording);
        assert!(state.is_track_recording(track_id));

        state.record_note_on(track_id, 64, 80, 0);
        state.record_note_off(track_id, 64, 48000);

        let clips = state.stop();
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0].0, track_id);
    }

    #[test]
    fn test_empty_recording() {
        let mut state = RecordingState::new();
        let track_id = Uuid::new_v4();

        state.start(vec![track_id], 0, 48000, 120.0);
        let clips = state.stop();
        assert!(clips.is_empty()); // No events = no clip
    }
}
