/// Maestro Audio Server
/// High-performance gRPC audio processing service

mod service;

use maestro_proto::audio::audio_processor_server::AudioProcessorServer;
use service::AudioProcessorService;
use tonic::transport::Server;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let addr = "0.0.0.0:50051".parse()?;
    let audio_service = AudioProcessorService::new(48000, 512);

    info!("🎸 Maestro Audio Server starting on {}", addr);
    info!("Sample rate: 48kHz, Buffer size: 512 samples");

    Server::builder()
        .add_service(AudioProcessorServer::new(audio_service))
        .serve(addr)
        .await?;

    Ok(())
}
