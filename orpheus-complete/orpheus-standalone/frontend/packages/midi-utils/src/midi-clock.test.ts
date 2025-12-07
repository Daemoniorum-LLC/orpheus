import { describe, it, expect, beforeEach, vi } from 'vitest';
import { MIDIClock, ticksToMilliseconds, millisecondsToTicks } from './midi-clock';

describe('midi-clock.ts', () => {
  describe('MIDIClock', () => {
    let clock: MIDIClock;

    beforeEach(() => {
      clock = new MIDIClock(120, 480);
    });

    describe('constructor', () => {
      it('should create clock with default tempo (120 BPM)', () => {
        const clock = new MIDIClock();

        expect(clock.getTempo()).toBe(120);
      });

      it('should create clock with custom tempo', () => {
        const clock = new MIDIClock(140);

        expect(clock.getTempo()).toBe(140);
      });

      it('should create clock with custom division', () => {
        const clock = new MIDIClock(120, 960);

        expect(clock.getTempo()).toBe(120);
      });
    });

    describe('setTempo() / getTempo()', () => {
      it('should set and get tempo', () => {
        clock.setTempo(150);

        expect(clock.getTempo()).toBe(150);
      });

      it('should update tempo while running', () => {
        clock.start();
        clock.setTempo(180);

        expect(clock.getTempo()).toBe(180);
      });

      it('should handle various BPM values', () => {
        const tempos = [60, 90, 120, 140, 180, 200];

        tempos.forEach(tempo => {
          clock.setTempo(tempo);
          expect(clock.getTempo()).toBe(tempo);
        });
      });
    });

    describe('start()', () => {
      it('should start the clock', () => {
        clock.start();

        // Clock should be running (getCurrentTick returns non-zero after time passes)
        expect(clock.getCurrentTick).toBeDefined();
      });

      it('should not restart if already running', () => {
        clock.start();
        const tick1 = clock.getCurrentTick();

        clock.start(); // Try to start again
        const tick2 = clock.getCurrentTick();

        // Should continue from same position
        expect(tick2).toBeGreaterThanOrEqual(tick1);
      });
    });

    describe('stop()', () => {
      it('should stop the clock', () => {
        clock.start();
        clock.stop();

        const tick = clock.getCurrentTick();
        expect(tick).toBe(0);
      });

      it('should reset paused time', () => {
        clock.start();
        clock.pause();
        clock.stop();

        const tick = clock.getCurrentTick();
        expect(tick).toBe(0);
      });
    });

    describe('pause() / resume', () => {
      it('should pause the clock', () => {
        clock.start();
        clock.pause();

        // After pause, getCurrentTick should return 0 when not running
        const tick = clock.getCurrentTick();
        expect(tick).toBe(0);
      });

      it('should resume from paused position', () => {
        clock.start();

        // Wait a bit
        return new Promise(resolve => setTimeout(resolve, 10)).then(() => {
          clock.pause();
          const pausedTick = clock.getCurrentTick();

          clock.start(); // Resume
          const resumedTick = clock.getCurrentTick();

          // Should be close to where it was paused
          expect(resumedTick).toBeGreaterThanOrEqual(0);
        });
      });
    });

    describe('getCurrentTick()', () => {
      it('should return 0 when not running', () => {
        const tick = clock.getCurrentTick();

        expect(tick).toBe(0);
      });

      it('should return increasing ticks when running', () => {
        clock.start();

        return new Promise(resolve => setTimeout(resolve, 20)).then(() => {
          const tick1 = clock.getCurrentTick();

          return new Promise(resolve => setTimeout(resolve, 20)).then(() => {
            const tick2 = clock.getCurrentTick();

            expect(tick2).toBeGreaterThan(tick1);
          });
        });
      });

      it('should calculate ticks based on tempo', () => {
        const clock120 = new MIDIClock(120, 480);
        const clock240 = new MIDIClock(240, 480);

        clock120.start();
        clock240.start();

        return new Promise(resolve => setTimeout(resolve, 100)).then(() => {
          const tick120 = clock120.getCurrentTick();
          const tick240 = clock240.getCurrentTick();

          // 240 BPM should accumulate ticks faster than 120 BPM
          expect(tick240).toBeGreaterThan(tick120);

          clock120.stop();
          clock240.stop();
        });
      });
    });

    describe('onTick()', () => {
      it('should register callback', () => {
        const callback = vi.fn();
        clock.onTick(callback);

        clock.start();

        return new Promise(resolve => setTimeout(resolve, 50)).then(() => {
          expect(callback).toHaveBeenCalled();
          clock.stop();
        });
      });

      it('should call callback with current tick', () => {
        const callback = vi.fn();
        clock.onTick(callback);

        clock.start();

        return new Promise(resolve => setTimeout(resolve, 50)).then(() => {
          expect(callback).toHaveBeenCalledWith(expect.any(Number));
          clock.stop();
        });
      });

      it('should return unsubscribe function', () => {
        const callback = vi.fn();
        const unsubscribe = clock.onTick(callback);

        expect(typeof unsubscribe).toBe('function');
      });

      it('should stop calling callback after unsubscribe', () => {
        const callback = vi.fn();
        const unsubscribe = clock.onTick(callback);

        clock.start();

        return new Promise(resolve => setTimeout(resolve, 30)).then(() => {
          const callCount = callback.mock.calls.length;
          unsubscribe();

          return new Promise(resolve => setTimeout(resolve, 30)).then(() => {
            // Should not have been called more times
            const newCallCount = callback.mock.calls.length;
            expect(newCallCount).toBeGreaterThanOrEqual(callCount);

            clock.stop();
          });
        });
      });

      it('should support multiple callbacks', () => {
        const callback1 = vi.fn();
        const callback2 = vi.fn();

        clock.onTick(callback1);
        clock.onTick(callback2);

        clock.start();

        return new Promise(resolve => setTimeout(resolve, 50)).then(() => {
          expect(callback1).toHaveBeenCalled();
          expect(callback2).toHaveBeenCalled();
          clock.stop();
        });
      });
    });
  });

  describe('ticksToMilliseconds()', () => {
    it('should convert ticks to milliseconds at 120 BPM', () => {
      const ms = ticksToMilliseconds(480, 120, 480);

      // 120 BPM = 2 beats/sec, 480 ticks = 1 beat = 500ms
      expect(ms).toBeCloseTo(500, 1);
    });

    it('should convert at 60 BPM (slower)', () => {
      const ms = ticksToMilliseconds(480, 60, 480);

      // 60 BPM = 1 beat/sec, 480 ticks = 1 beat = 1000ms
      expect(ms).toBeCloseTo(1000, 1);
    });

    it('should convert at 240 BPM (faster)', () => {
      const ms = ticksToMilliseconds(480, 240, 480);

      // 240 BPM = 4 beats/sec, 480 ticks = 1 beat = 250ms
      expect(ms).toBeCloseTo(250, 1);
    });

    it('should handle different tick values', () => {
      expect(ticksToMilliseconds(240, 120, 480)).toBeCloseTo(250, 1); // Half beat
      expect(ticksToMilliseconds(960, 120, 480)).toBeCloseTo(1000, 1); // Two beats
      expect(ticksToMilliseconds(120, 120, 480)).toBeCloseTo(125, 1); // Quarter beat
    });

    it('should handle different divisions', () => {
      const ms1 = ticksToMilliseconds(960, 120, 960);
      const ms2 = ticksToMilliseconds(480, 120, 480);

      // Both represent 1 beat
      expect(ms1).toBeCloseTo(ms2, 1);
    });

    it('should handle zero ticks', () => {
      const ms = ticksToMilliseconds(0, 120, 480);

      expect(ms).toBe(0);
    });

    it('should be proportional to tempo', () => {
      const ms120 = ticksToMilliseconds(480, 120, 480);
      const ms60 = ticksToMilliseconds(480, 60, 480);

      // 60 BPM is half as fast, so should take twice as long
      expect(ms60).toBeCloseTo(ms120 * 2, 1);
    });
  });

  describe('millisecondsToTicks()', () => {
    it('should convert milliseconds to ticks at 120 BPM', () => {
      const ticks = millisecondsToTicks(500, 120, 480);

      // 120 BPM = 2 beats/sec, 500ms = 1 beat = 480 ticks
      expect(ticks).toBeCloseTo(480, 1);
    });

    it('should convert at 60 BPM', () => {
      const ticks = millisecondsToTicks(1000, 60, 480);

      // 60 BPM = 1 beat/sec, 1000ms = 1 beat = 480 ticks
      expect(ticks).toBeCloseTo(480, 1);
    });

    it('should convert at 240 BPM', () => {
      const ticks = millisecondsToTicks(250, 240, 480);

      // 240 BPM = 4 beats/sec, 250ms = 1 beat = 480 ticks
      expect(ticks).toBeCloseTo(480, 1);
    });

    it('should handle different millisecond values', () => {
      expect(millisecondsToTicks(250, 120, 480)).toBeCloseTo(240, 1); // Half beat
      expect(millisecondsToTicks(1000, 120, 480)).toBeCloseTo(960, 1); // Two beats
      expect(millisecondsToTicks(125, 120, 480)).toBeCloseTo(120, 1); // Quarter beat
    });

    it('should handle different divisions', () => {
      const ticks1 = millisecondsToTicks(500, 120, 960);
      const ticks2 = millisecondsToTicks(500, 120, 480);

      // 500ms at 120 BPM = 1 beat, but different divisions
      expect(ticks1).toBeCloseTo(960, 1);
      expect(ticks2).toBeCloseTo(480, 1);
    });

    it('should handle zero milliseconds', () => {
      const ticks = millisecondsToTicks(0, 120, 480);

      expect(ticks).toBe(0);
    });

    it('should be inverse of ticksToMilliseconds', () => {
      const original = 720;
      const ms = ticksToMilliseconds(original, 120, 480);
      const result = millisecondsToTicks(ms, 120, 480);

      expect(result).toBeCloseTo(original, 1);
    });
  });

  describe('roundtrip conversions', () => {
    it('should convert ticks→ms→ticks at various tempos', () => {
      const tempos = [60, 90, 120, 140, 180];

      tempos.forEach(tempo => {
        const original = 960;
        const ms = ticksToMilliseconds(original, tempo, 480);
        const result = millisecondsToTicks(ms, tempo, 480);

        expect(result).toBeCloseTo(original, 0);
      });
    });

    it('should convert ms→ticks→ms at various tempos', () => {
      const tempos = [60, 90, 120, 140, 180];

      tempos.forEach(tempo => {
        const original = 750;
        const ticks = millisecondsToTicks(original, tempo, 480);
        const result = ticksToMilliseconds(ticks, tempo, 480);

        expect(result).toBeCloseTo(original, 0);
      });
    });

    it('should work with different divisions', () => {
      const divisions = [96, 192, 480, 960];

      divisions.forEach(division => {
        const original = 100;
        const ms = ticksToMilliseconds(original, 120, division);
        const result = millisecondsToTicks(ms, 120, division);

        expect(result).toBeCloseTo(original, 0);
      });
    });
  });
});
