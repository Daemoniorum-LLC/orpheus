/**
 * Chord progression utilities and generators
 */

import type { Note } from './notes';
import { transpose } from './notes';
import { COMMON_PROGRESSIONS, type ChordProgression } from './chords';

/**
 * Converts Roman numerals to actual chords in a key
 */
export function romanNumeralsToChords(
  key: Note,
  romanNumerals: string[],
  mode: 'major' | 'minor' = 'major'
): string[] {
  const scale = mode === 'major'
    ? [0, 2, 4, 5, 7, 9, 11]  // Major scale
    : [0, 2, 3, 5, 7, 8, 10]; // Natural minor

  const qualities = mode === 'major'
    ? ['', 'm', 'm', '', '', 'm', 'dim']
    : ['m', 'dim', '', 'm', 'm', '', ''];

  const romanToIndex: Record<string, number> = {
    'I': 0, 'i': 0,
    'II': 1, 'ii': 1,
    'III': 2, 'iii': 2,
    'IV': 3, 'iv': 3,
    'V': 4, 'v': 4,
    'VI': 5, 'vi': 5,
    'VII': 6, 'vii': 6,
  };

  return romanNumerals.map(numeral => {
    // Remove any chord extensions (7, 9, etc.) for simplicity
    const baseNumeral = numeral.replace(/[0-9°]/g, '');
    const index = romanToIndex[baseNumeral];

    if (index === undefined) {
      throw new Error(`Invalid Roman numeral: ${numeral}`);
    }

    const root = transpose(key, scale[index]);
    const quality = qualities[index];

    // Add back extensions if present
    const extension = numeral.match(/[0-9°]+/)?.[0] || '';

    return `${root}${quality}${extension}`;
  });
}

/**
 * Gets a common progression in a specific key
 */
export function getProgressionInKey(
  progressionName: string,
  key: Note,
  mode: 'major' | 'minor' = 'major'
): string[] {
  const progression = COMMON_PROGRESSIONS.find(
    p => p.name.toLowerCase() === progressionName.toLowerCase()
  );

  if (!progression) {
    throw new Error(`Unknown progression: ${progressionName}`);
  }

  return romanNumeralsToChords(key, progression.romanNumerals, mode);
}

/**
 * Generates a random progression based on common patterns
 */
export function generateProgression(
  key: Note,
  length: number = 4,
  genre?: string
): { chords: string[]; name?: string } {
  let validProgressions = COMMON_PROGRESSIONS;

  if (genre) {
    validProgressions = COMMON_PROGRESSIONS.filter(p =>
      p.genre.includes(genre.toLowerCase())
    );
  }

  if (validProgressions.length === 0) {
    validProgressions = COMMON_PROGRESSIONS;
  }

  // Pick a random progression
  const progression = validProgressions[
    Math.floor(Math.random() * validProgressions.length)
  ];

  // Convert to actual chords
  const chords = romanNumeralsToChords(key, progression.romanNumerals);

  // Repeat if needed to reach desired length
  const repeatedChords: string[] = [];
  while (repeatedChords.length < length) {
    repeatedChords.push(...chords);
  }

  return {
    chords: repeatedChords.slice(0, length),
    name: progression.name,
  };
}

/**
 * Suggests next chord based on current chord and key
 */
export function suggestNextChord(
  currentChord: string,
  key: Note,
  style: 'strong' | 'weak' | 'deceptive' = 'strong'
): string[] {
  // Get the diatonic chords in the key
  const diatonicChords = romanNumeralsToChords(key, ['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii°']);

  // Common progressions from each degree
  const commonMoves: Record<number, number[]> = {
    0: [3, 4, 5],      // I → IV, V, vi
    1: [4, 0],         // ii → V, I
    2: [5, 3],         // iii → vi, IV
    3: [1, 4, 0],      // IV → ii, V, I
    4: [0, 5, 3],      // V → I, vi, IV
    5: [1, 3, 4],      // vi → ii, IV, V
    6: [0],            // vii° → I
  };

  // Find current chord index
  const currentIndex = diatonicChords.findIndex(c =>
    c.replace(/[^A-G#b]/g, '') === currentChord.replace(/[^A-G#b]/g, '')
  );

  if (currentIndex === -1) {
    return diatonicChords.slice(0, 3); // Return first three if not found
  }

  const nextIndices = commonMoves[currentIndex] || [0];
  return nextIndices.map(i => diatonicChords[i]);
}

/**
 * Analyzes the harmonic function of chords in a progression
 */
export function analyzeProgression(chords: string[], key: Note): {
  chord: string;
  function: 'tonic' | 'subdominant' | 'dominant' | 'other';
  degree: string;
}[] {
  const diatonicChords = romanNumeralsToChords(key, ['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii°']);
  const romanNumerals = ['I', 'ii', 'iii', 'IV', 'V', 'vi', 'vii°'];

  return chords.map(chord => {
    const index = diatonicChords.findIndex(c =>
      c.replace(/[^A-G#b]/g, '') === chord.replace(/[^A-G#b]/g, '')
    );

    let harmonicFunction: 'tonic' | 'subdominant' | 'dominant' | 'other' = 'other';

    if (index === 0 || index === 2 || index === 5) {
      harmonicFunction = 'tonic';
    } else if (index === 1 || index === 3) {
      harmonicFunction = 'subdominant';
    } else if (index === 4 || index === 6) {
      harmonicFunction = 'dominant';
    }

    return {
      chord,
      function: harmonicFunction,
      degree: index >= 0 ? romanNumerals[index] : '?',
    };
  });
}
