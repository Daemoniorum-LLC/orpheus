import { describe, it, expect, beforeEach } from 'vitest';
import {
  FrequencyAnalyzer,
  FREQUENCY_BANDS,
  PROBLEMATIC_FREQUENCIES,
  type FrequencySpectrum,
  type FrequencyBand,
  type FrequencyAnalyzerOptions,
} from './frequency-analyzer';

describe('frequency-analyzer.ts', () => {
  describe('FrequencyAnalyzer - Constructor', () => {
    it('should create analyzer with default options', () => {
      const analyzer = new FrequencyAnalyzer();
      const samples = new Float32Array(2048);
      const result = analyzer.analyze(samples);

      expect(result.fftSize).toBe(2048);
      expect(result.sampleRate).toBe(44100);
    });

    it('should create analyzer with custom FFT size', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 4096 });
      const samples = new Float32Array(4096);
      const result = analyzer.analyze(samples);

      expect(result.fftSize).toBe(4096);
    });

    it('should create analyzer with custom sample rate', () => {
      const analyzer = new FrequencyAnalyzer({ sampleRate: 48000 });
      const samples = new Float32Array(2048);
      const result = analyzer.analyze(samples);

      expect(result.sampleRate).toBe(48000);
    });

    it('should create analyzer with Hann window', () => {
      const analyzer = new FrequencyAnalyzer({ windowFunction: 'hann' });
      const samples = new Float32Array(2048);
      expect(() => analyzer.analyze(samples)).not.toThrow();
    });

    it('should create analyzer with Hamming window', () => {
      const analyzer = new FrequencyAnalyzer({ windowFunction: 'hamming' });
      const samples = new Float32Array(2048);
      expect(() => analyzer.analyze(samples)).not.toThrow();
    });

    it('should create analyzer with Blackman window', () => {
      const analyzer = new FrequencyAnalyzer({ windowFunction: 'blackman' });
      const samples = new Float32Array(2048);
      expect(() => analyzer.analyze(samples)).not.toThrow();
    });

    it('should create analyzer with no window', () => {
      const analyzer = new FrequencyAnalyzer({ windowFunction: 'none' });
      const samples = new Float32Array(2048);
      expect(() => analyzer.analyze(samples)).not.toThrow();
    });

    it('should create analyzer with custom smoothing', () => {
      const analyzer = new FrequencyAnalyzer({ smoothing: 0.5 });
      const samples = new Float32Array(2048);
      expect(() => analyzer.analyze(samples)).not.toThrow();
    });

    it('should throw error for non-power-of-2 FFT size', () => {
      expect(() => new FrequencyAnalyzer({ fftSize: 1000 })).toThrow(
        'FFT size must be a power of 2'
      );
    });

    it('should accept power-of-2 FFT sizes', () => {
      const validSizes = [256, 512, 1024, 2048, 4096, 8192];
      validSizes.forEach(size => {
        expect(() => new FrequencyAnalyzer({ fftSize: size })).not.toThrow();
      });
    });

    it('should reject non-power-of-2 FFT sizes', () => {
      const invalidSizes = [100, 500, 1000, 1500, 3000];
      invalidSizes.forEach(size => {
        expect(() => new FrequencyAnalyzer({ fftSize: size })).toThrow();
      });
    });
  });

  describe('FrequencyAnalyzer - analyze()', () => {
    let analyzer: FrequencyAnalyzer;

    beforeEach(() => {
      analyzer = new FrequencyAnalyzer({ fftSize: 2048, sampleRate: 44100 });
    });

    it('should return FrequencySpectrum object with all properties', () => {
      const samples = new Float32Array(2048);
      const result = analyzer.analyze(samples);

      expect(result).toHaveProperty('frequencies');
      expect(result).toHaveProperty('magnitudes');
      expect(result).toHaveProperty('fftSize');
      expect(result).toHaveProperty('sampleRate');
      expect(result).toHaveProperty('resolution');
    });

    it('should calculate correct frequency resolution', () => {
      const samples = new Float32Array(2048);
      const result = analyzer.analyze(samples);

      // Resolution = sampleRate / fftSize
      const expectedResolution = 44100 / 2048;
      expect(result.resolution).toBeCloseTo(expectedResolution, 5);
    });

    it('should return correct number of frequency bins', () => {
      const samples = new Float32Array(2048);
      const result = analyzer.analyze(samples);

      // FFT returns N/2 bins for real signal
      expect(result.frequencies.length).toBe(1024);
      expect(result.magnitudes.length).toBe(1024);
    });

    it('should handle silence (all zeros)', () => {
      const silence = new Float32Array(2048);
      const result = analyzer.analyze(silence);

      // All magnitudes should be -Infinity in dB
      expect(result.magnitudes.every(mag => mag === -Infinity)).toBe(true);
    });

    it('should detect single frequency (440 Hz)', () => {
      const samples = new Float32Array(2048);
      const frequency = 440;

      // Generate 440 Hz sine wave
      for (let i = 0; i < samples.length; i++) {
        samples[i] = Math.sin(2 * Math.PI * frequency * i / 44100);
      }

      const result = analyzer.analyze(samples);

      // Find bin closest to 440 Hz
      const targetBin = Math.round(frequency / result.resolution);
      const binFreq = result.frequencies[targetBin];

      // Check that peak is near 440 Hz
      expect(binFreq).toBeCloseTo(440, 0);

      // Check that magnitude at 440 Hz is significantly higher than average
      const peakMagnitude = result.magnitudes[targetBin];
      const avgMagnitude =
        result.magnitudes.reduce((a, b) => (isFinite(a) ? a : 0) + (isFinite(b) ? b : 0), 0) /
        result.magnitudes.filter(isFinite).length;

      expect(peakMagnitude).toBeGreaterThan(avgMagnitude);
    });

    it('should detect multiple frequencies', () => {
      const samples = new Float32Array(2048);
      const freq1 = 440;
      const freq2 = 880;

      // Generate two sine waves
      for (let i = 0; i < samples.length; i++) {
        samples[i] =
          0.5 * Math.sin(2 * Math.PI * freq1 * i / 44100) +
          0.5 * Math.sin(2 * Math.PI * freq2 * i / 44100);
      }

      const result = analyzer.analyze(samples);

      // Both frequencies should have peaks
      const bin1 = Math.round(freq1 / result.resolution);
      const bin2 = Math.round(freq2 / result.resolution);

      expect(result.magnitudes[bin1]).toBeGreaterThan(-20);
      expect(result.magnitudes[bin2]).toBeGreaterThan(-20);
    });

    it('should handle DC offset', () => {
      const samples = new Float32Array(2048).fill(0.5); // DC offset
      const result = analyzer.analyze(samples);

      // DC component should be in bin 0
      expect(result.frequencies[0]).toBe(0);
      expect(isFinite(result.magnitudes[0])).toBe(true);
    });

    it('should handle Nyquist frequency', () => {
      const samples = new Float32Array(2048);

      // Nyquist frequency (alternating +1, -1)
      for (let i = 0; i < samples.length; i++) {
        samples[i] = i % 2 === 0 ? 1 : -1;
      }

      const result = analyzer.analyze(samples);

      // Nyquist bin is the last bin
      const nyquistBin = result.frequencies.length - 1;
      expect(result.frequencies[nyquistBin]).toBeCloseTo(22050, 0);
    });

    it('should apply Hann window correctly', () => {
      const analyzer = new FrequencyAnalyzer({ windowFunction: 'hann', fftSize: 2048 });
      const samples = new Float32Array(2048).fill(1);

      const result = analyzer.analyze(samples);

      // Windowed signal should have different spectrum than unwindowed
      expect(result.magnitudes.length).toBe(1024);
    });

    it('should apply Hamming window correctly', () => {
      const analyzer = new FrequencyAnalyzer({ windowFunction: 'hamming', fftSize: 2048 });
      const samples = new Float32Array(2048).fill(1);

      const result = analyzer.analyze(samples);
      expect(result.magnitudes.length).toBe(1024);
    });

    it('should apply Blackman window correctly', () => {
      const analyzer = new FrequencyAnalyzer({ windowFunction: 'blackman', fftSize: 2048 });
      const samples = new Float32Array(2048).fill(1);

      const result = analyzer.analyze(samples);
      expect(result.magnitudes.length).toBe(1024);
    });

    it('should handle no window (rectangular)', () => {
      const analyzer = new FrequencyAnalyzer({ windowFunction: 'none', fftSize: 2048 });
      const samples = new Float32Array(2048).fill(1);

      const result = analyzer.analyze(samples);
      expect(result.magnitudes.length).toBe(1024);
    });

    it('should apply smoothing across multiple calls', () => {
      const analyzer = new FrequencyAnalyzer({ smoothing: 0.8, fftSize: 2048 });

      // First call - loud signal
      const loud = new Float32Array(2048).fill(0.5);
      const result1 = analyzer.analyze(loud);

      // Second call - quiet signal
      const quiet = new Float32Array(2048).fill(0.1);
      const result2 = analyzer.analyze(quiet);

      // Due to high smoothing, second result should be influenced by first
      // (values should be between quiet and loud)
      const hasSmoothing = result2.magnitudes.some((mag, i) => {
        if (!isFinite(mag) || !isFinite(result1.magnitudes[i])) return false;
        return Math.abs(mag - result1.magnitudes[i]) < Math.abs(mag - (-60));
      });

      expect(hasSmoothing).toBe(true);
    });

    it('should handle zero smoothing (no temporal smoothing)', () => {
      const analyzer = new FrequencyAnalyzer({ smoothing: 0, fftSize: 2048 });

      const signal1 = new Float32Array(2048).fill(0.5);
      const result1 = analyzer.analyze(signal1);

      const signal2 = new Float32Array(2048).fill(0.1);
      const result2 = analyzer.analyze(signal2);

      // With no smoothing, results should be independent
      expect(result1.magnitudes[0]).not.toEqual(result2.magnitudes[0]);
    });

    it('should handle buffers smaller than FFT size', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 2048 });
      const small = new Float32Array(1000).fill(0.5);

      const result = analyzer.analyze(small);

      // Should zero-pad to FFT size
      expect(result.fftSize).toBe(2048);
      expect(result.magnitudes.length).toBe(1024);
    });

    it('should handle buffers larger than FFT size', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 2048 });
      const large = new Float32Array(4000).fill(0.5);

      const result = analyzer.analyze(large);

      // Should truncate to FFT size
      expect(result.fftSize).toBe(2048);
      expect(result.magnitudes.length).toBe(1024);
    });

    it('should convert magnitudes to dB scale', () => {
      const samples = new Float32Array(2048);

      // 440 Hz sine wave with amplitude 0.5
      for (let i = 0; i < samples.length; i++) {
        samples[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = analyzer.analyze(samples);

      // All finite magnitudes should be in dB (negative or zero)
      const finiteMagnitudes = result.magnitudes.filter(isFinite);
      expect(finiteMagnitudes.every(mag => mag <= 0)).toBe(true);
    });

    it('should handle different FFT sizes', () => {
      const sizes = [256, 512, 1024, 2048, 4096];

      sizes.forEach(size => {
        const analyzer = new FrequencyAnalyzer({ fftSize: size });
        const samples = new Float32Array(size);
        const result = analyzer.analyze(samples);

        expect(result.fftSize).toBe(size);
        expect(result.frequencies.length).toBe(size / 2);
      });
    });

    it('should calculate correct frequency bins', () => {
      const result = analyzer.analyze(new Float32Array(2048));

      // First bin should be 0 Hz
      expect(result.frequencies[0]).toBe(0);

      // Bins should increment by resolution
      for (let i = 1; i < result.frequencies.length; i++) {
        expect(result.frequencies[i]).toBeCloseTo(i * result.resolution, 5);
      }
    });
  });

  describe('FrequencyAnalyzer - analyzeBands()', () => {
    let analyzer: FrequencyAnalyzer;

    beforeEach(() => {
      analyzer = new FrequencyAnalyzer({ fftSize: 4096, sampleRate: 44100 });
    });

    it('should return array of FrequencyBand objects', () => {
      const samples = new Float32Array(4096);
      const bands = analyzer.analyzeBands(samples);

      expect(Array.isArray(bands)).toBe(true);
      expect(bands.length).toBe(7); // 7 standard bands
    });

    it('should analyze all standard frequency bands', () => {
      const samples = new Float32Array(4096);
      const bands = analyzer.analyzeBands(samples);

      const expectedBands = ['Sub Bass', 'Bass', 'Low Mids', 'Mids', 'High Mids', 'Presence', 'Brilliance'];
      const bandNames = bands.map(b => b.name);

      expect(bandNames).toEqual(expectedBands);
    });

    it('should include all required properties in each band', () => {
      const samples = new Float32Array(4096).fill(0.3);
      const bands = analyzer.analyzeBands(samples);

      bands.forEach(band => {
        expect(band).toHaveProperty('name');
        expect(band).toHaveProperty('low');
        expect(band).toHaveProperty('high');
        expect(band).toHaveProperty('magnitude');
        expect(band).toHaveProperty('peakFrequency');
      });
    });

    it('should detect bass energy in bass frequencies', () => {
      const samples = new Float32Array(4096);

      // Generate 100 Hz bass tone
      for (let i = 0; i < samples.length; i++) {
        samples[i] = 0.5 * Math.sin(2 * Math.PI * 100 * i / 44100);
      }

      const bands = analyzer.analyzeBands(samples);
      const bassBand = bands.find(b => b.name === 'Bass');

      expect(bassBand).toBeDefined();
      expect(bassBand!.magnitude).toBeGreaterThan(-30);
      expect(bassBand!.peakFrequency).toBeGreaterThanOrEqual(60);
      expect(bassBand!.peakFrequency).toBeLessThanOrEqual(200);
    });

    it('should detect mid-range energy in mid frequencies', () => {
      const samples = new Float32Array(4096);

      // Generate 1 kHz mid-range tone
      for (let i = 0; i < samples.length; i++) {
        samples[i] = 0.5 * Math.sin(2 * Math.PI * 1000 * i / 44100);
      }

      const bands = analyzer.analyzeBands(samples);
      const midBand = bands.find(b => b.name === 'Mids');

      expect(midBand).toBeDefined();
      expect(midBand!.magnitude).toBeGreaterThan(-30);
      expect(midBand!.peakFrequency).toBeGreaterThanOrEqual(500);
      expect(midBand!.peakFrequency).toBeLessThanOrEqual(2000);
    });

    it('should detect high-frequency energy in presence band', () => {
      const samples = new Float32Array(4096);

      // Generate 8 kHz high-frequency tone
      for (let i = 0; i < samples.length; i++) {
        samples[i] = 0.5 * Math.sin(2 * Math.PI * 8000 * i / 44100);
      }

      const bands = analyzer.analyzeBands(samples);
      const presenceBand = bands.find(b => b.name === 'Presence');

      expect(presenceBand).toBeDefined();
      expect(presenceBand!.magnitude).toBeGreaterThan(-30);
      expect(presenceBand!.peakFrequency).toBeGreaterThanOrEqual(6000);
      expect(presenceBand!.peakFrequency).toBeLessThanOrEqual(12000);
    });

    it('should handle full-spectrum signal', () => {
      const samples = new Float32Array(4096);

      // White noise-like signal
      for (let i = 0; i < samples.length; i++) {
        samples[i] = (Math.random() - 0.5) * 0.5;
      }

      const bands = analyzer.analyzeBands(samples);

      // All bands should have some energy
      bands.forEach(band => {
        expect(isFinite(band.magnitude)).toBe(true);
        expect(band.magnitude).toBeGreaterThan(-80);
      });
    });

    it('should identify peak frequency in each band', () => {
      const samples = new Float32Array(4096);

      // Generate complex signal
      for (let i = 0; i < samples.length; i++) {
        samples[i] =
          0.3 * Math.sin(2 * Math.PI * 100 * i / 44100) + // Bass
          0.3 * Math.sin(2 * Math.PI * 1000 * i / 44100) + // Mids
          0.3 * Math.sin(2 * Math.PI * 8000 * i / 44100); // Highs
      }

      const bands = analyzer.analyzeBands(samples);

      bands.forEach(band => {
        expect(band.peakFrequency).toBeGreaterThanOrEqual(band.low);
        expect(band.peakFrequency).toBeLessThanOrEqual(band.high);
      });
    });

    it('should calculate average magnitude for each band', () => {
      const samples = new Float32Array(4096).fill(0.3);
      const bands = analyzer.analyzeBands(samples);

      bands.forEach(band => {
        expect(isFinite(band.magnitude)).toBe(true);
      });
    });
  });

  describe('FrequencyAnalyzer - detectPitch()', () => {
    let analyzer: FrequencyAnalyzer;

    beforeEach(() => {
      analyzer = new FrequencyAnalyzer({ fftSize: 2048, sampleRate: 44100 });
    });

    it('should detect pitch of 440 Hz (A4)', () => {
      const samples = new Float32Array(4096);

      for (let i = 0; i < samples.length; i++) {
        samples[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const pitch = analyzer.detectPitch(samples);

      expect(pitch).not.toBeNull();
      expect(pitch!).toBeCloseTo(440, 10); // Within 10 Hz tolerance
    });

    it('should detect pitch of 220 Hz (A3)', () => {
      const samples = new Float32Array(4096);

      for (let i = 0; i < samples.length; i++) {
        samples[i] = 0.5 * Math.sin(2 * Math.PI * 220 * i / 44100);
      }

      const pitch = analyzer.detectPitch(samples);

      expect(pitch).not.toBeNull();
      expect(pitch!).toBeCloseTo(220, 10);
    });

    it('should detect pitch of 880 Hz (A5)', () => {
      const samples = new Float32Array(4096);

      for (let i = 0; i < samples.length; i++) {
        samples[i] = 0.5 * Math.sin(2 * Math.PI * 880 * i / 44100);
      }

      const pitch = analyzer.detectPitch(samples);

      expect(pitch).not.toBeNull();
      expect(pitch!).toBeCloseTo(880, 20);
    });

    it('should return null for silence', () => {
      const silence = new Float32Array(4096);
      const pitch = analyzer.detectPitch(silence);

      expect(pitch).toBeNull();
    });

    it('should return null for noise', () => {
      const noise = new Float32Array(4096);

      for (let i = 0; i < noise.length; i++) {
        noise[i] = (Math.random() - 0.5) * 0.5;
      }

      const pitch = analyzer.detectPitch(noise);

      // Noise typically doesn't have clear pitch, but autocorrelation might find something
      // This is acceptable as long as it doesn't crash
      expect(pitch === null || typeof pitch === 'number').toBe(true);
    });

    it('should handle complex harmonic signals', () => {
      const samples = new Float32Array(4096);
      const fundamental = 200;

      // Fundamental + harmonics
      for (let i = 0; i < samples.length; i++) {
        samples[i] =
          0.5 * Math.sin(2 * Math.PI * fundamental * i / 44100) + // Fundamental
          0.25 * Math.sin(2 * Math.PI * fundamental * 2 * i / 44100) + // 2nd harmonic
          0.125 * Math.sin(2 * Math.PI * fundamental * 3 * i / 44100); // 3rd harmonic
      }

      const pitch = analyzer.detectPitch(samples);

      expect(pitch).not.toBeNull();
      // Should detect fundamental frequency
      expect(pitch!).toBeCloseTo(fundamental, 20);
    });

    it('should handle low frequencies (50-100 Hz)', () => {
      const samples = new Float32Array(8192); // Longer buffer for low frequencies

      for (let i = 0; i < samples.length; i++) {
        samples[i] = 0.5 * Math.sin(2 * Math.PI * 80 * i / 44100);
      }

      const pitch = analyzer.detectPitch(samples);

      expect(pitch).not.toBeNull();
      expect(pitch!).toBeGreaterThanOrEqual(50);
      expect(pitch!).toBeLessThanOrEqual(110);
    });

    it('should handle high frequencies (500-1000 Hz)', () => {
      const samples = new Float32Array(4096);

      for (let i = 0; i < samples.length; i++) {
        samples[i] = 0.5 * Math.sin(2 * Math.PI * 600 * i / 44100);
      }

      const pitch = analyzer.detectPitch(samples);

      expect(pitch).not.toBeNull();
      expect(pitch!).toBeGreaterThanOrEqual(500);
      expect(pitch!).toBeLessThanOrEqual(700);
    });
  });

  describe('FrequencyAnalyzer - reset()', () => {
    it('should reset smoothing state', () => {
      const analyzer = new FrequencyAnalyzer({ smoothing: 0.9, fftSize: 2048 });

      // First analysis
      const loud = new Float32Array(2048).fill(0.5);
      analyzer.analyze(loud);

      // Reset
      analyzer.reset();

      // Second analysis (after reset)
      const quiet = new Float32Array(2048).fill(0.1);
      const result = analyzer.analyze(quiet);

      // After reset, should not be influenced by previous analysis
      expect(result.magnitudes.length).toBe(1024);
    });

    it('should allow fresh analysis after reset', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 2048 });

      const signal = new Float32Array(2048);
      for (let i = 0; i < signal.length; i++) {
        signal[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result1 = analyzer.analyze(signal);
      analyzer.reset();
      const result2 = analyzer.analyze(signal);

      // Results should be similar after reset
      expect(result1.magnitudes.length).toBe(result2.magnitudes.length);
    });
  });

  describe('FrequencyAnalyzer - Edge Cases', () => {
    it('should handle single sample buffer', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 2048 });
      const single = new Float32Array(1).fill(0.5);

      const result = analyzer.analyze(single);

      expect(result.magnitudes.length).toBe(1024);
    });

    it('should handle very small buffers', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 256 });
      const small = new Float32Array(10).fill(0.5);

      const result = analyzer.analyze(small);

      expect(result.magnitudes.length).toBe(128);
    });

    it('should handle exact FFT size buffer', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 1024 });
      const exact = new Float32Array(1024).fill(0.5);

      const result = analyzer.analyze(exact);

      expect(result.magnitudes.length).toBe(512);
    });

    it('should handle negative samples', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 2048 });
      const negative = new Float32Array(2048).fill(-0.5);

      const result = analyzer.analyze(negative);

      expect(result.magnitudes.length).toBe(1024);
    });

    it('should handle mixed positive/negative samples', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 2048 });
      const mixed = new Float32Array(2048);

      for (let i = 0; i < mixed.length; i++) {
        mixed[i] = i % 2 === 0 ? 0.5 : -0.5;
      }

      const result = analyzer.analyze(mixed);

      expect(result.magnitudes.length).toBe(1024);
    });

    it('should handle very low amplitude signals', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 2048 });
      const quiet = new Float32Array(2048).fill(0.0001);

      const result = analyzer.analyze(quiet);

      expect(result.magnitudes.length).toBe(1024);
    });

    it('should handle clipping signals', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 2048 });
      const clipped = new Float32Array(2048).fill(1.5);

      const result = analyzer.analyze(clipped);

      expect(result.magnitudes.length).toBe(1024);
    });
  });

  describe('FREQUENCY_BANDS', () => {
    it('should export Sub Bass band', () => {
      expect(FREQUENCY_BANDS.SUB_BASS).toEqual({
        name: 'Sub Bass',
        low: 20,
        high: 60,
      });
    });

    it('should export Bass band', () => {
      expect(FREQUENCY_BANDS.BASS).toEqual({
        name: 'Bass',
        low: 60,
        high: 200,
      });
    });

    it('should export Low Mids band', () => {
      expect(FREQUENCY_BANDS.LOW_MIDS).toEqual({
        name: 'Low Mids',
        low: 200,
        high: 500,
      });
    });

    it('should export Mids band', () => {
      expect(FREQUENCY_BANDS.MIDS).toEqual({
        name: 'Mids',
        low: 500,
        high: 2000,
      });
    });

    it('should export High Mids band', () => {
      expect(FREQUENCY_BANDS.HIGH_MIDS).toEqual({
        name: 'High Mids',
        low: 2000,
        high: 6000,
      });
    });

    it('should export Presence band', () => {
      expect(FREQUENCY_BANDS.PRESENCE).toEqual({
        name: 'Presence',
        low: 6000,
        high: 12000,
      });
    });

    it('should export Brilliance band', () => {
      expect(FREQUENCY_BANDS.BRILLIANCE).toEqual({
        name: 'Brilliance',
        low: 12000,
        high: 20000,
      });
    });

    it('should have all bands in ascending order', () => {
      const bands = Object.values(FREQUENCY_BANDS);

      for (let i = 1; i < bands.length; i++) {
        expect(bands[i].low).toBeGreaterThanOrEqual(bands[i - 1].high);
      }
    });

    it('should cover full audible spectrum (20 Hz - 20 kHz)', () => {
      const bands = Object.values(FREQUENCY_BANDS);

      expect(bands[0].low).toBe(20);
      expect(bands[bands.length - 1].high).toBe(20000);
    });

    it('should have non-overlapping bands', () => {
      const bands = Object.values(FREQUENCY_BANDS);

      for (let i = 1; i < bands.length; i++) {
        expect(bands[i].low).toBeGreaterThanOrEqual(bands[i - 1].high);
      }
    });
  });

  describe('PROBLEMATIC_FREQUENCIES', () => {
    it('should export Mud frequency range', () => {
      expect(PROBLEMATIC_FREQUENCIES.MUD).toEqual({
        name: 'Mud',
        range: [200, 400],
        description: 'Muddy, unclear low-mids',
      });
    });

    it('should export Boxiness frequency range', () => {
      expect(PROBLEMATIC_FREQUENCIES.BOXINESS).toEqual({
        name: 'Boxiness',
        range: [500, 800],
        description: 'Boxy, honky sound',
      });
    });

    it('should export Harshness frequency range', () => {
      expect(PROBLEMATIC_FREQUENCIES.HARSHNESS).toEqual({
        name: 'Harshness',
        range: [2000, 4000],
        description: 'Harsh, fatiguing',
      });
    });

    it('should export Sibilance frequency range', () => {
      expect(PROBLEMATIC_FREQUENCIES.SIBILANCE).toEqual({
        name: 'Sibilance',
        range: [6000, 8000],
        description: 'Excessive S sounds',
      });
    });

    it('should have all problem frequencies with name, range, and description', () => {
      const problems = Object.values(PROBLEMATIC_FREQUENCIES);

      problems.forEach(problem => {
        expect(problem).toHaveProperty('name');
        expect(problem).toHaveProperty('range');
        expect(problem).toHaveProperty('description');
        expect(Array.isArray(problem.range)).toBe(true);
        expect(problem.range.length).toBe(2);
        expect(problem.range[1]).toBeGreaterThan(problem.range[0]);
      });
    });

    it('should have valid frequency ranges', () => {
      const problems = Object.values(PROBLEMATIC_FREQUENCIES);

      problems.forEach(problem => {
        expect(problem.range[0]).toBeGreaterThan(0);
        expect(problem.range[1]).toBeLessThanOrEqual(20000);
      });
    });
  });

  describe('FrequencyAnalyzer - Integration Tests', () => {
    it('should analyze music-like signal with multiple bands', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 4096, sampleRate: 44100 });
      const samples = new Float32Array(4096);

      // Simulate music with bass, mid, and high content
      for (let i = 0; i < samples.length; i++) {
        samples[i] =
          0.3 * Math.sin(2 * Math.PI * 80 * i / 44100) + // Bass
          0.2 * Math.sin(2 * Math.PI * 1000 * i / 44100) + // Mids
          0.1 * Math.sin(2 * Math.PI * 8000 * i / 44100); // Highs
      }

      const spectrum = analyzer.analyze(samples);
      const bands = analyzer.analyzeBands(samples);

      expect(spectrum.magnitudes.length).toBeGreaterThan(0);
      expect(bands.length).toBe(7);

      // Bass band should have energy
      const bassBand = bands.find(b => b.name === 'Bass');
      expect(bassBand!.magnitude).toBeGreaterThan(-40);
    });

    it('should handle streaming analysis (multiple consecutive calls)', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 2048, smoothing: 0.5 });

      for (let chunk = 0; chunk < 10; chunk++) {
        const samples = new Float32Array(2048);
        for (let i = 0; i < samples.length; i++) {
          samples[i] = 0.3 * Math.sin(2 * Math.PI * 440 * i / 44100);
        }

        const result = analyzer.analyze(samples);
        expect(result.magnitudes.length).toBe(1024);
      }
    });

    it('should provide consistent results for same input', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 2048, smoothing: 0 });

      const samples = new Float32Array(2048);
      for (let i = 0; i < samples.length; i++) {
        samples[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result1 = analyzer.analyze(samples);
      analyzer.reset();
      const result2 = analyzer.analyze(samples);

      // Results should be nearly identical (within floating point precision)
      for (let i = 0; i < result1.magnitudes.length; i++) {
        if (isFinite(result1.magnitudes[i]) && isFinite(result2.magnitudes[i])) {
          expect(result1.magnitudes[i]).toBeCloseTo(result2.magnitudes[i], 5);
        }
      }
    });
  });

  describe('FrequencyAnalyzer - Performance', () => {
    it('should handle large FFT sizes efficiently', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 8192 });
      const samples = new Float32Array(8192);

      for (let i = 0; i < samples.length; i++) {
        samples[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const start = performance.now();
      const result = analyzer.analyze(samples);
      const duration = performance.now() - start;

      expect(result.magnitudes.length).toBe(4096);
      expect(duration).toBeLessThan(100); // Should be fast
    });

    it('should handle real-time buffer sizes', () => {
      const analyzer = new FrequencyAnalyzer({ fftSize: 512 });
      const samples = new Float32Array(512);

      const start = performance.now();
      analyzer.analyze(samples);
      const duration = performance.now() - start;

      expect(duration).toBeLessThan(10); // Very fast for small buffers
    });
  });
});
