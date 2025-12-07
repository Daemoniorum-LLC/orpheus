/**
 * Guitar Pro 7 (GP7) XML Parser
 * Parses .gp and .gpx files (ZIP archives containing XML)
 */

import JSZip from 'jszip';
import { XMLParser } from 'fast-xml-parser';
import type {
  GuitarProData,
  ScoreInfo,
  Track,
  MasterBar,
  Bar,
  Voice,
  Beat,
  Note,
  Rhythm,
} from './types';

export class GP7Parser {
  private xmlParser: XMLParser;

  constructor() {
    this.xmlParser = new XMLParser({
      ignoreAttributes: false,
      attributeNamePrefix: '@_',
      textNodeName: '#text',
      parseAttributeValue: true,
      parseTagValue: true,
    });
  }

  /**
   * Parses a GP7/GP6 file (ZIP containing XML)
   */
  async parse(fileBuffer: ArrayBuffer): Promise<GuitarProData> {
    // Load ZIP archive
    const zip = await JSZip.loadAsync(fileBuffer);

    // Extract score.gpif (main XML file)
    const gpifFile = zip.file('Content/score.gpif');
    if (!gpifFile) {
      throw new Error('Invalid GP7/GP6 file: score.gpif not found');
    }

    const gpifXml = await gpifFile.async('string');

    // Parse XML
    const gpif = this.xmlParser.parse(gpifXml);

    // Convert to GuitarProData
    return this.convertGPIF(gpif.GPIF);
  }

  /**
   * Converts GPIF XML structure to GuitarProData
   */
  private convertGPIF(gpif: any): GuitarProData {
    const version = gpif.GPVersion?.startsWith('7') ? 'GP7' : 'GP6';

    // Parse score metadata
    const score = this.parseScore(gpif.Score);

    // Parse tracks
    const tracks = this.parseTracks(gpif.Tracks?.Track || []);

    // Parse master bars (measures)
    const masterBars = this.parseMasterBars(gpif.MasterBars?.MasterBar || []);

    // Parse bars (track-specific measures)
    const bars = this.parseBars(gpif.Bars?.Bar || [], tracks.length);

    // Parse voices
    const voices = this.parseVoices(gpif.Voices?.Voice || []);

    // Parse beats
    const beats = this.parseBeats(gpif.Beats?.Beat || []);

    // Parse notes
    const notes = this.parseNotes(gpif.Notes?.Note || []);

    // Parse rhythms
    const rhythms = this.parseRhythms(gpif.Rhythms?.Rhythm || []);

    return {
      version,
      score,
      tracks,
      masterBars,
      bars,
      voices,
      beats,
      notes,
      rhythms,
    };
  }

  /**
   * Parses score metadata
   */
  private parseScore(scoreXml: any): ScoreInfo {
    return {
      title: scoreXml?.Title || 'Untitled',
      subtitle: scoreXml?.SubTitle,
      artist: scoreXml?.Artist || 'Unknown Artist',
      album: scoreXml?.Album,
      words: scoreXml?.Words,
      music: scoreXml?.Music,
      copyright: scoreXml?.Copyright,
      tabber: scoreXml?.Tabber,
      instructions: scoreXml?.Instructions,
      notices: scoreXml?.Notices ? [scoreXml.Notices] : undefined,
    };
  }

  /**
   * Parses tracks
   */
  private parseTracks(tracksXml: any[]): Track[] {
    const trackArray = Array.isArray(tracksXml) ? tracksXml : [tracksXml];

    return trackArray.map((trackXml, index) => {
      const color = trackXml.Color || { R: 255, G: 0, B: 0 };

      // Parse tuning
      const strings = trackXml.Instrument?.Strings?.String || [];
      const stringArray = Array.isArray(strings) ? strings : [strings];
      const tuning = stringArray.map((s: any) => parseInt(s.Tuning || s));

      return {
        id: parseInt(trackXml['@_id']) || index,
        name: trackXml.Name || `Track ${index + 1}`,
        color: {
          r: parseInt(color.R || 255),
          g: parseInt(color.G || 0),
          b: parseInt(color.B || 0),
        },
        instrument: {
          type: this.parseInstrumentType(trackXml.Instrument?.Type),
          strings: tuning.map((t, i) => ({ number: i + 1, tuning: t })),
        },
        channel: parseInt(trackXml.Channel || 0),
        volume: parseInt(trackXml.Volume || 15),
        balance: parseInt(trackXml.Balance || 8),
        chorus: parseInt(trackXml.Chorus || 0),
        reverb: parseInt(trackXml.Reverb || 0),
        phaser: parseInt(trackXml.Phaser || 0),
        tremolo: parseInt(trackXml.Tremolo || 0),
        tuning,
        capo: parseInt(trackXml.Capo || 0),
      };
    });
  }

  /**
   * Parses instrument type
   */
  private parseInstrumentType(type: string): 'guitar' | 'bass' | 'drums' | 'piano' | 'vocals' | 'other' {
    const typeStr = (type || '').toLowerCase();
    if (typeStr.includes('guitar')) return 'guitar';
    if (typeStr.includes('bass')) return 'bass';
    if (typeStr.includes('drum')) return 'drums';
    if (typeStr.includes('piano') || typeStr.includes('keyboard')) return 'piano';
    if (typeStr.includes('vocal')) return 'vocals';
    return 'other';
  }

  /**
   * Parses master bars (measures)
   */
  private parseMasterBars(masterBarsXml: any[]): MasterBar[] {
    const barArray = Array.isArray(masterBarsXml) ? masterBarsXml : [masterBarsXml];

    return barArray.map((barXml, index) => {
      const masterBar: MasterBar = {
        index,
      };

      // Key signature
      if (barXml.Key) {
        masterBar.key = {
          accidentalCount: parseInt(barXml.Key.AccidentalCount || 0),
          mode: barXml.Key.Mode || 'Major',
        };
      }

      // Time signature
      if (barXml.Time) {
        masterBar.time = {
          numerator: parseInt(barXml.Time.Numerator || 4),
          denominator: parseInt(barXml.Time.Denominator || 4),
        };
      }

      // Tempo
      if (barXml.Tempo) {
        masterBar.tempo = parseInt(barXml.Tempo.Value || barXml.Tempo);
      }

      // Section marker
      if (barXml.Section) {
        masterBar.section = {
          text: barXml.Section.Text || barXml.Section,
        };
      }

      // Repeat markers
      if (barXml.Repeat) {
        masterBar.repeat = {
          start: barXml.Repeat.Start === 'true' || barXml.Repeat.Start === true,
          end: barXml.Repeat.End === 'true' || barXml.Repeat.End === true,
          count: parseInt(barXml.Repeat.Count || 2),
        };
      }

      // Alternate endings
      if (barXml.AlternateEndings) {
        const endings = barXml.AlternateEndings.toString().split(',').map((e: string) => parseInt(e.trim()));
        masterBar.alternateEndings = endings;
      }

      // Triplet feel
      if (barXml.TripletFeel) {
        masterBar.tripletFeel = barXml.TripletFeel;
      }

      return masterBar;
    });
  }

  /**
   * Parses bars (track-specific measures)
   */
  private parseBars(barsXml: any[], trackCount: number): Bar[][] {
    const barArray = Array.isArray(barsXml) ? barsXml : [barsXml];
    const bars: Bar[][] = Array.from({ length: trackCount }, () => []);

    barArray.forEach((barXml) => {
      const bar: Bar = {
        id: parseInt(barXml['@_id']) || 0,
        trackId: 0,  // Will be determined by position
        masterBarIndex: 0,  // Will be determined by position
        clef: barXml.Clef || 'G2',
        voiceIds: this.parseIdList(barXml.Voices),
      };

      // Distribute bars to tracks (simplified - actual logic would track position)
      const trackId = Math.floor(bar.id / (barArray.length / trackCount));
      if (trackId < trackCount) {
        bar.trackId = trackId;
        bar.masterBarIndex = bars[trackId].length;
        bars[trackId].push(bar);
      }
    });

    return bars;
  }

  /**
   * Parses voices
   */
  private parseVoices(voicesXml: any[]): Voice[][] {
    const voiceArray = Array.isArray(voicesXml) ? voicesXml : [voicesXml];

    // Group voices by bar (simplified)
    const voices: Voice[][] = [];

    voiceArray.forEach((voiceXml) => {
      const voice: Voice = {
        id: parseInt(voiceXml['@_id']) || 0,
        barId: 0,  // Will be determined by context
        beatIds: this.parseIdList(voiceXml.Beats),
      };

      if (!voices[voice.barId]) {
        voices[voice.barId] = [];
      }
      voices[voice.barId].push(voice);
    });

    return voices;
  }

  /**
   * Parses beats
   */
  private parseBeats(beatsXml: any[]): Beat[] {
    const beatArray = Array.isArray(beatsXml) ? beatsXml : [beatsXml];

    return beatArray.map((beatXml) => {
      const beat: Beat = {
        id: parseInt(beatXml['@_id']) || 0,
        voiceId: 0,  // Will be determined by context
        rhythmId: parseInt(beatXml.Rhythm?.['@_ref']) || 0,
        noteIds: this.parseIdList(beatXml.Notes),
        dynamic: beatXml.Dynamic,
        accentuated: beatXml.Accentuated === 'true' || beatXml.Accentuated === true,
        graceNotes: beatXml.GraceNotes === 'true' || beatXml.GraceNotes === true,
        vibrato: beatXml.Vibrato,
        palmMute: beatXml.PalmMute === 'true' || beatXml.PalmMute === true,
        letRing: beatXml.LetRing === 'true' || beatXml.LetRing === true,
        slapped: beatXml.Slapped === 'true' || beatXml.Slapped === true,
        popped: beatXml.Popped === 'true' || beatXml.Popped === true,
        tapped: beatXml.Tapped === 'true' || beatXml.Tapped === true,
        text: beatXml.FreeText,
      };

      // Parse chord diagram
      if (beatXml.Chord) {
        beat.chord = {
          name: beatXml.Chord.Name,
          frets: this.parseChordFrets(beatXml.Chord.Frets),
        };
      }

      return beat;
    });
  }

  /**
   * Parses notes
   */
  private parseNotes(notesXml: any[]): Note[] {
    const noteArray = Array.isArray(notesXml) ? notesXml : [notesXml];

    return noteArray.map((noteXml) => {
      const note: Note = {
        id: parseInt(noteXml['@_id']) || 0,
        beatId: 0,  // Will be determined by context
        string: parseInt(noteXml.String || 0),
        fret: parseInt(noteXml.Fret || 0),
        velocity: parseInt(noteXml.Velocity || 95),
        hammerOn: !!noteXml.HammerOn,
        pullOff: !!noteXml.PullOff,
        vibrato: noteXml.Vibrato,
        deadNote: !!noteXml.DeadNote,
        ghostNote: !!noteXml.GhostNote,
        accentuated: noteXml.Accentuated === 'true' || noteXml.Accentuated === true,
        staccato: !!noteXml.Staccato,
      };

      // Parse tie
      if (noteXml.Tie) {
        note.tie = {
          destination: parseInt(noteXml.Tie['@_destination']) || 0,
        };
      }

      // Parse slide
      if (noteXml.Slide) {
        note.slide = {
          type: noteXml.Slide['@_type'] || 'ShiftSlideTo',
          destination: parseInt(noteXml.Slide['@_destination']),
        };
      }

      // Parse bend
      if (noteXml.Bend?.Point) {
        const points = Array.isArray(noteXml.Bend.Point) ? noteXml.Bend.Point : [noteXml.Bend.Point];
        note.bend = {
          points: points.map((p: any) => ({
            position: parseInt(p.Position || 0),
            value: parseInt(p.Value || 0),
          })),
        };
      }

      // Parse harmonic
      if (noteXml.Harmonic) {
        note.harmonic = noteXml.Harmonic['@_type'] || 'Natural';
      }

      return note;
    });
  }

  /**
   * Parses rhythms
   */
  private parseRhythms(rhythmsXml: any[]): Rhythm[] {
    const rhythmArray = Array.isArray(rhythmsXml) ? rhythmsXml : [rhythmsXml];

    return rhythmArray.map((rhythmXml) => {
      const rhythm: Rhythm = {
        id: parseInt(rhythmXml['@_id']) || 0,
        noteValue: rhythmXml.NoteValue || 'Quarter',
        augmentationDots: parseInt(rhythmXml.AugmentationDot?.['@_count'] || 0),
      };

      // Parse tuplet
      if (rhythmXml.Tuplet) {
        rhythm.tuplet = {
          numerator: parseInt(rhythmXml.Tuplet['@_numerator'] || 3),
          denominator: parseInt(rhythmXml.Tuplet['@_denominator'] || 2),
        };
      }

      return rhythm;
    });
  }

  /**
   * Parses a space-separated list of IDs
   */
  private parseIdList(value: any): number[] {
    if (!value) return [];
    const str = value.toString();
    return str.split(' ').map((id: string) => parseInt(id.trim())).filter((id: number) => !isNaN(id));
  }

  /**
   * Parses chord frets (e.g., "x 0 2 2 1 0")
   */
  private parseChordFrets(fretsStr: string): (number | 'x')[] {
    if (!fretsStr) return [];
    return fretsStr.split(' ').map(f => f.trim() === 'x' ? 'x' : parseInt(f));
  }
}

/**
 * Convenience function to parse a GP7/GP6 file
 */
export async function parseGP7(fileBuffer: ArrayBuffer): Promise<GuitarProData> {
  const parser = new GP7Parser();
  return parser.parse(fileBuffer);
}
