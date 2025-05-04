use genai_types::{
    messages::StopReason, CompletionRequest, CompletionResponse, Message, MessageContent,
    ToolChoice, Usage,
};
use genai_types::{ModelInfo, ModelPricing};
use mcp_protocol::tool::{Tool, ToolContent};
use serde::{Deserialize, Serialize};

/// Different types of content that can be in a message
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum AnthropicMessageContent {
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
        content: Vec<ToolContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

impl From<MessageContent> for AnthropicMessageContent {
    fn from(content: MessageContent) -> Self {
        match content {
            MessageContent::Text { text } => AnthropicMessageContent::Text { text },
            MessageContent::ToolUse { id, name, input } => {
                AnthropicMessageContent::ToolUse { id, name, input }
            }
            MessageContent::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => AnthropicMessageContent::ToolResult {
                tool_use_id,
                content,
                is_error,
            },
        }
    }
}

/// A single message in a conversation with Claude
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicMessage {
    /// Role of the message sender (user, assistant, system)
    pub role: String,

    /// Content of the message as vector of MessageContent objects
    pub content: Vec<AnthropicMessageContent>,
}

impl From<Message> for AnthropicMessage {
    fn from(message: Message) -> Self {
        Self {
            role: message.role,
            content: message
                .content
                .into_iter()
                .map(AnthropicMessageContent::from)
                .collect(),
        }
    }
}

impl AnthropicMessage {
    /// Create a new message with structured content
    pub fn new_structured(role: impl Into<String>, content: Vec<AnthropicMessageContent>) -> Self {
        Self {
            role: role.into(),
            content,
        }
    }
}

/// Request to generate a completion from Claude
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicCompletionRequest {
    /// The Claude model to use
    pub model: String,

    /// List of messages in the conversation
    pub messages: Vec<AnthropicMessage>,

    /// Maximum number of tokens to generate
    pub max_tokens: u32,

    /// Temperature parameter (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// System prompt to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    /// Tools to make available to Claude
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Tool choice configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<AnthropicToolChoice>,

    /// Whether to disable parallel tool use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_parallel_tool_use: Option<bool>,
}

impl From<CompletionRequest> for AnthropicCompletionRequest {
    fn from(request: CompletionRequest) -> Self {
        Self {
            model: request.model,
            messages: request
                .messages
                .into_iter()
                .map(AnthropicMessage::from)
                .collect(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            system: request.system,
            tools: request.tools,
            tool_choice: request.tool_choice.map(AnthropicToolChoice::from),
            disable_parallel_tool_use: request.disable_parallel_tool_use,
        }
    }
}

/// Information about token usage
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicUsage {
    pub input_tokens: u32,

    pub output_tokens: u32,

    pub cache_read_input_tokens: Option<u32>,

    pub cache_creation_input_tokens: Option<u32>,
}

impl From<Usage> for AnthropicUsage {
    fn from(usage: Usage) -> Self {
        Self {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cache_read_input_tokens: None,
            cache_creation_input_tokens: None,
        }
    }
}

impl From<AnthropicUsage> for Usage {
    fn from(usage: AnthropicUsage) -> Self {
        Self {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
        }
    }
}

/// Response from a completion request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicCompletionResponse {
    /// Generated content blocks
    pub content: Vec<AnthropicMessageContent>,

    /// ID of the message
    pub id: String,

    /// Model used for generation
    pub model: String,

    // always "assistant"
    pub role: String,

    /// Reason why generation stopped
    /// can be "end_turn", "max_tokens", "stop_sequence", "tool_use", null
    pub stop_reason: AnthropicStopReason,

    /// Stop sequence if applicable (deprecated - kept for backward compatibility)
    pub stop_sequence: Option<String>,

    /// Message type
    #[serde(rename = "type")]
    pub message_type: String,

    /// Token usage information
    pub usage: AnthropicUsage,
}

impl From<CompletionResponse> for AnthropicCompletionResponse {
    fn from(response: CompletionResponse) -> Self {
        Self {
            content: response
                .content
                .into_iter()
                .map(AnthropicMessageContent::from)
                .collect(),
            id: response.id,
            model: response.model,
            role: response.role,
            stop_reason: response.stop_reason.into(),
            stop_sequence: response.stop_sequence,
            message_type: response.message_type,
            usage: response.usage.into(),
        }
    }
}

impl From<AnthropicCompletionResponse> for CompletionResponse {
    fn from(response: AnthropicCompletionResponse) -> Self {
        Self {
            content: response
                .content
                .into_iter()
                .map(|c| match c {
                    AnthropicMessageContent::Text { text } => MessageContent::Text { text },
                    AnthropicMessageContent::ToolUse { id, name, input } => {
                        MessageContent::ToolUse { id, name, input }
                    }
                    AnthropicMessageContent::ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                    } => MessageContent::ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                    },
                })
                .collect(),
            id: response.id,
            model: response.model,
            role: response.role,
            stop_reason: response.stop_reason.into(),
            stop_sequence: response.stop_sequence,
            message_type: response.message_type,
            usage: response.usage.into(),
        }
    }
}

/// Reason why generation stopped
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AnthropicStopReason {
    /// Generation stopped because the end of a turn was reached
    #[serde(rename = "end_turn")]
    EndTurn,

    /// Generation stopped because the maximum token limit was reached
    #[serde(rename = "max_tokens")]
    MaxTokens,

    /// Generation stopped because a stop sequence was encountered
    #[serde(rename = "stop_sequence")]
    StopSequence,

    /// Generation stopped because a tool was used
    #[serde(rename = "tool_use")]
    ToolUse,
}

impl From<StopReason> for AnthropicStopReason {
    fn from(reason: StopReason) -> Self {
        match reason {
            StopReason::EndTurn => AnthropicStopReason::EndTurn,
            StopReason::MaxTokens => AnthropicStopReason::MaxTokens,
            StopReason::StopSequence => AnthropicStopReason::StopSequence,
            StopReason::ToolUse => AnthropicStopReason::ToolUse,
        }
    }
}

impl From<AnthropicStopReason> for StopReason {
    fn from(reason: AnthropicStopReason) -> Self {
        match reason {
            AnthropicStopReason::EndTurn => StopReason::EndTurn,
            AnthropicStopReason::MaxTokens => StopReason::MaxTokens,
            AnthropicStopReason::StopSequence => StopReason::StopSequence,
            AnthropicStopReason::ToolUse => StopReason::ToolUse,
        }
    }
}

/// Request format for the anthropic-proxy actor
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AnthropicRequest {
    ListModels,

    GenerateCompletion { request: AnthropicCompletionRequest },
}

/// Response status
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ResponseStatus {
    /// Operation succeeded
    #[serde(rename = "Success")]
    Success,

    /// Operation failed
    #[serde(rename = "Error")]
    Error,
}

/// Response format from the anthropic-proxy actor
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AnthropicResponse {
    /// List of available models
    ListModels { models: Vec<AnthropicModelInfo> },

    /// Generated completion
    Completion {
        completion: AnthropicCompletionResponse,
    },

    /// Error response
    Error { error: String },
}

/// Information about a model
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicModelInfo {
    /// Model ID
    pub id: String,

    /// Display name
    pub display_name: String,

    /// Maximum context window size
    pub max_tokens: u32,

    /// Provider name
    pub provider: String,

    /// Optional pricing information
    pub pricing: Option<AnthropicModelPricing>,
}

impl From<ModelInfo> for AnthropicModelInfo {
    fn from(model_info: ModelInfo) -> Self {
        Self {
            id: model_info.id,
            display_name: model_info.display_name,
            max_tokens: model_info.max_tokens,
            provider: model_info.provider,
            pricing: model_info.pricing.map(|p| p.into()),
        }
    }
}

impl From<AnthropicModelInfo> for ModelInfo {
    fn from(model_info: AnthropicModelInfo) -> Self {
        Self {
            id: model_info.id,
            display_name: model_info.display_name,
            max_tokens: model_info.max_tokens,
            provider: model_info.provider,
            pricing: model_info.pricing.map(|p| p.into()),
        }
    }
}

/// Pricing information for a model
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicModelPricing {
    /// Cost per million input tokens
    pub input_cost_per_million_tokens: f64,

    /// Cost per million output tokens
    pub output_cost_per_million_tokens: f64,
}

impl From<ModelPricing> for AnthropicModelPricing {
    fn from(pricing: ModelPricing) -> Self {
        Self {
            input_cost_per_million_tokens: pricing.input_cost_per_million_tokens,
            output_cost_per_million_tokens: pricing.output_cost_per_million_tokens,
        }
    }
}

impl From<AnthropicModelPricing> for ModelPricing {
    fn from(pricing: AnthropicModelPricing) -> Self {
        Self {
            input_cost_per_million_tokens: pricing.input_cost_per_million_tokens,
            output_cost_per_million_tokens: pricing.output_cost_per_million_tokens,
        }
    }
}

impl AnthropicModelInfo {
    /// Get maximum tokens for a given model ID
    pub fn get_max_tokens(model_id: &str) -> u32 {
        match model_id {
            // Claude 3.7 models
            "claude-3-7-sonnet-20250219" => 200000,

            // Claude 3.5 models
            "claude-3-5-sonnet-20241022"
            | "claude-3-5-haiku-20241022"
            | "claude-3-5-sonnet-20240620" => 200000,

            // Claude 3 models
            "claude-3-opus-20240229" => 200000,
            "claude-3-sonnet-20240229" => 200000,
            "claude-3-haiku-20240307" => 200000,

            // Claude 2 models
            "claude-2.1" | "claude-2.0" => 100000,

            // Default case
            _ => 100000, // Conservative default
        }
    }

    /// Get pricing information for a given model ID
    pub fn get_pricing(model_id: &str) -> AnthropicModelPricing {
        match model_id {
            // Claude 3.7 models
            "claude-3-7-sonnet-20250219" => AnthropicModelPricing {
                input_cost_per_million_tokens: 3.00,
                output_cost_per_million_tokens: 15.00,
            },

            // Claude 3.5 models
            "claude-3-5-sonnet-20241022" | "claude-3-5-sonnet-20240620" => AnthropicModelPricing {
                input_cost_per_million_tokens: 3.00,
                output_cost_per_million_tokens: 15.00,
            },
            "claude-3-5-haiku-20241022" => AnthropicModelPricing {
                input_cost_per_million_tokens: 0.80,
                output_cost_per_million_tokens: 4.00,
            },

            // Claude 3 models
            "claude-3-opus-20240229" => AnthropicModelPricing {
                input_cost_per_million_tokens: 15.00,
                output_cost_per_million_tokens: 75.00,
            },
            "claude-3-haiku-20240307" => AnthropicModelPricing {
                input_cost_per_million_tokens: 0.25,
                output_cost_per_million_tokens: 1.25,
            },
            "claude-3-sonnet-20240229" => AnthropicModelPricing {
                input_cost_per_million_tokens: 3.00,
                output_cost_per_million_tokens: 15.00,
            },

            // Default for older or unknown models
            _ => AnthropicModelPricing {
                input_cost_per_million_tokens: 8.00,
                output_cost_per_million_tokens: 24.00,
            },
        }
    }
}

/// Tool choice configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum AnthropicToolChoice {
    /// Model decides whether to use tools
    #[serde(rename = "auto")]
    Auto,

    /// Force model to use a specific tool
    #[serde(rename = "tool")]
    Tool {
        /// Name of the tool to use
        name: String,
    },

    /// Force model to use any available tool
    #[serde(rename = "any")]
    Any,

    /// Force model not to use tools
    #[serde(rename = "none")]
    None,
}

impl From<ToolChoice> for AnthropicToolChoice {
    fn from(choice: ToolChoice) -> Self {
        match choice {
            ToolChoice::Auto => Self::Auto,
            ToolChoice::Tool { name } => Self::Tool { name },
            ToolChoice::Any => Self::Any,
            ToolChoice::None => Self::None,
        }
    }
}

impl AnthropicToolChoice {
    /// Create a new auto tool choice
    pub fn auto() -> Self {
        Self::Auto
    }

    /// Create a new tool-specific choice
    pub fn specific(name: impl Into<String>) -> Self {
        Self::Tool { name: name.into() }
    }

    /// Create a new any tool choice
    pub fn any() -> Self {
        Self::Any
    }

    /// Create a new none tool choice
    pub fn none() -> Self {
        Self::None
    }
}

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
    ApiError { status: u16, message: String },

    /// Unexpected response format
    InvalidResponse(String),

    /// Rate limit exceeded
    RateLimitExceeded { retry_after: Option<u64> },

    /// Authentication error
    AuthenticationError(String),
}

impl fmt::Display for AnthropicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnthropicError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            AnthropicError::JsonError(msg) => write!(f, "JSON error: {}", msg),
            AnthropicError::ApiError { status, message } => {
                write!(f, "API error ({}): {}", status, message)
            }
            AnthropicError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            AnthropicError::RateLimitExceeded { retry_after } => {
                if let Some(seconds) = retry_after {
                    write!(f, "Rate limit exceeded. Retry after {} seconds", seconds)
                } else {
                    write!(f, "Rate limit exceeded")
                }
            }
            AnthropicError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
        }
    }
}

impl Error for AnthropicError {}

impl From<serde_json::Error> for AnthropicError {
    fn from(error: serde_json::Error) -> Self {
        AnthropicError::JsonError(error.to_string())
    }
}
