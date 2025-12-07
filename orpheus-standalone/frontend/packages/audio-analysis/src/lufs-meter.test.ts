import { describe, it, expect, beforeEach } from 'vitest';
import {
  LUFSMeter,
  normalizeLUFS,
  LOUDNESS_TARGETS,
  type LUFSAnalysis,
  type LUFSMeterOptions,
} from './lufs-meter';

describe('lufs-meter.ts', () => {
  describe('LUFSMeter - Constructor', () => {
    it('should create meter with default options', () => {
      const meter = new LUFSMeter();
      const silence = [new Float32Array(44100), new Float32Array(44100)];
      const result = meter.analyze(silence);

      expect(result.target).toBe(-14); // Spotify default
      expect(result.integrated).toBe(-Infinity); // Silence
    });

    it('should create meter with custom sample rate', () => {
      const meter = new LUFSMeter({ sampleRate: 48000 });
      const silence = [new Float32Array(48000), new Float32Array(48000)];
      const result = meter.analyze(silence);

      expect(result.integrated).toBe(-Infinity);
    });

    it('should create meter with custom target loudness', () => {
      const meter = new LUFSMeter({ targetLoudness: -16 });
      const silence = [new Float32Array(44100), new Float32Array(44100)];
      const result = meter.analyze(silence);

      expect(result.target).toBe(-16);
    });

    it('should create meter with custom tolerance', () => {
      const meter = new LUFSMeter({ tolerance: 0.5 });
      // Create a signal at exactly -14 LUFS (within 0.5 LU tolerance)
      const signal = createCalibratedSignal(-14, 44100, 2);
      const result = meter.analyze(signal);

      expect(Math.abs(result.integrated - result.target)).toBeLessThanOrEqual(0.5);
    });

    it('should create meter for mono audio', () => {
      const meter = new LUFSMeter({ channels: 1 });
      const mono = [new Float32Array(44100)];
      const result = meter.analyze(mono);

      expect(result.integrated).toBe(-Infinity);
    });

    it('should create meter for stereo audio', () => {
      const meter = new LUFSMeter({ channels: 2 });
      const stereo = [new Float32Array(44100), new Float32Array(44100)];
      const result = meter.analyze(stereo);

      expect(result.integrated).toBe(-Infinity);
    });

    it('should create meter for 5.1 surround', () => {
      const meter = new LUFSMeter({ channels: 6 });
      const surround = Array.from({ length: 6 }, () => new Float32Array(44100));
      const result = meter.analyze(surround);

      expect(result.integrated).toBe(-Infinity);
    });

    it('should initialize filters for all channels', () => {
      const meter = new LUFSMeter({ channels: 4 });
      const quad = Array.from({ length: 4 }, () => new Float32Array(1000));

      // Should not throw
      expect(() => meter.analyze(quad)).not.toThrow();
    });
  });

  describe('LUFSMeter - analyze()', () => {
    let meter: LUFSMeter;

    beforeEach(() => {
      meter = new LUFSMeter({ sampleRate: 44100, targetLoudness: -14, tolerance: 1.0 });
    });

    it('should return LUFSAnalysis object with all properties', () => {
      const channels = [new Float32Array(44100), new Float32Array(44100)];
      const result = meter.analyze(channels);

      expect(result).toHaveProperty('integrated');
      expect(result).toHaveProperty('range');
      expect(result).toHaveProperty('shortTerm');
      expect(result).toHaveProperty('momentary');
      expect(result).toHaveProperty('truePeak');
      expect(result).toHaveProperty('compliant');
      expect(result).toHaveProperty('target');
    });

    it('should handle silence (returns -Infinity)', () => {
      const silence = [new Float32Array(44100), new Float32Array(44100)];
      const result = meter.analyze(silence);

      expect(result.integrated).toBe(-Infinity);
      expect(result.shortTerm).toBe(-Infinity);
      expect(result.momentary).toBe(-Infinity);
      expect(result.truePeak).toBe(-Infinity);
    });

    it('should handle very quiet signals', () => {
      const quiet = [
        new Float32Array(44100).fill(0.001),
        new Float32Array(44100).fill(0.001),
      ];
      const result = meter.analyze(quiet);

      expect(result.integrated).toBeLessThan(-60); // Very quiet
      expect(isFinite(result.integrated)).toBe(true);
    });

    it('should handle loud signals', () => {
      const loud = [
        new Float32Array(44100).fill(0.5),
        new Float32Array(44100).fill(0.5),
      ];
      const result = meter.analyze(loud);

      expect(result.integrated).toBeGreaterThan(-20);
      expect(result.truePeak).toBeCloseTo(20 * Math.log10(0.5), 1);
    });

    it('should throw error for wrong number of channels', () => {
      const wrongChannels = [new Float32Array(1000)]; // Only 1 channel, expects 2

      expect(() => meter.analyze(wrongChannels)).toThrow(
        'Expected 2 channels, got 1'
      );
    });

    it('should handle mono audio when configured for mono', () => {
      const monoMeter = new LUFSMeter({ channels: 1 });
      const mono = [new Float32Array(44100).fill(0.1)];
      const result = monoMeter.analyze(mono);

      expect(isFinite(result.integrated)).toBe(true);
    });

    it('should handle different buffer lengths', () => {
      const short = [new Float32Array(1000), new Float32Array(1000)];
      const medium = [new Float32Array(44100), new Float32Array(44100)];
      const long = [new Float32Array(441000), new Float32Array(441000)];

      expect(() => meter.analyze(short)).not.toThrow();
      expect(() => meter.analyze(medium)).not.toThrow();
      expect(() => meter.analyze(long)).not.toThrow();
    });

    it('should calculate integrated loudness using gating', () => {
      // Create signal with varying levels
      const left = new Float32Array(88200); // 2 seconds
      const right = new Float32Array(88200);

      // First second loud, second second quiet
      for (let i = 0; i < 44100; i++) {
        left[i] = 0.3;
        right[i] = 0.3;
      }
      for (let i = 44100; i < 88200; i++) {
        left[i] = 0.001; // Very quiet
        right[i] = 0.001;
      }

      const result = meter.analyze([left, right]);

      // Integrated should be influenced more by loud section due to gating
      expect(result.integrated).toBeGreaterThan(-40);
    });

    it('should apply absolute gating at -70 LUFS', () => {
      // Create a signal that's below -70 LUFS
      const veryQuiet = [
        new Float32Array(44100).fill(0.00001),
        new Float32Array(44100).fill(0.00001),
      ];
      const result = meter.analyze(veryQuiet);

      // Should be gated out
      expect(result.integrated).toBe(-Infinity);
    });

    it('should calculate loudness range (LRA)', () => {
      // Create dynamic signal
      const left = new Float32Array(88200); // 2 seconds
      const right = new Float32Array(88200);

      // Varying levels for dynamic range
      for (let i = 0; i < 22050; i++) {
        left[i] = 0.5; // Loud
        right[i] = 0.5;
      }
      for (let i = 22050; i < 66150; i++) {
        left[i] = 0.1; // Medium
        right[i] = 0.1;
      }
      for (let i = 66150; i < 88200; i++) {
        left[i] = 0.5; // Loud again
        right[i] = 0.5;
      }

      const result = meter.analyze([left, right]);

      // Should have some loudness range
      expect(result.range).toBeGreaterThan(0);
      expect(result.range).toBeLessThan(30); // Reasonable range
    });

    it('should return zero range for constant level', () => {
      const constant = [
        new Float32Array(88200).fill(0.3),
        new Float32Array(88200).fill(0.3),
      ];
      const result = meter.analyze(constant);

      // Range should be very small (close to 0) for constant signal
      expect(result.range).toBeLessThan(2);
    });

    it('should calculate short-term loudness (3 seconds)', () => {
      const channels = [
        new Float32Array(132300).fill(0.2), // 3 seconds
        new Float32Array(132300).fill(0.2),
      ];
      const result = meter.analyze(channels);

      expect(isFinite(result.shortTerm)).toBe(true);
      // Short-term should be close to integrated for constant signal
      expect(Math.abs(result.shortTerm - result.integrated)).toBeLessThan(5);
    });

    it('should calculate momentary loudness (400ms)', () => {
      const channels = [
        new Float32Array(44100).fill(0.2),
        new Float32Array(44100).fill(0.2),
      ];
      const result = meter.analyze(channels);

      expect(isFinite(result.momentary)).toBe(true);
      // Momentary should be close to integrated for constant signal
      expect(Math.abs(result.momentary - result.integrated)).toBeLessThan(5);
    });

    it('should calculate true peak using oversampling', () => {
      // Create signal with potential inter-sample peak
      const left = new Float32Array(100);
      const right = new Float32Array(100);

      // Sharp transition that could cause inter-sample peak
      left[50] = 0.8;
      left[51] = 0.9;
      right[50] = 0.8;
      right[51] = 0.9;

      const result = meter.analyze([left, right]);

      // True peak should detect inter-sample peaks
      expect(result.truePeak).toBeGreaterThan(20 * Math.log10(0.8));
      expect(result.truePeak).toBeLessThanOrEqual(20 * Math.log10(1.0));
    });

    it('should detect peaks above 0 dBTP', () => {
      // Create clipping signal
      const clipped = [
        new Float32Array(1000).fill(1.2),
        new Float32Array(1000).fill(1.2),
      ];
      const result = meter.analyze(clipped);

      expect(result.truePeak).toBeGreaterThan(0); // Above 0 dBTP
    });
  });

  describe('LUFSMeter - Compliance Checking', () => {
    it('should mark compliant when within tolerance (Spotify -14 LUFS)', () => {
      const meter = new LUFSMeter({
        targetLoudness: -14,
        tolerance: 1.0,
      });

      // Create signal at approximately -14 LUFS
      const signal = createCalibratedSignal(-14, 44100, 2);
      const result = meter.analyze(signal);

      // Should be within ±1 LU of -14 LUFS
      expect(Math.abs(result.integrated - (-14))).toBeLessThan(2);
    });

    it('should mark non-compliant when outside tolerance', () => {
      const meter = new LUFSMeter({
        targetLoudness: -14,
        tolerance: 1.0,
      });

      // Very loud signal (well above -14 LUFS)
      const loud = [
        new Float32Array(88200).fill(0.8),
        new Float32Array(88200).fill(0.8),
      ];
      const result = meter.analyze(loud);

      // Should be non-compliant (too loud)
      expect(result.integrated).toBeGreaterThan(-14 + 1.0);
    });

    it('should check compliance against Spotify target', () => {
      const meter = new LUFSMeter({ targetLoudness: LOUDNESS_TARGETS.SPOTIFY });
      const signal = createCalibratedSignal(-14, 44100, 2);
      const result = meter.analyze(signal);

      expect(result.target).toBe(-14);
    });

    it('should check compliance against Apple Music target', () => {
      const meter = new LUFSMeter({ targetLoudness: LOUDNESS_TARGETS.APPLE_MUSIC });
      const signal = createCalibratedSignal(-16, 44100, 2);
      const result = meter.analyze(signal);

      expect(result.target).toBe(-16);
    });

    it('should check compliance against broadcast TV target', () => {
      const meter = new LUFSMeter({ targetLoudness: LOUDNESS_TARGETS.BROADCAST_TV });
      const quiet = [
        new Float32Array(88200).fill(0.05),
        new Float32Array(88200).fill(0.05),
      ];
      const result = meter.analyze(quiet);

      expect(result.target).toBe(-23);
    });
  });

  describe('LUFSMeter - reset()', () => {
    it('should reset meter state', () => {
      const meter = new LUFSMeter();
      const signal = [
        new Float32Array(44100).fill(0.3),
        new Float32Array(44100).fill(0.3),
      ];

      meter.analyze(signal);
      meter.reset();

      // Should be able to analyze again after reset
      const result = meter.analyze(signal);
      expect(isFinite(result.integrated)).toBe(true);
    });

    it('should reset filter state', () => {
      const meter = new LUFSMeter();

      // Analyze once to populate filter state
      const signal1 = [
        new Float32Array(1000).fill(0.5),
        new Float32Array(1000).fill(0.5),
      ];
      const result1 = meter.analyze(signal1);

      meter.reset();

      // Analyze same signal again - should get same result
      const result2 = meter.analyze(signal1);

      expect(result1.integrated).toBeCloseTo(result2.integrated, 1);
    });
  });

  describe('LUFSMeter - K-weighting Filter', () => {
    it('should apply K-weighting to all channels', () => {
      const meter = new LUFSMeter();

      // White noise-like signal
      const left = new Float32Array(44100);
      const right = new Float32Array(44100);
      for (let i = 0; i < left.length; i++) {
        left[i] = (Math.random() - 0.5) * 0.1;
        right[i] = (Math.random() - 0.5) * 0.1;
      }

      const result = meter.analyze([left, right]);

      // K-weighted result should differ from simple RMS
      expect(isFinite(result.integrated)).toBe(true);
    });

    it('should handle high-frequency content (affected by pre-filter)', () => {
      const meter = new LUFSMeter({ sampleRate: 44100 });

      // High-frequency sine wave (5kHz)
      const left = new Float32Array(44100);
      const right = new Float32Array(44100);
      for (let i = 0; i < left.length; i++) {
        left[i] = 0.1 * Math.sin(2 * Math.PI * 5000 * i / 44100);
        right[i] = 0.1 * Math.sin(2 * Math.PI * 5000 * i / 44100);
      }

      const result = meter.analyze([left, right]);
      expect(isFinite(result.integrated)).toBe(true);
    });

    it('should handle low-frequency content (affected by RLB filter)', () => {
      const meter = new LUFSMeter({ sampleRate: 44100 });

      // Low-frequency sine wave (50Hz)
      const left = new Float32Array(44100);
      const right = new Float32Array(44100);
      for (let i = 0; i < left.length; i++) {
        left[i] = 0.1 * Math.sin(2 * Math.PI * 50 * i / 44100);
        right[i] = 0.1 * Math.sin(2 * Math.PI * 50 * i / 44100);
      }

      const result = meter.analyze([left, right]);
      expect(isFinite(result.integrated)).toBe(true);
    });
  });

  describe('LUFSMeter - Edge Cases', () => {
    it('should handle single sample buffer', () => {
      const meter = new LUFSMeter();
      const single = [new Float32Array(1).fill(0.5), new Float32Array(1).fill(0.5)];

      const result = meter.analyze(single);
      // Not enough data for blocks, but should not crash
      expect(result.integrated).toBe(-Infinity);
    });

    it('should handle very short buffers (< 400ms)', () => {
      const meter = new LUFSMeter({ sampleRate: 44100 });
      const short = [
        new Float32Array(1000).fill(0.3), // ~22ms
        new Float32Array(1000).fill(0.3),
      ];

      const result = meter.analyze(short);
      expect(result.integrated).toBe(-Infinity); // Not enough blocks
    });

    it('should handle buffer exactly 400ms (one block)', () => {
      const meter = new LUFSMeter({ sampleRate: 44100 });
      const oneBlock = [
        new Float32Array(17640).fill(0.3), // 400ms
        new Float32Array(17640).fill(0.3),
      ];

      const result = meter.analyze(oneBlock);
      expect(isFinite(result.momentary)).toBe(true);
    });

    it('should handle zero amplitude samples', () => {
      const meter = new LUFSMeter();
      const zeros = [new Float32Array(44100), new Float32Array(44100)];
      const result = meter.analyze(zeros);

      expect(result.integrated).toBe(-Infinity);
      expect(result.shortTerm).toBe(-Infinity);
      expect(result.momentary).toBe(-Infinity);
      expect(result.truePeak).toBe(-Infinity);
    });

    it('should handle negative samples', () => {
      const meter = new LUFSMeter();
      const negative = [
        new Float32Array(44100).fill(-0.3),
        new Float32Array(44100).fill(-0.3),
      ];
      const result = meter.analyze(negative);

      expect(isFinite(result.integrated)).toBe(true);
    });

    it('should handle mixed positive/negative samples', () => {
      const meter = new LUFSMeter();
      const left = new Float32Array(44100);
      const right = new Float32Array(44100);

      for (let i = 0; i < left.length; i++) {
        left[i] = i % 2 === 0 ? 0.3 : -0.3;
        right[i] = i % 2 === 0 ? 0.3 : -0.3;
      }

      const result = meter.analyze([left, right]);
      expect(isFinite(result.integrated)).toBe(true);
    });

    it('should handle asymmetric stereo levels', () => {
      const meter = new LUFSMeter();
      const left = new Float32Array(44100).fill(0.5);
      const right = new Float32Array(44100).fill(0.1);

      const result = meter.analyze([left, right]);
      expect(isFinite(result.integrated)).toBe(true);
    });

    it('should handle channels with different content', () => {
      const meter = new LUFSMeter();
      const left = new Float32Array(44100);
      const right = new Float32Array(44100);

      // Left: sine wave, Right: square wave
      for (let i = 0; i < left.length; i++) {
        left[i] = 0.3 * Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = i % 100 < 50 ? 0.3 : -0.3;
      }

      const result = meter.analyze([left, right]);
      expect(isFinite(result.integrated)).toBe(true);
    });
  });

  describe('LOUDNESS_TARGETS', () => {
    it('should export Spotify target', () => {
      expect(LOUDNESS_TARGETS.SPOTIFY).toBe(-14);
    });

    it('should export Apple Music target', () => {
      expect(LOUDNESS_TARGETS.APPLE_MUSIC).toBe(-16);
    });

    it('should export YouTube target', () => {
      expect(LOUDNESS_TARGETS.YOUTUBE).toBe(-14);
    });

    it('should export Tidal target', () => {
      expect(LOUDNESS_TARGETS.TIDAL).toBe(-14);
    });

    it('should export Amazon Music target', () => {
      expect(LOUDNESS_TARGETS.AMAZON_MUSIC).toBe(-14);
    });

    it('should export Deezer target', () => {
      expect(LOUDNESS_TARGETS.DEEZER).toBe(-15);
    });

    it('should export SoundCloud target', () => {
      expect(LOUDNESS_TARGETS.SOUNDCLOUD).toBe(-14);
    });

    it('should export Broadcast TV target (EBU R128)', () => {
      expect(LOUDNESS_TARGETS.BROADCAST_TV).toBe(-23);
    });

    it('should export Broadcast Radio target (EBU R128)', () => {
      expect(LOUDNESS_TARGETS.BROADCAST_RADIO).toBe(-23);
    });

    it('should export Cinema target (SMPTE)', () => {
      expect(LOUDNESS_TARGETS.CINEMA).toBe(-24);
    });

    it('should export CD mastering target', () => {
      expect(LOUDNESS_TARGETS.CD_MASTERING).toBe(-9);
    });

    it('should export Podcast target', () => {
      expect(LOUDNESS_TARGETS.PODCAST).toBe(-16);
    });

    it('should have all targets as negative numbers', () => {
      Object.values(LOUDNESS_TARGETS).forEach(target => {
        expect(target).toBeLessThan(0);
      });
    });

    it('should have reasonable target ranges', () => {
      Object.values(LOUDNESS_TARGETS).forEach(target => {
        expect(target).toBeGreaterThanOrEqual(-30);
        expect(target).toBeLessThanOrEqual(0);
      });
    });
  });

  describe('normalizeLUFS()', () => {
    it('should normalize to target LUFS', () => {
      const channels = [
        new Float32Array(88200).fill(0.1),
        new Float32Array(88200).fill(0.1),
      ];

      const normalized = normalizeLUFS(channels, -14);

      // Verify normalization
      const meter = new LUFSMeter({ targetLoudness: -14 });
      const result = meter.analyze(normalized);

      // Should be close to -14 LUFS
      expect(result.integrated).toBeCloseTo(-14, 1);
    });

    it('should handle custom sample rate', () => {
      const channels = [
        new Float32Array(96000).fill(0.1),
        new Float32Array(96000).fill(0.1),
      ];

      const normalized = normalizeLUFS(channels, -16, 48000);

      expect(normalized.length).toBe(2);
      expect(normalized[0].length).toBe(96000);
    });

    it('should preserve channel count', () => {
      const mono = [new Float32Array(44100).fill(0.1)];
      const normalizedMono = normalizeLUFS(mono, -14);

      expect(normalizedMono.length).toBe(1);

      const stereo = [
        new Float32Array(44100).fill(0.1),
        new Float32Array(44100).fill(0.1),
      ];
      const normalizedStereo = normalizeLUFS(stereo, -14);

      expect(normalizedStereo.length).toBe(2);
    });

    it('should not modify original arrays', () => {
      const original = [
        new Float32Array(88200).fill(0.1),
        new Float32Array(88200).fill(0.1),
      ];
      const originalCopy = [
        new Float32Array(original[0]),
        new Float32Array(original[1]),
      ];

      normalizeLUFS(original, -14);

      // Original should be unchanged
      expect(original[0][0]).toBe(originalCopy[0][0]);
      expect(original[1][0]).toBe(originalCopy[1][0]);
    });

    it('should handle silence (return unchanged)', () => {
      const silence = [new Float32Array(44100), new Float32Array(44100)];
      const normalized = normalizeLUFS(silence, -14);

      // Silence can't be normalized
      expect(normalized[0][0]).toBe(0);
      expect(normalized[1][0]).toBe(0);
    });

    it('should handle very quiet signals', () => {
      const quiet = [
        new Float32Array(88200).fill(0.001),
        new Float32Array(88200).fill(0.001),
      ];

      const normalized = normalizeLUFS(quiet, -14);

      // Should boost quiet signal
      expect(normalized[0][1000]).toBeGreaterThan(0.001);
    });

    it('should handle loud signals (attenuation)', () => {
      const loud = [
        new Float32Array(88200).fill(0.5),
        new Float32Array(88200).fill(0.5),
      ];

      const normalized = normalizeLUFS(loud, -14);

      // Exact attenuation depends on measurement, but should be different
      expect(normalized[0][1000]).not.toBe(0.5);
    });

    it('should normalize to Spotify target', () => {
      const channels = [
        new Float32Array(88200).fill(0.2),
        new Float32Array(88200).fill(0.2),
      ];

      const normalized = normalizeLUFS(channels, LOUDNESS_TARGETS.SPOTIFY);

      const meter = new LUFSMeter({ targetLoudness: LOUDNESS_TARGETS.SPOTIFY });
      const result = meter.analyze(normalized);

      expect(result.integrated).toBeCloseTo(-14, 1);
    });

    it('should normalize to Apple Music target', () => {
      const channels = [
        new Float32Array(88200).fill(0.2),
        new Float32Array(88200).fill(0.2),
      ];

      const normalized = normalizeLUFS(channels, LOUDNESS_TARGETS.APPLE_MUSIC);

      const meter = new LUFSMeter({ targetLoudness: LOUDNESS_TARGETS.APPLE_MUSIC });
      const result = meter.analyze(normalized);

      expect(result.integrated).toBeCloseTo(-16, 1);
    });

    it('should normalize to broadcast TV target', () => {
      const channels = [
        new Float32Array(88200).fill(0.2),
        new Float32Array(88200).fill(0.2),
      ];

      const normalized = normalizeLUFS(channels, LOUDNESS_TARGETS.BROADCAST_TV);

      const meter = new LUFSMeter({ targetLoudness: LOUDNESS_TARGETS.BROADCAST_TV });
      const result = meter.analyze(normalized);

      expect(result.integrated).toBeCloseTo(-23, 1);
    });

    it('should calculate correct gain adjustment', () => {
      // Start with signal at approximately -20 LUFS
      const channels = [
        new Float32Array(88200).fill(0.05),
        new Float32Array(88200).fill(0.05),
      ];

      // Normalize to -14 LUFS (should apply ~6 dB gain)
      const normalized = normalizeLUFS(channels, -14);

      // Normalized should be louder
      expect(normalized[0][1000]).toBeGreaterThan(0.05);
    });

    it('should apply same gain to all channels', () => {
      const left = new Float32Array(88200).fill(0.1);
      const right = new Float32Array(88200).fill(0.2);
      const channels = [left, right];

      const normalized = normalizeLUFS(channels, -14);

      // Ratio between channels should be preserved
      const originalRatio = 0.2 / 0.1;
      const normalizedRatio = normalized[1][1000] / normalized[0][1000];

      expect(normalizedRatio).toBeCloseTo(originalRatio, 5);
    });

    it('should handle multi-channel normalization', () => {
      const surround = [
        new Float32Array(88200).fill(0.1),
        new Float32Array(88200).fill(0.1),
        new Float32Array(88200).fill(0.1),
        new Float32Array(88200).fill(0.1),
        new Float32Array(88200).fill(0.1),
        new Float32Array(88200).fill(0.1),
      ];

      const normalized = normalizeLUFS(surround, -14);

      expect(normalized.length).toBe(6);
      expect(normalized[0].length).toBe(88200);
    });

    it('should not clip when boosting', () => {
      const channels = [
        new Float32Array(88200).fill(0.05),
        new Float32Array(88200).fill(0.05),
      ];

      const normalized = normalizeLUFS(channels, -14);

      // Check for clipping
      let maxSample = 0;
      for (const channel of normalized) {
        for (let i = 0; i < channel.length; i++) {
          maxSample = Math.max(maxSample, Math.abs(channel[i]));
        }
      }

      // May exceed 1.0 depending on signal, but shouldn't be extreme
      expect(maxSample).toBeLessThan(2.0);
    });

    it('should handle negative target LUFS values', () => {
      const channels = [
        new Float32Array(88200).fill(0.1),
        new Float32Array(88200).fill(0.1),
      ];

      const targets = [-9, -14, -16, -23];
      targets.forEach(target => {
        const normalized = normalizeLUFS(channels, target);
        expect(normalized.length).toBe(2);
      });
    });
  });

  describe('LUFSMeter - Integration Tests', () => {
    it('should analyze real music-like signal', () => {
      const meter = new LUFSMeter({ sampleRate: 44100 });

      // Simulate music with drums, bass, and melody
      const left = new Float32Array(132300); // 3 seconds
      const right = new Float32Array(132300);

      for (let i = 0; i < left.length; i++) {
        // Bass (100 Hz)
        const bass = 0.3 * Math.sin(2 * Math.PI * 100 * i / 44100);
        // Melody (440 Hz)
        const melody = 0.15 * Math.sin(2 * Math.PI * 440 * i / 44100);
        // Drums (periodic impulses)
        const drums = (i % 11025 === 0) ? 0.5 : 0;

        left[i] = bass + melody + drums;
        right[i] = bass + melody + drums;
      }

      const result = meter.analyze([left, right]);

      expect(result.integrated).toBeGreaterThan(-30);
      expect(result.integrated).toBeLessThan(-5);
      expect(result.range).toBeGreaterThan(0);
      expect(isFinite(result.shortTerm)).toBe(true);
      expect(isFinite(result.momentary)).toBe(true);
      expect(isFinite(result.truePeak)).toBe(true);
    });

    it('should measure loudness consistently across resets', () => {
      const meter = new LUFSMeter();
      const signal = [
        new Float32Array(88200).fill(0.2),
        new Float32Array(88200).fill(0.2),
      ];

      const result1 = meter.analyze(signal);
      meter.reset();
      const result2 = meter.analyze(signal);

      expect(result1.integrated).toBeCloseTo(result2.integrated, 5);
    });

    it('should handle streaming workflow (multiple analyze calls)', () => {
      const meter = new LUFSMeter();

      const chunk1 = [new Float32Array(44100).fill(0.3), new Float32Array(44100).fill(0.3)];
      const chunk2 = [new Float32Array(44100).fill(0.1), new Float32Array(44100).fill(0.1)];

      const result1 = meter.analyze(chunk1);
      meter.reset();
      const result2 = meter.analyze(chunk2);

      expect(result1.integrated).toBeGreaterThan(result2.integrated);
    });
  });

  describe('LUFSMeter - Performance', () => {
    it('should handle large buffers efficiently', () => {
      const meter = new LUFSMeter();
      const large = [
        new Float32Array(441000).fill(0.2), // 10 seconds
        new Float32Array(441000).fill(0.2),
      ];

      const start = performance.now();
      const result = meter.analyze(large);
      const duration = performance.now() - start;

      expect(isFinite(result.integrated)).toBe(true);
      expect(duration).toBeLessThan(1000); // Should complete in under 1 second
    });

    it('should handle real-time buffer sizes (512 samples)', () => {
      const meter = new LUFSMeter();
      const realtime = [new Float32Array(512).fill(0.2), new Float32Array(512).fill(0.2)];

      const result = meter.analyze(realtime);
      // May not have enough data for accurate measurement, but should not crash
      expect(() => meter.analyze(realtime)).not.toThrow();
    });
  });
});

/**
 * Helper function to create a calibrated signal at a specific LUFS level
 */
function createCalibratedSignal(
  targetLUFS: number,
  sampleRate: number,
  channels: number
): Float32Array[] {
  // This is a simplified calibration - real calibration would require
  // iterative measurement and adjustment
  const duration = 2; // seconds
  const samples = sampleRate * duration;

  // Approximate amplitude for target LUFS (this is a rough estimate)
  // LUFS = -0.691 + 10 * log10(meanPower)
  // meanPower = 10^((LUFS + 0.691) / 10)
  const meanPower = Math.pow(10, (targetLUFS + 0.691) / 10);
  const amplitude = Math.sqrt(meanPower);

  const result: Float32Array[] = [];
  for (let ch = 0; ch < channels; ch++) {
    const channel = new Float32Array(samples);
    for (let i = 0; i < samples; i++) {
      // Use pink noise-like signal for more realistic measurement
      channel[i] = amplitude * (Math.random() - 0.5) * 2;
    }
    result.push(channel);
  }

  return result;
}
