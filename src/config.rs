use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Obsidian vault のパス
    pub vault_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vault_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("Documents")
                .join("vault"),
        }
    }
}

impl Config {
    /// 設定ファイルのデフォルトパスを取得
    pub fn default_config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir
                .join("obsidian-mcp-server")
                .join("config.toml")
        } else {
            PathBuf::from("config.toml")
        }
    }

    /// 設定ファイルを読み込み
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;
        
        let config: Config = toml::from_str(&contents)
            .with_context(|| "Failed to parse config file")?;
        
        Ok(config)
    }

    /// 設定ファイルを保存
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        std::fs::write(&path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;
        
        Ok(())
    }

    /// 設定を読み込み、ファイルが存在しない場合はデフォルト値を使用
    pub fn load_or_default() -> Result<Self> {
        let config_path = Self::default_config_path();
        
        if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            let config = Self::default();
            // デフォルト設定を保存
            if let Err(e) = config.save_to_file(&config_path) {
                eprintln!("Warning: Failed to save default config: {}", e);
            }
            Ok(config)
        }
    }
}
