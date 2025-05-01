use crate::bindings::ntwk::theater::http_client::{send_http, HttpRequest};
use crate::bindings::ntwk::theater::runtime::log;
use anthropic_types::{
    CompletionRequest, CompletionResponse, Message as ApiMessage, ModelInfo, ModelPricing, Usage,
    MessageContent, ToolDefinition, ToolChoice
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
    ApiError(String, Option<u16>),

    /// Invalid parameters
    InvalidParameters(String),
}

impl fmt::Display for AnthropicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnthropicError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            AnthropicError::JsonError(msg) => write!(f, "JSON error: {}", msg),
            AnthropicError::ApiError(msg, code) => match code {
                Some(code) => write!(f, "API error ({}): {}", code, msg),
                None => write!(f, "API error: {}", msg),
            },
            AnthropicError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
        }
    }
}

impl Error for AnthropicError {}

/// Simple function to evaluate a mathematical expression
/// This is a temporary implementation until anthropic-types is updated
fn evaluate_expression(expression: &str) -> Result<f64, String> {
    // This is a very simple evaluator that handles basic operations
    // In a real implementation, you'd use a proper expression parser
    // For now, we'll just check if it's a simple number
    expression.parse::<f64>().map_err(|_| {
        format!("Failed to evaluate expression: '{}'. Only simple numeric values are supported in this implementation.", expression)
    })
}

/// Client for the Anthropic API
pub struct AnthropicClient {
    api_key: String,
    api_version: String,
    base_url: String,
}

impl AnthropicClient {
    /// Create a new Anthropic API client
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            api_version: "2023-06-01".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
        }
    }

    /// Create a chat completion
    pub fn create_completion(
        &self,
        request: &CompletionRequest,
    ) -> Result<CompletionResponse, AnthropicError> {
        log("Creating chat completion with Anthropic API");
        
        // Validate the request
        if request.model.is_empty() {
            return Err(AnthropicError::InvalidParameters(
                "Model is required".to_string(),
            ));
        }
        
        if request.messages.is_empty() {
            return Err(AnthropicError::InvalidParameters(
                "At least one message is required".to_string(),
            ));
        }
        
        // Prepare the request payload
        let mut payload = json!({
            "model": request.model,
            "max_tokens": request.max_tokens.unwrap_or(1024),
            "temperature": request.temperature.unwrap_or(1.0),
            "messages": request.messages,
        });
        
        // Add optional parameters
        if let Some(system) = &request.system {
            payload["system"] = json!(system);
        }
        
        if let Some(top_p) = request.top_p {
            payload["top_p"] = json!(top_p);
        }
        
        // Handle tools if present
        if let Some(tools) = &request.tools {
            payload["tools"] = json!(tools);
        }
        
        if let Some(tool_choice) = &request.tool_choice {
            payload["tool_choice"] = json!(tool_choice);
        }
        
        if let Some(disable_parallel) = request.disable_parallel_tool_use {
            payload["disable_parallel_tool_use"] = json!(disable_parallel);
        }
        
        // Add any additional parameters
        if let Some(additional_params) = &request.additional_params {
            for (key, value) in additional_params {
                payload[key] = value.clone();
            }
        }
        
        log(&format!("Sending request to Anthropic API with payload: {}", 
                    serde_json::to_string(&payload).unwrap_or_default()));
        
        // Send the HTTP request
        let api_version = request.anthropic_version.clone().unwrap_or_else(|| self.api_version.clone());
        
        let request_body = match serde_json::to_vec(&payload) {
            Ok(body) => body,
            Err(e) => return Err(AnthropicError::JsonError(format!("Failed to serialize request body: {}", e)))
        };
        
        let http_request = HttpRequest {
            method: "POST".to_string(),
            uri: format!("{}/v1/messages", self.base_url),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("x-api-key".to_string(), self.api_key.clone()),
                ("anthropic-version".to_string(), api_version),
                ("anthropic-beta".to_string(), "tools-2024-05-16".to_string()), // Enable tools beta
            ],
            body: Some(request_body),
        };
        
        let response = match send_http(&http_request) {
            Ok(res) => res,
            Err(e) => return Err(AnthropicError::HttpError(format!("Failed to send HTTP request: {}", e)))
        };
        
        log(&format!("Received response from Anthropic API with status: {}", response.status));
        
        // Handle error responses
        if response.status < 200 || response.status >= 300 {
            let error_text = match &response.body {
                Some(body) => String::from_utf8_lossy(body),
                None => "No response body".into(),
            };
            
            return Err(AnthropicError::ApiError(
                format!("API request failed: {}", error_text),
                Some(response.status),
            ));
        }
        
        // Parse the response
        let api_response: serde_json::Value = match &response.body {
            Some(body) => match serde_json::from_slice(body) {
                Ok(json) => json,
                Err(e) => return Err(AnthropicError::JsonError(format!("Failed to parse response as JSON: {}", e)))
            },
            None => return Err(AnthropicError::ApiError("Empty response body".to_string(), None)),
        };
        
        log(&format!("Parsed API response: {}", serde_json::to_string(&api_response).unwrap_or_default()));
        
        // Extract the content blocks and other fields to build our completion response
        let content_blocks = if let Some(content) = api_response["content"].as_array() {
            match serde_json::from_value(json!(content)) {
                Ok(blocks) => blocks,
                Err(e) => return Err(AnthropicError::JsonError(format!("Failed to parse content blocks: {}", e)))
            }
        } else {
            vec![]
        };
        
        // Backward compatibility for tools with old content format
        let content = if content_blocks.iter().any(|block| match block {
            MessageContent::Text { text } => true,
            _ => false,
        }) {
            // Extract text content from blocks
            let text_content: String = content_blocks
                .iter()
                .filter_map(|block| match block {
                    MessageContent::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<&str>>()
                .join("\n");
            
            Some(text_content)
        } else {
            None
        };
        
        // Build the response
        let completion_response = CompletionResponse {
            content_blocks,
            id: api_response["id"].as_str().unwrap_or_default().to_string(),
            model: api_response["model"].as_str().unwrap_or_default().to_string(),
            stop_reason: api_response["stop_reason"].as_str().unwrap_or_default().to_string(),
            stop_sequence: None,
            message_type: None,
            usage: Usage {
                input_tokens: api_response["usage"]["input_tokens"]
                    .as_u64()
                    .unwrap_or(0) as u32,
                output_tokens: api_response["usage"]["output_tokens"]
                    .as_u64()
                    .unwrap_or(0) as u32,
            },
            content,
        };
        
        Ok(completion_response)
    }

    /// List available models
    pub fn list_models(&self) -> Result<Vec<ModelInfo>, AnthropicError> {
        // This is a mock implementation since Anthropic doesn't have a models endpoint
        // In a real implementation, you might fetch this from a database or config
        
        Ok(vec![
            ModelInfo {
                id: "claude-3-7-sonnet-20250219".to_string(),
                display_name: "Claude 3.7 Sonnet".to_string(),
                max_tokens: 200000,
                provider: "anthropic".to_string(),
                pricing: Some(ModelPricing {
                    input_cost_per_million_tokens: 15.0,
                    output_cost_per_million_tokens: 75.0,
                }),
            },
            ModelInfo {
                id: "claude-3-5-sonnet-20240307".to_string(),
                display_name: "Claude 3.5 Sonnet".to_string(),
                max_tokens: 200000,
                provider: "anthropic".to_string(),
                pricing: Some(ModelPricing {
                    input_cost_per_million_tokens: 3.0,
                    output_cost_per_million_tokens: 15.0,
                }),
            },
            ModelInfo {
                id: "claude-3-opus-20240229".to_string(),
                display_name: "Claude 3 Opus".to_string(),
                max_tokens: 200000,
                provider: "anthropic".to_string(),
                pricing: Some(ModelPricing {
                    input_cost_per_million_tokens: 15.0,
                    output_cost_per_million_tokens: 75.0,
                }),
            },
            ModelInfo {
                id: "claude-3-sonnet-20240229".to_string(),
                display_name: "Claude 3 Sonnet".to_string(),
                max_tokens: 200000,
                provider: "anthropic".to_string(),
                pricing: Some(ModelPricing {
                    input_cost_per_million_tokens: 3.0,
                    output_cost_per_million_tokens: 15.0,
                }),
            },
            ModelInfo {
                id: "claude-3-haiku-20240307".to_string(),
                display_name: "Claude 3 Haiku".to_string(),
                max_tokens: 200000,
                provider: "anthropic".to_string(),
                pricing: Some(ModelPricing {
                    input_cost_per_million_tokens: 0.25,
                    output_cost_per_million_tokens: 1.25,
                }),
            },
            ModelInfo {
                id: "claude-2.1".to_string(),
                display_name: "Claude 2.1".to_string(),
                max_tokens: 100000,
                provider: "anthropic".to_string(),
                pricing: Some(ModelPricing {
                    input_cost_per_million_tokens: 8.0,
                    output_cost_per_million_tokens: 24.0,
                }),
            },
            ModelInfo {
                id: "claude-instant-1.2".to_string(),
                display_name: "Claude Instant 1.2".to_string(),
                max_tokens: 100000,
                provider: "anthropic".to_string(),
                pricing: Some(ModelPricing {
                    input_cost_per_million_tokens: 1.63,
                    output_cost_per_million_tokens: 5.51,
                }),
            },
        ])
    }

    /// Execute a tool (currently only calculator supported)
    pub fn execute_tool(&self, tool_name: &str, input: &serde_json::Value) -> Result<serde_json::Value, AnthropicError> {
        match tool_name {
            "calculator" => {
                // Extract the expression
                let expression = input["expression"]
                    .as_str()
                    .ok_or_else(|| {
                        AnthropicError::InvalidParameters(
                            "Calculator tool requires an 'expression' parameter as string".to_string(),
                        )
                    })?;
                
                // Evaluate the expression - simple implementation until anthropic-types is updated
                match evaluate_expression(expression) {
                    Ok(result) => Ok(json!({ "result": result })),
                    Err(e) => Err(AnthropicError::InvalidParameters(format!(
                        "Failed to evaluate expression: {}",
                        e
                    ))),
                }
            },
            _ => Err(AnthropicError::InvalidParameters(format!(
                "Unsupported tool: {}",
                tool_name
            ))),
        }
    }
}
