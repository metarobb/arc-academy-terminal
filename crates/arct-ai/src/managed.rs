//! Arc Academy managed AI service

use crate::provider::{AIProvider, AIResponse, StreamingResponse};
use crate::types::{AIResult, CompletionOptions, Message, AIError};
use async_trait::async_trait;

// Placeholder URL for future Arc Academy managed service
#[allow(dead_code)]
const MANAGED_API_URL: &str = "https://api.arcacademy.sh/v1/ai";

/// Managed service provider (Arc Academy hosted)
/// Note: This feature is planned for future release
pub struct ManagedProvider {
    #[allow(dead_code)] // Reserved for future managed service implementation
    auth_token: String,
    #[allow(dead_code)] // Reserved for future managed service implementation
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
        // Arc Academy Managed Service (Planned Feature)
        // This will provide hosted AI access for users without their own API keys
        // Target release: v0.3.0 or later pending infrastructure setup
        Err(AIError::ApiError("Arc Academy Managed Service is planned for a future release. Please use your own API key for now.".to_string()))
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
