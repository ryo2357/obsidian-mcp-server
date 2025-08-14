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

    let config = match cli.debug {
        true =>{
          println!("Starting in debug mode...");
          let debug_config = DebugConfig::new();

          // デバッグ用の vault とダミーデータを作成
          debug_config.ensure_debug_vault()?;
          debug_config.generate_dummy_data()?;
          println!("Debug environment initialized");

          let mut config = Config::load_or_default(Some(&debug_config.config_path))?;
          config.set_vault_path(debug_config.vault_path.clone());

          config

        },
        false => {
          // 通常モードの設定を読み込み
          let mut config = Config::load_or_default(cli.config.as_deref())?;

          // コマンドライン引数で vault_path を上書き
          if let Some(vault_path) = cli.vault_path {
              config.set_vault_path(vault_path);
          }
          config
          
        },
        
    };

    // MCP サーバーを作成・起動
    let mut server = McpServer::new(config);

    if cli.sync {
        server.run_sync()?;
    } else {
        server.run_async().await?;
    }

    Ok(())
}
