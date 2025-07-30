use crate::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{EnvFilter, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
    pub console: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: Some("logs/mcp-server.log".to_string()),
            console: true,
        }
    }
}

impl LoggingConfig {
    /// 環境変数またはデフォルト設定でログを初期化
    /// Settings::load() より前に呼び出される
    pub fn init_early_logging() -> AppResult<()> {
        // 環境変数 RUST_LOG が設定されている場合はそれを使用、
        // そうでなければデフォルトの "info" レベルを使用
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));

        // 早期初期化では簡単なコンソール出力のみ
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .try_init()
            .map_err(|e| AppError::Internal(format!("Failed to initialize early logging: {}", e)))?;

        Ok(())
    }

    /// 設定ファイルに基づいてログを再初期化
    /// Settings::load() の後に呼び出される
    pub fn init_full_logging(&self) -> AppResult<()> {
        // 既存のグローバル subscriber を置き換えることはできないため、
        // 早期初期化済みの場合は警告を出すのみ
        tracing::warn!("Logging system already initialized. File output and advanced settings may not be applied.");
        tracing::info!("Current logging configuration:");
        tracing::info!("  Level: {}", self.level);
        tracing::info!("  Console: {}", self.console);
        if let Some(ref file) = self.file {
            tracing::info!("  File: {}", file);
        }

        Ok(())
    }

    /// 設定値を検証
    pub fn validate(&self) -> AppResult<()> {
        // ログレベルの検証
        match self.level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(AppError::Config(format!(
                    "Invalid log level: {}. Must be one of: trace, debug, info, warn, error",
                    self.level
                )));
            }
        }

        Ok(())
    }
}

/// 完全なログ初期化を行う関数
/// この関数は将来的に設定ファイルベースの完全な初期化用に使用される
pub fn init_logging_with_config(config: &LoggingConfig) -> AppResult<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    // ファイル出力の設定
    if let Some(ref log_file) = config.file {
        // ログディレクトリを作成
        if let Some(parent) = std::path::Path::new(log_file).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file_appender = tracing_appender::rolling::daily("logs", "mcp-server.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        
        if config.console {
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
                .map_err(|e| AppError::Internal(format!("Failed to initialize logging: {}", e)))?;
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
                .map_err(|e| AppError::Internal(format!("Failed to initialize logging: {}", e)))?;
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
            .map_err(|e| AppError::Internal(format!("Failed to initialize logging: {}", e)))?;
    }

    Ok(())
}
