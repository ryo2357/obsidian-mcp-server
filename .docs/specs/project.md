# Obsidian MCP Server プロジェクト仕様書

作成日時: 2025-07-31 12:00
更新日時: 2025-08-14 12:00

## プロジェクト概要

Obsidian Vault を操作するための Model Context Protocol (MCP) サーバーの実装。
Rust で記述され、JSON-RPC 2.0 プロトコルを使用してクライアントとの通信を行う。

## コマンドライン引数

```rust
#[derive(Parser)]
struct Cli {
    /// 設定ファイルのパス
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Obsidian vault のパス
    #[arg(short, long)]
    vault_path: Option<PathBuf>,

    /// 同期モードで実行（テスト用）
    #[arg(long)]
    sync: bool,

    /// デバッグモードで実行
    #[arg(long)]
    debug: bool,
}
```

利用可能なオプション:

- `-c, --config <CONFIG>`: 設定ファイルのパス
- `-v, --vault-path <VAULT_PATH>`: Obsidian vault のパス
- `--sync`: 同期モードで実行（テスト用）
- `--debug`: デバッグモードで実行（debug-vault/ を自動使用）

## プロジェクト構造

```text
src/
├── main.rs              # エントリーポイント
├── config.rs            # 設定管理
├── debug.rs             # デバッグ機能
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

test-scripts/            # テスト関連ファイル
├── test_mcp.bat         # 通常モードのテストスクリプト
├── test_mcp_root.bat    # ルートから移動したテストスクリプト
├── test_debug.bat       # デバッグモードのテストスクリプト
├── test_initialize.json # 初期化テスト用JSONファイル
├── test_list_tools.json # ツール一覧テスト用JSONファイル
└── test_save_markdown.json # Markdown保存テスト用JSONファイル

debug-vault/             # デバッグモード用vault（自動生成）
└── Tips/
    └── sample-note.md   # デバッグ用サンプルファイル
```

## データ構造

### Config (src/config.rs)

アプリケーション設定を管理する構造体。

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    vault_path: Option<PathBuf>,  // プライベートフィールド
}
```

主要メソッド:

- `default()` - デフォルト設定を生成（vault_path: `None`）
- `get_vault_path()` - vault_path を取得（None の場合はエラー）
- `set_vault_path<P: AsRef<Path>>(&mut self, path: P)` - vault_path を設定
- `load_or_default(config_path: Option<&Path>)` - 設定ファイルから読み込み、設定パスを指定可能
- `save_to_file()` - 設定をファイルに保存
- `load_from_file()` - 設定ファイルから読み込み

設定ファイルパス:

- Windows: `%APPDATA%\obsidian-mcp-server\config.toml`
- その他: `~/.config/obsidian-mcp-server/config.toml`
- デバッグモード時: `./config/config.toml`

設計変更:

- `vault_path`フィールドをプライベートに変更し、セッターメソッド経由でのアクセスを強制
- `load_or_default`メソッドに設定ファイルパスの指定機能を追加

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

### Debug Module (src/debug.rs)

デバッグ機能を提供するモジュール。

```rust
pub struct DebugConfig {
    pub vault_path: PathBuf,
    pub config_path: PathBuf,
}
```

主要メソッド:

- `new()` - デバッグ設定を作成（vault_path: `./debug-vault`, config_path: `./config/config.toml`）
- `ensure_debug_vault()` - デバッグ用 vault ディレクトリを作成
- `generate_dummy_data()` - デバッグ用のサンプルデータを生成

機能:

- `--debug` フラグによるデバッグモードの切り替え
- ハードコードされたデバッグ vault パス（`./debug-vault/`）
- 専用設定ファイルパス（`./config/config.toml`）の管理
- デバッグ用ディレクトリの自動作成
- サンプル Markdown ファイルの自動生成

設計変更:

- `enabled`フィールドを削除し、`config_path`フィールドを追加
- デバッグ用ログ出力機能を削除してシンプル化

## コーディング規約

### インポート規約

- アスタリスク（`*`）を使ったインポートは避け、具体的な型名を明示する
- 未使用のインポートは削除する

### 設定管理規約

- `vault_path`が None の場合は、適切なエラーメッセージでエラーを発生させる
- デフォルト設定では`vault_path`は`None`で、コマンドライン引数での指定が必須
- `vault_path`フィールドはプライベートとし、セッターメソッド経由でのアクセスを強制する
- 設定ファイルパスの指定は`load_or_default`メソッドのパラメータで行う

### ツール実装規約

- ツール固有の設定は定数として各ツールファイルに定義する
- セキュリティを重視し、vault 外へのアクセスを制限する
- ファイル名の検証を必ず実行する

### テストスクリプト管理規約

- 全てのテスト用 bat スクリプトは `test-scripts/` フォルダに保存する
- JSON テストファイルも同様に `test-scripts/` フォルダに保存する
- デバッグモード用とリリースモード用でスクリプトを分離する

## 依存関係

主要な依存関係:

- `anyhow` - エラーハンドリング
- `serde` / `serde_json` - シリアライゼーション
- `tokio` - 非同期ランタイム
- `toml` - 設定ファイル解析
- `dirs` - システムディレクトリ取得
- `clap` - コマンドライン解析
- `chrono` - 日時処理（デバッグモード用）

開発時依存関係:

- `tempfile` - テスト用一時ファイル作成

## 実装済み機能

### Markdown ファイル保存機能

- `save_markdown_file` MCP ツールとして実装済み
- Vault 内の`Tips`ディレクトリにファイルを保存
- セキュリティチェック、ファイル名検証、重複チェックを実装
- テスト済み、動作確認完了

### デバッグ機能

- `--debug` フラグによるデバッグモードの実装
- 固定パス（`./debug-vault/`）でのデバッグ環境自動構築
- サンプル Markdown ファイルの自動生成
- デバッグ用ログ出力機能
- テスト済み、動作確認完了

### コードリファクタリング

- ワイルドカードインポート（`use *`）の除去
- 明示的なインポートへの変更
- コードの可読性向上
- テスト実行済み、正常動作確認完了

### 設定管理リファクタリング

- `Config`構造体の`vault_path`フィールドをプライベート化
- `set_vault_path`メソッドの追加による設定アクセスの制御
- `load_or_default`メソッドのオプション設定パス機能
- `DebugConfig`の`config_path`フィールド追加
- デバッグモードと通常モードの統合処理
- コードの可読性向上とメンテナビリティの改善
- テスト実行済み、正常動作確認完了

## 今後の拡張予定

1. ~~Obsidian Vault 操作機能の実装~~ （完了）
2. テンプレートファイル読み込み機能
3. ファイル検索機能
4. メタデータ操作機能
