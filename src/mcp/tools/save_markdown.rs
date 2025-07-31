use crate::vault::VaultOperations;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const TARGET_DIRECTORY: &str = "Tips";

/// save_markdown_file ツールの入力パラメータ
#[derive(Debug, Deserialize)]
pub struct SaveMarkdownFileParams {
    /// ファイル名（.md拡張子は自動付与）
    pub filename: String,
    /// Markdownコンテンツ
    pub content: String,
}

/// save_markdown_file ツールの出力結果
#[derive(Debug, Serialize)]
pub struct SaveMarkdownFileResult {
    /// 保存されたファイルのパス
    pub file_path: String,
    /// 成功メッセージ
    pub message: String,
}

/// save_markdown_file ツールの実装
pub fn execute_save_markdown_file(
    vault_ops: &VaultOperations,
    params: Value,
) -> Result<SaveMarkdownFileResult> {
    // パラメータをデシリアライズ
    let params: SaveMarkdownFileParams = serde_json::from_value(params)
        .with_context(|| "Invalid parameters for save_markdown_file")?;

    // ファイルを保存
    let file_path = vault_ops.save_markdown_file(&params.filename, &params.content)
        .with_context(|| "Failed to save markdown file")?;

    Ok(SaveMarkdownFileResult {
        file_path: file_path.display().to_string(),
        message: format!("Successfully saved markdown file: {}", params.filename),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::VaultOperations;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_execute_save_markdown_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let vault_path = temp_dir.path().to_path_buf();
        let target_dir = "notes";
        
        // ターゲットディレクトリを作成
        fs::create_dir(vault_path.join(target_dir))?;

        let vault_ops = VaultOperations::new(vault_path.clone(), target_dir.to_string());

        // 正常なケースをテスト
        let params = json!({
            "filename": "test-note",
            "content": "# Test Note\n\nThis is a test."
        });

        let result = execute_save_markdown_file(&vault_ops, params)?;
        
        assert!(result.file_path.contains("test-note.md"));
        assert!(result.message.contains("Successfully saved"));

        // ファイルが実際に作成されているかチェック
        let expected_path = vault_path.join(target_dir).join("test-note.md");
        assert!(expected_path.exists());

        Ok(())
    }

    #[test]
    fn test_execute_save_markdown_file_invalid_params() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().to_path_buf();
        let target_dir = "notes";
        
        fs::create_dir(vault_path.join(target_dir)).unwrap();
        let vault_ops = VaultOperations::new(vault_path, target_dir.to_string());

        // 無効なパラメータ
        let invalid_params = json!({
            "invalid_field": "value"
        });

        let result = execute_save_markdown_file(&vault_ops, invalid_params);
        assert!(result.is_err());
    }
}
