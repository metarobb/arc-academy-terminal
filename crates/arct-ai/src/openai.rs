//! OpenAI (GPT) AI provider implementation

use crate::provider::{AIProvider, AIResponse, StreamingResponse};
use crate::types::{AIResult, CompletionOptions, Message, AIError, Role};
use async_trait::async_trait;
// Reserved for streaming implementation
// use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// OpenAI provider implementation
pub struct OpenAIProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<OpenAIMessage> {
        messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect()
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "OpenAI (GPT)"
    }

    async fn complete(
        &self,
        messages: &[Message],
        options: Option<CompletionOptions>,
    ) -> AIResult<AIResponse> {
        let opts = options.unwrap_or_default();

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            temperature: opts.temperature,
            max_tokens: opts.max_tokens,
            stream: false,
        };

        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

            return Err(match status.as_u16() {
                401 => AIError::InvalidApiKey,
                429 => AIError::RateLimitExceeded,
                _ => AIError::ApiError(format!("HTTP {}: {}", status, error_text)),
            });
        }

        let openai_response: OpenAIResponse = response.json().await?;

        Ok(AIResponse {
            content: openai_response
                .choices
                .first()
                .and_then(|c| Some(c.message.content.clone()))
                .unwrap_or_default(),
            model: openai_response.model,
            tokens_used: openai_response.usage.map(|u| u.total_tokens),
        })
    }

    async fn stream(
        &self,
        _messages: &[Message],
        _options: Option<CompletionOptions>,
    ) -> AIResult<StreamingResponse> {
        // Streaming requires SSE parsing which is more complex
        // For now, return a simple stream wrapper around complete()
        Err(AIError::ApiError("Streaming not yet implemented for OpenAI".to_string()))
    }

    async fn health_check(&self) -> AIResult<bool> {
        // Try to list models as a health check
        let response = self
            .client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn list_models(&self) -> AIResult<Vec<String>> {
        Ok(vec![
            "gpt-4-turbo-preview".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
            "gpt-3.5-turbo-16k".to_string(),
        ])
    }
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: usize,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    total_tokens: usize,
}
