import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { Playhead } from './playhead';

describe('playhead.ts - Playback Position Management', () => {
  let playhead: Playhead;

  beforeEach(() => {
    vi.useFakeTimers();
    playhead = new Playhead();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  describe('Playhead construction', () => {
    it('should create playhead at position 0', () => {
      expect(playhead.getPosition()).toBe(0);
    });

    it('should start in stopped state', () => {
      expect(playhead.isPlaying()).toBe(false);
    });

    it('should initialize with no callbacks', () => {
      // No errors should occur
      expect(playhead).toBeInstanceOf(Playhead);
    });
  });

  describe('play', () => {
    it('should start playback', () => {
      playhead.play();
      expect(playhead.isPlaying()).toBe(true);
    });

    it('should not restart if already playing', () => {
      playhead.play();
      const firstPlay = playhead.isPlaying();
      playhead.play();
      expect(playhead.isPlaying()).toBe(firstPlay);
    });

    it('should record start time', () => {
      playhead.play();
      expect(playhead.getPosition()).toBeGreaterThanOrEqual(0);
    });

    it('should trigger position change callbacks', () => {
      const callback = vi.fn();
      playhead.onPositionChange(callback);

      playhead.play();
      expect(callback).toHaveBeenCalled();
    });

    it('should advance position over time', () => {
      playhead.play();
      const pos1 = playhead.getPosition();

      vi.advanceTimersByTime(1000); // 1 second
      const pos2 = playhead.getPosition();

      expect(pos2).toBeGreaterThan(pos1);
    });
  });

  describe('pause', () => {
    it('should pause playback', () => {
      playhead.play();
      playhead.pause();
      expect(playhead.isPlaying()).toBe(false);
    });

    it('should not pause if already stopped', () => {
      playhead.pause();
      expect(playhead.isPlaying()).toBe(false);
    });

    it('should preserve position when paused', () => {
      playhead.play();
      vi.advanceTimersByTime(1000);
      playhead.pause();

      const pausedPos = playhead.getPosition();
      vi.advanceTimersByTime(1000);

      expect(playhead.getPosition()).toBeCloseTo(pausedPos, 2);
    });

    it('should trigger position change callbacks', () => {
      const callback = vi.fn();
      playhead.onPositionChange(callback);

      playhead.play();
      callback.mockClear();

      playhead.pause();
      expect(callback).toHaveBeenCalled();
    });

    it('should record paused time', () => {
      playhead.play();
      vi.advanceTimersByTime(2000);
      playhead.pause();

      expect(playhead.getPosition()).toBeCloseTo(2, 1);
    });
  });

  describe('stop', () => {
    it('should stop playback', () => {
      playhead.play();
      playhead.stop();
      expect(playhead.isPlaying()).toBe(false);
    });

    it('should reset position to 0', () => {
      playhead.play();
      vi.advanceTimersByTime(5000);
      playhead.stop();

      expect(playhead.getPosition()).toBe(0);
    });

    it('should trigger position change callbacks', () => {
      const callback = vi.fn();
      playhead.onPositionChange(callback);

      playhead.play();
      callback.mockClear();

      playhead.stop();
      expect(callback).toHaveBeenCalled();
    });

    it('should reset internal state', () => {
      playhead.play();
      vi.advanceTimersByTime(1000);
      playhead.stop();

      expect(playhead.getPosition()).toBe(0);
      expect(playhead.isPlaying()).toBe(false);
    });
  });

  describe('setPosition', () => {
    it('should set position when stopped', () => {
      playhead.setPosition(5.5);
      expect(playhead.getPosition()).toBe(5.5);
    });

    it('should set position when playing', () => {
      playhead.play();
      playhead.setPosition(10);
      expect(playhead.getPosition()).toBeCloseTo(10, 1);
    });

    it('should update paused time', () => {
      playhead.play();
      playhead.pause();
      playhead.setPosition(3);

      expect(playhead.getPosition()).toBe(3);
    });

    it('should trigger position change callbacks', () => {
      const callback = vi.fn();
      playhead.onPositionChange(callback);

      playhead.setPosition(5);
      expect(callback).toHaveBeenCalled();
    });

    it('should allow seeking to 0', () => {
      playhead.setPosition(10);
      playhead.setPosition(0);
      expect(playhead.getPosition()).toBe(0);
    });

    it('should allow seeking forward', () => {
      playhead.setPosition(5);
      playhead.setPosition(10);
      expect(playhead.getPosition()).toBe(10);
    });

    it('should allow seeking backward', () => {
      playhead.setPosition(10);
      playhead.setPosition(5);
      expect(playhead.getPosition()).toBe(5);
    });

    it('should continue playing after seek during playback', () => {
      playhead.play();
      playhead.setPosition(5);
      expect(playhead.isPlaying()).toBe(true);
    });
  });

  describe('getPosition', () => {
    it('should return 0 initially', () => {
      expect(playhead.getPosition()).toBe(0);
    });

    it('should return current time when playing', () => {
      playhead.play();
      vi.advanceTimersByTime(2500);
      expect(playhead.getPosition()).toBeCloseTo(2.5, 1);
    });

    it('should return fixed position when paused', () => {
      playhead.play();
      vi.advanceTimersByTime(3000);
      playhead.pause();

      const pos = playhead.getPosition();
      vi.advanceTimersByTime(1000);

      expect(playhead.getPosition()).toBeCloseTo(pos, 2);
    });

    it('should return set position', () => {
      playhead.setPosition(7.5);
      expect(playhead.getPosition()).toBe(7.5);
    });

    it('should track time accurately over long durations', () => {
      playhead.play();
      vi.advanceTimersByTime(60000); // 1 minute
      expect(playhead.getPosition()).toBeCloseTo(60, 1);
    });
  });

  describe('isPlaying', () => {
    it('should return false initially', () => {
      expect(playhead.isPlaying()).toBe(false);
    });

    it('should return true when playing', () => {
      playhead.play();
      expect(playhead.isPlaying()).toBe(true);
    });

    it('should return false when paused', () => {
      playhead.play();
      playhead.pause();
      expect(playhead.isPlaying()).toBe(false);
    });

    it('should return false when stopped', () => {
      playhead.play();
      playhead.stop();
      expect(playhead.isPlaying()).toBe(false);
    });

    it('should reflect state changes immediately', () => {
      expect(playhead.isPlaying()).toBe(false);
      playhead.play();
      expect(playhead.isPlaying()).toBe(true);
      playhead.pause();
      expect(playhead.isPlaying()).toBe(false);
      playhead.play();
      expect(playhead.isPlaying()).toBe(true);
      playhead.stop();
      expect(playhead.isPlaying()).toBe(false);
    });
  });

  describe('advance', () => {
    it('should advance position when playing', () => {
      playhead.play();
      playhead.advance(1.5);
      expect(playhead.getPosition()).toBeCloseTo(1.5, 2);
    });

    it('should not advance when stopped', () => {
      playhead.advance(1.5);
      expect(playhead.getPosition()).toBe(0);
    });

    it('should trigger position change callbacks', () => {
      const callback = vi.fn();
      playhead.onPositionChange(callback);

      playhead.play();
      callback.mockClear();

      playhead.advance(1);
      expect(callback).toHaveBeenCalled();
    });

    it('should accumulate multiple advances', () => {
      playhead.play();
      playhead.advance(1);
      playhead.advance(1.5);
      playhead.advance(0.5);
      expect(playhead.getPosition()).toBeCloseTo(3, 2);
    });

    it('should work with fractional deltas', () => {
      playhead.play();
      playhead.advance(0.016667); // ~1 frame at 60fps
      expect(playhead.getPosition()).toBeCloseTo(0.016667, 5);
    });

    it('should support negative deltas (rewind)', () => {
      playhead.play();
      playhead.advance(5);
      playhead.advance(-2);
      expect(playhead.getPosition()).toBeCloseTo(3, 2);
    });
  });

  describe('onPositionChange', () => {
    it('should register callback', () => {
      const callback = vi.fn();
      playhead.onPositionChange(callback);

      playhead.play();
      expect(callback).toHaveBeenCalled();
    });

    it('should call callback on play', () => {
      const callback = vi.fn();
      playhead.onPositionChange(callback);

      playhead.play();
      expect(callback).toHaveBeenCalledTimes(1);
    });

    it('should call callback on pause', () => {
      const callback = vi.fn();
      playhead.onPositionChange(callback);

      playhead.play();
      callback.mockClear();

      playhead.pause();
      expect(callback).toHaveBeenCalledTimes(1);
    });

    it('should call callback on stop', () => {
      const callback = vi.fn();
      playhead.onPositionChange(callback);

      playhead.play();
      callback.mockClear();

      playhead.stop();
      expect(callback).toHaveBeenCalledTimes(1);
    });

    it('should call callback on setPosition', () => {
      const callback = vi.fn();
      playhead.onPositionChange(callback);

      playhead.setPosition(5);
      expect(callback).toHaveBeenCalled();
    });

    it('should call callback on advance', () => {
      const callback = vi.fn();
      playhead.onPositionChange(callback);

      playhead.play();
      callback.mockClear();

      playhead.advance(1);
      expect(callback).toHaveBeenCalled();
    });

    it('should support multiple callbacks', () => {
      const callback1 = vi.fn();
      const callback2 = vi.fn();
      const callback3 = vi.fn();

      playhead.onPositionChange(callback1);
      playhead.onPositionChange(callback2);
      playhead.onPositionChange(callback3);

      playhead.play();

      expect(callback1).toHaveBeenCalled();
      expect(callback2).toHaveBeenCalled();
      expect(callback3).toHaveBeenCalled();
    });

    it('should return unsubscribe function', () => {
      const callback = vi.fn();
      const unsubscribe = playhead.onPositionChange(callback);

      expect(typeof unsubscribe).toBe('function');
    });

    it('should allow unsubscribing', () => {
      const callback = vi.fn();
      const unsubscribe = playhead.onPositionChange(callback);

      unsubscribe();
      playhead.play();

      expect(callback).not.toHaveBeenCalled();
    });

    it('should handle unsubscribe of one callback without affecting others', () => {
      const callback1 = vi.fn();
      const callback2 = vi.fn();

      playhead.onPositionChange(callback1);
      const unsubscribe2 = playhead.onPositionChange(callback2);

      unsubscribe2();
      playhead.play();

      expect(callback1).toHaveBeenCalled();
      expect(callback2).not.toHaveBeenCalled();
    });

    it('should handle multiple unsubscribes', () => {
      const callback = vi.fn();
      const unsubscribe = playhead.onPositionChange(callback);

      unsubscribe();
      unsubscribe(); // Should not error

      playhead.play();
      expect(callback).not.toHaveBeenCalled();
    });
  });

  describe('Play/Pause/Resume workflow', () => {
    it('should support play -> pause -> play (resume) workflow', () => {
      playhead.play();
      vi.advanceTimersByTime(1000);
      playhead.pause();

      const pausedPos = playhead.getPosition();

      playhead.play();
      vi.advanceTimersByTime(1000);

      expect(playhead.getPosition()).toBeGreaterThan(pausedPos);
    });

    it('should maintain accumulated time across pause/resume cycles', () => {
      playhead.play();
      vi.advanceTimersByTime(1000);
      playhead.pause();

      playhead.play();
      vi.advanceTimersByTime(1000);
      playhead.pause();

      expect(playhead.getPosition()).toBeCloseTo(2, 1);
    });

    it('should support multiple pause/resume cycles', () => {
      for (let i = 0; i < 5; i++) {
        playhead.play();
        vi.advanceTimersByTime(200);
        playhead.pause();
      }

      expect(playhead.getPosition()).toBeCloseTo(1, 1);
    });
  });

  describe('Edge cases', () => {
    it('should handle very small position values', () => {
      playhead.setPosition(0.001);
      expect(playhead.getPosition()).toBe(0.001);
    });

    it('should handle very large position values', () => {
      playhead.setPosition(10000);
      expect(playhead.getPosition()).toBe(10000);
    });

    it('should handle rapid play/stop cycles', () => {
      for (let i = 0; i < 10; i++) {
        playhead.play();
        playhead.stop();
      }

      expect(playhead.getPosition()).toBe(0);
      expect(playhead.isPlaying()).toBe(false);
    });

    it('should handle rapid setPosition calls', () => {
      for (let i = 0; i < 100; i++) {
        playhead.setPosition(i);
      }

      expect(playhead.getPosition()).toBe(99);
    });

    it('should handle position set to exactly 0', () => {
      playhead.setPosition(5);
      playhead.setPosition(0);
      expect(playhead.getPosition()).toBe(0);
    });

    it('should handle fractional advance values', () => {
      playhead.play();
      playhead.advance(Math.PI);
      expect(playhead.getPosition()).toBeCloseTo(Math.PI, 5);
    });

    it('should maintain precision over many operations', () => {
      playhead.play();
      for (let i = 0; i < 100; i++) {
        playhead.advance(0.01);
      }

      expect(playhead.getPosition()).toBeCloseTo(1, 2);
    });
  });

  describe('Integration tests', () => {
    it('should support complete playback scenario', () => {
      const callback = vi.fn();
      playhead.onPositionChange(callback);

      // Start playback
      playhead.play();
      expect(callback).toHaveBeenCalled();
      callback.mockClear();

      // Play for 2 seconds
      vi.advanceTimersByTime(2000);
      expect(playhead.getPosition()).toBeCloseTo(2, 1);

      // Pause
      playhead.pause();
      expect(callback).toHaveBeenCalled();
      const pausedPos = playhead.getPosition();

      // Resume
      playhead.play();
      vi.advanceTimersByTime(1000);
      expect(playhead.getPosition()).toBeGreaterThan(pausedPos);

      // Stop
      playhead.stop();
      expect(playhead.getPosition()).toBe(0);
    });

    it('should handle seeking during playback', () => {
      playhead.play();
      vi.advanceTimersByTime(5000);

      playhead.setPosition(2);
      vi.advanceTimersByTime(1000);

      expect(playhead.getPosition()).toBeCloseTo(3, 1);
    });

    it('should handle manual advance in animation loop simulation', () => {
      playhead.play();

      // Simulate 60fps animation loop
      for (let i = 0; i < 60; i++) {
        playhead.advance(1 / 60);
      }

      expect(playhead.getPosition()).toBeCloseTo(1, 2);
    });

    it('should support audio worklet-style advance', () => {
      playhead.play();

      // Simulate 128-sample blocks at 48kHz
      const blockSize = 128;
      const sampleRate = 48000;
      const deltaSeconds = blockSize / sampleRate;

      for (let i = 0; i < 100; i++) {
        playhead.advance(deltaSeconds);
      }

      const expectedTime = (100 * blockSize) / sampleRate;
      expect(playhead.getPosition()).toBeCloseTo(expectedTime, 3);
    });
  });
});
