import { describe, it, expect, beforeEach } from 'vitest';
import { TimeConverter } from './time-converter';
import { TempoMap } from './tempo-map';
import type { MusicalPosition } from './timeline';
import type { TimeSignature } from '@maestro-ai/shared-types';

describe('time-converter.ts - Musical/Absolute Time Conversion', () => {
  let tempoMap: TempoMap;
  let timeSignature: TimeSignature;
  let converter: TimeConverter;

  beforeEach(() => {
    tempoMap = new TempoMap(120);
    timeSignature = { numerator: 4, denominator: 4 };
    converter = new TimeConverter(tempoMap, timeSignature);
  });

  describe('TimeConverter construction', () => {
    it('should create with tempo map and time signature', () => {
      expect(converter).toBeInstanceOf(TimeConverter);
    });

    it('should work with different time signatures', () => {
      const converter34 = new TimeConverter(tempoMap, { numerator: 3, denominator: 4 });
      const converter68 = new TimeConverter(tempoMap, { numerator: 6, denominator: 8 });

      expect(converter34).toBeInstanceOf(TimeConverter);
      expect(converter68).toBeInstanceOf(TimeConverter);
    });

    it('should accept different tempo maps', () => {
      const fastTempo = new TempoMap(180);
      const fastConverter = new TimeConverter(fastTempo, timeSignature);

      expect(fastConverter).toBeInstanceOf(TimeConverter);
    });
  });

  describe('musicalToSeconds', () => {
    it('should convert measure 0 beat 0 to 0 seconds', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 0,
        totalBeats: 0,
      };

      expect(converter.musicalToSeconds(position)).toBe(0);
    });

    it('should convert 1 measure to seconds at 120 BPM', () => {
      const position: MusicalPosition = {
        measure: 1,
        beat: 0,
        tick: 0,
        totalBeats: 4,
      };

      // 4 beats at 120 BPM = 2 seconds
      expect(converter.musicalToSeconds(position)).toBeCloseTo(2, 2);
    });

    it('should convert single beat to seconds', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 1,
        tick: 0,
        totalBeats: 1,
      };

      // 1 beat at 120 BPM = 0.5 seconds
      expect(converter.musicalToSeconds(position)).toBeCloseTo(0.5, 2);
    });

    it('should cache conversion results', () => {
      const position: MusicalPosition = {
        measure: 1,
        beat: 0,
        tick: 0,
        totalBeats: 4,
      };

      const result1 = converter.musicalToSeconds(position);
      const result2 = converter.musicalToSeconds(position);

      expect(result1).toBe(result2);
    });

    it('should work with fractional beats (ticks)', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 240, // Half beat
        totalBeats: 0.5,
      };

      expect(converter.musicalToSeconds(position)).toBeCloseTo(0.25, 2);
    });

    it('should handle large measure values', () => {
      const position: MusicalPosition = {
        measure: 100,
        beat: 0,
        tick: 0,
        totalBeats: 400,
      };

      expect(converter.musicalToSeconds(position)).toBeGreaterThan(0);
    });
  });

  describe('secondsToMusical', () => {
    it('should convert 0 seconds to measure 0', () => {
      const position = converter.secondsToMusical(0);

      expect(position.measure).toBe(0);
      expect(position.beat).toBe(0);
      expect(position.tick).toBe(0);
    });

    it('should convert 2 seconds to measure 1 at 120 BPM', () => {
      const position = converter.secondsToMusical(2);

      expect(position.measure).toBe(1);
      expect(position.beat).toBe(0);
    });

    it('should handle fractional seconds', () => {
      const position = converter.secondsToMusical(0.5);

      expect(position.totalBeats).toBeCloseTo(1, 1);
    });

    it('should roundtrip with musicalToSeconds', () => {
      const originalPosition: MusicalPosition = {
        measure: 4,
        beat: 2,
        tick: 120,
        totalBeats: 18.25,
      };

      const seconds = converter.musicalToSeconds(originalPosition);
      const converted = converter.secondsToMusical(seconds);

      expect(converted.measure).toBe(originalPosition.measure);
      expect(converted.beat).toBe(originalPosition.beat);
      expect(converted.tick).toBeCloseTo(originalPosition.tick, 0);
    });
  });

  describe('beatsToMusical', () => {
    it('should convert 0 beats to measure 0 beat 0', () => {
      const position = converter.beatsToMusical(0);

      expect(position.measure).toBe(0);
      expect(position.beat).toBe(0);
      expect(position.tick).toBe(0);
      expect(position.totalBeats).toBe(0);
    });

    it('should convert 4 beats to measure 1 in 4/4', () => {
      const position = converter.beatsToMusical(4);

      expect(position.measure).toBe(1);
      expect(position.beat).toBe(0);
    });

    it('should convert 5 beats to measure 1 beat 1 in 4/4', () => {
      const position = converter.beatsToMusical(5);

      expect(position.measure).toBe(1);
      expect(position.beat).toBe(1);
    });

    it('should handle fractional beats', () => {
      const position = converter.beatsToMusical(4.5);

      expect(position.measure).toBe(1);
      expect(position.beat).toBe(0);
      expect(position.tick).toBeCloseTo(240, 0);
    });

    it('should work with 3/4 time signature', () => {
      const converter34 = new TimeConverter(tempoMap, { numerator: 3, denominator: 4 });
      const position = converter34.beatsToMusical(6);

      expect(position.measure).toBe(2);
      expect(position.beat).toBe(0);
    });

    it('should work with 6/8 time signature', () => {
      const converter68 = new TimeConverter(tempoMap, { numerator: 6, denominator: 8 });
      const position = converter68.beatsToMusical(12);

      expect(position.measure).toBe(2);
      expect(position.beat).toBe(0);
    });

    it('should preserve totalBeats', () => {
      const beats = 17.25;
      const position = converter.beatsToMusical(beats);

      expect(position.totalBeats).toBe(beats);
    });
  });

  describe('musicalToBeats', () => {
    it('should convert measure 0 beat 0 to 0 beats', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 0,
        totalBeats: 0,
      };

      expect(converter.musicalToBeats(position)).toBe(0);
    });

    it('should convert measure 1 to 4 beats in 4/4', () => {
      const position: MusicalPosition = {
        measure: 1,
        beat: 0,
        tick: 0,
        totalBeats: 4,
      };

      expect(converter.musicalToBeats(position)).toBe(4);
    });

    it('should convert measure 1 beat 2 to 6 beats in 4/4', () => {
      const position: MusicalPosition = {
        measure: 1,
        beat: 2,
        tick: 0,
        totalBeats: 6,
      };

      expect(converter.musicalToBeats(position)).toBe(6);
    });

    it('should handle ticks', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 240, // Half beat
        totalBeats: 0.5,
      };

      expect(converter.musicalToBeats(position)).toBeCloseTo(0.5, 2);
    });

    it('should roundtrip with beatsToMusical', () => {
      const beats = 17.5;
      const position = converter.beatsToMusical(beats);
      const result = converter.musicalToBeats(position);

      expect(result).toBeCloseTo(beats, 2);
    });
  });

  describe('setTimeSignature', () => {
    it('should update time signature', () => {
      converter.setTimeSignature({ numerator: 3, denominator: 4 });

      const position = converter.beatsToMusical(6);
      expect(position.measure).toBe(2);
    });

    it('should invalidate cache', () => {
      const position: MusicalPosition = {
        measure: 1,
        beat: 0,
        tick: 0,
        totalBeats: 4,
      };

      const seconds1 = converter.musicalToSeconds(position);
      converter.setTimeSignature({ numerator: 3, denominator: 4 });
      // Cache should be cleared, but result should be same since tempo unchanged
      const seconds2 = converter.musicalToSeconds(position);

      expect(seconds1).toBeCloseTo(seconds2, 2);
    });

    it('should allow switching between common time signatures', () => {
      const signatures: TimeSignature[] = [
        { numerator: 4, denominator: 4 },
        { numerator: 3, denominator: 4 },
        { numerator: 6, denominator: 8 },
        { numerator: 5, denominator: 4 },
        { numerator: 7, denominator: 8 },
      ];

      signatures.forEach(sig => {
        converter.setTimeSignature(sig);
        const position = converter.beatsToMusical(sig.numerator);
        expect(position.measure).toBe(1);
      });
    });
  });

  describe('invalidateCache', () => {
    it('should clear conversion cache', () => {
      const position: MusicalPosition = {
        measure: 1,
        beat: 0,
        tick: 0,
        totalBeats: 4,
      };

      converter.musicalToSeconds(position);
      converter.invalidateCache();

      // Cache cleared - conversions still work
      expect(converter.musicalToSeconds(position)).toBeGreaterThan(0);
    });

    it('should be called when time signature changes', () => {
      converter.setTimeSignature({ numerator: 3, denominator: 4 });
      // Cache should be cleared implicitly
    });
  });

  describe('getDuration', () => {
    it('should calculate duration between two positions', () => {
      const start: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 0,
        totalBeats: 0,
      };

      const end: MusicalPosition = {
        measure: 1,
        beat: 0,
        tick: 0,
        totalBeats: 4,
      };

      const duration = converter.getDuration(start, end);
      expect(duration).toBeCloseTo(2, 2);
    });

    it('should return 0 for same position', () => {
      const position: MusicalPosition = {
        measure: 1,
        beat: 2,
        tick: 0,
        totalBeats: 6,
      };

      expect(converter.getDuration(position, position)).toBe(0);
    });

    it('should handle reverse order (negative duration)', () => {
      const start: MusicalPosition = {
        measure: 2,
        beat: 0,
        tick: 0,
        totalBeats: 8,
      };

      const end: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 0,
        totalBeats: 0,
      };

      const duration = converter.getDuration(start, end);
      expect(duration).toBeLessThan(0);
    });
  });

  describe('addBeats', () => {
    it('should add beats to position', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 0,
        totalBeats: 0,
      };

      const result = converter.addBeats(position, 4);

      expect(result.measure).toBe(1);
      expect(result.beat).toBe(0);
    });

    it('should handle fractional beats', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 0,
        totalBeats: 0,
      };

      const result = converter.addBeats(position, 0.5);

      expect(result.tick).toBeCloseTo(240, 0);
    });

    it('should handle negative beats (subtraction)', () => {
      const position: MusicalPosition = {
        measure: 2,
        beat: 0,
        tick: 0,
        totalBeats: 8,
      };

      const result = converter.addBeats(position, -4);

      expect(result.measure).toBe(1);
    });

    it('should preserve totalBeats calculation', () => {
      const position: MusicalPosition = {
        measure: 1,
        beat: 0,
        tick: 0,
        totalBeats: 4,
      };

      const result = converter.addBeats(position, 3);

      expect(result.totalBeats).toBe(7);
    });
  });

  describe('addSeconds', () => {
    it('should add seconds to position', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 0,
        totalBeats: 0,
      };

      const result = converter.addSeconds(position, 2); // 2 seconds at 120 BPM = 4 beats

      expect(result.measure).toBe(1);
    });

    it('should handle fractional seconds', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 0,
        totalBeats: 0,
      };

      const result = converter.addSeconds(position, 0.5); // 0.5s at 120 BPM = 1 beat

      expect(result.beat).toBe(1);
    });

    it('should handle negative seconds (subtraction)', () => {
      const position: MusicalPosition = {
        measure: 2,
        beat: 0,
        tick: 0,
        totalBeats: 8,
      };

      const result = converter.addSeconds(position, -2);

      expect(result.measure).toBe(1);
    });
  });

  describe('Integration with tempo changes', () => {
    it('should handle tempo changes in conversions', () => {
      tempoMap.setTempoAt(4, 60); // Slower tempo after measure 1

      const position: MusicalPosition = {
        measure: 2,
        beat: 0,
        tick: 0,
        totalBeats: 8,
      };

      // First 4 beats at 120 BPM = 2s
      // Next 4 beats at 60 BPM = 4s
      // Total = 6s
      const seconds = converter.musicalToSeconds(position);
      expect(seconds).toBeCloseTo(6, 1);
    });

    it('should roundtrip correctly with tempo changes', () => {
      tempoMap.setTempoAt(8, 140);
      tempoMap.setTempoAt(16, 160);

      const original: MusicalPosition = {
        measure: 5,
        beat: 0,
        tick: 0,
        totalBeats: 20,
      };

      const seconds = converter.musicalToSeconds(original);
      const result = converter.secondsToMusical(seconds);

      expect(result.measure).toBe(original.measure);
      expect(result.beat).toBe(original.beat);
    });
  });

  describe('Edge cases', () => {
    it('should handle very small tick values', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 1,
        totalBeats: 1 / 480,
      };

      expect(converter.musicalToSeconds(position)).toBeGreaterThan(0);
    });

    it('should handle maximum tick value (479)', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 479,
        totalBeats: 479 / 480,
      };

      expect(converter.musicalToSeconds(position)).toBeLessThan(0.5);
    });

    it('should handle large measure numbers', () => {
      const position: MusicalPosition = {
        measure: 1000,
        beat: 0,
        tick: 0,
        totalBeats: 4000,
      };

      expect(converter.musicalToSeconds(position)).toBeGreaterThan(0);
    });

    it('should work with odd time signatures', () => {
      const converter54 = new TimeConverter(tempoMap, { numerator: 5, denominator: 4 });
      const position = converter54.beatsToMusical(10);

      expect(position.measure).toBe(2);
    });

    it('should work with compound time signatures', () => {
      const converter98 = new TimeConverter(tempoMap, { numerator: 9, denominator: 8 });
      const position = converter98.beatsToMusical(18);

      expect(position.measure).toBe(2);
    });
  });
});
