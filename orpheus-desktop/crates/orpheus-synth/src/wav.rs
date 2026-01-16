//! WAV file writer for audio export
//!
//! Simple WAV file generation without external dependencies.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// WAV file writer
pub struct WavWriter {
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
}

impl WavWriter {
    /// Create a new WAV writer
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
            bits_per_sample: 16, // 16-bit audio
        }
    }

    /// Create stereo WAV writer at 44100 Hz
    pub fn stereo_44100() -> Self {
        Self::new(44100, 2)
    }

    /// Create mono WAV writer at 44100 Hz
    pub fn mono_44100() -> Self {
        Self::new(44100, 1)
    }

    /// Write f32 samples to a WAV file
    /// For stereo, samples should be interleaved [L, R, L, R, ...]
    pub fn write_f32<P: AsRef<Path>>(&self, path: P, samples: &[f32]) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Calculate sizes
        let data_size = samples.len() as u32 * 2; // 16-bit = 2 bytes per sample
        let file_size = 36 + data_size;

        // Write RIFF header
        writer.write_all(b"RIFF")?;
        writer.write_all(&file_size.to_le_bytes())?;
        writer.write_all(b"WAVE")?;

        // Write fmt chunk
        writer.write_all(b"fmt ")?;
        writer.write_all(&16u32.to_le_bytes())?; // Chunk size
        writer.write_all(&1u16.to_le_bytes())?; // Audio format (1 = PCM)
        writer.write_all(&self.channels.to_le_bytes())?;
        writer.write_all(&self.sample_rate.to_le_bytes())?;

        let byte_rate = self.sample_rate * self.channels as u32 * self.bits_per_sample as u32 / 8;
        writer.write_all(&byte_rate.to_le_bytes())?;

        let block_align = self.channels * self.bits_per_sample / 8;
        writer.write_all(&block_align.to_le_bytes())?;
        writer.write_all(&self.bits_per_sample.to_le_bytes())?;

        // Write data chunk
        writer.write_all(b"data")?;
        writer.write_all(&data_size.to_le_bytes())?;

        // Convert f32 samples to i16 and write
        for sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let i16_sample = (clamped * 32767.0) as i16;
            writer.write_all(&i16_sample.to_le_bytes())?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Write stereo f32 samples from separate left/right channels
    pub fn write_stereo<P: AsRef<Path>>(
        &self,
        path: P,
        left: &[f32],
        right: &[f32],
    ) -> std::io::Result<()> {
        assert_eq!(left.len(), right.len(), "Left and right channels must have same length");

        // Interleave samples
        let mut interleaved = Vec::with_capacity(left.len() * 2);
        for (l, r) in left.iter().zip(right.iter()) {
            interleaved.push(*l);
            interleaved.push(*r);
        }

        self.write_f32(path, &interleaved)
    }
}

/// Quick helper to write stereo audio to a WAV file
pub fn write_wav<P: AsRef<Path>>(
    path: P,
    left: &[f32],
    right: &[f32],
    sample_rate: u32,
) -> std::io::Result<()> {
    WavWriter::new(sample_rate, 2).write_stereo(path, left, right)
}

/// Quick helper to write mono audio to a WAV file
pub fn write_wav_mono<P: AsRef<Path>>(
    path: P,
    samples: &[f32],
    sample_rate: u32,
) -> std::io::Result<()> {
    WavWriter::new(sample_rate, 1).write_f32(path, samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_write_sine_wave() {
        let sample_rate = 44100;
        let duration_secs = 1.0;
        let frequency = 440.0; // A4

        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * PI * frequency * t).sin() * 0.5
            })
            .collect();

        let path = "/tmp/test_sine.wav";
        write_wav_mono(path, &samples, sample_rate).unwrap();

        // Verify file was created
        assert!(std::path::Path::new(path).exists());

        // Clean up
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_write_stereo() {
        let sample_rate = 44100;
        let num_samples = 4410; // 0.1 seconds

        // Left channel: 440 Hz, Right channel: 880 Hz
        let left: Vec<f32> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * PI * 440.0 * t).sin() * 0.5
            })
            .collect();

        let right: Vec<f32> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * PI * 880.0 * t).sin() * 0.5
            })
            .collect();

        let path = "/tmp/test_stereo.wav";
        write_wav(path, &left, &right, sample_rate).unwrap();

        assert!(std::path::Path::new(path).exists());
        std::fs::remove_file(path).ok();
    }
}
