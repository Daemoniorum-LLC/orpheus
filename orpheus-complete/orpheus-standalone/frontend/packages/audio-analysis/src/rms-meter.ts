/**
 * RMS Level Meter - Measures Root Mean Square levels for perceived loudness
 */

export interface RMSAnalysis {
  /** RMS level in dB */
  rmsDb: number;
  /** RMS level as linear amplitude */
  rmsLinear: number;
  /** Dynamic range (peak - RMS) in dB */
  dynamicRange: number;
  /** Peak level in dB */
  peakDb: number;
}

export interface RMSMeterOptions {
  /** Window size in samples for RMS calculation */
  windowSize?: number;
  /** Hop size in samples (how much to advance window) */
  hopSize?: number;
  /** Whether to use time-weighted averaging */
  timeWeighted?: boolean;
  /** Time constant for time-weighted averaging in seconds */
  timeConstant?: number;
  /** Sample rate (needed for time-weighted averaging) */
  sampleRate?: number;
}

/**
 * Measures RMS levels in audio signals
 */
export class RMSMeter {
  private readonly windowSize: number;
  private readonly hopSize: number;
  private readonly timeWeighted: boolean;
  private readonly timeConstant: number;
  private readonly sampleRate: number;

  constructor(options: RMSMeterOptions = {}) {
    this.windowSize = options.windowSize || 4096;
    this.hopSize = options.hopSize || this.windowSize / 4;
    this.timeWeighted = options.timeWeighted || false;
    this.timeConstant = options.timeConstant || 0.3; // 300ms default
    this.sampleRate = options.sampleRate || 44100;
  }

  /**
   * Calculates RMS level for entire buffer
   */
  analyze(samples: Float32Array): RMSAnalysis {
    const rmsLinear = this.calculateRMS(samples);
    const rmsDb = this.linearToDb(rmsLinear);

    // Calculate peak for dynamic range
    let peak = 0;
    for (let i = 0; i < samples.length; i++) {
      peak = Math.max(peak, Math.abs(samples[i]));
    }
    const peakDb = this.linearToDb(peak);

    const dynamicRange = peakDb - rmsDb;

    return {
      rmsDb,
      rmsLinear,
      dynamicRange,
      peakDb,
    };
  }

  /**
   * Calculates RMS level over time using sliding window
   */
  analyzeOverTime(samples: Float32Array): Array<{
    time: number;
    rmsDb: number;
    rmsLinear: number;
  }> {
    const results: Array<{ time: number; rmsDb: number; rmsLinear: number }> = [];

    for (let i = 0; i <= samples.length - this.windowSize; i += this.hopSize) {
      const window = samples.slice(i, i + this.windowSize);
      const rmsLinear = this.calculateRMS(window);
      const rmsDb = this.linearToDb(rmsLinear);
      const time = i / this.sampleRate;

      results.push({ time, rmsDb, rmsLinear });
    }

    return results;
  }

  /**
   * Calculates RMS with time-weighted averaging (slow/fast attack/release)
   */
  analyzeTimeWeighted(samples: Float32Array): RMSAnalysis {
    const alpha = Math.exp(-1 / (this.timeConstant * this.sampleRate));
    let runningRMS = 0;

    for (let i = 0; i < samples.length; i++) {
      const sample = samples[i];
      const squaredSample = sample * sample;

      // Time-weighted averaging
      runningRMS = alpha * runningRMS + (1 - alpha) * squaredSample;
    }

    const rmsLinear = Math.sqrt(runningRMS);
    const rmsDb = this.linearToDb(rmsLinear);

    // Calculate peak
    let peak = 0;
    for (let i = 0; i < samples.length; i++) {
      peak = Math.max(peak, Math.abs(samples[i]));
    }
    const peakDb = this.linearToDb(peak);

    return {
      rmsDb,
      rmsLinear,
      dynamicRange: peakDb - rmsDb,
      peakDb,
    };
  }

  /**
   * Calculates RMS for stereo signal
   */
  analyzeStereo(
    leftChannel: Float32Array,
    rightChannel: Float32Array
  ): {
    left: RMSAnalysis;
    right: RMSAnalysis;
    stereo: RMSAnalysis;
  } {
    const left = this.analyze(leftChannel);
    const right = this.analyze(rightChannel);

    // Calculate combined stereo RMS
    const stereoSamples = new Float32Array(leftChannel.length);
    for (let i = 0; i < leftChannel.length; i++) {
      stereoSamples[i] = (leftChannel[i] + rightChannel[i]) / 2;
    }
    const stereo = this.analyze(stereoSamples);

    return { left, right, stereo };
  }

  /**
   * Calculates basic RMS from samples
   */
  private calculateRMS(samples: Float32Array): number {
    let sumSquares = 0;

    for (let i = 0; i < samples.length; i++) {
      sumSquares += samples[i] * samples[i];
    }

    return Math.sqrt(sumSquares / samples.length);
  }

  /**
   * Converts linear amplitude to dB
   */
  private linearToDb(linear: number): number {
    if (linear === 0) return -Infinity;
    return 20 * Math.log10(linear);
  }
}

/**
 * VU Meter - Classic VU meter with ballistics
 */
export class VUMeter {
  private currentLevel: number = 0;
  private readonly attackTime: number;
  private readonly releaseTime: number;
  private readonly sampleRate: number;
  private readonly attackCoeff: number;
  private readonly releaseCoeff: number;

  constructor(sampleRate: number = 44100, attackTime: number = 0.3, releaseTime: number = 0.3) {
    this.sampleRate = sampleRate;
    this.attackTime = attackTime; // 300ms (VU meter standard)
    this.releaseTime = releaseTime; // 300ms

    // Calculate coefficients for exponential averaging
    this.attackCoeff = Math.exp(-1 / (this.attackTime * this.sampleRate));
    this.releaseCoeff = Math.exp(-1 / (this.releaseTime * this.sampleRate));
  }

  /**
   * Processes audio buffer and returns VU level
   */
  process(samples: Float32Array): number {
    for (let i = 0; i < samples.length; i++) {
      const absSample = Math.abs(samples[i]);

      if (absSample > this.currentLevel) {
        // Attack (rising)
        this.currentLevel =
          this.attackCoeff * this.currentLevel + (1 - this.attackCoeff) * absSample;
      } else {
        // Release (falling)
        this.currentLevel =
          this.releaseCoeff * this.currentLevel + (1 - this.releaseCoeff) * absSample;
      }
    }

    return this.currentLevel;
  }

  /**
   * Gets current VU level in dB
   */
  getLevel(): number {
    return this.linearToDb(this.currentLevel);
  }

  /**
   * Resets the meter
   */
  reset(): void {
    this.currentLevel = 0;
  }

  private linearToDb(linear: number): number {
    if (linear === 0) return -Infinity;
    return 20 * Math.log10(linear);
  }
}
