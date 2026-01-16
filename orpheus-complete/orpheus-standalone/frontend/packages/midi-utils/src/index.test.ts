import { describe, it, expect } from 'vitest';
import * as MidiUtils from './index';

describe('index.ts - Package exports', () => {
  describe('midi-converter exports', () => {
    it('should export tabToMIDI function', () => {
      expect(MidiUtils.tabToMIDI).toBeDefined();
      expect(typeof MidiUtils.tabToMIDI).toBe('function');
    });

    it('should export midiToTab function', () => {
      expect(MidiUtils.midiToTab).toBeDefined();
      expect(typeof MidiUtils.midiToTab).toBe('function');
    });

    it('should export beatsToTicks function', () => {
      expect(MidiUtils.beatsToTicks).toBeDefined();
      expect(typeof MidiUtils.beatsToTicks).toBe('function');
    });

    it('should export ticksToBeats function', () => {
      expect(MidiUtils.ticksToBeats).toBeDefined();
      expect(typeof MidiUtils.ticksToBeats).toBe('function');
    });

    it('should export bpmToMicroseconds function', () => {
      expect(MidiUtils.bpmToMicroseconds).toBeDefined();
      expect(typeof MidiUtils.bpmToMicroseconds).toBe('function');
    });

    it('should export microsecondsToBPM function', () => {
      expect(MidiUtils.microsecondsToBPM).toBeDefined();
      expect(typeof MidiUtils.microsecondsToBPM).toBe('function');
    });

    it('should export quantizeNotes function', () => {
      expect(MidiUtils.quantizeNotes).toBeDefined();
      expect(typeof MidiUtils.quantizeNotes).toBe('function');
    });

    it('should export humanizeNotes function', () => {
      expect(MidiUtils.humanizeNotes).toBeDefined();
      expect(typeof MidiUtils.humanizeNotes).toBe('function');
    });
  });

  describe('midi-clock exports', () => {
    it('should export MIDIClock class', () => {
      expect(MidiUtils.MIDIClock).toBeDefined();
      expect(typeof MidiUtils.MIDIClock).toBe('function');
    });

    it('should export ticksToMilliseconds function', () => {
      expect(MidiUtils.ticksToMilliseconds).toBeDefined();
      expect(typeof MidiUtils.ticksToMilliseconds).toBe('function');
    });

    it('should export millisecondsToTicks function', () => {
      expect(MidiUtils.millisecondsToTicks).toBeDefined();
      expect(typeof MidiUtils.millisecondsToTicks).toBe('function');
    });

    it('should be able to create MIDIClock instance', () => {
      const clock = new MidiUtils.MIDIClock();
      expect(clock).toBeInstanceOf(MidiUtils.MIDIClock);
    });
  });

  describe('midi-file exports', () => {
    it('should export parseMIDIFile function', () => {
      expect(MidiUtils.parseMIDIFile).toBeDefined();
      expect(typeof MidiUtils.parseMIDIFile).toBe('function');
    });

    it('should export generateMIDIFile function', () => {
      expect(MidiUtils.generateMIDIFile).toBeDefined();
      expect(typeof MidiUtils.generateMIDIFile).toBe('function');
    });
  });

  describe('midi-player exports', () => {
    it('should export MIDIPlayer class', () => {
      expect(MidiUtils.MIDIPlayer).toBeDefined();
      expect(typeof MidiUtils.MIDIPlayer).toBe('function');
    });

    it('should be able to create MIDIPlayer instance', () => {
      const player = new MidiUtils.MIDIPlayer();
      expect(player).toBeInstanceOf(MidiUtils.MIDIPlayer);
    });
  });

  describe('midi-recorder exports', () => {
    it('should export MIDIRecorder class', () => {
      expect(MidiUtils.MIDIRecorder).toBeDefined();
      expect(typeof MidiUtils.MIDIRecorder).toBe('function');
    });

    it('should be able to create MIDIRecorder instance', () => {
      const recorder = new MidiUtils.MIDIRecorder();
      expect(recorder).toBeInstanceOf(MidiUtils.MIDIRecorder);
    });
  });

  describe('integration tests', () => {
    it('should convert tab to MIDI and back', () => {
      const { note } = MidiUtils.tabToMIDI(1, 5);
      const positions = MidiUtils.midiToTab(note);

      expect(positions).toBeDefined();
      expect(positions.length).toBeGreaterThan(0);
    });

    it('should convert beats to ticks and back', () => {
      const beats = 2.5;
      const ticks = MidiUtils.beatsToTicks(beats, 480);
      const result = MidiUtils.ticksToBeats(ticks, 480);

      expect(result).toBeCloseTo(beats, 10);
    });

    it('should convert BPM to microseconds and back', () => {
      const bpm = 135;
      const microseconds = MidiUtils.bpmToMicroseconds(bpm);
      const result = MidiUtils.microsecondsToBPM(microseconds);

      expect(result).toBeCloseTo(bpm, 0);
    });

    it('should convert ticks to milliseconds and back', () => {
      const ticks = 720;
      const ms = MidiUtils.ticksToMilliseconds(ticks, 120, 480);
      const result = MidiUtils.millisecondsToTicks(ms, 120, 480);

      expect(result).toBeCloseTo(ticks, 0);
    });

    it('should quantize and humanize notes', () => {
      const notes = [
        { note: 60, startTime: 0.12, duration: 0.5, velocity: 100 },
        { note: 62, startTime: 0.63, duration: 0.5, velocity: 100 },
      ];

      const quantized = MidiUtils.quantizeNotes(notes, 0.25);
      const humanized = MidiUtils.humanizeNotes(quantized, { timingVariation: 0.01 });

      expect(quantized.length).toBe(2);
      expect(humanized.length).toBe(2);
    });
  });

  describe('package completeness', () => {
    it('should have all converter utilities', () => {
      expect(MidiUtils.tabToMIDI).toBeDefined();
      expect(MidiUtils.midiToTab).toBeDefined();
      expect(MidiUtils.beatsToTicks).toBeDefined();
      expect(MidiUtils.ticksToBeats).toBeDefined();
      expect(MidiUtils.bpmToMicroseconds).toBeDefined();
      expect(MidiUtils.microsecondsToBPM).toBeDefined();
      expect(MidiUtils.quantizeNotes).toBeDefined();
      expect(MidiUtils.humanizeNotes).toBeDefined();
    });

    it('should have all clock utilities', () => {
      expect(MidiUtils.MIDIClock).toBeDefined();
      expect(MidiUtils.ticksToMilliseconds).toBeDefined();
      expect(MidiUtils.millisecondsToTicks).toBeDefined();
    });

    it('should have all file utilities', () => {
      expect(MidiUtils.parseMIDIFile).toBeDefined();
      expect(MidiUtils.generateMIDIFile).toBeDefined();
    });

    it('should have all player utilities', () => {
      expect(MidiUtils.MIDIPlayer).toBeDefined();
    });

    it('should have all recorder utilities', () => {
      expect(MidiUtils.MIDIRecorder).toBeDefined();
    });
  });

  describe('usage examples', () => {
    it('should convert guitar tablature to MIDI', () => {
      const { note, noteName, octave } = MidiUtils.tabToMIDI(1, 5);

      expect(note).toBe(69); // A4
      expect(noteName).toBe('A');
      expect(octave).toBe(4);
    });

    it('should find guitar positions for MIDI note', () => {
      const positions = MidiUtils.midiToTab(69); // A4

      expect(positions.length).toBeGreaterThan(0);
      expect(positions[0]).toHaveProperty('string');
      expect(positions[0]).toHaveProperty('fret');
    });

    it('should create and control MIDI clock', () => {
      const clock = new MidiUtils.MIDIClock(120, 480);

      expect(clock.getTempo()).toBe(120);

      clock.setTempo(140);
      expect(clock.getTempo()).toBe(140);
    });

    it('should create MIDI player with options', () => {
      const player = new MidiUtils.MIDIPlayer({
        tempo: 140,
        loop: true,
      });

      expect(player).toBeInstanceOf(MidiUtils.MIDIPlayer);
    });

    it('should create MIDI recorder with options', () => {
      const recorder = new MidiUtils.MIDIRecorder({
        quantize: true,
        quantizeValue: 16,
      });

      expect(recorder).toBeInstanceOf(MidiUtils.MIDIRecorder);
    });
  });
});
