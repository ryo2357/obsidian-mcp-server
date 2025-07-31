mod config;
mod error;
mod mcp;

use clap::Parser;
use config::Config;
use error::AppResult;
use mcp::McpServer;
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
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let cli = Cli::parse();

    // 設定を読み込み
    let mut config = if let Some(config_path) = cli.config {
        Config::load_from_file(config_path)?
    } else {
        Config::load_or_default()?
    };

    // コマンドライン引数で vault_path を上書き
    if let Some(vault_path) = cli.vault_path {
        config.vault_path = vault_path;
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
