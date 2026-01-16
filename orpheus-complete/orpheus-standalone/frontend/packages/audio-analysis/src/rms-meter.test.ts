import { describe, it, expect, beforeEach } from 'vitest';
import { RMSMeter, VUMeter } from './rms-meter';

describe('rms-meter.ts', () => {
  describe('RMSMeter', () => {
    let meter: RMSMeter;

    beforeEach(() => {
      meter = new RMSMeter();
    });

    describe('constructor', () => {
      it('should create with default options', () => {
        const m = new RMSMeter();
        expect(m).toBeInstanceOf(RMSMeter);
      });

      it('should create with custom window size', () => {
        const m = new RMSMeter({ windowSize: 2048 });
        expect(m).toBeInstanceOf(RMSMeter);
      });

      it('should create with custom hop size', () => {
        const m = new RMSMeter({ hopSize: 512 });
        expect(m).toBeInstanceOf(RMSMeter);
      });

      it('should create with time-weighted enabled', () => {
        const m = new RMSMeter({ timeWeighted: true });
        expect(m).toBeInstanceOf(RMSMeter);
      });

      it('should create with custom time constant', () => {
        const m = new RMSMeter({ timeConstant: 0.5 });
        expect(m).toBeInstanceOf(RMSMeter);
      });

      it('should create with custom sample rate', () => {
        const m = new RMSMeter({ sampleRate: 48000 });
        expect(m).toBeInstanceOf(RMSMeter);
      });
    });

    describe('analyze', () => {
      it('should calculate RMS for constant signal', () => {
        const samples = new Float32Array([0.5, 0.5, 0.5, 0.5]);
        const result = meter.analyze(samples);

        expect(result.rmsLinear).toBeCloseTo(0.5, 5);
      });

      it('should calculate RMS for varying signal', () => {
        const samples = new Float32Array([0.1, 0.5, 0.3, 0.7]);
        const result = meter.analyze(samples);

        // RMS = sqrt(mean of squares)
        const expected = Math.sqrt((0.01 + 0.25 + 0.09 + 0.49) / 4);
        expect(result.rmsLinear).toBeCloseTo(expected, 5);
      });

      it('should convert RMS to dB correctly', () => {
        const samples = new Float32Array([0.5, 0.5, 0.5, 0.5]);
        const result = meter.analyze(samples);

        // 20 * log10(0.5) ≈ -6.02 dB
        expect(result.rmsDb).toBeCloseTo(-6.02, 1);
      });

      it('should calculate peak level', () => {
        const samples = new Float32Array([0.1, 0.9, 0.3, 0.5]);
        const result = meter.analyze(samples);

        expect(result.peakDb).toBeCloseTo(20 * Math.log10(0.9), 5);
      });

      it('should calculate dynamic range', () => {
        const samples = new Float32Array([0.1, 0.1, 0.1, 1.0]);
        const result = meter.analyze(samples);

        expect(result.dynamicRange).toBeGreaterThan(0);
        expect(result.dynamicRange).toBe(result.peakDb - result.rmsDb);
      });

      it('should handle zero amplitude', () => {
        const samples = new Float32Array([0, 0, 0, 0]);
        const result = meter.analyze(samples);

        expect(result.rmsLinear).toBe(0);
        expect(result.rmsDb).toBe(-Infinity);
        expect(result.peakDb).toBe(-Infinity);
      });

      it('should handle single sample', () => {
        const samples = new Float32Array([0.7]);
        const result = meter.analyze(samples);

        expect(result.rmsLinear).toBe(0.7);
        expect(result.dynamicRange).toBe(0);
      });

      it('should handle negative values correctly', () => {
        const samples = new Float32Array([-0.5, 0.5, -0.3, 0.3]);
        const result = meter.analyze(samples);

        // RMS should be positive (uses squares)
        expect(result.rmsLinear).toBeGreaterThan(0);
      });

      it('should calculate RMS for sinusoidal signal', () => {
        const samples = new Float32Array(1000);
        const amplitude = 0.7;
        for (let i = 0; i < samples.length; i++) {
          samples[i] = amplitude * Math.sin(2 * Math.PI * i / 100);
        }
        const result = meter.analyze(samples);

        // RMS of sine wave = amplitude / sqrt(2)
        const expectedRMS = amplitude / Math.sqrt(2);
        expect(result.rmsLinear).toBeCloseTo(expectedRMS, 2);
      });
    });

    describe('analyzeOverTime', () => {
      it('should return array of time-stamped measurements', () => {
        const samples = new Float32Array(10000).fill(0.5);
        const m = new RMSMeter({ windowSize: 1024, hopSize: 512, sampleRate: 44100 });
        const results = m.analyzeOverTime(samples);

        expect(results.length).toBeGreaterThan(0);
        expect(results[0]).toHaveProperty('time');
        expect(results[0]).toHaveProperty('rmsDb');
        expect(results[0]).toHaveProperty('rmsLinear');
      });

      it('should calculate time stamps correctly', () => {
        const m = new RMSMeter({ windowSize: 1024, hopSize: 1024, sampleRate: 44100 });
        const samples = new Float32Array(5000).fill(0.5);
        const results = m.analyzeOverTime(samples);

        // Time should increment by hopSize / sampleRate
        if (results.length > 1) {
          const timeDiff = results[1].time - results[0].time;
          expect(timeDiff).toBeCloseTo(1024 / 44100, 3);
        }
      });

      it('should handle varying signal over time', () => {
        const samples = new Float32Array(10000);
        // First half loud, second half quiet
        for (let i = 0; i < 5000; i++) samples[i] = 0.8;
        for (let i = 5000; i < 10000; i++) samples[i] = 0.2;

        const m = new RMSMeter({ windowSize: 1024, hopSize: 1024 });
        const results = m.analyzeOverTime(samples);

        // Early measurements should be higher than later ones
        const earlyRMS = results[0].rmsLinear;
        const lateRMS = results[results.length - 1].rmsLinear;
        expect(earlyRMS).toBeGreaterThan(lateRMS);
      });

      it('should respect window size', () => {
        const samples = new Float32Array(5000).fill(0.5);
        const m = new RMSMeter({ windowSize: 2048, hopSize: 1024 });
        const results = m.analyzeOverTime(samples);

        expect(results.length).toBeGreaterThan(0);
      });

      it('should respect hop size', () => {
        const samples = new Float32Array(10000).fill(0.5);

        const m1 = new RMSMeter({ windowSize: 1024, hopSize: 512 });
        const m2 = new RMSMeter({ windowSize: 1024, hopSize: 1024 });

        const results1 = m1.analyzeOverTime(samples);
        const results2 = m2.analyzeOverTime(samples);

        // Smaller hop size = more measurements
        expect(results1.length).toBeGreaterThan(results2.length);
      });

      it('should handle small buffers', () => {
        const samples = new Float32Array(100).fill(0.5);
        const m = new RMSMeter({ windowSize: 512 });
        const results = m.analyzeOverTime(samples);

        // Might be empty if buffer smaller than window
        expect(Array.isArray(results)).toBe(true);
      });
    });

    describe('analyzeTimeWeighted', () => {
      it('should calculate time-weighted RMS', () => {
        const samples = new Float32Array(10000).fill(0.5);
        const m = new RMSMeter({ timeWeighted: true, sampleRate: 44100 });
        const result = m.analyzeTimeWeighted(samples);

        expect(result.rmsLinear).toBeGreaterThan(0);
        expect(result.rmsDb).toBeGreaterThan(-Infinity);
      });

      it('should handle transient signals', () => {
        const samples = new Float32Array(5000);
        // Impulse at start
        samples[0] = 1.0;
        for (let i = 1; i < samples.length; i++) {
          samples[i] = 0.1;
        }

        const m = new RMSMeter({ timeConstant: 0.1, sampleRate: 44100 });
        const result = m.analyzeTimeWeighted(samples);

        expect(result.rmsLinear).toBeGreaterThan(0);
      });

      it('should respect time constant', () => {
        const samples = new Float32Array(10000).fill(0.5);

        const fast = new RMSMeter({ timeConstant: 0.05, sampleRate: 44100 });
        const slow = new RMSMeter({ timeConstant: 0.5, sampleRate: 44100 });

        const fastResult = fast.analyzeTimeWeighted(samples);
        const slowResult = slow.analyzeTimeWeighted(samples);

        // Both should work, potentially with different results
        expect(fastResult.rmsLinear).toBeGreaterThan(0);
        expect(slowResult.rmsLinear).toBeGreaterThan(0);
      });

      it('should calculate peak and dynamic range', () => {
        const samples = new Float32Array([0.1, 0.5, 0.9, 0.3]);
        const result = meter.analyzeTimeWeighted(samples);

        expect(result.peakDb).toBeDefined();
        expect(result.dynamicRange).toBeDefined();
        expect(result.dynamicRange).toBeGreaterThanOrEqual(0);
      });

      it('should handle varying sample rates', () => {
        const samples = new Float32Array(5000).fill(0.5);

        const m1 = new RMSMeter({ sampleRate: 44100 });
        const m2 = new RMSMeter({ sampleRate: 48000 });

        const result1 = m1.analyzeTimeWeighted(samples);
        const result2 = m2.analyzeTimeWeighted(samples);

        expect(result1.rmsLinear).toBeGreaterThan(0);
        expect(result2.rmsLinear).toBeGreaterThan(0);
      });
    });

    describe('analyzeStereo', () => {
      it('should analyze both channels separately', () => {
        const left = new Float32Array([0.5, 0.5, 0.5]);
        const right = new Float32Array([0.3, 0.3, 0.3]);
        const result = meter.analyzeStereo(left, right);

        expect(result.left.rmsLinear).toBeCloseTo(0.5, 5);
        expect(result.right.rmsLinear).toBeCloseTo(0.3, 5);
      });

      it('should calculate combined stereo RMS', () => {
        const left = new Float32Array([0.5, 0.5, 0.5]);
        const right = new Float32Array([0.5, 0.5, 0.5]);
        const result = meter.analyzeStereo(left, right);

        // Stereo is average of both channels
        expect(result.stereo.rmsLinear).toBeCloseTo(0.5, 5);
      });

      it('should handle different levels in each channel', () => {
        const left = new Float32Array([0.8, 0.8, 0.8]);
        const right = new Float32Array([0.2, 0.2, 0.2]);
        const result = meter.analyzeStereo(left, right);

        expect(result.left.rmsLinear).toBeGreaterThan(result.right.rmsLinear);
        expect(result.stereo.rmsLinear).toBeGreaterThan(0);
      });

      it('should handle mono-in-stereo (identical channels)', () => {
        const samples = new Float32Array([0.7, 0.7, 0.7]);
        const result = meter.analyzeStereo(samples, samples);

        expect(result.left.rmsLinear).toBeCloseTo(result.right.rmsLinear, 10);
        expect(result.stereo.rmsLinear).toBeCloseTo(0.7, 5);
      });

      it('should calculate dynamic range for all channels', () => {
        const left = new Float32Array([0.1, 0.5, 0.9]);
        const right = new Float32Array([0.2, 0.6, 0.8]);
        const result = meter.analyzeStereo(left, right);

        expect(result.left.dynamicRange).toBeGreaterThan(0);
        expect(result.right.dynamicRange).toBeGreaterThan(0);
        expect(result.stereo.dynamicRange).toBeGreaterThan(0);
      });
    });

    describe('edge cases', () => {
      it('should handle very large buffers', () => {
        const samples = new Float32Array(100000).fill(0.5);
        const result = meter.analyze(samples);

        expect(result.rmsLinear).toBeCloseTo(0.5, 5);
      });

      it('should handle alternating positive/negative', () => {
        const samples = new Float32Array(100);
        for (let i = 0; i < samples.length; i++) {
          samples[i] = i % 2 === 0 ? 0.5 : -0.5;
        }
        const result = meter.analyze(samples);

        expect(result.rmsLinear).toBeCloseTo(0.5, 5);
      });

      it('should handle very quiet signals', () => {
        const samples = new Float32Array([0.001, 0.002, 0.001]);
        const result = meter.analyze(samples);

        expect(result.rmsDb).toBeLessThan(-50);
        expect(result.rmsLinear).toBeGreaterThan(0);
      });
    });
  });

  describe('VUMeter', () => {
    let vuMeter: VUMeter;

    beforeEach(() => {
      vuMeter = new VUMeter(44100, 0.3, 0.3);
    });

    describe('constructor', () => {
      it('should create with default parameters', () => {
        const vu = new VUMeter();
        expect(vu).toBeInstanceOf(VUMeter);
      });

      it('should create with custom sample rate', () => {
        const vu = new VUMeter(48000);
        expect(vu).toBeInstanceOf(VUMeter);
      });

      it('should create with custom attack time', () => {
        const vu = new VUMeter(44100, 0.1);
        expect(vu).toBeInstanceOf(VUMeter);
      });

      it('should create with custom release time', () => {
        const vu = new VUMeter(44100, 0.3, 0.5);
        expect(vu).toBeInstanceOf(VUMeter);
      });
    });

    describe('process', () => {
      it('should process audio buffer', () => {
        const samples = new Float32Array([0.5, 0.5, 0.5, 0.5]);
        const level = vuMeter.process(samples);

        expect(typeof level).toBe('number');
        expect(level).toBeGreaterThanOrEqual(0);
      });

      it('should return current level after processing', () => {
        const samples = new Float32Array([0.7, 0.7, 0.7, 0.7]);
        const level = vuMeter.process(samples);

        expect(level).toBeGreaterThan(0);
        expect(level).toBeLessThanOrEqual(1);
      });

      it('should rise on attack', () => {
        const quiet = new Float32Array(100).fill(0.1);
        const loud = new Float32Array(100).fill(0.9);

        vuMeter.process(quiet);
        const level1 = vuMeter.getLevel();

        vuMeter.process(loud);
        const level2 = vuMeter.getLevel();

        expect(level2).toBeGreaterThan(level1);
      });

      it('should fall on release', () => {
        const loud = new Float32Array(100).fill(0.9);
        const quiet = new Float32Array(100).fill(0.1);

        vuMeter.process(loud);
        const level1 = vuMeter.getLevel();

        vuMeter.process(quiet);
        const level2 = vuMeter.getLevel();

        expect(level2).toBeLessThan(level1);
      });

      it('should handle multiple buffers', () => {
        for (let i = 0; i < 10; i++) {
          const samples = new Float32Array(100).fill(0.5);
          const level = vuMeter.process(samples);
          expect(level).toBeGreaterThanOrEqual(0);
        }
      });

      it('should handle zero amplitude', () => {
        const samples = new Float32Array([0, 0, 0, 0]);
        vuMeter.process(samples);
        const level = vuMeter.getLevel();

        expect(level).toBeLessThanOrEqual(0);
      });

      it('should handle negative samples', () => {
        const samples = new Float32Array([-0.5, -0.7, -0.3]);
        const level = vuMeter.process(samples);

        expect(level).toBeGreaterThanOrEqual(0);
      });
    });

    describe('getLevel', () => {
      it('should return level in dB', () => {
        const samples = new Float32Array([0.5, 0.5, 0.5]);
        vuMeter.process(samples);
        const level = vuMeter.getLevel();

        expect(typeof level).toBe('number');
        expect(level).toBeLessThanOrEqual(0);
      });

      it('should return -Infinity for zero level', () => {
        vuMeter.reset();
        const level = vuMeter.getLevel();

        expect(level).toBe(-Infinity);
      });

      it('should track processed signal', () => {
        const samples = new Float32Array(1000).fill(0.7);
        vuMeter.process(samples);
        const level = vuMeter.getLevel();

        expect(level).toBeGreaterThan(-Infinity);
        expect(level).toBeLessThan(0);
      });
    });

    describe('reset', () => {
      it('should reset meter to zero', () => {
        const samples = new Float32Array([0.9, 0.9, 0.9]);
        vuMeter.process(samples);

        vuMeter.reset();
        const level = vuMeter.getLevel();

        expect(level).toBe(-Infinity);
      });

      it('should allow reuse after reset', () => {
        const samples = new Float32Array([0.5, 0.5, 0.5]);

        vuMeter.process(samples);
        vuMeter.reset();
        vuMeter.process(samples);

        const level = vuMeter.getLevel();
        expect(level).toBeGreaterThan(-Infinity);
      });
    });

    describe('ballistics', () => {
      it('should have smooth attack behavior', () => {
        const levels: number[] = [];
        const samples = new Float32Array(100).fill(0.8);

        for (let i = 0; i < 20; i++) {
          vuMeter.process(samples);
          levels.push(vuMeter.getLevel());
        }

        // Levels should gradually increase
        for (let i = 1; i < levels.length - 1; i++) {
          if (levels[i] > -Infinity && levels[i-1] > -Infinity) {
            expect(levels[i]).toBeGreaterThanOrEqual(levels[i-1] - 1);
          }
        }
      });

      it('should have smooth release behavior', () => {
        // Charge up the meter
        const loud = new Float32Array(1000).fill(0.9);
        vuMeter.process(loud);

        // Track release
        const levels: number[] = [];
        const quiet = new Float32Array(100).fill(0.0);

        for (let i = 0; i < 20; i++) {
          vuMeter.process(quiet);
          levels.push(vuMeter.getLevel());
        }

        // Levels should gradually decrease
        for (let i = 1; i < levels.length - 1; i++) {
          if (levels[i] > -Infinity && levels[i-1] > -Infinity) {
            expect(levels[i]).toBeLessThanOrEqual(levels[i-1] + 1);
          }
        }
      });

      it('should handle fast attack time', () => {
        const fast = new VUMeter(44100, 0.01, 0.3);
        const samples = new Float32Array(1000).fill(0.8);

        fast.process(samples);
        const level = fast.getLevel();

        expect(level).toBeGreaterThan(-Infinity);
      });

      it('should handle slow release time', () => {
        const slow = new VUMeter(44100, 0.3, 1.0);
        const loud = new Float32Array(1000).fill(0.9);
        const quiet = new Float32Array(1000).fill(0.0);

        slow.process(loud);
        const level1 = slow.getLevel();

        slow.process(quiet);
        const level2 = slow.getLevel();

        // With slow release, level shouldn't drop too fast
        expect(level2).toBeGreaterThan(level1 - 20);
      });
    });

    describe('VU meter standard (300ms)', () => {
      it('should use 300ms ballistics by default', () => {
        const vu = new VUMeter();
        const samples = new Float32Array(100).fill(0.5);

        const level = vu.process(samples);
        expect(level).toBeGreaterThanOrEqual(0);
      });

      it('should behave like classic VU meter', () => {
        const samples = new Float32Array(5000);
        for (let i = 0; i < samples.length; i++) {
          samples[i] = 0.775 * Math.sin(2 * Math.PI * i / 100);
        }

        vuMeter.process(samples);
        const level = vuMeter.getLevel();

        // Should be relatively close to 0 VU for 0.775 amplitude
        expect(level).toBeGreaterThan(-10);
        expect(level).toBeLessThan(5);
      });
    });
  });
});
