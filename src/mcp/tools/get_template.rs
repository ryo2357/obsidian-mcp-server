use crate::config::Config;
use crate::vault::VaultOperations;
use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// get_template_markdown ツールの出力
#[derive(Debug, Serialize)]
pub struct GetTemplateMarkdownResult {
    pub template_content: String,
    pub path: String,
    pub message: String,
}

/// テンプレート取得ツール本体
pub fn execute_get_template_markdown(
    config: &Config,
    vault_ops: &VaultOperations,
    _params: Option<Value>,
) -> Result<GetTemplateMarkdownResult> {
    let template_rel = match config.get_template_file() {
        Some(p) => p.clone(),
        None => {
            return Err(anyhow::anyhow!("Template not configured"));
        }
    };

    // vault_dir を取得
    let vault_dir = config.get_vault_dir()?;
    let raw_path = vault_dir.join(&template_rel);

    // 正規化 & vault 内確認
    let canonical = raw_path
        .canonicalize()
        .with_context(|| format!("Template file does not exist: {}", raw_path.display()))?;

    if !vault_ops.is_path_within_vault(&canonical)? {
        return Err(anyhow::anyhow!("Template path is outside vault"));
    }

    // 読み込み (UTF-8)
    let content = fs::read_to_string(&canonical)
        .with_context(|| format!("Failed to read template file: {}", canonical.display()))?;

    Ok(GetTemplateMarkdownResult {
        template_content: content,
        path: template_rel.display().to_string(),
        message: "Template loaded".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::execute_get_template_markdown;
    use crate::config::Config;
    use crate::vault::VaultOperations;
    use anyhow::Result;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_get_template_success() -> Result<()> {
        let tmp = TempDir::new()?;
        let vault_dir = tmp.path().to_path_buf();
        // ディレクトリ & テンプレート作成
        fs::create_dir_all(vault_dir.join("Templates"))?;
        fs::write(vault_dir.join("Templates/daily.md"), "# Daily\n")?;

        let mut cfg = Config::default();
        cfg.set_vault_dir(&vault_dir);
        cfg.set_template_file("Templates/daily.md");
        let vault_ops = VaultOperations::new(vault_dir.clone(), "Tips".into());

        let result = execute_get_template_markdown(&cfg, &vault_ops, None)?;
        assert!(result.template_content.contains("# Daily"));
        Ok(())
    }

    #[test]
    fn test_get_template_not_configured() -> Result<()> {
        let tmp = TempDir::new()?;
        let vault_dir = tmp.path().to_path_buf();
        let mut cfg = Config::default();
        cfg.set_vault_dir(&vault_dir);
        let vault_ops = VaultOperations::new(vault_dir.clone(), "Tips".into());

        let err = execute_get_template_markdown(&cfg, &vault_ops, None).unwrap_err();
        assert!(err.to_string().contains("Template not configured"));
        Ok(())
    }

    #[test]
    fn test_get_template_missing_file() -> Result<()> {
        let tmp = TempDir::new()?;
        let vault_dir = tmp.path().to_path_buf();
        let mut cfg = Config::default();
        cfg.set_vault_dir(&vault_dir);
        cfg.set_template_file("Templates/missing.md");
        let vault_ops = VaultOperations::new(vault_dir.clone(), "Tips".into());

        let err = execute_get_template_markdown(&cfg, &vault_ops, None).unwrap_err();
        assert!(err.to_string().contains("does not exist"));
        Ok(())
    }
}
