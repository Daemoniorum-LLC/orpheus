# Maestro Audio Service

High-performance audio processing service powered by **Sigil DSP** with a **Rust gRPC** interface.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Kubernetes / Docker                       │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Maestro Audio Service                      │ │
│  │  ┌──────────────────┐    ┌───────────────────────────┐ │ │
│  │  │   Rust gRPC      │    │      Sigil DSP Engine     │ │ │
│  │  │   Bridge         │◄──►│                           │ │ │
│  │  │                  │    │  • Biquad EQ (3-band)     │ │ │
│  │  │  • Proto server  │    │  • Compressor (soft knee) │ │ │
│  │  │  • JSON-RPC IPC  │    │  • Schroeder Reverb       │ │ │
│  │  │  • Stream mgmt   │    │  • Delay (feedback)       │ │ │
│  │  │                  │    │  • Brickwall Limiter      │ │ │
│  │  └────────┬─────────┘    │  • LUFS Analysis          │ │ │
│  │           │              └───────────────────────────┘ │ │
│  │           │ :50051 gRPC                                │ │
│  └───────────┼────────────────────────────────────────────┘ │
└──────────────┼──────────────────────────────────────────────┘
               │
    ┌──────────▼──────────┐
    │  Spring Boot        │
    │  (Kotlin Backend)   │
    └─────────────────────┘
```

## Why Sigil?

Sigil compiles to native code via LLVM, achieving **3.6x faster** performance than equivalent Rust implementations on audio DSP workloads. The polysynthetic language design is ideal for mathematical signal processing.

## Building

### Prerequisites

- Docker 24+
- Rust 1.82+ (for local development)
- LLVM 18 (for Sigil AOT compilation)

### Docker Build

```bash
docker build -t maestro-audio-service .
```

### Local Development

```bash
# Build Rust gRPC bridge
cd src/grpc-bridge
cargo build --release

# Run with Sigil JIT
SIGIL_DSP_PATH=../sigil/main.sigil cargo run --release
```

## Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `GRPC_LISTEN_ADDR` | `0.0.0.0:50051` | gRPC server bind address |
| `SIGIL_BIN` | `sigil` | Path to Sigil binary |
| `SIGIL_DSP_PATH` | `/app/sigil/main.sigil` | Path to DSP source |
| `RUST_LOG` | `info` | Log level |

## gRPC API

### ProcessAudio

Process a single audio buffer through the effects chain.

```protobuf
rpc ProcessAudio(AudioRequest) returns (AudioResponse);
```

### StreamAudio

Bidirectional streaming for real-time audio processing.

```protobuf
rpc StreamAudio(stream AudioBuffer) returns (stream AudioBuffer);
```

### UpdateEffect

Update effect parameters in real-time.

```protobuf
rpc UpdateEffect(EffectUpdate) returns (EffectResponse);
```

### AnalyzeAudio

Analyze audio (spectrum, loudness, peak, RMS).

```protobuf
rpc AnalyzeAudio(AnalyzeRequest) returns (AnalyzeResponse);
```

## DSP Effects

### 3-Band Parametric EQ

- **Low shelf**: 200 Hz, adjustable gain
- **Mid peak**: 1000 Hz, adjustable gain and Q
- **High shelf**: 4000 Hz, adjustable gain

### Compressor

- Threshold: -60 to 0 dB
- Ratio: 1:1 to 20:1
- Attack: 0.1 to 100 ms
- Release: 1 to 1000 ms
- Soft knee: 0 to 12 dB
- Makeup gain

### Reverb (Schroeder)

- 4 parallel comb filters
- 2 series allpass filters
- Wet/dry mix
- Room size control

### Delay

- Time: 1 ms to 2000 ms
- Feedback: 0 to 95%
- Wet/dry mix

### Limiter (Brickwall)

- 5ms lookahead
- Ceiling: adjustable (default -0.3 dBFS)
- Fast release

## Performance

| Metric | Value |
|--------|-------|
| Latency (512 samples @ 48kHz) | < 2ms |
| CPU usage (single track) | < 5% |
| Memory footprint | < 50MB |
| Throughput | 200+ concurrent streams |

## License

MIT - Daemoniorum LLC
