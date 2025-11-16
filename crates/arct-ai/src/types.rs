//! Common types for AI interactions

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for AI operations
pub type AIResult<T> = Result<T, AIError>;

/// AI-specific errors
#[derive(Debug, Error)]
pub enum AIError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Model not available: {0}")]
    ModelNotAvailable(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// AI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIConfig {
    /// Anthropic (Claude) provider
    Anthropic {
        api_key: String,
        model: String,
    },

    /// OpenAI (GPT) provider
    OpenAI {
        api_key: String,
        model: String,
    },

    /// Local LLM (Ollama, LM Studio, etc.)
    Local {
        endpoint: String,
        model: Option<String>,
    },

    /// Arc Academy managed service
    Managed {
        auth_token: String,
    },

    /// Claude Code CLI (for Max subscribers)
    ClaudeCLI {
        model: Option<String>,
    },

    /// AI disabled
    Disabled,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self::Disabled
    }
}

/// Chat message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// Options for AI completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionOptions {
    pub temperature: f32,
    pub max_tokens: usize,
    pub stream: bool,
}

impl Default for CompletionOptions {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 1000,
            stream: false,
        }
    }
}
