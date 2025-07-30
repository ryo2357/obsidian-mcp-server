use crate::config::Settings;
use crate::mcp::protocol::ProtocolHandler;
use crate::mcp::types::JsonRpcRequest;
use crate::AppResult;
use std::io::{self, BufRead, BufReader, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{debug, error, info};

pub struct McpServer {
    protocol_handler: ProtocolHandler,
    settings: Settings,
}

impl McpServer {
    pub fn new(settings: Settings) -> Self {
        let protocol_handler = ProtocolHandler::new(
            settings.server.name.clone(),
            settings.server.version.clone(),
        );

        Self {
            protocol_handler,
            settings,
        }
    }

    pub async fn run(&self) -> AppResult<()> {
        info!("Starting MCP server: {}", self.settings.server.name);
        info!("Protocol version: 2024-11-05");

        // STDIO ベースの通信を開始
        self.run_stdio().await
    }

    async fn run_stdio(&self) -> AppResult<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = AsyncBufReader::new(stdin);
        let mut line = String::new();

        info!("MCP server listening on STDIO");

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("EOF reached, shutting down server");
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    debug!("Received: {}", trimmed);

                    match self.process_line(trimmed).await {
                        Ok(response) => {
                            let response_str = serde_json::to_string(&response)?;
                            debug!("Sending: {}", response_str);

                            stdout.write_all(response_str.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                        Err(e) => {
                            error!("Failed to process request: {}", e);
                            // エラーでも JSON-RPC エラーレスポンスを返す
                            let error_response = serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": null,
                                "error": {
                                    "code": -32603,
                                    "message": e.to_string()
                                }
                            });
                            let error_str = serde_json::to_string(&error_response)?;
                            stdout.write_all(error_str.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read from stdin: {}", e);
                    break;
                }
            }
        }

        info!("MCP server shutting down");
        Ok(())
    }

    async fn process_line(
        &self,
        line: &str,
    ) -> AppResult<crate::mcp::types::JsonRpcResponse> {
        let request: JsonRpcRequest = serde_json::from_str(line)?;
        Ok(self.protocol_handler.handle_request(request).await)
    }

    pub fn run_sync(&self) -> AppResult<()> {
        info!("Starting MCP server (sync): {}", self.settings.server.name);

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = BufReader::new(stdin);

        info!("MCP server listening on STDIO (sync mode)");

        for line_result in reader.lines() {
            match line_result {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    debug!("Received: {}", trimmed);

                    match self.process_line_sync(trimmed) {
                        Ok(response) => {
                            let response_str = serde_json::to_string(&response)?;
                            debug!("Sending: {}", response_str);

                            writeln!(stdout, "{}", response_str)?;
                            stdout.flush()?;
                        }
                        Err(e) => {
                            error!("Failed to process request: {}", e);
                            let error_response = serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": null,
                                "error": {
                                    "code": -32603,
                                    "message": e.to_string()
                                }
                            });
                            let error_str = serde_json::to_string(&error_response)?;
                            writeln!(stdout, "{}", error_str)?;
                            stdout.flush()?;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read from stdin: {}", e);
                    break;
                }
            }
        }

        info!("MCP server shutting down");
        Ok(())
    }

    fn process_line_sync(
        &self,
        line: &str,
    ) -> AppResult<crate::mcp::types::JsonRpcResponse> {
        let request: JsonRpcRequest = serde_json::from_str(line)?;
        
        // 同期版では非同期処理を模擬（シンプルな実装）
        let response = match request.method.as_str() {
            "initialize" => {
                let result = crate::mcp::types::InitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: crate::mcp::types::ServerCapabilities {
                        logging: Some(crate::mcp::types::LoggingCapability {}),
                        prompts: None,
                        resources: None,
                        tools: Some(crate::mcp::types::ToolsCapability {
                            list_changed: Some(false),
                        }),
                    },
                    server_info: crate::mcp::types::ServerInfo {
                        name: self.settings.server.name.clone(),
                        version: self.settings.server.version.clone(),
                    },
                };
                
                crate::mcp::types::JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::to_value(result)?),
                    error: None,
                }
            }
            "tools/list" => {
                let tools = vec![
                    crate::mcp::types::Tool {
                        name: "ping".to_string(),
                        description: "A simple ping tool for testing".to_string(),
                        input_schema: serde_json::json!({
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
                
                let result = crate::mcp::types::ToolsListResult { tools };
                
                crate::mcp::types::JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::to_value(result)?),
                    error: None,
                }
            }
            "tools/call" => {
                let params: crate::mcp::types::CallToolParams = match request.params {
                    Some(p) => serde_json::from_value(p)?,
                    None => return Ok(crate::mcp::types::JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(crate::mcp::types::JsonRpcError {
                            code: crate::mcp::types::INVALID_PARAMS,
                            message: "Tool call params required".to_string(),
                            data: None,
                        }),
                    }),
                };
                
                if params.name == "ping" {
                    let message = params
                        .arguments
                        .as_ref()
                        .and_then(|args| args.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("pong");
                    
                    let result = crate::mcp::types::CallToolResult {
                        content: vec![crate::mcp::types::ToolCallContent::Text {
                            text: format!("Echo: {}", message),
                        }],
                        is_error: Some(false),
                    };
                    
                    crate::mcp::types::JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(serde_json::to_value(result)?),
                        error: None,
                    }
                } else {
                    crate::mcp::types::JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(crate::mcp::types::JsonRpcError {
                            code: crate::mcp::types::METHOD_NOT_FOUND,
                            message: format!("Unknown tool: {}", params.name),
                            data: None,
                        }),
                    }
                }
            }
            "initialized" => {
                crate::mcp::types::JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({})),
                    error: None,
                }
            }
            _ => {
                crate::mcp::types::JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(crate::mcp::types::JsonRpcError {
                        code: crate::mcp::types::METHOD_NOT_FOUND,
                        message: format!("Method not found: {}", request.method),
                        data: None,
                    }),
                }
            }
        };
        
        Ok(response)
    }
}
