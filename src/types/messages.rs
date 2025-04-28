use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single message in a conversation with Claude
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    /// Role of the message sender (user, assistant, system)
    pub role: String,
    
    /// Content of the message
    pub content: String,
}

/// Request to generate a completion from Claude
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompletionRequest {
    /// The Claude model to use
    pub model: String,
    
    /// List of messages in the conversation
    pub messages: Vec<Message>,
    
    /// Maximum number of tokens to generate
    pub max_tokens: Option<u32>,
    
    /// Temperature parameter (0.0 to 1.0)
    pub temperature: Option<f32>,
    
    /// System prompt to use
    pub system: Option<String>,
    
    /// Top-p sampling parameter
    pub top_p: Option<f32>,
    
    /// Anthropic API version to use
    pub anthropic_version: Option<String>,
    
    /// Additional parameters for the API
    pub additional_params: Option<HashMap<String, serde_json::Value>>,
}

/// Information about token usage
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Usage {
    /// Number of input tokens
    pub input_tokens: u32,
    
    /// Number of output tokens
    pub output_tokens: u32,
}

/// Response from a completion request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompletionResponse {
    /// Generated content
    pub content: String,
    
    /// ID of the message
    pub id: String,
    
    /// Model used for generation
    pub model: String,
    
    /// Reason why generation stopped
    pub stop_reason: String,
    
    /// Stop sequence if applicable
    pub stop_sequence: Option<String>,
    
    /// Type of message
    pub message_type: String,
    
    /// Token usage information
    pub usage: Usage,
}

/// Information about a model
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelInfo {
    /// Model ID
    pub id: String,
    
    /// Display name
    pub display_name: String,
    
    /// Maximum context window size
    pub max_tokens: u32,
    
    /// Provider name
    pub provider: String,
    
    /// Optional pricing information
    pub pricing: Option<ModelPricing>,
}

/// Pricing information for a model
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelPricing {
    /// Cost per million input tokens
    pub input_cost_per_million_tokens: f64,
    
    /// Cost per million output tokens
    pub output_cost_per_million_tokens: f64,
}

/// Operation types that this actor can handle
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OperationType {
    /// Generate a completion from messages
    ChatCompletion,
    
    /// List available models
    ListModels,
}

/// Request format for the anthropic-proxy actor
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicRequest {
    /// Version of the request format (for future compatibility)
    pub version: String,
    
    /// Type of operation to perform
    pub operation_type: OperationType,
    
    /// Request ID for tracking
    pub request_id: String,
    
    /// Chat completion request (if operation_type is ChatCompletion)
    pub completion_request: Option<CompletionRequest>,
    
    /// Additional parameters specific to the operation
    pub params: Option<HashMap<String, serde_json::Value>>,
}

/// Response status
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ResponseStatus {
    /// Operation succeeded
    Success,
    
    /// Operation failed
    Error,
}

/// Response format from the anthropic-proxy actor
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicResponse {
    /// Version of the response format (for future compatibility)
    pub version: String,
    
    /// Request ID (matching the request)
    pub request_id: String,
    
    /// Status of the operation
    pub status: ResponseStatus,
    
    /// Error message if status is Error
    pub error: Option<String>,
    
    /// Generated completion data (if operation_type was ChatCompletion)
    pub completion: Option<CompletionResponse>,
    
    /// List of available models (if operation_type was ListModels)
    pub models: Option<Vec<ModelInfo>>,
}
