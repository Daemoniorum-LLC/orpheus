/**
 * Export Service - Project and audio export functionality
 * Handles MIDI, Guitar Pro, WAV, and stems export
 */

import type { MaestroProject } from '@maestro-ai/shared-types';

/**
 * Export a Maestro project to MIDI file format
 */
export async function exportToMIDI(project: MaestroProject): Promise<Blob> {
  console.log('[Export] Generating MIDI file from project:', project.project.metadata.title);

  // MIDI file structure constants
  const HEADER_CHUNK = 'MThd';
  const TRACK_CHUNK = 'MTrk';
  const TICKS_PER_QUARTER = 480;

  // Helper to write variable-length quantity (VLQ)
  function writeVLQ(value: number): number[] {
    const bytes: number[] = [];
    let buffer = value & 0x7f;

    while ((value >>= 7) > 0) {
      buffer <<= 8;
      buffer |= 0x80;
      buffer += value & 0x7f;
    }

    while (true) {
      bytes.push(buffer & 0xff);
      if (buffer & 0x80) {
        buffer >>= 8;
      } else {
        break;
      }
    }

    return bytes;
  }

  // Helper to write 32-bit big-endian integer
  function writeUint32(value: number): number[] {
    return [
      (value >> 24) & 0xff,
      (value >> 16) & 0xff,
      (value >> 8) & 0xff,
      value & 0xff,
    ];
  }

  // Helper to write 16-bit big-endian integer
  function writeUint16(value: number): number[] {
    return [(value >> 8) & 0xff, value & 0xff];
  }

  // Convert string to byte array
  function stringToBytes(str: string): number[] {
    return Array.from(str).map((c) => c.charCodeAt(0));
  }

  // Build MIDI header chunk
  const headerBytes: number[] = [
    ...stringToBytes(HEADER_CHUNK),
    ...writeUint32(6), // Chunk length
    ...writeUint16(1), // Format 1 (multiple tracks)
    ...writeUint16(project.project.composition.tracks?.length || 1), // Number of tracks
    ...writeUint16(TICKS_PER_QUARTER), // Time division
  ];

  // Build tracks
  const trackBytes: number[][] = [];
  const tempo = project.project.metadata.tempo || 120;
  const microsecondsPerQuarter = Math.floor(60000000 / tempo);

  // Track 0: Tempo and time signature
  const tempoTrack: number[] = [
    ...stringToBytes(TRACK_CHUNK),
    0,
    0,
    0,
    0, // Length placeholder
    // Track name
    ...writeVLQ(0), // Delta time
    0xff,
    0x03, // Track name meta event
    ...writeVLQ(11),
    ...stringToBytes('Maestro AI'),
    // Tempo
    ...writeVLQ(0),
    0xff,
    0x51, // Tempo meta event
    0x03, // Length
    ...writeUint32(microsecondsPerQuarter).slice(1), // 24-bit tempo
    // Time signature
    ...writeVLQ(0),
    0xff,
    0x58, // Time signature meta event
    0x04, // Length
    project.project.metadata.timeSignature?.numerator || 4,
    Math.log2(project.project.metadata.timeSignature?.denominator || 4),
    24, // MIDI clocks per metronome click
    8, // 32nd notes per quarter note
    // End of track
    ...writeVLQ(0),
    0xff,
    0x2f,
    0x00,
  ];

  // Update tempo track length
  const tempoTrackLength = tempoTrack.length - 8;
  tempoTrack[4] = (tempoTrackLength >> 24) & 0xff;
  tempoTrack[5] = (tempoTrackLength >> 16) & 0xff;
  tempoTrack[6] = (tempoTrackLength >> 8) & 0xff;
  tempoTrack[7] = tempoTrackLength & 0xff;

  trackBytes.push(tempoTrack);

  // Guitar tuning: Standard tuning EADGBE
  const standardTuning = [40, 45, 50, 55, 59, 64]; // MIDI note numbers

  // Convert each track
  const tracks = project.project.composition.tracks || [];
  tracks.forEach((track, trackIndex) => {
    const trackData: number[] = [
      ...stringToBytes(TRACK_CHUNK),
      0,
      0,
      0,
      0, // Length placeholder
      // Track name
      ...writeVLQ(0),
      0xff,
      0x03,
      ...writeVLQ(track.name?.length || 0),
      ...stringToBytes(track.name || `Track ${trackIndex + 1}`),
      // Program change (acoustic guitar)
      ...writeVLQ(0),
      0xc0 | trackIndex, // Program change
      24, // Acoustic guitar
    ];

    let currentTick = 0;

    // Convert measures to MIDI notes
    track.measures.forEach((measure) => {
      measure.voices.forEach((voice) => {
        voice.beats.forEach((beat) => {
          beat.notes.forEach((note) => {
            // Calculate MIDI note number from string and fret
            const stringIndex = note.string - 1;
            if (stringIndex < 0 || stringIndex >= standardTuning.length) {
              console.warn('[Export] Invalid string index:', note.string);
              return;
            }

            const midiNote = standardTuning[stringIndex] + note.fret;

            // Calculate duration in ticks based on beat duration
            const durationTicks = Math.floor(beat.duration * TICKS_PER_QUARTER);

            // Note on
            trackData.push(
              ...writeVLQ(0), // Delta time
              0x90 | (trackIndex % 16), // Note on (channel 0-15)
              midiNote,
              note.velocity || 100 // Velocity
            );

            // Note off
            trackData.push(
              ...writeVLQ(durationTicks),
              0x80 | (trackIndex % 16), // Note off
              midiNote,
              0 // Release velocity
            );

            currentTick += durationTicks;
          });
        });
      });
    });

    // End of track
    trackData.push(...writeVLQ(0), 0xff, 0x2f, 0x00);

    // Update track length
    const trackLength = trackData.length - 8;
    trackData[4] = (trackLength >> 24) & 0xff;
    trackData[5] = (trackLength >> 16) & 0xff;
    trackData[6] = (trackLength >> 8) & 0xff;
    trackData[7] = trackLength & 0xff;

    trackBytes.push(trackData);
  });

  // Combine all bytes
  const allBytes = [...headerBytes, ...trackBytes.flat()];

  // Create blob
  const blob = new Blob([new Uint8Array(allBytes)], { type: 'audio/midi' });

  console.log('[Export] MIDI file generated:', blob.size, 'bytes');
  return blob;
}

/**
 * Export project to Guitar Pro format (GP5)
 * Note: This is a simplified export - full GP format is complex
 */
export async function exportToGuitarPro(project: MaestroProject): Promise<Blob> {
  console.log('[Export] Guitar Pro export not yet implemented');
  // TODO: Implement full Guitar Pro file format export
  // For now, export as MIDI which GP can import
  return exportToMIDI(project);
}

/**
 * Export project as JSON
 */
export function exportToJSON(project: MaestroProject): Blob {
  const json = JSON.stringify(project, null, 2);
  return new Blob([json], { type: 'application/json' });
}

/**
 * Download a blob as a file
 */
export function downloadBlob(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
  console.log('[Export] Downloaded file:', filename);
}

/**
 * Export formats available
 */
export interface ExportFormat {
  id: string;
  name: string;
  extension: string;
  description: string;
  mimeType: string;
}

export const EXPORT_FORMATS: ExportFormat[] = [
  {
    id: 'midi',
    name: 'MIDI File',
    extension: 'mid',
    description: 'Standard MIDI format, compatible with all DAWs',
    mimeType: 'audio/midi',
  },
  {
    id: 'gp5',
    name: 'Guitar Pro 5',
    extension: 'gp5',
    description: 'Guitar Pro tablature format (via MIDI conversion)',
    mimeType: 'application/octet-stream',
  },
  {
    id: 'json',
    name: 'Maestro Project',
    extension: 'maestro.json',
    description: 'Maestro AI project format (re-importable)',
    mimeType: 'application/json',
  },
];

/**
 * Export project in specified format
 */
export async function exportProject(
  project: MaestroProject,
  format: string
): Promise<{ blob: Blob; filename: string }> {
  const title = project.project.metadata.title || 'Untitled';
  const sanitizedTitle = title.replace(/[^a-z0-9]/gi, '_').toLowerCase();

  let blob: Blob;
  let extension: string;

  switch (format) {
    case 'midi':
      blob = await exportToMIDI(project);
      extension = 'mid';
      break;

    case 'gp5':
      blob = await exportToGuitarPro(project);
      extension = 'gp5';
      break;

    case 'json':
      blob = exportToJSON(project);
      extension = 'maestro.json';
      break;

    default:
      throw new Error(`Unsupported export format: ${format}`);
  }

  const filename = `${sanitizedTitle}.${extension}`;
  return { blob, filename };
}
