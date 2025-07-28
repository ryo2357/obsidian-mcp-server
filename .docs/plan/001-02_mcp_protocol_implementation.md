# MCP プロトコル実装プラン

## 基本情報

- 作成日: 2025-07-28
- 参照仕様書セクション: 要件 1（MCP サーバー基盤の実装）、要件 3（拡張可能な Tool アーキテクチャ）
- 参照設計書セクション: MCPServer 構造体、ToolRegistry 構造体、Tool トレイト、JSON-RPC 処理
- プラン進捗: 2/5（MCP プロトコル・Tool レジストリ実装）
- 前提条件: 001-01 の基盤構造（Config、エラーハンドリング）が実装済み

## 概要と目標

### 実装する機能の詳細説明

このフェーズでは、MCP プロトコルの中核となる JSON-RPC 通信機能と、拡張可能な Tool アーキテクチャを実装します。標準入出力を通じた MCP クライアントとの通信、Tool の登録・管理システム、および基本的な MCP リクエストのハンドリングを行います。

### 達成すべき具体的な目標

1. MCP プロトコルに準拠した JSON-RPC 通信の実装
2. Tool レジストリの実装（Tool 登録・管理・一覧機能）
3. Tool トレイトの定義と基本実装パターンの確立
4. MCP サーバーの中核となる MCPServer 構造体の実装
5. MCP クライアントからの基本リクエスト処理

### requirements.md との対応関係

- 要件 1-2: VSCode での MCP サーバー設定対応のための通信実装
- 要件 3-1: 複数の Tool を登録・管理できる構造の実装
- 要件 3-2: 新しい Tool 追加時の既存コードへの影響最小化
- 要件 3-3: MCP クライアントへの Tool リスト情報提供

## 実装手順

### ステップ 1: MCP プロトコル関連の構造体定義

1. src/mcp/mod.rs に基本構造の作成

   ```rust
   pub mod protocol;
   pub mod server;
   pub mod types;

   pub use server::MCPServer;
   pub use types::{JsonRpcRequest, JsonRpcResponse, ToolInfo, ToolResult};
   ```

2. src/mcp/types.rs に MCP 型定義

   ```rust
   use serde::{Deserialize, Serialize};
   use serde_json::Value;

   #[derive(Debug, Deserialize)]
   pub struct JsonRpcRequest {
       pub jsonrpc: String,
       pub id: Option<Value>,
       pub method: String,
       pub params: Option<Value>,
   }

   #[derive(Debug, Serialize)]
   pub struct JsonRpcResponse {
       pub jsonrpc: String,
       pub id: Option<Value>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub result: Option<Value>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub error: Option<JsonRpcError>,
   }

   #[derive(Debug, Serialize)]
   pub struct JsonRpcError {
       pub code: i32,
       pub message: String,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub data: Option<Value>,
   }

   #[derive(Debug, Serialize, Clone)]
   pub struct ToolInfo {
       pub name: String,
       pub description: String,
       pub input_schema: Value,
   }

   #[derive(Debug, Serialize)]
   pub struct ToolResult {
       pub content: Vec<TextContent>,
       pub is_error: bool,
   }

   #[derive(Debug, Serialize)]
   pub struct TextContent {
       pub text: String,
       #[serde(rename = "type")]
       pub content_type: String,
   }

   impl Default for TextContent {
       fn default() -> Self {
           Self {
               text: String::new(),
               content_type: "text".to_string(),
           }
       }
   }
   ```

### ステップ 2: Tool トレイトとレジストリの実装

1. src/tools/mod.rs に Tool アーキテクチャの作成

   ```rust
   pub mod registry;
   pub mod tool_trait;

   pub use registry::ToolRegistry;
   pub use tool_trait::Tool;

   use crate::mcp::types::{ToolInfo, ToolResult};
   use crate::error::McpError;
   use async_trait::async_trait;
   use serde_json::Value;

   #[async_trait]
   pub trait Tool: Send + Sync {
       fn name(&self) -> &str;
       fn description(&self) -> &str;
       fn input_schema(&self) -> Value;
       async fn execute(&self, args: Value) -> Result<ToolResult, McpError>;
   }
   ```

2. src/tools/registry.rs に ToolRegistry 実装

   ```rust
   use std::collections::HashMap;
   use std::sync::Arc;
   use crate::tools::Tool;
   use crate::mcp::types::ToolInfo;

   pub struct ToolRegistry {
       tools: HashMap<String, Arc<dyn Tool>>,
   }

   impl ToolRegistry {
       pub fn new() -> Self {
           Self {
               tools: HashMap::new(),
           }
       }

       pub fn register_tool(&mut self, tool: Arc<dyn Tool>) {
           let name = tool.name().to_string();
           self.tools.insert(name, tool);
       }

       pub fn get_tool(&self, name: &str) -> Option<&Arc<dyn Tool>> {
           self.tools.get(name)
       }

       pub fn list_tools(&self) -> Vec<ToolInfo> {
           self.tools
               .values()
               .map(|tool| ToolInfo {
                   name: tool.name().to_string(),
                   description: tool.description().to_string(),
                   input_schema: tool.input_schema(),
               })
               .collect()
       }

       pub fn tool_count(&self) -> usize {
           self.tools.len()
       }
   }

   impl Default for ToolRegistry {
       fn default() -> Self {
           Self::new()
       }
   }
   ```

### ステップ 3: MCP サーバーの実装

1. src/mcp/server.rs に MCPServer 構造体実装

   ```rust
   use std::sync::Arc;
   use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
   use log::{info, debug, error, warn};
   use serde_json::Value;

   use crate::config::Config;
   use crate::error::McpError;
   use crate::tools::ToolRegistry;
   use crate::mcp::types::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};

   pub struct MCPServer {
       config: Config,
       tool_registry: Arc<ToolRegistry>,
   }

   impl MCPServer {
       pub fn new(config: Config) -> Self {
           Self {
               config,
               tool_registry: Arc::new(ToolRegistry::new()),
           }
       }

       pub fn register_tool(&mut self, tool: Arc<dyn crate::tools::Tool>) {
           Arc::get_mut(&mut self.tool_registry)
               .expect("ToolRegistry should be mutable")
               .register_tool(tool);
       }

       pub async fn start(&self) -> Result<(), McpError> {
           info!("MCP サーバーを開始します");
           info!("登録済みツール数: {}", self.tool_registry.tool_count());

           let stdin = tokio::io::stdin();
           let mut stdout = tokio::io::stdout();
           let mut reader = BufReader::new(stdin);
           let mut line = String::new();

           loop {
               line.clear();
               match reader.read_line(&mut line).await {
                   Ok(0) => {
                       info!("クライアント接続が終了しました");
                       break;
                   }
                   Ok(_) => {
                       let trimmed = line.trim();
                       if trimmed.is_empty() {
                           continue;
                       }

                       debug!("受信したリクエスト: {}", trimmed);
                       let response = self.handle_request(trimmed).await;
                       let response_json = serde_json::to_string(&response)
                           .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"内部エラー"}}"#.to_string());

                       debug!("レスポンス送信: {}", response_json);
                       if let Err(e) = stdout.write_all(response_json.as_bytes()).await {
                           error!("レスポンス送信エラー: {}", e);
                           break;
                       }
                       if let Err(e) = stdout.write_all(b"\n").await {
                           error!("改行送信エラー: {}", e);
                           break;
                       }
                       if let Err(e) = stdout.flush().await {
                           error!("フラッシュエラー: {}", e);
                           break;
                       }
                   }
                   Err(e) => {
                       error!("標準入力読み込みエラー: {}", e);
                       break;
                   }
               }
           }

           Ok(())
       }

       async fn handle_request(&self, request_str: &str) -> JsonRpcResponse {
           let request: JsonRpcRequest = match serde_json::from_str(request_str) {
               Ok(req) => req,
               Err(e) => {
                   warn!("JSON パースエラー: {}", e);
                   return JsonRpcResponse {
                       jsonrpc: "2.0".to_string(),
                       id: None,
                       result: None,
                       error: Some(JsonRpcError {
                           code: -32700,
                           message: "JSONパースエラー".to_string(),
                           data: None,
                       }),
                   };
               }
           };

           debug!("処理中のメソッド: {}", request.method);

           match request.method.as_str() {
               "tools/list" => self.handle_tools_list(&request).await,
               "tools/call" => self.handle_tools_call(&request).await,
               "initialize" => self.handle_initialize(&request).await,
               _ => {
                   warn!("未対応のメソッド: {}", request.method);
                   JsonRpcResponse {
                       jsonrpc: "2.0".to_string(),
                       id: request.id,
                       result: None,
                       error: Some(JsonRpcError {
                           code: -32601,
                           message: format!("未対応のメソッド: {}", request.method),
                           data: None,
                       }),
                   }
               }
           }
       }

       async fn handle_initialize(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
           info!("初期化リクエストを処理中");
           JsonRpcResponse {
               jsonrpc: "2.0".to_string(),
               id: request.id.clone(),
               result: Some(serde_json::json!({
                   "protocolVersion": "2024-11-05",
                   "capabilities": {
                       "tools": {}
                   },
                   "serverInfo": {
                       "name": "obsidian-vault-mcp",
                       "version": "0.1.0"
                   }
               })),
               error: None,
           }
       }

       async fn handle_tools_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
           debug!("ツールリスト要求を処理中");
           let tools = self.tool_registry.list_tools();
           JsonRpcResponse {
               jsonrpc: "2.0".to_string(),
               id: request.id.clone(),
               result: Some(serde_json::json!({
                   "tools": tools
               })),
               error: None,
           }
       }

       async fn handle_tools_call(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
           let params = match &request.params {
               Some(p) => p,
               None => {
                   return JsonRpcResponse {
                       jsonrpc: "2.0".to_string(),
                       id: request.id.clone(),
                       result: None,
                       error: Some(JsonRpcError {
                           code: -32602,
                           message: "パラメータが必要です".to_string(),
                           data: None,
                       }),
                   };
               }
           };

           let tool_name = match params.get("name").and_then(|v| v.as_str()) {
               Some(name) => name,
               None => {
                   return JsonRpcResponse {
                       jsonrpc: "2.0".to_string(),
                       id: request.id.clone(),
                       result: None,
                       error: Some(JsonRpcError {
                           code: -32602,
                           message: "ツール名が指定されていません".to_string(),
                           data: None,
                       }),
                   };
               }
           };

           let tool = match self.tool_registry.get_tool(tool_name) {
               Some(t) => t,
               None => {
                   return JsonRpcResponse {
                       jsonrpc: "2.0".to_string(),
                       id: request.id.clone(),
                       result: None,
                       error: Some(JsonRpcError {
                           code: -32602,
                           message: format!("未知のツール: {}", tool_name),
                           data: None,
                       }),
                   };
               }
           };

           let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

           debug!("ツール実行: {} with args: {:?}", tool_name, arguments);

           match tool.execute(arguments).await {
               Ok(result) => JsonRpcResponse {
                   jsonrpc: "2.0".to_string(),
                   id: request.id.clone(),
                   result: Some(serde_json::to_value(result).unwrap_or(Value::Null)),
                   error: None,
               },
               Err(e) => {
                   error!("ツール実行エラー {}: {}", tool_name, e);
                   JsonRpcResponse {
                       jsonrpc: "2.0".to_string(),
                       id: request.id.clone(),
                       result: None,
                       error: Some(JsonRpcError {
                           code: -32000,
                           message: format!("ツール実行エラー: {}", e),
                           data: None,
                       }),
                   }
               }
           }
       }
   }
   ```

### ステップ 4: Cargo.toml への依存関係追加

1. 非同期処理用の依存関係追加

   ```toml
   [dependencies]
   # 既存の依存関係に加えて
   async-trait = "0.1"
   ```

### ステップ 5: main.rs の更新

1. src/main.rs に MCPServer 統合

   ```rust
   use env_logger;
   use log::{info, error};
   use obsidian_mcp_server::config::Config;
   use obsidian_mcp_server::error::McpError;
   use obsidian_mcp_server::mcp::MCPServer;

   #[tokio::main]
   async fn main() -> Result<(), McpError> {
       // ログ初期化
       env_logger::init();

       info!("obsidian-vault-mcp starting...");

       // 設定ファイル読み込み
       let config_path = Config::default_config_path();
       info!("設定ファイルパス: {}", config_path.display());

       let config = match Config::load_from_file(config_path.clone()) {
           Ok(config) => {
               info!("設定ファイル読み込み完了: {}", config_path.display());
               info!("Vault ルート: {}", config.vault_path.display());
               config
           }
           Err(e) => {
               error!("設定ファイル読み込みエラー: {}", e);
               return Err(McpError::Config(e));
           }
       };

       // MCP サーバー作成・起動
       let server = MCPServer::new(config);
       info!("MCP サーバー初期化完了");

       // サーバー開始
       if let Err(e) = server.start().await {
           error!("MCP サーバー実行エラー: {}", e);
           return Err(e);
       }

       Ok(())
   }
   ```

### ステップ 6: ビルドテストと動作確認

1. プロジェクトのビルド確認

   ```bash
   cargo build
   ```

2. MCP プロトコル基本動作テスト

   ```bash
   # アプリケーション起動
   RUST_LOG=debug cargo run

   # 別ターミナルで JSON-RPC テスト (手動入力)
   # 入力例1 (初期化):
   {"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}

   # 入力例2 (ツールリスト):
   {"jsonrpc":"2.0","id":2,"method":"tools/list"}
   ```

3. エラーハンドリングテスト

   ```bash
   # 不正なJSONでのテスト
   {"invalid": "json"

   # 未対応メソッドでのテスト
   {"jsonrpc":"2.0","id":3,"method":"unknown/method"}
   ```

## 品質保証

### 各フェーズでの確認ポイント

1. MCP 型定義実装後

   - [ ] JSON シリアライゼーション/デシリアライゼーションが正常動作すること
   - [ ] MCP プロトコル仕様に準拠した構造体定義がされていること

2. Tool トレイト・レジストリ実装後

   - [ ] Tool トレイトが適切に定義されていること
   - [ ] ToolRegistry での Tool 登録・取得が正常動作すること
   - [ ] ツールリスト機能が期待通り動作すること

3. MCPServer 実装後

   - [ ] JSON-RPC リクエストの解析が正常動作すること
   - [ ] 標準入出力での通信が確立されること
   - [ ] エラーレスポンスが適切に返されること

4. 統合テスト後
   - [ ] 初期化リクエストに対する適切なレスポンス
   - [ ] ツールリストリクエストの正常処理
   - [ ] 未対応メソッドへの適切なエラーレスポンス

### requirements.md との整合性確認

- [x] 要件 1-2: VSCode MCP サーバー設定対応（JSON-RPC 通信実装）
- [x] 要件 3-1: 複数 Tool の登録・管理構造実装
- [x] 要件 3-2: 新 Tool 追加時の既存コード影響最小化（Tool トレイト）
- [x] 要件 3-3: MCP クライアントへの Tool リスト提供

### design.md との整合性確認

- [x] MCPServer 構造体の設計パターンに従った実装
- [x] ToolRegistry 構造体の設計パターンに従った実装
- [x] Tool トレイトの設計パターンに従った定義
- [x] JSON-RPC 処理の設計パターンに従った実装

## 次のフェーズへの引き継ぎ事項

1. 実装済みのアーキテクチャ

   - MCP プロトコル通信基盤
   - Tool レジストリシステム
   - 基本的な JSON-RPC ハンドリング

2. 次フェーズでの実装対象

   - VaultHandler の実装
   - MarkdownSaveTool の具体実装
   - ファイル操作機能

3. 継続的な品質保証事項
   - MCP プロトコル仕様への準拠
   - Tool アーキテクチャの拡張性維持
   - エラーハンドリングの一貫性
