# Obsidian MCP Server プロジェクト仕様書

作成日時: 2025-07-31 12:00
更新日時: 2025-07-31 12:00

## プロジェクト概要

Obsidian Vault を操作するための Model Context Protocol (MCP) サーバーの実装。
Rust で記述され、JSON-RPC 2.0 プロトコルを使用してクライアントとの通信を行う。

## プロジェクト構造

```text
src/
├── main.rs              # エントリーポイント
├── config.rs            # 設定管理
├── error.rs             # エラーハンドリング
└── mcp/
    ├── mod.rs           # MCPモジュール定義
    ├── protocol.rs      # MCP プロトコル定義
    └── server.rs        # MCP サーバー実装
```

## データ構造

### Config (src/config.rs)

アプリケーション設定を管理する構造体。

```rust
pub struct Config {
    pub vault_path: Option<PathBuf>,
}
```

主要メソッド:

- `default()` - デフォルト設定を生成（vault_path: `./vault`）
- `get_vault_path()` - vault_path を取得（None の場合はエラー）
- `load_or_default()` - 設定ファイルから読み込み、存在しない場合はデフォルト値を使用
- `save_to_file()` - 設定をファイルに保存
- `load_from_file()` - 設定ファイルから読み込み

設定ファイルパス:

- Windows: `%APPDATA%\obsidian-mcp-server\config.toml`
- その他: `~/.config/obsidian-mcp-server/config.toml`

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

### MCP Server (src/mcp/server.rs)

MCP サーバーの実装。

```rust
pub struct McpServer {
    config: Config,
    initialized: bool,
}
```

主要メソッド:

- `new(config: Config)` - 新しいサーバーインスタンスを作成
- `run_sync()` - 同期版サーバー実行（テスト用）
- `run_async()` - 非同期版サーバー実行
- `handle_request()` - リクエスト処理

対応プロトコル:

- `initialize` - サーバー初期化
- `initialized` - 初期化完了通知
- `tools/list` - 利用可能ツール一覧

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
- デフォルト設定では`./vault`を vault_path として設定

## 依存関係

主要な依存関係:

- `anyhow` - エラーハンドリング
- `serde` / `serde_json` - シリアライゼーション
- `tokio` - 非同期ランタイム
- `toml` - 設定ファイル解析
- `dirs` - システムディレクトリ取得
- `clap` - コマンドライン解析

## 今後の拡張予定

1. Obsidian Vault 操作機能の実装
2. ファイル読み書き機能
3. テンプレート機能
4. 検索機能
