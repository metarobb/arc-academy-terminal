//! AI provider trait and common types

use crate::types::{AIResult, CompletionOptions, Message};
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

/// Response from AI completion
#[derive(Debug, Clone)]
pub struct AIResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: Option<usize>,
}

/// Streaming response type
pub type StreamingResponse = Pin<Box<dyn Stream<Item = AIResult<String>> + Send>>;

/// Trait that all AI providers must implement
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Complete a chat conversation (single response)
    async fn complete(
        &self,
        messages: &[Message],
        options: Option<CompletionOptions>,
    ) -> AIResult<AIResponse>;

    /// Stream a chat conversation (incremental responses)
    async fn stream(
        &self,
        messages: &[Message],
        options: Option<CompletionOptions>,
    ) -> AIResult<StreamingResponse>;

    /// Check if the provider is available/configured
    async fn health_check(&self) -> AIResult<bool>;

    /// Get available models
    async fn list_models(&self) -> AIResult<Vec<String>>;
}

/// Helper to build system prompts for command-line assistance
pub struct SystemPromptBuilder;

impl SystemPromptBuilder {
    /// Build a system prompt for command explanation
    pub fn command_explanation() -> String {
        r#"You are an expert Linux/Bash teacher integrated into Arc Academy Terminal.
Your role is to explain commands, flags, and concepts clearly and concisely.

Guidelines:
- Be concise but thorough
- Use examples when helpful
- Explain WHY, not just WHAT
- Warn about dangerous operations
- Suggest safer alternatives when applicable
- Use markdown formatting for code
- Be encouraging and supportive

Format your responses with clear sections:
1. Quick Summary
2. Detailed Explanation
3. Examples (if applicable)
4. Related Commands/Concepts
5. Tips & Warnings (if applicable)"#.to_string()
    }

    /// Build a system prompt for general terminal assistance
    pub fn terminal_assistant() -> String {
        r#"You are Arc Academy Terminal's AI assistant.
Help users accomplish their goals on the command line.

When suggesting commands:
- Provide the exact command to run
- Explain what it does
- Warn about potential risks
- Offer alternatives if applicable
- Use markdown code blocks

Be concise, practical, and educational."#.to_string()
    }

    /// Build a system prompt for troubleshooting
    pub fn troubleshooting() -> String {
        r#"You are a debugging assistant for terminal issues.
Help users understand and fix errors.

When helping with errors:
1. Identify the root cause
2. Explain what went wrong
3. Provide the fix
4. Explain how to prevent it

Be patient, clear, and educational."#.to_string()
    }
}
