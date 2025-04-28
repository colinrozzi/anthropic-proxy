use crate::api::AnthropicClient;
use crate::bindings::ntwk::theater::http_client::HttpRequest;
use crate::bindings::ntwk::theater::http_types::HttpResponse;
use crate::bindings::ntwk::theater::runtime::log;
use crate::types::messages::{CompletionRequest, Operation, ProxyRequest, ProxyResponse};
use crate::types::state::State;

use serde_json::{json, Value};
use std::error::Error;

pub fn handle_request(
    request: HttpRequest,
    state_bytes: Vec<u8>,
) -> Result<(Option<Vec<u8>>, (HttpResponse,)), String> {
    log("Handling HTTP request in anthropic-proxy actor");
    
    // Parse the state
    let state: State = match serde_json::from_slice(&state_bytes) {
        Ok(s) => s,
        Err(e) => {
            return Err(format!("Failed to parse state: {}", e));
        }
    };
    
    // Process based on the HTTP method and path
    match (request.method.as_str(), request.uri.as_str()) {
        // Documentation endpoint
        ("GET", "/") | ("GET", "/index.html") => {
            log("Serving documentation");
            Ok((
                Some(state_bytes),
                (HttpResponse {
                    status: 200,
                    headers: vec![
                        ("Content-Type".to_string(), "text/html".to_string()),
                    ],
                    body: Some(get_html_docs().into_bytes()),
                },)
            ))
        },
        
        // Models listing endpoint
        ("GET", "/models") => {
            log("Listing available models");
            let client = AnthropicClient::new(state.api_key.clone());
            
            match client.list_models() {
                Ok(models) => {
                    let response = serde_json::to_vec(&json!({
                        "models": models,
                    })).unwrap_or_default();
                    
                    Ok((
                        Some(state_bytes),
                        (HttpResponse {
                            status: 200,
                            headers: vec![
                                ("Content-Type".to_string(), "application/json".to_string()),
                            ],
                            body: Some(response),
                        },)
                    ))
                },
                Err(e) => {
                    Ok((
                        Some(state_bytes),
                        (HttpResponse {
                            status: 500,
                            headers: vec![
                                ("Content-Type".to_string(), "application/json".to_string()),
                            ],
                            body: Some(serde_json::to_vec(&json!({
                                "error": format!("Failed to list models: {}", e),
                            })).unwrap_or_default()),
                        },)
                    ))
                }
            }
        },
        
        // Chat completions API
        ("POST", "/chat/completions") => {
            log("Processing chat completion request");
            
            // Parse the request body
            let body = match request.body {
                Some(b) => b,
                None => {
                    return Ok((
                        Some(state_bytes), 
                        (HttpResponse {
                            status: 400,
                            headers: vec![
                                ("Content-Type".to_string(), "application/json".to_string()),
                            ],
                            body: Some(serde_json::to_vec(&json!({
                                "error": "Missing request body",
                            })).unwrap_or_default()),
                        },)
                    ));
                }
            };
            
            // Parse the request
            let completion_request: CompletionRequest = match serde_json::from_slice(&body) {
                Ok(req) => req,
                Err(e) => {
                    return Ok((
                        Some(state_bytes),
                        (HttpResponse {
                            status: 400,
                            headers: vec![
                                ("Content-Type".to_string(), "application/json".to_string()),
                            ],
                            body: Some(serde_json::to_vec(&json!({
                                "error": format!("Invalid request format: {}", e),
                            })).unwrap_or_default()),
                        },)
                    ));
                }
            };
            
            // Process with the Anthropic client
            let client = AnthropicClient::new(state.api_key.clone());
            
            match client.generate_completion(completion_request) {
                Ok(completion) => {
                    Ok((
                        Some(state_bytes),
                        (HttpResponse {
                            status: 200,
                            headers: vec![
                                ("Content-Type".to_string(), "application/json".to_string()),
                            ],
                            body: Some(serde_json::to_vec(&json!({
                                "completion": completion,
                            })).unwrap_or_default()),
                        },)
                    ))
                },
                Err(e) => {
                    Ok((
                        Some(state_bytes),
                        (HttpResponse {
                            status: 500,
                            headers: vec![
                                ("Content-Type".to_string(), "application/json".to_string()),
                            ],
                            body: Some(serde_json::to_vec(&json!({
                                "error": format!("Completion generation failed: {}", e),
                            })).unwrap_or_default()),
                        },)
                    ))
                }
            }
        },
        
        // Not found for any other path
        _ => {
            Ok((
                Some(state_bytes),
                (HttpResponse {
                    status: 404,
                    headers: vec![
                        ("Content-Type".to_string(), "application/json".to_string()),
                    ],
                    body: Some(serde_json::to_vec(&json!({
                        "error": format!("Not found: {}", request.uri),
                    })).unwrap_or_default()),
                },)
            ))
        }
    }
}

// Simple HTML documentation for the API
fn get_html_docs() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <title>Anthropic Proxy API Documentation</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }
        h1 {
            color: #2c3e50;
            border-bottom: 1px solid #eee;
            padding-bottom: 10px;
        }
        h2 {
            color: #3498db;
            margin-top: 30px;
        }
        code {
            background-color: #f8f8f8;
            padding: 2px 5px;
            border-radius: 3px;
            font-family: monospace;
        }
        pre {
            background-color: #f8f8f8;
            padding: 15px;
            border-radius: 5px;
            overflow-x: auto;
        }
        table {
            border-collapse: collapse;
            width: 100%;
            margin: 20px 0;
        }
        th, td {
            border: 1px solid #ddd;
            padding: 10px;
            text-align: left;
        }
        th {
            background-color: #f2f2f2;
        }
    </style>
</head>
<body>
    <h1>Anthropic Proxy API Documentation</h1>
    
    <p>This service provides a proxy to the Anthropic API, allowing you to interact with Claude models.</p>
    
    <h2>Endpoints</h2>
    
    <h3>GET /models</h3>
    <p>List all available Claude models.</p>
    
    <h4>Response</h4>
    <pre>
{
  "models": [
    {
      "id": "claude-3-7-sonnet-20250219",
      "display_name": "Claude 3.7 Sonnet",
      "max_tokens": 200000,
      "provider": "anthropic",
      "pricing": {
        "input_cost_per_million_tokens": 3.00,
        "output_cost_per_million_tokens": 15.00
      }
    },
    ...
  ]
}
    </pre>
    
    <h3>POST /chat/completions</h3>
    <p>Generate a completion using a Claude model.</p>
    
    <h4>Request</h4>
    <pre>
{
  "model": "claude-3-7-sonnet-20250219",
  "messages": [
    {
      "role": "user",
      "content": "Hello, Claude!"
    }
  ],
  "max_tokens": 4096,
  "temperature": 0.7,
  "system": "You are a helpful AI assistant."
}
    </pre>
    
    <h4>Response</h4>
    <pre>
{
  "completion": {
    "content": "Hello! I'm Claude, an AI assistant created by Anthropic. How can I help you today?",
    "id": "msg_01234abcdef",
    "model": "claude-3-7-sonnet-20250219",
    "stop_reason": "end_turn",
    "stop_sequence": null,
    "message_type": "message",
    "usage": {
      "input_tokens": 15,
      "output_tokens": 18
    }
  }
}
    </pre>
    
    <h2>Error Responses</h2>
    <p>In case of an error, the API will return a JSON response with an error message:</p>
    <pre>
{
  "error": "Error message here"
}
    </pre>
    
    <footer>
        <p>© 2025 Anthropic Proxy Actor</p>
    </footer>
</body>
</html>"#.to_string()
}
