//! AI provider integrations for Arc Academy Terminal
//!
//! This crate provides a unified interface for multiple AI providers,
//! allowing users to bring their own API keys or use managed services.

pub mod provider;
pub mod anthropic;
pub mod openai;
pub mod local;
pub mod managed;
pub mod claude_cli;
pub mod types;

pub use provider::{AIProvider, AIResponse, StreamingResponse};
pub use types::{AIConfig, AIError, AIResult, Message, Role};

/// Factory for creating AI providers
pub struct AIFactory;

impl AIFactory {
    /// Create an AI provider from configuration
    pub fn create(config: &AIConfig) -> AIResult<Box<dyn AIProvider>> {
        match config {
            AIConfig::Anthropic { api_key, model } => {
                Ok(Box::new(anthropic::AnthropicProvider::new(
                    api_key.clone(),
                    model.clone(),
                )))
            }
            AIConfig::OpenAI { api_key, model } => {
                Ok(Box::new(openai::OpenAIProvider::new(
                    api_key.clone(),
                    model.clone(),
                )))
            }
            AIConfig::Local { endpoint, model } => {
                Ok(Box::new(local::LocalProvider::new(
                    endpoint.clone(),
                    model.clone().unwrap_or_else(|| "default".to_string()),
                )))
            }
            AIConfig::Managed { auth_token } => {
                Ok(Box::new(managed::ManagedProvider::new(
                    auth_token.clone(),
                )))
            }
            AIConfig::ClaudeCLI { model } => {
                // Check if Claude CLI is available
                if !claude_cli::ClaudeCLIProvider::is_available() {
                    return Err(AIError::ConfigError(
                        "Claude CLI not found. Please install Claude Code: https://claude.com/claude-code".to_string()
                    ));
                }
                Ok(Box::new(claude_cli::ClaudeCLIProvider::new(model.clone())))
            }
            AIConfig::Disabled => {
                Err(AIError::ConfigError("AI is disabled".to_string()))
            }
        }
    }
}
