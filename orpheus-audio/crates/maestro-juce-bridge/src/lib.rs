/*!
 * Maestro JUCE Bridge
 * Safe Rust FFI bindings to the JUCE C++ audio engine
 */

pub mod ffi;
pub mod engine;
pub mod error;

pub use engine::AudioEngine;
pub use error::{AudioEngineError, Result};

// Re-export types
pub use ffi::{EQBand, CompressorParams, ReverbParams, DelayParams};
