//! VST3 FFI Definitions
//!
//! Raw C type definitions from the VST3 SDK specification.
//! These are manually defined to avoid external dependencies.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use super::com::*;
use std::ffi::c_void;

// =============================================================================
// Constants
// =============================================================================

/// Maximum length for strings in VST3
pub const VST3_MAX_STRING_LEN: usize = 128;

/// Bus directions
pub const K_INPUT: i32 = 0;
pub const K_OUTPUT: i32 = 1;

/// Bus types
pub const K_MAIN: i32 = 0;
pub const K_AUX: i32 = 1;

/// Media types
pub const K_AUDIO: i32 = 0;
pub const K_EVENT: i32 = 1;

/// IO modes
pub const K_SIMPLE: i32 = 0;
pub const K_ADVANCED: i32 = 1;
pub const K_OFFLINE_PROCESSING: i32 = 2;

/// Component flags
pub const K_DISTRIBUTABLE: u32 = 1 << 0;
pub const K_SIMPLE_MODE_SUPPORTED: u32 = 1 << 1;

/// Symbolic sample sizes
pub const K_SAMPLE_32: i32 = 0;
pub const K_SAMPLE_64: i32 = 1;

/// Process modes
pub const K_REALTIME: i32 = 0;
pub const K_PREFETCH: i32 = 1;
pub const K_OFFLINE: i32 = 2;

/// Event types
pub const K_NOTE_ON_EVENT: u16 = 0;
pub const K_NOTE_OFF_EVENT: u16 = 1;
pub const K_DATA_EVENT: u16 = 2;
pub const K_POLY_PRESSURE_EVENT: u16 = 3;
pub const K_NOTE_EXPRESSION_VALUE_EVENT: u16 = 4;
pub const K_NOTE_EXPRESSION_TEXT_EVENT: u16 = 5;
pub const K_CHORD_EVENT: u16 = 6;
pub const K_SCALE_EVENT: u16 = 7;
pub const K_LEGACY_MIDI_CC_OUT_EVENT: u16 = 65535;

// =============================================================================
// Audio Types
// =============================================================================

/// Sample rate type
pub type SampleRate = f64;

/// Sample32 type (single precision)
pub type Sample32 = f32;

/// Sample64 type (double precision)
pub type Sample64 = f64;

/// Parameter ID type
pub type ParamID = u32;

/// Parameter value type
pub type ParamValue = f64;

/// Note ID type
pub type NoteID = i32;

/// Speaker arrangement (bit field)
pub type SpeakerArrangement = u64;

// =============================================================================
// Process Data
// =============================================================================

/// Audio bus buffers
#[repr(C)]
pub struct AudioBusBuffers {
    /// Number of channels in the bus
    pub numChannels: i32,
    /// Silent flags for each channel (bit field)
    pub silenceFlags: u64,
    /// Channel buffer pointers (32-bit)
    pub channelBuffers32: *mut *mut Sample32,
    /// Channel buffer pointers (64-bit)
    pub channelBuffers64: *mut *mut Sample64,
}

impl Default for AudioBusBuffers {
    fn default() -> Self {
        Self {
            numChannels: 0,
            silenceFlags: 0,
            channelBuffers32: std::ptr::null_mut(),
            channelBuffers64: std::ptr::null_mut(),
        }
    }
}

/// Process setup information
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcessSetup {
    /// Process mode (realtime, prefetch, offline)
    pub processMode: i32,
    /// Symbolic sample size (32 or 64 bit)
    pub symbolicSampleSize: i32,
    /// Maximum block size
    pub maxSamplesPerBlock: i32,
    /// Sample rate
    pub sampleRate: SampleRate,
}

impl Default for ProcessSetup {
    fn default() -> Self {
        Self {
            processMode: K_REALTIME,
            symbolicSampleSize: K_SAMPLE_32,
            maxSamplesPerBlock: 1024,
            sampleRate: 44100.0,
        }
    }
}

/// Process context (musical context, transport info)
#[repr(C)]
pub struct ProcessContext {
    /// State flags
    pub state: u32,
    /// Sample rate
    pub sampleRate: SampleRate,
    /// Project time in samples
    pub projectTimeSamples: i64,
    /// System time in nanoseconds
    pub systemTime: i64,
    /// Continuous time samples (bar counter)
    pub continousTimeSamples: i64,
    /// Project time in music (quarters)
    pub projectTimeMusic: f64,
    /// Bar position in music (quarters)
    pub barPositionMusic: f64,
    /// Cycle start in music (quarters)
    pub cycleStartMusic: f64,
    /// Cycle end in music (quarters)
    pub cycleEndMusic: f64,
    /// Tempo in BPM
    pub tempo: f64,
    /// Time signature numerator
    pub timeSigNumerator: i32,
    /// Time signature denominator
    pub timeSigDenominator: i32,
    /// Chord info
    pub chord: i32,
    /// SMPTE offset subframes
    pub smpteOffsetSubframes: i32,
    /// Frame rate
    pub frameRate: u32,
    /// Samples to next clock
    pub samplesToNextClock: i32,
}

impl Default for ProcessContext {
    fn default() -> Self {
        Self {
            state: 0,
            sampleRate: 44100.0,
            projectTimeSamples: 0,
            systemTime: 0,
            continousTimeSamples: 0,
            projectTimeMusic: 0.0,
            barPositionMusic: 0.0,
            cycleStartMusic: 0.0,
            cycleEndMusic: 0.0,
            tempo: 120.0,
            timeSigNumerator: 4,
            timeSigDenominator: 4,
            chord: 0,
            smpteOffsetSubframes: 0,
            frameRate: 0,
            samplesToNextClock: 0,
        }
    }
}

/// Process data passed to audio processor
#[repr(C)]
pub struct ProcessData {
    /// Processing mode flags
    pub processMode: i32,
    /// Symbolic sample size
    pub symbolicSampleSize: i32,
    /// Number of samples to process
    pub numSamples: i32,
    /// Number of input audio buses
    pub numInputs: i32,
    /// Number of output audio buses
    pub numOutputs: i32,
    /// Input audio buffers
    pub inputs: *mut AudioBusBuffers,
    /// Output audio buffers
    pub outputs: *mut AudioBusBuffers,
    /// Input parameter changes
    pub inputParameterChanges: *mut c_void, // IParameterChanges*
    /// Output parameter changes
    pub outputParameterChanges: *mut c_void, // IParameterChanges*
    /// Input events
    pub inputEvents: *mut c_void, // IEventList*
    /// Output events
    pub outputEvents: *mut c_void, // IEventList*
    /// Process context
    pub processContext: *mut ProcessContext,
}

impl Default for ProcessData {
    fn default() -> Self {
        Self {
            processMode: K_REALTIME,
            symbolicSampleSize: K_SAMPLE_32,
            numSamples: 0,
            numInputs: 0,
            numOutputs: 0,
            inputs: std::ptr::null_mut(),
            outputs: std::ptr::null_mut(),
            inputParameterChanges: std::ptr::null_mut(),
            outputParameterChanges: std::ptr::null_mut(),
            inputEvents: std::ptr::null_mut(),
            outputEvents: std::ptr::null_mut(),
            processContext: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// Bus Info
// =============================================================================

/// Bus information
#[repr(C)]
pub struct BusInfo {
    /// Media type (audio or event)
    pub mediaType: i32,
    /// Direction (input or output)
    pub direction: i32,
    /// Number of channels (audio) or events (event)
    pub channelCount: i32,
    /// Bus name (UTF-16)
    pub name: [u16; 128],
    /// Bus type (main or aux)
    pub busType: i32,
    /// Flags
    pub flags: u32,
}

impl Default for BusInfo {
    fn default() -> Self {
        Self {
            mediaType: K_AUDIO,
            direction: K_INPUT,
            channelCount: 2,
            name: [0; 128],
            busType: K_MAIN,
            flags: 0,
        }
    }
}

/// Routing info
#[repr(C)]
pub struct RoutingInfo {
    /// Media type
    pub mediaType: i32,
    /// Bus index
    pub busIndex: i32,
    /// Channel
    pub channel: i32,
}

// =============================================================================
// Events
// =============================================================================

/// Note-on event
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NoteOnEvent {
    /// Channel index
    pub channel: i16,
    /// Pitch (MIDI note number)
    pub pitch: i16,
    /// Tuning offset in cents
    pub tuning: f32,
    /// Velocity (0.0 - 1.0)
    pub velocity: f32,
    /// Length in samples (0 = no note-off)
    pub length: i32,
    /// Note ID
    pub noteId: NoteID,
}

/// Note-off event
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NoteOffEvent {
    /// Channel index
    pub channel: i16,
    /// Pitch (MIDI note number)
    pub pitch: i16,
    /// Velocity (0.0 - 1.0)
    pub velocity: f32,
    /// Note ID
    pub noteId: NoteID,
    /// Tuning offset in cents
    pub tuning: f32,
}

/// Data event (SysEx, etc.)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DataEvent {
    /// Size of data
    pub size: u32,
    /// Data type
    pub type_: u32,
    /// Data pointer
    pub bytes: *const u8,
}

/// Polyphonic pressure event
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PolyPressureEvent {
    /// Channel
    pub channel: i16,
    /// Pitch
    pub pitch: i16,
    /// Pressure (0.0 - 1.0)
    pub pressure: f32,
    /// Note ID
    pub noteId: NoteID,
}

/// Event union
#[repr(C)]
#[derive(Clone, Copy)]
pub union EventData {
    pub noteOn: NoteOnEvent,
    pub noteOff: NoteOffEvent,
    pub data: DataEvent,
    pub polyPressure: PolyPressureEvent,
}

/// Event structure
#[repr(C)]
pub struct Event {
    /// Bus index
    pub busIndex: i32,
    /// Sample offset
    pub sampleOffset: i32,
    /// Position in PPQ
    pub ppqPosition: f64,
    /// Event flags
    pub flags: u16,
    /// Event type
    pub type_: u16,
    /// Event data
    pub data: EventData,
}

// =============================================================================
// Component Interfaces
// =============================================================================

/// IPluginBase VTable
#[repr(C)]
pub struct IPluginBaseVtbl {
    /// Base FUnknown vtable
    pub unknown: FUnknownVtbl,
    /// Initialize the plugin
    pub initialize: unsafe extern "system" fn(this: *mut c_void, context: *mut c_void) -> tresult,
    /// Terminate the plugin
    pub terminate: unsafe extern "system" fn(this: *mut c_void) -> tresult,
}

/// IPluginBase interface
#[repr(C)]
pub struct IPluginBase {
    pub vtbl: *const IPluginBaseVtbl,
}

impl AsRef<FUnknown> for IPluginBase {
    fn as_ref(&self) -> &FUnknown {
        unsafe { &*(self as *const _ as *const FUnknown) }
    }
}

/// IComponent VTable
#[repr(C)]
pub struct IComponentVtbl {
    /// Base IPluginBase vtable
    pub base: IPluginBaseVtbl,
    /// Get controller class ID
    pub getControllerClassId:
        unsafe extern "system" fn(this: *mut c_void, classId: *mut TUID) -> tresult,
    /// Set IO mode
    pub setIoMode: unsafe extern "system" fn(this: *mut c_void, mode: i32) -> tresult,
    /// Get bus count
    pub getBusCount:
        unsafe extern "system" fn(this: *mut c_void, type_: i32, dir: i32) -> i32,
    /// Get bus info
    pub getBusInfo: unsafe extern "system" fn(
        this: *mut c_void,
        type_: i32,
        dir: i32,
        index: i32,
        bus: *mut BusInfo,
    ) -> tresult,
    /// Get routing info
    pub getRoutingInfo: unsafe extern "system" fn(
        this: *mut c_void,
        inInfo: *mut RoutingInfo,
        outInfo: *mut RoutingInfo,
    ) -> tresult,
    /// Activate/deactivate bus
    pub activateBus: unsafe extern "system" fn(
        this: *mut c_void,
        type_: i32,
        dir: i32,
        index: i32,
        state: u8,
    ) -> tresult,
    /// Set active state
    pub setActive: unsafe extern "system" fn(this: *mut c_void, state: u8) -> tresult,
    /// Set/get state
    pub setState: unsafe extern "system" fn(this: *mut c_void, state: *mut c_void) -> tresult,
    /// Get state
    pub getState: unsafe extern "system" fn(this: *mut c_void, state: *mut c_void) -> tresult,
}

/// IComponent interface
#[repr(C)]
pub struct IComponent {
    pub vtbl: *const IComponentVtbl,
}

impl AsRef<FUnknown> for IComponent {
    fn as_ref(&self) -> &FUnknown {
        unsafe { &*(self as *const _ as *const FUnknown) }
    }
}

/// IAudioProcessor VTable
#[repr(C)]
pub struct IAudioProcessorVtbl {
    /// Base FUnknown vtable
    pub unknown: FUnknownVtbl,
    /// Set bus arrangements
    pub setBusArrangements: unsafe extern "system" fn(
        this: *mut c_void,
        inputs: *mut SpeakerArrangement,
        numIns: i32,
        outputs: *mut SpeakerArrangement,
        numOuts: i32,
    ) -> tresult,
    /// Get bus arrangement
    pub getBusArrangement: unsafe extern "system" fn(
        this: *mut c_void,
        dir: i32,
        index: i32,
        arr: *mut SpeakerArrangement,
    ) -> tresult,
    /// Can process sample size
    pub canProcessSampleSize:
        unsafe extern "system" fn(this: *mut c_void, symbolicSampleSize: i32) -> tresult,
    /// Get latency samples
    pub getLatencySamples: unsafe extern "system" fn(this: *mut c_void) -> u32,
    /// Setup processing
    pub setupProcessing:
        unsafe extern "system" fn(this: *mut c_void, setup: *mut ProcessSetup) -> tresult,
    /// Set processing state
    pub setProcessing: unsafe extern "system" fn(this: *mut c_void, state: u8) -> tresult,
    /// Process audio
    pub process: unsafe extern "system" fn(this: *mut c_void, data: *mut ProcessData) -> tresult,
    /// Get tail samples
    pub getTailSamples: unsafe extern "system" fn(this: *mut c_void) -> u32,
}

/// IAudioProcessor interface
#[repr(C)]
pub struct IAudioProcessor {
    pub vtbl: *const IAudioProcessorVtbl,
}

impl AsRef<FUnknown> for IAudioProcessor {
    fn as_ref(&self) -> &FUnknown {
        unsafe { &*(self as *const _ as *const FUnknown) }
    }
}

// =============================================================================
// Module Entry
// =============================================================================

/// Module entry function type
pub type GetPluginFactoryFn = unsafe extern "C" fn() -> *mut IPluginFactory;

/// Module init function type (platform-specific)
pub type InitModuleFn = unsafe extern "C" fn() -> bool;

/// Module exit function type (platform-specific)
pub type ExitModuleFn = unsafe extern "C" fn() -> bool;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(K_INPUT, 0);
        assert_eq!(K_OUTPUT, 1);
        assert_eq!(K_AUDIO, 0);
        assert_eq!(K_EVENT, 1);
    }

    #[test]
    fn test_process_setup_default() {
        let setup = ProcessSetup::default();
        assert_eq!(setup.processMode, K_REALTIME);
        assert_eq!(setup.sampleRate, 44100.0);
    }

    #[test]
    fn test_audio_bus_buffers_default() {
        let buffers = AudioBusBuffers::default();
        assert_eq!(buffers.numChannels, 0);
        assert!(buffers.channelBuffers32.is_null());
    }

    #[test]
    fn test_process_data_default() {
        let data = ProcessData::default();
        assert_eq!(data.numSamples, 0);
        assert!(data.inputs.is_null());
        assert!(data.outputs.is_null());
    }

    #[test]
    fn test_bus_info_default() {
        let info = BusInfo::default();
        assert_eq!(info.mediaType, K_AUDIO);
        assert_eq!(info.channelCount, 2);
    }
}
