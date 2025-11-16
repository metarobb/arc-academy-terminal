//! Anthropic (Claude) AI provider implementation

use crate::provider::{AIProvider, AIResponse, StreamingResponse};
use crate::types::{AIError, AIResult, CompletionOptions, Message};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic provider implementation
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: usize,
    temperature: f32,
    messages: Vec<AnthropicMessage>,
    system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    model: String,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: usize,
    output_tokens: usize,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }

    fn convert_messages(&self, messages: &[Message]) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system = None;
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            match msg.role {
                crate::types::Role::System => {
                    system = Some(msg.content.clone());
                }
                crate::types::Role::User => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: msg.content.clone(),
                    });
                }
                crate::types::Role::Assistant => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: msg.content.clone(),
                    });
                }
            }
        }

        (system, anthropic_messages)
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "Anthropic (Claude)"
    }

    async fn complete(
        &self,
        messages: &[Message],
        options: Option<CompletionOptions>,
    ) -> AIResult<AIResponse> {
        let opts = options.unwrap_or_default();
        let (system, anthropic_messages) = self.convert_messages(messages);

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: opts.max_tokens,
            temperature: opts.temperature,
            messages: anthropic_messages,
            system,
        };

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(AIError::ApiError(format!(
                "Anthropic API error ({}): {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response.json().await?;

        let content = anthropic_response
            .content
            .into_iter()
            .map(|block| block.text)
            .collect::<Vec<_>>()
            .join("\n");

        Ok(AIResponse {
            content,
            model: anthropic_response.model,
            tokens_used: Some(
                anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens,
            ),
        })
    }

    async fn stream(
        &self,
        _messages: &[Message],
        _options: Option<CompletionOptions>,
    ) -> AIResult<StreamingResponse> {
        // TODO: Implement streaming support
        Err(AIError::ApiError(
            "Streaming not yet implemented for Anthropic".to_string(),
        ))
    }

    async fn health_check(&self) -> AIResult<bool> {
        // Simple check by listing models
        self.list_models().await.map(|_| true)
    }

    async fn list_models(&self) -> AIResult<Vec<String>> {
        // Anthropic doesn't have a models endpoint, return known models
        Ok(vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = AnthropicProvider::new("test-key".to_string(), "claude-3-sonnet-20240229".to_string());
        assert_eq!(provider.name(), "Anthropic (Claude)");
    }
}
