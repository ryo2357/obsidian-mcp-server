# Obsidian MCP Server プロジェクト仕様書

作成日時: 2025-07-31 12:00
更新日時: 2025-08-14 15:25

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
    vault_dir: Option<PathBuf>,

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
- `-v, --vault-dir <VAULT_DIR>`: Obsidian vault のパス
- `--sync`: 同期モードで実行（テスト用）
- `--debug`: デバッグモードで実行（debug-vault/ を自動使用）

## プロジェクト構造

```text
src/
├── main.rs              # エントリーポイント
├── config.rs            # 設定管理
├── debug.rs             # デバッグ機能
├── logger.rs            # ログ初期化/設定
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
    vault_dir: Option<PathBuf>, // プライベートフィールド
}
```

主要メソッド:

- `get_vault_dir()` - vault_dir を取得（None の場合はエラー）
- `set_vault_dir<P: AsRef<Path>>(&mut self, path: P)` - vault_dir を設定
- `load_or_default(config_path: PathBuf)` - 設定ファイルから読み込み。存在しない場合はデフォルト生成し保存
- `save_to_file(path: &Path)` / `load_from_file(path: PathBuf)` - 永続化/読み込み

設計変更 (旧仕様との差分):

- フィールド名 `vault_path` → `vault_dir` にリネーム（整合性向上）
- `default_config_path()` を削除し、呼び出し元で明示パス決定
- `load_or_default` の引数を `Option<&Path>` から `PathBuf` に変更し責務を単純化

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

デバッグ実行時の補助機能を提供するモジュール。

```rust
pub struct DebugConfig {
    pub vault_dir: PathBuf,
    pub config_path: PathBuf,
    pub config_dir: PathBuf,
}
```

主要メソッド:

- `new()` - デフォルト値を設定
- `ensure_debug_vault()` - デバッグ用 vault とサブディレクトリ (`Tips`) を作成
- `generate_dummy_data()` - サンプル Markdown を生成

変更点:

- フィールド `vault_path` → `vault_dir` に追随リネーム
- ログ出力用 `config_dir` フィールドを追加（`.config/logs`）
- 標準出力へのデバッグログ依存を削減（ファイルロギングへ移行）

### Logging (src/logger.rs)

`flexi_logger` を用いたファイルベースのロギング初期化。

機能:

- ログレベル（`debug` / `info` 等）を文字列指定で初期化
- ログ出力先: 実行モードに応じた設定ディレクトリ配下 `logs/`
  - 通常モード: OS 毎の設定ルート（`dirs::config_dir()/obsidian-mcp-server/logs`）
  - デバッグモード: プロジェクトルート直下の `.config/logs/`
- ログファイル命名: `log-<timestamp>.log`
- ローテーション: サイズ基準 10MB (`Criterion::Size(10 * 1024 * 1024)`)
- 保持ポリシー: 最新 5 ファイル (`Cleanup::KeepLogFiles(5)`)
- フォーマット: `flexi_logger::detailed_format`

初期化フロー:

- `main.rs` 起動時にモード判定 → `logger::init_logger(level, path)` 呼び出し
- 以降 `log` クレートのマクロ（`debug!` 等）で記録

設計意図:

- 標準入出力は MCP プロトコル通信占有のため混在回避
- ファイルロギングでデバッグと運用の両立

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
- `chrono` - 日時処理（デバッグ用データ生成）
- `flexi_logger` - ファイルロギング & ローテーション
- `log` - ロギングファサード
- `once_cell` - グローバル遅延初期化 (`APP_DIR`)

開発時依存関係:

- `tempfile` - テスト用一時ファイル作成

## 実装済み機能

### ログ出力機能（新規）

- `flexi_logger` によるファイルベースログ
- デバッグ/通常モードで異なる出力先ディレクトリ
- サイズローテーション & 保持数制御
- 標準入出力混在を回避し通信チャネルを保全

### Markdown ファイル保存機能

- `save_markdown_file` MCP ツールとして実装済み
- Vault 内の`Tips`ディレクトリにファイルを保存
- セキュリティチェック、ファイル名検証、重複チェックを実装
- テスト済み、動作確認完了

### デバッグ機能

- `--debug` フラグによるデバッグモード
- 固定パス（`./debug-vault/`）でのデバッグ環境自動構築
- サンプル Markdown ファイルの自動生成
- ファイルロギング（標準出力依存から移行）

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
