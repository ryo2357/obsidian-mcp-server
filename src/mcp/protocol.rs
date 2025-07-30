use crate::mcp::types::*;
use crate::{AppError, AppResult};
use serde_json::{json, Value};
use tracing::{debug, error, info};

pub struct ProtocolHandler {
    server_name: String,
    server_version: String,
}

impl ProtocolHandler {
    pub fn new(server_name: String, server_version: String) -> Self {
        Self {
            server_name,
            server_version,
        }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        debug!("Processing request: {}", request.method);

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "initialized" => self.handle_initialized().await,
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tools_call(request.params).await,
            _ => Err(AppError::Protocol(format!(
                "Method not found: {}",
                request.method
            ))),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(err) => {
                error!("Request failed: {}", err);
                let error_code = match err {
                    AppError::Protocol(_) => METHOD_NOT_FOUND,
                    AppError::Json(_) => PARSE_ERROR,
                    _ => INTERNAL_ERROR,
                };

                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: error_code,
                        message: err.to_string(),
                        data: None,
                    }),
                }
            }
        }
    }

    async fn handle_initialize(&self, params: Option<Value>) -> AppResult<Value> {
        let _params: InitializeParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => return Err(AppError::Protocol("Initialize params required".to_string())),
        };

        info!("Client initialized");

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                logging: Some(LoggingCapability {}),
                prompts: None,
                resources: None,
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
            },
            server_info: ServerInfo {
                name: self.server_name.clone(),
                version: self.server_version.clone(),
            },
        };

        Ok(serde_json::to_value(result)?)
    }

    async fn handle_initialized(&self) -> AppResult<Value> {
        info!("Initialization completed");
        Ok(json!({}))
    }

    async fn handle_tools_list(&self) -> AppResult<Value> {
        debug!("Listing available tools");

        let tools = vec![
            // 基本的なサンプルツールを定義
            Tool {
                name: "ping".to_string(),
                description: "A simple ping tool for testing".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo back"
                        }
                    }
                }),
            },
        ];

        let result = ToolsListResult { tools };
        Ok(serde_json::to_value(result)?)
    }

    async fn handle_tools_call(&self, params: Option<Value>) -> AppResult<Value> {
        let params: CallToolParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => return Err(AppError::Protocol("Tool call params required".to_string())),
        };

        debug!("Calling tool: {}", params.name);

        match params.name.as_str() {
            "ping" => {
                let message = if let Some(ref args) = params.arguments {
                    args.get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("pong")
                } else {
                    "pong"
                };

                let result = CallToolResult {
                    content: vec![ToolCallContent::Text {
                        text: format!("Echo: {}", message),
                    }],
                    is_error: Some(false),
                };

                Ok(serde_json::to_value(result)?)
            }
            _ => Err(AppError::Tool(format!("Unknown tool: {}", params.name))),
        }
    }
}
