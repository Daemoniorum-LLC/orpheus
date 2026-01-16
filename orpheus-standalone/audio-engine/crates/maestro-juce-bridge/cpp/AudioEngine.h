/**
 * Maestro Audio Engine - JUCE C++ Implementation
 * High-performance audio processing with DSP effects chain
 */

#pragma once

#include <memory>
#include <vector>
#include <string>
#include <cstdint>

namespace maestro {

// Forward declarations for JUCE types (will be included in .cpp)
// This keeps the header clean and reduces compile dependencies

/**
 * Audio buffer for FFI transport
 */
struct AudioBufferFFI {
    float* data;           // Interleaved audio samples (L, R, L, R...)
    int32_t num_channels;  // Number of channels (1 or 2)
    int32_t num_samples;   // Samples per channel
    int32_t sample_rate;   // Sample rate in Hz
};

/**
 * EQ band parameters
 */
struct EQBand {
    float frequency;  // Hz
    float gain;       // dB (-12 to +12)
    float q;          // Quality factor (0.1 to 10.0)
};

/**
 * Compressor parameters
 */
struct CompressorParams {
    float threshold;   // dB (-60 to 0)
    float ratio;       // 1:1 to 20:1
    float attack_ms;   // Attack time in milliseconds
    float release_ms;  // Release time in milliseconds
    float knee;        // Knee width in dB
    float makeup_gain; // Makeup gain in dB
};

/**
 * Reverb parameters
 */
struct ReverbParams {
    float room_size;   // 0.0 to 1.0
    float decay;       // Decay time in seconds
    float pre_delay;   // Pre-delay in milliseconds
    float wet_dry;     // Wet/dry mix (0.0 to 1.0)
};

/**
 * Delay parameters
 */
struct DelayParams {
    float time_ms;     // Delay time in milliseconds
    float feedback;    // Feedback amount (0.0 to 1.0)
    float wet_dry;     // Wet/dry mix (0.0 to 1.0)
};

/**
 * Main audio engine class
 * Manages the complete DSP chain: EQ → Compressor → Reverb → Delay → Limiter
 */
class AudioEngine {
public:
    /**
     * Constructor
     * @param sample_rate Sample rate in Hz (typically 48000)
     * @param buffer_size Buffer size in samples (typically 256-512)
     */
    AudioEngine(int sample_rate, int buffer_size);

    /**
     * Destructor - cleans up all DSP resources
     */
    ~AudioEngine();

    /**
     * Process an audio buffer through the entire effects chain
     * @param input Input audio buffer (interleaved)
     * @param output Output audio buffer (interleaved, pre-allocated)
     */
    void process(const AudioBufferFFI* input, AudioBufferFFI* output);

    /**
     * Set EQ band parameters
     * @param band Band index (0-3)
     * @param params EQ band parameters
     */
    void setEQBand(int band, const EQBand& params);

    /**
     * Enable/disable EQ
     */
    void setEQEnabled(bool enabled);

    /**
     * Set compressor parameters
     */
    void setCompressor(const CompressorParams& params);

    /**
     * Enable/disable compressor
     */
    void setCompressorEnabled(bool enabled);

    /**
     * Set reverb parameters
     */
    void setReverb(const ReverbParams& params);

    /**
     * Enable/disable reverb
     */
    void setReverbEnabled(bool enabled);

    /**
     * Set delay parameters
     */
    void setDelay(const DelayParams& params);

    /**
     * Enable/disable delay
     */
    void setDelayEnabled(bool enabled);

    /**
     * Get current latency in samples
     */
    int getLatencySamples() const;

    /**
     * Load a VST3 plugin
     * @param plugin_path Path to VST3 plugin file
     * @return true if loaded successfully
     */
    bool loadPlugin(const char* plugin_path);

    /**
     * Reset all processing state (clear buffers, reset delays, etc.)
     */
    void reset();

private:
    class Impl;  // PIMPL idiom to hide JUCE dependencies
    std::unique_ptr<Impl> impl_;
};

} // namespace maestro

//=============================================================================
// C FFI Exports for Rust
//=============================================================================

extern "C" {
    /**
     * Create a new audio engine instance
     * @return Opaque pointer to AudioEngine
     */
    void* maestro_audio_engine_new(int32_t sample_rate, int32_t buffer_size);

    /**
     * Destroy an audio engine instance
     */
    void maestro_audio_engine_delete(void* engine);

    /**
     * Process audio buffer
     */
    void maestro_audio_engine_process(
        void* engine,
        const maestro::AudioBufferFFI* input,
        maestro::AudioBufferFFI* output
    );

    /**
     * Set EQ band parameters
     */
    void maestro_audio_engine_set_eq_band(
        void* engine,
        int32_t band,
        float frequency,
        float gain,
        float q
    );

    /**
     * Enable/disable EQ
     */
    void maestro_audio_engine_set_eq_enabled(void* engine, bool enabled);

    /**
     * Set compressor parameters
     */
    void maestro_audio_engine_set_compressor(
        void* engine,
        float threshold,
        float ratio,
        float attack_ms,
        float release_ms,
        float knee,
        float makeup_gain
    );

    /**
     * Enable/disable compressor
     */
    void maestro_audio_engine_set_compressor_enabled(void* engine, bool enabled);

    /**
     * Set reverb parameters
     */
    void maestro_audio_engine_set_reverb(
        void* engine,
        float room_size,
        float decay,
        float pre_delay,
        float wet_dry
    );

    /**
     * Enable/disable reverb
     */
    void maestro_audio_engine_set_reverb_enabled(void* engine, bool enabled);

    /**
     * Set delay parameters
     */
    void maestro_audio_engine_set_delay(
        void* engine,
        float time_ms,
        float feedback,
        float wet_dry
    );

    /**
     * Enable/disable delay
     */
    void maestro_audio_engine_set_delay_enabled(void* engine, bool enabled);

    /**
     * Get latency in samples
     */
    int32_t maestro_audio_engine_get_latency(void* engine);

    /**
     * Load VST plugin
     */
    bool maestro_audio_engine_load_plugin(void* engine, const char* plugin_path);

    /**
     * Reset processing state
     */
    void maestro_audio_engine_reset(void* engine);
}
