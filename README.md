# Anthropic Proxy Actor

A WebAssembly component actor that serves as a proxy for the Anthropic API, making it easy to interact with Claude models within the Theater system.

## Features

- **API Key Management**: Securely stores and manages Anthropic API keys
- **HTTP API**: Provides endpoints for model listing and chat completions
- **Message Passing**: Supports inter-actor communication through the Theater system
- **Model Information**: Includes details about available Claude models, context limits, and pricing
- **Error Handling**: Robust error reporting and handling

## Usage

### HTTP API

The actor exposes these endpoints:

- `GET /` - API documentation
- `GET /models` - List available Claude models
- `POST /chat/completions` - Generate completions from Claude

### Message Passing

The actor supports these operations via the message server:

- `ChatCompletion` - Generate a chat completion
- `ListModels` - Get a list of available models
- `StreamCompletion` - (Future support for streaming)

## Configuration

The actor accepts these configuration parameters during initialization:

```json
{
  "anthropic_api_key": "YOUR_ANTHROPIC_API_KEY",
  "store_id": "optional-store-id",
  "config": {
    "default_model": "claude-3-7-sonnet-20250219",
    "max_cache_size": 100,
    "timeout_ms": 30000,
    "enable_streaming": false
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

## Integration

This actor works seamlessly with the chat actor or any other actor that needs to communicate with Claude models. It abstracts away the details of API authentication, request formatting, and response parsing.

## Example

```rust
// Generate a completion
let request = ProxyRequest {
    operation: Operation::ChatCompletion(CompletionRequest {
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
        stream: None,
        anthropic_version: None,
        additional_params: None,
    }),
    request_id: Some("req-123".to_string()),
    callback: None,
};

// Send request to the actor
let response: ProxyResponse = request_message(actor_id, request);
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
