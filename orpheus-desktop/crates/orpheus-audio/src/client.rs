//! Audio client for Maestro connection

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::{debug, info, warn};

/// Audio client for connecting to Maestro audio service
pub struct AudioClient {
    /// Server address
    address: String,
    /// Connection state
    connected: Arc<AtomicBool>,
    /// Current latency in samples
    latency: Arc<AtomicU32>,
    /// Tokio runtime for async operations
    runtime: Option<Runtime>,
}

impl AudioClient {
    /// Create a new audio client
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            connected: Arc::new(AtomicBool::new(false)),
            latency: Arc::new(AtomicU32::new(0)),
            runtime: None,
        }
    }

    /// Default address for Maestro server
    pub fn default_address() -> &'static str {
        "http://localhost:50051"
    }

    /// Connect to the audio service
    pub fn connect(&mut self) -> crate::Result<()> {
        info!("Connecting to audio service at {}", self.address);

        // Create runtime if needed
        if self.runtime.is_none() {
            self.runtime = Some(
                Runtime::new()
                    .map_err(|e| crate::Error::ConnectionFailed(e.to_string()))?,
            );
        }

        // TODO: Implement actual gRPC connection to Maestro
        // For now, just mark as connected for UI testing
        self.connected.store(true, Ordering::SeqCst);
        debug!("Audio service connection established");

        Ok(())
    }

    /// Disconnect from the audio service
    pub fn disconnect(&mut self) {
        self.connected.store(false, Ordering::SeqCst);
        info!("Disconnected from audio service");
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Get current latency in samples
    pub fn latency(&self) -> u32 {
        self.latency.load(Ordering::SeqCst)
    }

    /// Get current latency in milliseconds (assuming 48kHz)
    pub fn latency_ms(&self) -> f32 {
        let samples = self.latency.load(Ordering::SeqCst);
        samples as f32 / 48.0
    }
}

impl Default for AudioClient {
    fn default() -> Self {
        Self::new(Self::default_address())
    }
}

impl Drop for AudioClient {
    fn drop(&mut self) {
        if self.is_connected() {
            self.disconnect();
        }
    }
}
