use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;

/// Vault操作に関する共通処理
pub struct VaultOperations {
    vault_path: PathBuf,
    target_directory: String,
}

impl VaultOperations {
    /// 新しいVaultOperationsインスタンスを作成
    pub fn new(vault_path: PathBuf, target_directory: String) -> Self {
        Self {
            vault_path,
            target_directory,
        }
    }

    /// ターゲットディレクトリのフルパスを取得
    pub fn get_target_directory_path(&self) -> PathBuf {
        self.vault_path.join(&self.target_directory)
    }

    /// ファイルパスがvault内にあるかチェック
    pub fn is_path_within_vault(&self, file_path: &Path) -> Result<bool> {
        let canonical_vault = self.vault_path.canonicalize()
            .with_context(|| format!("Failed to canonicalize vault path: {}", self.vault_path.display()))?;
        
        let canonical_file = file_path.canonicalize()
            .or_else(|_| {
                // ファイルが存在しない場合は親ディレクトリをチェック
                if let Some(parent) = file_path.parent() {
                    parent.canonicalize()
                        .map(|parent_canonical| parent_canonical.join(file_path.file_name().unwrap()))
                        .map_err(|e| anyhow::anyhow!("Failed to canonicalize parent directory: {}", e))
                } else {
                    Err(anyhow::anyhow!("Invalid file path"))
                }
            })
            .with_context(|| format!("Failed to canonicalize file path: {}", file_path.display()))?;

        Ok(canonical_file.starts_with(canonical_vault))
    }

    /// ターゲットディレクトリが存在するかチェック
    pub fn target_directory_exists(&self) -> bool {
        let target_path = self.get_target_directory_path();
        target_path.exists() && target_path.is_dir()
    }

    /// ファイル名を検証（危険な文字を排除）
    pub fn validate_filename(filename: &str) -> Result<()> {
        // 空文字チェック
        if filename.is_empty() {
            return Err(anyhow::anyhow!("Filename cannot be empty"));
        }

        // 危険な文字をチェック
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        for ch in invalid_chars {
            if filename.contains(ch) {
                return Err(anyhow::anyhow!("Filename contains invalid character: {}", ch));
            }
        }

        // パストラバーサル攻撃防止
        if filename.contains("..") {
            return Err(anyhow::anyhow!("Filename cannot contain '..'"));
        }

        // システムで予約された名前をチェック（Windows）
        let reserved_names = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", 
                             "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", 
                             "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
        let upper_filename = filename.to_uppercase();
        for reserved in reserved_names {
            if upper_filename == reserved || upper_filename.starts_with(&format!("{}.", reserved)) {
                return Err(anyhow::anyhow!("Filename is reserved: {}", filename));
            }
        }

        Ok(())
    }

    /// Markdownファイルを保存
    pub fn save_markdown_file(&self, filename: &str, content: &str) -> Result<PathBuf> {
        // ファイル名を検証
        Self::validate_filename(filename)?;

        // ターゲットディレクトリが存在するかチェック
        if !self.target_directory_exists() {
            return Err(anyhow::anyhow!(
                "Target directory does not exist: {}", 
                self.get_target_directory_path().display()
            ));
        }

        // .md拡張子を自動付与
        let filename_with_ext = if filename.ends_with(".md") {
            filename.to_string()
        } else {
            format!("{}.md", filename)
        };

        let file_path = self.get_target_directory_path().join(&filename_with_ext);

        // ファイルがvault内にあるかチェック
        if !self.is_path_within_vault(&file_path)? {
            return Err(anyhow::anyhow!(
                "File path is outside vault: {}", 
                file_path.display()
            ));
        }

        // 既存ファイルが存在するかチェック
        if file_path.exists() {
            return Err(anyhow::anyhow!(
                "File already exists: {}", 
                file_path.display()
            ));
        }

        // ファイルを保存
        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;

        Ok(file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_filename() {
        // 正常なケース
        assert!(VaultOperations::validate_filename("test").is_ok());
        assert!(VaultOperations::validate_filename("test-file").is_ok());
        assert!(VaultOperations::validate_filename("test_file").is_ok());

        // 異常なケース
        assert!(VaultOperations::validate_filename("").is_err());
        assert!(VaultOperations::validate_filename("test/file").is_err());
        assert!(VaultOperations::validate_filename("test\\file").is_err());
        assert!(VaultOperations::validate_filename("test:file").is_err());
        assert!(VaultOperations::validate_filename("test*file").is_err());
        assert!(VaultOperations::validate_filename("test?file").is_err());
        assert!(VaultOperations::validate_filename("test\"file").is_err());
        assert!(VaultOperations::validate_filename("test<file").is_err());
        assert!(VaultOperations::validate_filename("test>file").is_err());
        assert!(VaultOperations::validate_filename("test|file").is_err());
        assert!(VaultOperations::validate_filename("../test").is_err());
        assert!(VaultOperations::validate_filename("CON").is_err());
        assert!(VaultOperations::validate_filename("con.txt").is_err());
    }

    #[test]
    fn test_save_markdown_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let vault_path = temp_dir.path().to_path_buf();
        let target_dir = "notes";
        
        // ターゲットディレクトリを作成
        fs::create_dir(vault_path.join(target_dir))?;

        let vault_ops = VaultOperations::new(vault_path.clone(), target_dir.to_string());

        // 正常なケース
        let content = "# Test\n\nThis is a test markdown file.";
        let result = vault_ops.save_markdown_file("test", content)?;
        
        assert_eq!(result, vault_path.join(target_dir).join("test.md"));
        assert!(result.exists());
        
        let saved_content = fs::read_to_string(&result)?;
        assert_eq!(saved_content, content);

        // 既存ファイルがある場合のエラー
        let result = vault_ops.save_markdown_file("test", content);
        assert!(result.is_err());

        Ok(())
    }
}
