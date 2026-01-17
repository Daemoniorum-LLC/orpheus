//! Demo audio rendering example
//!
//! Renders various demo sequences to WAV files showcasing the synthesizers.
//!
//! Usage:
//!   cargo run --example render_demo
//!
//! Output files will be created in the current directory.

use orpheus_synth::{
    instrument_router::{InstrumentType, MultiInstrumentEngine},
    orchestra_renderer::OrchestraRenderer,
    sequencer::create_test_sequence,
    wav::write_wav,
    guitar::GuitarConfig,
    STANDARD_TUNING, DROP_D_TUNING,
};

const SAMPLE_RATE: f32 = 44100.0;

fn main() {
    println!("=== Orpheus Synth Demo Renderer ===\n");

    // Render all demos
    render_guitar_arpeggio();
    render_metal_riff();
    render_orchestra();
    render_rock_band();
    render_jazz_combo();
    render_choir();

    println!("\n=== All demos rendered successfully! ===");
}

/// Render a clean guitar arpeggio
fn render_guitar_arpeggio() {
    println!("Rendering: guitar_arpeggio.wav");

    let mut engine = MultiInstrumentEngine::new(SAMPLE_RATE);
    engine.add_track(1, "Clean Guitar".to_string(), InstrumentType::AcousticGuitar(GuitarConfig::default()));

    let events = create_test_sequence();
    let tempo = 120.0;
    let seconds_per_beat = 60.0 / tempo;
    let samples_per_beat = (SAMPLE_RATE * seconds_per_beat) as usize;

    // Schedule all notes
    let mut all_left = Vec::new();
    let mut all_right = Vec::new();

    for event in &events {
        let midi_note = string_fret_to_midi(event.string, event.fret, &STANDARD_TUNING);
        engine.note_on_by_id(1, midi_note, event.velocity);

        // Render this beat
        let samples = (event.duration_beats * samples_per_beat as f64) as usize;
        let (left, right) = engine.process(samples);
        all_left.extend(left);
        all_right.extend(right);

        engine.note_off_by_id(1, midi_note);
    }

    // Add decay tail
    let (tail_l, tail_r) = engine.process(SAMPLE_RATE as usize * 2);
    all_left.extend(tail_l);
    all_right.extend(tail_r);

    write_wav("guitar_arpeggio.wav", &all_left, &all_right, SAMPLE_RATE as u32).unwrap();
    println!("  -> {} samples ({:.2}s)", all_left.len(), all_left.len() as f32 / SAMPLE_RATE);
}

/// Render a heavy metal riff in Drop D
fn render_metal_riff() {
    println!("Rendering: metal_riff.wav");

    let mut engine = MultiInstrumentEngine::new(SAMPLE_RATE);
    engine.add_track(1, "Rhythm Guitar".to_string(), InstrumentType::ElectricGuitar(GuitarConfig::default()));
    engine.add_track(2, "Bass".to_string(), InstrumentType::ElectricBass);
    engine.add_track(3, "Drums".to_string(), InstrumentType::DrumKit);

    let tempo = 140.0;
    let beat_samples = (SAMPLE_RATE * 60.0 / tempo) as usize;

    let mut all_left = Vec::new();
    let mut all_right = Vec::new();

    // Drop D power chord riff pattern (4 bars)
    let riff: Vec<(u8, u8, f32)> = vec![
        // Bar 1: D5 power chord pattern
        (6, 0, 0.9), (6, 0, 0.7), (6, 0, 0.8), (6, 0, 0.6),
        // Bar 2: E5 to D5
        (6, 2, 0.9), (6, 2, 0.7), (6, 0, 0.8), (6, 0, 0.6),
        // Bar 3: F5 to E5
        (6, 3, 0.9), (6, 3, 0.7), (6, 2, 0.8), (6, 2, 0.6),
        // Bar 4: G5 to F5 to D5
        (6, 5, 0.9), (6, 3, 0.7), (6, 0, 0.9), (6, 0, 0.7),
    ];

    for (i, (string, fret, velocity)) in riff.iter().enumerate() {
        let guitar_note = string_fret_to_midi(*string, *fret, &DROP_D_TUNING);
        let bass_note = guitar_note - 12; // Octave lower

        // Play guitar and bass
        engine.note_on_by_id(1, guitar_note, *velocity);
        engine.note_on_by_id(1, guitar_note + 7, *velocity * 0.8); // Fifth
        engine.note_on_by_id(2, bass_note, *velocity);

        // Drums: kick on 1 and 3, snare on 2 and 4
        if i % 4 == 0 || i % 4 == 2 {
            engine.note_on_by_id(3, 36, 0.9); // Kick
        }
        if i % 4 == 1 || i % 4 == 3 {
            engine.note_on_by_id(3, 38, 0.85); // Snare
        }
        engine.note_on_by_id(3, 42, 0.6); // Hi-hat

        // Render eighth note
        let (left, right) = engine.process(beat_samples / 2);
        all_left.extend(left);
        all_right.extend(right);

        // Note offs
        engine.note_off_by_id(1, guitar_note);
        engine.note_off_by_id(1, guitar_note + 7);
        engine.note_off_by_id(2, bass_note);
    }

    // Decay tail
    let (tail_l, tail_r) = engine.process(SAMPLE_RATE as usize);
    all_left.extend(tail_l);
    all_right.extend(tail_r);

    write_wav("metal_riff.wav", &all_left, &all_right, SAMPLE_RATE as u32).unwrap();
    println!("  -> {} samples ({:.2}s)", all_left.len(), all_left.len() as f32 / SAMPLE_RATE);
}

/// Render a full orchestra with spatial audio
fn render_orchestra() {
    println!("Rendering: orchestra.wav");

    let mut renderer = OrchestraRenderer::symphony(SAMPLE_RATE);

    // Add orchestra sections
    renderer.add_track(1, "Violin I".to_string(), InstrumentType::Violin);
    renderer.add_track(2, "Violin II".to_string(), InstrumentType::Violin);
    renderer.add_track(3, "Viola".to_string(), InstrumentType::Viola);
    renderer.add_track(4, "Cello".to_string(), InstrumentType::Cello);
    renderer.add_track(5, "Bass".to_string(), InstrumentType::DoubleBass);
    renderer.add_track(6, "Flute".to_string(), InstrumentType::Flute);
    renderer.add_track(7, "Oboe".to_string(), InstrumentType::Oboe);
    renderer.add_track(8, "French Horn".to_string(), InstrumentType::FrenchHorn);
    renderer.add_track(9, "Trumpet".to_string(), InstrumentType::Trumpet);
    renderer.add_track(10, "Timpani".to_string(), InstrumentType::Timpani);

    let beat_samples = (SAMPLE_RATE * 0.5) as usize; // Quarter note at 120 BPM
    let mut all_left = Vec::new();
    let mut all_right = Vec::new();

    // C major chord progression: C - G - Am - F
    // Each chord: (strings tuple, bass root, flute note, oboe note)
    let chords: [(u8, u8, u8, u8, u8, u8, u8, u8); 4] = [
        // C major: strings (Vln1, Vln2, Vla, Cello, Bass), bass, flute, oboe
        (76, 72, 67, 60, 48, 48, 72, 67),
        // G major
        (74, 71, 67, 59, 43, 43, 74, 67),
        // A minor
        (72, 69, 64, 57, 45, 45, 72, 64),
        // F major
        (69, 65, 60, 53, 41, 41, 69, 65),
    ];

    for (vln1, vln2, vla, cello, bass, bass_root, flute, oboe) in &chords {
        // Strings play chord
        renderer.note_on(1, *vln1, 0.6);
        renderer.note_on(2, *vln2, 0.6);
        renderer.note_on(3, *vla, 0.6);
        renderer.note_on(4, *cello, 0.6);
        renderer.note_on(5, *bass, 0.7);

        // Woodwinds play melody
        renderer.note_on(6, *flute, 0.5);
        renderer.note_on(7, *oboe, 0.4);

        // Brass accents on downbeat
        renderer.note_on(8, *vla, 0.4);
        renderer.note_on(9, *vln1, 0.3);

        // Timpani on bass note
        renderer.note_on(10, *bass_root, 0.6);

        // Render whole note (4 beats)
        let (left, right) = renderer.process(beat_samples * 4);
        all_left.extend(left);
        all_right.extend(right);

        // Release all notes
        for i in 1..=10 {
            renderer.note_off(i as u32, 0); // Note off all
        }
    }

    // Long reverb tail
    let (tail_l, tail_r) = renderer.process(SAMPLE_RATE as usize * 3);
    all_left.extend(tail_l);
    all_right.extend(tail_r);

    write_wav("orchestra.wav", &all_left, &all_right, SAMPLE_RATE as u32).unwrap();
    println!("  -> {} samples ({:.2}s)", all_left.len(), all_left.len() as f32 / SAMPLE_RATE);
}

/// Render a rock band arrangement
fn render_rock_band() {
    println!("Rendering: rock_band.wav");

    let mut renderer = OrchestraRenderer::studio(SAMPLE_RATE);

    renderer.add_track(1, "Lead Guitar".to_string(), InstrumentType::ElectricGuitar(GuitarConfig::default()));
    renderer.add_track(2, "Rhythm Guitar".to_string(), InstrumentType::ElectricGuitar(GuitarConfig::default()));
    renderer.add_track(3, "Bass".to_string(), InstrumentType::ElectricBass);
    renderer.add_track(4, "Drums".to_string(), InstrumentType::DrumKit);
    renderer.add_track(5, "Keys".to_string(), InstrumentType::ElectricPiano);

    let tempo = 120.0;
    let beat_samples = (SAMPLE_RATE * 60.0 / tempo) as usize;
    let mut all_left = Vec::new();
    let mut all_right = Vec::new();

    // 8-bar rock progression
    let progression = [
        (64, 52, 40), // E
        (64, 52, 40), // E
        (69, 57, 45), // A
        (69, 57, 45), // A
        (71, 59, 47), // B
        (69, 57, 45), // A
        (64, 52, 40), // E
        (71, 59, 47), // B
    ];

    for (bar, (lead, rhythm, bass)) in progression.iter().enumerate() {
        // Lead melody (arpeggiate)
        for beat in 0..4 {
            let melody_note = *lead + [0, 4, 7, 12][beat % 4] as u8;
            renderer.note_on(1, melody_note, 0.7);

            // Rhythm guitar (power chord)
            if beat == 0 {
                renderer.note_on(2, *rhythm, 0.8);
                renderer.note_on(2, *rhythm + 7, 0.7);
            }

            // Bass on 1 and 3
            if beat == 0 || beat == 2 {
                renderer.note_on(3, *bass, 0.8);
            }

            // Drums
            if beat == 0 || beat == 2 {
                renderer.note_on(4, 36, 0.9); // Kick
            }
            if beat == 1 || beat == 3 {
                renderer.note_on(4, 38, 0.85); // Snare
            }
            renderer.note_on(4, 42, 0.5); // Hi-hat

            // Keys pad (on downbeat)
            if beat == 0 {
                renderer.note_on(5, *rhythm, 0.3);
                renderer.note_on(5, *rhythm + 4, 0.3);
                renderer.note_on(5, *rhythm + 7, 0.3);
            }

            let (left, right) = renderer.process(beat_samples);
            all_left.extend(left);
            all_right.extend(right);

            renderer.note_off(1, melody_note);
        }

        // Release sustained notes at bar end
        renderer.note_off(2, *rhythm);
        renderer.note_off(2, *rhythm + 7);
        renderer.note_off(3, *bass);
        renderer.note_off(5, *rhythm);
        renderer.note_off(5, *rhythm + 4);
        renderer.note_off(5, *rhythm + 7);
    }

    // Tail
    let (tail_l, tail_r) = renderer.process(SAMPLE_RATE as usize * 2);
    all_left.extend(tail_l);
    all_right.extend(tail_r);

    write_wav("rock_band.wav", &all_left, &all_right, SAMPLE_RATE as u32).unwrap();
    println!("  -> {} samples ({:.2}s)", all_left.len(), all_left.len() as f32 / SAMPLE_RATE);
}

/// Render a jazz combo
fn render_jazz_combo() {
    println!("Rendering: jazz_combo.wav");

    let mut renderer = OrchestraRenderer::jazz_club(SAMPLE_RATE);

    renderer.add_track(1, "Piano".to_string(), InstrumentType::GrandPiano);
    renderer.add_track(2, "Upright Bass".to_string(), InstrumentType::AcousticBass);
    renderer.add_track(3, "Drums".to_string(), InstrumentType::DrumKit);
    renderer.add_track(4, "Saxophone".to_string(), InstrumentType::Saxophone(orpheus_synth::instrument_router::SaxType::Tenor));

    let tempo = 140.0; // Swing tempo
    let beat_samples = (SAMPLE_RATE * 60.0 / tempo) as usize;
    let mut all_left = Vec::new();
    let mut all_right = Vec::new();

    // ii-V-I progression in C (Dm7 - G7 - Cmaj7)
    let chords = [
        // Dm7: D F A C
        ([62, 65, 69, 72], 50, 74),
        // G7: G B D F
        ([67, 71, 74, 77], 43, 79),
        // Cmaj7: C E G B
        ([60, 64, 67, 71], 48, 76),
        // Cmaj7 (resolve)
        ([60, 64, 67, 71], 48, 72),
    ];

    for (chord, bass_root, sax_note) in &chords {
        // Piano voicing
        for &note in chord {
            renderer.note_on(1, note, 0.5);
        }

        // Walking bass (root, fifth, approach)
        let bass_notes = [*bass_root, *bass_root + 7, *bass_root + 5, *bass_root + 7];

        // Sax melody
        renderer.note_on(4, *sax_note, 0.6);

        for (beat, &bass_note) in bass_notes.iter().enumerate() {
            renderer.note_on(2, bass_note, 0.7);

            // Swing ride cymbal
            renderer.note_on(3, 51, 0.4); // Ride
            if beat == 1 || beat == 3 {
                renderer.note_on(3, 51, 0.3); // Swing upbeat
            }

            let (left, right) = renderer.process(beat_samples);
            all_left.extend(left);
            all_right.extend(right);

            renderer.note_off(2, bass_note);
        }

        // Release chord
        for &note in chord {
            renderer.note_off(1, note);
        }
        renderer.note_off(4, *sax_note);
    }

    // Tail
    let (tail_l, tail_r) = renderer.process(SAMPLE_RATE as usize * 2);
    all_left.extend(tail_l);
    all_right.extend(tail_r);

    write_wav("jazz_combo.wav", &all_left, &all_right, SAMPLE_RATE as u32).unwrap();
    println!("  -> {} samples ({:.2}s)", all_left.len(), all_left.len() as f32 / SAMPLE_RATE);
}

/// Render a choir
fn render_choir() {
    println!("Rendering: choir.wav");

    let mut renderer = OrchestraRenderer::symphony(SAMPLE_RATE);

    renderer.add_track(1, "Soprano".to_string(), InstrumentType::SopranoVoice);
    renderer.add_track(2, "Alto".to_string(), InstrumentType::AltoVoice);
    renderer.add_track(3, "Tenor".to_string(), InstrumentType::TenorVoice);
    renderer.add_track(4, "Bass".to_string(), InstrumentType::BassVoice);

    renderer.set_reverb_mix(0.4); // More reverb for choir

    let beat_samples = (SAMPLE_RATE * 0.75) as usize; // Slow tempo
    let mut all_left = Vec::new();
    let mut all_right = Vec::new();

    // Simple hymn-like progression
    let chords = [
        // C major
        (72, 67, 60, 48),
        // F major
        (72, 69, 60, 53),
        // G major
        (71, 67, 59, 43),
        // C major
        (72, 64, 60, 48),
    ];

    for (soprano, alto, tenor, bass) in &chords {
        renderer.note_on(1, *soprano, 0.6);
        renderer.note_on(2, *alto, 0.6);
        renderer.note_on(3, *tenor, 0.6);
        renderer.note_on(4, *bass, 0.6);

        // Hold for 4 beats
        let (left, right) = renderer.process(beat_samples * 4);
        all_left.extend(left);
        all_right.extend(right);

        renderer.note_off(1, *soprano);
        renderer.note_off(2, *alto);
        renderer.note_off(3, *tenor);
        renderer.note_off(4, *bass);

        // Brief pause between chords
        let (left, right) = renderer.process(beat_samples / 4);
        all_left.extend(left);
        all_right.extend(right);
    }

    // Long reverb tail for choir
    let (tail_l, tail_r) = renderer.process(SAMPLE_RATE as usize * 4);
    all_left.extend(tail_l);
    all_right.extend(tail_r);

    write_wav("choir.wav", &all_left, &all_right, SAMPLE_RATE as u32).unwrap();
    println!("  -> {} samples ({:.2}s)", all_left.len(), all_left.len() as f32 / SAMPLE_RATE);
}

/// Convert string/fret to MIDI note
fn string_fret_to_midi(string: u8, fret: u8, tuning: &[u8]) -> u8 {
    let string_idx = (string - 1) as usize;
    if string_idx < tuning.len() {
        tuning[tuning.len() - 1 - string_idx] + fret
    } else {
        60 // Default to middle C
    }
}
