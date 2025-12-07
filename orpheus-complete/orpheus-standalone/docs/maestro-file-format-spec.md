# Maestro File Format Specification (.maestro)

**Version:** 1.0
**Date:** November 16, 2025
**Status:** Draft

## Overview

The `.maestro` file format is a unified project file that contains all data for composition (tablature/notation), recording (audio/MIDI), mixing, mastering, and practice modes. It enables seamless integration between all aspects of music production in a single file.

## File Structure

A `.maestro` file is a JSON-based format with the following top-level structure:

```json
{
  "formatVersion": "1.0",
  "project": {
    "metadata": {},
    "composition": {},
    "session": {},
    "mixing": {},
    "mastering": {},
    "practice": {},
    "aiHistory": {},
    "collaboration": {}
  }
}
```

## 1. Metadata Section

Project-level information shared across all modes.

```typescript
interface ProjectMetadata {
  id: string;                    // UUID for project
  title: string;                 // Project/song title
  artist?: string;               // Artist name
  album?: string;                // Album name
  genre?: string;                // Genre (rock, pop, jazz, etc.)
  tempo: number;                 // BPM (beats per minute)
  timeSignature: {
    numerator: number;           // e.g., 4 in 4/4
    denominator: number;         // e.g., 4 in 4/4
  };
  key: string;                   // Musical key (C, Dm, Bb, etc.)
  created: string;               // ISO 8601 timestamp
  modified: string;              // ISO 8601 timestamp
  duration?: number;             // Duration in seconds
  tags?: string[];               // User tags for organization
}
```

## 2. Composition Section

Tablature, notation, and score data (Cadenza AI features).

```typescript
interface CompositionData {
  scores: Score[];
  tempoMap: TempoChange[];
  markers: Marker[];
  songStructure?: SongStructure;
}

interface Score {
  id: string;
  title: string;
  instrument: InstrumentType;
  tuning?: string[];             // e.g., ["E", "A", "D", "G", "B", "E"]
  capo?: number;                 // Capo position (0 = no capo)
  tracks: NotationTrack[];

  // Guitar Pro compatibility
  guitarProVersion?: string;     // Original .gp file version if imported
  guitarProData?: any;           // Raw Guitar Pro data for round-trip
}

interface NotationTrack {
  id: string;
  name: string;
  voiceCount: number;            // 1-4 voices per track
  measures: Measure[];
}

interface Measure {
  number: number;
  timeSignature?: TimeSignature; // Override for this measure
  voices: Voice[];
}

interface Voice {
  voiceIndex: number;            // 0-3
  beats: Beat[];
}

interface Beat {
  startTime: number;             // Musical time (in quarters)
  duration: number;              // Duration in quarters
  notes: Note[];
  rest?: boolean;
  tuplet?: {
    actual: number;              // e.g., 3 for triplet
    normal: number;              // e.g., 2 for triplet
  };
}

interface Note {
  string: number;                // String number (1-6 for guitar)
  fret: number;                  // Fret number (0-24)
  velocity?: number;             // MIDI velocity (0-127)

  // Articulations and techniques
  techniques?: {
    bend?: BendTechnique;
    slide?: SlideTechnique;
    hammer?: boolean;            // Hammer-on
    pull?: boolean;              // Pull-off
    harmonic?: HarmonicType;
    palmMute?: boolean;
    letRing?: boolean;
    staccato?: boolean;
    vibrato?: boolean;
    trill?: { fret: number };
  };
}

interface BendTechnique {
  type: 'bend' | 'prebend' | 'release';
  value: number;                 // Semitones (0.5 = quarter bend, 1 = half step)
  points?: BendPoint[];
}

interface BendPoint {
  position: number;              // 0-1 (position in note duration)
  value: number;                 // Bend value at this point
}

interface SlideTechnique {
  type: 'slideInFromAbove' | 'slideInFromBelow' | 'slideOutUp' | 'slideOutDown' | 'shiftSlide' | 'legatoSlide';
  targetFret?: number;           // For shift/legato slides
}

type HarmonicType = 'natural' | 'artificial' | 'pinch' | 'tap';

type InstrumentType =
  | 'electric-guitar'
  | 'acoustic-guitar'
  | 'bass-guitar'
  | 'drums'
  | 'piano'
  | 'vocals'
  | 'synth'
  | 'strings'
  | 'brass'
  | 'woodwinds';

interface TempoChange {
  measureNumber: number;
  tempo: number;                 // New BPM
}

interface Marker {
  id: string;
  name: string;
  measureNumber: number;
  type: 'section' | 'rehearsal' | 'custom';
  color?: string;
}

interface SongStructure {
  sections: Section[];
}

interface Section {
  name: string;                  // "Intro", "Verse 1", "Chorus", etc.
  startMeasure: number;
  endMeasure: number;
  color?: string;
}
```

## 3. Session Section

Audio recording and MIDI data (Nexus DAW features).

```typescript
interface SessionData {
  sampleRate: number;            // 44100, 48000, 96000, etc.
  bitDepth: number;              // 16, 24, 32
  bufferSize: number;            // Audio buffer size (frames)

  tracks: AudioTrack[];
  buses: Bus[];
  sends: Send[];

  // Timeline
  timeline: {
    viewStart: number;           // Timeline view start (seconds)
    viewEnd: number;             // Timeline view end (seconds)
    loop?: {
      enabled: boolean;
      start: number;             // Loop start (seconds)
      end: number;               // Loop end (seconds)
    };
  };
}

interface AudioTrack {
  id: string;
  name: string;
  type: 'audio' | 'midi' | 'instrument' | 'aux';

  // Link to composition
  linkedScoreId?: string;        // Links to Score.id in composition

  // Track properties
  muted: boolean;
  solo: boolean;
  armed: boolean;                // Record armed
  monitorMode: 'off' | 'input' | 'auto';

  // Routing
  inputSource?: string;          // Audio interface input or bus
  outputBus: string;             // Usually "master" or bus ID

  // Audio/MIDI regions
  regions: Region[];

  // Mixing
  volume: number;                // dB (-∞ to +12)
  pan: number;                   // -1 (left) to +1 (right)
  plugins: Plugin[];
  automation: Automation[];

  // Color coding
  color?: string;
}

interface Region {
  id: string;
  name: string;
  startTime: number;             // Start position on timeline (seconds)
  duration: number;              // Duration (seconds)

  // For audio regions
  audioFile?: {
    path: string;                // Relative path to audio file
    sampleRate: number;
    channels: number;

    // Offset into the file
    offset: number;              // Start offset in source file (seconds)

    // Time-stretch/pitch-shift
    timeStretch?: number;        // Ratio (1.0 = normal, 0.5 = half speed)
    pitchShift?: number;         // Semitones
  };

  // For MIDI regions
  midiData?: {
    notes: MidiNote[];
    controlChanges?: MidiCC[];
  };

  // Fades
  fadeIn?: number;               // Fade in duration (seconds)
  fadeOut?: number;              // Fade out duration (seconds)

  // Gain
  gain: number;                  // dB
}

interface MidiNote {
  startTime: number;             // Relative to region start (beats)
  duration: number;              // Duration (beats)
  pitch: number;                 // MIDI note number (0-127)
  velocity: number;              // Velocity (0-127)
  channel: number;               // MIDI channel (0-15)
}

interface MidiCC {
  time: number;                  // Time (beats)
  controller: number;            // CC number (0-127)
  value: number;                 // Value (0-127)
  channel: number;               // MIDI channel (0-15)
}

interface Bus {
  id: string;
  name: string;
  type: 'master' | 'aux' | 'group';

  volume: number;
  pan: number;
  plugins: Plugin[];
  automation: Automation[];

  outputBus?: string;            // For aux/group buses
}

interface Send {
  id: string;
  sourceTrackId: string;
  destinationBusId: string;
  amount: number;                // Send level (dB)
  preFader: boolean;
}

interface Plugin {
  id: string;
  name: string;
  format: 'vst' | 'vst3' | 'au' | 'aax' | 'builtin';
  path?: string;                 // Plugin file path
  enabled: boolean;

  // Plugin state (opaque binary data)
  state?: string;                // Base64-encoded plugin state

  // For built-in plugins, structured parameters
  parameters?: Record<string, number>;
}

interface Automation {
  parameterId: string;           // e.g., "volume", "pan", "plugin.0.param.5"
  points: AutomationPoint[];
  mode: 'latch' | 'touch' | 'write' | 'read';
}

interface AutomationPoint {
  time: number;                  // Time on timeline (seconds)
  value: number;                 // Parameter value
  curve?: 'linear' | 'exponential' | 'logarithmic' | 'bezier';
}
```

## 4. Mixing Section

Mix-specific settings and AI suggestions.

```typescript
interface MixingData {
  // Reference tracks for mixing
  referenceTrack?: {
    path: string;                // Path to reference audio file
    volume: number;              // Reference track volume
  };

  // AI mixing suggestions
  aiSuggestions?: {
    timestamp: string;
    genre?: string;              // Detected genre
    suggestions: MixSuggestion[];
  };

  // Snapshots (mix versions)
  snapshots: MixSnapshot[];
}

interface MixSuggestion {
  trackId: string;
  type: 'eq' | 'compression' | 'reverb' | 'delay' | 'panning' | 'volume';
  description: string;
  parameters?: Record<string, any>;
  applied: boolean;
}

interface MixSnapshot {
  id: string;
  name: string;
  timestamp: string;

  // Snapshot of all track/bus states
  trackStates: Record<string, TrackState>;
  busStates: Record<string, BusState>;
}

interface TrackState {
  volume: number;
  pan: number;
  muted: boolean;
  solo: boolean;
  plugins: Plugin[];
}

interface BusState {
  volume: number;
  pan: number;
  plugins: Plugin[];
}
```

## 5. Mastering Section

Final mastering settings and presets.

```typescript
interface MasteringData {
  // Mastering chain
  masteringChain: Plugin[];

  // AI mastering settings
  aiMastering?: {
    enabled: boolean;
    genre?: string;
    targetLoudness: number;      // LUFS
    targetPlatform?: 'spotify' | 'apple-music' | 'youtube' | 'cd' | 'custom';
    settings: Record<string, any>;
  };

  // Loudness metering
  loudness: {
    integrated: number;          // LUFS
    shortTerm: number;           // LUFS
    momentary: number;           // LUFS
    truePeak: number;            // dBTP
    range: number;               // LU
  };

  // Export formats
  exports: ExportFormat[];
}

interface ExportFormat {
  name: string;
  format: 'wav' | 'mp3' | 'aac' | 'flac' | 'ogg';
  sampleRate: number;
  bitDepth?: number;             // For WAV/FLAC
  bitrate?: number;              // For MP3/AAC
  path: string;                  // Export file path
}
```

## 6. Practice Section

Practice mode settings and progress tracking.

```typescript
interface PracticeData {
  // Speed trainer settings
  speedTrainer: {
    enabled: boolean;
    currentSpeed: number;        // Percentage (50-150)
    targetSpeed: number;
    incrementStep: number;       // Speed increase step
  };

  // Loop settings
  loop: {
    enabled: boolean;
    startMeasure: number;
    endMeasure: number;
    repeatCount?: number;        // Null = infinite
  };

  // Progress tracking
  practiceLog: PracticeSession[];

  // Difficult sections identified by AI
  difficultSections: DifficultSection[];
}

interface PracticeSession {
  id: string;
  date: string;                  // ISO 8601
  duration: number;              // Minutes
  tempo: number;
  sectionsWorked: string[];      // Section names
  notes?: string;                // User notes

  // AI performance analysis
  analysis?: {
    timingAccuracy: number;      // 0-100%
    noteAccuracy: number;        // 0-100%
    strengths: string[];
    areasToImprove: string[];
  };
}

interface DifficultSection {
  sectionName: string;
  measures: [number, number];    // [start, end]
  difficulty: 'easy' | 'medium' | 'hard' | 'expert';
  reason: string;                // "Fast tempo changes", "Complex rhythm", etc.
  practiceCount: number;
  lastPracticed?: string;        // ISO 8601
}
```

## 7. AI History Section

Track all AI interactions and suggestions across the project.

```typescript
interface AIHistoryData {
  compositionSuggestions: AIInteraction[];
  mixingSuggestions: AIInteraction[];
  masteringSuggestions: AIInteraction[];
  transcriptionResults: AIInteraction[];
  practiceAnalysis: AIInteraction[];
  chatHistory: AIChatMessage[];
}

interface AIInteraction {
  id: string;
  timestamp: string;
  persona: string;               // e.g., "music-theory-tutor", "mixing-engineer-ai"
  type: string;                  // Interaction type
  input: any;                    // User input/context
  output: any;                   // AI response
  applied: boolean;              // Whether user applied the suggestion
  feedback?: 'positive' | 'negative' | null;
}

interface AIChatMessage {
  id: string;
  timestamp: string;
  role: 'user' | 'assistant';
  content: string;
  context?: {
    currentMode: 'compose' | 'record' | 'mix' | 'master' | 'practice';
    selectedTrackId?: string;
    selectedMeasures?: [number, number];
  };
}
```

## 8. Collaboration Section

Multi-user collaboration data.

```typescript
interface CollaborationData {
  enabled: boolean;

  // Collaborators
  collaborators: Collaborator[];

  // Permissions
  permissions: Record<string, Permission[]>;

  // Version history
  versions: Version[];

  // Comments
  comments: Comment[];
}

interface Collaborator {
  userId: string;
  name: string;
  email: string;
  role: 'owner' | 'editor' | 'viewer';
  joinedAt: string;
}

type Permission =
  | 'read'
  | 'write-composition'
  | 'write-session'
  | 'write-mixing'
  | 'export'
  | 'admin';

interface Version {
  id: string;
  timestamp: string;
  userId: string;
  description?: string;

  // Delta from previous version
  changes: Change[];
}

interface Change {
  path: string;                  // JSON path (e.g., "session.tracks.0.volume")
  operation: 'add' | 'remove' | 'replace';
  oldValue?: any;
  newValue?: any;
}

interface Comment {
  id: string;
  userId: string;
  timestamp: string;

  // Location in project
  context: {
    mode: 'compose' | 'record' | 'mix' | 'master' | 'practice';
    trackId?: string;
    measureNumber?: number;
    timePosition?: number;       // For DAW timeline (seconds)
  };

  content: string;
  resolved: boolean;
  replies: CommentReply[];
}

interface CommentReply {
  userId: string;
  timestamp: string;
  content: string;
}
```

## File Storage

### Option 1: Single JSON File (Small Projects)

For projects with minimal audio, the entire project can be stored in a single JSON file:

```
my-song.maestro  (JSON file)
```

### Option 2: Bundle Format (Large Projects)

For projects with multiple audio files, use a directory-based bundle:

```
my-song.maestro/
├── project.json           # Main project data
├── audio/
│   ├── track-1.wav        # Audio files
│   ├── track-2.wav
│   └── reference.mp3
├── cache/
│   ├── waveforms/         # Pre-generated waveform images
│   └── thumbnails/
└── metadata.json          # Bundle metadata
```

### Option 3: Cloud Format

For cloud-synced projects, data is stored remotely with local cache:

```
{
  "cloudId": "uuid",
  "syncStatus": "synced" | "syncing" | "conflict",
  "lastSync": "2025-11-16T10:00:00Z"
}
```

## Compatibility

### Guitar Pro Import

When importing Guitar Pro files (.gp, .gpx):
1. Parse Guitar Pro file format
2. Map to Maestro composition structure
3. Preserve original data in `guitarProData` for round-trip compatibility

### DAW Export (AAF/OMF)

Export session data to Pro Tools-compatible formats:
1. Convert audio tracks to AAF/OMF format
2. Include audio files
3. Map automation and plugin data (best effort)

### MIDI Export

Export composition as standard MIDI:
1. Convert notation beats to MIDI events
2. Include tempo map and markers
3. One MIDI track per notation track

## Version History

- **1.0** (2025-11-16): Initial specification

## Future Enhancements

- [ ] Video synchronization (timecode, video file references)
- [ ] Dolby Atmos spatial audio data
- [ ] Stem separation metadata
- [ ] Plugin preset library
- [ ] Marketplace content integration

---

**Next Steps:**
1. Implement TypeScript types based on this specification
2. Create serialization/deserialization utilities
3. Implement file format versioning and migration
4. Add validation schemas (JSON Schema)
