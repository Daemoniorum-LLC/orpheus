//! Example: Parse a Guitar Pro 5 file and display info

use orpheus_file::guitar_pro;
use std::env;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    // Get file path from command line or use default
    let path = env::args().nth(1).unwrap_or_else(|| {
        "/home/user/workspace/music/Sheet Music/Guitar Pro/Bands/Gojira/Gojira - Magma.gp5".into()
    });

    println!("Parsing: {}", path);
    println!("{}", "=".repeat(60));

    match guitar_pro::parse_file(&path) {
        Ok(gp) => {
            println!("Format Version: {:?}", gp.version);
            println!();
            println!("Song Info:");
            println!("  Title:    {}", gp.info.title);
            println!("  Artist:   {}", gp.info.artist);
            println!("  Album:    {}", gp.info.album);
            println!("  Author:   {}", gp.info.tab_author);
            println!();
            println!("Tempo: {} BPM", gp.tempo);
            println!("Time Signature: {}/{}", gp.time_signature.numerator, gp.time_signature.denominator);
            println!();
            println!("Tracks ({}):", gp.tracks.len());
            for track in &gp.tracks {
                println!("  {} - {} ({} strings, {})",
                    track.number,
                    track.name,
                    track.strings,
                    if track.is_drums { "drums" } else { "guitar/bass" }
                );
                println!("    Tuning: {:?}", track.tuning);
            }
            println!();
            println!("Measures: {}", gp.measures.len());

            // Show first few measures with markers
            let markers: Vec<_> = gp.measures.iter()
                .filter(|m| m.marker.is_some())
                .collect();
            if !markers.is_empty() {
                println!("\nSection Markers:");
                for m in markers.iter().take(10) {
                    println!("  Measure {}: {}", m.number, m.marker.as_ref().unwrap());
                }
            }

            // Count notes
            let total_beats: usize = gp.measures.iter()
                .flat_map(|m| &m.beats)
                .map(|tb| tb.beats.len())
                .sum();
            let total_notes: usize = gp.measures.iter()
                .flat_map(|m| &m.beats)
                .flat_map(|tb| &tb.beats)
                .map(|b| b.notes.len())
                .sum();
            println!("\nTotal Beats: {}", total_beats);
            println!("Total Notes: {}", total_notes);

            // Show first few notes
            if total_notes > 0 {
                println!("\nFirst Notes:");
                let mut shown = 0;
                'outer: for (m_idx, measure) in gp.measures.iter().enumerate() {
                    for track_beats in &measure.beats {
                        for beat in &track_beats.beats {
                            for note in &beat.notes {
                                println!(
                                    "  M{} T{}: String {} Fret {} (vel: {}{}{})",
                                    m_idx + 1,
                                    track_beats.track,
                                    note.string,
                                    note.fret,
                                    note.velocity,
                                    if note.tied { " tied" } else { "" },
                                    if note.ghost { " ghost" } else { "" }
                                );
                                shown += 1;
                                if shown >= 15 {
                                    println!("  ... and {} more notes", total_notes - shown);
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
            }

            println!("\n✓ Parse complete!");
        }
        Err(e) => {
            eprintln!("Error parsing file: {:?}", e);
            std::process::exit(1);
        }
    }
}
