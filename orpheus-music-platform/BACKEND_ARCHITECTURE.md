# Maestro Backend Architecture

## Overview

Maestro uses a **hybrid microservices architecture** combining Spring Boot (web API) and Rust (performance-critical services) to provide both accessibility and ultimate performance.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CLIENT LAYER                                 │
├─────────────────────┬───────────────────────┬───────────────────────┤
│   Web App (React)   │   Desktop (Rust+egui) │   Mobile (Flutter)    │
│   - Vite + TS       │   - 60 FPS GUI        │   - iOS/Android       │
│   - Fluent UI       │   - Native Audio      │   - Touch optimized   │
│   - Tone.js         │   - VST hosting       │   - Cloud sync        │
└──────────┬──────────┴───────────┬───────────┴───────────────────────┘
           │                      │
           │ REST/WebSocket       │ gRPC/QUIC
           │                      │
┌──────────▼──────────────────────▼───────────────────────────────────┐
│                      SERVICE LAYER                                   │
├──────────────────────────────────┬───────────────────────────────────┤
│   maestro-backend (Spring Boot)  │  maestro-audio (Rust)             │
│   ================================│  =================================│
│   - REST API                     │  - Real-time audio processing     │
│   - WebSocket (live updates)     │  - JUCE FFI bridge                │
│   - PostgreSQL (projects, users) │  - VST plugin hosting             │
│   - S3 (audio files, artwork)    │  - Low-latency streaming          │
│   - Redis (sessions, cache)      │  - gRPC server                    │
│   - Leviathan AI integration     │  - QUIC transport                 │
│   - Authentication/Authorization │  - tokio async runtime            │
│   - File upload/download         │  - Zero-copy audio buffers        │
└──────────────────────────────────┴───────────────────────────────────┘
           │                                  │
           │                                  │
┌──────────▼──────────────────────────────────▼───────────────────────┐
│                      DATA LAYER                                      │
├──────────────────────┬───────────────────────┬───────────────────────┤
│   PostgreSQL         │   Redis               │   S3 / MinIO          │
│   - Projects         │   - Sessions          │   - Audio files       │
│   - Tracks           │   - Cache             │   - Artwork           │
│   - Users            │   - Real-time state   │   - Exports           │
│   - Collaborators    │   - Pub/Sub           │   - Backups           │
└──────────────────────┴───────────────────────┴───────────────────────┘
```

## 1. maestro-backend (Spring Boot)

### Technology Stack
- **Framework**: Spring Boot 3.2+
- **Language**: Kotlin (for null safety and conciseness)
- **Database**: PostgreSQL 15+ with JPA/Hibernate
- **Cache**: Redis 7+ with Spring Data Redis
- **Storage**: AWS S3 / MinIO (S3-compatible)
- **Security**: Spring Security + JWT
- **API**: REST + WebSocket
- **Documentation**: OpenAPI 3.0 (Springdoc)
- **Monitoring**: Spring Boot Actuator + Micrometer

### Module Structure
```
maestro-backend/
├── src/main/kotlin/ai/maestro/backend/
│   ├── config/          # Spring configuration
│   │   ├── SecurityConfig.kt
│   │   ├── WebSocketConfig.kt
│   │   ├── S3Config.kt
│   │   └── RedisConfig.kt
│   ├── controller/      # REST endpoints
│   │   ├── ProjectController.kt
│   │   ├── TrackController.kt
│   │   ├── UserController.kt
│   │   ├── CollaborationController.kt
│   │   └── DistributionController.kt
│   ├── service/         # Business logic
│   │   ├── ProjectService.kt
│   │   ├── TrackService.kt
│   │   ├── AudioStorageService.kt
│   │   ├── LeviathanService.kt  # AI integration
│   │   └── AudioProcessingService.kt  # gRPC client to Rust service
│   ├── repository/      # JPA repositories
│   │   ├── ProjectRepository.kt
│   │   ├── TrackRepository.kt
│   │   └── UserRepository.kt
│   ├── model/           # Domain entities
│   │   ├── Project.kt
│   │   ├── Track.kt
│   │   ├── User.kt
│   │   └── Collaboration.kt
│   └── websocket/       # WebSocket handlers
│       ├── LiveEditHandler.kt
│       └── CollaborationHandler.kt
├── src/main/resources/
│   ├── application.yml
│   ├── db/migration/    # Flyway migrations
│   │   ├── V1__initial_schema.sql
│   │   ├── V2__add_collaboration.sql
│   │   └── V3__add_distribution.sql
│   └── openapi.yml
└── build.gradle.kts
```

### Key API Endpoints

#### Projects
```
POST   /api/v1/projects                    # Create project
GET    /api/v1/projects                    # List projects
GET    /api/v1/projects/{id}               # Get project
PUT    /api/v1/projects/{id}               # Update project
DELETE /api/v1/projects/{id}               # Delete project
POST   /api/v1/projects/{id}/import        # Import Guitar Pro file
GET    /api/v1/projects/{id}/export        # Export project
```

#### Tracks
```
POST   /api/v1/projects/{id}/tracks        # Add track
PUT    /api/v1/tracks/{id}                 # Update track
DELETE /api/v1/tracks/{id}                 # Delete track
POST   /api/v1/tracks/{id}/audio           # Upload audio
GET    /api/v1/tracks/{id}/audio           # Download audio
POST   /api/v1/tracks/{id}/process         # Process with Rust service
```

#### AI Integration
```
POST   /api/v1/ai/analyze-tab              # Analyze tablature
POST   /api/v1/ai/suggest-chords           # Suggest chord progression
POST   /api/v1/ai/generate-solo            # Generate guitar solo
POST   /api/v1/ai/auto-master              # Auto-master track
```

#### Collaboration
```
POST   /api/v1/projects/{id}/collaborators # Add collaborator
WS     /ws/projects/{id}/live              # Live editing session
```

### Database Schema (PostgreSQL)

```sql
-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Projects
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    artist VARCHAR(255),
    bpm INTEGER NOT NULL DEFAULT 120,
    time_signature VARCHAR(10) NOT NULL DEFAULT '4/4',
    key VARCHAR(10),
    metadata JSONB,  -- Flexible metadata storage
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Tracks
CREATE TABLE tracks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    instrument VARCHAR(100) NOT NULL,
    track_number INTEGER NOT NULL,
    tablature JSONB,  -- alphaTab format
    audio_file_url VARCHAR(500),  -- S3 URL
    processors JSONB,  -- EQ, compressor, effects settings
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Collaborators
CREATE TABLE collaborators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL,  -- owner, editor, viewer
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(project_id, user_id)
);

-- Distribution
CREATE TABLE distributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    artist VARCHAR(255) NOT NULL,
    album VARCHAR(255),
    genre VARCHAR(100),
    isrc VARCHAR(12),
    upc VARCHAR(13),
    artwork_url VARCHAR(500),  -- S3 URL
    platforms JSONB,  -- Array of platforms
    status VARCHAR(50) NOT NULL,  -- draft, submitted, published
    submitted_at TIMESTAMP,
    published_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_projects_user_id ON projects(user_id);
CREATE INDEX idx_tracks_project_id ON tracks(project_id);
CREATE INDEX idx_collaborators_project_id ON collaborators(project_id);
CREATE INDEX idx_collaborators_user_id ON collaborators(user_id);
```

## 2. maestro-audio (Rust)

Following the **Eidolon workspace pattern** for modular architecture.

### Technology Stack
- **Language**: Rust (stable)
- **Runtime**: tokio (async I/O)
- **GUI**: egui (immediate-mode, 60 FPS)
- **Audio**: JUCE (C++) via FFI
- **IPC**: gRPC + Protocol Buffers
- **Transport**: QUIC (for low-latency)
- **Plugins**: VST3 SDK

### Workspace Structure
```
maestro-audio/
├── Cargo.toml                    # Workspace definition
├── crates/
│   ├── maestro-proto/            # Protocol Buffers definitions
│   │   ├── Cargo.toml
│   │   ├── build.rs              # Protobuf compilation
│   │   └── src/
│   │       ├── lib.rs
│   │       └── audio.proto       # Audio service protocol
│   │
│   ├── maestro-audio-server/     # gRPC audio processing server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # Server entry point
│   │       ├── service.rs        # gRPC service implementation
│   │       ├── processor.rs      # Audio processing engine
│   │       └── juce_bridge.rs    # FFI to JUCE
│   │
│   ├── maestro-juce-bridge/      # Rust FFI to JUCE C++
│   │   ├── Cargo.toml
│   │   ├── build.rs              # C++ compilation
│   │   ├── src/
│   │   │   ├── lib.rs            # Rust FFI bindings
│   │   │   └── bridge.rs         # Safe Rust wrappers
│   │   └── cpp/
│   │       ├── AudioEngine.h
│   │       ├── AudioEngine.cpp
│   │       ├── PluginHost.h
│   │       └── PluginHost.cpp
│   │
│   ├── maestro-desktop/          # Native desktop application
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # GUI entry point
│   │       ├── app.rs            # egui application
│   │       ├── ui/               # UI modules
│   │       │   ├── mod.rs
│   │       │   ├── mixer.rs
│   │       │   ├── editor.rs
│   │       │   └── effects.rs
│   │       └── client.rs         # gRPC client
│   │
│   └── maestro-audio-common/     # Shared audio utilities
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── buffer.rs         # Zero-copy audio buffers
│           ├── format.rs         # Audio format conversion
│           └── utils.rs
│
└── juce/                         # JUCE framework (git submodule)
    └── modules/
```

### Protocol Buffers (maestro-proto/src/audio.proto)

```protobuf
syntax = "proto3";

package maestro.audio;

// Audio processing service
service AudioProcessor {
  // Process audio buffer with effects chain
  rpc ProcessAudio(AudioBuffer) returns (AudioBuffer);

  // Real-time streaming processing
  rpc StreamAudio(stream AudioBuffer) returns (stream AudioBuffer);

  // Load VST plugin
  rpc LoadPlugin(PluginRequest) returns (PluginResponse);

  // Update effect parameters
  rpc UpdateEffect(EffectUpdate) returns (EffectResponse);
}

// Audio buffer (zero-copy with shared memory)
message AudioBuffer {
  int32 sample_rate = 1;
  int32 num_channels = 2;
  int32 num_samples = 3;
  bytes audio_data = 4;  // Interleaved float32
  int64 timestamp_us = 5;
}

// Plugin request
message PluginRequest {
  string plugin_path = 1;
  int32 sample_rate = 2;
  int32 buffer_size = 3;
}

// Effect update
message EffectUpdate {
  string effect_id = 1;
  string parameter_id = 2;
  float value = 3;
}

// EQ settings
message EQSettings {
  repeated EQBand bands = 1;
}

message EQBand {
  float frequency = 1;
  float gain = 2;
  float q = 3;
}

// Compressor settings
message CompressorSettings {
  float threshold = 1;
  float ratio = 2;
  float attack_ms = 3;
  float release_ms = 4;
  float knee = 5;
  float makeup_gain = 6;
}
```

### JUCE C++ Audio Engine (maestro-juce-bridge/cpp/AudioEngine.h)

```cpp
#pragma once
#include <JuceHeader.h>
#include <vector>
#include <memory>

namespace maestro {

// Audio buffer wrapper for FFI
struct AudioBufferFFI {
    float* data;
    int num_channels;
    int num_samples;
    int sample_rate;
};

// Audio engine for processing
class AudioEngine {
public:
    AudioEngine(int sample_rate, int buffer_size);
    ~AudioEngine();

    // Process audio buffer
    void process(AudioBufferFFI* input, AudioBufferFFI* output);

    // Load VST plugin
    bool loadPlugin(const char* plugin_path);

    // Update effect parameters
    void setParameter(const char* effect_id, const char* param_id, float value);

    // EQ
    void setEQBand(int band, float freq, float gain, float q);

    // Compressor
    void setCompressor(float threshold, float ratio, float attack, float release);

private:
    int sample_rate_;
    int buffer_size_;

    // JUCE audio graph
    std::unique_ptr<juce::AudioProcessorGraph> audio_graph_;

    // Built-in effects
    std::unique_ptr<juce::dsp::ProcessorChain<
        juce::dsp::IIR::Filter<float>,  // EQ
        juce::dsp::Compressor<float>,   // Compressor
        juce::dsp::Reverb               // Reverb
    >> fx_chain_;

    // VST plugin host
    juce::AudioPluginFormatManager plugin_manager_;
    std::vector<std::unique_ptr<juce::AudioPluginInstance>> plugins_;
};

} // namespace maestro

// C FFI exports
extern "C" {
    void* maestro_audio_engine_new(int sample_rate, int buffer_size);
    void maestro_audio_engine_delete(void* engine);
    void maestro_audio_engine_process(void* engine, AudioBufferFFI* input, AudioBufferFFI* output);
    bool maestro_audio_engine_load_plugin(void* engine, const char* path);
    void maestro_audio_engine_set_eq_band(void* engine, int band, float freq, float gain, float q);
}
```

### Rust FFI Bridge (maestro-juce-bridge/src/lib.rs)

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[repr(C)]
pub struct AudioBufferFFI {
    pub data: *mut f32,
    pub num_channels: i32,
    pub num_samples: i32,
    pub sample_rate: i32,
}

extern "C" {
    fn maestro_audio_engine_new(sample_rate: i32, buffer_size: i32) -> *mut std::ffi::c_void;
    fn maestro_audio_engine_delete(engine: *mut std::ffi::c_void);
    fn maestro_audio_engine_process(
        engine: *mut std::ffi::c_void,
        input: *mut AudioBufferFFI,
        output: *mut AudioBufferFFI,
    );
    fn maestro_audio_engine_load_plugin(
        engine: *mut std::ffi::c_void,
        path: *const c_char,
    ) -> bool;
    fn maestro_audio_engine_set_eq_band(
        engine: *mut std::ffi::c_void,
        band: i32,
        freq: f32,
        gain: f32,
        q: f32,
    );
}

/// Safe Rust wrapper for JUCE audio engine
pub struct AudioEngine {
    engine: *mut std::ffi::c_void,
}

impl AudioEngine {
    pub fn new(sample_rate: i32, buffer_size: i32) -> Self {
        unsafe {
            let engine = maestro_audio_engine_new(sample_rate, buffer_size);
            assert!(!engine.is_null(), "Failed to create audio engine");
            Self { engine }
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], num_channels: i32) {
        let num_samples = (input.len() / num_channels as usize) as i32;

        let mut input_ffi = AudioBufferFFI {
            data: input.as_ptr() as *mut f32,
            num_channels,
            num_samples,
            sample_rate: 48000,
        };

        let mut output_ffi = AudioBufferFFI {
            data: output.as_mut_ptr(),
            num_channels,
            num_samples,
            sample_rate: 48000,
        };

        unsafe {
            maestro_audio_engine_process(self.engine, &mut input_ffi, &mut output_ffi);
        }
    }

    pub fn load_plugin(&mut self, path: &str) -> bool {
        let c_path = CString::new(path).unwrap();
        unsafe { maestro_audio_engine_load_plugin(self.engine, c_path.as_ptr()) }
    }

    pub fn set_eq_band(&mut self, band: i32, freq: f32, gain: f32, q: f32) {
        unsafe {
            maestro_audio_engine_set_eq_band(self.engine, band, freq, gain, q);
        }
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        unsafe {
            maestro_audio_engine_delete(self.engine);
        }
    }
}

// Safety: AudioEngine is Send because the JUCE engine is thread-safe
unsafe impl Send for AudioEngine {}
```

### gRPC Audio Service (maestro-audio-server/src/service.rs)

```rust
use maestro_proto::audio::{
    audio_processor_server::{AudioProcessor, AudioProcessorServer},
    AudioBuffer, EffectUpdate, PluginRequest, PluginResponse,
};
use maestro_juce_bridge::AudioEngine;
use tonic::{Request, Response, Status};
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct AudioProcessorService {
    engine: Arc<RwLock<AudioEngine>>,
}

impl AudioProcessorService {
    pub fn new(sample_rate: i32, buffer_size: i32) -> Self {
        Self {
            engine: Arc::new(RwLock::new(AudioEngine::new(sample_rate, buffer_size))),
        }
    }
}

#[tonic::async_trait]
impl AudioProcessor for AudioProcessorService {
    async fn process_audio(
        &self,
        request: Request<AudioBuffer>,
    ) -> Result<Response<AudioBuffer>, Status> {
        let input = request.into_inner();

        // Convert bytes to f32 samples
        let input_samples: Vec<f32> = input
            .audio_data
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let mut output_samples = vec![0.0f32; input_samples.len()];

        // Process through JUCE engine
        let mut engine = self.engine.write().await;
        engine.process(&input_samples, &mut output_samples, input.num_channels);

        // Convert back to bytes
        let output_data: Vec<u8> = output_samples
            .iter()
            .flat_map(|&sample| sample.to_le_bytes())
            .collect();

        let output = AudioBuffer {
            sample_rate: input.sample_rate,
            num_channels: input.num_channels,
            num_samples: input.num_samples,
            audio_data: output_data,
            timestamp_us: input.timestamp_us,
        };

        Ok(Response::new(output))
    }

    async fn load_plugin(
        &self,
        request: Request<PluginRequest>,
    ) -> Result<Response<PluginResponse>, Status> {
        let req = request.into_inner();
        let mut engine = self.engine.write().await;

        let success = engine.load_plugin(&req.plugin_path);

        Ok(Response::new(PluginResponse {
            success,
            plugin_id: if success { Some(req.plugin_path) } else { None },
            error: if !success { Some("Failed to load plugin".to_string()) } else { None },
        }))
    }

    async fn update_effect(
        &self,
        request: Request<EffectUpdate>,
    ) -> Result<Response<maestro_proto::audio::EffectResponse>, Status> {
        let update = request.into_inner();
        let mut engine = self.engine.write().await;

        // Parse effect type and update
        if update.effect_id.starts_with("eq") {
            // Handle EQ parameter update
            // ... implementation
        }

        Ok(Response::new(maestro_proto::audio::EffectResponse { success: true }))
    }
}
```

### Desktop GUI (maestro-desktop/src/app.rs)

```rust
use eframe::egui;
use maestro_proto::audio::audio_processor_client::AudioProcessorClient;
use tonic::transport::Channel;

pub struct MaestroApp {
    audio_client: AudioProcessorClient<Channel>,

    // UI state
    current_bpm: f32,
    is_playing: bool,
    current_track: Option<String>,

    // Mix state
    track_volumes: Vec<f32>,
    track_pans: Vec<f32>,

    // Effect state
    eq_bands: Vec<(f32, f32, f32)>,  // freq, gain, q
}

impl MaestroApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Connect to audio service
        let audio_client = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                AudioProcessorClient::connect("http://localhost:50051")
                    .await
                    .expect("Failed to connect to audio service")
            });

        Self {
            audio_client,
            current_bpm: 120.0,
            is_playing: false,
            current_track: None,
            track_volumes: vec![0.75; 8],
            track_pans: vec![0.5; 8],
            eq_bands: vec![
                (100.0, 0.0, 1.0),
                (400.0, 0.0, 1.0),
                (2000.0, 0.0, 1.0),
                (8000.0, 0.0, 1.0),
            ],
        }
    }
}

impl eframe::App for MaestroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request 60 FPS rendering
        ctx.request_repaint();

        // Top panel - Transport controls
        egui::TopBottomPanel::top("transport").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎸 Maestro");
                ui.separator();

                if ui.button(if self.is_playing { "⏸" } else { "▶" }).clicked() {
                    self.is_playing = !self.is_playing;
                }

                ui.label("BPM:");
                ui.add(egui::Slider::new(&mut self.current_bpm, 40.0..=240.0));
            });
        });

        // Left panel - Track list
        egui::SidePanel::left("tracks").min_width(200.0).show(ctx, |ui| {
            ui.heading("Tracks");
            ui.separator();

            for i in 0..8 {
                ui.horizontal(|ui| {
                    ui.label(format!("Track {}", i + 1));
                    if ui.small_button("S").clicked() {
                        // Solo
                    }
                    if ui.small_button("M").clicked() {
                        // Mute
                    }
                });
            }
        });

        // Right panel - Effects
        egui::SidePanel::right("effects").min_width(300.0).show(ctx, |ui| {
            ui.heading("Effects");
            ui.separator();

            ui.label("Parametric EQ");
            for (i, (freq, gain, q)) in self.eq_bands.iter_mut().enumerate() {
                ui.group(|ui| {
                    ui.label(format!("Band {}", i + 1));
                    ui.add(egui::Slider::new(freq, 20.0..=20000.0).text("Freq (Hz)").logarithmic(true));
                    ui.add(egui::Slider::new(gain, -12.0..=12.0).text("Gain (dB)"));
                    ui.add(egui::Slider::new(q, 0.1..=10.0).text("Q"));
                });
            }
        });

        // Central panel - Mixer
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mixer");
            ui.separator();

            ui.horizontal(|ui| {
                for i in 0..8 {
                    ui.vertical(|ui| {
                        ui.label(format!("Track {}", i + 1));

                        // Volume fader (vertical)
                        ui.add_sized(
                            [40.0, 200.0],
                            egui::Slider::new(&mut self.track_volumes[i], 0.0..=1.0)
                                .vertical()
                                .show_value(false)
                        );

                        ui.label(format!("{:.0}%", self.track_volumes[i] * 100.0));

                        // Pan knob
                        ui.add(
                            egui::Slider::new(&mut self.track_pans[i], 0.0..=1.0)
                                .text("Pan")
                        );
                    });
                }
            });
        });
    }
}
```

## 3. Integration Architecture

### Web App → Spring Boot → Rust Audio Service

```
┌─────────────────┐
│   React Web App │
│   (Port 5173)   │
└────────┬────────┘
         │ HTTP REST
         │ WebSocket
         ▼
┌─────────────────┐      ┌──────────────────┐
│  Spring Boot    │      │   Leviathan      │
│  maestro-backend│◄─────┤   AI Service     │
│  (Port 8080)    │ HTTP │   (Port 8081)    │
└────────┬────────┘      └──────────────────┘
         │ gRPC
         │ (Port 50051)
         ▼
┌─────────────────┐      ┌──────────────────┐
│  Rust Audio     │      │   PostgreSQL     │
│  Service        │      │   (Port 5432)    │
│  (Port 50051)   │      └──────────────────┘
└────────┬────────┘
         │ FFI
         ▼
┌─────────────────┐
│  JUCE C++       │
│  Audio Engine   │
└─────────────────┘
```

### Desktop App → Direct gRPC → Rust Audio Service

```
┌─────────────────┐
│  Rust Desktop   │
│  egui GUI       │
│  (Native)       │
└────────┬────────┘
         │ gRPC (QUIC)
         │ (Port 50051)
         ▼
┌─────────────────┐
│  Rust Audio     │
│  Service        │
│  (Port 50051)   │
└────────┬────────┘
         │ FFI
         ▼
┌─────────────────┐
│  JUCE C++       │
│  Audio Engine   │
│  + VST Hosting  │
└─────────────────┘
```

## 4. Deployment Architecture

### Development
- Docker Compose for local development
- Hot reload for all services
- Mock S3 with MinIO

### Production
- Kubernetes (EKS/GKE)
- PostgreSQL RDS
- Redis ElastiCache
- S3 for storage
- CloudFront CDN
- Load balancer for Spring Boot
- QUIC load balancer for Rust audio service

## 5. Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Audio latency | < 10ms | JUCE + Rust FFI |
| API response | < 100ms | Spring Boot |
| WebSocket latency | < 50ms | Real-time collaboration |
| Audio processing | 60 FPS | egui rendering |
| Concurrent users | 10,000+ | Horizontal scaling |
| Audio buffer size | 256-512 samples | Configurable |

## Next Steps

1. ✅ Design architecture
2. ⏳ Implement Spring Boot backend
3. ⏳ Implement Rust audio service
4. ⏳ Implement JUCE C++ engine
5. ⏳ Implement desktop GUI
6. ⏳ Integration testing
7. ⏳ Deployment configuration
