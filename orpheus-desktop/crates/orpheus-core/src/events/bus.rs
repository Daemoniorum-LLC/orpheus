//! Event bus for pub/sub communication

use super::AppEvent;
use crossbeam_channel::{unbounded, Receiver, Sender};
use parking_lot::RwLock;
use std::sync::Arc;

/// Event bus for application-wide event distribution
#[derive(Clone)]
pub struct EventBus {
    sender: Sender<AppEvent>,
    receiver: Receiver<AppEvent>,
    /// Pending events for current frame
    pending: Arc<RwLock<Vec<AppEvent>>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            sender,
            receiver,
            pending: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Emit an event
    pub fn emit(&self, event: AppEvent) {
        // Store in pending for same-frame access
        self.pending.write().push(event.clone());
        // Also send through channel for async handlers
        let _ = self.sender.send(event);
    }

    /// Emit multiple events
    pub fn emit_all(&self, events: impl IntoIterator<Item = AppEvent>) {
        for event in events {
            self.emit(event);
        }
    }

    /// Drain and return all pending events
    pub fn drain(&self) -> Vec<AppEvent> {
        let mut pending = self.pending.write();
        std::mem::take(&mut *pending)
    }

    /// Poll for events from channel (for async handlers)
    pub fn try_recv(&self) -> Option<AppEvent> {
        self.receiver.try_recv().ok()
    }

    /// Get a clone of the sender for external use
    pub fn sender(&self) -> Sender<AppEvent> {
        self.sender.clone()
    }

    /// Get a clone of the receiver for external use
    pub fn receiver(&self) -> Receiver<AppEvent> {
        self.receiver.clone()
    }

    /// Check if there are pending events
    pub fn has_pending(&self) -> bool {
        !self.pending.read().is_empty()
    }
}
