# GitHub Copilot 連携強化プラン（tips フォルダ限定版）

## 基本情報

- 作成日: 2025-07-28
- 参照仕様書セクション: 要件 5（GitHub Copilot 連携機能）、要件 4（Markdown ファイル保存機能）
- 参照設計書セクション: 制限付きファイル操作、UI/UX 設計、セキュリティ設計
- プラン進捗: 4/5（GitHub Copilot 連携・tips フォルダ限定機能実装）
- 前提条件: 001-01（基盤）、001-02（MCP プロトコル）、001-03（Vault 操作）が実装済み

## 概要と目標

### 実装する機能の詳細説明

このフェーズでは、vault/tips フォルダに限定したファイル操作機能を実装します。セキュリティを重視し、指定されたフォルダ内でのみ安全にファイル操作を行い、GitHub Copilot がコンテキストとして既存の tips ファイルを参考にして新しい tips を生成できる機能を提供します。

### 達成すべき具体的な目標

1. tips フォルダ限定のファイル保存機能の強化
2. tips フォルダ内の既存ファイル参照機能（ReadTipsTool）の実装
3. tips フォルダ内ファイル一覧取得機能（ListTipsTool）の実装
4. GitHub Copilot での tips 作成ワークフローの最適化
5. セキュリティ制約の強化（tips フォルダ外アクセス完全防止）

### requirements.md との対応関係

- 要件 5-1: 制限されたコンテキストファイル指定・読み込み・加工機能（tips フォルダ内のみ）
- 要件 5-2: 元ファイル保護での新ファイル保存機能（tips フォルダ内）
- 要件 5-3: Markdown 形式での適切なフォーマット保存
- 要件 4-1: 指定フォルダ（tips）への Markdown ファイル保存
- セキュリティ要件: 設定されたディレクトリ外（tips フォルダ外）へのアクセス制限

## 実装手順

### ステップ 1: tips フォルダ限定のファイル読み込み Tool の実装

1. src/tools/read_tips.rs を作成

   ```rust
   use async_trait::async_trait;
   use serde_json::{json, Value};
   use log::{info, debug, warn};

   use crate::tools::Tool;
   use crate::vault::VaultHandler;
   use crate::mcp::types::{ToolResult, TextContent};
   use crate::error::{McpError, ToolError};

   pub struct ReadTipsTool {
       vault_handler: VaultHandler,
   }

   impl ReadTipsTool {
       pub fn new(vault_handler: VaultHandler) -> Self {
           Self { vault_handler }
       }

       fn parse_arguments(&self, args: Value) -> Result<String, ToolError> {
           let file_name = args.get("file_name")
               .and_then(|v| v.as_str())
               .ok_or_else(|| ToolError::MissingParameter {
                   param: "file_name".to_string(),
               })?
               .to_string();

           // tips フォルダ内のパスに強制変換
           let tips_path = format!("tips/{}", file_name);
           Ok(tips_path)
       }

       fn create_success_result(&self, file_path: &str, content: &str) -> ToolResult {
           let lines = content.lines().count();
           let chars = content.chars().count();
           let message = format!("tips ファイルを読み込みました: {} ({} 行, {} 文字)", file_path, lines, chars);
           info!("{}", message);

           ToolResult {
               content: vec![TextContent {
                   text: json!({
                       "success": true,
                       "file_path": file_path,
                       "content": content,
                       "stats": {
                           "lines": lines,
                           "characters": chars,
                           "bytes": content.len()
                       },
                       "message": message
                   }).to_string(),
                   content_type: "text".to_string(),
               }],
               is_error: false,
           }
       }

       fn create_error_result(&self, error: &McpError) -> ToolResult {
           let error_message = format!("tips ファイル読み込みエラー: {}", error);
           warn!("{}", error_message);

           ToolResult {
               content: vec![TextContent {
                   text: json!({
                       "success": false,
                       "error": error_message
                   }).to_string(),
                   content_type: "text".to_string(),
               }],
               is_error: true,
           }
       }
   }

   #[async_trait]
   impl Tool for ReadTipsTool {
       fn name(&self) -> &str {
           "read_tips"
       }

       fn description(&self) -> &str {
           "vault/tips フォルダから Markdown ファイルを読み込みます。GitHub Copilot でのコンテキスト利用や内容の参照に使用できます。"
       }

       fn input_schema(&self) -> Value {
           json!({
               "type": "object",
               "properties": {
                   "file_name": {
                       "type": "string",
                       "description": "読み込むファイル名（例: sample.md）。tips/ は自動的に付与されます。"
                   }
               },
               "required": ["file_name"]
           })
       }

       async fn execute(&self, args: Value) -> Result<ToolResult, McpError> {
           debug!("read_tips ツール実行開始: {:?}", args);

           let tips_path = self.parse_arguments(args)?;

           match self.vault_handler.read_file(&tips_path).await {
               Ok(content) => Ok(self.create_success_result(&tips_path, &content)),
               Err(vault_error) => Ok(self.create_error_result(&McpError::Vault(vault_error))),
           }
       }
   }
   ```

### ステップ 2: tips フォルダ限定のファイル一覧取得 Tool の実装

1. src/tools/list_tips.rs を作成

   ```rust
   use async_trait::async_trait;
   use serde_json::{json, Value};
   use log::{info, debug, warn};
   use tokio::fs;
   use std::path::Path;

   use crate::tools::Tool;
   use crate::vault::VaultHandler;
   use crate::mcp::types::{ToolResult, TextContent};
   use crate::error::{McpError, ToolError};

   pub struct ListTipsTool {
       vault_handler: VaultHandler,
   }

   impl ListTipsTool {
       pub fn new(vault_handler: VaultHandler) -> Self {
           Self { vault_handler }
       }

       async fn scan_tips_directory(&self) -> Result<Vec<String>, McpError> {
           let tips_path = "tips";
           let full_tips_path = self.vault_handler.validate_path(tips_path)?;

           if !full_tips_path.exists() {
               // tips ディレクトリが存在しない場合は空のリストを返す
               return Ok(Vec::new());
           }

           let mut files = Vec::new();
           let mut entries = fs::read_dir(&full_tips_path).await
               .map_err(|e| McpError::Io(e))?;

           while let Some(entry) = entries.next_entry().await.map_err(|e| McpError::Io(e))? {
               let entry_path = entry.path();

               if entry_path.is_file() {
                   let extension = entry_path.extension()
                       .and_then(|ext| ext.to_str())
                       .unwrap_or("");

                   // .md ファイルのみを対象とする
                   if extension == "md" {
                       if let Some(file_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                           files.push(file_name.to_string());
                       }
                   }
               }
           }

           // ファイル名でソート
           files.sort();
           Ok(files)
       }

       fn create_success_result(&self, files: Vec<String>) -> ToolResult {
           let total_files = files.len();
           let message = format!("tips フォルダ内の Markdown ファイルを一覧取得しました ({} 件)", total_files);
           info!("{}", message);

           ToolResult {
               content: vec![TextContent {
                   text: json!({
                       "success": true,
                       "files": files,
                       "total_count": total_files,
                       "directory": "tips",
                       "message": message
                   }).to_string(),
                   content_type: "text".to_string(),
               }],
               is_error: false,
           }
       }

       fn create_error_result(&self, error: &McpError) -> ToolResult {
           let error_message = format!("tips ファイル一覧取得エラー: {}", error);
           warn!("{}", error_message);

           ToolResult {
               content: vec![TextContent {
                   text: json!({
                       "success": false,
                       "error": error_message
                   }).to_string(),
                   content_type: "text".to_string(),
               }],
               is_error: true,
           }
       }
   }

   #[async_trait]
   impl Tool for ListTipsTool {
       fn name(&self) -> &str {
           "list_tips"
       }

       fn description(&self) -> &str {
           "vault/tips フォルダ内の Markdown ファイル一覧を取得します。新しい tips 作成時の参考や重複確認に使用できます。"
       }

       fn input_schema(&self) -> Value {
           json!({
               "type": "object",
               "properties": {},
               "required": []
           })
       }

       async fn execute(&self, args: Value) -> Result<ToolResult, McpError> {
           debug!("list_tips ツール実行開始: {:?}", args);

           match self.scan_tips_directory().await {
               Ok(files) => Ok(self.create_success_result(files)),
               Err(error) => Ok(self.create_error_result(&error)),
           }
       }
   }
   ```

### ステップ 3: tips フォルダ限定の保存 Tool の修正

1. src/tools/markdown_save.rs を tips フォルダ限定に修正（001-03 で実装済みの場合）

   ```rust
   // MarkdownSaveTool の execute メソッドに tips フォルダ限定のロジックを追加
   async fn execute(&self, args: Value) -> Result<ToolResult, McpError> {
       debug!("save_markdown ツール実行開始: {:?}", args);

       let (original_file_path, content, overwrite) = self.parse_arguments(args)?;

       // tips フォルダに強制的にリダイレクト
       let tips_file_path = if original_file_path.starts_with("tips/") {
           original_file_path
       } else {
           format!("tips/{}", original_file_path)
       };

       info!("ファイルパスを tips フォルダに制限: {} -> {}", original_file_path, tips_file_path);

       match self.vault_handler.save_file(&tips_file_path, &content, overwrite).await {
           Ok(full_path) => {
               Ok(self.create_success_result(&tips_file_path, &full_path))
           }
           Err(vault_error) if matches!(vault_error, crate::error::VaultError::FileExists { .. }) && !overwrite => {
               // ファイルが既に存在する場合、一意な名前を生成して保存を試行
               info!("ファイルが既に存在します。一意な名前を生成します: {}", tips_file_path);

               match self.vault_handler.generate_unique_filename(&tips_file_path) {
                   Ok(unique_path) => {
                       match self.vault_handler.save_file(&unique_path, &content, false).await {
                           Ok(full_path) => {
                               let message = format!("ファイルを tips フォルダ内に別名で保存しました: {} -> {}", tips_file_path, unique_path);
                               info!("{}", message);

                               Ok(ToolResult {
                                   content: vec![TextContent {
                                       text: json!({
                                           "success": true,
                                           "file_path": unique_path,
                                           "original_path": tips_file_path,
                                           "full_path": full_path.to_string_lossy(),
                                           "message": message
                                       }).to_string(),
                                       content_type: "text".to_string(),
                                   }],
                                   is_error: false,
                               })
                           }
                           Err(e) => Ok(self.create_error_result(&McpError::Vault(e))),
                       }
                   }
                   Err(e) => Ok(self.create_error_result(&McpError::Vault(e))),
               }
           }
           Err(vault_error) => {
               Ok(self.create_error_result(&McpError::Vault(vault_error)))
           }
       }
   }
   ```

### ステップ 4: Tool 登録の更新

1. src/tools/mod.rs を更新

   ```rust
   pub mod registry;
   pub mod tool_trait;
   pub mod markdown_save;
   pub mod read_tips;
   pub mod list_tips;

   pub use registry::ToolRegistry;
   pub use tool_trait::Tool;
   pub use markdown_save::MarkdownSaveTool;
   pub use read_tips::ReadTipsTool;
   pub use list_tips::ListTipsTool;
   ```

2. src/mcp/server.rs の Tool 登録更新

   ```rust
   use crate::tools::{MarkdownSaveTool, ReadTipsTool, ListTipsTool};

   impl MCPServer {
       fn register_default_tools(&mut self, vault_handler: VaultHandler) {
           // MarkdownSaveTool を登録（tips フォルダ限定版）
           let markdown_save_tool = Arc::new(MarkdownSaveTool::new(vault_handler.clone()));
           self.register_tool(markdown_save_tool);

           // ReadTipsTool を登録（tips フォルダ限定）
           let read_tips_tool = Arc::new(ReadTipsTool::new(vault_handler.clone()));
           self.register_tool(read_tips_tool);

           // ListTipsTool を登録（tips フォルダ限定）
           let list_tips_tool = Arc::new(ListTipsTool::new(vault_handler.clone()));
           self.register_tool(list_tips_tool);

           info!("tips フォルダ限定の全ツールを登録しました (3ツール)");
       }
   }
   ```

### ステップ 5: ビルドテストと統合確認

1. プロジェクトのビルド確認

   ```bash
   cargo build
   ```

2. tips フォルダ限定 Tool の動作テスト

   ```bash
   # アプリケーション起動
   RUST_LOG=debug cargo run

   # ツールリスト確認:
   {"jsonrpc":"2.0","id":1,"method":"tools/list"}

   # tips ファイル一覧取得テスト:
   {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_tips","arguments":{}}}

   # tips ファイル読み込みテスト:
   {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"read_tips","arguments":{"file_name":"sample.md"}}}

   # tips ファイル保存テスト:
   {"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"save_markdown","arguments":{"file_path":"new_tip.md","content":"# 新しい Tips\n\nこれは新しい tips ファイルです。"}}}

   # コンテキスト活用ワークフローテスト:
   # 1. 既存 tips ファイル読み込み
   {"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"read_tips","arguments":{"file_name":"existing_tip.md"}}}

   # 2. 読み込み内容を参考にした新 tips 保存
   {"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"save_markdown","arguments":{"file_path":"based_on_existing.md","content":"# 既存 Tips を参考にした新しい Tips\n\n元ファイルの内容を参考に生成されました。"}}}
   ```

3. セキュリティテスト（tips フォルダ外アクセス防止）

   ```bash
   # tips フォルダ外への保存試行（自動的に tips フォルダに制限される）:
   {"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"save_markdown","arguments":{"file_path":"../outside.md","content":"# 外部ファイル"}}}

   # tips フォルダ外のファイル読み込み試行（エラーになるべき）:
   {"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"read_tips","arguments":{"file_name":"../outside.md"}}}
   ```

### ステップ 6: GitHub Copilot 連携の使用例ドキュメント作成

1. README.md に tips フォルダ限定の Copilot 連携セクションを追加

   ````markdown
   ## GitHub Copilot との連携（tips フォルダ限定）

   ### 基本的な使用方法

   1. **既存 tips ファイルの参照**

      ```text
      @obsidian-vault read_tips "existing_tip.md"
      ```
   ````

   1. **tips ファイル一覧の確認**

      ```text
      @obsidian-vault list_tips
      ```

   1. **新しい tips ファイルの保存**

      ```text
      @obsidian-vault save_markdown "new_tip.md" "# 新しい Tips\n\n内容..."
      ```

   ### 効率的な tips 作成ワークフロー

   1. **既存 tips を参考にした新 tips 作成**

      - 既存の tips ファイルを `read_tips` で読み込み
      - Copilot に内容の加工・拡張を依頼
      - `save_markdown` で新しい tips として保存

   1. **tips の整理と構造把握**

      - `list_tips` で tips フォルダの構造を確認
      - 関連する tips を `read_tips` で読み込み
      - 一貫性のある新しい tips を作成

   1. **安全な tips 管理**
      - すべての操作は tips フォルダ内に制限
      - 既存 tips の意図しない上書きを防止
      - 重複時は自動的に別名で保存

   ```markdown

   ```

## 品質保証

### 各フェーズでの確認ポイント

1. ReadTipsTool 実装後

   - [ ] tips フォルダ内のファイル読み込みが正常に動作すること
   - [ ] tips フォルダ外のファイルアクセスが適切にブロックされること
   - [ ] 存在しないファイルでの適切なエラーハンドリング

2. ListTipsTool 実装後

   - [ ] tips フォルダ内の .md ファイルのみが一覧表示されること
   - [ ] ファイル名のソートが正常に動作すること
   - [ ] tips フォルダが存在しない場合の適切な処理

3. tips 限定保存機能修正後

   - [ ] 任意のパス指定が tips フォルダにリダイレクトされること
   - [ ] tips フォルダ外への保存が完全に防止されること

4. 統合テスト後
   - [ ] 全 Tool が tips フォルダ限定で正常動作すること
   - [ ] tips 作成ワークフローが効率的に機能すること
   - [ ] セキュリティ制約が確実に維持されること

### requirements.md との整合性確認

- [x] 要件 5-1: 制限されたコンテキストファイル指定・読み込み・加工機能（tips フォルダ内のみ）
- [x] 要件 5-2: 元ファイル保護での新ファイル保存機能（tips フォルダ内）
- [x] 要件 5-3: Markdown 形式での適切なフォーマット保存
- [x] 要件 4-1: 指定フォルダ（tips）への Markdown ファイル保存
- [x] セキュリティ要件: 設定されたディレクトリ外（tips フォルダ外）へのアクセス制限

### design.md との整合性確認

- [x] Tool アーキテクチャの拡張性に従った実装（制限付き）
- [x] VaultHandler のセキュリティ設計に従った tips フォルダ制限
- [x] UI/UX 設計に従った tips 限定 Copilot 連携パターン
- [x] セキュリティ設計の強化（より厳格な制限）

## 次のフェーズへの引き継ぎ事項

1. 実装済みの機能

   - tips フォルダ限定のファイル読み込み機能（ReadTipsTool）
   - tips フォルダ限定のファイル一覧取得機能（ListTipsTool）
   - tips フォルダ限定の保存機能
   - セキュリティ強化されたワークフロー基盤

2. 次フェーズでの実装対象

   - VSCode 統合テスト（tips フォルダ限定版）
   - パフォーマンス最適化
   - tips フォルダの自動作成機能
   - 最終的な品質保証とドキュメント整備

3. 継続的な品質保証事項
   - tips フォルダ制限の確実な維持
   - Copilot での tips 作成ワークフローの使いやすさ
   - セキュリティ機能の継続的な検証
   - ユーザーの要求に応じた制限付き機能の提供
