import { describe, it, expect } from 'vitest';
import * as MusicTheory from './index';

describe('index.ts - Package exports', () => {
  describe('notes exports', () => {
    it('should export NOTES constant', () => {
      expect(MusicTheory.NOTES).toBeDefined();
      expect(Array.isArray(MusicTheory.NOTES)).toBe(true);
    });

    it('should export FLAT_NOTES constant', () => {
      expect(MusicTheory.FLAT_NOTES).toBeDefined();
      expect(Array.isArray(MusicTheory.FLAT_NOTES)).toBe(true);
    });

    it('should export note utility functions', () => {
      expect(typeof MusicTheory.toSharp).toBe('function');
      expect(typeof MusicTheory.toFlat).toBe('function');
      expect(typeof MusicTheory.getMidiNote).toBe('function');
      expect(typeof MusicTheory.fromMidiNote).toBe('function');
      expect(typeof MusicTheory.transpose).toBe('function');
      expect(typeof MusicTheory.getFrequency).toBe('function');
      expect(typeof MusicTheory.intervalBetween).toBe('function');
    });
  });

  describe('intervals exports', () => {
    it('should export INTERVALS constant', () => {
      expect(MusicTheory.INTERVALS).toBeDefined();
      expect(Array.isArray(MusicTheory.INTERVALS)).toBe(true);
    });

    it('should export interval utility functions', () => {
      expect(typeof MusicTheory.getInterval).toBe('function');
      expect(typeof MusicTheory.getIntervalByName).toBe('function');
    });
  });

  describe('chords exports', () => {
    it('should export CHORD_DEFINITIONS constant', () => {
      expect(MusicTheory.CHORD_DEFINITIONS).toBeDefined();
      expect(Array.isArray(MusicTheory.CHORD_DEFINITIONS)).toBe(true);
    });

    it('should export COMMON_PROGRESSIONS constant', () => {
      expect(MusicTheory.COMMON_PROGRESSIONS).toBeDefined();
      expect(Array.isArray(MusicTheory.COMMON_PROGRESSIONS)).toBe(true);
    });

    it('should export chord utility functions', () => {
      expect(typeof MusicTheory.createChord).toBe('function');
      expect(typeof MusicTheory.getChordDefinition).toBe('function');
      expect(typeof MusicTheory.analyzeChord).toBe('function');
    });
  });

  describe('scales exports', () => {
    it('should export SCALE_DEFINITIONS constant', () => {
      expect(MusicTheory.SCALE_DEFINITIONS).toBeDefined();
      expect(Array.isArray(MusicTheory.SCALE_DEFINITIONS)).toBe(true);
    });

    it('should export scale utility functions', () => {
      expect(typeof MusicTheory.createScale).toBe('function');
      expect(typeof MusicTheory.getAllScales).toBe('function');
      expect(typeof MusicTheory.getScalesByGenre).toBe('function');
      expect(typeof MusicTheory.findScalesWithNotes).toBe('function');
      expect(typeof MusicTheory.getMinorPentatonicPatterns).toBe('function');
      expect(typeof MusicTheory.getRelativeKey).toBe('function');
      expect(typeof MusicTheory.getParallelKey).toBe('function');
      expect(typeof MusicTheory.getScalesForChord).toBe('function');
    });
  });

  describe('progressions exports', () => {
    it('should export progression utility functions', () => {
      expect(typeof MusicTheory.romanNumeralsToChords).toBe('function');
      expect(typeof MusicTheory.getProgressionInKey).toBe('function');
      expect(typeof MusicTheory.generateProgression).toBe('function');
      expect(typeof MusicTheory.suggestNextChord).toBe('function');
      expect(typeof MusicTheory.analyzeProgression).toBe('function');
    });
  });

  describe('guitar-chords exports', () => {
    it('should export GUITAR_CHORD_LIBRARY constant', () => {
      expect(MusicTheory.GUITAR_CHORD_LIBRARY).toBeDefined();
      expect(Array.isArray(MusicTheory.GUITAR_CHORD_LIBRARY)).toBe(true);
    });

    it('should export STANDARD_TUNING constant', () => {
      expect(MusicTheory.STANDARD_TUNING).toBeDefined();
      expect(Array.isArray(MusicTheory.STANDARD_TUNING)).toBe(true);
    });

    it('should export ALTERNATE_TUNINGS constant', () => {
      expect(MusicTheory.ALTERNATE_TUNINGS).toBeDefined();
      expect(typeof MusicTheory.ALTERNATE_TUNINGS).toBe('object');
    });

    it('should export guitar chord utility functions', () => {
      expect(typeof MusicTheory.getGuitarChordVoicings).toBe('function');
      expect(typeof MusicTheory.getBeginnerChords).toBe('function');
      expect(typeof MusicTheory.transposeGuitarChord).toBe('function');
    });
  });

  describe('keys exports', () => {
    it('should export CIRCLE_OF_FIFTHS_MAJOR constant', () => {
      expect(MusicTheory.CIRCLE_OF_FIFTHS_MAJOR).toBeDefined();
      expect(Array.isArray(MusicTheory.CIRCLE_OF_FIFTHS_MAJOR)).toBe(true);
    });

    it('should export key utility functions', () => {
      expect(typeof MusicTheory.getKeySignature).toBe('function');
      expect(typeof MusicTheory.getDiatonicChords).toBe('function');
      expect(typeof MusicTheory.analyzeKey).toBe('function');
    });
  });

  describe('integration - all modules work together', () => {
    it('should create a chord and find a scale for it', () => {
      const chord = MusicTheory.createChord('C', 'major');
      const scales = MusicTheory.getScalesForChord(chord.root, chord.quality);

      expect(chord.notes).toBeDefined();
      expect(scales.length).toBeGreaterThan(0);
    });

    it('should transpose notes and create chords', () => {
      const transposedNote = MusicTheory.transpose('C', 2);
      const chord = MusicTheory.createChord(transposedNote, 'major');

      expect(transposedNote).toBe('D');
      expect(chord.root).toBe('D');
    });

    it('should convert roman numerals to chords and get voicings', () => {
      const chords = MusicTheory.romanNumeralsToChords('C', ['I', 'IV', 'V'], 'major');
      const guitarVoicings = MusicTheory.getGuitarChordVoicings('C', 'major');

      expect(chords).toEqual(['C', 'F', 'G']);
      expect(guitarVoicings.length).toBeGreaterThan(0);
    });

    it('should work with MIDI notes and frequencies', () => {
      const midiNote = MusicTheory.getMidiNote('A', 4);
      const frequency = MusicTheory.getFrequency('A', 4);

      expect(midiNote).toBe(69);
      expect(frequency).toBe(440);
    });

    it('should analyze progression and get key signature', () => {
      const chords = ['C', 'F', 'G', 'C'];
      const analysis = MusicTheory.analyzeProgression(chords, 'C');
      const keySignature = MusicTheory.getKeySignature('C', 'major');

      expect(analysis.length).toBe(4);
      expect(keySignature).toBeDefined();
    });

    it('should create scale and find guitar patterns', () => {
      const scale = MusicTheory.createScale('A', 'Minor Pentatonic');
      const patterns = MusicTheory.getMinorPentatonicPatterns('A');

      expect(scale.notes.length).toBeGreaterThan(0);
      expect(patterns.length).toBeGreaterThan(0);
    });
  });

  describe('module completeness', () => {
    it('should have comprehensive chord library', () => {
      expect(MusicTheory.CHORD_DEFINITIONS.length).toBeGreaterThanOrEqual(20);
    });

    it('should have comprehensive scale library', () => {
      expect(MusicTheory.SCALE_DEFINITIONS.length).toBeGreaterThanOrEqual(20);
    });

    it('should have comprehensive interval library', () => {
      expect(MusicTheory.INTERVALS.length).toBe(13); // unison through octave
    });

    it('should have common chord progressions', () => {
      expect(MusicTheory.COMMON_PROGRESSIONS.length).toBeGreaterThanOrEqual(5);
    });

    it('should have guitar chord library', () => {
      expect(MusicTheory.GUITAR_CHORD_LIBRARY.length).toBeGreaterThanOrEqual(10);
    });

    it('should have key signatures', () => {
      expect(MusicTheory.CIRCLE_OF_FIFTHS_MAJOR.length).toBeGreaterThanOrEqual(7);
    });
  });

  describe('type exports', () => {
    it('should be able to use exported types', () => {
      // This test verifies that types are properly exported
      // by using them in variable declarations
      const note: MusicTheory.Note = 'C';
      const chordQuality: MusicTheory.ChordQuality = 'major';

      expect(note).toBe('C');
      expect(chordQuality).toBe('major');
    });
  });
});
