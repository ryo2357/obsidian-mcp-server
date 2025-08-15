use crate::config::Config;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct ListTagsParams {
    #[allow(dead_code)]
    filter: Option<String>, // 現時点では未使用
}

#[derive(Debug, Serialize)]
pub struct ListNoteTagsResult {
    pub tags: Vec<String>,
    pub filtered: bool,
    pub message: String,
}

pub fn execute_list_note_tags(
    config: &Config,
    params: Option<Value>,
) -> anyhow::Result<ListNoteTagsResult> {
    // 将来フィルタ対応用に受け取るが現在は無視
    let _parsed: Option<ListTagsParams> = match params {
        Some(v) => serde_json::from_value(v).ok(),
        None => None,
    };

    Ok(ListNoteTagsResult {
        tags: config.get_tag_list().to_vec(),
        filtered: false,
        message: "Tag list returned".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::execute_list_note_tags;
    use crate::config::Config;
    use anyhow::Result;
    use serde_json::json;

    #[test]
    fn test_list_tags_default() -> Result<()> {
        let mut cfg = Config::default();
        cfg.set_tag_list(vec!["Tips".into(), "Diary".into()]);
        let result = execute_list_note_tags(&cfg, None)?;
        assert_eq!(result.tags.len(), 2);
        Ok(())
    }

    #[test]
    fn test_list_tags_with_filter_param_ignored() -> Result<()> {
        let cfg = Config::default();
        let result = execute_list_note_tags(&cfg, Some(json!({"filter":"test"})))?;
        assert!(!result.filtered);
        Ok(())
    }
}
