/// Zero-copy audio buffer management

use crate::format::{SampleRate, ChannelCount};

/// Audio buffer with interleaved samples
pub struct AudioBuffer {
    /// Sample data (interleaved f32)
    data: Vec<f32>,
    /// Number of channels
    channels: ChannelCount,
    /// Sample rate
    sample_rate: SampleRate,
}

impl AudioBuffer {
    /// Create a new audio buffer
    pub fn new(channels: ChannelCount, sample_rate: SampleRate, num_samples: usize) -> Self {
        let capacity = channels.count() * num_samples;
        Self {
            data: vec![0.0; capacity],
            channels,
            sample_rate,
        }
    }

    /// Create from existing data
    pub fn from_vec(data: Vec<f32>, channels: ChannelCount, sample_rate: SampleRate) -> Self {
        Self {
            data,
            channels,
            sample_rate,
        }
    }

    /// Get mutable slice of audio data
    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Get immutable slice of audio data
    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    /// Get number of samples per channel
    pub fn num_samples(&self) -> usize {
        self.data.len() / self.channels.count()
    }

    /// Get number of channels
    pub fn channels(&self) -> ChannelCount {
        self.channels
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    /// Convert to bytes (for proto)
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.iter()
            .flat_map(|&sample| sample.to_le_bytes())
            .collect()
    }

    /// Create from bytes (from proto)
    pub fn from_bytes(
        bytes: &[u8],
        channels: ChannelCount,
        sample_rate: SampleRate
    ) -> Self {
        let data: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        Self {
            data,
            channels,
            sample_rate,
        }
    }

    /// Clear buffer (fill with zeros)
    pub fn clear(&mut self) {
        self.data.fill(0.0);
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> f32 {
        let samples_per_channel = self.num_samples();
        (samples_per_channel as f32 / self.sample_rate.hz() as f32) * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = AudioBuffer::new(ChannelCount::Stereo, SampleRate::Hz48000, 1024);
        assert_eq!(buffer.num_samples(), 1024);
        assert_eq!(buffer.as_slice().len(), 2048); // 2 channels * 1024 samples
    }

    #[test]
    fn test_buffer_bytes() {
        let mut buffer = AudioBuffer::new(ChannelCount::Stereo, SampleRate::Hz48000, 10);
        buffer.as_mut_slice()[0] = 0.5;
        buffer.as_mut_slice()[1] = -0.5;

        let bytes = buffer.to_bytes();
        let restored = AudioBuffer::from_bytes(&bytes, ChannelCount::Stereo, SampleRate::Hz48000);

        assert_eq!(restored.as_slice()[0], 0.5);
        assert_eq!(restored.as_slice()[1], -0.5);
    }

    #[test]
    fn test_buffer_creation_mono() {
        let buffer = AudioBuffer::new(ChannelCount::Mono, SampleRate::Hz48000, 1024);
        assert_eq!(buffer.num_samples(), 1024);
        assert_eq!(buffer.as_slice().len(), 1024);
        assert_eq!(buffer.channels(), ChannelCount::Mono);
    }

    #[test]
    fn test_buffer_from_vec() {
        let data = vec![0.5, -0.5, 0.25, -0.25];
        let buffer = AudioBuffer::from_vec(data.clone(), ChannelCount::Stereo, SampleRate::Hz48000);
        assert_eq!(buffer.as_slice(), &data[..]);
        assert_eq!(buffer.num_samples(), 2);
    }

    #[test]
    fn test_buffer_clear() {
        let mut buffer = AudioBuffer::new(ChannelCount::Stereo, SampleRate::Hz48000, 10);
        for sample in buffer.as_mut_slice() {
            *sample = 1.0;
        }
        buffer.clear();
        assert!(buffer.as_slice().iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_buffer_duration_ms() {
        let buffer = AudioBuffer::new(ChannelCount::Stereo, SampleRate::Hz48000, 4800);
        let duration = buffer.duration_ms();
        assert!((duration - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_buffer_empty() {
        let buffer = AudioBuffer::new(ChannelCount::Mono, SampleRate::Hz48000, 0);
        assert_eq!(buffer.num_samples(), 0);
        assert_eq!(buffer.duration_ms(), 0.0);
    }

    #[test]
    fn test_buffer_all_sample_rates() {
        for rate in [SampleRate::Hz44100, SampleRate::Hz48000, SampleRate::Hz88200, SampleRate::Hz96000] {
            let buffer = AudioBuffer::new(ChannelCount::Stereo, rate, 100);
            assert_eq!(buffer.sample_rate(), rate);
        }
    }

    #[test]
    fn test_buffer_bytes_size() {
        let buffer = AudioBuffer::new(ChannelCount::Stereo, SampleRate::Hz48000, 100);
        let bytes = buffer.to_bytes();
        assert_eq!(bytes.len(), 800); // 100 samples * 2 channels * 4 bytes
    }

    #[test]
    fn test_buffer_large() {
        let buffer = AudioBuffer::new(ChannelCount::Stereo, SampleRate::Hz96000, 96000);
        assert_eq!(buffer.num_samples(), 96000);
        assert_eq!(buffer.as_slice().len(), 192000);
        assert!((buffer.duration_ms() - 1000.0).abs() < 0.1);
    }
}
