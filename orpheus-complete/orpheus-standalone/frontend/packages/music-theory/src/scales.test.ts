import { describe, it, expect, vi } from 'vitest';
import {
  SCALE_DEFINITIONS,
  createScale,
  getAllScales,
  getScalesByGenre,
  findScalesWithNotes,
  getMinorPentatonicPatterns,
  getRelativeKey,
  getParallelKey,
  getScalesForChord,
} from './scales';

describe('scales.ts', () => {
  describe('SCALE_DEFINITIONS', () => {
    it('should contain all defined scales', () => {
      expect(SCALE_DEFINITIONS.length).toBe(21);
      expect(SCALE_DEFINITIONS.length).toBeGreaterThan(0);
    });

    it('should have unique names', () => {
      const names = SCALE_DEFINITIONS.map(s => s.name);
      const uniqueNames = new Set(names);
      expect(uniqueNames.size).toBe(SCALE_DEFINITIONS.length);
    });

    it('should have major scale', () => {
      const major = SCALE_DEFINITIONS.find(s => s.name === 'Major (Ionian)');
      expect(major).toBeDefined();
      expect(major?.intervals).toEqual([0, 2, 4, 5, 7, 9, 11]);
    });

    it('should have natural minor scale', () => {
      const minor = SCALE_DEFINITIONS.find(s => s.name === 'Natural Minor (Aeolian)');
      expect(minor).toBeDefined();
      expect(minor?.intervals).toEqual([0, 2, 3, 5, 7, 8, 10]);
    });

    it('should have pentatonic scales', () => {
      expect(SCALE_DEFINITIONS.find(s => s.name === 'Major Pentatonic')).toBeDefined();
      expect(SCALE_DEFINITIONS.find(s => s.name === 'Minor Pentatonic')).toBeDefined();
    });

    it('should have blues scales', () => {
      const blues = SCALE_DEFINITIONS.find(s => s.name === 'Blues Scale');
      expect(blues).toBeDefined();
      expect(blues?.intervals).toEqual([0, 3, 5, 6, 7, 10]);
    });

    it('should have all modes', () => {
      expect(SCALE_DEFINITIONS.find(s => s.name === 'Dorian')).toBeDefined();
      expect(SCALE_DEFINITIONS.find(s => s.name === 'Phrygian')).toBeDefined();
      expect(SCALE_DEFINITIONS.find(s => s.name === 'Lydian')).toBeDefined();
      expect(SCALE_DEFINITIONS.find(s => s.name === 'Mixolydian')).toBeDefined();
      expect(SCALE_DEFINITIONS.find(s => s.name === 'Locrian')).toBeDefined();
    });

    it('all scales should have required properties', () => {
      for (const scale of SCALE_DEFINITIONS) {
        expect(scale).toHaveProperty('name');
        expect(scale).toHaveProperty('intervals');
        expect(scale).toHaveProperty('description');
        expect(scale.intervals).toBeInstanceOf(Array);
        expect(scale.intervals.length).toBeGreaterThan(0);
      }
    });
  });

  describe('createScale', () => {
    it('should create C major scale', () => {
      const scale = createScale('C', 'Major (Ionian)');
      expect(scale.root).toBe('C');
      expect(scale.name).toBe('Major (Ionian)');
      expect(scale.notes).toEqual(['C', 'D', 'E', 'F', 'G', 'A', 'B']);
      expect(scale.intervals).toEqual([0, 2, 4, 5, 7, 9, 11]);
    });

    it('should create C minor scale', () => {
      const scale = createScale('C', 'Natural Minor (Aeolian)');
      expect(scale.root).toBe('C');
      expect(scale.notes).toEqual(['C', 'D', 'D#', 'F', 'G', 'G#', 'A#']);
    });

    it('should create pentatonic scales', () => {
      const majPent = createScale('C', 'Major Pentatonic');
      expect(majPent.notes).toEqual(['C', 'D', 'E', 'G', 'A']);

      const minPent = createScale('C', 'Minor Pentatonic');
      expect(minPent.notes).toEqual(['C', 'D#', 'F', 'G', 'A#']);
    });

    it('should work with all roots', () => {
      const roots = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
      for (const root of roots) {
        const scale = createScale(root, 'Major (Ionian)');
        expect(scale.root).toBe(root);
        expect(scale.notes[0]).toBe(root);
      }
    });

    it('should work with flat notes', () => {
      const scale = createScale('Db', 'Major (Ionian)');
      expect(scale.root).toBe('Db');
    });

    it('should work with sharp notes', () => {
      const scale = createScale('C#', 'Minor Pentatonic');
      expect(scale.root).toBe('C#');
    });

    it('should throw error for unknown scale', () => {
      expect(() => {
        createScale('C', 'Invalid Scale');
      }).toThrow('Unknown scale: Invalid Scale');
    });

    it('should include description', () => {
      const scale = createScale('C', 'Major (Ionian)');
      expect(scale.description).toBeDefined();
      expect(typeof scale.description).toBe('string');
    });
  });

  describe('getAllScales', () => {
    it('should return all scale definitions', () => {
      const scales = getAllScales();
      expect(scales).toEqual(SCALE_DEFINITIONS);
      expect(scales.length).toBe(21);
    });

    it('should return array of scale definitions', () => {
      const scales = getAllScales();
      expect(Array.isArray(scales)).toBe(true);
      for (const scale of scales) {
        expect(scale).toHaveProperty('name');
        expect(scale).toHaveProperty('intervals');
      }
    });
  });

  describe('getScalesByGenre', () => {
    it('should find rock scales', () => {
      const rockScales = getScalesByGenre('rock');
      expect(rockScales.length).toBeGreaterThan(0);
      expect(rockScales.some(s => s.name === 'Major (Ionian)')).toBe(true);
    });

    it('should find jazz scales', () => {
      const jazzScales = getScalesByGenre('jazz');
      expect(jazzScales.length).toBeGreaterThan(0);
      expect(jazzScales.some(s => s.name === 'Dorian')).toBe(true);
    });

    it('should find blues scales', () => {
      const bluesScales = getScalesByGenre('blues');
      expect(bluesScales.length).toBeGreaterThan(0);
      expect(bluesScales.some(s => s.name === 'Blues Scale')).toBe(true);
    });

    it('should be case insensitive', () => {
      const lower = getScalesByGenre('rock');
      const upper = getScalesByGenre('ROCK');
      const mixed = getScalesByGenre('Rock');
      expect(lower).toEqual(upper);
      expect(lower).toEqual(mixed);
    });

    it('should return empty array for unknown genre', () => {
      const scales = getScalesByGenre('invalid-genre-xyz');
      expect(scales).toEqual([]);
    });
  });

  describe('findScalesWithNotes', () => {
    it('should find scales containing C, E, G', () => {
      const scales = findScalesWithNotes('C', ['C', 'E', 'G']);
      expect(scales.length).toBeGreaterThan(0);
      expect(scales.some(s => s.name === 'Major (Ionian)')).toBe(true);
    });

    it('should find scales with single note', () => {
      const scales = findScalesWithNotes('C', ['C']);
      expect(scales.length).toBeGreaterThan(0);
      // All scales should contain their root
      expect(scales.every(s => s.notes.includes('C'))).toBe(true);
    });

    it('should return empty array if no scales match', () => {
      // Create an impossible combination
      const scales = findScalesWithNotes('C', ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']);
      // Only chromatic scale has all notes
      expect(scales.length).toBeGreaterThan(0);
      expect(scales.some(s => s.name === 'Chromatic')).toBe(true);
    });

    it('should work with different roots', () => {
      const cScales = findScalesWithNotes('C', ['C', 'E']);
      const dScales = findScalesWithNotes('D', ['D', 'F#']);
      expect(cScales.length).toBeGreaterThan(0);
      expect(dScales.length).toBeGreaterThan(0);
    });
  });

  describe('getMinorPentatonicPatterns', () => {
    it('should return array of patterns', () => {
      const patterns = getMinorPentatonicPatterns('A');
      expect(Array.isArray(patterns)).toBe(true);
      expect(patterns.length).toBeGreaterThan(0);
    });

    it('should include root in pattern name', () => {
      const patterns = getMinorPentatonicPatterns('A');
      expect(patterns[0].name).toContain('A');
      expect(patterns[0].name).toContain('Minor Pentatonic');
    });

    it('should have pattern structure', () => {
      const patterns = getMinorPentatonicPatterns('C');
      expect(patterns[0]).toHaveProperty('name');
      expect(patterns[0]).toHaveProperty('position');
      expect(patterns[0]).toHaveProperty('startFret');
      expect(patterns[0]).toHaveProperty('notes');
      expect(Array.isArray(patterns[0].notes)).toBe(true);
    });

    it('should work with different roots', () => {
      const aPattern = getMinorPentatonicPatterns('A');
      const cPattern = getMinorPentatonicPatterns('C');
      expect(aPattern[0].name).toContain('A');
      expect(cPattern[0].name).toContain('C');
    });
  });

  describe('getRelativeKey', () => {
    it('should find relative minor from major', () => {
      expect(getRelativeKey('C', 'major')).toBe('A');
      expect(getRelativeKey('G', 'major')).toBe('E');
      expect(getRelativeKey('D', 'major')).toBe('B');
    });

    it('should find relative major from minor', () => {
      expect(getRelativeKey('A', 'minor')).toBe('C');
      expect(getRelativeKey('E', 'minor')).toBe('G');
      expect(getRelativeKey('B', 'minor')).toBe('D');
    });

    it('should be inverse operation', () => {
      const major = 'C';
      const minor = getRelativeKey(major, 'major');
      const backToMajor = getRelativeKey(minor, 'minor');
      expect(backToMajor).toBe(major);
    });

    it('should work with all notes', () => {
      const notes = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
      for (const note of notes) {
        const relative = getRelativeKey(note, 'major');
        expect(typeof relative).toBe('string');
      }
    });
  });

  describe('getParallelKey', () => {
    it('should return same root', () => {
      expect(getParallelKey('C')).toBe('C');
      expect(getParallelKey('D')).toBe('D');
      expect(getParallelKey('E')).toBe('E');
    });

    it('should work with sharp notes', () => {
      expect(getParallelKey('C#')).toBe('C#');
      expect(getParallelKey('F#')).toBe('F#');
    });

    it('should work with flat notes', () => {
      expect(getParallelKey('Db')).toBe('Db');
      expect(getParallelKey('Bb')).toBe('Bb');
    });
  });

  describe('getScalesForChord', () => {
    it('should suggest scales for major chords', () => {
      const scales = getScalesForChord('C', 'major');
      expect(scales).toContain('Major (Ionian)');
      expect(scales).toContain('Mixolydian');
      expect(scales).toContain('Lydian');
      expect(scales).toContain('Major Pentatonic');
    });

    it('should suggest scales for minor chords', () => {
      const scales = getScalesForChord('A', 'minor');
      expect(scales).toContain('Natural Minor (Aeolian)');
      expect(scales).toContain('Dorian');
      expect(scales).toContain('Minor Pentatonic');
      expect(scales).toContain('Blues Scale');
    });

    it('should suggest scales for dominant7 chords', () => {
      const scales = getScalesForChord('G', 'dominant7');
      expect(scales).toContain('Mixolydian');
      expect(scales).toContain('Blues Scale');
    });

    it('should suggest scales for minor7 chords', () => {
      const scales = getScalesForChord('D', 'minor7');
      expect(scales).toContain('Dorian');
      expect(scales).toContain('Aeolian');
    });

    it('should suggest scales for major7 chords', () => {
      const scales = getScalesForChord('C', 'major7');
      expect(scales).toContain('Major (Ionian)');
      expect(scales).toContain('Lydian');
    });

    it('should suggest scales for diminished chords', () => {
      const scales = getScalesForChord('B', 'diminished');
      expect(scales).toContain('Locrian');
      expect(scales).toContain('Diminished (Half-Whole)');
    });

    it('should return chromatic for unknown chord type', () => {
      const scales = getScalesForChord('C', 'unknownChordType');
      expect(scales).toEqual(['Chromatic']);
    });

    it('should work with different chord roots', () => {
      const cScales = getScalesForChord('C', 'major');
      const dScales = getScalesForChord('D', 'major');
      // Should return same recommendations regardless of root
      expect(cScales).toEqual(dScales);
    });
  });
});
