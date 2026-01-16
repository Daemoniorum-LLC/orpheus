/**
 * Chord definitions and utilities
 */

import type { Note } from './notes';
import { transpose } from './notes';

export type ChordQuality =
  | 'major'
  | 'minor'
  | 'diminished'
  | 'augmented'
  | 'dominant7'
  | 'major7'
  | 'minor7'
  | 'minorMajor7'
  | 'diminished7'
  | 'halfDiminished7'
  | 'sus2'
  | 'sus4'
  | 'add9'
  | 'major6'
  | 'minor6'
  | '9th'
  | 'minor9'
  | 'major9'
  | '11th'
  | '13th'
  | 'power';

export interface ChordDefinition {
  quality: ChordQuality;
  intervals: number[];  // Semitones from root
  symbol: string;       // How it's written (e.g., "m", "7", "maj7")
  name: string;         // Full name
  abbr?: string[];      // Alternative abbreviations
}

/**
 * Comprehensive chord library
 */
export const CHORD_DEFINITIONS: ChordDefinition[] = [
  // Triads
  {
    quality: 'major',
    intervals: [0, 4, 7],
    symbol: '',
    name: 'Major',
    abbr: ['M', 'maj'],
  },
  {
    quality: 'minor',
    intervals: [0, 3, 7],
    symbol: 'm',
    name: 'Minor',
    abbr: ['min', '-'],
  },
  {
    quality: 'diminished',
    intervals: [0, 3, 6],
    symbol: 'dim',
    name: 'Diminished',
    abbr: ['°', 'o'],
  },
  {
    quality: 'augmented',
    intervals: [0, 4, 8],
    symbol: 'aug',
    name: 'Augmented',
    abbr: ['+'],
  },

  // Seventh chords
  {
    quality: 'dominant7',
    intervals: [0, 4, 7, 10],
    symbol: '7',
    name: 'Dominant 7th',
  },
  {
    quality: 'major7',
    intervals: [0, 4, 7, 11],
    symbol: 'maj7',
    name: 'Major 7th',
    abbr: ['M7', 'Δ7'],
  },
  {
    quality: 'minor7',
    intervals: [0, 3, 7, 10],
    symbol: 'm7',
    name: 'Minor 7th',
    abbr: ['min7', '-7'],
  },
  {
    quality: 'minorMajor7',
    intervals: [0, 3, 7, 11],
    symbol: 'mM7',
    name: 'Minor-Major 7th',
    abbr: ['m(maj7)', '-Δ7'],
  },
  {
    quality: 'diminished7',
    intervals: [0, 3, 6, 9],
    symbol: 'dim7',
    name: 'Diminished 7th',
    abbr: ['°7', 'o7'],
  },
  {
    quality: 'halfDiminished7',
    intervals: [0, 3, 6, 10],
    symbol: 'm7♭5',
    name: 'Half-Diminished 7th',
    abbr: ['ø7', 'ø'],
  },

  // Suspended chords
  {
    quality: 'sus2',
    intervals: [0, 2, 7],
    symbol: 'sus2',
    name: 'Suspended 2nd',
  },
  {
    quality: 'sus4',
    intervals: [0, 5, 7],
    symbol: 'sus4',
    name: 'Suspended 4th',
  },

  // Sixth chords
  {
    quality: 'major6',
    intervals: [0, 4, 7, 9],
    symbol: '6',
    name: 'Major 6th',
  },
  {
    quality: 'minor6',
    intervals: [0, 3, 7, 9],
    symbol: 'm6',
    name: 'Minor 6th',
  },

  // Added tone chords
  {
    quality: 'add9',
    intervals: [0, 4, 7, 14],
    symbol: 'add9',
    name: 'Add 9',
  },

  // Extended chords
  {
    quality: '9th',
    intervals: [0, 4, 7, 10, 14],
    symbol: '9',
    name: 'Dominant 9th',
  },
  {
    quality: 'minor9',
    intervals: [0, 3, 7, 10, 14],
    symbol: 'm9',
    name: 'Minor 9th',
  },
  {
    quality: 'major9',
    intervals: [0, 4, 7, 11, 14],
    symbol: 'maj9',
    name: 'Major 9th',
    abbr: ['M9', 'Δ9'],
  },
  {
    quality: '11th',
    intervals: [0, 4, 7, 10, 14, 17],
    symbol: '11',
    name: 'Dominant 11th',
  },
  {
    quality: '13th',
    intervals: [0, 4, 7, 10, 14, 21],
    symbol: '13',
    name: 'Dominant 13th',
  },

  // Power chord (for guitar)
  {
    quality: 'power',
    intervals: [0, 7],
    symbol: '5',
    name: 'Power Chord',
  },
];

export interface Chord {
  root: Note;
  quality: ChordQuality;
  notes: Note[];
  symbol: string;  // e.g., "Cm7"
  name: string;    // e.g., "C minor 7th"
}

/**
 * Creates a chord from root note and quality
 */
export function createChord(root: Note, quality: ChordQuality): Chord {
  const definition = CHORD_DEFINITIONS.find(d => d.quality === quality);

  if (!definition) {
    throw new Error(`Unknown chord quality: ${quality}`);
  }

  const notes = definition.intervals.map(interval => transpose(root, interval));

  return {
    root,
    quality,
    notes,
    symbol: `${root}${definition.symbol}`,
    name: `${root} ${definition.name}`,
  };
}

/**
 * Gets all chord voicings for a quality
 */
export function getChordDefinition(quality: ChordQuality): ChordDefinition | undefined {
  return CHORD_DEFINITIONS.find(d => d.quality === quality);
}

/**
 * Analyzes notes to determine chord quality
 */
export function analyzeChord(notes: Note[]): ChordQuality | null {
  if (notes.length < 2) return null;

  // Convert notes to intervals from first note (assumed root)
  const root = notes[0];
  const intervals = notes.map(note => {
    const rootIndex = root.charCodeAt(0) - 'A'.charCodeAt(0);
    const noteIndex = note.charCodeAt(0) - 'A'.charCodeAt(0);
    return (noteIndex - rootIndex + 12) % 12;
  }).sort((a, b) => a - b);

  // Find matching chord definition
  for (const def of CHORD_DEFINITIONS) {
    const defIntervals = def.intervals.filter(i => i < 12).sort((a, b) => a - b);
    if (JSON.stringify(intervals) === JSON.stringify(defIntervals)) {
      return def.quality;
    }
  }

  return null;
}

/**
 * Gets common chord progressions
 */
export interface ChordProgression {
  name: string;
  romanNumerals: string[];
  description: string;
  genre: string[];
  example?: string;  // Example in C major
}

export const COMMON_PROGRESSIONS: ChordProgression[] = [
  {
    name: 'I-IV-V',
    romanNumerals: ['I', 'IV', 'V'],
    description: 'The most fundamental progression in Western music',
    genre: ['rock', 'pop', 'blues', 'country'],
    example: 'C - F - G',
  },
  {
    name: 'I-V-vi-IV',
    romanNumerals: ['I', 'V', 'vi', 'IV'],
    description: 'The "pop-punk" progression, used in countless hit songs',
    genre: ['pop', 'rock'],
    example: 'C - G - Am - F',
  },
  {
    name: 'I-vi-IV-V',
    romanNumerals: ['I', 'vi', 'IV', 'V'],
    description: 'The "50s progression" or "doo-wop" progression',
    genre: ['pop', 'rock', 'doo-wop'],
    example: 'C - Am - F - G',
  },
  {
    name: 'ii-V-I',
    romanNumerals: ['ii', 'V', 'I'],
    description: 'The fundamental jazz progression',
    genre: ['jazz'],
    example: 'Dm7 - G7 - Cmaj7',
  },
  {
    name: 'I-IV-I-V',
    romanNumerals: ['I', 'IV', 'I', 'V'],
    description: 'Classic 12-bar blues progression',
    genre: ['blues'],
    example: 'C7 - F7 - C7 - G7',
  },
  {
    name: 'vi-IV-I-V',
    romanNumerals: ['vi', 'IV', 'I', 'V'],
    description: 'The "sensitive" progression, often used for emotional songs',
    genre: ['pop', 'ballad'],
    example: 'Am - F - C - G',
  },
  {
    name: 'I-bVII-IV',
    romanNumerals: ['I', 'bVII', 'IV'],
    description: 'Mixolydian progression, common in rock',
    genre: ['rock'],
    example: 'C - Bb - F',
  },
  {
    name: 'i-VI-III-VII',
    romanNumerals: ['i', 'VI', 'III', 'VII'],
    description: 'Andalusian cadence, flamenco progression',
    genre: ['flamenco', 'metal'],
    example: 'Am - F - C - G',
  },
];
