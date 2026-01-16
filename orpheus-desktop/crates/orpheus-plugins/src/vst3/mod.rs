//! VST3 Plugin Support
//!
//! Implementation of VST3 (Virtual Studio Technology 3) host functionality.
//! VST3 is Steinberg's plugin format using COM interfaces.
//!
//! ## Architecture
//!
//! - `com`: COM foundation (IUnknown, GUID/TUID, reference counting)
//! - `ffi`: Raw C type definitions from VST3 SDK
//! - `host`: Host-side interface implementation
//! - `plugin`: Plugin loading and instance management

pub mod com;
pub mod ffi;
pub mod host;
pub mod plugin;

// Re-exports
pub use com::{ComPtr, TUID, tresult, K_RESULT_OK, K_NO_INTERFACE};
pub use ffi::{ProcessData, ProcessSetup, AudioBusBuffers, ProcessContext};
pub use host::{Vst3Host, Vst3HostState, Vst3ComponentHandler, AudioBufferManager};
pub use plugin::{Vst3PluginLoader, Vst3PluginInstance, FactoryInfo, ClassInfo};
