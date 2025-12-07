/**
 * Enhanced Maestro to alphaTab TEX Converter
 * Converts project notes to alphaTab notation format
 */

import type { MaestroProject } from '@orpheus/shared-types';

/**
 * Converts a Maestro project to alphaTab TEX notation with actual notes
 */
export function convertMaestroToAlphaTabTEX(project: MaestroProject): string {
  const metadata = project.project.metadata;
  const composition = project.project.composition;

  // Build TEX notation
  let tex = '';

  // Title and metadata
  tex += `\\title "${metadata.title || 'Untitled'}"\n`;
  if (metadata.artist) {
    tex += `\\artist "${metadata.artist}"\n`;
  }
  if (metadata.album) {
    tex += `\\album "${metadata.album}"\n`;
  }

  // Tempo
  tex += `\\tempo ${metadata.tempo || 120}\n`;

  // Time signature
  const timeSig = metadata.timeSignature;
  if (timeSig) {
    tex += `\\time ${timeSig.numerator}/${timeSig.denominator}\n`;
  } else {
    tex += `\\time 4/4\n`;
  }

  // Key signature
  tex += `\\key ${metadata.key || 'C'}\n`;

  tex += '.\n'; // Track separator

  // For each track
  const tracks = composition.tracks || [];

  if (tracks.length === 0) {
    // Add a default empty track
    tex += `\\track "Track 1"\n`;
    tex += `\\tuning E2 A2 D3 G3 B3 E4\n`;
    tex += `\\instrument 25\n`;
    tex += '.\n';
    tex += '1.1 2.1 3.1 4.1\n'; // Simple placeholder measure
  } else {
    tracks.forEach((track: any, trackIndex: number) => {
      tex += `\\track "${track.name || `Track ${trackIndex + 1}`}"\n`;

      // Set tuning (standard by default)
      tex += `\\tuning E2 A2 D3 G3 B3 E4\n`;

      // Instrument
      tex += `\\instrument ${mapInstrumentToMIDI(track.instrument?.type)}\n`;

      tex += '.\n';

      // Convert measures to TEX notation
      const measures = track.measures || [];
      if (measures.length === 0) {
        // Empty track - add placeholder
        tex += '1.1 2.1 3.1 4.1\n';
      } else {
        // Process each measure
        measures.forEach((measure: any, measureIndex: number) => {
          const measureTex = convertMeasureToTEX(measure);
          tex += measureTex;

          // Add measure separator (|) except for last measure
          if (measureIndex < measures.length - 1) {
            tex += ' | ';
          }
        });
        tex += '\n'; // End track with newline
      }
    });
  }

  console.log('[TEX Converter] Generated TEX:', tex);
  return tex;
}

/**
 * Convert a single measure to TEX notation
 */
function convertMeasureToTEX(measure: any): string {
  const voices = measure.voices || [];

  if (voices.length === 0) {
    // Empty measure - add rest
    return 'r.1';
  }

  // Use first voice (simplified - alphaTab TEX doesn't support multiple voices easily)
  const voice = voices[0];
  const beats = voice.beats || [];

  if (beats.length === 0) {
    return 'r.1'; // Whole rest
  }

  // Convert each beat to TEX
  const beatTexes: string[] = [];

  beats.forEach((beat: any) => {
    if (beat.rest) {
      beatTexes.push('r.4'); // Quarter rest (simplified)
    } else if (beat.notes && beat.notes.length > 0) {
      // Convert notes to TEX format: fret.string
      const noteTexes = beat.notes.map((note: any) => {
        // alphaTab uses 1-indexed strings from bottom to top
        // Our model uses 1-indexed from top to bottom, so we need to convert
        // String 1 (high E) -> alphaTab string 6
        // String 6 (low E) -> alphaTab string 1
        const alphaTabString = 7 - note.string;
        return `${note.fret}.${alphaTabString}`;
      });

      if (noteTexes.length === 1) {
        // Single note
        beatTexes.push(noteTexes[0]);
      } else {
        // Chord - wrap in parentheses
        beatTexes.push(`(${noteTexes.join(' ')})`);
      }
    }
  });

  return beatTexes.length > 0 ? beatTexes.join(' ') : 'r.1';
}

/**
 * Maps Maestro instrument types to MIDI program numbers for alphaTab
 */
function mapInstrumentToMIDI(type?: string): number {
  if (!type) return 25; // Default to acoustic guitar

  const instrumentMap: Record<string, number> = {
    'guitar': 25,           // Acoustic guitar (nylon)
    'acoustic-guitar': 25,  // Acoustic guitar (nylon)
    'electric-guitar': 30,  // Electric guitar (clean)
    'bass': 33,             // Acoustic bass
    'electric-bass': 34,    // Electric bass (finger)
    'drums': 0,             // Standard drum kit
    'piano': 0,             // Acoustic grand piano
    'keyboard': 0,          // Acoustic grand piano
    'strings': 49,          // String ensemble
    'synth': 81,            // Lead synth
  };

  const normalized = type.toLowerCase().replace(/[_\s-]/g, '-');
  return instrumentMap[normalized] || 25;
}

/**
 * Get a hash/version of the project for change detection
 */
export function getProjectHash(project: MaestroProject | null): string {
  if (!project) return '';

  // Create a simple hash based on project structure
  // In production, you might use a proper hash function
  const modified = project.project.metadata.modified;
  const trackCount = project.project.composition.tracks?.length || 0;
  const measureCounts = (project.project.composition.tracks || [])
    .map((track: any) => track.measures?.length || 0)
    .join('-');

  return `${modified}-${trackCount}-${measureCounts}`;
}
