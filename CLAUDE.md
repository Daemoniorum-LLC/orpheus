# CLAUDE.md

This file provides guidance to Claude Code when working with the Orpheus music production platform.

## Project Overview

**Orpheus** is a unified music production platform integrating tablature editing, notation, recording, mixing, mastering, and AI-powered learning. Version 0.3.0 (Alpha) - ~95% production ready.

**Vision:** "From first chord to final master - one application, infinite possibilities"

**Tech Stack:**
- **Frontend:** React 18, TypeScript 5.7, Vite 5.1, Fluent UI 9
- **Backend:** Spring Boot 3.2, Kotlin 1.9.20, Java 17
- **Audio Engine:** Rust + JUCE C++ (gRPC on port 50051)
- **Desktop:** Rust + egui
- **Database:** PostgreSQL 15+, Redis 7+

## Directory Structure

```
orpheus/
├── orpheus/                      # Main platform (NPM + Gradle)
│   ├── packages/                 # 8 TypeScript npm packages
│   │   ├── app-web/              # React/Fluent UI web app
│   │   ├── shared-types/         # Type definitions
│   │   ├── project-model/        # .maestro file I/O
│   │   ├── music-theory/         # 100+ chords, 20+ scales
│   │   ├── midi-utils/           # MIDI parsing/generation
│   │   ├── timeline-sync/        # Timeline synchronization
│   │   ├── audio-analysis/       # LUFS metering, FFT
│   │   └── guitar-pro-parser/    # GP3-7 file parser
│   ├── backend/                  # Spring Boot + Kotlin
│   └── ai/personas/              # 7 AI specialists
│
├── orpheus-audio/                # Rust audio implementation
│   └── crates/
│       ├── maestro-proto/        # gRPC definitions
│       ├── maestro-audio-server/ # gRPC server
│       └── maestro-juce-bridge/  # C++ FFI bridge
│
├── orpheus-desktop/              # Rust desktop app (egui)
│   └── crates/                   # 10 crates
│
├── orpheus-standalone/           # Complete integrated app
│   ├── frontend/packages/
│   ├── backend/
│   ├── audio-engine/
│   └── start-all.sh
│
└── orpheus-complete/             # Distribution wrapper
```

## Essential Commands

### NPM Packages

```bash
cd orpheus/packages/app-web

# Install and run
npm install
npm run dev              # Port 5176

# Build
npm run build
npm run build:all        # All packages

# Testing
npm test
npm run typecheck
npm run lint
```

### Backend (Maven + Kotlin)

```bash
cd orpheus/backend

# Build
./mvnw clean install

# Run (Port 8080)
./mvnw spring-boot:run

# Tests
./mvnw test

# Build without tests
./mvnw clean install -DskipTests
```

### Audio Engine (Rust)

```bash
cd orpheus-audio

# Build
cargo build --release

# Run gRPC server
cargo run --bin maestro-audio-server

# Tests
cargo test
```

### Desktop Application

```bash
cd orpheus-desktop

# Build and run
cargo build --release
cargo run
```

### Standalone Distribution

```bash
cd orpheus-standalone

# Full setup
./dev-setup.sh           # Install dependencies
./build-all.sh           # Build everything
./start-all.sh           # Start all services
./stop-all.sh            # Stop services
```

## Port Configuration

| Service | Port | Purpose |
|---------|------|---------|
| Frontend (dev) | 5176 | Vite dev server |
| Backend API | 8080 | Spring Boot REST |
| Audio gRPC | 50051 | Rust audio service |
| PostgreSQL | 5432 | Database |
| Redis | 6379 | Cache/queue |
| MinIO API | 9000 | Object storage |
| MinIO Console | 9001 | Storage UI |

## NPM Packages (@maestro-ai/*)

| Package | Purpose |
|---------|---------|
| shared-types | TypeScript definitions |
| project-model | .maestro file I/O |
| music-theory | 100+ chords, 20+ scales, progressions |
| midi-utils | MIDI parsing, quantization, humanization |
| timeline-sync | Musical time ↔ absolute time |
| audio-analysis | LUFS metering, peak detection, FFT |
| guitar-pro-parser | GP3-7 file parsing |
| app-web | React web application |

## Application Modes

1. **Compose Mode** - Tablature editing, chord library, MIDI playback
2. **Record Mode** - Multi-track audio/MIDI recording
3. **Mix Mode** - Professional mixing console, EQ, compression
4. **Master Mode** - LUFS metering, platform-specific loudness
5. **Practice Mode** - Speed trainer (0.5x-2.0x), loop sections
6. **Distribute Mode** - Multi-platform distribution

## AI Personas (7)

| Persona | Purpose |
|---------|---------|
| music-theory-tutor | Interactive education |
| music-composer-ai | Melody/chord generation |
| audio-production-tutor | Recording/mixing education |
| session-assistant-ai | Recording coordination |
| guitar-coach-ai | Technique analysis |
| mixing-engineer-ai | Mix analysis/suggestions |
| mastering-engineer-ai | Loudness optimization |

## Audio Engine Features

- **Latency:** <10ms at 512 samples
- **DSP:** 4-band parametric EQ, compressor, reverb, delay
- **Limiter:** Hard limiter at -0.3 dBFS
- **VST3:** Plugin infrastructure
- **Framework:** JUCE C++ with Rust FFI

## File Format

**.maestro** - JSON-based project file:
- Metadata (tempo, key, time signature)
- Composition (tablature, notation)
- Session (audio/MIDI tracks)
- Mixing (settings, snapshots)
- Mastering (chain, metrics)
- Practice (speed trainer)
- AI History

## Database

- **Type:** PostgreSQL 15+
- **Migrations:** Flyway (auto-run on startup)
- **Tables:** users, projects, tracks, collaborators

## Development Setup

### Prerequisites
- Node.js 18+
- Java 17+
- Rust 1.75+
- Docker & Docker Compose

### Quick Start

```bash
cd orpheus-standalone

# 1. Setup
./dev-setup.sh

# 2. Build
./build-all.sh

# 3. Start
./start-all.sh

# Access:
# Frontend: http://localhost:5176
# API: http://localhost:8080
# Swagger: http://localhost:8080/swagger-ui
# MinIO: http://localhost:9001 (minioadmin/minioadmin)
```

## Key Statistics

- ~21,500 lines of code
- 100+ files
- 8 TypeScript packages
- 7 AI specialists
- 6 production modes
- <10ms audio latency

## Troubleshooting

### Audio Service Issues

```bash
# Check gRPC server
grpcurl -plaintext localhost:50051 list

# Restart audio service
cargo run --bin maestro-audio-server
```

### Backend Issues

```bash
# Check health
curl http://localhost:8080/actuator/health

# Check database
psql -h localhost -U maestro -d maestro
```

### Frontend Issues

```bash
# Clean install
rm -rf node_modules
npm install

# Type check
npm run typecheck
```

## Skills

- `rust-build` - Build Rust projects
- `rust-test` - Run Rust tests
- `orpheus-track` - Manage music tracks
