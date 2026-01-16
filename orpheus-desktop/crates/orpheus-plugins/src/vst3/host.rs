//! VST3 Host Implementation
//!
//! Provides the host-side interface for VST3 plugins.

use super::com::*;
use super::ffi::*;
use crate::HostConfig;
use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

// =============================================================================
// Host Application
// =============================================================================

/// Host application state
pub struct Vst3HostState {
    /// Host configuration
    pub config: HostConfig,
    /// Reference count
    ref_count: AtomicU32,
}

impl Vst3HostState {
    /// Create new host state
    pub fn new(config: HostConfig) -> Self {
        Self {
            config,
            ref_count: AtomicU32::new(1),
        }
    }

    /// Add a reference
    pub fn add_ref(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Release a reference
    pub fn release(&self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::SeqCst) - 1
    }

    /// Get current reference count
    pub fn ref_count(&self) -> u32 {
        self.ref_count.load(Ordering::SeqCst)
    }
}

/// IHostApplication VTable implementation
#[repr(C)]
#[allow(non_snake_case)]
pub struct IHostApplicationVtbl {
    /// Base FUnknown vtable
    pub unknown: FUnknownVtbl,
    /// Get host name
    pub getName:
        unsafe extern "system" fn(this: *mut c_void, name: *mut u16) -> tresult,
    /// Create instance
    pub createInstance: unsafe extern "system" fn(
        this: *mut c_void,
        cid: *const TUID,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> tresult,
}

/// VST3 host wrapper that implements IHostApplication
pub struct Vst3Host {
    /// Host state
    state: Arc<Vst3HostState>,
    /// VTable (kept alive for COM interface)
    #[allow(dead_code)]
    vtbl: Box<IHostApplicationVtbl>,
    /// Host name (UTF-16)
    name_utf16: Vec<u16>,
}

impl Vst3Host {
    /// Create a new VST3 host
    pub fn new(config: HostConfig) -> Self {
        let state = Arc::new(Vst3HostState::new(config.clone()));

        // Convert name to UTF-16
        let name_utf16: Vec<u16> = config
            .host_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let vtbl = Box::new(IHostApplicationVtbl {
            unknown: FUnknownVtbl {
                queryInterface: Self::query_interface,
                addRef: Self::add_ref,
                release: Self::release,
            },
            getName: Self::get_name,
            createInstance: Self::create_instance,
        });

        Self {
            state,
            vtbl,
            name_utf16,
        }
    }

    /// Get pointer for passing to plugins
    pub fn as_ptr(&self) -> *mut c_void {
        self as *const _ as *mut c_void
    }

    /// Get reference to state
    pub fn state(&self) -> &Arc<Vst3HostState> {
        &self.state
    }

    // Callback implementations

    unsafe extern "system" fn query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> tresult {
        if this.is_null() || iid.is_null() || obj.is_null() {
            return K_INVALID_ARGUMENT;
        }

        let iid_ref = &*iid;

        // Check for supported interfaces
        if *iid_ref == FUNKNOWN_IID || *iid_ref == IHOST_APPLICATION_IID {
            *obj = this;
            Self::add_ref(this);
            return K_RESULT_OK;
        }

        *obj = ptr::null_mut();
        K_NO_INTERFACE
    }

    unsafe extern "system" fn add_ref(this: *mut c_void) -> u32 {
        if this.is_null() {
            return 0;
        }
        let host = &*(this as *const Vst3Host);
        host.state.add_ref()
    }

    unsafe extern "system" fn release(this: *mut c_void) -> u32 {
        if this.is_null() {
            return 0;
        }
        let host = &*(this as *const Vst3Host);
        host.state.release()
    }

    unsafe extern "system" fn get_name(this: *mut c_void, name: *mut u16) -> tresult {
        if this.is_null() || name.is_null() {
            return K_INVALID_ARGUMENT;
        }

        let host = &*(this as *const Vst3Host);
        let len = host.name_utf16.len().min(128);
        ptr::copy_nonoverlapping(host.name_utf16.as_ptr(), name, len);

        K_RESULT_OK
    }

    unsafe extern "system" fn create_instance(
        _this: *mut c_void,
        _cid: *const TUID,
        _iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> tresult {
        // We don't support creating instances from the host
        if !obj.is_null() {
            *obj = ptr::null_mut();
        }
        K_NOT_IMPLEMENTED
    }
}

// =============================================================================
// Component Handler
// =============================================================================

/// IComponentHandler interface ID
pub const ICOMPONENT_HANDLER_IID: TUID = TUID::from_u32(0x93A0BEA3, 0x0BD045DB, 0x8E890B0C, 0xC1E46AC6);

/// Component handler state for receiving plugin notifications
pub struct Vst3ComponentHandler {
    /// Reference count (for COM compatibility)
    #[allow(dead_code)]
    ref_count: AtomicU32,
    /// Pending parameter changes
    pending_changes: parking_lot::Mutex<Vec<(ParamID, ParamValue)>>,
    /// Restart requested
    restart_requested: std::sync::atomic::AtomicBool,
}

impl Vst3ComponentHandler {
    /// Create new component handler
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            ref_count: AtomicU32::new(1),
            pending_changes: parking_lot::Mutex::new(Vec::new()),
            restart_requested: std::sync::atomic::AtomicBool::new(false),
        })
    }

    /// Take pending parameter changes
    pub fn take_changes(&self) -> Vec<(ParamID, ParamValue)> {
        let mut changes = self.pending_changes.lock();
        std::mem::take(&mut *changes)
    }

    /// Check and clear restart request
    pub fn take_restart_request(&self) -> bool {
        self.restart_requested.swap(false, Ordering::SeqCst)
    }

    /// Record a parameter change from the plugin
    pub fn record_change(&self, param_id: ParamID, value: ParamValue) {
        let mut changes = self.pending_changes.lock();
        changes.push((param_id, value));
    }

    /// Request restart
    pub fn request_restart(&self) {
        self.restart_requested.store(true, Ordering::SeqCst);
    }
}

impl Default for Vst3ComponentHandler {
    fn default() -> Self {
        Self {
            ref_count: AtomicU32::new(1),
            pending_changes: parking_lot::Mutex::new(Vec::new()),
            restart_requested: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

// =============================================================================
// Audio Processing Helpers
// =============================================================================

/// Helper for managing audio buffers
pub struct AudioBufferManager {
    /// Input buffers
    input_buffers: Vec<Vec<f32>>,
    /// Output buffers
    output_buffers: Vec<Vec<f32>>,
    /// Input bus descriptors (for VST3 ProcessData)
    #[allow(dead_code)]
    input_buses: Vec<AudioBusBuffers>,
    /// Output bus descriptors (for VST3 ProcessData)
    #[allow(dead_code)]
    output_buses: Vec<AudioBusBuffers>,
    /// Block size
    block_size: usize,
}

impl AudioBufferManager {
    /// Create new buffer manager
    pub fn new(
        num_input_channels: usize,
        num_output_channels: usize,
        block_size: usize,
    ) -> Self {
        let mut input_buffers = Vec::with_capacity(num_input_channels);
        let mut output_buffers = Vec::with_capacity(num_output_channels);

        for _ in 0..num_input_channels {
            input_buffers.push(vec![0.0f32; block_size]);
        }
        for _ in 0..num_output_channels {
            output_buffers.push(vec![0.0f32; block_size]);
        }

        Self {
            input_buffers,
            output_buffers,
            input_buses: Vec::new(),
            output_buses: Vec::new(),
            block_size,
        }
    }

    /// Get block size
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// Get mutable reference to input buffer
    pub fn input_buffer_mut(&mut self, channel: usize) -> Option<&mut [f32]> {
        self.input_buffers.get_mut(channel).map(|b| b.as_mut_slice())
    }

    /// Get reference to output buffer
    pub fn output_buffer(&self, channel: usize) -> Option<&[f32]> {
        self.output_buffers.get(channel).map(|b| b.as_slice())
    }

    /// Get mutable reference to output buffer
    pub fn output_buffer_mut(&mut self, channel: usize) -> Option<&mut [f32]> {
        self.output_buffers.get_mut(channel).map(|b| b.as_mut_slice())
    }

    /// Clear all buffers
    pub fn clear(&mut self) {
        for buffer in &mut self.input_buffers {
            buffer.fill(0.0);
        }
        for buffer in &mut self.output_buffers {
            buffer.fill(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vst3_host_creation() {
        let config = HostConfig::default();
        let host = Vst3Host::new(config);
        assert!(!host.as_ptr().is_null());
    }

    #[test]
    fn test_vst3_host_state() {
        let config = HostConfig::default();
        let state = Vst3HostState::new(config);
        assert_eq!(state.ref_count(), 1);

        assert_eq!(state.add_ref(), 2);
        assert_eq!(state.ref_count(), 2);

        assert_eq!(state.release(), 1);
        assert_eq!(state.ref_count(), 1);
    }

    #[test]
    fn test_component_handler() {
        let handler = Vst3ComponentHandler::new();

        handler.record_change(1, 0.5);
        handler.record_change(2, 0.75);

        let changes = handler.take_changes();
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0], (1, 0.5));
        assert_eq!(changes[1], (2, 0.75));

        // Should be empty now
        assert!(handler.take_changes().is_empty());
    }

    #[test]
    fn test_component_handler_restart() {
        let handler = Vst3ComponentHandler::new();

        assert!(!handler.take_restart_request());

        handler.request_restart();
        assert!(handler.take_restart_request());

        // Should be cleared
        assert!(!handler.take_restart_request());
    }

    #[test]
    fn test_audio_buffer_manager() {
        let mut manager = AudioBufferManager::new(2, 2, 512);

        assert_eq!(manager.block_size(), 512);

        // Write to input
        if let Some(buffer) = manager.input_buffer_mut(0) {
            buffer[0] = 1.0;
            assert_eq!(buffer.len(), 512);
        }

        // Read from output
        if let Some(buffer) = manager.output_buffer(0) {
            assert_eq!(buffer[0], 0.0);
        }

        // Clear
        manager.clear();
        if let Some(buffer) = manager.input_buffer_mut(0) {
            assert_eq!(buffer[0], 0.0);
        }
    }

    #[test]
    fn test_audio_buffer_manager_channels() {
        let manager = AudioBufferManager::new(4, 6, 256);

        assert!(manager.output_buffer(5).is_some());
        assert!(manager.output_buffer(6).is_none());
    }
}
