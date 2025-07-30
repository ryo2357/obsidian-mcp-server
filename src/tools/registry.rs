use crate::{AppError, AppResult};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn execute(&self, params: Value) -> AppResult<Value>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T>(&mut self, tool: T) -> AppResult<()>
    where
        T: Tool + 'static,
    {
        let name = tool.name().to_string();
        if self.tools.contains_key(&name) {
            return Err(AppError::Tool(format!("Tool '{}' is already registered", name)));
        }

        self.tools.insert(name, Arc::new(tool));
        Ok(())
    }

    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn list_tools(&self) -> Vec<crate::mcp::types::Tool> {
        self.tools
            .values()
            .map(|tool| crate::mcp::types::Tool {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: tool.input_schema(),
            })
            .collect()
    }

    pub async fn execute_tool(&self, name: &str, params: Value) -> AppResult<Value> {
        let tool = self
            .get_tool(name)
            .ok_or_else(|| AppError::Tool(format!("Tool '{}' not found", name)))?;

        tool.execute(params).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// 基本的なサンプルツール
pub struct PingTool;

#[async_trait]
impl Tool for PingTool {
    fn name(&self) -> &str {
        "ping"
    }

    fn description(&self) -> &str {
        "A simple ping tool for testing connectivity"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to echo back",
                    "default": "pong"
                }
            }
        })
    }

    async fn execute(&self, params: Value) -> AppResult<Value> {
        let message = params
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("pong");

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": format!("Echo: {}", message)
            }],
            "isError": false
        }))
    }
}
