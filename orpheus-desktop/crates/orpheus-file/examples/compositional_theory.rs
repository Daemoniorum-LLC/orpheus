//! Deep theoretical analysis of compositional approach

use orpheus_file::guitar_pro;
use std::collections::HashMap;
use std::env;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        "/home/user/workspace/music/Sheet Music/Guitar Pro/Deicidal Carnage/A Calamitous Orchestration Revised (1).gp".into()
    });

    match guitar_pro::parse_file(&path) {
        Ok(gp) => analyze_theory(&gp),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn analyze_theory(gp: &guitar_pro::GuitarProFile) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     COMPOSITIONAL THEORY ANALYSIS                            ║");
    println!("║     \"{}\"                    ║", gp.info.title);
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // 1. Voice Independence Analysis
    println!("═══ VOICE INDEPENDENCE ═══\n");

    let mut track_stats: Vec<(String, usize, f32, f32, Vec<u8>)> = Vec::new();

    for track in &gp.tracks {
        let notes: Vec<_> = gp.measures.iter()
            .flat_map(|m| &m.beats)
            .filter(|tb| tb.track == track.number)
            .flat_map(|tb| &tb.beats)
            .flat_map(|b| &b.notes)
            .collect();

        let frets: Vec<u8> = notes.iter().map(|n| n.fret).collect();
        let strings: Vec<u8> = notes.iter().map(|n| n.string).collect();

        let avg_fret = if !frets.is_empty() {
            frets.iter().map(|f| *f as f32).sum::<f32>() / frets.len() as f32
        } else { 0.0 };

        let avg_string = if !strings.is_empty() {
            strings.iter().map(|s| *s as f32).sum::<f32>() / strings.len() as f32
        } else { 0.0 };

        track_stats.push((track.name.clone(), notes.len(), avg_fret, avg_string, frets.clone()));
    }

    // Calculate overlap between tracks
    println!("Track Register Zones (lower avg = bass register, higher = treble):");
    for (name, count, avg_fret, avg_string, _) in &track_stats {
        if *count == 0 { continue; }
        let register = match *avg_fret {
            f if f < 5.0 => "Low/Chug",
            f if f < 10.0 => "Mid/Rhythm",
            f if f < 15.0 => "Upper-Mid",
            _ => "High/Lead",
        };
        println!("  {:20} → {} (avg fret {:.1}, avg string {:.1})",
            name, register, avg_fret, avg_string);
    }

    // 2. Rhythmic Density Analysis Per Section
    println!("\n═══ DENSITY MAPPING ═══\n");

    // Get markers
    let markers: Vec<_> = gp.measures.iter()
        .filter(|m| m.marker.is_some())
        .map(|m| (m.number, m.marker.as_ref().unwrap().clone()))
        .collect();

    if markers.len() >= 2 {
        for i in 0..markers.len() {
            let (start, name) = &markers[i];
            let end = if i + 1 < markers.len() { markers[i + 1].0 - 1 } else { gp.measures.len() as u16 };

            // Calculate density per track in this section
            println!("Section: {} (m{}-{})", name, start, end);

            for track in &gp.tracks {
                let section_notes: usize = gp.measures.iter()
                    .filter(|m| m.number >= *start && m.number <= end)
                    .flat_map(|m| &m.beats)
                    .filter(|tb| tb.track == track.number)
                    .flat_map(|tb| &tb.beats)
                    .map(|b| b.notes.len())
                    .sum();

                let section_beats: usize = gp.measures.iter()
                    .filter(|m| m.number >= *start && m.number <= end)
                    .flat_map(|m| &m.beats)
                    .filter(|tb| tb.track == track.number)
                    .flat_map(|tb| &tb.beats)
                    .count();

                if section_beats > 0 {
                    let density = section_notes as f32 / section_beats as f32;
                    let bar = "█".repeat((density * 10.0) as usize);
                    println!("  {:20} {:>6} notes  {:.2}/beat  {}",
                        track.name, section_notes, density, bar);
                }
            }
            println!();
        }
    }

    // 3. Contrapuntal Analysis
    println!("═══ CONTRAPUNTAL RELATIONSHIPS ═══\n");

    // Analyze which tracks play together
    let mut simultaneous_notes: HashMap<(u8, u8), usize> = HashMap::new();

    for measure in &gp.measures {
        // Group beats by position (simplified - count overlapping notes)
        for tb1 in &measure.beats {
            for tb2 in &measure.beats {
                if tb1.track < tb2.track {
                    let count = tb1.beats.iter()
                        .zip(tb2.beats.iter())
                        .filter(|(b1, b2)| !b1.notes.is_empty() && !b2.notes.is_empty())
                        .count();
                    *simultaneous_notes.entry((tb1.track, tb2.track)).or_insert(0) += count;
                }
            }
        }
    }

    let mut pairs: Vec<_> = simultaneous_notes.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1));

    println!("Most common simultaneous pairings:");
    for ((t1, t2), count) in pairs.iter().take(5) {
        let n1 = gp.tracks.get(*t1 as usize - 1).map(|t| t.name.as_str()).unwrap_or("?");
        let n2 = gp.tracks.get(*t2 as usize - 1).map(|t| t.name.as_str()).unwrap_or("?");
        println!("  {} ↔ {}: {} coincidences", n1, n2, count);
    }

    // 4. Harmonic/Interval Analysis
    println!("\n═══ INTERVAL VOCABULARY ═══\n");

    // Find common intervals within chords/simultaneous notes
    let mut intervals: HashMap<i8, usize> = HashMap::new();

    for measure in &gp.measures {
        for tb in &measure.beats {
            for beat in &tb.beats {
                if beat.notes.len() >= 2 {
                    let mut frets: Vec<_> = beat.notes.iter().map(|n| n.fret as i8).collect();
                    frets.sort();
                    for i in 0..frets.len() - 1 {
                        let interval = frets[i + 1] - frets[i];
                        if interval > 0 && interval <= 12 {
                            *intervals.entry(interval).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    let interval_names = [
        (1, "minor 2nd"), (2, "major 2nd"), (3, "minor 3rd"), (4, "major 3rd"),
        (5, "perfect 4th"), (6, "tritone"), (7, "perfect 5th"), (8, "minor 6th"),
        (9, "major 6th"), (10, "minor 7th"), (11, "major 7th"), (12, "octave"),
    ];

    let mut interval_vec: Vec<_> = intervals.iter().collect();
    interval_vec.sort_by(|a, b| b.1.cmp(a.1));

    println!("Interval usage (from simultaneous notes):");
    for (interval, count) in interval_vec.iter().take(6) {
        let name = interval_names.iter()
            .find(|(i, _)| i == *interval)
            .map(|(_, n)| *n)
            .unwrap_or("unknown");
        let bar = "█".repeat((*count / 50).min(20));
        println!("  {:12} ({:2} frets): {:>4}  {}", name, interval, count, bar);
    }

    // 5. Motion Analysis
    println!("\n═══ MELODIC MOTION ═══\n");

    let mut step_motion = 0;
    let mut leap_motion = 0;
    let mut repeated = 0;

    for measure in &gp.measures {
        for tb in &measure.beats {
            let frets: Vec<u8> = tb.beats.iter()
                .flat_map(|b| &b.notes)
                .map(|n| n.fret)
                .collect();

            for i in 1..frets.len() {
                let diff = (frets[i] as i16 - frets[i-1] as i16).abs();
                match diff {
                    0 => repeated += 1,
                    1..=2 => step_motion += 1,
                    _ => leap_motion += 1,
                }
            }
        }
    }

    let total = (step_motion + leap_motion + repeated).max(1) as f32;
    println!("  Repeated notes: {:>5} ({:>5.1}%)", repeated, 100.0 * repeated as f32 / total);
    println!("  Step motion:    {:>5} ({:>5.1}%)", step_motion, 100.0 * step_motion as f32 / total);
    println!("  Leap motion:    {:>5} ({:>5.1}%)", leap_motion, 100.0 * leap_motion as f32 / total);

    let step_leap_ratio = step_motion as f32 / leap_motion.max(1) as f32;
    println!("\n  Step-to-leap ratio: {:.2}", step_leap_ratio);
    if step_leap_ratio < 0.5 {
        println!("  → Angular, disjunct style (characteristic of tech death)");
    } else if step_leap_ratio > 2.0 {
        println!("  → Conjunct, flowing melodic style");
    } else {
        println!("  → Balanced melodic vocabulary");
    }

    // 6. Role Assignment Inference
    println!("\n═══ COMPOSITIONAL ROLE INFERENCE ═══\n");

    for (name, count, avg_fret, avg_string, frets) in &track_stats {
        if *count == 0 { continue; }

        // Calculate variance
        let variance = if !frets.is_empty() {
            let mean = frets.iter().map(|f| *f as f32).sum::<f32>() / frets.len() as f32;
            frets.iter().map(|f| (*f as f32 - mean).powi(2)).sum::<f32>() / frets.len() as f32
        } else { 0.0 };

        let role = if *avg_fret > 12.0 && variance > 20.0 {
            "LEAD (high register, high variance = melodic exploration)"
        } else if *avg_fret < 6.0 && variance < 10.0 {
            "RHYTHM ANCHOR (low register, consistent = foundational pulse)"
        } else if variance > 50.0 {
            "ORCHESTRAL/SYNTH (extreme variance = likely non-guitar)"
        } else if *avg_string < 2.5 {
            "UPPER HARMONY (high strings = upper voice in texture)"
        } else if *avg_string > 4.5 {
            "BASS FOUNDATION (low strings = bottom of harmonic stack)"
        } else {
            "INNER VOICE (mid-register = harmonic filler/countermelody)"
        };

        println!("  {:20} → {}", name, role);
        println!("      (fret σ²={:.1}, avg_string={:.1})", variance, avg_string);
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║     SYNTHESIS                                                ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
}
