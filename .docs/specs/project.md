# Obsidian MCP Server プロジェクト仕様書

作成日時: 2025-07-31 12:00
更新日時: 2025-07-31 16:00

## プロジェクト概要

Obsidian Vault を操作するための Model Context Protocol (MCP) サーバーの実装。
Rust で記述され、JSON-RPC 2.0 プロトコルを使用してクライアントとの通信を行う。

## プロジェクト構造

```text
src/
├── main.rs              # エントリーポイント
├── config.rs            # 設定管理
├── error.rs             # エラーハンドリング
├── vault/               # Vault操作モジュール
│   ├── mod.rs
│   └── operations.rs    # vault操作の共通処理
└── mcp/
    ├── mod.rs           # MCPモジュール定義
    ├── protocol.rs      # MCP プロトコル定義
    ├── server.rs        # MCP サーバー実装
    └── tools/           # MCPツール実装
        ├── mod.rs
        └── save_markdown.rs # Markdownファイル保存ツール
```

## データ構造

### Config (src/config.rs)

アプリケーション設定を管理する構造体。

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub vault_path: Option<PathBuf>,
}
```

主要メソッド:

- `default()` - デフォルト設定を生成（vault_path: `None`）
- `get_vault_path()` - vault_path を取得（None の場合はエラー）
- `load_or_default()` - 設定ファイルから読み込み、存在しない場合はデフォルト値を使用
- `save_to_file()` - 設定をファイルに保存
- `load_from_file()` - 設定ファイルから読み込み

設定ファイルパス:

- Windows: `%APPDATA%\obsidian-mcp-server\config.toml`
- その他: `~/.config/obsidian-mcp-server/config.toml`

注意: デフォルト設定では `vault_path` は `None` のため、コマンドライン引数での指定が必須です。

### MCP Protocol (src/mcp/protocol.rs)

JSON-RPC 2.0 および MCP プロトコルに関連する構造体を定義。

主要構造体:

- `JsonRpcRequest` - JSON-RPC 2.0 リクエスト
- `JsonRpcResponse` - JSON-RPC 2.0 レスポンス
- `InitializeParams` - 初期化パラメータ
- `InitializeResult` - 初期化結果
- `ServerInfo` - サーバー情報
- `ServerCapabilities` - サーバー機能
- `ListToolsResult` - ツール一覧結果
- `Tool` - ツール定義
- `CallToolParams` - ツール呼び出しパラメータ
- `CallToolResult` - ツール呼び出し結果
- `ToolContent` - ツールの出力コンテンツ

### MCP Server (src/mcp/server.rs)

MCP サーバーの実装。

```rust
pub struct McpServer {
    config: Config,
    initialized: bool,
    vault_ops: Option<VaultOperations>,
}
```

主要メソッド:

- `new(config: Config)` - 新しいサーバーインスタンスを作成
- `run_sync()` - 同期版サーバー実行（テスト用）
- `run_async()` - 非同期版サーバー実行
- `handle_request()` - リクエスト処理
- `handle_initialize()` - 初期化処理
- `handle_list_tools()` - ツール一覧処理
- `handle_call_tool()` - ツール呼び出し処理

対応プロトコル:

- `initialize` - サーバー初期化
- `tools/list` - 利用可能ツール一覧
- `tools/call` - ツール実行

### Vault Operations (src/vault/operations.rs)

Vault 操作の共通処理を提供するモジュール。

```rust
pub struct VaultOperations {
    vault_path: PathBuf,
    target_directory: String,
}
```

主要メソッド:

- `new(vault_path: PathBuf, target_directory: String)` - 新しいインスタンスを作成
- `save_markdown_file(filename: &str, content: &str)` - Markdown ファイルを保存
- `validate_filename(filename: &str)` - ファイル名を検証
- `is_path_within_vault(file_path: &Path)` - パスが vault 内にあるかチェック
- `target_directory_exists()` - ターゲットディレクトリの存在確認

セキュリティ機能:

- パストラバーサル攻撃の防止
- ファイル名の検証（危険な文字の排除）
- vault 外へのアクセス制限

### MCP Tools (src/mcp/tools/)

MCP ツールの実装を格納するモジュール。

#### save_markdown_file ツール (src/mcp/tools/save_markdown.rs)

Obsidian vault 内に Markdown ファイルを保存するツール。

```rust
pub const TARGET_DIRECTORY: &str = "Tips";
```

機能:

- ファイル名: `.md`拡張子の自動付与
- 保存先: vault 内の`Tips`ディレクトリ（定数で固定）
- 入力パラメータ:
  - `filename`: ファイル名（拡張子なし）
  - `content`: Markdown コンテンツ
- エラーハンドリング:
  - 既存ファイルの重複チェック
  - ディレクトリ存在確認
  - ファイル名の検証

### Error Handling (src/error.rs)

エラーハンドリングと MCP エラーレスポンスの定義。

```rust
pub type AppResult<T> = Result<T, anyhow::Error>;

pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

## コーディング規約

### インポート規約

- アスタリスク（`*`）を使ったインポートは避け、具体的な型名を明示する
- 未使用のインポートは削除する

### 設定管理規約

- `vault_path`が None の場合は、適切なエラーメッセージでエラーを発生させる
- デフォルト設定では`vault_path`は`None`で、コマンドライン引数での指定が必須

### ツール実装規約

- ツール固有の設定は定数として各ツールファイルに定義する
- セキュリティを重視し、vault 外へのアクセスを制限する
- ファイル名の検証を必ず実行する

## 依存関係

主要な依存関係:

- `anyhow` - エラーハンドリング
- `serde` / `serde_json` - シリアライゼーション
- `tokio` - 非同期ランタイム
- `toml` - 設定ファイル解析
- `dirs` - システムディレクトリ取得
- `clap` - コマンドライン解析

開発時依存関係:

- `tempfile` - テスト用一時ファイル作成

## 実装済み機能

### Markdown ファイル保存機能

- `save_markdown_file` MCP ツールとして実装済み
- Vault 内の`Tips`ディレクトリにファイルを保存
- セキュリティチェック、ファイル名検証、重複チェックを実装
- テスト済み、動作確認完了

## 今後の拡張予定

1. ~~Obsidian Vault 操作機能の実装~~ （完了）
2. テンプレートファイル読み込み機能
3. ファイル検索機能
4. メタデータ操作機能
