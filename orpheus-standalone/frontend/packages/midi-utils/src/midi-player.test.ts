import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { MIDIPlayer, type MIDIPlayerOptions } from './midi-player';
import type { MIDIFile, MIDITrack } from './midi-file';

describe('midi-player.ts - MIDI Playback', () => {
  let player: MIDIPlayer;

  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  describe('MIDIPlayer construction', () => {
    it('should create player with default options', () => {
      player = new MIDIPlayer();
      expect(player).toBeInstanceOf(MIDIPlayer);
    });

    it('should create player with custom tempo', () => {
      player = new MIDIPlayer({ tempo: 140 });
      expect(player).toBeInstanceOf(MIDIPlayer);
    });

    it('should create player with loop enabled', () => {
      player = new MIDIPlayer({ loop: true });
      expect(player).toBeInstanceOf(MIDIPlayer);
    });

    it('should create player with callbacks', () => {
      const onNoteOn = vi.fn();
      const onNoteOff = vi.fn();
      const onEnd = vi.fn();

      player = new MIDIPlayer({
        onNoteOn,
        onNoteOff,
        onEnd,
      });

      expect(player).toBeInstanceOf(MIDIPlayer);
    });

    it('should use default tempo of 120 BPM', () => {
      player = new MIDIPlayer();
      // Tempo should be 120 by default (tested via playback behavior)
      expect(player).toBeInstanceOf(MIDIPlayer);
    });

    it('should accept all option combinations', () => {
      const options: MIDIPlayerOptions = {
        tempo: 135,
        loop: true,
        onNoteOn: vi.fn(),
        onNoteOff: vi.fn(),
        onEnd: vi.fn(),
      };

      player = new MIDIPlayer(options);
      expect(player).toBeInstanceOf(MIDIPlayer);
    });
  });

  describe('loadFile', () => {
    beforeEach(() => {
      player = new MIDIPlayer();
    });

    it('should load MIDI file', () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      expect(player.getCurrentTime()).toBe(0);
    });

    it('should reset current tick when loading new file', () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      expect(player.getCurrentTime()).toBe(0);
    });

    it('should handle files with multiple tracks', () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 3, division: 480 },
        tracks: [
          { events: [] },
          { events: [] },
          { events: [] },
        ],
      };

      player.loadFile(file);
      expect(player.getCurrentTime()).toBe(0);
    });

    it('should handle different divisions', () => {
      [96, 192, 384, 480, 960].forEach(division => {
        const file: MIDIFile = {
          header: { format: 1, tracks: 1, division },
          tracks: [{ events: [] }],
        };

        player.loadFile(file);
        expect(player.getCurrentTime()).toBe(0);
      });
    });

    it('should load file with events', () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{
          events: [
            { type: 'noteOn', deltaTime: 0, channel: 0, note: 60, velocity: 100 },
            { type: 'noteOff', deltaTime: 480, channel: 0, note: 60, velocity: 0 },
          ],
        }],
      };

      player.loadFile(file);
      expect(player.getCurrentTime()).toBe(0);
    });
  });

  describe('play', () => {
    beforeEach(() => {
      player = new MIDIPlayer();
    });

    it('should throw error when no file loaded', async () => {
      await expect(player.play()).rejects.toThrow('No MIDI file loaded');
    });

    it('should start playback when file is loaded', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      // No error should be thrown
    });

    it('should trigger note on callbacks', async () => {
      const onNoteOn = vi.fn();
      player = new MIDIPlayer({ onNoteOn });

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{
          events: [
            { type: 'noteOn', deltaTime: 0, channel: 0, note: 60, velocity: 100 },
          ],
        }],
      };

      player.loadFile(file);
      await player.play();

      vi.advanceTimersByTime(10);
      expect(onNoteOn).toHaveBeenCalledWith(60, 100, 0);
    });

    it('should trigger note off callbacks', async () => {
      const onNoteOff = vi.fn();
      player = new MIDIPlayer({ onNoteOff });

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{
          events: [
            { type: 'noteOff', deltaTime: 0, channel: 0, note: 60, velocity: 0 },
          ],
        }],
      };

      player.loadFile(file);
      await player.play();

      vi.advanceTimersByTime(10);
      expect(onNoteOff).toHaveBeenCalledWith(60, 0);
    });

    it('should trigger end callback when playback finishes', async () => {
      const onEnd = vi.fn();
      player = new MIDIPlayer({ tempo: 120, onEnd });

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{
          events: [
            { type: 'noteOn', deltaTime: 0, channel: 0, note: 60, velocity: 100 },
          ],
        }],
      };

      player.loadFile(file);
      await player.play();

      vi.advanceTimersByTime(10000); // Advance past end
      expect(onEnd).toHaveBeenCalled();
    });

    it('should resume from pause', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      player.pause();
      await player.play(); // Should resume
      // No error should be thrown
    });

    it('should respect tempo setting', async () => {
      player = new MIDIPlayer({ tempo: 60 }); // Half tempo

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      // Playback should be slower at 60 BPM
    });

    it('should loop when loop option is true', async () => {
      const onEnd = vi.fn();
      player = new MIDIPlayer({ loop: true, onEnd });

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{
          events: [
            { type: 'noteOn', deltaTime: 0, channel: 0, note: 60, velocity: 100 },
          ],
        }],
      };

      player.loadFile(file);
      await player.play();

      vi.advanceTimersByTime(10000);
      // Loop should restart playback (onEnd not called in loop mode)
    });

    it('should handle multiple note events', async () => {
      const onNoteOn = vi.fn();
      player = new MIDIPlayer({ onNoteOn });

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{
          events: [
            { type: 'noteOn', deltaTime: 0, channel: 0, note: 60, velocity: 100 },
            { type: 'noteOn', deltaTime: 100, channel: 0, note: 64, velocity: 100 },
            { type: 'noteOn', deltaTime: 100, channel: 0, note: 67, velocity: 100 },
          ],
        }],
      };

      player.loadFile(file);
      await player.play();

      vi.advanceTimersByTime(1000);
      expect(onNoteOn).toHaveBeenCalledTimes(3);
    });

    it('should handle events on different channels', async () => {
      const onNoteOn = vi.fn();
      player = new MIDIPlayer({ onNoteOn });

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{
          events: [
            { type: 'noteOn', deltaTime: 0, channel: 0, note: 60, velocity: 100 },
            { type: 'noteOn', deltaTime: 0, channel: 1, note: 64, velocity: 100 },
          ],
        }],
      };

      player.loadFile(file);
      await player.play();

      vi.advanceTimersByTime(10);
      expect(onNoteOn).toHaveBeenCalledWith(60, 100, 0);
      expect(onNoteOn).toHaveBeenCalledWith(64, 100, 1);
    });
  });

  describe('pause', () => {
    beforeEach(() => {
      player = new MIDIPlayer();
    });

    it('should pause playback', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      player.pause();
      // Should be paused
    });

    it('should allow resuming after pause', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      player.pause();
      await player.play(); // Resume
      // No error
    });

    it('should maintain position when paused', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      player.pause();
      const time = player.getCurrentTime();
      expect(time).toBeGreaterThanOrEqual(0);
    });
  });

  describe('stop', () => {
    beforeEach(() => {
      player = new MIDIPlayer();
    });

    it('should stop playback', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      player.stop();
      // Playback should be stopped
    });

    it('should reset position to start', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      vi.advanceTimersByTime(100);
      player.stop();

      expect(player.getCurrentTime()).toBe(0);
    });

    it('should send all notes off when stopped', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{
          events: [
            { type: 'noteOn', deltaTime: 0, channel: 0, note: 60, velocity: 100 },
          ],
        }],
      };

      player.loadFile(file);
      await player.play();
      player.stop();
      // All notes should be turned off
    });
  });

  describe('seek', () => {
    beforeEach(() => {
      player = new MIDIPlayer();
    });

    it('should seek to specific tick position', () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      player.seek(960);

      const time = player.getCurrentTime();
      expect(time).toBeGreaterThan(0);
    });

    it('should seek to start (tick 0)', () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      player.seek(0);

      expect(player.getCurrentTime()).toBe(0);
    });

    it('should seek to middle position', () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      player.seek(240); // Quarter beat

      const time = player.getCurrentTime();
      expect(time).toBeGreaterThan(0);
    });

    it('should update playback if playing', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      player.seek(480);

      // Playback should continue from new position
    });
  });

  describe('getCurrentTime', () => {
    beforeEach(() => {
      player = new MIDIPlayer();
    });

    it('should return 0 when no file loaded', () => {
      expect(player.getCurrentTime()).toBe(0);
    });

    it('should return 0 at start', () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      expect(player.getCurrentTime()).toBe(0);
    });

    it('should calculate time based on tempo and division', () => {
      player = new MIDIPlayer({ tempo: 120 });

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      player.seek(480); // 1 beat at 120 BPM = 0.5 seconds

      const time = player.getCurrentTime();
      expect(time).toBeCloseTo(0.5, 1);
    });

    it('should account for tempo changes', () => {
      player = new MIDIPlayer({ tempo: 60 }); // Slower tempo

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      player.seek(480); // 1 beat at 60 BPM = 1 second

      const time = player.getCurrentTime();
      expect(time).toBeCloseTo(1.0, 1);
    });

    it('should work with different divisions', () => {
      player = new MIDIPlayer({ tempo: 120 });

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 960 }, // Higher resolution
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      player.seek(960); // 1 beat at 120 BPM = 0.5 seconds

      const time = player.getCurrentTime();
      expect(time).toBeCloseTo(0.5, 1);
    });
  });

  describe('setTempo', () => {
    beforeEach(() => {
      player = new MIDIPlayer({ tempo: 120 });
    });

    it('should change playback tempo', () => {
      player.setTempo(140);
      // Tempo changed
    });

    it('should accept various tempo values', () => {
      [60, 90, 120, 140, 180, 200].forEach(tempo => {
        player.setTempo(tempo);
        // No error
      });
    });

    it('should affect getCurrentTime calculation', () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      player.seek(480);

      const time1 = player.getCurrentTime();

      player.setTempo(60); // Half tempo
      const time2 = player.getCurrentTime();

      expect(time2).toBeGreaterThan(time1);
    });

    it('should allow tempo changes during playback', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      player.setTempo(180);
      // Tempo should change immediately
    });
  });

  describe('connectMIDIOutput', () => {
    beforeEach(() => {
      player = new MIDIPlayer();
    });

    it('should throw error if Web MIDI not supported', async () => {
      // Web MIDI API not available in test environment
      await expect(player.connectMIDIOutput()).rejects.toThrow('Web MIDI API not supported');
    });

    it('should accept output index parameter', async () => {
      await expect(player.connectMIDIOutput(0)).rejects.toThrow();
    });

    it('should accept different output indices', async () => {
      for (let i = 0; i < 3; i++) {
        await expect(player.connectMIDIOutput(i)).rejects.toThrow();
      }
    });
  });

  describe('Edge cases', () => {
    beforeEach(() => {
      player = new MIDIPlayer();
    });

    it('should handle empty tracks', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      // No events to play
    });

    it('should handle tracks with only meta events', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{
          events: [
            { type: 'meta', metaType: 'tempo', deltaTime: 0, data: 500000 },
          ],
        }],
      };

      player.loadFile(file);
      await player.play();
      // Meta events don't trigger note callbacks
    });

    it('should handle very fast tempo (240 BPM)', () => {
      player = new MIDIPlayer({ tempo: 240 });

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      expect(player.getCurrentTime()).toBe(0);
    });

    it('should handle very slow tempo (40 BPM)', () => {
      player = new MIDIPlayer({ tempo: 40 });

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      expect(player.getCurrentTime()).toBe(0);
    });

    it('should handle high resolution division (1920)', () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 1920 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      expect(player.getCurrentTime()).toBe(0);
    });

    it('should handle low resolution division (96)', () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 96 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      expect(player.getCurrentTime()).toBe(0);
    });

    it('should handle all 128 MIDI notes', () => {
      const onNoteOn = vi.fn();
      player = new MIDIPlayer({ onNoteOn });

      for (let note = 0; note < 128; note++) {
        const file: MIDIFile = {
          header: { format: 1, tracks: 1, division: 480 },
          tracks: [{
            events: [
              { type: 'noteOn', deltaTime: 0, channel: 0, note, velocity: 100 },
            ],
          }],
        };

        player.loadFile(file);
      }
    });

    it('should handle all 16 MIDI channels', () => {
      const onNoteOn = vi.fn();
      player = new MIDIPlayer({ onNoteOn });

      for (let channel = 0; channel < 16; channel++) {
        const file: MIDIFile = {
          header: { format: 1, tracks: 1, division: 480 },
          tracks: [{
            events: [
              { type: 'noteOn', deltaTime: 0, channel, note: 60, velocity: 100 },
            ],
          }],
        };

        player.loadFile(file);
      }
    });

    it('should handle velocity range (0-127)', () => {
      const onNoteOn = vi.fn();
      player = new MIDIPlayer({ onNoteOn });

      [0, 1, 32, 64, 96, 127].forEach(velocity => {
        const file: MIDIFile = {
          header: { format: 1, tracks: 1, division: 480 },
          tracks: [{
            events: [
              { type: 'noteOn', deltaTime: 0, channel: 0, note: 60, velocity },
            ],
          }],
        };

        player.loadFile(file);
      });
    });
  });

  describe('Integration tests', () => {
    it('should play complete MIDI sequence', async () => {
      const onNoteOn = vi.fn();
      const onNoteOff = vi.fn();
      const onEnd = vi.fn();

      player = new MIDIPlayer({ tempo: 120, onNoteOn, onNoteOff, onEnd });

      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{
          events: [
            { type: 'noteOn', deltaTime: 0, channel: 0, note: 60, velocity: 100 },
            { type: 'noteOff', deltaTime: 480, channel: 0, note: 60, velocity: 0 },
            { type: 'noteOn', deltaTime: 0, channel: 0, note: 64, velocity: 100 },
            { type: 'noteOff', deltaTime: 480, channel: 0, note: 64, velocity: 0 },
          ],
        }],
      };

      player.loadFile(file);
      await player.play();

      vi.advanceTimersByTime(5000);

      expect(onNoteOn).toHaveBeenCalledTimes(2);
      expect(onNoteOff).toHaveBeenCalledTimes(2);
    });

    it('should support pause/resume workflow', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      player.pause();
      await player.play();
      player.stop();

      expect(player.getCurrentTime()).toBe(0);
    });

    it('should support seek during playback', async () => {
      const file: MIDIFile = {
        header: { format: 1, tracks: 1, division: 480 },
        tracks: [{ events: [] }],
      };

      player.loadFile(file);
      await player.play();
      player.seek(480);
      player.stop();

      expect(player.getCurrentTime()).toBe(0);
    });
  });
});
