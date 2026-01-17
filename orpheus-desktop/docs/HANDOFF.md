# Orpheus Desktop - Session Handoff Document

**Session Date:** 2025-12-20
**Branch:** `claude/resume-orpheus-rust-app-XW0iA`
**Agent:** Claude Code (Opus 4.5)

---

## Session Summary

This session completed the TDD implementation of **Phases 1-6B** of the Orpheus Rust desktop application, a professional DAW (Digital Audio Workstation) for music composition and production.

### What Was Accomplished

| Phase | Description | New Tests | Key Deliverables |
|-------|-------------|-----------|------------------|
| 1 | Audio Output | 17 | cpal integration, AudioEngine |
| 2 | MIDI I/O | 21 | midir input/output, MidiRouter |
| 3 | File Dialogs | 23 | rfd dialogs, project serialization |
| 4 | AI Integration | 12 | Leviathan client, persona support |
| 5 | UI Polish | 11 | daemoniorum-egui theming |
| 6 | Plugin Framework | 52 | Scanner, Host, Mock instances |
| 6B | Custom Plugins | 85 | Full CLAP + VST3 FFI & hosting |

**Total: 221+ tests, all passing**

---

## Key Files Created/Modified

### Phase 6B: Custom Plugin Hosting (Most Recent)

```
crates/orpheus-plugins/src/
├── lib.rs              # Module exports with clap + vst3
├── format.rs           # PluginFormat, PluginCategory (15 types)
├── scanner.rs          # PluginScanner, PluginMetadata
├── host.rs             # PluginHost, PluginInstance (mock)
├── clap/
│   ├── mod.rs          # CLAP module exports
│   ├── ffi.rs          # Complete CLAP 1.2.2 C type definitions
│   ├── host.rs         # ClapHost, ClapHostState, events
│   └── plugin.rs       # ClapPluginLoader, ClapPluginInstance
└── vst3/
    ├── mod.rs          # VST3 module exports
    ├── com.rs          # TUID, FUnknown, ComPtr, IPluginFactory
    ├── ffi.rs          # ProcessData, AudioBusBuffers, interfaces
    ├── host.rs         # Vst3Host, Vst3ComponentHandler
    └── plugin.rs       # Vst3PluginLoader, Vst3PluginInstance
```

### Configuration Changes

**`Cargo.toml` (workspace root):**
- Changed `daemoniorum-egui` from `features = ["full"]` to explicit features:
  ```toml
  features = ["basic-widgets", "advanced-widgets", "theming", "panel-system", "accessibility"]
  ```

---

## Technical Decisions

### 1. Custom Plugin Hosting

**Decision:** Implement CLAP and VST3 hosting from scratch rather than using external crates like `vst3-sys` or `clack`.

**Rationale:**
- Full control over the plugin lifecycle
- Avoid dependency on potentially unmaintained crates
- Ability to add custom extensions and optimizations
- Educational value for understanding plugin formats

**Trade-offs:**
- More initial development time
- Need to maintain FFI definitions ourselves
- Must handle platform-specific bundle layouts

### 2. CLAP-First Design

**Decision:** Prioritize CLAP support with VST3 as secondary.

**Rationale:**
- CLAP is open-source and well-documented
- Simpler than VST3's COM architecture
- Modern features (polyphonic modulation, thread-safe events)
- No licensing concerns

### 3. Mock vs Real Plugin Processing

**Decision:** Phase 6 creates a complete mock framework; Phase 6B adds real FFI but still uses mocks for testing.

**Rationale:**
- Tests can run without actual plugin binaries
- CI/CD doesn't require audio hardware
- Real plugin testing reserved for integration tests

---

## Architecture Highlights

### Plugin Loading Flow

```
1. User selects plugin file (.clap or .vst3)
   ↓
2. PluginScanner identifies format from extension
   ↓
3. Format-specific loader (ClapPluginLoader or Vst3PluginLoader)
   ↓
4. Dynamic library loaded via libloading
   ↓
5. Entry point found (clap_entry or GetPluginFactory)
   ↓
6. Factory queried for plugin descriptors
   ↓
7. Plugin instance created and initialized
   ↓
8. Plugin activated with sample rate and buffer size
   ↓
9. Audio processing loop begins
```

### VST3 COM Architecture

```
IPluginFactory (from GetPluginFactory)
    ↓ createInstance(cid, IComponent::iid)
IComponent (plugin component)
    ↓ queryInterface(IAudioProcessor::iid)
IAudioProcessor (audio processing)
    ↓ setupProcessing(ProcessSetup)
    ↓ setProcessing(true)
    ↓ process(ProcessData)
```

### CLAP Event Flow

```
ClapHost
    ├── request_restart() → Plugin needs full restart
    ├── request_process() → Resume audio processing
    └── request_callback() → Main thread callback needed

ClapInputEvents
    ├── NoteOn(channel, key, velocity)
    ├── NoteOff(channel, key)
    ├── ParamValue(param_id, value)
    └── Midi(port, data[3])

ClapOutputEvents
    └── (Plugin writes automation, note events)
```

---

## Known Issues & Warnings

### Compiler Warnings (Acceptable)

1. **Unused fields in loaders** - `library`, `path` kept for lifetime management
2. **Non-snake-case in FFI** - Matches C API naming conventions
3. **Dead code in event handlers** - Reserved for future use

### Potential Issues for Next Session

1. **Real plugin testing** - Need actual .vst3/.clap files for integration tests
2. **Audio thread safety** - Plugin processing should be lock-free
3. **Parameter automation** - Not yet connected to UI
4. **Plugin state persistence** - Save/load with project not implemented

---

## Commands for Verification

```bash
# Run all orpheus-plugins tests
cd /home/user/workspace/orpheus/orpheus-desktop
cargo test -p orpheus-plugins

# Run all workspace tests
cargo test --workspace

# Check for warnings
cargo clippy --workspace

# View recent commits
git log --oneline -10
```

---

## Commits This Session

```
052954bdf feat(orpheus-plugins): implement Phase 6B custom CLAP/VST3 hosting
fc0e898e5 feat(orpheus-plugins): implement Phase 6 plugin hosting framework
d38c59f57 feat(orpheus-ui): implement Phase 5 UI polish with daemoniorum-egui
```

(Earlier phases were completed in previous sessions)

---

## Recommended Next Steps

### Immediate (Phase 7)

1. **Integration test with real plugins**
   - Download open-source test plugins (e.g., JUCE AudioPluginHost demo)
   - Create integration test that loads, processes, and unloads

2. **Connect plugin instances to audio graph**
   - Wire PluginInstance.process() to AudioEngine
   - Handle buffer size mismatches

3. **Plugin parameter UI**
   - Enumerate parameters from plugin
   - Display sliders/knobs in UI
   - Connect to automation system

### Near-term (Phase 8)

1. **Performance profiling**
   - Measure plugin processing latency
   - Optimize hot paths in audio thread

2. **Plugin state management**
   - Serialize plugin state (getState/setState)
   - Include in project save format

### Long-term (Phase 9+)

1. **Plugin browser UI**
   - Scan results displayed in panel
   - Category filtering, search
   - Favorites and recently used

2. **Plugin chain/rack**
   - Multiple plugins per track
   - Drag-and-drop reordering
   - Bypass per plugin

---

## References

- [CLAP Specification](https://github.com/free-audio/clap)
- [VST3 SDK Documentation](https://steinbergmedia.github.io/vst3_dev_portal/)
- [Orpheus Implementation Plan](./IMPLEMENTATION_PLAN.md)

---

**Status: Ready for Phase 7 - Integration Testing**
