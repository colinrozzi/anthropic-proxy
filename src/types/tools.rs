use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Definition of a tool that Claude can use
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolDefinition {
    /// Name of the tool
    pub name: String,
    
    /// Description of what the tool does
    pub description: String,
    
    /// JSON schema defining the input format
    pub input_schema: Value,
}

/// Configuration for how Claude should choose tools
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ToolChoice {
    /// Claude automatically decides whether to use tools
    #[serde(rename = "auto")]
    Auto,
    
    /// Claude can use any available tool
    #[serde(rename = "any")]
    Any,
    
    /// Claude must use a specific tool
    #[serde(rename = "tool")]
    Tool { name: String },
    
    /// Claude cannot use any tools
    #[serde(rename = "none")]
    None,
}
