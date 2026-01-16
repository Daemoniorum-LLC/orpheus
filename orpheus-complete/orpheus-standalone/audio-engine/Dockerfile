# Multi-stage build for Rust audio service
FROM rust:1.75 AS builder

# Install C++ compiler for JUCE bridge
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --bin maestro-audio-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libstdc++6 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/maestro-audio-server /app/maestro-audio-server

EXPOSE 50051

CMD ["/app/maestro-audio-server"]
