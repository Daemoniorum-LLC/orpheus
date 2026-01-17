//! AI Client for Orpheus
//!
//! Provides connection to Leviathan AI backend for intelligent music assistance.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::graphql::{
    self, CreateAgentTaskInput, CreateConversationMessageInput, CreateConversationSessionInput,
    GraphQLRequest, GraphQLResponse, HealthStatus,
};
use crate::infernum::{ChatMessage, InfernumClient, InfernumConfig};
use crate::{AiProvider, Error, Persona, Result};

/// AI client configuration
#[derive(Debug, Clone)]
pub struct AiConfig {
    /// AI provider to use
    pub provider: AiProvider,
    /// Leviathan server endpoint
    pub endpoint: String,
    /// Infernum configuration (for local LLM)
    pub infernum: InfernumConfig,
    /// Default persona to use
    pub default_persona: Persona,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Enable streaming responses
    pub streaming: bool,
    /// Mock mode (for testing without a running server)
    pub mock_mode: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::Leviathan,
            endpoint: "http://localhost:8989".to_string(),
            infernum: InfernumConfig::default(),
            default_persona: Persona::MusicComposer,
            timeout_secs: 30,
            streaming: true,
            mock_mode: false,
        }
    }
}

impl AiConfig {
    /// Create with custom endpoint
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Set AI provider
    pub fn with_provider(mut self, provider: AiProvider) -> Self {
        self.provider = provider;
        if provider == AiProvider::Mock {
            self.mock_mode = true;
        }
        self
    }

    /// Use Infernum local LLM
    pub fn with_infernum(mut self) -> Self {
        self.provider = AiProvider::Infernum;
        self
    }

    /// Configure Infernum settings
    pub fn with_infernum_config(mut self, config: InfernumConfig) -> Self {
        self.infernum = config;
        self
    }

    /// Set default persona
    pub fn with_persona(mut self, persona: Persona) -> Self {
        self.default_persona = persona;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Enable mock mode (for testing without a running server)
    pub fn with_mock_mode(mut self) -> Self {
        self.mock_mode = true;
        self.provider = AiProvider::Mock;
        self
    }
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected and ready
    Connected,
    /// Connection error
    Error,
}

/// A message in the AI conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    /// Unique message ID
    pub id: String,
    /// Message role (user, assistant, system)
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Timestamp (unix millis)
    pub timestamp: u64,
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// User message
    User,
    /// AI assistant response
    Assistant,
    /// System message
    System,
}

impl AiMessage {
    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: content.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::Assistant,
            content: content.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::System,
            content: content.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }
}

/// AI request for the backend
#[derive(Debug, Clone)]
pub struct AiRequest {
    /// Unique request ID
    pub id: String,
    /// Messages in the conversation
    pub messages: Vec<AiMessage>,
    /// Persona to use
    pub persona: Persona,
    /// Additional context (key-value pairs)
    pub context: HashMap<String, String>,
}

impl AiRequest {
    /// Create a new request
    pub fn new(persona: Persona) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            messages: Vec::new(),
            persona,
            context: HashMap::new(),
        }
    }

    /// Add a message
    pub fn with_message(mut self, message: AiMessage) -> Self {
        self.messages.push(message);
        self
    }

    /// Add context
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// AI response from the backend
#[derive(Debug, Clone)]
pub struct AiResponse {
    /// Request ID this responds to
    pub request_id: String,
    /// Response content
    pub content: String,
    /// Whether this is a streaming chunk
    pub is_chunk: bool,
    /// Whether this completes the response
    pub is_complete: bool,
    /// Token usage (if available)
    pub usage: Option<TokenUsage>,
}

/// Token usage statistics
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    /// Input tokens
    pub input_tokens: u64,
    /// Output tokens
    pub output_tokens: u64,
}

impl TokenUsage {
    /// Total tokens
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

/// AI conversation history
#[derive(Debug, Clone, Default)]
pub struct Conversation {
    /// Conversation ID
    pub id: String,
    /// Messages in order
    pub messages: Vec<AiMessage>,
    /// Current persona
    pub persona: Persona,
}

impl Conversation {
    /// Create a new conversation
    pub fn new(persona: Persona) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            messages: Vec::new(),
            persona,
        }
    }

    /// Add a message
    pub fn add_message(&mut self, message: AiMessage) {
        self.messages.push(message);
    }

    /// Get all messages
    pub fn messages(&self) -> &[AiMessage] {
        &self.messages
    }

    /// Get message count
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Build a request from this conversation
    pub fn to_request(&self) -> AiRequest {
        AiRequest {
            id: Uuid::new_v4().to_string(),
            messages: self.messages.clone(),
            persona: self.persona,
            context: HashMap::new(),
        }
    }
}

/// Internal command type for async operations
/// Reserved for future Leviathan gRPC integration
#[allow(dead_code)]
enum ClientCommand {
    SendMessage {
        request: AiRequest,
        response_tx: oneshot::Sender<Result<AiResponse>>,
    },
    SetPersona(Persona),
    Disconnect,
}

/// AI Client for communicating with AI backends (Leviathan or Infernum)
pub struct AiClient {
    /// Configuration
    config: AiConfig,
    /// Connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Current persona
    persona: Arc<RwLock<Persona>>,
    /// Command sender (for async operations)
    command_tx: Option<Sender<ClientCommand>>,
    /// Response receiver for streaming
    stream_rx: Option<Receiver<AiResponse>>,
    /// HTTP client for Leviathan API
    http_client: Option<reqwest::Client>,
    /// Infernum client for local LLM
    infernum_client: Option<InfernumClient>,
    /// Active conversation session ID
    session_id: Arc<RwLock<Option<String>>>,
}

impl AiClient {
    /// Create a new AI client
    pub fn new(config: AiConfig) -> Self {
        Self {
            persona: Arc::new(RwLock::new(config.default_persona)),
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            command_tx: None,
            stream_rx: None,
            http_client: None,
            infernum_client: None,
            session_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current provider
    pub fn provider(&self) -> AiProvider {
        self.config.provider
    }

    /// Get current configuration
    pub fn config(&self) -> &AiConfig {
        &self.config
    }

    /// Get current connection state
    pub fn state(&self) -> ConnectionState {
        *self.state.read()
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state() == ConnectionState::Connected
    }

    /// Get current persona
    pub fn persona(&self) -> Persona {
        *self.persona.read()
    }

    /// Set current persona
    pub fn set_persona(&self, persona: Persona) {
        *self.persona.write() = persona;
        if let Some(ref tx) = self.command_tx {
            let _ = tx.send(ClientCommand::SetPersona(persona));
        }
    }

    /// Connect to the AI backend (Leviathan or Infernum)
    pub async fn connect(&mut self) -> Result<()> {
        *self.state.write() = ConnectionState::Connecting;

        // Mock mode - just mark as connected without real connection
        if self.config.mock_mode || self.config.provider == AiProvider::Mock {
            info!("Connecting in mock mode (no real server)");
            let (cmd_tx, _cmd_rx) = bounded::<ClientCommand>(32);
            let (stream_tx, stream_rx) = bounded::<AiResponse>(64);
            self.command_tx = Some(cmd_tx);
            self.stream_rx = Some(stream_rx);
            let _ = stream_tx;
            *self.state.write() = ConnectionState::Connected;
            return Ok(());
        }

        // Connect based on provider
        match self.config.provider {
            AiProvider::Infernum => self.connect_infernum().await,
            AiProvider::Leviathan => self.connect_leviathan().await,
            AiProvider::Mock => unreachable!(), // Handled above
        }
    }

    /// Connect to Infernum local LLM
    async fn connect_infernum(&mut self) -> Result<()> {
        info!("Connecting to Infernum at {}", self.config.infernum.endpoint);

        let mut client = InfernumClient::new(self.config.infernum.clone());
        client.connect().await?;

        self.infernum_client = Some(client);

        // Create command channel for async operations
        let (cmd_tx, _cmd_rx) = bounded::<ClientCommand>(32);
        let (stream_tx, stream_rx) = bounded::<AiResponse>(64);
        self.command_tx = Some(cmd_tx);
        self.stream_rx = Some(stream_rx);
        let _ = stream_tx;

        *self.state.write() = ConnectionState::Connected;
        info!("Connected to Infernum");

        Ok(())
    }

    /// Connect to Leviathan backend
    async fn connect_leviathan(&mut self) -> Result<()> {
        info!("Connecting to Leviathan at {}", self.config.endpoint);

        // Create HTTP client with timeout
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .build()
            .map_err(|e| Error::ConnectionError(format!("Failed to create HTTP client: {}", e)))?;

        // Check Leviathan health
        let health_url = format!("{}/api/health/status", self.config.endpoint);
        match http_client.get(&health_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<HealthStatus>().await {
                        Ok(health) => {
                            if health.is_healthy() {
                                info!("Leviathan is healthy");
                            } else {
                                warn!("Leviathan reports unhealthy status: {}", health.status);
                            }
                        }
                        Err(e) => {
                            debug!("Could not parse health response: {}", e);
                        }
                    }
                } else {
                    warn!("Leviathan health check returned status: {}", response.status());
                }
            }
            Err(e) => {
                // Don't fail connection on health check error - service might still work
                warn!("Leviathan health check failed: {} (continuing anyway)", e);
            }
        }

        // Store the HTTP client
        self.http_client = Some(http_client);

        // Create command channel for async operations
        let (cmd_tx, _cmd_rx) = bounded::<ClientCommand>(32);
        let (stream_tx, stream_rx) = bounded::<AiResponse>(64);

        self.command_tx = Some(cmd_tx);
        self.stream_rx = Some(stream_rx);

        // Store stream_tx for future streaming support
        let _ = stream_tx;

        // Create a conversation session
        if let Err(e) = self.create_session().await {
            warn!("Failed to create conversation session: {}", e);
            // Don't fail - we can try again on first message
        }

        *self.state.write() = ConnectionState::Connected;
        info!("Connected to Leviathan");

        Ok(())
    }

    /// Create a new conversation session
    async fn create_session(&self) -> Result<String> {
        let http_client = self.http_client.as_ref().ok_or(Error::NotConnected)?;

        let input = CreateConversationSessionInput {
            workspace_id: None,
            project_path: Some("orpheus-desktop".to_string()),
            user_id: None,
            metadata: Some(serde_json::json!({
                "client": "orpheus-ai",
                "persona": self.persona().grimoire_id(),
            })),
        };

        let request = GraphQLRequest::new(graphql::queries::CREATE_CONVERSATION_SESSION)
            .with_variables(serde_json::json!({ "input": input }));

        let graphql_url = format!("{}/graphql", self.config.endpoint);
        let response = http_client
            .post(&graphql_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::ConnectionError(format!("GraphQL request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::RequestFailed(format!(
                "GraphQL returned status: {}",
                response.status()
            )));
        }

        let result: GraphQLResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| Error::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        if result.has_errors() {
            return Err(Error::RequestFailed(
                result.first_error().unwrap_or("Unknown GraphQL error").to_string(),
            ));
        }

        // Extract session ID from response
        let session_id = result
            .data
            .as_ref()
            .and_then(|d| d.get("createConversationSession"))
            .and_then(|r| r.get("session"))
            .and_then(|s| s.get("id"))
            .and_then(|id| id.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::InvalidResponse("Missing session ID in response".to_string()))?;

        info!("Created conversation session: {}", session_id);
        *self.session_id.write() = Some(session_id.clone());

        Ok(session_id)
    }

    /// Get current session ID, creating one if needed
    async fn ensure_session(&self) -> Result<String> {
        if let Some(session_id) = self.session_id.read().clone() {
            return Ok(session_id);
        }
        self.create_session().await
    }

    /// Disconnect from the backend
    pub fn disconnect(&mut self) {
        if let Some(ref tx) = self.command_tx {
            let _ = tx.send(ClientCommand::Disconnect);
        }
        self.command_tx = None;
        self.stream_rx = None;
        self.http_client = None;
        if let Some(ref mut client) = self.infernum_client {
            client.disconnect();
        }
        self.infernum_client = None;
        *self.session_id.write() = None;
        *self.state.write() = ConnectionState::Disconnected;
        info!("Disconnected from AI backend");
    }

    /// Send a message and get a response (async)
    pub async fn send_message(&self, content: &str) -> Result<AiResponse> {
        if !self.is_connected() {
            return Err(Error::NotConnected);
        }

        let persona = self.persona();
        let request = AiRequest::new(persona)
            .with_message(AiMessage::user(content));

        self.send_request(request).await
    }

    /// Send a request with full control
    pub async fn send_request(&self, request: AiRequest) -> Result<AiResponse> {
        if !self.is_connected() {
            return Err(Error::NotConnected);
        }

        // Mock mode - return simulated response
        if self.config.mock_mode || self.config.provider == AiProvider::Mock {
            return Ok(AiResponse {
                request_id: request.id.clone(),
                content: format!(
                    "[{}] I would help with your music question, but I'm currently in mock mode.",
                    request.persona.display_name()
                ),
                is_chunk: false,
                is_complete: true,
                usage: Some(TokenUsage {
                    input_tokens: 50,
                    output_tokens: 25,
                }),
            });
        }

        // Route to appropriate provider
        match self.config.provider {
            AiProvider::Infernum => self.send_request_infernum(&request).await,
            AiProvider::Leviathan => self.send_request_leviathan(&request).await,
            AiProvider::Mock => unreachable!(), // Handled above
        }
    }

    /// Send request via Infernum local LLM
    async fn send_request_infernum(&self, request: &AiRequest) -> Result<AiResponse> {
        let infernum = self.infernum_client.as_ref().ok_or(Error::NotConnected)?;

        // Build messages for Infernum
        let mut messages = Vec::new();

        // Add context as a system message
        if !request.context.is_empty() {
            let mut context_str = String::from("Context:\n");
            for (key, value) in &request.context {
                context_str.push_str(&format!("- {}: {}\n", key, value));
            }
            messages.push(ChatMessage::system(context_str));
        }

        // Add user messages
        for msg in &request.messages {
            match msg.role {
                MessageRole::User => messages.push(ChatMessage::user(&msg.content)),
                MessageRole::Assistant => messages.push(ChatMessage::assistant(&msg.content)),
                MessageRole::System => messages.push(ChatMessage::system(&msg.content)),
            }
        }

        // Call Infernum
        let content = infernum.chat(messages, Some(&request.persona)).await?;

        Ok(AiResponse {
            request_id: request.id.clone(),
            content,
            is_chunk: false,
            is_complete: true,
            usage: None, // Infernum returns usage separately
        })
    }

    /// Send request via Leviathan GraphQL API
    async fn send_request_leviathan(&self, request: &AiRequest) -> Result<AiResponse> {
        let http_client = self.http_client.as_ref().ok_or(Error::NotConnected)?;

        // Ensure we have a session
        let session_id = self.ensure_session().await?;

        // Build the full message content with context and system prompt
        let mut full_content = String::new();

        // Add persona system prompt
        full_content.push_str(request.persona.system_prompt_prefix());
        full_content.push_str("\n\n");

        // Add any context
        if !request.context.is_empty() {
            full_content.push_str("Context:\n");
            for (key, value) in &request.context {
                full_content.push_str(&format!("- {}: {}\n", key, value));
            }
            full_content.push_str("\n");
        }

        // Add user messages
        for msg in &request.messages {
            match msg.role {
                MessageRole::User => {
                    full_content.push_str(&format!("User: {}\n", msg.content));
                }
                MessageRole::Assistant => {
                    full_content.push_str(&format!("Assistant: {}\n", msg.content));
                }
                MessageRole::System => {
                    full_content.push_str(&format!("System: {}\n", msg.content));
                }
            }
        }

        // Create message in conversation
        let message_input = CreateConversationMessageInput {
            session_id: session_id.clone(),
            role: "user".to_string(),
            content: full_content.clone(),
            tools_used: None,
            requires_approval: Some(false),
            task_id: None,
            intent_type: Some("music_assistance".to_string()),
            metadata: Some(serde_json::json!({
                "persona": request.persona.grimoire_id(),
                "request_id": request.id,
            })),
        };

        let gql_request = GraphQLRequest::new(graphql::queries::CREATE_CONVERSATION_MESSAGE)
            .with_variables(serde_json::json!({ "input": message_input }));

        let graphql_url = format!("{}/graphql", self.config.endpoint);
        let response = http_client
            .post(&graphql_url)
            .json(&gql_request)
            .send()
            .await
            .map_err(|e| Error::ConnectionError(format!("Failed to send message: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::RequestFailed(format!(
                "Message creation failed with status: {}",
                response.status()
            )));
        }

        let _result: GraphQLResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| Error::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        // Create an agent task to process the request
        let task_input = CreateAgentTaskInput {
            persona_code: request.persona.grimoire_id().to_string(),
            description: format!("Music assistance request: {}",
                request.messages.last().map(|m| m.content.as_str()).unwrap_or("No message")),
            goal: Some("Provide helpful music assistance".to_string()),
            project_path: "orpheus-desktop".to_string(),
            max_iterations: Some(3),
            requires_approval: Some(false),
            metadata: Some(serde_json::json!({
                "session_id": session_id,
                "request_id": request.id,
            })),
        };

        let task_request = GraphQLRequest::new(graphql::queries::CREATE_AGENT_TASK)
            .with_variables(serde_json::json!({ "input": task_input }));

        let task_response = http_client
            .post(&graphql_url)
            .json(&task_request)
            .send()
            .await;

        // Try to get AI response from task
        match task_response {
            Ok(resp) if resp.status().is_success() => {
                let task_result: GraphQLResponse<serde_json::Value> = resp
                    .json()
                    .await
                    .map_err(|e| Error::InvalidResponse(format!("Failed to parse task response: {}", e)))?;

                if task_result.has_errors() {
                    // Fall back to simple response if task creation fails
                    warn!("Agent task creation failed: {}", task_result.first_error().unwrap_or("Unknown"));
                    return Ok(self.generate_fallback_response(&request));
                }

                // Extract task result
                let task_id = task_result
                    .data
                    .as_ref()
                    .and_then(|d| d.get("createAgentTask"))
                    .and_then(|r| r.get("task"))
                    .and_then(|t| t.get("taskId"))
                    .and_then(|id| id.as_str());

                if let Some(task_id) = task_id {
                    debug!("Created agent task: {}", task_id);
                    // Poll for result (with timeout)
                    return self.poll_task_result(task_id, &request).await;
                }

                Ok(self.generate_fallback_response(&request))
            }
            Ok(resp) => {
                warn!("Agent task request failed with status: {}", resp.status());
                Ok(self.generate_fallback_response(&request))
            }
            Err(e) => {
                warn!("Agent task request failed: {}", e);
                Ok(self.generate_fallback_response(&request))
            }
        }
    }

    /// Poll for agent task result
    async fn poll_task_result(&self, task_id: &str, request: &AiRequest) -> Result<AiResponse> {
        let http_client = self.http_client.as_ref().ok_or(Error::NotConnected)?;
        let graphql_url = format!("{}/graphql", self.config.endpoint);

        // Poll up to 10 times with 1 second delay
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let poll_request = GraphQLRequest::new(graphql::queries::GET_AGENT_TASK)
                .with_variables(serde_json::json!({ "taskId": task_id }));

            let response = http_client
                .post(&graphql_url)
                .json(&poll_request)
                .send()
                .await;

            if let Ok(resp) = response {
                if let Ok(result) = resp.json::<GraphQLResponse<serde_json::Value>>().await {
                    if let Some(task) = result.data.as_ref().and_then(|d| d.get("agentTask")) {
                        let status = task.get("status").and_then(|s| s.as_str()).unwrap_or("");

                        match status {
                            "COMPLETED" => {
                                let result_text = task
                                    .get("result")
                                    .and_then(|r| r.as_str())
                                    .unwrap_or("Task completed successfully.");

                                return Ok(AiResponse {
                                    request_id: request.id.clone(),
                                    content: result_text.to_string(),
                                    is_chunk: false,
                                    is_complete: true,
                                    usage: None,
                                });
                            }
                            "FAILED" => {
                                let error_text = task
                                    .get("error")
                                    .and_then(|e| e.as_str())
                                    .unwrap_or("Task failed");

                                return Err(Error::RequestFailed(error_text.to_string()));
                            }
                            "CANCELLED" => {
                                return Err(Error::RequestFailed("Task was cancelled".to_string()));
                            }
                            _ => {
                                // Still running, continue polling
                                debug!("Task {} status: {}", task_id, status);
                            }
                        }
                    }
                }
            }
        }

        // Timeout - return fallback response
        warn!("Task {} timed out, returning fallback response", task_id);
        Ok(self.generate_fallback_response(request))
    }

    /// Generate a fallback response when Leviathan is unavailable
    fn generate_fallback_response(&self, request: &AiRequest) -> AiResponse {
        AiResponse {
            request_id: request.id.clone(),
            content: format!(
                "[{}] I apologize, but I'm currently unable to connect to the AI backend. \
                 Please try again later or check that the Leviathan service is running.",
                request.persona.display_name()
            ),
            is_chunk: false,
            is_complete: true,
            usage: None,
        }
    }

    /// Request a chord suggestion based on composition context
    ///
    /// Uses the Music Composer persona to suggest the next chord
    /// based on the current musical context.
    pub async fn suggest_next_chord(
        &self,
        context: &crate::CompositionContext,
    ) -> Result<crate::CompositionSuggestion> {
        use crate::{Chord, ChordType, CompositionSuggestion, SuggestionType};

        if !self.is_connected() {
            return Err(Error::NotConnected);
        }

        // Build a context-aware prompt
        let prompt = format!(
            "Given the current musical context:\n\
             Key: {}\n\
             Tempo: {} BPM\n\
             Time Signature: {}\n\
             Current Chord: {}\n\
             Recent Chords: {}\n\
             Genre: {}\n\n\
             What chord would you suggest next? Explain why.",
            context.key.display_name(),
            context.tempo,
            context.time_signature.display(),
            context.current_chord.as_ref().map(|c| c.display_name()).unwrap_or_else(|| "None".to_string()),
            context.recent_chords(4).iter().map(|c| c.display_name()).collect::<Vec<_>>().join(" → "),
            context.genre.map(|g| g.display_name()).unwrap_or("Unspecified"),
        );

        let request = AiRequest::new(crate::Persona::MusicComposer)
            .with_message(AiMessage::user(&prompt))
            .with_context("key", context.key.display_name())
            .with_context("tempo", context.tempo.to_string());

        let _response = self.send_request(request).await?;

        // In production, we would parse the AI response to extract chord suggestions
        // For now, return a sensible mock suggestion based on common progressions
        let suggested = match context.current_chord.as_ref().map(|c| c.root.as_str()) {
            Some("C") => Chord::new("G", ChordType::Major),
            Some("G") => Chord::new("D", ChordType::Major),
            Some("D") => Chord::new("A", ChordType::Major),
            Some("A") => Chord::new("E", ChordType::Major),
            Some("E") => Chord::new("B", ChordType::Major),
            Some("F") => Chord::new("C", ChordType::Major),
            _ => Chord::new("C", ChordType::Major), // Default to C major
        };

        Ok(CompositionSuggestion {
            suggestion_type: SuggestionType::NextChord(suggested.clone()),
            confidence: 0.85,
            explanation: format!(
                "Based on the {} key and common chord progressions, {} would provide \
                 a natural harmonic movement.",
                context.key.display_name(),
                suggested.display_name()
            ),
            alternatives: vec![
                SuggestionType::NextChord(Chord::new("Am", ChordType::Minor)),
                SuggestionType::NextChord(Chord::new("F", ChordType::Major)),
            ],
        })
    }

    /// Get streaming response receiver
    pub fn stream_receiver(&self) -> Option<&Receiver<AiResponse>> {
        self.stream_rx.as_ref()
    }

    /// Poll for streaming responses (non-blocking)
    pub fn poll_stream(&self) -> Option<AiResponse> {
        self.stream_rx.as_ref().and_then(|rx| rx.try_recv().ok())
    }
}

impl Default for AiClient {
    fn default() -> Self {
        Self::new(AiConfig::default())
    }
}

impl std::fmt::Debug for AiClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiClient")
            .field("endpoint", &self.config.endpoint)
            .field("state", &self.state())
            .field("persona", &self.persona())
            .finish()
    }
}

/// AI client manager for UI integration
pub struct AiManager {
    /// The AI client
    client: Arc<RwLock<AiClient>>,
    /// Active conversations per persona
    conversations: Arc<RwLock<HashMap<Persona, Conversation>>>,
}

impl AiManager {
    /// Create a new AI manager
    pub fn new(config: AiConfig) -> Self {
        Self {
            client: Arc::new(RwLock::new(AiClient::new(config))),
            conversations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the underlying client
    pub fn client(&self) -> Arc<RwLock<AiClient>> {
        Arc::clone(&self.client)
    }

    /// Get or create conversation for persona
    pub fn conversation(&self, persona: Persona) -> Conversation {
        let conversations = self.conversations.read();
        conversations.get(&persona).cloned().unwrap_or_else(|| Conversation::new(persona))
    }

    /// Add message to conversation
    pub fn add_to_conversation(&self, persona: Persona, message: AiMessage) {
        let mut conversations = self.conversations.write();
        conversations
            .entry(persona)
            .or_insert_with(|| Conversation::new(persona))
            .add_message(message);
    }

    /// Clear conversation for persona
    pub fn clear_conversation(&self, persona: Persona) {
        let mut conversations = self.conversations.write();
        if let Some(conv) = conversations.get_mut(&persona) {
            conv.clear();
        }
    }

    /// Get all conversations
    pub fn conversations(&self) -> HashMap<Persona, Conversation> {
        self.conversations.read().clone()
    }

    /// Connection state
    pub fn state(&self) -> ConnectionState {
        self.client.read().state()
    }

    /// Is connected
    pub fn is_connected(&self) -> bool {
        self.client.read().is_connected()
    }
}

impl Default for AiManager {
    fn default() -> Self {
        Self::new(AiConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_default() {
        let config = AiConfig::default();
        assert_eq!(config.endpoint, "http://localhost:8989");
        assert_eq!(config.default_persona, Persona::MusicComposer);
        assert_eq!(config.timeout_secs, 30);
        assert!(config.streaming);
    }

    #[test]
    fn test_ai_config_builder() {
        let config = AiConfig::default()
            .with_endpoint("http://ai.example.com:9000")
            .with_persona(Persona::GuitarCoach)
            .with_timeout(60);

        assert_eq!(config.endpoint, "http://ai.example.com:9000");
        assert_eq!(config.default_persona, Persona::GuitarCoach);
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn test_ai_message_creation() {
        let user_msg = AiMessage::user("What chord comes after G?");
        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(user_msg.content, "What chord comes after G?");
        assert!(!user_msg.id.is_empty());

        let assistant_msg = AiMessage::assistant("You could try C major or D major.");
        assert_eq!(assistant_msg.role, MessageRole::Assistant);

        let system_msg = AiMessage::system("You are a music theory tutor.");
        assert_eq!(system_msg.role, MessageRole::System);
    }

    #[test]
    fn test_ai_request_builder() {
        let request = AiRequest::new(Persona::MusicTheoryTutor)
            .with_message(AiMessage::user("Explain the circle of fifths"))
            .with_context("key", "C major")
            .with_context("tempo", "120");

        assert_eq!(request.persona, Persona::MusicTheoryTutor);
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.context.len(), 2);
        assert_eq!(request.context.get("key"), Some(&"C major".to_string()));
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
        };
        assert_eq!(usage.total(), 150);
    }

    #[test]
    fn test_conversation() {
        let mut conv = Conversation::new(Persona::MixingEngineer);
        assert!(conv.is_empty());
        assert_eq!(conv.len(), 0);
        assert_eq!(conv.persona, Persona::MixingEngineer);

        conv.add_message(AiMessage::user("How do I EQ vocals?"));
        conv.add_message(AiMessage::assistant("Start with a high-pass filter around 80Hz..."));

        assert_eq!(conv.len(), 2);
        assert!(!conv.is_empty());

        let request = conv.to_request();
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.persona, Persona::MixingEngineer);

        conv.clear();
        assert!(conv.is_empty());
    }

    #[test]
    fn test_ai_client_creation() {
        let client = AiClient::new(AiConfig::default());
        assert_eq!(client.state(), ConnectionState::Disconnected);
        assert_eq!(client.persona(), Persona::MusicComposer);
        assert!(!client.is_connected());
    }

    #[test]
    fn test_ai_client_persona_change() {
        let client = AiClient::new(AiConfig::default());
        assert_eq!(client.persona(), Persona::MusicComposer);

        client.set_persona(Persona::GuitarCoach);
        assert_eq!(client.persona(), Persona::GuitarCoach);
    }

    #[tokio::test]
    async fn test_ai_client_connect() {
        let mut client = AiClient::new(AiConfig::default().with_mock_mode());
        assert_eq!(client.state(), ConnectionState::Disconnected);

        let result = client.connect().await;
        assert!(result.is_ok());
        assert_eq!(client.state(), ConnectionState::Connected);
        assert!(client.is_connected());
    }

    #[tokio::test]
    async fn test_ai_client_disconnect() {
        let mut client = AiClient::new(AiConfig::default().with_mock_mode());
        client.connect().await.unwrap();
        assert!(client.is_connected());

        client.disconnect();
        assert!(!client.is_connected());
        assert_eq!(client.state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_ai_client_send_when_disconnected() {
        let client = AiClient::new(AiConfig::default());
        let result = client.send_message("Hello").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::NotConnected)));
    }

    #[tokio::test]
    async fn test_ai_client_send_message() {
        let mut client = AiClient::new(AiConfig::default().with_mock_mode());
        client.connect().await.unwrap();

        let response = client.send_message("What scale should I use?").await;
        assert!(response.is_ok());

        let response = response.unwrap();
        assert!(response.is_complete);
        assert!(!response.content.is_empty());
        assert!(response.usage.is_some());
    }

    #[tokio::test]
    async fn test_ai_client_send_request() {
        let mut client = AiClient::new(AiConfig::default().with_mock_mode());
        client.connect().await.unwrap();

        let request = AiRequest::new(Persona::MasteringEngineer)
            .with_message(AiMessage::user("Is my mix too loud?"))
            .with_context("lufs", "-14");

        let response = client.send_request(request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_ai_client_suggest_next_chord() {
        use crate::{Chord, ChordType, CompositionContext, Genre, Key, SuggestionType};

        let mut client = AiClient::new(AiConfig::default().with_mock_mode());
        client.connect().await.unwrap();

        let context = CompositionContext::new(Key::CMajor, 120.0)
            .with_genre(Genre::Pop)
            .with_current_chord(Chord::new("C", ChordType::Major));

        let suggestion = client.suggest_next_chord(&context).await;
        assert!(suggestion.is_ok());

        let suggestion = suggestion.unwrap();
        // Should suggest G major after C major (common I-V progression)
        if let SuggestionType::NextChord(chord) = &suggestion.suggestion_type {
            assert_eq!(chord.root, "G");
        } else {
            panic!("Expected NextChord suggestion");
        }

        assert!(suggestion.confidence > 0.0);
        assert!(!suggestion.explanation.is_empty());
        assert!(!suggestion.alternatives.is_empty());
    }

    #[tokio::test]
    async fn test_ai_client_suggest_chord_when_disconnected() {
        use crate::{CompositionContext, Key};

        let client = AiClient::new(AiConfig::default());
        let context = CompositionContext::new(Key::GMajor, 100.0);

        let result = client.suggest_next_chord(&context).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::NotConnected)));
    }

    #[test]
    fn test_ai_manager_creation() {
        let manager = AiManager::new(AiConfig::default());
        assert!(!manager.is_connected());
        assert!(manager.conversations().is_empty());
    }

    #[test]
    fn test_ai_manager_conversations() {
        let manager = AiManager::new(AiConfig::default());

        // Add messages to different personas
        manager.add_to_conversation(Persona::MusicComposer, AiMessage::user("Help me compose"));
        manager.add_to_conversation(Persona::GuitarCoach, AiMessage::user("Teach me guitar"));
        manager.add_to_conversation(Persona::MusicComposer, AiMessage::assistant("Sure!"));

        // Check conversations
        let composer_conv = manager.conversation(Persona::MusicComposer);
        assert_eq!(composer_conv.len(), 2);

        let guitar_conv = manager.conversation(Persona::GuitarCoach);
        assert_eq!(guitar_conv.len(), 1);

        // Clear one conversation
        manager.clear_conversation(Persona::MusicComposer);
        let composer_conv = manager.conversation(Persona::MusicComposer);
        assert!(composer_conv.is_empty());

        // Other conversation should still exist
        let guitar_conv = manager.conversation(Persona::GuitarCoach);
        assert_eq!(guitar_conv.len(), 1);
    }

    #[test]
    fn test_ai_response_structure() {
        let response = AiResponse {
            request_id: "test-123".to_string(),
            content: "This is a test response".to_string(),
            is_chunk: false,
            is_complete: true,
            usage: Some(TokenUsage {
                input_tokens: 10,
                output_tokens: 5,
            }),
        };

        assert_eq!(response.request_id, "test-123");
        assert!(response.is_complete);
        assert!(!response.is_chunk);
        assert_eq!(response.usage.unwrap().total(), 15);
    }

    // Integration test with real Leviathan server
    #[tokio::test]
    #[ignore] // Run with `cargo test -- --ignored` when Leviathan is running
    async fn test_real_leviathan_connection() {
        let mut client = AiClient::new(AiConfig::default());

        // Attempt to connect to real Leviathan
        let result = client.connect().await;
        println!("Connection result: {:?}", result);

        if client.is_connected() {
            println!("Connected to Leviathan!");

            // Try to send a message
            let response = client.send_message("What chord should follow a G major?").await;
            match response {
                Ok(resp) => {
                    println!("Response: {}", resp.content);
                    assert!(resp.is_complete);
                }
                Err(e) => {
                    println!("Request failed: {:?}", e);
                }
            }

            client.disconnect();
        } else {
            println!("Could not connect to Leviathan - make sure it's running on localhost:8989");
        }
    }
}
