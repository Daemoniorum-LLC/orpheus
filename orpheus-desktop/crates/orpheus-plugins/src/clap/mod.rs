//! CLAP Plugin Support
//!
//! Implementation of CLAP (CLever Audio Plugin) host functionality.
//! CLAP is an open, modern audio plugin format.
//!
//! ## Architecture
//!
//! - `ffi`: Raw C type definitions matching the CLAP specification
//! - `host`: Host-side interface that plugins interact with
//! - `plugin`: Plugin loading and instance management

pub mod ffi;
pub mod host;
pub mod plugin;

// Re-exports
pub use host::{ClapHost, ClapHostState, ClapInputEvents, ClapOutputEvents, ClapEvent};
pub use plugin::{ClapPluginLoader, ClapPluginInstance};
