/**
 * Musical intervals
 */

export type IntervalName =
  | 'unison'
  | 'minor2'
  | 'major2'
  | 'minor3'
  | 'major3'
  | 'perfect4'
  | 'tritone'
  | 'perfect5'
  | 'minor6'
  | 'major6'
  | 'minor7'
  | 'major7'
  | 'octave';

export interface Interval {
  name: IntervalName;
  semitones: number;
  shortName: string;
  quality: 'perfect' | 'major' | 'minor' | 'augmented' | 'diminished';
}

export const INTERVALS: Interval[] = [
  { name: 'unison', semitones: 0, shortName: 'P1', quality: 'perfect' },
  { name: 'minor2', semitones: 1, shortName: 'm2', quality: 'minor' },
  { name: 'major2', semitones: 2, shortName: 'M2', quality: 'major' },
  { name: 'minor3', semitones: 3, shortName: 'm3', quality: 'minor' },
  { name: 'major3', semitones: 4, shortName: 'M3', quality: 'major' },
  { name: 'perfect4', semitones: 5, shortName: 'P4', quality: 'perfect' },
  { name: 'tritone', semitones: 6, shortName: 'TT', quality: 'augmented' },
  { name: 'perfect5', semitones: 7, shortName: 'P5', quality: 'perfect' },
  { name: 'minor6', semitones: 8, shortName: 'm6', quality: 'minor' },
  { name: 'major6', semitones: 9, shortName: 'M6', quality: 'major' },
  { name: 'minor7', semitones: 10, shortName: 'm7', quality: 'minor' },
  { name: 'major7', semitones: 11, shortName: 'M7', quality: 'major' },
  { name: 'octave', semitones: 12, shortName: 'P8', quality: 'perfect' },
];

/**
 * Gets an interval by semitones
 */
export function getInterval(semitones: number): Interval | undefined {
  return INTERVALS.find(i => i.semitones === (semitones % 12));
}

/**
 * Gets an interval by name
 */
export function getIntervalByName(name: IntervalName): Interval | undefined {
  return INTERVALS.find(i => i.name === name);
}
