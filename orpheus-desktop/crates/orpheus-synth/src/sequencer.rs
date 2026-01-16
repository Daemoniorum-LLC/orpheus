//! Tablature sequencer - converts tablature data to playback events

use orpheus_file::guitar_pro::{Beat, Duration, GuitarProFile, Measure, Note, NoteEffects};
use crate::playback::{NoteEvent, NoteEventType, PlaybackEngine};

/// Event from sequencing tablature
#[derive(Debug, Clone)]
pub struct SequencerEvent {
    /// Time in beats from start
    pub beat_position: f64,
    /// String number (1-indexed)
    pub string: u8,
    /// Fret number
    pub fret: u8,
    /// Velocity (0.0 - 1.0)
    pub velocity: f32,
    /// Duration in beats
    pub duration_beats: f64,
    /// Is this a rest?
    pub is_rest: bool,
    /// Note effects
    pub effects: NoteEffectFlags,
}

/// Simplified note effect flags for playback
#[derive(Debug, Clone, Copy, Default)]
pub struct NoteEffectFlags {
    pub hammer_on: bool,
    pub pull_off: bool,
    pub slide: bool,
    pub vibrato: bool,
    pub palm_mute: bool,
    pub let_ring: bool,
}

impl From<&NoteEffects> for NoteEffectFlags {
    fn from(effects: &NoteEffects) -> Self {
        Self {
            hammer_on: effects.hammer_on,
            pull_off: effects.pull_off,
            slide: effects.slide.is_some(),
            vibrato: effects.vibrato,
            ..Default::default()
        }
    }
}

/// Tablature sequencer
pub struct TabSequencer {
    /// Events extracted from tablature
    events: Vec<SequencerEvent>,
    /// Current tempo
    tempo: f64,
    /// Time signature numerator
    time_sig_num: u8,
    /// Time signature denominator
    time_sig_denom: u8,
    /// Total duration in beats
    total_beats: f64,
}

impl TabSequencer {
    /// Create a new sequencer from a Guitar Pro file
    pub fn from_gp_file(file: &GuitarProFile, track_index: usize) -> Self {
        let mut sequencer = Self {
            events: Vec::new(),
            tempo: file.tempo as f64,
            time_sig_num: file.time_signature.numerator,
            time_sig_denom: file.time_signature.denominator,
            total_beats: 0.0,
        };

        sequencer.extract_events(file, track_index);
        sequencer
    }

    /// Extract events from the Guitar Pro file
    fn extract_events(&mut self, file: &GuitarProFile, track_index: usize) {
        let mut current_beat_pos = 0.0;
        let mut current_tempo = file.tempo as f64;

        for measure in &file.measures {
            // Update tempo if changed
            if let Some(tempo) = measure.tempo {
                current_tempo = tempo as f64;
            }

            // Get beats for this track
            let track_beats = match measure.beats.get(track_index) {
                Some(tb) => &tb.beats,
                None => continue,
            };

            // Process each beat
            for beat in track_beats {
                let beat_duration = self.duration_to_beats(beat.duration, beat.dotted, beat.tuplet);

                if beat.is_rest {
                    // Just advance position for rests
                    current_beat_pos += beat_duration;
                    continue;
                }

                // Process notes in this beat
                for note in &beat.notes {
                    if note.tied {
                        // Skip tied notes (they're continuations)
                        continue;
                    }

                    let velocity = (note.velocity as f32 / 127.0).clamp(0.0, 1.0);
                    let effects = NoteEffectFlags::from(&note.effects);

                    self.events.push(SequencerEvent {
                        beat_position: current_beat_pos,
                        string: note.string,
                        fret: note.fret,
                        velocity,
                        duration_beats: beat_duration,
                        is_rest: false,
                        effects,
                    });
                }

                current_beat_pos += beat_duration;
            }
        }

        self.total_beats = current_beat_pos;
    }

    /// Convert duration enum to beats
    fn duration_to_beats(&self, duration: Duration, dotted: bool, tuplet: Option<u8>) -> f64 {
        let base_beats = match duration {
            Duration::Whole => 4.0,
            Duration::Half => 2.0,
            Duration::Quarter => 1.0,
            Duration::Eighth => 0.5,
            Duration::Sixteenth => 0.25,
            Duration::ThirtySecond => 0.125,
            Duration::SixtyFourth => 0.0625,
        };

        let mut beats = base_beats;

        // Apply dot (adds half the value)
        if dotted {
            beats *= 1.5;
        }

        // Apply tuplet
        if let Some(tuplet_num) = tuplet {
            // Common tuplets: 3 = triplet (3 in space of 2)
            // 5 = quintuplet (5 in space of 4)
            // 6 = sextuplet (6 in space of 4)
            let tuplet_ratio = match tuplet_num {
                3 => 2.0 / 3.0,
                5 => 4.0 / 5.0,
                6 => 4.0 / 6.0,
                7 => 4.0 / 7.0,
                _ => 1.0,
            };
            beats *= tuplet_ratio;
        }

        beats
    }

    /// Get total duration in beats
    pub fn total_beats(&self) -> f64 {
        self.total_beats
    }

    /// Get total duration in seconds at current tempo
    pub fn total_seconds(&self) -> f64 {
        self.total_beats * 60.0 / self.tempo
    }

    /// Get tempo
    pub fn tempo(&self) -> f64 {
        self.tempo
    }

    /// Get all events
    pub fn events(&self) -> &[SequencerEvent] {
        &self.events
    }

    /// Convert to playback events for the engine
    pub fn to_playback_events(&self, sample_rate: u32) -> Vec<NoteEvent> {
        let seconds_per_beat = 60.0 / self.tempo;
        let samples_per_beat = (sample_rate as f64 * seconds_per_beat) as u64;

        let mut playback_events = Vec::with_capacity(self.events.len() * 2);

        for event in &self.events {
            if event.is_rest {
                continue;
            }

            let sample_pos = (event.beat_position * samples_per_beat as f64) as u64;
            let note_off_pos = ((event.beat_position + event.duration_beats) * samples_per_beat as f64) as u64;

            // Determine event type
            let event_type = if event.effects.hammer_on {
                NoteEventType::HammerOn
            } else if event.effects.pull_off {
                NoteEventType::PullOff
            } else if event.effects.slide {
                NoteEventType::Slide
            } else {
                NoteEventType::NoteOn
            };

            // Note on
            playback_events.push(NoteEvent {
                string: event.string,
                fret: event.fret,
                velocity: event.velocity,
                sample_pos,
                event_type,
            });

            // Note off (unless let ring)
            if !event.effects.let_ring {
                playback_events.push(NoteEvent {
                    string: event.string,
                    fret: event.fret,
                    velocity: 0.0,
                    sample_pos: note_off_pos.saturating_sub(100), // Slightly early to avoid overlap
                    event_type: NoteEventType::NoteOff,
                });
            }
        }

        // Sort by sample position
        playback_events.sort_by_key(|e| e.sample_pos);

        playback_events
    }

    /// Load events into a playback engine
    pub fn load_into_engine(&self, engine: &mut PlaybackEngine) {
        engine.clear_events();
        engine.set_tempo(self.tempo);

        let events = self.to_playback_events(44100); // Assume 44100 for now
        engine.schedule_events(events);
    }

    /// Render tablature to audio buffer
    pub fn render(&self, sample_rate: u32, stereo: bool) -> Vec<f32> {
        let mut engine = PlaybackEngine::new(sample_rate);
        self.load_into_engine(&mut engine);

        let duration = self.total_seconds() + 2.0; // Add 2 seconds for decay
        engine.render_to_buffer(duration, stereo)
    }
}

/// Create events for a simple test sequence
pub fn create_test_sequence() -> Vec<SequencerEvent> {
    // Simple E minor arpeggio
    vec![
        SequencerEvent {
            beat_position: 0.0,
            string: 6,
            fret: 0,
            velocity: 0.8,
            duration_beats: 0.5,
            is_rest: false,
            effects: NoteEffectFlags::default(),
        },
        SequencerEvent {
            beat_position: 0.5,
            string: 5,
            fret: 2,
            velocity: 0.7,
            duration_beats: 0.5,
            is_rest: false,
            effects: NoteEffectFlags::default(),
        },
        SequencerEvent {
            beat_position: 1.0,
            string: 4,
            fret: 2,
            velocity: 0.7,
            duration_beats: 0.5,
            is_rest: false,
            effects: NoteEffectFlags::default(),
        },
        SequencerEvent {
            beat_position: 1.5,
            string: 3,
            fret: 0,
            velocity: 0.7,
            duration_beats: 0.5,
            is_rest: false,
            effects: NoteEffectFlags::default(),
        },
        SequencerEvent {
            beat_position: 2.0,
            string: 2,
            fret: 0,
            velocity: 0.7,
            duration_beats: 0.5,
            is_rest: false,
            effects: NoteEffectFlags::default(),
        },
        SequencerEvent {
            beat_position: 2.5,
            string: 1,
            fret: 0,
            velocity: 0.8,
            duration_beats: 0.5,
            is_rest: false,
            effects: NoteEffectFlags::default(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_conversion() {
        let seq = TabSequencer {
            events: Vec::new(),
            tempo: 120.0,
            time_sig_num: 4,
            time_sig_denom: 4,
            total_beats: 0.0,
        };

        assert_eq!(seq.duration_to_beats(Duration::Quarter, false, None), 1.0);
        assert_eq!(seq.duration_to_beats(Duration::Quarter, true, None), 1.5);
        assert!((seq.duration_to_beats(Duration::Quarter, false, Some(3)) - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_render() {
        // Create a simple sequence manually
        let events = create_test_sequence();

        let mut engine = PlaybackEngine::new(44100);
        engine.set_tempo(120.0);

        // Convert to playback events
        let seconds_per_beat = 60.0 / 120.0;
        let samples_per_beat = (44100.0 * seconds_per_beat) as u64;

        for event in events {
            engine.schedule_event(NoteEvent {
                string: event.string,
                fret: event.fret,
                velocity: event.velocity,
                sample_pos: (event.beat_position * samples_per_beat as f64) as u64,
                event_type: NoteEventType::NoteOn,
            });
        }

        let buffer = engine.render_to_buffer(3.0, false);
        assert!(!buffer.is_empty());

        // Should have audio
        let max = buffer.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }
}
