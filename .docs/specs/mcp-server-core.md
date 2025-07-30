# MCP サーバーコア仕様書

作成日時: 2025-07-30 02:05  
更新日時: 2025-07-30 13:30  
参照計画: 001-01-project-setup.md

## 概要

Obsidian vault 操作用 MCP サーバーのコア機能仕様書です。JSON-RPC 2.0 プロトコルに基づく MCP サーバー実装、設定管理、エラーハンドリング、ツール登録フレームワークを提供します。

## アーキテクチャ

### プロジェクト構造

```text
src/
├── main.rs              # CLIエントリーポイント、早期ログ初期化
├── lib.rs               # ライブラリルート（AppError, AppResult公開）
├── error.rs             # カスタムエラー型定義
├── config/
│   ├── mod.rs           # 設定モジュール
│   ├── settings.rs      # TOML設定ファイル処理
│   └── logging.rs       # ログ機能独立モジュール
├── mcp/
│   ├── mod.rs           # MCPモジュール
│   ├── server.rs        # MCPサーバー実装（同期/非同期対応）
│   ├── protocol.rs      # JSON-RPCプロトコルハンドラ
│   └── types.rs         # MCP型定義（メッセージ、レスポンス等）
└── tools/
    ├── mod.rs           # ツールモジュール
    └── registry.rs      # ツール登録・実行管理
```

## コアコンポーネント

### 1. エラーハンドリング（error.rs）

#### AppError 型

```rust
pub enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Toml(toml::de::Error),
    Config(String),
    Protocol(String),
    Tool(String),
    Internal(String),
}
```

- **設計原則**: 標準ライブラリとの名前競合を避けるため`AppError`を使用
- **変換実装**: 外部ライブラリエラーからの自動変換をサポート
- **表示**: 各エラー種別に適した表示形式を提供

### 2. 設定管理（config/settings.rs）

#### 設定ファイル形式

**パス**: `~/.config/obsidian-vault-mcp.toml`

```toml
[vault]
path = "/path/to/obsidian/vault"

[server]
name = "obsidian-vault"
version = "0.1.0"
```

#### Settings 構造体

- **VaultConfig**: Obsidian vault 設定
- **ServerConfig**: サーバー基本情報

#### 機能

- **自動生成**: 設定ファイル不存在時のデフォルト設定作成
- **バリデーション**: パス存在確認
- **ホームディレクトリ検索**: dirs crate による設定ファイル位置解決

### 3. ログ管理（config/logging.rs）

#### ログ初期化戦略

1. **早期初期化**: アプリケーション起動時に環境変数ベースで初期化
2. **設定分離**: `Settings` 構造体からログ設定を独立
3. **環境変数制御**: `RUST_LOG` による標準的なログレベル制御

#### LoggingConfig 構造体

```rust
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
    pub console: bool,
}
```

**注意**: この Config 構造体は将来の拡張用に保持されていますが、現在のログ制御は環境変数 `RUST_LOG` で行います。

#### 初期化メソッド

- **`init_early_logging()`**: 環境変数またはデフォルト設定での早期初期化
- **`validate()`**: ログレベル設定の妥当性検証

### 4. MCP サーバー（mcp/server.rs）

#### 動作モード

1. **非同期モード**: tokio AsyncBufReader 使用
2. **同期モード**: std::io::BufReader 使用（`--sync`オプション）

#### 通信方式

- **STDIO**: 標準入力/出力による JSON-RPC 通信
- **行指向**: 改行区切りの JSON メッセージ処理

#### 処理フロー

1. **早期ログ初期化**: 環境変数ベースのログセットアップ
2. **設定読み込み**: TOML 設定ファイル読み込み
3. **リクエスト受信**: STDIN から JSON-RPC 読み取り
4. **プロトコル処理**: ProtocolHandler に委譲
5. **レスポンス送信**: STDOUT に JSON-RPC 応答

### 5. プロトコルハンドラ（mcp/protocol.rs）

#### サポートメソッド

| メソッド      | 機能               | 実装状況 |
| ------------- | ------------------ | -------- |
| `initialize`  | クライアント初期化 | ✅ 完了  |
| `initialized` | 初期化完了通知     | ✅ 完了  |
| `tools/list`  | 利用可能ツール一覧 | ✅ 完了  |
| `tools/call`  | ツール実行         | ✅ 完了  |

#### エラーハンドリング

- **METHOD_NOT_FOUND (-32601)**: 未知メソッド
- **PARSE_ERROR (-32700)**: JSON 解析エラー
- **INTERNAL_ERROR (-32603)**: 内部処理エラー

### 6. 型定義（mcp/types.rs）

#### 主要型

- **JsonRpcRequest**: リクエストメッセージ
- **JsonRpcResponse**: レスポンスメッセージ
- **InitializeParams/Result**: 初期化パラメータ・結果
- **ToolsListResult**: ツール一覧結果
- **CallToolParams/Result**: ツール実行パラメータ・結果

### 7. ツール登録フレームワーク（tools/registry.rs）

#### Tool トレイト

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn execute(&self, params: Value) -> AppResult<Value>;
}
```

#### ToolRegistry

- **動的登録**: `register()`メソッドによるツール追加
- **実行管理**: `execute_tool()`メソッドによる統一実行
- **スレッドセーフ**: Arc<dyn Tool>による並行実行対応

#### サンプルツール

- **PingTool**: 基本動作確認用エコーツール

## ログ機能

### アーキテクチャ変更（2025-07-30 更新）

#### 設計改善の背景

初回実装後、ログ初期化タイミングの問題が判明しました：

- **問題**: ログ初期化が `Settings::load()` 後に実行されるため、設定ファイル読み込み時の `debug!` マクロが表示されない
- **解決**: ログ機能を `config/logging.rs` に分離し、早期初期化を実装

#### 現在のログ制御方式

1. **環境変数制御**: `RUST_LOG` 環境変数による標準的なログレベル制御
2. **早期初期化**: アプリケーション起動時の即座ログセットアップ
3. **設定ファイル独立**: ログ設定と他の設定の完全分離

### 実装詳細

#### 初期化順序

```rust
// main.rs での処理順序
LoggingConfig::init_early_logging()?;  // 1. 早期ログ初期化
let settings = Settings::load()?;       // 2. 設定読み込み（debug!が正常表示）
```

#### ログレベル制御

- **開発時**: `RUST_LOG=debug cargo run` でデバッグログ表示
- **運用時**: `RUST_LOG=info` または環境変数未設定で info レベル
- **詳細調査**: `RUST_LOG=trace` で最詳細ログ

### 設定

- **レベル制御**: `RUST_LOG` 環境変数による制御
- **出力先**: コンソール（標準エラー出力）
- **フォーマット**: 構造化ログ形式（タイムスタンプ、レベル、メッセージ）

### 実装

- **tracing**: 構造化ログフレームワーク
- **tracing-subscriber**: フィルター・フォーマット制御
- **EnvFilter**: 環境変数ベースのレベル制御

#### 技術的利点

1. **標準的なアプローチ**: Rust エコシステムの標準的なログ制御方式
2. **シンプルな制御**: 環境変数一つでのログレベル制御
3. **開発効率**: デバッグログの確実な表示
4. **保守性**: ログ機能の独立とコードの単純化

## CLI インターフェース

### オプション

| オプション | 説明                | デフォルト                        |
| ---------- | ------------------- | --------------------------------- |
| `--config` | 設定ファイルパス    | ~/.config/obsidian-vault-mcp.toml |
| `--vault`  | Obsidian vault パス | 設定ファイル値                    |
| `--sync`   | 同期モード実行      | false（非同期）                   |

### 使用例

```bash
# 非同期モード（デフォルト）
RUST_LOG=info cargo run -- --vault ./my-vault

# デバッグログ有効
RUST_LOG=debug cargo run -- --vault ./my-vault

# 同期モード
RUST_LOG=info cargo run -- --sync --vault ./my-vault

# カスタム設定ファイル
cargo run -- --config ./custom-config.toml
```

## 技術仕様

### 依存関係

| クレート           | バージョン | 用途                 |
| ------------------ | ---------- | -------------------- |
| tokio              | 1.0        | 非同期ランタイム     |
| serde              | 1.0        | シリアライゼーション |
| serde_json         | 1.0        | JSON 処理            |
| toml               | 0.8        | TOML 設定ファイル    |
| clap               | 4.0        | CLI 引数処理         |
| anyhow             | 1.0        | エラーハンドリング   |
| tracing            | 0.1        | 構造化ログ           |
| tracing-subscriber | 0.3        | ログ設定・フィルター |
| dirs               | 5.0        | ディレクトリ検索     |
| async-trait        | 0.1        | 非同期トレイト       |

### プロトコル準拠

- **JSON-RPC 2.0**: 完全準拠
- **MCP Protocol 2024-11-05**: 基本機能対応

## セキュリティ考慮事項

### 入力検証

- JSON-RPC メッセージ形式検証
- パラメータ型安全性確保
- ファイルパス検証（ディレクトリトラバーサル対策）

### エラー情報漏洩対策

- 内部エラー詳細の適切なマスク
- ログレベルによる情報出力制御

## 制約事項

### 現在の制限

1. **ツール実装**: 基本的な PingTool のみ実装済み
2. **リソース機能**: 未実装（MCP プロトコルの resources 機能）
3. **プロンプト機能**: 未実装（MCP プロトコルの prompts 機能）
4. **認証**: 未実装（STDIO 通信のみ）

### 将来拡張予定

1. **Obsidian 専用ツール**: ファイル操作、検索、メタデータ処理
2. **WebSocket 通信**: ネットワーク経由アクセス
3. **認証機能**: セキュアな接続管理
4. **プラグインシステム**: 動的ツール読み込み

## 変更履歴

- **2025-07-30 02:05**: 初版作成、001-01 プラン実装完了時点
- **2025-07-30 13:30**: ログ設定分離アーキテクチャ改善反映
  - `config/logging.rs` 追加
  - 設定ファイルからログ設定削除
  - 早期ログ初期化実装
  - 環境変数による標準的ログ制御に変更
