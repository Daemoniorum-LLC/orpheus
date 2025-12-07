/**
 * Note Scheduler
 * Parses Maestro project data and schedules notes for playback
 */

import * as Tone from 'tone';
import type { MaestroProject } from '@maestro-ai/shared-types';

export interface ScheduledNote {
  trackId: string;
  midiNote: number;
  startTime: number;  // in seconds
  duration: number;   // in seconds
  velocity: number;   // 0-1
  techniques?: string[];
}

export interface PlaybackSchedule {
  notes: ScheduledNote[];
  totalDuration: number;
}

/**
 * Converts string/fret to MIDI note number
 */
function stringFretToMidi(string: number, fret: number, tuning: number[]): number {
  // Strings are 1-based, convert to 0-based
  const stringIndex = string - 1;

  // Get open string MIDI note from tuning
  const openStringMidi = tuning[stringIndex] || 40; // Default to E2 if tuning not available

  // Add fret number to get final MIDI note
  return openStringMidi + fret;
}

/**
 * Standard guitar tuning (E2, A2, D3, G3, B3, E4)
 */
const STANDARD_TUNING = [40, 45, 50, 55, 59, 64];

/**
 * Converts tuning strings (like ['E', 'A', 'D', 'G', 'B', 'E']) to MIDI numbers
 */
function parseTuning(tuningStrings?: string[]): number[] {
  if (!tuningStrings || tuningStrings.length === 0) {
    return STANDARD_TUNING;
  }

  const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

  // Standard guitar octaves (low to high)
  const standardOctaves = [2, 2, 3, 3, 3, 4];

  return tuningStrings.map((noteStr, index) => {
    const cleanNote = noteStr.trim();
    const noteIndex = noteNames.indexOf(cleanNote);

    if (noteIndex === -1) {
      // Fallback to standard tuning for this string
      return STANDARD_TUNING[index] || 40;
    }

    const octave = standardOctaves[index] || 3;
    return (octave + 1) * 12 + noteIndex;
  });
}

/**
 * Converts duration string to seconds
 */
function durationToSeconds(duration: string | undefined, tempo: number): number {
  if (!duration) return 0.5; // Default to quarter note

  const beatDuration = 60 / tempo; // Duration of one quarter note in seconds

  // Parse duration format like "1/4", "1/8", "1/16", etc.
  const match = duration.match(/^(\d+)\/(\d+)$/);
  if (match) {
    const numerator = parseInt(match[1]);
    const denominator = parseInt(match[2]);
    return (numerator / denominator) * beatDuration * 4; // *4 because quarter note is the base
  }

  // Duration map for named durations
  const durationMap: Record<string, number> = {
    'whole': beatDuration * 4,
    'half': beatDuration * 2,
    'quarter': beatDuration,
    'eighth': beatDuration / 2,
    'sixteenth': beatDuration / 4,
    'thirty-second': beatDuration / 8,
  };

  return durationMap[duration.toLowerCase()] || beatDuration;
}

/**
 * Schedules notes from a Maestro project for playback
 */
export function scheduleProjectNotes(project: MaestroProject): PlaybackSchedule {
  const notes: ScheduledNote[] = [];
  const tempo = project.project.metadata.tempo || 120;
  const composition = project.project.composition;

  // Get measures
  const measures = composition.measures || [];
  if (measures.length === 0) {
    console.warn('[NoteScheduler] No measures found in project');
    return { notes: [], totalDuration: 0 };
  }

  // Build tuning map for each track
  const trackTunings = new Map<string, number[]>();
  (composition.tracks || []).forEach((track: any) => {
    const tuning = parseTuning(track.instrument?.tuning);
    trackTunings.set(track.id, tuning);
  });

  let currentTime = 0;

  // Process each measure
  measures.forEach((measure: any) => {
    const measureTempo = measure.tempo || tempo;
    const beatDuration = 60 / measureTempo;

    // Process each track in this measure
    if (measure.tracks) {
      measure.tracks.forEach((trackMeasure: any) => {
        const trackId = trackMeasure.trackId;
        const tuning = trackTunings.get(trackId) || STANDARD_TUNING;

        // Process each note in this track/measure
        if (trackMeasure.notes) {
          trackMeasure.notes.forEach((note: any) => {
            // Convert string/fret to MIDI note
            const midiNote = stringFretToMidi(note.string, note.fret, tuning);

            // Calculate duration
            const duration = durationToSeconds(note.duration, measureTempo);

            // Get velocity (0-1)
            const velocity = note.velocity !== undefined ? note.velocity : 0.8;

            // Extract technique names
            const techniques = (note.techniques || []).map((t: any) =>
              typeof t === 'string' ? t : t.type
            );

            notes.push({
              trackId,
              midiNote,
              startTime: currentTime,
              duration,
              velocity,
              techniques,
            });
          });
        }
      });
    }

    // Advance time by measure duration
    // Calculate measure duration based on time signature
    const timeSignature = measure.timeSignature || project.project.metadata.timeSignature || { numerator: 4, denominator: 4 };
    const beatsPerMeasure = timeSignature.numerator;
    const measureDuration = beatsPerMeasure * beatDuration;

    currentTime += measureDuration;
  });

  console.log(`[NoteScheduler] Scheduled ${notes.length} notes across ${measures.length} measures`);
  console.log(`[NoteScheduler] Total duration: ${currentTime.toFixed(2)}s`);

  return {
    notes,
    totalDuration: currentTime,
  };
}

/**
 * Applies playing techniques to a synth
 * Note: This is a helper function for potential future use
 * Currently techniques are applied in MIDIPlaybackEngine.applyTechniqueModifications
 */
export function applyTechniques(
  _synth: Tone.PolySynth,
  note: ScheduledNote
): void {
  if (!note.techniques || note.techniques.length === 0) return;

  // Handle specific techniques
  note.techniques.forEach(technique => {
    switch (technique.toLowerCase()) {
      case 'palm-mute':
      case 'muted':
        // Reduce sustain and release for palm mute
        // This is a simplified approach - would need per-note envelope in production
        break;

      case 'staccato':
        // Shorten the note
        // This is handled by reducing duration in the note data
        break;

      case 'vibrato':
        // Apply vibrato effect (would need LFO in production)
        break;

      case 'bend':
        // Apply pitch bend (would need automation in production)
        break;

      case 'slide':
        // Apply portamento/glide (would need custom synth)
        break;

      case 'hammer-on':
      case 'pull-off':
        // Reduce attack for legato techniques
        break;
    }
  });
}

/**
 * Converts MIDI note number to Tone.js note name
 */
export function midiNoteToName(midiNote: number): string {
  const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
  const octave = Math.floor(midiNote / 12) - 1;
  const noteName = noteNames[midiNote % 12];
  return `${noteName}${octave}`;
}
