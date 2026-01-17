//! Infernum local LLM integration for Orpheus
//!
//! Provides connection to the Infernum local inference server for AI features
//! when Leviathan is unavailable or for offline operation.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::{Error, Persona, Result};

/// Infernum server configuration
#[derive(Debug, Clone)]
pub struct InfernumConfig {
    /// Server endpoint (default: http://localhost:8081)
    pub endpoint: String,
    /// Model to use (default: auto-detect from server)
    pub model: Option<String>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Sampling temperature (0.0 = deterministic, 1.0 = creative)
    pub temperature: f32,
    /// Top-p (nucleus) sampling
    pub top_p: f32,
    /// Maximum tokens to generate
    pub max_tokens: u32,
}

impl Default for InfernumConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8081".to_string(),
            model: None,
            timeout_secs: 60,
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 512,
        }
    }
}

impl InfernumConfig {
    /// Create with custom endpoint
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Set model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.clamp(0.0, 2.0);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }
}

/// OpenAI-compatible chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// OpenAI-compatible chat completion request
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    pub stream: bool,
}

/// OpenAI-compatible chat completion response
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    #[serde(default)]
    pub usage: Option<ChatUsage>,
}

/// Chat completion choice
#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

/// Token usage statistics
#[derive(Debug, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

/// Infernum health response
#[derive(Debug, Deserialize)]
pub struct InfernumHealth {
    pub status: String,
    #[serde(default)]
    pub model: Option<String>,
}

/// Infernum client for local LLM inference
pub struct InfernumClient {
    config: InfernumConfig,
    http_client: Option<reqwest::Client>,
    connected: bool,
    model_name: Option<String>,
}

impl InfernumClient {
    /// Create a new Infernum client
    pub fn new(config: InfernumConfig) -> Self {
        Self {
            config,
            http_client: None,
            connected: false,
            model_name: None,
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get current model name
    pub fn model(&self) -> Option<&str> {
        self.model_name.as_deref()
    }

    /// Connect to Infernum server
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to Infernum at {}", self.config.endpoint);

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .build()
            .map_err(|e| Error::ConnectionError(format!("Failed to create HTTP client: {}", e)))?;

        // Check health
        let health_url = format!("{}/health", self.config.endpoint);
        match http_client.get(&health_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(health) = response.json::<InfernumHealth>().await {
                        info!("Infernum is healthy, status: {}", health.status);
                        self.model_name = health.model;
                    }
                } else {
                    warn!("Infernum health check returned status: {}", response.status());
                }
            }
            Err(e) => {
                return Err(Error::ConnectionError(format!(
                    "Failed to connect to Infernum: {}. Is the server running?",
                    e
                )));
            }
        }

        self.http_client = Some(http_client);
        self.connected = true;
        info!("Connected to Infernum");

        Ok(())
    }

    /// Disconnect from Infernum
    pub fn disconnect(&mut self) {
        self.http_client = None;
        self.connected = false;
        self.model_name = None;
        info!("Disconnected from Infernum");
    }

    /// Generate a chat completion
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        persona: Option<&Persona>,
    ) -> Result<String> {
        if !self.is_connected() {
            return Err(Error::NotConnected);
        }

        let http_client = self.http_client.as_ref().ok_or(Error::NotConnected)?;

        // Build messages with optional system prompt
        let mut all_messages = Vec::new();

        if let Some(persona) = persona {
            all_messages.push(ChatMessage::system(persona.system_prompt_prefix()));
        }

        all_messages.extend(messages);

        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: all_messages,
            temperature: Some(self.config.temperature),
            top_p: Some(self.config.top_p),
            max_tokens: Some(self.config.max_tokens),
            stop: None,
            stream: false,
        };

        let url = format!("{}/v1/chat/completions", self.config.endpoint);
        debug!("Sending chat request to {}", url);

        let response = http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::ConnectionError(format!("Chat request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::RequestFailed(format!(
                "Chat completion failed with status {}: {}",
                status, body
            )));
        }

        let completion: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| Error::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        let content = completion
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| Error::InvalidResponse("No response content".to_string()))?;

        if let Some(usage) = &completion.usage {
            debug!(
                "Token usage: {} prompt, {} completion, {} total",
                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
            );
        }

        Ok(content)
    }

    /// Simple text generation with a single prompt
    pub async fn generate(&self, prompt: &str, persona: Option<&Persona>) -> Result<String> {
        self.chat(vec![ChatMessage::user(prompt)], persona).await
    }

    /// Generate a music-related response
    pub async fn music_chat(&self, prompt: &str, persona: &Persona) -> Result<String> {
        let messages = vec![ChatMessage::user(prompt)];
        self.chat(messages, Some(persona)).await
    }
}

impl Default for InfernumClient {
    fn default() -> Self {
        Self::new(InfernumConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infernum_config_default() {
        let config = InfernumConfig::default();
        assert_eq!(config.endpoint, "http://localhost:8081");
        assert!(config.model.is_none());
        assert_eq!(config.timeout_secs, 60);
        assert!((config.temperature - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_infernum_config_builder() {
        let config = InfernumConfig::default()
            .with_endpoint("http://ai.local:9000")
            .with_model("qwen2.5")
            .with_temperature(0.5)
            .with_max_tokens(1024);

        assert_eq!(config.endpoint, "http://ai.local:9000");
        assert_eq!(config.model, Some("qwen2.5".to_string()));
        assert!((config.temperature - 0.5).abs() < 0.01);
        assert_eq!(config.max_tokens, 1024);
    }

    #[test]
    fn test_chat_message_constructors() {
        let system = ChatMessage::system("You are helpful");
        assert_eq!(system.role, "system");
        assert_eq!(system.content, "You are helpful");

        let user = ChatMessage::user("Hello");
        assert_eq!(user.role, "user");

        let assistant = ChatMessage::assistant("Hi there!");
        assert_eq!(assistant.role, "assistant");
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatCompletionRequest {
            model: Some("test-model".to_string()),
            messages: vec![
                ChatMessage::system("Be helpful"),
                ChatMessage::user("What is music?"),
            ],
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(256),
            stop: None,
            stream: false,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test-model"));
        assert!(json.contains("What is music?"));
        assert!(json.contains("\"stream\":false"));
    }

    #[test]
    fn test_infernum_client_creation() {
        let client = InfernumClient::new(InfernumConfig::default());
        assert!(!client.is_connected());
        assert!(client.model().is_none());
    }

    #[tokio::test]
    #[ignore] // Run with `cargo test -- --ignored` when Infernum is running
    async fn test_real_infernum_connection() {
        let mut client = InfernumClient::new(InfernumConfig::default());

        let result = client.connect().await;
        println!("Connection result: {:?}", result);

        if client.is_connected() {
            println!("Connected! Model: {:?}", client.model());

            let response = client
                .generate("What chord comes after G7 in jazz?", Some(&Persona::MusicTheoryTutor))
                .await;

            match response {
                Ok(text) => println!("Response: {}", text),
                Err(e) => println!("Error: {:?}", e),
            }

            client.disconnect();
        } else {
            println!("Could not connect - make sure Infernum is running on localhost:8081");
        }
    }
}
