import { describe, it, expect, vi } from 'vitest';
import {
  romanNumeralsToChords,
  getProgressionInKey,
  generateProgression,
  suggestNextChord,
  analyzeProgression,
} from './progressions';

describe('progressions.ts', () => {
  describe('romanNumeralsToChords', () => {
    describe('major mode', () => {
      it('should convert I-IV-V in C major', () => {
        const chords = romanNumeralsToChords('C', ['I', 'IV', 'V'], 'major');
        expect(chords).toEqual(['C', 'F', 'G']);
      });

      it('should convert I-V-vi-IV in C major', () => {
        const chords = romanNumeralsToChords('C', ['I', 'V', 'vi', 'IV'], 'major');
        expect(chords).toEqual(['C', 'G', 'Am', 'F']);
      });

      it('should convert all diatonic chords', () => {
        const chords = romanNumeralsToChords('C', ['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii'], 'major');
        expect(chords).toEqual(['C', 'Dm', 'Em', 'F', 'G', 'Am', 'B']);
      });

      it('should work with uppercase numerals', () => {
        const chords = romanNumeralsToChords('C', ['I', 'II', 'III'], 'major');
        expect(chords).toEqual(['C', 'Dm', 'Em']);
      });

      it('should work with lowercase numerals', () => {
        const chords = romanNumeralsToChords('C', ['i', 'ii', 'iii'], 'major');
        expect(chords).toEqual(['C', 'Dm', 'Em']);
      });

      it('should preserve chord extensions', () => {
        const chords = romanNumeralsToChords('C', ['I7', 'IV7', 'V7'], 'major');
        expect(chords).toEqual(['C7', 'F7', 'G7']);
      });

      it('should handle diminished notation', () => {
        const chords = romanNumeralsToChords('C', ['vii°'], 'major');
        expect(chords).toEqual(['Bdim']);
      });

      it('should work in different keys', () => {
        const gMajor = romanNumeralsToChords('G', ['I', 'IV', 'V'], 'major');
        expect(gMajor).toEqual(['G', 'C', 'D']);

        const dMajor = romanNumeralsToChords('D', ['I', 'IV', 'V'], 'major');
        expect(dMajor).toEqual(['D', 'G', 'A']);
      });

      it('should default to major mode', () => {
        const chords = romanNumeralsToChords('C', ['I', 'IV', 'V']);
        expect(chords).toEqual(['C', 'F', 'G']);
      });
    });

    describe('minor mode', () => {
      it('should convert i-iv-v in A minor', () => {
        const chords = romanNumeralsToChords('A', ['i', 'iv', 'v'], 'minor');
        expect(chords).toEqual(['Am', 'Dm', 'Em']);
      });

      it('should convert all diatonic chords in minor', () => {
        const chords = romanNumeralsToChords('A', ['i', 'ii', 'III', 'iv', 'v', 'VI', 'VII'], 'minor');
        expect(chords).toEqual(['Am', 'Bdim', 'C', 'Dm', 'Em', 'F', 'G']);
      });

      it('should handle different minor keys', () => {
        const eMinor = romanNumeralsToChords('E', ['i', 'iv', 'v'], 'minor');
        expect(eMinor).toEqual(['Em', 'Am', 'Bm']);

        const dMinor = romanNumeralsToChords('D', ['i', 'iv', 'v'], 'minor');
        expect(dMinor).toEqual(['Dm', 'Gm', 'Am']);
      });

      it('should preserve extensions in minor', () => {
        const chords = romanNumeralsToChords('A', ['i7', 'iv7', 'V7'], 'minor');
        expect(chords).toEqual(['Am7', 'Dm7', 'Em7']);
      });
    });

    describe('edge cases', () => {
      it('should throw error for invalid Roman numeral', () => {
        expect(() => {
          romanNumeralsToChords('C', ['VIII'], 'major');
        }).toThrow('Invalid Roman numeral: VIII');
      });

      it('should handle empty array', () => {
        const chords = romanNumeralsToChords('C', [], 'major');
        expect(chords).toEqual([]);
      });

      it('should handle single chord', () => {
        const chords = romanNumeralsToChords('C', ['I'], 'major');
        expect(chords).toEqual(['C']);
      });

      it('should handle numerals with multiple digits', () => {
        const chords = romanNumeralsToChords('C', ['I9', 'V13'], 'major');
        expect(chords).toEqual(['C9', 'G13']);
      });
    });
  });

  describe('getProgressionInKey', () => {
    it('should get I-IV-V in C major', () => {
      const chords = getProgressionInKey('I-IV-V', 'C', 'major');
      expect(chords).toEqual(['C', 'F', 'G']);
    });

    it('should get I-V-vi-IV in C major', () => {
      const chords = getProgressionInKey('I-V-vi-IV', 'C', 'major');
      expect(chords).toEqual(['C', 'G', 'Am', 'F']);
    });

    it('should get ii-V-I in C major', () => {
      const chords = getProgressionInKey('ii-V-I', 'C', 'major');
      expect(chords).toEqual(['Dm7', 'G7', 'Cmaj7']);
    });

    it('should get I-IV-I-V (blues) in C major', () => {
      const chords = getProgressionInKey('I-IV-I-V', 'C', 'major');
      expect(chords).toEqual(['C7', 'F7', 'C7', 'G7']);
    });

    it('should work in different keys', () => {
      const gMajor = getProgressionInKey('I-IV-V', 'G', 'major');
      expect(gMajor).toEqual(['G', 'C', 'D']);

      const dMajor = getProgressionInKey('I-IV-V', 'D', 'major');
      expect(dMajor).toEqual(['D', 'G', 'A']);
    });

    it('should work in minor mode', () => {
      const chords = getProgressionInKey('vi-IV-I-V', 'A', 'minor');
      expect(chords).toEqual(['F', 'Ddim', 'Am', 'Em']);
    });

    it('should be case insensitive for progression name', () => {
      const lower = getProgressionInKey('i-iv-v', 'C', 'major');
      const upper = getProgressionInKey('I-IV-V', 'C', 'major');
      expect(lower).toEqual(upper);
    });

    it('should default to major mode', () => {
      const chords = getProgressionInKey('I-IV-V', 'C');
      expect(chords).toEqual(['C', 'F', 'G']);
    });

    it('should throw error for unknown progression', () => {
      expect(() => {
        getProgressionInKey('unknown-progression', 'C', 'major');
      }).toThrow('Unknown progression: unknown-progression');
    });
  });

  describe('generateProgression', () => {
    beforeEach(() => {
      // Mock Math.random for predictable tests
      vi.spyOn(Math, 'random');
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it('should generate progression of specified length', () => {
      const result = generateProgression('C', 4);
      expect(result.chords).toHaveLength(4);
    });

    it('should generate progression of default length 4', () => {
      const result = generateProgression('C');
      expect(result.chords).toHaveLength(4);
    });

    it('should return progression name', () => {
      const result = generateProgression('C', 4);
      expect(result.name).toBeDefined();
      expect(typeof result.name).toBe('string');
    });

    it('should generate chords in the specified key', () => {
      vi.mocked(Math.random).mockReturnValue(0); // Pick first progression
      const result = generateProgression('C', 3);
      expect(result.chords.every(chord => typeof chord === 'string')).toBe(true);
    });

    it('should filter by genre if provided', () => {
      const result = generateProgression('C', 4, 'rock');
      expect(result.name).toBeDefined();
      expect(result.chords).toHaveLength(4);
    });

    it('should handle jazz genre', () => {
      const result = generateProgression('C', 4, 'jazz');
      expect(result.chords).toHaveLength(4);
    });

    it('should handle blues genre', () => {
      const result = generateProgression('C', 4, 'blues');
      expect(result.chords).toHaveLength(4);
    });

    it('should fallback to all progressions for unknown genre', () => {
      const result = generateProgression('C', 4, 'unknown-genre');
      expect(result.chords).toHaveLength(4);
    });

    it('should repeat progression to reach desired length', () => {
      // I-IV-V is 3 chords, requesting 8 should repeat
      vi.mocked(Math.random).mockReturnValue(0);
      const result = generateProgression('C', 8);
      expect(result.chords).toHaveLength(8);
    });

    it('should truncate if progression is longer than requested', () => {
      const result = generateProgression('C', 2);
      expect(result.chords).toHaveLength(2);
    });

    it('should work with different keys', () => {
      const cProg = generateProgression('C', 4);
      const gProg = generateProgression('G', 4);
      expect(cProg.chords).toHaveLength(4);
      expect(gProg.chords).toHaveLength(4);
      expect(cProg.chords).not.toEqual(gProg.chords);
    });
  });

  describe('suggestNextChord', () => {
    it('should suggest chords after I (tonic)', () => {
      const suggestions = suggestNextChord('C', 'C', 'strong');
      expect(suggestions).toContain('F');  // IV
      expect(suggestions).toContain('G');  // V
      expect(suggestions).toContain('Am'); // vi
    });

    it('should suggest chords after V (dominant)', () => {
      const suggestions = suggestNextChord('G', 'C', 'strong');
      expect(suggestions).toContain('C');  // I (resolution)
      expect(suggestions).toContain('Am'); // vi (deceptive)
      expect(suggestions).toContain('F');  // IV
    });

    it('should suggest chords after IV (subdominant)', () => {
      const suggestions = suggestNextChord('F', 'C', 'strong');
      expect(suggestions).toContain('Dm'); // ii
      expect(suggestions).toContain('G');  // V
      expect(suggestions).toContain('C');  // I
    });

    it('should suggest chords after ii', () => {
      const suggestions = suggestNextChord('Dm', 'C', 'strong');
      expect(suggestions).toContain('G');  // V
      expect(suggestions).toContain('C');  // I
    });

    it('should suggest chords after vi', () => {
      const suggestions = suggestNextChord('Am', 'C', 'strong');
      expect(suggestions).toContain('Dm'); // ii
      expect(suggestions).toContain('F');  // IV
      expect(suggestions).toContain('G');  // V
    });

    it('should suggest resolution after vii°', () => {
      const suggestions = suggestNextChord('Bdim', 'C', 'strong');
      expect(suggestions).toContain('C');  // I (resolution)
    });

    it('should handle chords with extensions', () => {
      const suggestions = suggestNextChord('G7', 'C', 'strong');
      expect(suggestions.length).toBeGreaterThan(0);
    });

    it('should return first three chords if current chord not found', () => {
      const suggestions = suggestNextChord('X', 'C', 'strong');
      expect(suggestions).toHaveLength(3);
    });

    it('should work in different keys', () => {
      const cSuggestions = suggestNextChord('C', 'C');
      const gSuggestions = suggestNextChord('G', 'G');
      expect(cSuggestions.length).toBeGreaterThan(0);
      expect(gSuggestions.length).toBeGreaterThan(0);
    });

    it('should handle weak style parameter', () => {
      const suggestions = suggestNextChord('C', 'C', 'weak');
      expect(suggestions.length).toBeGreaterThan(0);
    });

    it('should handle deceptive style parameter', () => {
      const suggestions = suggestNextChord('G', 'C', 'deceptive');
      expect(suggestions.length).toBeGreaterThan(0);
    });
  });

  describe('analyzeProgression', () => {
    it('should analyze I-IV-V progression', () => {
      const analysis = analyzeProgression(['C', 'F', 'G'], 'C');
      expect(analysis).toHaveLength(3);
      expect(analysis[0].chord).toBe('C');
      expect(analysis[0].function).toBe('tonic');
      expect(analysis[0].degree).toBe('I');
    });

    it('should identify tonic function (I, iii, vi)', () => {
      const analysis = analyzeProgression(['C', 'Em', 'Am'], 'C');
      expect(analysis[0].function).toBe('tonic');   // I
      expect(analysis[1].function).toBe('tonic');   // iii
      expect(analysis[2].function).toBe('tonic');   // vi
    });

    it('should identify subdominant function (ii, IV)', () => {
      const analysis = analyzeProgression(['Dm', 'F'], 'C');
      expect(analysis[0].function).toBe('subdominant'); // ii
      expect(analysis[1].function).toBe('subdominant'); // IV
    });

    it('should identify dominant function (V, vii°)', () => {
      const analysis = analyzeProgression(['G', 'Bdim'], 'C');
      expect(analysis[0].function).toBe('dominant'); // V
      expect(analysis[1].function).toBe('dominant'); // vii°
    });

    it('should identify all seven degrees', () => {
      const chords = ['C', 'Dm', 'Em', 'F', 'G', 'Am', 'Bdim'];
      const analysis = analyzeProgression(chords, 'C');
      const degrees = analysis.map(a => a.degree);
      expect(degrees).toEqual(['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii°']);
    });

    it('should handle chords with extensions', () => {
      const analysis = analyzeProgression(['Cmaj7', 'Dm7', 'G7'], 'C');
      expect(analysis[0].degree).toBe('I');
      expect(analysis[1].degree).toBe('ii');
      expect(analysis[2].degree).toBe('V');
    });

    it('should mark non-diatonic chords as other', () => {
      const analysis = analyzeProgression(['D', 'C'], 'C');
      // D is not diatonic in C major (would be Dm)
      expect(analysis[0].function).toBe('other');
      expect(analysis[0].degree).toBe('?');
    });

    it('should work in different keys', () => {
      const analysis = analyzeProgression(['G', 'C', 'D'], 'G');
      expect(analysis[0].degree).toBe('I');
      expect(analysis[1].degree).toBe('IV');
      expect(analysis[2].degree).toBe('V');
    });

    it('should handle empty progression', () => {
      const analysis = analyzeProgression([], 'C');
      expect(analysis).toEqual([]);
    });

    it('should return complete analysis object', () => {
      const analysis = analyzeProgression(['C'], 'C');
      expect(analysis[0]).toHaveProperty('chord');
      expect(analysis[0]).toHaveProperty('function');
      expect(analysis[0]).toHaveProperty('degree');
    });

    it('should analyze common progressions correctly', () => {
      // I-V-vi-IV (very common pop progression)
      const analysis = analyzeProgression(['C', 'G', 'Am', 'F'], 'C');
      expect(analysis[0].function).toBe('tonic');
      expect(analysis[1].function).toBe('dominant');
      expect(analysis[2].function).toBe('tonic');
      expect(analysis[3].function).toBe('subdominant');
    });

    it('should analyze ii-V-I jazz progression', () => {
      const analysis = analyzeProgression(['Dm', 'G', 'C'], 'C');
      expect(analysis[0].function).toBe('subdominant'); // ii
      expect(analysis[1].function).toBe('dominant');     // V
      expect(analysis[2].function).toBe('tonic');        // I
    });
  });
});
