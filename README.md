# Anthropic Proxy Actor

A WebAssembly component actor that serves as a proxy for the Anthropic API, making it easy to interact with Claude models within the Theater system through message passing.

## Features

- **API Key Management**: Securely stores and manages Anthropic API keys
- **Message Interface**: Simple request-response messaging system
- **Model Information**: Includes details about available Claude models, context limits, and pricing
- **Error Handling**: Robust error reporting and handling

## Usage

The actor implements a simple request-response message interface that supports:

- **Chat Completion**: Generate responses from Claude models
- **Model Listing**: List available Claude models with their capabilities and pricing

## Configuration

The actor accepts these configuration parameters during initialization:

```json
{
  "anthropic_api_key": "YOUR_ANTHROPIC_API_KEY",
  "store_id": "optional-store-id",
  "config": {
    "default_model": "claude-3-7-sonnet-20250219",
    "max_cache_size": 100,
    "timeout_ms": 30000
  }
}
```

## Building

Build the actor using cargo-component:

```bash
cargo component build --release --target wasm32-unknown-unknown
```

Then update the `component_path` in `manifest.toml` to point to the built WASM file.

## Starting

Start the actor using the Theater system:

```rust
let actor_id = start_actor(
    "/path/to/anthropic-proxy/manifest.toml",
    Some(init_data),
    ("anthropic-proxy-instance",)
);
```

## Message Interface

### Request Format

```rust
AnthropicRequest {
    version: "1.0",
    operation_type: OperationType::ChatCompletion,
    request_id: "req-123",
    completion_request: Some(CompletionRequest {
        model: "claude-3-7-sonnet-20250219",
        messages: [...],
        max_tokens: Some(1024),
        temperature: Some(0.7),
        system: Some("You are a helpful AI assistant."),
        top_p: None,
        anthropic_version: None,
        additional_params: None,
    }),
    params: None,
}
```

### Response Format

```rust
AnthropicResponse {
    version: "1.0",
    request_id: "req-123",
    status: ResponseStatus::Success,
    error: None,
    completion: Some(CompletionResponse { ... }),
    models: None,
}
```

## Example

```rust
// Create a request to generate a completion
let request = AnthropicRequest {
    version: "1.0".to_string(),
    operation_type: OperationType::ChatCompletion,
    request_id: "req-12345".to_string(),
    completion_request: Some(CompletionRequest {
        model: "claude-3-7-sonnet-20250219".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Hello, Claude!".to_string(),
            },
        ],
        max_tokens: Some(1024),
        temperature: Some(0.7),
        system: Some("You are a helpful AI assistant.".to_string()),
        top_p: None,
        anthropic_version: None,
        additional_params: None,
    }),
    params: None,
};

// Send the request and receive response
let response_bytes = request_message(
    anthropic_proxy_actor_id, 
    "request", 
    serde_json::to_vec(&request).unwrap()
)?;

// Parse the response
let response: AnthropicResponse = 
    serde_json::from_slice(&response_bytes).unwrap();

// Handle the response
match response.status {
    ResponseStatus::Success => {
        if let Some(completion) = response.completion {
            println!("Claude said: {}", completion.content);
        } else if let Some(models) = response.models {
            println!("Available models: {}", models.len());
            for model in models {
                println!("- {}: {}", model.id, model.display_name);
            }
        }
    },
    ResponseStatus::Error => {
        println!("Error: {}", response.error.unwrap_or_default());
    },
}
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
