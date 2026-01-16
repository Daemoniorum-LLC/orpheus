//! Audio file handling - WAV, FLAC, AIFF loading, waveform extraction
//!
//! Supports true lossless formats for professional audio workflows.

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use hound::{WavReader, WavWriter, WavSpec, SampleFormat};
use claxon::FlacReader;
use crate::{Error, Result};

/// Supported audio formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// WAV (RIFF WAVE) - uncompressed PCM or float
    Wav,
    /// FLAC - lossless compressed
    Flac,
    /// AIFF - Audio Interchange File Format (Apple)
    Aiff,
    /// Unknown or unsupported format
    Unknown,
}

impl AudioFormat {
    /// Get format from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "wav" | "wave" => Self::Wav,
            "flac" => Self::Flac,
            "aiff" | "aif" => Self::Aiff,
            _ => Self::Unknown,
        }
    }

    /// Get format from file magic bytes
    pub fn from_magic(bytes: &[u8]) -> Self {
        if bytes.len() < 12 {
            return Self::Unknown;
        }

        // RIFF....WAVE = WAV
        if &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
            return Self::Wav;
        }

        // fLaC = FLAC
        if &bytes[0..4] == b"fLaC" {
            return Self::Flac;
        }

        // FORM....AIFF or FORM....AIFC = AIFF
        if &bytes[0..4] == b"FORM" && (&bytes[8..12] == b"AIFF" || &bytes[8..12] == b"AIFC") {
            return Self::Aiff;
        }

        Self::Unknown
    }

    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Aiff => "aiff",
            Self::Unknown => "",
        }
    }

    /// Human-readable format name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Wav => "WAV (RIFF WAVE)",
            Self::Flac => "FLAC (Free Lossless Audio Codec)",
            Self::Aiff => "AIFF (Audio Interchange File Format)",
            Self::Unknown => "Unknown",
        }
    }

    /// Is this a lossless format?
    pub fn is_lossless(&self) -> bool {
        matches!(self, Self::Wav | Self::Flac | Self::Aiff)
    }
}

/// Audio file metadata
#[derive(Debug, Clone)]
pub struct AudioMetadata {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Bits per sample
    pub bits_per_sample: u16,
    /// Total duration in samples (per channel)
    pub duration_samples: u64,
    /// Total duration in seconds
    pub duration_secs: f64,
    /// Source format
    pub format: AudioFormat,
}

/// Loaded audio file with samples
#[derive(Debug, Clone)]
pub struct AudioFile {
    /// File metadata
    pub metadata: AudioMetadata,
    /// Interleaved sample data (normalized to -1.0 to 1.0)
    pub samples: Vec<f32>,
}

impl AudioFile {
    /// Load an audio file from disk (auto-detects format)
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let format = detect_format(path)?;

        match format {
            AudioFormat::Wav => Self::load_wav(path),
            AudioFormat::Flac => Self::load_flac(path),
            AudioFormat::Aiff => Self::load_aiff(path),
            AudioFormat::Unknown => Err(Error::UnsupportedFormat(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            )),
        }
    }

    /// Load a WAV file
    pub fn load_wav(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let reader = WavReader::open(path)
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let spec = reader.spec();
        let duration_samples = reader.duration() as u64;
        let sample_rate = spec.sample_rate;
        let channels = spec.channels;

        let metadata = AudioMetadata {
            sample_rate,
            channels,
            bits_per_sample: spec.bits_per_sample,
            duration_samples,
            duration_secs: duration_samples as f64 / sample_rate as f64,
            format: AudioFormat::Wav,
        };

        // Read and normalize samples
        let samples = match spec.sample_format {
            SampleFormat::Float => {
                reader.into_samples::<f32>()
                    .map(|s| s.unwrap_or(0.0))
                    .collect()
            }
            SampleFormat::Int => {
                let max_value = (1i32 << (spec.bits_per_sample - 1)) as f32;
                reader.into_samples::<i32>()
                    .map(|s| s.unwrap_or(0) as f32 / max_value)
                    .collect()
            }
        };

        Ok(Self { metadata, samples })
    }

    /// Load a FLAC file
    pub fn load_flac(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut reader = FlacReader::open(path)
            .map_err(|e| Error::Parse(format!("FLAC parse error: {}", e)))?;

        let streaminfo = reader.streaminfo();
        let sample_rate = streaminfo.sample_rate;
        let channels = streaminfo.channels as u16;
        let bits_per_sample = streaminfo.bits_per_sample as u16;
        let duration_samples = streaminfo.samples.unwrap_or(0);

        let metadata = AudioMetadata {
            sample_rate,
            channels,
            bits_per_sample,
            duration_samples,
            duration_secs: duration_samples as f64 / sample_rate as f64,
            format: AudioFormat::Flac,
        };

        // Calculate normalization factor
        let max_value = (1i64 << (bits_per_sample - 1)) as f32;

        // Read all samples
        let mut samples = Vec::with_capacity((duration_samples as usize) * channels as usize);

        // FLAC samples are read frame by frame
        let mut frame_reader = reader.blocks();
        while let Some(block) = frame_reader.read_next_or_eof(Vec::new())
            .map_err(|e| Error::Parse(format!("FLAC decode error: {}", e)))?
        {
            let block_channels = block.channels();
            let block_len = block.len() as usize;

            // Interleave channels
            for i in 0..block_len {
                for ch in 0..block_channels {
                    let sample = block.channel(ch)[i];
                    samples.push(sample as f32 / max_value);
                }
            }
        }

        Ok(Self { metadata, samples })
    }

    /// Load an AIFF file
    pub fn load_aiff(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)
            .map_err(|e| Error::Io(e))?;
        let mut reader = BufReader::new(file);

        // Read FORM header
        let mut header = [0u8; 12];
        reader.read_exact(&mut header)
            .map_err(|e| Error::Io(e))?;

        if &header[0..4] != b"FORM" {
            return Err(Error::Parse("Not a valid AIFF file: missing FORM header".into()));
        }

        let is_aifc = &header[8..12] == b"AIFC";
        if &header[8..12] != b"AIFF" && !is_aifc {
            return Err(Error::Parse("Not a valid AIFF file: missing AIFF/AIFC marker".into()));
        }

        // Parse chunks
        let mut sample_rate = 0u32;
        let mut channels = 0u16;
        let mut bits_per_sample = 0u16;
        let mut num_frames = 0u32;
        let mut sample_data: Vec<u8> = Vec::new();
        let mut compression_type = *b"NONE";

        loop {
            let mut chunk_header = [0u8; 8];
            if reader.read_exact(&mut chunk_header).is_err() {
                break;
            }

            let chunk_id = &chunk_header[0..4];
            let chunk_size = u32::from_be_bytes([
                chunk_header[4], chunk_header[5], chunk_header[6], chunk_header[7]
            ]) as usize;

            match chunk_id {
                b"COMM" => {
                    let mut comm_data = vec![0u8; chunk_size];
                    reader.read_exact(&mut comm_data)
                        .map_err(|e| Error::Io(e))?;

                    channels = u16::from_be_bytes([comm_data[0], comm_data[1]]);
                    num_frames = u32::from_be_bytes([
                        comm_data[2], comm_data[3], comm_data[4], comm_data[5]
                    ]);
                    bits_per_sample = u16::from_be_bytes([comm_data[6], comm_data[7]]);

                    // Parse 80-bit extended precision sample rate
                    sample_rate = parse_ieee_extended(&comm_data[8..18]);

                    // AIFC has compression type
                    if is_aifc && comm_data.len() >= 22 {
                        compression_type = [comm_data[18], comm_data[19], comm_data[20], comm_data[21]];
                    }
                }
                b"SSND" => {
                    // Skip offset and block size (8 bytes)
                    let mut ssnd_header = [0u8; 8];
                    reader.read_exact(&mut ssnd_header)
                        .map_err(|e| Error::Io(e))?;

                    let data_size = chunk_size - 8;
                    sample_data = vec![0u8; data_size];
                    reader.read_exact(&mut sample_data)
                        .map_err(|e| Error::Io(e))?;
                }
                _ => {
                    // Skip unknown chunks
                    let skip_size = if chunk_size % 2 == 0 { chunk_size } else { chunk_size + 1 };
                    reader.seek(SeekFrom::Current(skip_size as i64))
                        .map_err(|e| Error::Io(e))?;
                }
            }

            // Pad byte for odd-sized chunks
            if chunk_size % 2 != 0 && chunk_id != b"SSND" {
                reader.seek(SeekFrom::Current(1))
                    .map_err(|e| Error::Io(e))?;
            }
        }

        // Only support uncompressed AIFF for now
        if compression_type != *b"NONE" && compression_type != *b"sowt" {
            return Err(Error::UnsupportedFormat(format!(
                "Compressed AIFF ({}) not supported",
                String::from_utf8_lossy(&compression_type)
            )));
        }

        let duration_samples = num_frames as u64;
        let metadata = AudioMetadata {
            sample_rate,
            channels,
            bits_per_sample,
            duration_samples,
            duration_secs: duration_samples as f64 / sample_rate as f64,
            format: AudioFormat::Aiff,
        };

        // Convert samples to f32
        let bytes_per_sample = (bits_per_sample / 8) as usize;
        let total_samples = (num_frames as usize) * (channels as usize);
        let mut samples = Vec::with_capacity(total_samples);

        let is_little_endian = compression_type == *b"sowt";

        for i in 0..total_samples {
            let offset = i * bytes_per_sample;
            if offset + bytes_per_sample > sample_data.len() {
                break;
            }

            let sample = match bits_per_sample {
                8 => {
                    // 8-bit AIFF is unsigned
                    let val = sample_data[offset] as i32 - 128;
                    val as f32 / 128.0
                }
                16 => {
                    let val = if is_little_endian {
                        i16::from_le_bytes([sample_data[offset], sample_data[offset + 1]])
                    } else {
                        i16::from_be_bytes([sample_data[offset], sample_data[offset + 1]])
                    };
                    val as f32 / 32768.0
                }
                24 => {
                    let val = if is_little_endian {
                        let bytes = [sample_data[offset], sample_data[offset + 1], sample_data[offset + 2], 0];
                        (i32::from_le_bytes(bytes) << 8) >> 8
                    } else {
                        let bytes = [0, sample_data[offset], sample_data[offset + 1], sample_data[offset + 2]];
                        (i32::from_be_bytes(bytes) << 8) >> 8
                    };
                    val as f32 / 8388608.0
                }
                32 => {
                    let val = if is_little_endian {
                        i32::from_le_bytes([
                            sample_data[offset], sample_data[offset + 1],
                            sample_data[offset + 2], sample_data[offset + 3]
                        ])
                    } else {
                        i32::from_be_bytes([
                            sample_data[offset], sample_data[offset + 1],
                            sample_data[offset + 2], sample_data[offset + 3]
                        ])
                    };
                    val as f32 / 2147483648.0
                }
                _ => 0.0,
            };

            samples.push(sample);
        }

        Ok(Self { metadata, samples })
    }

    /// Create an AudioFile from raw samples
    pub fn from_samples(
        samples: Vec<f32>,
        sample_rate: u32,
        channels: u16,
        bits_per_sample: u16,
    ) -> Self {
        let frame_count = samples.len() / channels as usize;
        Self {
            metadata: AudioMetadata {
                sample_rate,
                channels,
                bits_per_sample,
                duration_samples: frame_count as u64,
                duration_secs: frame_count as f64 / sample_rate as f64,
                format: AudioFormat::Wav, // Default to WAV
            },
            samples,
        }
    }

    /// Get the number of frames (samples per channel)
    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.metadata.channels as usize
    }

    /// Get samples for a specific channel (0-indexed)
    pub fn channel_samples(&self, channel: usize) -> Vec<f32> {
        if channel >= self.metadata.channels as usize {
            return Vec::new();
        }

        let channels = self.metadata.channels as usize;
        self.samples
            .iter()
            .skip(channel)
            .step_by(channels)
            .copied()
            .collect()
    }

    /// Mix down to mono
    pub fn to_mono(&self) -> Vec<f32> {
        if self.metadata.channels == 1 {
            return self.samples.clone();
        }

        let channels = self.metadata.channels as usize;
        let frame_count = self.frame_count();
        let mut mono = Vec::with_capacity(frame_count);

        for frame in 0..frame_count {
            let mut sum = 0.0f32;
            for ch in 0..channels {
                sum += self.samples[frame * channels + ch];
            }
            mono.push(sum / channels as f32);
        }

        mono
    }

    /// Generate waveform data for display (downsampled peaks)
    pub fn generate_waveform(&self, width: usize) -> WaveformData {
        let channels = self.metadata.channels as usize;
        let frame_count = self.frame_count();

        if frame_count == 0 || width == 0 {
            return WaveformData {
                peaks: vec![],
                sample_rate: self.metadata.sample_rate,
                duration_samples: self.metadata.duration_samples,
            };
        }

        let frames_per_pixel = (frame_count as f64 / width as f64).max(1.0);
        let mut peaks = Vec::with_capacity(width);

        for i in 0..width {
            let start_frame = (i as f64 * frames_per_pixel) as usize;
            let end_frame = ((i + 1) as f64 * frames_per_pixel) as usize;
            let end_frame = end_frame.min(frame_count);

            let mut min_val = 0.0f32;
            let mut max_val = 0.0f32;

            for frame in start_frame..end_frame {
                // Sum all channels for the peak
                let mut sum = 0.0f32;
                for ch in 0..channels {
                    let idx = frame * channels + ch;
                    if idx < self.samples.len() {
                        sum += self.samples[idx];
                    }
                }
                sum /= channels as f32;

                min_val = min_val.min(sum);
                max_val = max_val.max(sum);
            }

            peaks.push(WaveformPeak { min: min_val, max: max_val });
        }

        WaveformData {
            peaks,
            sample_rate: self.metadata.sample_rate,
            duration_samples: self.metadata.duration_samples,
        }
    }

    /// Save audio to WAV file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let spec = WavSpec {
            channels: self.metadata.channels,
            sample_rate: self.metadata.sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };

        let mut writer = WavWriter::create(path.as_ref(), spec)
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        for &sample in &self.samples {
            writer.write_sample(sample)
                .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        }

        writer.finalize()
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        Ok(())
    }
}

/// A single peak in the waveform (min and max values for a time range)
#[derive(Debug, Clone, Copy)]
pub struct WaveformPeak {
    /// Minimum sample value in this range
    pub min: f32,
    /// Maximum sample value in this range
    pub max: f32,
}

/// Waveform data for display
#[derive(Debug, Clone)]
pub struct WaveformData {
    /// Peak data for each pixel column
    pub peaks: Vec<WaveformPeak>,
    /// Sample rate of the source audio
    pub sample_rate: u32,
    /// Total duration in samples
    pub duration_samples: u64,
}

impl WaveformData {
    /// Get the peak at a given position (0.0 to 1.0)
    pub fn peak_at(&self, position: f32) -> Option<WaveformPeak> {
        if self.peaks.is_empty() {
            return None;
        }
        let idx = ((position * self.peaks.len() as f32) as usize).min(self.peaks.len() - 1);
        Some(self.peaks[idx])
    }

    /// Get waveform width
    pub fn width(&self) -> usize {
        self.peaks.len()
    }
}

/// Check if a file is a supported audio format
pub fn is_audio_file(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext.to_lowercase().as_str(), "wav" | "wave" | "flac" | "aiff" | "aif")
    } else {
        false
    }
}

/// Detect audio format from file
pub fn detect_format(path: impl AsRef<Path>) -> Result<AudioFormat> {
    let path = path.as_ref();

    // Try to read magic bytes first
    if let Ok(mut file) = File::open(path) {
        let mut magic = [0u8; 12];
        if file.read_exact(&mut magic).is_ok() {
            let format = AudioFormat::from_magic(&magic);
            if format != AudioFormat::Unknown {
                return Ok(format);
            }
        }
    }

    // Fall back to extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let format = AudioFormat::from_extension(ext);
        if format != AudioFormat::Unknown {
            return Ok(format);
        }
    }

    Err(Error::UnsupportedFormat(
        path.to_string_lossy().to_string()
    ))
}

/// Get audio file metadata without loading full samples
pub fn get_audio_metadata(path: impl AsRef<Path>) -> Result<AudioMetadata> {
    let path = path.as_ref();
    let format = detect_format(path)?;

    match format {
        AudioFormat::Wav => {
            let reader = WavReader::open(path)
                .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

            let spec = reader.spec();
            let duration_samples = reader.duration() as u64;

            Ok(AudioMetadata {
                sample_rate: spec.sample_rate,
                channels: spec.channels,
                bits_per_sample: spec.bits_per_sample,
                duration_samples,
                duration_secs: duration_samples as f64 / spec.sample_rate as f64,
                format: AudioFormat::Wav,
            })
        }
        AudioFormat::Flac => {
            let reader = FlacReader::open(path)
                .map_err(|e| Error::Parse(format!("FLAC parse error: {}", e)))?;

            let streaminfo = reader.streaminfo();
            let duration_samples = streaminfo.samples.unwrap_or(0);

            Ok(AudioMetadata {
                sample_rate: streaminfo.sample_rate,
                channels: streaminfo.channels as u16,
                bits_per_sample: streaminfo.bits_per_sample as u16,
                duration_samples,
                duration_secs: duration_samples as f64 / streaminfo.sample_rate as f64,
                format: AudioFormat::Flac,
            })
        }
        AudioFormat::Aiff => {
            // For AIFF, we need to parse the file partially
            // For now, load the full file (can be optimized later)
            let audio = AudioFile::load_aiff(path)?;
            Ok(audio.metadata)
        }
        AudioFormat::Unknown => Err(Error::UnsupportedFormat(
            path.to_string_lossy().to_string()
        )),
    }
}

/// Parse IEEE 754 80-bit extended precision to u32
/// Used for AIFF sample rate
fn parse_ieee_extended(bytes: &[u8]) -> u32 {
    if bytes.len() < 10 {
        return 0;
    }

    let sign = (bytes[0] >> 7) & 1;
    let exponent = (((bytes[0] & 0x7F) as u16) << 8) | (bytes[1] as u16);
    let mantissa = u64::from_be_bytes([
        bytes[2], bytes[3], bytes[4], bytes[5],
        bytes[6], bytes[7], bytes[8], bytes[9],
    ]);

    if exponent == 0 && mantissa == 0 {
        return 0;
    }

    // IEEE 754 extended precision has a bias of 16383
    let exp = (exponent as i32) - 16383;

    // The mantissa has an explicit integer bit
    let value = (mantissa as f64) / (1u64 << 63) as f64;
    let result = value * 2.0f64.powi(exp);

    if sign == 1 {
        -(result as i32) as u32
    } else {
        result as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_wav() -> NamedTempFile {
        let spec = WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut file = NamedTempFile::new().unwrap();
        {
            let mut writer = WavWriter::new(&mut file, spec).unwrap();
            // Write 1 second of sine wave
            for i in 0..44100 {
                let t = i as f32 / 44100.0;
                let sample = ((t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 16000.0) as i16;
                writer.write_sample(sample).unwrap(); // Left
                writer.write_sample(sample).unwrap(); // Right
            }
            writer.finalize().unwrap();
        }
        file
    }

    #[test]
    fn test_load_wav() {
        let file = create_test_wav();
        let audio = AudioFile::load(file.path()).unwrap();

        assert_eq!(audio.metadata.sample_rate, 44100);
        assert_eq!(audio.metadata.channels, 2);
        assert_eq!(audio.metadata.format, AudioFormat::Wav);
        assert_eq!(audio.frame_count(), 44100);
    }

    #[test]
    fn test_waveform_generation() {
        let file = create_test_wav();
        let audio = AudioFile::load(file.path()).unwrap();
        let waveform = audio.generate_waveform(100);

        assert_eq!(waveform.width(), 100);
        assert!(waveform.peaks.iter().all(|p| p.min >= -1.0 && p.max <= 1.0));
    }

    #[test]
    fn test_format_detection() {
        // WAV magic
        let wav_magic = b"RIFF\x00\x00\x00\x00WAVE";
        assert_eq!(AudioFormat::from_magic(wav_magic), AudioFormat::Wav);

        // FLAC magic
        let flac_magic = b"fLaC\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(AudioFormat::from_magic(flac_magic), AudioFormat::Flac);

        // AIFF magic
        let aiff_magic = b"FORM\x00\x00\x00\x00AIFF";
        assert_eq!(AudioFormat::from_magic(aiff_magic), AudioFormat::Aiff);
    }

    #[test]
    fn test_format_from_extension() {
        assert_eq!(AudioFormat::from_extension("wav"), AudioFormat::Wav);
        assert_eq!(AudioFormat::from_extension("WAV"), AudioFormat::Wav);
        assert_eq!(AudioFormat::from_extension("flac"), AudioFormat::Flac);
        assert_eq!(AudioFormat::from_extension("FLAC"), AudioFormat::Flac);
        assert_eq!(AudioFormat::from_extension("aiff"), AudioFormat::Aiff);
        assert_eq!(AudioFormat::from_extension("aif"), AudioFormat::Aiff);
        assert_eq!(AudioFormat::from_extension("mp3"), AudioFormat::Unknown);
    }

    #[test]
    fn test_to_mono() {
        let file = create_test_wav();
        let audio = AudioFile::load(file.path()).unwrap();
        let mono = audio.to_mono();

        assert_eq!(mono.len(), audio.frame_count());
    }

    #[test]
    fn test_channel_samples() {
        let file = create_test_wav();
        let audio = AudioFile::load(file.path()).unwrap();

        let left = audio.channel_samples(0);
        let right = audio.channel_samples(1);

        assert_eq!(left.len(), audio.frame_count());
        assert_eq!(right.len(), audio.frame_count());

        // In our test file, left and right are the same
        for (l, r) in left.iter().zip(right.iter()) {
            assert!((l - r).abs() < 0.0001);
        }
    }
}
