mod calculator;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub use calculator::{get_calculator_tool, evaluate_expression};

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

/// Registry to manage available tools
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>
}

impl ToolRegistry {
    /// Create a new tool registry with built-in tools
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new()
        };
        
        // Register built-in tools
        registry.register(get_calculator_tool());
        
        registry
    }
    
    /// Register a new tool in the registry
    pub fn register(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.name.clone(), tool);
    }
    
    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }
    
    /// Get all registered tools
    pub fn get_all(&self) -> Vec<ToolDefinition> {
        self.tools.values().cloned().collect()
    }
}
