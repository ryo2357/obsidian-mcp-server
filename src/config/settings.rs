use crate::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub vault: VaultConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
    pub console: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            vault: VaultConfig { path: None },
            server: ServerConfig {
                name: "obsidian-vault".to_string(),
                version: "0.1.0".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: Some("logs/mcp-server.log".to_string()),
                console: true,
            },
        }
    }
}

impl Settings {
    pub fn load() -> AppResult<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            info!("Config file not found, creating default: {:?}", config_path);
            let default_settings = Self::default();
            default_settings.save(&config_path)?;
            return Ok(default_settings);
        }

        debug!("Loading config from: {:?}", config_path);
        let content = std::fs::read_to_string(&config_path)?;
        
        // デバッグ用：設定ファイルの内容を表示
        debug!("Config file content: {}", content);
        
        let settings: Settings = toml::from_str(&content)?;

        // 設定値のバリデーション
        settings.validate()?;

        Ok(settings)
    }

    pub fn save(&self, path: &Path) -> AppResult<()> {
        // 親ディレクトリを作成
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)?;
        info!("Config saved to: {:?}", path);
        Ok(())
    }

    fn get_config_path() -> AppResult<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| AppError::Config("Cannot determine config directory".to_string()))?;

        Ok(config_dir.join("obsidian-vault-mcp.toml"))
    }

    fn validate(&self) -> AppResult<()> {
        // ログレベルの検証
        match self.logging.level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(AppError::Config(format!(
                    "Invalid log level: {}. Must be one of: trace, debug, info, warn, error",
                    self.logging.level
                )));
            }
        }

        // Vault パスの検証（設定されている場合）
        if let Some(ref vault_path) = self.vault.path {
            let path = Path::new(vault_path);
            if !path.exists() {
                warn!("Vault path does not exist: {}", vault_path);
            } else if !path.is_dir() {
                return Err(AppError::Config(format!(
                    "Vault path is not a directory: {}",
                    vault_path
                )));
            }
        }

        Ok(())
    }

    pub fn get_vault_path(&self) -> Option<&str> {
        self.vault.path.as_deref()
    }

    pub fn set_vault_path(&mut self, path: String) {
        self.vault.path = Some(path);
    }
}
