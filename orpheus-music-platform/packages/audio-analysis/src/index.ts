/**
 * @orpheus/audio-analysis
 *
 * Comprehensive audio analysis utilities for Maestro AI
 */

// Peak detection
export {
  PeakDetector,
  TruePeakDetector,
  type PeakAnalysis,
} from './peak-detector';

// RMS metering
export {
  RMSMeter,
  VUMeter,
  type RMSAnalysis,
  type RMSMeterOptions,
} from './rms-meter';

// LUFS loudness metering
export {
  LUFSMeter,
  normalizeLUFS,
  LOUDNESS_TARGETS,
  type LUFSAnalysis,
  type LUFSMeterOptions,
} from './lufs-meter';

// Frequency analysis
export {
  FrequencyAnalyzer,
  FREQUENCY_BANDS,
  PROBLEMATIC_FREQUENCIES,
  type FrequencySpectrum,
  type FrequencyBand,
  type FrequencyAnalyzerOptions,
} from './frequency-analyzer';

// Phase and stereo analysis
export {
  PhaseMeter,
  StereoWidthAnalyzer,
  Goniometer,
  type PhaseAnalysis,
  type PhaseMeterOptions,
} from './phase-meter';
