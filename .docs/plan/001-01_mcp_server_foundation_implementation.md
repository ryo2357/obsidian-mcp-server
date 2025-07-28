# MCP サーバー基盤実装プラン

## 基本情報

- 作成日: 2025-07-28
- 参照仕様書セクション: 要件 1（MCP サーバー基盤の実装）、要件 2（設定管理システム）
- 参照設計書セクション: MCPServer 構造体、Config 構造体、システム全体の構造設計
- プラン進捗: 1/5（基本設計・環境構築・初回ビルド確認）

## 概要と目標

### 実装する機能の詳細説明

このフェーズでは、MCP サーバーの基盤となる環境構築と基本的なプロジェクト構造を確立します。Rust プロジェクトの依存関係設定、基本的なモジュール構造の作成、および設定管理システムの実装を行います。

### 達成すべき具体的な目標

1. Cargo.toml への必要な依存関係の追加
2. プロジェクトのモジュール構造の作成
3. 設定管理システム（Config モジュール）の実装
4. 基本的なエラーハンドリング構造の準備
5. 初回ビルド成功の確認

### requirements.md との対応関係

- 要件 1-1: cargo install での obsidian-vault-mcp バイナリ配置に向けた基盤構築
- 要件 1-3: RUST_LOG=info レベルでのログ出力基盤準備
- 要件 2-1: ~/.config/obsidian-vault-mcp.toml 設定ファイル読み込み実装
- 要件 2-2: 設定ファイル不在時のエラーメッセージ表示
- 要件 2-3: vault_path 設定値の Vault ルートディレクトリ利用

## 実装手順

### ステップ 1: 依存関係とプロジェクト構造の準備

1. Cargo.toml への依存関係追加

   ```toml
   [dependencies]
   tokio = { version = "1.0", features = ["full"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   toml = "0.8"
   log = "0.4"
   env_logger = "0.11"
   anyhow = "1.0"
   thiserror = "1.0"
   dirs = "5.0"
   ```

2. src/lib.rs の作成とモジュール定義

   ```rust
   pub mod config;
   pub mod error;
   pub mod mcp;
   pub mod tools;
   pub mod vault;
   ```

3. 基本的なディレクトリ構造の作成
   - src/config/mod.rs
   - src/error/mod.rs
   - src/mcp/mod.rs
   - src/tools/mod.rs
   - src/vault/mod.rs

### ステップ 2: エラーハンドリング構造の実装

1. src/error/mod.rs にエラー型定義

   ```rust
   use thiserror::Error;

   #[derive(Error, Debug)]
   pub enum McpError {
       #[error("設定エラー: {0}")]
       Config(#[from] ConfigError),
       #[error("IO エラー: {0}")]
       Io(#[from] std::io::Error),
       #[error("JSON パースエラー: {0}")]
       Json(#[from] serde_json::Error),
   }

   #[derive(Error, Debug)]
   pub enum ConfigError {
       #[error("設定ファイルが見つかりません: {path}")]
       FileNotFound { path: String },
       #[error("設定ファイルの解析に失敗しました: {0}")]
       ParseError(#[from] toml::de::Error),
       #[error("無効な設定値: {field}")]
       InvalidValue { field: String },
   }
   ```

### ステップ 3: 設定管理システムの実装

1. src/config/mod.rs に Config 構造体実装

   ```rust
   use serde::Deserialize;
   use std::path::PathBuf;
   use crate::error::{ConfigError, McpError};

   #[derive(Deserialize, Clone, Debug)]
   pub struct Config {
       pub vault_path: PathBuf,
       pub allowed_extensions: Vec<String>,
       pub max_file_size: usize,
       pub log_level: String,
   }

   impl Config {
       pub fn load_from_file(path: PathBuf) -> Result<Self, ConfigError> {
           if !path.exists() {
               return Err(ConfigError::FileNotFound {
                   path: path.to_string_lossy().to_string(),
               });
           }

           let content = std::fs::read_to_string(&path)
               .map_err(|e| ConfigError::ParseError(toml::de::Error::custom(e)))?;

           let config: Config = toml::from_str(&content)?;
           config.validate()?;
           Ok(config)
       }

       pub fn default_config_path() -> PathBuf {
           dirs::config_dir()
               .unwrap_or_else(|| PathBuf::from("."))
               .join("obsidian-vault-mcp.toml")
       }

       pub fn validate(&self) -> Result<(), ConfigError> {
           if !self.vault_path.exists() {
               return Err(ConfigError::InvalidValue {
                   field: format!("vault_path: {} は存在しません", self.vault_path.display()),
               });
           }

           if self.max_file_size == 0 {
               return Err(ConfigError::InvalidValue {
                   field: "max_file_size は 0 より大きい値である必要があります".to_string(),
               });
           }

           Ok(())
       }
   }

   impl Default for Config {
       fn default() -> Self {
           Self {
               vault_path: PathBuf::from("./vault"),
               allowed_extensions: vec!["md".to_string(), "txt".to_string()],
               max_file_size: 1024 * 1024, // 1MB
               log_level: "info".to_string(),
           }
       }
   }
   ```

### ステップ 4: main.rs の基本構造実装

1. src/main.rs の更新

   ```rust
   use env_logger;
   use log::{info, error};
   use obsidian_mcp_server::config::Config;
   use obsidian_mcp_server::error::McpError;

   #[tokio::main]
   async fn main() -> Result<(), McpError> {
       // ログ初期化
       env_logger::init();

       info!("obsidian-vault-mcp starting...");

       // 設定ファイル読み込み
       let config_path = Config::default_config_path();
       info!("設定ファイルパス: {}", config_path.display());

       let config = match Config::load_from_file(config_path.clone()) {
           Ok(config) => {
               info!("設定ファイル読み込み完了: {}", config_path.display());
               info!("Vault ルート: {}", config.vault_path.display());
               config
           }
           Err(e) => {
               error!("設定ファイル読み込みエラー: {}", e);
               return Err(McpError::Config(e));
           }
       };

       info!("MCP サーバー初期化完了");

       // TODO: 次のフェーズで MCP サーバー実装
       Ok(())
   }
   ```

### ステップ 5: ビルドテストと動作確認

1. プロジェクトのビルド確認

   ```bash
   cargo build
   ```

2. 設定ファイルの作成と動作テスト

   ```bash
   # テスト用設定ファイル作成
   mkdir -p ~/.config
   cat > ~/.config/obsidian-vault-mcp.toml << 'EOF'
   vault_path = "./test-vault"
   allowed_extensions = ["md", "txt"]
   max_file_size = 1048576
   log_level = "info"
   EOF

   # テスト用 Vault ディレクトリ作成
   mkdir -p ./test-vault

   # アプリケーション実行テスト
   RUST_LOG=info cargo run
   ```

3. エラーハンドリングのテスト

   ```bash
   # 設定ファイルを一時移動してエラーテスト
   mv ~/.config/obsidian-vault-mcp.toml ~/.config/obsidian-vault-mcp.toml.bak
   RUST_LOG=info cargo run
   # 設定ファイルを復元
   mv ~/.config/obsidian-vault-mcp.toml.bak ~/.config/obsidian-vault-mcp.toml
   ```

## 品質保証

### 各フェーズでの確認ポイント

1. 依存関係追加後

   - [ ] cargo build が成功すること
   - [ ] 追加した依存関係が適切にインポートできること

2. モジュール構造作成後

   - [ ] 各モジュールが適切に認識されること
   - [ ] lib.rs でモジュールが正しく公開されること

3. エラーハンドリング実装後

   - [ ] カスタムエラー型が適切に動作すること
   - [ ] エラーメッセージが日本語で表示されること

4. 設定管理実装後

   - [ ] 設定ファイルが正常に読み込まれること
   - [ ] 設定ファイル不在時に適切なエラーが表示されること
   - [ ] 設定値のバリデーションが動作すること

5. 全体統合後
   - [ ] アプリケーションが正常に起動すること
   - [ ] ログが適切に出力されること

### requirements.md との整合性確認

- [x] 要件 1-3: RUST_LOG=info レベルでのログ出力（env_logger で実装）
- [x] 要件 2-1: ~/.config/obsidian-vault-mcp.toml からの設定読み込み
- [x] 要件 2-2: 設定ファイル不在時の適切なエラーメッセージ表示
- [x] 要件 2-3: vault_path 設定値の Vault ルートディレクトリ利用

### design.md との整合性確認

- [x] Config 構造体の設計パターンに従った実装
- [x] エラーハンドリングの設計パターンに従った実装
- [x] モジュール構造の設計パターンに従った配置
- [x] 日本語エラーメッセージの実装

## 次のフェーズへの引き継ぎ事項

1. 実装済みの基盤構造

   - 設定管理システム (Config モジュール)
   - エラーハンドリング構造 (error モジュール)
   - 基本的なプロジェクト構造

2. 次フェーズでの実装対象

   - MCP プロトコルハンドラーの実装
   - JSON-RPC リクエスト/レスポンス処理
   - Tool レジストリの基本構造

3. 継続的な品質保証事項
   - ビルド成功の維持
   - ログ出力の一貫性
   - エラーハンドリングの拡張
