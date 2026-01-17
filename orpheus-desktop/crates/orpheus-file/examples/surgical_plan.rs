//! Surgical analysis of sections that need rewriting
//! Generate target parameters based on composer's fingerprint

use orpheus_file::guitar_pro::{self, GuitarProFile};
use std::collections::HashMap;

fn main() {
    let path = "/home/user/workspace/music/Sheet Music/Guitar Pro/Deicidal Carnage/A Calamitous Orchestration Revised (1).gp";

    match guitar_pro::parse_file(path) {
        Ok(gp) => surgical_plan(&gp),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn surgical_plan(gp: &GuitarProFile) {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  SURGICAL REWRITE PLAN: A Calamitous Orchestration               ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Define the problem sections
    let problem_sections = vec![
        (72, 144, "Primary deviation - M solo setup"),
        (220, 263, "Secondary deviation"),
    ];

    // Calculate YOUR fingerprint from the good sections
    println!("═══ STEP 1: YOUR COMPOSITIONAL FINGERPRINT ═══\n");
    println!("Calculated from measures 1-71, 145-219, 264+ (your authentic voice)\n");

    let good_measures: Vec<_> = gp.measures.iter()
        .filter(|m| {
            let n = m.number;
            (n >= 1 && n <= 71) || (n >= 145 && n <= 219) || n >= 264
        })
        .collect();

    let (your_variance, your_density, your_dissonance, your_root_dist, your_intervals) =
        calculate_fingerprint(&good_measures, gp);

    println!("  Your baseline metrics:");
    println!("  ┌────────────────────────────────────────┐");
    println!("  │ Fret Variance:     {:>6.1}              │", your_variance);
    println!("  │ Note Density:      {:>6.2} notes/beat   │", your_density);
    println!("  │ Dissonance Index:  {:>6.2}              │", your_dissonance);
    println!("  └────────────────────────────────────────┘");

    println!("\n  Your interval signature:");
    for (interval, pct) in your_intervals.iter().take(5) {
        println!("    {:>8}: {:>5.1}%", interval_name(*interval), pct);
    }

    println!("\n  Your root preferences:");
    for (root, pct) in your_root_dist.iter().take(4) {
        println!("    {:>2}: {:>5.1}%", pitch_name(*root), pct);
    }

    // Analyze each problem section
    for (start, end, label) in &problem_sections {
        println!("\n═══ PROBLEM SECTION: Measures {}-{} ═══", start, end);
        println!("    {}\n", label);

        let problem_measures: Vec<_> = gp.measures.iter()
            .filter(|m| m.number >= *start && m.number <= *end)
            .collect();

        let (prob_variance, prob_density, prob_dissonance, prob_roots, prob_intervals) =
            calculate_fingerprint(&problem_measures, gp);

        println!("  Current state:");
        println!("  ┌────────────────────────────────────────────────────────────┐");
        println!("  │ Metric           │ Current  │ Target   │ Delta            │");
        println!("  ├────────────────────────────────────────────────────────────┤");

        let var_delta = ((prob_variance - your_variance) / your_variance * 100.0) as i32;
        let den_delta = ((prob_density - your_density) / your_density * 100.0) as i32;
        let dis_delta = ((prob_dissonance - your_dissonance) / your_dissonance.max(0.01) * 100.0) as i32;

        println!("  │ Fret Variance    │ {:>6.1}   │ {:>6.1}   │ {:>+4}% {}│",
            prob_variance, your_variance, var_delta,
            if var_delta.abs() > 50 { "⚠️ " } else { "   " });
        println!("  │ Note Density     │ {:>6.2}   │ {:>6.2}   │ {:>+4}% {}│",
            prob_density, your_density, den_delta,
            if den_delta.abs() > 50 { "⚠️ " } else { "   " });
        println!("  │ Dissonance       │ {:>6.2}   │ {:>6.2}   │ {:>+4}% {}│",
            prob_dissonance, your_dissonance, dis_delta,
            if dis_delta.abs() > 50 { "⚠️ " } else { "   " });
        println!("  └────────────────────────────────────────────────────────────┘");

        // Specific issues
        println!("\n  Issues detected:");

        if var_delta < -50 {
            println!("  ❌ VARIANCE TOO LOW: Writing is formulaic/repetitive");
            println!("     → Add more fret position variety");
            println!("     → Break up repeated patterns with register shifts");
        }

        if prob_dissonance < your_dissonance * 0.7 {
            println!("  ❌ DISSONANCE TOO CLEAN: Missing your harmonic tension");
            println!("     → Add more M7 intervals (semitone rub)");
            println!("     → Incorporate tritone relationships");
        }

        // Check root motion
        println!("\n  Root motion comparison:");
        println!("    Current section roots:");
        for (root, pct) in prob_roots.iter().take(3) {
            let target = your_root_dist.iter()
                .find(|(r, _)| r == root)
                .map(|(_, p)| *p)
                .unwrap_or(0.0);
            let diff = *pct - target;
            println!("      {:>2}: {:>5.1}% (target: {:>5.1}%, {:>+5.1}%)",
                pitch_name(*root), pct, target, diff);
        }

        // Prescription
        println!("\n  ┌─────────────────────────────────────────────────────────────┐");
        println!("  │ REWRITE PRESCRIPTION                                        │");
        println!("  ├─────────────────────────────────────────────────────────────┤");

        if var_delta < -50 {
            println!("  │ 1. Increase fret variance from {:.1} → {:.1}+             │",
                prob_variance, your_variance * 0.8);
            println!("  │    Break repetitive patterns, add register exploration     │");
        }

        println!("  │ 2. Maintain E↔F oscillation as primary root motion          │");
        println!("  │ 3. Stack M7 intervals for your signature tension            │");

        if *start == 72 {
            println!("  │ 4. This section sets up M solo - build tension TO it,      │");
            println!("  │    don't just chug underneath it                           │");
        }

        println!("  └─────────────────────────────────────────────────────────────┘");
    }

    // Generate a "what good looks like" reference
    println!("\n═══ REFERENCE: Your Best Sections ═══\n");

    // Find highest variance measures (your most "you" writing)
    let mut measure_scores: Vec<(u16, f32)> = Vec::new();

    for measure in &gp.measures {
        let mut frets: Vec<u8> = Vec::new();
        for tb in &measure.beats {
            for beat in &tb.beats {
                for note in &beat.notes {
                    frets.push(note.fret);
                }
            }
        }

        if frets.len() >= 4 {
            let mean = frets.iter().map(|f| *f as f32).sum::<f32>() / frets.len() as f32;
            let variance = frets.iter()
                .map(|f| (*f as f32 - mean).powi(2))
                .sum::<f32>() / frets.len() as f32;
            measure_scores.push((measure.number, variance));
        }
    }

    measure_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("  Highest variance measures (most textural exploration):");
    for (measure, variance) in measure_scores.iter().take(10) {
        // Check if in problem section
        let in_problem = problem_sections.iter()
            .any(|(s, e, _)| *measure >= *s && *measure <= *e);
        let marker = if in_problem { " (in problem section)" } else { "" };
        println!("    Measure {:>3}: variance {:.1}{}", measure, variance, marker);
    }

    println!("\n  Use these as reference for textural density when rewriting.");

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  SUMMARY: {} measures need rewriting                       ║",
        problem_sections.iter().map(|(s, e, _)| e - s + 1).sum::<u16>());
    println!("║  Primary issue: Variance/texture collapse (formulaic writing)    ║");
    println!("║  Your voice: High variance, E↔F oscillation, M7 tension          ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");
}

fn calculate_fingerprint(measures: &[&guitar_pro::Measure], gp: &GuitarProFile)
    -> (f32, f32, f32, Vec<(u8, f32)>, Vec<(u8, f32)>)
{
    let mut all_frets: Vec<u8> = Vec::new();
    let mut note_count = 0usize;
    let mut beat_count = 0usize;
    let mut intervals: HashMap<u8, usize> = HashMap::new();
    let mut roots: HashMap<u8, usize> = HashMap::new();

    let tuning = gp.tracks.first()
        .map(|t| &t.tuning)
        .cloned()
        .unwrap_or_else(|| vec![64, 59, 55, 50, 45, 40]);

    for measure in measures {
        for tb in &measure.beats {
            for beat in &tb.beats {
                beat_count += 1;
                note_count += beat.notes.len();

                for note in &beat.notes {
                    all_frets.push(note.fret);
                }

                if beat.notes.len() >= 2 {
                    let mut pitches: Vec<u8> = beat.notes.iter()
                        .filter_map(|n| {
                            let idx = (n.string as usize).saturating_sub(1);
                            tuning.get(idx).map(|open| open + n.fret)
                        })
                        .collect();
                    pitches.sort();

                    if let Some(&root) = pitches.first() {
                        *roots.entry(root % 12).or_insert(0) += 1;
                    }

                    for i in 0..pitches.len().saturating_sub(1) {
                        let interval = ((pitches[i + 1] as i16 - pitches[i] as i16).abs() % 12) as u8;
                        if interval > 0 {
                            *intervals.entry(interval).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    let variance = if !all_frets.is_empty() {
        let mean = all_frets.iter().map(|f| *f as f32).sum::<f32>() / all_frets.len() as f32;
        all_frets.iter().map(|f| (*f as f32 - mean).powi(2)).sum::<f32>() / all_frets.len() as f32
    } else { 0.0 };

    let density = if beat_count > 0 {
        note_count as f32 / beat_count as f32
    } else { 0.0 };

    // Dissonance = weighted sum of dissonant intervals
    let total_intervals: usize = intervals.values().sum();
    let dissonance = if total_intervals > 0 {
        intervals.iter().map(|(i, c)| {
            let weight = match *i {
                1 | 11 => 3.0,  // m2, M7
                6 => 2.5,       // tritone
                2 | 10 => 1.5,  // M2, m7
                _ => 0.5,
            };
            weight * (*c as f32)
        }).sum::<f32>() / total_intervals as f32
    } else { 0.0 };

    // Convert to sorted percentages
    let total_roots: usize = roots.values().sum();
    let mut root_vec: Vec<(u8, f32)> = roots.iter()
        .map(|(k, v)| (*k, 100.0 * *v as f32 / total_roots.max(1) as f32))
        .collect();
    root_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut interval_vec: Vec<(u8, f32)> = intervals.iter()
        .map(|(k, v)| (*k, 100.0 * *v as f32 / total_intervals.max(1) as f32))
        .collect();
    interval_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    (variance, density, dissonance, root_vec, interval_vec)
}

fn pitch_name(pc: u8) -> &'static str {
    match pc % 12 {
        0 => "C", 1 => "C#", 2 => "D", 3 => "D#", 4 => "E", 5 => "F",
        6 => "F#", 7 => "G", 8 => "G#", 9 => "A", 10 => "A#", 11 => "B",
        _ => "?"
    }
}

fn interval_name(semitones: u8) -> &'static str {
    match semitones % 12 {
        0 => "unison", 1 => "m2", 2 => "M2", 3 => "m3", 4 => "M3",
        5 => "P4", 6 => "tritone", 7 => "P5", 8 => "m6", 9 => "M6",
        10 => "m7", 11 => "M7",
        _ => "?"
    }
}
