/**
 * Guitar Pro data structure types
 * Based on GP7/GP6/GP5 file formats
 */

export type GuitarProVersion = 'GP3' | 'GP4' | 'GP5' | 'GP6' | 'GP7';

/**
 * Root Guitar Pro data structure
 */
export interface GuitarProData {
  version: GuitarProVersion;
  score: ScoreInfo;
  tracks: Track[];
  masterBars: MasterBar[];
  bars: Bar[][];  // bars[trackIndex][barIndex]
  voices: Voice[][];
  beats: Beat[];
  notes: Note[];
  rhythms: Rhythm[];
  lyrics?: Lyrics;
}

/**
 * Score metadata and information
 */
export interface ScoreInfo {
  title: string;
  subtitle?: string;
  artist: string;
  album?: string;
  words?: string;  // Lyrics author
  music?: string;  // Music author
  copyright?: string;
  tabber?: string;
  instructions?: string;
  notices?: string[];
}

/**
 * Track definition (instrument)
 */
export interface Track {
  id: number;
  name: string;
  color: { r: number; g: number; b: number };
  instrument: Instrument;
  channel: number;
  volume: number;
  balance: number;
  chorus: number;
  reverb: number;
  phaser: number;
  tremolo: number;
  tuning?: number[];  // MIDI note numbers
  capo?: number;
}

/**
 * Instrument configuration
 */
export interface Instrument {
  type: 'guitar' | 'bass' | 'drums' | 'piano' | 'vocals' | 'other';
  strings: InstrumentString[];
}

export interface InstrumentString {
  number: number;  // String number (1-based)
  tuning: number;  // MIDI note number
}

/**
 * Master bar (measure shared across all tracks)
 */
export interface MasterBar {
  index: number;
  key?: KeySignature;
  time?: TimeSignature;
  tempo?: number;
  section?: SectionMarker;
  repeat?: RepeatMarker;
  alternateEndings?: number[];
  tripletFeel?: 'none' | 'eighth' | 'sixteenth';
}

export interface KeySignature {
  accidentalCount: number;  // -7 to +7 (flats negative, sharps positive)
  mode: 'Major' | 'Minor';
}

export interface TimeSignature {
  numerator: number;
  denominator: number;
}

export interface SectionMarker {
  text: string;
  color?: { r: number; g: number; b: number };
}

export interface RepeatMarker {
  start?: boolean;
  end?: boolean;
  count?: number;
}

/**
 * Bar (measure for specific track)
 */
export interface Bar {
  id: number;
  trackId: number;
  masterBarIndex: number;
  clef: 'G2' | 'F4' | 'C3' | 'C4' | 'percussion';
  voiceIds: number[];
}

/**
 * Voice (polyphony layer within a bar)
 */
export interface Voice {
  id: number;
  barId: number;
  beatIds: number[];
}

/**
 * Beat (rhythmic event with notes)
 */
export interface Beat {
  id: number;
  voiceId: number;
  rhythmId: number;
  noteIds: number[];

  // Beat properties
  dynamic?: Dynamic;
  accentuated?: boolean;
  graceNotes?: boolean;
  tremolo?: TremoloType;
  vibrato?: VibratoType;
  palmMute?: boolean;
  letRing?: boolean;
  slapped?: boolean;
  popped?: boolean;
  tapped?: boolean;

  // Chord diagram
  chord?: ChordDiagram;

  // Text annotation
  text?: string;
}

export type Dynamic = 'ppp' | 'pp' | 'p' | 'mp' | 'mf' | 'f' | 'ff' | 'fff';
export type TremoloType = '1/8' | '1/16' | '1/32';
export type VibratoType = 'none' | 'slight' | 'wide';

export interface ChordDiagram {
  name: string;
  frets: (number | 'x')[];  // Fret numbers or 'x' for muted
  fingers?: number[];
  baseFret?: number;
}

/**
 * Note definition
 */
export interface Note {
  id: number;
  beatId: number;
  string: number;  // String number (0-based)
  fret: number;
  velocity: number;  // MIDI velocity (0-127)

  // Note techniques
  tie?: { destination: number };
  hammerOn?: boolean;
  pullOff?: boolean;
  slide?: Slide;
  bend?: Bend;
  harmonic?: HarmonicType;
  vibrato?: VibratoType;
  leftHandFinger?: number;  // 0-4 (thumb, index, middle, ring, pinky)
  rightHandFinger?: 'p' | 'i' | 'm' | 'a' | 'c';  // Thumb, index, middle, ring, pinky
  deadNote?: boolean;
  ghostNote?: boolean;
  accentuated?: boolean;
  staccato?: boolean;
}

export interface Slide {
  type: 'ShiftSlideTo' | 'LegatoSlideTo' | 'SlideOutDown' | 'SlideOutUp' | 'SlideInAbove' | 'SlideInBelow';
  destination?: number;  // Destination fret (for ShiftSlideTo, LegatoSlideTo)
}

export interface Bend {
  points: BendPoint[];
}

export interface BendPoint {
  position: number;  // Position in beat (0-60)
  value: number;     // Bend amount in semitones * 100 (100 = 1 semitone)
}

export type HarmonicType = 'Natural' | 'Artificial' | 'Tap' | 'Pinch' | 'Semi';

/**
 * Rhythm pattern
 */
export interface Rhythm {
  id: number;
  noteValue: NoteValue;
  augmentationDots?: number;
  tuplet?: Tuplet;
}

export type NoteValue = 'Whole' | 'Half' | 'Quarter' | 'Eighth' | 'Sixteenth' | 'ThirtySecond' | 'SixtyFourth';

export interface Tuplet {
  numerator: number;
  denominator: number;
}

/**
 * Lyrics
 */
export interface Lyrics {
  trackId: number;
  lines: LyricsLine[];
}

export interface LyricsLine {
  startBar: number;
  text: string;
}

/**
 * Binary reader helper for GP5 parsing
 */
export class BinaryReader {
  private view: DataView;
  private offset: number = 0;

  constructor(buffer: ArrayBuffer) {
    this.view = new DataView(buffer);
  }

  readByte(): number {
    const value = this.view.getUint8(this.offset);
    this.offset += 1;
    return value;
  }

  readInt(): number {
    const value = this.view.getInt32(this.offset, true);  // Little-endian
    this.offset += 4;
    return value;
  }

  readShort(): number {
    const value = this.view.getInt16(this.offset, true);
    this.offset += 2;
    return value;
  }

  readString(length: number): string {
    const bytes: number[] = [];
    for (let i = 0; i < length; i++) {
      bytes.push(this.readByte());
    }
    return String.fromCharCode(...bytes);
  }

  readIntSizeString(): string {
    const length = this.readInt();
    return this.readString(length);
  }

  readByteSizeString(): string {
    const length = this.readByte();
    return this.readString(length);
  }

  skip(bytes: number): void {
    this.offset += bytes;
  }

  getOffset(): number {
    return this.offset;
  }

  setOffset(offset: number): void {
    this.offset = offset;
  }

  hasMore(): boolean {
    return this.offset < this.view.byteLength;
  }
}

/**
 * Binary writer helper for GP5 serialization
 */
export class BinaryWriter {
  private buffer: number[] = [];

  writeByte(value: number): void {
    this.buffer.push(value & 0xFF);
  }

  writeInt(value: number): void {
    this.buffer.push(value & 0xFF);
    this.buffer.push((value >> 8) & 0xFF);
    this.buffer.push((value >> 16) & 0xFF);
    this.buffer.push((value >> 24) & 0xFF);
  }

  writeShort(value: number): void {
    this.buffer.push(value & 0xFF);
    this.buffer.push((value >> 8) & 0xFF);
  }

  writeString(value: string, length: number): void {
    for (let i = 0; i < length; i++) {
      this.writeByte(i < value.length ? value.charCodeAt(i) : 0);
    }
  }

  writeIntSizeString(value: string): void {
    this.writeInt(value.length);
    this.writeString(value, value.length);
  }

  writeByteSizeString(value: string): void {
    this.writeByte(value.length);
    this.writeString(value, value.length);
  }

  getBuffer(): ArrayBuffer {
    return new Uint8Array(this.buffer).buffer;
  }
}
