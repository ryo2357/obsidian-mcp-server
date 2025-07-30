use clap::{Arg, Command};
use obsidian_mcp_server::{config::{Settings, LoggingConfig}, mcp::McpServer, AppResult};
use tracing::info;

#[tokio::main]
async fn main() -> AppResult<()> {
    let matches = Command::new("obsidian-mcp-server")
        .version("0.1.0")
        .about("MCP server for Obsidian vault operations")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Custom config file path"),
        )
        .arg(
            Arg::new("vault")
                .short('v')
                .long("vault")
                .value_name("PATH")
                .help("Obsidian vault path"),
        )
        .arg(
            Arg::new("sync")
                .long("sync")
                .action(clap::ArgAction::SetTrue)
                .help("Run in synchronous mode (for testing)"),
        )
        .get_matches();

    // 早期ログ初期化（環境変数またはデフォルト設定）
    LoggingConfig::init_early_logging()?;

    // 設定の読み込み
    let mut settings = Settings::load()?;

    // コマンドライン引数で Vault パスが指定された場合は設定を上書き
    if let Some(vault_path) = matches.get_one::<String>("vault") {
        settings.set_vault_path(vault_path.clone());
    }

    info!("Starting Obsidian MCP Server");
    info!("Server: {} v{}", settings.server.name, settings.server.version);
    
    if let Some(vault_path) = settings.get_vault_path() {
        info!("Vault path: {}", vault_path);
    } else {
        info!("No vault path configured");
    }

    // MCP サーバーの起動
    let server = McpServer::new(settings);

    if matches.get_flag("sync") {
        info!("Running in synchronous mode");
        server.run_sync()?;
    } else {
        server.run().await?;
    }

    Ok(())
}
