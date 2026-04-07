# Tab Editor WASM Specification

**Version:** 0.1.0
**Status:** Draft
**Date:** 2026-02-12
**Authors:** Claude (Opus 4.5) + Human

---

## Abstract

This specification defines the tablature/notation editor implementation in Sigil/WASM for the Orpheus music production platform. The editor provides full Guitar Pro 8 feature parity using AlphaTab for professional notation rendering, with native Sigil data structures and WASM-compatible interaction patterns.

---

## 1. Data Model Specification

### 1.1 Design Principles

The data model follows the existing Maestro file format (`maestro-file-format-spec.md`) for compatibility, with Sigil-native type representations for WASM execution.

### 1.2 CompositionData (Top-Level)

```sigil
/// Top-level composition container
☉ Σ CompositionData {
    tracks: Vec<Track>,
    tempo_map: Vec<TempoChange>,
    markers: Vec<Marker>,
    metadata: ProjectMetadata,
}
```

### 1.3 Track

```sigil
/// Individual instrument track
☉ Σ Track {
    id: TrackId,
    name: String,
    instrument: InstrumentType,
    tuning: Vec<u8>,          // MIDI note numbers, e.g., [64, 59, 55, 50, 45, 40] for standard
    capo: u8,                 // Fret number (0 = no capo)
    measures: Vec<Measure>,
}

☉ ᛈ InstrumentType {
    ElectricGuitar,
    AcousticGuitar,
    BassGuitar,
    Drums,
    Piano,
    Vocals,
    Other(String),
}
```

### 1.4 Measure

```sigil
/// Time-bounded segment containing voices
☉ Σ Measure {
    number: u32,
    time_signature: TimeSignature,
    voices: Vec<Voice>,
    repeat_start: bool,
    repeat_end: Option<u8>,   // Repeat count
    alternate_ending: Option<u8>,
}

☉ Σ TimeSignature {
    numerator: u8,
    denominator: u8,
}
```

**Invariants:**
- `time_signature.numerator > 0`
- `time_signature.denominator ∈ {1, 2, 4, 8, 16, 32}`

### 1.5 Voice

```sigil
/// Polyphonic layer within measure (up to 4 voices per track)
☉ Σ Voice {
    voice_index: u8,          // 0-3
    beats: Vec<Beat>,
}
```

**Invariant:** `voice_index < 4`

### 1.6 Beat

```sigil
/// Rhythmic event containing simultaneous notes
☉ Σ Beat {
    start_time: f64,          // Position in quarters from measure start
    duration: Duration,
    notes: Vec<Note>,
    rest: bool,
    tuplet: Option<Tuplet>,
}

☉ Σ Tuplet {
    actual: u8,               // e.g., 3 for triplet
    normal: u8,               // e.g., 2 for triplet
}
```

### 1.7 Note

```sigil
/// Individual note on a string
☉ Σ Note {
    string: u8,               // 1-6 (1 = high E, 6 = low E)
    fret: u8,                 // 0-24
    velocity: u8,             // 0-127 MIDI velocity
    tied_to_previous: bool,
    techniques: Vec<Technique>,
}
```

**Invariants:**
- `note.string ∈ 1..=6`
- `note.fret ∈ 0..=24`
- `note.velocity ∈ 0..=127`
- Within a beat, at most one note per string

---

## 2. Duration Specification

### 2.1 Duration Enum

```sigil
☉ ᛈ Duration {
    Whole,              // 4 quarters
    Half,               // 2 quarters
    Quarter,            // 1 quarter
    Eighth,             // 0.5 quarters
    Sixteenth,          // 0.25 quarters
    ThirtySecond,       // 0.125 quarters
    SixtyFourth,        // 0.0625 quarters
    Dotted(Box<Duration>),    // 1.5x duration
    DoubleDotted(Box<Duration>), // 1.75x duration
}
```

### 2.2 Duration Conversion

| Duration | Quarters | AlphaTab TEX |
|----------|----------|--------------|
| Whole | 4.0 | `1` |
| Half | 2.0 | `2` |
| Quarter | 1.0 | `4` |
| Eighth | 0.5 | `8` |
| Sixteenth | 0.25 | `16` |
| Thirty-second | 0.125 | `32` |
| Sixty-fourth | 0.0625 | `64` |
| Dotted quarter | 1.5 | `4.` |

---

## 3. Technique Specifications

### 3.1 Technique Enum

All 18 Guitar Pro 8 compatible techniques:

```sigil
☉ ᛈ Technique {
    // Legato
    HammerOn,
    PullOff,

    // Slides
    Slide(SlideType),

    // Pitch Effects
    Bend(BendData),
    Vibrato(VibratoStyle),
    WhammyBar(Vec<BendPoint>),
    Trill { interval: i8 },

    // Muting
    PalmMute,
    LetRing,
    GhostNote,
    DeadNote,

    // Dynamics
    Staccato,
    Accent,

    // Harmonics
    Harmonic(HarmonicType),

    // Picking
    TremoloPicking(TremoloSpeed),

    // Bass
    Slap,
    Pop,

    // Tapping
    Tap,
}

☉ ᛈ SlideType {
    ShiftSlide { to_fret: u8 },
    LegatoSlide { to_fret: u8 },
    SlideInFromAbove,
    SlideInFromBelow,
    SlideOutUp,
    SlideOutDown,
}

☉ Σ BendData {
    bend_type: BendType,
    points: Vec<BendPoint>,
}

☉ ᛈ BendType {
    Bend,
    BendRelease,
    Prebend,
    PrebendRelease,
}

☉ Σ BendPoint {
    position: f32,            // 0.0-1.0 (position in note duration)
    value: f32,               // Semitones (0.5 = quarter, 1.0 = half, 2.0 = whole)
}

☉ ᛈ VibratoStyle {
    Normal,
    Wide,
    Slight,
}

☉ ᛈ HarmonicType {
    Natural,
    Artificial { pitch: u8 },
    Pinch,
    Tap,
    Semi,
}

☉ ᛈ TremoloSpeed {
    Eighth,
    Sixteenth,
    ThirtySecond,
}
```

### 3.2 Technique Behavioral Contracts

| Technique | Preconditions | Visual | Audio Effect |
|-----------|---------------|--------|--------------|
| HammerOn | Target fret > source fret, same string | "h" arc | Attack on first note only |
| PullOff | Target fret < source fret, same string | "p" arc | Attack on first note only |
| Slide | Same string | "/" or "\" line | Pitch glide |
| Bend | fret > 0 | Curved arrow | Pitch shift |
| Vibrato | fret > 0 | "~" wavy line | Pitch oscillation |
| PalmMute | Any note | "PM---" bracket | Dampened tone |
| LetRing | Any note | "LR---" bracket | Extended sustain |
| GhostNote | Any note | Parentheses | Reduced velocity |
| Harmonic | Natural: frets 5,7,12,19 | Diamond | Overtone |
| Tap | Any note | "T" marker | Right-hand attack |

---

## 4. Component Behavior Specifications

### 4.1 Fretboard Component

#### 4.1.1 Layout

```
Fret:    0    1    2    3    4    5    6    7    8    9   10   11   12
         │    │    │    │    │    ●    │    ●    │    ●    │    │   ●●
String 1 ├────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┤  e
String 2 ├────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┤  B
String 3 ├────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┤  G
String 4 ├────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┤  D
String 5 ├────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┤  A
String 6 ├────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┤  E
```

- Fret markers at positions: 3, 5, 7, 9, 12 (double), 15, 17, 19, 21, 24 (double)
- String spacing: 24px
- Fret width: Logarithmic scale (narrower at higher frets)
- Visible frets: Configurable (default 0-12, scrollable to 24)

#### 4.1.2 Interaction

| Input | Action |
|-------|--------|
| Click on fret | Add/select note at cursor position |
| 0-9 keys | Enter fret number (multi-digit with 500ms timeout) |
| Arrow Up/Down | Move cursor between strings |
| Arrow Left/Right | Move cursor between positions |
| h/p/s/b/v/m | Toggle technique on selected note |
| Delete/Backspace | Remove selected note |
| Enter | Confirm note, advance cursor |
| Escape | Cancel pending input |

#### 4.1.3 Visual States

| State | Appearance |
|-------|------------|
| Empty fret | Fret line only |
| Note (unselected) | Blue circle with fret number |
| Note (selected) | Yellow border, shadow |
| Note (at cursor) | Green dashed border |
| Pending input | Gray circle with partial number |
| Scale note | Subtle highlight (when scale display enabled) |

### 4.2 Tab Staff Component

#### 4.2.1 Layout

```
Measure 1                    │ Measure 2
─────────────────────────────┼─────────────────────────────
e ├──3──────5──7─────────────┼──────────────────────────────
B ├─────5──────────8─────────┼──────────────────────────────
G ├──────────────────5───────┼──────────────────────────────
D ├──────────────────────5───┼──────────────────────────────
A ├──────────────────────────┼──────────────────────────────
E ├──────────────────────────┼──────────────────────────────
     1    2    3    4             1    2    3    4
```

- 6 horizontal lines representing strings
- Fret numbers at beat positions
- Measure bars between measures
- Beat numbers below for reference
- Technique symbols above/below notes

#### 4.2.2 Cursor

```sigil
☉ Σ CursorPosition {
    measure: usize,
    string: u8,               // 1-6
    beat_position: f64,       // 0.0 to beats_per_measure
}
```

#### 4.2.3 Selection

- Single note selection: Click on note
- Range selection: Shift+click or drag
- Multi-select: Ctrl+click

### 4.3 Editor Component

#### 4.3.1 Edit Modes

```sigil
☉ ᛈ EditMode {
    Navigate,                 // Arrow keys move between measures
    Insert,                   // Typing adds notes
    Select,                   // Click/drag selects
    Technique,                // Apply techniques to selection
}
```

#### 4.3.2 Undo/Redo System

```sigil
☉ ᛈ EditCommand {
    InsertNote { measure: usize, beat_idx: usize, note: Note },
    DeleteNote { measure: usize, beat_idx: usize, note_idx: usize },
    ModifyNote { measure: usize, beat_idx: usize, note_idx: usize, new_note: Note },
    AddTechnique { note_ref: NoteRef, technique: Technique },
    RemoveTechnique { note_ref: NoteRef, technique_idx: usize },
    InsertMeasure { position: usize },
    DeleteMeasure { position: usize, measure: Measure },
    SetTimeSignature { measure: usize, time_sig: TimeSignature },
}

⊢ EditCommand {
    /// Apply command to score
    rite apply(&self, score: &Δ CompositionData) -> Result<(), EditError>;

    /// Generate inverse command for undo
    rite inverse(&self) -> EditCommand;
}
```

---

## 5. AlphaTab Integration

### 5.1 Architecture

AlphaTab runs as JavaScript library. Sigil WASM communicates via imports/exports:

```
┌─────────────────────┐     ┌─────────────────────┐
│   Sigil WASM        │     │   AlphaTab JS       │
│                     │     │                     │
│  ┌───────────────┐  │     │  ┌───────────────┐  │
│  │ CompositionData│◄──────┼──│ Score Object  │  │
│  └───────────────┘  │     │  └───────────────┘  │
│         │           │     │         ▲           │
│         ▼           │     │         │           │
│  ┌───────────────┐  │     │  ┌───────────────┐  │
│  │ convert_to_tex│──┼─────┼─►│ api.tex(...)  │  │
│  └───────────────┘  │     │  └───────────────┘  │
│                     │     │         │           │
│  ┌───────────────┐  │     │         ▼           │
│  │ Position      │◄─┼─────┼──│ playerPosition │  │
│  │ Callbacks     │  │     │  │ Changed        │  │
│  └───────────────┘  │     │  └───────────────┘  │
└─────────────────────┘     └─────────────────────┘
```

### 5.2 WASM Imports

```sigil
extern "wasm" {
    // AlphaTab API
    rite alphatab_create(container_selector: &str) -> i32;
    rite alphatab_load_tex(handle: i32, tex: &str);
    rite alphatab_load_data(handle: i32, data: &[u8]);
    rite alphatab_play(handle: i32);
    rite alphatab_pause(handle: i32);
    rite alphatab_stop(handle: i32);
    rite alphatab_seek(handle: i32, tick: i32);
    rite alphatab_set_speed(handle: i32, speed: f64);
    rite alphatab_destroy(handle: i32);
}
```

### 5.3 WASM Exports (Callbacks)

```sigil
/// Called when AlphaTab player position changes
#[export_name = "alphatab_on_position"]
☉ rite alphatab_on_position(handle: i32, current_tick: i32, end_tick: i32);

/// Called when AlphaTab player state changes
#[export_name = "alphatab_on_state"]
☉ rite alphatab_on_state(handle: i32, state: i32);

/// Called when score is loaded
#[export_name = "alphatab_on_score_loaded"]
☉ rite alphatab_on_score_loaded(handle: i32, track_count: i32);
```

### 5.4 TEX Conversion

TEX format specification for converting TabScore to AlphaTab:

```
\title "Song Title"
\artist "Artist Name"
\tempo 120
\track "Guitar" "acoustic-guitar-steel"
\tuning E4 B3 G3 D3 A2 E2
\ts 4 4
.
:4 (3.1) (5.2) (5.3) (3.4) | (3.1) (5.2) (5.3) (3.4) |
:8 3.1.8{b(0 4 0)} 5.1 | 7.1{h} 5.1{p} |
```

---

## 6. Playback Specification

### 6.1 Transport States

```sigil
☉ ᛈ TransportState {
    Stopped,
    Playing,
    Paused,
    Recording,
}
```

### 6.2 Playback Coordinator

```sigil
☉ Σ PlaybackCoordinator {
    state: TransportState,
    alphatab_handle: i32,
    tempo: f64,
    current_tick: i32,
    loop_enabled: bool,
    loop_start: i32,
    loop_end: i32,
}

⊢ PlaybackCoordinator {
    rite play(&Δ self);
    rite pause(&Δ self);
    rite stop(&Δ self);
    rite seek(&Δ self, tick: i32);
    rite set_loop(&Δ self, start: i32, end: i32);
}
```

### 6.3 Cursor Synchronization

When playing:
1. AlphaTab calls `alphatab_on_position` with current tick
2. Convert tick to measure/beat position
3. Update editor cursor position
4. Scroll view if cursor exits visible area

---

## 7. File Format Support

### 7.1 Guitar Pro Import

| Format | Extension | Method |
|--------|-----------|--------|
| GP7 | .gp | ZIP containing GPIF XML |
| GP6 | .gpx | ZIP containing score.gpif |
| GP5 | .gp5 | Binary format |
| GP4 | .gp4 | Binary format |
| GP3 | .gp3 | Binary format |

Import uses existing `@orpheus/guitar-pro-parser` JavaScript package via WASM bridge.

### 7.2 Guitar Pro Export

Export to GP7 format:
1. Generate GPIF XML from CompositionData
2. Create ZIP archive with structure:
   - `Content/score.gpif`
   - `Content/[track].wav` (if audio present)
   - `thumbnail.png`

### 7.3 Maestro Native Format

Direct JSON serialization compatible with `maestro-file-format-spec.md`.

---

## 8. Invariants and Property Tests

### 8.1 Note Invariants

```sigil
//@ property: Note string bounds
∀ note: Note ⇒ note.string ≥ 1 ∧ note.string ≤ 6

//@ property: Note fret bounds
∀ note: Note ⇒ note.fret ≤ 24

//@ property: Note velocity bounds
∀ note: Note ⇒ note.velocity ≤ 127

//@ property: Beat notes unique per string
∀ beat: Beat ⇒
    |{n.string | n ∈ beat.notes}| = |beat.notes|
```

### 8.2 Technique Invariants

```sigil
//@ property: HammerOn requires ascending fret
∀ (n1, n2): (Note, Note) where n1.has_technique(HammerOn) ⇒
    n1.string = n2.string ∧ n2.fret > n1.fret

//@ property: PullOff requires descending fret
∀ (n1, n2): (Note, Note) where n1.has_technique(PullOff) ⇒
    n1.string = n2.string ∧ n2.fret < n1.fret

//@ property: Bend value in valid range
∀ bend: BendData ⇒
    ∀ point ∈ bend.points ⇒ point.value ∈ {0.5, 1.0, 1.5, 2.0, 2.5, 3.0}
```

### 8.3 Measure Invariants

```sigil
//@ property: Time signature denominator is power of 2
∀ ts: TimeSignature ⇒
    ts.denominator ∈ {1, 2, 4, 8, 16, 32}

//@ property: Beats fit within measure
∀ measure: Measure ⇒
    ∀ voice ∈ measure.voices ⇒
        sum(beat.duration_quarters() for beat in voice.beats)
        ≤ measure.time_signature.quarters_per_measure()
```

### 8.4 Serialization Invariants

```sigil
//@ property: Note roundtrip
∀ note: Note ⇒ Note::from_json(note.to_json()) = note

//@ property: CompositionData roundtrip
∀ comp: CompositionData ⇒
    CompositionData::from_json(comp.to_json()) = comp

//@ property: TEX conversion preserves notes
∀ comp: CompositionData ⇒
    parse_tex(comp.to_tex()).notes() = comp.all_notes()
```

---

## 9. Event Handling Specification

### 9.1 JS Bridge Events

All keyboard and mouse events are handled in JavaScript and routed to WASM exports:

```javascript
// Keyboard events
document.addEventListener('keydown', (e) => {
    if (e.target.matches('[data-tab-editor]')) {
        wasmExports.tab_editor_keydown(
            encodeKey(e.key),
            e.shiftKey ? 1 : 0,
            e.ctrlKey ? 1 : 0,
            e.altKey ? 1 : 0
        );
        e.preventDefault();
    }
});

// Fretboard clicks via data attributes
document.querySelectorAll('[data-fret]').forEach(el => {
    el.addEventListener('click', () => {
        const [string, fret] = el.dataset.fret.split(',');
        wasmExports.fretboard_click(parseInt(string), parseInt(fret));
    });
});

// Tab staff clicks
document.querySelectorAll('[data-beat]').forEach(el => {
    el.addEventListener('click', () => {
        const [measure, beat] = el.dataset.beat.split(',');
        wasmExports.tab_staff_click(parseInt(measure), parseInt(beat));
    });
});
```

### 9.2 WASM Exports for Events

```sigil
#[export_name = "tab_editor_keydown"]
☉ rite tab_editor_keydown(key_code: i32, shift: i32, ctrl: i32, alt: i32);

#[export_name = "fretboard_click"]
☉ rite fretboard_click(string: i32, fret: i32);

#[export_name = "tab_staff_click"]
☉ rite tab_staff_click(measure: i32, beat: i32);
```

---

## 10. Gap Documentation

This section tracks gaps discovered during implementation (per SDD methodology).

### 10.1 Discovered Gaps

_None yet - this section will be updated as implementation reveals gaps._

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2026-02-12 | Initial specification draft |

---

## References

- [Maestro File Format Specification](../maestro-file-format-spec.md)
- [Guitar Pro File Format](../guitar-pro-file-format.md)
- [AlphaTab Documentation](https://alphatab.net/docs)
- [Spec-Driven Development Methodology](~/dev2/workspace/docs/methodologies/SPEC-DRIVEN-DEVELOPMENT.md)
- [Agent-TDD Methodology](~/dev2/workspace/docs/methodologies/AGENT-TDD.md)
