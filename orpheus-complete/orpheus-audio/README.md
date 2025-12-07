# Maestro Audio - Rust Audio Engine

High-performance native audio processing engine for Maestro, built with Rust following the Eidolon architecture patterns.

## Architecture

```
┌──────────────────────────────────────────────────────┐
│            maestro-desktop (egui GUI)                │
│            60 FPS immediate-mode interface            │
└────────────────────┬─────────────────────────────────┘
                     │ gRPC
                     ▼
┌──────────────────────────────────────────────────────┐
│         maestro-audio-server (gRPC Service)          │
│         Real-time audio processing                    │
└────────────────────┬─────────────────────────────────┘
                     │ FFI
                     ▼
┌──────────────────────────────────────────────────────┐
│         maestro-juce-bridge (TODO)                   │
│         Rust FFI to JUCE C++ audio engine            │
└──────────────────────────────────────────────────────┘
```

## Workspace Structure

- **maestro-proto** - Protocol Buffers definitions (gRPC service)
- **maestro-audio-common** - Shared audio utilities and buffer management
- **maestro-audio-server** - gRPC audio processing server (tokio)
- **maestro-juce-bridge** - Rust FFI bridge to JUCE C++ (TODO)
- **maestro-desktop** - Native GUI application with egui

## Quick Start

### Build All Crates

```bash
cargo build --release
```

### Run Audio Server

```bash
cargo run --bin maestro-audio-server --release
```

Server will start on: `0.0.0.0:50051`

### Run Desktop App

```bash
cargo run --bin maestro-desktop --release
```

## Features

### Implemented

- ✅ Protocol Buffers service definition
- ✅ gRPC audio processing server (mock implementation)
- ✅ Zero-copy audio buffer management
- ✅ egui desktop GUI with 60 FPS rendering
- ✅ Transport controls (play/pause/stop)
- ✅ Multi-track mixer interface
- ✅ Parametric EQ and compressor UI
- ✅ Mode switching (Compose/Mix/Master/Practice)

### TODO

- [ ] JUCE C++ audio engine integration
- [ ] Rust FFI bridge to JUCE
- [ ] VST plugin hosting
- [ ] Real-time audio I/O (ASIO/CoreAudio)
- [ ] Tab editor integration
- [ ] Speed trainer implementation
- [ ] Metronome with audio output
- [ ] File I/O (import/export)
- [ ] Project serialization

## Technology Stack

- **Language**: Rust (stable)
- **GUI**: egui (immediate-mode, 60 FPS)
- **RPC**: tonic (gRPC) + Protocol Buffers
- **Runtime**: tokio (async I/O)
- **Audio**: JUCE (C++) via FFI (planned)

## Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| GUI Frame Rate | 60 FPS | ✅ Achieved |
| Audio Latency | < 10ms | ⏳ Pending JUCE integration |
| Buffer Size | 256-512 samples | ✅ Configurable |
| Sample Rate | 48kHz | ✅ Implemented |
| Max Tracks | 64+ | ✅ No hard limit |

## Development

### Build Server Only

```bash
cargo build --bin maestro-audio-server
```

### Build Desktop Only

```bash
cargo build --bin maestro-desktop
```

### Run Tests

```bash
cargo test
```

### Check All Crates

```bash
cargo check --workspace
```

## gRPC Service

The audio server exposes the following RPCs:

- `ProcessAudio(AudioBuffer) -> AudioBuffer` - Process single buffer
- `StreamAudio(stream AudioBuffer) -> stream AudioBuffer` - Bidirectional streaming
- `LoadPlugin(PluginRequest) -> PluginResponse` - Load VST plugin
- `UpdateEffect(EffectUpdate) -> EffectResponse` - Update effect parameter
- `GetLatency() -> LatencyResponse` - Get current latency

## Desktop GUI

The native desktop app features:

- **Transport**: Play, pause, stop, record controls
- **Modes**: Compose, Mix, Master, Practice
- **Mixer**: Multi-track faders with pan controls
- **Effects**: Parametric EQ, compressor, reverb, delay
- **Track Management**: Add, remove, solo, mute tracks

## Integration with Spring Boot Backend

The Rust audio service integrates with the Spring Boot backend via gRPC:

```
Spring Boot (8080)
    ↓ gRPC client
Rust Audio Service (50051)
    ↓ FFI
JUCE C++ Engine
```

## License

Copyright © 2024 Orpheus
