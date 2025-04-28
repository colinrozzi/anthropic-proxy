use crate::bindings::ntwk::theater::http_client::{send_http, HttpRequest, HttpResponse};
use crate::bindings::ntwk::theater::runtime::log;
use crate::types::messages::{
    CompletionRequest, CompletionResponse, Message as ApiMessage, ModelInfo, ModelPricing, Usage,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use std::fmt;

/// Error type for Anthropic API operations
#[derive(Debug)]
pub enum AnthropicError {
    /// HTTP request failed
    HttpError(String),

    /// Failed to serialize/deserialize JSON
    JsonError(String),

    /// API returned an error
    ApiError { status: u16, message: String },

    /// Unexpected response format
    InvalidResponse(String),
}

impl fmt::Display for AnthropicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnthropicError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            AnthropicError::JsonError(msg) => write!(f, "JSON error: {}", msg),
            AnthropicError::ApiError { status, message } => {
                write!(f, "API error ({}): {}", status, message)
            }
            AnthropicError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
        }
    }
}

impl Error for AnthropicError {}

impl From<serde_json::Error> for AnthropicError {
    fn from(error: serde_json::Error) -> Self {
        AnthropicError::JsonError(error.to_string())
    }
}

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
    pub fn list_models(&self) -> Result<Vec<ModelInfo>, AnthropicError> {
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
            return Err(AnthropicError::ApiError {
                status: response.status,
                message: String::from_utf8_lossy(&response.body.unwrap_or_default()).to_string(),
            });
        }

        // Parse the response
        let body = response
            .body
            .ok_or_else(|| AnthropicError::InvalidResponse("No response body".to_string()))?;

        let response_data: Value = serde_json::from_slice(&body)?;

        // Extract the models
        let mut models = Vec::new();

        if let Some(data) = response_data.get("data").and_then(|d| d.as_array()) {
            for model_data in data {
                if let (Some(id), Some(name)) = (
                    model_data.get("id").and_then(|v| v.as_str()),
                    model_data.get("name").and_then(|v| v.as_str()),
                ) {
                    // Get max tokens based on model ID
                    let max_tokens = self.get_model_max_tokens(id);
                    let pricing = self.get_model_pricing(id);

                    models.push(ModelInfo {
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
        request: CompletionRequest,
    ) -> Result<CompletionResponse, AnthropicError> {
        log("Generating completion with Anthropic API");

        // Build the request body
        let mut request_body = json!({
            "model": request.model,
            "messages": request.messages,
            "max_tokens": request.max_tokens.unwrap_or(4096),
        });

        // Add optional parameters
        if let Some(temp) = request.temperature {
            request_body["temperature"] = json!(temp);
        }

        if let Some(system) = request.system {
            request_body["system"] = json!(system);
        }

        if let Some(top_p) = request.top_p {
            request_body["top_p"] = json!(top_p);
        }

        // Add any additional parameters
        if let Some(additional) = request.additional_params {
            for (key, value) in additional {
                request_body[key] = value;
            }
        }

        let api_version = request
            .anthropic_version
            .unwrap_or_else(|| self.api_version.clone());

        // Create the HTTP request
        let http_request = HttpRequest {
            method: "POST".to_string(),
            uri: format!("{}/messages", self.base_url),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("x-api-key".to_string(), self.api_key.clone()),
                ("anthropic-version".to_string(), api_version),
            ],
            body: Some(serde_json::to_vec(&request_body)?),
        };

        // Send the request
        let response = match send_http(&http_request) {
            Ok(resp) => resp,
            Err(e) => return Err(AnthropicError::HttpError(e)),
        };

        // Check status code
        if response.status != 200 {
            return Err(AnthropicError::ApiError {
                status: response.status,
                message: String::from_utf8_lossy(&response.body.unwrap_or_default()).to_string(),
            });
        }

        // Parse the response
        let body = response
            .body
            .ok_or_else(|| AnthropicError::InvalidResponse("No response body".to_string()))?;

        log(&format!("Got response: {}", String::from_utf8_lossy(&body)));

        let response_data: Value = serde_json::from_slice(&body)?;

        // Extract required fields
        let content = response_data["content"][0]["text"]
            .as_str()
            .ok_or_else(|| AnthropicError::InvalidResponse("No content text".to_string()))?
            .to_string();

        let id = response_data["id"]
            .as_str()
            .ok_or_else(|| AnthropicError::InvalidResponse("No message ID".to_string()))?
            .to_string();

        let model = response_data["model"]
            .as_str()
            .ok_or_else(|| AnthropicError::InvalidResponse("No model info".to_string()))?
            .to_string();

        let stop_reason = response_data["stop_reason"]
            .as_str()
            .ok_or_else(|| AnthropicError::InvalidResponse("No stop reason".to_string()))?
            .to_string();

        let message_type = response_data["type"]
            .as_str()
            .ok_or_else(|| AnthropicError::InvalidResponse("No message type".to_string()))?
            .to_string();

        let stop_sequence = response_data["stop_sequence"].as_str().map(String::from);

        let input_tokens = response_data["usage"]["input_tokens"]
            .as_u64()
            .ok_or_else(|| AnthropicError::InvalidResponse("No input tokens".to_string()))?
            as u32;

        let output_tokens = response_data["usage"]["output_tokens"]
            .as_u64()
            .ok_or_else(|| AnthropicError::InvalidResponse("No output tokens".to_string()))?
            as u32;

        // Create the completion response
        Ok(CompletionResponse {
            content,
            id,
            model,
            stop_reason,
            stop_sequence,
            message_type,
            usage: Usage {
                input_tokens,
                output_tokens,
            },
        })
    }

    /// Helper function to get max tokens for a given model
    fn get_model_max_tokens(&self, model_id: &str) -> u32 {
        match model_id {
            // Claude 3.7 models
            "claude-3-7-sonnet-20250219" => 200000,

            // Claude 3.5 models
            "claude-3-5-sonnet-20241022"
            | "claude-3-5-haiku-20241022"
            | "claude-3-5-sonnet-20240620" => 200000,

            // Claude 3 models
            "claude-3-opus-20240229" => 200000,
            "claude-3-sonnet-20240229" => 200000,
            "claude-3-haiku-20240307" => 200000,

            // Claude 2 models
            "claude-2.1" | "claude-2.0" => 100000,

            // Default case
            _ => 100000, // Conservative default
        }
    }

    /// Helper function to get pricing for a given model
    fn get_model_pricing(&self, model_id: &str) -> ModelPricing {
        match model_id {
            // Claude 3.7 models
            "claude-3-7-sonnet-20250219" => ModelPricing {
                input_cost_per_million_tokens: 3.00,
                output_cost_per_million_tokens: 15.00,
            },

            // Claude 3.5 models
            "claude-3-5-sonnet-20241022" | "claude-3-5-sonnet-20240620" => ModelPricing {
                input_cost_per_million_tokens: 3.00,
                output_cost_per_million_tokens: 15.00,
            },
            "claude-3-5-haiku-20241022" => ModelPricing {
                input_cost_per_million_tokens: 0.80,
                output_cost_per_million_tokens: 4.00,
            },

            // Claude 3 models
            "claude-3-opus-20240229" => ModelPricing {
                input_cost_per_million_tokens: 15.00,
                output_cost_per_million_tokens: 75.00,
            },
            "claude-3-haiku-20240307" => ModelPricing {
                input_cost_per_million_tokens: 0.25,
                output_cost_per_million_tokens: 1.25,
            },
            "claude-3-sonnet-20240229" => ModelPricing {
                input_cost_per_million_tokens: 3.00,
                output_cost_per_million_tokens: 15.00,
            },

            // Default for older or unknown models
            _ => ModelPricing {
                input_cost_per_million_tokens: 8.00,
                output_cost_per_million_tokens: 24.00,
            },
        }
    }
}
