//! MIDI file export for TabDocument
//!
//! Converts tablature documents to Standard MIDI Files (SMF) format.

use std::path::Path;
use midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track as MidiTrack,
    TrackEvent, TrackEventKind,
    num::{u4, u7, u15, u24, u28},
};
use orpheus_core::tab::{TabDocument, TabTrack, TabBeat, TabNote, Instrument};

/// Ticks per quarter note (standard resolution)
const TICKS_PER_QUARTER: u16 = 480;

/// MIDI export options
#[derive(Debug, Clone)]
pub struct MidiExportOptions {
    /// Include tempo track
    pub include_tempo: bool,
    /// Include track names
    pub include_track_names: bool,
    /// Base velocity (0-127)
    pub base_velocity: u8,
    /// Velocity variation for dynamics (percentage)
    pub velocity_variation: u8,
}

impl Default for MidiExportOptions {
    fn default() -> Self {
        Self {
            include_tempo: true,
            include_track_names: true,
            base_velocity: 100,
            velocity_variation: 20,
        }
    }
}

/// Export a TabDocument to MIDI file
pub fn export_midi(doc: &TabDocument, path: &Path, options: &MidiExportOptions) -> crate::Result<()> {
    let smf = tab_to_smf(doc, options)?;

    smf.save(path)?;

    tracing::info!("Exported MIDI to: {:?}", path);
    Ok(())
}

/// Export a TabDocument to MIDI bytes
pub fn export_midi_bytes(doc: &TabDocument, options: &MidiExportOptions) -> crate::Result<Vec<u8>> {
    let smf = tab_to_smf(doc, options)?;

    let mut bytes = Vec::new();
    smf.write_std(&mut bytes)?;
    Ok(bytes)
}

/// Convert TabDocument to MIDI SMF
fn tab_to_smf<'a>(doc: &TabDocument, options: &MidiExportOptions) -> crate::Result<Smf<'a>> {
    let mut tracks: Vec<MidiTrack<'a>> = Vec::new();

    // Track 0: Tempo and metadata (conductor track)
    if options.include_tempo {
        tracks.push(create_tempo_track(doc, options));
    }

    // Convert each TabTrack to MIDI track
    for (track_idx, tab_track) in doc.tracks.iter().enumerate() {
        let midi_track = convert_track(doc, tab_track, track_idx, options)?;
        tracks.push(midi_track);
    }

    let smf = Smf {
        header: Header {
            format: Format::Parallel,
            timing: Timing::Metrical(u15::new(TICKS_PER_QUARTER)),
        },
        tracks,
    };

    Ok(smf)
}

/// Create tempo/metadata track
fn create_tempo_track<'a>(doc: &TabDocument, options: &MidiExportOptions) -> MidiTrack<'a> {
    let mut events = Vec::new();

    // Track name
    if options.include_track_names && !doc.metadata.title.is_empty() {
        events.push(TrackEvent {
            delta: u28::new(0),
            kind: TrackEventKind::Meta(MetaMessage::TrackName(
                doc.metadata.title.as_bytes().to_vec().leak()
            )),
        });
    }

    // Tempo (microseconds per quarter note)
    let tempo_bpm = doc.tempo_map.base_tempo;
    let microseconds_per_beat = (60_000_000.0 / tempo_bpm) as u32;
    events.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(microseconds_per_beat))),
    });

    // Time signature (default 4/4)
    events.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::TimeSignature(4, 2, 24, 8)),
    });

    // End of track
    let total_ticks = calculate_total_ticks(doc);
    events.push(TrackEvent {
        delta: u28::new(total_ticks),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    events
}

/// Convert a TabTrack to MIDI track
fn convert_track<'a>(
    doc: &TabDocument,
    track: &TabTrack,
    track_idx: usize,
    options: &MidiExportOptions,
) -> crate::Result<MidiTrack<'a>> {
    let mut events = Vec::new();
    let channel = u4::new((track_idx % 16) as u8);

    // Track name
    if options.include_track_names {
        events.push(TrackEvent {
            delta: u28::new(0),
            kind: TrackEventKind::Meta(MetaMessage::TrackName(
                track.name.as_bytes().to_vec().leak()
            )),
        });
    }

    // Program change (instrument)
    let program = match &track.instrument {
        Instrument::StringedInstrument(s) => {
            // Use appropriate MIDI program
            match s.string_count {
                4 => 33, // Electric Bass (finger)
                5 => 33, // 5-string bass
                _ => 25, // Acoustic Guitar (steel)
            }
        }
        Instrument::Drums(_) => 0, // Drums are on channel 10
        Instrument::Keys(_) => 0,  // Grand Piano
    };

    events.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Midi {
            channel,
            message: MidiMessage::ProgramChange {
                program: u7::new(program),
            },
        },
    });

    // Get tuning for note calculation
    let tuning = get_track_tuning(track);

    // Process all measures
    let mut current_tick: u32 = 0;
    let mut last_event_tick: u32 = 0;
    let mut pending_note_offs: Vec<(u32, u8)> = Vec::new(); // (tick, note)

    for measure in &doc.measures {
        // Find this track's beats in the measure
        if let Some(track_beats) = measure.track_beats.iter().find(|tb| tb.track_id == track.id) {
            for beat in &track_beats.beats {
                // Process any pending note offs before this beat
                pending_note_offs.sort_by_key(|(t, _)| *t);
                while let Some(&(off_tick, note)) = pending_note_offs.first() {
                    if off_tick <= current_tick {
                        let delta = off_tick.saturating_sub(last_event_tick);
                        events.push(TrackEvent {
                            delta: u28::new(delta),
                            kind: TrackEventKind::Midi {
                                channel,
                                message: MidiMessage::NoteOff {
                                    key: u7::new(note),
                                    vel: u7::new(0),
                                },
                            },
                        });
                        last_event_tick = off_tick;
                        pending_note_offs.remove(0);
                    } else {
                        break;
                    }
                }

                if !beat.is_rest {
                    // Calculate note duration in ticks
                    let duration_ticks = rhythm_to_ticks(&beat.rhythm);

                    // Add note on events for each note in the beat
                    for note in &beat.notes {
                        if note.dead {
                            continue; // Skip dead notes for MIDI
                        }

                        let midi_note = fret_to_midi_note(note.string, note.fret, &tuning);
                        let velocity = calculate_velocity(note, options);

                        let delta = current_tick.saturating_sub(last_event_tick);
                        events.push(TrackEvent {
                            delta: u28::new(delta),
                            kind: TrackEventKind::Midi {
                                channel,
                                message: MidiMessage::NoteOn {
                                    key: u7::new(midi_note),
                                    vel: u7::new(velocity),
                                },
                            },
                        });
                        last_event_tick = current_tick;

                        // Schedule note off
                        pending_note_offs.push((current_tick + duration_ticks, midi_note));
                    }
                }

                // Advance tick position
                current_tick += rhythm_to_ticks(&beat.rhythm);
            }
        }
    }

    // Flush remaining note offs
    pending_note_offs.sort_by_key(|(t, _)| *t);
    for (off_tick, note) in pending_note_offs {
        let delta = off_tick.saturating_sub(last_event_tick);
        events.push(TrackEvent {
            delta: u28::new(delta),
            kind: TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOff {
                    key: u7::new(note),
                    vel: u7::new(0),
                },
            },
        });
        last_event_tick = off_tick;
    }

    // End of track
    events.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    Ok(events)
}

/// Get tuning (open string MIDI notes) for a track
fn get_track_tuning(track: &TabTrack) -> Vec<u8> {
    match &track.instrument {
        Instrument::StringedInstrument(s) => {
            s.tuning.iter().map(|&n| n as u8).collect()
        }
        _ => vec![64, 59, 55, 50, 45, 40], // Standard guitar tuning (E2 to E4)
    }
}

/// Convert fret position to MIDI note number
fn fret_to_midi_note(string: u8, fret: u8, tuning: &[u8]) -> u8 {
    let string_idx = (string as usize).saturating_sub(1);
    let open_note = tuning.get(string_idx).copied().unwrap_or(64);
    open_note.saturating_add(fret).min(127)
}

/// Convert rhythm value to MIDI ticks
fn rhythm_to_ticks(rhythm: &orpheus_core::tab::RhythmValue) -> u32 {
    use orpheus_core::tab::BaseDuration;

    let base_ticks = match rhythm.base {
        BaseDuration::Longa => TICKS_PER_QUARTER as u32 * 16,
        BaseDuration::Breve => TICKS_PER_QUARTER as u32 * 8,
        BaseDuration::Whole => TICKS_PER_QUARTER as u32 * 4,
        BaseDuration::Half => TICKS_PER_QUARTER as u32 * 2,
        BaseDuration::Quarter => TICKS_PER_QUARTER as u32,
        BaseDuration::Eighth => TICKS_PER_QUARTER as u32 / 2,
        BaseDuration::Sixteenth => TICKS_PER_QUARTER as u32 / 4,
        BaseDuration::ThirtySecond => TICKS_PER_QUARTER as u32 / 8,
        BaseDuration::SixtyFourth => TICKS_PER_QUARTER as u32 / 16,
        BaseDuration::OneHundredTwentyEighth => TICKS_PER_QUARTER as u32 / 32,
        BaseDuration::TwoHundredFiftySixth => TICKS_PER_QUARTER as u32 / 64,
    };

    // Apply dots
    let mut dotted_ticks = base_ticks;
    let mut dot_value = base_ticks / 2;
    for _ in 0..rhythm.dots {
        dotted_ticks += dot_value;
        dot_value /= 2;
    }

    // Apply tuplet
    if let Some(ref tuplet) = rhythm.tuplet {
        dotted_ticks = (dotted_ticks * tuplet.actual as u32) / tuplet.normal as u32;
    }

    dotted_ticks.max(1)
}

/// Calculate note velocity based on note properties
fn calculate_velocity(note: &TabNote, options: &MidiExportOptions) -> u8 {
    let mut velocity = note.velocity.min(127);

    // Use base velocity if note velocity is default
    if velocity == 100 {
        velocity = options.base_velocity;
    }

    // Reduce velocity for ghost notes
    if note.ghost {
        velocity = (velocity as u32 * 60 / 100).min(127) as u8;
    }

    velocity.max(1).min(127)
}

/// Calculate total ticks for the document
fn calculate_total_ticks(doc: &TabDocument) -> u32 {
    let mut total: u32 = 0;

    for measure in &doc.measures {
        // Get the longest track in this measure
        let measure_ticks = measure.track_beats.iter()
            .map(|tb| {
                tb.beats.iter()
                    .map(|b| rhythm_to_ticks(&b.rhythm))
                    .sum::<u32>()
            })
            .max()
            .unwrap_or(TICKS_PER_QUARTER as u32 * 4); // Default to 4/4

        total += measure_ticks;
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use orpheus_core::tab::{TabTrack, RhythmValue, BaseDuration};

    #[test]
    fn test_rhythm_to_ticks() {
        let quarter = RhythmValue::new(BaseDuration::Quarter);
        assert_eq!(rhythm_to_ticks(&quarter), TICKS_PER_QUARTER as u32);

        let eighth = RhythmValue::new(BaseDuration::Eighth);
        assert_eq!(rhythm_to_ticks(&eighth), TICKS_PER_QUARTER as u32 / 2);

        let dotted_quarter = RhythmValue::new(BaseDuration::Quarter).dotted();
        assert_eq!(rhythm_to_ticks(&dotted_quarter), TICKS_PER_QUARTER as u32 + TICKS_PER_QUARTER as u32 / 2);
    }

    #[test]
    fn test_fret_to_midi() {
        let tuning = vec![64, 59, 55, 50, 45, 40]; // Standard guitar

        // String 1 (high E), fret 0 = E4 = 64
        assert_eq!(fret_to_midi_note(1, 0, &tuning), 64);

        // String 1, fret 12 = E5 = 76
        assert_eq!(fret_to_midi_note(1, 12, &tuning), 76);

        // String 6 (low E), fret 0 = E2 = 40
        assert_eq!(fret_to_midi_note(6, 0, &tuning), 40);
    }
}
