import { describe, it, expect } from 'vitest';
import {
  GUITAR_CHORD_LIBRARY,
  STANDARD_TUNING,
  ALTERNATE_TUNINGS,
  getGuitarChordVoicings,
  getBeginnerChords,
  transposeGuitarChord,
} from './guitar-chords';

describe('guitar-chords.ts', () => {
  describe('GUITAR_CHORD_LIBRARY', () => {
    it('should contain guitar chord voicings', () => {
      expect(GUITAR_CHORD_LIBRARY.length).toBeGreaterThan(0);
      expect(GUITAR_CHORD_LIBRARY.length).toBe(18); // As defined in source
    });

    it('all voicings should have required properties', () => {
      for (const chord of GUITAR_CHORD_LIBRARY) {
        expect(chord).toHaveProperty('root');
        expect(chord).toHaveProperty('quality');
        expect(chord).toHaveProperty('name');
        expect(chord).toHaveProperty('symbol');
        expect(chord).toHaveProperty('strings');
        expect(chord).toHaveProperty('baseFret');
        expect(chord).toHaveProperty('difficulty');
        expect(chord).toHaveProperty('category');
        expect(chord.strings).toHaveLength(6);
      }
    });

    it('should have C major voicing', () => {
      const cMajor = GUITAR_CHORD_LIBRARY.find(c => c.root === 'C' && c.quality === 'major');
      expect(cMajor).toBeDefined();
      expect(cMajor?.symbol).toBe('C');
      expect(cMajor?.name).toBe('C major');
      expect(cMajor?.category).toBe('open');
      expect(cMajor?.difficulty).toBe('beginner');
    });

    it('should have beginner chords', () => {
      const beginnerChords = GUITAR_CHORD_LIBRARY.filter(c => c.difficulty === 'beginner');
      expect(beginnerChords.length).toBeGreaterThan(0);
    });

    it('should have barre chords', () => {
      const barreChords = GUITAR_CHORD_LIBRARY.filter(c => c.category === 'barre');
      expect(barreChords.length).toBeGreaterThan(0);
      // Barre chords should have barres defined
      for (const chord of barreChords) {
        expect(chord.barres).toBeDefined();
        expect(chord.barres!.length).toBeGreaterThan(0);
      }
    });

    it('should have open chords', () => {
      const openChords = GUITAR_CHORD_LIBRARY.filter(c => c.category === 'open');
      expect(openChords.length).toBeGreaterThan(0);
    });

    it('should have power chords', () => {
      const powerChords = GUITAR_CHORD_LIBRARY.filter(c => c.quality === 'power');
      expect(powerChords.length).toBeGreaterThan(0);
      expect(powerChords.some(c => c.root === 'E')).toBe(true);
      expect(powerChords.some(c => c.root === 'A')).toBe(true);
    });

    it('should have jazz voicings', () => {
      const jazzChords = GUITAR_CHORD_LIBRARY.filter(c => c.category === 'jazz');
      expect(jazzChords.length).toBeGreaterThan(0);
    });

    it('should have F barre chord', () => {
      const fMajor = GUITAR_CHORD_LIBRARY.find(c => c.root === 'F' && c.quality === 'major');
      expect(fMajor).toBeDefined();
      expect(fMajor?.category).toBe('barre');
      expect(fMajor?.difficulty).toBe('intermediate');
      expect(fMajor?.barres).toEqual([1]);
    });

    it('strings should have correct structure', () => {
      for (const chord of GUITAR_CHORD_LIBRARY) {
        for (const string of chord.strings) {
          expect(string).toHaveProperty('string');
          expect(string).toHaveProperty('fret');
          expect(string.string).toBeGreaterThanOrEqual(1);
          expect(string.string).toBeLessThanOrEqual(6);
          expect(string.fret).toBeGreaterThanOrEqual(-1);
        }
      }
    });

    it('should have 6 strings for each voicing', () => {
      for (const chord of GUITAR_CHORD_LIBRARY) {
        expect(chord.strings).toHaveLength(6);
      }
    });

    it('should have valid difficulty levels', () => {
      const validDifficulties = ['beginner', 'intermediate', 'advanced'];
      for (const chord of GUITAR_CHORD_LIBRARY) {
        expect(validDifficulties).toContain(chord.difficulty);
      }
    });

    it('should have valid categories', () => {
      const validCategories = ['open', 'barre', 'moveable', 'jazz'];
      for (const chord of GUITAR_CHORD_LIBRARY) {
        expect(validCategories).toContain(chord.category);
      }
    });
  });

  describe('getGuitarChordVoicings', () => {
    it('should return voicings for C major', () => {
      const voicings = getGuitarChordVoicings('C', 'major');
      expect(voicings.length).toBeGreaterThan(0);
      expect(voicings.every(v => v.root === 'C' && v.quality === 'major')).toBe(true);
    });

    it('should return voicings for E major', () => {
      const voicings = getGuitarChordVoicings('E', 'major');
      expect(voicings.length).toBeGreaterThan(0);
      expect(voicings[0].root).toBe('E');
      expect(voicings[0].quality).toBe('major');
    });

    it('should return voicings for minor chords', () => {
      const aMinor = getGuitarChordVoicings('A', 'minor');
      expect(aMinor.length).toBeGreaterThan(0);
      expect(aMinor[0].symbol).toBe('Am');
    });

    it('should return voicings for dominant 7th chords', () => {
      const c7 = getGuitarChordVoicings('C', 'dominant7');
      expect(c7.length).toBeGreaterThan(0);
      expect(c7[0].symbol).toBe('C7');
    });

    it('should return voicings for power chords', () => {
      const e5 = getGuitarChordVoicings('E', 'power');
      expect(e5.length).toBeGreaterThan(0);
      expect(e5[0].symbol).toBe('E5');
    });

    it('should return empty array if no voicings exist', () => {
      const voicings = getGuitarChordVoicings('Z' as any, 'major');
      expect(voicings).toEqual([]);
    });

    it('should return multiple voicings if available', () => {
      // C has both open and barre voicings for some qualities
      const allC = GUITAR_CHORD_LIBRARY.filter(c => c.root === 'C');
      expect(allC.length).toBeGreaterThan(1);
    });

    it('should filter by both root and quality', () => {
      const cMajor = getGuitarChordVoicings('C', 'major');
      const cMinor = getGuitarChordVoicings('C', 'minor');
      expect(cMajor).not.toEqual(cMinor);
      expect(cMajor.every(v => v.quality === 'major')).toBe(true);
      expect(cMinor.every(v => v.quality === 'minor')).toBe(true);
    });
  });

  describe('getBeginnerChords', () => {
    it('should return only beginner difficulty chords', () => {
      const beginnerChords = getBeginnerChords();
      expect(beginnerChords.length).toBeGreaterThan(0);
      expect(beginnerChords.every(c => c.difficulty === 'beginner')).toBe(true);
    });

    it('should include common open chords', () => {
      const beginnerChords = getBeginnerChords();
      const roots = beginnerChords.map(c => c.root);
      expect(roots).toContain('C');
      expect(roots).toContain('D');
      expect(roots).toContain('E');
      expect(roots).toContain('G');
      expect(roots).toContain('A');
    });

    it('should not include barre chords', () => {
      const beginnerChords = getBeginnerChords();
      const barreChords = beginnerChords.filter(c => c.category === 'barre');
      expect(barreChords.length).toBe(0);
    });

    it('should include E major', () => {
      const beginnerChords = getBeginnerChords();
      const eMajor = beginnerChords.find(c => c.root === 'E' && c.quality === 'major');
      expect(eMajor).toBeDefined();
    });

    it('should include A minor', () => {
      const beginnerChords = getBeginnerChords();
      const aMinor = beginnerChords.find(c => c.root === 'A' && c.quality === 'minor');
      expect(aMinor).toBeDefined();
    });

    it('should include power chords', () => {
      const beginnerChords = getBeginnerChords();
      const powerChords = beginnerChords.filter(c => c.quality === 'power');
      expect(powerChords.length).toBeGreaterThan(0);
    });

    it('should return at least 10 chords', () => {
      const beginnerChords = getBeginnerChords();
      expect(beginnerChords.length).toBeGreaterThanOrEqual(10);
    });
  });

  describe('transposeGuitarChord', () => {
    it('should transpose chord up by semitones', () => {
      const cMajor = GUITAR_CHORD_LIBRARY.find(c => c.root === 'C' && c.quality === 'major')!;
      const transposed = transposeGuitarChord(cMajor, 2);

      // All non-muted frets should be shifted up by 2
      for (let i = 0; i < 6; i++) {
        if (cMajor.strings[i].fret >= 0) {
          expect(transposed.strings[i].fret).toBe(cMajor.strings[i].fret + 2);
        } else {
          expect(transposed.strings[i].fret).toBe(-1);
        }
      }
    });

    it('should transpose chord down by negative semitones', () => {
      const gMajor = GUITAR_CHORD_LIBRARY.find(c => c.root === 'G' && c.quality === 'major')!;
      const transposed = transposeGuitarChord(gMajor, -2);

      for (let i = 0; i < 6; i++) {
        if (gMajor.strings[i].fret >= 0) {
          expect(transposed.strings[i].fret).toBe(gMajor.strings[i].fret - 2);
        }
      }
    });

    it('should not transpose muted strings', () => {
      const cMajor = GUITAR_CHORD_LIBRARY.find(c => c.root === 'C' && c.quality === 'major')!;
      const transposed = transposeGuitarChord(cMajor, 5);

      // Muted strings should remain -1
      for (let i = 0; i < 6; i++) {
        if (cMajor.strings[i].fret === -1) {
          expect(transposed.strings[i].fret).toBe(-1);
        }
      }
    });

    it('should update baseFret', () => {
      const chord = GUITAR_CHORD_LIBRARY[0];
      const transposed = transposeGuitarChord(chord, 3);
      expect(transposed.baseFret).toBe(chord.baseFret + 3);
    });

    it('should preserve other properties', () => {
      const chord = GUITAR_CHORD_LIBRARY[0];
      const transposed = transposeGuitarChord(chord, 2);

      expect(transposed.root).toBe(chord.root);
      expect(transposed.quality).toBe(chord.quality);
      expect(transposed.name).toBe(chord.name);
      expect(transposed.symbol).toBe(chord.symbol);
      expect(transposed.difficulty).toBe(chord.difficulty);
      expect(transposed.category).toBe(chord.category);
    });

    it('should handle transpose by 0', () => {
      const chord = GUITAR_CHORD_LIBRARY[0];
      const transposed = transposeGuitarChord(chord, 0);

      expect(transposed.strings).toEqual(chord.strings);
      expect(transposed.baseFret).toBe(chord.baseFret);
    });

    it('should transpose barre chords', () => {
      const fMajor = GUITAR_CHORD_LIBRARY.find(c => c.root === 'F' && c.quality === 'major')!;
      const transposed = transposeGuitarChord(fMajor, 2);

      expect(transposed.baseFret).toBe(fMajor.baseFret + 2);
      expect(transposed.barres).toEqual(fMajor.barres);
    });

    it('should work with large transpositions', () => {
      const chord = GUITAR_CHORD_LIBRARY[0];
      const transposed = transposeGuitarChord(chord, 12);

      expect(transposed.baseFret).toBe(chord.baseFret + 12);
      for (let i = 0; i < 6; i++) {
        if (chord.strings[i].fret >= 0) {
          expect(transposed.strings[i].fret).toBe(chord.strings[i].fret + 12);
        }
      }
    });
  });

  describe('STANDARD_TUNING', () => {
    it('should have 6 strings', () => {
      expect(STANDARD_TUNING).toHaveLength(6);
    });

    it('should be EADGBE from high to low', () => {
      expect(STANDARD_TUNING).toEqual(['E', 'B', 'G', 'D', 'A', 'E']);
    });

    it('should have E as first string (high E)', () => {
      expect(STANDARD_TUNING[0]).toBe('E');
    });

    it('should have E as sixth string (low E)', () => {
      expect(STANDARD_TUNING[5]).toBe('E');
    });

    it('should be in correct order from high to low', () => {
      expect(STANDARD_TUNING[0]).toBe('E');  // String 1 (high E)
      expect(STANDARD_TUNING[1]).toBe('B');  // String 2
      expect(STANDARD_TUNING[2]).toBe('G');  // String 3
      expect(STANDARD_TUNING[3]).toBe('D');  // String 4
      expect(STANDARD_TUNING[4]).toBe('A');  // String 5
      expect(STANDARD_TUNING[5]).toBe('E');  // String 6 (low E)
    });
  });

  describe('ALTERNATE_TUNINGS', () => {
    it('should have multiple tunings defined', () => {
      const tunings = Object.keys(ALTERNATE_TUNINGS);
      expect(tunings.length).toBeGreaterThan(0);
      expect(tunings.length).toBe(6); // Drop D, Open G, Open D, DADGAD, Half Step Down, Whole Step Down
    });

    it('should have Drop D tuning', () => {
      expect(ALTERNATE_TUNINGS['Drop D']).toBeDefined();
      expect(ALTERNATE_TUNINGS['Drop D']).toEqual(['E', 'B', 'G', 'D', 'A', 'D']);
    });

    it('should have Open G tuning', () => {
      expect(ALTERNATE_TUNINGS['Open G']).toBeDefined();
      expect(ALTERNATE_TUNINGS['Open G']).toEqual(['D', 'B', 'G', 'D', 'G', 'D']);
    });

    it('should have Open D tuning', () => {
      expect(ALTERNATE_TUNINGS['Open D']).toBeDefined();
      expect(ALTERNATE_TUNINGS['Open D']).toEqual(['D', 'A', 'F#', 'D', 'A', 'D']);
    });

    it('should have DADGAD tuning', () => {
      expect(ALTERNATE_TUNINGS['DADGAD']).toBeDefined();
      expect(ALTERNATE_TUNINGS['DADGAD']).toEqual(['D', 'A', 'G', 'D', 'A', 'D']);
    });

    it('should have Half Step Down tuning', () => {
      expect(ALTERNATE_TUNINGS['Half Step Down']).toBeDefined();
      expect(ALTERNATE_TUNINGS['Half Step Down']).toEqual(['D#', 'A#', 'F#', 'C#', 'G#', 'D#']);
    });

    it('should have Whole Step Down tuning', () => {
      expect(ALTERNATE_TUNINGS['Whole Step Down']).toBeDefined();
      expect(ALTERNATE_TUNINGS['Whole Step Down']).toEqual(['D', 'A', 'F', 'C', 'G', 'D']);
    });

    it('all tunings should have 6 strings', () => {
      for (const tuning of Object.values(ALTERNATE_TUNINGS)) {
        expect(tuning).toHaveLength(6);
      }
    });

    it('all tuning notes should be valid', () => {
      const validNotes = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B',
                         'Db', 'Eb', 'Gb', 'Ab', 'Bb'];
      for (const tuning of Object.values(ALTERNATE_TUNINGS)) {
        for (const note of tuning) {
          expect(validNotes).toContain(note);
        }
      }
    });
  });

  describe('integration tests', () => {
    it('beginner chords should be playable', () => {
      const beginnerChords = getBeginnerChords();

      // Beginner chords shouldn't require high frets
      for (const chord of beginnerChords) {
        const maxFret = Math.max(...chord.strings.map(s => s.fret));
        expect(maxFret).toBeLessThanOrEqual(5);
      }
    });

    it('should be able to find voicings for common chords', () => {
      const commonChords = [
        { root: 'C' as const, quality: 'major' as const },
        { root: 'D' as const, quality: 'major' as const },
        { root: 'E' as const, quality: 'major' as const },
        { root: 'G' as const, quality: 'major' as const },
        { root: 'A' as const, quality: 'major' as const },
        { root: 'A' as const, quality: 'minor' as const },
        { root: 'E' as const, quality: 'minor' as const },
      ];

      for (const { root, quality } of commonChords) {
        const voicings = getGuitarChordVoicings(root, quality);
        expect(voicings.length).toBeGreaterThan(0);
      }
    });

    it('power chords should only use 2-3 strings', () => {
      const powerChords = GUITAR_CHORD_LIBRARY.filter(c => c.quality === 'power');

      for (const chord of powerChords) {
        const playedStrings = chord.strings.filter(s => s.fret >= 0);
        expect(playedStrings.length).toBeLessThanOrEqual(3);
      }
    });

    it('barre chords should have consistent barre frets', () => {
      const barreChords = GUITAR_CHORD_LIBRARY.filter(c => c.barres && c.barres.length > 0);

      for (const chord of barreChords) {
        // Barred fret should appear multiple times
        for (const barreFret of chord.barres!) {
          const count = chord.strings.filter(s => s.fret === barreFret).length;
          expect(count).toBeGreaterThanOrEqual(2);
        }
      }
    });
  });
});
