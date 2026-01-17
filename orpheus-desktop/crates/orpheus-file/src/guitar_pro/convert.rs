//! Convert Guitar Pro files to TabDocument format
//!
//! Bridges orpheus-file's GuitarProFile to orpheus-core's TabDocument.

use super::types::*;
use orpheus_core::tab::{
    self, ArpeggioDirection, BaseDuration, BendAmount, BendData, BendPoint as TabBendPoint,
    DrumKit, Fingering, GraceNoteData, GraceNoteDuration, GraceNoteTransition, Instrument,
    RhythmValue, SectionMarker, SlideDirection, StringedConfig, StringedType, TabBeat,
    TabDocument, TabMeasure, TabMetadata, TabNote, TabTrack, Technique, TempoChange,
    TempoChangeType, TempoMap, TimeSignature as TabTimeSignature, TrackMeasure, TrillData,
    TrillSpeed, VibratoStyle,
};
use tracing::debug;
use uuid::Uuid;

/// Convert a GuitarProFile to a TabDocument
pub fn convert_gp_to_tab(gp: &GuitarProFile) -> TabDocument {
    let mut doc = TabDocument::new();

    // Convert metadata
    doc.metadata = convert_metadata(&gp.info);

    // Set up tempo map
    doc.tempo_map = TempoMap::new(gp.tempo as f64);

    // Convert tracks
    for (i, track) in gp.tracks.iter().enumerate() {
        let tab_track = convert_track(track, i);
        doc.tracks.push(tab_track);
    }

    // Convert measures
    for (i, measure) in gp.measures.iter().enumerate() {
        let tab_measure = convert_measure(measure, i, &doc.tracks);
        doc.measures.push(tab_measure);

        // Add marker if present
        if let Some(ref marker) = measure.marker {
            doc.markers.push(SectionMarker::new(i, marker));
        }
    }

    debug!(
        "Converted GP file: {} tracks, {} measures",
        doc.tracks.len(),
        doc.measures.len()
    );

    doc
}

/// Convert song metadata
fn convert_metadata(info: &SongInfo) -> TabMetadata {
    TabMetadata {
        title: info.title.clone(),
        artist: info.artist.clone(),
        album: info.album.clone(),
        year: None,
        transcriber: info.tab_author.clone(),
        copyright: info.copyright.clone(),
        instructions: info.instructions.clone(),
        comments: info.comments.clone(),
        tags: Vec::new(),
        difficulty: 5, // Default mid difficulty
    }
}

/// Convert a GP track to TabTrack
fn convert_track(track: &Track, index: usize) -> TabTrack {
    let instrument = if track.is_drums {
        Instrument::Drums(DrumKit::standard())
    } else {
        // Create stringed instrument config
        let config = create_stringed_config(track);
        Instrument::StringedInstrument(config)
    };

    TabTrack {
        id: Uuid::new_v4(),
        name: if track.name.is_empty() {
            format!("Track {}", index + 1)
        } else {
            track.name.clone()
        },
        instrument,
        color: track.color,
        volume: track.volume as f32 / 127.0,
        pan: (track.pan as f32 - 64.0) / 64.0,
        muted: false,
        solo: false,
    }
}

/// Create a StringedConfig from GP track data
fn create_stringed_config(track: &Track) -> StringedConfig {
    // Detect instrument type based on string count and tuning
    let instrument_type = if track.strings <= 4 {
        StringedType::Bass
    } else {
        StringedType::ElectricGuitar
    };

    // Start with a base config and customize
    let mut config = match track.strings {
        4 => StringedConfig::bass_4_standard(),
        5 => StringedConfig::bass_5_standard(),
        6 => StringedConfig::guitar_6_standard(),
        7 => StringedConfig::guitar_7_standard(),
        8 => StringedConfig::guitar_8_standard(),
        _ => StringedConfig::guitar_6_standard(),
    };

    // Apply custom tuning if different from standard
    if !track.tuning.is_empty() {
        config.tuning = track.tuning.clone();
        config.string_count = track.strings;
    }

    // Apply capo
    config.capo = track.capo;
    config.instrument_type = instrument_type;

    config
}

/// Convert a GP measure to TabMeasure
fn convert_measure(measure: &Measure, index: usize, tracks: &[TabTrack]) -> TabMeasure {
    let mut tab_measure = TabMeasure::new(index + 1);

    // Convert time signature if present
    if let Some(ref ts) = measure.time_signature {
        tab_measure.time_signature = Some(TabTimeSignature::new(ts.numerator, ts.denominator));
    }

    // Convert tempo if present
    if let Some(tempo) = measure.tempo {
        tab_measure.tempo = Some(TempoChange {
            bpm: tempo as f64,
            change_type: TempoChangeType::Immediate,
        });
    }

    // Convert repeat markers
    if measure.repeat_start {
        tab_measure.repeat = tab::RepeatMarker::Start;
    } else if measure.repeat_end > 0 {
        tab_measure.repeat = tab::RepeatMarker::End(measure.repeat_end);
    }

    // Convert beats for each track
    for (i, track) in tracks.iter().enumerate() {
        let track_beats = measure
            .beats
            .iter()
            .find(|tb| tb.track as usize == i + 1)
            .map(|tb| convert_track_beats(tb, track))
            .unwrap_or_else(|| TrackMeasure {
                track_id: track.id,
                beats: vec![TabBeat::rest(RhythmValue::whole())],
            });
        tab_measure.track_beats.push(track_beats);
    }

    tab_measure
}

/// Convert GP track beats to TrackMeasure
fn convert_track_beats(track_beats: &TrackBeats, track: &TabTrack) -> TrackMeasure {
    let beats: Vec<TabBeat> = track_beats.beats.iter().map(convert_beat).collect();

    TrackMeasure {
        track_id: track.id,
        beats: if beats.is_empty() {
            vec![TabBeat::rest(RhythmValue::whole())]
        } else {
            beats
        },
    }
}

/// Convert a GP beat to TabBeat
fn convert_beat(beat: &Beat) -> TabBeat {
    let rhythm = convert_duration(beat.duration, beat.dotted, beat.tuplet);

    if beat.is_rest {
        return TabBeat::rest(rhythm);
    }

    let notes: Vec<TabNote> = beat.notes.iter().map(convert_note).collect();

    let mut effects = tab::BeatEffect::default();

    // Convert stroke to arpeggio
    if let Some(ref stroke) = beat.effects.stroke {
        effects.arpeggio = Some(match stroke {
            Stroke::Down(_) => ArpeggioDirection::Down,
            Stroke::Up(_) => ArpeggioDirection::Up,
        });
    }

    TabBeat {
        id: Uuid::new_v4(),
        notes,
        rhythm,
        is_rest: false,
        effects,
        text: beat.text.clone(),
        voice: 0,
    }
}

/// Convert GP duration to RhythmValue
fn convert_duration(duration: Duration, dotted: bool, tuplet: Option<u8>) -> RhythmValue {
    let base = match duration {
        Duration::Whole => BaseDuration::Whole,
        Duration::Half => BaseDuration::Half,
        Duration::Quarter => BaseDuration::Quarter,
        Duration::Eighth => BaseDuration::Eighth,
        Duration::Sixteenth => BaseDuration::Sixteenth,
        Duration::ThirtySecond => BaseDuration::ThirtySecond,
        Duration::SixtyFourth => BaseDuration::SixtyFourth,
    };

    let mut rhythm = RhythmValue::new(base);

    if dotted {
        rhythm.dots = 1;
    }

    if let Some(3) = tuplet {
        rhythm = rhythm.triplet();
    }

    rhythm
}

/// Convert a GP note to TabNote
fn convert_note(note: &Note) -> TabNote {
    let mut tab_note = TabNote::new(note.string, note.fret);
    tab_note.velocity = note.velocity;
    tab_note.tied = note.tied;
    tab_note.ghost = note.ghost;

    // Convert note effects to techniques
    let effects = &note.effects;

    if effects.hammer_on {
        tab_note.techniques.push(Technique::HammerOn);
    }

    if effects.pull_off {
        tab_note.techniques.push(Technique::PullOff);
    }

    if effects.vibrato {
        tab_note.techniques.push(Technique::Vibrato(VibratoStyle::Standard));
    }

    // Convert slide
    if let Some(ref slide) = effects.slide {
        let dir = match slide {
            SlideType::IntoFromAbove | SlideType::OutDownwards => SlideDirection::Down,
            SlideType::IntoFromBelow | SlideType::OutUpwards => SlideDirection::Up,
            SlideType::ShiftSlide => SlideDirection::Up,
            SlideType::LegatoSlide => SlideDirection::Up,
        };
        match slide {
            SlideType::LegatoSlide => tab_note.techniques.push(Technique::LegatoSlide(dir)),
            SlideType::ShiftSlide => tab_note.techniques.push(Technique::ShiftSlide(dir)),
            SlideType::IntoFromAbove | SlideType::IntoFromBelow => {
                tab_note.techniques.push(Technique::SlideIn(dir))
            }
            SlideType::OutDownwards | SlideType::OutUpwards => {
                tab_note.techniques.push(Technique::SlideOut(dir))
            }
        }
    }

    // Convert bend
    if let Some(ref bend_points) = effects.bend {
        if !bend_points.is_empty() {
            let max_bend = bend_points.iter().map(|p| p.value.abs()).max().unwrap_or(0);
            let amount = gp_bend_to_amount(max_bend);

            let curve: Vec<TabBendPoint> = bend_points
                .iter()
                .map(|p| TabBendPoint {
                    position: p.position as f32 / 60.0,
                    amount: gp_bend_to_amount(p.value.abs()),
                })
                .collect();

            tab_note.techniques.push(Technique::Bend(BendData {
                amount,
                curve,
                hold: false,
                release: bend_points.last().map(|p| p.value == 0).unwrap_or(false),
            }));
        }
    }

    // Convert harmonic
    if let Some(ref harmonic) = effects.harmonic {
        match harmonic {
            HarmonicType::Natural => tab_note.techniques.push(Technique::NaturalHarmonic),
            HarmonicType::Pinch => tab_note.techniques.push(Technique::PinchHarmonic),
            HarmonicType::Artificial => {
                tab_note.techniques.push(Technique::ArtificialHarmonic(12))
            }
            HarmonicType::Tap => tab_note.techniques.push(Technique::TapHarmonic(note.fret + 12)),
            HarmonicType::Semi => tab_note.techniques.push(Technique::SemiHarmonic),
            HarmonicType::Feedback => tab_note.techniques.push(Technique::FeedbackHarmonic),
        }
    }

    // Convert trill
    if let Some((trill_fret, speed)) = effects.trill {
        let trill_speed = match speed {
            Duration::Sixteenth => TrillSpeed::Sixteenth,
            Duration::ThirtySecond => TrillSpeed::ThirtySecond,
            Duration::SixtyFourth => TrillSpeed::SixtyFourth,
            _ => TrillSpeed::Sixteenth,
        };
        tab_note.techniques.push(Technique::Trill(TrillData {
            upper_fret: trill_fret,
            speed: trill_speed,
        }));
    }

    // Convert grace note
    if let Some(ref grace) = effects.grace_note {
        tab_note.techniques.push(Technique::GraceNote(GraceNoteData {
            fret: grace.fret,
            duration: GraceNoteDuration::ThirtySecond,
            on_beat: grace.on_beat,
            transition: GraceNoteTransition::None,
            velocity: grace.velocity,
        }));
    }

    // Convert fingering
    if let Some(ref finger) = effects.left_hand_finger {
        tab_note.left_finger = Some(convert_finger(finger));
    }
    if let Some(ref finger) = effects.right_hand_finger {
        tab_note.right_finger = Some(convert_finger(finger));
    }

    tab_note
}

/// Convert GP bend value (quarter tones, 100 = semitone) to BendAmount
fn gp_bend_to_amount(value: i16) -> BendAmount {
    match value {
        0..=50 => BendAmount::Quarter,
        51..=100 => BendAmount::Half,
        101..=200 => BendAmount::Full,
        201..=300 => BendAmount::OneAndHalf,
        301..=400 => BendAmount::Two,
        401..=500 => BendAmount::TwoAndHalf,
        _ => BendAmount::Three,
    }
}

/// Convert GP finger to orpheus Fingering
fn convert_finger(finger: &Finger) -> Fingering {
    match finger {
        Finger::Open => Fingering::Open,
        Finger::Thumb => Fingering::Thumb,
        Finger::Index => Fingering::Index,
        Finger::Middle => Fingering::Middle,
        Finger::Ring => Fingering::Ring,
        Finger::Pinky => Fingering::Pinky,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_empty_gp() {
        let gp = GuitarProFile::default();
        let doc = convert_gp_to_tab(&gp);
        assert!(doc.tracks.is_empty());
        assert!(doc.measures.is_empty());
    }

    #[test]
    fn test_convert_metadata() {
        let mut gp = GuitarProFile::default();
        gp.info.title = "Test Song".to_string();
        gp.info.artist = "Test Artist".to_string();
        gp.tempo = 140;

        let doc = convert_gp_to_tab(&gp);
        assert_eq!(doc.metadata.title, "Test Song");
        assert_eq!(doc.metadata.artist, "Test Artist");
    }

    #[test]
    fn test_convert_duration() {
        let rhythm = convert_duration(Duration::Quarter, false, None);
        assert_eq!(rhythm.base, BaseDuration::Quarter);
        assert_eq!(rhythm.dots, 0);

        let dotted = convert_duration(Duration::Eighth, true, None);
        assert_eq!(dotted.base, BaseDuration::Eighth);
        assert_eq!(dotted.dots, 1);

        let triplet = convert_duration(Duration::Sixteenth, false, Some(3));
        assert!(triplet.tuplet.is_some());
    }

    #[test]
    fn test_bend_conversion() {
        assert!(matches!(gp_bend_to_amount(50), BendAmount::Quarter));
        assert!(matches!(gp_bend_to_amount(100), BendAmount::Half));
        assert!(matches!(gp_bend_to_amount(200), BendAmount::Full));
        assert!(matches!(gp_bend_to_amount(300), BendAmount::OneAndHalf));
    }
}
