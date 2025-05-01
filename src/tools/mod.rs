mod calculator;

use crate::types::tools::ToolDefinition;
use std::collections::HashMap;

pub use calculator::{get_calculator_tool, evaluate_expression};

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
