use anyhow::{Context, Result};
use std::path::{Path, PathBuf, Component};
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


    fn resolve_relative_in_vault(&self, relative:PathBuf) -> Result<PathBuf> {

        if relative.as_os_str().is_empty() {
            return Err(anyhow::anyhow!("Empty relative path"));
        }
        if relative.is_absolute() {
            return Err(anyhow::anyhow!("Absolute path is not allowed: {}", relative.display()));
        }

        // ベースは canonicalize しておく (シンボリックリンクを固定化)
        let base = self.vault_path.canonicalize()
            .with_context(|| format!("Failed to canonicalize vault path: {}", self.vault_path.display()))?;

        let mut path = base.clone();

        for comp in relative.components() {
            match comp {
                Component::CurDir => { /* skip */ }
                Component::Normal(seg) => path.push(seg),
                Component::ParentDir => {
                    // vault の外に出そうならエラー
                    if path == base {
                        return Err(
                          anyhow::anyhow!("Path escapes vault: {}", relative.display())
                        );
                    }
                    path.pop();
                }
                // 想定外 (RootDir, Prefix など) は相対パスとして不正
                other => {
                    return Err(
                      anyhow::anyhow!("Invalid component {:?} in relative path: {}", other, relative.display())
                    );
                }
            }
        }

        // 追加の防御 (万一) : starts_with チェック
        if !path.starts_with(&base) {
            return Err(anyhow::anyhow!("Path escapes vault: {}", relative.display()));
        }

        Ok(path)
    }

    // 既存ファイルパスを取得（存在必須）
    fn get_existing_file_path(&self, relative: PathBuf) -> Result<PathBuf> {
        let path = self.resolve_relative_in_vault(relative)?;
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", path.display()));
        }
        if !path.is_file() {
            return Err(anyhow::anyhow!("Not a file: {}", path.display()));
        }
        Ok(path)
    }

    // テキストファイルを読み込み (UTF-8 想定)  内容 を返す
    pub fn read_text_file(&self, relative: PathBuf) -> Result<String> {
        let file_path = self.get_existing_file_path(relative.clone())?;
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read file: {:?}", relative))?;
        Ok(content)
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
