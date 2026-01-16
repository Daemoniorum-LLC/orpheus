/**
 * Musical scales and modes
 */

import type { Note } from './notes';
import { transpose } from './notes';

export interface ScaleDefinition {
  name: string;
  intervals: number[];  // Semitones from root
  description: string;
  modes?: string[];     // Associated modes
  genres?: string[];
}

/**
 * Comprehensive scale library
 */
export const SCALE_DEFINITIONS: ScaleDefinition[] = [
  // Major scale and modes
  {
    name: 'Major (Ionian)',
    intervals: [0, 2, 4, 5, 7, 9, 11],
    description: 'The foundational major scale, happy and bright',
    modes: ['Ionian', 'Dorian', 'Phrygian', 'Lydian', 'Mixolydian', 'Aeolian', 'Locrian'],
    genres: ['pop', 'rock', 'classical', 'country'],
  },
  {
    name: 'Natural Minor (Aeolian)',
    intervals: [0, 2, 3, 5, 7, 8, 10],
    description: 'The natural minor scale, sad and melancholic',
    genres: ['rock', 'metal', 'classical'],
  },
  {
    name: 'Harmonic Minor',
    intervals: [0, 2, 3, 5, 7, 8, 11],
    description: 'Minor scale with raised 7th, exotic and dramatic',
    genres: ['classical', 'metal', 'flamenco'],
  },
  {
    name: 'Melodic Minor',
    intervals: [0, 2, 3, 5, 7, 9, 11],
    description: 'Minor scale with raised 6th and 7th, smooth and jazzy',
    genres: ['jazz', 'classical'],
  },

  // Modes of major scale
  {
    name: 'Dorian',
    intervals: [0, 2, 3, 5, 7, 9, 10],
    description: 'Minor scale with raised 6th, jazzy and sophisticated',
    genres: ['jazz', 'rock', 'funk'],
  },
  {
    name: 'Phrygian',
    intervals: [0, 1, 3, 5, 7, 8, 10],
    description: 'Minor scale with flat 2nd, Spanish and exotic',
    genres: ['flamenco', 'metal', 'spanish'],
  },
  {
    name: 'Lydian',
    intervals: [0, 2, 4, 6, 7, 9, 11],
    description: 'Major scale with raised 4th, dreamy and floating',
    genres: ['jazz', 'film', 'prog-rock'],
  },
  {
    name: 'Mixolydian',
    intervals: [0, 2, 4, 5, 7, 9, 10],
    description: 'Major scale with flat 7th, bluesy and groovy',
    genres: ['blues', 'rock', 'country'],
  },
  {
    name: 'Locrian',
    intervals: [0, 1, 3, 5, 6, 8, 10],
    description: 'Diminished scale, dark and unstable',
    genres: ['jazz', 'metal', 'experimental'],
  },

  // Pentatonic scales
  {
    name: 'Major Pentatonic',
    intervals: [0, 2, 4, 7, 9],
    description: 'Five-note major scale, simple and universal',
    genres: ['rock', 'blues', 'country', 'pop'],
  },
  {
    name: 'Minor Pentatonic',
    intervals: [0, 3, 5, 7, 10],
    description: 'Five-note minor scale, backbone of blues and rock',
    genres: ['blues', 'rock', 'metal'],
  },

  // Blues scales
  {
    name: 'Blues Scale',
    intervals: [0, 3, 5, 6, 7, 10],
    description: 'Minor pentatonic with added flat 5th, classic blues sound',
    genres: ['blues', 'rock', 'jazz'],
  },
  {
    name: 'Major Blues',
    intervals: [0, 2, 3, 4, 7, 9],
    description: 'Major pentatonic with blues notes',
    genres: ['blues', 'jazz'],
  },

  // Exotic scales
  {
    name: 'Chromatic',
    intervals: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    description: 'All twelve notes, used for chromatic passages',
    genres: ['jazz', 'classical'],
  },
  {
    name: 'Whole Tone',
    intervals: [0, 2, 4, 6, 8, 10],
    description: 'Only whole steps, dreamy and ambiguous',
    genres: ['jazz', 'impressionist'],
  },
  {
    name: 'Diminished (Whole-Half)',
    intervals: [0, 2, 3, 5, 6, 8, 9, 11],
    description: 'Symmetrical scale for diminished chords',
    genres: ['jazz', 'classical'],
  },
  {
    name: 'Diminished (Half-Whole)',
    intervals: [0, 1, 3, 4, 6, 7, 9, 10],
    description: 'Symmetrical scale for dominant chords',
    genres: ['jazz', 'classical'],
  },

  // World scales
  {
    name: 'Hungarian Minor',
    intervals: [0, 2, 3, 6, 7, 8, 11],
    description: 'Exotic minor scale with augmented 4th',
    genres: ['classical', 'metal', 'gypsy'],
  },
  {
    name: 'Phrygian Dominant',
    intervals: [0, 1, 4, 5, 7, 8, 10],
    description: 'Spanish/Middle Eastern sounding scale',
    genres: ['flamenco', 'metal', 'middle-eastern'],
  },
  {
    name: 'Japanese (In Scale)',
    intervals: [0, 1, 5, 7, 8],
    description: 'Traditional Japanese pentatonic scale',
    genres: ['japanese', 'asian'],
  },
  {
    name: 'Arabic',
    intervals: [0, 1, 4, 5, 7, 8, 11],
    description: 'Middle Eastern scale with augmented 2nd',
    genres: ['middle-eastern', 'world'],
  },
];

export interface Scale {
  root: Note;
  name: string;
  notes: Note[];
  intervals: number[];
  description: string;
}

/**
 * Creates a scale from root note and scale name
 */
export function createScale(root: Note, scaleName: string): Scale {
  const definition = SCALE_DEFINITIONS.find(s => s.name === scaleName);

  if (!definition) {
    throw new Error(`Unknown scale: ${scaleName}`);
  }

  const notes = definition.intervals.map(interval => transpose(root, interval));

  return {
    root,
    name: definition.name,
    notes,
    intervals: definition.intervals,
    description: definition.description,
  };
}

/**
 * Gets all scales available
 */
export function getAllScales(): ScaleDefinition[] {
  return SCALE_DEFINITIONS;
}

/**
 * Gets scales by genre
 */
export function getScalesByGenre(genre: string): ScaleDefinition[] {
  return SCALE_DEFINITIONS.filter(scale =>
    scale.genres?.some(g => g.toLowerCase() === genre.toLowerCase())
  );
}

/**
 * Finds scales that contain specific notes
 */
export function findScalesWithNotes(root: Note, notes: Note[]): Scale[] {
  const scales: Scale[] = [];

  for (const definition of SCALE_DEFINITIONS) {
    const scale = createScale(root, definition.name);
    const hasAllNotes = notes.every(note => scale.notes.includes(note));

    if (hasAllNotes) {
      scales.push(scale);
    }
  }

  return scales;
}

/**
 * Guitar-specific: Gets scale patterns (box positions)
 */
export interface ScalePattern {
  name: string;
  position: number;  // 1-5 for CAGED positions
  startFret: number;
  notes: Array<{ string: number; fret: number }>;
}

/**
 * Generates minor pentatonic box patterns for guitar
 */
export function getMinorPentatonicPatterns(root: Note): ScalePattern[] {
  // This would generate the 5 box positions for minor pentatonic
  // For brevity, returning a simplified version
  return [
    {
      name: `${root} Minor Pentatonic - Box 1`,
      position: 1,
      startFret: 5, // Example for A minor
      notes: [
        { string: 6, fret: 5 },
        { string: 6, fret: 8 },
        { string: 5, fret: 5 },
        { string: 5, fret: 7 },
        // ... more notes
      ],
    },
  ];
}

/**
 * Gets the relative major/minor key
 */
export function getRelativeKey(key: Note, currentMode: 'major' | 'minor'): Note {
  if (currentMode === 'major') {
    // Relative minor is 3 semitones down
    return transpose(key, -3);
  } else {
    // Relative major is 3 semitones up
    return transpose(key, 3);
  }
}

/**
 * Gets the parallel major/minor key
 */
export function getParallelKey(key: Note): Note {
  // Parallel keys share the same root
  return key;
}

/**
 * Determines what scales work over a chord
 */
export function getScalesForChord(chordRoot: Note, chordQuality: string): string[] {
  const scaleRecommendations: Record<string, string[]> = {
    'major': ['Major (Ionian)', 'Mixolydian', 'Lydian', 'Major Pentatonic'],
    'minor': ['Natural Minor (Aeolian)', 'Dorian', 'Phrygian', 'Minor Pentatonic', 'Blues Scale'],
    'dominant7': ['Mixolydian', 'Blues Scale', 'Lydian Dominant'],
    'minor7': ['Dorian', 'Aeolian', 'Minor Pentatonic'],
    'major7': ['Major (Ionian)', 'Lydian'],
    'diminished': ['Locrian', 'Diminished (Half-Whole)'],
  };

  return scaleRecommendations[chordQuality] || ['Chromatic'];
}
