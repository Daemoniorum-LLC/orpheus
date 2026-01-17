/// Audio processor gRPC service implementation

use maestro_juce_bridge::{AudioEngine, CompressorParams, DelayParams, EQBand, ReverbParams};
use maestro_proto::audio::{
    audio_processor_server::AudioProcessor,
    AudioBuffer, EffectResponse, EffectUpdate, LatencyRequest, LatencyResponse,
    PluginRequest, PluginResponse,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, warn};

/// Default values for effect parameters
mod defaults {
    pub const EQ_FREQUENCIES: [f32; 4] = [100.0, 400.0, 2000.0, 8000.0];
    pub const EQ_GAIN: f32 = 0.0;
    pub const EQ_Q: f32 = 1.0;

    pub const COMP_THRESHOLD: f32 = -20.0;
    pub const COMP_RATIO: f32 = 4.0;
    pub const COMP_ATTACK: f32 = 10.0;
    pub const COMP_RELEASE: f32 = 100.0;
    pub const COMP_KNEE: f32 = 2.0;
    pub const COMP_MAKEUP: f32 = 0.0;

    pub const REVERB_ROOM_SIZE: f32 = 0.5;
    pub const REVERB_DECAY: f32 = 1.5;
    pub const REVERB_PRE_DELAY: f32 = 20.0;
    pub const REVERB_WET_DRY: f32 = 0.3;

    pub const DELAY_TIME: f32 = 250.0;
    pub const DELAY_FEEDBACK: f32 = 0.4;
    pub const DELAY_WET_DRY: f32 = 0.25;
}

/// Stored state for a single EQ band
#[derive(Clone, Copy)]
struct EQBandState {
    frequency: f32,
    gain: f32,
    q: f32,
}

impl Default for EQBandState {
    fn default() -> Self {
        Self {
            frequency: 1000.0,
            gain: defaults::EQ_GAIN,
            q: defaults::EQ_Q,
        }
    }
}

impl EQBandState {
    fn with_frequency(index: usize) -> Self {
        Self {
            frequency: defaults::EQ_FREQUENCIES.get(index).copied().unwrap_or(1000.0),
            gain: defaults::EQ_GAIN,
            q: defaults::EQ_Q,
        }
    }

    fn to_params(&self) -> EQBand {
        EQBand::new(self.frequency, self.gain, self.q)
    }
}

/// Stored state for compressor
#[derive(Clone, Copy)]
struct CompressorState {
    threshold: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    knee: f32,
    makeup_gain: f32,
}

impl Default for CompressorState {
    fn default() -> Self {
        Self {
            threshold: defaults::COMP_THRESHOLD,
            ratio: defaults::COMP_RATIO,
            attack_ms: defaults::COMP_ATTACK,
            release_ms: defaults::COMP_RELEASE,
            knee: defaults::COMP_KNEE,
            makeup_gain: defaults::COMP_MAKEUP,
        }
    }
}

impl CompressorState {
    fn to_params(&self) -> CompressorParams {
        CompressorParams::new(
            self.threshold,
            self.ratio,
            self.attack_ms,
            self.release_ms,
            self.knee,
            self.makeup_gain,
        )
    }
}

/// Stored state for reverb
#[derive(Clone, Copy)]
struct ReverbState {
    room_size: f32,
    decay: f32,
    pre_delay: f32,
    wet_dry: f32,
}

impl Default for ReverbState {
    fn default() -> Self {
        Self {
            room_size: defaults::REVERB_ROOM_SIZE,
            decay: defaults::REVERB_DECAY,
            pre_delay: defaults::REVERB_PRE_DELAY,
            wet_dry: defaults::REVERB_WET_DRY,
        }
    }
}

impl ReverbState {
    fn to_params(&self) -> ReverbParams {
        ReverbParams::new(self.room_size, self.decay, self.pre_delay, self.wet_dry)
    }
}

/// Stored state for delay
#[derive(Clone, Copy)]
struct DelayState {
    time_ms: f32,
    feedback: f32,
    wet_dry: f32,
}

impl Default for DelayState {
    fn default() -> Self {
        Self {
            time_ms: defaults::DELAY_TIME,
            feedback: defaults::DELAY_FEEDBACK,
            wet_dry: defaults::DELAY_WET_DRY,
        }
    }
}

impl DelayState {
    fn to_params(&self) -> DelayParams {
        DelayParams::new(self.time_ms, self.feedback, self.wet_dry)
    }
}

/// Audio processor state with JUCE engine and effect parameters
struct ProcessorState {
    engine: AudioEngine,
    // Effect parameter state (persisted across updates)
    eq_bands: [EQBandState; 4],
    compressor: CompressorState,
    reverb: ReverbState,
    delay: DelayState,
}

pub struct AudioProcessorService {
    state: Arc<RwLock<ProcessorState>>,
}

impl AudioProcessorService {
    pub fn new(sample_rate: i32, buffer_size: i32) -> Self {
        info!(
            "Initializing AudioProcessorService ({}Hz, {} samples)",
            sample_rate, buffer_size
        );

        // Create JUCE audio engine
        let engine = AudioEngine::new(sample_rate, buffer_size)
            .expect("Failed to create JUCE audio engine");

        info!("JUCE audio engine initialized successfully");

        // Initialize effect state with default frequencies per band
        let eq_bands = [
            EQBandState::with_frequency(0),
            EQBandState::with_frequency(1),
            EQBandState::with_frequency(2),
            EQBandState::with_frequency(3),
        ];

        Self {
            state: Arc::new(RwLock::new(ProcessorState {
                engine,
                eq_bands,
                compressor: CompressorState::default(),
                reverb: ReverbState::default(),
                delay: DelayState::default(),
            })),
        }
    }

    /// Process audio buffer through JUCE engine
    async fn process_buffer(&self, input: AudioBuffer) -> Result<AudioBuffer, Status> {
        debug!(
            "Processing buffer: {} samples, {} channels",
            input.num_samples, input.num_channels
        );

        // Convert bytes to f32 samples
        let input_samples: Vec<f32> = input
            .audio_data
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let mut output_samples = vec![0.0f32; input_samples.len()];

        // Process through JUCE engine
        let mut state = self.state.write().await;
        if let Err(e) = state.engine.process(&input_samples, &mut output_samples, input.num_channels) {
            error!("Audio processing error: {}", e);
            return Err(Status::internal(format!("Processing error: {}", e)));
        }

        // Convert back to bytes
        let output_data: Vec<u8> = output_samples
            .iter()
            .flat_map(|&sample| sample.to_le_bytes())
            .collect();

        Ok(AudioBuffer {
            sample_rate: input.sample_rate,
            num_channels: input.num_channels,
            num_samples: input.num_samples,
            audio_data: output_data,
            timestamp_us: input.timestamp_us,
        })
    }
}

#[tonic::async_trait]
impl AudioProcessor for AudioProcessorService {
    async fn process_audio(
        &self,
        request: Request<AudioBuffer>,
    ) -> Result<Response<AudioBuffer>, Status> {
        let input = request.into_inner();

        // Validate input
        if input.num_samples <= 0 {
            return Err(Status::invalid_argument("Invalid number of samples"));
        }

        if input.num_channels < 1 || input.num_channels > 2 {
            return Err(Status::invalid_argument(
                "Invalid number of channels (must be 1 or 2)",
            ));
        }

        // Process audio through JUCE engine
        let output = self.process_buffer(input).await?;

        Ok(Response::new(output))
    }

    type StreamAudioStream =
        tokio_stream::wrappers::ReceiverStream<Result<AudioBuffer, Status>>;

    async fn stream_audio(
        &self,
        request: Request<tonic::Streaming<AudioBuffer>>,
    ) -> Result<Response<Self::StreamAudioStream>, Status> {
        let mut in_stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        let service_clone = self.state.clone();

        // Spawn processing task
        tokio::spawn(async move {
            while let Ok(Some(input)) = in_stream.message().await {
                // Process through JUCE engine
                let mut state = service_clone.write().await;

                // Convert input
                let input_samples: Vec<f32> = input
                    .audio_data
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();

                let mut output_samples = vec![0.0f32; input_samples.len()];

                // Process
                if let Err(e) = state.engine.process(&input_samples, &mut output_samples, input.num_channels) {
                    warn!("Processing error in stream: {}", e);
                    continue;
                }

                // Convert output
                let output_data: Vec<u8> = output_samples
                    .iter()
                    .flat_map(|&sample| sample.to_le_bytes())
                    .collect();

                let output = AudioBuffer {
                    sample_rate: input.sample_rate,
                    num_channels: input.num_channels,
                    num_samples: input.num_samples,
                    audio_data: output_data,
                    timestamp_us: input.timestamp_us,
                };

                if tx.send(Ok(output)).await.is_err() {
                    warn!("Client disconnected during stream");
                    break;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }

    async fn load_plugin(
        &self,
        request: Request<PluginRequest>,
    ) -> Result<Response<PluginResponse>, Status> {
        let req = request.into_inner();
        info!("Loading plugin: {}", req.plugin_path);

        let mut state = self.state.write().await;

        match state.engine.load_plugin(&req.plugin_path) {
            Ok(_) => Ok(Response::new(PluginResponse {
                success: true,
                plugin_id: Some(req.plugin_path.clone()),
                error: None,
            })),
            Err(e) => Ok(Response::new(PluginResponse {
                success: false,
                plugin_id: None,
                error: Some(e.to_string()),
            })),
        }
    }

    async fn update_effect(
        &self,
        request: Request<EffectUpdate>,
    ) -> Result<Response<EffectResponse>, Status> {
        let update = request.into_inner();
        debug!(
            "Updating effect: {} / {} = {}",
            update.effect_id, update.parameter_id, update.value
        );

        let mut state = self.state.write().await;

        // Route to appropriate effect based on effect_id
        match update.effect_id.as_str() {
            "eq" => {
                // Parse parameter_id to get band number: "band0_gain", "band0_freq", etc.
                if let Some(band_str) = update.parameter_id.strip_prefix("band") {
                    if let Some((band_num, param)) = band_str.split_once('_') {
                        if let Ok(band) = band_num.parse::<usize>() {
                            if band >= 4 {
                                return Err(Status::invalid_argument("EQ band must be 0-3"));
                            }
                            // Update only the changed parameter, preserve others
                            match param {
                                "freq" => state.eq_bands[band].frequency = update.value,
                                "gain" => state.eq_bands[band].gain = update.value,
                                "q" => state.eq_bands[band].q = update.value,
                                _ => return Err(Status::invalid_argument("Unknown EQ parameter")),
                            };
                            // Apply the full band state to the engine
                            let _ = state.engine.set_eq_band(band, state.eq_bands[band].to_params());
                        }
                    }
                } else if update.parameter_id == "enabled" {
                    state.engine.set_eq_enabled(update.value > 0.5);
                }
            }
            "compressor" => {
                if update.parameter_id == "enabled" {
                    state.engine.set_compressor_enabled(update.value > 0.5);
                } else {
                    // Update only the changed parameter, preserve others
                    match update.parameter_id.as_str() {
                        "threshold" => state.compressor.threshold = update.value,
                        "ratio" => state.compressor.ratio = update.value,
                        "attack" => state.compressor.attack_ms = update.value,
                        "release" => state.compressor.release_ms = update.value,
                        "knee" => state.compressor.knee = update.value,
                        "makeup" => state.compressor.makeup_gain = update.value,
                        _ => return Err(Status::invalid_argument("Unknown compressor parameter")),
                    }
                    // Apply the full compressor state to the engine
                    state.engine.set_compressor(state.compressor.to_params());
                }
            }
            "reverb" => {
                if update.parameter_id == "enabled" {
                    state.engine.set_reverb_enabled(update.value > 0.5);
                } else {
                    // Update only the changed parameter, preserve others
                    match update.parameter_id.as_str() {
                        "room_size" => state.reverb.room_size = update.value,
                        "decay" => state.reverb.decay = update.value,
                        "pre_delay" => state.reverb.pre_delay = update.value,
                        "wet_dry" | "mix" => state.reverb.wet_dry = update.value,
                        _ => return Err(Status::invalid_argument("Unknown reverb parameter")),
                    }
                    // Apply the full reverb state to the engine
                    state.engine.set_reverb(state.reverb.to_params());
                }
            }
            "delay" => {
                if update.parameter_id == "enabled" {
                    state.engine.set_delay_enabled(update.value > 0.5);
                } else {
                    // Update only the changed parameter, preserve others
                    match update.parameter_id.as_str() {
                        "time" => state.delay.time_ms = update.value,
                        "feedback" => state.delay.feedback = update.value,
                        "wet_dry" | "mix" => state.delay.wet_dry = update.value,
                        _ => return Err(Status::invalid_argument("Unknown delay parameter")),
                    }
                    // Apply the full delay state to the engine
                    state.engine.set_delay(state.delay.to_params());
                }
            }
            _ => {
                return Err(Status::invalid_argument(format!(
                    "Unknown effect: {}",
                    update.effect_id
                )));
            }
        }

        Ok(Response::new(EffectResponse {
            success: true,
            error: None,
        }))
    }

    async fn get_latency(
        &self,
        _request: Request<LatencyRequest>,
    ) -> Result<Response<LatencyResponse>, Status> {
        let state = self.state.read().await;

        // Get latency from JUCE engine
        let latency_samples = state.engine.get_latency_samples();
        let latency_ms = state.engine.get_latency_ms();

        Ok(Response::new(LatencyResponse {
            latency_samples,
            latency_ms,
        }))
    }
}
