import { describe, it, expect } from 'vitest';
import {
  tabToMIDI,
  midiToTab,
  beatsToTicks,
  ticksToBeats,
  bpmToMicroseconds,
  microsecondsToBPM,
  quantizeNotes,
  humanizeNotes,
} from './midi-converter';

describe('midi-converter.ts', () => {
  describe('tabToMIDI()', () => {
    it('should convert string 1 fret 0 (high E open) to E4', () => {
      const result = tabToMIDI(1, 0);

      expect(result.noteName).toBe('E');
      expect(result.octave).toBe(4);
      expect(result.note).toBe(64); // E4 MIDI note
    });

    it('should convert string 6 fret 0 (low E open) to E2', () => {
      const result = tabToMIDI(6, 0);

      expect(result.noteName).toBe('E');
      expect(result.octave).toBe(2);
      expect(result.note).toBe(40); // E2 MIDI note
    });

    it('should add fret offset correctly', () => {
      const result = tabToMIDI(1, 5); // High E, 5th fret = A4

      expect(result.noteName).toBe('A');
      expect(result.octave).toBe(4);
      expect(result.note).toBe(69); // A4 MIDI note (440 Hz)
    });

    it('should handle 12th fret (octave up)', () => {
      const result = tabToMIDI(6, 12); // Low E, 12th fret = E3

      expect(result.noteName).toBe('E');
      expect(result.octave).toBe(3);
      expect(result.note).toBe(52); // E3
    });

    it('should work with all strings', () => {
      const strings = [1, 2, 3, 4, 5, 6];
      const expectedNotes = ['E', 'B', 'G', 'D', 'A', 'E']; // Standard tuning

      strings.forEach((string, index) => {
        const result = tabToMIDI(string, 0);
        expect(result.noteName).toBe(expectedNotes[index]);
      });
    });

    it('should handle high frets', () => {
      const result = tabToMIDI(1, 24); // High E, 24th fret

      expect(result.note).toBe(88); // E6
    });

    it('should work with custom tuning', () => {
      // Drop D tuning: D-A-D-G-B-E
      const dropD = ['E', 'B', 'G', 'D', 'A', 'D'];
      const result = tabToMIDI(6, 0, dropD);

      expect(result.noteName).toBe('D');
      expect(result.octave).toBe(2);
    });
  });

  describe('midiToTab()', () => {
    it('should find all positions for a MIDI note', () => {
      const positions = midiToTab(69); // A4 (440 Hz)

      expect(positions.length).toBeGreaterThan(0);
      expect(positions.every(p => p.string >= 1 && p.string <= 6)).toBe(true);
      expect(positions.every(p => p.fret >= 0 && p.fret <= 24)).toBe(true);
    });

    it('should find high E open string', () => {
      const positions = midiToTab(64); // E4

      const highEOpen = positions.find(p => p.string === 1 && p.fret === 0);
      expect(highEOpen).toBeDefined();
    });

    it('should find low E open string', () => {
      const positions = midiToTab(40); // E2

      const lowEOpen = positions.find(p => p.string === 6 && p.fret === 0);
      expect(lowEOpen).toBeDefined();
    });

    it('should return multiple positions for same note', () => {
      const positions = midiToTab(69); // A4

      // A4 can be played on: string 1 fret 5, string 2 fret 10, etc.
      expect(positions.length).toBeGreaterThan(1);
    });

    it('should not return invalid fret numbers (>24)', () => {
      const positions = midiToTab(100); // Very high note

      expect(positions.every(p => p.fret <= 24)).toBe(true);
    });

    it('should return empty array for notes below low E', () => {
      const positions = midiToTab(30); // Below low E2

      expect(positions).toEqual([]);
    });

    it('should work with custom tuning', () => {
      const dropD = ['E', 'B', 'G', 'D', 'A', 'D'];
      const positions = midiToTab(38, dropD); // D2

      const dropDOpen = positions.find(p => p.string === 6 && p.fret === 0);
      expect(dropDOpen).toBeDefined();
    });

    it('should validate fret range (0-24)', () => {
      const positions = midiToTab(69); // A4

      positions.forEach(pos => {
        expect(pos.fret).toBeGreaterThanOrEqual(0);
        expect(pos.fret).toBeLessThanOrEqual(24);
      });
    });
  });

  describe('beatsToTicks()', () => {
    it('should convert 1 beat to ticks at 480 division', () => {
      const ticks = beatsToTicks(1, 480);

      expect(ticks).toBe(480);
    });

    it('should convert quarter note (0.25 beats)', () => {
      const ticks = beatsToTicks(0.25, 480);

      expect(ticks).toBe(120);
    });

    it('should convert half beat', () => {
      const ticks = beatsToTicks(0.5, 480);

      expect(ticks).toBe(240);
    });

    it('should convert 4 beats (whole note)', () => {
      const ticks = beatsToTicks(4, 480);

      expect(ticks).toBe(1920);
    });

    it('should handle different divisions', () => {
      expect(beatsToTicks(1, 96)).toBe(96);
      expect(beatsToTicks(1, 192)).toBe(192);
      expect(beatsToTicks(1, 960)).toBe(960);
    });

    it('should round to nearest tick', () => {
      const ticks = beatsToTicks(0.333, 480);

      expect(Number.isInteger(ticks)).toBe(true);
      expect(ticks).toBeCloseTo(160, 0);
    });

    it('should handle zero beats', () => {
      const ticks = beatsToTicks(0, 480);

      expect(ticks).toBe(0);
    });

    it('should handle negative beats', () => {
      const ticks = beatsToTicks(-1, 480);

      expect(ticks).toBe(-480);
    });
  });

  describe('ticksToBeats()', () => {
    it('should convert 480 ticks to 1 beat', () => {
      const beats = ticksToBeats(480, 480);

      expect(beats).toBe(1);
    });

    it('should convert 120 ticks to 0.25 beats', () => {
      const beats = ticksToBeats(120, 480);

      expect(beats).toBe(0.25);
    });

    it('should handle different divisions', () => {
      expect(ticksToBeats(96, 96)).toBe(1);
      expect(ticksToBeats(192, 192)).toBe(1);
      expect(ticksToBeats(960, 960)).toBe(1);
    });

    it('should be inverse of beatsToTicks', () => {
      const original = 2.5;
      const ticks = beatsToTicks(original, 480);
      const result = ticksToBeats(ticks, 480);

      expect(result).toBeCloseTo(original, 10);
    });

    it('should handle zero ticks', () => {
      const beats = ticksToBeats(0, 480);

      expect(beats).toBe(0);
    });

    it('should handle fractional results', () => {
      const beats = ticksToBeats(100, 480);

      expect(beats).toBeCloseTo(0.208, 3);
    });
  });

  describe('bpmToMicroseconds()', () => {
    it('should convert 120 BPM to microseconds', () => {
      const microseconds = bpmToMicroseconds(120);

      // 60,000,000 / 120 = 500,000
      expect(microseconds).toBe(500000);
    });

    it('should convert 60 BPM (1 second per beat)', () => {
      const microseconds = bpmToMicroseconds(60);

      expect(microseconds).toBe(1000000); // 1 second
    });

    it('should convert 240 BPM (fast tempo)', () => {
      const microseconds = bpmToMicroseconds(240);

      expect(microseconds).toBe(250000);
    });

    it('should convert 90 BPM', () => {
      const microseconds = bpmToMicroseconds(90);

      // 60,000,000 / 90 = 666,667
      expect(microseconds).toBeCloseTo(666667, 0);
    });

    it('should handle very slow tempo (30 BPM)', () => {
      const microseconds = bpmToMicroseconds(30);

      expect(microseconds).toBe(2000000);
    });

    it('should handle very fast tempo (300 BPM)', () => {
      const microseconds = bpmToMicroseconds(300);

      expect(microseconds).toBe(200000);
    });

    it('should round to integer', () => {
      const microseconds = bpmToMicroseconds(133);

      expect(Number.isInteger(microseconds)).toBe(true);
    });
  });

  describe('microsecondsToBPM()', () => {
    it('should convert 500,000 microseconds to 120 BPM', () => {
      const bpm = microsecondsToBPM(500000);

      expect(bpm).toBe(120);
    });

    it('should convert 1,000,000 microseconds to 60 BPM', () => {
      const bpm = microsecondsToBPM(1000000);

      expect(bpm).toBe(60);
    });

    it('should be inverse of bpmToMicroseconds', () => {
      const original = 145;
      const microseconds = bpmToMicroseconds(original);
      const result = microsecondsToBPM(microseconds);

      expect(result).toBeCloseTo(original, 0);
    });

    it('should handle typical MIDI tempos', () => {
      const tempos = [60, 90, 120, 140, 180];

      tempos.forEach(tempo => {
        const microseconds = bpmToMicroseconds(tempo);
        const result = microsecondsToBPM(microseconds);
        expect(result).toBeCloseTo(tempo, 0);
      });
    });

    it('should round to integer', () => {
      const bpm = microsecondsToBPM(666667);

      expect(Number.isInteger(bpm)).toBe(true);
    });
  });

  describe('quantizeNotes()', () => {
    it('should quantize to 16th note grid', () => {
      const notes = [
        { note: 60, startTime: 0.12, duration: 0.5, velocity: 100 },
        { note: 62, startTime: 0.38, duration: 0.5, velocity: 100 },
      ];

      const quantized = quantizeNotes(notes, 0.25); // Quarter note grid

      expect(quantized[0].startTime).toBe(0); // Snapped to 0
      expect(quantized[1].startTime).toBeCloseTo(0.25, 2); // Snapped to 0.25 or 0.5
    });

    it('should snap to nearest grid position', () => {
      const notes = [
        { note: 60, startTime: 0.63, duration: 0.5, velocity: 100 },
      ];

      const quantized = quantizeNotes(notes, 0.25);

      // 0.63 is closer to 0.75 than 0.5
      expect(quantized[0].startTime).toBeCloseTo(0.75, 2);
    });

    it('should handle 8th note grid', () => {
      const notes = [
        { note: 60, startTime: 0.11, duration: 0.5, velocity: 100 },
        { note: 62, startTime: 0.89, duration: 0.5, velocity: 100 },
      ];

      const quantized = quantizeNotes(notes, 0.5); // Half beat grid

      expect(quantized[0].startTime).toBe(0);
      expect(quantized[1].startTime).toBe(1);
    });

    it('should not modify other note properties', () => {
      const notes = [
        { note: 60, startTime: 0.12, duration: 0.5, velocity: 100 },
      ];

      const quantized = quantizeNotes(notes, 0.25);

      expect(quantized[0].note).toBe(60);
      expect(quantized[0].duration).toBe(0.5);
      expect(quantized[0].velocity).toBe(100);
    });

    it('should handle empty array', () => {
      const quantized = quantizeNotes([], 0.25);

      expect(quantized).toEqual([]);
    });

    it('should handle notes already on grid', () => {
      const notes = [
        { note: 60, startTime: 0.25, duration: 0.5, velocity: 100 },
        { note: 62, startTime: 0.5, duration: 0.5, velocity: 100 },
      ];

      const quantized = quantizeNotes(notes, 0.25);

      expect(quantized[0].startTime).toBe(0.25);
      expect(quantized[1].startTime).toBe(0.5);
    });

    it('should work with different grid sizes', () => {
      const notes = [{ note: 60, startTime: 0.333, duration: 0.5, velocity: 100 }];

      const quantized16th = quantizeNotes(notes, 0.25);
      const quantized8th = quantizeNotes(notes, 0.5);

      expect(quantized16th[0].startTime).not.toBe(quantized8th[0].startTime);
    });
  });

  describe('humanizeNotes()', () => {
    it('should add timing variation', () => {
      const notes = [
        { note: 60, startTime: 1.0, duration: 0.5, velocity: 100 },
      ];

      const humanized = humanizeNotes(notes, { timingVariation: 0.05 });

      expect(humanized[0].startTime).not.toBe(1.0);
      expect(Math.abs(humanized[0].startTime - 1.0)).toBeLessThanOrEqual(0.05);
    });

    it('should add velocity variation', () => {
      const notes = [
        { note: 60, startTime: 1.0, duration: 0.5, velocity: 100 },
      ];

      const humanized = humanizeNotes(notes, { velocityVariation: 10 });

      expect(humanized[0].velocity).not.toBe(100);
      expect(Math.abs(humanized[0].velocity - 100)).toBeLessThanOrEqual(10);
    });

    it('should clamp velocity to valid range (1-127)', () => {
      const notes = [
        { note: 60, startTime: 1.0, duration: 0.5, velocity: 1 },
        { note: 62, startTime: 1.0, duration: 0.5, velocity: 127 },
      ];

      const humanized = humanizeNotes(notes, { velocityVariation: 50 });

      humanized.forEach(note => {
        expect(note.velocity).toBeGreaterThanOrEqual(1);
        expect(note.velocity).toBeLessThanOrEqual(127);
      });
    });

    it('should use default variations when not specified', () => {
      const notes = [
        { note: 60, startTime: 1.0, duration: 0.5, velocity: 100 },
      ];

      const humanized = humanizeNotes(notes);

      // Should still add some variation with defaults
      expect(humanized[0]).toBeDefined();
    });

    it('should handle empty array', () => {
      const humanized = humanizeNotes([]);

      expect(humanized).toEqual([]);
    });

    it('should not modify original notes', () => {
      const notes = [
        { note: 60, startTime: 1.0, duration: 0.5, velocity: 100 },
      ];
      const original = JSON.parse(JSON.stringify(notes));

      humanizeNotes(notes);

      expect(notes).toEqual(original);
    });

    it('should preserve note pitch and duration', () => {
      const notes = [
        { note: 60, startTime: 1.0, duration: 0.5, velocity: 100 },
      ];

      const humanized = humanizeNotes(notes);

      expect(humanized[0].note).toBe(60);
      expect(humanized[0].duration).toBe(0.5);
    });

    it('should add random variation (not deterministic)', () => {
      const notes = [
        { note: 60, startTime: 1.0, duration: 0.5, velocity: 100 },
      ];

      const humanized1 = humanizeNotes(notes);
      const humanized2 = humanizeNotes(notes);

      // Two calls should produce different results (very likely)
      expect(humanized1[0].startTime).not.toBe(humanized2[0].startTime);
    });

    it('should handle multiple notes independently', () => {
      const notes = [
        { note: 60, startTime: 1.0, duration: 0.5, velocity: 100 },
        { note: 62, startTime: 2.0, duration: 0.5, velocity: 100 },
        { note: 64, startTime: 3.0, duration: 0.5, velocity: 100 },
      ];

      const humanized = humanizeNotes(notes, { timingVariation: 0.02, velocityVariation: 5 });

      // Each note should have independent variation
      const timings = humanized.map(n => n.startTime);
      const velocities = humanized.map(n => n.velocity);

      expect(new Set(timings).size).toBe(3); // All different (very likely)
      expect(new Set(velocities).size).toBeGreaterThanOrEqual(2); // At least some different
    });
  });

  describe('roundtrip conversions', () => {
    it('should convert beats→ticks→beats', () => {
      const original = 3.5;
      const ticks = beatsToTicks(original, 480);
      const result = ticksToBeats(ticks, 480);

      expect(result).toBeCloseTo(original, 10);
    });

    it('should convert BPM→microseconds→BPM', () => {
      const original = 137;
      const microseconds = bpmToMicroseconds(original);
      const result = microsecondsToBPM(microseconds);

      expect(result).toBeCloseTo(original, 0);
    });

    it('should convert tab→MIDI→tab', () => {
      const original = { string: 3, fret: 5 };
      const { note } = tabToMIDI(original.string, original.fret);
      const positions = midiToTab(note);

      const match = positions.find(p => p.string === original.string && p.fret === original.fret);
      expect(match).toBeDefined();
    });
  });
});
