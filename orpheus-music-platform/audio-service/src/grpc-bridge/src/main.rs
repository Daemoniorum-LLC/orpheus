//! Maestro Audio Service - gRPC Bridge
//!
//! This service acts as a thin gRPC wrapper around the Sigil DSP engine.
//! It translates gRPC requests to JSON-RPC calls to the Sigil process.

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use futures::Stream;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, Mutex};
use tonic::{transport::Server, Request, Response, Status};
use tracing::{debug, error, info, warn};

pub mod audio_proto {
    tonic::include_proto!("maestro.audio");
}

use audio_proto::audio_processor_server::{AudioProcessor, AudioProcessorServer};
use audio_proto::*;

// ============================================================================
// JSON-RPC Types for Sigil Communication
// ============================================================================

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

// ============================================================================
// Audio Data Conversion (Safe)
// ============================================================================

/// Safely convert bytes to f32 samples, returning error if data is malformed
fn bytes_to_samples(data: &[u8]) -> Result<Vec<f32>, Status> {
    if data.len() % 4 != 0 {
        return Err(Status::invalid_argument(format!(
            "Audio data length {} is not a multiple of 4 bytes (float32)",
            data.len()
        )));
    }

    Ok(data
        .chunks(4)
        .map(|chunk| {
            // chunks() guarantees exactly 4 bytes when len % 4 == 0
            f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
        })
        .collect())
}

/// Convert f32 samples back to bytes
fn samples_to_bytes(samples: &[f32]) -> Vec<u8> {
    samples.iter().flat_map(|f| f.to_le_bytes()).collect()
}

// ============================================================================
// Sigil Process Manager
// ============================================================================

struct SigilProcess {
    child: Child,
    stdin: ChildStdin,
    stdout_reader: BufReader<ChildStdout>,
    request_id: u64,
    healthy: Arc<AtomicBool>,
}

impl SigilProcess {
    async fn spawn(healthy: Arc<AtomicBool>) -> Result<Self> {
        info!("Spawning Sigil DSP process...");

        let sigil_bin = std::env::var("SIGIL_BIN").unwrap_or_else(|_| "sigil".to_string());
        let dsp_path = std::env::var("SIGIL_DSP_PATH")
            .unwrap_or_else(|_| "/app/sigil/main.sigil".to_string());

        let mut child = Command::new(&sigil_bin)
            .arg("run")
            .arg(&dsp_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn Sigil process: {}", e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdin"))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdout"))?;

        let stdout_reader = BufReader::new(stdout);

        healthy.store(true, Ordering::SeqCst);
        info!("Sigil DSP process spawned successfully");

        Ok(Self {
            child,
            stdin,
            stdout_reader,
            request_id: 0,
            healthy,
        })
    }

    /// Check if the child process is still running
    async fn check_health(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(status)) => {
                error!("Sigil process exited with status: {}", status);
                self.healthy.store(false, Ordering::SeqCst);
                false
            }
            Ok(None) => true, // Still running
            Err(e) => {
                error!("Failed to check Sigil process status: {}", e);
                self.healthy.store(false, Ordering::SeqCst);
                false
            }
        }
    }

    async fn call(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Status> {
        // Check process health before making call
        if !self.check_health().await {
            return Err(Status::unavailable("Sigil process has crashed"));
        }

        self.request_id += 1;
        let request = JsonRpcRequest {
            id: self.request_id,
            method: method.to_string(),
            params,
        };

        let request_json =
            serde_json::to_string(&request).map_err(|e| Status::internal(e.to_string()))?;

        debug!("Sending to Sigil: {}", request_json);

        // Write request to Sigil stdin
        self.stdin
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| {
                self.healthy.store(false, Ordering::SeqCst);
                Status::internal(format!("Failed to write to Sigil: {}", e))
            })?;

        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|e| Status::internal(format!("Failed to write newline: {}", e)))?;

        self.stdin
            .flush()
            .await
            .map_err(|e| Status::internal(format!("Failed to flush: {}", e)))?;

        // Read response from Sigil stdout with timeout
        let mut response_line = String::new();
        let read_result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.stdout_reader.read_line(&mut response_line),
        )
        .await;

        match read_result {
            Ok(Ok(0)) => {
                self.healthy.store(false, Ordering::SeqCst);
                return Err(Status::unavailable("Sigil process closed stdout (crashed?)"));
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                self.healthy.store(false, Ordering::SeqCst);
                return Err(Status::internal(format!("Failed to read from Sigil: {}", e)));
            }
            Err(_) => {
                return Err(Status::deadline_exceeded("Sigil response timeout (30s)"));
            }
        }

        debug!("Received from Sigil: {}", response_line.trim());

        let response: JsonRpcResponse = serde_json::from_str(&response_line)
            .map_err(|e| Status::internal(format!("Failed to parse Sigil response: {}", e)))?;

        if let Some(error) = response.error {
            return Err(Status::internal(format!(
                "Sigil error {}: {}",
                error.code, error.message
            )));
        }

        response
            .result
            .ok_or_else(|| Status::internal("No result in Sigil response"))
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Sigil process...");
        let _ = self.call("shutdown", serde_json::json!({})).await;
        self.child.kill().await?;
        self.healthy.store(false, Ordering::SeqCst);
        Ok(())
    }
}

// ============================================================================
// Audio Service Implementation
// ============================================================================

pub struct AudioProcessorService {
    sigil: Arc<Mutex<Option<SigilProcess>>>,
    healthy: Arc<AtomicBool>,
}

impl AudioProcessorService {
    pub fn new() -> Self {
        Self {
            sigil: Arc::new(Mutex::new(None)),
            healthy: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn init(&self) -> Result<()> {
        let process = SigilProcess::spawn(self.healthy.clone()).await?;
        *self.sigil.lock().await = Some(process);
        Ok(())
    }

    pub fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::SeqCst)
    }

    async fn call_sigil(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Status> {
        let mut guard = self.sigil.lock().await;
        let sigil = guard
            .as_mut()
            .ok_or_else(|| Status::unavailable("Sigil process not initialized"))?;

        sigil.call(method, params).await
    }
}

#[tonic::async_trait]
impl AudioProcessor for AudioProcessorService {
    async fn process_audio(
        &self,
        request: Request<AudioRequest>,
    ) -> Result<Response<AudioResponse>, Status> {
        let req = request.into_inner();
        let buffer = req
            .buffer
            .ok_or_else(|| Status::invalid_argument("Missing audio buffer"))?;

        // Safe conversion with validation
        let samples = bytes_to_samples(&buffer.audio_data)?;

        let params = serde_json::json!({
            "chain_id": req.chain_id,
            "sample_rate": buffer.sample_rate,
            "samples": samples,
            "config": null
        });

        let result = self.call_sigil("process_audio", params).await?;

        let output_samples: Vec<f32> = result["samples"]
            .as_array()
            .ok_or_else(|| Status::internal("Invalid samples in response"))?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        let audio_data = samples_to_bytes(&output_samples);

        let response = AudioResponse {
            buffer: Some(AudioBuffer {
                sample_rate: buffer.sample_rate,
                num_channels: buffer.num_channels,
                num_samples: output_samples.len() as i32,
                audio_data,
                timestamp_us: buffer.timestamp_us,
                track_id: buffer.track_id,
            }),
            metrics: Some(ProcessingMetrics {
                processing_time_us: result["processing_time_us"].as_f64().unwrap_or(0.0),
                peak_level_db: result["peak_db"].as_f64().unwrap_or(-96.0),
                rms_level_db: result["rms_db"].as_f64().unwrap_or(-96.0),
                clipping_detected: result["clipping"].as_bool().unwrap_or(false),
            }),
        };

        Ok(Response::new(response))
    }

    type StreamAudioStream = Pin<Box<dyn Stream<Item = Result<AudioBuffer, Status>> + Send>>;

    async fn stream_audio(
        &self,
        request: Request<tonic::Streaming<AudioBuffer>>,
    ) -> Result<Response<Self::StreamAudioStream>, Status> {
        let mut stream = request.into_inner();
        let sigil = self.sigil.clone();

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            while let Ok(Some(buffer)) = stream.message().await {
                // Safe conversion with validation
                let samples = match bytes_to_samples(&buffer.audio_data) {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                };

                let params = serde_json::json!({
                    "chain_id": buffer.track_id.clone(),
                    "sample_rate": buffer.sample_rate,
                    "samples": samples,
                    "config": null
                });

                // Use async mutex properly
                let result = {
                    let mut guard = sigil.lock().await;
                    match guard.as_mut() {
                        Some(s) => s.call("process_audio", params).await,
                        None => Err(Status::unavailable("Sigil not initialized")),
                    }
                };

                match result {
                    Ok(res) => {
                        let output_samples: Vec<f32> = res["samples"]
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|v| v.as_f64().map(|f| f as f32))
                            .collect();

                        let audio_data = samples_to_bytes(&output_samples);

                        let output_buffer = AudioBuffer {
                            sample_rate: buffer.sample_rate,
                            num_channels: buffer.num_channels,
                            num_samples: output_samples.len() as i32,
                            audio_data,
                            timestamp_us: buffer.timestamp_us,
                            track_id: buffer.track_id,
                        };

                        if tx.send(Ok(output_buffer)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });

        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream)))
    }

    async fn load_plugin(
        &self,
        _request: Request<PluginRequest>,
    ) -> Result<Response<PluginResponse>, Status> {
        warn!("LoadPlugin called but not implemented");
        Ok(Response::new(PluginResponse {
            success: false,
            plugin_id: String::new(),
            error_message: "Plugin loading not yet implemented".to_string(),
            parameters: vec![],
        }))
    }

    async fn unload_plugin(
        &self,
        _request: Request<UnloadPluginRequest>,
    ) -> Result<Response<UnloadPluginResponse>, Status> {
        warn!("UnloadPlugin called but not implemented");
        Ok(Response::new(UnloadPluginResponse {
            success: false,
            error_message: "Plugin unloading not yet implemented".to_string(),
        }))
    }

    async fn update_effect(
        &self,
        request: Request<EffectUpdate>,
    ) -> Result<Response<EffectResponse>, Status> {
        let req = request.into_inner();

        let params = serde_json::json!({
            "chain_id": req.chain_id,
            "effect_id": req.effect_id,
            "parameter_id": req.parameter_id,
            "value": req.value
        });

        let result = self.call_sigil("update_effect", params).await?;

        Ok(Response::new(EffectResponse {
            success: result["success"].as_bool().unwrap_or(false),
            error_message: result["error"].as_str().unwrap_or("").to_string(),
        }))
    }

    async fn get_chain_config(
        &self,
        request: Request<ChainConfigRequest>,
    ) -> Result<Response<ChainConfig>, Status> {
        let req = request.into_inner();

        let params = serde_json::json!({
            "chain_id": req.chain_id
        });

        let result = self.call_sigil("get_chain_config", params).await?;

        let effects: Vec<EffectConfig> = result["effects"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|e| {
                let effect_type = match e["effect_type"].as_str().unwrap_or("") {
                    "EQ" => EffectType::Eq,
                    "COMPRESSOR" => EffectType::Compressor,
                    "REVERB" => EffectType::Reverb,
                    "DELAY" => EffectType::Delay,
                    "LIMITER" => EffectType::Limiter,
                    _ => EffectType::Unspecified,
                };

                let parameters: Vec<ParameterValue> = e["parameters"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|p| ParameterValue {
                        parameter_id: p["id"].as_str().unwrap_or("").to_string(),
                        value: p["value"].as_f64().unwrap_or(0.0) as f32,
                    })
                    .collect();

                EffectConfig {
                    effect_id: e["effect_id"].as_str().unwrap_or("").to_string(),
                    effect_type: effect_type.into(),
                    enabled: e["enabled"].as_bool().unwrap_or(false),
                    parameters,
                }
            })
            .collect();

        Ok(Response::new(ChainConfig {
            chain_id: result["chain_id"].as_str().unwrap_or("").to_string(),
            effects,
        }))
    }

    async fn set_chain_config(
        &self,
        request: Request<ChainConfig>,
    ) -> Result<Response<ChainConfigResponse>, Status> {
        let config = request.into_inner();

        let effects: Vec<serde_json::Value> = config
            .effects
            .iter()
            .map(|e| {
                let effect_type =
                    match EffectType::try_from(e.effect_type).unwrap_or(EffectType::Unspecified) {
                        EffectType::Eq => "EQ",
                        EffectType::Compressor => "COMPRESSOR",
                        EffectType::Reverb => "REVERB",
                        EffectType::Delay => "DELAY",
                        EffectType::Limiter => "LIMITER",
                        _ => "UNSPECIFIED",
                    };

                let params: Vec<serde_json::Value> = e
                    .parameters
                    .iter()
                    .map(|p| {
                        serde_json::json!({
                            "id": p.parameter_id,
                            "value": p.value
                        })
                    })
                    .collect();

                serde_json::json!({
                    "effect_id": e.effect_id,
                    "effect_type": effect_type,
                    "enabled": e.enabled,
                    "parameters": params
                })
            })
            .collect();

        let params = serde_json::json!({
            "chain_id": config.chain_id,
            "effects": effects
        });

        let result = self.call_sigil("set_chain_config", params).await?;

        Ok(Response::new(ChainConfigResponse {
            success: result["success"].as_bool().unwrap_or(false),
            error_message: result["error"].as_str().unwrap_or("").to_string(),
        }))
    }

    async fn analyze_audio(
        &self,
        request: Request<AnalyzeRequest>,
    ) -> Result<Response<AnalyzeResponse>, Status> {
        let req = request.into_inner();
        let buffer = req
            .buffer
            .ok_or_else(|| Status::invalid_argument("Missing audio buffer"))?;

        // Safe conversion with validation
        let samples = bytes_to_samples(&buffer.audio_data)?;

        let analysis_types: Vec<&str> = req
            .analysis_types
            .iter()
            .map(
                |t| match AnalysisType::try_from(*t).unwrap_or(AnalysisType::Unspecified) {
                    AnalysisType::Peak => "PEAK",
                    AnalysisType::Rms => "RMS",
                    AnalysisType::Loudness => "LOUDNESS",
                    AnalysisType::DynamicRange => "DYNAMIC_RANGE",
                    AnalysisType::Spectrum => "SPECTRUM",
                    AnalysisType::PhaseCorrelation => "PHASE_CORRELATION",
                    _ => "UNSPECIFIED",
                },
            )
            .collect();

        let params = serde_json::json!({
            "samples": samples,
            "analysis_types": analysis_types
        });

        let result = self.call_sigil("analyze_audio", params).await?;

        Ok(Response::new(AnalyzeResponse {
            spectrum: None,
            loudness: Some(LoudnessAnalysis {
                integrated_lufs: 0.0,
                short_term_lufs: 0.0,
                momentary_lufs: result["momentary_lufs"].as_f64().unwrap_or(-70.0),
                loudness_range_lu: 0.0,
                true_peak_dbtp: result["peak_db"].as_f64().unwrap_or(-96.0),
            }),
            peak_db: result["peak_db"].as_f64().unwrap_or(-96.0),
            rms_db: result["rms_db"].as_f64().unwrap_or(-96.0),
            phase_correlation: 0.0,
            dynamic_range_db: result["dynamic_range_db"].as_f64().unwrap_or(0.0),
        }))
    }

    // Tuning system RPCs
    async fn get_tuning(
        &self,
        request: Request<TuningRequest>,
    ) -> Result<Response<TuningResponse>, Status> {
        let req = request.into_inner();

        let tuning_type_str = match TuningType::try_from(req.r#type).unwrap_or(TuningType::Unspecified) {
            TuningType::Sacred => "sacred",
            TuningType::Chakra => "chakra",
            TuningType::Shruti => "shruti",
            TuningType::Maqam => "maqam",
            TuningType::Pelog => "pelog",
            TuningType::Slendro => "slendro",
            TuningType::Midi => "midi",
            _ => "equal",
        };

        let temperament_str = match Temperament::try_from(req.temperament).unwrap_or(Temperament::Equal) {
            Temperament::Just => "just",
            Temperament::Pythagorean => "pythagorean",
            Temperament::Temperament432 => "432",
            Temperament::Verdi => "verdi",
            _ => "equal",
        };

        let params = serde_json::json!({
            "type": tuning_type_str,
            "name": req.name,
            "index": req.index,
            "base_freq": if req.base_freq > 0.0 { req.base_freq } else { 440.0 },
            "system": temperament_str
        });

        let result = self.call_sigil("get_tuning", params).await?;

        Ok(Response::new(TuningResponse {
            frequency: result["frequency"].as_f64().unwrap_or(440.0),
            description: result["description"].as_str().unwrap_or("").to_string(),
            tuning_type: req.r#type,
        }))
    }

    async fn generate_drone(
        &self,
        request: Request<DroneRequest>,
    ) -> Result<Response<DroneResponse>, Status> {
        let req = request.into_inner();

        let tuning_type_str = match TuningType::try_from(req.r#type).unwrap_or(TuningType::Sacred) {
            TuningType::Sacred => "sacred",
            TuningType::Chakra => "chakra",
            _ => "custom",
        };

        let params = serde_json::json!({
            "type": tuning_type_str,
            "name": req.name,
            "frequency": req.frequency,
            "duration_ms": if req.duration_ms > 0.0 { req.duration_ms } else { 1000.0 },
            "amplitude": if req.amplitude > 0.0 { req.amplitude } else { 0.3 }
        });

        let result = self.call_sigil("generate_drone", params).await?;

        let samples: Vec<f32> = result["samples"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(Response::new(DroneResponse {
            audio_data: samples_to_bytes(&samples),
            frequency: result["frequency"].as_f64().unwrap_or(136.1),
            duration_ms: result["duration_ms"].as_f64().unwrap_or(1000.0),
            sample_rate: result["sample_rate"].as_i64().unwrap_or(48000) as i32,
        }))
    }

    async fn list_tunings(
        &self,
        _request: Request<ListTuningsRequest>,
    ) -> Result<Response<ListTuningsResponse>, Status> {
        let result = self.call_sigil("list_tunings", serde_json::json!({})).await?;

        Ok(Response::new(ListTuningsResponse {
            sacred_frequencies: result["sacred"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            chakra_names: result["chakra"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            maqam_scales: result["maqam"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            temperaments: result["tuning_systems"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            shruti_count: result["shruti_count"].as_i64().unwrap_or(22) as i32,
            pelog_notes: result["pelog_notes"].as_i64().unwrap_or(7) as i32,
            slendro_notes: result["slendro_notes"].as_i64().unwrap_or(5) as i32,
        }))
    }
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("maestro_audio_bridge=info".parse().unwrap())
                .add_directive("tonic=info".parse().unwrap()),
        )
        .init();

    info!("Maestro Audio Service starting...");

    let addr = std::env::var("GRPC_LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string())
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid GRPC_LISTEN_ADDR: {}", e))?;

    let service = AudioProcessorService::new();

    if let Err(e) = service.init().await {
        error!("Failed to initialize Sigil process: {}", e);
        error!("Make sure 'sigil' is in PATH and SIGIL_DSP_PATH points to main.sigil");
        return Err(e);
    }

    info!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(AudioProcessorServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
