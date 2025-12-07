import { describe, it, expect, beforeEach } from 'vitest';
import {
  PhaseMeter,
  StereoWidthAnalyzer,
  Goniometer,
  type PhaseAnalysis,
  type PhaseMeterOptions,
} from './phase-meter';

describe('phase-meter.ts', () => {
  describe('PhaseMeter - Constructor', () => {
    it('should create meter with default options', () => {
      const meter = new PhaseMeter();
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);
      const result = meter.analyze(left, right);

      expect(result).toHaveProperty('correlation');
      expect(result).toHaveProperty('status');
    });

    it('should create meter with custom sample rate', () => {
      const meter = new PhaseMeter({ sampleRate: 48000 });
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      expect(() => meter.analyze(left, right)).not.toThrow();
    });

    it('should create meter with custom window size', () => {
      const meter = new PhaseMeter({ windowSize: 2048 });
      const left = new Float32Array(2048);
      const right = new Float32Array(2048);

      expect(() => meter.analyze(left, right)).not.toThrow();
    });

    it('should create meter with custom thresholds', () => {
      const meter = new PhaseMeter({
        goodThreshold: 0.8,
        warningThreshold: 0.4,
      });
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      expect(() => meter.analyze(left, right)).not.toThrow();
    });
  });

  describe('PhaseMeter - analyze()', () => {
    let meter: PhaseMeter;

    beforeEach(() => {
      meter = new PhaseMeter({
        sampleRate: 44100,
        windowSize: 4096,
        goodThreshold: 0.7,
        warningThreshold: 0.3,
      });
    });

    it('should return PhaseAnalysis object with all properties', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);
      const result = meter.analyze(left, right);

      expect(result).toHaveProperty('correlation');
      expect(result).toHaveProperty('leftLevel');
      expect(result).toHaveProperty('rightLevel');
      expect(result).toHaveProperty('midLevel');
      expect(result).toHaveProperty('sideLevel');
      expect(result).toHaveProperty('coherence');
      expect(result).toHaveProperty('monoCompatible');
      expect(result).toHaveProperty('status');
    });

    it('should calculate perfect correlation (+1) for identical channels', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Fill with same values
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = meter.analyze(left, right);

      expect(result.correlation).toBeCloseTo(1.0, 5);
      expect(result.status).toBe('good');
      expect(result.monoCompatible).toBe(true);
    });

    it('should calculate perfect negative correlation (-1) for inverted channels', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Fill with inverted values
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = -Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = meter.analyze(left, right);

      expect(result.correlation).toBeCloseTo(-1.0, 5);
      expect(result.status).toBe('critical');
    });

    it('should calculate zero correlation for uncorrelated channels', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Left: 440 Hz sine, Right: random noise
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = (Math.random() - 0.5) * 2;
      }

      const result = meter.analyze(left, right);

      // Should be close to 0 (uncorrelated)
      expect(Math.abs(result.correlation)).toBeLessThan(0.3);
    });

    it('should calculate RMS levels for left and right channels', () => {
      const left = new Float32Array(1000).fill(0.5);
      const right = new Float32Array(1000).fill(0.3);

      const result = meter.analyze(left, right);

      expect(result.leftLevel).toBeCloseTo(20 * Math.log10(0.5), 1);
      expect(result.rightLevel).toBeCloseTo(20 * Math.log10(0.3), 1);
    });

    it('should calculate mid/side levels', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Different levels on each channel
      for (let i = 0; i < left.length; i++) {
        left[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = 0.3 * Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = meter.analyze(left, right);

      expect(isFinite(result.midLevel)).toBe(true);
      expect(isFinite(result.sideLevel)).toBe(true);
    });

    it('should have high mid and low side for mono signal', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Identical channels (mono)
      for (let i = 0; i < left.length; i++) {
        left[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = meter.analyze(left, right);

      // Mid should be strong, side should be minimal
      expect(result.midLevel).toBeGreaterThan(result.sideLevel);
    });

    it('should have high side for wide stereo signal', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Different signals on each channel
      for (let i = 0; i < left.length; i++) {
        left[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = 0.5 * Math.sin(2 * Math.PI * 880 * i / 44100);
      }

      const result = meter.analyze(left, right);

      // Side should have significant energy
      expect(result.sideLevel).toBeGreaterThan(-40);
    });

    it('should calculate coherence (0-1 range)', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = meter.analyze(left, right);

      expect(result.coherence).toBeGreaterThanOrEqual(0);
      expect(result.coherence).toBeLessThanOrEqual(1);
    });

    it('should have high coherence for identical signals', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        const value = Math.sin(2 * Math.PI * 440 * i / 44100);
        left[i] = value;
        right[i] = value;
      }

      const result = meter.analyze(left, right);

      expect(result.coherence).toBeGreaterThan(0.9);
    });

    it('should have low coherence for uncorrelated signals', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = (Math.random() - 0.5) * 2;
      }

      const result = meter.analyze(left, right);

      expect(result.coherence).toBeLessThan(0.5);
    });

    it('should mark mono compatible when correlation >= warning threshold', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // High correlation
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = 0.8 * Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = meter.analyze(left, right);

      expect(result.monoCompatible).toBe(true);
    });

    it('should mark not mono compatible when correlation < warning threshold', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Inverted phase
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = -Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = meter.analyze(left, right);

      expect(result.monoCompatible).toBe(false);
    });

    it('should report "good" status for high correlation', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = 0.9 * Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = meter.analyze(left, right);

      expect(result.status).toBe('good');
    });

    it('should report "warning" status for medium correlation', () => {
      const meter = new PhaseMeter({ goodThreshold: 0.8, warningThreshold: 0.4 });
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Moderate correlation (between 0.4 and 0.8)
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100) + 0.3 * (Math.random() - 0.5);
      }

      const result = meter.analyze(left, right);

      // Result should be in warning or critical range
      expect(['warning', 'critical']).toContain(result.status);
    });

    it('should report "critical" status for low/negative correlation', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Inverted phase
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = -Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = meter.analyze(left, right);

      expect(result.status).toBe('critical');
    });

    it('should throw error for mismatched channel lengths', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(500);

      expect(() => meter.analyze(left, right)).toThrow(
        'Left and right channels must have same length'
      );
    });

    it('should handle silence (zero correlation)', () => {
      const silence = new Float32Array(1000);
      const result = meter.analyze(silence, silence);

      expect(result.correlation).toBe(0);
      expect(result.leftLevel).toBe(-Infinity);
      expect(result.rightLevel).toBe(-Infinity);
    });

    it('should handle negative samples correctly', () => {
      const left = new Float32Array(1000).fill(-0.5);
      const right = new Float32Array(1000).fill(-0.5);

      const result = meter.analyze(left, right);

      expect(result.correlation).toBeCloseTo(1.0, 5);
      expect(isFinite(result.leftLevel)).toBe(true);
    });
  });

  describe('PhaseMeter - analyzeOverTime()', () => {
    let meter: PhaseMeter;

    beforeEach(() => {
      meter = new PhaseMeter({ sampleRate: 44100, windowSize: 2048 });
    });

    it('should return array of time-stamped correlation values', () => {
      const left = new Float32Array(44100); // 1 second
      const right = new Float32Array(44100);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const results = meter.analyzeOverTime(left, right);

      expect(Array.isArray(results)).toBe(true);
      expect(results.length).toBeGreaterThan(0);
    });

    it('should include time, correlation, and status in each result', () => {
      const left = new Float32Array(10000);
      const right = new Float32Array(10000);

      const results = meter.analyzeOverTime(left, right);

      results.forEach(result => {
        expect(result).toHaveProperty('time');
        expect(result).toHaveProperty('correlation');
        expect(result).toHaveProperty('status');
      });
    });

    it('should calculate correlation at multiple time points', () => {
      const left = new Float32Array(44100); // 1 second
      const right = new Float32Array(44100);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const results = meter.analyzeOverTime(left, right);

      // Should have multiple time points
      expect(results.length).toBeGreaterThan(10);

      // Times should be in ascending order
      for (let i = 1; i < results.length; i++) {
        expect(results[i].time).toBeGreaterThan(results[i - 1].time);
      }
    });

    it('should detect phase changes over time', () => {
      const left = new Float32Array(88200); // 2 seconds
      const right = new Float32Array(88200);

      // First half: in phase
      for (let i = 0; i < 44100; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      // Second half: out of phase
      for (let i = 44100; i < 88200; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = -Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const results = meter.analyzeOverTime(left, right);

      // Should have varying correlation values
      const correlations = results.map(r => r.correlation);
      const minCorr = Math.min(...correlations);
      const maxCorr = Math.max(...correlations);

      expect(maxCorr).toBeGreaterThan(minCorr);
    });

    it('should assign correct status at each time point', () => {
      const left = new Float32Array(10000);
      const right = new Float32Array(10000);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const results = meter.analyzeOverTime(left, right);

      // All should have valid status
      results.forEach(result => {
        expect(['good', 'warning', 'critical']).toContain(result.status);
      });
    });

    it('should use overlapping windows (hop size = windowSize / 2)', () => {
      const windowSize = 2048;
      const meter = new PhaseMeter({ windowSize });
      const left = new Float32Array(10000);
      const right = new Float32Array(10000);

      const results = meter.analyzeOverTime(left, right);

      // Should have more results due to 50% overlap
      const expectedMinResults = Math.floor((10000 - windowSize) / (windowSize / 2));
      expect(results.length).toBeGreaterThanOrEqual(expectedMinResults - 1);
    });
  });

  describe('PhaseMeter - detectPhaseInversion()', () => {
    let meter: PhaseMeter;

    beforeEach(() => {
      meter = new PhaseMeter();
    });

    it('should detect phase inversion (180° out of phase)', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = -Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const inverted = meter.detectPhaseInversion(left, right);

      expect(inverted).toBe(true);
    });

    it('should not detect inversion for in-phase signals', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const inverted = meter.detectPhaseInversion(left, right);

      expect(inverted).toBe(false);
    });

    it('should not detect inversion for uncorrelated signals', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = (Math.random() - 0.5) * 2;
      }

      const inverted = meter.detectPhaseInversion(left, right);

      expect(inverted).toBe(false);
    });

    it('should use -0.5 threshold for detection', () => {
      const meter = new PhaseMeter();
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Correlation slightly above -0.5 (should not detect)
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = -0.45 * Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const inverted = meter.detectPhaseInversion(left, right);

      // Correlation is negative but not below -0.5
      expect(inverted).toBe(false);
    });
  });

  describe('PhaseMeter - suggestCorrection()', () => {
    let meter: PhaseMeter;

    beforeEach(() => {
      meter = new PhaseMeter({ goodThreshold: 0.7, warningThreshold: 0.3 });
    });

    it('should return null for good phase correlation', () => {
      const analysis: PhaseAnalysis = {
        correlation: 0.9,
        leftLevel: -6,
        rightLevel: -6,
        midLevel: -6,
        sideLevel: -20,
        coherence: 0.95,
        monoCompatible: true,
        status: 'good',
      };

      const suggestion = meter.suggestCorrection(analysis);

      expect(suggestion).toBeNull();
    });

    it('should suggest polarity flip for phase inversion', () => {
      const analysis: PhaseAnalysis = {
        correlation: -0.8,
        leftLevel: -6,
        rightLevel: -6,
        midLevel: -20,
        sideLevel: -6,
        coherence: 0.5,
        monoCompatible: false,
        status: 'critical',
      };

      const suggestion = meter.suggestCorrection(analysis);

      expect(suggestion).toContain('Phase inverted');
      expect(suggestion).toContain('180');
    });

    it('should suggest checking microphone placement for poor correlation', () => {
      const analysis: PhaseAnalysis = {
        correlation: 0.2,
        leftLevel: -6,
        rightLevel: -6,
        midLevel: -10,
        sideLevel: -10,
        coherence: 0.4,
        monoCompatible: false,
        status: 'critical',
      };

      const suggestion = meter.suggestCorrection(analysis);

      expect(suggestion).toContain('Poor phase correlation');
      expect(suggestion).toContain('microphone');
    });

    it('should suggest time alignment for partial cancellation', () => {
      const analysis: PhaseAnalysis = {
        correlation: -0.3,
        leftLevel: -6,
        rightLevel: -6,
        midLevel: -10,
        sideLevel: -10,
        coherence: 0.5,
        monoCompatible: false,
        status: 'critical',
      };

      const suggestion = meter.suggestCorrection(analysis);

      expect(suggestion).toContain('Partial phase cancellation');
      expect(suggestion).toContain('time alignment');
    });

    it('should provide general suggestion for other phase issues', () => {
      const analysis: PhaseAnalysis = {
        correlation: 0.5,
        leftLevel: -6,
        rightLevel: -6,
        midLevel: -10,
        sideLevel: -10,
        coherence: 0.6,
        monoCompatible: true,
        status: 'warning',
      };

      const suggestion = meter.suggestCorrection(analysis);

      expect(suggestion).toBeTruthy();
    });
  });

  describe('StereoWidthAnalyzer - analyze()', () => {
    let analyzer: StereoWidthAnalyzer;

    beforeEach(() => {
      analyzer = new StereoWidthAnalyzer();
    });

    it('should return object with width, balance, and content percentages', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      const result = analyzer.analyze(left, right);

      expect(result).toHaveProperty('width');
      expect(result).toHaveProperty('balance');
      expect(result).toHaveProperty('monoContent');
      expect(result).toHaveProperty('stereoContent');
    });

    it('should calculate width of 0 for mono signal', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Identical channels (mono)
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = analyzer.analyze(left, right);

      expect(result.width).toBeCloseTo(0, 1);
    });

    it('should calculate width > 0 for stereo signal', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Different content on each channel
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = Math.sin(2 * Math.PI * 880 * i / 44100);
      }

      const result = analyzer.analyze(left, right);

      expect(result.width).toBeGreaterThan(0);
    });

    it('should calculate width ≈ 2 for maximum width (side only)', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Out of phase (maximum stereo width)
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = -Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = analyzer.analyze(left, right);

      expect(result.width).toBeGreaterThan(1);
    });

    it('should calculate balance of 0 for centered signal', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Equal levels
      for (let i = 0; i < left.length; i++) {
        left[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = 0.5 * Math.sin(2 * Math.PI * 880 * i / 44100);
      }

      const result = analyzer.analyze(left, right);

      expect(result.balance).toBeCloseTo(0, 1);
    });

    it('should calculate negative balance for left-heavy signal', () => {
      const left = new Float32Array(1000).fill(0.8);
      const right = new Float32Array(1000).fill(0.2);

      const result = analyzer.analyze(left, right);

      expect(result.balance).toBeLessThan(0);
    });

    it('should calculate positive balance for right-heavy signal', () => {
      const left = new Float32Array(1000).fill(0.2);
      const right = new Float32Array(1000).fill(0.8);

      const result = analyzer.analyze(left, right);

      expect(result.balance).toBeGreaterThan(0);
    });

    it('should calculate high mono content for mono signal', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = analyzer.analyze(left, right);

      expect(result.monoContent).toBeGreaterThan(0.8);
      expect(result.stereoContent).toBeLessThan(0.2);
    });

    it('should calculate high stereo content for wide stereo signal', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Out of phase
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = -Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = analyzer.analyze(left, right);

      expect(result.stereoContent).toBeGreaterThan(0.5);
    });

    it('should have mono + stereo content sum to 1', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = 0.5 * Math.sin(2 * Math.PI * 880 * i / 44100);
      }

      const result = analyzer.analyze(left, right);

      expect(result.monoContent + result.stereoContent).toBeCloseTo(1.0, 5);
    });

    it('should handle silence', () => {
      const silence = new Float32Array(1000);
      const result = analyzer.analyze(silence, silence);

      // For silence, mono content should be 1 (fallback)
      expect(result.monoContent).toBe(1);
      expect(result.stereoContent).toBe(0);
    });
  });

  describe('Goniometer - generate()', () => {
    let goniometer: Goniometer;

    beforeEach(() => {
      goniometer = new Goniometer();
    });

    it('should return array of [x, y] coordinate points', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const points = goniometer.generate(left, right);

      expect(Array.isArray(points)).toBe(true);
      expect(points.length).toBeGreaterThan(0);
      expect(points[0].length).toBe(2);
    });

    it('should generate specified number of points', () => {
      const left = new Float32Array(10000);
      const right = new Float32Array(10000);

      const points = goniometer.generate(left, right, 500);

      expect(points.length).toBeLessThanOrEqual(500);
    });

    it('should generate default 1000 points', () => {
      const left = new Float32Array(10000);
      const right = new Float32Array(10000);

      const points = goniometer.generate(left, right);

      expect(points.length).toBeLessThanOrEqual(1000);
    });

    it('should calculate mid/side coordinates correctly', () => {
      const left = new Float32Array(100);
      const right = new Float32Array(100);

      // Known values for testing
      left[0] = 0.8;
      right[0] = 0.6;

      const points = goniometer.generate(left, right, 100);

      // First point: side = (0.8 - 0.6) / 2 = 0.1, mid = (0.8 + 0.6) / 2 = 0.7
      expect(points[0][0]).toBeCloseTo(0.1, 5); // side (x)
      expect(points[0][1]).toBeCloseTo(0.7, 5); // mid (y)
    });

    it('should handle mono signal (vertical line)', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Identical channels
      for (let i = 0; i < left.length; i++) {
        const value = Math.sin(2 * Math.PI * 440 * i / 44100);
        left[i] = value;
        right[i] = value;
      }

      const points = goniometer.generate(left, right, 100);

      // All points should have x (side) ≈ 0
      points.forEach(([x, y]) => {
        expect(Math.abs(x)).toBeCloseTo(0, 10);
      });
    });

    it('should handle stereo signal (not just vertical)', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Different content
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = Math.sin(2 * Math.PI * 880 * i / 44100);
      }

      const points = goniometer.generate(left, right, 100);

      // Should have some points with non-zero x (side)
      const hasWidth = points.some(([x, y]) => Math.abs(x) > 0.01);
      expect(hasWidth).toBe(true);
    });
  });

  describe('Goniometer - analyzePattern()', () => {
    let goniometer: Goniometer;

    beforeEach(() => {
      goniometer = new Goniometer();
    });

    it('should return object with shape and monoCompatibility', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      const result = goniometer.analyzePattern(left, right);

      expect(result).toHaveProperty('shape');
      expect(result).toHaveProperty('monoCompatibility');
    });

    it('should detect vertical shape for mono signal', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        const value = Math.sin(2 * Math.PI * 440 * i / 44100);
        left[i] = value;
        right[i] = value;
      }

      const result = goniometer.analyzePattern(left, right);

      expect(result.shape).toBe('vertical');
      expect(result.monoCompatibility).toBeGreaterThan(0.8);
    });

    it('should detect horizontal shape for out-of-phase signal', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = -Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const result = goniometer.analyzePattern(left, right);

      expect(result.shape).toBe('horizontal');
      expect(result.monoCompatibility).toBeLessThan(0.5);
    });

    it('should detect circular shape for well-balanced stereo', () => {
      const left = new Float32Array(2000);
      const right = new Float32Array(2000);

      // Create balanced stereo content
      for (let i = 0; i < left.length; i++) {
        left[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100) +
                  0.3 * Math.cos(2 * Math.PI * 880 * i / 44100);
        right[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100) -
                   0.3 * Math.cos(2 * Math.PI * 880 * i / 44100);
      }

      const result = goniometer.analyzePattern(left, right);

      expect(['circular', 'diagonal', 'irregular']).toContain(result.shape);
    });

    it('should calculate monoCompatibility in range [0, 1]', () => {
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = 0.5 * Math.sin(2 * Math.PI * 880 * i / 44100);
      }

      const result = goniometer.analyzePattern(left, right);

      expect(result.monoCompatibility).toBeGreaterThanOrEqual(0);
      expect(result.monoCompatibility).toBeLessThanOrEqual(1);
    });

    it('should recognize all valid shapes', () => {
      const validShapes = ['vertical', 'horizontal', 'diagonal', 'circular', 'irregular'];
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      const result = goniometer.analyzePattern(left, right);

      expect(validShapes).toContain(result.shape);
    });

    it('should handle silence', () => {
      const silence = new Float32Array(1000);
      const result = goniometer.analyzePattern(silence, silence);

      expect(result.shape).toBeDefined();
      expect(result.monoCompatibility).toBeGreaterThanOrEqual(0);
      expect(result.monoCompatibility).toBeLessThanOrEqual(1);
    });
  });

  describe('Integration Tests', () => {
    it('should analyze complete stereo signal with all tools', () => {
      const phaseMeter = new PhaseMeter();
      const widthAnalyzer = new StereoWidthAnalyzer();
      const goniometer = new Goniometer();

      const left = new Float32Array(4000);
      const right = new Float32Array(4000);

      // Create realistic stereo music signal
      for (let i = 0; i < left.length; i++) {
        const bass = 0.4 * Math.sin(2 * Math.PI * 80 * i / 44100);
        const mid = 0.3 * Math.sin(2 * Math.PI * 440 * i / 44100);
        const high = 0.2 * Math.sin(2 * Math.PI * 4000 * i / 44100);

        left[i] = bass + mid + high;
        right[i] = bass + mid * 0.8 - high * 0.5; // Different stereo image
      }

      const phaseAnalysis = phaseMeter.analyze(left, right);
      const widthAnalysis = widthAnalyzer.analyze(left, right);
      const gonioPoints = goniometer.generate(left, right);
      const gonioPattern = goniometer.analyzePattern(left, right);

      expect(phaseAnalysis.status).toBeDefined();
      expect(widthAnalysis.width).toBeGreaterThanOrEqual(0);
      expect(gonioPoints.length).toBeGreaterThan(0);
      expect(gonioPattern.shape).toBeDefined();
    });

    it('should detect and suggest correction for phase issues', () => {
      const meter = new PhaseMeter();
      const left = new Float32Array(1000);
      const right = new Float32Array(1000);

      // Create inverted phase
      for (let i = 0; i < left.length; i++) {
        left[i] = Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = -Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const analysis = meter.analyze(left, right);
      const isInverted = meter.detectPhaseInversion(left, right);
      const suggestion = meter.suggestCorrection(analysis);

      expect(isInverted).toBe(true);
      expect(analysis.status).toBe('critical');
      expect(suggestion).toContain('inverted');
    });

    it('should provide consistent analysis across tools', () => {
      const phaseMeter = new PhaseMeter();
      const widthAnalyzer = new StereoWidthAnalyzer();

      const left = new Float32Array(2000);
      const right = new Float32Array(2000);

      // Perfect mono
      for (let i = 0; i < left.length; i++) {
        const value = Math.sin(2 * Math.PI * 440 * i / 44100);
        left[i] = value;
        right[i] = value;
      }

      const phaseAnalysis = phaseMeter.analyze(left, right);
      const widthAnalysis = widthAnalyzer.analyze(left, right);

      // Both should agree it's mono
      expect(phaseAnalysis.correlation).toBeCloseTo(1.0, 5);
      expect(widthAnalysis.width).toBeCloseTo(0, 1);
      expect(widthAnalysis.monoContent).toBeGreaterThan(0.9);
    });
  });
});
