//! Real-time audio output using cpal

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig, SampleFormat, SupportedStreamConfig};
use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;

use crate::Error;

/// Audio output message
pub enum AudioMessage {
    /// Audio samples to play (interleaved stereo)
    Samples(Vec<f32>),
    /// Stop playback
    Stop,
    /// Set volume (0.0 - 1.0)
    SetVolume(f32),
}

/// Audio output state
struct OutputState {
    /// Ring buffer for audio samples
    buffer: Vec<f32>,
    /// Read position
    read_pos: usize,
    /// Write position
    write_pos: usize,
    /// Master volume
    volume: f32,
    /// Is playing
    playing: bool,
}

impl OutputState {
    fn new(buffer_size: usize) -> Self {
        Self {
            buffer: vec![0.0; buffer_size],
            read_pos: 0,
            write_pos: 0,
            volume: 0.8,
            playing: false,
        }
    }

    fn write(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.buffer.len();
        }
    }

    fn read(&mut self) -> f32 {
        let sample = self.buffer[self.read_pos] * self.volume;
        self.buffer[self.read_pos] = 0.0; // Clear after reading
        self.read_pos = (self.read_pos + 1) % self.buffer.len();
        sample
    }

    fn available(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.write_pos - self.read_pos
        } else {
            self.buffer.len() - self.read_pos + self.write_pos
        }
    }
}

/// Real-time audio output handle
pub struct AudioOutput {
    /// Sender for audio messages
    sender: Sender<AudioMessage>,
    /// Audio stream (kept alive)
    _stream: Stream,
    /// Sample rate
    sample_rate: u32,
    /// Number of channels
    channels: u16,
}

impl AudioOutput {
    /// Create a new audio output using the default device
    pub fn new() -> Result<Self, Error> {
        Self::with_buffer_size(8192)
    }

    /// Create with custom buffer size
    pub fn with_buffer_size(buffer_size: usize) -> Result<Self, Error> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| Error::AudioOutput("No output device found".into()))?;

        Self::with_device(&device, buffer_size)
    }

    /// Create with specific device
    pub fn with_device(device: &Device, buffer_size: usize) -> Result<Self, Error> {
        let supported_config = device.default_output_config()
            .map_err(|e| Error::AudioOutput(format!("Failed to get config: {}", e)))?;

        let sample_rate = supported_config.sample_rate().0;
        let channels = supported_config.channels();
        let sample_format = supported_config.sample_format();

        let config: StreamConfig = supported_config.into();

        // Create shared state
        let state = Arc::new(Mutex::new(OutputState::new(buffer_size)));
        let state_clone = Arc::clone(&state);

        // Create message channel
        let (sender, receiver) = bounded::<AudioMessage>(64);

        // Spawn message handler thread
        let state_handler = Arc::clone(&state);
        thread::spawn(move || {
            while let Ok(msg) = receiver.recv() {
                let mut state = state_handler.lock();
                match msg {
                    AudioMessage::Samples(samples) => {
                        state.write(&samples);
                        state.playing = true;
                    }
                    AudioMessage::Stop => {
                        state.playing = false;
                    }
                    AudioMessage::SetVolume(vol) => {
                        state.volume = vol.clamp(0.0, 1.0);
                    }
                }
            }
        });

        // Create audio stream
        let stream = match sample_format {
            SampleFormat::F32 => {
                device.build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let mut state = state_clone.lock();
                        for sample in data.iter_mut() {
                            *sample = if state.playing && state.available() > 0 {
                                state.read()
                            } else {
                                0.0
                            };
                        }
                    },
                    |err| eprintln!("Audio stream error: {}", err),
                    None,
                )
            }
            SampleFormat::I16 => {
                device.build_output_stream(
                    &config,
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        let mut state = state_clone.lock();
                        for sample in data.iter_mut() {
                            let f = if state.playing && state.available() > 0 {
                                state.read()
                            } else {
                                0.0
                            };
                            *sample = (f * i16::MAX as f32) as i16;
                        }
                    },
                    |err| eprintln!("Audio stream error: {}", err),
                    None,
                )
            }
            SampleFormat::U16 => {
                device.build_output_stream(
                    &config,
                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        let mut state = state_clone.lock();
                        for sample in data.iter_mut() {
                            let f = if state.playing && state.available() > 0 {
                                state.read()
                            } else {
                                0.0
                            };
                            *sample = ((f + 1.0) * 0.5 * u16::MAX as f32) as u16;
                        }
                    },
                    |err| eprintln!("Audio stream error: {}", err),
                    None,
                )
            }
            _ => {
                return Err(Error::AudioOutput(format!("Unsupported sample format: {:?}", sample_format)));
            }
        }.map_err(|e| Error::AudioOutput(format!("Failed to build stream: {}", e)))?;

        // Start the stream
        stream.play().map_err(|e| Error::AudioOutput(format!("Failed to start stream: {}", e)))?;

        Ok(Self {
            sender,
            _stream: stream,
            sample_rate,
            channels,
        })
    }

    /// Get the sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the number of channels
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Send audio samples to output
    pub fn send_samples(&self, samples: Vec<f32>) -> Result<(), Error> {
        self.sender.send(AudioMessage::Samples(samples))
            .map_err(|_| Error::AudioOutput("Failed to send samples".into()))
    }

    /// Stop playback
    pub fn stop(&self) -> Result<(), Error> {
        self.sender.send(AudioMessage::Stop)
            .map_err(|_| Error::AudioOutput("Failed to stop".into()))
    }

    /// Set volume
    pub fn set_volume(&self, volume: f32) -> Result<(), Error> {
        self.sender.send(AudioMessage::SetVolume(volume))
            .map_err(|_| Error::AudioOutput("Failed to set volume".into()))
    }
}

/// List available audio output devices
pub fn list_output_devices() -> Result<Vec<String>, Error> {
    let host = cpal::default_host();
    let devices = host.output_devices()
        .map_err(|e| Error::AudioOutput(format!("Failed to enumerate devices: {}", e)))?;

    let names: Vec<String> = devices
        .filter_map(|d| d.name().ok())
        .collect();

    Ok(names)
}

/// Get default output device name
pub fn default_output_device_name() -> Option<String> {
    let host = cpal::default_host();
    host.default_output_device()
        .and_then(|d| d.name().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices() {
        // This test may fail on CI without audio devices
        let result = list_output_devices();
        // Just check it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_output_state() {
        let mut state = OutputState::new(1024);
        assert_eq!(state.available(), 0);

        // Write some samples
        state.write(&[0.5, -0.5, 0.3, -0.3]);
        assert_eq!(state.available(), 4);

        // Read back
        assert!((state.read() - 0.4).abs() < 0.01); // 0.5 * 0.8 volume
        assert_eq!(state.available(), 3);
    }
}
