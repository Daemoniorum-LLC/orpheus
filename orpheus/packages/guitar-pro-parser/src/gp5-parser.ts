/**
 * Guitar Pro 5 (GP5) Binary Parser
 * Parses .gp5 files (binary format)
 */

import {
  BinaryReader,
  type GuitarProData,
  type ScoreInfo,
  type Track,
  type MasterBar,
  type Bar,
  type Voice,
  type Beat,
  type Note,
  type Rhythm,
  type KeySignature,
  type TimeSignature,
  type Dynamic,
  type VibratoType,
  type Slide,
  type Bend,
} from './types';

/**
 * GP5 Format Constants
 * Bit flags and magic numbers used in the binary format
 */
const GP5 = {
  // Version thresholds
  VERSION_5_0: 500,
  VERSION_5_10: 510,

  // Master bar header flags
  MASTER_BAR: {
    TIME_SIGNATURE: 0x01,
    REPEAT_START: 0x04,
    REPEAT_END: 0x08,
    ALTERNATE_ENDING: 0x10,
    SECTION_MARKER: 0x20,
    KEY_SIGNATURE: 0x40,
    DOUBLE_BAR: 0x80,
  },

  // Track header flags
  TRACK: {
    IS_PERCUSSION: 0x01,
    IS_12_STRING: 0x02,
    IS_BANJO: 0x04,
  },

  // Beat header flags
  BEAT: {
    DOTTED: 0x01,
    HAS_CHORD: 0x02,
    HAS_TEXT: 0x04,
    HAS_EFFECTS: 0x08,
    HAS_MIX_TABLE: 0x10,
    HAS_TUPLET: 0x20,
    IS_REST: 0x40,
  },

  // Beat effect flags (first byte)
  BEAT_EFFECT_1: {
    VIBRATO: 0x04,
    STACCATO: 0x20,
    TAP_SLAP_POP: 0x20,
    PICKSTROKE: 0x40,
  },

  // Beat effect flags (second byte)
  BEAT_EFFECT_2: {
    TREMOLO_BAR: 0x04,
    TREMOLO_PICKING: 0x08,
  },

  // Note header flags
  NOTE: {
    HAS_DURATION: 0x01,
    ACCENTUATED: 0x02,
    GHOST_NOTE: 0x04,
    HAS_EFFECTS: 0x08,
    HAS_DYNAMIC: 0x10,
    HAS_TYPE: 0x20,
    LEFT_HAND_FINGER: 0x80,
  },

  // Note effect flags (first byte)
  NOTE_EFFECT_1: {
    HAS_BEND: 0x01,
    HAS_GRACE: 0x02,
    LET_RING: 0x08,
    HAMMER_PULL: 0x10,
    STACCATO: 0x20,
  },

  // Note effect flags (second byte)
  NOTE_EFFECT_2: {
    HAS_SLIDE: 0x04,
    HAS_HARMONIC: 0x08,
    HAS_TRILL: 0x10,
    VIBRATO: 0x40,
  },

  // Note types
  NOTE_TYPE: {
    NORMAL: 1,
    TIE: 2,
    DEAD: 3,
  },

  // Triplet feel values
  TRIPLET_FEEL: {
    NONE: 0,
    EIGHTH: 1,
    SIXTEENTH: 2,
  },

  // Tap/slap/pop effect values
  TAP_SLAP_POP: {
    TAP: 1,
    SLAP: 2,
    POP: 3,
  },

  // Slide type values
  SLIDE: {
    SHIFT: 1,
    LEGATO: 2,
    OUT_DOWN: 4,
    OUT_UP: 8,
    IN_ABOVE: 16,
    IN_BELOW: 32,
  },

  // Harmonic type values
  HARMONIC: {
    NATURAL: 1,
    ARTIFICIAL: 2,
    TAP: 3,
    PINCH: 4,
    SEMI: 5,
  },

  // Duration values (signed byte)
  DURATION: {
    WHOLE: -2,
    HALF: -1,
    QUARTER: 0,
    EIGHTH: 1,
    SIXTEENTH: 2,
    THIRTY_SECOND: 3,
    SIXTY_FOURTH: 4,
  },

  // MIDI channels count
  MIDI_CHANNELS: 64,

  // Maximum string count read from file
  MAX_STRINGS: 7,

  // Number of lyric lines in GP5
  LYRIC_LINES: 5,

  // Maximum alternate endings
  MAX_ALTERNATE_ENDINGS: 8,

  // Number of page setup strings
  PAGE_SETUP_STRINGS: 11,

  // Number of musical directions
  MUSICAL_DIRECTIONS: 19,

  // Number of EQ bands
  EQ_BANDS: 4,
} as const;

export class GP5Parser {
  private reader!: BinaryReader;
  private version: string = '';
  private versionNumber: number = 0;

  /**
   * Parses a GP5 file
   */
  async parse(fileBuffer: ArrayBuffer): Promise<GuitarProData> {
    this.reader = new BinaryReader(fileBuffer);
    const debug = false; // Set to true for debugging

    // Read version header
    this.version = this.readVersionString();
    this.versionNumber = this.parseVersionNumber(this.version);
    if (debug) console.log(`Version: ${this.version} (${this.versionNumber}), offset: ${this.reader.getOffset()}`);

    if (this.versionNumber < GP5.VERSION_5_0) {
      throw new Error(`Unsupported Guitar Pro version: ${this.version}`);
    }

    // Read score info
    const score = this.readScoreInfo();
    if (debug) console.log(`Score: "${score.title}" by ${score.artist}, offset: ${this.reader.getOffset()}`);

    // Read lyrics
    const lyricsTrack = this.reader.readInt();
    const lyrics = this.readLyrics();
    if (debug) console.log(`Lyrics track: ${lyricsTrack}, offset: ${this.reader.getOffset()}`);

    // GP5.10+: Master RSE Equalizer
    if (this.versionNumber >= GP5.VERSION_5_10) {
      this.skipRSEEqualizer();
      if (debug) console.log(`After RSE EQ, offset: ${this.reader.getOffset()}`);
    }

    // Read page setup (skip)
    this.skipPageSetup();
    if (debug) console.log(`After page setup, offset: ${this.reader.getOffset()}`);

    // Read tempo
    const initialTempo = this.reader.readInt();
    if (debug) console.log(`Initial tempo: ${initialTempo}, offset: ${this.reader.getOffset()}`);

    // GP5: Key and octave (8ve can affect display)
    this.reader.skip(1); // Key
    if (this.versionNumber >= GP5.VERSION_5_0) {
      this.reader.skip(4); // Octave (int32)
    }

    // MIDI channels
    this.readMidiChannels();
    if (debug) console.log(`After MIDI channels, offset: ${this.reader.getOffset()}`);

    // GP5.10+: Musical directions
    if (this.versionNumber >= GP5.VERSION_5_10) {
      this.skipMusicalDirections();
      if (debug) console.log(`After musical directions, offset: ${this.reader.getOffset()}`);
    }

    // GP5.10+: Master reverb setting
    if (this.versionNumber >= GP5.VERSION_5_10) {
      this.reader.skip(4); // Master reverb
    }

    // Read measure and track count
    const measureCount = this.reader.readInt();
    const trackCount = this.reader.readInt();
    if (debug) console.log(`Measures: ${measureCount}, Tracks: ${trackCount}, offset: ${this.reader.getOffset()}`);

    // Read master bars (measures)
    const masterBars = this.readMasterBars(measureCount, initialTempo, debug);
    if (debug) console.log(`After master bars, offset: ${this.reader.getOffset()}`);

    // Read tracks
    const tracks = this.readTracks(trackCount);

    // Initialize arrays for data
    const bars: Bar[][] = Array.from({ length: trackCount }, () => []);
    const allVoices: Voice[][] = [];
    const allBeats: Beat[] = [];
    const allNotes: Note[] = [];
    const allRhythms: Rhythm[] = [];

    let voiceId = 0;
    let beatId = 0;
    let noteId = 0;
    let rhythmId = 0;

    // Read measure-track data
    for (let measureIndex = 0; measureIndex < measureCount; measureIndex++) {
      for (let trackIndex = 0; trackIndex < trackCount; trackIndex++) {
        const bar: Bar = {
          id: measureIndex * trackCount + trackIndex,
          trackId: trackIndex,
          masterBarIndex: measureIndex,
          clef: tracks[trackIndex].instrument.type === 'bass' ? 'F4' : 'G2',
          voiceIds: [],
        };

        // GP5 has 2 voices per measure
        const numVoices = 2;
        const measureVoices: Voice[] = [];

        for (let voiceIndex = 0; voiceIndex < numVoices; voiceIndex++) {
          const beatCount = this.reader.readInt();

          const voice: Voice = {
            id: voiceId++,
            barId: bar.id,
            beatIds: [],
          };

          for (let beatIndex = 0; beatIndex < beatCount; beatIndex++) {
            const { beat, notes, rhythm } = this.readBeat(beatId, noteId, rhythmId);
            beat.voiceId = voice.id;

            voice.beatIds.push(beat.id);
            allBeats.push(beat);

            notes.forEach(n => {
              n.beatId = beat.id;
              allNotes.push(n);
            });

            if (rhythm) {
              allRhythms.push(rhythm);
              rhythmId++;
            }

            beatId++;
            noteId += notes.length;
          }

          measureVoices.push(voice);
          bar.voiceIds.push(voice.id);
        }

        allVoices.push(measureVoices);
        bars[trackIndex].push(bar);

        // Skip line break (newer versions)
        if (this.versionNumber >= GP5.VERSION_5_0) {
          this.reader.skip(1);
        }
      }
    }

    return {
      version: 'GP5',
      score,
      tracks,
      masterBars,
      bars,
      voices: allVoices,
      beats: allBeats,
      notes: allNotes,
      rhythms: allRhythms,
      lyrics: lyricsTrack > 0 ? { trackId: lyricsTrack, lines: lyrics } : undefined,
    };
  }

  /**
   * Reads version string
   */
  private readVersionString(): string {
    const length = this.reader.readByte();
    const version = this.reader.readString(30);
    return version.substring(0, length);
  }

  /**
   * Parses version number from string
   */
  private parseVersionNumber(versionStr: string): number {
    const match = versionStr.match(/v(\d)\.(\d+)/i);
    if (match) {
      return parseInt(match[1]) * 100 + parseInt(match[2]);
    }
    return 500;
  }

  /**
   * Reads score info/metadata
   */
  private readScoreInfo(): ScoreInfo {
    const title = this.readGP5String();
    const subtitle = this.readGP5String();
    const artist = this.readGP5String();
    const album = this.readGP5String();
    const words = this.readGP5String();
    const music = this.readGP5String();
    const copyright = this.readGP5String();
    const tabber = this.readGP5String();
    const instructions = this.readGP5String();

    // Read notices (comments)
    const noticeCount = this.reader.readInt();
    const notices: string[] = [];
    for (let i = 0; i < Math.min(noticeCount, 100); i++) {
      notices.push(this.readGP5String());
    }

    return {
      title: title || 'Untitled',
      subtitle: subtitle || undefined,
      artist: artist || 'Unknown Artist',
      album: album || undefined,
      words: words || undefined,
      music: music || undefined,
      copyright: copyright || undefined,
      tabber: tabber || undefined,
      instructions: instructions || undefined,
      notices: notices.length > 0 ? notices : undefined,
    };
  }

  /**
   * Reads a GP5 string (int-prefixed with byte length)
   */
  private readGP5String(): string {
    const fullLength = this.reader.readInt();
    if (fullLength <= 0 || fullLength > 65535) return '';

    // The actual string length follows
    const actualLength = this.reader.readByte();

    // Read the string data (fullLength - 1 bytes total including length byte)
    const data = this.reader.readString(fullLength - 1);

    return data.substring(0, actualLength).replace(/\0+$/, '');
  }


  /**
   * Reads lyrics
   */
  private readLyrics(): { startBar: number; text: string }[] {
    const lines: { startBar: number; text: string }[] = [];

    for (let i = 0; i < GP5.LYRIC_LINES; i++) {
      const startBar = this.reader.readInt();
      const length = this.reader.readInt();
      const text = this.reader.readString(length);

      if (text.trim()) {
        lines.push({ startBar, text });
      }
    }

    return lines;
  }

  /**
   * Skips RSE Equalizer data (GP5.10+)
   */
  private skipRSEEqualizer(): void {
    // Master RSE EQ: 8 band values + master volume
    this.reader.skip(11); // 8 bands (1 byte each) + 3 bytes
  }

  /**
   * Skips musical directions (GP5.10+)
   */
  private skipMusicalDirections(): void {
    // Direction markers (coda, double coda, segno, etc.)
    // Each is an int16 indicating the measure number (-1 if not set)
    this.reader.skip(GP5.MUSICAL_DIRECTIONS * 2); // 2 bytes each
  }

  /**
   * Skips page setup data
   */
  private skipPageSetup(): void {
    // Page size and margins (each is 4 bytes int)
    this.reader.skip(4); // pageWidth
    this.reader.skip(4); // pageHeight
    this.reader.skip(4); // marginLeft
    this.reader.skip(4); // marginRight
    this.reader.skip(4); // marginTop
    this.reader.skip(4); // marginBottom
    this.reader.skip(4); // scoreSizeProportion

    // Header/footer flags
    const headerFooter = this.reader.readShort();

    // Page text fields (Title, subtitle, artist, album, words, music, words&music, copyright, pageNumber, tabber, instructions)
    for (let i = 0; i < GP5.PAGE_SETUP_STRINGS; i++) {
      this.skipPageSetupString();
    }
  }

  /**
   * Skips a page setup string
   */
  private skipPageSetupString(): void {
    const fullLength = this.reader.readInt();
    if (fullLength > 0 && fullLength < 10000) {
      this.reader.skip(fullLength);
    }
  }

  /**
   * Reads MIDI channel settings
   */
  private readMidiChannels(): void {
    // 64 MIDI channels with port/channel/effects data
    for (let i = 0; i < GP5.MIDI_CHANNELS; i++) {
      this.reader.skip(4 * 2); // program, volume
      this.reader.skip(4 * 2); // balance, chorus
      this.reader.skip(4 * 2); // reverb, phaser
      this.reader.skip(4 * 2); // tremolo, blank
    }
  }

  /**
   * Reads master bars (measure headers)
   */
  private readMasterBars(measureCount: number, initialTempo: number, debug: boolean = false): MasterBar[] {
    const masterBars: MasterBar[] = [];
    let currentTempo = initialTempo;
    let currentKey: KeySignature = { accidentalCount: 0, mode: 'Major' };
    let currentTime: TimeSignature = { numerator: 4, denominator: 4 };

    for (let i = 0; i < measureCount; i++) {
      const masterBar: MasterBar = { index: i };

      const header = this.reader.readByte();
      if (debug && i < 5) console.log(`  MasterBar ${i}: header=0x${header.toString(16)}, offset=${this.reader.getOffset()}`);

      // Time signature change
      if (header & GP5.MASTER_BAR.TIME_SIGNATURE) {
        currentTime = {
          numerator: this.reader.readByte(),
          denominator: this.reader.readByte(),
        };
      }
      masterBar.time = { ...currentTime };

      // Repeat start
      if (header & GP5.MASTER_BAR.REPEAT_START) {
        masterBar.repeat = { start: true };
      }

      // Repeat end
      if (header & GP5.MASTER_BAR.REPEAT_END) {
        const repeatCount = this.reader.readByte();
        masterBar.repeat = { ...masterBar.repeat, end: true, count: repeatCount };
      }

      // Alternate ending
      if (header & GP5.MASTER_BAR.ALTERNATE_ENDING) {
        const endings = this.reader.readByte();
        masterBar.alternateEndings = this.parseAlternateEndings(endings);
      }

      // Section marker
      if (header & GP5.MASTER_BAR.SECTION_MARKER) {
        const text = this.readGP5String();
        const r = this.reader.readByte();
        const g = this.reader.readByte();
        const b = this.reader.readByte();
        masterBar.section = { text, color: { r, g, b } };
      }

      // Key signature change
      if (header & GP5.MASTER_BAR.KEY_SIGNATURE) {
        const accidentals = this.reader.readByte(); // Signed byte
        const minor = this.reader.readByte();
        currentKey = {
          accidentalCount: accidentals > 127 ? accidentals - 256 : accidentals,
          mode: minor ? 'Minor' : 'Major',
        };
      }
      masterBar.key = { ...currentKey };

      // Double bar line (marker only)
      if (header & GP5.MASTER_BAR.DOUBLE_BAR) {
        // Skip - just a display marker
      }

      // GP5: Triplet feel and beam groups
      if (this.versionNumber >= GP5.VERSION_5_0) {
        // Triplet feel
        const tripletFeel = this.reader.readByte();
        if (tripletFeel === GP5.TRIPLET_FEEL.EIGHTH) masterBar.tripletFeel = 'eighth';
        else if (tripletFeel === GP5.TRIPLET_FEEL.SIXTEENTH) masterBar.tripletFeel = 'sixteenth';
        else masterBar.tripletFeel = 'none';

        // Skip beam groups (4 bytes)
        this.reader.skip(4);
      }

      // Tempo change (in some versions stored per measure)
      masterBar.tempo = currentTempo;

      masterBars.push(masterBar);
    }

    return masterBars;
  }

  /**
   * Parses alternate ending bitmask
   */
  private parseAlternateEndings(bitmask: number): number[] {
    const endings: number[] = [];
    for (let i = 0; i < GP5.MAX_ALTERNATE_ENDINGS; i++) {
      if (bitmask & (1 << i)) {
        endings.push(i + 1);
      }
    }
    return endings;
  }

  /**
   * Reads tracks
   */
  private readTracks(trackCount: number): Track[] {
    const tracks: Track[] = [];

    for (let i = 0; i < trackCount; i++) {
      const track = this.readTrack(i);
      tracks.push(track);
    }

    return tracks;
  }

  /**
   * Reads a single track
   */
  private readTrack(trackIndex: number): Track {
    const header = this.reader.readByte();

    const isPercussion = !!(header & GP5.TRACK.IS_PERCUSSION);
    const is12String = !!(header & GP5.TRACK.IS_12_STRING);
    const isBanjo = !!(header & GP5.TRACK.IS_BANJO);

    // Track name (40 bytes, null padded)
    const nameLength = this.reader.readByte();
    const name = this.reader.readString(40);

    // String count and tuning
    const stringCount = this.reader.readInt();
    const tuning: number[] = [];

    for (let i = 0; i < GP5.MAX_STRINGS; i++) { // Always 7 tuning values
      const midi = this.reader.readInt();
      if (i < stringCount) {
        tuning.push(midi);
      }
    }

    // MIDI port/channel/effects
    const port = this.reader.readInt();
    const channel = this.reader.readInt();
    const channelEffects = this.reader.readInt();

    // Fret count
    const frets = this.reader.readInt();

    // Capo
    const capo = this.reader.readInt();

    // Color
    const r = this.reader.readByte();
    const g = this.reader.readByte();
    const b = this.reader.readByte();
    this.reader.skip(1); // Alpha/padding

    // GP5 specific data
    if (this.versionNumber >= GP5.VERSION_5_0) {
      // Skip RSE data
      this.skipRSEData();
    }

    return {
      id: trackIndex,
      name: name.substring(0, nameLength).trim() || `Track ${trackIndex + 1}`,
      color: { r, g, b },
      instrument: {
        type: isPercussion ? 'drums' : (stringCount <= 4 ? 'bass' : 'guitar'),
        strings: tuning.map((midi, idx) => ({ number: idx + 1, tuning: midi })),
      },
      channel,
      volume: 15,
      balance: 8,
      chorus: 0,
      reverb: 0,
      phaser: 0,
      tremolo: 0,
      tuning,
      capo,
    };
  }

  /**
   * Skips RSE (Realistic Sound Engine) data
   */
  private skipRSEData(): void {
    // This is complex nested data - try to skip safely
    try {
      // RSE effects
      this.reader.skip(4 * 4); // Various effect values

      // RSE instrument info
      const rseInstrument = this.reader.readInt();
      if (rseInstrument !== -1) {
        this.reader.skip(4); // RSE instrument type
        const nameLength = this.reader.readInt();
        if (nameLength > 0 && nameLength < 256) {
          this.reader.skip(nameLength);
        }
      }

      // Equalizer
      this.reader.skip(8 + 1); // 8 band EQ + master
    } catch {
      // RSE parsing is complex and may fail - continue anyway
    }
  }

  /**
   * Reads a single beat
   */
  private readBeat(beatId: number, noteIdStart: number, rhythmId: number): {
    beat: Beat;
    notes: Note[];
    rhythm: Rhythm | null;
  } {
    const header = this.reader.readByte();

    const beat: Beat = {
      id: beatId,
      voiceId: 0,
      rhythmId: rhythmId,
      noteIds: [],
    };

    const notes: Note[] = [];
    let rhythm: Rhythm | null = null;

    // Dotted note
    const dotted = !!(header & 0x01);

    // Chord diagram
    if (header & 0x02) {
      beat.chord = this.readChordDiagram();
    }

    // Text annotation
    if (header & 0x04) {
      beat.text = this.readGP5String();
    }

    // Beat effects
    if (header & 0x08) {
      this.readBeatEffects(beat);
    }

    // Mix table change
    if (header & 0x10) {
      this.readMixTableChange();
    }

    // Duration
    const duration = this.reader.readByte(); // Signed: -2=whole, -1=half, 0=quarter, 1=8th, etc.
    rhythm = {
      id: rhythmId,
      noteValue: this.durationToNoteValue(duration > 127 ? duration - 256 : duration),
      augmentationDots: dotted ? 1 : 0,
    };

    // Tuplet
    if (header & 0x20) {
      const tupletNum = this.reader.readInt();
      rhythm.tuplet = this.parseTuplet(tupletNum);
    }

    // Note presence flags
    if (header & 0x40) {
      // Empty beat / rest
    } else {
      const stringFlags = this.reader.readByte();

      for (let stringNum = 0; stringNum < 7; stringNum++) {
        if (stringFlags & (1 << (6 - stringNum))) {
          const note = this.readNote(noteIdStart + notes.length, stringNum);
          notes.push(note);
          beat.noteIds.push(note.id);
        }
      }
    }

    return { beat, notes, rhythm };
  }

  /**
   * Reads chord diagram
   */
  private readChordDiagram(): { name: string; frets: (number | 'x')[] } {
    const newFormat = this.reader.readByte();

    if (newFormat) {
      // New chord format (GP5)
      this.reader.skip(1); // Sharp flag
      this.reader.skip(3); // Blank

      const root = this.reader.readByte();
      const type = this.reader.readByte();
      this.reader.skip(1); // Extension

      const bassNote = this.reader.readInt();
      const tonality = this.reader.readInt();
      const add = this.reader.readByte();

      // Chord name
      const nameLength = this.reader.readByte();
      const name = this.reader.readString(20);

      this.reader.skip(2); // Fifth/ninth alterations

      // Diagram
      const baseFret = this.reader.readInt();
      const frets: (number | 'x')[] = [];

      for (let i = 0; i < 7; i++) {
        const fret = this.reader.readInt();
        frets.push(fret === -1 ? 'x' : fret);
      }

      // Skip barres and fingering
      const barreCount = this.reader.readByte();
      this.reader.skip(barreCount * 5); // Barre data
      this.reader.skip(7); // Fingering
      this.reader.skip(1); // Show fingering flag

      return { name: name.substring(0, nameLength).trim(), frets };
    } else {
      // Old chord format
      const nameLength = this.reader.readByte();
      const name = this.reader.readString(20);

      const baseFret = this.reader.readInt();
      const frets: (number | 'x')[] = [];

      if (baseFret > 0) {
        for (let i = 0; i < 6; i++) {
          const fret = this.reader.readInt();
          frets.push(fret === -1 ? 'x' : fret);
        }
      }

      return { name: name.substring(0, nameLength).trim(), frets };
    }
  }

  /**
   * Reads beat effects
   */
  private readBeatEffects(beat: Beat): void {
    const flags1 = this.reader.readByte();
    const flags2 = this.reader.readByte();

    // Tap/slap/pop
    if (flags1 & 0x20) {
      const effect = this.reader.readByte();
      if (effect === 1) beat.tapped = true;
      else if (effect === 2) beat.slapped = true;
      else if (effect === 3) beat.popped = true;
    }

    // Tremolo bar
    if (flags2 & 0x04) {
      this.readTremoloBar();
    }

    // Pickstroke
    if (flags1 & 0x40) {
      const stroke = this.reader.readByte();
      // 1 = up, 2 = down
    }

    // Tremolo picking
    if (flags2 & 0x08) {
      const duration = this.reader.readByte();
      beat.tremolo = duration === 1 ? '1/8' : duration === 2 ? '1/16' : '1/32';
    }
  }

  /**
   * Reads tremolo bar effect
   */
  private readTremoloBar(): void {
    const pointCount = this.reader.readInt();
    for (let i = 0; i < pointCount; i++) {
      this.reader.skip(4 * 3); // time, value, vibrato
    }
  }

  /**
   * Reads mix table change
   */
  private readMixTableChange(): void {
    const instrument = this.reader.readByte();
    const rse = this.versionNumber >= GP5.VERSION_5_0;

    // RSE instrument info
    if (rse) {
      this.reader.skip(4); // RSE instrument number
      this.reader.skip(4); // RSE effect number
      this.reader.skip(1); // RSE effect present flag
    }

    const volume = this.reader.readByte();
    const balance = this.reader.readByte();
    const chorus = this.reader.readByte();
    const reverb = this.reader.readByte();
    const phaser = this.reader.readByte();
    const tremolo = this.reader.readByte();

    // Tempo
    const tempo = this.reader.readInt();

    // Transition durations
    if (volume !== 0xFF) this.reader.skip(1);
    if (balance !== 0xFF) this.reader.skip(1);
    if (chorus !== 0xFF) this.reader.skip(1);
    if (reverb !== 0xFF) this.reader.skip(1);
    if (phaser !== 0xFF) this.reader.skip(1);
    if (tremolo !== 0xFF) this.reader.skip(1);
    if (tempo > 0) {
      this.reader.skip(1);
      if (this.versionNumber >= GP5.VERSION_5_10) {
        this.reader.skip(1); // Hide tempo flag
      }
    }

    // All tracks flag
    if (this.versionNumber >= GP5.VERSION_5_0) {
      this.reader.skip(1);
    }

    // RSE instrument preset
    if (rse) {
      this.reader.skip(1); // Use RSE flag
      const presetLength = this.reader.readByte();
      if (presetLength > 0 && presetLength < 128) {
        this.reader.skip(presetLength);
      }
    }
  }

  /**
   * Reads a single note
   */
  private readNote(noteId: number, stringNum: number): Note {
    const header = this.reader.readByte();

    const note: Note = {
      id: noteId,
      beatId: 0,
      string: stringNum,
      fret: 0,
      velocity: 95,
    };

    // Accentuated
    if (header & 0x02) {
      note.accentuated = true;
    }

    // Ghost note
    if (header & 0x04) {
      note.ghostNote = true;
    }

    // Note effects
    if (header & 0x08) {
      this.readNoteEffects(note);
    }

    // Dynamic
    if (header & 0x10) {
      const dynamic = this.reader.readByte();
      note.velocity = this.dynamicToVelocity(dynamic);
    }

    // Note type
    if (header & 0x20) {
      const noteType = this.reader.readByte();
      if (noteType === 2) {
        note.tie = { destination: 0 }; // Tie to previous
      } else if (noteType === 3) {
        note.deadNote = true;
      }
    }

    // Fret number (if not determined by effects)
    if (!note.tie || !(header & 0x08)) {
      note.fret = this.reader.readByte();
    }

    // Left hand fingering
    if (header & 0x80) {
      note.leftHandFinger = this.reader.readByte();
      const rightFinger = this.reader.readByte();
      const fingerNames: ('p' | 'i' | 'm' | 'a' | 'c')[] = ['p', 'i', 'm', 'a', 'c'];
      if (rightFinger >= 0 && rightFinger < 5) {
        note.rightHandFinger = fingerNames[rightFinger];
      }
    }

    // GP5: Note duration percentage
    if (this.versionNumber >= GP5.VERSION_5_0 && (header & GP5.NOTE.HAS_DURATION)) {
      this.reader.skip(4); // Note duration percentage
    }

    return note;
  }

  /**
   * Reads note effects
   */
  private readNoteEffects(note: Note): void {
    const flags1 = this.reader.readByte();
    const flags2 = this.reader.readByte();

    // Bend
    if (flags1 & 0x01) {
      note.bend = this.readBend();
    }

    // Grace note
    if (flags1 & 0x02) {
      // Read grace note data
      this.reader.skip(4); // fret, velocity, transition, duration
    }

    // Let ring
    if (flags1 & 0x08) {
      // Just a flag - applies to entire beat
    }

    // Hammer-on / Pull-off
    if (flags1 & 0x10) {
      note.hammerOn = true; // Could be pull-off based on context
    }

    // Slide
    if (flags2 & 0x04) {
      const slideType = this.reader.readByte();
      note.slide = this.parseSlideType(slideType);
    }

    // Harmonic
    if (flags2 & 0x08) {
      const harmonicType = this.reader.readByte();
      note.harmonic = this.parseHarmonicType(harmonicType);
    }

    // Trill
    if (flags2 & 0x10) {
      const fret = this.reader.readByte();
      const period = this.reader.readByte();
      // Store as vibrato-like effect
    }

    // Vibrato
    if (flags2 & 0x40) {
      note.vibrato = 'wide';
    } else if (flags1 & 0x04) {
      note.vibrato = 'slight';
    }

    // Staccato
    if (flags1 & 0x20) {
      note.staccato = true;
    }
  }

  /**
   * Reads bend data
   */
  private readBend(): Bend {
    const bendType = this.reader.readByte();
    const value = this.reader.readInt();
    const pointCount = this.reader.readInt();

    const points: { position: number; value: number }[] = [];

    for (let i = 0; i < pointCount; i++) {
      const position = this.reader.readInt();
      const pointValue = this.reader.readInt();
      const vibrato = this.reader.readByte();

      points.push({
        position: position,
        value: pointValue,
      });
    }

    return { points };
  }

  /**
   * Parses slide type byte
   */
  private parseSlideType(slideType: number): Slide {
    switch (slideType) {
      case 1: return { type: 'ShiftSlideTo' };
      case 2: return { type: 'LegatoSlideTo' };
      case 4: return { type: 'SlideOutDown' };
      case 8: return { type: 'SlideOutUp' };
      case 16: return { type: 'SlideInAbove' };
      case 32: return { type: 'SlideInBelow' };
      default: return { type: 'ShiftSlideTo' };
    }
  }

  /**
   * Parses harmonic type byte
   */
  private parseHarmonicType(harmonicType: number): 'Natural' | 'Artificial' | 'Tap' | 'Pinch' | 'Semi' {
    switch (harmonicType) {
      case 1: return 'Natural';
      case 2: return 'Artificial';
      case 3: return 'Tap';
      case 4: return 'Pinch';
      case 5: return 'Semi';
      default: return 'Natural';
    }
  }

  /**
   * Converts duration byte to note value
   */
  private durationToNoteValue(duration: number): 'Whole' | 'Half' | 'Quarter' | 'Eighth' | 'Sixteenth' | 'ThirtySecond' | 'SixtyFourth' {
    switch (duration) {
      case -2: return 'Whole';
      case -1: return 'Half';
      case 0: return 'Quarter';
      case 1: return 'Eighth';
      case 2: return 'Sixteenth';
      case 3: return 'ThirtySecond';
      case 4: return 'SixtyFourth';
      default: return 'Quarter';
    }
  }

  /**
   * Parses tuplet value
   */
  private parseTuplet(tupletNum: number): { numerator: number; denominator: number } {
    // Common tuplet mappings
    switch (tupletNum) {
      case 3: return { numerator: 3, denominator: 2 }; // Triplet
      case 5: return { numerator: 5, denominator: 4 };
      case 6: return { numerator: 6, denominator: 4 }; // Sextuplet
      case 7: return { numerator: 7, denominator: 4 };
      case 9: return { numerator: 9, denominator: 8 };
      case 10: return { numerator: 10, denominator: 8 };
      case 11: return { numerator: 11, denominator: 8 };
      case 12: return { numerator: 12, denominator: 8 };
      case 13: return { numerator: 13, denominator: 8 };
      default: return { numerator: tupletNum, denominator: tupletNum - 1 };
    }
  }

  /**
   * Converts dynamic byte to MIDI velocity
   */
  private dynamicToVelocity(dynamic: number): number {
    // GP5 dynamics: 1=ppp, 2=pp, 3=p, 4=mp, 5=mf, 6=f, 7=ff, 8=fff
    const velocityMap: Record<number, number> = {
      1: 15,   // ppp
      2: 31,   // pp
      3: 47,   // p
      4: 63,   // mp
      5: 79,   // mf
      6: 95,   // f
      7: 111,  // ff
      8: 127,  // fff
    };
    return velocityMap[dynamic] || 95;
  }
}

/**
 * Convenience function to parse a GP5 file
 */
export async function parseGP5(fileBuffer: ArrayBuffer): Promise<GuitarProData> {
  const parser = new GP5Parser();
  return parser.parse(fileBuffer);
}
