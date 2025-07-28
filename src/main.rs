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

    let _config = match Config::load_from_file(config_path.clone()) {
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
