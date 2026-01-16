/**
 * LUFS Loudness Meter - ITU-R BS.1770-4 compliant loudness measurement
 * Used for broadcast and streaming platform loudness standards
 */

export interface LUFSAnalysis {
  /** Integrated loudness (LUFS) - overall program loudness */
  integrated: number;
  /** Loudness range (LU) - dynamic variation */
  range: number;
  /** Short-term loudness (LUFS) - 3 second window */
  shortTerm: number;
  /** Momentary loudness (LUFS) - 400ms window */
  momentary: number;
  /** True peak level (dBTP) */
  truePeak: number;
  /** Compliant with target loudness */
  compliant: boolean;
  /** Target loudness for comparison */
  target: number;
}

export interface LUFSMeterOptions {
  /** Sample rate */
  sampleRate?: number;
  /** Target loudness in LUFS (e.g., -14 for Spotify, -16 for Apple Music) */
  targetLoudness?: number;
  /** Tolerance in LU for compliance */
  tolerance?: number;
  /** Number of channels (1=mono, 2=stereo, 6=5.1) */
  channels?: number;
}

/**
 * K-weighting filter coefficients (ITU-R BS.1770-4)
 * Pre-filter: High-shelf filter at 1681 Hz
 * RLB filter: High-pass filter at 38 Hz
 */
class KWeightingFilter {
  // Pre-filter (high-shelf)
  private preB0: number = 1.53512485958697;
  private preB1: number = -2.69169618940638;
  private preB2: number = 1.19839281085285;
  private preA1: number = -1.69065929318241;
  private preA2: number = 0.73248077421585;

  // RLB filter (high-pass)
  private rlbB0: number = 1.0;
  private rlbB1: number = -2.0;
  private rlbB2: number = 1.0;
  private rlbA1: number = -1.99004745483398;
  private rlbA2: number = 0.99007225036621;

  // State variables for pre-filter
  private preX1: number = 0;
  private preX2: number = 0;
  private preY1: number = 0;
  private preY2: number = 0;

  // State variables for RLB filter
  private rlbX1: number = 0;
  private rlbX2: number = 0;
  private rlbY1: number = 0;
  private rlbY2: number = 0;

  /**
   * Processes a single sample through K-weighting filters
   */
  process(input: number): number {
    // Pre-filter (high-shelf at 1681 Hz)
    const preOutput =
      this.preB0 * input +
      this.preB1 * this.preX1 +
      this.preB2 * this.preX2 -
      this.preA1 * this.preY1 -
      this.preA2 * this.preY2;

    this.preX2 = this.preX1;
    this.preX1 = input;
    this.preY2 = this.preY1;
    this.preY1 = preOutput;

    // RLB filter (high-pass at 38 Hz)
    const rlbOutput =
      this.rlbB0 * preOutput +
      this.rlbB1 * this.rlbX1 +
      this.rlbB2 * this.rlbX2 -
      this.rlbA1 * this.rlbY1 -
      this.rlbA2 * this.rlbY2;

    this.rlbX2 = this.rlbX1;
    this.rlbX1 = preOutput;
    this.rlbY2 = this.rlbY1;
    this.rlbY1 = rlbOutput;

    return rlbOutput;
  }

  /**
   * Resets filter state
   */
  reset(): void {
    this.preX1 = this.preX2 = this.preY1 = this.preY2 = 0;
    this.rlbX1 = this.rlbX2 = this.rlbY1 = this.rlbY2 = 0;
  }
}

/**
 * LUFS Meter implementation
 */
export class LUFSMeter {
  private readonly sampleRate: number;
  private readonly targetLoudness: number;
  private readonly tolerance: number;
  private readonly channels: number;

  // K-weighting filters (one per channel)
  private filters: KWeightingFilter[] = [];

  // Gating blocks for integrated loudness
  private gatingBlocks: number[] = [];
  private readonly blockDuration: number = 0.4; // 400ms blocks
  private blockSize: number;

  // State for short-term and momentary
  private shortTermWindow: number[] = []; // 3 seconds
  private momentaryWindow: number[] = []; // 400ms

  constructor(options: LUFSMeterOptions = {}) {
    this.sampleRate = options.sampleRate || 44100;
    this.targetLoudness = options.targetLoudness || -14; // Spotify default
    this.tolerance = options.tolerance || 1.0; // ±1 LU
    this.channels = options.channels || 2; // Stereo default

    this.blockSize = Math.round(this.blockDuration * this.sampleRate);

    // Initialize filters for each channel
    for (let i = 0; i < this.channels; i++) {
      this.filters.push(new KWeightingFilter());
    }
  }

  /**
   * Analyzes LUFS loudness for audio buffer
   */
  analyze(channels: Float32Array[]): LUFSAnalysis {
    if (channels.length !== this.channels) {
      throw new Error(
        `Expected ${this.channels} channels, got ${channels.length}`
      );
    }

    // Apply K-weighting to all channels
    const filtered = channels.map((channel, i) => this.applyKWeighting(channel, i));

    // Calculate integrated loudness
    const integrated = this.calculateIntegrated(filtered);

    // Calculate loudness range
    const range = this.calculateRange(filtered);

    // Calculate short-term (3 seconds)
    const shortTerm = this.calculateShortTerm(filtered);

    // Calculate momentary (400ms)
    const momentary = this.calculateMomentary(filtered);

    // Calculate true peak
    const truePeak = this.calculateTruePeak(channels);

    // Check compliance
    const compliant = Math.abs(integrated - this.targetLoudness) <= this.tolerance;

    return {
      integrated,
      range,
      shortTerm,
      momentary,
      truePeak,
      compliant,
      target: this.targetLoudness,
    };
  }

  /**
   * Applies K-weighting filter to a channel
   */
  private applyKWeighting(samples: Float32Array, channelIndex: number): Float32Array {
    const filtered = new Float32Array(samples.length);
    const filter = this.filters[channelIndex];

    for (let i = 0; i < samples.length; i++) {
      filtered[i] = filter.process(samples[i]);
    }

    return filtered;
  }

  /**
   * Calculates integrated loudness using gating
   */
  private calculateIntegrated(channels: Float32Array[]): number {
    const blockPowers: number[] = [];
    const numBlocks = Math.floor(channels[0].length / this.blockSize);

    // Calculate mean square power for each block
    for (let block = 0; block < numBlocks; block++) {
      const start = block * this.blockSize;
      const end = start + this.blockSize;

      let sumPower = 0;
      for (const channel of channels) {
        for (let i = start; i < end && i < channel.length; i++) {
          sumPower += channel[i] * channel[i];
        }
      }

      const meanPower = sumPower / (this.blockSize * channels.length);
      blockPowers.push(meanPower);
    }

    // Apply absolute gating (-70 LUFS)
    const absoluteThreshold = Math.pow(10, -70 / 10);
    const gatedBlocks = blockPowers.filter(power => power >= absoluteThreshold);

    if (gatedBlocks.length === 0) {
      return -Infinity; // Silence
    }

    // Calculate mean of gated blocks
    const meanGatedPower = gatedBlocks.reduce((a, b) => a + b, 0) / gatedBlocks.length;

    // Apply relative gating (-10 LU below mean)
    const relativeThreshold = meanGatedPower / Math.pow(10, 10 / 10);
    const relativeGatedBlocks = blockPowers.filter(power => power >= relativeThreshold);

    if (relativeGatedBlocks.length === 0) {
      return -Infinity;
    }

    const finalMeanPower =
      relativeGatedBlocks.reduce((a, b) => a + b, 0) / relativeGatedBlocks.length;

    // Convert to LUFS
    return -0.691 + 10 * Math.log10(finalMeanPower);
  }

  /**
   * Calculates loudness range (LRA)
   */
  private calculateRange(channels: Float32Array[]): number {
    const blockPowers: number[] = [];
    const numBlocks = Math.floor(channels[0].length / this.blockSize);

    for (let block = 0; block < numBlocks; block++) {
      const start = block * this.blockSize;
      const end = start + this.blockSize;

      let sumPower = 0;
      for (const channel of channels) {
        for (let i = start; i < end && i < channel.length; i++) {
          sumPower += channel[i] * channel[i];
        }
      }

      const meanPower = sumPower / (this.blockSize * channels.length);
      blockPowers.push(meanPower);
    }

    // Apply gating
    const absoluteThreshold = Math.pow(10, -70 / 10);
    const gatedBlocks = blockPowers.filter(power => power >= absoluteThreshold);

    if (gatedBlocks.length < 2) {
      return 0;
    }

    // Sort and calculate range (difference between 10th and 95th percentile)
    const sorted = gatedBlocks.sort((a, b) => a - b);
    const lowIndex = Math.floor(sorted.length * 0.1);
    const highIndex = Math.floor(sorted.length * 0.95);

    const lowLoudness = -0.691 + 10 * Math.log10(sorted[lowIndex]);
    const highLoudness = -0.691 + 10 * Math.log10(sorted[highIndex]);

    return highLoudness - lowLoudness;
  }

  /**
   * Calculates short-term loudness (3 seconds)
   */
  private calculateShortTerm(channels: Float32Array[]): number {
    const windowSize = Math.round(3.0 * this.sampleRate);
    const startIndex = Math.max(0, channels[0].length - windowSize);

    let sumPower = 0;
    for (const channel of channels) {
      for (let i = startIndex; i < channel.length; i++) {
        sumPower += channel[i] * channel[i];
      }
    }

    const meanPower = sumPower / (windowSize * channels.length);

    if (meanPower === 0) return -Infinity;

    return -0.691 + 10 * Math.log10(meanPower);
  }

  /**
   * Calculates momentary loudness (400ms)
   */
  private calculateMomentary(channels: Float32Array[]): number {
    const windowSize = this.blockSize;
    const startIndex = Math.max(0, channels[0].length - windowSize);

    let sumPower = 0;
    for (const channel of channels) {
      for (let i = startIndex; i < channel.length; i++) {
        sumPower += channel[i] * channel[i];
      }
    }

    const meanPower = sumPower / (windowSize * channels.length);

    if (meanPower === 0) return -Infinity;

    return -0.691 + 10 * Math.log10(meanPower);
  }

  /**
   * Calculates true peak (dBTP) using oversampling
   */
  private calculateTruePeak(channels: Float32Array[]): number {
    let maxPeak = 0;

    for (const channel of channels) {
      // Simple oversampling (4x) for true peak detection
      // Real implementation should use polyphase filters
      for (let i = 0; i < channel.length - 1; i++) {
        const sample1 = channel[i];
        const sample2 = channel[i + 1];

        // Check 4 interpolated points
        for (let j = 0; j < 4; j++) {
          const fraction = j / 4;
          const interpolated = sample1 + (sample2 - sample1) * fraction;
          maxPeak = Math.max(maxPeak, Math.abs(interpolated));
        }
      }
    }

    if (maxPeak === 0) return -Infinity;

    return 20 * Math.log10(maxPeak);
  }

  /**
   * Resets the meter state
   */
  reset(): void {
    this.filters.forEach(filter => filter.reset());
    this.gatingBlocks = [];
    this.shortTermWindow = [];
    this.momentaryWindow = [];
  }
}

/**
 * Platform-specific loudness targets
 */
export const LOUDNESS_TARGETS = {
  SPOTIFY: -14, // LUFS
  APPLE_MUSIC: -16, // LUFS
  YOUTUBE: -14, // LUFS
  TIDAL: -14, // LUFS
  AMAZON_MUSIC: -14, // LUFS
  DEEZER: -15, // LUFS
  SOUNDCLOUD: -14, // LUFS (recommended)
  BROADCAST_TV: -23, // LUFS (EBU R128)
  BROADCAST_RADIO: -23, // LUFS (EBU R128)
  CINEMA: -24, // LUFS (SMPTE)
  CD_MASTERING: -9, // LUFS (typical, not standardized)
  PODCAST: -16, // LUFS (recommended)
} as const;

/**
 * Normalizes audio to target LUFS
 */
export function normalizeLUFS(
  channels: Float32Array[],
  targetLUFS: number,
  sampleRate: number = 44100
): Float32Array[] {
  const meter = new LUFSMeter({ sampleRate, channels: channels.length });
  const analysis = meter.analyze(channels);

  if (!isFinite(analysis.integrated)) {
    return channels; // Can't normalize silence
  }

  // Calculate gain adjustment
  const gainDb = targetLUFS - analysis.integrated;
  const gainLinear = Math.pow(10, gainDb / 20);

  // Apply gain to all channels
  return channels.map(channel => {
    const normalized = new Float32Array(channel.length);
    for (let i = 0; i < channel.length; i++) {
      normalized[i] = channel[i] * gainLinear;
    }
    return normalized;
  });
}
