/*!
 * Error types for the audio engine
 */

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioEngineError {
    #[error("Failed to create audio engine")]
    CreationFailed,

    #[error("Audio engine is null")]
    NullPointer,

    #[error("Invalid buffer parameters: {0}")]
    InvalidBuffer(String),

    #[error("Invalid parameter value: {0}")]
    InvalidParameter(String),

    #[error("Plugin loading failed: {0}")]
    PluginLoadFailed(String),

    #[error("Audio processing error: {0}")]
    ProcessingError(String),
}

pub type Result<T> = std::result::Result<T, AudioEngineError>;
