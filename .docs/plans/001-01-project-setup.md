# 001-01 プロジェクト基盤構築と MCP サーバー実装

作成日時: 2025-07-29 14:30
参照アイディアファイル: .docs/ideas/001\_プロジェクトの概要.md

## 概要

Obsidian vault 操作用 MCP サーバーの基盤構築を行います。Rust での基本的な MCP サーバー実装、設定ファイル読み込み機能、およびプロジェクト構造の整備を実施します。

## 全体計画

- 001-01: プロジェクト基盤構築と MCP サーバー実装（本計画）
- 001-02: Obsidian vault 操作ツールの実装
- 001-03: インストール・設定・テスト環境の整備

## 実装内容

### 1. プロジェクト構造の整備

```
src/
├── main.rs              # エントリーポイント
├── lib.rs               # ライブラリルート
├── mcp/
│   ├── mod.rs           # MCPモジュール
│   ├── server.rs        # MCPサーバー実装
│   ├── protocol.rs      # MCPプロトコル処理
│   └── types.rs         # MCP型定義
├── config/
│   ├── mod.rs           # 設定モジュール
│   └── settings.rs      # 設定ファイル処理
├── tools/
│   ├── mod.rs           # ツールモジュール
│   └── registry.rs      # ツール登録管理
└── error.rs             # エラー型定義
```

### 2. 依存関係の追加

Cargo.toml に以下の依存関係を追加：

- `tokio`: 非同期ランタイム
- `serde`: シリアライゼーション
- `serde_json`: JSON 処理
- `toml`: TOML 設定ファイル処理
- `clap`: CLI 引数処理
- `anyhow`: エラーハンドリング
- `tracing`: ログ出力
- `tracing-subscriber`: ログ設定
- `dirs`: ホームディレクトリ取得

### 3. 基本的な MCP サーバー実装

#### MCP プロトコルの基本構造

- Initialize/Initialized メッセージ処理
- Tools/List メッセージ処理
- Tools/Call メッセージ処理
- JSON-RPC 2.0 プロトコル対応

#### エラーハンドリング

- カスタムエラー型の定義
- 適切なエラーレスポンス生成
- ログ出力との連携

### 4. 設定ファイル機能の実装

#### 設定ファイル仕様

ファイルパス: `~/.config/obsidian-vault-mcp.toml`

```toml
[vault]
path = "/path/to/obsidian/vault"

[server]
name = "obsidian-vault"
version = "0.1.0"

[logging]
level = "info"
```

#### 設定読み込み機能

- ホームディレクトリからの設定ファイル検索
- デフォルト設定の提供
- 設定値のバリデーション

### 5. ツール登録フレームワーク

#### ツールレジストリ

- 動的ツール登録機能
- ツール一覧の管理
- ツール実行の統一インターフェース

#### ツールトレイト定義

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value>;
}
```

### 6. ログ機能の実装

- 環境変数 `RUST_LOG` による制御
- 構造化ログ出力
- デバッグ情報の適切な出力

## ビルド・動作確認

### ビルド確認

```bash
cargo build
cargo test
```

### 基本動作確認

```bash
cargo run -- --help
```

### 設定ファイル動作確認

- 設定ファイルの自動生成確認
- 設定値の正常読み込み確認

## 期待される成果

- MCP サーバーとしての基本機能が動作
- 設定ファイルからの設定読み込みが正常動作
- ツール登録フレームワークが整備
- 次フェーズでのツール実装準備完了

## 技術的考慮事項

- JSON-RPC 2.0 プロトコルの正確な実装
- 非同期処理の適切な設計
- エラーハンドリングの統一
- 拡張性を考慮したアーキテクチャ設計
