# Orpheus - Audio Engine Specification

**Version:** 1.0
**Date:** November 16, 2025
**Technology:** JUCE C++ Framework
**Status:** Specification

## Overview

The Orpheus audio engine is built on JUCE (Jules' Utility Class Extensions), a cross-platform C++ framework for audio applications. This engine handles all low-latency audio I/O, plugin hosting, real-time processing, and mixing operations.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│               Frontend (TypeScript/React)                    │
└────────────────────┬────────────────────────────────────────┘
                     │ IPC/WebSocket
┌────────────────────▼────────────────────────────────────────┐
│            Audio Engine Bridge (Node.js/Native)              │
│  - Converts JSON commands to C++ calls                       │
│  - Streams audio data                                        │
│  - Reports status and meters                                 │
└────────────────────┬────────────────────────────────────────┘
                     │ Native FFI
┌────────────────────▼────────────────────────────────────────┐
│              Audio Engine Core (JUCE C++)                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Audio Device Manager                        │  │
│  │  ASIO, Core Audio, ALSA, WASAPI drivers              │  │
│  └───────────────────┬──────────────────────────────────┘  │
│                      ▼                                       │
│  ┌──────────────────────────────────────────────────────┐  │
│  │             Audio Graph                               │  │
│  │  - Track Processors                                   │  │
│  │  - Plugin Hosts (VST3, AU, AAX)                       │  │
│  │  - Bus Processors                                     │  │
│  │  - Master Output                                      │  │
│  └───────────────────┬──────────────────────────────────┘  │
│                      ▼                                       │
│  ┌──────────────────────────────────────────────────────┐  │
│  │        Real-Time Audio Processing                     │  │
│  │  - Sample-accurate mixing                             │  │
│  │  - Automation processing                              │  │
│  │  - Metering and analysis                              │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Audio Device Manager

**Responsibilities:**
- Enumerate available audio devices
- Initialize audio I/O with specified configuration
- Handle buffer size and sample rate changes
- Manage device hot-plugging

**Interface:**
```cpp
class MaestroAudioDeviceManager {
public:
  // Device enumeration
  std::vector<AudioDevice> getAvailableInputDevices();
  std::vector<AudioDevice> getAvailableOutputDevices();

  // Configuration
  bool initialize(const AudioConfig& config);
  bool setBufferSize(int samples);
  bool setSampleRate(double sampleRate);

  // Status
  AudioConfig getCurrentConfig();
  double getCurrentLatency(); // Round-trip in milliseconds
};

struct AudioDevice {
  std::string id;
  std::string name;
  std::string driver; // ASIO, Core Audio, etc.
  std::vector<int> supportedSampleRates;
  std::vector<int> supportedBufferSizes;
  int maxInputChannels;
  int maxOutputChannels;
};

struct AudioConfig {
  std::string inputDeviceId;
  std::string outputDeviceId;
  int sampleRate;      // 44100, 48000, 96000, etc.
  int bufferSize;      // 64, 128, 256, 512, 1024, etc.
  int inputChannels;
  int outputChannels;
};
```

### 2. Audio Track

**Responsibilities:**
- Audio and MIDI recording
- Playback of audio regions
- Plugin chain processing
- Automation handling
- Real-time metering

**Interface:**
```cpp
class AudioTrack {
public:
  // Identification
  std::string getId();
  std::string getName();
  void setName(const std::string& name);

  // Recording
  void arm(bool armed);
  bool isArmed();
  void setMonitorMode(MonitorMode mode);

  // Playback
  void addRegion(const AudioRegion& region);
  void removeRegion(const std::string& regionId);
  std::vector<AudioRegion> getRegions();

  // Mixing
  void setVolume(double db);          // -∞ to +12 dB
  void setPan(double pan);             // -1.0 (L) to +1.0 (R)
  void setMute(bool muted);
  void setSolo(bool soloed);

  // Plugins
  void addPlugin(std::shared_ptr<AudioPlugin> plugin, int index);
  void removePlugin(int index);
  void movePlugin(int fromIndex, int toIndex);
  std::vector<std::shared_ptr<AudioPlugin>> getPlugins();

  // Automation
  void addAutomation(const AutomationCurve& curve);
  AutomationCurve getAutomation(const std::string& parameterId);

  // Metering
  MeterData getCurrentMeters();
};

enum class MonitorMode {
  Off,     // No monitoring
  Input,   // Always monitor input
  Auto     // Monitor when armed and not playing
};

struct AudioRegion {
  std::string id;
  std::string name;
  double startTime;    // Seconds on timeline
  double duration;     // Seconds
  std::string audioFileId;
  double fileOffset;   // Offset into source file
  double fadeIn;       // Fade in duration
  double fadeOut;      // Fade out duration
  double gain;         // Region gain in dB
};

struct MeterData {
  double peakL;         // Peak level left channel (dB)
  double peakR;         // Peak level right channel (dB)
  double rmsL;          // RMS level left channel (dB)
  double rmsR;          // RMS level right channel (dB)
  bool clipping;        // True if clipping detected
};
```

### 3. Audio Plugin Host

**Responsibilities:**
- Load VST3, AU, AAX plugins
- Manage plugin state
- Route audio through plugins
- Expose plugin parameters

**Interface:**
```cpp
class AudioPlugin {
public:
  // Plugin info
  std::string getId();
  std::string getName();
  std::string getVendor();
  PluginFormat getFormat();

  // State management
  void loadState(const std::vector<uint8_t>& state);
  std::vector<uint8_t> saveState();

  // Parameters
  std::vector<PluginParameter> getParameters();
  void setParameter(int index, double value);
  double getParameter(int index);

  // Processing
  void processBlock(AudioBuffer& buffer, MidiBuffer& midi);

  // UI
  bool hasEditor();
  void openEditor();
  void closeEditor();
};

enum class PluginFormat {
  VST3,
  AudioUnit,
  AAX,
  Builtin
};

struct PluginParameter {
  int index;
  std::string id;
  std::string name;
  double defaultValue;
  double minValue;
  double maxValue;
  bool isAutomatable;
  std::string unit;        // "dB", "Hz", "%", etc.
  std::vector<std::string> valueStrings; // For discrete parameters
};
```

### 4. Master Bus

**Responsibilities:**
- Final mix summing
- Master effects chain
- Loudness metering (LUFS)
- Output limiting

**Interface:**
```cpp
class MasterBus {
public:
  // Mixing
  void setVolume(double db);
  double getVolume();

  // Plugins
  void addPlugin(std::shared_ptr<AudioPlugin> plugin, int index);
  void removePlugin(int index);
  std::vector<std::shared_ptr<AudioPlugin>> getPlugins();

  // Metering
  MeterData getCurrentMeters();
  LoudnessMetrics getLoudnessMetrics();

  // Export
  bool exportAudio(const ExportSettings& settings);
};

struct LoudnessMetrics {
  double integratedLUFS;   // Integrated loudness
  double shortTermLUFS;    // Short-term loudness (3s)
  double momentaryLUFS;    // Momentary loudness (400ms)
  double truePeakdBTP;     // True peak level
  double loudnessRange;    // LU (Loudness Units)
};

struct ExportSettings {
  std::string filepath;
  AudioFormat format;      // WAV, MP3, FLAC, etc.
  int sampleRate;
  int bitDepth;
  int bitrate;             // For lossy formats
  double startTime;
  double endTime;
  bool normalize;
  double normalizationTarget; // LUFS
};

enum class AudioFormat {
  WAV,
  AIFF,
  FLAC,
  MP3,
  AAC,
  OGG
};
```

### 5. Transport Controller

**Responsibilities:**
- Playback control (play, stop, pause)
- Timeline position management
- Tempo and time signature handling
- Loop control

**Interface:**
```cpp
class TransportController {
public:
  // Playback control
  void play();
  void stop();
  void pause();
  bool isPlaying();
  bool isPaused();

  // Position
  void setPosition(double seconds);
  double getPosition();                    // Current position in seconds
  int getCurrentMeasure();                 // Musical position
  double getCurrentBeat();

  // Tempo
  void setTempo(double bpm);
  double getTempo();

  // Time signature
  void setTimeSignature(int numerator, int denominator);
  TimeSignature getTimeSignature();

  // Loop
  void setLoopEnabled(bool enabled);
  void setLoopRange(double startSeconds, double endSeconds);
  bool isLoopEnabled();
  LoopRange getLoopRange();

  // Callbacks
  void setPositionCallback(std::function<void(double)> callback);
};

struct TimeSignature {
  int numerator;
  int denominator;
};

struct LoopRange {
  double start;  // Seconds
  double end;    // Seconds
};
```

### 6. Audio File Manager

**Responsibilities:**
- Load and cache audio files
- Handle various formats (WAV, MP3, FLAC, etc.)
- Resample if needed
- Provide efficient access for playback

**Interface:**
```cpp
class AudioFileManager {
public:
  // File loading
  std::string loadFile(const std::string& filepath);
  void unloadFile(const std::string& fileId);

  // File info
  AudioFileInfo getFileInfo(const std::string& fileId);

  // Audio data access
  bool readSamples(
    const std::string& fileId,
    double startTime,
    int numSamples,
    AudioBuffer& buffer
  );
};

struct AudioFileInfo {
  std::string id;
  std::string filepath;
  int sampleRate;
  int channels;
  int bitDepth;
  double duration;      // Seconds
  int64_t totalSamples;
  AudioFormat format;
};
```

## Real-Time Processing Pipeline

### Audio Callback Flow

```cpp
void audioCallback(const float** inputChannels,
                   int numInputChannels,
                   float** outputChannels,
                   int numOutputChannels,
                   int numSamples) {

  // 1. Clear output buffers
  for (int ch = 0; ch < numOutputChannels; ++ch) {
    std::memset(outputChannels[ch], 0, numSamples * sizeof(float));
  }

  // 2. Get current transport position
  double currentPosition = transport.getPosition();

  // 3. Process each track
  for (auto& track : tracks) {
    if (track->isMuted()) continue;

    AudioBuffer trackBuffer(numOutputChannels, numSamples);

    // 3a. Playback regions
    track->renderRegions(currentPosition, numSamples, trackBuffer);

    // 3b. Recording (if armed)
    if (track->isArmed() && isRecording) {
      track->recordInput(inputChannels, numInputChannels, numSamples);
    }

    // 3c. Plugin processing
    for (auto& plugin : track->getPlugins()) {
      plugin->processBlock(trackBuffer, midiBuffer);
    }

    // 3d. Automation
    track->applyAutomation(currentPosition, numSamples, trackBuffer);

    // 3e. Mix into output (with volume/pan)
    track->mixIntoOutput(trackBuffer, outputChannels, numSamples);
  }

  // 4. Master bus processing
  AudioBuffer masterBuffer(outputChannels, numOutputChannels, numSamples);
  for (auto& plugin : masterBus->getPlugins()) {
    plugin->processBlock(masterBuffer, midiBuffer);
  }

  // 5. Master volume
  masterBus->applyVolume(masterBuffer);

  // 6. Metering
  updateMeters(masterBuffer);

  // 7. Advance transport
  double deltaTime = numSamples / sampleRate;
  transport.advance(deltaTime);
}
```

## Performance Optimization

### Buffer Management

```cpp
class BufferPool {
public:
  // Reusable buffers to avoid allocations in audio thread
  AudioBuffer* acquire(int channels, int samples);
  void release(AudioBuffer* buffer);

private:
  std::vector<std::unique_ptr<AudioBuffer>> pool;
  std::mutex mutex; // For non-real-time thread safety
};
```

### Lock-Free Communication

```cpp
// Thread-safe message queue for UI → Audio thread
template<typename T>
class LockFreeQueue {
public:
  bool push(const T& item);
  bool pop(T& item);

private:
  juce::AbstractFifo fifo;
  std::vector<T> buffer;
};

// Usage in audio callback:
void audioCallback(...) {
  ParameterChange change;
  while (parameterQueue.pop(change)) {
    applyParameterChange(change);
  }

  // ... normal processing
}
```

## Built-In Plugins

Orpheus includes essential built-in plugins:

### 1. EQ (4-band parametric)
- High-pass filter
- 2 parametric mid bands
- High-shelf

### 2. Compressor
- Threshold, ratio, attack, release, makeup gain
- RMS/Peak detection
- Sidechain input

### 3. Limiter
- True peak detection
- Look-ahead
- Transparent algorithm

### 4. Reverb
- Room, hall, plate algorithms
- Pre-delay, decay, wet/dry mix

### 5. Delay
- Stereo delay with feedback
- Tempo-sync option
- Filtering

### 6. Gate/Expander
- Noise gate with attack/release
- Range control

### 7. Saturation
- Tape/tube emulation
- Harmonic enhancement

### 8. Stereo Tool
- Width control
- Mid/side processing
- Haas effect

### 9. Metering
- LUFS loudness metering
- Spectrum analyzer
- Phase correlation

### 10. Utility
- Gain, polarity invert
- Mono/stereo conversion
- DC offset removal

## Testing

### Unit Tests
```cpp
TEST(AudioTrackTest, VolumeControl) {
  AudioTrack track;
  track.setVolume(-6.0);
  EXPECT_NEAR(track.getVolume(), -6.0, 0.01);
}

TEST(TransportTest, PlaybackPosition) {
  TransportController transport;
  transport.setTempo(120);
  transport.setPosition(10.0);
  EXPECT_NEAR(transport.getPosition(), 10.0, 0.001);
}
```

### Integration Tests
```cpp
TEST(AudioEngineTest, RecordPlaybackRoundtrip) {
  // 1. Set up audio engine
  MaestroAudioEngine engine;
  engine.initialize({.sampleRate = 48000, .bufferSize = 512});

  // 2. Create track and arm for recording
  auto track = engine.createTrack();
  track->arm(true);

  // 3. Record 10 seconds of input
  engine.startRecording();
  std::this_thread::sleep_for(std::chrono::seconds(10));
  engine.stopRecording();

  // 4. Play back and verify
  track->arm(false);
  engine.play();

  // ... verify audio matches input
}
```

## Platform-Specific Considerations

### Windows (ASIO)
- Low-latency ASIO drivers required for pro audio
- Fallback to WASAPI for consumer hardware
- Direct2D for plugin UI rendering

### macOS (Core Audio)
- Native Core Audio driver (excellent latency)
- Audio Unit plugin format
- Metal for plugin UI rendering

### Linux (ALSA/JACK)
- ALSA for basic audio
- JACK for professional routing and low latency
- X11 or Wayland for plugin UIs

## Future Enhancements

- [ ] GPU-accelerated audio processing (CUDA/Metal)
- [ ] Distributed processing (offload to cloud)
- [ ] Advanced time-stretching algorithms
- [ ] AI-powered noise reduction plugin
- [ ] Spatial audio rendering (Dolby Atmos)

---

**Version History:**
- 1.0 (2025-11-16): Initial specification

**Next Steps:**
1. Implement core JUCE audio engine
2. Build plugin hosting system
3. Create built-in plugins
4. Integration with frontend
5. Performance profiling and optimization
