mod config;
mod debug;
mod error;
mod mcp;
mod vault;
mod logger;

use clap::Parser;
use config::Config;
use debug::DebugConfig;
use error::AppResult;
use mcp::server::McpServer;
use std::path::PathBuf;
use once_cell::sync::Lazy;
use log::{debug};

pub static APP_DIR: Lazy<PathBuf>  = Lazy::new(|| 
  if let Some(config_dir) = dirs::config_dir() {
    config_dir
      .join("obsidian-mcp-server")
    } else {
      PathBuf::from("./.config")
});


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
    vault_dir: Option<PathBuf>,

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

          let debug_config = DebugConfig::new();
          // デバッグモードでロガーを初期化
          logger::init_logger("debug", debug_config.config_dir.clone())?;
          
          debug!("Starting in debug mode...");
          // デバッグ用の vault とダミーデータを作成
          debug_config.ensure_debug_vault()?;
          debug_config.generate_dummy_data()?;
          debug!("Debug environment initialized");

          let mut config = Config::load_or_default(debug_config.config_path.clone())?;
          config.set_vault_dir(debug_config.vault_dir.clone());

          config

        },
        false => {
          // 通常モードでロガーを初期化
          logger::init_logger("info", APP_DIR.join("logs"))?;

          // 通常モードの設定を読み込み
          let config_path = match cli.config {
              Some(path) => path,
              None => APP_DIR.join("config.toml"),
          };
          let mut config = Config::load_or_default(config_path)?;

          // コマンドライン引数で vault_dir を上書き
          if let Some(vault_dir) = cli.vault_dir {
              config.set_vault_dir(vault_dir);
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
