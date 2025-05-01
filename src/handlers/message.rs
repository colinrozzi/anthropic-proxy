use crate::api::AnthropicClient;
use crate::bindings::ntwk::theater::runtime::log;
use anthropic_types::{
    AnthropicRequest, AnthropicResponse, CompletionRequest, OperationType, ResponseStatus
};
use crate::types::state::State;
use crate::tools;

use serde_json::{json, Value};
use std::error::Error;

pub fn handle_request(
    data: Vec<u8>,
    state_bytes: Vec<u8>,
) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
    log("Handling request in anthropic-proxy actor");
    
    // Parse the state
    let state: State = match serde_json::from_slice(&state_bytes) {
        Ok(s) => s,
        Err(e) => {
            log(&format!("Error parsing state: {}", e));
            return Err(format!("Failed to parse state: {}", e));
        }
    };
    
    // Parse the request
    let request: AnthropicRequest = match serde_json::from_slice(&data) {
        Ok(req) => req,
        Err(e) => {
            log(&format!("Error parsing request: {}", e));
            
            // Try to respond with a properly formatted error
            let error_response = AnthropicResponse {
                version: "1.0".to_string(),
                request_id: "unknown".to_string(),
                status: ResponseStatus::Error,
                error: Some(format!("Invalid request format: {}", e)),
                completion: None,
                models: None,
                tool_result: None,
            };
            
            match serde_json::to_vec(&error_response) {
                Ok(bytes) => return Ok((Some(state_bytes), (Some(bytes),))),
                Err(_) => return Err(format!("Invalid request format: {}", e)),
            }
        }
    };
    
    log(&format!("Processing request type: {:?}", request.operation_type));
    
    // Create Anthropic client
    let client = AnthropicClient::new(state.api_key.clone());
    
    // Process based on operation type
    let response = match request.operation_type {
        OperationType::ChatCompletion => {
            // Ensure the completion request is provided
            let completion_req = match request.completion_request {
                Some(req) => req,
                None => {
                    return create_error_response(
                        &state_bytes,
                        &request.request_id,
                        "ChatCompletion operation requires a completion_request",
                    );
                }
            };
            
            log(&format!("Generating completion with model: {}", completion_req.model));
            
            match client.generate_completion(completion_req) {
                Ok(completion) => {
                    AnthropicResponse {
                        version: "1.0".to_string(),
                        request_id: request.request_id,
                        status: ResponseStatus::Success,
                        error: None,
                        completion: Some(completion),
                        models: None,
                        tool_result: None,
                    }
                },
                Err(e) => {
                    log(&format!("Error generating completion: {}", e));
                    AnthropicResponse {
                        version: "1.0".to_string(),
                        request_id: request.request_id,
                        status: ResponseStatus::Error,
                        error: Some(format!("Completion generation failed: {}", e)),
                        completion: None,
                        models: None,
                        tool_result: None,
                    }
                }
            }
        },
        
        OperationType::ListModels => {
            log("Listing available models");
            
            match client.list_models() {
                Ok(models) => {
                    AnthropicResponse {
                        version: "1.0".to_string(),
                        request_id: request.request_id,
                        status: ResponseStatus::Success,
                        error: None,
                        completion: None,
                        models: Some(models),
                        tool_result: None,
                    }
                },
                Err(e) => {
                    log(&format!("Error listing models: {}", e));
                    AnthropicResponse {
                        version: "1.0".to_string(),
                        request_id: request.request_id,
                        status: ResponseStatus::Error,
                        error: Some(format!("Failed to list models: {}", e)),
                        completion: None,
                        models: None,
                        tool_result: None,
                    }
                }
            }
        },
        
        OperationType::ExecuteTool => {
            log("Executing tool");
            
            // Extract tool parameters from the request
            let tool_name = match &request.params {
                Some(params) => {
                    match params.get("tool_name") {
                        Some(name) => match name.as_str() {
                            Some(s) => s,
                            None => {
                                return create_error_response(
                                    &state_bytes,
                                    &request.request_id,
                                    "Tool name must be a string",
                                );
                            }
                        },
                        None => {
                            return create_error_response(
                                &state_bytes,
                                &request.request_id,
                                "Tool name not provided",
                            );
                        }
                    }
                },
                None => {
                    return create_error_response(
                        &state_bytes,
                        &request.request_id,
                        "Tool parameters not provided",
                    );
                }
            };
            
            let tool_input = match &request.params {
                Some(params) => {
                    match params.get("tool_input") {
                        Some(input) => input,
                        None => {
                            return create_error_response(
                                &state_bytes,
                                &request.request_id,
                                "Tool input not provided",
                            );
                        }
                    }
                },
                None => {
                    return create_error_response(
                        &state_bytes,
                        &request.request_id,
                        "Tool parameters not provided",
                    );
                }
            };
            
            log(&format!("Executing tool: {} with input: {}", tool_name, tool_input));
            
            // Execute the tool
            match client.execute_tool(tool_name, tool_input) {
                Ok(result) => {
                    AnthropicResponse {
                        version: "1.0".to_string(),
                        request_id: request.request_id,
                        status: ResponseStatus::Success,
                        error: None,
                        completion: None,
                        models: None,
                        tool_result: Some(result),
                    }
                },
                Err(e) => {
                    log(&format!("Error executing tool: {}", e));
                    AnthropicResponse {
                        version: "1.0".to_string(),
                        request_id: request.request_id,
                        status: ResponseStatus::Error,
                        error: Some(format!("Tool execution failed: {}", e)),
                        completion: None,
                        models: None,
                        tool_result: None,
                    }
                }
            }
        },
    };
    
    // Serialize the response
    let response_bytes = match serde_json::to_vec(&response) {
        Ok(bytes) => bytes,
        Err(e) => {
            log(&format!("Error serializing response: {}", e));
            return Err(format!("Failed to serialize response: {}", e));
        }
    };
    
    // Return the updated state and response
    Ok((Some(state_bytes), (Some(response_bytes),)))
}

/// Helper function to create an error response
fn create_error_response(
    state_bytes: &[u8],
    request_id: &str,
    error_message: &str,
) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
    log(&format!("Creating error response: {}", error_message));
    
    let error_response = AnthropicResponse {
        version: "1.0".to_string(),
        request_id: request_id.to_string(),
        status: ResponseStatus::Error,
        error: Some(error_message.to_string()),
        completion: None,
        models: None,
        tool_result: None,
    };
    
    match serde_json::to_vec(&error_response) {
        Ok(bytes) => Ok((Some(state_bytes.to_vec()), (Some(bytes),))),
        Err(e) => Err(format!("Failed to serialize error response: {}", e)),
    }
}
