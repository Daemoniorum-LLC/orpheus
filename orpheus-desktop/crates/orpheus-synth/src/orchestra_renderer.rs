//! Orchestra Renderer - Spatial audio rendering for multi-instrument playback
//!
//! This module bridges the MultiInstrumentEngine with the Orchestra spatial positioning
//! system to create immersive concert hall audio.
//!
//! # Features
//! - Automatic instrument-to-section mapping
//! - Spatial positioning in virtual concert hall
//! - Early reflections and reverb
//! - Conductor view visualization data
//! - Real-time position updates

use crate::effects::Reverb;
use crate::instrument_router::{InstrumentTrack, InstrumentType, MultiInstrumentEngine, SynthFactory};
use crate::orchestra::{
    ConcertHall, HallType, Listener, Musician, Orchestra, OrchestraSection,
    OrchestraVisualization, Position, SeatingArrangement, SectionMixer, SpatialProcessor,
};

/// Configuration for the orchestra renderer
#[derive(Debug, Clone)]
pub struct OrchestraRendererConfig {
    /// Sample rate
    pub sample_rate: f32,
    /// Hall type for acoustic simulation
    pub hall_type: HallType,
    /// Seating arrangement
    pub seating: SeatingArrangement,
    /// Master reverb mix (0.0 - 1.0)
    pub reverb_mix: f32,
    /// Enable early reflections
    pub early_reflections: bool,
    /// Listener position in the hall
    pub listener_position: Position,
}

impl Default for OrchestraRendererConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100.0,
            hall_type: HallType::SymphonyHall,
            seating: SeatingArrangement::American,
            reverb_mix: 0.3,
            early_reflections: true,
            listener_position: Position::new(0.0, 15.0, 1.2), // Center, 15m back, ear height
        }
    }
}

/// Track positioning information
#[derive(Debug, Clone)]
pub struct TrackPosition {
    /// Track ID
    pub track_id: u32,
    /// Position in the concert hall
    pub position: Position,
    /// Associated orchestra section (if any)
    pub section: Option<OrchestraSection>,
    /// Spatial processor for this track
    processor_index: usize,
}

/// The orchestra renderer combines synthesis with spatial audio
pub struct OrchestraRenderer {
    /// Multi-instrument engine for synthesis
    engine: MultiInstrumentEngine,
    /// Concert hall for acoustic simulation
    hall: ConcertHall,
    /// Listener position
    listener: Listener,
    /// Spatial processors for each track
    spatial_processors: Vec<SpatialProcessor>,
    /// Track positions
    track_positions: Vec<TrackPosition>,
    /// Master reverb
    reverb: Reverb,
    /// Reverb mix amount
    reverb_mix: f32,
    /// Sample rate
    sample_rate: f32,
    /// Seating arrangement
    seating: SeatingArrangement,
    /// Master volume
    master_volume: f32,
}

impl OrchestraRenderer {
    /// Create a new orchestra renderer with default configuration
    pub fn new(sample_rate: f32) -> Self {
        Self::with_config(OrchestraRendererConfig {
            sample_rate,
            ..Default::default()
        })
    }

    /// Create with custom configuration
    pub fn with_config(config: OrchestraRendererConfig) -> Self {
        let hall = ConcertHall::new(config.hall_type);
        let listener = Listener {
            position: config.listener_position,
            rotation: 0.0,
        };

        let mut reverb = Reverb::new(config.sample_rate as u32);
        reverb.set_room_size(config.hall_type.reverb_time() / 3.0); // Scale RT60 to room size param
        reverb.set_damping(0.5);
        reverb.set_mix(config.reverb_mix);

        Self {
            engine: MultiInstrumentEngine::new(config.sample_rate),
            hall,
            listener,
            spatial_processors: Vec::new(),
            track_positions: Vec::new(),
            reverb,
            reverb_mix: config.reverb_mix,
            sample_rate: config.sample_rate,
            seating: config.seating,
            master_volume: 0.8,
        }
    }

    /// Create for symphony orchestra
    pub fn symphony(sample_rate: f32) -> Self {
        Self::with_config(OrchestraRendererConfig {
            sample_rate,
            hall_type: HallType::SymphonyHall,
            seating: SeatingArrangement::American,
            reverb_mix: 0.35,
            ..Default::default()
        })
    }

    /// Create for chamber music
    pub fn chamber(sample_rate: f32) -> Self {
        Self::with_config(OrchestraRendererConfig {
            sample_rate,
            hall_type: HallType::ChamberHall,
            seating: SeatingArrangement::Baroque,
            reverb_mix: 0.25,
            listener_position: Position::new(0.0, 8.0, 1.2),
            ..Default::default()
        })
    }

    /// Create for jazz ensemble
    pub fn jazz_club(sample_rate: f32) -> Self {
        Self::with_config(OrchestraRendererConfig {
            sample_rate,
            hall_type: HallType::RecitalHall,
            seating: SeatingArrangement::American,
            reverb_mix: 0.2,
            listener_position: Position::new(0.0, 5.0, 1.2),
            ..Default::default()
        })
    }

    /// Create for rock/pop band
    pub fn studio(sample_rate: f32) -> Self {
        Self::with_config(OrchestraRendererConfig {
            sample_rate,
            hall_type: HallType::Studio,
            seating: SeatingArrangement::American,
            reverb_mix: 0.15,
            listener_position: Position::new(0.0, 3.0, 1.2),
            ..Default::default()
        })
    }

    /// Add a track with automatic positioning
    pub fn add_track(&mut self, id: u32, name: String, instrument_type: InstrumentType) {
        // Add to engine
        self.engine.add_track(id, name, instrument_type.clone());

        // Determine position based on instrument type
        let (position, section) = self.position_for_instrument(&instrument_type);

        // Create spatial processor
        let processor = SpatialProcessor::new(position, self.listener, &self.hall, self.sample_rate);

        let processor_index = self.spatial_processors.len();
        self.spatial_processors.push(processor);

        self.track_positions.push(TrackPosition {
            track_id: id,
            position,
            section,
            processor_index,
        });
    }

    /// Position an instrument based on its type and seating arrangement
    fn position_for_instrument(&self, instrument: &InstrumentType) -> (Position, Option<OrchestraSection>) {
        let section = instrument.orchestra_section();

        if let Some(sec) = section {
            // Get position from seating arrangement
            let pos = self.seating.section_center(sec);
            (pos, Some(sec))
        } else {
            // Non-orchestral instruments get default positions
            match instrument {
                InstrumentType::AcousticGuitar(_)
                | InstrumentType::ElectricGuitar(_)
                | InstrumentType::ClassicalGuitar(_) => {
                    // Guitar: center-left
                    (Position::new(-2.0, 3.0, 1.0), None)
                }
                InstrumentType::ElectricBass | InstrumentType::AcousticBass => {
                    // Bass: center-right
                    (Position::new(2.0, 3.0, 1.0), None)
                }
                InstrumentType::DrumKit => {
                    // Drums: center-back
                    (Position::new(0.0, 5.0, 0.5), None)
                }
                InstrumentType::GrandPiano | InstrumentType::ElectricPiano => {
                    // Piano: left of center
                    (Position::new(-3.0, 4.0, 0.8), None)
                }
                InstrumentType::Organ => {
                    // Organ: back center (like in a church)
                    (Position::new(0.0, 8.0, 2.0), None)
                }
                _ => {
                    // Default center position
                    (Position::new(0.0, 4.0, 1.0), None)
                }
            }
        }
    }

    /// Remove a track
    pub fn remove_track(&mut self, id: u32) {
        self.engine.remove_track(id);

        // Find and remove track position (we keep the processor to avoid index invalidation)
        if let Some(idx) = self.track_positions.iter().position(|t| t.track_id == id) {
            self.track_positions.remove(idx);
        }
    }

    /// Move a track to a new position
    pub fn move_track(&mut self, id: u32, position: Position) {
        if let Some(track_pos) = self.track_positions.iter_mut().find(|t| t.track_id == id) {
            track_pos.position = position;
            self.spatial_processors[track_pos.processor_index].set_position(position, &self.hall);
        }
    }

    /// Send note on to a track
    pub fn note_on(&mut self, track_id: u32, note: u8, velocity: f32) {
        self.engine.note_on_by_id(track_id, note, velocity);
    }

    /// Send note off to a track
    pub fn note_off(&mut self, track_id: u32, note: u8) {
        self.engine.note_off_by_id(track_id, note);
    }

    /// Set track volume
    pub fn set_track_volume(&mut self, track_id: u32, volume: f32) {
        self.engine.set_track_volume_by_id(track_id, volume);
    }

    /// Mute a track
    pub fn mute_track(&mut self, track_id: u32, muted: bool) {
        self.engine.mute_track_by_id(track_id, muted);
    }

    /// Solo a track
    pub fn solo_track(&mut self, track_id: u32, solo: bool) {
        self.engine.solo_track_by_id(track_id, solo);
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 2.0);
    }

    /// Set reverb mix
    pub fn set_reverb_mix(&mut self, mix: f32) {
        self.reverb_mix = mix.clamp(0.0, 1.0);
        self.reverb.set_mix(self.reverb_mix);
    }

    /// Set hall type
    pub fn set_hall_type(&mut self, hall_type: HallType) {
        self.hall = ConcertHall::new(hall_type);
        self.reverb.set_room_size(hall_type.reverb_time() / 3.0);

        // Update all spatial processors
        for (track_pos, processor) in self.track_positions.iter().zip(self.spatial_processors.iter_mut()) {
            processor.set_position(track_pos.position, &self.hall);
        }
    }

    /// Move the listener position
    pub fn move_listener(&mut self, position: Position) {
        self.listener = Listener {
            position,
            rotation: 0.0,
        };

        // Update all spatial processors - recreate them with new listener
        for track_pos in &self.track_positions {
            if track_pos.processor_index < self.spatial_processors.len() {
                self.spatial_processors[track_pos.processor_index] =
                    SpatialProcessor::new(track_pos.position, self.listener, &self.hall, self.sample_rate);
            }
        }
    }

    /// Process audio and return stereo output
    pub fn process(&mut self, num_samples: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = vec![0.0; num_samples];
        let mut right = vec![0.0; num_samples];

        // Get list of track IDs to process
        let track_ids: Vec<u32> = self.track_positions.iter().map(|t| t.track_id).collect();

        // Process each track with spatial positioning
        for track_id in track_ids {
            // Find track position info
            let track_pos = match self.track_positions.iter().find(|t| t.track_id == track_id) {
                Some(tp) => tp.clone(),
                None => continue,
            };

            // Get dry audio from the track
            if let Some((track_left, track_right)) = self.engine.process_track(track_id, num_samples) {
                let processor = &mut self.spatial_processors[track_pos.processor_index];

                // Apply spatial processing
                for i in 0..num_samples {
                    // Combine stereo to mono for spatial processing
                    let mono = (track_left[i] + track_right[i]) * 0.5;

                    // Apply spatial positioning
                    let (l, r) = processor.process(mono);

                    left[i] += l;
                    right[i] += r;
                }
            }
        }

        // Apply master reverb
        if self.reverb_mix > 0.0 {
            for i in 0..num_samples {
                let (rev_l, rev_r) = self.reverb.process(left[i], right[i]);
                left[i] = left[i] * (1.0 - self.reverb_mix) + rev_l * self.reverb_mix;
                right[i] = right[i] * (1.0 - self.reverb_mix) + rev_r * self.reverb_mix;
            }
        }

        // Apply master volume
        for i in 0..num_samples {
            left[i] *= self.master_volume;
            right[i] *= self.master_volume;
        }

        (left, right)
    }

    /// Get visualization data for the conductor view
    pub fn get_visualization(&self) -> OrchestraRendererVisualization {
        let track_viz: Vec<TrackVisualization> = self
            .track_positions
            .iter()
            .map(|tp| {
                let is_active = self.engine
                    .get_track_by_id(tp.track_id)
                    .map(|t| t.synth.is_active())
                    .unwrap_or(false);

                let is_muted = self.engine
                    .get_track_by_id(tp.track_id)
                    .map(|t| t.muted)
                    .unwrap_or(false);

                let volume = self.engine
                    .get_track_by_id(tp.track_id)
                    .map(|t| t.volume)
                    .unwrap_or(0.0);

                TrackVisualization {
                    track_id: tp.track_id,
                    position: tp.position,
                    section: tp.section,
                    is_active,
                    is_muted,
                    volume,
                }
            })
            .collect();

        OrchestraRendererVisualization {
            tracks: track_viz,
            listener_position: self.listener.position,
            hall_type: self.hall.hall_type,
            hall_width: self.hall.width,
            hall_depth: self.hall.depth,
        }
    }

    /// Get access to the underlying engine
    pub fn engine(&self) -> &MultiInstrumentEngine {
        &self.engine
    }

    /// Get mutable access to the underlying engine
    pub fn engine_mut(&mut self) -> &mut MultiInstrumentEngine {
        &mut self.engine
    }

    /// Get track count
    pub fn track_count(&self) -> usize {
        self.track_positions.len()
    }

    /// Get track positions
    pub fn track_positions(&self) -> &[TrackPosition] {
        &self.track_positions
    }
}

/// Visualization data for a single track
#[derive(Debug, Clone)]
pub struct TrackVisualization {
    /// Track ID
    pub track_id: u32,
    /// Position in the hall
    pub position: Position,
    /// Orchestra section (if any)
    pub section: Option<OrchestraSection>,
    /// Whether the track is currently playing
    pub is_active: bool,
    /// Whether the track is muted
    pub is_muted: bool,
    /// Track volume (0.0 - 1.0)
    pub volume: f32,
}

/// Complete visualization data for the renderer
#[derive(Debug, Clone)]
pub struct OrchestraRendererVisualization {
    /// All track visualizations
    pub tracks: Vec<TrackVisualization>,
    /// Listener position
    pub listener_position: Position,
    /// Hall type
    pub hall_type: HallType,
    /// Hall width (meters)
    pub hall_width: f32,
    /// Hall depth (meters)
    pub hall_depth: f32,
}

impl OrchestraRendererVisualization {
    /// Get tracks by section
    pub fn tracks_in_section(&self, section: OrchestraSection) -> Vec<&TrackVisualization> {
        self.tracks
            .iter()
            .filter(|t| t.section == Some(section))
            .collect()
    }

    /// Get active tracks
    pub fn active_tracks(&self) -> Vec<&TrackVisualization> {
        self.tracks.iter().filter(|t| t.is_active).collect()
    }

    /// Convert position to stage coordinates (0-1 range for UI)
    pub fn position_to_stage_coords(&self, pos: &Position) -> (f32, f32) {
        let x = (pos.x + self.hall_width / 2.0) / self.hall_width;
        let y = pos.y / self.hall_depth;
        (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestra_renderer_creation() {
        let renderer = OrchestraRenderer::new(44100.0);
        assert_eq!(renderer.track_count(), 0);
    }

    #[test]
    fn test_add_tracks() {
        let mut renderer = OrchestraRenderer::symphony(44100.0);

        renderer.add_track(1, "Violin 1".to_string(), InstrumentType::Violin);
        renderer.add_track(2, "Cello".to_string(), InstrumentType::Cello);
        renderer.add_track(3, "Trumpet".to_string(), InstrumentType::Trumpet);

        assert_eq!(renderer.track_count(), 3);

        // Check that positions are assigned
        let positions = renderer.track_positions();
        assert!(positions.iter().any(|p| p.section == Some(OrchestraSection::Violin1)));
        assert!(positions.iter().any(|p| p.section == Some(OrchestraSection::Cello)));
        assert!(positions.iter().any(|p| p.section == Some(OrchestraSection::Trumpet)));
    }

    #[test]
    fn test_process_audio() {
        let mut renderer = OrchestraRenderer::chamber(44100.0);

        renderer.add_track(1, "Piano".to_string(), InstrumentType::GrandPiano);
        renderer.note_on(1, 60, 0.8);

        let (left, right) = renderer.process(1024);

        assert_eq!(left.len(), 1024);
        assert_eq!(right.len(), 1024);

        // Should have some output
        let max_l = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        let max_r = right.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_l > 0.0 || max_r > 0.0);
    }

    #[test]
    fn test_move_track() {
        let mut renderer = OrchestraRenderer::new(44100.0);

        renderer.add_track(1, "Violin".to_string(), InstrumentType::Violin);

        let new_pos = Position::new(5.0, 10.0, 1.5);
        renderer.move_track(1, new_pos);

        let positions = renderer.track_positions();
        let violin_pos = positions.iter().find(|p| p.track_id == 1).unwrap();

        assert!((violin_pos.position.x - 5.0).abs() < 0.001);
        assert!((violin_pos.position.y - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_visualization() {
        let mut renderer = OrchestraRenderer::symphony(44100.0);

        renderer.add_track(1, "Violin".to_string(), InstrumentType::Violin);
        renderer.add_track(2, "Flute".to_string(), InstrumentType::Flute);

        let viz = renderer.get_visualization();

        assert_eq!(viz.tracks.len(), 2);
        assert!(viz.hall_width > 0.0);
        assert!(viz.hall_depth > 0.0);

        // Test coordinate conversion
        let (x, y) = viz.position_to_stage_coords(&Position::new(0.0, 0.0, 0.0));
        assert!(x >= 0.0 && x <= 1.0);
        assert!(y >= 0.0 && y <= 1.0);
    }

    #[test]
    fn test_different_venues() {
        // Test all venue presets compile and work
        let _symphony = OrchestraRenderer::symphony(44100.0);
        let _chamber = OrchestraRenderer::chamber(44100.0);
        let _jazz = OrchestraRenderer::jazz_club(44100.0);
        let _studio = OrchestraRenderer::studio(44100.0);
    }

    #[test]
    fn test_hall_type_change() {
        let mut renderer = OrchestraRenderer::new(44100.0);

        renderer.add_track(1, "Violin".to_string(), InstrumentType::Violin);

        renderer.set_hall_type(HallType::Cathedral);
        renderer.set_reverb_mix(0.5);

        // Should still work
        let (left, right) = renderer.process(512);
        assert_eq!(left.len(), 512);
        assert_eq!(right.len(), 512);
    }
}
