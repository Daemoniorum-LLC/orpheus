/**
 * Note and pitch utilities
 */

export const NOTES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'] as const;
export const FLAT_NOTES = ['C', 'Db', 'D', 'Eb', 'E', 'F', 'Gb', 'G', 'Ab', 'A', 'Bb', 'B'] as const;

export type NoteName = typeof NOTES[number];
export type FlatNoteName = typeof FLAT_NOTES[number];
export type Note = NoteName | FlatNoteName;

/**
 * Converts a note to its enharmonic sharp equivalent
 */
export function toSharp(note: Note): NoteName {
  const flatToSharp: Record<string, NoteName> = {
    'Db': 'C#',
    'Eb': 'D#',
    'Gb': 'F#',
    'Ab': 'G#',
    'Bb': 'A#',
  };

  return (flatToSharp[note] || note) as NoteName;
}

/**
 * Converts a note to its enharmonic flat equivalent
 */
export function toFlat(note: Note): FlatNoteName {
  const sharpToFlat: Record<string, FlatNoteName> = {
    'C#': 'Db',
    'D#': 'Eb',
    'F#': 'Gb',
    'G#': 'Ab',
    'A#': 'Bb',
  };

  return (sharpToFlat[note] || note) as FlatNoteName;
}

/**
 * Gets the MIDI note number for a note and octave
 */
export function getMidiNote(note: Note, octave: number): number {
  const noteIndex = NOTES.indexOf(toSharp(note));
  return (octave + 1) * 12 + noteIndex;
}

/**
 * Gets the note name and octave from a MIDI note number
 */
export function fromMidiNote(midiNote: number): { note: NoteName; octave: number } {
  const octave = Math.floor(midiNote / 12) - 1;
  const noteIndex = midiNote % 12;
  const note = NOTES[noteIndex];

  return { note, octave };
}

/**
 * Transposes a note by semitones
 */
export function transpose(note: Note, semitones: number): NoteName {
  const noteIndex = NOTES.indexOf(toSharp(note));
  const newIndex = (noteIndex + semitones + 12) % 12;
  return NOTES[newIndex];
}

/**
 * Gets the frequency in Hz for a note and octave (A4 = 440Hz)
 */
export function getFrequency(note: Note, octave: number): number {
  const midiNote = getMidiNote(note, octave);
  const a4 = 69; // MIDI note for A4
  return 440 * Math.pow(2, (midiNote - a4) / 12);
}

/**
 * Calculates the interval in semitones between two notes
 */
export function intervalBetween(note1: Note, note2: Note): number {
  const index1 = NOTES.indexOf(toSharp(note1));
  const index2 = NOTES.indexOf(toSharp(note2));
  return (index2 - index1 + 12) % 12;
}
