/**
 * Guitar-specific chord voicings and fingerings
 */

import type { Note, ChordQuality } from './index';

export interface GuitarString {
  string: number;  // 1-6 (high E to low E)
  fret: number;    // 0 = open, -1 = muted/not played
}

export interface GuitarChordVoicing {
  root: Note;
  quality: ChordQuality;
  name: string;          // e.g., "C major"
  symbol: string;        // e.g., "C"
  strings: GuitarString[];
  baseFret: number;      // Starting fret for barre chords
  barres?: number[];     // Frets that are barred
  fingers?: number[];    // Finger numbers (0=open, 1-4=fingers)
  difficulty: 'beginner' | 'intermediate' | 'advanced';
  category: 'open' | 'barre' | 'moveable' | 'jazz';
}

/**
 * Comprehensive library of guitar chord voicings
 * String numbering: 1=high E, 6=low E
 * Fret numbering: 0=open, -1=muted
 */
export const GUITAR_CHORD_LIBRARY: GuitarChordVoicing[] = [
  // ===== C Major Family =====
  {
    root: 'C',
    quality: 'major',
    name: 'C major',
    symbol: 'C',
    strings: [
      { string: 1, fret: 0 },  // E
      { string: 2, fret: 1 },  // C
      { string: 3, fret: 0 },  // G
      { string: 4, fret: 2 },  // E
      { string: 5, fret: 3 },  // C
      { string: 6, fret: -1 }, // Muted
    ],
    baseFret: 0,
    fingers: [0, 1, 0, 2, 3, 0],
    difficulty: 'beginner',
    category: 'open',
  },
  {
    root: 'C',
    quality: 'minor',
    name: 'C minor',
    symbol: 'Cm',
    strings: [
      { string: 1, fret: 3 },
      { string: 2, fret: 4 },
      { string: 3, fret: 5 },
      { string: 4, fret: 5 },
      { string: 5, fret: 3 },
      { string: 6, fret: -1 },
    ],
    baseFret: 3,
    barres: [3],
    fingers: [1, 3, 4, 4, 1, 0],
    difficulty: 'intermediate',
    category: 'barre',
  },
  {
    root: 'C',
    quality: 'dominant7',
    name: 'C dominant 7th',
    symbol: 'C7',
    strings: [
      { string: 1, fret: 0 },
      { string: 2, fret: 1 },
      { string: 3, fret: 3 },
      { string: 4, fret: 2 },
      { string: 5, fret: 3 },
      { string: 6, fret: -1 },
    ],
    baseFret: 0,
    fingers: [0, 1, 3, 2, 4, 0],
    difficulty: 'beginner',
    category: 'open',
  },

  // ===== D Major Family =====
  {
    root: 'D',
    quality: 'major',
    name: 'D major',
    symbol: 'D',
    strings: [
      { string: 1, fret: 2 },
      { string: 2, fret: 3 },
      { string: 3, fret: 2 },
      { string: 4, fret: 0 },
      { string: 5, fret: -1 },
      { string: 6, fret: -1 },
    ],
    baseFret: 0,
    fingers: [1, 3, 2, 0, 0, 0],
    difficulty: 'beginner',
    category: 'open',
  },
  {
    root: 'D',
    quality: 'minor',
    name: 'D minor',
    symbol: 'Dm',
    strings: [
      { string: 1, fret: 1 },
      { string: 2, fret: 3 },
      { string: 3, fret: 2 },
      { string: 4, fret: 0 },
      { string: 5, fret: -1 },
      { string: 6, fret: -1 },
    ],
    baseFret: 0,
    fingers: [1, 3, 2, 0, 0, 0],
    difficulty: 'beginner',
    category: 'open',
  },

  // ===== E Major Family =====
  {
    root: 'E',
    quality: 'major',
    name: 'E major',
    symbol: 'E',
    strings: [
      { string: 1, fret: 0 },
      { string: 2, fret: 0 },
      { string: 3, fret: 1 },
      { string: 4, fret: 2 },
      { string: 5, fret: 2 },
      { string: 6, fret: 0 },
    ],
    baseFret: 0,
    fingers: [0, 0, 1, 2, 3, 0],
    difficulty: 'beginner',
    category: 'open',
  },
  {
    root: 'E',
    quality: 'minor',
    name: 'E minor',
    symbol: 'Em',
    strings: [
      { string: 1, fret: 0 },
      { string: 2, fret: 0 },
      { string: 3, fret: 0 },
      { string: 4, fret: 2 },
      { string: 5, fret: 2 },
      { string: 6, fret: 0 },
    ],
    baseFret: 0,
    fingers: [0, 0, 0, 2, 3, 0],
    difficulty: 'beginner',
    category: 'open',
  },
  {
    root: 'E',
    quality: 'dominant7',
    name: 'E dominant 7th',
    symbol: 'E7',
    strings: [
      { string: 1, fret: 0 },
      { string: 2, fret: 0 },
      { string: 3, fret: 1 },
      { string: 4, fret: 0 },
      { string: 5, fret: 2 },
      { string: 6, fret: 0 },
    ],
    baseFret: 0,
    fingers: [0, 0, 1, 0, 2, 0],
    difficulty: 'beginner',
    category: 'open',
  },

  // ===== G Major Family =====
  {
    root: 'G',
    quality: 'major',
    name: 'G major',
    symbol: 'G',
    strings: [
      { string: 1, fret: 3 },
      { string: 2, fret: 0 },
      { string: 3, fret: 0 },
      { string: 4, fret: 0 },
      { string: 5, fret: 2 },
      { string: 6, fret: 3 },
    ],
    baseFret: 0,
    fingers: [3, 0, 0, 0, 2, 4],
    difficulty: 'beginner',
    category: 'open',
  },
  {
    root: 'G',
    quality: 'dominant7',
    name: 'G dominant 7th',
    symbol: 'G7',
    strings: [
      { string: 1, fret: 1 },
      { string: 2, fret: 0 },
      { string: 3, fret: 0 },
      { string: 4, fret: 0 },
      { string: 5, fret: 2 },
      { string: 6, fret: 3 },
    ],
    baseFret: 0,
    fingers: [1, 0, 0, 0, 2, 3],
    difficulty: 'beginner',
    category: 'open',
  },

  // ===== A Major Family =====
  {
    root: 'A',
    quality: 'major',
    name: 'A major',
    symbol: 'A',
    strings: [
      { string: 1, fret: 0 },
      { string: 2, fret: 2 },
      { string: 3, fret: 2 },
      { string: 4, fret: 2 },
      { string: 5, fret: 0 },
      { string: 6, fret: -1 },
    ],
    baseFret: 0,
    fingers: [0, 2, 3, 4, 0, 0],
    difficulty: 'beginner',
    category: 'open',
  },
  {
    root: 'A',
    quality: 'minor',
    name: 'A minor',
    symbol: 'Am',
    strings: [
      { string: 1, fret: 0 },
      { string: 2, fret: 1 },
      { string: 3, fret: 2 },
      { string: 4, fret: 2 },
      { string: 5, fret: 0 },
      { string: 6, fret: -1 },
    ],
    baseFret: 0,
    fingers: [0, 1, 2, 3, 0, 0],
    difficulty: 'beginner',
    category: 'open',
  },

  // ===== Power Chords (Moveable) =====
  {
    root: 'E',
    quality: 'power',
    name: 'E5 power chord',
    symbol: 'E5',
    strings: [
      { string: 1, fret: -1 },
      { string: 2, fret: -1 },
      { string: 3, fret: -1 },
      { string: 4, fret: 2 },
      { string: 5, fret: 2 },
      { string: 6, fret: 0 },
    ],
    baseFret: 0,
    fingers: [0, 0, 0, 1, 1, 0],
    difficulty: 'beginner',
    category: 'moveable',
  },
  {
    root: 'A',
    quality: 'power',
    name: 'A5 power chord',
    symbol: 'A5',
    strings: [
      { string: 1, fret: -1 },
      { string: 2, fret: -1 },
      { string: 3, fret: -1 },
      { string: 4, fret: 2 },
      { string: 5, fret: 0 },
      { string: 6, fret: -1 },
    ],
    baseFret: 0,
    fingers: [0, 0, 0, 1, 0, 0],
    difficulty: 'beginner',
    category: 'moveable',
  },

  // ===== Barre Chords (F shape) =====
  {
    root: 'F',
    quality: 'major',
    name: 'F major',
    symbol: 'F',
    strings: [
      { string: 1, fret: 1 },
      { string: 2, fret: 1 },
      { string: 3, fret: 2 },
      { string: 4, fret: 3 },
      { string: 5, fret: 3 },
      { string: 6, fret: 1 },
    ],
    baseFret: 1,
    barres: [1],
    fingers: [1, 1, 2, 3, 4, 1],
    difficulty: 'intermediate',
    category: 'barre',
  },
  {
    root: 'F',
    quality: 'minor',
    name: 'F minor',
    symbol: 'Fm',
    strings: [
      { string: 1, fret: 1 },
      { string: 2, fret: 1 },
      { string: 3, fret: 1 },
      { string: 4, fret: 3 },
      { string: 5, fret: 3 },
      { string: 6, fret: 1 },
    ],
    baseFret: 1,
    barres: [1],
    fingers: [1, 1, 1, 3, 4, 1],
    difficulty: 'intermediate',
    category: 'barre',
  },

  // ===== Jazz Voicings =====
  {
    root: 'C',
    quality: 'major7',
    name: 'C major 7th (jazz voicing)',
    symbol: 'Cmaj7',
    strings: [
      { string: 1, fret: 0 },
      { string: 2, fret: 0 },
      { string: 3, fret: 0 },
      { string: 4, fret: 2 },
      { string: 5, fret: 3 },
      { string: 6, fret: -1 },
    ],
    baseFret: 0,
    fingers: [0, 0, 0, 2, 3, 0],
    difficulty: 'intermediate',
    category: 'jazz',
  },
  {
    root: 'D',
    quality: 'minor7',
    name: 'D minor 7th',
    symbol: 'Dm7',
    strings: [
      { string: 1, fret: 1 },
      { string: 2, fret: 1 },
      { string: 3, fret: 2 },
      { string: 4, fret: 0 },
      { string: 5, fret: -1 },
      { string: 6, fret: -1 },
    ],
    baseFret: 0,
    fingers: [1, 1, 2, 0, 0, 0],
    difficulty: 'beginner',
    category: 'open',
  },
];

/**
 * Gets all voicings for a specific chord
 */
export function getGuitarChordVoicings(root: Note, quality: ChordQuality): GuitarChordVoicing[] {
  return GUITAR_CHORD_LIBRARY.filter(
    chord => chord.root === root && chord.quality === quality
  );
}

/**
 * Gets beginner-friendly chord voicings
 */
export function getBeginnerChords(): GuitarChordVoicing[] {
  return GUITAR_CHORD_LIBRARY.filter(chord => chord.difficulty === 'beginner');
}

/**
 * Transposes a guitar chord voicing to a different key
 */
export function transposeGuitarChord(
  voicing: GuitarChordVoicing,
  semitones: number
): GuitarChordVoicing {
  return {
    ...voicing,
    strings: voicing.strings.map(s => ({
      ...s,
      fret: s.fret >= 0 ? s.fret + semitones : s.fret,
    })),
    baseFret: voicing.baseFret + semitones,
  };
}

/**
 * Standard guitar tuning (EADGBE)
 */
export const STANDARD_TUNING: Note[] = ['E', 'B', 'G', 'D', 'A', 'E'];

/**
 * Common alternate tunings
 */
export const ALTERNATE_TUNINGS: Record<string, Note[]> = {
  'Drop D': ['E', 'B', 'G', 'D', 'A', 'D'],
  'Open G': ['D', 'B', 'G', 'D', 'G', 'D'],
  'Open D': ['D', 'A', 'F#', 'D', 'A', 'D'],
  'DADGAD': ['D', 'A', 'G', 'D', 'A', 'D'],
  'Half Step Down': ['D#', 'A#', 'F#', 'C#', 'G#', 'D#'],
  'Whole Step Down': ['D', 'A', 'F', 'C', 'G', 'D'],
};
