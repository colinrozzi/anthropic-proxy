use crate::bindings::ntwk::theater::http_client::{send_http, HttpRequest};
use crate::bindings::ntwk::theater::runtime::log;
use anthropic_types::{
    CompletionRequest, CompletionResponse, Message, MessageContent, ModelInfo, Usage,
};

use serde_json::{json, Value};

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
    pub fn list_models(&self) -> Result<Vec<ModelInfo>, anthropic_types::AnthropicError> {
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
            Err(e) => return Err(anthropic_types::AnthropicError::HttpError(e)),
        };

        // Check status code
        if response.status != 200 {
            let message = String::from_utf8_lossy(&response.body.unwrap_or_default()).to_string();
            return Err(anthropic_types::AnthropicError::ApiError {
                status: response.status,
                message,
            });
        }

        // Parse the response
        let body = response.body.ok_or_else(|| {
            anthropic_types::AnthropicError::InvalidResponse("No response body".to_string())
        })?;

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
                    let max_tokens = ModelInfo::get_max_tokens(id);
                    let pricing = ModelInfo::get_pricing(id);

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
    ) -> Result<CompletionResponse, anthropic_types::AnthropicError> {
        log("Generating completion with Anthropic API");

        // Build the request body
        let mut request_body = json!({
            "model": request.model,
            "max_tokens": request.max_tokens.unwrap_or(4096),
        });

        // Format messages correctly based on content type
        let formatted_messages = self.format_messages(&request.messages);
        request_body["messages"] = json!(formatted_messages);

        // Add optional parameters
        if let Some(temp) = request.temperature {
            request_body["temperature"] = json!(temp);
        }

        if let Some(system) = &request.system {
            request_body["system"] = json!(system);
        }

        // Add tool-related parameters
        if let Some(tools) = &request.tools {
            request_body["tools"] = json!(tools);
        }

        if let Some(tool_choice) = &request.tool_choice {
            request_body["tool_choice"] = json!(tool_choice);
        }

        if let Some(disable_parallel) = request.disable_parallel_tool_use {
            request_body["disable_parallel_tool_use"] = json!(disable_parallel);
        }

        // Add any additional parameters
        if let Some(additional) = &request.additional_params {
            for (key, value) in additional {
                request_body[key] = value.clone();
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
            Err(e) => return Err(anthropic_types::AnthropicError::HttpError(e)),
        };

        // Check status code
        if response.status != 200 {
            let message = String::from_utf8_lossy(&response.body.unwrap_or_default()).to_string();
            return Err(anthropic_types::AnthropicError::ApiError {
                status: response.status,
                message,
            });
        }

        // Parse the response
        let body = response.body.ok_or_else(|| {
            anthropic_types::AnthropicError::InvalidResponse("No response body".to_string())
        })?;

        log(&format!("Got response: {}", String::from_utf8_lossy(&body)));

        self.parse_response(&body)
    }

    /// Parse the API response into a CompletionResponse
    fn parse_response(
        &self,
        body: &[u8],
    ) -> Result<CompletionResponse, anthropic_types::AnthropicError> {
        let response_data: Value = serde_json::from_slice(body)?;

        // Extract required fields
        let id = response_data["id"]
            .as_str()
            .ok_or_else(|| {
                anthropic_types::AnthropicError::InvalidResponse("No message ID".to_string())
            })?
            .to_string();

        let model = response_data["model"]
            .as_str()
            .ok_or_else(|| {
                anthropic_types::AnthropicError::InvalidResponse("No model info".to_string())
            })?
            .to_string();

        let stop_reason = response_data["stop_reason"]
            .as_str()
            .ok_or_else(|| {
                anthropic_types::AnthropicError::InvalidResponse("No stop reason".to_string())
            })?
            .to_string();

        // Extract usage information
        let input_tokens = response_data["usage"]["input_tokens"]
            .as_u64()
            .ok_or_else(|| {
                anthropic_types::AnthropicError::InvalidResponse("No input tokens".to_string())
            })? as u32;

        let output_tokens = response_data["usage"]["output_tokens"]
            .as_u64()
            .ok_or_else(|| {
                anthropic_types::AnthropicError::InvalidResponse("No output tokens".to_string())
            })? as u32;

        // For backward compatibility
        let message_type = response_data["type"].as_str().map(String::from);
        let stop_sequence = response_data["stop_sequence"].as_str().map(String::from);

        // Parse content blocks
        let content_blocks = if let Some(content_array) = response_data["content"].as_array() {
            self.parse_content_blocks(content_array)?
        } else {
            vec![MessageContent::Text {
                text: "".to_string(),
            }]
        };

        // For backward compatibility, extract text content if present
        let content = if !content_blocks.is_empty() {
            if let MessageContent::Text { text } = &content_blocks[0] {
                Some(text.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Create the completion response
        Ok(CompletionResponse {
            content_blocks,
            id,
            model,
            stop_reason,
            stop_sequence,
            message_type,
            usage: Usage {
                input_tokens,
                output_tokens,
            },
            content,
        })
    }

    /// Parse content blocks from API response
    fn parse_content_blocks(
        &self,
        content_array: &[Value],
    ) -> Result<Vec<MessageContent>, anthropic_types::AnthropicError> {
        let mut content_blocks = Vec::new();

        for block in content_array {
            let block_type = block["type"].as_str().unwrap_or("text");

            match block_type {
                "text" => {
                    let text = block["text"]
                        .as_str()
                        .ok_or_else(|| {
                            anthropic_types::AnthropicError::InvalidResponse(
                                "Missing text in text block".to_string(),
                            )
                        })?
                        .to_string();

                    content_blocks.push(MessageContent::Text { text });
                }
                "tool_use" => {
                    let id = block["id"]
                        .as_str()
                        .ok_or_else(|| {
                            anthropic_types::AnthropicError::InvalidResponse(
                                "Missing id in tool_use block".to_string(),
                            )
                        })?
                        .to_string();

                    let name = block["name"]
                        .as_str()
                        .ok_or_else(|| {
                            anthropic_types::AnthropicError::InvalidResponse(
                                "Missing name in tool_use block".to_string(),
                            )
                        })?
                        .to_string();

                    let input = block["input"].clone();

                    content_blocks.push(MessageContent::ToolUse { id, name, input });
                }
                "tool_result" => {
                    let tool_use_id = block["tool_use_id"]
                        .as_str()
                        .ok_or_else(|| {
                            anthropic_types::AnthropicError::InvalidResponse(
                                "Missing tool_use_id in tool_result block".to_string(),
                            )
                        })?
                        .to_string();

                    let content = block["content"].clone();
                    let is_error = block["is_error"].as_bool();

                    content_blocks.push(MessageContent::ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                    });
                }
                _ => {
                    log(&format!("Unknown content block type: {}", block_type));
                    // Skip unknown content types
                }
            }
        }

        Ok(content_blocks)
    }

    /// Helper method to format messages for API request
    fn format_messages(&self, messages: &[Message]) -> Vec<serde_json::Value> {
        messages
            .iter()
            .map(|msg| {
                let mut message_json = json!({
                    "role": msg.role
                });

                // Handle the content field based on whether it's a string or array of content blocks
                if !msg.content.is_empty() {
                    message_json["content"] = json!(msg.content);
                } else {
                    // Legacy support - convert the content string to a text block
                    message_json["content"] = json!([{
                        "type": "text",
                        "text": ""
                    }]);
                }

                message_json
            })
            .collect()
    }
}
