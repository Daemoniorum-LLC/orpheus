//! CLAP Host Implementation
//!
//! Provides the host-side interface for CLAP plugins.

use super::ffi::*;
use crate::HostConfig;
use std::ffi::{c_void, CString};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::debug;

/// Host state shared between callbacks
pub struct ClapHostState {
    /// Host configuration
    pub config: HostConfig,
    /// Request to restart the plugin
    pub restart_requested: AtomicBool,
    /// Request to continue processing
    pub process_requested: AtomicBool,
    /// Request for main thread callback
    pub callback_requested: AtomicBool,
}

impl ClapHostState {
    /// Create new host state
    pub fn new(config: HostConfig) -> Self {
        Self {
            config,
            restart_requested: AtomicBool::new(false),
            process_requested: AtomicBool::new(false),
            callback_requested: AtomicBool::new(false),
        }
    }

    /// Check and clear restart request
    pub fn take_restart_request(&self) -> bool {
        self.restart_requested.swap(false, Ordering::SeqCst)
    }

    /// Check and clear process request
    pub fn take_process_request(&self) -> bool {
        self.process_requested.swap(false, Ordering::SeqCst)
    }

    /// Check and clear callback request
    pub fn take_callback_request(&self) -> bool {
        self.callback_requested.swap(false, Ordering::SeqCst)
    }
}

/// CLAP host wrapper
pub struct ClapHost {
    /// The raw clap_host struct (must be kept alive)
    raw: Box<clap_host>,
    /// Host state
    state: Arc<ClapHostState>,
    /// Host name (kept alive for pointer)
    _name: CString,
    /// Host vendor (kept alive for pointer)
    _vendor: CString,
    /// Host URL (kept alive for pointer)
    _url: CString,
    /// Host version (kept alive for pointer)
    _version: CString,
}

impl ClapHost {
    /// Create a new CLAP host
    pub fn new(config: HostConfig) -> Self {
        let state = Arc::new(ClapHostState::new(config.clone()));

        let name = CString::new(config.host_name.as_str()).unwrap_or_default();
        let vendor = CString::new(config.host_vendor.as_str()).unwrap_or_default();
        let url = CString::new("https://orpheus.daemoniorum.com").unwrap_or_default();
        let version = CString::new(config.host_version.as_str()).unwrap_or_default();

        // Create the raw host struct
        let state_ptr = Arc::into_raw(state.clone()) as *mut c_void;

        let raw = Box::new(clap_host {
            clap_version: CLAP_VERSION,
            host_data: state_ptr,
            name: name.as_ptr(),
            vendor: vendor.as_ptr(),
            url: url.as_ptr(),
            version: version.as_ptr(),
            request_restart: Some(Self::request_restart_callback),
            request_process: Some(Self::request_process_callback),
            request_callback: Some(Self::request_callback_callback),
        });

        // Reconstruct the Arc (we still own it)
        let _ = unsafe { Arc::from_raw(state_ptr as *const ClapHostState) };

        Self {
            raw,
            state,
            _name: name,
            _vendor: vendor,
            _url: url,
            _version: version,
        }
    }

    /// Get raw pointer to clap_host (for passing to plugins)
    pub fn as_ptr(&self) -> *const clap_host {
        self.raw.as_ref() as *const clap_host
    }

    /// Get reference to host state
    pub fn state(&self) -> &Arc<ClapHostState> {
        &self.state
    }

    // Callback implementations

    unsafe extern "C" fn request_restart_callback(host: *const clap_host) {
        if let Some(state) = Self::get_state(host) {
            debug!("CLAP plugin requested restart");
            state.restart_requested.store(true, Ordering::SeqCst);
        }
    }

    unsafe extern "C" fn request_process_callback(host: *const clap_host) {
        if let Some(state) = Self::get_state(host) {
            state.process_requested.store(true, Ordering::SeqCst);
        }
    }

    unsafe extern "C" fn request_callback_callback(host: *const clap_host) {
        if let Some(state) = Self::get_state(host) {
            debug!("CLAP plugin requested main thread callback");
            state.callback_requested.store(true, Ordering::SeqCst);
        }
    }

    /// Get state from host pointer
    unsafe fn get_state(host: *const clap_host) -> Option<&'static ClapHostState> {
        if host.is_null() {
            return None;
        }
        let host_data = (*host).host_data;
        if host_data.is_null() {
            return None;
        }
        Some(&*(host_data as *const ClapHostState))
    }
}

impl Drop for ClapHost {
    fn drop(&mut self) {
        // The state Arc will be dropped automatically
    }
}

/// Event list for input events
pub struct ClapInputEvents {
    events: Vec<ClapEvent>,
    raw: clap_input_events,
}

/// Wrapped CLAP event
#[derive(Clone)]
pub enum ClapEvent {
    NoteOn {
        time: u32,
        channel: i16,
        key: i16,
        velocity: f64,
        note_id: i32,
    },
    NoteOff {
        time: u32,
        channel: i16,
        key: i16,
        velocity: f64,
        note_id: i32,
    },
    ParamValue {
        time: u32,
        param_id: u32,
        value: f64,
    },
    Midi {
        time: u32,
        port: u16,
        data: [u8; 3],
    },
}

impl ClapInputEvents {
    /// Create new empty event list
    pub fn new() -> Self {
        let mut events = Self {
            events: Vec::new(),
            raw: clap_input_events {
                ctx: ptr::null_mut(),
                size: Some(Self::size_callback),
                get: Some(Self::get_callback),
            },
        };
        events.raw.ctx = &mut events.events as *mut _ as *mut c_void;
        events
    }

    /// Add a note on event
    pub fn push_note_on(&mut self, time: u32, channel: i16, key: i16, velocity: f64) {
        self.events.push(ClapEvent::NoteOn {
            time,
            channel,
            key,
            velocity,
            note_id: -1,
        });
    }

    /// Add a note off event
    pub fn push_note_off(&mut self, time: u32, channel: i16, key: i16, velocity: f64) {
        self.events.push(ClapEvent::NoteOff {
            time,
            channel,
            key,
            velocity,
            note_id: -1,
        });
    }

    /// Add a parameter value event
    pub fn push_param_value(&mut self, time: u32, param_id: u32, value: f64) {
        self.events.push(ClapEvent::ParamValue {
            time,
            param_id,
            value,
        });
    }

    /// Add a MIDI event
    pub fn push_midi(&mut self, time: u32, port: u16, data: [u8; 3]) {
        self.events.push(ClapEvent::Midi { time, port, data });
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get raw pointer
    pub fn as_ptr(&self) -> *const clap_input_events {
        &self.raw as *const _
    }

    /// Number of events
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    // Callbacks - these work with raw event storage

    unsafe extern "C" fn size_callback(list: *const clap_input_events) -> u32 {
        if list.is_null() {
            return 0;
        }
        let ctx = (*list).ctx;
        if ctx.is_null() {
            return 0;
        }
        let events = &*(ctx as *const Vec<ClapEvent>);
        events.len() as u32
    }

    unsafe extern "C" fn get_callback(
        list: *const clap_input_events,
        _index: u32,
    ) -> *const clap_event_header {
        if list.is_null() {
            return ptr::null();
        }
        let ctx = (*list).ctx;
        if ctx.is_null() {
            return ptr::null();
        }
        // This is a simplified implementation - in practice we'd need
        // to store the actual event structs and return pointers to them
        // For now, return null (events won't be delivered)
        ptr::null()
    }
}

impl Default for ClapInputEvents {
    fn default() -> Self {
        Self::new()
    }
}

/// Event list for output events
pub struct ClapOutputEvents {
    events: Vec<ClapEvent>,
    raw: clap_output_events,
}

impl ClapOutputEvents {
    /// Create new empty output event list
    pub fn new() -> Self {
        let mut events = Self {
            events: Vec::new(),
            raw: clap_output_events {
                ctx: ptr::null_mut(),
                try_push: Some(Self::try_push_callback),
            },
        };
        events.raw.ctx = &mut events.events as *mut _ as *mut c_void;
        events
    }

    /// Get raw pointer
    pub fn as_ptr(&self) -> *const clap_output_events {
        &self.raw as *const _
    }

    /// Get collected events
    pub fn events(&self) -> &[ClapEvent] {
        &self.events
    }

    /// Clear events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    unsafe extern "C" fn try_push_callback(
        list: *const clap_output_events,
        event: *const clap_event_header,
    ) -> bool {
        if list.is_null() || event.is_null() {
            return false;
        }
        // Parse and store the event
        // Simplified - just return true
        true
    }
}

impl Default for ClapOutputEvents {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clap_host_creation() {
        let config = HostConfig::default();
        let host = ClapHost::new(config);

        assert!(!host.as_ptr().is_null());
    }

    #[test]
    fn test_clap_host_state() {
        let config = HostConfig::default();
        let host = ClapHost::new(config);

        assert!(!host.state().restart_requested.load(Ordering::SeqCst));
        assert!(!host.state().process_requested.load(Ordering::SeqCst));
    }

    #[test]
    fn test_clap_host_state_requests() {
        let state = ClapHostState::new(HostConfig::default());

        state.restart_requested.store(true, Ordering::SeqCst);
        assert!(state.take_restart_request());
        assert!(!state.take_restart_request()); // Should be cleared

        state.process_requested.store(true, Ordering::SeqCst);
        assert!(state.take_process_request());
        assert!(!state.take_process_request());
    }

    #[test]
    fn test_clap_input_events() {
        let mut events = ClapInputEvents::new();
        assert!(events.is_empty());

        events.push_note_on(0, 0, 60, 0.8);
        events.push_note_off(100, 0, 60, 0.0);
        events.push_param_value(50, 1, 0.5);
        events.push_midi(25, 0, [0x90, 64, 100]);

        assert_eq!(events.len(), 4);
        assert!(!events.as_ptr().is_null());

        events.clear();
        assert!(events.is_empty());
    }

    #[test]
    fn test_clap_output_events() {
        let events = ClapOutputEvents::new();
        assert!(events.events().is_empty());
        assert!(!events.as_ptr().is_null());
    }
}
