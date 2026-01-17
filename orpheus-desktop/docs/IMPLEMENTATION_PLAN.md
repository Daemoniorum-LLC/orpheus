# Orpheus Desktop - TDD Implementation Plan

**Created:** 2025-12-20
**Branch:** `claude/resume-orpheus-rust-app-XW0iA`
**Last Updated:** 2025-12-20
**Current Status:** Phases 1-6B COMPLETE - Core foundation ready for integration testing

---

## Executive Summary

This plan resumes development of the Orpheus Rust desktop application with a focus on:

1. **TDD-first approach** - Tests written before implementation
2. **Workspace SDK integration** - Leverage existing Daemoniorum crates
3. **Incremental milestones** - Each phase delivers working functionality
4. **Audio-first priority** - Connect synthesis to real audio output

---

## Progress Summary

| Phase | Description | Status | Tests | Commit |
|-------|-------------|--------|-------|--------|
| 1 | Audio Output via cpal | COMPLETE | 17 | `16d401ce6` |
| 2 | MIDI I/O with midir | COMPLETE | 21 | `40a99ead3` |
| 3 | File Dialogs & Project I/O | COMPLETE | 23 | `e9df1cd2e` |
| 4 | AI Integration with Leviathan | COMPLETE | 12 | `88a918bc6` |
| 5 | UI Polish with daemoniorum-egui | COMPLETE | 11 | `d38c59f57` |
| 6 | Plugin Hosting (Mock Framework) | COMPLETE | 52 | `fc0e898e5` |
| 6B | Custom CLAP/VST3 Hosting | COMPLETE | 85 | `052954bdf` |

**Total Tests:** 221+ passing across all crates

---

## Workspace SDK Integration

### Priority 1: Core Dependencies (INTEGRATED)

| SDK | Path | Purpose in Orpheus | Status |
|-----|------|-------------------|--------|
| `daemoniorum-egui` | `nyx/daemoniorum-egui` | Shared UI components, theming, panel system | DONE |
| `platform-events` | `daemoniorum-platform/crates/platform-events` | Event bus for track/project changes | Ready |
| `platform-runtime` | `daemoniorum-platform/crates/platform-runtime` | Async task management | Ready |
| `platform-core` | `daemoniorum-platform/crates/platform-core` | Error handling, core abstractions | Ready |

### Priority 2: Audio & Input (Ready to Integrate)

| SDK | Path | Purpose in Orpheus | Status |
|-----|------|-------------------|--------|
| `aether-audio` | `aether/aether-framework/.../aether-audio` | Audio playback via kira, spatial audio | Available |
| `aether-input` | `aether/aether-framework/.../aether-input` | MIDI controller mapping | Available |
| `aether-animation` | `aether/aether-framework/.../aether-animation` | Parameter automation curves | Available |

### Priority 3: AI Integration (INTEGRATED)

| SDK | Path | Purpose in Orpheus | Status |
|-----|------|-------------------|--------|
| `daemoniorum-agent-sdk` | `nyx/daemoniorum-agent-sdk` | AI agent framework | DONE |
| `grimoire-loader` | `nyx/infernum/.../grimoire-loader` | Load AI personas | Ready |
| `infernum-core` | `nyx/infernum/.../infernum-core` | LLM types and traits | Ready |

---

## Phase 1: Audio Output & Playback (COMPLETE)

**Goal:** Connect existing synthesis engines to real audio output via cpal.

### Completed:
- [x] Create `AudioOutput` struct wrapping cpal stream
- [x] Implement `AudioConfig` with sample rate, buffer size, channels
- [x] Add callback-based audio processing
- [x] Connect `GuitarSynth`, `PianoSynth`, `BassSynth`, `DrumMachine` to output
- [x] Add start/stop/pause controls
- [x] Implement latency reporting

### Files:
- `crates/orpheus-audio/src/output.rs` - AudioOutput, AudioConfig, OutputState
- `crates/orpheus-audio/src/engine.rs` - AudioEngine orchestration
- `crates/orpheus-audio/src/lib.rs` - Module exports

---

## Phase 2: Real MIDI I/O (COMPLETE)

**Goal:** Enable MIDI input from controllers and output to external devices.

### Completed:
- [x] Enumerate MIDI input devices via `midir`
- [x] Parse MIDI messages (note on/off, CC, pitch bend)
- [x] Create `MidiRouter` to map channels to synths
- [x] Add device connection tracking
- [x] Implement MIDI output sending

### Files:
- `crates/orpheus-midi/src/input.rs` - MidiInput, device enumeration
- `crates/orpheus-midi/src/output.rs` - MidiOutput, message sending
- `crates/orpheus-midi/src/router.rs` - MidiRouter, channel mapping
- `crates/orpheus-midi/src/message.rs` - MidiMessage types

---

## Phase 3: File Dialogs & Project I/O (COMPLETE)

**Goal:** Complete file operations with native dialogs.

### Completed:
- [x] Integrate `rfd` for native file dialogs
- [x] Create file filter presets (Maestro, Audio, MIDI)
- [x] Implement async dialog handling
- [x] Add recent files tracking
- [x] Implement project serialization
- [x] Add autosave functionality

### Files:
- `crates/orpheus-file/src/dialog.rs` - FileDialog, OpenDialog, SaveDialog
- `crates/orpheus-file/src/project.rs` - ProjectSerializer, ProjectFormat
- `crates/orpheus-file/src/recent.rs` - RecentFiles tracking
- `crates/orpheus-file/src/autosave.rs` - AutoSaver

---

## Phase 4: AI Integration (COMPLETE)

**Goal:** Connect Leviathan personas for intelligent assistance.

### Completed:
- [x] Create AI client for Leviathan backend
- [x] Implement persona loading and switching
- [x] Add composition context generation
- [x] Create AI suggestion request/response types
- [x] Implement chat session management

### Files:
- `crates/orpheus-ai/src/client.rs` - LeviathanClient, AiConfig
- `crates/orpheus-ai/src/context.rs` - CompositionContext, TrackContext
- `crates/orpheus-ai/src/suggestion.rs` - Suggestion types
- `crates/orpheus-ai/src/session.rs` - AiSession, chat management

---

## Phase 5: UI Polish with daemoniorum-egui (COMPLETE)

**Goal:** Replace custom widgets with standardized ecosystem components.

### Completed:
- [x] Replace custom `Theme` with `daemoniorum-egui` theming
- [x] Add Orpheus-specific color accents (golden)
- [x] Use standardized panel system
- [x] Adopt consistent button/control styles
- [x] Implement widget replacements (Knob, LevelMeter, Fader)

### Files:
- `crates/orpheus-ui/src/theme.rs` - OrpheusTheme using daemoniorum-egui
- `crates/orpheus-ui/src/widgets/mod.rs` - Widget re-exports
- Workspace Cargo.toml - daemoniorum-egui features configured

---

## Phase 6: Plugin Hosting (COMPLETE)

**Goal:** Load and run VST3/CLAP plugins.

### Phase 6 (Mock Framework) - COMPLETE:
- [x] Create `PluginScanner` for discovery
- [x] Implement platform-specific default paths
- [x] Add `PluginMetadata` extraction
- [x] Create `PluginHost` for instance management
- [x] Implement `PluginInstance` with mock processing
- [x] Add `PluginFormat` (VST3, CLAP) support
- [x] Create `PluginCategory` classification (15 categories)

### Phase 6B (Custom Implementation) - COMPLETE:
- [x] **CLAP FFI** - Complete C type definitions from CLAP 1.2.2 spec
- [x] **CLAP Host** - ClapHost with request callbacks, event handling
- [x] **CLAP Plugin Loader** - Dynamic loading via libloading
- [x] **VST3 COM Foundation** - TUID, FUnknown, ComPtr, IPluginFactory
- [x] **VST3 FFI** - AudioBusBuffers, ProcessData, interfaces
- [x] **VST3 Host** - Vst3Host implementing IHostApplication
- [x] **VST3 Plugin Loader** - Platform-specific bundle resolution

### Files:
- `crates/orpheus-plugins/src/format.rs` - PluginFormat, PluginCategory
- `crates/orpheus-plugins/src/scanner.rs` - PluginScanner, PluginMetadata
- `crates/orpheus-plugins/src/host.rs` - PluginHost, PluginInstance
- `crates/orpheus-plugins/src/clap/` - CLAP implementation (4 files)
- `crates/orpheus-plugins/src/vst3/` - VST3 implementation (5 files)

---

## Testing Strategy

### Test Organization

```
crates/orpheus-*/
├── src/
│   ├── lib.rs
│   ├── module.rs
│   └── module_tests.rs      # Unit tests adjacent to code
└── tests/
    └── integration_tests.rs  # Integration tests
```

### Current Test Coverage

| Crate | Tests | Status |
|-------|-------|--------|
| orpheus-audio | 17 | PASS |
| orpheus-midi | 21 | PASS |
| orpheus-file | 23 | PASS |
| orpheus-ai | 12 | PASS |
| orpheus-ui | 11 | PASS |
| orpheus-plugins | 85 | PASS |
| orpheus-core | ~30 | PASS |
| orpheus-synth | ~20 | PASS |
| **Total** | **219+** | **ALL PASSING** |

---

## Next Steps (Phase 7+)

### Phase 7: Integration Testing
- [ ] End-to-end audio pipeline (MIDI → Synth → Plugin → Output)
- [ ] Load real VST3/CLAP plugins
- [ ] UI ↔ Audio thread communication
- [ ] Project save/load with plugin state

### Phase 8: Performance Optimization
- [ ] Audio thread priority tuning
- [ ] Plugin processing latency measurement
- [ ] UI rendering optimization
- [ ] Memory usage profiling

### Phase 9: Platform Packaging
- [ ] Linux AppImage/Flatpak
- [ ] macOS DMG with code signing
- [ ] Windows MSI installer
- [ ] Plugin path configuration

---

## Architecture Overview

```
orpheus-desktop/
├── crates/
│   ├── orpheus-core/     # Project, Track, State management
│   ├── orpheus-audio/    # cpal output, AudioEngine
│   ├── orpheus-midi/     # midir input/output, routing
│   ├── orpheus-synth/    # Guitar, Piano, Bass, Drums
│   ├── orpheus-file/     # Project I/O, dialogs
│   ├── orpheus-ai/       # Leviathan integration
│   ├── orpheus-ui/       # egui + daemoniorum-egui
│   ├── orpheus-plugins/  # VST3/CLAP hosting
│   └── orpheus-export/   # Audio/MIDI export
└── src/
    └── main.rs           # Application entry
```

---

## Dependency Updates (COMPLETE)

### Cargo.toml Changes Applied

```toml
# Daemoniorum UI (ADDED)
daemoniorum-egui = { path = "../../nyx/daemoniorum-egui", features = [
    "basic-widgets", "advanced-widgets", "theming",
    "panel-system", "accessibility"
] }

# Audio (ADDED)
cpal = "0.15"
midir = "0.10"

# File I/O (ADDED)
rfd = "0.15"
dirs = "5.0"

# Plugin Hosting (ADDED)
libloading = "0.8"
parking_lot = "0.12"
```

---

**Foundation Complete - Ready for Integration Testing!**
