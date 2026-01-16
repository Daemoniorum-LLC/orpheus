/**
 * Key signatures and circle of fifths
 */

import type { Note } from './notes';
import { transpose } from './notes';

export interface KeySignature {
  key: Note;
  mode: 'major' | 'minor';
  sharps: number;
  flats: number;
  accidentals: Note[];
  relativeKey: Note;
  parallelKey: Note;
  diatonicChords: string[];  // Roman numerals
}

/**
 * Circle of Fifths - Major Keys
 */
export const CIRCLE_OF_FIFTHS_MAJOR: KeySignature[] = [
  {
    key: 'C',
    mode: 'major',
    sharps: 0,
    flats: 0,
    accidentals: [],
    relativeKey: 'A',
    parallelKey: 'C',
    diatonicChords: ['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii°'],
  },
  {
    key: 'G',
    mode: 'major',
    sharps: 1,
    flats: 0,
    accidentals: ['F#'],
    relativeKey: 'E',
    parallelKey: 'G',
    diatonicChords: ['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii°'],
  },
  {
    key: 'D',
    mode: 'major',
    sharps: 2,
    flats: 0,
    accidentals: ['F#', 'C#'],
    relativeKey: 'B',
    parallelKey: 'D',
    diatonicChords: ['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii°'],
  },
  {
    key: 'A',
    mode: 'major',
    sharps: 3,
    flats: 0,
    accidentals: ['F#', 'C#', 'G#'],
    relativeKey: 'F#',
    parallelKey: 'A',
    diatonicChords: ['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii°'],
  },
  {
    key: 'E',
    mode: 'major',
    sharps: 4,
    flats: 0,
    accidentals: ['F#', 'C#', 'G#', 'D#'],
    relativeKey: 'C#',
    parallelKey: 'E',
    diatonicChords: ['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii°'],
  },
  {
    key: 'F',
    mode: 'major',
    sharps: 0,
    flats: 1,
    accidentals: ['Bb'],
    relativeKey: 'D',
    parallelKey: 'F',
    diatonicChords: ['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii°'],
  },
  {
    key: 'Bb',
    mode: 'major',
    sharps: 0,
    flats: 2,
    accidentals: ['Bb', 'Eb'],
    relativeKey: 'G',
    parallelKey: 'Bb',
    diatonicChords: ['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii°'],
  },
];

/**
 * Gets the key signature for a given key
 */
export function getKeySignature(key: Note, mode: 'major' | 'minor' = 'major'): KeySignature | undefined {
  if (mode === 'major') {
    return CIRCLE_OF_FIFTHS_MAJOR.find(k => k.key === key);
  } else {
    // Convert to relative major for lookup
    const relativeMajor = transpose(key, 3);
    const majorKey = CIRCLE_OF_FIFTHS_MAJOR.find(k => k.key === relativeMajor);

    if (majorKey) {
      return {
        ...majorKey,
        key,
        mode: 'minor',
        relativeKey: majorKey.key,
      };
    }
  }
}

/**
 * Gets diatonic chords in a key
 */
export function getDiatonicChords(key: Note, mode: 'major' | 'minor' = 'major'): string[] {
  const qualities = mode === 'major'
    ? ['maj', 'min', 'min', 'maj', 'maj', 'min', 'dim']
    : ['min', 'dim', 'maj', 'min', 'min', 'maj', 'maj'];

  const roots: Note[] = [];
  const scale = mode === 'major'
    ? [0, 2, 4, 5, 7, 9, 11]  // Major scale intervals
    : [0, 2, 3, 5, 7, 8, 10]; // Natural minor scale intervals

  for (const interval of scale) {
    roots.push(transpose(key, interval));
  }

  return roots.map((root, i) => `${root}${qualities[i] === 'maj' ? '' : qualities[i]}`);
}

/**
 * Analyzes a chord progression to determine the key
 */
export function analyzeKey(chords: string[]): { key: Note; confidence: number } {
  // Simplified key detection based on chord frequency
  // In a real implementation, this would be more sophisticated

  const chordRoots = chords.map(chord => chord.charAt(0) as Note);
  const frequency: Record<string, number> = {};

  for (const root of chordRoots) {
    frequency[root] = (frequency[root] || 0) + 1;
  }

  // Most frequent chord is likely the tonic
  const sortedRoots = Object.entries(frequency)
    .sort(([, a], [, b]) => b - a);

  const likelyKey = sortedRoots[0][0] as Note;
  const confidence = sortedRoots[0][1] / chords.length;

  return { key: likelyKey, confidence };
}
