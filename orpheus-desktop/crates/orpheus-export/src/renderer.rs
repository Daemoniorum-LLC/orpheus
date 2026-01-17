//! Offline audio renderer
//!
//! Processes project clips through the synth engine without real-time constraints,
//! generating audio samples for export.

use orpheus_core::{ClipPlaybackEngine, Project, Track};
use orpheus_synth::PianoSynth;
use tracing::info;

/// Offline audio renderer for project export
pub struct OfflineRenderer {
    /// Sample rate
    sample_rate: u32,
    /// Piano synth for MIDI clips
    piano: PianoSynth,
    /// Clip playback engine
    clip_playback: ClipPlaybackEngine,
    /// Master volume (0.0 - 1.0)
    master_volume: f32,
}

impl OfflineRenderer {
    /// Create a new offline renderer
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            piano: PianoSynth::new(sample_rate, 32), // 32-voice polyphony for export
            clip_playback: ClipPlaybackEngine::new(sample_rate, 120.0),
            master_volume: 0.8,
        }
    }

    /// Set master volume (0.0 - 1.0)
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Render a project to audio samples
    ///
    /// Returns interleaved stereo samples (left, right, left, right, ...)
    pub fn render_project(&mut self, project: &Project) -> Vec<f32> {
        info!("Starting offline render of project '{}'", project.metadata.name);

        // Set tempo
        self.clip_playback.set_tempo(project.tempo);

        // Load all tracks with clips
        let tracks: Vec<&Track> = project.tracks_ordered();
        self.clip_playback.load_clips(&tracks);

        let event_count = self.clip_playback.event_count();
        info!("Loaded {} MIDI events for rendering", event_count);

        if event_count == 0 {
            info!("No events to render");
            return Vec::new();
        }

        // Calculate project duration based on clip positions
        let duration_samples = self.calculate_project_duration(project);
        info!("Project duration: {} samples ({:.2} seconds)",
              duration_samples, duration_samples as f64 / self.sample_rate as f64);

        // Allocate output buffer (stereo interleaved)
        let total_samples = duration_samples * 2; // stereo
        let mut output = Vec::with_capacity(total_samples);

        // Render in frames
        const FRAME_SIZE: usize = 256;
        let mut position: u64 = 0;

        self.clip_playback.play();

        while position < duration_samples as u64 {
            let frame_samples = FRAME_SIZE.min((duration_samples as u64 - position) as usize);

            // Process clip events for this frame
            let events = self.clip_playback.process_frame(frame_samples as u64);
            for event in events {
                if event.is_note_on {
                    let velocity = event.velocity as f32 / 127.0;
                    self.piano.note_on(event.note, velocity);
                } else {
                    self.piano.note_off(event.note);
                }
            }

            // Generate audio samples
            for _ in 0..frame_samples {
                let (left, right) = self.piano.next_sample_stereo();
                output.push(left * self.master_volume);
                output.push(right * self.master_volume);
            }

            position += frame_samples as u64;
        }

        self.clip_playback.stop();

        info!("Render complete: {} stereo samples", output.len() / 2);
        output
    }

    /// Render specific tracks to audio samples
    ///
    /// Returns interleaved stereo samples
    pub fn render_tracks(&mut self, tracks: &[&Track], tempo: f64) -> Vec<f32> {
        info!("Rendering {} tracks", tracks.len());

        // Set tempo
        self.clip_playback.set_tempo(tempo);

        // Load tracks
        self.clip_playback.load_clips(tracks);

        let event_count = self.clip_playback.event_count();
        if event_count == 0 {
            return Vec::new();
        }

        // Calculate duration from tracks
        let duration_samples = self.calculate_tracks_duration(tracks);
        info!("Tracks duration: {} samples", duration_samples);

        // Allocate output buffer (stereo interleaved)
        let mut output = Vec::with_capacity(duration_samples * 2);

        // Render in frames
        const FRAME_SIZE: usize = 256;
        let mut position: u64 = 0;

        self.clip_playback.play();

        while position < duration_samples as u64 {
            let frame_samples = FRAME_SIZE.min((duration_samples as u64 - position) as usize);

            // Process clip events
            let events = self.clip_playback.process_frame(frame_samples as u64);
            for event in events {
                if event.is_note_on {
                    let velocity = event.velocity as f32 / 127.0;
                    self.piano.note_on(event.note, velocity);
                } else {
                    self.piano.note_off(event.note);
                }
            }

            // Generate audio samples
            for _ in 0..frame_samples {
                let (left, right) = self.piano.next_sample_stereo();
                output.push(left * self.master_volume);
                output.push(right * self.master_volume);
            }

            position += frame_samples as u64;
        }

        self.clip_playback.stop();

        output
    }

    /// Calculate project duration in samples based on clip end positions
    fn calculate_project_duration(&self, project: &Project) -> usize {
        let mut max_end: u64 = 0;

        for (_, track) in &project.tracks {
            for clip in &track.clips {
                let clip_end = clip.start + clip.length;
                max_end = max_end.max(clip_end);
            }
        }

        // Add a tail for note release (1 second)
        let tail_samples = self.sample_rate as u64;
        (max_end + tail_samples) as usize
    }

    /// Calculate duration from a list of tracks
    fn calculate_tracks_duration(&self, tracks: &[&Track]) -> usize {
        let mut max_end: u64 = 0;

        for track in tracks {
            for clip in &track.clips {
                let clip_end = clip.start + clip.length;
                max_end = max_end.max(clip_end);
            }
        }

        // Add a tail for note release (1 second)
        let tail_samples = self.sample_rate as u64;
        (max_end + tail_samples) as usize
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

/// Export progress callback
pub type ProgressCallback = Box<dyn Fn(f32) + Send>;

/// Extended renderer with progress reporting
pub struct ProgressRenderer {
    renderer: OfflineRenderer,
    progress_callback: Option<ProgressCallback>,
}

impl ProgressRenderer {
    /// Create a new progress renderer
    pub fn new(sample_rate: u32) -> Self {
        Self {
            renderer: OfflineRenderer::new(sample_rate),
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

    /// Render project with progress updates
    pub fn render_project(&mut self, project: &Project) -> Vec<f32> {
        info!("Starting offline render with progress reporting");

        // Set tempo
        self.renderer.clip_playback.set_tempo(project.tempo);

        // Load all tracks with clips
        let tracks: Vec<&Track> = project.tracks_ordered();
        self.renderer.clip_playback.load_clips(&tracks);

        let event_count = self.renderer.clip_playback.event_count();
        if event_count == 0 {
            if let Some(ref cb) = self.progress_callback {
                cb(1.0);
            }
            return Vec::new();
        }

        // Calculate project duration
        let duration_samples = self.renderer.calculate_project_duration(project);

        // Allocate output buffer
        let mut output = Vec::with_capacity(duration_samples * 2);

        // Render in frames with progress updates
        const FRAME_SIZE: usize = 256;
        let mut position: u64 = 0;
        let total = duration_samples as f64;

        self.renderer.clip_playback.play();

        while position < duration_samples as u64 {
            let frame_samples = FRAME_SIZE.min((duration_samples as u64 - position) as usize);

            // Process clip events
            let events = self.renderer.clip_playback.process_frame(frame_samples as u64);
            for event in events {
                if event.is_note_on {
                    let velocity = event.velocity as f32 / 127.0;
                    self.renderer.piano.note_on(event.note, velocity);
                } else {
                    self.renderer.piano.note_off(event.note);
                }
            }

            // Generate audio samples
            for _ in 0..frame_samples {
                let (left, right) = self.renderer.piano.next_sample_stereo();
                output.push(left * self.renderer.master_volume);
                output.push(right * self.renderer.master_volume);
            }

            position += frame_samples as u64;

            // Report progress every ~4096 samples
            if position % 4096 == 0 {
                if let Some(ref cb) = self.progress_callback {
                    let progress = position as f64 / total;
                    cb(progress as f32);
                }
            }
        }

        self.renderer.clip_playback.stop();

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
    use orpheus_core::{Clip, ClipContent, MidiNote, TrackType};
    use uuid::Uuid;

    #[test]
    fn test_render_empty_project() {
        let mut renderer = OfflineRenderer::new(44100);
        let project = Project::new("Test");
        let output = renderer.render_project(&project);
        assert!(output.is_empty());
    }

    #[test]
    fn test_render_project_with_clip() {
        let mut renderer = OfflineRenderer::new(44100);
        let mut project = Project::new("Test");

        // Create a track with a clip
        let mut track = Track::new("Piano", TrackType::Midi);
        track.clips.push(Clip {
            id: Uuid::new_v4(),
            name: "Test Clip".into(),
            start: 0,
            length: 44100, // 1 second
            content: ClipContent::Midi {
                notes: vec![
                    MidiNote {
                        note: 60, // Middle C
                        velocity: 100,
                        start: 0,
                        duration: 480, // Quarter note at 480 PPQN
                    },
                ],
            },
        });
        project.add_track(track);

        let output = renderer.render_project(&project);

        // Should have stereo audio
        assert!(!output.is_empty());
        // Should be stereo (even number of samples)
        assert_eq!(output.len() % 2, 0);

        // Check that some non-zero audio was produced
        let has_audio = output.iter().any(|&s| s.abs() > 0.001);
        assert!(has_audio, "Output should contain non-zero audio");
    }

    #[test]
    fn test_render_tracks() {
        let mut renderer = OfflineRenderer::new(44100);

        let mut track = Track::new("Piano", TrackType::Midi);
        track.clips.push(Clip {
            id: Uuid::new_v4(),
            name: "Test Clip".into(),
            start: 0,
            length: 22050, // 0.5 seconds
            content: ClipContent::Midi {
                notes: vec![
                    MidiNote {
                        note: 64, // E
                        velocity: 80,
                        start: 0,
                        duration: 240,
                    },
                ],
            },
        });

        let output = renderer.render_tracks(&[&track], 120.0);

        assert!(!output.is_empty());
        assert_eq!(output.len() % 2, 0);
    }
}
