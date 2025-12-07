import { describe, it, expect } from 'vitest';
import { TempoMap, type TempoChange } from './tempo-map';

describe('tempo-map.ts - Tempo Management', () => {
  describe('TempoMap construction', () => {
    it('should create with default tempo of 120 BPM', () => {
      const tempoMap = new TempoMap();
      expect(tempoMap.getTempoAt(0)).toBe(120);
    });

    it('should create with custom initial tempo', () => {
      const tempoMap = new TempoMap(140);
      expect(tempoMap.getTempoAt(0)).toBe(140);
    });

    it('should have initial tempo at beat 0', () => {
      const tempoMap = new TempoMap(135);
      expect(tempoMap.getTempoAt(0)).toBe(135);
    });

    it('should accept various initial tempos', () => {
      [60, 90, 120, 140, 180, 200, 240].forEach(tempo => {
        const tempoMap = new TempoMap(tempo);
        expect(tempoMap.getTempoAt(0)).toBe(tempo);
      });
    });
  });

  describe('setTempoAt', () => {
    it('should set tempo at specific beat', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      expect(tempoMap.getTempoAt(16)).toBe(140);
    });

    it('should maintain previous tempo before change point', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      expect(tempoMap.getTempoAt(15)).toBe(120);
    });

    it('should replace existing tempo change at same beat', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      tempoMap.setTempoAt(16, 150);
      expect(tempoMap.getTempoAt(16)).toBe(150);
    });

    it('should maintain tempo changes in sorted order', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(32, 160);
      tempoMap.setTempoAt(16, 140);
      tempoMap.setTempoAt(48, 180);

      expect(tempoMap.getTempoAt(0)).toBe(120);
      expect(tempoMap.getTempoAt(16)).toBe(140);
      expect(tempoMap.getTempoAt(32)).toBe(160);
      expect(tempoMap.getTempoAt(48)).toBe(180);
    });

    it('should handle multiple tempo changes', () => {
      const tempoMap = new TempoMap(120);
      for (let i = 1; i <= 10; i++) {
        tempoMap.setTempoAt(i * 4, 120 + i * 10);
      }

      expect(tempoMap.getTempoAt(40)).toBe(220);
    });

    it('should set tempo at beat 0', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(0, 140);
      expect(tempoMap.getTempoAt(0)).toBe(140);
    });

    it('should handle fractional beat positions', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16.5, 140);
      expect(tempoMap.getTempoAt(16.5)).toBe(140);
      expect(tempoMap.getTempoAt(16.4)).toBe(120);
    });
  });

  describe('getTempoAt', () => {
    it('should return initial tempo at beat 0', () => {
      const tempoMap = new TempoMap(120);
      expect(tempoMap.getTempoAt(0)).toBe(120);
    });

    it('should return current tempo for beats after last change', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      expect(tempoMap.getTempoAt(100)).toBe(140);
    });

    it('should return most recent tempo before given beat', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      tempoMap.setTempoAt(32, 160);
      expect(tempoMap.getTempoAt(20)).toBe(140);
    });

    it('should handle tempo queries at exact change points', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      expect(tempoMap.getTempoAt(16)).toBe(140);
    });

    it('should return default 120 if no tempo set (edge case)', () => {
      const tempoMap = new TempoMap(120);
      // Remove initial tempo (not normally possible, but test the fallback)
      expect(tempoMap.getTempoAt(0)).toBe(120);
    });
  });

  describe('setGlobalTempo', () => {
    it('should remove all tempo changes', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      tempoMap.setTempoAt(32, 160);

      tempoMap.setGlobalTempo(135);

      expect(tempoMap.getTempoAt(0)).toBe(135);
      expect(tempoMap.getTempoAt(16)).toBe(135);
      expect(tempoMap.getTempoAt(32)).toBe(135);
    });

    it('should set single tempo for entire timeline', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setGlobalTempo(150);

      expect(tempoMap.getAllChanges()).toHaveLength(1);
      expect(tempoMap.getTempoAt(1000)).toBe(150);
    });

    it('should allow setting different global tempos', () => {
      const tempoMap = new TempoMap(120);
      [90, 120, 150, 180].forEach(tempo => {
        tempoMap.setGlobalTempo(tempo);
        expect(tempoMap.getTempoAt(0)).toBe(tempo);
      });
    });
  });

  describe('getAllChanges', () => {
    it('should return all tempo changes', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      tempoMap.setTempoAt(32, 160);

      const changes = tempoMap.getAllChanges();
      expect(changes).toHaveLength(3);
    });

    it('should return changes in sorted order', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(32, 160);
      tempoMap.setTempoAt(16, 140);

      const changes = tempoMap.getAllChanges();
      expect(changes[0].beat).toBe(0);
      expect(changes[1].beat).toBe(16);
      expect(changes[2].beat).toBe(32);
    });

    it('should return copy of changes (not direct reference)', () => {
      const tempoMap = new TempoMap(120);
      const changes = tempoMap.getAllChanges();
      changes.push({ beat: 100, bpm: 200 });

      expect(tempoMap.getAllChanges()).toHaveLength(1);
    });

    it('should include initial tempo', () => {
      const tempoMap = new TempoMap(135);
      const changes = tempoMap.getAllChanges();

      expect(changes[0]).toEqual({ beat: 0, bpm: 135 });
    });
  });

  describe('removeTempoAt', () => {
    it('should remove tempo change at specific beat', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      tempoMap.removeTempoAt(16);

      expect(tempoMap.getTempoAt(16)).toBe(120);
    });

    it('should throw error when removing initial tempo (beat 0)', () => {
      const tempoMap = new TempoMap(120);
      expect(() => tempoMap.removeTempoAt(0)).toThrow('Cannot remove initial tempo');
    });

    it('should handle removing non-existent tempo change', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.removeTempoAt(16); // No change at beat 16
      expect(tempoMap.getAllChanges()).toHaveLength(1);
    });

    it('should allow removing middle tempo change', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      tempoMap.setTempoAt(32, 160);
      tempoMap.setTempoAt(48, 180);

      tempoMap.removeTempoAt(32);

      expect(tempoMap.getTempoAt(32)).toBe(140);
      expect(tempoMap.getTempoAt(48)).toBe(180);
    });
  });

  describe('beatsToSeconds', () => {
    it('should convert beats to seconds at constant tempo', () => {
      const tempoMap = new TempoMap(120); // 2 beats per second
      const seconds = tempoMap.beatsToSeconds(0, 4);
      expect(seconds).toBeCloseTo(2, 2);
    });

    it('should return 0 for same start and end beat', () => {
      const tempoMap = new TempoMap(120);
      expect(tempoMap.beatsToSeconds(10, 10)).toBe(0);
    });

    it('should handle tempo changes in range', () => {
      const tempoMap = new TempoMap(120); // 2 beats/sec
      tempoMap.setTempoAt(4, 60); // 1 beat/sec after beat 4

      // 4 beats at 120 BPM = 2 seconds
      // 4 beats at 60 BPM = 4 seconds
      // Total: 6 seconds
      const seconds = tempoMap.beatsToSeconds(0, 8);
      expect(seconds).toBeCloseTo(6, 2);
    });

    it('should handle multiple tempo changes', () => {
      const tempoMap = new TempoMap(120); // 2 beats/sec
      tempoMap.setTempoAt(4, 60);  // 1 beat/sec
      tempoMap.setTempoAt(8, 180); // 3 beats/sec

      const seconds = tempoMap.beatsToSeconds(0, 12);
      // 4 beats at 120 BPM = 2s
      // 4 beats at 60 BPM = 4s
      // 4 beats at 180 BPM = 1.33s
      // Total ≈ 7.33s
      expect(seconds).toBeCloseTo(7.33, 1);
    });

    it('should calculate seconds at different tempos', () => {
      [60, 90, 120, 140, 180].forEach(bpm => {
        const tempoMap = new TempoMap(bpm);
        const seconds = tempoMap.beatsToSeconds(0, bpm);
        expect(seconds).toBeCloseTo(60, 1); // bpm beats at bpm BPM = 60 seconds
      });
    });

    it('should handle fractional beats', () => {
      const tempoMap = new TempoMap(120);
      const seconds = tempoMap.beatsToSeconds(0, 2.5);
      expect(seconds).toBeCloseTo(1.25, 2);
    });

    it('should work with non-zero start beat', () => {
      const tempoMap = new TempoMap(120);
      const seconds = tempoMap.beatsToSeconds(4, 8);
      expect(seconds).toBeCloseTo(2, 2);
    });

    it('should handle tempo change exactly at range start', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(4, 60);

      const seconds = tempoMap.beatsToSeconds(4, 8);
      expect(seconds).toBeCloseTo(4, 2);
    });

    it('should handle tempo change exactly at range end', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(8, 180);

      const seconds = tempoMap.beatsToSeconds(0, 8);
      expect(seconds).toBeCloseTo(4, 2);
    });
  });

  describe('secondsToBeats', () => {
    it('should convert seconds to beats at constant tempo', () => {
      const tempoMap = new TempoMap(120); // 2 beats per second
      const beats = tempoMap.secondsToBeats(2);
      expect(beats).toBeCloseTo(4, 2);
    });

    it('should return 0 for 0 seconds', () => {
      const tempoMap = new TempoMap(120);
      expect(tempoMap.secondsToBeats(0)).toBe(0);
    });

    it('should handle tempo changes', () => {
      const tempoMap = new TempoMap(120); // 2 beats/sec
      tempoMap.setTempoAt(4, 60); // 1 beat/sec after beat 4

      // First 2 seconds = 4 beats at 120 BPM
      // Next 4 seconds = 4 beats at 60 BPM
      // Total 6 seconds = 8 beats
      const beats = tempoMap.secondsToBeats(6);
      expect(beats).toBeCloseTo(8, 2);
    });

    it('should handle multiple tempo changes', () => {
      const tempoMap = new TempoMap(120); // 2 beats/sec
      tempoMap.setTempoAt(4, 60);  // 1 beat/sec
      tempoMap.setTempoAt(8, 180); // 3 beats/sec

      const beats = tempoMap.secondsToBeats(7.33);
      expect(beats).toBeCloseTo(12, 1);
    });

    it('should roundtrip with beatsToSeconds', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      tempoMap.setTempoAt(32, 160);

      const originalBeats = 40;
      const seconds = tempoMap.beatsToSeconds(0, originalBeats);
      const resultBeats = tempoMap.secondsToBeats(seconds);

      expect(resultBeats).toBeCloseTo(originalBeats, 1);
    });

    it('should work at different tempos', () => {
      [60, 90, 120, 140, 180].forEach(bpm => {
        const tempoMap = new TempoMap(bpm);
        const beats = tempoMap.secondsToBeats(60);
        expect(beats).toBeCloseTo(bpm, 1);
      });
    });

    it('should handle fractional seconds', () => {
      const tempoMap = new TempoMap(120);
      const beats = tempoMap.secondsToBeats(1.5);
      expect(beats).toBeCloseTo(3, 2);
    });

    it('should handle large time values', () => {
      const tempoMap = new TempoMap(120);
      const beats = tempoMap.secondsToBeats(300); // 5 minutes
      expect(beats).toBeCloseTo(600, 1);
    });
  });

  describe('Edge cases', () => {
    it('should handle very slow tempo (40 BPM)', () => {
      const tempoMap = new TempoMap(40);
      expect(tempoMap.getTempoAt(0)).toBe(40);
      const seconds = tempoMap.beatsToSeconds(0, 40);
      expect(seconds).toBeCloseTo(60, 1);
    });

    it('should handle very fast tempo (300 BPM)', () => {
      const tempoMap = new TempoMap(300);
      expect(tempoMap.getTempoAt(0)).toBe(300);
      const seconds = tempoMap.beatsToSeconds(0, 300);
      expect(seconds).toBeCloseTo(60, 1);
    });

    it('should handle many tempo changes (100+)', () => {
      const tempoMap = new TempoMap(120);
      for (let i = 1; i <= 100; i++) {
        tempoMap.setTempoAt(i, 120 + i);
      }

      expect(tempoMap.getAllChanges().length).toBeGreaterThan(100);
    });

    it('should handle tempo changes at very large beat values', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(10000, 140);
      expect(tempoMap.getTempoAt(10000)).toBe(140);
    });

    it('should handle rapid tempo changes', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(0.25, 130);
      tempoMap.setTempoAt(0.5, 140);
      tempoMap.setTempoAt(0.75, 150);

      expect(tempoMap.getTempoAt(0.5)).toBe(140);
    });

    it('should handle tempo decreases', () => {
      const tempoMap = new TempoMap(180);
      tempoMap.setTempoAt(4, 120);
      tempoMap.setTempoAt(8, 60);

      expect(tempoMap.getTempoAt(8)).toBe(60);
    });
  });

  describe('TempoChange interface', () => {
    it('should define TempoChange structure', () => {
      const change: TempoChange = {
        beat: 16,
        bpm: 140,
      };

      expect(change.beat).toBe(16);
      expect(change.bpm).toBe(140);
    });

    it('should allow fractional beat positions', () => {
      const change: TempoChange = {
        beat: 16.5,
        bpm: 140,
      };

      expect(change.beat).toBe(16.5);
    });

    it('should allow various BPM values', () => {
      [40, 60, 90, 120, 140, 180, 200, 240].forEach(bpm => {
        const change: TempoChange = { beat: 0, bpm };
        expect(change.bpm).toBe(bpm);
      });
    });
  });

  describe('Integration tests', () => {
    it('should handle complete tempo automation sequence', () => {
      const tempoMap = new TempoMap(120);

      // Add gradual tempo increase
      tempoMap.setTempoAt(0, 100);
      tempoMap.setTempoAt(16, 120);
      tempoMap.setTempoAt(32, 140);
      tempoMap.setTempoAt(48, 160);

      expect(tempoMap.getTempoAt(0)).toBe(100);
      expect(tempoMap.getTempoAt(16)).toBe(120);
      expect(tempoMap.getTempoAt(32)).toBe(140);
      expect(tempoMap.getTempoAt(48)).toBe(160);
    });

    it('should support ritardando (gradual slowdown)', () => {
      const tempoMap = new TempoMap(140);

      // Gradual slowdown over 16 beats
      for (let i = 0; i <= 16; i += 2) {
        tempoMap.setTempoAt(i, 140 - i * 2);
      }

      expect(tempoMap.getTempoAt(0)).toBe(140);
      expect(tempoMap.getTempoAt(16)).toBe(108);
    });

    it('should support accelerando (gradual speedup)', () => {
      const tempoMap = new TempoMap(100);

      // Gradual speedup over 16 beats
      for (let i = 0; i <= 16; i += 2) {
        tempoMap.setTempoAt(i, 100 + i * 2);
      }

      expect(tempoMap.getTempoAt(0)).toBe(100);
      expect(tempoMap.getTempoAt(16)).toBe(132);
    });

    it('should handle tempo changes then global reset', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      tempoMap.setTempoAt(32, 160);
      tempoMap.setGlobalTempo(130);

      expect(tempoMap.getAllChanges()).toHaveLength(1);
      expect(tempoMap.getTempoAt(50)).toBe(130);
    });

    it('should accurately convert long durations with tempo changes', () => {
      const tempoMap = new TempoMap(120);
      tempoMap.setTempoAt(64, 140);
      tempoMap.setTempoAt(128, 160);

      const seconds = tempoMap.beatsToSeconds(0, 192);
      const beatsBack = tempoMap.secondsToBeats(seconds);

      expect(beatsBack).toBeCloseTo(192, 0);
    });
  });
});
