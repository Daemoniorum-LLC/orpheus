//! Live interactive synthesizer demo
//!
//! Play instruments in real-time using keyboard controls.
//!
//! Usage:
//!   cargo run --example live_play --features audio-output
//!
//! Controls:
//!   Row 1 (z-m): Play notes (C4-B4)
//!   Row 2 (a-k): Play notes (C5-B5)
//!   1-9: Select instrument
//!   +/-: Adjust volume
//!   Space: Stop all notes
//!   Q: Quit

use orpheus_synth::{
    instrument_router::{InstrumentType, MultiInstrumentEngine},
    orchestra_renderer::OrchestraRenderer,
    guitar::GuitarConfig,
};
use std::io::{self, Read, Write};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

#[cfg(feature = "audio-output")]
use orpheus_synth::output::AudioOutput;

const SAMPLE_RATE: f32 = 48000.0;
const BUFFER_SIZE: usize = 512;

fn main() {
    println!("=== Orpheus Live Synth ===\n");

    #[cfg(not(feature = "audio-output"))]
    {
        println!("Error: audio-output feature required!");
        println!("Run with: cargo run --example live_play --features audio-output");
        return;
    }

    #[cfg(feature = "audio-output")]
    run_live();
}

#[cfg(feature = "audio-output")]
fn run_live() {
    // Set up audio output
    let output = match AudioOutput::with_buffer_size(BUFFER_SIZE * 8) {
        Ok(o) => o,
        Err(e) => {
            println!("Failed to initialize audio: {}", e);
            println!("Make sure you have audio output available.");
            return;
        }
    };

    let actual_sample_rate = output.sample_rate() as f32;
    println!("Audio initialized: {} Hz", actual_sample_rate);

    // Create renderer with studio preset (less reverb for immediate response)
    let renderer = Arc::new(parking_lot::Mutex::new(OrchestraRenderer::studio(actual_sample_rate)));

    // Add initial instrument
    {
        let mut r = renderer.lock();
        r.add_track(1, "Piano".to_string(), InstrumentType::GrandPiano);
        r.set_reverb_mix(0.15);
    }

    println!("\nInstruments:");
    println!("  1: Grand Piano");
    println!("  2: Electric Piano");
    println!("  3: Acoustic Guitar");
    println!("  4: Electric Guitar");
    println!("  5: Violin");
    println!("  6: Trumpet");
    println!("  7: Flute");
    println!("  8: Organ");
    println!("  9: Choir");

    println!("\nControls:");
    println!("  z x c v b n m ,  →  C4 D4 E4 F4 G4 A4 B4 C5");
    println!("  a s d f g h j k  →  C5 D5 E5 F5 G5 A5 B5 C6");
    println!("  +/- : Volume");
    println!("  Space: Stop all");
    println!("  q: Quit\n");

    // Shared state
    let running = Arc::new(AtomicBool::new(true));
    let running_audio = Arc::clone(&running);
    let renderer_audio = Arc::clone(&renderer);

    // Audio generation thread
    let audio_handle = thread::spawn(move || {
        let mut buffer_l = vec![0.0f32; BUFFER_SIZE];
        let mut buffer_r = vec![0.0f32; BUFFER_SIZE];

        while running_audio.load(Ordering::Relaxed) {
            // Generate audio
            {
                let mut r = renderer_audio.lock();
                let (left, right) = r.process(BUFFER_SIZE);
                buffer_l.copy_from_slice(&left);
                buffer_r.copy_from_slice(&right);
            }

            // Interleave for stereo output
            let mut interleaved = Vec::with_capacity(BUFFER_SIZE * 2);
            for i in 0..BUFFER_SIZE {
                interleaved.push(buffer_l[i]);
                interleaved.push(buffer_r[i]);
            }

            if let Err(_) = output.send_samples(interleaved) {
                break;
            }

            // Small sleep to prevent spinning too fast
            thread::sleep(Duration::from_micros(100));
        }
    });

    // Set terminal to raw mode for immediate key input
    let _raw_mode = RawMode::enable();

    let mut current_instrument = 1u8;
    let mut volume = 0.8f32;
    let mut active_notes: Vec<u8> = Vec::new();

    // Input loop
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut buf = [0u8; 1];

    println!("Ready! Press keys to play...\n");

    while running.load(Ordering::Relaxed) {
        if input.read(&mut buf).is_ok() {
            let key = buf[0] as char;

            // Map keys to MIDI notes
            let note = match key {
                // Lower row: C4-C5
                'z' => Some(60), // C4
                'x' => Some(62), // D4
                'c' => Some(64), // E4
                'v' => Some(65), // F4
                'b' => Some(67), // G4
                'n' => Some(69), // A4
                'm' => Some(71), // B4
                ',' => Some(72), // C5

                // Middle row: C5-C6
                'a' => Some(72), // C5
                's' => Some(74), // D5
                'd' => Some(76), // E5
                'f' => Some(77), // F5
                'g' => Some(79), // G5
                'h' => Some(81), // A5
                'j' => Some(83), // B5
                'k' => Some(84), // C6

                _ => None,
            };

            if let Some(midi_note) = note {
                let mut r = renderer.lock();
                r.note_on(1, midi_note, volume);
                active_notes.push(midi_note);
                print!("♪ ");
                io::stdout().flush().ok();
            }

            // Instrument selection
            match key {
                '1' => change_instrument(&renderer, 1, "Grand Piano", InstrumentType::GrandPiano, &mut current_instrument),
                '2' => change_instrument(&renderer, 2, "Electric Piano", InstrumentType::ElectricPiano, &mut current_instrument),
                '3' => change_instrument(&renderer, 3, "Acoustic Guitar", InstrumentType::AcousticGuitar(GuitarConfig::default()), &mut current_instrument),
                '4' => change_instrument(&renderer, 4, "Electric Guitar", InstrumentType::ElectricGuitar(GuitarConfig::default()), &mut current_instrument),
                '5' => change_instrument(&renderer, 5, "Violin", InstrumentType::Violin, &mut current_instrument),
                '6' => change_instrument(&renderer, 6, "Trumpet", InstrumentType::Trumpet, &mut current_instrument),
                '7' => change_instrument(&renderer, 7, "Flute", InstrumentType::Flute, &mut current_instrument),
                '8' => change_instrument(&renderer, 8, "Organ", InstrumentType::Organ, &mut current_instrument),
                '9' => change_instrument(&renderer, 9, "Choir", InstrumentType::FullChoir, &mut current_instrument),
                _ => {}
            }

            // Volume control
            match key {
                '+' | '=' => {
                    volume = (volume + 0.1).min(1.0);
                    println!("\rVolume: {:.0}%  ", volume * 100.0);
                }
                '-' | '_' => {
                    volume = (volume - 0.1).max(0.1);
                    println!("\rVolume: {:.0}%  ", volume * 100.0);
                }
                _ => {}
            }

            // Stop all notes
            if key == ' ' {
                let mut r = renderer.lock();
                for &note in &active_notes {
                    r.note_off(1, note);
                }
                active_notes.clear();
                println!("\r[silence]    ");
            }

            // Quit
            if key == 'q' || key == 'Q' || buf[0] == 3 /* Ctrl-C */ {
                running.store(false, Ordering::Relaxed);
                println!("\n\nGoodbye!");
            }
        }
    }

    // Clean up
    drop(_raw_mode);
    audio_handle.join().ok();
}

#[cfg(feature = "audio-output")]
fn change_instrument(
    renderer: &Arc<parking_lot::Mutex<OrchestraRenderer>>,
    id: u8,
    name: &str,
    instrument: InstrumentType,
    current: &mut u8,
) {
    if *current != id {
        let mut r = renderer.lock();
        // Remove old track and add new one
        r.remove_track(1);
        r.add_track(1, name.to_string(), instrument);
        *current = id;
        println!("\r[{}]          ", name);
    }
}

/// RAII wrapper for terminal raw mode
struct RawMode {
    #[cfg(unix)]
    original: libc::termios,
}

impl RawMode {
    #[cfg(unix)]
    fn enable() -> Option<Self> {
        use std::mem::MaybeUninit;
        use std::os::unix::io::AsRawFd;

        unsafe {
            let fd = io::stdin().as_raw_fd();
            let mut termios = MaybeUninit::<libc::termios>::uninit();

            if libc::tcgetattr(fd, termios.as_mut_ptr()) != 0 {
                return None;
            }

            let original = termios.assume_init();
            let mut raw = original;

            // Disable canonical mode and echo
            raw.c_lflag &= !(libc::ICANON | libc::ECHO);
            // Set minimum bytes to read
            raw.c_cc[libc::VMIN] = 1;
            raw.c_cc[libc::VTIME] = 0;

            if libc::tcsetattr(fd, libc::TCSANOW, &raw) != 0 {
                return None;
            }

            Some(Self { original })
        }
    }

    #[cfg(not(unix))]
    fn enable() -> Option<Self> {
        // Windows would need different handling
        Some(Self {})
    }
}

impl Drop for RawMode {
    #[cfg(unix)]
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;
        unsafe {
            let fd = io::stdin().as_raw_fd();
            libc::tcsetattr(fd, libc::TCSANOW, &self.original);
        }
    }

    #[cfg(not(unix))]
    fn drop(&mut self) {}
}
