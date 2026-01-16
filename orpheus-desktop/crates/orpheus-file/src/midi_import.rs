//! MIDI file import for TabDocument
//!
//! Converts Standard MIDI Files (SMF) to tablature documents with:
//! - Tempo map preservation
//! - Time signature preservation
//! - Marker/section import
//! - Intelligent fret position assignment
//! - Multi-track support

use std::path::Path;
use std::collections::HashMap;
use midly::{Smf, TrackEventKind, MetaMessage, MidiMessage};
use orpheus_core::tab::{
    BaseDuration, Instrument, RhythmValue, SectionMarker, StringedConfig,
    TabBeat, TabDocument, TabMeasure, TabNote, TabTrack, TempoChange,
    TempoChangeType, TempoMap, TimeSignature as TabTimeSignature, TrackMeasure,
};
use uuid::Uuid;
use tracing::{debug, warn};

/// Ticks per quarter note (standard resolution, will be read from file)
const DEFAULT_PPQN: u16 = 480;

/// MIDI import options
#[derive(Debug, Clone)]
pub struct MidiImportOptions {
    /// Target instrument type (determines string count and tuning)
    pub instrument_type: ImportInstrumentType,
    /// Quantize notes to nearest grid value
    pub quantize_to: Option<BaseDuration>,
    /// Minimum velocity to consider (filters ghost notes)
    pub min_velocity: u8,
    /// Whether to import tempo changes
    pub import_tempo: bool,
    /// Whether to import time signatures
    pub import_time_signatures: bool,
    /// Whether to import markers as sections
    pub import_markers: bool,
    /// Fret position preference (lower = prefer lower frets)
    pub fret_preference: FretPreference,
}

impl Default for MidiImportOptions {
    fn default() -> Self {
        Self {
            instrument_type: ImportInstrumentType::Guitar6Standard,
            quantize_to: Some(BaseDuration::Sixteenth),
            min_velocity: 1,
            import_tempo: true,
            import_time_signatures: true,
            import_markers: true,
            fret_preference: FretPreference::LowerFrets,
        }
    }
}

/// Target instrument for MIDI import
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportInstrumentType {
    Guitar6Standard,
    Guitar7Standard,
    Guitar8Standard,
    Bass4Standard,
    Bass5Standard,
    Custom { strings: u8, tuning: [i8; 8] },
}

impl ImportInstrumentType {
    /// Get the tuning as MIDI note numbers
    pub fn tuning(&self) -> Vec<u8> {
        match self {
            Self::Guitar6Standard => vec![64, 59, 55, 50, 45, 40], // E4 B3 G3 D3 A2 E2
            Self::Guitar7Standard => vec![64, 59, 55, 50, 45, 40, 35], // + B1
            Self::Guitar8Standard => vec![64, 59, 55, 50, 45, 40, 35, 30], // + F#1
            Self::Bass4Standard => vec![43, 38, 33, 28], // G2 D2 A1 E1
            Self::Bass5Standard => vec![43, 38, 33, 28, 23], // + B0
            Self::Custom { tuning, strings } => tuning[..*strings as usize].iter().map(|&n| n as u8).collect(),
        }
    }

    /// Get the string count
    pub fn string_count(&self) -> u8 {
        match self {
            Self::Guitar6Standard => 6,
            Self::Guitar7Standard => 7,
            Self::Guitar8Standard => 8,
            Self::Bass4Standard => 4,
            Self::Bass5Standard => 5,
            Self::Custom { strings, .. } => *strings,
        }
    }

    /// Get the number of usable frets
    pub fn fret_count(&self) -> u8 {
        match self {
            Self::Bass4Standard | Self::Bass5Standard => 24,
            _ => 24, // Standard guitar fret count
        }
    }
}

/// Fret position preference for note assignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FretPreference {
    /// Prefer lower frets (easier to play)
    LowerFrets,
    /// Prefer middle frets (balanced)
    MiddleFrets,
    /// Prefer higher frets (lead guitar style)
    HigherFrets,
    /// Minimize hand movement between notes
    MinimizeMovement,
}

/// Extracted MIDI data before conversion
#[derive(Debug)]
struct MidiData {
    /// Pulses per quarter note
    ppqn: u16,
    /// Tempo changes (tick, microseconds per beat)
    tempo_changes: Vec<(u32, u32)>,
    /// Time signature changes (tick, numerator, denominator)
    time_sig_changes: Vec<(u32, u8, u8)>,
    /// Markers (tick, name)
    markers: Vec<(u32, String)>,
    /// Track name
    track_name: Option<String>,
    /// Notes (tick, channel, note, velocity, duration_ticks)
    notes: Vec<MidiNote>,
}

#[derive(Debug, Clone)]
struct MidiNote {
    tick: u32,
    channel: u8,
    note: u8,
    velocity: u8,
    duration_ticks: u32,
}

/// Import result with metadata
#[derive(Debug)]
pub struct MidiImportResult {
    /// The converted TabDocument
    pub document: TabDocument,
    /// Number of notes imported
    pub note_count: usize,
    /// Number of notes that couldn't be assigned to frets
    pub unassigned_notes: usize,
    /// Detected tempo (BPM)
    pub detected_tempo: f64,
    /// Detected time signature
    pub detected_time_sig: (u8, u8),
    /// Warnings during import
    pub warnings: Vec<String>,
}

/// Import a MIDI file into a TabDocument
pub fn import_midi(path: &Path, options: &MidiImportOptions) -> crate::Result<MidiImportResult> {
    let data = std::fs::read(path)?;
    import_midi_bytes(&data, options)
}

/// Import MIDI from bytes
pub fn import_midi_bytes(data: &[u8], options: &MidiImportOptions) -> crate::Result<MidiImportResult> {
    let smf = Smf::parse(data).map_err(|e| crate::Error::Parse(format!("MIDI parse error: {}", e)))?;

    let ppqn = match smf.header.timing {
        midly::Timing::Metrical(ticks) => ticks.as_int(),
        midly::Timing::Timecode(_, _) => {
            warn!("SMPTE timecode not fully supported, using default PPQN");
            DEFAULT_PPQN
        }
    };

    // Extract data from all tracks
    let midi_data = extract_midi_data(&smf, ppqn);

    // Convert to TabDocument
    convert_to_tab_document(midi_data, options)
}

/// Extract note and metadata from MIDI tracks
fn extract_midi_data(smf: &Smf, ppqn: u16) -> MidiData {
    let mut data = MidiData {
        ppqn,
        tempo_changes: Vec::new(),
        time_sig_changes: Vec::new(),
        markers: Vec::new(),
        track_name: None,
        notes: Vec::new(),
    };

    // Track active notes for duration calculation
    let mut active_notes: HashMap<(u8, u8), (u32, u8)> = HashMap::new(); // (channel, note) -> (start_tick, velocity)

    for track in &smf.tracks {
        let mut current_tick: u32 = 0;

        for event in track {
            current_tick += event.delta.as_int();

            match event.kind {
                TrackEventKind::Meta(meta) => {
                    match meta {
                        MetaMessage::Tempo(tempo) => {
                            data.tempo_changes.push((current_tick, tempo.as_int()));
                        }
                        MetaMessage::TimeSignature(num, denom, _, _) => {
                            let denom_value = 2u8.pow(denom as u32);
                            data.time_sig_changes.push((current_tick, num, denom_value));
                        }
                        MetaMessage::Marker(text) => {
                            if let Ok(s) = std::str::from_utf8(text) {
                                data.markers.push((current_tick, s.to_string()));
                            }
                        }
                        MetaMessage::CuePoint(text) => {
                            // Also treat cue points as markers
                            if let Ok(s) = std::str::from_utf8(text) {
                                data.markers.push((current_tick, s.to_string()));
                            }
                        }
                        MetaMessage::TrackName(name) => {
                            if let Ok(s) = std::str::from_utf8(name) {
                                if data.track_name.is_none() {
                                    data.track_name = Some(s.to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                TrackEventKind::Midi { channel, message } => {
                    let ch = channel.as_int();
                    match message {
                        MidiMessage::NoteOn { key, vel } => {
                            let note = key.as_int();
                            let velocity = vel.as_int();

                            if velocity > 0 {
                                // Note on
                                active_notes.insert((ch, note), (current_tick, velocity));
                            } else {
                                // Note off (velocity 0)
                                if let Some((start_tick, vel)) = active_notes.remove(&(ch, note)) {
                                    let duration = current_tick.saturating_sub(start_tick);
                                    data.notes.push(MidiNote {
                                        tick: start_tick,
                                        channel: ch,
                                        note,
                                        velocity: vel,
                                        duration_ticks: duration.max(1),
                                    });
                                }
                            }
                        }
                        MidiMessage::NoteOff { key, .. } => {
                            let note = key.as_int();
                            if let Some((start_tick, vel)) = active_notes.remove(&(ch, note)) {
                                let duration = current_tick.saturating_sub(start_tick);
                                data.notes.push(MidiNote {
                                    tick: start_tick,
                                    channel: ch,
                                    note,
                                    velocity: vel,
                                    duration_ticks: duration.max(1),
                                });
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    // Sort notes by tick
    data.notes.sort_by_key(|n| n.tick);

    debug!(
        "Extracted {} notes, {} tempo changes, {} time sigs, {} markers",
        data.notes.len(),
        data.tempo_changes.len(),
        data.time_sig_changes.len(),
        data.markers.len()
    );

    data
}

/// Convert extracted MIDI data to TabDocument
fn convert_to_tab_document(
    midi_data: MidiData,
    options: &MidiImportOptions,
) -> crate::Result<MidiImportResult> {
    let mut doc = TabDocument::new();
    let mut warnings = Vec::new();
    let mut unassigned = 0;

    // Get tuning for fret calculation
    let tuning = options.instrument_type.tuning();
    let fret_count = options.instrument_type.fret_count();
    let string_count = options.instrument_type.string_count();

    // Determine tempo (use first tempo change or default)
    let base_tempo = midi_data.tempo_changes.first()
        .map(|(_, us_per_beat)| 60_000_000.0 / *us_per_beat as f64)
        .unwrap_or(120.0);

    doc.tempo_map = TempoMap::new(base_tempo);

    // Determine time signature (use first or default)
    let (ts_num, ts_denom) = midi_data.time_sig_changes.first()
        .map(|(_, n, d)| (*n, *d))
        .unwrap_or((4, 4));

    // Create track
    let track = create_track_from_options(options);
    let track_id = track.id;
    doc.tracks.push(track);

    // Calculate ticks per measure
    let ticks_per_beat = midi_data.ppqn as u32;
    let beats_per_measure = ts_num as u32;
    let ticks_per_measure = ticks_per_beat * beats_per_measure * 4 / ts_denom as u32;

    // Find total duration
    let max_tick = midi_data.notes.iter()
        .map(|n| n.tick + n.duration_ticks)
        .max()
        .unwrap_or(ticks_per_measure);

    let measure_count = (max_tick / ticks_per_measure) as usize + 1;

    // Create measures
    for measure_idx in 0..measure_count {
        let measure_start_tick = measure_idx as u32 * ticks_per_measure;
        let measure_end_tick = measure_start_tick + ticks_per_measure;

        let mut tab_measure = TabMeasure::new(measure_idx + 1);

        // Set time signature on first measure
        if measure_idx == 0 {
            tab_measure.time_signature = Some(TabTimeSignature::new(ts_num, ts_denom));
        }

        // Check for time signature changes
        if options.import_time_signatures {
            for (tick, num, denom) in &midi_data.time_sig_changes {
                if *tick >= measure_start_tick && *tick < measure_end_tick && measure_idx > 0 {
                    tab_measure.time_signature = Some(TabTimeSignature::new(*num, *denom));
                }
            }
        }

        // Check for tempo changes
        if options.import_tempo {
            for (tick, us_per_beat) in &midi_data.tempo_changes {
                if *tick >= measure_start_tick && *tick < measure_end_tick && measure_idx > 0 {
                    let bpm = 60_000_000.0 / *us_per_beat as f64;
                    tab_measure.tempo = Some(TempoChange {
                        bpm,
                        change_type: TempoChangeType::Immediate,
                    });
                }
            }
        }

        // Collect notes for this measure
        let measure_notes: Vec<&MidiNote> = midi_data.notes.iter()
            .filter(|n| n.tick >= measure_start_tick && n.tick < measure_end_tick)
            .filter(|n| n.velocity >= options.min_velocity)
            .collect();

        // Convert notes to beats
        let beats = convert_notes_to_beats(
            &measure_notes,
            measure_start_tick,
            ticks_per_beat,
            beats_per_measure as usize,
            &tuning,
            fret_count,
            string_count,
            options,
            &mut unassigned,
            &mut warnings,
        );

        tab_measure.track_beats.push(TrackMeasure {
            track_id,
            beats,
        });

        doc.measures.push(tab_measure);
    }

    // Import markers as sections
    if options.import_markers {
        for (tick, name) in &midi_data.markers {
            let measure_idx = (*tick / ticks_per_measure) as usize;
            if measure_idx < doc.measures.len() {
                doc.markers.push(SectionMarker::new(measure_idx, name));
            }
        }
    }

    // Set metadata
    if let Some(name) = midi_data.track_name {
        doc.metadata.title = name;
    }

    let note_count = midi_data.notes.len();

    Ok(MidiImportResult {
        document: doc,
        note_count,
        unassigned_notes: unassigned,
        detected_tempo: base_tempo,
        detected_time_sig: (ts_num, ts_denom),
        warnings,
    })
}

/// Create a TabTrack from import options
fn create_track_from_options(options: &MidiImportOptions) -> TabTrack {
    let config = match options.instrument_type {
        ImportInstrumentType::Guitar6Standard => StringedConfig::guitar_6_standard(),
        ImportInstrumentType::Guitar7Standard => StringedConfig::guitar_7_standard(),
        ImportInstrumentType::Guitar8Standard => StringedConfig::guitar_8_standard(),
        ImportInstrumentType::Bass4Standard => StringedConfig::bass_4_standard(),
        ImportInstrumentType::Bass5Standard => StringedConfig::bass_5_standard(),
        ImportInstrumentType::Custom { strings, tuning } => {
            let mut config = StringedConfig::guitar_6_standard();
            config.string_count = strings;
            config.tuning = tuning[..strings as usize].iter().map(|&n| n as u8).collect();
            config
        }
    };

    TabTrack {
        id: Uuid::new_v4(),
        name: "Imported Track".to_string(),
        instrument: Instrument::StringedInstrument(config),
        color: (100, 149, 237), // Cornflower blue
        volume: 1.0,
        pan: 0.0,
        muted: false,
        solo: false,
    }
}

/// Convert MIDI notes to tab beats
fn convert_notes_to_beats(
    notes: &[&MidiNote],
    measure_start_tick: u32,
    ticks_per_beat: u32,
    beats_per_measure: usize,
    tuning: &[u8],
    fret_count: u8,
    string_count: u8,
    options: &MidiImportOptions,
    unassigned: &mut usize,
    _warnings: &mut Vec<String>,
) -> Vec<TabBeat> {
    let mut beats = Vec::new();

    // Group notes by beat position
    let mut notes_by_beat: Vec<Vec<&MidiNote>> = vec![Vec::new(); beats_per_measure];

    for note in notes {
        let relative_tick = note.tick.saturating_sub(measure_start_tick);
        let beat_idx = (relative_tick / ticks_per_beat) as usize;
        if beat_idx < beats_per_measure {
            notes_by_beat[beat_idx].push(note);
        }
    }

    // Convert each beat
    for beat_notes in notes_by_beat {
        if beat_notes.is_empty() {
            // Rest
            beats.push(TabBeat::rest(RhythmValue::new(BaseDuration::Quarter)));
        } else {
            // Convert notes to fret positions
            let mut tab_notes = Vec::new();
            let mut used_strings = Vec::new();

            for midi_note in &beat_notes {
                if let Some((string, fret)) = find_fret_position(
                    midi_note.note,
                    tuning,
                    fret_count,
                    string_count,
                    &used_strings,
                    options.fret_preference,
                ) {
                    used_strings.push(string);
                    let mut tab_note = TabNote::new(string, fret);
                    tab_note.velocity = midi_note.velocity;
                    tab_notes.push(tab_note);
                } else {
                    *unassigned += 1;
                }
            }

            let rhythm = duration_ticks_to_rhythm(
                beat_notes.first().map(|n| n.duration_ticks).unwrap_or(ticks_per_beat),
                ticks_per_beat,
                options.quantize_to,
            );

            if tab_notes.is_empty() {
                beats.push(TabBeat::rest(rhythm));
            } else {
                beats.push(TabBeat {
                    id: Uuid::new_v4(),
                    notes: tab_notes,
                    rhythm,
                    is_rest: false,
                    effects: Default::default(),
                    text: None,
                    voice: 0,
                });
            }
        }
    }

    // Ensure we have at least the right number of beats
    while beats.len() < beats_per_measure {
        beats.push(TabBeat::rest(RhythmValue::new(BaseDuration::Quarter)));
    }

    beats
}

/// Find the best fret position for a MIDI note
fn find_fret_position(
    midi_note: u8,
    tuning: &[u8],
    fret_count: u8,
    _string_count: u8,
    used_strings: &[u8],
    preference: FretPreference,
) -> Option<(u8, u8)> {
    let mut candidates: Vec<(u8, u8, i32)> = Vec::new(); // (string, fret, score)

    for (string_idx, &open_note) in tuning.iter().enumerate() {
        let string = (string_idx + 1) as u8;

        if used_strings.contains(&string) {
            continue;
        }

        if midi_note >= open_note {
            let fret = midi_note - open_note;
            if fret <= fret_count {
                let score = calculate_fret_score(fret, preference);
                candidates.push((string, fret, score));
            }
        }
    }

    // Sort by score (lower is better)
    candidates.sort_by_key(|(_, _, score)| *score);

    candidates.first().map(|(s, f, _)| (*s, *f))
}

/// Calculate a score for fret preference (lower = better)
fn calculate_fret_score(fret: u8, preference: FretPreference) -> i32 {
    match preference {
        FretPreference::LowerFrets => fret as i32,
        FretPreference::HigherFrets => (24 - fret) as i32,
        FretPreference::MiddleFrets => ((fret as i32) - 7).abs(),
        FretPreference::MinimizeMovement => fret as i32, // Simple version; could track last position
    }
}

/// Convert duration in ticks to RhythmValue
fn duration_ticks_to_rhythm(
    ticks: u32,
    ticks_per_beat: u32,
    quantize: Option<BaseDuration>,
) -> RhythmValue {
    let ratio = ticks as f64 / ticks_per_beat as f64;

    let base = if ratio >= 3.5 {
        BaseDuration::Whole
    } else if ratio >= 1.75 {
        BaseDuration::Half
    } else if ratio >= 0.875 {
        BaseDuration::Quarter
    } else if ratio >= 0.4375 {
        BaseDuration::Eighth
    } else if ratio >= 0.21875 {
        BaseDuration::Sixteenth
    } else {
        BaseDuration::ThirtySecond
    };

    // Apply quantization if requested
    if let Some(quant) = quantize {
        if (base as u8) > (quant as u8) {
            return RhythmValue::new(quant);
        }
    }

    RhythmValue::new(base)
}

/// Check if a file is a MIDI file
pub fn is_midi_file<P: AsRef<Path>>(path: P) -> bool {
    let extension = path.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    matches!(extension.as_str(), "mid" | "midi" | "smf")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_fret_position() {
        let tuning = vec![64, 59, 55, 50, 45, 40]; // Standard guitar
        let used = vec![];

        // E4 (64) should be string 1, fret 0
        let result = find_fret_position(64, &tuning, 24, 6, &used, FretPreference::LowerFrets);
        assert_eq!(result, Some((1, 0)));

        // F4 (65) should be string 1, fret 1
        let result = find_fret_position(65, &tuning, 24, 6, &used, FretPreference::LowerFrets);
        assert_eq!(result, Some((1, 1)));

        // A2 (45) should be string 5, fret 0
        let result = find_fret_position(45, &tuning, 24, 6, &used, FretPreference::LowerFrets);
        assert_eq!(result, Some((5, 0)));
    }

    #[test]
    fn test_duration_to_rhythm() {
        let ticks_per_beat = 480;

        // Quarter note
        let rhythm = duration_ticks_to_rhythm(480, ticks_per_beat, None);
        assert_eq!(rhythm.base, BaseDuration::Quarter);

        // Eighth note
        let rhythm = duration_ticks_to_rhythm(240, ticks_per_beat, None);
        assert_eq!(rhythm.base, BaseDuration::Eighth);

        // Sixteenth note
        let rhythm = duration_ticks_to_rhythm(120, ticks_per_beat, None);
        assert_eq!(rhythm.base, BaseDuration::Sixteenth);
    }

    #[test]
    fn test_import_instrument_tuning() {
        let guitar = ImportInstrumentType::Guitar6Standard;
        assert_eq!(guitar.string_count(), 6);
        assert_eq!(guitar.tuning(), vec![64, 59, 55, 50, 45, 40]);

        let bass = ImportInstrumentType::Bass4Standard;
        assert_eq!(bass.string_count(), 4);
        assert_eq!(bass.tuning(), vec![43, 38, 33, 28]);
    }
}
