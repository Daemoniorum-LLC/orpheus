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

/// Audio processor state with JUCE engine
struct ProcessorState {
    engine: AudioEngine,
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

        Self {
            state: Arc::new(RwLock::new(ProcessorState { engine })),
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
                            // Get current band params (simplified - in production, store state)
                            let params = match param {
                                "freq" => EQBand::new(update.value, 0.0, 1.0),
                                "gain" => EQBand::new(1000.0, update.value, 1.0),
                                "q" => EQBand::new(1000.0, 0.0, update.value),
                                _ => return Err(Status::invalid_argument("Unknown EQ parameter")),
                            };
                            let _ = state.engine.set_eq_band(band, params);
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
                    // Update compressor params (simplified)
                    let params = CompressorParams::new(-20.0, 4.0, 10.0, 100.0, 2.0, 0.0);
                    state.engine.set_compressor(params);
                }
            }
            "reverb" => {
                if update.parameter_id == "enabled" {
                    state.engine.set_reverb_enabled(update.value > 0.5);
                } else {
                    let params = ReverbParams::new(0.5, 1.5, 20.0, 0.3);
                    state.engine.set_reverb(params);
                }
            }
            "delay" => {
                if update.parameter_id == "enabled" {
                    state.engine.set_delay_enabled(update.value > 0.5);
                } else {
                    let params = DelayParams::new(250.0, 0.4, 0.25);
                    state.engine.set_delay(params);
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
