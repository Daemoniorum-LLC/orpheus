//! Detect stylistic anomalies in a composition
//! Looking for the "foreign" section that doesn't match the composer's voice

use orpheus_file::guitar_pro;
use std::collections::HashMap;

fn main() {
    let path = "/home/user/workspace/music/Sheet Music/Guitar Pro/Deicidal Carnage/A Calamitous Orchestration Revised (1).gp";

    match guitar_pro::parse_file(path) {
        Ok(gp) => find_anomalies(&gp),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn find_anomalies(gp: &guitar_pro::GuitarProFile) {
    println!("═══ STYLISTIC ANOMALY DETECTION ═══\n");

    // First, find where M solo appears
    println!("1. LOCATING 'M solo' TRACK ACTIVITY\n");

    let m_solo_track = gp.tracks.iter().find(|t| t.name.contains("M solo"));
    if let Some(track) = m_solo_track {
        println!("   M solo is track #{}", track.number);

        // Find measures where M solo has notes
        let m_solo_measures: Vec<u16> = gp.measures.iter()
            .filter(|m| {
                m.beats.iter()
                    .filter(|tb| tb.track == track.number)
                    .flat_map(|tb| &tb.beats)
                    .any(|b| !b.notes.is_empty())
            })
            .map(|m| m.number)
            .collect();

        if !m_solo_measures.is_empty() {
            let first = m_solo_measures.first().unwrap();
            let last = m_solo_measures.last().unwrap();
            println!("   M solo appears in measures: {}-{} ({} measures)", first, last, m_solo_measures.len());
            println!("   Setup section likely: measures {}-{}", first.saturating_sub(20), first);
        }
    }

    // Analyze measure-by-measure characteristics
    println!("\n2. MEASURE-BY-MEASURE FINGERPRINTING\n");

    #[derive(Debug, Clone)]
    struct MeasureFingerprint {
        number: u16,
        density: f32,
        avg_fret: f32,
        fret_variance: f32,
        interval_dissonance: f32,  // Higher = more dissonant intervals
        rhythmic_complexity: f32,  // Based on note duration variety
        string_spread: f32,        // How many strings used
    }

    let mut fingerprints: Vec<MeasureFingerprint> = Vec::new();

    for measure in &gp.measures {
        let mut all_frets: Vec<u8> = Vec::new();
        let mut all_strings: Vec<u8> = Vec::new();
        let mut intervals: Vec<i8> = Vec::new();
        let mut note_count = 0;
        let mut beat_count = 0;

        for tb in &measure.beats {
            for beat in &tb.beats {
                beat_count += 1;
                note_count += beat.notes.len();

                for note in &beat.notes {
                    all_frets.push(note.fret);
                    all_strings.push(note.string);
                }

                // Calculate intervals within chord
                if beat.notes.len() >= 2 {
                    let mut frets: Vec<i8> = beat.notes.iter().map(|n| n.fret as i8).collect();
                    frets.sort();
                    for i in 0..frets.len() - 1 {
                        intervals.push((frets[i + 1] - frets[i]).abs());
                    }
                }
            }
        }

        let density = if beat_count > 0 { note_count as f32 / beat_count as f32 } else { 0.0 };

        let avg_fret = if !all_frets.is_empty() {
            all_frets.iter().map(|f| *f as f32).sum::<f32>() / all_frets.len() as f32
        } else { 0.0 };

        let fret_variance = if !all_frets.is_empty() {
            let mean = avg_fret;
            all_frets.iter().map(|f| (*f as f32 - mean).powi(2)).sum::<f32>() / all_frets.len() as f32
        } else { 0.0 };

        // Dissonance score: minor 2nds (1) and tritones (6) score high
        let interval_dissonance = if !intervals.is_empty() {
            intervals.iter().map(|i| match *i {
                1 => 3.0,  // minor 2nd - max dissonance
                2 => 1.5,  // major 2nd
                6 => 2.5,  // tritone
                _ => 0.5,
            }).sum::<f32>() / intervals.len() as f32
        } else { 0.0 };

        let unique_strings: std::collections::HashSet<u8> = all_strings.iter().copied().collect();
        let string_spread = unique_strings.len() as f32;

        fingerprints.push(MeasureFingerprint {
            number: measure.number,
            density,
            avg_fret,
            fret_variance,
            interval_dissonance,
            rhythmic_complexity: 0.0, // TODO
            string_spread,
        });
    }

    // Calculate baseline statistics (your "voice")
    let baseline_density: f32 = fingerprints.iter().map(|f| f.density).sum::<f32>() / fingerprints.len() as f32;
    let baseline_fret: f32 = fingerprints.iter().map(|f| f.avg_fret).sum::<f32>() / fingerprints.len() as f32;
    let baseline_variance: f32 = fingerprints.iter().map(|f| f.fret_variance).sum::<f32>() / fingerprints.len() as f32;
    let baseline_dissonance: f32 = fingerprints.iter().map(|f| f.interval_dissonance).sum::<f32>() / fingerprints.len() as f32;

    println!("   BASELINE (your compositional voice):");
    println!("   - Density:    {:.2} notes/beat", baseline_density);
    println!("   - Avg fret:   {:.1}", baseline_fret);
    println!("   - Variance:   {:.1}", baseline_variance);
    println!("   - Dissonance: {:.2}", baseline_dissonance);

    // Find anomalous regions - consecutive measures that deviate
    println!("\n3. ANOMALY DETECTION\n");

    let mut anomaly_scores: Vec<(u16, f32, String)> = Vec::new();

    for fp in &fingerprints {
        let mut score = 0.0;
        let mut reasons = Vec::new();

        // Check for LOW variance (formulaic playing)
        if fp.fret_variance < baseline_variance * 0.3 && fp.density > 0.5 {
            score += 2.0;
            reasons.push(format!("low variance ({:.1} vs {:.1})", fp.fret_variance, baseline_variance));
        }

        // Check for LOW dissonance (too "clean" for tech death)
        if fp.interval_dissonance < baseline_dissonance * 0.5 && fp.density > 0.5 {
            score += 1.5;
            reasons.push(format!("low dissonance ({:.2} vs {:.2})", fp.interval_dissonance, baseline_dissonance));
        }

        // Check for unusual density patterns
        if fp.density > 0.0 && (fp.density < baseline_density * 0.5 || fp.density > baseline_density * 2.0) {
            score += 1.0;
            reasons.push(format!("unusual density ({:.2})", fp.density));
        }

        // Check for limited string spread (less textural variety)
        if fp.string_spread < 2.0 && fp.density > 0.5 {
            score += 1.0;
            reasons.push(format!("limited strings ({:.0})", fp.string_spread));
        }

        if score > 1.5 {
            anomaly_scores.push((fp.number, score, reasons.join(", ")));
        }
    }

    // Group consecutive anomalies into regions
    println!("   Detected anomalous regions:\n");

    if anomaly_scores.is_empty() {
        println!("   No strong anomalies detected.");
    } else {
        // Find consecutive runs
        let mut regions: Vec<(u16, u16, f32, Vec<String>)> = Vec::new();
        let mut current_start = 0u16;
        let mut current_end = 0u16;
        let mut current_score = 0.0;
        let mut current_reasons: Vec<String> = Vec::new();

        for (measure, score, reason) in &anomaly_scores {
            if current_start == 0 {
                current_start = *measure;
                current_end = *measure;
                current_score = *score;
                current_reasons.push(reason.clone());
            } else if *measure <= current_end + 4 {  // Allow gaps of up to 4 measures
                current_end = *measure;
                current_score += *score;
                if !current_reasons.contains(reason) {
                    current_reasons.push(reason.clone());
                }
            } else {
                if current_end - current_start >= 4 {  // Only report regions of 5+ measures
                    regions.push((current_start, current_end, current_score, current_reasons.clone()));
                }
                current_start = *measure;
                current_end = *measure;
                current_score = *score;
                current_reasons = vec![reason.clone()];
            }
        }
        // Don't forget the last region
        if current_end - current_start >= 4 {
            regions.push((current_start, current_end, current_score, current_reasons));
        }

        // Sort by score
        regions.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        for (start, end, score, reasons) in regions.iter().take(5) {
            println!("   ┌─ MEASURES {}-{} (anomaly score: {:.1})", start, end, score);
            println!("   │  Length: {} measures", end - start + 1);
            println!("   │  Issues: {}", reasons.join("; "));

            // Show what's happening in this region
            let region_fp: Vec<_> = fingerprints.iter()
                .filter(|f| f.number >= *start && f.number <= *end)
                .collect();

            let region_density: f32 = region_fp.iter().map(|f| f.density).sum::<f32>() / region_fp.len() as f32;
            let region_variance: f32 = region_fp.iter().map(|f| f.fret_variance).sum::<f32>() / region_fp.len() as f32;
            let region_dissonance: f32 = region_fp.iter().map(|f| f.interval_dissonance).sum::<f32>() / region_fp.len() as f32;

            println!("   │  ");
            println!("   │  Region stats vs baseline:");
            println!("   │    Density:    {:.2} vs {:.2} ({:+.0}%)",
                region_density, baseline_density,
                100.0 * (region_density - baseline_density) / baseline_density);
            println!("   │    Variance:   {:.1} vs {:.1} ({:+.0}%)",
                region_variance, baseline_variance,
                100.0 * (region_variance - baseline_variance) / baseline_variance);
            println!("   │    Dissonance: {:.2} vs {:.2} ({:+.0}%)",
                region_dissonance, baseline_dissonance,
                100.0 * (region_dissonance - baseline_dissonance) / baseline_dissonance);
            println!("   └─\n");
        }
    }

    // Specific analysis around M solo
    println!("\n4. M SOLO SETUP ANALYSIS\n");

    if let Some(track) = m_solo_track {
        let m_solo_first: Option<u16> = gp.measures.iter()
            .filter(|m| {
                m.beats.iter()
                    .filter(|tb| tb.track == track.number)
                    .flat_map(|tb| &tb.beats)
                    .any(|b| !b.notes.is_empty())
            })
            .map(|m| m.number)
            .next();

        if let Some(first) = m_solo_first {
            let setup_start = first.saturating_sub(16);
            let setup_end = first.saturating_sub(1);

            println!("   M solo enters at measure {}", first);
            println!("   Analyzing setup region: measures {}-{}\n", setup_start, setup_end);

            let setup_fps: Vec<_> = fingerprints.iter()
                .filter(|f| f.number >= setup_start && f.number <= setup_end)
                .collect();

            if !setup_fps.is_empty() {
                let setup_variance: f32 = setup_fps.iter().map(|f| f.fret_variance).sum::<f32>() / setup_fps.len() as f32;
                let setup_dissonance: f32 = setup_fps.iter().map(|f| f.interval_dissonance).sum::<f32>() / setup_fps.len() as f32;

                println!("   Setup region characteristics:");
                println!("   - Variance:   {:.1} (baseline: {:.1}) → {:+.0}%",
                    setup_variance, baseline_variance,
                    100.0 * (setup_variance - baseline_variance) / baseline_variance);
                println!("   - Dissonance: {:.2} (baseline: {:.2}) → {:+.0}%",
                    setup_dissonance, baseline_dissonance,
                    100.0 * (setup_dissonance - baseline_dissonance) / baseline_dissonance);

                if setup_variance < baseline_variance * 0.5 {
                    println!("\n   ⚠️  DETECTED: Setup section has significantly LOWER variance");
                    println!("      This suggests more repetitive/formulaic writing");
                }
                if setup_dissonance < baseline_dissonance * 0.7 {
                    println!("\n   ⚠️  DETECTED: Setup section has LOWER harmonic tension");
                    println!("      This suggests \"cleaner\" voicings than your typical style");
                }
            }
        }
    }

    println!("\n═══ END ANALYSIS ═══\n");
}
