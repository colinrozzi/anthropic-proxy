use crate::bindings::ntwk::theater::http_client::{send_http, HttpRequest};
use crate::bindings::ntwk::theater::runtime::log;
use crate::types::api::{
    AnthropicCompletionRequest, AnthropicCompletionResponse, AnthropicError, AnthropicModelInfo,
};

use serde_json::Value;

/// Client for interacting with the Anthropic API
pub struct AnthropicClient {
    /// Anthropic API key
    api_key: String,

    /// Base URL for the API
    base_url: String,

    /// API version to use
    api_version: String,
}

impl AnthropicClient {
    /// Create a new Anthropic client
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            api_version: "2023-06-01".to_string(),
        }
    }

    /// List available models from the Anthropic API
    pub fn list_models(&self) -> Result<Vec<AnthropicModelInfo>, AnthropicError> {
        log("Listing available Anthropic models");

        let request = HttpRequest {
            method: "GET".to_string(),
            uri: format!("{}/models", self.base_url),
            headers: vec![
                ("x-api-key".to_string(), self.api_key.clone()),
                ("anthropic-version".to_string(), self.api_version.clone()),
                ("content-type".to_string(), "application/json".to_string()),
            ],
            body: None,
        };

        // Send the request
        let response = match send_http(&request) {
            Ok(resp) => resp,
            Err(e) => return Err(AnthropicError::HttpError(e)),
        };

        // Check status code
        if response.status != 200 {
            let message = String::from_utf8_lossy(&response.body.unwrap_or_default()).to_string();
            return Err(AnthropicError::ApiError {
                status: response.status,
                message,
            });
        }

        // Parse the response
        let body = response
            .body
            .ok_or_else(|| AnthropicError::InvalidResponse("No response body".to_string()))?;

        log(&format!(
            "Models API response: {}",
            String::from_utf8_lossy(&body)
        ));

        let response_data: Value = serde_json::from_slice(&body)?;

        // Extract the models
        let mut models = Vec::new();

        if let Some(data) = response_data.get("data").and_then(|d| d.as_array()) {
            for model_data in data {
                if let (Some(id), Some(name)) = (
                    model_data.get("id").and_then(|v| v.as_str()),
                    model_data.get("display_name").and_then(|v| v.as_str()),
                ) {
                    let max_tokens = AnthropicModelInfo::get_max_tokens(id);
                    let pricing = AnthropicModelInfo::get_pricing(id);

                    models.push(AnthropicModelInfo {
                        id: id.to_string(),
                        display_name: name.to_string(),
                        max_tokens,
                        provider: "anthropic".to_string(),
                        pricing: Some(pricing),
                    });
                }
            }
        }

        Ok(models)
    }

    /// Generate a completion using the Anthropic API
    pub fn generate_completion(
        &self,
        request: AnthropicCompletionRequest,
    ) -> Result<AnthropicCompletionResponse, AnthropicError> {
        log("Generating completion with Anthropic API");

        // Create the HTTP request
        let http_request = HttpRequest {
            method: "POST".to_string(),
            uri: format!("{}/messages", self.base_url),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("x-api-key".to_string(), self.api_key.clone()),
                ("anthropic-version".to_string(), "2023-06-01".to_string()),
            ],
            body: Some(serde_json::to_vec(&request)?),
        };

        // Send the request
        let response = match send_http(&http_request) {
            Ok(resp) => resp,
            Err(e) => return Err(AnthropicError::HttpError(e)),
        };

        // Check status code
        if response.status != 200 {
            let message = String::from_utf8_lossy(&response.body.unwrap_or_default()).to_string();
            return Err(AnthropicError::ApiError {
                status: response.status,
                message,
            });
        }

        // Parse the response
        let body = response
            .body
            .ok_or_else(|| AnthropicError::InvalidResponse("No response body".to_string()))?;

        log(&format!("Got response: {}", String::from_utf8_lossy(&body)));

        serde_json::from_slice(&body).map_err(|e| AnthropicError::InvalidResponse(e.to_string()))
    }
}
