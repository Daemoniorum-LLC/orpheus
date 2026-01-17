//! # Orpheus AI
//!
//! AI integration for Orpheus - Leviathan persona client.
//!
//! This crate provides:
//! - Connection to Leviathan AI backend
//! - Music-specific personas (composer, theory tutor, mixing engineer, etc.)
//! - Composition context for intelligent suggestions
//! - Conversation management
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use orpheus_ai::{AiClient, AiConfig, Persona};
//!
//! // Create and connect client
//! let mut client = AiClient::new(AiConfig::default());
//! client.connect().await?;
//!
//! // Send a message
//! client.set_persona(Persona::MusicComposer);
//! let response = client.send_message("What chord should follow G7?").await?;
//! println!("AI: {}", response.content);
//! ```

pub mod client;
pub mod context;
pub mod graphql;
pub mod infernum;

// Re-exports
pub use client::{
    AiClient, AiConfig, AiManager, AiMessage, AiRequest, AiResponse,
    ConnectionState, Conversation, MessageRole, TokenUsage,
};

pub use context::{
    Chord, ChordType, CompositionContext, CompositionSuggestion,
    Genre, Key, SuggestionType, TimeSignature,
};

pub use infernum::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ChatUsage,
    InfernumClient, InfernumConfig, InfernumHealth,
};

/// AI provider selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiProvider {
    /// Leviathan backend (GraphQL API)
    #[default]
    Leviathan,
    /// Infernum local LLM (OpenAI-compatible API)
    Infernum,
    /// Mock mode for testing
    Mock,
}

impl AiProvider {
    /// Check if this is a local provider (doesn't require external service)
    pub fn is_local(&self) -> bool {
        matches!(self, Self::Infernum | Self::Mock)
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Leviathan => "Leviathan (Cloud)",
            Self::Infernum => "Infernum (Local)",
            Self::Mock => "Mock (Testing)",
        }
    }
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Result type for AI operations
pub type Result<T> = std::result::Result<T, Error>;

/// AI error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Not connected to AI service
    #[error("Not connected to AI service")]
    NotConnected,

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// AI request failed
    #[error("AI request failed: {0}")]
    RequestFailed(String),

    /// Invalid response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Timeout
    #[error("Request timed out")]
    Timeout,

    /// Persona not found
    #[error("Persona not found: {0}")]
    PersonaNotFound(String),
}

/// Available AI personas for music assistance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Persona {
    /// General music composition assistance
    #[default]
    MusicComposer,
    /// Music theory explanations and lessons
    MusicTheoryTutor,
    /// Mixing and production advice
    MixingEngineer,
    /// Mastering and loudness optimization
    MasteringEngineer,
    /// Guitar technique and learning
    GuitarCoach,
    /// Recording session workflow
    SessionAssistant,
    /// General production tutorials
    ProductionTutor,
}

impl Persona {
    /// Get human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::MusicComposer => "Music Composer",
            Self::MusicTheoryTutor => "Music Theory Tutor",
            Self::MixingEngineer => "Mixing Engineer",
            Self::MasteringEngineer => "Mastering Engineer",
            Self::GuitarCoach => "Guitar Coach",
            Self::SessionAssistant => "Session Assistant",
            Self::ProductionTutor => "Production Tutor",
        }
    }

    /// Get persona description
    pub fn description(&self) -> &'static str {
        match self {
            Self::MusicComposer => "Helps with melody, harmony, and arrangement",
            Self::MusicTheoryTutor => "Explains music theory concepts and answers questions",
            Self::MixingEngineer => "Provides mixing techniques and feedback",
            Self::MasteringEngineer => "Advises on mastering, loudness, and final polish",
            Self::GuitarCoach => "Teaches guitar techniques and suggests improvements",
            Self::SessionAssistant => "Helps manage recording sessions and workflows",
            Self::ProductionTutor => "Teaches production techniques and DAW tips",
        }
    }

    /// Get Grimoire persona ID (for loading from grimoire-loader)
    pub fn grimoire_id(&self) -> &'static str {
        match self {
            Self::MusicComposer => "music-composer",
            Self::MusicTheoryTutor => "music-theory-tutor",
            Self::MixingEngineer => "mixing-engineer",
            Self::MasteringEngineer => "mastering-engineer",
            Self::GuitarCoach => "guitar-coach",
            Self::SessionAssistant => "session-assistant",
            Self::ProductionTutor => "production-tutor",
        }
    }

    /// Get all available personas
    pub fn all() -> &'static [Persona] {
        &[
            Persona::MusicComposer,
            Persona::MusicTheoryTutor,
            Persona::MixingEngineer,
            Persona::MasteringEngineer,
            Persona::GuitarCoach,
            Persona::SessionAssistant,
            Persona::ProductionTutor,
        ]
    }

    /// Get suggested system prompt prefix for this persona
    pub fn system_prompt_prefix(&self) -> &'static str {
        match self {
            Self::MusicComposer =>
                "You are a skilled music composer assistant. Help users with melody creation, \
                 chord progressions, song structure, and arrangement ideas.",
            Self::MusicTheoryTutor =>
                "You are a patient music theory tutor. Explain concepts clearly, use examples, \
                 and adapt to the user's skill level.",
            Self::MixingEngineer =>
                "You are an experienced mixing engineer. Provide practical mixing advice, \
                 EQ suggestions, compression techniques, and spatial placement tips.",
            Self::MasteringEngineer =>
                "You are a professional mastering engineer. Advise on loudness targets, \
                 dynamic range, frequency balance, and final polish techniques.",
            Self::GuitarCoach =>
                "You are a friendly guitar coach. Help with technique, chord voicings, \
                 scales, and practice strategies.",
            Self::SessionAssistant =>
                "You are a helpful session assistant. Guide recording workflows, \
                 suggest microphone placements, and help organize tracks.",
            Self::ProductionTutor =>
                "You are a knowledgeable music production tutor. Teach DAW techniques, \
                 synthesis, sampling, and creative production methods.",
        }
    }
}

impl std::fmt::Display for Persona {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_display_name() {
        assert_eq!(Persona::MusicComposer.display_name(), "Music Composer");
        assert_eq!(Persona::GuitarCoach.display_name(), "Guitar Coach");
        assert_eq!(Persona::MixingEngineer.display_name(), "Mixing Engineer");
    }

    #[test]
    fn test_persona_grimoire_id() {
        assert_eq!(Persona::MusicComposer.grimoire_id(), "music-composer");
        assert_eq!(Persona::MusicTheoryTutor.grimoire_id(), "music-theory-tutor");
    }

    #[test]
    fn test_persona_all() {
        let all = Persona::all();
        assert_eq!(all.len(), 7);
        assert!(all.contains(&Persona::MusicComposer));
        assert!(all.contains(&Persona::GuitarCoach));
    }

    #[test]
    fn test_persona_description() {
        assert!(!Persona::MusicComposer.description().is_empty());
        assert!(Persona::MixingEngineer.description().contains("mixing"));
    }

    #[test]
    fn test_persona_default() {
        assert_eq!(Persona::default(), Persona::MusicComposer);
    }

    #[test]
    fn test_persona_display_trait() {
        let persona = Persona::GuitarCoach;
        assert_eq!(format!("{}", persona), "Guitar Coach");
    }

    #[test]
    fn test_error_types() {
        let err = Error::NotConnected;
        assert!(err.to_string().contains("Not connected"));

        let err = Error::ConnectionError("timeout".to_string());
        assert!(err.to_string().contains("timeout"));

        let err = Error::PersonaNotFound("unknown".to_string());
        assert!(err.to_string().contains("unknown"));
    }
}
