# Orpheus UI Specification

**Version:** 0.1.0
**Status:** Draft
**Date:** 2026-02-11
**Parent Spec:** Qliphoth UI Framework (pending)

---

## 1. Conceptual Foundation

### 1.1 Purpose

Orpheus UI provides the graphical interface for the Orpheus music production platform. It enables musicians to compose, record, mix, master, and practice music through a unified interface that compiles to both web (WASM) and desktop (GTK4) targets via the Qliphoth UI framework.

### 1.2 Design Philosophy

**Mode-Based Workflow:** The application organizes functionality into distinct modes (Compose, Record, Mix, Master, Practice, Release, Distribute) that present focused interfaces for specific workflows while sharing common state.

**Corporate Goth Aesthetic:** Visual design follows the Daemoniorum Corporate Goth design system with dark backgrounds (VOID, ABYSS), silver text, and phthalo green accents.

**Professional Audio UI Conventions:** Controls follow DAW industry standards (vertical faders, arc-style knobs, dB-scale meters) to ensure familiarity for audio professionals.

### 1.3 Target Users

- Guitarists who compose with tablature
- Recording musicians tracking audio/MIDI
- Mixing engineers with multi-track projects
- Mastering engineers targeting loudness standards
- Learners using practice tools

---

## 2. Type Architecture

### 2.1 Design Tokens

```
ColorPalette:
    // Base colors (from Qliphoth)
    VOID: Color           // Deepest background (#0a0a0f)
    ABYSS: Color          // Panel backgrounds
    SILVER: Color         // Primary text
    PHTHALO: Color        // Accent/interactive

    // Audio-specific extensions
    METER_GREEN: Color    // Safe audio level
    METER_YELLOW: Color   // Warning level (-6 to 0 dB)
    METER_RED: Color      // Clipping (> 0 dB)

    // Track type colors
    AUDIO_TRACK: Color    // Muted green
    MIDI_TRACK: Color     // Muted purple
    BUS_TRACK: Color      // Muted gold
    MASTER_TRACK: Color   // Muted red

Spacing:
    TRACK_HEIGHT: f32         // Default track lane height
    TRACK_HEIGHT_COMPACT: f32 // Collapsed track height
    FADER_WIDTH: f32          // Vertical fader width
    METER_WIDTH: f32          // Level meter width
    CHANNEL_STRIP_WIDTH: f32  // Mixer channel width
```

### 2.2 Audio Controls

```
Knob:
    value: f32              // Normalized 0.0 - 1.0
    min: f32                // Display minimum
    max: f32                // Display maximum
    bipolar: bool           // Center is zero
    style: Arc | Notched | Dot

    invariant: 0.0 <= value <= 1.0
    behavior: drag_y(delta) → value' where value' = clamp(value - delta * sensitivity, 0, 1)

LevelMeter:
    level_db~: f32          // Current level (external/uncertain)
    peak_db~: f32           // Peak hold (external)
    min_db: f32             // Display floor (typically -60)
    max_db: f32             // Display ceiling (typically +6)
    stereo: bool            // Show L/R channels

    invariant: min_db < max_db
    behavior: display_height = (level_db - min_db) / (max_db - min_db)
    behavior: clip_indicator visible when peak_db > 0

Fader:
    value: f32              // Normalized 0.0 - 1.0
    orientation: Vertical | Horizontal

    invariant: 0.0 <= value <= 1.0
    behavior: maps to dB via standard audio curve
    behavior: drag perpendicular to orientation changes value
```

### 2.3 Timeline Components

```
TimePosition:
    beats: f64              // Position in musical beats

    derived:
        bars = floor(beats / beats_per_bar) + 1
        beat_in_bar = floor(beats mod beats_per_bar) + 1
        ticks = (beats mod 1.0) * TICKS_PER_BEAT

TimelineRuler:
    scroll_x: f64           // Left edge in beats
    zoom: f32               // Pixels per beat multiplier
    time_signature: (u8, u8) // Numerator, denominator

    behavior: displays bar numbers at bar boundaries
    behavior: displays beat markers at beat boundaries
    behavior: click sets playhead position

Playhead:
    position~: f64          // Current playback position (external)

    behavior: renders as vertical line across all tracks
    behavior: follows transport state during playback
```

### 2.4 Track System

```
TrackType: Audio | Midi | Bus | Master

Track:
    id: TrackId
    name: String
    track_type: TrackType
    color: Color            // Derived from track_type
    volume: f32             // 0.0 - 1.5 (allows +3.5 dB headroom)
    pan: f32                // -1.0 (L) to +1.0 (R)
    muted: bool
    soloed: bool
    armed: bool             // Record arm
    clips: [Clip]

    invariant: Master track cannot be armed
    invariant: 0.0 <= volume <= 1.5
    invariant: -1.0 <= pan <= 1.0

Clip:
    id: ClipId
    name: String
    start: f64              // Start position in beats
    end: f64                // End position in beats
    clip_type: AudioClip { waveform } | MidiClip { notes }

    invariant: start < end
    invariant: end - start > MIN_CLIP_LENGTH
```

### 2.5 Mode System

```
OrpheusMode: Compose | Record | Mix | Master | Practice | Release | Distribute | Settings

ModeContainer:
    current_mode: OrpheusMode

    behavior: exactly one mode active at any time
    behavior: mode change triggers lazy-load of mode component
    behavior: mode state persists across mode switches within session
```

### 2.6 Mixer Components

```
ChannelStrip:
    channel: Channel
    level_db~: f32          // Realtime meter level
    peak_db~: f32           // Peak hold
    selected: bool

    contains:
        - Insert slots (0..N)
        - Send knobs (0..N)
        - Pan knob
        - Fader
        - Level meter
        - Mute/Solo/Arm buttons

    behavior: volume fader controls channel.volume
    behavior: pan knob controls channel.pan
    behavior: M button toggles mute
    behavior: S button toggles solo
    behavior: R button toggles arm (except Master)

MixerView:
    channels: [ChannelStrip]
    master: ChannelStrip

    layout: channels displayed horizontally
    layout: master separated by divider on right
```

### 2.7 Loudness Metering

```
LufsReading:
    momentary~: f32         // 400ms window
    short_term~: f32        // 3s window
    integrated~: f32        // Full program
    true_peak~: f32         // dBTP
    loudness_range~: f32    // LU

    all values external (from audio analysis)

TargetStandard:
    Streaming { target: -14.0 LUFS, ceiling: -1.0 dBTP }
    CD { target: -9.0 LUFS, ceiling: -0.3 dBTP }
    Broadcast { target: -24.0 LUFS, ceiling: -2.0 dBTP }
    Film { target: -27.0 LUFS, ceiling: -3.0 dBTP }

LufsMeter:
    reading~: LufsReading
    target: TargetStandard
    show_history: bool

    behavior: color indicates compliance with target
        too_loud (> target + 2 LU) → red
        slightly_loud (0 to +2 LU) → yellow
        good (-3 to 0 LU) → green
        too_quiet (< -3 LU) → dimmed
```

### 2.8 Instrument Editors

```
TabEditor:
    score: TabScore
    playhead~: f64
    selected_notes: [(measure_idx, string_idx, position)]
    edit_mode: Select | Insert | Delete
    tuning: [u8; 6]         // MIDI notes, high to low

    invariant: tuning.len() == number of strings displayed
    behavior: note entry via number keys sets fret
    behavior: arrow keys navigate between positions
    behavior: playhead follows transport

FretboardDisplay:
    tuning: [u8]            // MIDI notes for open strings
    frets: u8               // Number of frets to display
    active_notes~: [u8]     // Currently sounding MIDI notes
    scale: Option<Scale>    // Scale to highlight
    root: Option<Note>      // Root note for scale

    behavior: fret positions use logarithmic scaling (real guitar)
    behavior: active notes displayed as filled circles
    behavior: scale notes displayed as hollow circles
    behavior: fret markers at 3, 5, 7, 9, 12, 15, 17, 19, 21, 24
    behavior: double markers at octaves (12, 24)

PianoRollEditor:
    notes: [MidiNote]
    playhead~: f64
    selection: [NoteId]
    snap_resolution: f64    // In beats (0.25 = 16th note)
    velocity_visible: bool

    behavior: vertical axis is pitch (C-1 to G9)
    behavior: horizontal axis is time in beats
    behavior: note height indicates velocity (if visible)
    behavior: drag creates new note
    behavior: click selects note
```

---

## 3. Behavioral Contracts

### 3.1 Transport

```
TransportState: Stopped | Playing | Recording

transport.play():
    pre: state = Stopped ∨ state = Playing
    post: state' = Playing
    post: playhead advances with realtime

transport.stop():
    pre: true
    post: state' = Stopped
    post: playhead = 0 (if stop twice)

transport.record():
    pre: ∃ track ∈ tracks: track.armed
    post: state' = Recording
    post: armed tracks receive input
```

### 3.2 Mixer Routing

```
Solo behavior (standard DAW):
    ∀ track:
        audible(track) =
            (¬∃ t: t.soloed) ∧ ¬track.muted  // No solos → normal mute behavior
            ∨ track.soloed                     // Or track is soloed

Volume to dB conversion:
    db(volume) =
        -∞           if volume ≤ 0.0001
        20 * log10(volume)  otherwise
```

### 3.3 Clip Operations

```
clip.move(new_start, new_track):
    pre: new_start >= 0
    post: clip.start' = snap(new_start)
    post: clip.end' = clip.start' + (clip.end - clip.start)
    post: clip ∈ new_track.clips

clip.resize_left(new_start):
    pre: new_start < clip.end - MIN_CLIP_LENGTH
    post: clip.start' = snap(new_start)
    invariant: clip.end unchanged

clip.resize_right(new_end):
    pre: new_end > clip.start + MIN_CLIP_LENGTH
    post: clip.end' = snap(new_end)
    invariant: clip.start unchanged
```

### 3.4 Mode Transitions

```
mode.switch(target_mode):
    pre: target_mode ∈ OrpheusMode
    post: current_mode' = target_mode
    invariant: project state preserved across switch
    behavior: previous mode state preserved (lazy unload)
```

---

## 4. Constraints & Invariants

### 4.1 Audio Constraints

```
P1: ∀ meter:
    meter.min_db < meter.max_db
    // Meter range is valid

P2: ∀ channel:
    0.0 <= channel.volume <= 1.5
    // Volume in valid range (with headroom)

P3: ∀ channel:
    -1.0 <= channel.pan <= 1.0
    // Pan in valid range

P4: master_track.armed = false
    // Master track cannot be armed for recording
```

### 4.2 Timeline Constraints

```
P5: ∀ clip ∈ track.clips:
    clip.start < clip.end
    // Clips have positive duration

P6: playhead >= 0
    // Playhead never negative

P7: ∀ track:
    ∀ clip_a, clip_b ∈ track.clips where clip_a ≠ clip_b:
        clip_a.end <= clip_b.start ∨ clip_b.end <= clip_a.start
    // Clips do not overlap within track
```

### 4.3 UI State Constraints

```
P8: exactly_one(modes, m => m.active)
    // Exactly one mode active at a time

P9: ∀ knob:
    0.0 <= knob.value <= 1.0
    // Knob values normalized

P10: ∀ fader:
    0.0 <= fader.value <= 1.0
    // Fader values normalized
```

### 4.4 LUFS Constraints

```
P11: ∀ target ∈ TargetStandard:
    target.ceiling <= 0.0 dBTP
    // True peak ceiling never exceeds 0 dBTP

P12: lufs_meter.integrated~ observable only after >= 400ms audio
    // Integration requires minimum audio
```

---

## 5. Error Conditions

### 5.1 Audio Engine Errors

| Condition | Signal | Recovery |
|-----------|--------|----------|
| Engine disconnected | Connection toast | Retry connection |
| Buffer underrun | Glitch indicator | Auto-recover |
| Sample rate mismatch | Warning dialog | Offer resample |

### 5.2 File Errors

| Condition | Signal | Recovery |
|-----------|--------|----------|
| Audio file not found | Missing clip indicator | Offer locate |
| Corrupt project file | Error dialog | Offer backup |
| Disk full | Save error | Offer alternative location |

### 5.3 UI Errors

| Condition | Signal | Recovery |
|-----------|--------|----------|
| Component render failure | Fallback UI | Log error, continue |
| Invalid state | Debug assertion | Reset to known state |

---

## 6. Integration Points

### 6.1 Orpheus Core

```
orpheus-core provides:
    - Track, Clip, Region types
    - Transport state machine
    - Audio routing graph
    - MIDI event types

orpheus-ui consumes:
    - Project state (tracks, clips)
    - Transport state (playing, position)
    - Mixer state (levels, peaks)
    - LUFS analysis data
```

### 6.2 Qliphoth UI Framework

```
qliphoth-ui provides:
    - Component trait and lifecycle
    - Layout primitives (Stack, Grid, Flex)
    - Basic inputs (Button, Slider, Input)
    - Render context and element! macro

orpheus-ui extends:
    - Audio-specific controls (Knob, LevelMeter, Fader)
    - Design tokens (Corporate Goth + audio semantics)
```

### 6.3 Audio Engine (via gRPC)

```
Audio engine provides:
    - Realtime level metering (50Hz update)
    - LUFS analysis (per-block)
    - Peak detection
    - Waveform data for visualization

orpheus-ui receives:
    - Meter levels via streaming
    - Analysis results via request/response
```

---

## 7. Open Questions

### 7.1 Architecture

1. **Component lazy loading:** Should mode components be code-split for WASM bundle size, or preloaded?
   - Pro split: Smaller initial load
   - Con split: Mode switch latency

2. **State persistence:** Where does UI state (zoom, scroll, panel sizes) persist?
   - Project file? User preferences? Session only?

3. **Undo/redo scope:** Should UI state changes (zoom, selection) be undoable?
   - Pro: Consistent behavior
   - Con: Clutters undo history

### 7.2 Audio-Specific

4. **Meter ballistics:** What attack/release curves for level meters?
   - VU-style (300ms)? PPM-style (fast attack)? User configurable?

5. **Waveform resolution:** How to handle zoom levels for waveform display?
   - Pre-computed mip-maps? On-demand rendering?

6. **MIDI preview in clips:** Show velocity as color or height in arrangement view?

### 7.3 Platform-Specific

7. **GTK4 vs WASM visual parity:** How to handle platform-specific rendering differences?
   - Canvas vs native widgets for meters?
   - Font rendering consistency?

8. **Keyboard shortcuts:** Platform-specific (Cmd vs Ctrl) or unified?

### 7.4 Accessibility

9. **Screen reader support:** How to announce audio levels and transport state?

10. **High contrast mode:** Should there be an alternative to Corporate Goth for accessibility?

---

## 8. Component Inventory

### 8.1 Status Overview

| Category | Component | Status | Notes |
|----------|-----------|--------|-------|
| **Tokens** | tokens.sg | ⚠️ | Implemented, needs spec compliance check |
| **Widgets** | Knob | ⚠️ | Implemented, no tests |
| | LevelMeter | ⚠️ | Implemented, no tests |
| | Fader | ⚠️ | Implemented, no tests |
| | Waveform | ⚠️ | Implemented, no tests |
| | Transport | ⚠️ | Implemented, no tests |
| **Editors** | ArrangementEditor | ⚠️ | Implemented, no tests |
| | PianoRollEditor | ⚠️ | Implemented, no tests |
| | TabEditor | ⚠️ | Implemented, no tests |
| **Panels** | ChannelStrip | ⚠️ | Implemented, no tests |
| | Fretboard | ⚠️ | Implemented, no tests |
| | LufsMeter | ✅ | Has tests (example pattern) |
| | Browser | ⚠️ | Implemented, no tests |
| | Inspector | ⚠️ | Implemented, no tests |
| | MixerView | ⚠️ | Implemented, no tests |
| **Layout** | DockLayout | ⚠️ | Implemented, no tests |
| | SplitPane | ⚠️ | Implemented, no tests |
| **Modes** | ModeSelector | ⚠️ | Implemented, no tests |
| | ComposeMode | ⚠️ | Implemented, no tests |
| | RecordMode | ⚠️ | Implemented, no tests |
| | MixMode | ⚠️ | Implemented, no tests |
| | MasterMode | ⚠️ | Implemented, no tests |
| | PracticeMode | 🔮 | Not started |
| | ReleaseMode | 🔮 | Not started |
| | DistributeMode | 🔮 | Not started |

### 8.2 Compliance Gap

~6000 lines of implementation exist without tests.
Only `lufs_meter.sg` follows the Agent-TDD pattern with embedded tests.

**Next step:** Add tests to existing components following `lufs_meter.sg` pattern.

---

## 9. Revision History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2026-02-11 | Initial draft after methodology violation recognition |

---

## 10. References

- [SPEC-FORMATTING.md](/home/crook/dev2/workspace/docs/methodologies/SPEC-FORMATTING.md)
- [AGENT-TDD.md](/home/crook/dev2/workspace/docs/methodologies/AGENT-TDD.md)
- [Qliphoth UI Framework](pending)
- [Corporate Goth Design System](pending)
