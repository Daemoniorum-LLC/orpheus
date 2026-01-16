//! Tonal structure and harmonic language analysis
//! Extracting the composer's approach to chord progressions and tonal movement

use orpheus_file::guitar_pro::{self, GuitarProFile};
use std::collections::HashMap;

fn main() {
    let path = "/home/user/workspace/music/Sheet Music/Guitar Pro/Deicidal Carnage/A Calamitous Orchestration Revised (1).gp";

    match guitar_pro::parse_file(path) {
        Ok(gp) => analyze_harmony(&gp),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

/// Convert fret + string + tuning to MIDI pitch
fn fret_to_pitch(fret: u8, string: u8, tuning: &[u8]) -> Option<u8> {
    // String is 1-indexed, tuning is 0-indexed (high to low)
    let string_idx = (string as usize).saturating_sub(1);
    tuning.get(string_idx).map(|open| open + fret)
}

/// Get pitch class (0-11, where 0=C)
fn pitch_class(midi: u8) -> u8 {
    midi % 12
}

/// Pitch class name
fn pitch_name(pc: u8) -> &'static str {
    match pc {
        0 => "C", 1 => "C#", 2 => "D", 3 => "D#", 4 => "E", 5 => "F",
        6 => "F#", 7 => "G", 8 => "G#", 9 => "A", 10 => "A#", 11 => "B",
        _ => "?"
    }
}

/// Interval name
fn interval_name(semitones: i8) -> &'static str {
    match semitones.abs() % 12 {
        0 => "unison", 1 => "m2", 2 => "M2", 3 => "m3", 4 => "M3",
        5 => "P4", 6 => "tritone", 7 => "P5", 8 => "m6", 9 => "M6",
        10 => "m7", 11 => "M7",
        _ => "?"
    }
}

/// Analyze chord quality from intervals
fn chord_quality(intervals: &[u8]) -> String {
    let has = |i: u8| intervals.contains(&i);

    // Check for various chord qualities based on intervals from root
    if intervals.is_empty() {
        return "single".to_string();
    }

    let mut qualities = Vec::new();

    // Thirds
    if has(3) { qualities.push("m3"); }
    if has(4) { qualities.push("M3"); }

    // Fifths
    if has(6) { qualities.push("b5"); }
    if has(7) { qualities.push("P5"); }
    if has(8) { qualities.push("#5"); }

    // Extensions
    if has(10) { qualities.push("m7"); }
    if has(11) { qualities.push("M7"); }
    if has(1) || has(13) { qualities.push("b9"); }
    if has(2) || has(14) { qualities.push("9"); }

    // Tritone presence
    if has(6) { qualities.push("tritone"); }

    if qualities.is_empty() {
        // Power chord or other
        if has(7) { return "5".to_string(); }
        return format!("intervals:{:?}", intervals);
    }

    qualities.join("+")
}

fn analyze_harmony(gp: &GuitarProFile) {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║     HARMONIC LANGUAGE ANALYSIS                                   ║");
    println!("║     \"{}\"                                    ║", gp.info.title);
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Get tuning from first guitar track
    let tuning = gp.tracks.iter()
        .find(|t| !t.is_drums)
        .map(|t| t.tuning.clone())
        .unwrap_or_else(|| vec![64, 59, 55, 50, 45, 40]); // Standard E

    println!("Tuning: {:?} ({})\n",
        tuning.iter().map(|p| pitch_name(pitch_class(*p))).collect::<Vec<_>>(),
        tuning.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("-"));

    // Extract all chords (simultaneous notes)
    #[derive(Debug, Clone)]
    struct Chord {
        measure: u16,
        beat_idx: usize,
        pitches: Vec<u8>,
        pitch_classes: Vec<u8>,
        root: u8,  // Lowest pitch class
        intervals: Vec<u8>,  // From root
    }

    let mut chords: Vec<Chord> = Vec::new();

    for measure in &gp.measures {
        for tb in &measure.beats {
            // Get track tuning
            let track_tuning = gp.tracks.iter()
                .find(|t| t.number == tb.track)
                .map(|t| &t.tuning)
                .unwrap_or(&tuning);

            for (beat_idx, beat) in tb.beats.iter().enumerate() {
                if beat.notes.len() >= 2 {
                    let mut pitches: Vec<u8> = beat.notes.iter()
                        .filter_map(|n| fret_to_pitch(n.fret, n.string, track_tuning))
                        .collect();
                    pitches.sort();
                    pitches.dedup();

                    if pitches.len() >= 2 {
                        let pitch_classes: Vec<u8> = pitches.iter()
                            .map(|p| pitch_class(*p))
                            .collect();

                        let root = *pitches.first().unwrap();
                        let root_pc = pitch_class(root);

                        let intervals: Vec<u8> = pitches.iter()
                            .map(|p| ((*p as i16 - root as i16).abs() % 12) as u8)
                            .filter(|i| *i > 0)
                            .collect();

                        chords.push(Chord {
                            measure: measure.number,
                            beat_idx,
                            pitches,
                            pitch_classes,
                            root: root_pc,
                            intervals,
                        });
                    }
                }
            }
        }
    }

    println!("═══ CHORD VOCABULARY ═══\n");
    println!("Total chords analyzed: {}\n", chords.len());

    // Root distribution
    let mut root_counts: HashMap<u8, usize> = HashMap::new();
    for chord in &chords {
        *root_counts.entry(chord.root).or_insert(0) += 1;
    }

    let mut roots: Vec<_> = root_counts.iter().collect();
    roots.sort_by(|a, b| b.1.cmp(a.1));

    println!("Root note distribution:");
    for (root, count) in roots.iter().take(12) {
        let pct = 100.0 * **count as f32 / chords.len() as f32;
        let bar = "█".repeat((pct / 2.0) as usize);
        println!("  {:2} ({:>2}): {:>4} ({:>5.1}%) {}", pitch_name(**root), root, count, pct, bar);
    }

    // Interval vocabulary in chords
    println!("\n═══ INTERVAL VOCABULARY IN CHORDS ═══\n");

    let mut interval_counts: HashMap<u8, usize> = HashMap::new();
    for chord in &chords {
        for interval in &chord.intervals {
            *interval_counts.entry(*interval).or_insert(0) += 1;
        }
    }

    let mut intervals: Vec<_> = interval_counts.iter().collect();
    intervals.sort_by(|a, b| b.1.cmp(a.1));

    println!("Most used intervals within chords:");
    for (interval, count) in intervals.iter().take(10) {
        let pct = 100.0 * **count as f32 / chords.len() as f32;
        println!("  {:>8} ({:>2} st): {:>4} ({:>5.1}%)",
            interval_name(**interval as i8), interval, count, pct);
    }

    // Chord qualities
    println!("\n═══ CHORD QUALITY ANALYSIS ═══\n");

    let mut quality_counts: HashMap<String, usize> = HashMap::new();
    for chord in &chords {
        let quality = chord_quality(&chord.intervals);
        *quality_counts.entry(quality).or_insert(0) += 1;
    }

    let mut qualities: Vec<_> = quality_counts.iter().collect();
    qualities.sort_by(|a, b| b.1.cmp(a.1));

    println!("Chord types used:");
    for (quality, count) in qualities.iter().take(15) {
        let pct = 100.0 * **count as f32 / chords.len() as f32;
        println!("  {:30} {:>4} ({:>5.1}%)", quality, count, pct);
    }

    // Root motion analysis
    println!("\n═══ ROOT MOTION PATTERNS ═══\n");

    let mut root_motions: HashMap<i8, usize> = HashMap::new();
    for i in 1..chords.len() {
        if chords[i].measure == chords[i-1].measure ||
           chords[i].measure == chords[i-1].measure + 1 {
            let motion = (chords[i].root as i8 - chords[i-1].root as i8 + 18) % 12 - 6;
            // Normalize to -6 to +5 range
            let normalized = if motion > 6 { motion - 12 } else if motion < -6 { motion + 12 } else { motion };
            *root_motions.entry(normalized).or_insert(0) += 1;
        }
    }

    let mut motions: Vec<_> = root_motions.iter().collect();
    motions.sort_by(|a, b| b.1.cmp(a.1));

    println!("Root movement tendencies (semitones):");
    let total_motions: usize = motions.iter().map(|(_, c)| **c).sum();
    for (motion, count) in &motions {
        let pct = 100.0 * **count as f32 / total_motions as f32;
        let direction = match **motion {
            0 => "static      ",
            m if m > 0 => "ascending   ",
            _ => "descending  ",
        };
        let interval = interval_name(**motion);
        let bar = "█".repeat((pct / 3.0) as usize);
        println!("  {:+2} ({:>8}) {}: {:>4} ({:>5.1}%) {}",
            motion, interval, direction, count, pct, bar);
    }

    // Tritone analysis
    println!("\n═══ TRITONE USAGE ═══\n");

    let tritone_chords: Vec<_> = chords.iter()
        .filter(|c| c.intervals.contains(&6))
        .collect();

    println!("Chords containing tritone: {} ({:.1}% of all chords)",
        tritone_chords.len(),
        100.0 * tritone_chords.len() as f32 / chords.len() as f32);

    // Tritone roots
    let mut tritone_roots: HashMap<u8, usize> = HashMap::new();
    for chord in &tritone_chords {
        *tritone_roots.entry(chord.root).or_insert(0) += 1;
    }
    let mut tr: Vec<_> = tritone_roots.iter().collect();
    tr.sort_by(|a, b| b.1.cmp(a.1));

    if !tr.is_empty() {
        println!("\nTritone chord roots:");
        for (root, count) in tr.iter().take(5) {
            println!("  {}: {}", pitch_name(**root), count);
        }
    }

    // Tonal center detection
    println!("\n═══ TONAL CENTER ANALYSIS ═══\n");

    // Use pitch class distribution weighted by position
    let mut pc_weights: HashMap<u8, f32> = HashMap::new();
    for chord in &chords {
        // Root gets more weight
        *pc_weights.entry(chord.root).or_insert(0.0) += 2.0;
        // Other notes get less
        for pc in &chord.pitch_classes {
            if *pc != chord.root {
                *pc_weights.entry(*pc).or_insert(0.0) += 0.5;
            }
        }
    }

    let mut pcs: Vec<_> = pc_weights.iter().collect();
    pcs.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

    println!("Pitch class gravity (weighted by structural importance):");
    let max_weight = pcs.first().map(|(_, w)| **w).unwrap_or(1.0);
    for (pc, weight) in pcs.iter().take(12) {
        let normalized = **weight / max_weight;
        let bar = "█".repeat((normalized * 20.0) as usize);
        println!("  {:>2}: {:>6.1} {}", pitch_name(**pc), weight, bar);
    }

    // Check for tonal ambiguity
    if pcs.len() >= 2 {
        let top = pcs[0].1;
        let second = pcs[1].1;
        let ratio = second / top;

        println!("\nTonal center clarity: {:.2}", 1.0 - ratio);
        if ratio > 0.85 {
            println!("→ HIGHLY AMBIGUOUS: Multiple competing centers ({} vs {})",
                pitch_name(*pcs[0].0), pitch_name(*pcs[1].0));
        } else if ratio > 0.7 {
            println!("→ MODERATELY AMBIGUOUS: Secondary center present");
        } else {
            println!("→ Relatively clear center on {}", pitch_name(*pcs[0].0));
        }
    }

    // Chromatic vs diatonic analysis
    println!("\n═══ CHROMATIC DENSITY ═══\n");

    let unique_pcs: std::collections::HashSet<u8> = chords.iter()
        .flat_map(|c| c.pitch_classes.iter().copied())
        .collect();

    println!("Unique pitch classes used: {} / 12", unique_pcs.len());
    println!("Pitch classes: {:?}",
        unique_pcs.iter().map(|p| pitch_name(*p)).collect::<Vec<_>>());

    if unique_pcs.len() >= 10 {
        println!("\n→ CHROMATIC SATURATION: You use nearly the full chromatic set");
        println!("   This suggests non-diatonic, chromatic harmony");
    }

    // Chord progression patterns (common sequences)
    println!("\n═══ PROGRESSION PATTERNS ═══\n");

    // Look for repeated root sequences
    let mut trigrams: HashMap<(u8, u8, u8), usize> = HashMap::new();
    for i in 2..chords.len() {
        if chords[i].measure <= chords[i-2].measure + 2 {
            let seq = (chords[i-2].root, chords[i-1].root, chords[i].root);
            *trigrams.entry(seq).or_insert(0) += 1;
        }
    }

    let mut seqs: Vec<_> = trigrams.iter().filter(|(_, c)| **c >= 3).collect();
    seqs.sort_by(|a, b| b.1.cmp(a.1));

    if !seqs.is_empty() {
        println!("Recurring 3-chord patterns:");
        for ((a, b, c), count) in seqs.iter().take(10) {
            println!("  {} → {} → {}: {} times",
                pitch_name(*a), pitch_name(*b), pitch_name(*c), count);
        }
    } else {
        println!("No strongly recurring 3-chord patterns found.");
        println!("→ This suggests through-composed or non-repetitive harmonic structure");
    }

    // Motion tendency summary
    println!("\n═══ HARMONIC MOTION SUMMARY ═══\n");

    let static_motion = root_motions.get(&0).copied().unwrap_or(0);
    let chromatic_motion = root_motions.get(&1).copied().unwrap_or(0) +
                          root_motions.get(&-1).copied().unwrap_or(0);
    let tritone_motion = root_motions.get(&6).copied().unwrap_or(0) +
                        root_motions.get(&-6).copied().unwrap_or(0);
    let fourth_fifth = root_motions.get(&5).copied().unwrap_or(0) +
                      root_motions.get(&-5).copied().unwrap_or(0) +
                      root_motions.get(&7).copied().unwrap_or(0);

    println!("Motion type breakdown:");
    println!("  Static (pedal):     {:>4} ({:>5.1}%)", static_motion,
        100.0 * static_motion as f32 / total_motions as f32);
    println!("  Chromatic (±1 st):  {:>4} ({:>5.1}%)", chromatic_motion,
        100.0 * chromatic_motion as f32 / total_motions as f32);
    println!("  Tritone (±6 st):    {:>4} ({:>5.1}%)", tritone_motion,
        100.0 * tritone_motion as f32 / total_motions as f32);
    println!("  4th/5th (±5,±7 st): {:>4} ({:>5.1}%)", fourth_fifth,
        100.0 * fourth_fifth as f32 / total_motions as f32);

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║     SYNTHESIS                                                    ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");
}
