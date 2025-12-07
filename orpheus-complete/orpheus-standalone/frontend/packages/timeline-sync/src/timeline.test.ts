import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { Timeline, type TimelineConfig, type MusicalPosition, type AbsolutePosition } from './timeline';

describe('timeline.ts - Unified Timeline System', () => {
  let timeline: Timeline;
  let config: TimelineConfig;

  beforeEach(() => {
    vi.useFakeTimers();
    config = {
      initialTempo: 120,
      timeSignature: { numerator: 4, denominator: 4 },
      sampleRate: 48000,
    };
    timeline = new Timeline(config);
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  describe('Timeline construction', () => {
    it('should create timeline with config', () => {
      expect(timeline).toBeInstanceOf(Timeline);
    });

    it('should accept different sample rates', () => {
      [44100, 48000, 96000].forEach(sampleRate => {
        const tl = new Timeline({ ...config, sampleRate });
        expect(tl).toBeInstanceOf(Timeline);
      });
    });

    it('should accept different time signatures', () => {
      const tl34 = new Timeline({ ...config, timeSignature: { numerator: 3, denominator: 4 } });
      const tl68 = new Timeline({ ...config, timeSignature: { numerator: 6, denominator: 8 } });

      expect(tl34).toBeInstanceOf(Timeline);
      expect(tl68).toBeInstanceOf(Timeline);
    });

    it('should accept different initial tempos', () => {
      [60, 120, 180].forEach(tempo => {
        const tl = new Timeline({ ...config, initialTempo: tempo });
        expect(tl).toBeInstanceOf(Timeline);
      });
    });
  });

  describe('musicalToAbsolute', () => {
    it('should convert musical position to absolute time', () => {
      const musical: MusicalPosition = {
        measure: 1,
        beat: 0,
        tick: 0,
        totalBeats: 4,
      };

      const absolute = timeline.musicalToAbsolute(musical);

      expect(absolute.seconds).toBeCloseTo(2, 1);
      expect(absolute.samples).toBeGreaterThan(0);
      expect(absolute.frames).toBeGreaterThan(0);
    });

    it('should calculate samples correctly', () => {
      const musical: MusicalPosition = {
        measure: 0,
        beat: 1,
        tick: 0,
        totalBeats: 1,
      };

      const absolute = timeline.musicalToAbsolute(musical);

      // 1 beat at 120 BPM = 0.5s = 24000 samples at 48kHz
      expect(absolute.samples).toBeCloseTo(24000, -2);
    });

    it('should calculate frames at 30fps', () => {
      const musical: MusicalPosition = {
        measure: 1,
        beat: 0,
        tick: 0,
        totalBeats: 4,
      };

      const absolute = timeline.musicalToAbsolute(musical);

      // 2 seconds at 30fps = 60 frames
      expect(absolute.frames).toBeCloseTo(60, 0);
    });
  });

  describe('absoluteToMusical', () => {
    it('should convert absolute time to musical position', () => {
      const absolute: AbsolutePosition = {
        seconds: 2,
        samples: 96000,
        frames: 60,
      };

      const musical = timeline.absoluteToMusical(absolute);

      expect(musical.measure).toBe(1);
      expect(musical.beat).toBe(0);
    });

    it('should roundtrip with musicalToAbsolute', () => {
      const original: MusicalPosition = {
        measure: 2,
        beat: 3,
        tick: 120,
        totalBeats: 11.25,
      };

      const absolute = timeline.musicalToAbsolute(original);
      const result = timeline.absoluteToMusical(absolute);

      expect(result.measure).toBe(original.measure);
      expect(result.beat).toBe(original.beat);
    });
  });

  describe('snapToGrid', () => {
    it('should snap to quarter note grid', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 120, // Slightly off
        totalBeats: 0.25,
      };

      const snapped = timeline.snapToGrid(position, 1);

      expect(snapped.totalBeats).toBe(0);
    });

    it('should snap to eighth note grid', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 260, // Slightly more than half beat
        totalBeats: 0.54,
      };

      const snapped = timeline.snapToGrid(position, 0.5);

      expect(snapped.totalBeats).toBe(0.5);
    });

    it('should snap to sixteenth note grid', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 130, // Slightly more than quarter beat
        totalBeats: 0.27,
      };

      const snapped = timeline.snapToGrid(position, 0.25);

      expect(snapped.totalBeats).toBe(0.25);
    });
  });

  describe('Playhead control', () => {
    it('should start playback', () => {
      timeline.play();
      expect(timeline.isPlaying()).toBe(true);
    });

    it('should pause playback', () => {
      timeline.play();
      timeline.pause();
      expect(timeline.isPlaying()).toBe(false);
    });

    it('should stop and return to start', () => {
      timeline.play();
      vi.advanceTimersByTime(2000);
      timeline.stop();

      const position = timeline.getPosition();
      expect(position.musical.measure).toBe(0);
      expect(position.musical.beat).toBe(0);
    });

    it('should set position (musical)', () => {
      const musical: MusicalPosition = {
        measure: 2,
        beat: 0,
        tick: 0,
        totalBeats: 8,
      };

      timeline.setPosition(musical);
      const position = timeline.getPosition();

      expect(position.musical.measure).toBe(2);
    });

    it('should set position (absolute)', () => {
      const absolute: AbsolutePosition = {
        seconds: 2,
        samples: 96000,
        frames: 60,
      };

      timeline.setPosition(absolute);
      const position = timeline.getPosition();

      expect(position.absolute.seconds).toBeCloseTo(2, 1);
    });

    it('should get position in both formats', () => {
      timeline.play();
      vi.advanceTimersByTime(1000);

      const position = timeline.getPosition();

      expect(position.musical).toBeDefined();
      expect(position.absolute).toBeDefined();
      expect(position.absolute.seconds).toBeCloseTo(1, 1);
    });
  });

  describe('Tempo management', () => {
    it('should set global tempo', () => {
      timeline.setGlobalTempo(140);
      expect(timeline.getCurrentTempo()).toBe(140);
    });

    it('should set tempo at specific position', () => {
      const position: MusicalPosition = {
        measure: 4,
        beat: 0,
        tick: 0,
        totalBeats: 16,
      };

      timeline.setTempoAt(position, 140);
      expect(timeline.getTempoAt(position)).toBe(140);
    });

    it('should get current tempo at playhead', () => {
      timeline.setGlobalTempo(135);
      expect(timeline.getCurrentTempo()).toBe(135);
    });
  });

  describe('Time signature', () => {
    it('should set time signature', () => {
      timeline.setTimeSignature({ numerator: 3, denominator: 4 });
      expect(timeline.getTimeSignature()).toEqual({ numerator: 3, denominator: 4 });
    });

    it('should get time signature', () => {
      const ts = timeline.getTimeSignature();
      expect(ts.numerator).toBe(4);
      expect(ts.denominator).toBe(4);
    });
  });

  describe('Markers', () => {
    it('should add marker', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 0,
        totalBeats: 0,
      };

      const id = timeline.addMarker(position, 'Intro', 'section');
      expect(id).toBeDefined();
    });

    it('should remove marker', () => {
      const position: MusicalPosition = {
        measure: 0,
        beat: 0,
        tick: 0,
        totalBeats: 0,
      };

      const id = timeline.addMarker(position, 'Test', 'custom');
      timeline.removeMarker(id);

      const markers = timeline.getMarkers();
      expect(markers.find(m => m.id === id)).toBeUndefined();
    });

    it('should get all markers', () => {
      const pos1: MusicalPosition = { measure: 0, beat: 0, tick: 0, totalBeats: 0 };
      const pos2: MusicalPosition = { measure: 4, beat: 0, tick: 0, totalBeats: 16 };

      timeline.addMarker(pos1, 'A', 'section');
      timeline.addMarker(pos2, 'B', 'section');

      expect(timeline.getMarkers()).toHaveLength(2);
    });

    it('should get markers in range', () => {
      const pos1: MusicalPosition = { measure: 0, beat: 0, tick: 0, totalBeats: 0 };
      const pos2: MusicalPosition = { measure: 4, beat: 0, tick: 0, totalBeats: 16 };
      const pos3: MusicalPosition = { measure: 8, beat: 0, tick: 0, totalBeats: 32 };

      timeline.addMarker(pos1, 'A', 'section');
      timeline.addMarker(pos2, 'B', 'section');
      timeline.addMarker(pos3, 'C', 'section');

      const inRange = timeline.getMarkersInRange(pos1, pos2);
      expect(inRange.length).toBeGreaterThan(0);
    });

    it('should use correct default colors for marker types', () => {
      const pos: MusicalPosition = { measure: 0, beat: 0, tick: 0, totalBeats: 0 };

      const sectionId = timeline.addMarker(pos, 'Section', 'section');
      const rehearsalId = timeline.addMarker(pos, 'Rehearsal', 'rehearsal');
      const customId = timeline.addMarker(pos, 'Custom', 'custom');

      const markers = timeline.getMarkers();
      const section = markers.find(m => m.id === sectionId);
      const rehearsal = markers.find(m => m.id === rehearsalId);
      const custom = markers.find(m => m.id === customId);

      expect(section?.color).toBe('#3b82f6');
      expect(rehearsal?.color).toBe('#10b981');
      expect(custom?.color).toBe('#6b7280');
    });
  });

  describe('Loop management', () => {
    it('should set loop range', () => {
      const start: MusicalPosition = { measure: 0, beat: 0, tick: 0, totalBeats: 0 };
      const end: MusicalPosition = { measure: 4, beat: 0, tick: 0, totalBeats: 16 };

      timeline.setLoopRange(start, end);
      const range = timeline.getLoopRange();

      expect(range?.start).toEqual(start);
      expect(range?.end).toEqual(end);
    });

    it('should enable looping', () => {
      timeline.setLoopEnabled(true);
      expect(timeline.isLoopEnabled()).toBe(true);
    });

    it('should disable looping', () => {
      timeline.setLoopEnabled(true);
      timeline.setLoopEnabled(false);
      expect(timeline.isLoopEnabled()).toBe(false);
    });

    it('should loop back when reaching end', () => {
      const start: MusicalPosition = { measure: 0, beat: 0, tick: 0, totalBeats: 0 };
      const end: MusicalPosition = { measure: 1, beat: 0, tick: 0, totalBeats: 4 };

      timeline.setLoopRange(start, end);
      timeline.setLoopEnabled(true);
      timeline.play();

      timeline.update(3); // Past loop end

      const position = timeline.getPosition();
      expect(position.absolute.seconds).toBeLessThan(3);
    });
  });

  describe('update', () => {
    it('should update playback position', () => {
      timeline.play();
      timeline.update(1);

      const position = timeline.getPosition();
      expect(position.absolute.seconds).toBeCloseTo(1, 1);
    });

    it('should not update when stopped', () => {
      timeline.update(1);

      const position = timeline.getPosition();
      expect(position.absolute.seconds).toBe(0);
    });

    it('should handle loop wraparound', () => {
      const start: MusicalPosition = { measure: 0, beat: 0, tick: 0, totalBeats: 0 };
      const end: MusicalPosition = { measure: 1, beat: 0, tick: 0, totalBeats: 4 };

      timeline.setLoopRange(start, end);
      timeline.setLoopEnabled(true);
      timeline.play();

      timeline.update(5); // Past loop end

      const position = timeline.getPosition();
      // Should have wrapped back
      expect(position.absolute.seconds).toBeLessThan(5);
    });
  });

  describe('getDuration', () => {
    it('should calculate duration', () => {
      const end: MusicalPosition = { measure: 4, beat: 0, tick: 0, totalBeats: 16 };
      const duration = timeline.getDuration(end);

      expect(duration.musical).toBe(16);
      expect(duration.absolute).toBeCloseTo(8, 1);
    });
  });

  describe('formatPosition', () => {
    it('should format as measures', () => {
      const position: MusicalPosition = { measure: 1, beat: 2, tick: 120, totalBeats: 6.25 };
      const formatted = timeline.formatPosition(position, 'measures');

      expect(formatted).toContain('2:3'); // measure+1:beat+1
    });

    it('should format as time', () => {
      const position: MusicalPosition = { measure: 1, beat: 0, tick: 0, totalBeats: 4 };
      const formatted = timeline.formatPosition(position, 'time');

      expect(formatted).toMatch(/\d+:\d{2}\.\d{2}/);
    });
  });

  describe('onPositionChange', () => {
    it('should subscribe to position changes', () => {
      const callback = vi.fn();
      timeline.onPositionChange(callback);

      timeline.play();
      expect(callback).toHaveBeenCalled();
    });

    it('should receive position in callback', () => {
      const callback = vi.fn();
      timeline.onPositionChange(callback);

      timeline.play();

      expect(callback).toHaveBeenCalledWith(
        expect.objectContaining({
          musical: expect.any(Object),
          absolute: expect.any(Object),
        })
      );
    });

    it('should return unsubscribe function', () => {
      const callback = vi.fn();
      const unsubscribe = timeline.onPositionChange(callback);

      expect(typeof unsubscribe).toBe('function');

      unsubscribe();
      timeline.play();

      // Callback should not be called after unsubscribe
      // (already called once during play, but not again)
    });
  });

  describe('Integration tests', () => {
    it('should support complete playback workflow', () => {
      timeline.play();
      vi.advanceTimersByTime(1000);
      timeline.pause();

      const pausedPos = timeline.getPosition();

      timeline.play();
      vi.advanceTimersByTime(1000);
      timeline.stop();

      const stoppedPos = timeline.getPosition();
      expect(stoppedPos.musical.measure).toBe(0);
    });

    it('should handle tempo changes during playback', () => {
      timeline.play();
      vi.advanceTimersByTime(1000);

      timeline.setGlobalTempo(60); // Half speed

      vi.advanceTimersByTime(1000);
      // Position should advance slower now
    });

    it('should support full song structure with markers', () => {
      timeline.addMarker({ measure: 0, beat: 0, tick: 0, totalBeats: 0 }, 'Intro', 'section');
      timeline.addMarker({ measure: 4, beat: 0, tick: 0, totalBeats: 16 }, 'Verse', 'section');
      timeline.addMarker({ measure: 8, beat: 0, tick: 0, totalBeats: 32 }, 'Chorus', 'section');

      expect(timeline.getMarkers()).toHaveLength(3);
    });

    it('should coordinate all subsystems', () => {
      // Set up timeline
      timeline.setGlobalTempo(140);
      timeline.setTimeSignature({ numerator: 3, denominator: 4 });

      // Add markers
      timeline.addMarker({ measure: 0, beat: 0, tick: 0, totalBeats: 0 }, 'A', 'rehearsal');

      // Set loop
      const start: MusicalPosition = { measure: 0, beat: 0, tick: 0, totalBeats: 0 };
      const end: MusicalPosition = { measure: 4, beat: 0, tick: 0, totalBeats: 12 };
      timeline.setLoopRange(start, end);
      timeline.setLoopEnabled(true);

      // Start playback
      timeline.play();
      expect(timeline.isPlaying()).toBe(true);

      // All systems working together
      const position = timeline.getPosition();
      expect(position.musical).toBeDefined();
      expect(position.absolute).toBeDefined();
    });
  });
});
