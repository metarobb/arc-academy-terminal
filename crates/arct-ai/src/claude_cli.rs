//! Claude Code CLI provider - uses the `claude` command directly

use crate::{
    provider::{AIProvider, AIResponse, StreamingResponse},
    types::{AIError, AIResult, CompletionOptions, Message, Role},
};
use async_trait::async_trait;
use futures::stream;
use std::process::Command;

/// Claude Code CLI provider
pub struct ClaudeCLIProvider {
    model: String,
}

impl ClaudeCLIProvider {
    /// Create a new Claude CLI provider
    pub fn new(model: Option<String>) -> Self {
        Self {
            model: model.unwrap_or_else(|| "claude-sonnet-4".to_string()),
        }
    }

    /// Check if Claude CLI is available
    pub fn is_available() -> bool {
        Command::new("claude")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[async_trait]
impl AIProvider for ClaudeCLIProvider {
    fn name(&self) -> &str {
        "Claude CLI"
    }

    async fn complete(
        &self,
        messages: &[Message],
        _options: Option<CompletionOptions>,
    ) -> AIResult<AIResponse> {
        // Build a formatted prompt with system context and conversation
        let mut formatted_prompt = String::new();

        // Add system message if present
        if let Some(system_msg) = messages.iter().find(|m| m.role == Role::System) {
            formatted_prompt.push_str(&system_msg.content);
            formatted_prompt.push_str("\n\n");
        }

        // Add conversation history
        for msg in messages.iter() {
            match msg.role {
                Role::System => continue, // Already added
                Role::User => {
                    formatted_prompt.push_str("User: ");
                    formatted_prompt.push_str(&msg.content);
                    formatted_prompt.push('\n');
                }
                Role::Assistant => {
                    formatted_prompt.push_str("Assistant: ");
                    formatted_prompt.push_str(&msg.content);
                    formatted_prompt.push('\n');
                }
            }
        }

        // Call claude CLI with the message in non-interactive mode
        let output = tokio::task::spawn_blocking({
            let prompt = formatted_prompt.clone();
            move || {
                Command::new("claude")
                    .arg("-p")  // Print mode - non-interactive
                    .arg(&prompt)
                    .output()
            }
        })
        .await
        .map_err(|e| AIError::Unknown(e.to_string()))?
        .map_err(|e| AIError::Unknown(e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AIError::ApiError(format!("Claude CLI failed: {}", error)));
        }

        let response = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(AIResponse {
            content: response,
            model: self.model.clone(),
            tokens_used: None,
        })
    }

    async fn stream(
        &self,
        messages: &[Message],
        options: Option<CompletionOptions>,
    ) -> AIResult<StreamingResponse> {
        // Claude CLI doesn't support streaming, so just return the full message
        let response = self.complete(messages, options).await?;
        let stream_content = stream::once(async move { Ok(response.content) });
        Ok(Box::pin(stream_content))
    }

    async fn health_check(&self) -> AIResult<bool> {
        Ok(Self::is_available())
    }

    async fn list_models(&self) -> AIResult<Vec<String>> {
        // Claude CLI doesn't expose model list, return the configured model
        Ok(vec![self.model.clone()])
    }
}
