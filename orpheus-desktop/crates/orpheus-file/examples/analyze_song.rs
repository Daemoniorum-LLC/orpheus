//! Deep analysis of a Guitar Pro file for compositional feedback

use orpheus_file::guitar_pro;
use std::collections::HashMap;
use std::env;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        "/home/user/workspace/music/Sheet Music/Guitar Pro/Deicidal Carnage/A Calamitous Orchestration Revised (1).gp".into()
    });

    println!("=== COMPOSITIONAL ANALYSIS ===\n");

    match guitar_pro::parse_file(&path) {
        Ok(gp) => {
            let title = if gp.info.title.is_empty() { "Untitled" } else { &gp.info.title };
            println!("Track: {}", title);
            println!("Tempo: {} BPM", gp.tempo);
            println!("Time Signature: {}/{}", gp.time_signature.numerator, gp.time_signature.denominator);
            println!("Duration: {} measures (~{:.1} minutes at {} BPM)",
                gp.measures.len(),
                (gp.measures.len() as f64 * 4.0) / (gp.tempo as f64),
                gp.tempo
            );
            println!();

            // Track analysis
            println!("=== TRACK ARRANGEMENT ===");
            for track in &gp.tracks {
                let track_notes: usize = gp.measures.iter()
                    .flat_map(|m| &m.beats)
                    .filter(|tb| tb.track == track.number)
                    .flat_map(|tb| &tb.beats)
                    .map(|b| b.notes.len())
                    .sum();
                let track_beats: usize = gp.measures.iter()
                    .flat_map(|m| &m.beats)
                    .filter(|tb| tb.track == track.number)
                    .flat_map(|tb| &tb.beats)
                    .count();

                // Calculate average fret position
                let frets: Vec<u8> = gp.measures.iter()
                    .flat_map(|m| &m.beats)
                    .filter(|tb| tb.track == track.number)
                    .flat_map(|tb| &tb.beats)
                    .flat_map(|b| &b.notes)
                    .map(|n| n.fret)
                    .collect();
                let avg_fret = if !frets.is_empty() {
                    frets.iter().map(|f| *f as f64).sum::<f64>() / frets.len() as f64
                } else { 0.0 };

                let highest_fret = frets.iter().max().copied().unwrap_or(0);

                println!("  {} [{}]: {} notes, {} beats, avg fret {:.1}, highest fret {}",
                    track.number, track.name, track_notes, track_beats, avg_fret, highest_fret);
            }
            println!();

            // Measure density analysis
            println!("=== SECTION DENSITY ANALYSIS ===");
            let mut measure_densities: Vec<(usize, f64, Option<String>)> = Vec::new();

            for (idx, measure) in gp.measures.iter().enumerate() {
                let notes: usize = measure.beats.iter()
                    .flat_map(|tb| &tb.beats)
                    .map(|b| b.notes.len())
                    .sum();
                let beats = measure.beats.iter()
                    .flat_map(|tb| &tb.beats)
                    .count().max(1);
                let density = notes as f64 / beats as f64;
                measure_densities.push((idx + 1, density, measure.marker.clone()));
            }

            // Find peaks (high density sections)
            let avg_density: f64 = measure_densities.iter().map(|m| m.1).sum::<f64>() / measure_densities.len() as f64;
            let max_density = measure_densities.iter().map(|m| m.1).fold(0.0f64, |a, b| a.max(b));
            let min_density = measure_densities.iter().map(|m| m.1).fold(f64::MAX, |a, b| a.min(b));

            println!("Average density: {:.2} notes/beat", avg_density);
            println!("Peak density:    {:.2} notes/beat", max_density);
            println!("Minimum density: {:.2} notes/beat", min_density);
            println!();

            // Show sections by marker
            println!("=== SECTIONS (by markers) ===");
            let markers: Vec<_> = measure_densities.iter()
                .filter(|m| m.2.is_some())
                .collect();

            for i in 0..markers.len() {
                let (start, _, name) = markers[i];
                let end = if i + 1 < markers.len() { markers[i + 1].0 - 1 } else { gp.measures.len() };
                let section_measures: Vec<_> = measure_densities.iter()
                    .filter(|m| m.0 >= *start && m.0 <= end)
                    .collect();
                let section_avg = section_measures.iter().map(|m| m.1).sum::<f64>() / section_measures.len() as f64;
                let section_max = section_measures.iter().map(|m| m.1).fold(0.0f64, |a, b| a.max(b));

                println!("  {} (m{}-{}, {} bars): avg {:.2}, peak {:.2}",
                    name.as_ref().unwrap(), start, end, end - start + 1, section_avg, section_max);
            }
            println!();

            // Fret position analysis (register usage)
            println!("=== REGISTER ANALYSIS ===");
            let all_frets: Vec<u8> = gp.measures.iter()
                .flat_map(|m| &m.beats)
                .flat_map(|tb| &tb.beats)
                .flat_map(|b| &b.notes)
                .map(|n| n.fret)
                .collect();

            let mut fret_histogram: HashMap<u8, usize> = HashMap::new();
            for fret in &all_frets {
                *fret_histogram.entry(*fret).or_insert(0) += 1;
            }

            // Group by register
            let low = all_frets.iter().filter(|f| **f <= 5).count();
            let mid = all_frets.iter().filter(|f| **f > 5 && **f <= 12).count();
            let high = all_frets.iter().filter(|f| **f > 12).count();
            let total = all_frets.len() as f64;

            println!("  Low register (0-5):   {:5} notes ({:5.1}%)", low, 100.0 * low as f64 / total);
            println!("  Mid register (6-12):  {:5} notes ({:5.1}%)", mid, 100.0 * mid as f64 / total);
            println!("  High register (13+):  {:5} notes ({:5.1}%)", high, 100.0 * high as f64 / total);
            println!();

            // Most used frets
            let mut fret_counts: Vec<_> = fret_histogram.iter().collect();
            fret_counts.sort_by(|a, b| b.1.cmp(a.1));
            println!("  Most used frets:");
            for (fret, count) in fret_counts.iter().take(5) {
                println!("    Fret {:2}: {:5} notes ({:.1}%)", fret, count, 100.0 * **count as f64 / total);
            }
            println!();

            // String usage
            println!("=== STRING USAGE ===");
            let mut string_counts: HashMap<u8, usize> = HashMap::new();
            for note in gp.measures.iter()
                .flat_map(|m| &m.beats)
                .flat_map(|tb| &tb.beats)
                .flat_map(|b| &b.notes)
            {
                *string_counts.entry(note.string).or_insert(0) += 1;
            }

            let strings = ["E (high)", "B", "G", "D", "A", "E (low)"];
            for s in 1..=6 {
                let count = string_counts.get(&s).copied().unwrap_or(0);
                println!("  String {} ({}): {:5} notes ({:5.1}%)",
                    s, strings.get(s as usize - 1).unwrap_or(&"?"), count, 100.0 * count as f64 / total);
            }
            println!();

            // Identify characteristics
            println!("=== COMPOSITIONAL OBSERVATIONS ===");

            // High BPM check
            if gp.tempo >= 200 {
                println!("  ⚡ BLAZING tempo ({} BPM) - characteristic of tech death/thrash", gp.tempo);
            } else if gp.tempo >= 160 {
                println!("  🔥 Fast tempo ({} BPM)", gp.tempo);
            }

            // Density check
            if avg_density > 1.5 {
                println!("  💀 HIGH note density ({:.2} notes/beat avg) - consistent intensity", avg_density);
            }

            // Low register dominance
            if low as f64 / total > 0.5 {
                println!("  🎸 LOW REGISTER dominant ({:.1}%) - emphasis on chugging/riffs", 100.0 * low as f64 / total);
            }

            // Open string usage
            let open_frets = fret_histogram.get(&0).copied().unwrap_or(0);
            if open_frets as f64 / total > 0.3 {
                println!("  🎵 Heavy OPEN STRING usage ({:.1}%) - pedal tone/drone approach", 100.0 * open_frets as f64 / total);
            }

            // Low string emphasis
            let low_strings = string_counts.get(&5).copied().unwrap_or(0) + string_counts.get(&6).copied().unwrap_or(0);
            if low_strings as f64 / total > 0.5 {
                println!("  🔊 LOW STRING dominant ({:.1}%) - crushing bass frequencies", 100.0 * low_strings as f64 / total);
            }

            // Section balance
            if markers.len() >= 3 {
                println!("  📐 Structured composition with {} marked sections", markers.len());
            }

            // Track count
            if gp.tracks.len() >= 4 {
                println!("  🎭 Multi-layered arrangement ({} guitar tracks)", gp.tracks.len());
            }

            // Calculate the empty measures (very low density)
            let sparse_measures = measure_densities.iter().filter(|m| m.1 < 0.5).count();
            let sparse_pct = 100.0 * sparse_measures as f64 / measure_densities.len() as f64;
            if sparse_pct > 20.0 {
                println!("  ⏸️  Dynamic contrast: {:.1}% sparse measures (breathing room)", sparse_pct);
            } else if sparse_pct < 5.0 {
                println!("  ⚔️  RELENTLESS: Only {:.1}% sparse measures - no mercy", sparse_pct);
            }

            println!("\n✓ Analysis complete");
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}
