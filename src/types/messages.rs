use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::tools::{ToolDefinition, ToolChoice};

/// Different types of content that can be in a message
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MessageContent {
    #[serde(rename = "text")]
    Text { text: String },
    
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: serde_json::Value,
        is_error: Option<bool>,
    },
}

/// A single message in a conversation with Claude
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    /// Role of the message sender (user, assistant, system)
    pub role: String,
    
    /// Content of the message as vector of MessageContent objects
    #[serde(default)]
    pub content: Vec<MessageContent>,
    
    /// For backward compatibility, support simple string content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_str: Option<String>,
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
    
    /// Tools to make available to Claude
    pub tools: Option<Vec<ToolDefinition>>,
    
    /// Tool choice configuration
    pub tool_choice: Option<ToolChoice>,
    
    /// Whether to disable parallel tool use
    pub disable_parallel_tool_use: Option<bool>,
    
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
    /// Generated content blocks
    pub content_blocks: Vec<MessageContent>,
    
    /// ID of the message
    pub id: String,
    
    /// Model used for generation
    pub model: String,
    
    /// Reason why generation stopped
    pub stop_reason: String,
    
    /// Stop sequence if applicable (deprecated - kept for backward compatibility)
    pub stop_sequence: Option<String>,
    
    /// Type of message (deprecated - kept for backward compatibility)
    pub message_type: Option<String>,
    
    /// Token usage information
    pub usage: Usage,
    
    /// Original content string for backward compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
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
    
    /// Execute a tool
    ExecuteTool,
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
    
    /// Tool execution result (if operation_type was ExecuteTool)
    pub tool_result: Option<String>,
    
    /// List of available models (if operation_type was ListModels)
    pub models: Option<Vec<ModelInfo>>,
}
