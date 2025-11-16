//! Arc Academy managed AI service

use crate::provider::{AIProvider, AIResponse, StreamingResponse};
use crate::types::{AIResult, CompletionOptions, Message, AIError};
use async_trait::async_trait;

const MANAGED_API_URL: &str = "https://api.example.com/v1/ai"; // Placeholder for future managed service

/// Managed service provider (Arc Academy hosted)
pub struct ManagedProvider {
    auth_token: String,
    client: reqwest::Client,
}

impl ManagedProvider {
    pub fn new(auth_token: String) -> Self {
        Self {
            auth_token,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AIProvider for ManagedProvider {
    fn name(&self) -> &str {
        "Arc Academy Managed"
    }

    async fn complete(
        &self,
        _messages: &[Message],
        _options: Option<CompletionOptions>,
    ) -> AIResult<AIResponse> {
        // TODO: Implement Arc Academy managed service API
        Err(AIError::ApiError("Service not yet available".to_string()))
    }

    async fn stream(
        &self,
        _messages: &[Message],
        _options: Option<CompletionOptions>,
    ) -> AIResult<StreamingResponse> {
        Err(AIError::ApiError("Service not yet available".to_string()))
    }

    async fn health_check(&self) -> AIResult<bool> {
        // Check Arc Academy service status
        Ok(true)
    }

    async fn list_models(&self) -> AIResult<Vec<String>> {
        Ok(vec!["arc-academy-default".to_string()])
    }
}
