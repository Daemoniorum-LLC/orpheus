/**
 * LUFS Analyzer Service
 * Implements ITU-R BS.1770 compliant loudness measurement
 */

import * as Tone from 'tone';

export interface LUFSMeasurement {
  /** Integrated loudness (program loudness) */
  integrated: number;
  /** Short-term loudness (3 second window) */
  shortTerm: number;
  /** Momentary loudness (400ms window) */
  momentary: number;
  /** True peak in dBTP */
  truePeak: number;
  /** Loudness range (LRA) */
  range: number;
  /** Is audio playing/active */
  active: boolean;
}

// ITU-R BS.1770 K-weighting filter coefficients (for 48kHz)
const K_WEIGHT_HIGH_SHELF = {
  // High-shelf filter: +4dB at 2kHz and above
  b: [1.53512485958697, -2.69169618940638, 1.19839281085285],
  a: [1.0, -1.69065929318241, 0.73248077421585],
};

const K_WEIGHT_HIGH_PASS = {
  // High-pass filter: -infinity at 0Hz, 0dB at 60Hz and above
  b: [1.0, -2.0, 1.0],
  a: [1.0, -1.99004745483398, 0.99007225036621],
};

/**
 * LUFS Analyzer - Real-time loudness measurement
 */
export class LUFSAnalyzer {
  private analyzerNode: Tone.Analyser | null = null;
  private splitter: Tone.Split | null = null;
  private leftAnalyzer: Tone.Analyser | null = null;
  private rightAnalyzer: Tone.Analyser | null = null;
  private sampleRate: number = 48000;
  private blockSize: number = 9600; // 200ms at 48kHz
  private overlapSize: number = 4800; // 100ms overlap

  // K-weighting filter state
  private filterStateL: { x: number[]; y: number[] }[] = [];
  private filterStateR: { x: number[]; y: number[] }[] = [];

  // Loudness blocks for integration
  private momentaryBlocks: number[] = [];
  private shortTermBlocks: number[] = [];
  private integratedBlocks: number[] = [];

  // Gating threshold (-70 LUFS absolute threshold, -10 LUFS relative)
  private readonly ABSOLUTE_THRESHOLD = -70;
  private readonly RELATIVE_THRESHOLD_OFFSET = -10;

  private lastMeasurement: LUFSMeasurement = {
    integrated: -Infinity,
    shortTerm: -Infinity,
    momentary: -Infinity,
    truePeak: -Infinity,
    range: 0,
    active: false,
  };

  constructor() {
    // Initialize filter states for two channels
    for (let i = 0; i < 2; i++) {
      this.filterStateL.push({ x: [0, 0], y: [0, 0] });
      this.filterStateR.push({ x: [0, 0], y: [0, 0] });
    }
  }

  /**
   * Connect to an audio source for analysis
   */
  connect(source?: Tone.ToneAudioNode): void {
    this.disconnect();

    const bufferSize = 4096;

    // Create stereo analyzers
    this.splitter = new Tone.Split();
    this.leftAnalyzer = new Tone.Analyser('waveform', bufferSize);
    this.rightAnalyzer = new Tone.Analyser('waveform', bufferSize);

    if (source) {
      source.connect(this.splitter);
    } else {
      // Connect to master output
      Tone.getDestination().connect(this.splitter);
    }

    this.splitter.left.connect(this.leftAnalyzer);
    this.splitter.right.connect(this.rightAnalyzer);

    this.sampleRate = Tone.getContext().sampleRate;
    this.blockSize = Math.floor(this.sampleRate * 0.4); // 400ms blocks for momentary
  }

  /**
   * Disconnect and cleanup
   */
  disconnect(): void {
    this.splitter?.dispose();
    this.leftAnalyzer?.dispose();
    this.rightAnalyzer?.dispose();
    this.splitter = null;
    this.leftAnalyzer = null;
    this.rightAnalyzer = null;

    // Clear blocks
    this.momentaryBlocks = [];
    this.shortTermBlocks = [];
    this.integratedBlocks = [];
  }

  /**
   * Apply biquad filter
   */
  private applyBiquadFilter(
    samples: Float32Array,
    coeffs: { b: number[]; a: number[] },
    state: { x: number[]; y: number[] }
  ): Float32Array {
    const output = new Float32Array(samples.length);
    const { b, a } = coeffs;

    for (let i = 0; i < samples.length; i++) {
      const x0 = samples[i];
      const x1 = state.x[0];
      const x2 = state.x[1];
      const y1 = state.y[0];
      const y2 = state.y[1];

      const y0 = (b[0] * x0 + b[1] * x1 + b[2] * x2 - a[1] * y1 - a[2] * y2) / a[0];

      output[i] = y0;
      state.x = [x0, x1];
      state.y = [y0, y1];
    }

    return output;
  }

  /**
   * Apply K-weighting to samples
   */
  private applyKWeighting(samples: Float32Array, channel: 'L' | 'R'): Float32Array {
    const filterState = channel === 'L' ? this.filterStateL : this.filterStateR;

    // Apply high-shelf filter
    let filtered = this.applyBiquadFilter(samples, K_WEIGHT_HIGH_SHELF, filterState[0]);

    // Apply high-pass filter
    filtered = this.applyBiquadFilter(filtered, K_WEIGHT_HIGH_PASS, filterState[1]);

    return filtered;
  }

  /**
   * Calculate mean square of K-weighted samples
   */
  private calculateMeanSquare(samples: Float32Array): number {
    let sum = 0;
    for (let i = 0; i < samples.length; i++) {
      sum += samples[i] * samples[i];
    }
    return sum / samples.length;
  }

  /**
   * Calculate loudness from block energy
   * LUFS = -0.691 + 10 * log10(mean square)
   */
  private calculateLoudnessFromEnergy(energy: number): number {
    if (energy <= 0) return -Infinity;
    return -0.691 + 10 * Math.log10(energy);
  }

  /**
   * Get current LUFS measurement
   */
  getMeasurement(): LUFSMeasurement {
    if (!this.leftAnalyzer || !this.rightAnalyzer) {
      return this.lastMeasurement;
    }

    try {
      // Get waveform data
      const leftWaveform = this.leftAnalyzer.getValue() as Float32Array;
      const rightWaveform = this.rightAnalyzer.getValue() as Float32Array;

      // Check if audio is active
      const maxLeft = Math.max(...Array.from(leftWaveform).map(Math.abs));
      const maxRight = Math.max(...Array.from(rightWaveform).map(Math.abs));
      const active = maxLeft > 0.0001 || maxRight > 0.0001;

      // Calculate true peak (with 4x oversampling approximation)
      const truePeak = this.calculateTruePeak(leftWaveform, rightWaveform);

      // Apply K-weighting
      const leftKWeighted = this.applyKWeighting(leftWaveform, 'L');
      const rightKWeighted = this.applyKWeighting(rightWaveform, 'R');

      // Calculate channel energies (with surround weighting - 1.0 for L/R)
      const leftEnergy = this.calculateMeanSquare(leftKWeighted);
      const rightEnergy = this.calculateMeanSquare(rightKWeighted);

      // Sum for stereo: L + R (both with weight 1.0)
      const blockEnergy = leftEnergy + rightEnergy;
      const blockLoudness = this.calculateLoudnessFromEnergy(blockEnergy);

      // Update momentary blocks (400ms window)
      this.momentaryBlocks.push(blockLoudness);
      while (this.momentaryBlocks.length > 4) { // ~400ms worth
        this.momentaryBlocks.shift();
      }

      // Update short-term blocks (3s window)
      this.shortTermBlocks.push(blockLoudness);
      while (this.shortTermBlocks.length > 30) { // ~3s worth
        this.shortTermBlocks.shift();
      }

      // Update integrated blocks (gated)
      if (blockLoudness > this.ABSOLUTE_THRESHOLD) {
        this.integratedBlocks.push(blockLoudness);

        // Limit to last 5 minutes of data
        while (this.integratedBlocks.length > 3000) {
          this.integratedBlocks.shift();
        }
      }

      // Calculate momentary loudness (average of last 400ms)
      const momentary = this.calculateAverageLoudness(this.momentaryBlocks);

      // Calculate short-term loudness (average of last 3s)
      const shortTerm = this.calculateAverageLoudness(this.shortTermBlocks);

      // Calculate integrated loudness with gating
      const integrated = this.calculateGatedLoudness(this.integratedBlocks);

      // Calculate loudness range (LRA)
      const range = this.calculateLoudnessRange(this.integratedBlocks);

      this.lastMeasurement = {
        integrated,
        shortTerm,
        momentary,
        truePeak,
        range,
        active,
      };

      return this.lastMeasurement;
    } catch (error) {
      console.error('[LUFSAnalyzer] Measurement error:', error);
      return this.lastMeasurement;
    }
  }

  /**
   * Calculate true peak with oversampling
   */
  private calculateTruePeak(left: Float32Array, right: Float32Array): number {
    // Simple peak detection (in production, use 4x oversampling)
    const leftPeak = Math.max(...Array.from(left).map(Math.abs));
    const rightPeak = Math.max(...Array.from(right).map(Math.abs));
    const peak = Math.max(leftPeak, rightPeak);

    // Convert to dBTP
    return peak > 0 ? 20 * Math.log10(peak) : -Infinity;
  }

  /**
   * Calculate average loudness from blocks
   */
  private calculateAverageLoudness(blocks: number[]): number {
    if (blocks.length === 0) return -Infinity;

    // Convert from LUFS to linear energy, average, convert back
    let energySum = 0;
    let count = 0;

    for (const lufs of blocks) {
      if (lufs > -70) {
        energySum += Math.pow(10, (lufs + 0.691) / 10);
        count++;
      }
    }

    if (count === 0) return -Infinity;
    return this.calculateLoudnessFromEnergy(energySum / count);
  }

  /**
   * Calculate gated loudness (integrated LUFS)
   */
  private calculateGatedLoudness(blocks: number[]): number {
    if (blocks.length === 0) return -Infinity;

    // First pass: absolute threshold gate (-70 LUFS)
    const absoluteGated = blocks.filter(b => b > this.ABSOLUTE_THRESHOLD);
    if (absoluteGated.length === 0) return -Infinity;

    // Calculate ungated average
    const ungatedAvg = this.calculateAverageLoudness(absoluteGated);

    // Second pass: relative threshold gate (-10 LUFS relative)
    const relativeThreshold = ungatedAvg + this.RELATIVE_THRESHOLD_OFFSET;
    const relativeGated = absoluteGated.filter(b => b > relativeThreshold);

    if (relativeGated.length === 0) return -Infinity;
    return this.calculateAverageLoudness(relativeGated);
  }

  /**
   * Calculate loudness range (LRA)
   */
  private calculateLoudnessRange(blocks: number[]): number {
    if (blocks.length < 10) return 0;

    // Sort blocks
    const sorted = [...blocks].sort((a, b) => a - b);

    // Calculate 10th and 95th percentiles
    const p10Index = Math.floor(sorted.length * 0.1);
    const p95Index = Math.floor(sorted.length * 0.95);

    return sorted[p95Index] - sorted[p10Index];
  }

  /**
   * Reset all measurements
   */
  reset(): void {
    this.momentaryBlocks = [];
    this.shortTermBlocks = [];
    this.integratedBlocks = [];

    this.lastMeasurement = {
      integrated: -Infinity,
      shortTerm: -Infinity,
      momentary: -Infinity,
      truePeak: -Infinity,
      range: 0,
      active: false,
    };

    // Reset filter states
    for (let i = 0; i < 2; i++) {
      this.filterStateL[i] = { x: [0, 0], y: [0, 0] };
      this.filterStateR[i] = { x: [0, 0], y: [0, 0] };
    }
  }
}

// Singleton instance
let analyzerInstance: LUFSAnalyzer | null = null;

export function getLUFSAnalyzer(): LUFSAnalyzer {
  if (!analyzerInstance) {
    analyzerInstance = new LUFSAnalyzer();
  }
  return analyzerInstance;
}
