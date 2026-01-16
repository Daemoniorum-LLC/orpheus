//! Dantalion Integration
//!
//! Client for the Dantalion Vision Service's audio generation capabilities.
//! Enables Orpheus to leverage AI-generated audio:
//! - Music generation (MusicGen, Stable Audio Open)
//! - Sound effects (AudioGen, AudioLDM2)
//! - Text-to-speech (Bark, Parler-TTS, XTTS v2)
//!
//! This is complementary to Orpheus's real-time synthesis:
//! - Dantalion: Batch AI generation from text prompts
//! - Orpheus Synth: Real-time physical modeling from notation

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Audio generation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioType {
    /// AI-generated music from text prompts
    Music,
    /// Sound effects and ambient audio
    Sfx,
    /// Text-to-speech synthesis
    Tts,
}

impl AudioType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AudioType::Music => "music",
            AudioType::Sfx => "sfx",
            AudioType::Tts => "tts",
        }
    }
}

/// Available audio models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioModel {
    // Music Generation
    MusicGenSmall,
    MusicGenMedium,
    MusicGenLarge,
    StableAudioOpen,

    // Sound Effects
    AudioGen,
    AudioLdm2,
    AudioLdm2Large,

    // Text-to-Speech
    Bark,
    ParlerTts,
    XttsV2,
}

impl AudioModel {
    /// Get the model ID used by Dantalion API
    pub fn model_id(&self) -> &'static str {
        match self {
            AudioModel::MusicGenSmall => "musicgen-small",
            AudioModel::MusicGenMedium => "musicgen-medium",
            AudioModel::MusicGenLarge => "musicgen-large",
            AudioModel::StableAudioOpen => "stable-audio-open",
            AudioModel::AudioGen => "audiogen",
            AudioModel::AudioLdm2 => "audioldm2",
            AudioModel::AudioLdm2Large => "audioldm2-large",
            AudioModel::Bark => "bark",
            AudioModel::ParlerTts => "parler-tts",
            AudioModel::XttsV2 => "xtts-v2",
        }
    }

    /// Get the audio type this model produces
    pub fn audio_type(&self) -> AudioType {
        match self {
            AudioModel::MusicGenSmall
            | AudioModel::MusicGenMedium
            | AudioModel::MusicGenLarge
            | AudioModel::StableAudioOpen => AudioType::Music,

            AudioModel::AudioGen
            | AudioModel::AudioLdm2
            | AudioModel::AudioLdm2Large => AudioType::Sfx,

            AudioModel::Bark | AudioModel::ParlerTts | AudioModel::XttsV2 => AudioType::Tts,
        }
    }

    /// Get max duration in seconds
    pub fn max_duration(&self) -> f32 {
        match self {
            AudioModel::MusicGenSmall
            | AudioModel::MusicGenMedium
            | AudioModel::MusicGenLarge => 30.0,
            AudioModel::StableAudioOpen => 47.0,
            AudioModel::AudioGen
            | AudioModel::AudioLdm2
            | AudioModel::AudioLdm2Large => 10.0,
            AudioModel::Bark => 15.0,
            AudioModel::ParlerTts => 30.0,
            AudioModel::XttsV2 => 60.0,
        }
    }

    /// Get default sample rate
    pub fn sample_rate(&self) -> u32 {
        match self {
            AudioModel::MusicGenSmall
            | AudioModel::MusicGenMedium
            | AudioModel::MusicGenLarge => 32000,
            AudioModel::StableAudioOpen | AudioModel::ParlerTts => 44100,
            AudioModel::AudioGen
            | AudioModel::AudioLdm2
            | AudioModel::AudioLdm2Large => 16000,
            AudioModel::Bark | AudioModel::XttsV2 => 24000,
        }
    }

    /// Get VRAM requirement in GB
    pub fn vram_gb(&self) -> f32 {
        match self {
            AudioModel::MusicGenSmall | AudioModel::ParlerTts | AudioModel::XttsV2 => 4.0,
            AudioModel::AudioLdm2 | AudioModel::Bark => 6.0,
            AudioModel::MusicGenMedium
            | AudioModel::StableAudioOpen
            | AudioModel::AudioGen => 8.0,
            AudioModel::AudioLdm2Large => 10.0,
            AudioModel::MusicGenLarge => 12.0,
        }
    }

    /// Whether this model supports voice cloning
    pub fn supports_voice_cloning(&self) -> bool {
        matches!(self, AudioModel::XttsV2)
    }

    /// Get recommended model for audio type
    pub fn recommended(audio_type: AudioType) -> Self {
        match audio_type {
            AudioType::Music => AudioModel::StableAudioOpen,
            AudioType::Sfx => AudioModel::AudioLdm2Large,
            AudioType::Tts => AudioModel::ParlerTts,
        }
    }
}

/// Request for audio generation
#[derive(Debug, Clone)]
pub struct AudioGenerationRequest {
    /// Text prompt describing the audio to generate
    pub prompt: String,
    /// Optional negative prompt
    pub negative_prompt: Option<String>,
    /// Model to use (defaults to recommended for type)
    pub model: AudioModel,
    /// Duration in seconds
    pub duration_seconds: f32,
    /// Optional: Voice description (for TTS)
    pub voice_description: Option<String>,
    /// Optional: Reference audio URL for voice cloning
    pub voice_reference_url: Option<String>,
    /// Optional: Priority (critical, high, standard)
    pub priority: Option<String>,
    /// Optional: Webhook URL for completion notification
    pub webhook_url: Option<String>,
    /// Optional: User ID for cost tracking
    pub user_id: Option<String>,
}

impl AudioGenerationRequest {
    /// Create a music generation request
    pub fn music(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            negative_prompt: None,
            model: AudioModel::StableAudioOpen,
            duration_seconds: 30.0,
            voice_description: None,
            voice_reference_url: None,
            priority: None,
            webhook_url: None,
            user_id: None,
        }
    }

    /// Create a sound effect generation request
    pub fn sfx(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            negative_prompt: None,
            model: AudioModel::AudioLdm2Large,
            duration_seconds: 5.0,
            voice_description: None,
            voice_reference_url: None,
            priority: None,
            webhook_url: None,
            user_id: None,
        }
    }

    /// Create a text-to-speech request
    pub fn tts(text: impl Into<String>) -> Self {
        Self {
            prompt: text.into(),
            negative_prompt: None,
            model: AudioModel::ParlerTts,
            duration_seconds: 15.0,
            voice_description: None,
            voice_reference_url: None,
            priority: None,
            webhook_url: None,
            user_id: None,
        }
    }

    /// Set the model to use
    pub fn with_model(mut self, model: AudioModel) -> Self {
        self.model = model;
        self
    }

    /// Set the duration
    pub fn with_duration(mut self, seconds: f32) -> Self {
        self.duration_seconds = seconds.min(self.model.max_duration());
        self
    }

    /// Set voice description (for TTS models like Parler-TTS)
    pub fn with_voice(mut self, description: impl Into<String>) -> Self {
        self.voice_description = Some(description.into());
        self
    }

    /// Set voice reference for cloning (for XTTS v2)
    pub fn with_voice_clone(mut self, audio_url: impl Into<String>) -> Self {
        self.voice_reference_url = Some(audio_url.into());
        self.model = AudioModel::XttsV2;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: impl Into<String>) -> Self {
        self.priority = Some(priority.into());
        self
    }

    /// Set webhook URL
    pub fn with_webhook(mut self, url: impl Into<String>) -> Self {
        self.webhook_url = Some(url.into());
        self
    }

    /// Convert to JSON for API request
    pub fn to_json(&self) -> serde_json::Value {
        let mut json = serde_json::json!({
            "prompt": self.prompt,
            "model": self.model.model_id(),
            "duration_seconds": self.duration_seconds,
        });

        if let Some(ref neg) = self.negative_prompt {
            json["negative_prompt"] = serde_json::json!(neg);
        }
        if let Some(ref voice) = self.voice_description {
            json["voice_description"] = serde_json::json!(voice);
        }
        if let Some(ref url) = self.voice_reference_url {
            json["voice_reference_url"] = serde_json::json!(url);
        }
        if let Some(ref priority) = self.priority {
            json["priority"] = serde_json::json!(priority);
        }
        if let Some(ref webhook) = self.webhook_url {
            json["webhook_url"] = serde_json::json!(webhook);
        }
        if let Some(ref user_id) = self.user_id {
            json["user_id"] = serde_json::json!(user_id);
        }

        json
    }
}

/// Status of a generation request
#[derive(Debug, Clone, PartialEq)]
pub enum GenerationStatus {
    /// Request is queued
    Pending { queue_position: usize },
    /// Generation in progress
    Processing { progress_percent: f32 },
    /// Generation complete
    Completed { output_url: String },
    /// Generation failed
    Failed { error: String },
}

/// Response from generation request
#[derive(Debug, Clone)]
pub struct GenerationResponse {
    pub generation_id: String,
    pub status: GenerationStatus,
    pub estimated_time_seconds: Option<f32>,
}

/// Dantalion client configuration
#[derive(Debug, Clone)]
pub struct DantalionConfig {
    /// Base URL for Dantalion API
    pub base_url: String,
    /// Optional API key
    pub api_key: Option<String>,
    /// Request timeout
    pub timeout: Duration,
    /// Poll interval for checking status
    pub poll_interval: Duration,
    /// Local cache directory for downloaded audio
    pub cache_dir: Option<PathBuf>,
}

impl Default for DantalionConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8083".to_string(),
            api_key: None,
            timeout: Duration::from_secs(120),
            poll_interval: Duration::from_secs(2),
            cache_dir: None,
        }
    }
}

impl DantalionConfig {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            ..Default::default()
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_cache_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(dir.into());
        self
    }
}

/// Dantalion client for audio generation
///
/// This client interacts with the Dantalion API to generate audio from text prompts.
/// Generation is asynchronous - you submit a request and poll for completion.
pub struct DantalionClient {
    config: DantalionConfig,
    /// Cache of generation IDs to their status
    generations: HashMap<String, GenerationStatus>,
}

impl DantalionClient {
    /// Create a new client with default configuration
    pub fn new() -> Self {
        Self {
            config: DantalionConfig::default(),
            generations: HashMap::new(),
        }
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: DantalionConfig) -> Self {
        Self {
            config,
            generations: HashMap::new(),
        }
    }

    /// Get the API base URL
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Build URL for an endpoint
    pub fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url, path)
    }

    /// Get the audio generation endpoint
    pub fn audio_generate_url(&self) -> String {
        self.endpoint("/audio/generate")
    }

    /// Get the generation status endpoint
    pub fn generation_status_url(&self, id: &str) -> String {
        self.endpoint(&format!("/generations/{}", id))
    }

    /// Get the models list endpoint
    pub fn audio_models_url(&self) -> String {
        self.endpoint("/audio/models")
    }

    /// Check if Dantalion service is available
    pub fn health_check_url(&self) -> String {
        self.endpoint("/health")
    }

    /// Store a generation status
    pub fn track_generation(&mut self, id: String, status: GenerationStatus) {
        self.generations.insert(id, status);
    }

    /// Get a tracked generation's status
    pub fn get_generation(&self, id: &str) -> Option<&GenerationStatus> {
        self.generations.get(id)
    }

    /// List all tracked generations
    pub fn list_generations(&self) -> impl Iterator<Item = (&String, &GenerationStatus)> {
        self.generations.iter()
    }
}

impl Default for DantalionClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Preset prompts for common audio generation scenarios
pub mod presets {
    use super::*;

    /// Orchestra/Orchestral presets
    pub mod orchestra {
        use super::*;

        pub fn epic_strings() -> AudioGenerationRequest {
            AudioGenerationRequest::music(
                "Epic orchestral strings, dramatic crescendo, cinematic film score, \
                 full string section, emotional and powerful"
            ).with_duration(30.0)
        }

        pub fn gentle_woodwinds() -> AudioGenerationRequest {
            AudioGenerationRequest::music(
                "Gentle woodwinds melody, pastoral, flute and oboe duet, \
                 peaceful forest ambiance, classical style"
            ).with_duration(20.0)
        }

        pub fn brass_fanfare() -> AudioGenerationRequest {
            AudioGenerationRequest::music(
                "Triumphant brass fanfare, French horns and trumpets, \
                 royal proclamation, majestic and heroic"
            ).with_duration(15.0)
        }

        pub fn orchestral_hit() -> AudioGenerationRequest {
            AudioGenerationRequest::sfx(
                "Orchestral hit, dramatic stinger, full orchestra impact, \
                 timpani and brass, cinematic trailer sound"
            ).with_duration(3.0)
        }
    }

    /// Ambient/Background presets
    pub mod ambient {
        use super::*;

        pub fn forest_ambience() -> AudioGenerationRequest {
            AudioGenerationRequest::sfx(
                "Forest ambience, birds chirping, gentle wind through leaves, \
                 distant stream, peaceful nature sounds"
            ).with_duration(10.0)
        }

        pub fn rain_thunder() -> AudioGenerationRequest {
            AudioGenerationRequest::sfx(
                "Heavy rain on roof, distant thunder rolling, storm ambience, \
                 cozy indoor atmosphere"
            ).with_duration(10.0)
        }

        pub fn space_drone() -> AudioGenerationRequest {
            AudioGenerationRequest::music(
                "Space ambient drone, ethereal synthesizers, vast cosmic emptiness, \
                 slow evolving textures, dark and mysterious"
            ).with_duration(45.0)
        }
    }

    /// Choir/Vocal presets
    pub mod choir {
        use super::*;

        pub fn gregorian_chant() -> AudioGenerationRequest {
            AudioGenerationRequest::music(
                "Gregorian chant, male choir, Latin liturgical music, \
                 reverberant cathedral acoustics, medieval sacred music"
            ).with_duration(30.0)
        }

        pub fn angelic_voices() -> AudioGenerationRequest {
            AudioGenerationRequest::music(
                "Angelic choir, female voices, ethereal harmonies, \
                 heavenly and pure, soft reverb, wordless vocals"
            ).with_duration(25.0)
        }

        pub fn epic_choir() -> AudioGenerationRequest {
            AudioGenerationRequest::music(
                "Epic choir, massive vocal ensemble, dramatic Latin text, \
                 cinematic trailer music, powerful crescendo"
            ).with_duration(30.0)
        }
    }

    /// Sound effects presets
    pub mod sfx {
        use super::*;

        pub fn footsteps_wood() -> AudioGenerationRequest {
            AudioGenerationRequest::sfx(
                "Footsteps on wooden floor, walking pace, indoor room, \
                 slight creaking, realistic foley sound"
            ).with_duration(5.0)
        }

        pub fn sword_clash() -> AudioGenerationRequest {
            AudioGenerationRequest::sfx(
                "Metal sword clash, combat sound, steel on steel, \
                 sharp impact, medieval battle foley"
            ).with_duration(2.0)
        }

        pub fn magic_spell() -> AudioGenerationRequest {
            AudioGenerationRequest::sfx(
                "Magic spell cast, mystical whoosh, sparkling energy, \
                 fantasy game sound effect, ethereal power"
            ).with_duration(3.0)
        }

        pub fn door_creak() -> AudioGenerationRequest {
            AudioGenerationRequest::sfx(
                "Old wooden door creaking open slowly, haunted house, \
                 horror atmosphere, rusty hinges"
            ).with_duration(4.0)
        }
    }

    /// Voice/TTS presets
    pub mod voice {
        use super::*;

        pub fn narrator_male() -> AudioGenerationRequest {
            AudioGenerationRequest::tts("Sample narrator text")
                .with_voice("A deep male voice with clear diction, \
                            warm and authoritative, audiobook narrator style")
        }

        pub fn narrator_female() -> AudioGenerationRequest {
            AudioGenerationRequest::tts("Sample narrator text")
                .with_voice("A pleasant female voice with gentle tone, \
                            warm and expressive, documentary narrator style")
        }

        pub fn announcement() -> AudioGenerationRequest {
            AudioGenerationRequest::tts("Attention please")
                .with_voice("Clear and professional voice, \
                            neutral accent, public announcement style")
        }
    }
}

/// Utility functions for working with generated audio
pub mod utils {
    /// Convert sample rate (simple linear interpolation)
    pub fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
        if from_rate == to_rate {
            return samples.to_vec();
        }

        let ratio = from_rate as f64 / to_rate as f64;
        let new_len = (samples.len() as f64 / ratio).ceil() as usize;
        let mut output = Vec::with_capacity(new_len);

        for i in 0..new_len {
            let src_pos = i as f64 * ratio;
            let src_idx = src_pos as usize;
            let frac = src_pos - src_idx as f64;

            let sample = if src_idx + 1 < samples.len() {
                let a = samples[src_idx] as f64;
                let b = samples[src_idx + 1] as f64;
                (a + (b - a) * frac) as f32
            } else if src_idx < samples.len() {
                samples[src_idx]
            } else {
                0.0
            };

            output.push(sample);
        }

        output
    }

    /// Normalize audio to peak at target level
    pub fn normalize(samples: &mut [f32], target_peak: f32) {
        let max = samples
            .iter()
            .map(|s| s.abs())
            .fold(0.0_f32, f32::max);

        if max > 0.0 {
            let scale = target_peak / max;
            for sample in samples {
                *sample *= scale;
            }
        }
    }

    /// Fade in/out to avoid clicks
    pub fn apply_fades(samples: &mut [f32], fade_samples: usize) {
        let len = samples.len();
        let fade = fade_samples.min(len / 2);

        // Fade in
        for i in 0..fade {
            let t = i as f32 / fade as f32;
            samples[i] *= t * t; // Quadratic fade
        }

        // Fade out
        for i in 0..fade {
            let t = i as f32 / fade as f32;
            samples[len - 1 - i] *= t * t;
        }
    }

    /// Mix two audio buffers
    pub fn mix(a: &[f32], b: &[f32], b_gain: f32) -> Vec<f32> {
        let len = a.len().max(b.len());
        let mut output = vec![0.0; len];

        for (i, sample) in output.iter_mut().enumerate() {
            let sa = if i < a.len() { a[i] } else { 0.0 };
            let sb = if i < b.len() { b[i] * b_gain } else { 0.0 };
            *sample = sa + sb;
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_model_properties() {
        assert_eq!(AudioModel::StableAudioOpen.audio_type(), AudioType::Music);
        assert_eq!(AudioModel::AudioLdm2.audio_type(), AudioType::Sfx);
        assert_eq!(AudioModel::ParlerTts.audio_type(), AudioType::Tts);

        assert_eq!(AudioModel::StableAudioOpen.max_duration(), 47.0);
        assert_eq!(AudioModel::XttsV2.max_duration(), 60.0);

        assert!(AudioModel::XttsV2.supports_voice_cloning());
        assert!(!AudioModel::ParlerTts.supports_voice_cloning());
    }

    #[test]
    fn test_request_builder() {
        let req = AudioGenerationRequest::music("epic orchestra")
            .with_duration(45.0)
            .with_priority("high");

        assert_eq!(req.prompt, "epic orchestra");
        assert_eq!(req.duration_seconds, 45.0);
        assert_eq!(req.priority, Some("high".to_string()));
    }

    #[test]
    fn test_tts_request() {
        let req = AudioGenerationRequest::tts("Hello world")
            .with_voice("warm male voice");

        assert_eq!(req.model.audio_type(), AudioType::Tts);
        assert_eq!(req.voice_description, Some("warm male voice".to_string()));
    }

    #[test]
    fn test_voice_cloning_request() {
        let req = AudioGenerationRequest::tts("Clone this voice")
            .with_voice_clone("https://example.com/voice.wav");

        assert_eq!(req.model, AudioModel::XttsV2);
        assert!(req.voice_reference_url.is_some());
    }

    #[test]
    fn test_client_endpoints() {
        let client = DantalionClient::new();
        assert!(client.audio_generate_url().contains("/audio/generate"));
        assert!(client.health_check_url().contains("/health"));
    }

    #[test]
    fn test_resample() {
        let samples = vec![0.0, 1.0, 0.0, -1.0, 0.0];
        let resampled = utils::resample(&samples, 44100, 22050);
        assert!(resampled.len() < samples.len());
    }

    #[test]
    fn test_normalize() {
        let mut samples = vec![0.5, -0.3, 0.2, -0.5];
        utils::normalize(&mut samples, 1.0);
        let max = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!((max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_presets() {
        let epic = presets::orchestra::epic_strings();
        assert_eq!(epic.model.audio_type(), AudioType::Music);

        let rain = presets::ambient::rain_thunder();
        assert_eq!(rain.model.audio_type(), AudioType::Sfx);

        let narrator = presets::voice::narrator_male();
        assert_eq!(narrator.model.audio_type(), AudioType::Tts);
    }
}
