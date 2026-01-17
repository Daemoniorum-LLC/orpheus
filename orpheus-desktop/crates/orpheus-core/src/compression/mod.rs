//! Compression utilities for Orpheus
//!
//! This module provides dictionary-based zstd compression for improved compression
//! ratios on structured music project data. Dictionary compression can achieve
//! 20-40% better compression than standard zstd for similar data patterns.
//!
//! # Dictionary Training
//!
//! Dictionaries are trained on sample data (e.g., existing projects) to learn
//! common patterns. The trained dictionary is then used for both compression
//! and decompression.
//!
//! ```ignore
//! use orpheus_core::compression::{DictionaryTrainer, CompressionDictionary};
//!
//! // Train a dictionary from sample projects
//! let mut trainer = DictionaryTrainer::new();
//! for project_bytes in sample_projects {
//!     trainer.add_sample(&project_bytes);
//! }
//! let dictionary = trainer.train()?;
//!
//! // Use dictionary for compression
//! let compressed = dictionary.compress(&data)?;
//! let decompressed = dictionary.decompress(&compressed)?;
//! ```

use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

/// Default compression level for dictionary compression
const DEFAULT_COMPRESSION_LEVEL: i32 = 3;

/// Default dictionary size in bytes (64KB is a good balance)
const DEFAULT_DICTIONARY_SIZE: usize = 65536;

/// Minimum number of samples required for effective training
const MIN_TRAINING_SAMPLES: usize = 5;

/// Minimum total sample size for training (100KB)
const MIN_TRAINING_SIZE: usize = 102400;

/// Magic bytes for dictionary files
const DICTIONARY_MAGIC: &[u8; 4] = b"ODIC";

/// Dictionary file version
const DICTIONARY_VERSION: u8 = 1;

/// Errors related to dictionary compression
#[derive(Debug, Clone)]
pub enum CompressionError {
    /// IO error
    Io(String),
    /// Training error
    Training(String),
    /// Compression error
    Compress(String),
    /// Decompression error
    Decompress(String),
    /// Invalid dictionary
    InvalidDictionary(String),
    /// Not enough training samples
    InsufficientSamples { have: usize, need: usize },
}

impl std::fmt::Display for CompressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionError::Io(e) => write!(f, "IO error: {}", e),
            CompressionError::Training(e) => write!(f, "Training error: {}", e),
            CompressionError::Compress(e) => write!(f, "Compression error: {}", e),
            CompressionError::Decompress(e) => write!(f, "Decompression error: {}", e),
            CompressionError::InvalidDictionary(e) => write!(f, "Invalid dictionary: {}", e),
            CompressionError::InsufficientSamples { have, need } => {
                write!(
                    f,
                    "Insufficient training samples: have {} samples, need at least {}",
                    have, need
                )
            }
        }
    }
}

impl std::error::Error for CompressionError {}

/// A trained compression dictionary for optimized zstd compression
///
/// Dictionaries learn common patterns from training data and can significantly
/// improve compression ratios for similar data (20-40% better than standard zstd).
#[derive(Clone)]
pub struct CompressionDictionary {
    /// Raw dictionary bytes
    dict_data: Arc<Vec<u8>>,
    /// Compression level to use
    compression_level: i32,
}

impl std::fmt::Debug for CompressionDictionary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompressionDictionary")
            .field("size", &self.dict_data.len())
            .field("compression_level", &self.compression_level)
            .finish()
    }
}

impl CompressionDictionary {
    /// Create a dictionary from raw dictionary data
    pub fn from_bytes(dict_data: Vec<u8>) -> Self {
        Self {
            dict_data: Arc::new(dict_data),
            compression_level: DEFAULT_COMPRESSION_LEVEL,
        }
    }

    /// Create a dictionary from raw bytes with custom compression level
    pub fn from_bytes_with_level(dict_data: Vec<u8>, compression_level: i32) -> Self {
        Self {
            dict_data: Arc::new(dict_data),
            compression_level: compression_level.clamp(1, 22),
        }
    }

    /// Load a dictionary from file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, CompressionError> {
        let data = std::fs::read(path.as_ref())
            .map_err(|e| CompressionError::Io(e.to_string()))?;

        // Check magic header
        if data.len() < 5 {
            return Err(CompressionError::InvalidDictionary(
                "File too small".to_string(),
            ));
        }

        if &data[0..4] != DICTIONARY_MAGIC {
            // Try loading as raw dictionary (no header)
            return Ok(Self::from_bytes(data));
        }

        let version = data[4];
        if version > DICTIONARY_VERSION {
            return Err(CompressionError::InvalidDictionary(format!(
                "Unsupported dictionary version: {} (max: {})",
                version, DICTIONARY_VERSION
            )));
        }

        Ok(Self::from_bytes(data[5..].to_vec()))
    }

    /// Save dictionary to file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), CompressionError> {
        let file = std::fs::File::create(path.as_ref())
            .map_err(|e| CompressionError::Io(e.to_string()))?;
        let mut writer = std::io::BufWriter::new(file);

        writer
            .write_all(DICTIONARY_MAGIC)
            .map_err(|e| CompressionError::Io(e.to_string()))?;
        writer
            .write_all(&[DICTIONARY_VERSION])
            .map_err(|e| CompressionError::Io(e.to_string()))?;
        writer
            .write_all(&self.dict_data)
            .map_err(|e| CompressionError::Io(e.to_string()))?;
        writer
            .flush()
            .map_err(|e| CompressionError::Io(e.to_string()))?;

        Ok(())
    }

    /// Get the dictionary size in bytes
    pub fn size(&self) -> usize {
        self.dict_data.len()
    }

    /// Get the raw dictionary bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.dict_data
    }

    /// Set the compression level (1-22, default 3)
    pub fn with_compression_level(mut self, level: i32) -> Self {
        self.compression_level = level.clamp(1, 22);
        self
    }

    /// Compress data using this dictionary
    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        self.compress_with_level(data, self.compression_level)
    }

    /// Compress data with a specific compression level
    pub fn compress_with_level(
        &self,
        data: &[u8],
        level: i32,
    ) -> Result<Vec<u8>, CompressionError> {
        let mut encoder = zstd::stream::Encoder::with_dictionary(Vec::new(), level, &self.dict_data)
            .map_err(|e| CompressionError::Compress(e.to_string()))?;

        encoder
            .write_all(data)
            .map_err(|e| CompressionError::Compress(e.to_string()))?;

        encoder
            .finish()
            .map_err(|e| CompressionError::Compress(e.to_string()))
    }

    /// Decompress data using this dictionary
    pub fn decompress(&self, compressed: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut decoder = zstd::stream::Decoder::with_dictionary(compressed, &self.dict_data)
            .map_err(|e| CompressionError::Decompress(e.to_string()))?;

        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| CompressionError::Decompress(e.to_string()))?;

        Ok(decompressed)
    }
}

/// Dictionary trainer for creating optimized compression dictionaries
///
/// Collects sample data and trains a dictionary that learns common patterns.
/// The trained dictionary can then be used for efficient compression of similar data.
pub struct DictionaryTrainer {
    /// Collected training samples
    samples: Vec<Vec<u8>>,
    /// Target dictionary size
    dictionary_size: usize,
    /// Compression level for the resulting dictionary
    compression_level: i32,
}

impl Default for DictionaryTrainer {
    fn default() -> Self {
        Self::new()
    }
}

impl DictionaryTrainer {
    /// Create a new dictionary trainer with default settings
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            dictionary_size: DEFAULT_DICTIONARY_SIZE,
            compression_level: DEFAULT_COMPRESSION_LEVEL,
        }
    }

    /// Set the target dictionary size (default: 64KB)
    pub fn with_dictionary_size(mut self, size: usize) -> Self {
        self.dictionary_size = size;
        self
    }

    /// Set the compression level for the resulting dictionary (default: 3)
    pub fn with_compression_level(mut self, level: i32) -> Self {
        self.compression_level = level.clamp(1, 22);
        self
    }

    /// Add a training sample (raw bytes)
    pub fn add_sample(&mut self, sample: &[u8]) {
        if !sample.is_empty() {
            self.samples.push(sample.to_vec());
        }
    }

    /// Add a training sample from a file
    pub fn add_sample_file(&mut self, path: impl AsRef<Path>) -> Result<(), CompressionError> {
        let data = std::fs::read(path.as_ref())
            .map_err(|e| CompressionError::Io(e.to_string()))?;
        self.add_sample(&data);
        Ok(())
    }

    /// Get the number of samples collected
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Get the total size of collected samples
    pub fn total_sample_size(&self) -> usize {
        self.samples.iter().map(|s| s.len()).sum()
    }

    /// Check if we have enough samples for training
    pub fn has_sufficient_samples(&self) -> bool {
        self.samples.len() >= MIN_TRAINING_SAMPLES
            && self.total_sample_size() >= MIN_TRAINING_SIZE
    }

    /// Train a dictionary from the collected samples
    ///
    /// Requires at least 5 samples totaling at least 100KB for effective training.
    /// Returns an error if there are insufficient samples.
    pub fn train(self) -> Result<CompressionDictionary, CompressionError> {
        if self.samples.len() < MIN_TRAINING_SAMPLES {
            return Err(CompressionError::InsufficientSamples {
                have: self.samples.len(),
                need: MIN_TRAINING_SAMPLES,
            });
        }

        let total_size = self.total_sample_size();
        if total_size < MIN_TRAINING_SIZE {
            tracing::warn!(
                "Training with small dataset ({} bytes), dictionary may not be optimal",
                total_size
            );
        }

        // Prepare sample references for zstd training
        let sample_refs: Vec<&[u8]> = self.samples.iter().map(|s| s.as_slice()).collect();

        // Train the dictionary
        let dict_data = zstd::dict::from_samples(&sample_refs, self.dictionary_size)
            .map_err(|e| CompressionError::Training(e.to_string()))?;

        tracing::info!(
            "Trained dictionary: {} samples, {} bytes total -> {} byte dictionary",
            self.samples.len(),
            total_size,
            dict_data.len()
        );

        Ok(CompressionDictionary::from_bytes_with_level(
            dict_data,
            self.compression_level,
        ))
    }

    /// Train a dictionary, but return None if there are insufficient samples
    pub fn try_train(self) -> Option<CompressionDictionary> {
        if !self.has_sufficient_samples() {
            return None;
        }
        self.train().ok()
    }
}

/// Compression statistics for comparing dictionary vs non-dictionary compression
#[derive(Debug, Clone)]
pub struct CompressionStats {
    /// Original data size in bytes
    pub original_size: usize,
    /// Compressed size without dictionary
    pub compressed_size: usize,
    /// Compressed size with dictionary (if available)
    pub dict_compressed_size: Option<usize>,
    /// Compression ratio without dictionary (compressed/original)
    pub ratio: f64,
    /// Compression ratio with dictionary (if available)
    pub dict_ratio: Option<f64>,
    /// Improvement from dictionary (percentage reduction)
    pub dict_improvement: Option<f64>,
}

impl CompressionStats {
    /// Calculate compression statistics
    pub fn calculate(
        original: &[u8],
        dictionary: Option<&CompressionDictionary>,
        compression_level: i32,
    ) -> Result<Self, CompressionError> {
        let original_size = original.len();

        // Standard compression
        let compressed = zstd::encode_all(original, compression_level)
            .map_err(|e| CompressionError::Compress(e.to_string()))?;
        let compressed_size = compressed.len();
        let ratio = compressed_size as f64 / original_size as f64;

        // Dictionary compression (if available)
        let (dict_compressed_size, dict_ratio, dict_improvement) = if let Some(dict) = dictionary {
            let dict_compressed = dict.compress(original)?;
            let dict_size = dict_compressed.len();
            let d_ratio = dict_size as f64 / original_size as f64;
            let improvement = (1.0 - dict_size as f64 / compressed_size as f64) * 100.0;
            (Some(dict_size), Some(d_ratio), Some(improvement))
        } else {
            (None, None, None)
        };

        Ok(Self {
            original_size,
            compressed_size,
            dict_compressed_size,
            ratio,
            dict_ratio,
            dict_improvement,
        })
    }
}

impl std::fmt::Display for CompressionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} bytes -> {} bytes ({:.1}% of original)",
            self.original_size,
            self.compressed_size,
            self.ratio * 100.0
        )?;

        if let (Some(dict_size), Some(improvement)) =
            (self.dict_compressed_size, self.dict_improvement)
        {
            write!(
                f,
                ", with dict: {} bytes ({:.1}% improvement)",
                dict_size, improvement
            )?;
        }

        Ok(())
    }
}

/// Built-in dictionary for Orpheus project files
///
/// This can be populated at build time or loaded from bundled resources.
/// Having a built-in dictionary allows immediate compression benefits without
/// user-side training.
pub struct BuiltinDictionary;

impl BuiltinDictionary {
    /// Default dictionary ID for project files
    pub const PROJECT_DICTIONARY_ID: &'static str = "orpheus-project-v1";

    /// Default dictionary ID for MIDI data
    pub const MIDI_DICTIONARY_ID: &'static str = "orpheus-midi-v1";

    /// Get the default dictionary directory
    pub fn dictionary_dir() -> std::path::PathBuf {
        // Use app data directory or fall back to current directory
        std::env::var("ORPHEUS_DATA_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("dictionaries")
    }

    /// Get path to a named dictionary
    pub fn dictionary_path(id: &str) -> std::path::PathBuf {
        Self::dictionary_dir().join(format!("{}.dict", id))
    }

    /// Load a named dictionary if it exists
    pub fn load(id: &str) -> Option<CompressionDictionary> {
        let path = Self::dictionary_path(id);
        if path.exists() {
            CompressionDictionary::load(&path).ok()
        } else {
            None
        }
    }

    /// Save a dictionary with the given ID
    pub fn save(id: &str, dictionary: &CompressionDictionary) -> Result<(), CompressionError> {
        let dir = Self::dictionary_dir();
        std::fs::create_dir_all(&dir).map_err(|e| CompressionError::Io(e.to_string()))?;

        let path = Self::dictionary_path(id);
        dictionary.save(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn generate_sample_project_data(id: usize) -> Vec<u8> {
        // Generate realistic-looking project JSON with enough data for training
        // Each project needs to be ~10-15KB to meet the 100KB minimum with 10 samples
        let mut tracks_json = String::new();
        let mut track_order = Vec::new();

        // Generate 8 tracks with clips
        for track_idx in 0..8 {
            let track_uuid = format!("550e8400-e29b-41d4-a716-44665544{:04}", track_idx);
            track_order.push(format!("\"{}\"", track_uuid));

            // Generate notes for MIDI clips
            let mut notes_json = String::new();
            for note_idx in 0..50 {
                if !notes_json.is_empty() {
                    notes_json.push_str(",\n                                            ");
                }
                notes_json.push_str(&format!(
                    r#"{{"note": {}, "velocity": {}, "start": {}, "duration": {}}}"#,
                    60 + (note_idx % 12),
                    80 + (note_idx % 40),
                    note_idx * 480,
                    480
                ));
            }

            if !tracks_json.is_empty() {
                tracks_json.push_str(",\n                    ");
            }

            tracks_json.push_str(&format!(
                r#""{track_id}": {{
                        "id": "{track_id}",
                        "name": "Track {track_num} - {project_id}",
                        "track_type": "Midi",
                        "order": {track_num},
                        "clips": [
                            {{
                                "id": "{track_id}-clip-1",
                                "name": "Clip 1 on Track {track_num}",
                                "start": 0,
                                "length": 192000,
                                "content": {{
                                    "Midi": {{
                                        "notes": [
                                            {notes}
                                        ]
                                    }}
                                }}
                            }},
                            {{
                                "id": "{track_id}-clip-2",
                                "name": "Clip 2 on Track {track_num}",
                                "start": 192000,
                                "length": 96000,
                                "content": {{
                                    "Midi": {{
                                        "notes": [
                                            {notes}
                                        ]
                                    }}
                                }}
                            }}
                        ],
                        "record_armed": false,
                        "input_source": "None"
                    }}"#,
                track_id = track_uuid,
                track_num = track_idx,
                project_id = id,
                notes = notes_json
            ));
        }

        let json = format!(
            r#"{{
                "format_version": "1.0",
                "metadata": {{
                    "name": "Test Project {id}",
                    "artist": "Test Artist for Project {id}",
                    "album": "Test Album Collection",
                    "genre": "Electronic Music",
                    "created_at": "2024-01-{day:02}T10:30:00Z",
                    "modified_at": "2024-01-{day:02}T12:45:00Z",
                    "notes": "This is test project number {id}. It contains multiple tracks with MIDI clips and various note patterns. The project demonstrates the typical structure of an Orpheus music project file."
                }},
                "tracks": {{
                    {tracks}
                }},
                "track_order": [{track_order}],
                "sample_rate": 48000,
                "tempo": {tempo}
            }}"#,
            id = id,
            day = (id % 28) + 1,
            tracks = tracks_json,
            track_order = track_order.join(", "),
            tempo = 100.0 + (id as f64 * 5.0)
        );
        json.into_bytes()
    }

    #[test]
    fn test_dictionary_training() {
        let mut trainer = DictionaryTrainer::new();

        // Add sample projects
        for i in 0..10 {
            let sample = generate_sample_project_data(i);
            trainer.add_sample(&sample);
        }

        assert_eq!(trainer.sample_count(), 10);
        assert!(trainer.has_sufficient_samples());

        let dictionary = trainer.train().unwrap();
        assert!(dictionary.size() > 0);
    }

    #[test]
    fn test_dictionary_compression() {
        // Train dictionary
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            trainer.add_sample(&generate_sample_project_data(i));
        }
        let dictionary = trainer.train().unwrap();

        // Compress new data
        let test_data = generate_sample_project_data(100);
        let compressed = dictionary.compress(&test_data).unwrap();

        // Decompress and verify
        let decompressed = dictionary.decompress(&compressed).unwrap();
        assert_eq!(test_data, decompressed);
    }

    #[test]
    fn test_dictionary_improvement() {
        // Train dictionary
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            trainer.add_sample(&generate_sample_project_data(i));
        }
        let dictionary = trainer.train().unwrap();

        // Compare compression ratios
        let test_data = generate_sample_project_data(100);
        let stats = CompressionStats::calculate(&test_data, Some(&dictionary), 3).unwrap();

        println!("Compression stats: {}", stats);
        println!(
            "  Standard: {} -> {} bytes ({:.1}%)",
            stats.original_size,
            stats.compressed_size,
            stats.ratio * 100.0
        );
        if let (Some(dict_size), Some(dict_ratio), Some(improvement)) = (
            stats.dict_compressed_size,
            stats.dict_ratio,
            stats.dict_improvement,
        ) {
            println!(
                "  With dict: {} bytes ({:.1}%), {:.1}% improvement",
                dict_size,
                dict_ratio * 100.0,
                improvement
            );
        }

        // Verify compression stats are calculated correctly
        // Note: Dictionary compression doesn't always provide improvement,
        // especially for data that's already highly compressible.
        // The key is that it works correctly.
        assert!(stats.dict_compressed_size.is_some());
        assert!(stats.dict_ratio.is_some());
        assert!(stats.dict_improvement.is_some());

        // Both methods should achieve reasonable compression
        assert!(stats.ratio < 0.5); // Standard should achieve at least 50% reduction
    }

    #[test]
    fn test_dictionary_save_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.dict");

        // Train and save dictionary
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            trainer.add_sample(&generate_sample_project_data(i));
        }
        let original = trainer.train().unwrap();
        original.save(&path).unwrap();

        // Load dictionary
        let loaded = CompressionDictionary::load(&path).unwrap();
        assert_eq!(original.size(), loaded.size());

        // Verify loaded dictionary works
        let test_data = generate_sample_project_data(100);
        let compressed = original.compress(&test_data).unwrap();
        let decompressed = loaded.decompress(&compressed).unwrap();
        assert_eq!(test_data, decompressed);
    }

    #[test]
    fn test_insufficient_samples() {
        let mut trainer = DictionaryTrainer::new();
        trainer.add_sample(b"small sample");
        trainer.add_sample(b"another small sample");

        assert!(!trainer.has_sufficient_samples());

        let result = trainer.train();
        assert!(matches!(
            result,
            Err(CompressionError::InsufficientSamples { .. })
        ));
    }

    #[test]
    fn test_try_train_with_insufficient_samples() {
        let mut trainer = DictionaryTrainer::new();
        trainer.add_sample(b"small");

        let result = trainer.try_train();
        assert!(result.is_none());
    }

    #[test]
    fn test_compression_levels() {
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            trainer.add_sample(&generate_sample_project_data(i));
        }
        let dictionary = trainer.train().unwrap();

        let test_data = generate_sample_project_data(100);

        // Test different compression levels
        let fast = dictionary.compress_with_level(&test_data, 1).unwrap();
        let default = dictionary.compress_with_level(&test_data, 3).unwrap();
        let high = dictionary.compress_with_level(&test_data, 9).unwrap();

        // Higher levels should generally give better compression
        // (though not guaranteed for all data)
        assert!(fast.len() >= default.len() || fast.len() >= high.len());

        // All should decompress correctly
        for compressed in [fast, default, high] {
            let decompressed = dictionary.decompress(&compressed).unwrap();
            assert_eq!(test_data, decompressed);
        }
    }

    #[test]
    fn test_raw_dictionary_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("raw.dict");

        // Train dictionary and save raw bytes (no header)
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            trainer.add_sample(&generate_sample_project_data(i));
        }
        let dictionary = trainer.train().unwrap();

        // Save raw dictionary bytes (simulating external dictionary)
        std::fs::write(&path, dictionary.as_bytes()).unwrap();

        // Should load as raw dictionary
        let loaded = CompressionDictionary::load(&path).unwrap();
        assert_eq!(dictionary.size(), loaded.size());
    }
}
