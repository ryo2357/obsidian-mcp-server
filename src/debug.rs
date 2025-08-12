use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// デバッグ用の設定
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// デバッグ用 vault のパス（固定）
    pub vault_path: PathBuf,
    /// デバッグモードが有効かどうか
    pub enabled: bool,
}

impl DebugConfig {
    /// デバッグ設定を作成
    pub fn new() -> Self {
        Self {
            vault_path: PathBuf::from("./debug-vault"),
            enabled: true,
        }
    }

    /// デバッグ用 vault ディレクトリを作成
    pub fn ensure_debug_vault(&self) -> Result<()> {
        if !self.vault_path.exists() {
            fs::create_dir_all(&self.vault_path)
                .with_context(|| format!("Failed to create debug vault directory: {}", self.vault_path.display()))?;
            
            println!("Created debug vault directory: {}", self.vault_path.display());
        }

        // ターゲットディレクトリも作成
        let target_dir = self.vault_path.join("Tips");
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)
                .with_context(|| format!("Failed to create debug target directory: {}", target_dir.display()))?;
            
            println!("Created debug target directory: {}", target_dir.display());
        }

        Ok(())
    }

    /// デバッグ用ダミーデータを生成
    pub fn generate_dummy_data(&self) -> Result<()> {
        let sample_file = self.vault_path.join("Tips").join("sample-note.md");
        
        if !sample_file.exists() {
            let sample_content = r#"# サンプルノート

これはデバッグ用に自動生成されたサンプルノートです。

## 特徴

- デバッグモードで自動作成
- テスト用のMarkdownコンテンツ
- Obsidian MCPサーバーのテスト用

## テスト内容

- **太字テキスト**
- *斜体テキスト*
- `コードブロック`

### リスト

1. 番号付きリスト
2. 第二項目
3. 第三項目

- 箇条書き
- 別の項目
- 最後の項目

### コードブロック

```rust
fn main() {
    println!("Hello, Debug World!");
}
```

### リンク

[Obsidian](https://obsidian.md)

---

作成日時: {timestamp}
"#;
            
            let content = sample_content.replace("{timestamp}", &chrono::Utc::now().to_rfc3339());
            
            fs::write(&sample_file, content)
                .with_context(|| format!("Failed to create sample file: {}", sample_file.display()))?;
            
            println!("Created sample debug file: {}", sample_file.display());
        }

        Ok(())
    }

    /// デバッグ用ログを出力
    pub fn debug_log(&self, message: &str) {
        if self.enabled {
            println!("[DEBUG] {}", message);
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_debug_config_creation() {
        let config = DebugConfig::new();
        assert_eq!(config.vault_path, PathBuf::from("./debug-vault"));
        assert!(config.enabled);
    }

    #[test]
    fn test_ensure_debug_vault() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let vault_path = temp_dir.path().join("test-debug-vault");
        
        let mut config = DebugConfig::new();
        config.vault_path = vault_path.clone();
        
        config.ensure_debug_vault()?;
        
        assert!(vault_path.exists());
        assert!(vault_path.join("Tips").exists());
        
        Ok(())
    }

    #[test]
    fn test_generate_dummy_data() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let vault_path = temp_dir.path().join("test-debug-vault");
        
        let mut config = DebugConfig::new();
        config.vault_path = vault_path.clone();
        
        config.ensure_debug_vault()?;
        config.generate_dummy_data()?;
        
        let sample_file = vault_path.join("Tips").join("sample-note.md");
        assert!(sample_file.exists());
        
        let content = fs::read_to_string(&sample_file)?;
        assert!(content.contains("# サンプルノート"));
        assert!(content.contains("デバッグ用に自動生成"));
        
        Ok(())
    }
}
