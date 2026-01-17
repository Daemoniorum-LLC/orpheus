//! Render a Guitar Pro 5 file to WAV audio
//!
//! Demonstrates the complete GP5 → synth pipeline:
//! 1. Parse GP5 file with orpheus-file
//! 2. Sequence notes with TabSequencer
//! 3. Render with PlaybackEngine
//! 4. Export to WAV
//!
//! Usage:
//!   cargo run --example render_gp5 [path/to/file.gp5]
//!
//! Default: Uses a test GP5 file if available

use orpheus_file::guitar_pro;
use orpheus_synth::{
    playback::PlaybackEngine,
    sequencer::TabSequencer,
    wav::write_wav,
};
use std::env;

const SAMPLE_RATE: u32 = 44100;

fn main() {
    // Initialize tracing for debug output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("orpheus_file::guitar_pro::gp5=trace".parse().unwrap()))
        .init();

    // Get file path from command line or use default
    let path = env::args().nth(1).unwrap_or_else(|| {
        "/home/user/workspace/music/Sheet Music/Guitar Pro/Bands/Gojira/Gojira - Magma.gp5".into()
    });

    println!("=== GP5 to Audio Renderer ===\n");
    println!("Input: {}", path);

    // Parse the GP5 file
    println!("\n1. Parsing GP5 file...");
    let gp_file = match guitar_pro::parse_file(&path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error parsing GP5 file: {:?}", e);
            std::process::exit(1);
        }
    };

    println!("   Title: {}", gp_file.info.title);
    println!("   Artist: {}", gp_file.info.artist);
    println!("   Tempo: {} BPM", gp_file.tempo);
    println!("   Tracks: {}", gp_file.tracks.len());
    println!("   Measures: {}", gp_file.measures.len());

    // Show track list
    println!("\n   Track listing:");
    for (i, track) in gp_file.tracks.iter().enumerate() {
        println!("     {} - {} ({} strings{})",
            i + 1,
            track.name,
            track.strings,
            if track.is_drums { ", drums" } else { "" }
        );
    }

    // Find first non-drum track with content
    let track_idx = gp_file.tracks.iter()
        .position(|t| !t.is_drums)
        .unwrap_or(0);

    println!("\n2. Creating sequencer for track {}...", track_idx + 1);

    // Create sequencer from GP file
    let sequencer = TabSequencer::from_gp_file(&gp_file, track_idx);

    println!("   Events: {}", sequencer.events().len());
    println!("   Duration: {:.1} seconds", sequencer.total_seconds());
    println!("   Tempo: {} BPM", sequencer.tempo());

    if sequencer.events().is_empty() {
        println!("\n   No events found in track. Trying first track with notes...");

        // Try each track until we find one with events
        let mut found_track = None;
        for i in 0..gp_file.tracks.len() {
            let seq = TabSequencer::from_gp_file(&gp_file, i);
            if !seq.events().is_empty() {
                found_track = Some((i, seq));
                break;
            }
        }

        if let Some((idx, seq)) = found_track {
            println!("   Found events in track {}: {}", idx + 1, gp_file.tracks[idx].name);
            render_track(&seq, &gp_file.info.title, idx);
        } else {
            println!("   No tracks with note events found.");
        }
    } else {
        render_track(&sequencer, &gp_file.info.title, track_idx);
    }
}

fn render_track(sequencer: &TabSequencer, title: &str, track_idx: usize) {
    println!("\n3. Rendering audio...");

    // Create playback engine
    let mut engine = PlaybackEngine::new(SAMPLE_RATE);
    sequencer.load_into_engine(&mut engine);

    // Render with some extra time for decay
    let duration = sequencer.total_seconds() + 2.0;
    let buffer = engine.render_to_buffer(duration, true);

    println!("   Rendered {} samples ({:.1}s stereo)", buffer.len() / 2, duration);

    // Split into left/right channels
    let mut left = Vec::with_capacity(buffer.len() / 2);
    let mut right = Vec::with_capacity(buffer.len() / 2);
    for chunk in buffer.chunks(2) {
        left.push(chunk[0]);
        right.push(chunk[1]);
    }

    // Calculate peak level
    let peak = buffer.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    println!("   Peak level: {:.1} dB", 20.0 * peak.max(0.0001).log10());

    // Create output filename
    let safe_title = title.chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ')
        .collect::<String>()
        .replace(' ', "_");
    let output_file = format!("gp5_render_t{}_{}.wav",
        track_idx + 1,
        if safe_title.is_empty() { "output" } else { &safe_title }
    );

    println!("\n4. Writing WAV file...");
    match write_wav(&output_file, &left, &right, SAMPLE_RATE) {
        Ok(_) => {
            let size_mb = (left.len() * 4 * 2) as f32 / 1_000_000.0;
            println!("   Output: {} ({:.1} MB)", output_file, size_mb);
            println!("\n=== Complete! ===");
        }
        Err(e) => {
            eprintln!("Error writing WAV: {:?}", e);
        }
    }
}
