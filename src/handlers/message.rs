use crate::api::AnthropicClient;
use crate::bindings::ntwk::theater::runtime::log;
use crate::types::state::State;
use anthropic_types::{AnthropicRequest, AnthropicResponse};

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

    // Debug log the incoming request
    log(&format!(
        "Received request data: {}",
        String::from_utf8_lossy(&data)
    ));

    // Parse the request using the shared AnthropicRequest type
    let request: AnthropicRequest = match serde_json::from_slice(&data) {
        Ok(req) => req,
        Err(e) => {
            log(&format!("Error parsing request: {}", e));

            // Try to respond with a properly formatted error
            let error_response = AnthropicResponse::Error {
                error: format!("Invalid request format: {}", e),
            };

            match serde_json::to_vec(&error_response) {
                Ok(bytes) => return Ok((Some(state_bytes), (Some(bytes),))),
                Err(_) => return Err(format!("Invalid request format: {}", e)),
            }
        }
    };

    // Create Anthropic client
    let client = AnthropicClient::new(state.api_key.clone());

    // Process based on operation type
    let response = match request {
        AnthropicRequest::GenerateCompletion { request } => {
            log(&format!(
                "Generating completion with model: {}",
                request.model
            ));

            match client.generate_completion(request) {
                Ok(completion) => AnthropicResponse::Completion { completion },
                Err(e) => {
                    log(&format!("Error generating completion: {}", e));
                    AnthropicResponse::Error {
                        error: format!("Failed to generate completion: {}", e),
                    }
                }
            }
        }

        AnthropicRequest::ListModels => {
            log("Listing available models");

            match client.list_models() {
                Ok(models) => AnthropicResponse::ListModels { models },
                Err(e) => {
                    log(&format!("Error listing models: {}", e));
                    AnthropicResponse::Error {
                        error: format!("Failed to list models: {}", e),
                    }
                }
            }
        }
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
