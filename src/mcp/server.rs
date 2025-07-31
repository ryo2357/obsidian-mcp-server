use crate::config::Config;
use crate::error::{AppResult, McpError};
use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse, InitializeParams, InitializeResult, ServerInfo, ProtocolVersion, ServerCapabilities, ToolsCapability, ListToolsResult};
use anyhow::Context;
use std::io::{BufRead, BufReader, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};

/// MCP サーバー
pub struct McpServer {
    config: Config,
    initialized: bool,
}

impl McpServer {
    /// 新しい MCP サーバーを作成
    pub fn new(config: Config) -> Self {
        Self {
            config,
            initialized: false,
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
            "list_tools" => self.handle_list_tools(&request),
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

    /// list_tools リクエストを処理
    fn handle_list_tools(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        if !self.initialized {
            let error = McpError::internal_error("Server not initialized");
            return JsonRpcResponse::error(request.id.clone(), error);
        }

        // 現在は空のツールリストを返す（001-02 で実装予定）
        let result = ListToolsResult { tools: vec![] };

        let result_value = serde_json::to_value(result).unwrap();
        JsonRpcResponse::success(request.id.clone(), result_value)
    }
}
