/**
 * Frequency Analyzer - FFT-based frequency analysis for spectrum visualization and analysis
 */

export interface FrequencySpectrum {
  /** Frequency bins (Hz) */
  frequencies: number[];
  /** Magnitude values (dB) */
  magnitudes: number[];
  /** FFT size used */
  fftSize: number;
  /** Sample rate */
  sampleRate: number;
  /** Frequency resolution (Hz per bin) */
  resolution: number;
}

export interface FrequencyBand {
  /** Band name */
  name: string;
  /** Lower frequency bound (Hz) */
  low: number;
  /** Upper frequency bound (Hz) */
  high: number;
  /** Average magnitude in band (dB) */
  magnitude: number;
  /** Peak frequency in band (Hz) */
  peakFrequency: number;
}

export interface FrequencyAnalyzerOptions {
  /** FFT size (must be power of 2) */
  fftSize?: number;
  /** Sample rate */
  sampleRate?: number;
  /** Window function type */
  windowFunction?: 'hann' | 'hamming' | 'blackman' | 'none';
  /** Smoothing factor for spectrum (0-1) */
  smoothing?: number;
}

/**
 * Frequency analyzer using FFT
 */
export class FrequencyAnalyzer {
  private readonly fftSize: number;
  private readonly sampleRate: number;
  private readonly windowFunction: 'hann' | 'hamming' | 'blackman' | 'none';
  private readonly smoothing: number;

  // Previous spectrum for smoothing
  private previousSpectrum: number[] | null = null;

  constructor(options: FrequencyAnalyzerOptions = {}) {
    this.fftSize = options.fftSize || 2048;
    this.sampleRate = options.sampleRate || 44100;
    this.windowFunction = options.windowFunction || 'hann';
    this.smoothing = options.smoothing || 0.8;

    // Validate FFT size is power of 2
    if ((this.fftSize & (this.fftSize - 1)) !== 0) {
      throw new Error('FFT size must be a power of 2');
    }
  }

  /**
   * Analyzes frequency spectrum of audio buffer
   */
  analyze(samples: Float32Array): FrequencySpectrum {
    // Apply window function
    const windowed = this.applyWindow(samples);

    // Perform FFT
    const fftResult = this.fft(windowed);

    // Calculate magnitude spectrum
    const magnitudes = this.calculateMagnitude(fftResult);

    // Apply smoothing
    const smoothed = this.applySmoothing(magnitudes);

    // Convert to dB
    const magnitudesDb = smoothed.map(mag => this.linearToDb(mag));

    // Calculate frequency bins
    const resolution = this.sampleRate / this.fftSize;
    const frequencies = Array.from(
      { length: magnitudesDb.length },
      (_, i) => i * resolution
    );

    return {
      frequencies,
      magnitudes: magnitudesDb,
      fftSize: this.fftSize,
      sampleRate: this.sampleRate,
      resolution,
    };
  }

  /**
   * Analyzes frequency spectrum and groups into bands
   */
  analyzeBands(samples: Float32Array): FrequencyBand[] {
    const spectrum = this.analyze(samples);

    return [
      this.analyzeBand('Sub Bass', 20, 60, spectrum),
      this.analyzeBand('Bass', 60, 200, spectrum),
      this.analyzeBand('Low Mids', 200, 500, spectrum),
      this.analyzeBand('Mids', 500, 2000, spectrum),
      this.analyzeBand('High Mids', 2000, 6000, spectrum),
      this.analyzeBand('Presence', 6000, 12000, spectrum),
      this.analyzeBand('Brilliance', 12000, 20000, spectrum),
    ];
  }

  /**
   * Analyzes a specific frequency band
   */
  private analyzeBand(
    name: string,
    low: number,
    high: number,
    spectrum: FrequencySpectrum
  ): FrequencyBand {
    const startBin = Math.floor(low / spectrum.resolution);
    const endBin = Math.ceil(high / spectrum.resolution);

    let sum = 0;
    let peakMagnitude = -Infinity;
    let peakFrequency = low;

    for (let i = startBin; i < endBin && i < spectrum.magnitudes.length; i++) {
      const magnitude = spectrum.magnitudes[i];
      sum += magnitude;

      if (magnitude > peakMagnitude) {
        peakMagnitude = magnitude;
        peakFrequency = spectrum.frequencies[i];
      }
    }

    const magnitude = sum / (endBin - startBin);

    return {
      name,
      low,
      high,
      magnitude,
      peakFrequency,
    };
  }

  /**
   * Detects fundamental frequency (pitch detection)
   */
  detectPitch(samples: Float32Array): number | null {
    // Use autocorrelation for pitch detection
    const autocorr = this.autocorrelation(samples);

    // Find first peak in autocorrelation
    let maxCorr = 0;
    let maxLag = 0;

    const minPeriod = Math.floor(this.sampleRate / 1000); // 1000 Hz max
    const maxPeriod = Math.floor(this.sampleRate / 50); // 50 Hz min

    for (let lag = minPeriod; lag < maxPeriod && lag < autocorr.length; lag++) {
      if (autocorr[lag] > maxCorr) {
        maxCorr = autocorr[lag];
        maxLag = lag;
      }
    }

    if (maxLag === 0) return null;

    return this.sampleRate / maxLag;
  }

  /**
   * Applies window function to samples
   */
  private applyWindow(samples: Float32Array): Float32Array {
    const windowed = new Float32Array(this.fftSize);
    const length = Math.min(samples.length, this.fftSize);

    for (let i = 0; i < length; i++) {
      windowed[i] = samples[i] * this.getWindowValue(i, this.fftSize);
    }

    return windowed;
  }

  /**
   * Gets window function value at index
   */
  private getWindowValue(index: number, size: number): number {
    switch (this.windowFunction) {
      case 'hann':
        return 0.5 * (1 - Math.cos((2 * Math.PI * index) / (size - 1)));

      case 'hamming':
        return 0.54 - 0.46 * Math.cos((2 * Math.PI * index) / (size - 1));

      case 'blackman':
        const a0 = 0.42;
        const a1 = 0.5;
        const a2 = 0.08;
        return (
          a0 -
          a1 * Math.cos((2 * Math.PI * index) / (size - 1)) +
          a2 * Math.cos((4 * Math.PI * index) / (size - 1))
        );

      case 'none':
      default:
        return 1.0;
    }
  }

  /**
   * Performs FFT (simplified Cooley-Tukey algorithm)
   * Returns complex numbers as [real, imag, real, imag, ...]
   */
  private fft(samples: Float32Array): Float32Array {
    const N = this.fftSize;
    const result = new Float32Array(N * 2);

    // Copy input to real parts
    for (let i = 0; i < N; i++) {
      result[i * 2] = samples[i];
      result[i * 2 + 1] = 0; // Imaginary part
    }

    // Bit-reversal permutation
    let j = 0;
    for (let i = 0; i < N - 1; i++) {
      if (i < j) {
        // Swap
        [result[i * 2], result[j * 2]] = [result[j * 2], result[i * 2]];
        [result[i * 2 + 1], result[j * 2 + 1]] = [result[j * 2 + 1], result[i * 2 + 1]];
      }

      let k = N / 2;
      while (k <= j) {
        j -= k;
        k /= 2;
      }
      j += k;
    }

    // Cooley-Tukey FFT
    for (let size = 2; size <= N; size *= 2) {
      const halfSize = size / 2;
      const step = N / size;

      for (let i = 0; i < N; i += size) {
        for (let j = 0; j < halfSize; j++) {
          const k = j * step;
          const thetaReal = Math.cos((-2 * Math.PI * k) / N);
          const thetaImag = Math.sin((-2 * Math.PI * k) / N);

          const evenIdx = (i + j) * 2;
          const oddIdx = (i + j + halfSize) * 2;

          const oddReal = result[oddIdx];
          const oddImag = result[oddIdx + 1];

          const tReal = oddReal * thetaReal - oddImag * thetaImag;
          const tImag = oddReal * thetaImag + oddImag * thetaReal;

          result[oddIdx] = result[evenIdx] - tReal;
          result[oddIdx + 1] = result[evenIdx + 1] - tImag;

          result[evenIdx] += tReal;
          result[evenIdx + 1] += tImag;
        }
      }
    }

    return result;
  }

  /**
   * Calculates magnitude spectrum from complex FFT result
   */
  private calculateMagnitude(fftResult: Float32Array): number[] {
    const magnitudes: number[] = [];
    const halfSize = this.fftSize / 2;

    for (let i = 0; i < halfSize; i++) {
      const real = fftResult[i * 2];
      const imag = fftResult[i * 2 + 1];
      const magnitude = Math.sqrt(real * real + imag * imag) / halfSize;
      magnitudes.push(magnitude);
    }

    return magnitudes;
  }

  /**
   * Applies temporal smoothing to spectrum
   */
  private applySmoothing(magnitudes: number[]): number[] {
    if (!this.previousSpectrum || this.smoothing === 0) {
      this.previousSpectrum = magnitudes;
      return magnitudes;
    }

    const smoothed = magnitudes.map((mag, i) => {
      return this.smoothing * this.previousSpectrum![i] + (1 - this.smoothing) * mag;
    });

    this.previousSpectrum = smoothed;
    return smoothed;
  }

  /**
   * Calculates autocorrelation for pitch detection
   */
  private autocorrelation(samples: Float32Array): Float32Array {
    const N = samples.length;
    const result = new Float32Array(N);

    for (let lag = 0; lag < N; lag++) {
      let sum = 0;
      for (let i = 0; i < N - lag; i++) {
        sum += samples[i] * samples[i + lag];
      }
      result[lag] = sum / (N - lag);
    }

    return result;
  }

  /**
   * Converts linear magnitude to dB
   */
  private linearToDb(linear: number): number {
    if (linear === 0) return -Infinity;
    return 20 * Math.log10(linear);
  }

  /**
   * Resets smoothing state
   */
  reset(): void {
    this.previousSpectrum = null;
  }
}

/**
 * Standard frequency bands for audio analysis
 */
export const FREQUENCY_BANDS = {
  SUB_BASS: { name: 'Sub Bass', low: 20, high: 60 },
  BASS: { name: 'Bass', low: 60, high: 200 },
  LOW_MIDS: { name: 'Low Mids', low: 200, high: 500 },
  MIDS: { name: 'Mids', low: 500, high: 2000 },
  HIGH_MIDS: { name: 'High Mids', low: 2000, high: 6000 },
  PRESENCE: { name: 'Presence', low: 6000, high: 12000 },
  BRILLIANCE: { name: 'Brilliance', low: 12000, high: 20000 },
} as const;

/**
 * Problematic frequency ranges (common EQ issues)
 */
export const PROBLEMATIC_FREQUENCIES = {
  MUD: { name: 'Mud', range: [200, 400], description: 'Muddy, unclear low-mids' },
  BOXINESS: { name: 'Boxiness', range: [500, 800], description: 'Boxy, honky sound' },
  HARSHNESS: { name: 'Harshness', range: [2000, 4000], description: 'Harsh, fatiguing' },
  SIBILANCE: { name: 'Sibilance', range: [6000, 8000], description: 'Excessive S sounds' },
} as const;
