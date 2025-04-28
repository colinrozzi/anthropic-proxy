use crate::api::AnthropicClient;
use crate::bindings::ntwk::theater::runtime::log;
use crate::types::messages::{CompletionRequest, Operation, ProxyRequest, ProxyResponse};
use crate::types::state::State;

use serde_json::{json, Value};
use std::error::Error;

pub fn handle_message(
    data: Vec<u8>,
    state_bytes: Vec<u8>,
) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
    log("Handling message in anthropic-proxy actor");
    
    // Parse the state
    let state: State = match serde_json::from_slice(&state_bytes) {
        Ok(s) => s,
        Err(e) => {
            return Err(format!("Failed to parse state: {}", e));
        }
    };
    
    // Parse the request
    let request: ProxyRequest = match serde_json::from_slice(&data) {
        Ok(req) => req,
        Err(e) => {
            // If we can't parse as ProxyRequest, try to parse as raw CompletionRequest for compatibility
            match serde_json::from_slice::<CompletionRequest>(&data) {
                Ok(completion_req) => {
                    ProxyRequest {
                        operation: Operation::ChatCompletion(completion_req),
                        request_id: None,
                        callback: None,
                    }
                },
                Err(_) => {
                    return Err(format!("Invalid request format: {}", e));
                }
            }
        }
    };
    
    // Create Anthropic client
    let client = AnthropicClient::new(state.api_key.clone());
    
    // Process based on operation type
    let response = match request.operation {
        Operation::ChatCompletion(completion_req) => {
            log("Processing chat completion request");
            
            match client.generate_completion(completion_req) {
                Ok(completion) => {
                    ProxyResponse {
                        request_id: request.request_id,
                        success: true,
                        error: None,
                        completion: Some(completion),
                        models: None,
                    }
                },
                Err(e) => {
                    ProxyResponse {
                        request_id: request.request_id,
                        success: false,
                        error: Some(format!("Completion generation failed: {}", e)),
                        completion: None,
                        models: None,
                    }
                }
            }
        },
        
        Operation::ListModels => {
            log("Processing list models request");
            
            match client.list_models() {
                Ok(models) => {
                    ProxyResponse {
                        request_id: request.request_id,
                        success: true,
                        error: None,
                        completion: None,
                        models: Some(models),
                    }
                },
                Err(e) => {
                    ProxyResponse {
                        request_id: request.request_id,
                        success: false,
                        error: Some(format!("Failed to list models: {}", e)),
                        completion: None,
                        models: None,
                    }
                }
            }
        },
        
        Operation::StreamCompletion(_) => {
            // Streaming not yet supported in basic message handling
            ProxyResponse {
                request_id: request.request_id,
                success: false,
                error: Some("Streaming is not supported via basic message handling".to_string()),
                completion: None,
                models: None,
            }
        },
    };
    
    // Serialize the response
    let response_bytes = match serde_json::to_vec(&response) {
        Ok(bytes) => bytes,
        Err(e) => {
            return Err(format!("Failed to serialize response: {}", e));
        }
    };
    
    // Return the updated state and response
    Ok((Some(state_bytes), (Some(response_bytes),)))
}
