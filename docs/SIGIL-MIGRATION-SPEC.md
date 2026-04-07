# Orpheus Sigil Migration Specification

**Version:** 0.3.0
**Status:** Draft
**Date:** 2026-02-11
**Branch:** feature/sigil-migration

---

## 1. Conceptual Foundation

### 1.1 Migration Intent

Migrate Orpheus from a multi-language, multi-platform codebase to a unified pure-Sigil implementation. This consolidates 5 divergent variants into a single canonical codebase that compiles to all target platforms from one source.

**Primary goals:**
- Unify desktop and web into one codebase (Qliphoth → GTK4 + WASM)
- Replace external audio dependencies with Amdusias (Daemoniorum's native engine)
- Leverage Sigil's polycultural sound support for global music systems
- Eliminate React, Spring Boot, and JUCE dependencies

### 1.2 What This Migration IS

- ✅ Consolidation of divergent implementations into one canonical source
- ✅ Automated migration via `rust-to-sigil` compiler tool for existing Rust code
- ✅ Fresh implementation of UI using Qliphoth framework
- ✅ Integration with Amdusias audio engine (also migrating to Sigil)
- ✅ Native support for polycultural sound systems via Sigil

### 1.3 What This Migration IS NOT

- ❌ Line-by-line port of 270k+ lines of existing code
- ❌ Maintaining backwards compatibility with TypeScript/Kotlin APIs
- ❌ Preserving the 5 divergent variant structure
- ❌ Using JUCE, cpal, or external audio libraries

### 1.4 Success Criteria

- [ ] Single Sigil codebase compiles to desktop (GTK4) and web (WASM)
- [ ] Audio engine runs natively (WASAPI/CoreAudio/ALSA) and in browser (AudioWorklet)
- [ ] Feature parity with current orpheus-desktop capabilities
- [ ] Polycultural tuning systems supported (not just 12-TET)
- [ ] All tests passing with Agent-TDD methodology

---

## 2. Current State Analysis

### 2.1 Existing Architecture

| Component | Language | Lines | Migration Path |
|-----------|----------|-------|----------------|
| orpheus-desktop | Rust + egui | 90,519 | `rust-to-sigil` → cleanup |
| orpheus-standalone (TS) | TypeScript | 53,912 | Replace with Qliphoth |
| orpheus-complete (TS) | TypeScript | 49,338 | Replace with Qliphoth |
| orpheus (TS) | TypeScript | 29,791 | Replace with Qliphoth |
| orpheus-music-platform (TS) | TypeScript | 29,791 | Replace with Qliphoth |
| Backend | Kotlin/Spring | 9,249 | Native Sigil server |
| Audio (JUCE bridge) | Rust + C++ | ~4,600 | Replace with Amdusias |
| **Total** | | **~267,200** | |

### 2.2 Amdusias (Audio Engine)

| Crate | Purpose | Lines |
|-------|---------|-------|
| amdusias | Unified re-export | - |
| amdusias-core | Lock-free, SIMD, scheduling | - |
| amdusias-hal | WASAPI/CoreAudio/ALSA/PipeWire | - |
| amdusias-dsp | Biquad, compressor, limiter, reverb | - |
| amdusias-graph | Node routing, automatic PDC | - |
| amdusias-siren | RSE (multi-sample instruments) | - |
| amdusias-web | WASM + AudioWorklet | - |
| **Total** | | **16,966** |

### 2.3 Target Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     ORPHEUS (Pure Sigil)                        │
├─────────────────────────────────────────────────────────────────┤
│  UI Layer: Qliphoth                                             │
│  ├── Desktop: GTK4 (native compile)                             │
│  ├── Web: WASM + DOM (wasm32 compile)                           │
│  └── Server: SSR (native compile)                               │
├─────────────────────────────────────────────────────────────────┤
│  Audio Engine: Amdusias (Sigil)                                 │
│  ├── Native: WASAPI / CoreAudio / ALSA / PipeWire               │
│  ├── Web: AudioWorklet + SharedArrayBuffer                      │
│  ├── DSP: biquad, compressor, limiter, reverb, convolution      │
│  ├── Graph: node-based routing, automatic PDC                   │
│  └── RSE: multi-sample instruments with articulations           │
├─────────────────────────────────────────────────────────────────┤
│  Sigil Language Features                                        │
│  ├── Polycultural Sound: non-Western tuning, scales, rhythms    │
│  ├── Evidentiality: !, ~, ?, ‽ for data provenance              │
│  └── Pipe operators: φ (filter), σ (sort), τ (map)              │
└─────────────────────────────────────────────────────────────────┘
```

### 2.4 Dependency Replacement

| Current | Replacement | Status |
|---------|-------------|--------|
| egui | Qliphoth (GTK4 + WASM) | ✅ Available |
| React/Fluent UI | Qliphoth (WASM) | ✅ Available |
| JUCE | Amdusias | ✅ Available |
| cpal | amdusias-hal | ✅ Available |
| Spring Boot | Native Sigil server | 🔮 To build |

---

## 3. Sigil Language Features

### 3.1 Polycultural Sound Support

Sigil provides **native stdlib support** for global music systems:

#### Tuning Systems

```sigil
// 22-Shruti Indian tuning (microtonal)
≔ sa = shruti_freq(1)                 // 256.0 Hz (Sa - tonic)
≔ re = shruti_freq(3)                 // Komal Re (flat 2nd)

// Arabic quarter-tones (24-TET)
≔ rast = arabic_quarter_freq(0)       // 440.0 Hz (Rast maqam root)

// Sacred frequencies
≔ om = sacred_freq("om")              // 136.1 Hz (Earth year frequency)
≔ healing = sacred_freq("528")        // 528.0 Hz (Solfeggio - "DNA repair")

// Chakra frequencies
≔ root = chakra_freq("root")          // 396.0 Hz
≔ heart = chakra_freq("heart")        // 639.0 Hz
≔ crown = chakra_freq("crown")        // 963.0 Hz
```

#### Integration with Amdusias

```sigil
// Generate shruti-tuned oscillator
≔ osc = Oscillator::new(shruti_freq(1)!)
≔ samples = osc |> generate(buffer_size)!
               |> apply_envelope(adsr)~

// Maqam-aware pitch shifting
≔ shifted = samples |τ{ pitch_shift(_, arabic_quarter_freq(step)!) }
```

#### Cultural Color-Sound Synesthesia

```sigil
// Full synesthesia with cultural context
≔ unified = synesthesia("love", "indian")
// → {color: red, chakra: "Root", wu_xing: "Fire (火)", frequency: 639.0}

// Map chakra to audio parameters
≔ freq = chakra_freq(unified.chakra)!
≔ color = chakra_color(unified.chakra)!
```

#### Implications for Orpheus

- **Tab Editor**: Support shruti/maqam notation, not just Western frets
- **Tuning Panel**: Preset tunings for 22-shruti, 24-TET, just intonation
- **Practice Mode**: Drone generation at culturally-appropriate pitches
- **Export**: Microtonal MIDI (MPE) or audio rendering

### 3.2 Evidentiality in Audio

Sigil's evidentiality markers apply to audio data:

| Marker | Audio Context |
|--------|---------------|
| `!` | Locally computed samples, verified buffers |
| `~` | Streamed audio, external sources |
| `?` | Optional audio data, nullable buffers |
| `‽` | User input, untrusted audio files |

### 3.3 Pipe-Based DSP

```sigil
// Example: audio processing chain
samples
    |> normalize!
    |> eq_band(1000.0, 0.707, 3.0)~
    |> compress(-20.0, 4.0)!
    |> limit(-0.3)!
```

---

## 4. Migration Strategy

### 4.1 Approach

**Component-by-component with automated migration + idiomatic refactoring:**

1. Use `rust-to-sigil` compiler tool on Rust codebases (starting point)
2. **Refactor for idiomatic Sigil:**
   - Add evidentiality markers (`!` computed, `~` external, `?` uncertain, `‽` untrusted)
   - Convert to morpheme operators (φ filter, σ sort, τ map, ρ reduce, etc.)
   - Leverage pipe syntax for composition (`|>`, `|φ`, `|σ`, `|τ`)
   - Increase semantic density (Sigil is intentionally expressive)
3. Build Qliphoth UI components
4. Integrate Amdusias audio engine
5. Consolidate into unified app

> **Key insight:** `rust-to-sigil` produces valid but non-idiomatic code. The real work is refactoring to leverage Sigil's unique features for clarity and correctness.

### 4.2 Phase Breakdown

#### Phase 0: Infrastructure ✅

- [x] Create feature branch
- [x] Set up Daemoniorum methodology infrastructure
- [x] Create CONCLAVE.sigil
- [x] Document migration spec

#### Phase 1: Amdusias Migration

- [ ] Copy amdusias from monorepo to ~/dev/amdusias
- [ ] Run `rust-to-sigil` on amdusias crates
- [ ] Clean up migrated code
- [ ] Add polycultural sound primitives
- [ ] Verify native + WASM compilation

#### Phase 2: Core Migration

- [ ] Run `rust-to-sigil` on orpheus-desktop/crates/orpheus-core
- [ ] Run `rust-to-sigil` on orpheus-desktop/crates/orpheus-file
- [ ] Run `rust-to-sigil` on orpheus-desktop/crates/orpheus-midi
- [ ] Clean up and integrate with Amdusias

#### Phase 3: UI Migration

- [ ] Design Qliphoth component architecture
- [ ] Build core UI components (tab editor, mixer, transport)
- [ ] Build mode views (compose, record, mix, master, practice, distribute)
- [ ] Implement state management

#### Phase 4: Integration

- [ ] Connect UI to audio engine
- [ ] Implement file I/O (.maestro format)
- [ ] Build server component
- [ ] Cross-platform testing

### 4.3 Migration Order

```
1. amdusias (audio foundation)
   └── Enables all audio features

2. orpheus-core (data models)
   └── Project, Track, Note, etc.

3. orpheus-file (persistence)
   └── .maestro, Guitar Pro import

4. orpheus-midi (MIDI handling)
   └── Playback, recording, quantization

5. orpheus-synth (instruments)
   └── Integrate with amdusias-siren

6. orpheus-ui (Qliphoth)
   └── All user interface

7. orpheus-app (orchestration)
   └── Tie everything together
```

---

## 5. Constraints & Invariants

### 5.1 Hard Constraints

- Must compile to both native (GTK4) and WASM from single codebase
- Audio latency must remain <10ms on native
- Must support offline operation (no required network)
- Zero external audio library dependencies

### 5.2 Soft Constraints

- Prefer Sigil idioms over direct Rust translation
- Prefer property tests over example tests
- UI should feel native on each platform

### 5.3 Invariants

```
P1: ∀ component ∈ orpheus:
    compiles(component, native) ∧ compiles(component, wasm32)
    // Every component must compile to both targets

P2: ∀ tuning ∈ supported_tunings:
    playable(tuning, native) ⟺ playable(tuning, web)
    // All tuning systems work identically on all platforms

P3: project_save(project) |> project_load = project
    // Save/load roundtrip preserves all data
```

---

## 6. Open Questions

1. **Polycultural sound**: What specific primitives does Sigil provide? How do we expose them in Orpheus?

2. **GTK4 on Windows**: Is Qliphoth's GTK4 backend cross-platform, or do we need a Windows-specific renderer?

3. **rust-to-sigil quality**: How much cleanup is typically needed after automated migration?

4. **Amdusias API stability**: Is the current amdusias API the target, or will it change during Sigil migration?

5. **Feature flags**: Should we support building with/without certain features (e.g., RSE, web)?

---

## 7. Testing Strategy

### 7.1 Agent-TDD Approach

Tests are crystallized understanding, not coverage theater:

- **Property tests** for audio processing (roundtrip, invariants)
- **Boundary tests** for file parsing (malformed input handling)
- **Specification tests** for music theory (scale generation, chord voicing)

### 7.2 Migration Validation

For each migrated component:
1. Port existing Rust tests via `rust-to-sigil`
2. Verify identical behavior on reference inputs
3. Add property tests for discovered invariants

### 7.3 Cross-Platform Testing

- Native: Linux (primary), macOS, Windows
- Web: Chrome, Firefox, Safari
- Audio: Verify latency and quality on all HAL backends

---

## 8. Related Repositories

| Repository | Purpose | Status |
|------------|---------|--------|
| ~/dev/orpheus | Main application | This repo |
| ~/dev/amdusias | Audio engine | Fresh repo, needs content |
| ~/dev/qliphoth | UI framework | Available |
| ~/dev2/workspace/nyx/amdusias | Monorepo source | Source for migration |

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2026-02-11 | Initial scaffold |
| 0.2.0 | 2026-02-11 | Full scope documented after codebase exploration. Added Amdusias, Qliphoth, polycultural sound, migration phases. |
| 0.3.0 | 2026-02-11 | Documented Sigil's native polycultural audio stdlib (shruti, maqam, chakra frequencies). Added idiomatic refactoring requirements. |
