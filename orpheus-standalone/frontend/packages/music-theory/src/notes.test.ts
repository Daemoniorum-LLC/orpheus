import { describe, it, expect } from 'vitest';
import {
  NOTES,
  FLAT_NOTES,
  toSharp,
  toFlat,
  getMidiNote,
  fromMidiNote,
  transpose,
  getFrequency,
  intervalBetween,
} from './notes';

describe('notes.ts', () => {
  describe('NOTES', () => {
    it('should contain all 12 chromatic notes with sharps', () => {
      expect(NOTES).toEqual(['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']);
      expect(NOTES).toHaveLength(12);
    });
  });

  describe('FLAT_NOTES', () => {
    it('should contain all 12 chromatic notes with flats', () => {
      expect(FLAT_NOTES).toEqual(['C', 'Db', 'D', 'Eb', 'E', 'F', 'Gb', 'G', 'Ab', 'A', 'Bb', 'B']);
      expect(FLAT_NOTES).toHaveLength(12);
    });
  });

  describe('toSharp', () => {
    it('should convert flat notes to sharp equivalents', () => {
      expect(toSharp('Db')).toBe('C#');
      expect(toSharp('Eb')).toBe('D#');
      expect(toSharp('Gb')).toBe('F#');
      expect(toSharp('Ab')).toBe('G#');
      expect(toSharp('Bb')).toBe('A#');
    });

    it('should return natural notes unchanged', () => {
      expect(toSharp('C')).toBe('C');
      expect(toSharp('D')).toBe('D');
      expect(toSharp('E')).toBe('E');
      expect(toSharp('F')).toBe('F');
      expect(toSharp('G')).toBe('G');
      expect(toSharp('A')).toBe('A');
      expect(toSharp('B')).toBe('B');
    });

    it('should return sharp notes unchanged', () => {
      expect(toSharp('C#')).toBe('C#');
      expect(toSharp('D#')).toBe('D#');
      expect(toSharp('F#')).toBe('F#');
      expect(toSharp('G#')).toBe('G#');
      expect(toSharp('A#')).toBe('A#');
    });
  });

  describe('toFlat', () => {
    it('should convert sharp notes to flat equivalents', () => {
      expect(toFlat('C#')).toBe('Db');
      expect(toFlat('D#')).toBe('Eb');
      expect(toFlat('F#')).toBe('Gb');
      expect(toFlat('G#')).toBe('Ab');
      expect(toFlat('A#')).toBe('Bb');
    });

    it('should return natural notes unchanged', () => {
      expect(toFlat('C')).toBe('C');
      expect(toFlat('D')).toBe('D');
      expect(toFlat('E')).toBe('E');
      expect(toFlat('F')).toBe('F');
      expect(toFlat('G')).toBe('G');
      expect(toFlat('A')).toBe('A');
      expect(toFlat('B')).toBe('B');
    });

    it('should return flat notes unchanged', () => {
      expect(toFlat('Db')).toBe('Db');
      expect(toFlat('Eb')).toBe('Eb');
      expect(toFlat('Gb')).toBe('Gb');
      expect(toFlat('Ab')).toBe('Ab');
      expect(toFlat('Bb')).toBe('Bb');
    });
  });

  describe('getMidiNote', () => {
    it('should calculate correct MIDI note numbers', () => {
      expect(getMidiNote('C', 4)).toBe(60);  // Middle C
      expect(getMidiNote('A', 4)).toBe(69);  // A440
      expect(getMidiNote('C', 0)).toBe(12);  // C0
      expect(getMidiNote('C', -1)).toBe(0);  // C-1
    });

    it('should work with flat notes', () => {
      expect(getMidiNote('Db', 4)).toBe(61);
      expect(getMidiNote('Eb', 4)).toBe(63);
      expect(getMidiNote('Gb', 4)).toBe(66);
    });

    it('should work with sharp notes', () => {
      expect(getMidiNote('C#', 4)).toBe(61);
      expect(getMidiNote('D#', 4)).toBe(63);
      expect(getMidiNote('F#', 4)).toBe(66);
    });

    it('should handle different octaves', () => {
      expect(getMidiNote('C', 0)).toBe(12);
      expect(getMidiNote('C', 1)).toBe(24);
      expect(getMidiNote('C', 2)).toBe(36);
      expect(getMidiNote('C', 5)).toBe(72);
      expect(getMidiNote('C', 8)).toBe(108);
    });
  });

  describe('fromMidiNote', () => {
    it('should convert MIDI numbers back to notes and octaves', () => {
      expect(fromMidiNote(60)).toEqual({ note: 'C', octave: 4 });
      expect(fromMidiNote(69)).toEqual({ note: 'A', octave: 4 });
      expect(fromMidiNote(12)).toEqual({ note: 'C', octave: 0 });
      expect(fromMidiNote(0)).toEqual({ note: 'C', octave: -1 });
    });

    it('should return sharp notes (not flats)', () => {
      expect(fromMidiNote(61)).toEqual({ note: 'C#', octave: 4 });
      expect(fromMidiNote(63)).toEqual({ note: 'D#', octave: 4 });
      expect(fromMidiNote(66)).toEqual({ note: 'F#', octave: 4 });
    });

    it('should handle all notes in an octave', () => {
      const octave4Notes = [];
      for (let i = 60; i < 72; i++) {
        octave4Notes.push(fromMidiNote(i).note);
      }
      expect(octave4Notes).toEqual(NOTES);
    });

    it('should be the inverse of getMidiNote', () => {
      for (let octave = -1; octave <= 8; octave++) {
        for (const note of NOTES) {
          const midiNote = getMidiNote(note, octave);
          const { note: resultNote, octave: resultOctave } = fromMidiNote(midiNote);
          expect(resultNote).toBe(note);
          expect(resultOctave).toBe(octave);
        }
      }
    });
  });

  describe('transpose', () => {
    it('should transpose notes up by semitones', () => {
      expect(transpose('C', 1)).toBe('C#');
      expect(transpose('C', 2)).toBe('D');
      expect(transpose('C', 7)).toBe('G');
      expect(transpose('C', 12)).toBe('C');
    });

    it('should transpose notes down by negative semitones', () => {
      expect(transpose('C', -1)).toBe('B');
      expect(transpose('C', -2)).toBe('A#');
      expect(transpose('G', -7)).toBe('C');
      expect(transpose('C', -12)).toBe('C');
    });

    it('should handle flat notes', () => {
      expect(transpose('Db', 1)).toBe('D');
      expect(transpose('Eb', 2)).toBe('F');
    });

    it('should wrap around octave correctly', () => {
      expect(transpose('B', 1)).toBe('C');
      expect(transpose('B', 2)).toBe('C#');
      expect(transpose('A#', 2)).toBe('C');
    });

    it('should handle large transpositions', () => {
      expect(transpose('C', 24)).toBe('C');  // 2 octaves up
      expect(transpose('C', 25)).toBe('C#');
      expect(transpose('C', -24)).toBe('C'); // 2 octaves down
    });

    it('should work for all notes', () => {
      for (const note of NOTES) {
        expect(transpose(note, 0)).toBe(note);
        expect(transpose(note, 12)).toBe(note);
        expect(transpose(note, -12)).toBe(note);
      }
    });
  });

  describe('getFrequency', () => {
    it('should calculate A4 as 440Hz', () => {
      expect(getFrequency('A', 4)).toBe(440);
    });

    it('should calculate correct frequencies', () => {
      expect(getFrequency('C', 4)).toBeCloseTo(261.63, 2);  // Middle C
      expect(getFrequency('C', 0)).toBeCloseTo(16.35, 2);   // C0
      expect(getFrequency('C', 8)).toBeCloseTo(4186.01, 2); // C8
    });

    it('should calculate octaves correctly', () => {
      const a3 = getFrequency('A', 3);
      const a4 = getFrequency('A', 4);
      const a5 = getFrequency('A', 5);

      expect(a4 / a3).toBeCloseTo(2, 5);  // One octave = 2x frequency
      expect(a5 / a4).toBeCloseTo(2, 5);
    });

    it('should calculate semitones correctly', () => {
      const c4 = getFrequency('C', 4);
      const cSharp4 = getFrequency('C#', 4);

      // Semitone ratio = 2^(1/12) ≈ 1.059463
      expect(cSharp4 / c4).toBeCloseTo(Math.pow(2, 1/12), 5);
    });

    it('should work with flat notes', () => {
      expect(getFrequency('Db', 4)).toBe(getFrequency('C#', 4));
      expect(getFrequency('Eb', 4)).toBe(getFrequency('D#', 4));
    });
  });

  describe('intervalBetween', () => {
    it('should calculate intervals correctly', () => {
      expect(intervalBetween('C', 'C')).toBe(0);   // Unison
      expect(intervalBetween('C', 'D')).toBe(2);   // Major 2nd
      expect(intervalBetween('C', 'E')).toBe(4);   // Major 3rd
      expect(intervalBetween('C', 'F')).toBe(5);   // Perfect 4th
      expect(intervalBetween('C', 'G')).toBe(7);   // Perfect 5th
      expect(intervalBetween('C', 'A')).toBe(9);   // Major 6th
      expect(intervalBetween('C', 'B')).toBe(11);  // Major 7th
    });

    it('should calculate chromatic intervals', () => {
      expect(intervalBetween('C', 'C#')).toBe(1);
      expect(intervalBetween('C', 'D#')).toBe(3);
      expect(intervalBetween('C', 'F#')).toBe(6);
      expect(intervalBetween('C', 'G#')).toBe(8);
      expect(intervalBetween('C', 'A#')).toBe(10);
    });

    it('should handle descending intervals', () => {
      expect(intervalBetween('G', 'C')).toBe(5);  // G up to C (wraps around)
      expect(intervalBetween('E', 'C')).toBe(8);
      expect(intervalBetween('B', 'C')).toBe(1);
    });

    it('should work with flat notes', () => {
      expect(intervalBetween('C', 'Db')).toBe(1);
      expect(intervalBetween('C', 'Eb')).toBe(3);
      expect(intervalBetween('C', 'Gb')).toBe(6);
    });

    it('should work with mixed sharp and flat notes', () => {
      expect(intervalBetween('C#', 'Eb')).toBe(2);
      expect(intervalBetween('Db', 'F#')).toBe(5);
    });

    it('should calculate all intervals within an octave', () => {
      for (let i = 0; i < NOTES.length; i++) {
        for (let j = 0; j < NOTES.length; j++) {
          const interval = intervalBetween(NOTES[i], NOTES[j]);
          expect(interval).toBeGreaterThanOrEqual(0);
          expect(interval).toBeLessThan(12);
        }
      }
    });
  });
});
