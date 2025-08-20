
#![allow(dead_code)]
use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::ErrorCode,
    handler::server::{router::tool::ToolRouter},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
// use serde_json::json;
use tokio::sync::Mutex;
use log::info;

use crate::{Config};
use crate::vault::VaultOperations;


/// カウンター機能の本体。カウンター値とツールルーターを保持する。
#[derive(Clone)]
pub struct ObsidianServer {
    // 参照する設定ファイル
    config: Arc<Mutex<Config>>,
    vault: Arc<Mutex<VaultOperations>>,
    /// MCPツールルーター。ツール呼び出しのエントリポイント。
    tool_router: ToolRouter<ObsidianServer>,
}

#[tool_router]
impl ObsidianServer {

    #[allow(dead_code)]
    pub fn new(config: Config) -> Self {

        let vault= VaultOperations::new(
            config.get_vault_dir().expect("error: config::get_vault_dir()"),
            config.get_output_dir(),
          );
          
        Self {
            config: Arc::new(Mutex::new(config)),
            vault: Arc::new(Mutex::new(vault)),
            tool_router: Self::tool_router(),
        }
    }



    // タグリストを取得するツール
    #[tool(description = "Vaultのタグリストを取得する")]
    async fn get_tags(&self) -> Result<CallToolResult, McpError> {
        let tags:Vec<String> = self.config.lock().await.get_tag_list();

        let contents: Vec<Content> = tags.into_iter()
            .map(Content::text)
            .collect();

        Ok(CallToolResult::success(contents))
    }

    // テンプレートファイルを取得するツール
    #[tool(description = "Vaultのテンプレートファイルを取得する")]
    async fn get_template(&self) -> Result<CallToolResult, McpError> {
        let template_rel_path = {
            let cfg = self.config.lock().await;
            cfg.get_template_file()
                .ok_or_else(|| McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    "テンプレートファイルが設定されていません",
                    None,
                ))?
        };

        let content = {
            let vault = self.vault.lock().await;
            vault.read_text_file(template_rel_path).map_err(|e| McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("テンプレート読込に失敗: {e}"),
                None,
            ))?
        };

        let contents = vec![
            Content::text(content),
        ];

        Ok(CallToolResult::success(contents))
    }

    // markdownファイルをVaultに追加するツール

}

/// MCPサーバーハンドラの実装。各種APIエンドポイントを提供する。
#[tool_handler]
impl ServerHandler for ObsidianServer {
    /// サーバー情報を返す。プロトコルバージョンや機能説明などを含む。
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            // instructions: Some("このサーバーはカウンターツールを提供します。カウンター値は'increment'と'decrement'ツールで変更でき、'get_value'で現在値を取得できます。初期値は0です。".to_string()),
            instructions: Some(concat!(
              "このサーバーはObsidianのVaultを操作するツールを提供します。",
              "'get_tags'と'get_template'ツールでVaultの情報値を取得し、",
              "'push_markdown'でVaultにMarkdownファイルを追加できます。",
            ).to_string())
        }
    }



    /// サーバー初期化処理。HTTPリクエスト情報をログ出力。
    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            // initialize_headers: HTTPリクエストのヘッダー情報（axum::http::HeaderMap型）
            // 認証情報やUser-Agentなど
            let initialize_headers = &http_request_part.headers;
            // initialize_uri: HTTPリクエストのURI（axum::http::Uri型）
            // リクエストのパスやクエリ部分 
            let initialize_uri = &http_request_part.uri;
            info!("initialize from http server headers: {:?}", initialize_headers);
            info!("initialize from http server uri: {:?}", initialize_uri);
        }
        Ok(self.get_info())
    }
}