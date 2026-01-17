//! # Orpheus Audio
//!
//! Audio service client for connecting to the Maestro audio engine.

pub mod client;

pub use client::AudioClient;

/// Result type for audio operations
pub type Result<T> = std::result::Result<T, Error>;

/// Audio error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not connected to audio service")]
    NotConnected,

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Audio processing error: {0}")]
    ProcessingError(String),

    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
}
