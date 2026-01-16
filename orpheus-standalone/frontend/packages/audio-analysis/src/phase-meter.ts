/**
 * Phase Correlation Meter - Detects stereo phase issues
 */

export interface PhaseAnalysis {
  /** Phase correlation coefficient (-1 to +1) */
  correlation: number;
  /** Left channel RMS level (dB) */
  leftLevel: number;
  /** Right channel RMS level (dB) */
  rightLevel: number;
  /** Mid (sum) level (dB) */
  midLevel: number;
  /** Side (difference) level (dB) */
  sideLevel: number;
  /** Phase coherence (0-1, higher is better) */
  coherence: number;
  /** Potential mono compatibility issues */
  monoCompatible: boolean;
  /** Phase status */
  status: 'good' | 'warning' | 'critical';
}

export interface PhaseMeterOptions {
  /** Sample rate */
  sampleRate?: number;
  /** Window size for analysis (samples) */
  windowSize?: number;
  /** Minimum correlation for good phase */
  goodThreshold?: number;
  /** Minimum correlation for acceptable phase */
  warningThreshold?: number;
}

/**
 * Phase correlation meter for stereo signals
 */
export class PhaseMeter {
  private readonly sampleRate: number;
  private readonly windowSize: number;
  private readonly goodThreshold: number;
  private readonly warningThreshold: number;

  constructor(options: PhaseMeterOptions = {}) {
    this.sampleRate = options.sampleRate || 44100;
    this.windowSize = options.windowSize || 4096;
    this.goodThreshold = options.goodThreshold || 0.7;
    this.warningThreshold = options.warningThreshold || 0.3;
  }

  /**
   * Analyzes phase correlation between left and right channels
   */
  analyze(leftChannel: Float32Array, rightChannel: Float32Array): PhaseAnalysis {
    if (leftChannel.length !== rightChannel.length) {
      throw new Error('Left and right channels must have same length');
    }

    // Calculate phase correlation
    const correlation = this.calculateCorrelation(leftChannel, rightChannel);

    // Calculate mid/side
    const { mid, side } = this.calculateMidSide(leftChannel, rightChannel);

    // Calculate RMS levels
    const leftLevel = this.calculateRMS(leftChannel);
    const rightLevel = this.calculateRMS(rightChannel);
    const midLevel = this.calculateRMS(mid);
    const sideLevel = this.calculateRMS(side);

    // Calculate coherence
    const coherence = this.calculateCoherence(leftChannel, rightChannel);

    // Determine mono compatibility
    const monoCompatible = correlation >= this.warningThreshold;

    // Determine status
    let status: 'good' | 'warning' | 'critical';
    if (correlation >= this.goodThreshold) {
      status = 'good';
    } else if (correlation >= this.warningThreshold) {
      status = 'warning';
    } else {
      status = 'critical';
    }

    return {
      correlation,
      leftLevel,
      rightLevel,
      midLevel,
      sideLevel,
      coherence,
      monoCompatible,
      status,
    };
  }

  /**
   * Analyzes phase correlation over time
   */
  analyzeOverTime(
    leftChannel: Float32Array,
    rightChannel: Float32Array
  ): Array<{ time: number; correlation: number; status: 'good' | 'warning' | 'critical' }> {
    const results: Array<{
      time: number;
      correlation: number;
      status: 'good' | 'warning' | 'critical';
    }> = [];

    const hopSize = this.windowSize / 2;

    for (let i = 0; i <= leftChannel.length - this.windowSize; i += hopSize) {
      const leftWindow = leftChannel.slice(i, i + this.windowSize);
      const rightWindow = rightChannel.slice(i, i + this.windowSize);

      const correlation = this.calculateCorrelation(leftWindow, rightWindow);
      const time = i / this.sampleRate;

      let status: 'good' | 'warning' | 'critical';
      if (correlation >= this.goodThreshold) {
        status = 'good';
      } else if (correlation >= this.warningThreshold) {
        status = 'warning';
      } else {
        status = 'critical';
      }

      results.push({ time, correlation, status });
    }

    return results;
  }

  /**
   * Calculates Pearson correlation coefficient
   */
  private calculateCorrelation(left: Float32Array, right: Float32Array): number {
    const N = left.length;

    // Calculate means
    let leftSum = 0;
    let rightSum = 0;
    for (let i = 0; i < N; i++) {
      leftSum += left[i];
      rightSum += right[i];
    }
    const leftMean = leftSum / N;
    const rightMean = rightSum / N;

    // Calculate correlation components
    let numerator = 0;
    let leftVariance = 0;
    let rightVariance = 0;

    for (let i = 0; i < N; i++) {
      const leftDiff = left[i] - leftMean;
      const rightDiff = right[i] - rightMean;

      numerator += leftDiff * rightDiff;
      leftVariance += leftDiff * leftDiff;
      rightVariance += rightDiff * rightDiff;
    }

    const denominator = Math.sqrt(leftVariance * rightVariance);

    if (denominator === 0) return 0;

    return numerator / denominator;
  }

  /**
   * Calculates mid/side from left/right
   */
  private calculateMidSide(
    left: Float32Array,
    right: Float32Array
  ): { mid: Float32Array; side: Float32Array } {
    const mid = new Float32Array(left.length);
    const side = new Float32Array(left.length);

    for (let i = 0; i < left.length; i++) {
      mid[i] = (left[i] + right[i]) / 2; // Sum
      side[i] = (left[i] - right[i]) / 2; // Difference
    }

    return { mid, side };
  }

  /**
   * Calculates RMS level in dB
   */
  private calculateRMS(samples: Float32Array): number {
    let sumSquares = 0;

    for (let i = 0; i < samples.length; i++) {
      sumSquares += samples[i] * samples[i];
    }

    const rms = Math.sqrt(sumSquares / samples.length);

    if (rms === 0) return -Infinity;

    return 20 * Math.log10(rms);
  }

  /**
   * Calculates phase coherence using magnitude squared coherence
   */
  private calculateCoherence(left: Float32Array, right: Float32Array): number {
    // Simplified coherence calculation
    // Real implementation would use Welch's method with FFT

    let crossPower = 0;
    let leftPower = 0;
    let rightPower = 0;

    for (let i = 0; i < left.length; i++) {
      crossPower += left[i] * right[i];
      leftPower += left[i] * left[i];
      rightPower += right[i] * right[i];
    }

    if (leftPower === 0 || rightPower === 0) return 0;

    const coherence = (crossPower * crossPower) / (leftPower * rightPower);

    return Math.max(0, Math.min(1, coherence));
  }

  /**
   * Detects phase inversion (180° out of phase)
   */
  detectPhaseInversion(left: Float32Array, right: Float32Array): boolean {
    const correlation = this.calculateCorrelation(left, right);

    // Negative correlation indicates phase inversion
    return correlation < -0.5;
  }

  /**
   * Suggests phase correction
   */
  suggestCorrection(analysis: PhaseAnalysis): string | null {
    if (analysis.status === 'good') {
      return null;
    }

    if (analysis.correlation < -0.5) {
      return 'Phase inverted: Flip polarity of one channel (180° phase flip)';
    }

    if (analysis.correlation >= 0 && analysis.correlation < this.warningThreshold) {
      return 'Poor phase correlation: Check microphone placement and polarity';
    }

    if (analysis.correlation < 0 && analysis.correlation > -0.5) {
      return 'Partial phase cancellation: Adjust microphone timing or use time alignment';
    }

    return 'Phase issues detected: Review microphone setup and signal routing';
  }
}

/**
 * Stereo width analyzer
 */
export class StereoWidthAnalyzer {
  /**
   * Analyzes stereo width
   */
  analyze(
    leftChannel: Float32Array,
    rightChannel: Float32Array
  ): {
    width: number; // 0 = mono, 1 = normal stereo, >1 = wide stereo
    balance: number; // -1 = left, 0 = center, +1 = right
    monoContent: number; // Percentage of mono content (0-1)
    stereoContent: number; // Percentage of stereo content (0-1)
  } {
    // Calculate mid/side
    const mid = new Float32Array(leftChannel.length);
    const side = new Float32Array(leftChannel.length);

    for (let i = 0; i < leftChannel.length; i++) {
      mid[i] = (leftChannel[i] + rightChannel[i]) / 2;
      side[i] = (leftChannel[i] - rightChannel[i]) / 2;
    }

    // Calculate RMS of mid and side
    const midRMS = this.calculateRMS(mid);
    const sideRMS = this.calculateRMS(side);

    // Calculate stereo width
    let width: number;
    if (midRMS === 0) {
      width = 2; // Maximum width (only side signal)
    } else if (sideRMS === 0) {
      width = 0; // Mono
    } else {
      width = sideRMS / midRMS;
    }

    // Calculate balance
    const leftRMS = this.calculateRMS(leftChannel);
    const rightRMS = this.calculateRMS(rightChannel);
    const balance = (rightRMS - leftRMS) / Math.max(rightRMS, leftRMS);

    // Calculate mono/stereo content percentages
    const totalEnergy = midRMS * midRMS + sideRMS * sideRMS;
    const monoContent = totalEnergy > 0 ? (midRMS * midRMS) / totalEnergy : 1;
    const stereoContent = 1 - monoContent;

    return {
      width,
      balance,
      monoContent,
      stereoContent,
    };
  }

  private calculateRMS(samples: Float32Array): number {
    let sumSquares = 0;

    for (let i = 0; i < samples.length; i++) {
      sumSquares += samples[i] * samples[i];
    }

    return Math.sqrt(sumSquares / samples.length);
  }
}

/**
 * Goniometer for stereo phase visualization
 */
export class Goniometer {
  /**
   * Generates goniometer points for visualization
   * Returns array of [x, y] coordinates where:
   * - x represents left-right (horizontal)
   * - y represents mid-side (vertical)
   */
  generate(
    leftChannel: Float32Array,
    rightChannel: Float32Array,
    numPoints: number = 1000
  ): Array<[number, number]> {
    const points: Array<[number, number]> = [];
    const step = Math.max(1, Math.floor(leftChannel.length / numPoints));

    for (let i = 0; i < leftChannel.length; i += step) {
      const left = leftChannel[i];
      const right = rightChannel[i];

      // Calculate mid/side
      const mid = (left + right) / 2;
      const side = (left - right) / 2;

      points.push([side, mid]); // X = side, Y = mid
    }

    return points;
  }

  /**
   * Analyzes goniometer pattern
   */
  analyzePattern(
    leftChannel: Float32Array,
    rightChannel: Float32Array
  ): {
    shape: 'vertical' | 'horizontal' | 'diagonal' | 'circular' | 'irregular';
    monoCompatibility: number; // 0-1
  } {
    const points = this.generate(leftChannel, rightChannel, 100);

    // Calculate spread in horizontal vs vertical directions
    let horizontalSpread = 0;
    let verticalSpread = 0;

    for (const [x, y] of points) {
      horizontalSpread += Math.abs(x);
      verticalSpread += Math.abs(y);
    }

    horizontalSpread /= points.length;
    verticalSpread /= points.length;

    // Determine shape
    let shape: 'vertical' | 'horizontal' | 'diagonal' | 'circular' | 'irregular';

    const ratio = horizontalSpread / (verticalSpread + 0.0001);

    if (ratio < 0.2) {
      shape = 'vertical'; // Mono signal
    } else if (ratio > 5.0) {
      shape = 'horizontal'; // Out of phase
    } else if (ratio > 0.7 && ratio < 1.3) {
      shape = 'circular'; // Well-balanced stereo
    } else if (ratio > 0.3 && ratio < 0.7) {
      shape = 'diagonal'; // Mostly mono with some stereo
    } else {
      shape = 'irregular';
    }

    // Mono compatibility (0 = poor, 1 = excellent)
    const monoCompatibility = Math.min(1, verticalSpread / (horizontalSpread + verticalSpread));

    return { shape, monoCompatibility };
  }
}
