//! CLAP FFI Definitions
//!
//! Raw C type definitions from the CLAP (CLever Audio Plugin) specification.
//! These are manually defined to avoid external dependencies.
//!
//! Reference: https://github.com/free-audio/clap

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::{c_char, c_void};

// =============================================================================
// Version
// =============================================================================

/// CLAP version structure
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct clap_version {
    pub major: u32,
    pub minor: u32,
    pub revision: u32,
}

impl clap_version {
    pub const fn new(major: u32, minor: u32, revision: u32) -> Self {
        Self { major, minor, revision }
    }
}

/// Current CLAP version we support
pub const CLAP_VERSION: clap_version = clap_version::new(1, 2, 2);

// =============================================================================
// Plugin Entry
// =============================================================================

/// Plugin entry point - found by looking for `clap_entry` symbol
#[repr(C)]
pub struct clap_plugin_entry {
    pub clap_version: clap_version,
    pub init: Option<unsafe extern "C" fn(plugin_path: *const c_char) -> bool>,
    pub deinit: Option<unsafe extern "C" fn()>,
    pub get_factory: Option<unsafe extern "C" fn(factory_id: *const c_char) -> *const c_void>,
}

/// Factory ID for plugin factory
pub const CLAP_PLUGIN_FACTORY_ID: &[u8] = b"clap.plugin-factory\0";

// =============================================================================
// Plugin Factory
// =============================================================================

/// Plugin factory - used to enumerate and create plugins
#[repr(C)]
pub struct clap_plugin_factory {
    pub get_plugin_count: Option<unsafe extern "C" fn(factory: *const clap_plugin_factory) -> u32>,
    pub get_plugin_descriptor: Option<
        unsafe extern "C" fn(
            factory: *const clap_plugin_factory,
            index: u32,
        ) -> *const clap_plugin_descriptor,
    >,
    pub create_plugin: Option<
        unsafe extern "C" fn(
            factory: *const clap_plugin_factory,
            host: *const clap_host,
            plugin_id: *const c_char,
        ) -> *const clap_plugin,
    >,
}

// =============================================================================
// Plugin Descriptor
// =============================================================================

/// Plugin descriptor - metadata about a plugin
#[repr(C)]
pub struct clap_plugin_descriptor {
    pub clap_version: clap_version,
    pub id: *const c_char,
    pub name: *const c_char,
    pub vendor: *const c_char,
    pub url: *const c_char,
    pub manual_url: *const c_char,
    pub support_url: *const c_char,
    pub version: *const c_char,
    pub description: *const c_char,
    /// Null-terminated array of feature strings
    pub features: *const *const c_char,
}

/// Plugin feature strings
pub const CLAP_PLUGIN_FEATURE_INSTRUMENT: &[u8] = b"instrument\0";
pub const CLAP_PLUGIN_FEATURE_AUDIO_EFFECT: &[u8] = b"audio-effect\0";
pub const CLAP_PLUGIN_FEATURE_ANALYZER: &[u8] = b"analyzer\0";
pub const CLAP_PLUGIN_FEATURE_SYNTHESIZER: &[u8] = b"synthesizer\0";
pub const CLAP_PLUGIN_FEATURE_SAMPLER: &[u8] = b"sampler\0";
pub const CLAP_PLUGIN_FEATURE_DRUM: &[u8] = b"drum\0";
pub const CLAP_PLUGIN_FEATURE_EQUALIZER: &[u8] = b"equalizer\0";
pub const CLAP_PLUGIN_FEATURE_COMPRESSOR: &[u8] = b"compressor\0";
pub const CLAP_PLUGIN_FEATURE_DISTORTION: &[u8] = b"distortion\0";
pub const CLAP_PLUGIN_FEATURE_FILTER: &[u8] = b"filter\0";
pub const CLAP_PLUGIN_FEATURE_DELAY: &[u8] = b"delay\0";
pub const CLAP_PLUGIN_FEATURE_REVERB: &[u8] = b"reverb\0";
pub const CLAP_PLUGIN_FEATURE_CHORUS: &[u8] = b"chorus\0";
pub const CLAP_PLUGIN_FEATURE_FLANGER: &[u8] = b"flanger\0";
pub const CLAP_PLUGIN_FEATURE_PHASER: &[u8] = b"phaser\0";
pub const CLAP_PLUGIN_FEATURE_LIMITER: &[u8] = b"limiter\0";
pub const CLAP_PLUGIN_FEATURE_STEREO: &[u8] = b"stereo\0";
pub const CLAP_PLUGIN_FEATURE_MONO: &[u8] = b"mono\0";

// =============================================================================
// Host
// =============================================================================

/// Host structure - callbacks provided to plugin
#[repr(C)]
pub struct clap_host {
    pub clap_version: clap_version,
    pub host_data: *mut c_void,
    pub name: *const c_char,
    pub vendor: *const c_char,
    pub url: *const c_char,
    pub version: *const c_char,

    /// Request the host to call plugin->on_main_thread() on the main thread
    pub request_restart: Option<unsafe extern "C" fn(host: *const clap_host)>,
    /// Request the host to call plugin->process() again
    pub request_process: Option<unsafe extern "C" fn(host: *const clap_host)>,
    /// Request a callback on the main thread
    pub request_callback: Option<unsafe extern "C" fn(host: *const clap_host)>,
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin structure - the main plugin interface
#[repr(C)]
pub struct clap_plugin {
    pub desc: *const clap_plugin_descriptor,
    pub plugin_data: *mut c_void,

    pub init: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> bool>,
    pub destroy: Option<unsafe extern "C" fn(plugin: *const clap_plugin)>,
    pub activate: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            sample_rate: f64,
            min_frames_count: u32,
            max_frames_count: u32,
        ) -> bool,
    >,
    pub deactivate: Option<unsafe extern "C" fn(plugin: *const clap_plugin)>,
    pub start_processing: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> bool>,
    pub stop_processing: Option<unsafe extern "C" fn(plugin: *const clap_plugin)>,
    pub reset: Option<unsafe extern "C" fn(plugin: *const clap_plugin)>,
    pub process: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, process: *const clap_process) -> clap_process_status,
    >,
    pub get_extension:
        Option<unsafe extern "C" fn(plugin: *const clap_plugin, id: *const c_char) -> *const c_void>,
    pub on_main_thread: Option<unsafe extern "C" fn(plugin: *const clap_plugin)>,
}

// =============================================================================
// Process
// =============================================================================

/// Process status returned by plugin
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum clap_process_status {
    /// Processing failed
    CLAP_PROCESS_ERROR = 0,
    /// Processing succeeded, plugin is done
    CLAP_PROCESS_CONTINUE = 1,
    /// Processing succeeded, plugin wants to keep processing (tail)
    CLAP_PROCESS_CONTINUE_IF_NOT_QUIET = 2,
    /// Processing succeeded, plugin is in tail and will produce silence
    CLAP_PROCESS_TAIL = 3,
    /// Processing succeeded, plugin is sleeping
    CLAP_PROCESS_SLEEP = 4,
}

/// Transport flags
pub const CLAP_TRANSPORT_HAS_TEMPO: u32 = 1 << 0;
pub const CLAP_TRANSPORT_HAS_BEATS_TIMELINE: u32 = 1 << 1;
pub const CLAP_TRANSPORT_HAS_SECONDS_TIMELINE: u32 = 1 << 2;
pub const CLAP_TRANSPORT_HAS_TIME_SIGNATURE: u32 = 1 << 3;
pub const CLAP_TRANSPORT_IS_PLAYING: u32 = 1 << 4;
pub const CLAP_TRANSPORT_IS_RECORDING: u32 = 1 << 5;
pub const CLAP_TRANSPORT_IS_LOOP_ACTIVE: u32 = 1 << 6;
pub const CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL: u32 = 1 << 7;

/// Fixed-point time representation (for beat time)
pub type clap_beattime = i64;
/// Fixed-point time representation (for seconds)
pub type clap_sectime = i64;

/// Transport information
#[repr(C)]
#[derive(Debug, Clone)]
pub struct clap_event_transport {
    pub header: clap_event_header,
    pub flags: u32,
    pub song_pos_beats: clap_beattime,
    pub song_pos_seconds: clap_sectime,
    pub tempo: f64,
    pub tempo_inc: f64,
    pub loop_start_beats: clap_beattime,
    pub loop_end_beats: clap_beattime,
    pub loop_start_seconds: clap_sectime,
    pub loop_end_seconds: clap_sectime,
    pub bar_start: clap_beattime,
    pub bar_number: i32,
    pub tsig_num: u16,
    pub tsig_denom: u16,
}

/// Audio buffer for a single port
#[repr(C)]
pub struct clap_audio_buffer {
    pub data32: *mut *mut f32,
    pub data64: *mut *mut f64,
    pub channel_count: u32,
    pub latency: u32,
    pub constant_mask: u64,
}

/// Process context
#[repr(C)]
pub struct clap_process {
    pub steady_time: i64,
    pub frames_count: u32,
    pub transport: *const clap_event_transport,
    pub audio_inputs: *const clap_audio_buffer,
    pub audio_outputs: *mut clap_audio_buffer,
    pub audio_inputs_count: u32,
    pub audio_outputs_count: u32,
    pub in_events: *const clap_input_events,
    pub out_events: *const clap_output_events,
}

// =============================================================================
// Events
// =============================================================================

/// Event header - common to all events
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct clap_event_header {
    pub size: u32,
    pub time: u32,
    pub space_id: u16,
    pub type_: u16,
    pub flags: u32,
}

/// Core event space ID
pub const CLAP_CORE_EVENT_SPACE_ID: u16 = 0;

/// Event types
pub const CLAP_EVENT_NOTE_ON: u16 = 0;
pub const CLAP_EVENT_NOTE_OFF: u16 = 1;
pub const CLAP_EVENT_NOTE_CHOKE: u16 = 2;
pub const CLAP_EVENT_NOTE_END: u16 = 3;
pub const CLAP_EVENT_NOTE_EXPRESSION: u16 = 4;
pub const CLAP_EVENT_PARAM_VALUE: u16 = 5;
pub const CLAP_EVENT_PARAM_MOD: u16 = 6;
pub const CLAP_EVENT_PARAM_GESTURE_BEGIN: u16 = 7;
pub const CLAP_EVENT_PARAM_GESTURE_END: u16 = 8;
pub const CLAP_EVENT_TRANSPORT: u16 = 9;
pub const CLAP_EVENT_MIDI: u16 = 10;
pub const CLAP_EVENT_MIDI_SYSEX: u16 = 11;
pub const CLAP_EVENT_MIDI2: u16 = 12;

/// Note event
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct clap_event_note {
    pub header: clap_event_header,
    pub note_id: i32,
    pub port_index: i16,
    pub channel: i16,
    pub key: i16,
    pub velocity: f64,
}

/// Parameter value event
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct clap_event_param_value {
    pub header: clap_event_header,
    pub param_id: u32,
    pub cookie: *mut c_void,
    pub note_id: i32,
    pub port_index: i16,
    pub channel: i16,
    pub key: i16,
    pub value: f64,
}

/// MIDI event (1-3 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct clap_event_midi {
    pub header: clap_event_header,
    pub port_index: u16,
    pub data: [u8; 3],
}

/// Input events interface
#[repr(C)]
pub struct clap_input_events {
    pub ctx: *mut c_void,
    pub size: Option<unsafe extern "C" fn(list: *const clap_input_events) -> u32>,
    pub get: Option<
        unsafe extern "C" fn(list: *const clap_input_events, index: u32) -> *const clap_event_header,
    >,
}

/// Output events interface
#[repr(C)]
pub struct clap_output_events {
    pub ctx: *mut c_void,
    pub try_push:
        Option<unsafe extern "C" fn(list: *const clap_output_events, event: *const clap_event_header) -> bool>,
}

// =============================================================================
// Extensions
// =============================================================================

/// Extension ID for audio ports
pub const CLAP_EXT_AUDIO_PORTS: &[u8] = b"clap.audio-ports\0";
/// Extension ID for note ports
pub const CLAP_EXT_NOTE_PORTS: &[u8] = b"clap.note-ports\0";
/// Extension ID for parameters
pub const CLAP_EXT_PARAMS: &[u8] = b"clap.params\0";
/// Extension ID for state
pub const CLAP_EXT_STATE: &[u8] = b"clap.state\0";
/// Extension ID for GUI
pub const CLAP_EXT_GUI: &[u8] = b"clap.gui\0";
/// Extension ID for latency
pub const CLAP_EXT_LATENCY: &[u8] = b"clap.latency\0";

// Audio ports extension
#[repr(C)]
pub struct clap_audio_port_info {
    pub id: u32,
    pub name: [c_char; 256],
    pub flags: u32,
    pub channel_count: u32,
    pub port_type: *const c_char,
    pub in_place_pair: u32,
}

pub const CLAP_AUDIO_PORT_IS_MAIN: u32 = 1 << 0;
pub const CLAP_AUDIO_PORT_SUPPORTS_64BITS: u32 = 1 << 1;
pub const CLAP_AUDIO_PORT_PREFERS_64BITS: u32 = 1 << 2;
pub const CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE: u32 = 1 << 3;

pub const CLAP_PORT_MONO: &[u8] = b"mono\0";
pub const CLAP_PORT_STEREO: &[u8] = b"stereo\0";

#[repr(C)]
pub struct clap_plugin_audio_ports {
    pub count: Option<unsafe extern "C" fn(plugin: *const clap_plugin, is_input: bool) -> u32>,
    pub get: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            index: u32,
            is_input: bool,
            info: *mut clap_audio_port_info,
        ) -> bool,
    >,
}

// Note ports extension
#[repr(C)]
pub struct clap_note_port_info {
    pub id: u32,
    pub supported_dialects: u32,
    pub preferred_dialect: u32,
    pub name: [c_char; 256],
}

pub const CLAP_NOTE_DIALECT_CLAP: u32 = 1 << 0;
pub const CLAP_NOTE_DIALECT_MIDI: u32 = 1 << 1;
pub const CLAP_NOTE_DIALECT_MIDI_MPE: u32 = 1 << 2;
pub const CLAP_NOTE_DIALECT_MIDI2: u32 = 1 << 3;

#[repr(C)]
pub struct clap_plugin_note_ports {
    pub count: Option<unsafe extern "C" fn(plugin: *const clap_plugin, is_input: bool) -> u32>,
    pub get: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            index: u32,
            is_input: bool,
            info: *mut clap_note_port_info,
        ) -> bool,
    >,
}

// Parameters extension
#[repr(C)]
pub struct clap_param_info {
    pub id: u32,
    pub flags: u32,
    pub cookie: *mut c_void,
    pub name: [c_char; 256],
    pub module: [c_char; 1024],
    pub min_value: f64,
    pub max_value: f64,
    pub default_value: f64,
}

pub const CLAP_PARAM_IS_STEPPED: u32 = 1 << 0;
pub const CLAP_PARAM_IS_PERIODIC: u32 = 1 << 1;
pub const CLAP_PARAM_IS_HIDDEN: u32 = 1 << 2;
pub const CLAP_PARAM_IS_READONLY: u32 = 1 << 3;
pub const CLAP_PARAM_IS_BYPASS: u32 = 1 << 4;
pub const CLAP_PARAM_IS_AUTOMATABLE: u32 = 1 << 5;
pub const CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID: u32 = 1 << 6;
pub const CLAP_PARAM_IS_AUTOMATABLE_PER_KEY: u32 = 1 << 7;
pub const CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL: u32 = 1 << 8;
pub const CLAP_PARAM_IS_AUTOMATABLE_PER_PORT: u32 = 1 << 9;
pub const CLAP_PARAM_IS_MODULATABLE: u32 = 1 << 10;
pub const CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID: u32 = 1 << 11;
pub const CLAP_PARAM_IS_MODULATABLE_PER_KEY: u32 = 1 << 12;
pub const CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL: u32 = 1 << 13;
pub const CLAP_PARAM_IS_MODULATABLE_PER_PORT: u32 = 1 << 14;
pub const CLAP_PARAM_REQUIRES_PROCESS: u32 = 1 << 15;

#[repr(C)]
pub struct clap_plugin_params {
    pub count: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> u32>,
    pub get_info: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, param_index: u32, param_info: *mut clap_param_info) -> bool,
    >,
    pub get_value:
        Option<unsafe extern "C" fn(plugin: *const clap_plugin, param_id: u32, out_value: *mut f64) -> bool>,
    pub value_to_text: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            param_id: u32,
            value: f64,
            out_buffer: *mut c_char,
            out_buffer_capacity: u32,
        ) -> bool,
    >,
    pub text_to_value: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            param_id: u32,
            param_value_text: *const c_char,
            out_value: *mut f64,
        ) -> bool,
    >,
    pub flush: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            in_events: *const clap_input_events,
            out_events: *const clap_output_events,
        ),
    >,
}

// State extension
#[repr(C)]
pub struct clap_istream {
    pub ctx: *mut c_void,
    pub read: Option<unsafe extern "C" fn(stream: *const clap_istream, buffer: *mut c_void, size: u64) -> i64>,
}

#[repr(C)]
pub struct clap_ostream {
    pub ctx: *mut c_void,
    pub write: Option<unsafe extern "C" fn(stream: *const clap_ostream, buffer: *const c_void, size: u64) -> i64>,
}

#[repr(C)]
pub struct clap_plugin_state {
    pub save: Option<unsafe extern "C" fn(plugin: *const clap_plugin, stream: *const clap_ostream) -> bool>,
    pub load: Option<unsafe extern "C" fn(plugin: *const clap_plugin, stream: *const clap_istream) -> bool>,
}

// Latency extension
#[repr(C)]
pub struct clap_plugin_latency {
    pub get: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> u32>,
}

#[repr(C)]
pub struct clap_host_latency {
    pub changed: Option<unsafe extern "C" fn(host: *const clap_host)>,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_clap_version() {
        assert_eq!(CLAP_VERSION.major, 1);
        assert_eq!(CLAP_VERSION.minor, 2);
        assert_eq!(CLAP_VERSION.revision, 2);
    }

    #[test]
    fn test_struct_sizes() {
        // Verify struct sizes match expected C ABI
        assert_eq!(mem::size_of::<clap_version>(), 12);
        assert_eq!(mem::size_of::<clap_event_header>(), 16);
    }

    #[test]
    fn test_event_types() {
        assert_eq!(CLAP_EVENT_NOTE_ON, 0);
        assert_eq!(CLAP_EVENT_NOTE_OFF, 1);
        assert_eq!(CLAP_EVENT_MIDI, 10);
    }

    #[test]
    fn test_process_status() {
        assert_eq!(clap_process_status::CLAP_PROCESS_ERROR as i32, 0);
        assert_eq!(clap_process_status::CLAP_PROCESS_CONTINUE as i32, 1);
    }
}
