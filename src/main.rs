use clap::{Arg, Command};
use obsidian_mcp_server::{config::Settings, mcp::McpServer, AppResult};
use tracing::info;
use tracing_subscriber::{EnvFilter, prelude::*};

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

    // 設定の読み込み
    let mut settings = Settings::load()?;

    // コマンドライン引数で Vault パスが指定された場合は設定を上書き
    if let Some(vault_path) = matches.get_one::<String>("vault") {
        settings.set_vault_path(vault_path.clone());
    }

    // ログの初期化
    init_logging(&settings)?;

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

fn init_logging(settings: &Settings) -> AppResult<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&settings.logging.level));

    // ファイル出力の設定
    if let Some(ref log_file) = settings.logging.file {
        // ログディレクトリを作成
        if let Some(parent) = std::path::Path::new(log_file).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file_appender = tracing_appender::rolling::daily("logs", "mcp-server.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        
        if settings.logging.console {
            // ファイルとコンソール両方への出力
            // コンソール用のレイヤー（色付き）
            let console_layer = tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .with_writer(std::io::stdout);
            
            // ファイル用のレイヤー（色なし）
            let file_layer = tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .with_writer(non_blocking);
            
            tracing_subscriber::registry()
                .with(filter)
                .with(console_layer)
                .with(file_layer)
                .try_init()
                .map_err(|e| obsidian_mcp_server::AppError::Internal(format!("Failed to initialize logging: {}", e)))?;
        } else {
            // ファイルのみ（色なし）
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_ansi(false)
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .with_writer(non_blocking)
                .try_init()
                .map_err(|e| obsidian_mcp_server::AppError::Internal(format!("Failed to initialize logging: {}", e)))?;
        }
        
        // _guardを保持するためにメモリリークを防ぐ（本来は適切な管理が必要）
        std::mem::forget(_guard);
    } else {
        // コンソールのみ（従来の方式）
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .try_init()
            .map_err(|e| obsidian_mcp_server::AppError::Internal(format!("Failed to initialize logging: {}", e)))?;
    }

    Ok(())
}
