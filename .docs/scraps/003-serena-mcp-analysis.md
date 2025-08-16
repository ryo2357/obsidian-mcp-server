# 003 Serena MCP 分析サマリ

作成日時: 2025-08-16 00:00

## 概要

本 scrap は Serena MCP ツールを用いて `obsidian-mcp-server` プロジェクトを初回オンボーディング・静的分析した結果を集約したもの。

### プロジェクト目的

Obsidian 用 MCP (Model Context Protocol) サーバー。JSON-RPC 2.0 経由で以下ツール機能を提供:

- save_markdown_file: 指定 vault 配下 (既定 `Tips`) に Markdown ノート新規保存
- get_template_markdown: 設定ファイルで指定されたテンプレート Markdown を取得
- list_note_tags: 設定タグ候補一覧を返却 (将来 filter 拡張予定)

### 動作モード

- 通常: `--vault-dir` で vault 指定。OS の config ディレクトリ配下にログ。
- デバッグ: `--debug` でローカル `./debug-vault` 自動生成 + サンプルノート生成 + 詳細ログ。
- 同期: `--sync` で同期 I/O 版ループ (テスト用)。
- 非同期: 既定 (tokio) で標準入出力 JSON-RPC 行単位処理。

### 主要構成要素

- `main.rs`: CLI 解析 → Debug/通常設定組立 → `McpServer` 起動 (sync/async)。
- `config.rs`: 設定 (vault_dir, template_file, tag_list) の読み書き・デフォルト補完。
- `debug.rs`: デバッグ用 vault/サンプルノート生成 (`Tips/sample-note.md`)。
- `logger.rs`: flexi_logger によるサイズローテーション (10MB, 5 世代保持)。
- `error.rs`: JSON-RPC エラー表現 `McpError` + `AppResult`。
- `mcp/protocol.rs`: JSON-RPC/MCP メッセージ & Capabilities/Tool 定義。
- `mcp/server.rs`: initialize / tools/list / tools/call 処理、ツール実行ディスパッチ。
- `mcp/tools/*.rs`: 3 種ツール実装 (保存 / テンプレ取得 / タグ一覧)。
- `vault/operations.rs`: 安全なファイル保存 (ファイル名検証, canonicalize による vault 内包チェック, 既存上書き防止)。

### セキュリティ / バリデーション

- ファイル名: 禁止文字, 予約語, `..` 排除。
- パス: canonicalize して vault の prefix であること確認。
- 既存ファイル存在時はエラー。
- テンプレート取得時も vault 内包含検証。

### ロギング

- debug: レベル `debug`, ディレクトリ `./.config/logs`。
- 通常: レベル `info`, OS config (`dirs::config_dir()/obsidian-mcp-server/logs`)。
- 出力形式: detailed_format。サイズ 10MB でローテーション。

### 設定ファイル

- 既定 (存在しなければ生成) `config.toml`。
- デフォルト `tag_list = ["Tips"]`。
- `template_file` 未指定時 get_template_markdown はエラー。

### MCP ツール仕様 (tools/list 出力)

1. save_markdown_file: filename + content 必須。
2. get_template_markdown: 引数なし。
3. list_note_tags: filter (現状無視) 任意。

### テスト状況

- 各モジュール毎にユニットテスト (Config, VaultOperations, 3 ツール, DebugConfig) あり。
- サーバ統合テスト (initialize→tools/list→tools/call) は未整備。

### 改善候補

- ツールディスパッチの汎用化 (テーブル駆動)。
- list_note_tags の filter 実装と部分一致/ケース無視仕様整理。
- エラー返却形式統一 (ツール内部エラーを JSON-RPC error として返すか現行 isError=true 包装継続か方針明文化)。
- テンプレート未設定時のデフォルトテンプレ生成オプション。
- 統合テスト追加で回帰保証強化。

### 推奨次ステップ (軽量)

1. 統合テスト: in-memory パイプ (stdin/stdout) シミュレーションで initialize → save_markdown_file 成功ケース。
2. list_note_tags filter 実装 & テスト。
3. エラーハンドリング方針ドキュメント化。

以上。
