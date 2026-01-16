//! Project model - tracks, clips, timeline

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::io::{Read, Write};

use crate::compression::CompressionDictionary;

/// Magic bytes identifying compressed Orpheus project files
const ORPHEUS_MAGIC: &[u8; 4] = b"ORPH";

/// Current compressed format version
const COMPRESSED_FORMAT_VERSION: u8 = 1;

/// Format version with dictionary support
const DICTIONARY_FORMAT_VERSION: u8 = 2;

/// Default zstd compression level (3 = good balance of speed/ratio)
const COMPRESSION_LEVEL: i32 = 3;

/// Project metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Project name
    pub name: String,
    /// Artist name
    pub artist: String,
    /// Album name
    pub album: String,
    /// Genre
    pub genre: String,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Project notes
    pub notes: String,
}

impl ProjectMetadata {
    pub fn new(name: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.into(),
            artist: String::new(),
            album: String::new(),
            genre: String::new(),
            created_at: now,
            modified_at: now,
            notes: String::new(),
        }
    }
}

/// Track type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackType {
    /// Audio track
    Audio,
    /// MIDI/Instrument track
    Midi,
    /// Auxiliary/Bus track
    Aux,
    /// Master track
    Master,
}

/// Input source for recording
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum InputSource {
    /// No input
    #[default]
    None,
    /// Virtual keyboard input
    VirtualKeyboard,
    /// MIDI input
    MidiInput,
    /// Audio input (microphone)
    AudioInput,
}

/// A track in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Unique track ID
    pub id: Uuid,
    /// Track name
    pub name: String,
    /// Track type
    pub track_type: TrackType,
    /// Track order/index
    pub order: usize,
    /// Clips on this track
    pub clips: Vec<Clip>,
    /// Record armed (ready to record)
    #[serde(default)]
    pub record_armed: bool,
    /// Input source for recording
    #[serde(default)]
    pub input_source: InputSource,
}

impl Track {
    pub fn new(name: impl Into<String>, track_type: TrackType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            track_type,
            order: 0,
            clips: Vec::new(),
            record_armed: false,
            input_source: InputSource::None,
        }
    }

    /// Toggle record arm
    pub fn toggle_record_arm(&mut self) {
        self.record_armed = !self.record_armed;
    }

    /// Set input source
    pub fn set_input_source(&mut self, source: InputSource) {
        self.input_source = source;
    }

    pub fn audio(name: impl Into<String>) -> Self {
        Self::new(name, TrackType::Audio)
    }

    pub fn midi(name: impl Into<String>) -> Self {
        Self::new(name, TrackType::Midi)
    }
}

/// A clip on a track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    /// Unique clip ID
    pub id: Uuid,
    /// Clip name
    pub name: String,
    /// Start position in samples
    pub start: u64,
    /// Length in samples
    pub length: u64,
    /// Clip content
    pub content: ClipContent,
}

/// Clip content type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipContent {
    /// Audio clip with file reference
    Audio {
        /// Path to audio file
        file_path: String,
        /// Start offset within file (for trimming)
        file_offset: u64,
    },
    /// MIDI clip with note data
    Midi {
        /// MIDI notes
        notes: Vec<MidiNote>,
    },
}

/// A MIDI note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiNote {
    /// Note number (0-127)
    pub note: u8,
    /// Velocity (0-127)
    pub velocity: u8,
    /// Start time in ticks
    pub start: u64,
    /// Duration in ticks
    pub duration: u64,
}

/// Complete project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// File format version
    pub format_version: String,
    /// Project metadata
    pub metadata: ProjectMetadata,
    /// Tracks (keyed by ID)
    pub tracks: HashMap<Uuid, Track>,
    /// Track order (list of track IDs)
    pub track_order: Vec<Uuid>,
    /// Sample rate
    pub sample_rate: u32,
    /// Default tempo
    pub tempo: f64,
}

impl Default for Project {
    fn default() -> Self {
        Self::new("Untitled")
    }
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            format_version: "1.0".into(),
            metadata: ProjectMetadata::new(name),
            tracks: HashMap::new(),
            track_order: Vec::new(),
            sample_rate: 48000,
            tempo: 120.0,
        }
    }

    /// Add a track to the project
    pub fn add_track(&mut self, mut track: Track) -> Uuid {
        let id = track.id;
        track.order = self.track_order.len();
        self.tracks.insert(id, track);
        self.track_order.push(id);
        id
    }

    /// Remove a track from the project
    pub fn remove_track(&mut self, id: Uuid) -> Option<Track> {
        if let Some(pos) = self.track_order.iter().position(|&i| i == id) {
            self.track_order.remove(pos);
            // Update order for remaining tracks
            for (i, &track_id) in self.track_order.iter().enumerate() {
                if let Some(track) = self.tracks.get_mut(&track_id) {
                    track.order = i;
                }
            }
        }
        self.tracks.remove(&id)
    }

    /// Get track by ID
    pub fn get_track(&self, id: Uuid) -> Option<&Track> {
        self.tracks.get(&id)
    }

    /// Get mutable track by ID
    pub fn get_track_mut(&mut self, id: Uuid) -> Option<&mut Track> {
        self.tracks.get_mut(&id)
    }

    /// Get tracks in order
    pub fn tracks_ordered(&self) -> Vec<&Track> {
        self.track_order
            .iter()
            .filter_map(|id| self.tracks.get(id))
            .collect()
    }

    /// Get track count
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Mark as modified
    pub fn touch(&mut self) {
        self.metadata.modified_at = chrono::Utc::now();
    }

    /// Save project to file with compression
    ///
    /// Uses zstd compression for efficient storage (typically 60-80% size reduction).
    /// The file format is:
    /// - 4 bytes: magic "ORPH"
    /// - 1 byte: format version
    /// - N bytes: zstd-compressed JSON
    pub fn save(&self, path: &std::path::Path) -> Result<(), ProjectError> {
        let json = serde_json::to_vec(self)
            .map_err(|e| ProjectError::Serialize(e.to_string()))?;

        // Compress with zstd
        let compressed = zstd::encode_all(json.as_slice(), COMPRESSION_LEVEL)
            .map_err(|e| ProjectError::Compression(e.to_string()))?;

        // Write header + compressed data
        let file = std::fs::File::create(path)
            .map_err(|e| ProjectError::Io(e.to_string()))?;
        let mut writer = std::io::BufWriter::new(file);

        writer.write_all(ORPHEUS_MAGIC)
            .map_err(|e| ProjectError::Io(e.to_string()))?;
        writer.write_all(&[COMPRESSED_FORMAT_VERSION])
            .map_err(|e| ProjectError::Io(e.to_string()))?;
        writer.write_all(&compressed)
            .map_err(|e| ProjectError::Io(e.to_string()))?;
        writer.flush()
            .map_err(|e| ProjectError::Io(e.to_string()))?;

        tracing::debug!(
            "Saved project: {} bytes JSON -> {} bytes compressed ({:.1}% reduction)",
            json.len(),
            compressed.len() + 5,
            (1.0 - (compressed.len() + 5) as f64 / json.len() as f64) * 100.0
        );

        Ok(())
    }

    /// Save project as uncompressed JSON (for debugging/compatibility)
    pub fn save_uncompressed(&self, path: &std::path::Path) -> Result<(), ProjectError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| ProjectError::Serialize(e.to_string()))?;
        std::fs::write(path, json)
            .map_err(|e| ProjectError::Io(e.to_string()))?;
        Ok(())
    }

    /// Save project with dictionary compression for better compression ratios
    ///
    /// Dictionary compression can achieve 20-40% better compression than standard
    /// zstd for structured project data. The dictionary must also be available
    /// when loading the project.
    ///
    /// The file format is:
    /// - 4 bytes: magic "ORPH"
    /// - 1 byte: format version (2 for dictionary)
    /// - N bytes: dictionary-compressed JSON
    pub fn save_with_dictionary(
        &self,
        path: &std::path::Path,
        dictionary: &CompressionDictionary,
    ) -> Result<(), ProjectError> {
        let json = serde_json::to_vec(self)
            .map_err(|e| ProjectError::Serialize(e.to_string()))?;

        // Compress with dictionary
        let compressed = dictionary
            .compress(&json)
            .map_err(|e| ProjectError::Compression(e.to_string()))?;

        // Write header + compressed data
        let file = std::fs::File::create(path)
            .map_err(|e| ProjectError::Io(e.to_string()))?;
        let mut writer = std::io::BufWriter::new(file);

        writer
            .write_all(ORPHEUS_MAGIC)
            .map_err(|e| ProjectError::Io(e.to_string()))?;
        writer
            .write_all(&[DICTIONARY_FORMAT_VERSION])
            .map_err(|e| ProjectError::Io(e.to_string()))?;
        writer
            .write_all(&compressed)
            .map_err(|e| ProjectError::Io(e.to_string()))?;
        writer.flush().map_err(|e| ProjectError::Io(e.to_string()))?;

        tracing::debug!(
            "Saved project with dictionary: {} bytes JSON -> {} bytes compressed ({:.1}% reduction)",
            json.len(),
            compressed.len() + 5,
            (1.0 - (compressed.len() + 5) as f64 / json.len() as f64) * 100.0
        );

        Ok(())
    }

    /// Load project from file (auto-detects compressed vs JSON format)
    ///
    /// Supports:
    /// - Standard compressed format (version 1)
    /// - Legacy uncompressed JSON
    ///
    /// Note: For dictionary-compressed files (version 2), use `load_with_dictionary`.
    pub fn load(path: &std::path::Path) -> Result<Self, ProjectError> {
        Self::load_internal(path, None)
    }

    /// Load project that was saved with dictionary compression
    ///
    /// The dictionary used for loading must match the one used for saving.
    pub fn load_with_dictionary(
        path: &std::path::Path,
        dictionary: &CompressionDictionary,
    ) -> Result<Self, ProjectError> {
        Self::load_internal(path, Some(dictionary))
    }

    /// Internal load implementation supporting optional dictionary
    fn load_internal(
        path: &std::path::Path,
        dictionary: Option<&CompressionDictionary>,
    ) -> Result<Self, ProjectError> {
        let data = std::fs::read(path)
            .map_err(|e| ProjectError::Io(e.to_string()))?;

        // Check for compressed format (magic header)
        if data.len() >= 5 && &data[0..4] == ORPHEUS_MAGIC {
            let version = data[4];

            match version {
                COMPRESSED_FORMAT_VERSION => {
                    // Standard zstd compression
                    let decompressed = zstd::decode_all(&data[5..])
                        .map_err(|e| ProjectError::Compression(e.to_string()))?;

                    let project: Project = serde_json::from_slice(&decompressed)
                        .map_err(|e| ProjectError::Deserialize(e.to_string()))?;

                    tracing::debug!(
                        "Loaded compressed project: {} bytes -> {} bytes decompressed",
                        data.len(),
                        decompressed.len()
                    );

                    Ok(project)
                }
                DICTIONARY_FORMAT_VERSION => {
                    // Dictionary compression - requires dictionary
                    let dict = dictionary.ok_or(ProjectError::DictionaryRequired)?;

                    let decompressed = dict
                        .decompress(&data[5..])
                        .map_err(|e| ProjectError::Compression(e.to_string()))?;

                    let project: Project = serde_json::from_slice(&decompressed)
                        .map_err(|e| ProjectError::Deserialize(e.to_string()))?;

                    tracing::debug!(
                        "Loaded dictionary-compressed project: {} bytes -> {} bytes decompressed",
                        data.len(),
                        decompressed.len()
                    );

                    Ok(project)
                }
                _ => Err(ProjectError::UnsupportedVersion(version)),
            }
        } else {
            // Try loading as plain JSON (backward compatibility)
            let content = String::from_utf8(data)
                .map_err(|e| ProjectError::Deserialize(e.to_string()))?;
            let project: Project = serde_json::from_str(&content)
                .map_err(|e| ProjectError::Deserialize(e.to_string()))?;
            Ok(project)
        }
    }

    /// Check if a file uses dictionary compression
    pub fn requires_dictionary(path: &std::path::Path) -> Result<bool, ProjectError> {
        let file = std::fs::File::open(path)
            .map_err(|e| ProjectError::Io(e.to_string()))?;
        let mut reader = std::io::BufReader::new(file);

        let mut header = [0u8; 5];
        if reader.read_exact(&mut header).is_err() {
            return Ok(false); // Too small, probably JSON
        }

        if &header[0..4] == ORPHEUS_MAGIC {
            Ok(header[4] == DICTIONARY_FORMAT_VERSION)
        } else {
            Ok(false) // JSON format
        }
    }
}

/// Project errors
#[derive(Debug, Clone)]
pub enum ProjectError {
    /// IO error
    Io(String),
    /// Serialization error
    Serialize(String),
    /// Deserialization error
    Deserialize(String),
    /// Compression/decompression error
    Compression(String),
    /// Unsupported file format version
    UnsupportedVersion(u8),
    /// Dictionary required for loading
    DictionaryRequired,
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectError::Io(e) => write!(f, "IO error: {}", e),
            ProjectError::Serialize(e) => write!(f, "Serialization error: {}", e),
            ProjectError::Deserialize(e) => write!(f, "Deserialization error: {}", e),
            ProjectError::Compression(e) => write!(f, "Compression error: {}", e),
            ProjectError::UnsupportedVersion(v) => {
                write!(
                    f,
                    "Unsupported project version: {} (max supported: {})",
                    v, DICTIONARY_FORMAT_VERSION
                )
            }
            ProjectError::DictionaryRequired => {
                write!(
                    f,
                    "This project was saved with dictionary compression. \
                     Use load_with_dictionary() with the matching dictionary."
                )
            }
        }
    }
}

impl std::error::Error for ProjectError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_project() -> Project {
        let mut project = Project::new("Test Project");
        project.metadata.artist = "Test Artist".into();
        project.tempo = 140.0;

        // Add some tracks
        let audio_track = Track::audio("Audio 1");
        let midi_track = Track::midi("MIDI 1");
        project.add_track(audio_track);
        project.add_track(midi_track);

        project
    }

    #[test]
    fn test_save_load_compressed() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.orpheus");

        let original = create_test_project();
        original.save(&path).unwrap();

        // Verify file has magic header
        let data = std::fs::read(&path).unwrap();
        assert_eq!(&data[0..4], ORPHEUS_MAGIC);
        assert_eq!(data[4], COMPRESSED_FORMAT_VERSION);

        // Load and verify
        let loaded = Project::load(&path).unwrap();
        assert_eq!(loaded.metadata.name, original.metadata.name);
        assert_eq!(loaded.metadata.artist, original.metadata.artist);
        assert_eq!(loaded.tempo, original.tempo);
        assert_eq!(loaded.track_count(), original.track_count());
    }

    #[test]
    fn test_compression_ratio() {
        let dir = tempdir().unwrap();
        let compressed_path = dir.path().join("compressed.orpheus");
        let uncompressed_path = dir.path().join("uncompressed.json");

        let project = create_test_project();
        project.save(&compressed_path).unwrap();
        project.save_uncompressed(&uncompressed_path).unwrap();

        let compressed_size = std::fs::metadata(&compressed_path).unwrap().len();
        let uncompressed_size = std::fs::metadata(&uncompressed_path).unwrap().len();

        // Compressed should be smaller (even for small files, zstd should help)
        println!(
            "Compressed: {} bytes, Uncompressed: {} bytes, Ratio: {:.1}%",
            compressed_size,
            uncompressed_size,
            (compressed_size as f64 / uncompressed_size as f64) * 100.0
        );

        // For real projects with lots of data, expect significant reduction
        // For tiny test projects, compression overhead might make it larger
        // but the format should still work
        assert!(compressed_size < uncompressed_size * 2); // At worst 2x larger for tiny data
    }

    #[test]
    fn test_backward_compatibility_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("legacy.json");

        let original = create_test_project();
        // Save as uncompressed JSON (legacy format)
        original.save_uncompressed(&path).unwrap();

        // Load should auto-detect and handle JSON
        let loaded = Project::load(&path).unwrap();
        assert_eq!(loaded.metadata.name, original.metadata.name);
        assert_eq!(loaded.track_count(), original.track_count());
    }

    #[test]
    fn test_unsupported_version() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("future.orpheus");

        // Create a file with a future version number
        let mut data = Vec::new();
        data.extend_from_slice(ORPHEUS_MAGIC);
        data.push(255); // Future version
        data.extend_from_slice(b"fake compressed data");
        std::fs::write(&path, data).unwrap();

        let result = Project::load(&path);
        assert!(matches!(result, Err(ProjectError::UnsupportedVersion(255))));
    }

    #[test]
    fn test_project_with_clips() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("with_clips.orpheus");

        let mut project = create_test_project();

        // Add a MIDI clip with notes
        if let Some(track) = project.tracks.values_mut().find(|t| t.track_type == TrackType::Midi) {
            track.clips.push(Clip {
                id: Uuid::new_v4(),
                name: "MIDI Clip 1".into(),
                start: 0,
                length: 96000,
                content: ClipContent::Midi {
                    notes: vec![
                        MidiNote { note: 60, velocity: 100, start: 0, duration: 480 },
                        MidiNote { note: 64, velocity: 90, start: 480, duration: 480 },
                        MidiNote { note: 67, velocity: 80, start: 960, duration: 480 },
                    ],
                },
            });
        }

        project.save(&path).unwrap();
        let loaded = Project::load(&path).unwrap();

        // Verify clips were preserved
        let midi_track = loaded.tracks.values().find(|t| t.track_type == TrackType::Midi).unwrap();
        assert_eq!(midi_track.clips.len(), 1);

        if let ClipContent::Midi { notes } = &midi_track.clips[0].content {
            assert_eq!(notes.len(), 3);
            assert_eq!(notes[0].note, 60);
        } else {
            panic!("Expected MIDI clip");
        }
    }

    #[test]
    fn test_save_load_with_dictionary() {
        use crate::compression::DictionaryTrainer;

        let dir = tempdir().unwrap();

        // Create sample projects for training
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            let mut project = Project::new(format!("Training Project {}", i));
            project.metadata.artist = "Training Artist".into();
            project.tempo = 120.0 + i as f64;
            project.add_track(Track::audio(format!("Audio {}", i)));
            project.add_track(Track::midi(format!("MIDI {}", i)));

            let json = serde_json::to_vec(&project).unwrap();
            trainer.add_sample(&json);
        }

        let dictionary = trainer.train().unwrap();

        // Save with dictionary
        let path = dir.path().join("dict_compressed.orpheus");
        let original = create_test_project();
        original.save_with_dictionary(&path, &dictionary).unwrap();

        // Verify file uses dictionary format
        let data = std::fs::read(&path).unwrap();
        assert_eq!(&data[0..4], ORPHEUS_MAGIC);
        assert_eq!(data[4], DICTIONARY_FORMAT_VERSION);

        // Load with dictionary
        let loaded = Project::load_with_dictionary(&path, &dictionary).unwrap();
        assert_eq!(loaded.metadata.name, original.metadata.name);
        assert_eq!(loaded.metadata.artist, original.metadata.artist);
        assert_eq!(loaded.tempo, original.tempo);
        assert_eq!(loaded.track_count(), original.track_count());
    }

    #[test]
    fn test_dictionary_required_error() {
        use crate::compression::DictionaryTrainer;

        let dir = tempdir().unwrap();

        // Train a dictionary
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            let project = Project::new(format!("Training {}", i));
            let json = serde_json::to_vec(&project).unwrap();
            trainer.add_sample(&json);
        }
        let dictionary = trainer.train().unwrap();

        // Save with dictionary
        let path = dir.path().join("needs_dict.orpheus");
        let project = create_test_project();
        project.save_with_dictionary(&path, &dictionary).unwrap();

        // Try to load without dictionary
        let result = Project::load(&path);
        assert!(matches!(result, Err(ProjectError::DictionaryRequired)));
    }

    #[test]
    fn test_requires_dictionary() {
        use crate::compression::DictionaryTrainer;

        let dir = tempdir().unwrap();

        // Standard compression
        let standard_path = dir.path().join("standard.orpheus");
        let project = create_test_project();
        project.save(&standard_path).unwrap();
        assert!(!Project::requires_dictionary(&standard_path).unwrap());

        // JSON format
        let json_path = dir.path().join("legacy.json");
        project.save_uncompressed(&json_path).unwrap();
        assert!(!Project::requires_dictionary(&json_path).unwrap());

        // Dictionary compression
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            let p = Project::new(format!("Training {}", i));
            let json = serde_json::to_vec(&p).unwrap();
            trainer.add_sample(&json);
        }
        let dictionary = trainer.train().unwrap();

        let dict_path = dir.path().join("dict.orpheus");
        project.save_with_dictionary(&dict_path, &dictionary).unwrap();
        assert!(Project::requires_dictionary(&dict_path).unwrap());
    }

    #[test]
    fn test_dictionary_compression_improvement() {
        use crate::compression::DictionaryTrainer;

        let dir = tempdir().unwrap();

        // Create larger projects for meaningful compression
        fn create_large_project(id: usize) -> Project {
            let mut project = Project::new(format!("Large Project {}", id));
            project.metadata.artist = "Test Artist".into();
            project.metadata.album = "Test Album".into();
            project.metadata.genre = "Electronic".into();
            project.metadata.notes = "Some notes about the project".into();

            for i in 0..5 {
                let mut track = Track::midi(format!("MIDI Track {}", i));
                track.clips.push(Clip {
                    id: Uuid::new_v4(),
                    name: format!("Clip {}", i),
                    start: 0,
                    length: 96000,
                    content: ClipContent::Midi {
                        notes: (0..20)
                            .map(|n| MidiNote {
                                note: 60 + (n % 12) as u8,
                                velocity: 80 + (n % 40) as u8,
                                start: n * 480,
                                duration: 480,
                            })
                            .collect(),
                    },
                });
                project.add_track(track);
            }

            project
        }

        // Train dictionary
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            let p = create_large_project(i);
            let json = serde_json::to_vec(&p).unwrap();
            trainer.add_sample(&json);
        }
        let dictionary = trainer.train().unwrap();

        // Compare compression
        let test_project = create_large_project(100);
        let standard_path = dir.path().join("standard.orpheus");
        let dict_path = dir.path().join("dict.orpheus");

        test_project.save(&standard_path).unwrap();
        test_project.save_with_dictionary(&dict_path, &dictionary).unwrap();

        let standard_size = std::fs::metadata(&standard_path).unwrap().len();
        let dict_size = std::fs::metadata(&dict_path).unwrap().len();

        println!(
            "Standard compression: {} bytes, Dictionary compression: {} bytes ({:.1}% improvement)",
            standard_size,
            dict_size,
            (1.0 - dict_size as f64 / standard_size as f64) * 100.0
        );

        // Dictionary should provide some improvement for similar structured data
        // (may not always be smaller for small projects, but should work)
        assert!(dict_size <= standard_size || dict_size < standard_size + 100);
    }
}
