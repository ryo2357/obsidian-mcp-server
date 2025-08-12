mod config;
mod debug;
mod error;
mod mcp;
mod vault;

use clap::Parser;
use config::Config;
use debug::DebugConfig;
use error::AppResult;
use mcp::server::McpServer;
use std::path::PathBuf;

/// Obsidian MCP サーバー
#[derive(Parser)]
#[command(name = "obsidian-mcp-server")]
#[command(about = "Model Context Protocol server for Obsidian")]
#[command(version = env!("CARGO_PKG_VERSION"))]
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

#[tokio::main]
async fn main() -> AppResult<()> {
    let cli = Cli::parse();

    // デバッグモードの処理
    if cli.debug {
        println!("Starting in debug mode...");
        let debug_config = DebugConfig::new();
        debug_config.ensure_debug_vault()?;
        debug_config.generate_dummy_data()?;
        debug_config.debug_log("Debug environment initialized");

        // デバッグモード用の設定を作成
        let config = Config {
            vault_path: Some(debug_config.vault_path),
        };

        let mut server = McpServer::new(config);

        if cli.sync {
            server.run_sync()?;
        } else {
            server.run_async().await?;
        }

        return Ok(());
    }

    // 通常モードの処理
    let mut config = if let Some(config_path) = cli.config {
        Config::load_from_file(config_path)?
    } else {
        Config::load_or_default()?
    };

    // コマンドライン引数で vault_path を上書き
    if let Some(vault_path) = cli.vault_path {
        config.vault_path = Some(vault_path);
    }

    // MCP サーバーを作成・起動
    let mut server = McpServer::new(config);

    if cli.sync {
        server.run_sync()?;
    } else {
        server.run_async().await?;
    }

    Ok(())
}
