use serde::Deserialize;
use std::path::PathBuf;
use crate::error::ConfigError;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub vault_path: PathBuf,
    pub allowed_extensions: Vec<String>,
    pub max_file_size: usize,
    pub log_level: String,
}

impl Config {
    pub fn load_from_file(path: PathBuf) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| ConfigError::InvalidValue { 
                field: format!("設定ファイル読み込みエラー: {}", e) 
            })?;

        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("obsidian-vault-mcp.toml")
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.vault_path.exists() {
            return Err(ConfigError::InvalidValue {
                field: format!("vault_path: {} は存在しません", self.vault_path.display()),
            });
        }

        if self.max_file_size == 0 {
            return Err(ConfigError::InvalidValue {
                field: "max_file_size は 0 より大きい値である必要があります".to_string(),
            });
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vault_path: PathBuf::from("./vault"),
            allowed_extensions: vec!["md".to_string(), "txt".to_string()],
            max_file_size: 1024 * 1024, // 1MB
            log_level: "info".to_string(),
        }
    }
}
