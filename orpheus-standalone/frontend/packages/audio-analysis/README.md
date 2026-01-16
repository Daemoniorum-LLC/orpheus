# @maestro-ai/audio-analysis

Comprehensive audio analysis utilities for Orpheus, providing professional-grade metering, frequency analysis, and stereo phase detection.

## Features

- **Peak Detection** - Sample-accurate peak detection with true peak metering
- **RMS Metering** - Root Mean Square level measurement with VU ballistics
- **LUFS Metering** - ITU-R BS.1770-4 compliant loudness measurement
- **Frequency Analysis** - FFT-based spectrum analysis with configurable window functions
- **Phase Correlation** - Stereo phase relationship analysis and mono compatibility checking
- **Stereo Width** - Mid/side analysis and stereo field measurement

## Installation

```bash
npm install @maestro-ai/audio-analysis
```

## Usage

### Peak Detection

```typescript
import { PeakDetector, TruePeakDetector } from '@maestro-ai/audio-analysis';

// Basic peak detection
const peakDetector = new PeakDetector();
const analysis = peakDetector.analyze(audioSamples);

console.log(`Peak level: ${analysis.peakDb.toFixed(2)} dB`);
console.log(`Clipped: ${analysis.clipped}`);
console.log(`Crest factor: ${analysis.crestFactor.toFixed(2)} dB`);

// True peak detection (ITU-R BS.1770-4)
const truePeakDetector = new TruePeakDetector(4); // 4x oversampling
const truePeakAnalysis = truePeakDetector.analyze(audioSamples);

// Real-time clipping detection
if (peakDetector.detectClipping(audioBuffer)) {
  console.warn('Clipping detected!');
}

// Find all clipping events
const events = peakDetector.findClippingEvents(audioSamples);
events.forEach(event => {
  console.log(`Clipping at samples ${event.start}-${event.end}, peak: ${event.peakValue}`);
});
```

### RMS Level Metering

```typescript
import { RMSMeter, VUMeter } from '@maestro-ai/audio-analysis';

// RMS analysis
const rmsMeter = new RMSMeter({
  windowSize: 4096,
  sampleRate: 44100,
});

const rmsAnalysis = rmsMeter.analyze(audioSamples);
console.log(`RMS level: ${rmsAnalysis.rmsDb.toFixed(2)} dB`);
console.log(`Dynamic range: ${rmsAnalysis.dynamicRange.toFixed(2)} dB`);

// RMS over time (sliding window)
const timeAnalysis = rmsMeter.analyzeOverTime(audioSamples);
timeAnalysis.forEach(({ time, rmsDb }) => {
  console.log(`Time: ${time.toFixed(2)}s, RMS: ${rmsDb.toFixed(2)} dB`);
});

// VU meter with ballistics
const vuMeter = new VUMeter(44100);
const level = vuMeter.process(audioBuffer);
console.log(`VU level: ${vuMeter.getLevel().toFixed(2)} dB`);
```

### LUFS Loudness Metering

```typescript
import { LUFSMeter, normalizeLUFS, LOUDNESS_TARGETS } from '@maestro-ai/audio-analysis';

// LUFS analysis (stereo)
const lufsMeter = new LUFSMeter({
  sampleRate: 44100,
  targetLoudness: LOUDNESS_TARGETS.SPOTIFY, // -14 LUFS
  channels: 2,
});

const lufsAnalysis = lufsMeter.analyze([leftChannel, rightChannel]);

console.log(`Integrated loudness: ${lufsAnalysis.integrated.toFixed(2)} LUFS`);
console.log(`Loudness range: ${lufsAnalysis.range.toFixed(2)} LU`);
console.log(`Short-term loudness: ${lufsAnalysis.shortTerm.toFixed(2)} LUFS`);
console.log(`Momentary loudness: ${lufsAnalysis.momentary.toFixed(2)} LUFS`);
console.log(`True peak: ${lufsAnalysis.truePeak.toFixed(2)} dBTP`);
console.log(`Compliant: ${lufsAnalysis.compliant ? 'Yes' : 'No'}`);

// Normalize audio to target LUFS
const normalized = normalizeLUFS([leftChannel, rightChannel], -14);
console.log('Audio normalized to -14 LUFS (Spotify)');

// Platform-specific targets
console.log('Loudness targets:');
console.log(`  Spotify: ${LOUDNESS_TARGETS.SPOTIFY} LUFS`);
console.log(`  Apple Music: ${LOUDNESS_TARGETS.APPLE_MUSIC} LUFS`);
console.log(`  YouTube: ${LOUDNESS_TARGETS.YOUTUBE} LUFS`);
console.log(`  Broadcast TV: ${LOUDNESS_TARGETS.BROADCAST_TV} LUFS`);
```

### Frequency Analysis

```typescript
import { FrequencyAnalyzer, FREQUENCY_BANDS } from '@maestro-ai/audio-analysis';

// Frequency spectrum analysis
const freqAnalyzer = new FrequencyAnalyzer({
  fftSize: 2048,
  sampleRate: 44100,
  windowFunction: 'hann',
  smoothing: 0.8,
});

const spectrum = freqAnalyzer.analyze(audioSamples);

console.log(`FFT size: ${spectrum.fftSize}`);
console.log(`Frequency resolution: ${spectrum.resolution.toFixed(2)} Hz`);

// Visualize spectrum
spectrum.frequencies.forEach((freq, i) => {
  const magnitude = spectrum.magnitudes[i];
  console.log(`${freq.toFixed(1)} Hz: ${magnitude.toFixed(2)} dB`);
});

// Analyze by frequency bands
const bands = freqAnalyzer.analyzeBands(audioSamples);
bands.forEach(band => {
  console.log(`${band.name} (${band.low}-${band.high} Hz): ${band.magnitude.toFixed(2)} dB`);
  console.log(`  Peak at: ${band.peakFrequency.toFixed(1)} Hz`);
});

// Pitch detection
const pitch = freqAnalyzer.detectPitch(audioSamples);
if (pitch) {
  console.log(`Detected pitch: ${pitch.toFixed(2)} Hz`);
}

// Standard frequency bands
console.log('Analyzing frequency bands:');
console.log(`  ${FREQUENCY_BANDS.SUB_BASS.name}: ${FREQUENCY_BANDS.SUB_BASS.low}-${FREQUENCY_BANDS.SUB_BASS.high} Hz`);
console.log(`  ${FREQUENCY_BANDS.BASS.name}: ${FREQUENCY_BANDS.BASS.low}-${FREQUENCY_BANDS.BASS.high} Hz`);
console.log(`  ${FREQUENCY_BANDS.MIDS.name}: ${FREQUENCY_BANDS.MIDS.low}-${FREQUENCY_BANDS.MIDS.high} Hz`);
console.log(`  ${FREQUENCY_BANDS.PRESENCE.name}: ${FREQUENCY_BANDS.PRESENCE.low}-${FREQUENCY_BANDS.PRESENCE.high} Hz`);
```

### Phase Correlation & Stereo Analysis

```typescript
import { PhaseMeter, StereoWidthAnalyzer, Goniometer } from '@maestro-ai/audio-analysis';

// Phase correlation analysis
const phaseMeter = new PhaseMeter({
  sampleRate: 44100,
  goodThreshold: 0.7,
  warningThreshold: 0.3,
});

const phaseAnalysis = phaseMeter.analyze(leftChannel, rightChannel);

console.log(`Phase correlation: ${phaseAnalysis.correlation.toFixed(3)}`);
console.log(`Coherence: ${phaseAnalysis.coherence.toFixed(3)}`);
console.log(`Mono compatible: ${phaseAnalysis.monoCompatible ? 'Yes' : 'No'}`);
console.log(`Status: ${phaseAnalysis.status}`);

// Get correction suggestion
const suggestion = phaseMeter.suggestCorrection(phaseAnalysis);
if (suggestion) {
  console.warn(`Phase issue: ${suggestion}`);
}

// Detect phase inversion
if (phaseMeter.detectPhaseInversion(leftChannel, rightChannel)) {
  console.error('Channels are 180° out of phase!');
}

// Stereo width analysis
const stereoAnalyzer = new StereoWidthAnalyzer();
const stereoAnalysis = stereoAnalyzer.analyze(leftChannel, rightChannel);

console.log(`Stereo width: ${stereoAnalysis.width.toFixed(2)}`);
console.log(`Balance: ${stereoAnalysis.balance.toFixed(2)} (${stereoAnalysis.balance > 0 ? 'right' : 'left'})`);
console.log(`Mono content: ${(stereoAnalysis.monoContent * 100).toFixed(1)}%`);
console.log(`Stereo content: ${(stereoAnalysis.stereoContent * 100).toFixed(1)}%`);

// Goniometer visualization
const goniometer = new Goniometer();
const points = goniometer.generate(leftChannel, rightChannel, 1000);

const pattern = goniometer.analyzePattern(leftChannel, rightChannel);
console.log(`Goniometer shape: ${pattern.shape}`);
console.log(`Mono compatibility: ${(pattern.monoCompatibility * 100).toFixed(1)}%`);
```

## API Reference

### Peak Detection

#### `PeakDetector`

Detects peak levels in audio signals.

**Methods:**
- `analyze(samples: Float32Array): PeakAnalysis` - Analyzes peak levels
- `analyzeStereo(left, right): { left, right, stereo }` - Analyzes stereo peaks
- `detectClipping(samples): boolean` - Real-time clipping detection
- `findClippingEvents(samples): ClipEvent[]` - Finds all clipping events

#### `TruePeakDetector`

ITU-R BS.1770-4 compliant true peak detector using oversampling.

### RMS Metering

#### `RMSMeter`

Measures RMS levels for perceived loudness.

**Options:**
- `windowSize` - Window size in samples (default: 4096)
- `hopSize` - Hop size for sliding window
- `timeWeighted` - Use time-weighted averaging
- `timeConstant` - Time constant for averaging (seconds)
- `sampleRate` - Sample rate

**Methods:**
- `analyze(samples): RMSAnalysis` - Full buffer analysis
- `analyzeOverTime(samples): TimeAnalysis[]` - Sliding window analysis
- `analyzeStereo(left, right): StereoRMSAnalysis` - Stereo analysis

#### `VUMeter`

Classic VU meter with 300ms attack/release ballistics.

### LUFS Metering

#### `LUFSMeter`

ITU-R BS.1770-4 compliant loudness meter.

**Options:**
- `sampleRate` - Sample rate (default: 44100)
- `targetLoudness` - Target LUFS (default: -14)
- `tolerance` - Compliance tolerance in LU
- `channels` - Number of channels (1, 2, or 6)

**Methods:**
- `analyze(channels: Float32Array[]): LUFSAnalysis` - Full LUFS analysis
- `reset()` - Resets meter state

**Constants:**
- `LOUDNESS_TARGETS` - Platform-specific loudness targets

### Frequency Analysis

#### `FrequencyAnalyzer`

FFT-based frequency spectrum analyzer.

**Options:**
- `fftSize` - FFT size, must be power of 2 (default: 2048)
- `sampleRate` - Sample rate
- `windowFunction` - 'hann', 'hamming', 'blackman', or 'none'
- `smoothing` - Temporal smoothing factor (0-1)

**Methods:**
- `analyze(samples): FrequencySpectrum` - Spectrum analysis
- `analyzeBands(samples): FrequencyBand[]` - Band-based analysis
- `detectPitch(samples): number | null` - Pitch detection
- `reset()` - Resets smoothing state

### Phase & Stereo Analysis

#### `PhaseMeter`

Stereo phase correlation analyzer.

**Methods:**
- `analyze(left, right): PhaseAnalysis` - Phase analysis
- `analyzeOverTime(left, right): TimePhaseAnalysis[]` - Time-based analysis
- `detectPhaseInversion(left, right): boolean` - Detects 180° phase flip
- `suggestCorrection(analysis): string | null` - Correction suggestions

#### `StereoWidthAnalyzer`

Analyzes stereo width and balance.

#### `Goniometer`

Generates goniometer visualization data for stereo phase.

## Performance

All analyzers are optimized for real-time performance:

- **Peak Detection**: O(n) - suitable for real-time monitoring
- **RMS Metering**: O(n) - real-time capable
- **LUFS Metering**: O(n) - buffered analysis, real-time streaming possible
- **FFT Analysis**: O(n log n) - real-time for FFT sizes up to 8192
- **Phase Correlation**: O(n) - real-time capable

## Standards Compliance

- **ITU-R BS.1770-4** - Loudness measurement (LUFS)
- **EBU R128** - Broadcast loudness
- **AES17** - Digital audio measurement
- **IEC 61606** - Digital audio interfaces

## License

MIT

## Author

Orpheus Team
