# バイナリクレートにおける lib.rs の必要性について

作成日時: 2025-07-30 14:30

## 議論の背景

コーディング計画 `001-01-project-setup.md` において、プロジェクト構造に `lib.rs` が含まれていたが、このプロジェクトがバイナリクレートであることを考慮し、その必要性について検討した。

## プロジェクトの性質

- **プロジェクト名**: obsidian-mcp-server
- **クレートタイプ**: バイナリクレート（`main.rs` が存在）
- **用途**: MCP（Model Context Protocol）サーバーとしてのスタンドアロンアプリケーション

## バイナリクレートでの lib.rs の利点

### 1. コードの再利用可能性

#### 例示

```rust
// lib.rs でのパブリック API 定義
pub mod mcp;
pub mod config;
pub mod tools;

pub use mcp::Server;
pub use config::Settings;
pub use tools::Tool;

// 他のプロジェクトから利用可能
// external_project/src/main.rs
use obsidian_mcp_server::{Server, Settings};

fn main() {
    let settings = Settings::load().unwrap();
    let server = Server::new(settings);
    // サーバーをライブラリとして組み込み
}
```

### 2. テストの書きやすさ

#### 例示

```rust
// lib.rs
pub fn validate_vault_path(path: &str) -> Result<(), String> {
    // バリデーションロジック
}

// tests/integration_test.rs
use obsidian_mcp_server::validate_vault_path;

#[test]
fn test_valid_vault_path() {
    assert!(validate_vault_path("/valid/path").is_ok());
}

#[test]
fn test_invalid_vault_path() {
    assert!(validate_vault_path("").is_err());
}
```

### 3. モジュール構造の整理

#### 例示

```rust
// lib.rs でのモジュール構造定義
pub mod mcp {
    pub mod server;
    pub mod protocol;
    pub mod types;
}

pub mod config {
    pub mod settings;
}

pub mod tools {
    pub mod registry;
}

pub mod error;

// main.rs はシンプルに
use obsidian_mcp_server::mcp::Server;
use obsidian_mcp_server::config::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::load()?;
    let server = Server::new(settings);
    server.run().await
}
```

### 4. 他のプロジェクトからライブラリとして利用可能

#### 例示

```toml
# 他のプロジェクトの Cargo.toml
[dependencies]
obsidian-mcp-server = { path = "../obsidian-mcp-server" }
```

```rust
// 他のプロジェクトでの利用例
use obsidian_mcp_server::tools::{Tool, Registry};

struct CustomTool;

impl Tool for CustomTool {
    fn name(&self) -> &str { "custom-tool" }
    // 他のメソッド実装
}

fn main() {
    let mut registry = Registry::new();
    registry.register(Box::new(CustomTool));
    // カスタムツールを含むレジストリの利用
}
```

## 本プロジェクトでの結論

### lib.rs を使用しない場合の構造

```text
src/
├── main.rs              # エントリーポイント
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

### 推奨事項

MCP サーバーという特定用途のスタンドアロンアプリケーションであることを考慮すると、現時点では `lib.rs` は必須ではない。しかし、将来的に以下のニーズが生じた場合は追加を検討する：

1. 他のプロジェクトで MCP サーバー機能を再利用したい場合
2. 統合テストを充実させたい場合
3. プラグイン機能を外部ライブラリとして提供したい場合

## コーディング計画への反映

`001-01-project-setup.md` から `lib.rs` の記述を削除し、バイナリクレートとしてよりシンプルな構造に修正することを推奨する。
