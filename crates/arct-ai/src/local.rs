//! Local LLM provider (Ollama, LM Studio, etc.)

use crate::provider::{AIProvider, AIResponse, StreamingResponse};
use crate::types::{AIResult, CompletionOptions, Message, AIError, Role};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Local LLM provider implementation
/// Supports OpenAI-compatible APIs like Ollama and LM Studio
pub struct LocalProvider {
    endpoint: String,
    model: String,
    client: reqwest::Client,
}

impl LocalProvider {
    pub fn new(endpoint: String, model: String) -> Self {
        Self {
            endpoint,
            model,
            client: reqwest::Client::new(),
        }
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<LocalMessage> {
        messages
            .iter()
            .map(|msg| LocalMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect()
    }

    fn get_completion_url(&self) -> String {
        format!("{}/v1/chat/completions", self.endpoint.trim_end_matches('/'))
    }

    fn get_models_url(&self) -> String {
        format!("{}/v1/models", self.endpoint.trim_end_matches('/'))
    }
}

#[async_trait]
impl AIProvider for LocalProvider {
    fn name(&self) -> &str {
        "Local LLM"
    }

    async fn complete(
        &self,
        messages: &[Message],
        options: Option<CompletionOptions>,
    ) -> AIResult<AIResponse> {
        let opts = options.unwrap_or_default();

        let request = LocalRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            temperature: opts.temperature,
            max_tokens: opts.max_tokens,
            stream: false,
        };

        let response = self
            .client
            .post(&self.get_completion_url())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

            return Err(match status.as_u16() {
                404 => AIError::ApiError(format!("Model '{}' not found. Check if the model is pulled.", self.model)),
                _ => AIError::ApiError(format!("HTTP {}: {}", status, error_text)),
            });
        }

        let local_response: LocalResponse = response.json().await?;

        Ok(AIResponse {
            content: local_response
                .choices
                .first()
                .and_then(|c| Some(c.message.content.clone()))
                .unwrap_or_default(),
            model: local_response.model.unwrap_or_else(|| self.model.clone()),
            tokens_used: local_response.usage.map(|u| u.total_tokens),
        })
    }

    async fn stream(
        &self,
        _messages: &[Message],
        _options: Option<CompletionOptions>,
    ) -> AIResult<StreamingResponse> {
        Err(AIError::ApiError("Streaming not yet implemented for Local LLM".to_string()))
    }

    async fn health_check(&self) -> AIResult<bool> {
        let response = self
            .client
            .get(&self.get_models_url())
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn list_models(&self) -> AIResult<Vec<String>> {
        let response = self
            .client
            .get(&self.get_models_url())
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(vec![self.model.clone()]);
        }

        let models_response: ModelsResponse = response.json().await?;
        Ok(models_response.data.into_iter().map(|m| m.id).collect())
    }
}

#[derive(Debug, Serialize)]
struct LocalRequest {
    model: String,
    messages: Vec<LocalMessage>,
    temperature: f32,
    max_tokens: usize,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct LocalMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct LocalResponse {
    model: Option<String>,
    choices: Vec<LocalChoice>,
    usage: Option<LocalUsage>,
}

#[derive(Debug, Deserialize)]
struct LocalChoice {
    message: LocalMessage,
}

#[derive(Debug, Deserialize)]
struct LocalUsage {
    total_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
}
