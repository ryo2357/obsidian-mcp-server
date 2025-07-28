use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("設定エラー: {0}")]
    Config(#[from] ConfigError),
    #[error("IO エラー: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON パースエラー: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("設定ファイルが見つかりません: {path}")]
    FileNotFound { path: String },
    #[error("設定ファイルの解析に失敗しました: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("無効な設定値: {field}")]
    InvalidValue { field: String },
}
