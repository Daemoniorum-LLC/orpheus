//! Offline tab document renderer
//!
//! Renders tablature documents to audio samples using guitar synthesis
//! with optional amp simulation for realistic electric guitar tones.

use orpheus_core::tab::TabDocument;
use orpheus_synth::{GuitarSynth, GuitarConfig, AmpSimulator, AmpPreset, CabinetSimulator, CabinetPreset, Reverb};
use tracing::info;

/// Tab playback event for scheduling
#[derive(Debug, Clone)]
struct TabEvent {
    /// Sample position
    sample_pos: u64,
    /// String number (1-indexed)
    string: u8,
    /// Fret number
    fret: u8,
    /// Velocity (0.0 - 1.0)
    velocity: f32,
    /// Whether this is a note-on (true) or note-off/mute (false)
    is_note_on: bool,
}

/// Offline renderer for tab documents
pub struct TabRenderer {
    /// Sample rate
    sample_rate: u32,
    /// Guitar synth
    guitar: GuitarSynth,
    /// Amp simulator (optional)
    amp: Option<AmpSimulator>,
    /// Cabinet simulator (optional, used with amp)
    cabinet: Option<CabinetSimulator>,
    /// Reverb (optional, adds space/ambience)
    reverb: Option<Reverb>,
    /// Master volume (0.0 - 1.0)
    master_volume: f32,
}

impl TabRenderer {
    /// Create a new tab renderer with default settings (no amp)
    pub fn new(sample_rate: u32) -> Self {
        let config = GuitarConfig {
            sample_rate,
            ..GuitarConfig::default()
        };

        Self {
            sample_rate,
            guitar: GuitarSynth::new(config),
            amp: None,
            cabinet: None,
            reverb: None,
            master_volume: 0.8,
        }
    }

    /// Create a new tab renderer with amp and cabinet simulation
    pub fn with_amp(sample_rate: u32) -> Self {
        let config = GuitarConfig {
            sample_rate,
            ..GuitarConfig::default()
        };

        let mut amp = AmpSimulator::new(sample_rate as f32);
        // Default to modern metal preset
        AmpPreset::modern_metal().apply_to(&mut amp);

        let mut cabinet = CabinetSimulator::new(sample_rate as f32);
        CabinetPreset::modern_metal().apply_to(&mut cabinet);

        // Add subtle room reverb for realism
        let reverb = Reverb::small_room(sample_rate);

        Self {
            sample_rate,
            guitar: GuitarSynth::new(config),
            amp: Some(amp),
            cabinet: Some(cabinet),
            reverb: Some(reverb),
            master_volume: 0.8,
        }
    }

    /// Create with bass guitar configuration
    pub fn new_bass(sample_rate: u32) -> Self {
        let base_config = GuitarConfig::bass();
        let config = GuitarConfig { sample_rate, ..base_config };

        Self {
            sample_rate,
            guitar: GuitarSynth::new(config),
            amp: None,
            cabinet: None,
            reverb: None,
            master_volume: 0.8,
        }
    }

    /// Enable amp simulation with a preset
    pub fn set_amp_preset(&mut self, preset: AmpPreset) {
        let amp = self.amp.get_or_insert_with(|| AmpSimulator::new(self.sample_rate as f32));
        preset.apply_to(amp);
    }

    /// Enable cabinet simulation with a preset
    pub fn set_cabinet_preset(&mut self, preset: CabinetPreset) {
        let cab = self.cabinet.get_or_insert_with(|| CabinetSimulator::new(self.sample_rate as f32));
        preset.apply_to(cab);
    }

    /// Disable amp simulation
    pub fn disable_amp(&mut self) {
        self.amp = None;
    }

    /// Disable cabinet simulation
    pub fn disable_cabinet(&mut self) {
        self.cabinet = None;
    }

    /// Enable reverb with a preset
    pub fn set_reverb(&mut self, preset: &str) {
        let reverb = match preset {
            "small" | "small_room" => Reverb::small_room(self.sample_rate),
            "medium" | "medium_room" => Reverb::medium_room(self.sample_rate),
            "hall" | "large" => Reverb::hall(self.sample_rate),
            "cathedral" | "ambient" => Reverb::cathedral(self.sample_rate),
            "plate" => Reverb::plate(self.sample_rate),
            _ => Reverb::small_room(self.sample_rate),
        };
        self.reverb = Some(reverb);
    }

    /// Disable reverb
    pub fn disable_reverb(&mut self) {
        self.reverb = None;
    }

    /// Get mutable reference to amp (for custom configuration)
    pub fn amp_mut(&mut self) -> Option<&mut AmpSimulator> {
        self.amp.as_mut()
    }

    /// Get mutable reference to cabinet (for custom configuration)
    pub fn cabinet_mut(&mut self) -> Option<&mut CabinetSimulator> {
        self.cabinet.as_mut()
    }

    /// Get mutable reference to reverb (for custom configuration)
    pub fn reverb_mut(&mut self) -> Option<&mut Reverb> {
        self.reverb.as_mut()
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Render a tab document to audio samples
    ///
    /// Returns interleaved stereo samples (left, right, left, right, ...)
    pub fn render(&mut self, document: &TabDocument, tempo: f64) -> Vec<f32> {
        info!("Starting tab render: {} measures at {} BPM",
              document.measures.len(), tempo);

        // Convert document to timed events
        let events = self.document_to_events(document, tempo);

        if events.is_empty() {
            info!("No events to render");
            return Vec::new();
        }

        info!("Rendering {} tab events", events.len());

        // Calculate duration from last event plus tail
        let last_event_pos = events.iter()
            .map(|e| e.sample_pos)
            .max()
            .unwrap_or(0);

        // Add 2 seconds of tail for note decay
        let tail_samples = self.sample_rate as u64 * 2;
        let total_samples = (last_event_pos + tail_samples) as usize;

        // Allocate output buffer (stereo)
        let mut output = Vec::with_capacity(total_samples * 2);

        // Create event iterator
        let mut event_idx = 0;

        // Render sample by sample
        for sample_pos in 0..total_samples as u64 {
            // Process any events at this sample position
            while event_idx < events.len() && events[event_idx].sample_pos <= sample_pos {
                let event = &events[event_idx];
                if event.is_note_on {
                    self.guitar.pluck(event.string, event.fret, event.velocity);
                } else {
                    self.guitar.mute_string(event.string);
                }
                event_idx += 1;
            }

            // Generate stereo sample from guitar
            let (mut left, mut right) = self.guitar.next_sample_stereo();

            // Process through amp if enabled
            if let Some(ref mut amp) = self.amp {
                // Sum to mono for amp processing, then split back to stereo
                let mono = (left + right) * 0.5;
                let amped = amp.process(mono);
                // Maintain original stereo width
                let stereo_diff = (left - right) * 0.3;
                left = amped + stereo_diff;
                right = amped - stereo_diff;
            }

            // Process through cabinet if enabled
            if let Some(ref mut cab) = self.cabinet {
                // Cabinet is mono processing, apply to both channels
                left = cab.process(left);
                // For stereo cabinet, we'd need a second instance
                // For now, apply slight variation by using previous sample
                right = cab.process(right);
            }

            // Process through reverb if enabled
            if let Some(ref mut reverb) = self.reverb {
                let (rev_l, rev_r) = reverb.process(left, right);
                left = rev_l;
                right = rev_r;
            }

            output.push(left * self.master_volume);
            output.push(right * self.master_volume);
        }

        info!("Tab render complete: {} stereo samples ({:.2}s)",
              output.len() / 2,
              output.len() as f64 / (2.0 * self.sample_rate as f64));

        output
    }

    /// Convert tab document to timed events
    fn document_to_events(&self, document: &TabDocument, tempo: f64) -> Vec<TabEvent> {
        let mut events = Vec::new();

        // PPQN = 480 (ticks per quarter note/beat)
        const PPQN: f64 = 480.0;

        // Samples per tick at current tempo
        // At tempo BPM: 1 beat (quarter) = 60/BPM seconds = 60 * sample_rate / BPM samples
        // 1 beat = 480 ticks, so samples_per_tick = (60 * sample_rate / BPM) / 480
        let samples_per_tick = (60.0 * self.sample_rate as f64 / tempo) / PPQN;

        // Track current position in ticks
        let mut current_tick: u64 = 0;

        for measure in &document.measures {
            // Process each track's beats in this measure
            for track_measure in &measure.track_beats {
                let mut tick_offset: u64 = 0;

                for beat in &track_measure.beats {
                    // Get rhythm duration in ticks
                    let beat_ticks = beat.rhythm.ticks();

                    for note in &beat.notes {
                        // Calculate sample position
                        let sample_pos = ((current_tick + tick_offset) as f64 * samples_per_tick) as u64;

                        // Determine velocity based on note properties and techniques
                        let velocity = if note.ghost {
                            0.4
                        } else if note.dead {
                            0.5
                        } else {
                            // Use note velocity, normalized to 0.0-1.0
                            // Check for hammer-on/pull-off in techniques
                            let has_legato = note.techniques.iter().any(|t| {
                                matches!(t, orpheus_core::tab::Technique::HammerOn | orpheus_core::tab::Technique::PullOff)
                            });
                            if has_legato {
                                0.7
                            } else {
                                note.velocity as f32 / 127.0
                            }
                        };

                        // Add note-on event
                        events.push(TabEvent {
                            sample_pos,
                            string: note.string,
                            fret: note.fret,
                            velocity,
                            is_note_on: true,
                        });

                        // For palm mutes and dead notes, add early note-off
                        let has_palm_mute = note.techniques.iter().any(|t| {
                            matches!(t, orpheus_core::tab::Technique::PalmMute(_))
                        });
                        if has_palm_mute || note.dead {
                            let mute_pos = sample_pos + (beat_ticks as f64 * samples_per_tick * 0.3) as u64;
                            events.push(TabEvent {
                                sample_pos: mute_pos,
                                string: note.string,
                                fret: note.fret,
                                velocity: 0.0,
                                is_note_on: false,
                            });
                        }
                    }

                    tick_offset += beat_ticks;
                }
            }

            // Move to next measure
            let time_sig = measure.time_signature.unwrap_or_default();
            let ticks_per_measure = (time_sig.numerator as u64) * (PPQN as u64 * 4 / time_sig.denominator as u64);
            current_tick += ticks_per_measure;
        }

        // Sort events by sample position
        events.sort_by_key(|e| e.sample_pos);

        events
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

/// Extended renderer with progress reporting
pub struct TabProgressRenderer {
    renderer: TabRenderer,
    progress_callback: Option<Box<dyn Fn(f32) + Send>>,
}

impl TabProgressRenderer {
    /// Create a new progress renderer
    pub fn new(sample_rate: u32) -> Self {
        Self {
            renderer: TabRenderer::new(sample_rate),
            progress_callback: None,
        }
    }

    /// Create a new progress renderer with amp simulation
    pub fn with_amp(sample_rate: u32) -> Self {
        Self {
            renderer: TabRenderer::with_amp(sample_rate),
            progress_callback: None,
        }
    }

    /// Set progress callback
    pub fn on_progress<F: Fn(f32) + Send + 'static>(&mut self, callback: F) {
        self.progress_callback = Some(Box::new(callback));
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.renderer.set_master_volume(volume);
    }

    /// Set amp preset
    pub fn set_amp_preset(&mut self, preset: AmpPreset) {
        self.renderer.set_amp_preset(preset);
    }

    /// Set cabinet preset
    pub fn set_cabinet_preset(&mut self, preset: CabinetPreset) {
        self.renderer.set_cabinet_preset(preset);
    }

    /// Set reverb preset
    pub fn set_reverb(&mut self, preset: &str) {
        self.renderer.set_reverb(preset);
    }

    /// Render tab document with progress updates
    pub fn render(&mut self, document: &TabDocument, tempo: f64) -> Vec<f32> {
        info!("Starting tab render with progress reporting");

        // Convert document to timed events
        let events = self.renderer.document_to_events(document, tempo);

        if events.is_empty() {
            if let Some(ref cb) = self.progress_callback {
                cb(1.0);
            }
            return Vec::new();
        }

        // Calculate duration
        let last_event_pos = events.iter()
            .map(|e| e.sample_pos)
            .max()
            .unwrap_or(0);

        let tail_samples = self.renderer.sample_rate as u64 * 2;
        let total_samples = (last_event_pos + tail_samples) as usize;

        // Allocate output buffer
        let mut output = Vec::with_capacity(total_samples * 2);

        // Create event iterator
        let mut event_idx = 0;

        // Render with progress updates
        for sample_pos in 0..total_samples as u64 {
            // Process events at this position
            while event_idx < events.len() && events[event_idx].sample_pos <= sample_pos {
                let event = &events[event_idx];
                if event.is_note_on {
                    self.renderer.guitar.pluck(event.string, event.fret, event.velocity);
                } else {
                    self.renderer.guitar.mute_string(event.string);
                }
                event_idx += 1;
            }

            // Generate stereo sample from guitar
            let (mut left, mut right) = self.renderer.guitar.next_sample_stereo();

            // Process through amp if enabled
            if let Some(ref mut amp) = self.renderer.amp {
                let mono = (left + right) * 0.5;
                let amped = amp.process(mono);
                let stereo_diff = (left - right) * 0.3;
                left = amped + stereo_diff;
                right = amped - stereo_diff;
            }

            // Process through cabinet if enabled
            if let Some(ref mut cab) = self.renderer.cabinet {
                left = cab.process(left);
                right = cab.process(right);
            }

            // Process through reverb if enabled
            if let Some(ref mut reverb) = self.renderer.reverb {
                let (rev_l, rev_r) = reverb.process(left, right);
                left = rev_l;
                right = rev_r;
            }

            output.push(left * self.renderer.master_volume);
            output.push(right * self.renderer.master_volume);

            // Report progress every ~4096 samples
            if sample_pos % 4096 == 0 {
                if let Some(ref cb) = self.progress_callback {
                    let progress = sample_pos as f64 / total_samples as f64;
                    cb(progress as f32);
                }
            }
        }

        // Final progress update
        if let Some(ref cb) = self.progress_callback {
            cb(1.0);
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orpheus_core::tab::{
        TabDocument, TabMeasure, TrackMeasure, TabBeat, TabNote,
        RhythmValue, BaseDuration, TabTrack, Instrument,
    };

    fn create_test_document() -> TabDocument {
        // Create document with proper constructor
        let mut doc = TabDocument::new();
        doc.metadata.title = "Test Tab".to_string();
        doc.metadata.artist = "Test Artist".to_string();

        // Create guitar track with proper constructor
        let track = TabTrack::new("Guitar", Instrument::guitar_standard());
        let track_id = track.id;
        doc.tracks.push(track);

        // Add a simple measure with one beat
        let beat = TabBeat {
            notes: vec![
                TabNote::new(1, 0), // String 1, fret 0 (open high E)
                TabNote::new(2, 1), // String 2, fret 1
            ],
            rhythm: RhythmValue {
                base: BaseDuration::Quarter,
                dots: 0,
                tuplet: None,
                tied: false,
            },
            ..TabBeat::default()
        };

        let mut measure = TabMeasure::new(1);
        measure.track_beats.push(TrackMeasure {
            track_id,
            beats: vec![beat],
        });

        doc.measures.push(measure);
        doc
    }

    #[test]
    fn test_render_empty_document() {
        let mut renderer = TabRenderer::new(44100);
        let doc = TabDocument::default();
        let output = renderer.render(&doc, 120.0);
        assert!(output.is_empty());
    }

    #[test]
    fn test_render_simple_tab() {
        let mut renderer = TabRenderer::new(44100);
        let doc = create_test_document();
        let output = renderer.render(&doc, 120.0);

        // Should have stereo audio
        assert!(!output.is_empty());
        // Should be stereo (even number of samples)
        assert_eq!(output.len() % 2, 0);

        // Check that some non-zero audio was produced
        let has_audio = output.iter().any(|&s| s.abs() > 0.001);
        assert!(has_audio, "Output should contain non-zero audio");
    }

    #[test]
    fn test_render_with_progress() {
        let mut renderer = TabProgressRenderer::new(44100);
        let doc = create_test_document();

        renderer.on_progress(|_progress| {
            // Validates the progress callback API
        });

        let output = renderer.render(&doc, 120.0);
        assert!(!output.is_empty());
    }
}
