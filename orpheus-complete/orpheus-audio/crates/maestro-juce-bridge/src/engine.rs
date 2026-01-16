/*!
 * Safe Rust wrapper for the JUCE audio engine
 */

use crate::error::{AudioEngineError, Result};
use crate::ffi;
use std::ffi::CString;
use std::ptr;

/// Safe wrapper for the JUCE audio engine
pub struct AudioEngine {
    engine: *mut libc::c_void,
    sample_rate: i32,
    buffer_size: i32,
}

impl AudioEngine {
    /// Create a new audio engine instance
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz (e.g. 48000)
    /// * `buffer_size` - Buffer size in samples (e.g. 512)
    ///
    /// # Errors
    /// Returns `AudioEngineError::CreationFailed` if the engine cannot be created
    pub fn new(sample_rate: i32, buffer_size: i32) -> Result<Self> {
        let engine = unsafe { ffi::maestro_audio_engine_new(sample_rate, buffer_size) };

        if engine.is_null() {
            return Err(AudioEngineError::CreationFailed);
        }

        Ok(Self {
            engine,
            sample_rate,
            buffer_size,
        })
    }

    /// Process an audio buffer through the effects chain
    ///
    /// # Arguments
    /// * `input` - Input audio samples (interleaved)
    /// * `output` - Output buffer (must be same size as input)
    /// * `num_channels` - Number of audio channels (1 or 2)
    ///
    /// # Safety
    /// The input and output buffers must be valid and have the same length
    pub fn process(&mut self, input: &[f32], output: &mut [f32], num_channels: i32) -> Result<()> {
        if input.len() != output.len() {
            return Err(AudioEngineError::InvalidBuffer(
                "Input and output buffers must have the same length".to_string(),
            ));
        }

        if num_channels < 1 || num_channels > 2 {
            return Err(AudioEngineError::InvalidParameter(
                "num_channels must be 1 or 2".to_string(),
            ));
        }

        let num_samples = (input.len() / num_channels as usize) as i32;

        let input_ffi = ffi::AudioBufferFFI {
            data: input.as_ptr() as *mut f32,
            num_channels,
            num_samples,
            sample_rate: self.sample_rate,
        };

        let mut output_ffi = ffi::AudioBufferFFI {
            data: output.as_mut_ptr(),
            num_channels,
            num_samples,
            sample_rate: self.sample_rate,
        };

        unsafe {
            ffi::maestro_audio_engine_process(self.engine, &input_ffi, &mut output_ffi);
        }

        Ok(())
    }

    /// Set parameters for an EQ band
    ///
    /// # Arguments
    /// * `band` - Band index (0-3)
    /// * `params` - EQ band parameters
    pub fn set_eq_band(&mut self, band: usize, params: ffi::EQBand) -> Result<()> {
        if band >= 4 {
            return Err(AudioEngineError::InvalidParameter(
                "EQ band index must be 0-3".to_string(),
            ));
        }

        unsafe {
            ffi::maestro_audio_engine_set_eq_band(
                self.engine,
                band as i32,
                params.frequency,
                params.gain,
                params.q,
            );
        }

        Ok(())
    }

    /// Enable or disable the EQ
    pub fn set_eq_enabled(&mut self, enabled: bool) {
        unsafe {
            ffi::maestro_audio_engine_set_eq_enabled(self.engine, enabled);
        }
    }

    /// Set compressor parameters
    pub fn set_compressor(&mut self, params: ffi::CompressorParams) {
        unsafe {
            ffi::maestro_audio_engine_set_compressor(
                self.engine,
                params.threshold,
                params.ratio,
                params.attack_ms,
                params.release_ms,
                params.knee,
                params.makeup_gain,
            );
        }
    }

    /// Enable or disable the compressor
    pub fn set_compressor_enabled(&mut self, enabled: bool) {
        unsafe {
            ffi::maestro_audio_engine_set_compressor_enabled(self.engine, enabled);
        }
    }

    /// Set reverb parameters
    pub fn set_reverb(&mut self, params: ffi::ReverbParams) {
        unsafe {
            ffi::maestro_audio_engine_set_reverb(
                self.engine,
                params.room_size,
                params.decay,
                params.pre_delay,
                params.wet_dry,
            );
        }
    }

    /// Enable or disable reverb
    pub fn set_reverb_enabled(&mut self, enabled: bool) {
        unsafe {
            ffi::maestro_audio_engine_set_reverb_enabled(self.engine, enabled);
        }
    }

    /// Set delay parameters
    pub fn set_delay(&mut self, params: ffi::DelayParams) {
        unsafe {
            ffi::maestro_audio_engine_set_delay(
                self.engine,
                params.time_ms,
                params.feedback,
                params.wet_dry,
            );
        }
    }

    /// Enable or disable delay
    pub fn set_delay_enabled(&mut self, enabled: bool) {
        unsafe {
            ffi::maestro_audio_engine_set_delay_enabled(self.engine, enabled);
        }
    }

    /// Get the current latency in samples
    pub fn get_latency_samples(&self) -> i32 {
        unsafe { ffi::maestro_audio_engine_get_latency(self.engine) }
    }

    /// Get the current latency in milliseconds
    pub fn get_latency_ms(&self) -> f32 {
        let samples = self.get_latency_samples();
        (samples as f32 / self.sample_rate as f32) * 1000.0
    }

    /// Load a VST3 plugin
    ///
    /// # Arguments
    /// * `plugin_path` - Path to the VST3 plugin file
    ///
    /// # Errors
    /// Returns `AudioEngineError::PluginLoadFailed` if the plugin cannot be loaded
    pub fn load_plugin(&mut self, plugin_path: &str) -> Result<()> {
        let c_path = CString::new(plugin_path)
            .map_err(|_| AudioEngineError::PluginLoadFailed("Invalid path".to_string()))?;

        let success = unsafe { ffi::maestro_audio_engine_load_plugin(self.engine, c_path.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(AudioEngineError::PluginLoadFailed(
                plugin_path.to_string(),
            ))
        }
    }

    /// Reset all processing state (clear buffers, reset delays, etc.)
    pub fn reset(&mut self) {
        unsafe {
            ffi::maestro_audio_engine_reset(self.engine);
        }
    }

    /// Get the sample rate
    pub fn sample_rate(&self) -> i32 {
        self.sample_rate
    }

    /// Get the buffer size
    pub fn buffer_size(&self) -> i32 {
        self.buffer_size
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        if !self.engine.is_null() {
            unsafe {
                ffi::maestro_audio_engine_delete(self.engine);
            }
            self.engine = ptr::null_mut();
        }
    }
}

// Safety: The C++ engine is thread-safe
unsafe impl Send for AudioEngine {}
unsafe impl Sync for AudioEngine {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = AudioEngine::new(48000, 512);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_engine_process() {
        let mut engine = AudioEngine::new(48000, 512).unwrap();

        let input = vec![0.5f32; 1024]; // 512 samples * 2 channels
        let mut output = vec![0.0f32; 1024];

        let result = engine.process(&input, &mut output, 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eq_band() {
        let mut engine = AudioEngine::new(48000, 512).unwrap();

        let params = ffi::EQBand::new(1000.0, 3.0, 1.0);
        let result = engine.set_eq_band(0, params);
        assert!(result.is_ok());

        // Invalid band index
        let result = engine.set_eq_band(4, params);
        assert!(result.is_err());
    }

    #[test]
    fn test_latency() {
        let engine = AudioEngine::new(48000, 512).unwrap();

        let latency_samples = engine.get_latency_samples();
        let latency_ms = engine.get_latency_ms();

        assert!(latency_samples > 0);
        assert!(latency_ms > 0.0);
    }
}
