

use clap::Parser;
use config::Config;
use debug::DebugConfig;
use error::AppResult;
use std::path::PathBuf;
use once_cell::sync::Lazy;
use log::{debug};
use rmcp::{ServiceExt, transport::io::stdio};

mod config;
mod debug;
mod error;
// mod vault;
mod logger;
mod server;



pub static APP_DIR: Lazy<PathBuf>  = Lazy::new(|| 
  if let Some(config_dir) = dirs::config_dir() {
    config_dir
      .join("obsidian-mcp-server")
    } else {
      PathBuf::from("./.config")
});

pub static CONFIG_PATH: Lazy<PathBuf>  = Lazy::new(|| 
  APP_DIR.join("config.toml"));


/// Obsidian MCP サーバー
#[derive(Parser)]
#[command(name = "obsidian-mcp-server")]
#[command(about = "Model Context Protocol server for Obsidian")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// デバッグモードで実行
    #[arg(short,long)]
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

          Config::load_or_default(CONFIG_PATH.clone())?

        },
        
    };

    // MCP サービスを作成し、標準入出力トランスポートで提供
    let service = server::ObsidianServer::new(config).serve(stdio()).await?;
    // クライアントからの要求を待機
    service.waiting().await?;


    Ok(())
}
