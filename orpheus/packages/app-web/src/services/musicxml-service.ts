/**
 * MusicXML Service - Import and export MusicXML format
 * Industry-standard music notation interchange format
 */

import type {
  MaestroProject,
  ProjectMetadata,
  CompositionData,
  NotationTrack,
  Measure,
  Voice,
  Beat,
  Note,
  NoteTechniques,
  TimeSignature,
} from '@orpheus/shared-types';

// MusicXML Duration Constants (divisions are typically per quarter note)
const DURATIONS = {
  whole: 4,
  half: 2,
  quarter: 1,
  eighth: 0.5,
  '16th': 0.25,
  '32nd': 0.125,
  '64th': 0.0625,
};

// MusicXML Step to MIDI pitch mapping
const STEP_TO_MIDI: Record<string, number> = {
  C: 0,
  D: 2,
  E: 4,
  F: 5,
  G: 7,
  A: 9,
  B: 11,
};

// Guitar standard tuning (string 1 = high E, string 6 = low E)
const STANDARD_TUNING = [64, 59, 55, 50, 45, 40]; // E4, B3, G3, D3, A2, E2

/**
 * Parse MusicXML content and convert to MaestroProject
 */
export function parseMusicXML(xmlContent: string): MaestroProject {
  const parser = new DOMParser();
  const doc = parser.parseFromString(xmlContent, 'application/xml');

  // Check for parse errors
  const parseError = doc.querySelector('parsererror');
  if (parseError) {
    throw new Error(`MusicXML parse error: ${parseError.textContent}`);
  }

  // Determine if it's partwise or timewise
  const isPartwise = !!doc.querySelector('score-partwise');
  const isTimewise = !!doc.querySelector('score-timewise');

  if (!isPartwise && !isTimewise) {
    throw new Error('Invalid MusicXML: must be score-partwise or score-timewise');
  }

  // Extract metadata
  const metadata = extractMetadata(doc);

  // Extract composition data
  const composition = isPartwise
    ? extractPartwiseComposition(doc)
    : extractTimewiseComposition(doc);

  return {
    formatVersion: '1.0',
    project: {
      metadata,
      composition,
      session: { recordedTakes: [], currentTakeId: null },
      mixing: { channels: [], masterBus: { volume: 1, pan: 0, muted: false, solo: false, effects: [] } },
      mastering: { presets: [], loudnessTargets: { integrated: -14, truePeak: -1 } },
      practice: { exercises: [], progress: {} },
      aiHistory: { conversations: [] },
      collaboration: { members: [], comments: [], changes: [] },
    },
  };
}

/**
 * Extract metadata from MusicXML document
 */
function extractMetadata(doc: Document): ProjectMetadata {
  const work = doc.querySelector('work');
  const identification = doc.querySelector('identification');
  const defaults = doc.querySelector('defaults');

  // Work information
  const title =
    work?.querySelector('work-title')?.textContent ||
    doc.querySelector('movement-title')?.textContent ||
    'Untitled';

  // Creator information
  const creators = identification?.querySelectorAll('creator') || [];
  let artist = '';
  creators.forEach((creator) => {
    if (creator.getAttribute('type') === 'composer') {
      artist = creator.textContent || '';
    }
  });

  // Get tempo from first measure's direction
  let tempo = 120;
  const metronome = doc.querySelector('direction metronome');
  if (metronome) {
    const perMinute = metronome.querySelector('per-minute')?.textContent;
    if (perMinute) {
      tempo = parseInt(perMinute, 10) || 120;
    }
  }

  // Get time signature from first measure
  const timeElement = doc.querySelector('attributes time');
  const timeSignature: TimeSignature = {
    numerator: 4,
    denominator: 4,
  };
  if (timeElement) {
    const beats = timeElement.querySelector('beats')?.textContent;
    const beatType = timeElement.querySelector('beat-type')?.textContent;
    if (beats) timeSignature.numerator = parseInt(beats, 10);
    if (beatType) timeSignature.denominator = parseInt(beatType, 10);
  }

  // Get key signature
  const keyElement = doc.querySelector('attributes key');
  let key = 'C';
  if (keyElement) {
    const fifths = parseInt(keyElement.querySelector('fifths')?.textContent || '0', 10);
    const mode = keyElement.querySelector('mode')?.textContent || 'major';
    key = fifthsToKey(fifths, mode === 'minor');
  }

  return {
    id: `imported-${Date.now()}`,
    title,
    artist: artist || undefined,
    tempo,
    timeSignature,
    key,
    created: new Date().toISOString(),
    modified: new Date().toISOString(),
  };
}

/**
 * Convert fifths to key name
 */
function fifthsToKey(fifths: number, isMinor: boolean): string {
  const majorKeys = ['C', 'G', 'D', 'A', 'E', 'B', 'F#', 'C#'];
  const flatMajorKeys = ['C', 'F', 'Bb', 'Eb', 'Ab', 'Db', 'Gb', 'Cb'];
  const minorKeys = ['A', 'E', 'B', 'F#', 'C#', 'G#', 'D#', 'A#'];
  const flatMinorKeys = ['A', 'D', 'G', 'C', 'F', 'Bb', 'Eb', 'Ab'];

  if (isMinor) {
    if (fifths >= 0) return minorKeys[fifths] + 'm';
    return flatMinorKeys[-fifths] + 'm';
  } else {
    if (fifths >= 0) return majorKeys[fifths];
    return flatMajorKeys[-fifths];
  }
}

/**
 * Extract composition from partwise MusicXML
 */
function extractPartwiseComposition(doc: Document): CompositionData {
  const parts = doc.querySelectorAll('part');
  const partList = doc.querySelector('part-list');
  const tracks: NotationTrack[] = [];
  const measures: Measure[] = [];

  // Process each part
  parts.forEach((part, partIndex) => {
    const partId = part.getAttribute('id') || `part-${partIndex}`;
    const scorePart = partList?.querySelector(`score-part[id="${partId}"]`);
    const partName = scorePart?.querySelector('part-name')?.textContent || `Track ${partIndex + 1}`;

    const track: NotationTrack = {
      id: partId,
      name: partName,
      voiceCount: 1,
      measures: [],
    };

    // Process measures in this part
    const partMeasures = part.querySelectorAll('measure');
    let divisions = 1; // Divisions per quarter note

    partMeasures.forEach((measureEl, measureIndex) => {
      const measureNumber = parseInt(measureEl.getAttribute('number') || String(measureIndex + 1), 10);

      // Check for divisions change
      const divisionsEl = measureEl.querySelector('attributes divisions');
      if (divisionsEl) {
        divisions = parseInt(divisionsEl.textContent || '1', 10);
      }

      // Check for time signature change
      let timeSignature: TimeSignature | undefined;
      const timeEl = measureEl.querySelector('attributes time');
      if (timeEl) {
        const beats = timeEl.querySelector('beats')?.textContent;
        const beatType = timeEl.querySelector('beat-type')?.textContent;
        if (beats && beatType) {
          timeSignature = {
            numerator: parseInt(beats, 10),
            denominator: parseInt(beatType, 10),
          };
        }
      }

      // Process notes/rests
      const voices = processMeasureNotes(measureEl, divisions);

      const measure: Measure = {
        number: measureNumber,
        timeSignature,
        voices,
      };

      track.measures.push(measure);
    });

    tracks.push(track);
  });

  return {
    scores: [],
    tempoMap: [],
    markers: [],
    tracks,
    measures,
  };
}

/**
 * Extract composition from timewise MusicXML (less common)
 */
function extractTimewiseComposition(doc: Document): CompositionData {
  // Timewise MusicXML organizes by measure first, then part
  // For simplicity, convert to partwise logic
  const measures = doc.querySelectorAll('measure');
  const tracks: NotationTrack[] = [];

  measures.forEach((measureEl, measureIndex) => {
    const parts = measureEl.querySelectorAll('part');
    parts.forEach((part, partIndex) => {
      // Ensure track exists
      if (!tracks[partIndex]) {
        tracks[partIndex] = {
          id: `track-${partIndex}`,
          name: `Track ${partIndex + 1}`,
          voiceCount: 1,
          measures: [],
        };
      }

      const voices = processMeasureNotes(part, 1);
      const measure: Measure = {
        number: measureIndex + 1,
        voices,
      };

      tracks[partIndex].measures.push(measure);
    });
  });

  return {
    scores: [],
    tempoMap: [],
    markers: [],
    tracks,
  };
}

/**
 * Process notes in a measure element
 */
function processMeasureNotes(measureEl: Element, divisions: number): Voice[] {
  const voiceMap = new Map<number, Beat[]>();
  const notes = measureEl.querySelectorAll('note');
  let currentTime = 0;

  notes.forEach((noteEl) => {
    // Skip grace notes for now
    if (noteEl.querySelector('grace')) return;

    // Get voice (default to 1)
    const voiceNum = parseInt(noteEl.querySelector('voice')?.textContent || '1', 10);
    if (!voiceMap.has(voiceNum)) {
      voiceMap.set(voiceNum, []);
    }

    // Check if this is a chord (same time as previous note)
    const isChord = !!noteEl.querySelector('chord');

    // Get duration
    const durationEl = noteEl.querySelector('duration');
    const duration = durationEl ? parseInt(durationEl.textContent || '0', 10) / divisions : 1;

    // Check if rest
    const isRest = !!noteEl.querySelector('rest');

    if (isRest) {
      voiceMap.get(voiceNum)!.push({
        startTime: currentTime,
        duration,
        notes: [],
        rest: true,
      });
      currentTime += duration;
      return;
    }

    // Get pitch
    const pitch = noteEl.querySelector('pitch');
    const unpitched = noteEl.querySelector('unpitched');

    if (!pitch && !unpitched) {
      currentTime += duration;
      return;
    }

    // Parse pitch to MIDI note
    let midiNote = 60; // Middle C default
    if (pitch) {
      const step = pitch.querySelector('step')?.textContent || 'C';
      const alter = parseInt(pitch.querySelector('alter')?.textContent || '0', 10);
      const octave = parseInt(pitch.querySelector('octave')?.textContent || '4', 10);
      midiNote = (octave + 1) * 12 + STEP_TO_MIDI[step] + alter;
    }

    // Convert to tab (find best string/fret combination)
    const { string, fret } = midiToTab(midiNote, STANDARD_TUNING);

    // Parse techniques
    const techniques = parseTechniques(noteEl);

    // Create note
    const note: Note = {
      string,
      fret,
      techniques: Object.keys(techniques).length > 0 ? techniques : undefined,
    };

    // Get tuplet info
    const timeModification = noteEl.querySelector('time-modification');
    let tuplet: Beat['tuplet'];
    if (timeModification) {
      const actual = parseInt(timeModification.querySelector('actual-notes')?.textContent || '1', 10);
      const normal = parseInt(timeModification.querySelector('normal-notes')?.textContent || '1', 10);
      tuplet = { actual, normal };
    }

    if (isChord) {
      // Add note to previous beat
      const beats = voiceMap.get(voiceNum)!;
      if (beats.length > 0) {
        beats[beats.length - 1].notes.push(note);
      }
    } else {
      // Create new beat
      voiceMap.get(voiceNum)!.push({
        startTime: currentTime,
        duration,
        notes: [note],
        tuplet,
      });
      currentTime += duration;
    }
  });

  // Convert voice map to array
  const voices: Voice[] = [];
  voiceMap.forEach((beats, voiceIndex) => {
    voices.push({
      voiceIndex: voiceIndex - 1, // Convert 1-based to 0-based
      beats,
    });
  });

  return voices;
}

/**
 * Convert MIDI note to string/fret for guitar
 */
function midiToTab(midiNote: number, tuning: number[]): { string: number; fret: number } {
  let bestString = 1;
  let bestFret = 0;
  let bestScore = Infinity;

  tuning.forEach((openNote, stringIndex) => {
    const fret = midiNote - openNote;
    if (fret >= 0 && fret <= 24) {
      // Prefer lower positions and middle strings
      const score = fret + Math.abs(stringIndex - 2.5) * 0.5;
      if (score < bestScore) {
        bestScore = score;
        bestString = stringIndex + 1;
        bestFret = fret;
      }
    }
  });

  return { string: bestString, fret: bestFret };
}

/**
 * Parse note techniques from MusicXML
 */
function parseTechniques(noteEl: Element): NoteTechniques {
  const techniques: NoteTechniques = {};
  const notations = noteEl.querySelector('notations');
  const technical = notations?.querySelector('technical');
  const articulations = notations?.querySelector('articulations');

  if (technical) {
    // Hammer-on/Pull-off
    if (technical.querySelector('hammer-on[type="start"]')) techniques.hammer = true;
    if (technical.querySelector('pull-off[type="start"]')) techniques.pull = true;

    // Harmonics
    const harmonic = technical.querySelector('harmonic');
    if (harmonic) {
      if (harmonic.querySelector('natural')) techniques.harmonic = 'natural';
      else if (harmonic.querySelector('artificial')) techniques.harmonic = 'artificial';
    }

    // Bend
    const bend = technical.querySelector('bend');
    if (bend) {
      const alterValue = parseFloat(bend.querySelector('bend-alter')?.textContent || '1');
      techniques.bend = {
        type: 'bend',
        value: alterValue * 2, // Convert semitones to half-steps * 2
      };
    }
  }

  if (articulations) {
    // Staccato
    if (articulations.querySelector('staccato')) techniques.staccato = true;
  }

  // Slurs for slides
  const slur = notations?.querySelector('slide');
  if (slur) {
    techniques.slide = { type: 'shiftSlide' };
  }

  // Ornaments for trills, vibrato
  const ornaments = notations?.querySelector('ornaments');
  if (ornaments) {
    if (ornaments.querySelector('trill-mark')) {
      techniques.trill = { fret: 0 };
    }
    if (ornaments.querySelector('tremolo')) {
      techniques.vibrato = true;
    }
  }

  return techniques;
}

/**
 * Export MaestroProject to MusicXML format
 */
export function exportToMusicXML(project: MaestroProject): string {
  const { metadata, composition } = project.project;

  let xml = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE score-partwise PUBLIC "-//Recordare//DTD MusicXML 3.1 Partwise//EN" "http://www.musicxml.org/dtds/partwise.dtd">
<score-partwise version="3.1">
  <work>
    <work-title>${escapeXml(metadata.title)}</work-title>
  </work>
  <identification>
    ${metadata.artist ? `<creator type="composer">${escapeXml(metadata.artist)}</creator>` : ''}
    <encoding>
      <software>Orpheus Music Platform</software>
      <encoding-date>${new Date().toISOString().split('T')[0]}</encoding-date>
    </encoding>
  </identification>
  <defaults>
    <scaling>
      <millimeters>7.05556</millimeters>
      <tenths>40</tenths>
    </scaling>
  </defaults>
`;

  // Part list
  const tracks = composition.tracks || [];
  xml += '  <part-list>\n';
  tracks.forEach((track, index) => {
    xml += `    <score-part id="P${index + 1}">
      <part-name>${escapeXml(track.name)}</part-name>
    </score-part>\n`;
  });
  xml += '  </part-list>\n';

  // Parts with measures
  tracks.forEach((track, trackIndex) => {
    xml += `  <part id="P${trackIndex + 1}">\n`;

    track.measures.forEach((measure, measureIndex) => {
      xml += `    <measure number="${measure.number}">\n`;

      // Add attributes on first measure
      if (measureIndex === 0) {
        xml += `      <attributes>
        <divisions>1</divisions>
        <key>
          <fifths>${keyToFifths(metadata.key)}</fifths>
        </key>
        <time>
          <beats>${metadata.timeSignature.numerator}</beats>
          <beat-type>${metadata.timeSignature.denominator}</beat-type>
        </time>
        <clef>
          <sign>TAB</sign>
          <line>5</line>
        </clef>
        <staff-details>
          <staff-lines>6</staff-lines>
          <staff-tuning line="1"><tuning-step>E</tuning-step><tuning-octave>4</tuning-octave></staff-tuning>
          <staff-tuning line="2"><tuning-step>B</tuning-step><tuning-octave>3</tuning-octave></staff-tuning>
          <staff-tuning line="3"><tuning-step>G</tuning-step><tuning-octave>3</tuning-octave></staff-tuning>
          <staff-tuning line="4"><tuning-step>D</tuning-step><tuning-octave>3</tuning-octave></staff-tuning>
          <staff-tuning line="5"><tuning-step>A</tuning-step><tuning-octave>2</tuning-octave></staff-tuning>
          <staff-tuning line="6"><tuning-step>E</tuning-step><tuning-octave>2</tuning-octave></staff-tuning>
        </staff-details>
      </attributes>
      <direction placement="above">
        <direction-type>
          <metronome>
            <beat-unit>quarter</beat-unit>
            <per-minute>${metadata.tempo}</per-minute>
          </metronome>
        </direction-type>
      </direction>\n`;
      }

      // Process voices
      measure.voices.forEach((voice) => {
        voice.beats.forEach((beat) => {
          if (beat.rest) {
            xml += `      <note>
        <rest/>
        <duration>${Math.round(beat.duration)}</duration>
        <voice>${voice.voiceIndex + 1}</voice>
        <type>quarter</type>
      </note>\n`;
          } else {
            beat.notes.forEach((note, noteIndex) => {
              const midiNote = STANDARD_TUNING[note.string - 1] + note.fret;
              const pitch = midiToPitch(midiNote);

              xml += `      <note>\n`;
              if (noteIndex > 0) {
                xml += `        <chord/>\n`;
              }
              xml += `        <pitch>
          <step>${pitch.step}</step>
          ${pitch.alter !== 0 ? `<alter>${pitch.alter}</alter>` : ''}
          <octave>${pitch.octave}</octave>
        </pitch>
        <duration>${Math.round(beat.duration)}</duration>
        <voice>${voice.voiceIndex + 1}</voice>
        <type>${durationToType(beat.duration)}</type>`;

              // Add tuplet if present
              if (beat.tuplet) {
                xml += `
        <time-modification>
          <actual-notes>${beat.tuplet.actual}</actual-notes>
          <normal-notes>${beat.tuplet.normal}</normal-notes>
        </time-modification>`;
              }

              xml += `
        <notations>
          <technical>
            <string>${note.string}</string>
            <fret>${note.fret}</fret>`;

              // Add techniques
              if (note.techniques?.hammer) {
                xml += `\n            <hammer-on type="start">H</hammer-on>`;
              }
              if (note.techniques?.pull) {
                xml += `\n            <pull-off type="start">P</pull-off>`;
              }
              if (note.techniques?.bend) {
                xml += `\n            <bend>
              <bend-alter>${note.techniques.bend.value / 2}</bend-alter>
            </bend>`;
              }
              if (note.techniques?.harmonic) {
                xml += `\n            <harmonic>
              <${note.techniques.harmonic}/>
            </harmonic>`;
              }

              xml += `
          </technical>`;

              if (note.techniques?.staccato) {
                xml += `
          <articulations>
            <staccato/>
          </articulations>`;
              }

              xml += `
        </notations>
      </note>\n`;
            });
          }
        });
      });

      xml += '    </measure>\n';
    });

    xml += '  </part>\n';
  });

  xml += '</score-partwise>\n';

  return xml;
}

/**
 * Convert MIDI note to pitch elements
 */
function midiToPitch(midiNote: number): { step: string; alter: number; octave: number } {
  const steps = ['C', 'C', 'D', 'D', 'E', 'F', 'F', 'G', 'G', 'A', 'A', 'B'];
  const alters = [0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0];

  const noteIndex = midiNote % 12;
  const octave = Math.floor(midiNote / 12) - 1;

  return {
    step: steps[noteIndex],
    alter: alters[noteIndex],
    octave,
  };
}

/**
 * Convert duration to MusicXML type
 */
function durationToType(duration: number): string {
  if (duration >= 4) return 'whole';
  if (duration >= 2) return 'half';
  if (duration >= 1) return 'quarter';
  if (duration >= 0.5) return 'eighth';
  if (duration >= 0.25) return '16th';
  if (duration >= 0.125) return '32nd';
  return '64th';
}

/**
 * Convert key name to fifths
 */
function keyToFifths(key: string): number {
  const keyMap: Record<string, number> = {
    Cb: -7, Gb: -6, Db: -5, Ab: -4, Eb: -3, Bb: -2, F: -1,
    C: 0, G: 1, D: 2, A: 3, E: 4, B: 5, 'F#': 6, 'C#': 7,
    // Minor keys
    Abm: -7, Ebm: -6, Bbm: -5, Fm: -4, Cm: -3, Gm: -2, Dm: -1,
    Am: 0, Em: 1, Bm: 2, 'F#m': 3, 'C#m': 4, 'G#m': 5, 'D#m': 6, 'A#m': 7,
  };

  return keyMap[key] || 0;
}

/**
 * Escape XML special characters
 */
function escapeXml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

/**
 * Import MusicXML file
 */
export async function importMusicXMLFile(file: File): Promise<MaestroProject> {
  const text = await file.text();
  return parseMusicXML(text);
}

/**
 * Check if file is MusicXML
 */
export function isMusicXMLFile(fileName: string): boolean {
  const ext = fileName.toLowerCase().split('.').pop();
  return ext === 'xml' || ext === 'musicxml' || ext === 'mxl';
}
