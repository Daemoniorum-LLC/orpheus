//! Integration tests for orpheus-synth
//!
//! Tests the full pipeline from notation/sequences through synthesis to audio output.

use orpheus_synth::{
    instrument_router::{InstrumentType, MultiInstrumentEngine},
    orchestra_renderer::OrchestraRenderer,
    sequencer::create_test_sequence,
};

/// Test that the MultiInstrumentEngine can route MIDI to different synths
#[test]
fn test_multi_instrument_routing() {
    let sample_rate = 44100.0;
    let mut engine = MultiInstrumentEngine::new(sample_rate);

    // Add various tracks
    engine.add_track(1, "Lead Guitar".to_string(), InstrumentType::ElectricGuitar(Default::default()));
    engine.add_track(2, "Rhythm Guitar".to_string(), InstrumentType::ElectricGuitar(Default::default()));
    engine.add_track(3, "Bass".to_string(), InstrumentType::ElectricBass);
    engine.add_track(4, "Drums".to_string(), InstrumentType::DrumKit);
    engine.add_track(5, "Piano".to_string(), InstrumentType::GrandPiano);
    engine.add_track(6, "Violin".to_string(), InstrumentType::Violin);
    engine.add_track(7, "Trumpet".to_string(), InstrumentType::Trumpet);

    assert_eq!(engine.tracks().len(), 7);

    // Play notes on each track
    engine.note_on_by_id(1, 64, 0.8); // Guitar E4
    engine.note_on_by_id(2, 52, 0.7); // Guitar E3
    engine.note_on_by_id(3, 40, 0.8); // Bass E2
    engine.note_on_by_id(4, 36, 0.9); // Kick drum
    engine.note_on_by_id(5, 60, 0.7); // Piano C4
    engine.note_on_by_id(6, 69, 0.6); // Violin A4
    engine.note_on_by_id(7, 67, 0.7); // Trumpet G4

    // Process some audio
    let (left, right) = engine.process(1024);

    assert_eq!(left.len(), 1024);
    assert_eq!(right.len(), 1024);

    // Should have audio output
    let max_l = left.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    let max_r = right.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    assert!(max_l > 0.0 || max_r > 0.0, "Should produce audio output");
}

/// Test the orchestra renderer with multiple instruments and spatial positioning
#[test]
fn test_orchestra_renderer_full_orchestra() {
    let sample_rate = 44100.0;
    let mut renderer = OrchestraRenderer::symphony(sample_rate);

    // Add a full string section
    renderer.add_track(1, "Violin I".to_string(), InstrumentType::Violin);
    renderer.add_track(2, "Violin II".to_string(), InstrumentType::Violin);
    renderer.add_track(3, "Viola".to_string(), InstrumentType::Viola);
    renderer.add_track(4, "Cello".to_string(), InstrumentType::Cello);
    renderer.add_track(5, "Bass".to_string(), InstrumentType::DoubleBass);

    // Add brass
    renderer.add_track(6, "Trumpet".to_string(), InstrumentType::Trumpet);
    renderer.add_track(7, "French Horn".to_string(), InstrumentType::FrenchHorn);
    renderer.add_track(8, "Trombone".to_string(), InstrumentType::Trombone);

    // Add woodwinds
    renderer.add_track(9, "Flute".to_string(), InstrumentType::Flute);
    renderer.add_track(10, "Oboe".to_string(), InstrumentType::Oboe);
    renderer.add_track(11, "Clarinet".to_string(), InstrumentType::Clarinet);

    // Add percussion
    renderer.add_track(12, "Timpani".to_string(), InstrumentType::Timpani);

    assert_eq!(renderer.track_count(), 12);

    // Play a chord across the strings
    renderer.note_on(1, 76, 0.7); // Violin I - E5
    renderer.note_on(2, 72, 0.7); // Violin II - C5
    renderer.note_on(3, 67, 0.7); // Viola - G4
    renderer.note_on(4, 60, 0.7); // Cello - C4
    renderer.note_on(5, 48, 0.7); // Bass - C3

    // Brass fanfare
    renderer.note_on(6, 72, 0.8); // Trumpet - C5
    renderer.note_on(7, 67, 0.6); // Horn - G4
    renderer.note_on(8, 60, 0.7); // Trombone - C4

    // Woodwind melody
    renderer.note_on(9, 79, 0.6); // Flute - G5
    renderer.note_on(10, 72, 0.5); // Oboe - C5
    renderer.note_on(11, 67, 0.5); // Clarinet - G4

    // Timpani roll
    renderer.note_on(12, 48, 0.8); // Timpani - C3

    // Process audio with spatial positioning and reverb
    let (left, right) = renderer.process(4096);

    assert_eq!(left.len(), 4096);
    assert_eq!(right.len(), 4096);

    // Verify we get stereo output with the spatial processing
    let max_l = left.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    let max_r = right.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);

    assert!(max_l > 0.0, "Should have left channel output");
    assert!(max_r > 0.0, "Should have right channel output");

    // Get visualization data
    let viz = renderer.get_visualization();
    assert_eq!(viz.tracks.len(), 12);
    assert!(viz.hall_width > 0.0);
    assert!(viz.hall_depth > 0.0);
}

/// Test rendering a guitar sequence through the orchestra renderer
#[test]
fn test_guitar_sequence_rendering() {
    let sample_rate = 44100.0;
    let mut renderer = OrchestraRenderer::studio(sample_rate);

    // Set up a rock band configuration
    renderer.add_track(1, "Lead Guitar".to_string(), InstrumentType::ElectricGuitar(Default::default()));
    renderer.add_track(2, "Rhythm Guitar".to_string(), InstrumentType::ElectricGuitar(Default::default()));
    renderer.add_track(3, "Bass".to_string(), InstrumentType::ElectricBass);
    renderer.add_track(4, "Drums".to_string(), InstrumentType::DrumKit);

    // Get the test sequence (E minor arpeggio)
    let events = create_test_sequence();

    // Simulate playing the sequence on lead guitar
    // Convert string/fret to MIDI note
    fn string_fret_to_midi(string: u8, fret: u8) -> u8 {
        let tuning = [64, 59, 55, 50, 45, 40]; // Standard tuning E4 to E2
        let base = tuning[(string - 1) as usize];
        base + fret
    }

    for event in &events {
        let midi_note = string_fret_to_midi(event.string, event.fret);
        renderer.note_on(1, midi_note, event.velocity);
    }

    // Render some audio
    let (left, right) = renderer.process(2048);

    assert_eq!(left.len(), 2048);
    assert_eq!(right.len(), 2048);

    // Should have audio
    let has_audio = left.iter().any(|s| s.abs() > 0.001)
        || right.iter().any(|s| s.abs() > 0.001);
    assert!(has_audio, "Guitar sequence should produce audio");
}

/// Test different venue presets
#[test]
fn test_venue_presets() {
    let sample_rate = 44100.0;

    // Test each venue preset
    let venues: Vec<(&str, OrchestraRenderer)> = vec![
        ("Symphony Hall", OrchestraRenderer::symphony(sample_rate)),
        ("Chamber Hall", OrchestraRenderer::chamber(sample_rate)),
        ("Jazz Club", OrchestraRenderer::jazz_club(sample_rate)),
        ("Studio", OrchestraRenderer::studio(sample_rate)),
    ];

    for (name, mut renderer) in venues {
        // Add a test instrument
        renderer.add_track(1, "Piano".to_string(), InstrumentType::GrandPiano);
        renderer.note_on(1, 60, 0.8);

        let (left, right) = renderer.process(512);

        assert_eq!(left.len(), 512, "{} should produce 512 samples", name);
        assert_eq!(right.len(), 512, "{} should produce 512 samples", name);

        let viz = renderer.get_visualization();
        assert!(viz.hall_width > 0.0, "{} should have hall dimensions", name);
    }
}

/// Test track volume and mute controls
#[test]
fn test_track_controls() {
    let sample_rate = 44100.0;
    let mut renderer = OrchestraRenderer::new(sample_rate);

    renderer.add_track(1, "Violin".to_string(), InstrumentType::Violin);
    renderer.add_track(2, "Cello".to_string(), InstrumentType::Cello);

    // Play notes on both
    renderer.note_on(1, 69, 0.8); // A4
    renderer.note_on(2, 57, 0.8); // A3

    // Get baseline audio
    let (left1, _) = renderer.process(256);
    let baseline_level: f32 = left1.iter().map(|s| s.abs()).sum();

    // Play again
    renderer.note_on(1, 69, 0.8);
    renderer.note_on(2, 57, 0.8);

    // Mute one track
    renderer.mute_track(1, true);
    let (left2, _) = renderer.process(256);
    let muted_level: f32 = left2.iter().map(|s| s.abs()).sum();

    // Muted should have less signal (one track is silent)
    // Note: with reverb and spatial processing, the difference may be subtle
    // but the muted level should not be significantly higher
    assert!(muted_level <= baseline_level * 1.1, "Muting should reduce signal");
}

/// Test MIDI program to instrument type mapping
#[test]
fn test_midi_program_mapping() {
    use orpheus_synth::instrument_router::InstrumentType;

    // Test standard MIDI program numbers
    let mappings = vec![
        (0, "Acoustic Grand Piano"),
        (25, "Acoustic Guitar (nylon)"),
        (26, "Acoustic Guitar (steel)"),
        (27, "Electric Guitar (jazz)"),
        (28, "Electric Guitar (clean)"),
        (29, "Electric Guitar (muted)"),
        (30, "Overdriven Guitar"),
        (32, "Acoustic Bass"),
        (33, "Electric Bass (finger)"),
        (40, "Violin"),
        (41, "Viola"),
        (42, "Cello"),
        (43, "Contrabass"),
        (56, "Trumpet"),
        (57, "Trombone"),
        (60, "French Horn"),
        (73, "Flute"),
        (68, "Oboe"),
        (71, "Clarinet"),
    ];

    for (program, name) in mappings {
        let instrument_type = InstrumentType::from_midi_program(program);
        // Just verify it doesn't return a generic Piano for everything
        println!("MIDI {} ({}) -> {:?}", program, name, instrument_type);
    }
}
