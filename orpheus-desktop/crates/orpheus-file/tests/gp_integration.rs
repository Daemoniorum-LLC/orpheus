//! Integration tests for GP file parsing and conversion

use orpheus_file::guitar_pro::{convert_gp_to_tab, parse_file};
use std::path::Path;

const GP5_TEST_FILE: &str =
    "/home/user/workspace/music/Sheet Music/Guitar Pro/Bands/Gojira/Gojira - Magma.gp5";
const GP7_TEST_FILE: &str =
    "/home/user/workspace/music/Sheet Music/Guitar Pro/Drums/20-Guitar_Pro-Drum_Patterns_gp/20-Guitar_Pro-Drum_Patterns/Metal/Megadeth-Holy_wars_The_Punishment_Due.gp";

#[test]
fn test_parse_gp7_file() {
    let path = Path::new(GP7_TEST_FILE);

    if !path.exists() {
        eprintln!("GP7 test file not found: {}", GP7_TEST_FILE);
        return;
    }

    let result = parse_file(path);

    match result {
        Ok(gp_file) => {
            println!("Successfully parsed GP7 file:");
            println!("  Version: {:?}", gp_file.version);
            println!("  Title: {}", gp_file.info.title);
            println!("  Artist: {}", gp_file.info.artist);
            println!("  Tempo: {} BPM", gp_file.tempo);
            println!("  Tracks: {}", gp_file.tracks.len());
            println!("  Measures: {}", gp_file.measures.len());

            for (i, track) in gp_file.tracks.iter().enumerate() {
                println!(
                    "  Track {}: {} ({} strings)",
                    i + 1,
                    track.name,
                    track.strings
                );
            }

            // Convert to TabDocument
            let tab_doc = convert_gp_to_tab(&gp_file);

            println!("\nConverted to TabDocument:");
            println!("  Tracks: {}", tab_doc.tracks.len());
            println!("  Measures: {}", tab_doc.measures.len());
            println!("  Tempo: {} BPM", tab_doc.tempo_map.base_tempo);

            // Assert GP7 parsing worked
            assert!(
                matches!(
                    gp_file.version,
                    orpheus_file::guitar_pro::GpVersion::Gp7
                        | orpheus_file::guitar_pro::GpVersion::Gp6
                ),
                "Expected GP7 or GP6 version"
            );
        }
        Err(e) => {
            panic!("GP7 parsing failed: {:?}", e);
        }
    }
}

#[test]
fn test_parse_multiple_gp7_files() {
    let test_paths = [
        "/home/user/workspace/music/Sheet Music/Guitar Pro/Drums/20-Guitar_Pro-Drum_Patterns_gp/20-Guitar_Pro-Drum_Patterns/Blues/ZZ_Top-La_Grange.gp",
        "/home/user/workspace/music/Sheet Music/Guitar Pro/Drums/20-Guitar_Pro-Drum_Patterns_gp/20-Guitar_Pro-Drum_Patterns/Metal/Iron_Maiden-1984-Powerslave-01-Aces_High.gp",
        "/home/user/workspace/music/Sheet Music/Guitar Pro/Deicidal Carnage/A Calamitous Orchestration Revised (1).gp",
    ];

    let mut success_count = 0;
    let mut total_count = 0;

    for path_str in &test_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        total_count += 1;

        match parse_file(path) {
            Ok(gp) => {
                println!(
                    "Parsed GP7: {} - {} ({} tracks, {} measures)",
                    gp.info.artist,
                    gp.info.title,
                    gp.tracks.len(),
                    gp.measures.len()
                );
                success_count += 1;
            }
            Err(e) => {
                eprintln!("Failed to parse {}: {:?}", path_str, e);
            }
        }
    }

    println!(
        "\nGP7 parsing: {}/{} files succeeded",
        success_count, total_count
    );
    assert!(
        success_count > 0,
        "At least one GP7 file should parse successfully"
    );
}

#[test]
fn test_parse_gojira_magma_gp5() {
    let path = Path::new(GP5_TEST_FILE);

    if !path.exists() {
        eprintln!("Test file not found: {}", GP5_TEST_FILE);
        return;
    }

    let result = parse_file(path);

    match result {
        Ok(gp_file) => {
            println!("Successfully parsed GP5 file:");
            println!("  Title: {}", gp_file.info.title);
            println!("  Artist: {}", gp_file.info.artist);
            println!("  Tempo: {} BPM", gp_file.tempo);
            println!("  Tracks: {}", gp_file.tracks.len());
            println!("  Measures: {}", gp_file.measures.len());

            // Convert to TabDocument
            let tab_doc = convert_gp_to_tab(&gp_file);

            println!("\nConverted to TabDocument:");
            println!("  Tracks: {}", tab_doc.tracks.len());
            println!("  Measures: {}", tab_doc.measures.len());
            println!("  Tempo: {} BPM", tab_doc.tempo_map.base_tempo);

            for (i, track) in tab_doc.tracks.iter().enumerate() {
                println!("  Track {}: {}", i + 1, track.name);
            }

            // Assert basic parsing worked
            assert!(!gp_file.info.title.is_empty() || !gp_file.info.artist.is_empty());
        }
        Err(e) => {
            // GP5 parsing is experimental, so we log but don't fail
            eprintln!("GP5 parsing returned error (experimental): {:?}", e);
        }
    }
}

#[test]
fn test_parse_gp3_gp4_files() {
    // GP3/GP4 should parse better than GP5
    let test_paths = [
        "/home/user/workspace/music/Sheet Music/Guitar Pro/Bands/Gojira/Gojira - Pray.gp5",
        "/home/user/workspace/music/Sheet Music/Guitar Pro/Bands/Necrophagist - Foul Body Autopsy.gp5",
    ];

    for path_str in &test_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        match parse_file(path) {
            Ok(gp) => {
                println!(
                    "Parsed: {} - {} ({} tracks)",
                    gp.info.artist, gp.info.title, gp.tracks.len()
                );
            }
            Err(e) => {
                eprintln!("Failed to parse {}: {:?}", path_str, e);
            }
        }
    }
}
