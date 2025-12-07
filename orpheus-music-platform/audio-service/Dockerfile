# Maestro Audio Service
# Multi-stage build: Rust gRPC bridge + Sigil DSP engine

# =============================================================================
# Stage 1: Build Sigil compiler and compile DSP
# =============================================================================
FROM rust:1.82-bookworm AS sigil-builder

# Install LLVM 18 for AOT compilation
RUN apt-get update && apt-get install -y \
    llvm-18 \
    clang-18 \
    libzstd-dev \
    libpolly-18-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Clone and build Sigil
WORKDIR /sigil
RUN git clone --depth 1 https://github.com/Daemoniorum-LLC/sigil-lang.git . \
    && cd parser \
    && cargo build --release

# Copy DSP source and compile to native binary
WORKDIR /app
COPY src/sigil/ ./sigil/

# Compile Sigil DSP to native code (AOT for maximum performance)
RUN /sigil/parser/target/release/sigil compile \
    ./sigil/main.sigil \
    -o ./sigil/maestro-dsp \
    --lto \
    || echo "AOT compilation not available, will use JIT"

# =============================================================================
# Stage 2: Build Rust gRPC bridge
# =============================================================================
FROM rust:1.82-bookworm AS rust-builder

RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy proto files
COPY proto/ ./proto/

# Copy Rust source
COPY src/grpc-bridge/ ./src/grpc-bridge/

# Build the gRPC bridge
WORKDIR /app/src/grpc-bridge
RUN cargo build --release

# =============================================================================
# Stage 3: Runtime image
# =============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies and grpcurl for health checks
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    # LLVM runtime for Sigil JIT (if AOT not available)
    libllvm18 \
    && rm -rf /var/lib/apt/lists/*

# Install grpcurl for health checks
RUN curl -sSL https://github.com/fullstorydev/grpcurl/releases/download/v1.9.1/grpcurl_1.9.1_linux_x86_64.tar.gz \
    | tar -xz -C /usr/local/bin grpcurl

WORKDIR /app

# Copy Sigil compiler (for JIT fallback)
COPY --from=sigil-builder /sigil/parser/target/release/sigil /usr/local/bin/sigil

# Copy compiled DSP (or source for JIT)
COPY --from=sigil-builder /app/sigil/ ./sigil/

# Copy gRPC bridge binary
COPY --from=rust-builder /app/src/grpc-bridge/target/release/maestro-audio-service /usr/local/bin/

# Environment configuration
ENV RUST_LOG=info
ENV GRPC_LISTEN_ADDR=0.0.0.0:50051
ENV SIGIL_BIN=/usr/local/bin/sigil
ENV SIGIL_DSP_PATH=/app/sigil/main.sigil

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD grpcurl -plaintext localhost:50051 maestro.audio.AudioProcessor/Ping || exit 1

EXPOSE 50051

ENTRYPOINT ["/usr/local/bin/maestro-audio-service"]
