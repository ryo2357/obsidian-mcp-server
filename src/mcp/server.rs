use crate::config::Config;
use crate::error::{AppResult, McpError};
use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse, InitializeParams, InitializeResult, ServerInfo, ProtocolVersion, ServerCapabilities, ToolsCapability, ListToolsResult, Tool, CallToolParams, CallToolResult, ToolContent};
use crate::mcp::tools::{execute_save_markdown_file, TARGET_DIRECTORY};
use crate::vault::VaultOperations;
use anyhow::Context;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};

/// MCP サーバー
pub struct McpServer {
    config: Config,
    initialized: bool,
    vault_ops: Option<VaultOperations>,
}

impl McpServer {
    /// 新しい MCP サーバーを作成
    pub fn new(config: Config) -> Self {
        // VaultOperationsを初期化（vault_pathが設定されている場合のみ）
        let vault_ops = if let Ok(vault_path) = config.get_vault_path() {
            Some(VaultOperations::new(
                vault_path.clone(),
                TARGET_DIRECTORY.to_string(),
            ))
        } else {
            None
        };

        Self {
            config,
            initialized: false,
            vault_ops,
        }
    }

    /// サーバーを開始（同期版 - テスト用）
    pub fn run_sync(&mut self) -> AppResult<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let reader = BufReader::new(stdin);

        for line in reader.lines() {
            let line = line.context("Failed to read from stdin")?;
            if line.trim().is_empty() {
                continue;
            }

            let response = self.handle_request(&line)?;
            if let Some(response_json) = response {
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
        }

        Ok(())
    }

    /// サーバーを開始（非同期版）
    pub async fn run_async(&mut self) -> AppResult<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = AsyncBufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await
                .context("Failed to read from stdin")?;
            
            if bytes_read == 0 {
                break; // EOF
            }

            if line.trim().is_empty() {
                continue;
            }

            let response = self.handle_request(&line)?;
            if let Some(response_json) = response {
                stdout.write_all(response_json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }

        Ok(())
    }

    /// リクエストを処理
    fn handle_request(&mut self, request_line: &str) -> AppResult<Option<String>> {
        // JSON-RPC リクエストをパース
        let request: JsonRpcRequest = serde_json::from_str(request_line)
            .context("Failed to parse JSON-RPC request")?;

        let response = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request),
            "tools/list" => self.handle_list_tools(&request),
            "tools/call" => self.handle_call_tool(&request),
            method => {
                let error = McpError::method_not_found(method);
                JsonRpcResponse::error(request.id, error)
            }
        };

        let response_json = serde_json::to_string(&response)
            .context("Failed to serialize response")?;

        Ok(Some(response_json))
    }

    /// initialize リクエストを処理
    fn handle_initialize(&mut self, request: &JsonRpcRequest) -> JsonRpcResponse {
        // パラメータをパース
        let params: Result<InitializeParams, _> = match &request.params {
            Some(params) => serde_json::from_value(params.clone()),
            None => {
                let error = McpError::invalid_params("Missing initialize parameters");
                return JsonRpcResponse::error(request.id.clone(), error);
            }
        };

        let _params = match params {
            Ok(p) => p,
            Err(_) => {
                let error = McpError::invalid_params("Invalid initialize parameters");
                return JsonRpcResponse::error(request.id.clone(), error);
            }
        };

        // 初期化結果を作成
        let result = InitializeResult {
            protocol_version: ProtocolVersion::default(),
            server_info: ServerInfo {
                name: "obsidian-mcp-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: ServerCapabilities {
                experimental: None,
                logging: None,
                prompts: None,
                resources: None,
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
            },
        };

        self.initialized = true;

        let result_value = serde_json::to_value(result).unwrap();
        JsonRpcResponse::success(request.id.clone(), result_value)
    }

    /// tools/list リクエストを処理
    fn handle_list_tools(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        if !self.initialized {
            let error = McpError::internal_error("Server not initialized");
            return JsonRpcResponse::error(request.id.clone(), error);
        }

        let mut tools = vec![];

        // save_markdown_file ツールを追加（vault_opsが利用可能な場合のみ）
        if self.vault_ops.is_some() {
            tools.push(Tool {
                name: "save_markdown_file".to_string(),
                description: "Save a markdown file to the Obsidian vault".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "filename": {
                            "type": "string",
                            "description": "The filename for the markdown file (without .md extension)"
                        },
                        "content": {
                            "type": "string",
                            "description": "The markdown content to save"
                        }
                    },
                    "required": ["filename", "content"]
                }),
            });
        }

        let result = ListToolsResult { tools };

        let result_value = serde_json::to_value(result).unwrap();
        JsonRpcResponse::success(request.id.clone(), result_value)
    }

    /// tools/call リクエストを処理
    fn handle_call_tool(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        if !self.initialized {
            let error = McpError::internal_error("Server not initialized");
            return JsonRpcResponse::error(request.id.clone(), error);
        }

        let params: CallToolParams = match &request.params {
            Some(params) => match serde_json::from_value(params.clone()) {
                Ok(p) => p,
                Err(_) => {
                    let error = McpError::invalid_params("Invalid call tool parameters");
                    return JsonRpcResponse::error(request.id.clone(), error);
                }
            },
            None => {
                let error = McpError::invalid_params("Missing call tool parameters");
                return JsonRpcResponse::error(request.id.clone(), error);
            }
        };

        // ツールを実行
        let result = match params.name.as_str() {
            "save_markdown_file" => self.execute_save_markdown_file(params.arguments),
            _ => {
                let error = McpError::method_not_found(&format!("Tool not found: {}", params.name));
                return JsonRpcResponse::error(request.id.clone(), error);
            }
        };

        match result {
            Ok(content) => {
                let call_result = CallToolResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: content,
                    }],
                    is_error: Some(false),
                };
                let result_value = serde_json::to_value(call_result).unwrap();
                JsonRpcResponse::success(request.id.clone(), result_value)
            }
            Err(error) => {
                let call_result = CallToolResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: format!("Error: {}", error),
                    }],
                    is_error: Some(true),
                };
                let result_value = serde_json::to_value(call_result).unwrap();
                JsonRpcResponse::success(request.id.clone(), result_value)
            }
        }
    }

    /// save_markdown_file ツールを実行
    fn execute_save_markdown_file(&self, arguments: Option<Value>) -> Result<String, anyhow::Error> {
        let vault_ops = self.vault_ops.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vault not configured"))?;

        let args = arguments
            .ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

        let result = execute_save_markdown_file(vault_ops, args)?;
        Ok(serde_json::to_string_pretty(&result)?)
    }
}
