use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};


/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Obsidian vault のパス
    vault_dir: Option<PathBuf>,
    /// テンプレートファイル (vault_dir からの相対パス)
    template_file: Option<PathBuf>,
    /// ノートタグ候補一覧
    #[serde(default)]
    tag_list: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vault_dir: None,
            template_file: None,
            tag_list: vec!["Tips".to_string()],
        }
    }
}


impl Config {
    /// vault_dirを取得（Noneの場合はエラー）
    pub fn get_vault_dir(&self) -> Result<&PathBuf> {
        self.vault_dir.as_ref()
            .with_context(|| "Vault path is not configured. Please set vault_dir in config file.")
    }

    pub fn set_vault_dir<P: AsRef<Path>>(&mut self, path: P) {
        self.vault_dir = Some(path.as_ref().to_path_buf());
    }

    /// template_file を取得 (Option)
    pub fn get_template_file(&self) -> Option<&PathBuf> { self.template_file.as_ref() }

    /// tag_list を取得
    pub fn get_tag_list(&self) -> Vec<String> { self.tag_list.clone() }

    /// 設定ファイルを読み込み
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;
        
        let mut config: Config = toml::from_str(&contents)
            .with_context(|| "Failed to parse config file")?;
        
        // 欠落フィールドのデフォルト補完（serde default で補完されるが念のため）
        if config.tag_list.is_empty() { config.tag_list = vec!["Tips".to_string()]; }
        
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
    pub fn load_or_default(config_path: PathBuf) -> Result<Self> {

        if config_path.exists() {
            Self::load_from_file(config_path)
        } else {
            let config = Self::default();
            // デフォルト設定を保存
            if let Err(e) = config.save_to_file(config_path) {
                eprintln!("Warning: Failed to save default config: {}", e);
            }
            Ok(config)
        }
    }
}
