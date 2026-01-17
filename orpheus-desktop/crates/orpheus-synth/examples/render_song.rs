//! Render a complete song demonstrating all synth capabilities
//!
//! Renders "Ode to Code" - a multi-section piece showcasing instruments
//!
//! Usage:
//!   cargo run --example render_song
//!
//! Output: ode_to_code.wav

use orpheus_synth::{
    instrument_router::{InstrumentType, SaxType},
    orchestra_renderer::OrchestraRenderer,
    wav::write_wav,
    guitar::GuitarConfig,
    HallType,
};

const SAMPLE_RATE: f32 = 44100.0;

fn main() {
    println!("=== Rendering: Ode to Code ===\n");
    println!("A multi-section piece demonstrating Orpheus synthesizers\n");

    let mut all_left = Vec::new();
    let mut all_right = Vec::new();

    // Section 1: Orchestral Introduction
    println!("Section 1: Orchestral Introduction...");
    let (l, r) = render_orchestral_intro();
    all_left.extend(l);
    all_right.extend(r);

    // Section 2: Rock Band verse
    println!("Section 2: Rock Verse...");
    let (l, r) = render_rock_verse();
    all_left.extend(l);
    all_right.extend(r);

    // Section 3: Jazz Interlude
    println!("Section 3: Jazz Interlude...");
    let (l, r) = render_jazz_section();
    all_left.extend(l);
    all_right.extend(r);

    // Section 4: Metal Breakdown
    println!("Section 4: Metal Breakdown...");
    let (l, r) = render_metal_breakdown();
    all_left.extend(l);
    all_right.extend(r);

    // Section 5: Choir Finale
    println!("Section 5: Choir Finale...");
    let (l, r) = render_choir_finale();
    all_left.extend(l);
    all_right.extend(r);

    // Write the complete song
    write_wav("ode_to_code.wav", &all_left, &all_right, SAMPLE_RATE as u32).unwrap();

    let duration = all_left.len() as f32 / SAMPLE_RATE;
    println!("\n=== Complete! ===");
    println!("Duration: {:.1}s", duration);
    println!("Output: ode_to_code.wav ({:.1} MB)", (all_left.len() * 4) as f32 / 1_000_000.0);
}

/// Section 1: Orchestral strings and brass introduction
fn render_orchestral_intro() -> (Vec<f32>, Vec<f32>) {
    let mut renderer = OrchestraRenderer::symphony(SAMPLE_RATE);
    renderer.set_reverb_mix(0.35);

    // Full string section
    renderer.add_track(1, "Violin I".to_string(), InstrumentType::Violin);
    renderer.add_track(2, "Violin II".to_string(), InstrumentType::Violin);
    renderer.add_track(3, "Viola".to_string(), InstrumentType::Viola);
    renderer.add_track(4, "Cello".to_string(), InstrumentType::Cello);
    renderer.add_track(5, "Bass".to_string(), InstrumentType::DoubleBass);
    renderer.add_track(6, "Horn".to_string(), InstrumentType::FrenchHorn);
    renderer.add_track(7, "Timpani".to_string(), InstrumentType::Timpani);

    let beat = (SAMPLE_RATE * 0.5) as usize; // 120 BPM
    let mut left = Vec::new();
    let mut right = Vec::new();

    // Intro: Sustained C major building up
    // Bar 1-2: Strings enter pp
    let c_chord = [(72, 0.3), (67, 0.3), (64, 0.3), (60, 0.4), (48, 0.5)];
    for (i, &(note, vel)) in c_chord.iter().enumerate() {
        renderer.note_on((i + 1) as u32, note, vel);
    }
    extend_audio(&mut left, &mut right, &mut renderer, beat * 8);

    // Bar 3-4: Add horn, crescendo
    renderer.note_on(6, 60, 0.4); // Horn C4
    for i in 1..=5 {
        // Crescendo strings
        let new_vel = c_chord[i - 1].1 + 0.2;
        renderer.note_on(i as u32, c_chord[i - 1].0, new_vel.min(0.8));
    }
    extend_audio(&mut left, &mut right, &mut renderer, beat * 8);

    // Bar 5-6: Move to G major
    release_all(&mut renderer, 7);
    let g_chord = [(71, 0.6), (67, 0.6), (62, 0.6), (59, 0.7), (43, 0.8)];
    for (i, &(note, vel)) in g_chord.iter().enumerate() {
        renderer.note_on((i + 1) as u32, note, vel);
    }
    renderer.note_on(6, 67, 0.5); // Horn G4
    renderer.note_on(7, 43, 0.6); // Timpani G
    extend_audio(&mut left, &mut right, &mut renderer, beat * 8);

    // Bar 7-8: Resolve to C with timpani roll
    release_all(&mut renderer, 7);
    for (i, &(note, vel)) in c_chord.iter().enumerate() {
        renderer.note_on((i + 1) as u32, note, vel + 0.2);
    }
    renderer.note_on(6, 60, 0.6);
    renderer.note_on(7, 48, 0.7);
    extend_audio(&mut left, &mut right, &mut renderer, beat * 8);

    // Decay
    release_all(&mut renderer, 7);
    extend_audio(&mut left, &mut right, &mut renderer, beat * 4);

    (left, right)
}

/// Section 2: Rock band verse
fn render_rock_verse() -> (Vec<f32>, Vec<f32>) {
    let mut renderer = OrchestraRenderer::studio(SAMPLE_RATE);
    renderer.set_reverb_mix(0.2);

    renderer.add_track(1, "Lead".to_string(), InstrumentType::ElectricGuitar(GuitarConfig::default()));
    renderer.add_track(2, "Rhythm".to_string(), InstrumentType::ElectricGuitar(GuitarConfig::default()));
    renderer.add_track(3, "Bass".to_string(), InstrumentType::ElectricBass);
    renderer.add_track(4, "Drums".to_string(), InstrumentType::DrumKit);
    renderer.add_track(5, "Keys".to_string(), InstrumentType::ElectricPiano);

    let beat = (SAMPLE_RATE * 60.0 / 130.0) as usize; // 130 BPM
    let eighth = beat / 2;
    let mut left = Vec::new();
    let mut right = Vec::new();

    // Rock riff in E minor
    let riff = [
        (64, 52, 40), // Em
        (64, 52, 40),
        (67, 55, 43), // G
        (67, 55, 43),
        (69, 57, 45), // A
        (67, 55, 43), // G
        (64, 52, 40), // Em
        (62, 50, 38), // D
    ];

    for _ in 0..2 { // 2 times through
        for (bar, (lead, rhythm, bass)) in riff.iter().enumerate() {
            // Rhythm guitar power chord
            renderer.note_on(2, *rhythm, 0.8);
            renderer.note_on(2, *rhythm + 7, 0.7);

            // Bass follows root
            renderer.note_on(3, *bass, 0.85);

            // Lead plays melody on alternating beats
            if bar % 2 == 0 {
                renderer.note_on(1, *lead, 0.7);
            } else {
                renderer.note_on(1, *lead + 3, 0.65);
            }

            // Drums - standard rock beat
            for beat_num in 0..2 {
                if beat_num == 0 {
                    renderer.note_on(4, 36, 0.9); // Kick
                }
                renderer.note_on(4, 42, 0.5); // Hi-hat

                extend_audio(&mut left, &mut right, &mut renderer, eighth);
                renderer.note_off(4, 42);

                if beat_num == 1 {
                    renderer.note_on(4, 38, 0.85); // Snare
                }
                renderer.note_on(4, 42, 0.4);
                extend_audio(&mut left, &mut right, &mut renderer, eighth);
                renderer.note_off(4, 42);
                renderer.note_off(4, 38);
            }

            // Release
            renderer.note_off(1, *lead);
            renderer.note_off(1, *lead + 3);
            renderer.note_off(2, *rhythm);
            renderer.note_off(2, *rhythm + 7);
            renderer.note_off(3, *bass);
        }
    }

    // Tail
    extend_audio(&mut left, &mut right, &mut renderer, beat * 2);

    (left, right)
}

/// Section 3: Jazz interlude
fn render_jazz_section() -> (Vec<f32>, Vec<f32>) {
    let mut renderer = OrchestraRenderer::jazz_club(SAMPLE_RATE);

    renderer.add_track(1, "Piano".to_string(), InstrumentType::GrandPiano);
    renderer.add_track(2, "Bass".to_string(), InstrumentType::AcousticBass);
    renderer.add_track(3, "Drums".to_string(), InstrumentType::DrumKit);
    renderer.add_track(4, "Sax".to_string(), InstrumentType::Saxophone(SaxType::Tenor));

    let beat = (SAMPLE_RATE * 60.0 / 140.0) as usize; // 140 BPM swing
    let mut left = Vec::new();
    let mut right = Vec::new();

    // ii-V-I in C major (Dm7 - G7 - Cmaj7)
    let changes = [
        // Dm7: D F A C
        ([62, 65, 69, 72], 50, [74, 77, 74, 72]),
        // G7: G B D F
        ([67, 71, 74, 77], 43, [79, 77, 74, 71]),
        // Cmaj7: C E G B
        ([60, 64, 67, 71], 48, [72, 76, 79, 76]),
        // Cmaj7 (turnaround)
        ([60, 64, 67, 71], 48, [72, 74, 72, 71]),
    ];

    for (chord, bass_root, melody) in &changes {
        // Piano comps
        for &note in chord {
            renderer.note_on(1, note, 0.5);
        }

        // Walking bass line
        let bass_line = [*bass_root, *bass_root + 4, *bass_root + 7, *bass_root + 5];

        for (beat_num, (&bass_note, &sax_note)) in bass_line.iter().zip(melody.iter()).enumerate() {
            renderer.note_on(2, bass_note, 0.7);
            renderer.note_on(4, sax_note, 0.55);

            // Swing ride cymbal
            renderer.note_on(3, 51, 0.35); // Ride
            extend_audio(&mut left, &mut right, &mut renderer, beat * 2 / 3);
            renderer.note_off(3, 51);

            renderer.note_on(3, 51, 0.25); // Swing upbeat
            extend_audio(&mut left, &mut right, &mut renderer, beat / 3);
            renderer.note_off(3, 51);

            renderer.note_off(2, bass_note);
            renderer.note_off(4, sax_note);
        }

        // Release piano chord
        for &note in chord {
            renderer.note_off(1, note);
        }
    }

    // Tail with ride decay
    extend_audio(&mut left, &mut right, &mut renderer, beat * 4);

    (left, right)
}

/// Section 4: Metal breakdown
fn render_metal_breakdown() -> (Vec<f32>, Vec<f32>) {
    let mut renderer = OrchestraRenderer::studio(SAMPLE_RATE);
    renderer.set_reverb_mix(0.15);

    renderer.add_track(1, "Guitar L".to_string(), InstrumentType::ElectricGuitar(GuitarConfig::default()));
    renderer.add_track(2, "Guitar R".to_string(), InstrumentType::ElectricGuitar(GuitarConfig::default()));
    renderer.add_track(3, "Bass".to_string(), InstrumentType::ElectricBass);
    renderer.add_track(4, "Drums".to_string(), InstrumentType::DrumKit);

    let beat = (SAMPLE_RATE * 60.0 / 85.0) as usize; // Slow, heavy 85 BPM
    let sixteenth = beat / 4;
    let mut left = Vec::new();
    let mut right = Vec::new();

    // Heavy breakdown riff (Drop A tuning simulation)
    let riff_notes: Vec<(u8, f32)> = vec![
        (33, 0.95), // A1
        (33, 0.85),
        (33, 0.9),
        (35, 0.8),  // B1
        (33, 0.95),
        (33, 0.75),
        (36, 0.85), // C2
        (33, 0.9),
    ];

    for _ in 0..4 { // 4 times through
        for (i, &(note, vel)) in riff_notes.iter().enumerate() {
            // Chugging guitars (palm muted simulation via short notes)
            renderer.note_on(1, note, vel);
            renderer.note_on(2, note, vel);
            renderer.note_on(3, note, vel * 0.9);

            // Kick on most hits
            if i != 3 {
                renderer.note_on(4, 36, 0.95);
            }

            // China cymbal accents
            if i == 0 || i == 4 {
                renderer.note_on(4, 52, 0.7);
            }

            extend_audio(&mut left, &mut right, &mut renderer, sixteenth * 3);

            renderer.note_off(1, note);
            renderer.note_off(2, note);
            renderer.note_off(3, note);
            renderer.note_off(4, 36);

            // Gap between chugs
            extend_audio(&mut left, &mut right, &mut renderer, sixteenth);
        }
    }

    // Final hit
    renderer.note_on(1, 33, 1.0);
    renderer.note_on(2, 33, 1.0);
    renderer.note_on(3, 33, 1.0);
    renderer.note_on(4, 36, 1.0);
    renderer.note_on(4, 49, 0.9); // Crash
    extend_audio(&mut left, &mut right, &mut renderer, beat * 4);

    (left, right)
}

/// Section 5: Choir finale
fn render_choir_finale() -> (Vec<f32>, Vec<f32>) {
    let mut renderer = OrchestraRenderer::symphony(SAMPLE_RATE);
    renderer.set_reverb_mix(0.45);
    renderer.set_hall_type(HallType::Cathedral);

    renderer.add_track(1, "Soprano".to_string(), InstrumentType::SopranoVoice);
    renderer.add_track(2, "Alto".to_string(), InstrumentType::AltoVoice);
    renderer.add_track(3, "Tenor".to_string(), InstrumentType::TenorVoice);
    renderer.add_track(4, "Bass".to_string(), InstrumentType::BassVoice);
    renderer.add_track(5, "Strings".to_string(), InstrumentType::StringSection);
    renderer.add_track(6, "Organ".to_string(), InstrumentType::Organ);

    let beat = (SAMPLE_RATE * 0.75) as usize; // Slow, majestic
    let mut left = Vec::new();
    let mut right = Vec::new();

    // Ascending hymn progression
    let progression = [
        // C major
        (72, 67, 64, 48, 60),
        // D minor
        (74, 69, 65, 50, 62),
        // F major
        (76, 72, 69, 53, 65),
        // G major (climax)
        (79, 74, 71, 55, 67),
        // C major (resolution)
        (84, 79, 76, 60, 72),
    ];

    for (i, (soprano, alto, tenor, bass, organ_root)) in progression.iter().enumerate() {
        // Crescendo through progression
        let vel = 0.5 + (i as f32 * 0.1);

        renderer.note_on(1, *soprano, vel);
        renderer.note_on(2, *alto, vel);
        renderer.note_on(3, *tenor, vel);
        renderer.note_on(4, *bass, vel);

        // String pad
        renderer.note_on(5, *alto, vel * 0.6);
        renderer.note_on(5, *tenor, vel * 0.6);

        // Organ pedal and chord
        renderer.note_on(6, *organ_root - 12, vel * 0.7);
        renderer.note_on(6, *organ_root, vel * 0.5);

        // Hold duration increases for each chord
        let hold = beat * (3 + i);
        extend_audio(&mut left, &mut right, &mut renderer, hold);

        // Brief pause between chords (except last)
        if i < 4 {
            release_all(&mut renderer, 6);
            extend_audio(&mut left, &mut right, &mut renderer, beat / 2);
        }
    }

    // Long final chord with fade
    extend_audio(&mut left, &mut right, &mut renderer, beat * 8);
    release_all(&mut renderer, 6);

    // Cathedral reverb tail
    extend_audio(&mut left, &mut right, &mut renderer, (SAMPLE_RATE * 3.0) as usize);

    (left, right)
}

/// Helper to render and accumulate audio
fn extend_audio(
    left: &mut Vec<f32>,
    right: &mut Vec<f32>,
    renderer: &mut OrchestraRenderer,
    samples: usize,
) {
    let (l, r) = renderer.process(samples);
    left.extend(l);
    right.extend(r);
}

/// Helper to release all notes on all tracks
fn release_all(renderer: &mut OrchestraRenderer, num_tracks: u32) {
    for i in 1..=num_tracks {
        for note in 0..=127 {
            renderer.note_off(i, note);
        }
    }
}
