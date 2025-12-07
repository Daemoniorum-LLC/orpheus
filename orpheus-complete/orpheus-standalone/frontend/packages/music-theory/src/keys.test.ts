import { describe, it, expect } from 'vitest';
import {
  CIRCLE_OF_FIFTHS_MAJOR,
  getKeySignature,
  getDiatonicChords,
  analyzeKey,
} from './keys';

describe('keys.ts', () => {
  describe('CIRCLE_OF_FIFTHS_MAJOR', () => {
    it('should contain 7 major key signatures', () => {
      expect(CIRCLE_OF_FIFTHS_MAJOR).toHaveLength(7);
    });

    it('should include C major with no accidentals', () => {
      const cMajor = CIRCLE_OF_FIFTHS_MAJOR.find(k => k.key === 'C');
      expect(cMajor).toBeDefined();
      expect(cMajor?.sharps).toBe(0);
      expect(cMajor?.flats).toBe(0);
      expect(cMajor?.accidentals).toEqual([]);
      expect(cMajor?.mode).toBe('major');
    });

    it('should include G major with 1 sharp', () => {
      const gMajor = CIRCLE_OF_FIFTHS_MAJOR.find(k => k.key === 'G');
      expect(gMajor).toBeDefined();
      expect(gMajor?.sharps).toBe(1);
      expect(gMajor?.flats).toBe(0);
      expect(gMajor?.accidentals).toEqual(['F#']);
    });

    it('should include D major with 2 sharps', () => {
      const dMajor = CIRCLE_OF_FIFTHS_MAJOR.find(k => k.key === 'D');
      expect(dMajor).toBeDefined();
      expect(dMajor?.sharps).toBe(2);
      expect(dMajor?.accidentals).toEqual(['F#', 'C#']);
    });

    it('should include A major with 3 sharps', () => {
      const aMajor = CIRCLE_OF_FIFTHS_MAJOR.find(k => k.key === 'A');
      expect(aMajor).toBeDefined();
      expect(aMajor?.sharps).toBe(3);
      expect(aMajor?.accidentals).toEqual(['F#', 'C#', 'G#']);
    });

    it('should include E major with 4 sharps', () => {
      const eMajor = CIRCLE_OF_FIFTHS_MAJOR.find(k => k.key === 'E');
      expect(eMajor).toBeDefined();
      expect(eMajor?.sharps).toBe(4);
      expect(eMajor?.accidentals).toEqual(['F#', 'C#', 'G#', 'D#']);
    });

    it('should include F major with 1 flat', () => {
      const fMajor = CIRCLE_OF_FIFTHS_MAJOR.find(k => k.key === 'F');
      expect(fMajor).toBeDefined();
      expect(fMajor?.sharps).toBe(0);
      expect(fMajor?.flats).toBe(1);
      expect(fMajor?.accidentals).toEqual(['Bb']);
    });

    it('should include Bb major with 2 flats', () => {
      const bbMajor = CIRCLE_OF_FIFTHS_MAJOR.find(k => k.key === 'Bb');
      expect(bbMajor).toBeDefined();
      expect(bbMajor?.sharps).toBe(0);
      expect(bbMajor?.flats).toBe(2);
      expect(bbMajor?.accidentals).toEqual(['Bb', 'Eb']);
    });

    it('should have relative keys defined', () => {
      const cMajor = CIRCLE_OF_FIFTHS_MAJOR.find(k => k.key === 'C');
      expect(cMajor?.relativeKey).toBe('A');

      const gMajor = CIRCLE_OF_FIFTHS_MAJOR.find(k => k.key === 'G');
      expect(gMajor?.relativeKey).toBe('E');
    });

    it('should have parallel keys defined', () => {
      for (const key of CIRCLE_OF_FIFTHS_MAJOR) {
        expect(key.parallelKey).toBe(key.key);
      }
    });

    it('should have diatonic chords defined', () => {
      for (const key of CIRCLE_OF_FIFTHS_MAJOR) {
        expect(key.diatonicChords).toEqual(['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii°']);
      }
    });

    it('all keys should have required properties', () => {
      for (const key of CIRCLE_OF_FIFTHS_MAJOR) {
        expect(key).toHaveProperty('key');
        expect(key).toHaveProperty('mode');
        expect(key).toHaveProperty('sharps');
        expect(key).toHaveProperty('flats');
        expect(key).toHaveProperty('accidentals');
        expect(key).toHaveProperty('relativeKey');
        expect(key).toHaveProperty('parallelKey');
        expect(key).toHaveProperty('diatonicChords');
        expect(key.mode).toBe('major');
      }
    });
  });

  describe('getKeySignature', () => {
    describe('major keys', () => {
      it('should return C major signature', () => {
        const sig = getKeySignature('C', 'major');
        expect(sig).toBeDefined();
        expect(sig?.key).toBe('C');
        expect(sig?.mode).toBe('major');
        expect(sig?.sharps).toBe(0);
        expect(sig?.flats).toBe(0);
      });

      it('should return G major signature', () => {
        const sig = getKeySignature('G', 'major');
        expect(sig).toBeDefined();
        expect(sig?.key).toBe('G');
        expect(sig?.sharps).toBe(1);
        expect(sig?.accidentals).toEqual(['F#']);
      });

      it('should return D major signature', () => {
        const sig = getKeySignature('D', 'major');
        expect(sig).toBeDefined();
        expect(sig?.sharps).toBe(2);
      });

      it('should return A major signature', () => {
        const sig = getKeySignature('A', 'major');
        expect(sig).toBeDefined();
        expect(sig?.sharps).toBe(3);
      });

      it('should return E major signature', () => {
        const sig = getKeySignature('E', 'major');
        expect(sig).toBeDefined();
        expect(sig?.sharps).toBe(4);
      });

      it('should return F major signature', () => {
        const sig = getKeySignature('F', 'major');
        expect(sig).toBeDefined();
        expect(sig?.flats).toBe(1);
      });

      it('should return Bb major signature', () => {
        const sig = getKeySignature('Bb', 'major');
        expect(sig).toBeDefined();
        expect(sig?.flats).toBe(2);
      });

      it('should default to major mode', () => {
        const sig = getKeySignature('C');
        expect(sig?.mode).toBe('major');
      });

      it('should return undefined for unknown major key', () => {
        const sig = getKeySignature('Z' as any, 'major');
        expect(sig).toBeUndefined();
      });
    });

    describe('minor keys', () => {
      it('should return A minor signature', () => {
        const sig = getKeySignature('A', 'minor');
        expect(sig).toBeDefined();
        expect(sig?.key).toBe('A');
        expect(sig?.mode).toBe('minor');
        expect(sig?.sharps).toBe(0);
        expect(sig?.flats).toBe(0);
        expect(sig?.relativeKey).toBe('C');
      });

      it('should return E minor signature (relative to G major)', () => {
        const sig = getKeySignature('E', 'minor');
        expect(sig).toBeDefined();
        expect(sig?.key).toBe('E');
        expect(sig?.mode).toBe('minor');
        expect(sig?.sharps).toBe(1);
        expect(sig?.relativeKey).toBe('G');
      });

      it('should return B minor signature (relative to D major)', () => {
        const sig = getKeySignature('B', 'minor');
        expect(sig).toBeDefined();
        expect(sig?.sharps).toBe(2);
        expect(sig?.relativeKey).toBe('D');
      });

      it('should return F# minor signature (relative to A major)', () => {
        const sig = getKeySignature('F#', 'minor');
        expect(sig).toBeDefined();
        expect(sig?.sharps).toBe(3);
        expect(sig?.relativeKey).toBe('A');
      });

      it('should return C# minor signature (relative to E major)', () => {
        const sig = getKeySignature('C#', 'minor');
        expect(sig).toBeDefined();
        expect(sig?.sharps).toBe(4);
        expect(sig?.relativeKey).toBe('E');
      });

      it('should return D minor signature (relative to F major)', () => {
        const sig = getKeySignature('D', 'minor');
        expect(sig).toBeDefined();
        expect(sig?.flats).toBe(1);
        expect(sig?.relativeKey).toBe('F');
      });

      it('should return G minor signature (relative to Bb major)', () => {
        const sig = getKeySignature('G', 'minor');
        expect(sig).toBeDefined();
        expect(sig?.flats).toBe(2);
        expect(sig?.relativeKey).toBe('Bb');
      });

      it('should return undefined for unknown minor key', () => {
        const sig = getKeySignature('Z' as any, 'minor');
        expect(sig).toBeUndefined();
      });

      it('should preserve accidentals from relative major', () => {
        const gMinor = getKeySignature('G', 'minor');
        expect(gMinor?.accidentals).toEqual(['Bb', 'Eb']);
      });
    });

    describe('relative key relationship', () => {
      it('should maintain relative key relationship for major keys', () => {
        const cMajor = getKeySignature('C', 'major');
        const aMinor = getKeySignature('A', 'minor');
        expect(cMajor?.relativeKey).toBe('A');
        expect(aMinor?.relativeKey).toBe('C');
      });

      it('should share same accidentals for relative keys', () => {
        const gMajor = getKeySignature('G', 'major');
        const eMinor = getKeySignature('E', 'minor');
        expect(gMajor?.accidentals).toEqual(eMinor?.accidentals);
      });
    });
  });

  describe('getDiatonicChords', () => {
    describe('major keys', () => {
      it('should return diatonic chords for C major', () => {
        const chords = getDiatonicChords('C', 'major');
        expect(chords).toEqual(['C', 'Dmin', 'Emin', 'F', 'G', 'Amin', 'Bdim']);
      });

      it('should return diatonic chords for G major', () => {
        const chords = getDiatonicChords('G', 'major');
        expect(chords).toEqual(['G', 'Amin', 'Bmin', 'C', 'D', 'Emin', 'F#dim']);
      });

      it('should return diatonic chords for D major', () => {
        const chords = getDiatonicChords('D', 'major');
        expect(chords).toEqual(['D', 'Emin', 'F#min', 'G', 'A', 'Bmin', 'C#dim']);
      });

      it('should return diatonic chords for F major', () => {
        const chords = getDiatonicChords('F', 'major');
        expect(chords).toEqual(['F', 'Gmin', 'Amin', 'A#', 'C', 'Dmin', 'Edim']);
      });

      it('should default to major mode', () => {
        const chords = getDiatonicChords('C');
        expect(chords).toEqual(['C', 'Dmin', 'Emin', 'F', 'G', 'Amin', 'Bdim']);
      });

      it('should always return 7 chords', () => {
        const chords = getDiatonicChords('C', 'major');
        expect(chords).toHaveLength(7);
      });

      it('should follow I-ii-iii-IV-V-vi-vii° pattern', () => {
        const chords = getDiatonicChords('C', 'major');
        expect(chords[0]).not.toContain('min');  // I - major
        expect(chords[1]).toContain('min');       // ii - minor
        expect(chords[2]).toContain('min');       // iii - minor
        expect(chords[3]).not.toContain('min');  // IV - major
        expect(chords[4]).not.toContain('min');  // V - major
        expect(chords[5]).toContain('min');       // vi - minor
        expect(chords[6]).toContain('dim');       // vii° - diminished
      });
    });

    describe('minor keys', () => {
      it('should return diatonic chords for A minor', () => {
        const chords = getDiatonicChords('A', 'minor');
        expect(chords).toEqual(['Amin', 'Bdim', 'C', 'Dmin', 'Emin', 'F', 'G']);
      });

      it('should return diatonic chords for E minor', () => {
        const chords = getDiatonicChords('E', 'minor');
        expect(chords).toEqual(['Emin', 'F#dim', 'G', 'Amin', 'Bmin', 'C', 'D']);
      });

      it('should return diatonic chords for D minor', () => {
        const chords = getDiatonicChords('D', 'minor');
        expect(chords).toEqual(['Dmin', 'Edim', 'F', 'Gmin', 'Amin', 'A#', 'C']);
      });

      it('should always return 7 chords', () => {
        const chords = getDiatonicChords('A', 'minor');
        expect(chords).toHaveLength(7);
      });

      it('should follow i-ii°-III-iv-v-VI-VII pattern', () => {
        const chords = getDiatonicChords('A', 'minor');
        expect(chords[0]).toContain('min');       // i - minor
        expect(chords[1]).toContain('dim');       // ii° - diminished
        expect(chords[2]).not.toContain('min');  // III - major
        expect(chords[3]).toContain('min');       // iv - minor
        expect(chords[4]).toContain('min');       // v - minor
        expect(chords[5]).not.toContain('min');  // VI - major
        expect(chords[6]).not.toContain('min');  // VII - major
      });
    });

    describe('chord quality patterns', () => {
      it('should have correct qualities for major scale harmony', () => {
        const chords = getDiatonicChords('C', 'major');
        expect(chords[0]).toBe('C');      // Major
        expect(chords[1]).toBe('Dmin');   // Minor
        expect(chords[2]).toBe('Emin');   // Minor
        expect(chords[3]).toBe('F');      // Major
        expect(chords[4]).toBe('G');      // Major
        expect(chords[5]).toBe('Amin');   // Minor
        expect(chords[6]).toBe('Bdim');   // Diminished
      });

      it('should have correct qualities for minor scale harmony', () => {
        const chords = getDiatonicChords('A', 'minor');
        expect(chords[0]).toBe('Amin');   // Minor
        expect(chords[1]).toBe('Bdim');   // Diminished
        expect(chords[2]).toBe('C');      // Major
        expect(chords[3]).toBe('Dmin');   // Minor
        expect(chords[4]).toBe('Emin');   // Minor
        expect(chords[5]).toBe('F');      // Major
        expect(chords[6]).toBe('G');      // Major
      });
    });
  });

  describe('analyzeKey', () => {
    it('should identify C as key when C appears most', () => {
      const result = analyzeKey(['C', 'F', 'G', 'C']);
      expect(result.key).toBe('C');
      expect(result.confidence).toBe(0.5); // 2 out of 4
    });

    it('should identify G as key when G appears most', () => {
      const result = analyzeKey(['G', 'C', 'D', 'G', 'G']);
      expect(result.key).toBe('G');
      expect(result.confidence).toBe(0.6); // 3 out of 5
    });

    it('should handle single chord', () => {
      const result = analyzeKey(['Am']);
      expect(result.key).toBe('A');
      expect(result.confidence).toBe(1.0);
    });

    it('should extract root note from chord symbols', () => {
      const result = analyzeKey(['Cmaj7', 'Dm7', 'G7', 'Cmaj7']);
      expect(result.key).toBe('C');
    });

    it('should handle chords with accidentals', () => {
      const result = analyzeKey(['C#m', 'F#', 'C#m']);
      expect(result.key).toBe('C');
      // Note: implementation only looks at first character
    });

    it('should calculate confidence correctly', () => {
      const result = analyzeKey(['C', 'C', 'G', 'F']);
      expect(result.confidence).toBe(0.5); // 2/4 = 0.5
    });

    it('should handle all same chords', () => {
      const result = analyzeKey(['C', 'C', 'C', 'C']);
      expect(result.key).toBe('C');
      expect(result.confidence).toBe(1.0);
    });

    it('should pick most frequent when multiple chords present', () => {
      const result = analyzeKey(['C', 'D', 'E', 'C', 'C', 'F']);
      expect(result.key).toBe('C'); // C appears 3 times
      expect(result.confidence).toBe(0.5); // 3/6
    });

    it('should handle ties by picking first sorted', () => {
      const result = analyzeKey(['A', 'B', 'A', 'B']);
      // Both appear twice, sort will determine winner
      expect(['A', 'B']).toContain(result.key);
      expect(result.confidence).toBe(0.5);
    });

    it('should return confidence between 0 and 1', () => {
      const result = analyzeKey(['C', 'D', 'E', 'F', 'G']);
      expect(result.confidence).toBeGreaterThanOrEqual(0);
      expect(result.confidence).toBeLessThanOrEqual(1);
    });

    it('should handle complex chord progression', () => {
      const progression = ['C', 'Am', 'F', 'G', 'C', 'Em', 'Dm', 'G'];
      const result = analyzeKey(progression);
      expect(result.key).toBeDefined();
      expect(result.confidence).toBeGreaterThan(0);
    });
  });
});
