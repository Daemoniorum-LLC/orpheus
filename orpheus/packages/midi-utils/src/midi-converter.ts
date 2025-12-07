/**
 * Convert between MIDI and other formats
 */

import type { Note } from '@orpheus/music-theory';
import type { MidiNote } from '@orpheus/shared-types';
import { fromMidiNote, getMidiNote } from '@orpheus/music-theory';

/**
 * Converts tablature notation to MIDI notes
 */
export function tabToMIDI(
  string: number,
  fret: number,
  tuning: Note[] = ['E', 'A', 'D', 'G', 'B', 'E']
): { note: number; noteName: Note; octave: number } {
  // Standard guitar tuning: E2, A2, D3, G3, B3, E4
  const stringTuning = tuning[6 - string]; // Strings are numbered 1-6 (high to low)

  // Get base MIDI note for open string
  const openStringOctaves = [4, 3, 3, 3, 2, 2]; // E4, B3, G3, D3, A2, E2
  const openStringNote = getMidiNote(stringTuning, openStringOctaves[string - 1]);

  // Add fret offset
  const midiNote = openStringNote + fret;

  const { note: noteName, octave } = fromMidiNote(midiNote);

  return { note: midiNote, noteName, octave };
}

/**
 * Converts MIDI note to tablature notation (finds best position)
 */
export function midiToTab(
  midiNote: number,
  tuning: Note[] = ['E', 'A', 'D', 'G', 'B', 'E']
): Array<{ string: number; fret: number }> {
  const positions: Array<{ string: number; fret: number }> = [];

  // Check each string to see if the note can be played
  for (let string = 1; string <= 6; string++) {
    const openStringOctaves = [4, 3, 3, 3, 2, 2];
    const stringTuning = tuning[6 - string];
    const openStringNote = getMidiNote(stringTuning, openStringOctaves[string - 1]);

    const fret = midiNote - openStringNote;

    // Valid fret range: 0-24
    if (fret >= 0 && fret <= 24) {
      positions.push({ string, fret });
    }
  }

  return positions;
}

/**
 * Converts musical time (beats) to MIDI ticks
 */
export function beatsToTicks(beats: number, division: number): number {
  return Math.round(beats * division);
}

/**
 * Converts MIDI ticks to musical time (beats)
 */
export function ticksToBeats(ticks: number, division: number): number {
  return ticks / division;
}

/**
 * Converts tempo (BPM) to microseconds per quarter note
 */
export function bpmToMicroseconds(bpm: number): number {
  return Math.round(60000000 / bpm);
}

/**
 * Converts microseconds per quarter note to tempo (BPM)
 */
export function microsecondsToBPM(microseconds: number): number {
  return Math.round(60000000 / microseconds);
}

/**
 * Quantizes MIDI note timing to a grid
 */
export function quantizeNotes(
  notes: MidiNote[],
  gridSize: number, // in beats (e.g., 0.25 for 16th notes)
  division: number = 480
): MidiNote[] {
  return notes.map(note => {
    const startBeats = note.startTime;
    const quantizedStart = Math.round(startBeats / gridSize) * gridSize;

    return {
      ...note,
      startTime: quantizedStart,
    };
  });
}

/**
 * Humanizes MIDI notes by adding slight timing and velocity variations
 */
export function humanizeNotes(
  notes: MidiNote[],
  options: {
    timingVariation?: number;  // Max variation in beats (e.g., 0.01)
    velocityVariation?: number; // Max variation in velocity (e.g., 5)
  } = {}
): MidiNote[] {
  const { timingVariation = 0.01, velocityVariation = 5 } = options;

  return notes.map(note => {
    const timingOffset = (Math.random() - 0.5) * 2 * timingVariation;
    const velocityOffset = Math.round((Math.random() - 0.5) * 2 * velocityVariation);

    return {
      ...note,
      startTime: note.startTime + timingOffset,
      velocity: Math.max(1, Math.min(127, note.velocity + velocityOffset)),
    };
  });
}
