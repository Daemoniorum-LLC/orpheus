/**
 * MIDI file parsing and generation
 */

export interface MIDIFile {
  header: MIDIHeader;
  tracks: MIDITrack[];
}

export interface MIDIHeader {
  format: 0 | 1 | 2;  // 0: single track, 1: multiple tracks, 2: multiple sequences
  tracks: number;
  division: number;    // Ticks per quarter note
}

export interface MIDITrack {
  events: MIDIEvent[];
}

export type MIDIEvent =
  | MIDINoteOnEvent
  | MIDINoteOffEvent
  | MIDIControlChangeEvent
  | MIDIProgramChangeEvent
  | MIDIPitchBendEvent
  | MIDIMetaEvent;

export interface MIDINoteOnEvent {
  type: 'noteOn';
  deltaTime: number;  // Ticks since last event
  channel: number;    // 0-15
  note: number;       // 0-127
  velocity: number;   // 0-127
}

export interface MIDINoteOffEvent {
  type: 'noteOff';
  deltaTime: number;
  channel: number;
  note: number;
  velocity: number;
}

export interface MIDIControlChangeEvent {
  type: 'controlChange';
  deltaTime: number;
  channel: number;
  controller: number; // 0-127
  value: number;      // 0-127
}

export interface MIDIProgramChangeEvent {
  type: 'programChange';
  deltaTime: number;
  channel: number;
  program: number;    // 0-127 (instrument)
}

export interface MIDIPitchBendEvent {
  type: 'pitchBend';
  deltaTime: number;
  channel: number;
  value: number;      // -8192 to 8191 (0 = no bend)
}

export interface MIDIMetaEvent {
  type: 'meta';
  metaType: 'tempo' | 'timeSignature' | 'keySignature' | 'text' | 'copyright' | 'trackName' | 'endOfTrack';
  deltaTime: number;
  data: any;
}

/**
 * Parses a MIDI file from binary data
 */
export function parseMIDIFile(data: ArrayBuffer): MIDIFile {
  const view = new DataView(data);
  let offset = 0;

  // Read header
  const headerChunk = readChunk(view, offset);
  if (headerChunk.type !== 'MThd') {
    throw new Error('Invalid MIDI file: Missing header chunk');
  }

  const format = view.getUint16(offset + 8, false);
  const tracks = view.getUint16(offset + 10, false);
  const division = view.getUint16(offset + 12, false);

  offset += 8 + headerChunk.length;

  const header: MIDIHeader = { format: format as 0 | 1 | 2, tracks, division };
  const trackList: MIDITrack[] = [];

  // Read tracks
  for (let i = 0; i < tracks; i++) {
    const trackChunk = readChunk(view, offset);
    if (trackChunk.type !== 'MTrk') {
      throw new Error(`Invalid MIDI file: Expected MTrk chunk at track ${i}`);
    }

    const events = readTrackEvents(view, offset + 8, trackChunk.length);
    trackList.push({ events });

    offset += 8 + trackChunk.length;
  }

  return { header, tracks: trackList };
}

function readChunk(view: DataView, offset: number): { type: string; length: number } {
  const type = String.fromCharCode(
    view.getUint8(offset),
    view.getUint8(offset + 1),
    view.getUint8(offset + 2),
    view.getUint8(offset + 3)
  );
  const length = view.getUint32(offset + 4, false);
  return { type, length };
}

function readTrackEvents(view: DataView, offset: number, length: number): MIDIEvent[] {
  const events: MIDIEvent[] = [];
  const endOffset = offset + length;
  let runningStatus = 0;

  while (offset < endOffset) {
    const deltaTime = readVariableLength(view, offset);
    offset += getVariableLengthSize(view, offset);

    let status = view.getUint8(offset);

    // Running status
    if ((status & 0x80) === 0) {
      status = runningStatus;
    } else {
      offset++;
      runningStatus = status;
    }

    const eventType = status >> 4;
    const channel = status & 0x0F;

    switch (eventType) {
      case 0x8: // Note Off
        events.push({
          type: 'noteOff',
          deltaTime,
          channel,
          note: view.getUint8(offset),
          velocity: view.getUint8(offset + 1),
        });
        offset += 2;
        break;

      case 0x9: // Note On
        const velocity = view.getUint8(offset + 1);
        events.push({
          type: velocity > 0 ? 'noteOn' : 'noteOff',
          deltaTime,
          channel,
          note: view.getUint8(offset),
          velocity,
        });
        offset += 2;
        break;

      case 0xB: // Control Change
        events.push({
          type: 'controlChange',
          deltaTime,
          channel,
          controller: view.getUint8(offset),
          value: view.getUint8(offset + 1),
        });
        offset += 2;
        break;

      case 0xC: // Program Change
        events.push({
          type: 'programChange',
          deltaTime,
          channel,
          program: view.getUint8(offset),
        });
        offset += 1;
        break;

      case 0xE: // Pitch Bend
        const lsb = view.getUint8(offset);
        const msb = view.getUint8(offset + 1);
        const value = ((msb << 7) | lsb) - 8192;
        events.push({
          type: 'pitchBend',
          deltaTime,
          channel,
          value,
        });
        offset += 2;
        break;

      case 0xF: // Meta event
        const metaType = view.getUint8(offset);
        offset++;
        const metaLength = readVariableLength(view, offset);
        offset += getVariableLengthSize(view, offset);

        // Parse meta events (simplified)
        events.push({
          type: 'meta',
          metaType: 'text', // Simplified
          deltaTime,
          data: null,
        });

        offset += metaLength;
        break;

      default:
        // Unknown event, skip
        console.warn(`Unknown MIDI event type: 0x${eventType.toString(16)}`);
        break;
    }
  }

  return events;
}

function readVariableLength(view: DataView, offset: number): number {
  let value = 0;
  let byte: number;

  do {
    byte = view.getUint8(offset++);
    value = (value << 7) | (byte & 0x7F);
  } while (byte & 0x80);

  return value;
}

function getVariableLengthSize(view: DataView, offset: number): number {
  let size = 0;
  let byte: number;

  do {
    byte = view.getUint8(offset + size);
    size++;
  } while (byte & 0x80);

  return size;
}

/**
 * Generates a MIDI file from tracks
 */
export function generateMIDIFile(tracks: MIDITrack[], division: number = 480): ArrayBuffer {
  // Simplified MIDI file generation
  // In a full implementation, this would encode all events properly
  const header = new Uint8Array([
    0x4D, 0x54, 0x68, 0x64, // "MThd"
    0x00, 0x00, 0x00, 0x06, // Header length
    0x00, 0x01,             // Format 1
    (tracks.length >> 8) & 0xFF, tracks.length & 0xFF,
    (division >> 8) & 0xFF, division & 0xFF,
  ]);

  // For now, return just the header
  // Full implementation would encode all tracks
  return header.buffer;
}
