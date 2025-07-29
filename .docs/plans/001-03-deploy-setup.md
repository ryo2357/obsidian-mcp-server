# 001-03 インストール・設定・テスト環境の整備

作成日時: 2025-07-29 14:30
参照アイディアファイル: .docs/ideas/001\_プロジェクトの概要.md

## 概要

cargo install によるインストール機能、VSCode との連携設定、総合テスト環境の整備を行います。プロジェクトの完成とデプロイ準備を実施します。

## 前提条件

- 001-01 のプロジェクト基盤構築が完了していること
- 001-02 の Obsidian vault 操作ツールが実装されていること
- MCP サーバーとツールが正常動作していること

## 実装内容

### 1. cargo install によるインストール機能

#### バイナリ設定の追加

Cargo.toml の設定:

```toml
[[bin]]
name = "obsidian-vault-mcp"
path = "src/main.rs"

[package.metadata]
description = "Obsidian vault操作用Model Context Protocolサーバー"
homepage = "https://github.com/ryo2357/obsidian-mcp-server"
repository = "https://github.com/ryo2357/obsidian-mcp-server"
keywords = ["obsidian", "mcp", "model-context-protocol"]
categories = ["command-line-utilities"]
```

#### CLI 引数処理の実装

```rust
#[derive(Parser)]
#[command(name = "obsidian-vault-mcp")]
#[command(about = "Obsidian vault操作用MCPサーバー")]
pub struct Cli {
    #[arg(long, help = "設定ファイルのパス")]
    config: Option<PathBuf>,

    #[arg(long, help = "デバッグモードで実行")]
    debug: bool,

    #[arg(long, help = "設定ファイルの初期化")]
    init_config: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 設定ファイルの検証
    ValidateConfig,
    /// vault接続テスト
    TestVault,
    /// MCPサーバーの起動
    Serve,
}
```

### 2. 設定ファイルの初期化機能

#### 設定ファイル構造

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub vault_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vault_path: PathBuf::new(), // 空のパス（要設定）
        }
    }
}
```

#### 設定ファイルパス決定

```rust
const APP_NAME: &str = "obsidian-vault-mcp";

#[cfg(debug_assertions)]
pub fn config_path() -> PathBuf {
    PathBuf::from("./.config/config.toml")
}

#[cfg(not(debug_assertions))]
pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to get home directory")
        .join(format!("{}.toml", APP_NAME))
}
```

#### 設定ファイル自動読み込み

```rust
impl Config {
    pub fn load() -> Self {
        let config_path = config_path();

        // デバッグ時の設定ディレクトリ作成
        #[cfg(debug_assertions)]
        {
            if let Some(parent) = config_path.parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        eprintln!("設定ディレクトリの作成に失敗しました: {}", e);
                        return Self::default();
                    }
                }
            }
        }

        let config_content = if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("設定ファイルの読み込みに失敗しました: {}", e);
                    return Self::default();
                }
            }
        } else {
            // ファイルが存在しない場合はデフォルト設定を作成
            let default_config = Self::default();
            match toml::to_string(&default_config) {
                Ok(toml_string) => {
                    if let Err(e) = fs::write(&config_path, &toml_string) {
                        eprintln!("設定ファイルの書き込みに失敗しました: {}", e);
                    }
                    toml_string
                }
                Err(e) => {
                    eprintln!("設定のシリアライズに失敗しました: {}", e);
                    return Self::default();
                }
            }
        };

        let config = match toml::from_str::<Config>(&config_content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("設定ファイルのパースに失敗しました: {}. デフォルト値を使用します", e);
                Self::load_with_fallback(&config_content)
            }
        };

        // vault_pathの検証
        if config.vault_path.as_os_str().is_empty() {
            eprintln!("エラー: vault_pathが設定されていません");
            eprintln!("設定ファイルの初期化を実行してください:");
            eprintln!("  obsidian-vault-mcp --init-config");
            std::process::exit(1);
        }

        if !config.vault_path.exists() {
            eprintln!("エラー: 指定されたvault_pathが存在しません: {:?}", config.vault_path);
            eprintln!("設定ファイルを確認してください:");
            #[cfg(debug_assertions)]
            eprintln!("  ./.config/config.toml");
            #[cfg(not(debug_assertions))]
            eprintln!("  ~/obsidian-vault-mcp.toml");
            std::process::exit(1);
        }

        if !config.vault_path.is_dir() {
            eprintln!("エラー: vault_pathがディレクトリではありません: {:?}", config.vault_path);
            std::process::exit(1);
        }

        config
    }

    fn load_with_fallback(config_content: &str) -> Self {
        let mut config = Self::default();

        if let Ok(table) = toml::from_str::<toml::Table>(config_content) {
            // vault_path設定の復旧
            if let Some(path_value) = table.get("vault_path") {
                if let Some(path_str) = path_value.as_str() {
                    config.vault_path = PathBuf::from(path_str);
                }
            }
        }

        config
    }

    pub fn save(&self) {
        let config_path = config_path();

        #[cfg(debug_assertions)]
        {
            if let Some(parent) = config_path.parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        eprintln!("設定ディレクトリの作成に失敗しました: {}", e);
                        return;
                    }
                }
            }
        }

        match toml::to_string(self) {
            Ok(toml_string) => {
                if let Err(e) = fs::write(&config_path, toml_string) {
                    eprintln!("設定ファイルの書き込みに失敗しました: {}", e);
                }
            }
            Err(e) => {
                eprintln!("設定のシリアライズに失敗しました: {}", e);
            }
        }
    }
}
```

#### インタラクティブ設定機能

```rust
pub fn interactive_config_setup() -> Result<Config> {
    println!("Obsidian vault MCPサーバーの設定を開始します");

    let vault_path = prompt_vault_path()?;

    let config = Config {
        vault_path,
    };

    config.save();
    Ok(config)
}

fn prompt_vault_path() -> Result<PathBuf> {
    use std::io::{self, Write};

    print!("Obsidian vaultのパスを入力してください: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let path = PathBuf::from(input.trim());

    if !path.exists() {
        return Err("指定されたパスが存在しません".into());
    }

    if !path.is_dir() {
        return Err("指定されたパスはディレクトリではありません".into());
    }

    Ok(path)
}
```

### 3. VSCode 連携設定

#### README.md への設定手順追加

VSCode 設定の詳細説明:

```json
{
  "github.copilot.chat.modelContextProtocol.servers": {
    "obsidian-vault": {
      "command": "obsidian-vault-mcp",
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

#### 設定検証ツール

```rust
pub fn validate_vscode_integration() -> Result<()> {
    // ~/.cargo/binの存在確認
    // obsidian-vault-mcpの実行可能性確認
    // 環境変数設定の確認
}
```

### 4. 総合テスト環境の整備

#### 統合テストスイート

```rust
#[cfg(test)]
mod integration_tests {
    // テスト用vault作成
    // MCPサーバー起動テスト
    // ツール実行テスト
    // エラーケーステスト
}
```

#### テスト用 vault 構築

```text
tests/
├── fixtures/
│   └── test_vault/
│       ├── .obsidian/
│       ├── drafts/
│       ├── notes/
│       └── copilot-generated/
└── integration/
    ├── server_test.rs
    ├── tools_test.rs
    └── config_test.rs
```

#### 性能テスト

```rust
#[cfg(test)]
mod performance_tests {
    // 大量ファイル処理テスト
    // メモリ使用量テスト
    // 応答時間テスト
}
```

### 5. ドキュメントの整備

#### README.md の充実

- プロジェクト概要
- インストール手順
- 設定方法
- 使用方法

#### 使用例の追加

````markdown
## 使用例

### 基本的な使用方法

1. インストール

   ```bash
   cargo install --path .
   ```

2. 設定ファイル初期化

   ```bash
   obsidian-vault-mcp --init-config
   ```

3. VSCode 設定
   settings.json に設定を追加

4. GitHub Copilot での使用
   チャットで markdown ファイル保存を実行

### 6. エラー処理とログの改善

#### 包括的なエラーハンドリング

```rust
pub fn handle_cli_error(error: &anyhow::Error) {
    match error.downcast_ref::<ConfigError>() {
        Some(config_err) => {
            eprintln!("設定エラー: {}", config_err);
            #[cfg(debug_assertions)]
            eprintln!("設定ファイルを確認してください: ./.config/config.toml");
            #[cfg(not(debug_assertions))]
            eprintln!("設定ファイルを確認してください: ~/obsidian-vault-mcp.toml");
        },
        None => {
            eprintln!("予期しないエラー: {}", error);
        }
    }
}
```
````

#### 運用監視機能

```rust
pub struct HealthChecker;

impl HealthChecker {
    pub fn check_vault_health(&self) -> Result<HealthStatus> {
        // vault接続確認
        // ディスク容量確認
        // 権限確認
    }
}
```

## ビルド・動作確認

### インストールテスト

```bash
# ローカルインストール
cargo install --path .

# インストール確認
which obsidian-vault-mcp
obsidian-vault-mcp --help
```

### VSCode 連携テスト

1. VSCode 設定の追加
2. GitHub Copilot チャットでの動作確認
3. ツール実行とファイル保存確認

### 総合テスト

```bash
# 全テスト実行
cargo test

# 統合テスト実行
cargo test --test integration

# パフォーマンステスト実行
cargo test --release performance
```

## 期待される成果

- cargo install による簡単インストール
- VSCode との完全な連携動作
- 安定した動作と適切なエラーハンドリング
- 包括的なドキュメントとサポート
- プロダクション環境での使用準備完了
