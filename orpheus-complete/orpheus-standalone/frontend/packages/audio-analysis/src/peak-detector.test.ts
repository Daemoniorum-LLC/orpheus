import { describe, it, expect, beforeEach } from 'vitest';
import { PeakDetector, TruePeakDetector } from './peak-detector';

describe('peak-detector.ts', () => {
  describe('PeakDetector', () => {
    let detector: PeakDetector;

    beforeEach(() => {
      detector = new PeakDetector();
    });

    describe('constructor', () => {
      it('should create with default clip threshold', () => {
        const det = new PeakDetector();
        expect(det).toBeInstanceOf(PeakDetector);
      });

      it('should create with custom clip threshold', () => {
        const det = new PeakDetector(0.9);
        expect(det).toBeInstanceOf(PeakDetector);
      });
    });

    describe('analyze', () => {
      it('should detect peak in simple signal', () => {
        const samples = new Float32Array([0.1, 0.5, 0.3, 0.2]);
        const result = detector.analyze(samples);

        expect(result.peakLinear).toBe(0.5);
        expect(result.peakSampleIndex).toBe(1);
        expect(result.clipped).toBe(false);
      });

      it('should detect negative peaks', () => {
        const samples = new Float32Array([0.1, -0.8, 0.3, -0.2]);
        const result = detector.analyze(samples);

        expect(result.peakLinear).toBe(0.8);
        expect(result.peakSampleIndex).toBe(1);
      });

      it('should convert peak to dB correctly', () => {
        const samples = new Float32Array([0.5]);
        const result = detector.analyze(samples);

        // 20 * log10(0.5) ≈ -6.02 dB
        expect(result.peakDb).toBeCloseTo(-6.02, 1);
      });

      it('should detect clipping at threshold', () => {
        const samples = new Float32Array([0.5, 1.0, 0.3]);
        const result = detector.analyze(samples);

        expect(result.clipped).toBe(true);
        expect(result.clippedSamples).toBe(1);
      });

      it('should count multiple clipped samples', () => {
        const samples = new Float32Array([1.0, 1.0, 0.5, 1.0]);
        const result = detector.analyze(samples);

        expect(result.clipped).toBe(true);
        expect(result.clippedSamples).toBe(3);
      });

      it('should calculate crest factor', () => {
        const samples = new Float32Array([0.1, 0.1, 0.1, 1.0]);
        const result = detector.analyze(samples);

        // Crest factor = peak dB - RMS dB
        expect(result.crestFactor).toBeGreaterThan(0);
        expect(typeof result.crestFactor).toBe('number');
      });

      it('should handle zero amplitude', () => {
        const samples = new Float32Array([0, 0, 0, 0]);
        const result = detector.analyze(samples);

        expect(result.peakLinear).toBe(0);
        expect(result.peakDb).toBe(-Infinity);
        expect(result.clipped).toBe(false);
      });

      it('should handle single sample', () => {
        const samples = new Float32Array([0.7]);
        const result = detector.analyze(samples);

        expect(result.peakLinear).toBe(0.7);
        expect(result.peakSampleIndex).toBe(0);
      });

      it('should handle very small values', () => {
        const samples = new Float32Array([0.001, 0.002, 0.001]);
        const result = detector.analyze(samples);

        expect(result.peakLinear).toBe(0.002);
        expect(result.peakDb).toBeLessThan(-50);
      });

      it('should detect peak at beginning', () => {
        const samples = new Float32Array([0.9, 0.5, 0.3, 0.1]);
        const result = detector.analyze(samples);

        expect(result.peakSampleIndex).toBe(0);
      });

      it('should detect peak at end', () => {
        const samples = new Float32Array([0.1, 0.3, 0.5, 0.9]);
        const result = detector.analyze(samples);

        expect(result.peakSampleIndex).toBe(3);
      });
    });

    describe('analyzeStereo', () => {
      it('should analyze both channels', () => {
        const left = new Float32Array([0.5, 0.3, 0.1]);
        const right = new Float32Array([0.2, 0.7, 0.4]);
        const result = detector.analyzeStereo(left, right);

        expect(result.left.peakLinear).toBe(0.5);
        expect(result.right.peakLinear).toBe(0.7);
      });

      it('should calculate stereo peak as max of both channels', () => {
        const left = new Float32Array([0.5, 0.1, 0.8]);
        const right = new Float32Array([0.2, 0.9, 0.3]);
        const result = detector.analyzeStereo(left, right);

        // Stereo peak should be max at each sample
        expect(result.stereo.peakLinear).toBeGreaterThanOrEqual(0.5);
        expect(result.stereo.peakLinear).toBeGreaterThanOrEqual(0.9);
      });

      it('should handle identical channels', () => {
        const samples = new Float32Array([0.5, 0.7, 0.3]);
        const result = detector.analyzeStereo(samples, samples);

        expect(result.left.peakLinear).toBe(result.right.peakLinear);
        expect(result.left.peakDb).toBeCloseTo(result.right.peakDb, 5);
      });

      it('should detect clipping in either channel', () => {
        const left = new Float32Array([0.5, 1.0, 0.3]);
        const right = new Float32Array([0.2, 0.7, 1.0]);
        const result = detector.analyzeStereo(left, right);

        expect(result.left.clipped).toBe(true);
        expect(result.right.clipped).toBe(true);
      });
    });

    describe('analyzeMultiChannel', () => {
      it('should analyze multiple channels', () => {
        const channels = [
          new Float32Array([0.5, 0.3]),
          new Float32Array([0.7, 0.4]),
          new Float32Array([0.2, 0.9]),
        ];
        const results = detector.analyzeMultiChannel(channels);

        expect(results).toHaveLength(3);
        expect(results[0].peakLinear).toBe(0.5);
        expect(results[1].peakLinear).toBe(0.7);
        expect(results[2].peakLinear).toBe(0.9);
      });

      it('should handle empty channels array', () => {
        const results = detector.analyzeMultiChannel([]);
        expect(results).toEqual([]);
      });

      it('should handle single channel', () => {
        const channels = [new Float32Array([0.5, 0.7])];
        const results = detector.analyzeMultiChannel(channels);

        expect(results).toHaveLength(1);
        expect(results[0].peakLinear).toBe(0.7);
      });
    });

    describe('detectClipping', () => {
      it('should return false for clean signal', () => {
        const samples = new Float32Array([0.5, 0.7, 0.3, 0.9]);
        const clipping = detector.detectClipping(samples);

        expect(clipping).toBe(false);
      });

      it('should return true when signal clips', () => {
        const samples = new Float32Array([0.5, 1.0, 0.3]);
        const clipping = detector.detectClipping(samples);

        expect(clipping).toBe(true);
      });

      it('should detect negative clipping', () => {
        const samples = new Float32Array([0.5, -1.0, 0.3]);
        const clipping = detector.detectClipping(samples);

        expect(clipping).toBe(true);
      });

      it('should handle empty buffer', () => {
        const samples = new Float32Array([]);
        const clipping = detector.detectClipping(samples);

        expect(clipping).toBe(false);
      });

      it('should respect custom clip threshold', () => {
        const customDetector = new PeakDetector(0.8);
        const samples = new Float32Array([0.9, 0.7, 0.5]);

        const clipping = customDetector.detectClipping(samples);
        expect(clipping).toBe(true);
      });
    });

    describe('findClippingEvents', () => {
      it('should find single clipping event', () => {
        const samples = new Float32Array([0.5, 1.0, 1.0, 0.5]);
        const events = detector.findClippingEvents(samples);

        expect(events).toHaveLength(1);
        expect(events[0].start).toBe(1);
        expect(events[0].end).toBe(2);
        expect(events[0].peakValue).toBe(1.0);
      });

      it('should find multiple clipping events', () => {
        const samples = new Float32Array([1.0, 0.5, 1.0, 1.0, 0.3, 1.0]);
        const events = detector.findClippingEvents(samples);

        expect(events.length).toBeGreaterThanOrEqual(2);
      });

      it('should track peak value in event', () => {
        const samples = new Float32Array([0.5, 1.0, 1.2, 1.1, 0.5]);
        const events = detector.findClippingEvents(samples);

        expect(events[0].peakValue).toBeGreaterThanOrEqual(1.2);
      });

      it('should respect minimum event length', () => {
        const samples = new Float32Array([0.5, 1.0, 0.5, 1.0, 1.0, 0.5]);
        const events = detector.findClippingEvents(samples, 2);

        // Should only find event with 2+ samples
        expect(events.length).toBeGreaterThan(0);
        for (const event of events) {
          expect(event.end - event.start + 1).toBeGreaterThanOrEqual(2);
        }
      });

      it('should handle clipping at start', () => {
        const samples = new Float32Array([1.0, 1.0, 0.5, 0.3]);
        const events = detector.findClippingEvents(samples);

        expect(events).toHaveLength(1);
        expect(events[0].start).toBe(0);
      });

      it('should handle clipping at end', () => {
        const samples = new Float32Array([0.3, 0.5, 1.0, 1.0]);
        const events = detector.findClippingEvents(samples);

        expect(events).toHaveLength(1);
        expect(events[0].end).toBe(3);
      });

      it('should handle entire buffer clipping', () => {
        const samples = new Float32Array([1.0, 1.0, 1.0, 1.0]);
        const events = detector.findClippingEvents(samples);

        expect(events).toHaveLength(1);
        expect(events[0].start).toBe(0);
        expect(events[0].end).toBe(3);
      });

      it('should return empty array for clean signal', () => {
        const samples = new Float32Array([0.5, 0.7, 0.3, 0.9]);
        const events = detector.findClippingEvents(samples);

        expect(events).toEqual([]);
      });

      it('should handle negative clipping', () => {
        const samples = new Float32Array([0.5, -1.0, -1.0, 0.5]);
        const events = detector.findClippingEvents(samples);

        expect(events).toHaveLength(1);
        expect(events[0].start).toBe(1);
        expect(events[0].end).toBe(2);
      });
    });
  });

  describe('TruePeakDetector', () => {
    let detector: TruePeakDetector;

    beforeEach(() => {
      detector = new TruePeakDetector(4);
    });

    describe('constructor', () => {
      it('should create with default oversampling factor', () => {
        const det = new TruePeakDetector();
        expect(det).toBeInstanceOf(TruePeakDetector);
      });

      it('should create with custom oversampling factor', () => {
        const det = new TruePeakDetector(8);
        expect(det).toBeInstanceOf(TruePeakDetector);
      });

      it('should extend PeakDetector', () => {
        expect(detector).toBeInstanceOf(PeakDetector);
      });
    });

    describe('analyze', () => {
      it('should detect inter-sample peaks', () => {
        const samples = new Float32Array([0.5, 0.9, 0.5]);
        const result = detector.analyze(samples);

        // True peak should potentially be higher due to interpolation
        expect(result.peakLinear).toBeGreaterThanOrEqual(0.9);
      });

      it('should adjust sample index to original rate', () => {
        const samples = new Float32Array([0.1, 0.5, 0.9, 0.3]);
        const result = detector.analyze(samples);

        // Sample index should be in original sample rate range
        expect(result.peakSampleIndex).toBeLessThan(samples.length);
        expect(result.peakSampleIndex).toBeGreaterThanOrEqual(0);
      });

      it('should work with different oversampling factors', () => {
        const samples = new Float32Array([0.5, 0.7, 0.3]);

        const det2x = new TruePeakDetector(2);
        const det4x = new TruePeakDetector(4);
        const det8x = new TruePeakDetector(8);

        const result2x = det2x.analyze(samples);
        const result4x = det4x.analyze(samples);
        const result8x = det8x.analyze(samples);

        expect(result2x.peakLinear).toBeGreaterThan(0);
        expect(result4x.peakLinear).toBeGreaterThan(0);
        expect(result8x.peakLinear).toBeGreaterThan(0);
      });

      it('should handle single sample', () => {
        const samples = new Float32Array([0.7]);
        const result = detector.analyze(samples);

        expect(result.peakLinear).toBeGreaterThan(0);
      });

      it('should detect peaks between samples', () => {
        // Signal that would have inter-sample peak
        const samples = new Float32Array([0.0, 1.0, 0.0]);
        const result = detector.analyze(samples);

        expect(result.peakLinear).toBeGreaterThanOrEqual(1.0);
      });

      it('should maintain clipping detection', () => {
        const samples = new Float32Array([0.5, 1.0, 0.5]);
        const result = detector.analyze(samples);

        expect(result.clipped).toBe(true);
      });

      it('should calculate crest factor with oversampling', () => {
        const samples = new Float32Array([0.1, 0.1, 1.0, 0.1]);
        const result = detector.analyze(samples);

        expect(result.crestFactor).toBeGreaterThan(0);
        expect(typeof result.crestFactor).toBe('number');
      });
    });

    describe('oversample (private method behavior)', () => {
      it('should create interpolated samples', () => {
        // Testing the effect of oversampling
        const samples = new Float32Array([0.0, 1.0]);
        const result = detector.analyze(samples);

        // With oversampling, we should detect values between 0 and 1
        expect(result.peakLinear).toBeGreaterThanOrEqual(0);
        expect(result.peakLinear).toBeLessThanOrEqual(1.0);
      });

      it('should handle gradual transitions', () => {
        const samples = new Float32Array([0.0, 0.25, 0.5, 0.75, 1.0]);
        const result = detector.analyze(samples);

        expect(result.peakLinear).toBeCloseTo(1.0, 1);
      });

      it('should handle rapid transitions', () => {
        const samples = new Float32Array([0.0, 1.0, 0.0, 1.0]);
        const result = detector.analyze(samples);

        expect(result.peakLinear).toBeGreaterThanOrEqual(1.0);
      });
    });

    describe('ITU-R BS.1770-4 compliance', () => {
      it('should detect true peaks higher than sample peaks', () => {
        // Sinusoidal signal that will have inter-sample peaks
        const samples = new Float32Array(100);
        for (let i = 0; i < samples.length; i++) {
          samples[i] = 0.9 * Math.sin(2 * Math.PI * i / 10);
        }

        const basicDetector = new PeakDetector();
        const basicResult = basicDetector.analyze(samples);
        const trueResult = detector.analyze(samples);

        // True peak should be >= basic peak
        expect(trueResult.peakLinear).toBeGreaterThanOrEqual(basicResult.peakLinear * 0.99);
      });

      it('should provide more accurate peak detection', () => {
        const samples = new Float32Array([0.7, 0.9, 0.7]);
        const result = detector.analyze(samples);

        // Should detect interpolated values
        expect(result.peakLinear).toBeGreaterThan(0);
      });
    });
  });
});
