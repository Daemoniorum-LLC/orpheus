# Guitar Pro File Format Specification

## Overview

This document specifies the Guitar Pro file format for import/export in Orpheus's Cadenza mode. Guitar Pro files (.gp, .gp3, .gp4, .gp5, .gpx) are the industry standard for guitar tablature and multi-track notation.

## Supported Versions

| Version | Extension | Format | Priority | Status |
|---------|-----------|--------|----------|--------|
| Guitar Pro 3 | .gp3 | Binary | Medium | Planned |
| Guitar Pro 4 | .gp4 | Binary | Medium | Planned |
| Guitar Pro 5 | .gp5 | Binary | High | Phase 1 |
| Guitar Pro 6 | .gpx | XML (ZIP) | High | Phase 1 |
| Guitar Pro 7 | .gp | XML (ZIP) | Highest | Phase 1 |

**Focus:** GP5, GP6, and GP7 formats cover 95%+ of modern Guitar Pro files.

## File Structure

### GP7 Format (.gp)

GP7 files are ZIP archives containing XML files.

```
song.gp (ZIP archive)
├── Content/
│   ├── score.gpif          # Main score data (XML)
│   ├── BinaryStylesheet    # Style information
│   ├── PartConfiguration   # Track/part settings
│   └── LayoutConfiguration # Layout settings
└── META-INF/
    └── Container.xml       # Archive metadata
```

### GPIF (Guitar Pro Interchange Format)

The `score.gpif` file contains the complete musical content in XML format.

#### Root Structure

```xml
<?xml version="1.0" encoding="UTF-8"?>
<GPIF>
  <GPVersion>7.5</GPVersion>
  <Encoding>UTF-8</Encoding>

  <Score>
    <!-- Score metadata -->
    <Title>Song Title</Title>
    <Artist>Artist Name</Artist>
    <Album>Album Name</Album>
    <Words>Lyrics Author</Words>
    <Music>Music Author</Music>
    <Copyright>© 2025</Copyright>
    <Tabber>Tab Author</Tabber>
    <Instructions>Performance notes</Instructions>
    <Notices>Additional info</Notices>
  </Score>

  <Tracks>
    <!-- Track definitions -->
  </Tracks>

  <MasterBars>
    <!-- Master bar (measure) definitions -->
  </MasterBars>

  <Bars>
    <!-- Individual track bars -->
  </Bars>

  <Voices>
    <!-- Voice definitions (polyphony) -->
  </Voices>

  <Beats>
    <!-- Beat definitions -->
  </Beats>

  <Notes>
    <!-- Note definitions -->
  </Notes>

  <Rhythms>
    <!-- Rhythm patterns -->
  </Rhythms>
</GPIF>
```

## Data Structures

### Track Definition

```xml
<Track id="0">
  <Name>Electric Guitar</Name>
  <Color>
    <R>255</R>
    <G>0</G>
    <B>0</B>
  </Color>
  <Instrument>
    <Type>Guitar</Type>
    <Strings>
      <String>
        <Tuning>64</Tuning>  <!-- MIDI note: E2 -->
      </String>
      <String>
        <Tuning>59</Tuning>  <!-- MIDI note: A2 -->
      </String>
      <String>
        <Tuning>55</Tuning>  <!-- MIDI note: G2 -->
      </String>
      <String>
        <Tuning>50</Tuning>  <!-- MIDI note: D3 -->
      </String>
      <String>
        <Tuning>45</Tuning>  <!-- MIDI note: A3 -->
      </String>
      <String>
        <Tuning>40</Tuning>  <!-- MIDI note: E4 -->
      </String>
    </Strings>
  </Instrument>
  <Channel>0</Channel>
  <Volume>15</Volume>
  <Balance>8</Balance>
  <Chorus>0</Chorus>
  <Reverb>0</Reverb>
  <Phaser>0</Phaser>
  <Tremolo>0</Tremolo>
</Track>
```

### MasterBar (Measure)

```xml
<MasterBar>
  <Key>
    <AccidentalCount>0</AccidentalCount>  <!-- 0 = C major -->
    <Mode>Major</Mode>
  </Key>
  <Time>
    <Numerator>4</Numerator>
    <Denominator>4</Denominator>
  </Time>
  <Tempo>
    <Value>120</Value>
  </Tempo>
  <Section>
    <Text>Intro</Text>
  </Section>
  <Repeat>
    <Start>true</Start>
    <Count>2</Count>
  </Repeat>
  <AlternateEndings>1,2</AlternateEndings>
</MasterBar>
```

### Bar (Track-specific measure)

```xml
<Bar id="0">
  <Voices>0 1</Voices>  <!-- Voice IDs in this bar -->
  <Clef>G2</Clef>       <!-- G clef, 2nd line -->
</Bar>
```

### Voice (Polyphony layer)

```xml
<Voice id="0">
  <Beats>0 1 2 3</Beats>  <!-- Beat IDs in this voice -->
</Voice>
```

### Beat (Single rhythmic event)

```xml
<Beat id="0">
  <Rhythm ref="0"/>  <!-- Rhythm pattern reference -->
  <Notes>0 1 2</Notes>  <!-- Note IDs in this beat -->

  <!-- Optional beat properties -->
  <Dynamic>f</Dynamic>  <!-- Forte -->
  <Accentuated>true</Accentuated>
  <GraceNotes>true</GraceNotes>
  <Tremolo>
    <Type>1/8</Type>
  </Tremolo>
  <Vibrato>Slight</Vibrato>
  <PalmMute>true</PalmMute>
  <LetRing>true</LetRing>
  <Slapped>true</Slapped>
  <Popped>true</Popped>
  <Tapped>true</Tapped>

  <!-- Chord diagram -->
  <Chord>
    <Name>Am</Name>
    <Frets>x 0 2 2 1 0</Frets>
  </Chord>

  <!-- Text annotation -->
  <FreeText>Play with distortion</FreeText>
</Beat>
```

### Note Definition

```xml
<Note id="0">
  <String>2</String>      <!-- String number (0-based) -->
  <Fret>5</Fret>          <!-- Fret number -->
  <Velocity>95</Velocity>  <!-- MIDI velocity (0-127) -->

  <!-- Note techniques -->
  <Tie destination="1"/>  <!-- Tied to note 1 -->
  <HammerOn/>
  <PullOff/>
  <Slide type="ShiftSlideTo" destination="7"/>
  <Bend>
    <Point>
      <Position>0</Position>
      <Value>100</Value>  <!-- 1 semitone = 100 -->
    </Point>
    <Point>
      <Position>60</Position>
      <Value>100</Value>
    </Point>
  </Bend>
  <Harmonic type="Natural"/>
  <Vibrato type="Slight"/>
  <LeftHandFinger>2</LeftHandFinger>  <!-- Index = 1, Middle = 2, etc. -->
  <RightHandFinger>p</RightHandFinger> <!-- p = thumb, i = index, m = middle, a = ring -->
</Note>
```

### Rhythm Pattern

```xml
<Rhythm id="0">
  <NoteValue>Quarter</NoteValue>  <!-- Quarter note -->
  <AugmentationDot count="1"/>    <!-- Dotted -->
  <Tuplet numerator="3" denominator="2"/>  <!-- Triplet -->
</Rhythm>
```

## GP5 Format (.gp5)

GP5 is a binary format. Key differences from GP7:

### File Header

```
Offset  Size  Type    Description
------  ----  ------  -----------
0x00    31    char[]  Version string ("FICHIER GUITAR PRO v5.10")
0x1F    4     int     Score information length
...     var   -       Score information (title, artist, etc.)
```

### Binary Structure

1. **Header** - Version and metadata
2. **Score Properties** - Title, artist, album, etc.
3. **Lyrics** - Song lyrics with timing
4. **Page Setup** - Margins, orientation
5. **Tempo** - Initial tempo and tempo changes
6. **Key Signature** - Initial key
7. **MIDI Channels** - Channel assignments
8. **Tracks** - Track definitions
9. **Measures** - Measure-by-measure data
10. **Notes** - Note data with techniques

### Data Types

```cpp
// GP5 binary data types
struct GP5Header {
    char version[31];       // "FICHIER GUITAR PRO v5.10"
    int scoreInfoLength;
    // ... followed by variable-length data
};

struct GP5Track {
    byte flags;             // Track properties bitfield
    char name[40];          // Track name
    int stringCount;        // Number of strings
    int tuning[7];          // MIDI notes for each string
    int port;               // MIDI port
    int channel;            // MIDI channel (1-16)
    int effects[5];         // Effect settings
    int fret;               // Capo fret
    int color;              // RGB color
};

struct GP5Measure {
    byte header;            // Flags for repeat, time sig change, etc.
    byte numerator;         // Time signature numerator
    byte denominator;       // Time signature denominator (power of 2)
    byte repeatClose;       // Repeat closing count
    byte repeatAlternate;   // Alternate endings bitfield
};

struct GP5Beat {
    byte status;            // Beat status flags
    byte duration;          // Note duration
    int tuple;              // Tuplet definition
    byte chord;             // Chord diagram present
    byte text;              // Text annotation present
    byte effects;           // Beat effects flags
    byte mixTableChange;    // Mix table change present
};

struct GP5Note {
    byte flags;             // Note flags (tied, dead, etc.)
    byte type;              // Note type (normal, tie, dead, etc.)
    byte velocity;          // MIDI velocity
    byte fret;              // Fret number
    byte leftFinger;        // Left hand fingering
    byte rightFinger;       // Right hand fingering
    short duration;         // Note duration (for ties)
    byte effects;           // Note effects flags
    // ... followed by variable effect data
};
```

## Mapping to Maestro Format

### Track Mapping

```typescript
// Guitar Pro Track → Maestro Track
interface TrackMapping {
  guitarPro: {
    id: number;
    name: string;
    strings: number[];  // MIDI tuning
    channel: number;
  };

  maestro: {
    id: string;
    name: string;
    type: 'instrument' | 'vocal';
    instrument: {
      type: 'guitar' | 'bass' | 'drums' | 'keyboard';
      tuning: Note[];  // ['E', 'A', 'D', 'G', 'B', 'E']
    };
  };
}

// Standard tunings
const STANDARD_TUNINGS = {
  guitar: [64, 59, 55, 50, 45, 40],      // E A D G B E
  bass: [43, 38, 33, 28],                // E A D G
  bass5: [48, 43, 38, 33, 28],           // B E A D G
  dropD: [62, 59, 55, 50, 45, 40],       // D A D G B E
  openG: [62, 59, 55, 50, 47, 43],       // D G D G B D
};
```

### Measure Mapping

```typescript
interface MeasureMapping {
  guitarPro: {
    index: number;
    timeSignature: { numerator: number; denominator: number };
    tempo: number;
    keySignature: { accidentals: number; mode: 'Major' | 'Minor' };
    repeat: { start: boolean; end: boolean; count: number };
  };

  maestro: {
    number: number;
    timeSignature: TimeSignature;
    tempo: number;
    key: KeySignature;
    markers: Marker[];  // Repeat markers, sections, etc.
  };
}
```

### Note Techniques Mapping

```typescript
const TECHNIQUE_MAPPING = {
  // Guitar Pro → Maestro
  'HammerOn': 'hammer-on',
  'PullOff': 'pull-off',
  'Slide': 'slide',
  'Bend': 'bend',
  'Vibrato': 'vibrato',
  'PalmMute': 'palm-mute',
  'LetRing': 'let-ring',
  'Harmonic': 'harmonic',
  'Trill': 'trill',
  'Tremolo': 'tremolo',
  'PickStroke': 'pick-stroke',
  'Slap': 'slap',
  'Pop': 'pop',
  'Tap': 'tap',
  'DeadNote': 'muted',
  'GhostNote': 'ghost',
  'AccentuatedNote': 'accent',
  'HeavyAccentuatedNote': 'strong-accent',
  'Staccato': 'staccato',
};
```

## Import Algorithm

### Phase 1: Parse File

```typescript
async function importGuitarProFile(filePath: string): Promise<MaestroProject> {
  // 1. Detect file version
  const version = detectGuitarProVersion(filePath);

  // 2. Parse based on version
  let gpData: GuitarProData;
  if (version === 'GP7' || version === 'GP6') {
    gpData = await parseGP7(filePath);  // XML parser
  } else if (version === 'GP5') {
    gpData = await parseGP5(filePath);  // Binary parser
  }

  // 3. Convert to Maestro format
  const maestroProject = convertToMaestro(gpData);

  return maestroProject;
}
```

### Phase 2: Convert Structure

```typescript
function convertToMaestro(gpData: GuitarProData): MaestroProject {
  const project = createProject({
    title: gpData.score.title,
    tempo: gpData.masterBars[0].tempo,
    key: convertKey(gpData.masterBars[0].key),
    timeSignature: convertTimeSignature(gpData.masterBars[0].time),
  });

  // Convert tracks
  for (const gpTrack of gpData.tracks) {
    const maestroTrack = convertTrack(gpTrack);
    project.composition.tracks.push(maestroTrack);
  }

  // Convert measures and notes
  for (let i = 0; i < gpData.masterBars.length; i++) {
    const masterBar = gpData.masterBars[i];
    const measure = convertMeasure(masterBar, i);

    // Add notes for each track
    for (const track of gpData.tracks) {
      const bar = gpData.bars[track.id][i];
      const notes = convertNotes(bar, track);
      measure.tracks[track.id].notes = notes;
    }

    project.composition.measures.push(measure);
  }

  return project;
}
```

### Phase 3: Handle Techniques

```typescript
function convertNote(gpNote: GP5Note, string: number): Note {
  const note: Note = {
    string,
    fret: gpNote.fret,
    duration: convertDuration(gpNote.duration),
    velocity: gpNote.velocity,
    techniques: [],
  };

  // Map techniques
  if (gpNote.flags & NOTE_FLAG_HAMMER_ON) {
    note.techniques.push({ type: 'hammer-on' });
  }

  if (gpNote.flags & NOTE_FLAG_PULL_OFF) {
    note.techniques.push({ type: 'pull-off' });
  }

  if (gpNote.bend) {
    note.techniques.push({
      type: 'bend',
      points: gpNote.bend.points.map(p => ({
        position: p.position / 60,  // Normalize to 0-1
        value: p.value / 100,        // Semitones
      })),
    });
  }

  if (gpNote.slide) {
    note.techniques.push({
      type: 'slide',
      direction: gpNote.slide.type,
      destination: gpNote.slide.destination,
    });
  }

  return note;
}
```

## Export Algorithm

### Phase 1: Convert Maestro → Guitar Pro

```typescript
async function exportToGuitarPro(
  project: MaestroProject,
  version: 'GP7' | 'GP5'
): Promise<ArrayBuffer> {
  // Convert Maestro project to Guitar Pro structure
  const gpData: GuitarProData = {
    score: {
      title: project.project.metadata.title,
      artist: project.project.metadata.artist,
      // ... other metadata
    },
    tracks: project.composition.tracks.map(convertMaestroTrack),
    masterBars: project.composition.measures.map(convertMaestroMeasure),
    // ... other data
  };

  // Serialize based on version
  if (version === 'GP7') {
    return serializeGP7(gpData);
  } else {
    return serializeGP5(gpData);
  }
}
```

### Phase 2: Serialize GP7 (XML + ZIP)

```typescript
async function serializeGP7(gpData: GuitarProData): Promise<ArrayBuffer> {
  // 1. Generate GPIF XML
  const gpifXml = generateGPIF(gpData);

  // 2. Create ZIP archive
  const zip = new JSZip();
  zip.file('Content/score.gpif', gpifXml);
  zip.file('META-INF/Container.xml', generateContainerXml());

  // 3. Generate binary
  return await zip.generateAsync({ type: 'arraybuffer' });
}
```

### Phase 3: Serialize GP5 (Binary)

```typescript
function serializeGP5(gpData: GuitarProData): ArrayBuffer {
  const writer = new BinaryWriter();

  // Write header
  writer.writeString('FICHIER GUITAR PRO v5.10', 31);

  // Write score information
  writeScoreInfo(writer, gpData.score);

  // Write lyrics
  writeLyrics(writer, gpData.lyrics);

  // Write tempo
  writer.writeInt(gpData.masterBars[0].tempo);

  // Write key signature
  writeKeySignature(writer, gpData.masterBars[0].key);

  // Write tracks
  writer.writeByte(gpData.tracks.length);
  for (const track of gpData.tracks) {
    writeTrack(writer, track);
  }

  // Write measures
  writer.writeInt(gpData.masterBars.length);
  for (const masterBar of gpData.masterBars) {
    writeMasterBar(writer, masterBar);
    writeTrackBars(writer, masterBar, gpData.tracks);
  }

  return writer.getBuffer();
}
```

## Implementation Priority

### Phase 1: GP7 Import (High Priority)
- ✅ Parse GP7 ZIP archive
- ✅ Parse GPIF XML format
- ✅ Convert tracks and measures
- ✅ Convert notes and basic techniques
- ✅ Handle time signatures and key changes

### Phase 2: GP5 Import (Medium Priority)
- ⏳ Parse GP5 binary format
- ⏳ Handle all GP5-specific features
- ⏳ Backward compatibility with GP3/GP4

### Phase 3: Export (Medium Priority)
- ⏳ Export to GP7 format
- ⏳ Export to GP5 format
- ⏳ Preserve all techniques and annotations

### Phase 4: Advanced Features (Low Priority)
- ⏳ Chord diagrams and positions
- ⏳ Fingering suggestions
- ⏳ Percussion notation
- ⏳ Bass tablature
- ⏳ Multi-instrument scores

## Testing Strategy

### Test Files

```
tests/
├── fixtures/
│   ├── simple-guitar.gp7       # Basic guitar tab
│   ├── multi-track.gp7         # Multiple instruments
│   ├── techniques.gp7          # All techniques
│   ├── complex-timing.gp7      # Time signature changes
│   ├── simple-guitar.gp5       # GP5 version
│   └── legacy.gp4              # Backward compatibility
```

### Test Cases

1. **Import Tests**
   - Parse GP7/GP6/GP5 files without errors
   - Correctly extract all tracks
   - Preserve time signatures and tempo changes
   - Map all note techniques correctly

2. **Export Tests**
   - Round-trip: Import → Export → Import (lossless)
   - Compatible with Guitar Pro 7/5 software
   - Preserve all notation and techniques

3. **Edge Cases**
   - Empty measures
   - Rest notation
   - Triplets and tuplets
   - Multi-voice measures
   - Capo and tuning variations

## References

- [Guitar Pro File Format Documentation](https://github.com/CoderLine/alphaTab/wiki/Guitar-Pro)
- [alphaTab Guitar Pro Parser](https://github.com/CoderLine/alphaTab)
- [TuxGuitar GP Parser](https://github.com/helge17/tuxguitar)
- [GP7 GPIF Format](https://github.com/slundi/guitar-pro-format)

## Notes

- **alphaTab** is an excellent open-source reference for GP parsing (TypeScript)
- **TuxGuitar** provides comprehensive GP5 binary parsing (Java)
- GP7 format is well-documented and easier to parse than GP5
- Prioritize GP7/GP6 (XML) over GP5 (binary) for initial implementation

---

**Status**: Specification complete, ready for implementation in Phase 1.
