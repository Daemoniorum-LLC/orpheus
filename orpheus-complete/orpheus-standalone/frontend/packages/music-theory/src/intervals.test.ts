import { describe, it, expect } from 'vitest';
import {
  INTERVALS,
  getInterval,
  getIntervalByName,
  type IntervalName,
} from './intervals';

describe('intervals.ts', () => {
  describe('INTERVALS', () => {
    it('should contain all 13 intervals (including octave)', () => {
      expect(INTERVALS).toHaveLength(13);
    });

    it('should have correct unison', () => {
      const unison = INTERVALS.find(i => i.name === 'unison');
      expect(unison).toBeDefined();
      expect(unison?.semitones).toBe(0);
      expect(unison?.shortName).toBe('P1');
      expect(unison?.quality).toBe('perfect');
    });

    it('should have correct second intervals', () => {
      const minor2 = INTERVALS.find(i => i.name === 'minor2');
      expect(minor2?.semitones).toBe(1);
      expect(minor2?.shortName).toBe('m2');
      expect(minor2?.quality).toBe('minor');

      const major2 = INTERVALS.find(i => i.name === 'major2');
      expect(major2?.semitones).toBe(2);
      expect(major2?.shortName).toBe('M2');
      expect(major2?.quality).toBe('major');
    });

    it('should have correct third intervals', () => {
      const minor3 = INTERVALS.find(i => i.name === 'minor3');
      expect(minor3?.semitones).toBe(3);
      expect(minor3?.shortName).toBe('m3');
      expect(minor3?.quality).toBe('minor');

      const major3 = INTERVALS.find(i => i.name === 'major3');
      expect(major3?.semitones).toBe(4);
      expect(major3?.shortName).toBe('M3');
      expect(major3?.quality).toBe('major');
    });

    it('should have correct perfect4', () => {
      const perfect4 = INTERVALS.find(i => i.name === 'perfect4');
      expect(perfect4?.semitones).toBe(5);
      expect(perfect4?.shortName).toBe('P4');
      expect(perfect4?.quality).toBe('perfect');
    });

    it('should have correct tritone', () => {
      const tritone = INTERVALS.find(i => i.name === 'tritone');
      expect(tritone?.semitones).toBe(6);
      expect(tritone?.shortName).toBe('TT');
      expect(tritone?.quality).toBe('augmented');
    });

    it('should have correct perfect5', () => {
      const perfect5 = INTERVALS.find(i => i.name === 'perfect5');
      expect(perfect5?.semitones).toBe(7);
      expect(perfect5?.shortName).toBe('P5');
      expect(perfect5?.quality).toBe('perfect');
    });

    it('should have correct sixth intervals', () => {
      const minor6 = INTERVALS.find(i => i.name === 'minor6');
      expect(minor6?.semitones).toBe(8);
      expect(minor6?.shortName).toBe('m6');
      expect(minor6?.quality).toBe('minor');

      const major6 = INTERVALS.find(i => i.name === 'major6');
      expect(major6?.semitones).toBe(9);
      expect(major6?.shortName).toBe('M6');
      expect(major6?.quality).toBe('major');
    });

    it('should have correct seventh intervals', () => {
      const minor7 = INTERVALS.find(i => i.name === 'minor7');
      expect(minor7?.semitones).toBe(10);
      expect(minor7?.shortName).toBe('m7');
      expect(minor7?.quality).toBe('minor');

      const major7 = INTERVALS.find(i => i.name === 'major7');
      expect(major7?.semitones).toBe(11);
      expect(major7?.shortName).toBe('M7');
      expect(major7?.quality).toBe('major');
    });

    it('should have correct octave', () => {
      const octave = INTERVALS.find(i => i.name === 'octave');
      expect(octave?.semitones).toBe(12);
      expect(octave?.shortName).toBe('P8');
      expect(octave?.quality).toBe('perfect');
    });

    it('should have unique semitone values for each interval', () => {
      const semitones = INTERVALS.map(i => i.semitones);
      const uniqueSemitones = new Set(semitones);
      expect(uniqueSemitones.size).toBe(INTERVALS.length);
    });

    it('should have unique names for each interval', () => {
      const names = INTERVALS.map(i => i.name);
      const uniqueNames = new Set(names);
      expect(uniqueNames.size).toBe(INTERVALS.length);
    });
  });

  describe('getInterval', () => {
    it('should return interval by exact semitones', () => {
      expect(getInterval(0)?.name).toBe('unison');
      expect(getInterval(1)?.name).toBe('minor2');
      expect(getInterval(2)?.name).toBe('major2');
      expect(getInterval(3)?.name).toBe('minor3');
      expect(getInterval(4)?.name).toBe('major3');
      expect(getInterval(5)?.name).toBe('perfect4');
      expect(getInterval(6)?.name).toBe('tritone');
      expect(getInterval(7)?.name).toBe('perfect5');
      expect(getInterval(8)?.name).toBe('minor6');
      expect(getInterval(9)?.name).toBe('major6');
      expect(getInterval(10)?.name).toBe('minor7');
      expect(getInterval(11)?.name).toBe('major7');
    });

    it('should handle octave (12 semitones)', () => {
      const result = getInterval(12);
      expect(result?.name).toBe('octave');
      expect(result?.semitones).toBe(12);
    });

    it('should handle semitones wrapping within octave', () => {
      // 13 semitones = 1 semitone (mod 12)
      expect(getInterval(13)?.name).toBe('minor2');
      expect(getInterval(14)?.name).toBe('major2');
      expect(getInterval(24)?.name).toBe('octave');
      expect(getInterval(25)).toBeUndefined(); // 25 % 12 = 1, but we're checking exact match
    });

    it('should return undefined for invalid semitones', () => {
      expect(getInterval(13)).toBeUndefined();
      expect(getInterval(25)).toBeUndefined();
      expect(getInterval(100)).toBeUndefined();
    });

    it('should handle negative semitones', () => {
      // -1 % 12 in JavaScript is -1, so this won't match
      expect(getInterval(-1)).toBeUndefined();
    });

    it('should return complete interval object', () => {
      const interval = getInterval(4);
      expect(interval).toBeDefined();
      expect(interval).toHaveProperty('name');
      expect(interval).toHaveProperty('semitones');
      expect(interval).toHaveProperty('shortName');
      expect(interval).toHaveProperty('quality');
    });
  });

  describe('getIntervalByName', () => {
    const intervalNames: IntervalName[] = [
      'unison',
      'minor2',
      'major2',
      'minor3',
      'major3',
      'perfect4',
      'tritone',
      'perfect5',
      'minor6',
      'major6',
      'minor7',
      'major7',
      'octave',
    ];

    it('should return interval for all valid names', () => {
      for (const name of intervalNames) {
        const interval = getIntervalByName(name);
        expect(interval).toBeDefined();
        expect(interval?.name).toBe(name);
      }
    });

    it('should return correct semitones for named intervals', () => {
      expect(getIntervalByName('unison')?.semitones).toBe(0);
      expect(getIntervalByName('minor2')?.semitones).toBe(1);
      expect(getIntervalByName('major2')?.semitones).toBe(2);
      expect(getIntervalByName('minor3')?.semitones).toBe(3);
      expect(getIntervalByName('major3')?.semitones).toBe(4);
      expect(getIntervalByName('perfect4')?.semitones).toBe(5);
      expect(getIntervalByName('tritone')?.semitones).toBe(6);
      expect(getIntervalByName('perfect5')?.semitones).toBe(7);
      expect(getIntervalByName('minor6')?.semitones).toBe(8);
      expect(getIntervalByName('major6')?.semitones).toBe(9);
      expect(getIntervalByName('minor7')?.semitones).toBe(10);
      expect(getIntervalByName('major7')?.semitones).toBe(11);
      expect(getIntervalByName('octave')?.semitones).toBe(12);
    });

    it('should return correct quality for named intervals', () => {
      expect(getIntervalByName('unison')?.quality).toBe('perfect');
      expect(getIntervalByName('minor2')?.quality).toBe('minor');
      expect(getIntervalByName('major2')?.quality).toBe('major');
      expect(getIntervalByName('perfect4')?.quality).toBe('perfect');
      expect(getIntervalByName('tritone')?.quality).toBe('augmented');
      expect(getIntervalByName('perfect5')?.quality).toBe('perfect');
      expect(getIntervalByName('octave')?.quality).toBe('perfect');
    });

    it('should return correct short names', () => {
      expect(getIntervalByName('unison')?.shortName).toBe('P1');
      expect(getIntervalByName('minor2')?.shortName).toBe('m2');
      expect(getIntervalByName('major2')?.shortName).toBe('M2');
      expect(getIntervalByName('perfect4')?.shortName).toBe('P4');
      expect(getIntervalByName('tritone')?.shortName).toBe('TT');
      expect(getIntervalByName('perfect5')?.shortName).toBe('P5');
      expect(getIntervalByName('octave')?.shortName).toBe('P8');
    });

    it('should return complete interval object', () => {
      const interval = getIntervalByName('major3');
      expect(interval).toBeDefined();
      expect(interval).toHaveProperty('name');
      expect(interval).toHaveProperty('semitones');
      expect(interval).toHaveProperty('shortName');
      expect(interval).toHaveProperty('quality');
    });
  });

  describe('edge cases and consistency', () => {
    it('getInterval and getIntervalByName should be consistent', () => {
      for (const interval of INTERVALS) {
        const byName = getIntervalByName(interval.name);
        const bySemitones = interval.semitones < 13 ? getInterval(interval.semitones) : undefined;

        expect(byName).toEqual(interval);
        if (interval.semitones <= 12) {
          expect(bySemitones).toEqual(interval);
        }
      }
    });

    it('should handle all quality types', () => {
      const qualities = INTERVALS.map(i => i.quality);
      expect(qualities).toContain('perfect');
      expect(qualities).toContain('major');
      expect(qualities).toContain('minor');
      expect(qualities).toContain('augmented');
    });
  });
});
