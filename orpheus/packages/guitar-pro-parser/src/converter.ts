/**
 * Converts Guitar Pro data to Maestro format
 */

import { createProject } from '@orpheus/project-model';
import type { MaestroProject } from '@orpheus/shared-types';
import type {
  GuitarProData,
  Track as GPTrack,
  MasterBar as GPMasterBar,
  Note as GPNote,
  Dynamic,
} from './types';

/**
 * Converts Guitar Pro data to Maestro project
 */
export function convertToMaestro(gpData: GuitarProData): MaestroProject {
  // Create base project with metadata
  const project = createProject({
    title: gpData.score.title,
    artist: gpData.score.artist,
    tempo: gpData.masterBars[0]?.tempo || 120,
    key: convertKey(gpData.masterBars[0]?.key),
    timeSignature: convertTimeSignature(gpData.masterBars[0]?.time),
  });

  // Add additional metadata
  if (gpData.score.album) {
    project.project.metadata.album = gpData.score.album;
  }
  // Note: copyright and notes fields would need to be added to ProjectMetadata type

  // Initialize tracks/measures arrays if not present
  if (!project.project.composition.tracks) {
    project.project.composition.tracks = [];
  }
  if (!project.project.composition.measures) {
    project.project.composition.measures = [];
  }

  // Convert tracks
  gpData.tracks.forEach((gpTrack, index) => {
    const maestroTrack = convertTrack(gpTrack, index);
    project.project.composition.tracks!.push(maestroTrack);
  });

  // Convert measures
  gpData.masterBars.forEach((masterBar, index) => {
    const measure = convertMeasure(masterBar, index, gpData);
    project.project.composition.measures!.push(measure);
  });

  return project;
}

/**
 * Converts Guitar Pro track to Maestro track
 */
function convertTrack(gpTrack: GPTrack, _index: number): any {
  return {
    id: `track-${gpTrack.id}`,
    name: gpTrack.name,
    type: 'instrument' as const,
    instrument: {
      type: gpTrack.instrument.type,
      tuning: gpTrack.tuning ? convertTuning(gpTrack.tuning) : undefined,
      capo: gpTrack.capo || 0,
    },
    color: `#${rgbToHex(gpTrack.color.r, gpTrack.color.g, gpTrack.color.b)}`,
    volume: gpTrack.volume / 15,  // Normalize to 0-1
    pan: (gpTrack.balance - 8) / 8,  // Convert 0-16 to -1 to 1
    muted: false,
    solo: false,
    effects: {
      chorus: gpTrack.chorus,
      reverb: gpTrack.reverb,
      phaser: gpTrack.phaser,
      tremolo: gpTrack.tremolo,
    },
  };
}

/**
 * Converts MIDI tuning to note names
 */
function convertTuning(midiTuning: number[]): string[] {
  const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

  return midiTuning.map(midi => {
    const note = noteNames[midi % 12];
    const octave = Math.floor(midi / 12) - 1;
    return `${note}${octave}`;
  }).map(fullNote => fullNote.replace(/\d+$/, ''));  // Remove octave for Maestro format
}

/**
 * Converts Guitar Pro measure to Maestro measure
 */
function convertMeasure(masterBar: GPMasterBar, index: number, gpData: GuitarProData): any {
  const measure: any = {
    number: index + 1,
    timeSignature: convertTimeSignature(masterBar.time),
    tempo: masterBar.tempo,
    key: convertKey(masterBar.key),
    tracks: [],
  };

  // Add section marker
  if (masterBar.section) {
    measure.marker = {
      text: masterBar.section.text,
      color: masterBar.section.color ? `#${rgbToHex(
        masterBar.section.color.r,
        masterBar.section.color.g,
        masterBar.section.color.b
      )}` : undefined,
    };
  }

  // Add repeat markers
  if (masterBar.repeat) {
    measure.repeat = {
      start: masterBar.repeat.start,
      end: masterBar.repeat.end,
      count: masterBar.repeat.count,
    };
  }

  // Add alternate endings
  if (masterBar.alternateEndings) {
    measure.alternateEndings = masterBar.alternateEndings;
  }

  // Convert notes for each track
  gpData.tracks.forEach((track, trackIndex) => {
    const trackMeasure = {
      trackId: `track-${track.id}`,
      notes: [] as any[],
    };

    // Get bars for this track and measure
    const bars = gpData.bars[trackIndex];
    if (bars && bars[index]) {
      const bar = bars[index];

      // Get voices for this bar
      bar.voiceIds.forEach(voiceId => {
        const voice = gpData.voices.flat().find(v => v.id === voiceId);
        if (voice) {
          // Get beats for this voice
          voice.beatIds.forEach(beatId => {
            const beat = gpData.beats.find(b => b.id === beatId);
            if (beat) {
              // Get rhythm for this beat
              const rhythm = gpData.rhythms.find(r => r.id === beat.rhythmId);

              // Get notes for this beat
              beat.noteIds.forEach(noteId => {
                const gpNote = gpData.notes.find(n => n.id === noteId);
                if (gpNote) {
                  const maestroNote = convertNote(gpNote, beat, rhythm);
                  trackMeasure.notes.push(maestroNote);
                }
              });
            }
          });
        }
      });
    }

    measure.tracks.push(trackMeasure);
  });

  return measure;
}

/**
 * Converts Guitar Pro note to Maestro note
 */
function convertNote(gpNote: GPNote, beat: any, rhythm: any): any {
  const note: any = {
    string: gpNote.string + 1,  // Convert 0-based to 1-based
    fret: gpNote.fret,
    duration: convertDuration(rhythm?.noteValue),
    velocity: gpNote.velocity / 127,  // Normalize to 0-1
    dynamic: beat.dynamic ? convertDynamic(beat.dynamic) : 'mf',
    techniques: [] as any[],
  };

  // Add techniques
  if (gpNote.hammerOn) {
    note.techniques.push({ type: 'hammer-on' });
  }

  if (gpNote.pullOff) {
    note.techniques.push({ type: 'pull-off' });
  }

  if (gpNote.slide) {
    note.techniques.push({
      type: 'slide',
      direction: gpNote.slide.type.toLowerCase().includes('up') ? 'up' : 'down',
      destination: gpNote.slide.destination,
    });
  }

  if (gpNote.bend) {
    note.techniques.push({
      type: 'bend',
      points: gpNote.bend.points.map(p => ({
        position: p.position / 60,  // Normalize to 0-1
        value: p.value / 100,  // Convert to semitones
      })),
    });
  }

  if (gpNote.vibrato && gpNote.vibrato !== 'none') {
    note.techniques.push({
      type: 'vibrato',
      intensity: gpNote.vibrato,
    });
  }

  if (gpNote.harmonic) {
    note.techniques.push({
      type: 'harmonic',
      harmonicType: gpNote.harmonic.toLowerCase(),
    });
  }

  if (gpNote.deadNote) {
    note.techniques.push({ type: 'muted' });
  }

  if (gpNote.ghostNote) {
    note.techniques.push({ type: 'ghost' });
  }

  if (gpNote.accentuated || beat.accentuated) {
    note.techniques.push({ type: 'accent' });
  }

  if (gpNote.staccato) {
    note.techniques.push({ type: 'staccato' });
  }

  if (beat.palmMute) {
    note.techniques.push({ type: 'palm-mute' });
  }

  if (beat.letRing) {
    note.techniques.push({ type: 'let-ring' });
  }

  if (beat.tapped) {
    note.techniques.push({ type: 'tap' });
  }

  if (beat.slapped) {
    note.techniques.push({ type: 'slap' });
  }

  if (beat.popped) {
    note.techniques.push({ type: 'pop' });
  }

  // Add tie
  if (gpNote.tie) {
    note.tie = {
      destination: gpNote.tie.destination,
    };
  }

  // Add fingering
  if (gpNote.leftHandFinger !== undefined) {
    note.leftHandFinger = gpNote.leftHandFinger;
  }

  if (gpNote.rightHandFinger) {
    note.rightHandFinger = gpNote.rightHandFinger;
  }

  return note;
}

/**
 * Converts Guitar Pro key signature to Maestro key
 */
function convertKey(keySignature: { accidentalCount: number; mode: string } | undefined): string {
  if (!keySignature) return 'C';

  const sharpKeys = ['C', 'G', 'D', 'A', 'E', 'B', 'F#', 'C#'];
  const flatKeys = ['C', 'F', 'Bb', 'Eb', 'Ab', 'Db', 'Gb', 'Cb'];
  const minorSharpKeys = ['A', 'E', 'B', 'F#', 'C#', 'G#', 'D#', 'A#'];
  const minorFlatKeys = ['A', 'D', 'G', 'C', 'F', 'Bb', 'Eb', 'Ab'];

  const accidentals = keySignature.accidentalCount;
  const isMinor = keySignature.mode === 'Minor';

  if (accidentals >= 0) {
    // Sharps
    const keys = isMinor ? minorSharpKeys : sharpKeys;
    return keys[Math.min(accidentals, keys.length - 1)];
  } else {
    // Flats
    const keys = isMinor ? minorFlatKeys : flatKeys;
    return keys[Math.min(Math.abs(accidentals), keys.length - 1)];
  }
}

/**
 * Converts Guitar Pro time signature to Maestro format
 */
function convertTimeSignature(timeSignature: { numerator: number; denominator: number } | undefined): { numerator: number; denominator: number } {
  if (!timeSignature) return { numerator: 4, denominator: 4 };
  return { numerator: timeSignature.numerator, denominator: timeSignature.denominator };
}

/**
 * Converts Guitar Pro note value to duration
 */
function convertDuration(noteValue: string | undefined): string {
  const durationMap: Record<string, string> = {
    'Whole': '1',
    'Half': '1/2',
    'Quarter': '1/4',
    'Eighth': '1/8',
    'Sixteenth': '1/16',
    'ThirtySecond': '1/32',
    'SixtyFourth': '1/64',
  };

  return durationMap[noteValue || 'Quarter'] || '1/4';
}

/**
 * Converts Guitar Pro dynamic to Maestro dynamic
 */
function convertDynamic(dynamic: Dynamic): string {
  const dynamicMap: Record<Dynamic, string> = {
    'ppp': 'ppp',
    'pp': 'pp',
    'p': 'p',
    'mp': 'mp',
    'mf': 'mf',
    'f': 'f',
    'ff': 'ff',
    'fff': 'fff',
  };

  return dynamicMap[dynamic] || 'mf';
}

/**
 * Converts RGB to hex color
 */
function rgbToHex(r: number, g: number, b: number): string {
  const toHex = (n: number) => {
    const hex = Math.max(0, Math.min(255, n)).toString(16);
    return hex.length === 1 ? '0' + hex : hex;
  };

  return toHex(r) + toHex(g) + toHex(b);
}
