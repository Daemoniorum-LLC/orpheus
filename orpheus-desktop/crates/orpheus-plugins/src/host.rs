//! Plugin host for loading and running plugins
//!
//! Provides the host environment for VST3/CLAP plugins including
//! audio processing, parameter management, and state handling.

use crate::clap::{ClapHost, ClapPluginLoader, ClapPluginInstance, ClapInputEvents, ClapOutputEvents};
use crate::clap::ffi::{clap_process, clap_audio_buffer, clap_process_status};
use crate::vst3::{Vst3Host, Vst3PluginLoader, Vst3PluginInstance, ProcessData, ProcessSetup, AudioBusBuffers};
use crate::vst3::ffi::{K_SAMPLE_32, K_REALTIME};
use crate::{Error, PluginFormat, PluginMetadata, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::ptr;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Configuration for the plugin host
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Buffer size in samples
    pub buffer_size: u32,
    /// Maximum number of audio inputs
    pub max_inputs: u32,
    /// Maximum number of audio outputs
    pub max_outputs: u32,
    /// Host name reported to plugins
    pub host_name: String,
    /// Host vendor reported to plugins
    pub host_vendor: String,
    /// Host version
    pub host_version: String,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 512,
            max_inputs: 2,
            max_outputs: 2,
            host_name: String::from("Orpheus"),
            host_vendor: String::from("Daemoniorum LLC"),
            host_version: String::from("0.1.0"),
        }
    }
}

impl HostConfig {
    /// Create config with specific sample rate
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    /// Create config with specific buffer size
    pub fn with_buffer_size(mut self, buffer_size: u32) -> Self {
        self.buffer_size = buffer_size;
        self
    }
}

/// Plugin parameter value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValue {
    /// Parameter ID
    pub id: u32,
    /// Parameter name
    pub name: String,
    /// Normalized value (0.0 - 1.0)
    pub normalized: f64,
    /// Display value string
    pub display: String,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Default value
    pub default: f64,
    /// Whether parameter is automatable
    pub automatable: bool,
}

impl ParameterValue {
    /// Create a new parameter value
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            normalized: 0.0,
            display: String::from("0.0"),
            min: 0.0,
            max: 1.0,
            default: 0.0,
            automatable: true,
        }
    }

    /// Get denormalized value
    pub fn value(&self) -> f64 {
        self.min + self.normalized * (self.max - self.min)
    }

    /// Set from denormalized value
    pub fn set_value(&mut self, value: f64) {
        self.normalized = (value - self.min) / (self.max - self.min);
        self.normalized = self.normalized.clamp(0.0, 1.0);
    }
}

/// Saved plugin state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    /// Plugin format
    pub format: PluginFormat,
    /// Plugin identifier (path or unique ID)
    pub plugin_id: String,
    /// Parameter values
    pub parameters: Vec<ParameterValue>,
    /// Opaque plugin state data (format-specific)
    pub chunk_data: Option<Vec<u8>>,
    /// Bypass state
    pub bypassed: bool,
}

impl PluginState {
    /// Create new empty state
    pub fn new(format: PluginFormat, plugin_id: impl Into<String>) -> Self {
        Self {
            format,
            plugin_id: plugin_id.into(),
            parameters: Vec::new(),
            chunk_data: None,
            bypassed: false,
        }
    }
}

/// Internal state for real CLAP plugin
struct RealClapPlugin {
    /// The CLAP plugin loader (keeps library alive)
    _loader: ClapPluginLoader,
    /// The host interface
    host: ClapHost,
    /// The actual plugin instance
    instance: ClapPluginInstance,
    /// Input events for processing
    input_events: ClapInputEvents,
    /// Output events for processing
    output_events: ClapOutputEvents,
    /// Cached audio buffer pointers for processing
    input_buffer_ptrs: Vec<*mut f32>,
    output_buffer_ptrs: Vec<*mut f32>,
}

// Safety: RealClapPlugin is accessed only from the audio thread through proper locking.
// The plugin instance pointer is valid for the lifetime of the loader.
unsafe impl Send for RealClapPlugin {}
unsafe impl Sync for RealClapPlugin {}

/// Internal state for real VST3 plugin
struct RealVst3Plugin {
    /// The VST3 plugin loader (keeps library alive)
    _loader: Vst3PluginLoader,
    /// The host interface
    _host: Vst3Host,
    /// The actual plugin instance
    instance: Vst3PluginInstance,
    /// Cached input audio bus buffers
    input_bus: AudioBusBuffers,
    /// Cached output audio bus buffers
    output_bus: AudioBusBuffers,
    /// Input buffer pointers
    input_buffer_ptrs: Vec<*mut f32>,
    /// Output buffer pointers
    output_buffer_ptrs: Vec<*mut f32>,
}

// Safety: RealVst3Plugin is accessed only from the audio thread through proper locking.
unsafe impl Send for RealVst3Plugin {}
unsafe impl Sync for RealVst3Plugin {}

/// A loaded plugin instance
pub struct PluginInstance {
    /// Unique instance ID
    id: Uuid,
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Host configuration
    config: HostConfig,
    /// Current parameters
    parameters: RwLock<HashMap<u32, ParameterValue>>,
    /// Whether plugin is bypassed
    bypassed: RwLock<bool>,
    /// Whether plugin is active
    active: RwLock<bool>,
    /// Real CLAP plugin (if loaded)
    clap_plugin: RwLock<Option<RealClapPlugin>>,
    /// Real VST3 plugin (if loaded)
    vst3_plugin: RwLock<Option<RealVst3Plugin>>,
}

impl PluginInstance {
    /// Create a new plugin instance (internal, mock mode)
    fn new(metadata: PluginMetadata, config: HostConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            metadata,
            config,
            parameters: RwLock::new(HashMap::new()),
            bypassed: RwLock::new(false),
            active: RwLock::new(false),
            clap_plugin: RwLock::new(None),
            vst3_plugin: RwLock::new(None),
        }
    }

    /// Create a new plugin instance with a real CLAP plugin
    fn new_with_clap(
        metadata: PluginMetadata,
        config: HostConfig,
        loader: ClapPluginLoader,
        host: ClapHost,
        instance: ClapPluginInstance,
    ) -> Self {
        let input_events = ClapInputEvents::new();
        let output_events = ClapOutputEvents::new();

        Self {
            id: Uuid::new_v4(),
            metadata,
            config,
            parameters: RwLock::new(HashMap::new()),
            bypassed: RwLock::new(false),
            active: RwLock::new(false),
            clap_plugin: RwLock::new(Some(RealClapPlugin {
                _loader: loader,
                host,
                instance,
                input_events,
                output_events,
                input_buffer_ptrs: Vec::new(),
                output_buffer_ptrs: Vec::new(),
            })),
            vst3_plugin: RwLock::new(None),
        }
    }

    /// Create a new plugin instance with a real VST3 plugin
    fn new_with_vst3(
        metadata: PluginMetadata,
        config: HostConfig,
        loader: Vst3PluginLoader,
        host: Vst3Host,
        instance: Vst3PluginInstance,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            metadata,
            config,
            parameters: RwLock::new(HashMap::new()),
            bypassed: RwLock::new(false),
            active: RwLock::new(false),
            clap_plugin: RwLock::new(None),
            vst3_plugin: RwLock::new(Some(RealVst3Plugin {
                _loader: loader,
                _host: host,
                instance,
                input_bus: AudioBusBuffers::default(),
                output_bus: AudioBusBuffers::default(),
                input_buffer_ptrs: Vec::new(),
                output_buffer_ptrs: Vec::new(),
            })),
        }
    }

    /// Check if this is a real plugin (not mock)
    pub fn is_real(&self) -> bool {
        self.clap_plugin.read().is_some() || self.vst3_plugin.read().is_some()
    }

    /// Get instance ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get plugin metadata
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// Get plugin name
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Get plugin format
    pub fn format(&self) -> PluginFormat {
        self.metadata.format
    }

    /// Check if plugin is active
    pub fn is_active(&self) -> bool {
        *self.active.read()
    }

    /// Check if plugin is bypassed
    pub fn is_bypassed(&self) -> bool {
        *self.bypassed.read()
    }

    /// Set bypass state
    pub fn set_bypassed(&self, bypassed: bool) {
        *self.bypassed.write() = bypassed;
    }

    /// Activate the plugin for processing
    pub fn activate(&self) -> Result<()> {
        if *self.active.read() {
            return Ok(());
        }

        debug!("Activating plugin: {}", self.name());

        // If we have a real CLAP plugin, activate it
        if let Some(ref mut clap) = *self.clap_plugin.write() {
            let sample_rate = self.config.sample_rate as f64;
            let min_frames = 1;
            let max_frames = self.config.buffer_size;

            // Activate the plugin
            clap.instance.activate(sample_rate, min_frames, max_frames)?;

            // Start processing
            clap.instance.start_processing()?;

            info!("Real CLAP plugin activated: {} @ {}Hz", self.name(), sample_rate);
        }

        // If we have a real VST3 plugin, activate it
        if let Some(ref mut vst3) = *self.vst3_plugin.write() {
            let sample_rate = self.config.sample_rate as f64;
            let max_samples = self.config.buffer_size as i32;

            // Setup processing parameters
            let setup = ProcessSetup {
                processMode: K_REALTIME,
                symbolicSampleSize: K_SAMPLE_32,
                maxSamplesPerBlock: max_samples,
                sampleRate: sample_rate,
            };

            // Setup and activate
            vst3.instance.setup_processing(&setup)?;
            vst3.instance.set_active(true)?;
            vst3.instance.set_processing(true)?;

            info!("Real VST3 plugin activated: {} @ {}Hz", self.name(), sample_rate);
        }

        *self.active.write() = true;
        Ok(())
    }

    /// Deactivate the plugin
    pub fn deactivate(&self) -> Result<()> {
        if !*self.active.read() {
            return Ok(());
        }

        debug!("Deactivating plugin: {}", self.name());

        // If we have a real CLAP plugin, deactivate it
        if let Some(ref mut clap) = *self.clap_plugin.write() {
            clap.instance.stop_processing();
            clap.instance.deactivate();
            info!("Real CLAP plugin deactivated: {}", self.name());
        }

        // If we have a real VST3 plugin, deactivate it
        if let Some(ref mut vst3) = *self.vst3_plugin.write() {
            let _ = vst3.instance.set_processing(false);
            let _ = vst3.instance.set_active(false);
            info!("Real VST3 plugin deactivated: {}", self.name());
        }

        *self.active.write() = false;
        Ok(())
    }

    /// Get number of audio inputs
    pub fn num_inputs(&self) -> u32 {
        self.metadata.num_inputs
    }

    /// Get number of audio outputs
    pub fn num_outputs(&self) -> u32 {
        self.metadata.num_outputs
    }

    /// Get all parameters
    pub fn parameters(&self) -> Vec<ParameterValue> {
        self.parameters.read().values().cloned().collect()
    }

    /// Get a parameter value
    pub fn get_parameter(&self, id: u32) -> Option<ParameterValue> {
        self.parameters.read().get(&id).cloned()
    }

    /// Set a parameter value (normalized 0.0 - 1.0)
    pub fn set_parameter(&self, id: u32, normalized: f64) -> Result<()> {
        let mut params = self.parameters.write();
        if let Some(param) = params.get_mut(&id) {
            param.normalized = normalized.clamp(0.0, 1.0);
            Ok(())
        } else {
            Err(Error::ParameterError(format!("Parameter {} not found", id)))
        }
    }

    /// Process audio through the plugin
    ///
    /// # Arguments
    /// * `inputs` - Input audio buffers (one per channel)
    /// * `outputs` - Output audio buffers (one per channel, will be overwritten)
    pub fn process(&self, inputs: &[&[f32]], outputs: &mut [&mut [f32]]) -> Result<()> {
        if !*self.active.read() {
            return Err(Error::ProcessingError("Plugin not active".to_string()));
        }

        // If bypassed, copy input to output
        if *self.bypassed.read() {
            for (i, output) in outputs.iter_mut().enumerate() {
                if i < inputs.len() {
                    output.copy_from_slice(inputs[i]);
                } else {
                    output.fill(0.0);
                }
            }
            return Ok(());
        }

        // Try to use real CLAP plugin if available
        let mut clap_guard = self.clap_plugin.write();
        if let Some(ref mut clap) = *clap_guard {
            // Get frame count (assume all buffers have same length)
            let frames_count = if !inputs.is_empty() {
                inputs[0].len() as u32
            } else if !outputs.is_empty() {
                outputs[0].len() as u32
            } else {
                return Ok(());
            };

            // Prepare input audio buffer pointers
            clap.input_buffer_ptrs.clear();
            for input in inputs.iter() {
                // Cast away const - CLAP uses mutable pointers but we won't modify input
                clap.input_buffer_ptrs.push(input.as_ptr() as *mut f32);
            }

            // Prepare output audio buffer pointers
            clap.output_buffer_ptrs.clear();
            for output in outputs.iter_mut() {
                clap.output_buffer_ptrs.push(output.as_mut_ptr());
            }

            // Create input audio buffer descriptor
            let input_buffer = clap_audio_buffer {
                data32: if clap.input_buffer_ptrs.is_empty() {
                    ptr::null_mut()
                } else {
                    clap.input_buffer_ptrs.as_mut_ptr()
                },
                data64: ptr::null_mut(),
                channel_count: inputs.len() as u32,
                latency: 0,
                constant_mask: 0,
            };

            // Create output audio buffer descriptor
            let mut output_buffer = clap_audio_buffer {
                data32: if clap.output_buffer_ptrs.is_empty() {
                    ptr::null_mut()
                } else {
                    clap.output_buffer_ptrs.as_mut_ptr()
                },
                data64: ptr::null_mut(),
                channel_count: outputs.len() as u32,
                latency: 0,
                constant_mask: 0,
            };

            // Clear input/output events
            clap.input_events.clear();
            clap.output_events.clear();

            // Create process struct
            let process = clap_process {
                steady_time: -1, // Unknown
                frames_count,
                transport: ptr::null(),
                audio_inputs: &input_buffer as *const _,
                audio_outputs: &mut output_buffer as *mut _,
                audio_inputs_count: 1,
                audio_outputs_count: 1,
                in_events: clap.input_events.as_ptr(),
                out_events: clap.output_events.as_ptr(),
            };

            // Call the plugin's process function
            let status = unsafe { clap.instance.process(&process) };

            match status {
                clap_process_status::CLAP_PROCESS_ERROR => {
                    return Err(Error::ProcessingError("CLAP process error".to_string()));
                }
                clap_process_status::CLAP_PROCESS_SLEEP => {
                    // Plugin wants to sleep, fill output with silence
                    for output in outputs.iter_mut() {
                        output.fill(0.0);
                    }
                }
                _ => {
                    // CLAP_PROCESS_CONTINUE, CLAP_PROCESS_CONTINUE_IF_NOT_QUIET, CLAP_PROCESS_TAIL
                    // Processing succeeded, output buffers are already filled
                }
            }

            return Ok(());
        }
        drop(clap_guard);

        // Try to use real VST3 plugin if available
        let mut vst3_guard = self.vst3_plugin.write();
        if let Some(ref mut vst3) = *vst3_guard {
            // Get frame count
            let num_samples = if !inputs.is_empty() {
                inputs[0].len() as i32
            } else if !outputs.is_empty() {
                outputs[0].len() as i32
            } else {
                return Ok(());
            };

            // Prepare input buffer pointers
            vst3.input_buffer_ptrs.clear();
            for input in inputs.iter() {
                vst3.input_buffer_ptrs.push(input.as_ptr() as *mut f32);
            }

            // Prepare output buffer pointers
            vst3.output_buffer_ptrs.clear();
            for output in outputs.iter_mut() {
                vst3.output_buffer_ptrs.push(output.as_mut_ptr());
            }

            // Setup input bus
            vst3.input_bus = AudioBusBuffers {
                numChannels: inputs.len() as i32,
                silenceFlags: 0,
                channelBuffers32: if vst3.input_buffer_ptrs.is_empty() {
                    ptr::null_mut()
                } else {
                    vst3.input_buffer_ptrs.as_mut_ptr()
                },
                channelBuffers64: ptr::null_mut(),
            };

            // Setup output bus
            vst3.output_bus = AudioBusBuffers {
                numChannels: outputs.len() as i32,
                silenceFlags: 0,
                channelBuffers32: if vst3.output_buffer_ptrs.is_empty() {
                    ptr::null_mut()
                } else {
                    vst3.output_buffer_ptrs.as_mut_ptr()
                },
                channelBuffers64: ptr::null_mut(),
            };

            // Create process data
            let mut process_data = ProcessData {
                processMode: K_REALTIME,
                symbolicSampleSize: K_SAMPLE_32,
                numSamples: num_samples,
                numInputs: 1,
                numOutputs: 1,
                inputs: &mut vst3.input_bus as *mut _,
                outputs: &mut vst3.output_bus as *mut _,
                inputParameterChanges: ptr::null_mut(),
                outputParameterChanges: ptr::null_mut(),
                inputEvents: ptr::null_mut(),
                outputEvents: ptr::null_mut(),
                processContext: ptr::null_mut(),
            };

            // Process audio
            unsafe {
                vst3.instance.process(&mut process_data)?;
            }

            return Ok(());
        }
        drop(vst3_guard);

        // Fallback: Mock processing for unsupported plugins (pass-through with marker)
        for (i, output) in outputs.iter_mut().enumerate() {
            if i < inputs.len() {
                for (out_sample, in_sample) in output.iter_mut().zip(inputs[i].iter()) {
                    *out_sample = *in_sample * 0.999;
                }
            } else {
                output.fill(0.0);
            }
        }

        Ok(())
    }

    /// Process MIDI events (for instrument plugins)
    pub fn process_midi(&self, _events: &[MidiEvent]) -> Result<()> {
        if !self.metadata.has_midi_input() {
            return Ok(());
        }

        // Mock MIDI processing
        debug!("Processing MIDI events for: {}", self.name());
        Ok(())
    }

    /// Get current state for saving
    pub fn get_state(&self) -> PluginState {
        let params = self.parameters();
        PluginState {
            format: self.format(),
            plugin_id: self.metadata.path.display().to_string(),
            parameters: params,
            chunk_data: None, // Real implementation would get plugin's internal state
            bypassed: *self.bypassed.read(),
        }
    }

    /// Restore state
    pub fn set_state(&self, state: &PluginState) -> Result<()> {
        *self.bypassed.write() = state.bypassed;

        for param in &state.parameters {
            let _ = self.set_parameter(param.id, param.normalized);
        }

        Ok(())
    }
}

impl std::fmt::Debug for PluginInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginInstance")
            .field("id", &self.id)
            .field("name", &self.metadata.name)
            .field("format", &self.metadata.format)
            .field("active", &*self.active.read())
            .field("bypassed", &*self.bypassed.read())
            .finish()
    }
}

/// Simple MIDI event for plugin processing
#[derive(Debug, Clone, Copy)]
pub struct MidiEvent {
    /// Sample offset within the buffer
    pub offset: u32,
    /// MIDI data bytes (up to 3)
    pub data: [u8; 3],
    /// Number of valid bytes
    pub size: u8,
}

impl MidiEvent {
    /// Create a note on event
    pub fn note_on(offset: u32, channel: u8, note: u8, velocity: u8) -> Self {
        Self {
            offset,
            data: [0x90 | (channel & 0x0F), note & 0x7F, velocity & 0x7F],
            size: 3,
        }
    }

    /// Create a note off event
    pub fn note_off(offset: u32, channel: u8, note: u8, velocity: u8) -> Self {
        Self {
            offset,
            data: [0x80 | (channel & 0x0F), note & 0x7F, velocity & 0x7F],
            size: 3,
        }
    }

    /// Create a control change event
    pub fn control_change(offset: u32, channel: u8, controller: u8, value: u8) -> Self {
        Self {
            offset,
            data: [0xB0 | (channel & 0x0F), controller & 0x7F, value & 0x7F],
            size: 3,
        }
    }
}

/// Plugin host for managing plugin instances
pub struct PluginHost {
    /// Host configuration
    config: HostConfig,
    /// Loaded plugin instances
    instances: RwLock<HashMap<Uuid, Arc<PluginInstance>>>,
}

impl PluginHost {
    /// Create a new plugin host
    pub fn new(config: HostConfig) -> Self {
        info!(
            "Creating plugin host: {}Hz, {} samples",
            config.sample_rate, config.buffer_size
        );

        Self {
            config,
            instances: RwLock::new(HashMap::new()),
        }
    }

    /// Get host configuration
    pub fn config(&self) -> &HostConfig {
        &self.config
    }

    /// Load a plugin from path
    pub fn load(&self, path: &Path) -> Result<Arc<PluginInstance>> {
        if !path.exists() {
            return Err(Error::NotFound(path.display().to_string()));
        }

        let format = PluginFormat::from_path(path)
            .ok_or_else(|| Error::UnsupportedFormat(path.display().to_string()))?;

        info!("Loading plugin: {} ({})", path.display(), format);

        // Create metadata from path
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let metadata = PluginMetadata::new(name.clone(), path.to_path_buf(), format);

        // Try to load the real plugin based on format
        let instance = match format {
            PluginFormat::Clap => {
                match self.load_clap_plugin(path, metadata) {
                    Ok(inst) => inst,
                    Err(e) => {
                        warn!("Failed to load real CLAP plugin, falling back to mock: {}", e);
                        let fallback_metadata = PluginMetadata::new(name, path.to_path_buf(), format);
                        Arc::new(PluginInstance::new(fallback_metadata, self.config.clone()))
                    }
                }
            }
            PluginFormat::Vst3 => {
                match self.load_vst3_plugin(path, metadata) {
                    Ok(inst) => inst,
                    Err(e) => {
                        warn!("Failed to load real VST3 plugin, falling back to mock: {}", e);
                        let fallback_metadata = PluginMetadata::new(name, path.to_path_buf(), format);
                        Arc::new(PluginInstance::new(fallback_metadata, self.config.clone()))
                    }
                }
            }
        };

        // Store in instances map
        self.instances.write().insert(instance.id(), instance.clone());

        Ok(instance)
    }

    /// Load a real CLAP plugin
    fn load_clap_plugin(&self, path: &Path, metadata: PluginMetadata) -> Result<Arc<PluginInstance>> {
        // Load the plugin library
        let loader = ClapPluginLoader::load(path)?;

        // Get the first plugin descriptor (most CLAP bundles have one plugin)
        let descriptors = loader.descriptors();
        if descriptors.is_empty() {
            return Err(Error::LoadError("No plugins found in CLAP bundle".to_string()));
        }

        let desc = &descriptors[0];
        info!(
            "Found CLAP plugin: {} by {} ({})",
            desc.name, desc.vendor, desc.id
        );

        // Update metadata with real info from plugin
        let mut real_metadata = metadata;
        real_metadata.name = desc.name.clone();
        real_metadata.vendor = desc.vendor.clone();
        real_metadata.version = desc.version.clone();

        // Create host interface
        let host = ClapHost::new(self.config.clone());

        // Create plugin instance
        let clap_instance = loader.create_instance(&desc.id, &host)?;

        // Create the plugin instance wrapper
        let instance = Arc::new(PluginInstance::new_with_clap(
            real_metadata,
            self.config.clone(),
            loader,
            host,
            clap_instance,
        ));

        info!("Successfully loaded real CLAP plugin: {}", instance.name());

        Ok(instance)
    }

    /// Load a real VST3 plugin
    fn load_vst3_plugin(&self, path: &Path, metadata: PluginMetadata) -> Result<Arc<PluginInstance>> {
        // Load the plugin library
        let loader = Vst3PluginLoader::load(path)?;

        // Get factory info
        let factory_info = loader.get_factory_info()?;
        info!("VST3 factory: {}", factory_info.vendor);

        // Get the first audio processor class
        let classes = loader.classes();
        let audio_class = classes
            .iter()
            .find(|c| c.category.contains("Audio"))
            .or_else(|| classes.first())
            .ok_or_else(|| Error::LoadError("No audio processor found in VST3".to_string()))?;

        info!(
            "Found VST3 plugin: {} ({})",
            audio_class.name, audio_class.category
        );

        // Update metadata with real info from plugin
        let mut real_metadata = metadata;
        real_metadata.name = audio_class.name.clone();
        real_metadata.vendor = factory_info.vendor.clone();

        // Create host interface
        let host = Vst3Host::new(self.config.clone());

        // Create plugin instance
        let mut vst3_instance = loader.create_instance(&audio_class.cid, &host)?;

        // Initialize the instance
        vst3_instance.initialize(&host)?;

        // Create the plugin instance wrapper
        let instance = Arc::new(PluginInstance::new_with_vst3(
            real_metadata,
            self.config.clone(),
            loader,
            host,
            vst3_instance,
        ));

        info!("Successfully loaded real VST3 plugin: {}", instance.name());

        Ok(instance)
    }

    /// Load a plugin from metadata
    pub fn load_from_metadata(&self, metadata: &PluginMetadata) -> Result<Arc<PluginInstance>> {
        self.load(&metadata.path)
    }

    /// Unload a plugin instance
    pub fn unload(&self, id: Uuid) -> Result<()> {
        let instance = self
            .instances
            .write()
            .remove(&id)
            .ok_or_else(|| Error::NotFound(format!("Plugin instance {} not found", id)))?;

        // Deactivate before unloading
        instance.deactivate()?;

        info!("Unloaded plugin: {}", instance.name());
        Ok(())
    }

    /// Get a loaded plugin instance
    pub fn get(&self, id: Uuid) -> Option<Arc<PluginInstance>> {
        self.instances.read().get(&id).cloned()
    }

    /// Get all loaded plugin instances
    pub fn instances(&self) -> Vec<Arc<PluginInstance>> {
        self.instances.read().values().cloned().collect()
    }

    /// Number of loaded plugins
    pub fn count(&self) -> usize {
        self.instances.read().len()
    }

    /// Unload all plugins
    pub fn unload_all(&self) -> Result<()> {
        let ids: Vec<Uuid> = self.instances.read().keys().cloned().collect();

        for id in ids {
            self.unload(id)?;
        }

        Ok(())
    }

    /// Update host configuration
    pub fn set_config(&mut self, config: HostConfig) {
        info!(
            "Updating host config: {}Hz, {} samples",
            config.sample_rate, config.buffer_size
        );
        self.config = config;
    }
}

impl Drop for PluginHost {
    fn drop(&mut self) {
        if let Err(e) = self.unload_all() {
            warn!("Error unloading plugins on host drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_host_config_default() {
        let config = HostConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.buffer_size, 512);
        assert_eq!(config.host_name, "Orpheus");
    }

    #[test]
    fn test_host_config_builder() {
        let config = HostConfig::default()
            .with_sample_rate(96000)
            .with_buffer_size(1024);

        assert_eq!(config.sample_rate, 96000);
        assert_eq!(config.buffer_size, 1024);
    }

    #[test]
    fn test_parameter_value_new() {
        let param = ParameterValue::new(0, "Gain");
        assert_eq!(param.id, 0);
        assert_eq!(param.name, "Gain");
        assert_eq!(param.normalized, 0.0);
    }

    #[test]
    fn test_parameter_value_denormalization() {
        let mut param = ParameterValue::new(0, "Frequency");
        param.min = 20.0;
        param.max = 20000.0;
        param.normalized = 0.5;

        let value = param.value();
        assert!((value - 10010.0).abs() < 0.1);
    }

    #[test]
    fn test_parameter_value_set_value() {
        let mut param = ParameterValue::new(0, "Pan");
        param.min = -1.0;
        param.max = 1.0;

        param.set_value(0.0);
        assert!((param.normalized - 0.5).abs() < 0.001);

        param.set_value(-1.0);
        assert!((param.normalized - 0.0).abs() < 0.001);

        param.set_value(1.0);
        assert!((param.normalized - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_plugin_state_new() {
        let state = PluginState::new(PluginFormat::Vst3, "test-plugin");
        assert_eq!(state.format, PluginFormat::Vst3);
        assert_eq!(state.plugin_id, "test-plugin");
        assert!(!state.bypassed);
        assert!(state.parameters.is_empty());
    }

    #[test]
    fn test_midi_event_note_on() {
        let event = MidiEvent::note_on(0, 0, 60, 100);
        assert_eq!(event.data[0], 0x90);
        assert_eq!(event.data[1], 60);
        assert_eq!(event.data[2], 100);
        assert_eq!(event.size, 3);
    }

    #[test]
    fn test_midi_event_note_off() {
        let event = MidiEvent::note_off(10, 1, 64, 64);
        assert_eq!(event.data[0], 0x81);
        assert_eq!(event.data[1], 64);
        assert_eq!(event.data[2], 64);
        assert_eq!(event.offset, 10);
    }

    #[test]
    fn test_midi_event_control_change() {
        let event = MidiEvent::control_change(0, 0, 1, 127);
        assert_eq!(event.data[0], 0xB0);
        assert_eq!(event.data[1], 1);
        assert_eq!(event.data[2], 127);
    }

    #[test]
    fn test_plugin_host_new() {
        let host = PluginHost::new(HostConfig::default());
        assert_eq!(host.count(), 0);
        assert_eq!(host.config().sample_rate, 48000);
    }

    #[test]
    fn test_plugin_host_load_not_found() {
        let host = PluginHost::new(HostConfig::default());
        let result = host.load(Path::new("/nonexistent/plugin.vst3"));

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotFound(_)));
    }

    #[test]
    fn test_plugin_host_load_unsupported() {
        let temp = tempdir().unwrap();
        let dll_path = temp.path().join("plugin.dll");
        std::fs::write(&dll_path, b"fake").unwrap();

        let host = PluginHost::new(HostConfig::default());
        let result = host.load(&dll_path);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::UnsupportedFormat(_)));
    }

    #[test]
    fn test_plugin_host_load_mock_plugin() {
        let temp = tempdir().unwrap();
        let vst3_path = temp.path().join("TestPlugin.vst3");
        std::fs::create_dir(&vst3_path).unwrap();

        let host = PluginHost::new(HostConfig::default());
        let instance = host.load(&vst3_path).unwrap();

        assert_eq!(instance.name(), "TestPlugin");
        assert_eq!(instance.format(), PluginFormat::Vst3);
        assert!(!instance.is_active());
        assert!(!instance.is_bypassed());
        assert_eq!(host.count(), 1);
    }

    #[test]
    fn test_plugin_instance_activate_deactivate() {
        let temp = tempdir().unwrap();
        let vst3_path = temp.path().join("Plugin.vst3");
        std::fs::create_dir(&vst3_path).unwrap();

        let host = PluginHost::new(HostConfig::default());
        let instance = host.load(&vst3_path).unwrap();

        assert!(!instance.is_active());

        instance.activate().unwrap();
        assert!(instance.is_active());

        instance.deactivate().unwrap();
        assert!(!instance.is_active());
    }

    #[test]
    fn test_plugin_instance_bypass() {
        let temp = tempdir().unwrap();
        let vst3_path = temp.path().join("Plugin.vst3");
        std::fs::create_dir(&vst3_path).unwrap();

        let host = PluginHost::new(HostConfig::default());
        let instance = host.load(&vst3_path).unwrap();

        assert!(!instance.is_bypassed());

        instance.set_bypassed(true);
        assert!(instance.is_bypassed());

        instance.set_bypassed(false);
        assert!(!instance.is_bypassed());
    }

    #[test]
    fn test_plugin_instance_process() {
        let temp = tempdir().unwrap();
        let vst3_path = temp.path().join("Plugin.vst3");
        std::fs::create_dir(&vst3_path).unwrap();

        let host = PluginHost::new(HostConfig::default());
        let instance = host.load(&vst3_path).unwrap();

        instance.activate().unwrap();

        let input = vec![0.5f32; 512];
        let mut output = vec![0.0f32; 512];

        instance
            .process(&[&input], &mut [&mut output])
            .unwrap();

        // Output should be slightly modified (mock processing)
        assert!(output.iter().any(|&s| s != 0.0));
        assert!(output.iter().all(|&s| (s - 0.5 * 0.999).abs() < 0.001));
    }

    #[test]
    fn test_plugin_instance_process_bypassed() {
        let temp = tempdir().unwrap();
        let vst3_path = temp.path().join("Plugin.vst3");
        std::fs::create_dir(&vst3_path).unwrap();

        let host = PluginHost::new(HostConfig::default());
        let instance = host.load(&vst3_path).unwrap();

        instance.activate().unwrap();
        instance.set_bypassed(true);

        let input = vec![0.5f32; 512];
        let mut output = vec![0.0f32; 512];

        instance
            .process(&[&input], &mut [&mut output])
            .unwrap();

        // Bypassed: output should equal input exactly
        assert!(output.iter().all(|&s| (s - 0.5).abs() < 0.0001));
    }

    #[test]
    fn test_plugin_instance_process_not_active() {
        let temp = tempdir().unwrap();
        let vst3_path = temp.path().join("Plugin.vst3");
        std::fs::create_dir(&vst3_path).unwrap();

        let host = PluginHost::new(HostConfig::default());
        let instance = host.load(&vst3_path).unwrap();

        let input = vec![0.5f32; 512];
        let mut output = vec![0.0f32; 512];

        let result = instance.process(&[&input], &mut [&mut output]);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ProcessingError(_)));
    }

    #[test]
    fn test_plugin_instance_state() {
        let temp = tempdir().unwrap();
        let vst3_path = temp.path().join("Plugin.vst3");
        std::fs::create_dir(&vst3_path).unwrap();

        let host = PluginHost::new(HostConfig::default());
        let instance = host.load(&vst3_path).unwrap();

        instance.set_bypassed(true);

        let state = instance.get_state();
        assert!(state.bypassed);
        assert_eq!(state.format, PluginFormat::Vst3);
    }

    #[test]
    fn test_plugin_instance_restore_state() {
        let temp = tempdir().unwrap();
        let vst3_path = temp.path().join("Plugin.vst3");
        std::fs::create_dir(&vst3_path).unwrap();

        let host = PluginHost::new(HostConfig::default());
        let instance = host.load(&vst3_path).unwrap();

        let mut state = PluginState::new(PluginFormat::Vst3, "test");
        state.bypassed = true;

        instance.set_state(&state).unwrap();
        assert!(instance.is_bypassed());
    }

    #[test]
    fn test_plugin_host_unload() {
        let temp = tempdir().unwrap();
        let vst3_path = temp.path().join("Plugin.vst3");
        std::fs::create_dir(&vst3_path).unwrap();

        let host = PluginHost::new(HostConfig::default());
        let instance = host.load(&vst3_path).unwrap();
        let id = instance.id();

        assert_eq!(host.count(), 1);

        host.unload(id).unwrap();
        assert_eq!(host.count(), 0);
    }

    #[test]
    fn test_plugin_host_unload_all() {
        let temp = tempdir().unwrap();

        let host = PluginHost::new(HostConfig::default());

        for i in 0..3 {
            let path = temp.path().join(format!("Plugin{}.vst3", i));
            std::fs::create_dir(&path).unwrap();
            host.load(&path).unwrap();
        }

        assert_eq!(host.count(), 3);

        host.unload_all().unwrap();
        assert_eq!(host.count(), 0);
    }

    #[test]
    fn test_plugin_host_get_instance() {
        let temp = tempdir().unwrap();
        let vst3_path = temp.path().join("Plugin.vst3");
        std::fs::create_dir(&vst3_path).unwrap();

        let host = PluginHost::new(HostConfig::default());
        let instance = host.load(&vst3_path).unwrap();
        let id = instance.id();

        let retrieved = host.get(id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id(), id);

        let unknown = host.get(Uuid::new_v4());
        assert!(unknown.is_none());
    }

    #[test]
    fn test_plugin_host_instances() {
        let temp = tempdir().unwrap();

        let host = PluginHost::new(HostConfig::default());

        for i in 0..2 {
            let path = temp.path().join(format!("Plugin{}.vst3", i));
            std::fs::create_dir(&path).unwrap();
            host.load(&path).unwrap();
        }

        let instances = host.instances();
        assert_eq!(instances.len(), 2);
    }
}
