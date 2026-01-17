//! # Orpheus Export
//!
//! Export functionality for Orpheus - WAV, FLAC, AIFF, and more.
//!
//! Supports true lossless formats for professional audio workflows.
//!
//! # Example
//!
//! ```rust,ignore
//! use orpheus_export::{export_project, ExportSettings, ExportFormat};
//! use orpheus_core::Project;
//!
//! let project = Project::new("My Song");
//! let settings = ExportSettings::high_quality();
//! export_project(&project, "output.wav", &settings)?;
//! ```

pub mod renderer;
pub mod tab_renderer;

use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use hound::{WavSpec, WavWriter, SampleFormat};
use orpheus_core::Project;
use orpheus_core::tab::TabDocument;
use orpheus_core::compression::CompressionDictionary;

pub use renderer::{OfflineRenderer, ProgressRenderer, ProgressCallback};
pub use tab_renderer::{TabRenderer, TabProgressRenderer};

/// Result type for export operations
pub type Result<T> = std::result::Result<T, Error>;

/// Export error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("Format not supported: {0}")]
    UnsupportedFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Audio encoding error: {0}")]
    Encoding(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("No audio data to export")]
    NoAudioData,
}

/// Export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// 16-bit PCM WAV
    Wav16,
    /// 24-bit PCM WAV
    Wav24,
    /// 32-bit float WAV
    Wav32Float,
    /// FLAC 16-bit (lossless compressed)
    Flac16,
    /// FLAC 24-bit (lossless compressed)
    Flac24,
    /// AIFF 16-bit
    Aiff16,
    /// AIFF 24-bit
    Aiff24,
    /// MP3 (lossy) - not yet implemented
    Mp3,
    /// OGG Vorbis (lossy) - not yet implemented
    Ogg,
    /// AAC (lossy) - not yet implemented
    Aac,
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self::Wav24
    }
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Wav16 | Self::Wav24 | Self::Wav32Float => "wav",
            Self::Flac16 | Self::Flac24 => "flac",
            Self::Aiff16 | Self::Aiff24 => "aiff",
            Self::Mp3 => "mp3",
            Self::Ogg => "ogg",
            Self::Aac => "m4a",
        }
    }

    pub fn is_wav(&self) -> bool {
        matches!(self, Self::Wav16 | Self::Wav24 | Self::Wav32Float)
    }

    pub fn is_flac(&self) -> bool {
        matches!(self, Self::Flac16 | Self::Flac24)
    }

    pub fn is_aiff(&self) -> bool {
        matches!(self, Self::Aiff16 | Self::Aiff24)
    }

    pub fn is_lossless(&self) -> bool {
        matches!(self,
            Self::Wav16 | Self::Wav24 | Self::Wav32Float |
            Self::Flac16 | Self::Flac24 |
            Self::Aiff16 | Self::Aiff24
        )
    }

    pub fn bits_per_sample(&self) -> u16 {
        match self {
            Self::Wav16 | Self::Flac16 | Self::Aiff16 => 16,
            Self::Wav24 | Self::Flac24 | Self::Aiff24 => 24,
            Self::Wav32Float => 32,
            Self::Mp3 | Self::Ogg | Self::Aac => 16, // Lossy formats don't have fixed bit depth
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Wav16 => "WAV 16-bit PCM",
            Self::Wav24 => "WAV 24-bit PCM",
            Self::Wav32Float => "WAV 32-bit Float",
            Self::Flac16 => "FLAC 16-bit (lossless)",
            Self::Flac24 => "FLAC 24-bit (lossless)",
            Self::Aiff16 => "AIFF 16-bit",
            Self::Aiff24 => "AIFF 24-bit",
            Self::Mp3 => "MP3 (lossy)",
            Self::Ogg => "OGG Vorbis (lossy)",
            Self::Aac => "AAC (lossy)",
        }
    }

    /// Get all available lossless formats
    pub fn lossless_formats() -> &'static [ExportFormat] {
        &[
            Self::Wav16, Self::Wav24, Self::Wav32Float,
            Self::Flac16, Self::Flac24,
            Self::Aiff16, Self::Aiff24,
        ]
    }

    /// Get all available lossy formats
    pub fn lossy_formats() -> &'static [ExportFormat] {
        &[Self::Mp3, Self::Ogg, Self::Aac]
    }

    /// Check if this format is currently available (has required dependencies)
    pub fn is_available(&self) -> bool {
        match self {
            Self::Wav16 | Self::Wav24 | Self::Wav32Float => true,
            Self::Flac16 | Self::Flac24 => true,
            Self::Aiff16 | Self::Aiff24 => true,
            #[cfg(feature = "mp3")]
            Self::Mp3 => true,
            #[cfg(not(feature = "mp3"))]
            Self::Mp3 => false,
            #[cfg(feature = "ogg")]
            Self::Ogg => true,
            #[cfg(not(feature = "ogg"))]
            Self::Ogg => false,
            Self::Aac => false, // Not implemented
        }
    }

    /// Get all currently available formats
    pub fn available_formats() -> Vec<ExportFormat> {
        [
            Self::Wav16, Self::Wav24, Self::Wav32Float,
            Self::Flac16, Self::Flac24,
            Self::Aiff16, Self::Aiff24,
            Self::Mp3, Self::Ogg, Self::Aac,
        ]
        .into_iter()
        .filter(|f| f.is_available())
        .collect()
    }
}

/// FLAC compression level (0-8, higher = smaller file, slower encoding)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlacCompressionLevel(u8);

impl Default for FlacCompressionLevel {
    fn default() -> Self {
        Self(5) // Good balance of size and speed
    }
}

impl FlacCompressionLevel {
    pub fn fast() -> Self { Self(0) }
    pub fn default_level() -> Self { Self(5) }
    pub fn best() -> Self { Self(8) }

    pub fn level(&self) -> u8 {
        self.0.min(8)
    }
}

/// Zstd compression settings for compressed audio archival
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZstdCompression {
    /// Compression level (1-22, higher = better compression, slower)
    level: i32,
}

impl Default for ZstdCompression {
    fn default() -> Self {
        Self { level: 3 } // Good balance of speed and ratio
    }
}

impl ZstdCompression {
    /// Fast compression (level 1) - fastest, larger files
    pub fn fast() -> Self { Self { level: 1 } }

    /// Default compression (level 3) - good balance
    pub fn default_level() -> Self { Self { level: 3 } }

    /// High compression (level 9) - smaller files, slower
    pub fn high() -> Self { Self { level: 9 } }

    /// Maximum compression (level 19) - smallest files, slowest
    pub fn max() -> Self { Self { level: 19 } }

    /// Custom compression level (clamped to 1-22)
    pub fn custom(level: i32) -> Self {
        Self { level: level.clamp(1, 22) }
    }

    /// Get the compression level
    pub fn level(&self) -> i32 {
        self.level
    }
}

/// Export settings
#[derive(Debug, Clone)]
pub struct ExportSettings {
    /// Output format
    pub format: ExportFormat,
    /// Sample rate (e.g., 44100, 48000, 96000)
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Normalize audio to this peak dB (None = no normalization)
    pub normalize_to_db: Option<f32>,
    /// Apply dithering when converting to lower bit depths
    pub dither: bool,
    /// FLAC compression level
    pub flac_compression: FlacCompressionLevel,
    /// Zstd compression for archival (None = no compression, outputs .wav/.flac)
    /// When enabled, output is compressed with zstd (e.g., .wav.zst, .flac.zst)
    pub zstd_compression: Option<ZstdCompression>,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: ExportFormat::Wav24,
            sample_rate: 44100,
            channels: 2,
            normalize_to_db: None,
            dither: true,
            flac_compression: FlacCompressionLevel::default(),
            zstd_compression: None,
        }
    }
}

impl ExportSettings {
    /// Create settings for CD-quality WAV (16-bit, 44.1kHz stereo)
    pub fn cd_quality() -> Self {
        Self {
            format: ExportFormat::Wav16,
            sample_rate: 44100,
            channels: 2,
            normalize_to_db: Some(-0.3),
            dither: true,
            flac_compression: FlacCompressionLevel::default(),
            zstd_compression: None,
        }
    }

    /// Create settings for high-quality WAV (24-bit, 48kHz stereo)
    pub fn high_quality() -> Self {
        Self {
            format: ExportFormat::Wav24,
            sample_rate: 48000,
            channels: 2,
            normalize_to_db: None,
            dither: false,
            flac_compression: FlacCompressionLevel::default(),
            zstd_compression: None,
        }
    }

    /// Create settings for broadcast WAV (24-bit, 48kHz stereo)
    pub fn broadcast() -> Self {
        Self {
            format: ExportFormat::Wav24,
            sample_rate: 48000,
            channels: 2,
            normalize_to_db: Some(-1.0),
            dither: false,
            flac_compression: FlacCompressionLevel::default(),
            zstd_compression: None,
        }
    }

    /// Create settings for FLAC archival (24-bit, 96kHz stereo, best compression)
    pub fn flac_archival() -> Self {
        Self {
            format: ExportFormat::Flac24,
            sample_rate: 96000,
            channels: 2,
            normalize_to_db: None,
            dither: false,
            flac_compression: FlacCompressionLevel::best(),
            zstd_compression: None,
        }
    }

    /// Create settings for FLAC distribution (16-bit, 44.1kHz stereo)
    pub fn flac_distribution() -> Self {
        Self {
            format: ExportFormat::Flac16,
            sample_rate: 44100,
            channels: 2,
            normalize_to_db: Some(-0.3),
            dither: true,
            flac_compression: FlacCompressionLevel::default(),
            zstd_compression: None,
        }
    }

    /// Create settings for compressed WAV archival (24-bit, 48kHz stereo, zstd compressed)
    ///
    /// Outputs .wav.zst file with ~40-60% size reduction
    pub fn wav_compressed() -> Self {
        Self {
            format: ExportFormat::Wav24,
            sample_rate: 48000,
            channels: 2,
            normalize_to_db: None,
            dither: false,
            flac_compression: FlacCompressionLevel::default(),
            zstd_compression: Some(ZstdCompression::default_level()),
        }
    }

    /// Create settings for maximum compressed archival (24-bit, 48kHz stereo, high zstd)
    ///
    /// Outputs .wav.zst file with maximum compression for long-term storage
    pub fn wav_max_compression() -> Self {
        Self {
            format: ExportFormat::Wav24,
            sample_rate: 48000,
            channels: 2,
            normalize_to_db: None,
            dither: false,
            flac_compression: FlacCompressionLevel::default(),
            zstd_compression: Some(ZstdCompression::high()),
        }
    }

    /// Create settings for MP3 (320kbps, 44.1kHz stereo)
    ///
    /// Note: Requires 'mp3' feature to be enabled
    pub fn mp3_high_quality() -> Self {
        Self {
            format: ExportFormat::Mp3,
            sample_rate: 44100,
            channels: 2,
            normalize_to_db: Some(-0.5),
            dither: false, // MP3 encoder handles internally
            flac_compression: FlacCompressionLevel::default(),
            zstd_compression: None,
        }
    }

    /// Create settings for OGG Vorbis (~256kbps equivalent, 44.1kHz stereo)
    ///
    /// Note: Requires 'ogg' feature to be enabled
    pub fn ogg_high_quality() -> Self {
        Self {
            format: ExportFormat::Ogg,
            sample_rate: 44100,
            channels: 2,
            normalize_to_db: Some(-0.5),
            dither: false, // Vorbis encoder handles internally
            flac_compression: FlacCompressionLevel::default(),
            zstd_compression: None,
        }
    }

    /// Get the file extension for these settings (includes .zst if compressed)
    pub fn extension(&self) -> String {
        let base = self.format.extension();
        if self.zstd_compression.is_some() {
            format!("{}.zst", base)
        } else {
            base.to_string()
        }
    }

    /// Check if zstd compression is enabled
    pub fn is_compressed(&self) -> bool {
        self.zstd_compression.is_some()
    }

    /// Enable zstd compression with default settings
    pub fn with_compression(mut self) -> Self {
        self.zstd_compression = Some(ZstdCompression::default_level());
        self
    }

    /// Enable zstd compression with custom settings
    pub fn with_compression_level(mut self, compression: ZstdCompression) -> Self {
        self.zstd_compression = Some(compression);
        self
    }

    /// Disable zstd compression
    pub fn without_compression(mut self) -> Self {
        self.zstd_compression = None;
        self
    }
}

/// Audio exporter
pub struct Exporter {
    settings: ExportSettings,
}

impl Default for Exporter {
    fn default() -> Self {
        Self::new(ExportSettings::default())
    }
}

impl Exporter {
    /// Create a new exporter with the given settings
    pub fn new(settings: ExportSettings) -> Self {
        Self { settings }
    }

    /// Export audio samples to a file
    ///
    /// # Arguments
    /// * `samples` - Interleaved audio samples normalized to -1.0 to 1.0
    /// * `path` - Output file path
    ///
    /// If zstd compression is enabled, the audio is first encoded to memory,
    /// then compressed and written to the file.
    pub fn export(&self, samples: &[f32], path: impl AsRef<Path>) -> Result<()> {
        if samples.is_empty() {
            return Err(Error::NoAudioData);
        }

        let samples = if let Some(target_db) = self.settings.normalize_to_db {
            normalize_samples(samples, target_db)
        } else {
            samples.to_vec()
        };

        // If compression is enabled, we need a different approach
        if let Some(zstd_settings) = &self.settings.zstd_compression {
            self.export_compressed(&samples, path.as_ref(), zstd_settings)
        } else {
            self.export_uncompressed(&samples, path)
        }
    }

    /// Export without compression (standard export)
    fn export_uncompressed(&self, samples: &[f32], path: impl AsRef<Path>) -> Result<()> {
        match self.settings.format {
            ExportFormat::Wav16 => self.export_wav_int(samples, path, 16),
            ExportFormat::Wav24 => self.export_wav_int(samples, path, 24),
            ExportFormat::Wav32Float => self.export_wav_float(samples, path),
            ExportFormat::Flac16 => self.export_flac(samples, path, 16),
            ExportFormat::Flac24 => self.export_flac(samples, path, 24),
            ExportFormat::Aiff16 => self.export_aiff(samples, path, 16),
            ExportFormat::Aiff24 => self.export_aiff(samples, path, 24),
            #[cfg(feature = "mp3")]
            ExportFormat::Mp3 => self.export_mp3(samples, path),
            #[cfg(not(feature = "mp3"))]
            ExportFormat::Mp3 => Err(Error::UnsupportedFormat(
                "MP3 export requires 'mp3' feature (libmp3lame)".into()
            )),
            #[cfg(feature = "ogg")]
            ExportFormat::Ogg => self.export_ogg(samples, path),
            #[cfg(not(feature = "ogg"))]
            ExportFormat::Ogg => Err(Error::UnsupportedFormat(
                "OGG export requires 'ogg' feature (libvorbis)".into()
            )),
            ExportFormat::Aac => Err(Error::UnsupportedFormat(
                "AAC export not yet implemented".into()
            )),
        }
    }

    /// Export with zstd compression
    ///
    /// Encodes audio to memory buffer, then compresses with zstd
    fn export_compressed(&self, samples: &[f32], path: &Path, zstd_settings: &ZstdCompression) -> Result<()> {
        // First encode to a memory buffer
        let audio_data = match self.settings.format {
            ExportFormat::Wav16 => self.encode_wav_int_to_memory(samples, 16)?,
            ExportFormat::Wav24 => self.encode_wav_int_to_memory(samples, 24)?,
            ExportFormat::Wav32Float => self.encode_wav_float_to_memory(samples)?,
            ExportFormat::Flac16 => self.encode_flac_to_memory(samples, 16)?,
            ExportFormat::Flac24 => self.encode_flac_to_memory(samples, 24)?,
            ExportFormat::Aiff16 => self.encode_aiff_to_memory(samples, 16)?,
            ExportFormat::Aiff24 => self.encode_aiff_to_memory(samples, 24)?,
            _ => return Err(Error::UnsupportedFormat(
                "Compression only supported for lossless formats".into()
            )),
        };

        // Compress with zstd
        let compressed = zstd::encode_all(audio_data.as_slice(), zstd_settings.level())
            .map_err(|e| Error::Compression(e.to_string()))?;

        // Write to file
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&compressed)?;
        writer.flush()?;

        tracing::debug!(
            "Exported compressed audio: {} bytes -> {} bytes ({:.1}% of original)",
            audio_data.len(),
            compressed.len(),
            (compressed.len() as f64 / audio_data.len() as f64) * 100.0
        );

        Ok(())
    }

    /// Encode WAV to memory buffer (integer PCM)
    fn encode_wav_int_to_memory(&self, samples: &[f32], bits: u16) -> Result<Vec<u8>> {
        let spec = WavSpec {
            channels: self.settings.channels,
            sample_rate: self.settings.sample_rate,
            bits_per_sample: bits,
            sample_format: SampleFormat::Int,
        };

        let mut buffer = std::io::Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut buffer, spec)
                .map_err(|e| Error::Encoding(e.to_string()))?;

            let max_value = ((1i64 << (bits - 1)) - 1) as f32;

            for &sample in samples {
                let clamped = sample.clamp(-1.0, 1.0);
                let dithered = if self.settings.dither && bits < 32 {
                    let dither = triangular_dither() / max_value;
                    clamped + dither
                } else {
                    clamped
                };
                let int_sample = (dithered * max_value).round() as i32;
                writer.write_sample(int_sample)
                    .map_err(|e| Error::Encoding(e.to_string()))?;
            }

            writer.finalize()
                .map_err(|e| Error::Encoding(e.to_string()))?;
        }

        Ok(buffer.into_inner())
    }

    /// Encode WAV to memory buffer (32-bit float)
    fn encode_wav_float_to_memory(&self, samples: &[f32]) -> Result<Vec<u8>> {
        let spec = WavSpec {
            channels: self.settings.channels,
            sample_rate: self.settings.sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };

        let mut buffer = std::io::Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut buffer, spec)
                .map_err(|e| Error::Encoding(e.to_string()))?;

            for &sample in samples {
                writer.write_sample(sample)
                    .map_err(|e| Error::Encoding(e.to_string()))?;
            }

            writer.finalize()
                .map_err(|e| Error::Encoding(e.to_string()))?;
        }

        Ok(buffer.into_inner())
    }

    /// Encode FLAC to memory buffer
    fn encode_flac_to_memory(&self, samples: &[f32], bits: u16) -> Result<Vec<u8>> {
        use flacenc::component::BitRepr;
        use flacenc::config::Encoder as EncoderConfig;
        use flacenc::error::Verify;
        use flacenc::source::MemSource;

        let channels = self.settings.channels as usize;
        let sample_rate = self.settings.sample_rate as usize;

        let max_value = ((1i64 << (bits - 1)) - 1) as f32;
        let mut int_samples: Vec<i32> = Vec::with_capacity(samples.len());

        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let dithered = if self.settings.dither && bits < 24 {
                let dither = triangular_dither() / max_value;
                clamped + dither
            } else {
                clamped
            };
            int_samples.push((dithered * max_value).round() as i32);
        }

        let config = EncoderConfig::default()
            .into_verified()
            .map_err(|(_, e)| Error::Encoding(format!("FLAC config error: {:?}", e)))?;

        let source = MemSource::from_samples(&int_samples, channels, bits as usize, sample_rate);

        let flac_stream = flacenc::encode_with_fixed_block_size(&config, source, 4096)
            .map_err(|e| Error::Encoding(format!("FLAC encode error: {:?}", e)))?;

        let mut sink = flacenc::bitsink::ByteSink::new();
        flac_stream.write(&mut sink)
            .map_err(|e| Error::Encoding(format!("FLAC write error: {:?}", e)))?;

        Ok(sink.into_inner())
    }

    /// Encode AIFF to memory buffer
    fn encode_aiff_to_memory(&self, samples: &[f32], bits: u16) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        let channels = self.settings.channels;
        let sample_rate = self.settings.sample_rate;
        let frame_count = samples.len() / channels as usize;
        let bytes_per_sample = bits / 8;
        let sample_data_size = frame_count * channels as usize * bytes_per_sample as usize;

        let comm_chunk_size = 18u32;
        let ssnd_chunk_size = (8 + sample_data_size) as u32;
        let form_size = 4 + 8 + comm_chunk_size + 8 + ssnd_chunk_size;

        // FORM header
        buffer.extend_from_slice(b"FORM");
        buffer.extend_from_slice(&form_size.to_be_bytes());
        buffer.extend_from_slice(b"AIFF");

        // COMM chunk
        buffer.extend_from_slice(b"COMM");
        buffer.extend_from_slice(&comm_chunk_size.to_be_bytes());
        buffer.extend_from_slice(&channels.to_be_bytes());
        buffer.extend_from_slice(&(frame_count as u32).to_be_bytes());
        buffer.extend_from_slice(&bits.to_be_bytes());
        buffer.extend_from_slice(&float_to_ieee_extended(sample_rate as f64));

        // SSND chunk
        buffer.extend_from_slice(b"SSND");
        buffer.extend_from_slice(&ssnd_chunk_size.to_be_bytes());
        buffer.extend_from_slice(&0u32.to_be_bytes()); // offset
        buffer.extend_from_slice(&0u32.to_be_bytes()); // blockSize

        let max_value = ((1i64 << (bits - 1)) - 1) as f32;

        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let dithered = if self.settings.dither && bits < 24 {
                let dither = triangular_dither() / max_value;
                clamped + dither
            } else {
                clamped
            };
            let int_sample = (dithered * max_value).round() as i32;

            match bits {
                16 => buffer.extend_from_slice(&(int_sample as i16).to_be_bytes()),
                24 => {
                    let bytes = int_sample.to_be_bytes();
                    buffer.extend_from_slice(&bytes[1..4]);
                }
                _ => return Err(Error::Encoding(format!("Unsupported AIFF bit depth: {}", bits))),
            }
        }

        Ok(buffer)
    }

    /// Export to integer PCM WAV
    fn export_wav_int(&self, samples: &[f32], path: impl AsRef<Path>, bits: u16) -> Result<()> {
        let spec = WavSpec {
            channels: self.settings.channels,
            sample_rate: self.settings.sample_rate,
            bits_per_sample: bits,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path.as_ref(), spec)
            .map_err(|e| Error::Encoding(e.to_string()))?;

        let max_value = ((1i64 << (bits - 1)) - 1) as f32;

        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let dithered = if self.settings.dither && bits < 32 {
                let dither = triangular_dither() / max_value;
                clamped + dither
            } else {
                clamped
            };
            let int_sample = (dithered * max_value).round() as i32;
            writer.write_sample(int_sample)
                .map_err(|e| Error::Encoding(e.to_string()))?;
        }

        writer.finalize()
            .map_err(|e| Error::Encoding(e.to_string()))?;

        Ok(())
    }

    /// Export to 32-bit float WAV
    fn export_wav_float(&self, samples: &[f32], path: impl AsRef<Path>) -> Result<()> {
        let spec = WavSpec {
            channels: self.settings.channels,
            sample_rate: self.settings.sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };

        let mut writer = WavWriter::create(path.as_ref(), spec)
            .map_err(|e| Error::Encoding(e.to_string()))?;

        for &sample in samples {
            writer.write_sample(sample)
                .map_err(|e| Error::Encoding(e.to_string()))?;
        }

        writer.finalize()
            .map_err(|e| Error::Encoding(e.to_string()))?;

        Ok(())
    }

    /// Export to FLAC
    fn export_flac(&self, samples: &[f32], path: impl AsRef<Path>, bits: u16) -> Result<()> {
        use flacenc::component::BitRepr;
        use flacenc::config::Encoder as EncoderConfig;
        use flacenc::error::Verify;
        use flacenc::source::MemSource;

        let channels = self.settings.channels as usize;
        let sample_rate = self.settings.sample_rate as usize;

        // Convert f32 samples to i32 samples (FLAC uses integer samples)
        // Samples should remain interleaved
        let max_value = ((1i64 << (bits - 1)) - 1) as f32;
        let mut int_samples: Vec<i32> = Vec::with_capacity(samples.len());

        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let dithered = if self.settings.dither && bits < 24 {
                let dither = triangular_dither() / max_value;
                clamped + dither
            } else {
                clamped
            };
            int_samples.push((dithered * max_value).round() as i32);
        }

        // Create FLAC encoder config with defaults
        let config = EncoderConfig::default()
            .into_verified()
            .map_err(|(_, e)| Error::Encoding(format!("FLAC config error: {:?}", e)))?;

        // Create source from interleaved samples
        let source = MemSource::from_samples(&int_samples, channels, bits as usize, sample_rate);

        // Encode
        let flac_stream = flacenc::encode_with_fixed_block_size(
            &config,
            source,
            4096, // block size
        ).map_err(|e| Error::Encoding(format!("FLAC encode error: {:?}", e)))?;

        // Write to file
        let file = File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);

        // Write FLAC stream
        let mut sink = flacenc::bitsink::ByteSink::new();
        flac_stream.write(&mut sink)
            .map_err(|e| Error::Encoding(format!("FLAC write error: {:?}", e)))?;

        writer.write_all(sink.as_slice())?;
        writer.flush()?;

        Ok(())
    }

    /// Export to AIFF
    fn export_aiff(&self, samples: &[f32], path: impl AsRef<Path>, bits: u16) -> Result<()> {
        let file = File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);

        let channels = self.settings.channels;
        let sample_rate = self.settings.sample_rate;
        let frame_count = samples.len() / channels as usize;
        let bytes_per_sample = bits / 8;
        let sample_data_size = frame_count * channels as usize * bytes_per_sample as usize;

        // AIFF uses big-endian byte order

        // Calculate chunk sizes
        let comm_chunk_size = 18u32; // Fixed size for COMM chunk
        let ssnd_chunk_size = (8 + sample_data_size) as u32; // offset + blockSize + data
        let form_size = 4 + 8 + comm_chunk_size + 8 + ssnd_chunk_size; // AIFF + chunks with headers

        // Write FORM header
        writer.write_all(b"FORM")?;
        writer.write_all(&form_size.to_be_bytes())?;
        writer.write_all(b"AIFF")?;

        // Write COMM chunk
        writer.write_all(b"COMM")?;
        writer.write_all(&comm_chunk_size.to_be_bytes())?;
        writer.write_all(&channels.to_be_bytes())?; // numChannels
        writer.write_all(&(frame_count as u32).to_be_bytes())?; // numSampleFrames
        writer.write_all(&bits.to_be_bytes())?; // sampleSize

        // Write sample rate as 80-bit IEEE 754 extended
        let sample_rate_extended = float_to_ieee_extended(sample_rate as f64);
        writer.write_all(&sample_rate_extended)?;

        // Write SSND chunk
        writer.write_all(b"SSND")?;
        writer.write_all(&ssnd_chunk_size.to_be_bytes())?;
        writer.write_all(&0u32.to_be_bytes())?; // offset
        writer.write_all(&0u32.to_be_bytes())?; // blockSize

        // Write sample data (big-endian)
        let max_value = ((1i64 << (bits - 1)) - 1) as f32;

        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let dithered = if self.settings.dither && bits < 24 {
                let dither = triangular_dither() / max_value;
                clamped + dither
            } else {
                clamped
            };
            let int_sample = (dithered * max_value).round() as i32;

            match bits {
                16 => {
                    writer.write_all(&(int_sample as i16).to_be_bytes())?;
                }
                24 => {
                    let bytes = int_sample.to_be_bytes();
                    writer.write_all(&bytes[1..4])?; // Skip highest byte
                }
                _ => return Err(Error::Encoding(format!("Unsupported AIFF bit depth: {}", bits))),
            }
        }

        writer.flush()?;
        Ok(())
    }

    /// Export to MP3 (requires 'mp3' feature)
    #[cfg(feature = "mp3")]
    fn export_mp3(&self, samples: &[f32], path: impl AsRef<Path>) -> Result<()> {
        use mp3lame_encoder::{Builder, FlushNoGap, InterleavedPcm};
        use std::mem::MaybeUninit;

        let channels = self.settings.channels as usize;
        let sample_rate = self.settings.sample_rate;

        // Create MP3 encoder
        let mut encoder = Builder::new()
            .ok_or_else(|| Error::Encoding("Failed to create MP3 encoder".into()))?;

        encoder.set_num_channels(channels as u8)
            .map_err(|e| Error::Encoding(format!("MP3 channel config error: {:?}", e)))?;
        encoder.set_sample_rate(sample_rate)
            .map_err(|e| Error::Encoding(format!("MP3 sample rate error: {:?}", e)))?;
        encoder.set_brate(mp3lame_encoder::Bitrate::Kbps320)
            .map_err(|e| Error::Encoding(format!("MP3 bitrate error: {:?}", e)))?;
        encoder.set_quality(mp3lame_encoder::Quality::Best)
            .map_err(|e| Error::Encoding(format!("MP3 quality error: {:?}", e)))?;

        let mut encoder = encoder.build()
            .map_err(|e| Error::Encoding(format!("MP3 encoder build error: {:?}", e)))?;

        // Convert f32 samples to i16
        let int_samples: Vec<i16> = samples.iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
            .collect();

        // Allocate output buffer - estimate 1.25x input for MP3 output
        // Using uninit buffer as required by mp3lame-encoder API
        let output_size = samples.len() * 5 / 4 + 7200; // LAME recommends extra padding
        let mut output_buffer: Vec<MaybeUninit<u8>> = vec![MaybeUninit::uninit(); output_size];

        // Encode
        let input = InterleavedPcm(&int_samples);
        let encoded_len = encoder.encode(input, &mut output_buffer)
            .map_err(|e| Error::Encoding(format!("MP3 encode error: {:?}", e)))?;

        // Flush encoder
        let flush_len = encoder.flush::<FlushNoGap>(&mut output_buffer[encoded_len..])
            .map_err(|e| Error::Encoding(format!("MP3 flush error: {:?}", e)))?;

        // Extract the initialized portion
        let total_len = encoded_len + flush_len;
        let mp3_data: Vec<u8> = output_buffer[..total_len]
            .iter()
            .map(|b| unsafe { b.assume_init() })
            .collect();

        // Write to file
        let file = File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&mp3_data)?;
        writer.flush()?;

        Ok(())
    }

    /// Export to OGG Vorbis (requires 'ogg' feature)
    #[cfg(feature = "ogg")]
    fn export_ogg(&self, samples: &[f32], path: impl AsRef<Path>) -> Result<()> {
        use vorbis_encoder::Encoder;

        let channels = self.settings.channels as u32;
        let sample_rate = self.settings.sample_rate;

        // Create Vorbis encoder with high quality (0.8 = ~256kbps equivalent)
        let mut encoder = Encoder::new(channels, sample_rate as u64, 0.8)
            .map_err(|e| Error::Encoding(format!("OGG encoder error: {:?}", e)))?;

        // Convert f32 samples to i16 interleaved
        let int_samples: Vec<i16> = samples.iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
            .collect();

        // Encode - vorbis_encoder encodes all samples at once and returns complete OGG data
        let encoded = encoder.encode(&int_samples)
            .map_err(|e| Error::Encoding(format!("OGG encode error: {:?}", e)))?;

        // Write to file
        let file = File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&encoded)?;
        writer.flush()?;

        Ok(())
    }

    /// Get the current export settings
    pub fn settings(&self) -> &ExportSettings {
        &self.settings
    }

    /// Update export settings
    pub fn set_settings(&mut self, settings: ExportSettings) {
        self.settings = settings;
    }

    /// Export with dictionary compression for better compression ratios
    ///
    /// Dictionary compression can achieve 20-40% better compression than standard
    /// zstd for audio data with similar patterns (e.g., same instrument, similar mix).
    ///
    /// # Arguments
    /// * `samples` - Interleaved audio samples normalized to -1.0 to 1.0
    /// * `path` - Output file path
    /// * `dictionary` - Trained compression dictionary
    ///
    /// # Returns
    /// Ok(()) on success, or an error if export fails
    pub fn export_with_dictionary(
        &self,
        samples: &[f32],
        path: impl AsRef<Path>,
        dictionary: &CompressionDictionary,
    ) -> Result<()> {
        if samples.is_empty() {
            return Err(Error::NoAudioData);
        }

        let samples = if let Some(target_db) = self.settings.normalize_to_db {
            normalize_samples(samples, target_db)
        } else {
            samples.to_vec()
        };

        self.export_with_dictionary_internal(&samples, path.as_ref(), dictionary)
    }

    /// Internal implementation of dictionary compression export
    fn export_with_dictionary_internal(
        &self,
        samples: &[f32],
        path: &Path,
        dictionary: &CompressionDictionary,
    ) -> Result<()> {
        // First encode to a memory buffer
        let audio_data = match self.settings.format {
            ExportFormat::Wav16 => self.encode_wav_int_to_memory(samples, 16)?,
            ExportFormat::Wav24 => self.encode_wav_int_to_memory(samples, 24)?,
            ExportFormat::Wav32Float => self.encode_wav_float_to_memory(samples)?,
            ExportFormat::Flac16 => self.encode_flac_to_memory(samples, 16)?,
            ExportFormat::Flac24 => self.encode_flac_to_memory(samples, 24)?,
            ExportFormat::Aiff16 => self.encode_aiff_to_memory(samples, 16)?,
            ExportFormat::Aiff24 => self.encode_aiff_to_memory(samples, 24)?,
            _ => {
                return Err(Error::UnsupportedFormat(
                    "Dictionary compression only supported for lossless formats".into(),
                ))
            }
        };

        // Compress with dictionary
        let compressed = dictionary
            .compress(&audio_data)
            .map_err(|e| Error::Compression(e.to_string()))?;

        // Write to file
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&compressed)?;
        writer.flush()?;

        tracing::debug!(
            "Exported with dictionary: {} bytes -> {} bytes ({:.1}% of original)",
            audio_data.len(),
            compressed.len(),
            (compressed.len() as f64 / audio_data.len() as f64) * 100.0
        );

        Ok(())
    }
}

/// Convert a float to IEEE 754 80-bit extended precision
/// Used for AIFF sample rate field
fn float_to_ieee_extended(value: f64) -> [u8; 10] {
    if value == 0.0 {
        return [0; 10];
    }

    let sign = if value < 0.0 { 1u8 } else { 0u8 };
    let value = value.abs();

    // Calculate exponent and mantissa
    let log2 = value.log2();
    let exponent = log2.floor() as i32;
    let mantissa = value / 2.0f64.powi(exponent);

    // IEEE 754 extended uses bias of 16383
    let biased_exp = (exponent + 16383) as u16;

    // Convert mantissa to 64-bit integer (explicit integer bit)
    let mantissa_int = (mantissa * (1u64 << 63) as f64) as u64;

    let mut bytes = [0u8; 10];
    bytes[0] = (sign << 7) | ((biased_exp >> 8) as u8 & 0x7F);
    bytes[1] = biased_exp as u8;
    let mantissa_bytes = mantissa_int.to_be_bytes();
    bytes[2..10].copy_from_slice(&mantissa_bytes);

    bytes
}

/// Normalize audio samples to a target peak level in dB
pub fn normalize_samples(samples: &[f32], target_db: f32) -> Vec<f32> {
    let current_peak = peak_amplitude(samples);
    if current_peak < 1e-10 {
        return samples.to_vec();
    }

    let target_linear = db_to_linear(target_db);
    let gain = target_linear / current_peak;

    samples.iter().map(|&s| s * gain).collect()
}

/// Find the peak amplitude of samples
pub fn peak_amplitude(samples: &[f32]) -> f32 {
    samples.iter()
        .map(|s| s.abs())
        .fold(0.0f32, f32::max)
}

/// Convert dB to linear amplitude
pub fn db_to_linear(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

/// Convert linear amplitude to dB
pub fn linear_to_db(linear: f32) -> f32 {
    if linear < 1e-10 {
        -200.0
    } else {
        20.0 * linear.log10()
    }
}

/// Calculate peak level in dB
pub fn peak_db(samples: &[f32]) -> f32 {
    linear_to_db(peak_amplitude(samples))
}

/// Calculate RMS level in dB
pub fn rms_db(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return -200.0;
    }

    let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
    let rms = (sum_squares / samples.len() as f32).sqrt();
    linear_to_db(rms)
}

/// Generate triangular dither noise
fn triangular_dither() -> f32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEED: AtomicU32 = AtomicU32::new(0x12345678);

    let mut state = SEED.load(Ordering::Relaxed);
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    SEED.store(state, Ordering::Relaxed);

    let r1 = (state & 0xFFFF) as f32 / 65536.0;
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    SEED.store(state, Ordering::Relaxed);
    let r2 = (state & 0xFFFF) as f32 / 65536.0;

    r1 - r2
}

/// Decompress a zstd-compressed audio file
///
/// This function reads a .wav.zst or .flac.zst file and decompresses it,
/// returning the raw audio file bytes.
///
/// # Arguments
/// * `path` - Path to the compressed file
///
/// # Returns
/// The decompressed audio file bytes (WAV, FLAC, or AIFF format)
pub fn decompress_audio_file(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    use std::io::Read;

    let file = File::open(path.as_ref())?;
    let mut reader = std::io::BufReader::new(file);
    let mut compressed = Vec::new();
    reader.read_to_end(&mut compressed)?;

    let decompressed = zstd::decode_all(compressed.as_slice())
        .map_err(|e| Error::Compression(e.to_string()))?;

    Ok(decompressed)
}

/// Decompress and save a zstd-compressed audio file
///
/// This function reads a .wav.zst or .flac.zst file, decompresses it,
/// and writes the result to the output path.
///
/// # Arguments
/// * `input_path` - Path to the compressed file
/// * `output_path` - Path to write the decompressed file
pub fn decompress_audio_file_to(input_path: impl AsRef<Path>, output_path: impl AsRef<Path>) -> Result<()> {
    let decompressed = decompress_audio_file(input_path)?;

    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(&decompressed)?;
    writer.flush()?;

    Ok(())
}

/// Get compression statistics for a file
///
/// Returns (original_size, compressed_size, ratio)
pub fn compression_stats(compressed_path: impl AsRef<Path>) -> Result<(u64, u64, f64)> {
    let compressed_size = std::fs::metadata(compressed_path.as_ref())?.len();
    let decompressed = decompress_audio_file(compressed_path)?;
    let original_size = decompressed.len() as u64;
    let ratio = compressed_size as f64 / original_size as f64;

    Ok((original_size, compressed_size, ratio))
}

/// Decompress a dictionary-compressed audio file
///
/// This function reads a file that was compressed with a dictionary and
/// decompresses it using the same dictionary.
///
/// # Arguments
/// * `path` - Path to the compressed file
/// * `dictionary` - The dictionary used during compression
///
/// # Returns
/// The decompressed audio file bytes (WAV, FLAC, or AIFF format)
pub fn decompress_audio_file_with_dictionary(
    path: impl AsRef<Path>,
    dictionary: &CompressionDictionary,
) -> Result<Vec<u8>> {
    let file = File::open(path.as_ref())?;
    let mut reader = std::io::BufReader::new(file);
    let mut compressed = Vec::new();
    reader.read_to_end(&mut compressed)?;

    let decompressed = dictionary
        .decompress(&compressed)
        .map_err(|e| Error::Compression(e.to_string()))?;

    Ok(decompressed)
}

/// Decompress and save a dictionary-compressed audio file
///
/// This function reads a dictionary-compressed file, decompresses it,
/// and writes the result to the output path.
///
/// # Arguments
/// * `input_path` - Path to the compressed file
/// * `output_path` - Path to write the decompressed file
/// * `dictionary` - The dictionary used during compression
pub fn decompress_audio_file_with_dictionary_to(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    dictionary: &CompressionDictionary,
) -> Result<()> {
    let decompressed = decompress_audio_file_with_dictionary(input_path, dictionary)?;

    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(&decompressed)?;
    writer.flush()?;

    Ok(())
}

/// Get compression statistics for a dictionary-compressed file
///
/// Returns (original_size, compressed_size, ratio)
pub fn compression_stats_with_dictionary(
    compressed_path: impl AsRef<Path>,
    dictionary: &CompressionDictionary,
) -> Result<(u64, u64, f64)> {
    let compressed_size = std::fs::metadata(compressed_path.as_ref())?.len();
    let decompressed = decompress_audio_file_with_dictionary(compressed_path, dictionary)?;
    let original_size = decompressed.len() as u64;
    let ratio = compressed_size as f64 / original_size as f64;

    Ok((original_size, compressed_size, ratio))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_samples(freq: f32, sample_rate: u32, duration_secs: f32, channels: u16) -> Vec<f32> {
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let mut samples = Vec::with_capacity(num_samples * channels as usize);

        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            let sample = (t * freq * 2.0 * std::f32::consts::PI).sin() * 0.5;
            for _ in 0..channels {
                samples.push(sample);
            }
        }

        samples
    }

    #[test]
    fn test_export_wav16() {
        let samples = create_test_samples(440.0, 44100, 1.0, 2);
        let file = NamedTempFile::new().unwrap();

        let settings = ExportSettings {
            format: ExportFormat::Wav16,
            sample_rate: 44100,
            channels: 2,
            normalize_to_db: None,
            dither: true,
            flac_compression: FlacCompressionLevel::default(),
            zstd_compression: None,
        };

        let exporter = Exporter::new(settings);
        assert!(exporter.export(&samples, file.path()).is_ok());

        let metadata = std::fs::metadata(file.path()).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_export_wav24() {
        let samples = create_test_samples(440.0, 48000, 0.5, 2);
        let file = NamedTempFile::new().unwrap();

        let settings = ExportSettings::high_quality();
        let exporter = Exporter::new(settings);
        assert!(exporter.export(&samples, file.path()).is_ok());
    }

    #[test]
    fn test_export_wav32_float() {
        let samples = create_test_samples(880.0, 44100, 0.5, 1);
        let file = NamedTempFile::new().unwrap();

        let mut settings = ExportSettings::default();
        settings.format = ExportFormat::Wav32Float;
        settings.channels = 1;

        let exporter = Exporter::new(settings);
        assert!(exporter.export(&samples, file.path()).is_ok());
    }

    #[test]
    fn test_export_flac16() {
        let samples = create_test_samples(440.0, 44100, 0.5, 2);
        let file = NamedTempFile::new().unwrap();

        let settings = ExportSettings::flac_distribution();
        let exporter = Exporter::new(settings);

        let result = exporter.export(&samples, file.path());
        assert!(result.is_ok(), "FLAC export failed: {:?}", result);

        // Verify file was created
        let metadata = std::fs::metadata(file.path()).unwrap();
        assert!(metadata.len() > 0);

        // Verify FLAC magic
        let data = std::fs::read(file.path()).unwrap();
        assert_eq!(&data[0..4], b"fLaC", "Invalid FLAC magic");
    }

    #[test]
    fn test_export_flac24() {
        let samples = create_test_samples(440.0, 48000, 0.5, 2);
        let file = NamedTempFile::new().unwrap();

        let mut settings = ExportSettings::default();
        settings.format = ExportFormat::Flac24;
        settings.sample_rate = 48000;

        let exporter = Exporter::new(settings);
        let result = exporter.export(&samples, file.path());
        assert!(result.is_ok(), "FLAC 24-bit export failed: {:?}", result);
    }

    #[test]
    fn test_export_aiff16() {
        let samples = create_test_samples(440.0, 44100, 0.5, 2);
        let file = NamedTempFile::new().unwrap();

        let mut settings = ExportSettings::default();
        settings.format = ExportFormat::Aiff16;

        let exporter = Exporter::new(settings);
        let result = exporter.export(&samples, file.path());
        assert!(result.is_ok(), "AIFF export failed: {:?}", result);

        // Verify AIFF magic
        let data = std::fs::read(file.path()).unwrap();
        assert_eq!(&data[0..4], b"FORM");
        assert_eq!(&data[8..12], b"AIFF");
    }

    #[test]
    fn test_export_aiff24() {
        let samples = create_test_samples(440.0, 48000, 0.5, 2);
        let file = NamedTempFile::new().unwrap();

        let mut settings = ExportSettings::default();
        settings.format = ExportFormat::Aiff24;
        settings.sample_rate = 48000;

        let exporter = Exporter::new(settings);
        let result = exporter.export(&samples, file.path());
        assert!(result.is_ok(), "AIFF 24-bit export failed: {:?}", result);
    }

    #[test]
    fn test_normalize_samples() {
        let samples = vec![0.5, -0.5, 0.25, -0.25];
        let normalized = normalize_samples(&samples, 0.0);

        let peak = peak_amplitude(&normalized);
        assert!((peak - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_peak_db() {
        let samples = vec![1.0, -1.0, 0.5, -0.5];
        let db = peak_db(&samples);
        assert!((db - 0.0).abs() < 0.01);

        let quiet_samples = vec![0.1, -0.1];
        let db = peak_db(&quiet_samples);
        assert!((db - (-20.0)).abs() < 0.1);
    }

    #[test]
    fn test_rms_db() {
        let samples = create_test_samples(440.0, 44100, 1.0, 1);
        let rms = rms_db(&samples);
        assert!(rms > -12.0 && rms < -6.0);
    }

    #[test]
    fn test_db_conversion() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 0.001);
        assert!((db_to_linear(-6.0) - 0.5012).abs() < 0.01);
        assert!((db_to_linear(-20.0) - 0.1).abs() < 0.001);

        assert!((linear_to_db(1.0) - 0.0).abs() < 0.001);
        assert!((linear_to_db(0.5) - (-6.02)).abs() < 0.1);
        assert!((linear_to_db(0.1) - (-20.0)).abs() < 0.001);
    }

    #[test]
    fn test_empty_samples_error() {
        let exporter = Exporter::default();
        let file = NamedTempFile::new().unwrap();

        let result = exporter.export(&[], file.path());
        assert!(matches!(result, Err(Error::NoAudioData)));
    }

    #[test]
    fn test_export_with_normalization() {
        let samples = vec![0.25, -0.25, 0.1, -0.1];
        let file = NamedTempFile::new().unwrap();

        let mut settings = ExportSettings::default();
        settings.format = ExportFormat::Wav16;
        settings.channels = 1;
        settings.normalize_to_db = Some(-1.0);
        settings.dither = false;

        let exporter = Exporter::new(settings);
        assert!(exporter.export(&samples, file.path()).is_ok());
    }

    #[test]
    fn test_lossless_formats() {
        let formats = ExportFormat::lossless_formats();
        assert!(formats.len() >= 7);
        assert!(formats.iter().all(|f| f.is_lossless()));
    }

    // ========================================================================
    // Compressed export tests
    // ========================================================================

    #[test]
    fn test_export_wav_compressed() {
        let samples = create_test_samples(440.0, 48000, 1.0, 2);
        let file = NamedTempFile::new().unwrap();

        let settings = ExportSettings::wav_compressed();
        let exporter = Exporter::new(settings);

        let result = exporter.export(&samples, file.path());
        assert!(result.is_ok(), "Compressed WAV export failed: {:?}", result);

        // Verify file was created
        let compressed_size = std::fs::metadata(file.path()).unwrap().len();
        assert!(compressed_size > 0);

        // Decompress and verify WAV magic
        let decompressed = decompress_audio_file(file.path()).unwrap();
        assert_eq!(&decompressed[0..4], b"RIFF", "Invalid WAV magic after decompression");
        assert_eq!(&decompressed[8..12], b"WAVE");

        println!(
            "Compressed WAV: {} bytes -> {} bytes ({:.1}% of original)",
            decompressed.len(),
            compressed_size,
            (compressed_size as f64 / decompressed.len() as f64) * 100.0
        );
    }

    #[test]
    fn test_export_flac_compressed() {
        let samples = create_test_samples(440.0, 44100, 1.0, 2);
        let file = NamedTempFile::new().unwrap();

        let mut settings = ExportSettings::flac_distribution();
        settings.zstd_compression = Some(ZstdCompression::default_level());

        let exporter = Exporter::new(settings);

        let result = exporter.export(&samples, file.path());
        assert!(result.is_ok(), "Compressed FLAC export failed: {:?}", result);

        // Decompress and verify FLAC magic
        let decompressed = decompress_audio_file(file.path()).unwrap();
        assert_eq!(&decompressed[0..4], b"fLaC", "Invalid FLAC magic after decompression");
    }

    #[test]
    fn test_export_aiff_compressed() {
        let samples = create_test_samples(440.0, 44100, 0.5, 2);
        let file = NamedTempFile::new().unwrap();

        let mut settings = ExportSettings::default();
        settings.format = ExportFormat::Aiff24;
        settings.zstd_compression = Some(ZstdCompression::fast());

        let exporter = Exporter::new(settings);

        let result = exporter.export(&samples, file.path());
        assert!(result.is_ok(), "Compressed AIFF export failed: {:?}", result);

        // Decompress and verify AIFF magic
        let decompressed = decompress_audio_file(file.path()).unwrap();
        assert_eq!(&decompressed[0..4], b"FORM");
        assert_eq!(&decompressed[8..12], b"AIFF");
    }

    #[test]
    fn test_compression_ratio_wav() {
        // Generate 10 seconds of audio for meaningful compression test
        let samples = create_test_samples(440.0, 48000, 10.0, 2);

        let uncompressed_file = NamedTempFile::new().unwrap();
        let compressed_file = NamedTempFile::new().unwrap();

        // Export uncompressed
        let settings = ExportSettings::high_quality();
        let exporter = Exporter::new(settings);
        exporter.export(&samples, uncompressed_file.path()).unwrap();

        // Export compressed
        let settings = ExportSettings::wav_compressed();
        let exporter = Exporter::new(settings);
        exporter.export(&samples, compressed_file.path()).unwrap();

        let uncompressed_size = std::fs::metadata(uncompressed_file.path()).unwrap().len();
        let compressed_size = std::fs::metadata(compressed_file.path()).unwrap().len();

        println!(
            "WAV compression: {} bytes -> {} bytes ({:.1}% of original)",
            uncompressed_size,
            compressed_size,
            (compressed_size as f64 / uncompressed_size as f64) * 100.0
        );

        // WAV compression should achieve at least some reduction on real audio
        // (sine waves compress very well, expect 40-60% of original)
        assert!(
            compressed_size < uncompressed_size,
            "Compressed file should be smaller than uncompressed"
        );
    }

    #[test]
    fn test_decompress_audio_file_to() {
        let samples = create_test_samples(440.0, 44100, 0.5, 2);
        let compressed_file = NamedTempFile::new().unwrap();
        let decompressed_file = NamedTempFile::new().unwrap();

        // Export compressed
        let settings = ExportSettings::wav_compressed();
        let exporter = Exporter::new(settings);
        exporter.export(&samples, compressed_file.path()).unwrap();

        // Decompress to file
        decompress_audio_file_to(compressed_file.path(), decompressed_file.path()).unwrap();

        // Verify the decompressed file is valid WAV
        let data = std::fs::read(decompressed_file.path()).unwrap();
        assert_eq!(&data[0..4], b"RIFF");
        assert_eq!(&data[8..12], b"WAVE");
    }

    #[test]
    fn test_compression_stats() {
        // Use longer audio for meaningful compression
        let samples = create_test_samples(440.0, 44100, 5.0, 2);
        let file = NamedTempFile::new().unwrap();

        let settings = ExportSettings::wav_compressed();
        let exporter = Exporter::new(settings);
        exporter.export(&samples, file.path()).unwrap();

        let (original_size, compressed_size, ratio) = compression_stats(file.path()).unwrap();

        // Ratio should be between 0 and 2 (allowing for overhead on very small files)
        assert!(ratio > 0.0);
        assert!(ratio < 2.0);

        println!(
            "Compression stats: {} -> {} bytes (ratio: {:.2})",
            original_size, compressed_size, ratio
        );
    }

    #[test]
    fn test_settings_extension() {
        let settings = ExportSettings::high_quality();
        assert_eq!(settings.extension(), "wav");

        let settings = ExportSettings::wav_compressed();
        assert_eq!(settings.extension(), "wav.zst");

        let settings = ExportSettings::flac_distribution();
        assert_eq!(settings.extension(), "flac");

        let settings = ExportSettings::flac_distribution().with_compression();
        assert_eq!(settings.extension(), "flac.zst");
    }

    #[test]
    fn test_settings_builder_methods() {
        let settings = ExportSettings::high_quality()
            .with_compression();
        assert!(settings.is_compressed());

        let settings = settings.without_compression();
        assert!(!settings.is_compressed());

        let settings = ExportSettings::cd_quality()
            .with_compression_level(ZstdCompression::max());
        assert!(settings.is_compressed());
        assert_eq!(settings.zstd_compression.unwrap().level(), 19);
    }

    // ========================================================================
    // Dictionary compression tests
    // ========================================================================

    #[test]
    fn test_export_with_dictionary() {
        use orpheus_core::compression::DictionaryTrainer;

        // Create audio samples for training
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            let samples = create_test_samples(440.0 + i as f32 * 10.0, 44100, 2.0, 2);

            // Encode to WAV format for training
            let mut settings = ExportSettings::high_quality();
            settings.sample_rate = 44100;

            let exporter = Exporter::new(settings);
            let wav_data = exporter.encode_wav_int_to_memory(&samples, 24).unwrap();
            trainer.add_sample(&wav_data);
        }

        let dictionary = trainer.train().unwrap();

        // Export with dictionary
        let samples = create_test_samples(440.0, 44100, 1.0, 2);
        let file = NamedTempFile::new().unwrap();

        let settings = ExportSettings::high_quality();
        let exporter = Exporter::new(settings);

        let result = exporter.export_with_dictionary(&samples, file.path(), &dictionary);
        assert!(result.is_ok(), "Dictionary export failed: {:?}", result);

        // Verify we can decompress
        let decompressed = decompress_audio_file_with_dictionary(file.path(), &dictionary).unwrap();
        assert_eq!(&decompressed[0..4], b"RIFF", "Invalid WAV magic after decompression");
        assert_eq!(&decompressed[8..12], b"WAVE");
    }

    #[test]
    fn test_dictionary_compression_improvement_audio() {
        use orpheus_core::compression::DictionaryTrainer;

        // Create similar audio samples for training
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            // Similar audio with slight variations
            let samples = create_test_samples(440.0 + i as f32 * 5.0, 44100, 3.0, 2);

            let settings = ExportSettings::high_quality();
            let exporter = Exporter::new(settings);
            let wav_data = exporter.encode_wav_int_to_memory(&samples, 24).unwrap();
            trainer.add_sample(&wav_data);
        }

        let dictionary = trainer.train().unwrap();

        // Test with new similar audio
        let samples = create_test_samples(445.0, 44100, 3.0, 2);
        let standard_file = NamedTempFile::new().unwrap();
        let dict_file = NamedTempFile::new().unwrap();

        // Standard compression
        let settings = ExportSettings::wav_compressed();
        let exporter = Exporter::new(settings);
        exporter.export(&samples, standard_file.path()).unwrap();

        // Dictionary compression
        let settings = ExportSettings::high_quality();
        let exporter = Exporter::new(settings);
        exporter.export_with_dictionary(&samples, dict_file.path(), &dictionary).unwrap();

        let standard_size = std::fs::metadata(standard_file.path()).unwrap().len();
        let dict_size = std::fs::metadata(dict_file.path()).unwrap().len();

        println!(
            "Audio compression comparison:\n  Standard zstd: {} bytes\n  Dictionary: {} bytes\n  Improvement: {:.1}%",
            standard_size,
            dict_size,
            (1.0 - dict_size as f64 / standard_size as f64) * 100.0
        );

        // Both should work (dictionary may or may not be smaller depending on data)
        assert!(dict_size > 0);
        assert!(standard_size > 0);
    }

    #[test]
    fn test_decompress_with_dictionary() {
        use orpheus_core::compression::DictionaryTrainer;

        // Train dictionary
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            let samples = create_test_samples(440.0, 44100, 2.0, 2);
            let settings = ExportSettings::high_quality();
            let exporter = Exporter::new(settings);
            let wav_data = exporter.encode_wav_int_to_memory(&samples, 24).unwrap();
            trainer.add_sample(&wav_data);
        }
        let dictionary = trainer.train().unwrap();

        // Export with dictionary
        let samples = create_test_samples(440.0, 44100, 0.5, 2);
        let compressed_file = NamedTempFile::new().unwrap();
        let decompressed_file = NamedTempFile::new().unwrap();

        let settings = ExportSettings::high_quality();
        let exporter = Exporter::new(settings);
        exporter.export_with_dictionary(&samples, compressed_file.path(), &dictionary).unwrap();

        // Decompress to file
        decompress_audio_file_with_dictionary_to(
            compressed_file.path(),
            decompressed_file.path(),
            &dictionary,
        )
        .unwrap();

        // Verify decompressed file
        let data = std::fs::read(decompressed_file.path()).unwrap();
        assert_eq!(&data[0..4], b"RIFF");
        assert_eq!(&data[8..12], b"WAVE");
    }

    #[test]
    fn test_compression_stats_with_dictionary() {
        use orpheus_core::compression::DictionaryTrainer;

        // Train dictionary
        let mut trainer = DictionaryTrainer::new();
        for i in 0..10 {
            let samples = create_test_samples(440.0, 44100, 3.0, 2);
            let settings = ExportSettings::high_quality();
            let exporter = Exporter::new(settings);
            let wav_data = exporter.encode_wav_int_to_memory(&samples, 24).unwrap();
            trainer.add_sample(&wav_data);
        }
        let dictionary = trainer.train().unwrap();

        // Export with dictionary
        let samples = create_test_samples(440.0, 44100, 2.0, 2);
        let file = NamedTempFile::new().unwrap();

        let settings = ExportSettings::high_quality();
        let exporter = Exporter::new(settings);
        exporter.export_with_dictionary(&samples, file.path(), &dictionary).unwrap();

        // Get stats
        let (original_size, compressed_size, ratio) =
            compression_stats_with_dictionary(file.path(), &dictionary).unwrap();

        println!(
            "Dictionary compression stats: {} -> {} bytes (ratio: {:.2})",
            original_size, compressed_size, ratio
        );

        assert!(original_size > 0);
        assert!(compressed_size > 0);
        assert!(ratio > 0.0);
        assert!(ratio < 2.0);
    }
}

// ============================================================================
// High-level project export API
// ============================================================================

/// Export a project to an audio file
///
/// This is the main entry point for exporting projects. It handles:
/// 1. Offline rendering of all clips through the synth engine
/// 2. Exporting the rendered audio to the specified format
///
/// # Arguments
/// * `project` - The project to export
/// * `path` - Output file path
/// * `settings` - Export settings (format, sample rate, etc.)
///
/// # Example
/// ```rust,ignore
/// let project = Project::new("My Song");
/// let settings = ExportSettings::high_quality();
/// export_project(&project, "output.wav", &settings)?;
/// ```
pub fn export_project(
    project: &Project,
    path: impl AsRef<Path>,
    settings: &ExportSettings,
) -> Result<ExportResult> {
    use tracing::info;

    info!("Exporting project '{}' to {:?}", project.metadata.name, path.as_ref());

    // Create offline renderer with project's sample rate
    let mut renderer = OfflineRenderer::new(settings.sample_rate);

    // Render project to audio samples
    let samples = renderer.render_project(project);

    if samples.is_empty() {
        return Err(Error::NoAudioData);
    }

    // Create exporter with settings
    let exporter = Exporter::new(settings.clone());

    // Export to file
    exporter.export(&samples, path.as_ref())?;

    // Calculate duration
    let duration_secs = samples.len() as f64 / (settings.sample_rate as f64 * settings.channels as f64);

    info!("Export complete: {:.2} seconds of audio", duration_secs);

    Ok(ExportResult {
        path: path.as_ref().to_path_buf(),
        format: settings.format,
        sample_rate: settings.sample_rate,
        channels: settings.channels,
        duration_secs,
        file_size_bytes: std::fs::metadata(path.as_ref())
            .map(|m| m.len())
            .unwrap_or(0),
    })
}

/// Export a project with progress callback
///
/// Similar to `export_project` but calls the progress callback periodically
/// during rendering to allow UI updates.
pub fn export_project_with_progress<F>(
    project: &Project,
    path: impl AsRef<Path>,
    settings: &ExportSettings,
    on_progress: F,
) -> Result<ExportResult>
where
    F: Fn(f32) + Send + 'static,
{
    use tracing::info;

    info!("Exporting project '{}' with progress", project.metadata.name);

    // Create progress renderer
    let mut renderer = ProgressRenderer::new(settings.sample_rate);
    renderer.on_progress(on_progress);

    // Render project to audio samples
    let samples = renderer.render_project(project);

    if samples.is_empty() {
        return Err(Error::NoAudioData);
    }

    // Create exporter with settings
    let exporter = Exporter::new(settings.clone());

    // Export to file
    exporter.export(&samples, path.as_ref())?;

    // Calculate duration
    let duration_secs = samples.len() as f64 / (settings.sample_rate as f64 * settings.channels as f64);

    info!("Export complete: {:.2} seconds of audio", duration_secs);

    Ok(ExportResult {
        path: path.as_ref().to_path_buf(),
        format: settings.format,
        sample_rate: settings.sample_rate,
        channels: settings.channels,
        duration_secs,
        file_size_bytes: std::fs::metadata(path.as_ref())
            .map(|m| m.len())
            .unwrap_or(0),
    })
}

/// Result of a successful export operation
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// Path to the exported file
    pub path: std::path::PathBuf,
    /// Format used
    pub format: ExportFormat,
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Duration in seconds
    pub duration_secs: f64,
    /// File size in bytes
    pub file_size_bytes: u64,
}

impl ExportResult {
    /// Get file size in human-readable format
    pub fn file_size_human(&self) -> String {
        let bytes = self.file_size_bytes as f64;
        if bytes < 1024.0 {
            format!("{} B", bytes)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.1} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Get duration in human-readable format
    pub fn duration_human(&self) -> String {
        let total_secs = self.duration_secs as u64;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        let millis = ((self.duration_secs - total_secs as f64) * 1000.0) as u64;

        if hours > 0 {
            format!("{}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
        } else if minutes > 0 {
            format!("{}:{:02}.{:03}", minutes, seconds, millis)
        } else {
            format!("{}.{:03}s", seconds, millis)
        }
    }
}

// ============================================================================
// Tab document export API
// ============================================================================

/// Export a tab document to an audio file
///
/// This renders the tablature through the guitar synth and exports to the
/// specified format (WAV, FLAC, AIFF).
///
/// # Arguments
/// * `document` - The tab document to export
/// * `tempo` - Tempo in BPM
/// * `path` - Output file path
/// * `settings` - Export settings (format, sample rate, etc.)
///
/// # Example
/// ```rust,ignore
/// use orpheus_export::{export_tab, ExportSettings};
/// use orpheus_core::tab::TabDocument;
///
/// let document = TabDocument::default();
/// let settings = ExportSettings::cd_quality();
/// export_tab(&document, 120.0, "output.wav", &settings)?;
/// ```
pub fn export_tab(
    document: &TabDocument,
    tempo: f64,
    path: impl AsRef<Path>,
    settings: &ExportSettings,
) -> Result<ExportResult> {
    use tracing::info;

    info!("Exporting tab '{}' to {:?}", document.metadata.title, path.as_ref());

    // Create tab renderer
    let mut renderer = TabRenderer::new(settings.sample_rate);

    // Render tab to audio samples
    let samples = renderer.render(document, tempo);

    if samples.is_empty() {
        return Err(Error::NoAudioData);
    }

    // Create exporter with settings
    let exporter = Exporter::new(settings.clone());

    // Export to file
    exporter.export(&samples, path.as_ref())?;

    // Calculate duration
    let duration_secs = samples.len() as f64 / (settings.sample_rate as f64 * settings.channels as f64);

    info!("Tab export complete: {:.2} seconds of audio", duration_secs);

    Ok(ExportResult {
        path: path.as_ref().to_path_buf(),
        format: settings.format,
        sample_rate: settings.sample_rate,
        channels: settings.channels,
        duration_secs,
        file_size_bytes: std::fs::metadata(path.as_ref())
            .map(|m| m.len())
            .unwrap_or(0),
    })
}

/// Export a tab document with progress callback
///
/// Similar to `export_tab` but calls the progress callback periodically
/// during rendering to allow UI updates.
pub fn export_tab_with_progress<F>(
    document: &TabDocument,
    tempo: f64,
    path: impl AsRef<Path>,
    settings: &ExportSettings,
    on_progress: F,
) -> Result<ExportResult>
where
    F: Fn(f32) + Send + 'static,
{
    use tracing::info;

    info!("Exporting tab '{}' with progress", document.metadata.title);

    // Create progress renderer
    let mut renderer = TabProgressRenderer::new(settings.sample_rate);
    renderer.on_progress(on_progress);

    // Render tab to audio samples
    let samples = renderer.render(document, tempo);

    if samples.is_empty() {
        return Err(Error::NoAudioData);
    }

    // Create exporter with settings
    let exporter = Exporter::new(settings.clone());

    // Export to file
    exporter.export(&samples, path.as_ref())?;

    // Calculate duration
    let duration_secs = samples.len() as f64 / (settings.sample_rate as f64 * settings.channels as f64);

    info!("Tab export complete: {:.2} seconds of audio", duration_secs);

    Ok(ExportResult {
        path: path.as_ref().to_path_buf(),
        format: settings.format,
        sample_rate: settings.sample_rate,
        channels: settings.channels,
        duration_secs,
        file_size_bytes: std::fs::metadata(path.as_ref())
            .map(|m| m.len())
            .unwrap_or(0),
    })
}
