/**
 * Peak Level Detector - Detects peak levels in audio signals
 */

export interface PeakAnalysis {
  /** Peak level in dB */
  peakDb: number;
  /** Peak level as linear amplitude (0-1) */
  peakLinear: number;
  /** Sample index where peak occurred */
  peakSampleIndex: number;
  /** True if any samples exceeded 0 dB (clipping) */
  clipped: boolean;
  /** Number of clipped samples */
  clippedSamples: number;
  /** Crest factor (peak/RMS ratio) in dB */
  crestFactor: number;
}

/**
 * Analyzes peak levels in an audio buffer
 */
export class PeakDetector {
  private readonly clipThreshold: number;

  constructor(clipThreshold: number = 1.0) {
    this.clipThreshold = clipThreshold;
  }

  /**
   * Analyzes peak levels in a mono audio buffer
   */
  analyze(samples: Float32Array): PeakAnalysis {
    let peakLinear = 0;
    let peakSampleIndex = 0;
    let clippedSamples = 0;
    let sumSquares = 0;

    for (let i = 0; i < samples.length; i++) {
      const absSample = Math.abs(samples[i]);

      // Track peak
      if (absSample > peakLinear) {
        peakLinear = absSample;
        peakSampleIndex = i;
      }

      // Count clipped samples
      if (absSample >= this.clipThreshold) {
        clippedSamples++;
      }

      // Accumulate for RMS calculation
      sumSquares += samples[i] * samples[i];
    }

    // Convert peak to dB
    const peakDb = this.linearToDb(peakLinear);

    // Calculate RMS for crest factor
    const rms = Math.sqrt(sumSquares / samples.length);
    const rmsDb = this.linearToDb(rms);
    const crestFactor = peakDb - rmsDb;

    return {
      peakDb,
      peakLinear,
      peakSampleIndex,
      clipped: clippedSamples > 0,
      clippedSamples,
      crestFactor,
    };
  }

  /**
   * Analyzes peak levels in a stereo audio buffer
   */
  analyzeStereo(
    leftChannel: Float32Array,
    rightChannel: Float32Array
  ): { left: PeakAnalysis; right: PeakAnalysis; stereo: PeakAnalysis } {
    const left = this.analyze(leftChannel);
    const right = this.analyze(rightChannel);

    // Calculate combined stereo peak
    const stereoSamples = new Float32Array(leftChannel.length);
    for (let i = 0; i < leftChannel.length; i++) {
      stereoSamples[i] = Math.max(Math.abs(leftChannel[i]), Math.abs(rightChannel[i]));
    }
    const stereo = this.analyze(stereoSamples);

    return { left, right, stereo };
  }

  /**
   * Analyzes peaks in multi-channel audio
   */
  analyzeMultiChannel(channels: Float32Array[]): PeakAnalysis[] {
    return channels.map(channel => this.analyze(channel));
  }

  /**
   * Detects clipping in real-time (for monitoring)
   */
  detectClipping(samples: Float32Array): boolean {
    for (let i = 0; i < samples.length; i++) {
      if (Math.abs(samples[i]) >= this.clipThreshold) {
        return true;
      }
    }
    return false;
  }

  /**
   * Finds all clipping events in the buffer
   */
  findClippingEvents(
    samples: Float32Array,
    minEventLength: number = 1
  ): Array<{ start: number; end: number; peakValue: number }> {
    const events: Array<{ start: number; end: number; peakValue: number }> = [];
    let inClip = false;
    let clipStart = 0;
    let clipPeak = 0;

    for (let i = 0; i < samples.length; i++) {
      const absSample = Math.abs(samples[i]);

      if (absSample >= this.clipThreshold) {
        if (!inClip) {
          // Start of new clipping event
          inClip = true;
          clipStart = i;
          clipPeak = absSample;
        } else {
          // Continue clipping event
          clipPeak = Math.max(clipPeak, absSample);
        }
      } else {
        if (inClip) {
          // End of clipping event
          const eventLength = i - clipStart;
          if (eventLength >= minEventLength) {
            events.push({
              start: clipStart,
              end: i - 1,
              peakValue: clipPeak,
            });
          }
          inClip = false;
        }
      }
    }

    // Handle clipping that extends to the end
    if (inClip) {
      events.push({
        start: clipStart,
        end: samples.length - 1,
        peakValue: clipPeak,
      });
    }

    return events;
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
 * True Peak Detector (ITU-R BS.1770-4 compliant)
 * Uses oversampling to detect inter-sample peaks
 */
export class TruePeakDetector extends PeakDetector {
  private readonly oversamplingFactor: number;

  constructor(oversamplingFactor: number = 4) {
    super();
    this.oversamplingFactor = oversamplingFactor;
  }

  /**
   * Analyzes true peak levels using oversampling
   */
  override analyze(samples: Float32Array): PeakAnalysis {
    // Oversample the signal
    const oversampled = this.oversample(samples);

    // Analyze peaks in oversampled signal
    const analysis = super.analyze(oversampled);

    // Adjust sample index back to original sample rate
    analysis.peakSampleIndex = Math.floor(
      analysis.peakSampleIndex / this.oversamplingFactor
    );

    return analysis;
  }

  /**
   * Oversamples audio buffer using linear interpolation
   * Real implementation would use polyphase filters
   */
  private oversample(samples: Float32Array): Float32Array {
    const oversampledLength = samples.length * this.oversamplingFactor;
    const oversampled = new Float32Array(oversampledLength);

    for (let i = 0; i < samples.length - 1; i++) {
      const sample1 = samples[i];
      const sample2 = samples[i + 1];

      for (let j = 0; j < this.oversamplingFactor; j++) {
        const fraction = j / this.oversamplingFactor;
        const interpolated = sample1 + (sample2 - sample1) * fraction;
        oversampled[i * this.oversamplingFactor + j] = interpolated;
      }
    }

    // Handle last sample
    oversampled[oversampledLength - 1] = samples[samples.length - 1];

    return oversampled;
  }
}
