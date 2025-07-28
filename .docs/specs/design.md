# 設計書

## 概要

### システムの技術的全体像

このシステムは、Rust で実装された Model Context Protocol（MCP）サーバーです。GitHub Copilot と VSCode の間で標準化されたプロトコルを使用して通信し、Obsidian Vault へのファイル操作を安全に提供します。

### 既存システムとの関係性

- VSCode の GitHub Copilot Chat 拡張機能のクライアント
- MCP プロトコル v1.0 に準拠したサーバー実装
- Obsidian アプリケーションとは独立した外部ツール
- ~/.cargo/bin にインストールされるスタンドアロンアプリケーション

### 実装予定のコンポーネント概要

- MCP サーバーコア（プロトコル処理）
- 設定管理モジュール（TOML ファイル読み込み）
- Tool レジストリ（拡張可能な Tool 管理）
- ファイル操作ハンドラー（Vault 内ファイル操作）
- ログ管理システム

## アーキテクチャ

### システム全体の構造設計

```text
┌─────────────────┐    MCP Protocol    ┌─────────────────┐
│   VSCode +      │◀─────────────────▶│ obsidian-vault- │
│ GitHub Copilot  │     JSON-RPC       │     mcp         │
└─────────────────┘                    └─────────────────┘
                                                │
                                                ▼
                                        ┌─────────────────┐
                                        │ Obsidian Vault  │
                                        │   (File System) │
                                        └─────────────────┘
```

### メインコンポーネント構成

1. **MCPServer**: プロトコル処理の中心
2. **ConfigManager**: 設定ファイル管理
3. **ToolRegistry**: Tool の登録・管理
4. **VaultHandler**: Vault 操作の実装
5. **MarkdownTool**: Markdown ファイル保存 Tool

### 新規追加コンポーネントの配置

- `src/main.rs`: エントリーポイント、サーバー起動
- `src/lib.rs`: ライブラリモジュール定義
- `src/mcp/`: MCP プロトコル関連モジュール
- `src/config/`: 設定管理モジュール
- `src/tools/`: Tool 実装モジュール
- `src/vault/`: Vault 操作モジュール

## コンポーネントとインターフェース

### MCPServer 構造体

```rust
pub struct MCPServer {
    config: Config,
    tool_registry: ToolRegistry,
    vault_handler: VaultHandler,
}

impl MCPServer {
    pub fn new(config_path: PathBuf) -> Result<Self, McpError>;
    pub async fn start(&self) -> Result<(), McpError>;
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse;
}
```

### Config 構造体

```rust
#[derive(Deserialize, Clone)]
pub struct Config {
    pub vault_path: PathBuf,
    pub allowed_extensions: Vec<String>,
    pub max_file_size: usize,
}

impl Config {
    pub fn load_from_file(path: PathBuf) -> Result<Self, ConfigError>;
    pub fn validate(&self) -> Result<(), ConfigError>;
}
```

### ToolRegistry 構造体

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self;
    pub fn register_tool(&mut self, name: String, tool: Box<dyn Tool>);
    pub fn get_tool(&self, name: &str) -> Option<&dyn Tool>;
    pub fn list_tools(&self) -> Vec<ToolInfo>;
}
```

### Tool トレイト

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError>;
}
```

### MarkdownSaveTool 構造体

```rust
pub struct MarkdownSaveTool {
    vault_handler: VaultHandler,
}

impl MarkdownSaveTool {
    pub fn new(vault_handler: VaultHandler) -> Self;
}

// Tool トレイトの実装
impl Tool for MarkdownSaveTool {
    // save_markdown メソッドで以下を実装：
    // - ファイル名とディレクトリパスの検証
    // - Vault ルート配下への保存制限
    // - ディレクトリの自動作成
    // - ファイル重複時の処理
}
```

### VaultHandler 構造体

```rust
pub struct VaultHandler {
    vault_root: PathBuf,
    config: Config,
}

impl VaultHandler {
    pub fn new(vault_root: PathBuf, config: Config) -> Self;
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf, VaultError>;
    pub async fn save_file(&self, path: &Path, content: &str) -> Result<(), VaultError>;
    pub async fn read_file(&self, path: &Path) -> Result<String, VaultError>;
    pub fn create_directories(&self, path: &Path) -> Result<(), VaultError>;
}
```

## データ構造

### 設定ファイル構造（~/.config/obsidian-vault-mcp.toml）

```toml
# Obsidian Vault のルートパス
vault_path = "/Users/username/Documents/ObsidianVault"

# 許可するファイル拡張子
allowed_extensions = ["md", "txt"]

# 最大ファイルサイズ（バイト）
max_file_size = 1048576  # 1MB

# ログレベル設定
log_level = "info"
```

### MCP Tool 入力スキーマ（save_markdown）

```json
{
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
}
```

### MCP Tool 出力構造

```json
{
  "success": true,
  "file_path": "notes/new_note.md",
  "full_path": "/Users/username/Documents/ObsidianVault/notes/new_note.md",
  "message": "ファイルが正常に保存されました"
}
```

### エラーレスポンス構造

```json
{
  "success": false,
  "error_code": "INVALID_PATH",
  "message": "指定されたパスはVault外部です",
  "details": {
    "provided_path": "../../outside.md",
    "vault_root": "/Users/username/Documents/ObsidianVault"
  }
}
```

## UI/UX 設計

### GitHub Copilot Chat での利用方法

1. **Tool 呼び出しパターン**

   ```text
   @obsidian-vault save_markdown ファイル名: "daily/2025-07-28.md" 内容: "# 今日の学習内容\n..."
   ```

2. **コンテキスト活用パターン**

   ```text
   @obsidian-vault このファイルを参考に新しいノートを作成して: [ファイル添付]
   ```

### エラーメッセージの日本語表示

- 設定ファイル不在: "設定ファイル ~/.config/obsidian-vault-mcp.toml が見つかりません"
- パス検証エラー: "指定されたパス '{path}' は Vault 外部のため保存できません"
- 権限エラー: "ファイル '{path}' への書き込み権限がありません"
- ファイル重複: "ファイル '{path}' は既に存在します。上書きする場合は overwrite: true を指定してください"

### ログ出力設計

```text
[INFO] obsidian-vault-mcp starting...
[INFO] Configuration loaded from: ~/.config/obsidian-vault-mcp.toml
[INFO] Vault root: /Users/username/Documents/ObsidianVault
[INFO] Registered tools: save_markdown
[DEBUG] MCP request received: tools/call
[DEBUG] Tool executed: save_markdown -> success
[WARN] Invalid path attempted: ../../outside.md
[ERROR] Failed to save file: Permission denied
```

## 実装順序と統合方針

### フェーズ 1: 基盤実装

- Cargo.toml への依存関係追加（tokio, serde, toml, log など）
- 基本的なプロジェクト構造の作成
- Config モジュールの実装

### フェーズ 2: MCP コア実装

- MCP プロトコルハンドラーの実装
- JSON-RPC リクエスト/レスポンス処理
- Tool レジストリの実装

### フェーズ 3: Tool 実装

- VaultHandler の実装
- MarkdownSaveTool の実装
- エラーハンドリングの実装

### フェーズ 4: 統合・テスト

- 全コンポーネントの統合
- VSCode との連携テスト
- エラーケースのテスト

## セキュリティ設計

### パス検証機能

- canonicalize() を使用した絶対パス解決
- Vault ルート外へのアクセス防止
- シンボリックリンクによる迂回の防止

### ファイルサイズ制限

- 設定可能な最大ファイルサイズ
- メモリ効率的なファイル処理

### 権限管理

- ファイル書き込み権限の事前チェック
- ディレクトリ作成権限の確認
