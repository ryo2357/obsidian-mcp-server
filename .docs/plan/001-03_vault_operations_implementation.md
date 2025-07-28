# Vault 操作とファイル保存機能実装プラン

## 基本情報

- 作成日: 2025-07-28
- 参照仕様書セクション: 要件 4（Markdown ファイル保存機能）、要件 5（GitHub Copilot 連携機能）
- 参照設計書セクション: VaultHandler 構造体、MarkdownSaveTool 構造体、セキュリティ設計
- プラン進捗: 3/5（Vault 操作・ファイル保存機能実装）
- 前提条件: 001-01（基盤）、001-02（MCP プロトコル）が実装済み

## 概要と目標

### 実装する機能の詳細説明

このフェーズでは、Obsidian Vault への安全なファイル操作機能を実装します。パス検証、ディレクトリ作成、ファイル保存といった核となるファイル操作機能と、MCP Tool として利用可能な Markdown ファイル保存機能を提供します。

### 達成すべき具体的な目標

1. VaultHandler による安全なファイル操作システムの実装
2. パス検証機能（Vault 外部アクセス防止）の実装
3. MarkdownSaveTool の Tool トレイト実装
4. ファイル重複処理とディレクトリ自動作成機能
5. セキュリティ機能（パス正規化、権限チェック）の実装

### requirements.md との対応関係

- 要件 4-1: 指定フォルダへの Markdown ファイル保存
- 要件 4-2: Vault ルートからの相対パス指定
- 要件 4-3: 保存先ディレクトリの自動作成
- 要件 4-4: ファイル重複時の上書き確認・新名前保存
- 要件 5-1: コンテキストファイル読み込み・加工
- 要件 5-2: 元ファイル保護での新ファイル保存
- 要件 5-3: Markdown 形式での適切なフォーマット保存

## 実装手順

### ステップ 1: Vault 操作エラー型の定義

1. src/error/mod.rs に VaultError 追加

   ```rust
   // 既存のエラー型に追加
   #[derive(Error, Debug)]
   pub enum McpError {
       #[error("設定エラー: {0}")]
       Config(#[from] ConfigError),
       #[error("IO エラー: {0}")]
       Io(#[from] std::io::Error),
       #[error("JSON パースエラー: {0}")]
       Json(#[from] serde_json::Error),
       #[error("Vault エラー: {0}")]
       Vault(#[from] VaultError),
       #[error("Tool エラー: {0}")]
       Tool(#[from] ToolError),
   }

   #[derive(Error, Debug)]
   pub enum VaultError {
       #[error("パスがVault外部です: {path}")]
       PathOutsideVault { path: String },
       #[error("ファイルが存在します: {path}")]
       FileExists { path: String },
       #[error("ディレクトリ作成に失敗しました: {path}")]
       DirectoryCreationFailed { path: String },
       #[error("ファイル保存に失敗しました: {path}")]
       FileSaveFailed { path: String },
       #[error("ファイル読み込みに失敗しました: {path}")]
       FileReadFailed { path: String },
       #[error("無効なファイル拡張子: {extension}")]
       InvalidExtension { extension: String },
       #[error("ファイルサイズが制限を超えています: {size} > {limit}")]
       FileSizeExceeded { size: usize, limit: usize },
   }

   #[derive(Error, Debug)]
   pub enum ToolError {
       #[error("必須パラメータが不足しています: {param}")]
       MissingParameter { param: String },
       #[error("パラメータの型が無効です: {param}")]
       InvalidParameterType { param: String },
       #[error("ツール実行エラー: {message}")]
       ExecutionError { message: String },
   }
   ```

### ステップ 2: VaultHandler の実装

1. src/vault/mod.rs に VaultHandler 作成

   ```rust
   pub mod handler;

   pub use handler::VaultHandler;
   ```

2. src/vault/handler.rs に VaultHandler 実装

   ```rust
   use std::path::{Path, PathBuf};
   use tokio::fs;
   use log::{debug, info, warn};

   use crate::config::Config;
   use crate::error::VaultError;

   #[derive(Clone)]
   pub struct VaultHandler {
       vault_root: PathBuf,
       config: Config,
   }

   impl VaultHandler {
       pub fn new(vault_root: PathBuf, config: Config) -> Self {
           Self {
               vault_root: vault_root.canonicalize().unwrap_or(vault_root),
               config,
           }
       }

       pub fn validate_path(&self, relative_path: &str) -> Result<PathBuf, VaultError> {
           // 相対パスを絶対パスに変換
           let full_path = self.vault_root.join(relative_path);

           // パスを正規化（シンボリックリンク解決等）
           let canonical_path = match full_path.canonicalize() {
               Ok(path) => path,
               Err(_) => {
                   // ファイルが存在しない場合は親ディレクトリまでの正規化を試行
                   let parent = full_path.parent().unwrap_or(&full_path);
                   let canonical_parent = parent.canonicalize()
                       .map_err(|_| VaultError::PathOutsideVault {
                           path: relative_path.to_string(),
                       })?;
                   canonical_parent.join(full_path.file_name().unwrap_or_default())
               }
           };

           // Vault外部アクセス防止チェック
           if !canonical_path.starts_with(&self.vault_root) {
               return Err(VaultError::PathOutsideVault {
                   path: relative_path.to_string(),
               });
           }

           debug!("パス検証成功: {} -> {}", relative_path, canonical_path.display());
           Ok(canonical_path)
       }

       pub fn validate_extension(&self, path: &Path) -> Result<(), VaultError> {
           let extension = path.extension()
               .and_then(|ext| ext.to_str())
               .unwrap_or("");

           if !self.config.allowed_extensions.contains(&extension.to_string()) {
               return Err(VaultError::InvalidExtension {
                   extension: extension.to_string(),
               });
           }

           Ok(())
       }

       pub fn validate_content_size(&self, content: &str) -> Result<(), VaultError> {
           let size = content.len();
           if size > self.config.max_file_size {
               return Err(VaultError::FileSizeExceeded {
                   size,
                   limit: self.config.max_file_size,
               });
           }
           Ok(())
       }

       pub async fn create_directories(&self, file_path: &Path) -> Result<(), VaultError> {
           if let Some(parent) = file_path.parent() {
               if !parent.exists() {
                   info!("ディレクトリを作成します: {}", parent.display());
                   fs::create_dir_all(parent).await
                       .map_err(|_| VaultError::DirectoryCreationFailed {
                           path: parent.to_string_lossy().to_string(),
                       })?;
               }
           }
           Ok(())
       }

       pub async fn save_file(&self, relative_path: &str, content: &str, overwrite: bool) -> Result<PathBuf, VaultError> {
           // パス検証
           let full_path = self.validate_path(relative_path)?;

           // 拡張子検証
           self.validate_extension(&full_path)?;

           // ファイルサイズ検証
           self.validate_content_size(content)?;

           // ファイル重複チェック
           if full_path.exists() && !overwrite {
               return Err(VaultError::FileExists {
                   path: relative_path.to_string(),
               });
           }

           // ディレクトリ作成
           self.create_directories(&full_path).await?;

           // ファイル保存
           fs::write(&full_path, content).await
               .map_err(|e| {
                   warn!("ファイル保存失敗 {}: {}", full_path.display(), e);
                   VaultError::FileSaveFailed {
                       path: relative_path.to_string(),
                   }
               })?;

           info!("ファイル保存成功: {}", full_path.display());
           Ok(full_path)
       }

       pub async fn read_file(&self, relative_path: &str) -> Result<String, VaultError> {
           let full_path = self.validate_path(relative_path)?;

           if !full_path.exists() {
               return Err(VaultError::FileReadFailed {
                   path: relative_path.to_string(),
               });
           }

           let content = fs::read_to_string(&full_path).await
               .map_err(|e| {
                   warn!("ファイル読み込み失敗 {}: {}", full_path.display(), e);
                   VaultError::FileReadFailed {
                       path: relative_path.to_string(),
                   }
               })?;

           debug!("ファイル読み込み成功: {} ({} bytes)", full_path.display(), content.len());
           Ok(content)
       }

       pub fn generate_unique_filename(&self, relative_path: &str) -> Result<String, VaultError> {
           let original_path = self.validate_path(relative_path)?;

           if !original_path.exists() {
               return Ok(relative_path.to_string());
           }

           let file_stem = original_path.file_stem()
               .and_then(|s| s.to_str())
               .unwrap_or("untitled");
           let extension = original_path.extension()
               .and_then(|s| s.to_str())
               .unwrap_or("");
           let parent = original_path.parent().unwrap_or(&self.vault_root);

           for i in 1..=999 {
               let new_filename = if extension.is_empty() {
                   format!("{}-{}", file_stem, i)
               } else {
                   format!("{}-{}.{}", file_stem, i, extension)
               };

               let new_path = parent.join(&new_filename);
               if !new_path.exists() {
                   // Vault ルートからの相対パスを計算
                   let relative = new_path.strip_prefix(&self.vault_root)
                       .map_err(|_| VaultError::PathOutsideVault {
                           path: new_filename.clone(),
                       })?;
                   return Ok(relative.to_string_lossy().to_string());
               }
           }

           Err(VaultError::FileSaveFailed {
               path: "一意なファイル名を生成できませんでした".to_string(),
           })
       }
   }
   ```

### ステップ 3: MarkdownSaveTool の実装

1. src/tools/mod.rs を更新

   ```rust
   pub mod registry;
   pub mod tool_trait;
   pub mod markdown_save;

   pub use registry::ToolRegistry;
   pub use tool_trait::Tool;
   pub use markdown_save::MarkdownSaveTool;
   ```

2. src/tools/markdown_save.rs を作成

   ```rust
   use async_trait::async_trait;
   use serde_json::{json, Value};
   use log::{info, debug, warn};

   use crate::tools::Tool;
   use crate::vault::VaultHandler;
   use crate::mcp::types::{ToolResult, TextContent};
   use crate::error::{McpError, ToolError};

   pub struct MarkdownSaveTool {
       vault_handler: VaultHandler,
   }

   impl MarkdownSaveTool {
       pub fn new(vault_handler: VaultHandler) -> Self {
           Self { vault_handler }
       }

       fn parse_arguments(&self, args: Value) -> Result<(String, String, bool), ToolError> {
           let file_path = args.get("file_path")
               .and_then(|v| v.as_str())
               .ok_or_else(|| ToolError::MissingParameter {
                   param: "file_path".to_string(),
               })?
               .to_string();

           let content = args.get("content")
               .and_then(|v| v.as_str())
               .ok_or_else(|| ToolError::MissingParameter {
                   param: "content".to_string(),
               })?
               .to_string();

           let overwrite = args.get("overwrite")
               .and_then(|v| v.as_bool())
               .unwrap_or(false);

           Ok((file_path, content, overwrite))
       }

       fn create_success_result(&self, file_path: &str, full_path: &std::path::Path) -> ToolResult {
           let message = format!("ファイルが正常に保存されました: {}", file_path);
           info!("{}", message);

           ToolResult {
               content: vec![TextContent {
                   text: json!({
                       "success": true,
                       "file_path": file_path,
                       "full_path": full_path.to_string_lossy(),
                       "message": message
                   }).to_string(),
                   content_type: "text".to_string(),
               }],
               is_error: false,
           }
       }

       fn create_error_result(&self, error: &McpError) -> ToolResult {
           let error_message = format!("ファイル保存エラー: {}", error);
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
   impl Tool for MarkdownSaveTool {
       fn name(&self) -> &str {
           "save_markdown"
       }

       fn description(&self) -> &str {
           "Obsidian Vault に Markdown ファイルを保存します。相対パスで保存先を指定でき、必要に応じてディレクトリを自動作成します。"
       }

       fn input_schema(&self) -> Value {
           json!({
               "type": "object",
               "properties": {
                   "file_path": {
                       "type": "string",
                       "description": "Vault内の相対パス（例: notes/new_note.md）"
                   },
                   "content": {
                       "type": "string",
                       "description": "保存するMarkdownファイルの内容"
                   },
                   "overwrite": {
                       "type": "boolean",
                       "description": "既存ファイルの上書きを許可するか",
                       "default": false
                   }
               },
               "required": ["file_path", "content"]
           })
       }

       async fn execute(&self, args: Value) -> Result<ToolResult, McpError> {
           debug!("save_markdown ツール実行開始: {:?}", args);

           let (file_path, content, overwrite) = self.parse_arguments(args)?;

           match self.vault_handler.save_file(&file_path, &content, overwrite).await {
               Ok(full_path) => {
                   Ok(self.create_success_result(&file_path, &full_path))
               }
               Err(vault_error) if matches!(vault_error, crate::error::VaultError::FileExists { .. }) && !overwrite => {
                   // ファイルが既に存在する場合、一意な名前を生成して保存を試行
                   info!("ファイルが既に存在します。一意な名前を生成します: {}", file_path);

                   match self.vault_handler.generate_unique_filename(&file_path) {
                       Ok(unique_path) => {
                           match self.vault_handler.save_file(&unique_path, &content, false).await {
                               Ok(full_path) => {
                                   let message = format!("ファイルを別名で保存しました: {} -> {}", file_path, unique_path);
                                   info!("{}", message);

                                   Ok(ToolResult {
                                       content: vec![TextContent {
                                           text: json!({
                                               "success": true,
                                               "file_path": unique_path,
                                               "original_path": file_path,
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
   }
   ```

### ステップ 4: MCPServer に Tool 登録機能を追加

1. src/mcp/server.rs の更新（Tool 登録部分）

   ```rust
   use crate::tools::{MarkdownSaveTool};
   use crate::vault::VaultHandler;

   impl MCPServer {
       pub fn new(config: Config) -> Self {
           let mut server = Self {
               config: config.clone(),
               tool_registry: Arc::new(ToolRegistry::new()),
           };

           // VaultHandler を作成
           let vault_handler = VaultHandler::new(config.vault_path.clone(), config.clone());

           // Tool を登録
           server.register_default_tools(vault_handler);

           server
       }

       fn register_default_tools(&mut self, vault_handler: VaultHandler) {
           // MarkdownSaveTool を登録
           let markdown_tool = Arc::new(MarkdownSaveTool::new(vault_handler));
           self.register_tool(markdown_tool);

           info!("デフォルトツールを登録しました");
       }
   }
   ```

### ステップ 5: ビルドテストと動作確認

1. プロジェクトのビルド確認

   ```bash
   cargo build
   ```

2. MarkdownSaveTool の基本動作テスト

   ```bash
   # アプリケーション起動
   RUST_LOG=debug cargo run

   # 別ターミナルで Tool 動作テスト
   # ツールリスト確認:
   {"jsonrpc":"2.0","id":1,"method":"tools/list"}

   # Markdown保存テスト:
   {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"save_markdown","arguments":{"file_path":"test/sample.md","content":"# テストファイル\n\nこれはテスト用のMarkdownファイルです。"}}}

   # 上書き禁止テスト（同じファイル名で再実行）:
   {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"save_markdown","arguments":{"file_path":"test/sample.md","content":"# 重複テストファイル"}}}

   # 上書き許可テスト:
   {"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"save_markdown","arguments":{"file_path":"test/sample.md","content":"# 上書きテストファイル","overwrite":true}}}
   ```

3. パス検証とセキュリティテスト

   ```bash
   # Vault外部アクセステスト:
   {"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"save_markdown","arguments":{"file_path":"../../outside.md","content":"# 外部ファイル"}}}

   # 無効な拡張子テスト:
   {"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"save_markdown","arguments":{"file_path":"test/sample.exe","content":"invalid content"}}}
   ```

## 品質保証

### 各フェーズでの確認ポイント

1. VaultError 実装後

   - [ ] エラー型が適切に定義されていること
   - [ ] エラーメッセージが日本語で表示されること

2. VaultHandler 実装後

   - [ ] パス検証が正常に動作すること
   - [ ] Vault 外部アクセスが適切にブロックされること
   - [ ] ファイル保存・読み込みが正常動作すること
   - [ ] ディレクトリ自動作成が動作すること

3. MarkdownSaveTool 実装後

   - [ ] Tool トレイトが適切に実装されていること
   - [ ] JSON スキーマが正しく定義されていること
   - [ ] ファイル重複時の処理が適切に動作すること

4. 統合テスト後
   - [ ] MCP 経由でのファイル保存が正常動作すること
   - [ ] エラーケースが適切にハンドリングされること
   - [ ] セキュリティ機能が正常に動作すること

### requirements.md との整合性確認

- [x] 要件 4-1: 指定フォルダへの Markdown ファイル保存
- [x] 要件 4-2: Vault ルートからの相対パス指定
- [x] 要件 4-3: 保存先ディレクトリの自動作成
- [x] 要件 4-4: ファイル重複時の上書き確認・新名前保存
- [x] 要件 5-2: 元ファイル保護での新ファイル保存
- [x] 要件 5-3: Markdown 形式での適切なフォーマット保存

### design.md との整合性確認

- [x] VaultHandler の設計パターンに従った実装
- [x] MarkdownSaveTool の設計パターンに従った実装
- [x] セキュリティ設計に従ったパス検証実装
- [x] Tool アーキテクチャの設計パターンへの準拠

## 次のフェーズへの引き継ぎ事項

1. 実装済みの機能

   - Vault 操作基盤（VaultHandler）
   - Markdown ファイル保存機能（MarkdownSaveTool）
   - セキュリティ機能（パス検証、権限チェック）

2. 次フェーズでの実装対象

   - ファイル読み込み機能の Tool 化
   - GitHub Copilot 連携の高度化
   - VSCode 統合テスト

3. 継続的な品質保証事項
   - セキュリティ機能の維持
   - Tool アーキテクチャの拡張性保持
   - エラーハンドリングの一貫性
