# 005 push_markdown ツール実装計画

作成日時: 2025-08-20 16:00
参照アイディアファイル: `005_メモ転送機能の実装.md`

## 目的

クライアント (Copilot など) からファイル名と Markdown 本文を受け取り、Obsidian Vault 内の設定出力ディレクトリに安全に保存する `push_markdown` MCP ツールを追加する。既存 `VaultOperations::save_markdown_file` を公開ツール化し、メモ転送フローを完結させる。

## 実装対象機能概要

- MCP ツール `push_markdown` の追加
  - 入力: `filename` (String), `content` (String)
  - 出力: 成功時 保存パス / メッセージ
  - 失敗時: バリデーション / 既存重複 / ディレクトリ不存在 等をエラーとして返却
- 既存 `ObsidianServer` へのツール追加 (tools/list, tools/call)
- エラーを `McpError` (適切な ErrorCode) で返却
- 仕様書 (`.docs/specs/project.md`) は実装完了後に別ステップで更新予定

## 詳細仕様

### ツール: push_markdown

| 項目           | 内容                                                                                      |
| -------------- | ----------------------------------------------------------------------------------------- |
| name           | push_markdown                                                                             |
| description    | Save markdown file into configured vault output directory                                 |
| 入力スキーマ   | { filename: string, content: string }                                                     |
| 前提条件       | Config.vault_dir 設定済み / output_dir 存在 (存在しない場合はエラー)                      |
| ファイル名仕様 | 拡張子不要 (自動で .md 付与) / 既に .md ならそのまま / 既存ファイル不可                   |
| バリデーション | `VaultOperations::validate_filename` に委譲 (危険文字 / .. / 予約語)                      |
| 出力成功       | `[{ "type": "text", "text": <JSON文字列> }]` ; JSON 例: {"status":"ok","message":"Saved"} |
| 出力失敗       | `CallToolResult` 内 is_error=true (McpError)                                              |

出力では保存先パスは返却しない (クライアント側でパスが必要な場合は将来オプション化を検討)。

### エラーポリシー

| ケース                    | 返却メッセージ例                       | ErrorCode      |
| ------------------------- | -------------------------------------- | -------------- |
| filename 空 / 無効        | "Invalid filename: ..."                | INTERNAL_ERROR |
| 既存ファイルあり          | "File already exists: ..."             | INTERNAL_ERROR |
| 出力ディレクトリ不存在    | "Target directory does not exist: ..." | INTERNAL_ERROR |
| Vault 外パス検出 (理論上) | "File path is outside vault"           | INTERNAL_ERROR |
| 書込失敗                  | "Failed to write file: ..."            | INTERNAL_ERROR |

(現行設計では `rmcp` の標準コード `INTERNAL_ERROR` を暫定使用。将来 granular なコードに拡張可。)

### セキュリティ / 安全性

- パストラバーサル防止: 既存 `validate_filename` と `is_path_within_vault` による二重防御
- 上書き防止: 既存ファイル存在時はエラー
- 非同期アクセス: 単純保存のためロック競合リスク低 (同名同時保存は片方失敗)

### ロギング

- 成功時: INFO レベルで `Saved markdown file: <path>`
- 失敗時: エラー内容を WARN/ERROR (既存 logger 方針に合わせ ERROR) で出力

### (参考) 想定動作確認観点 (今回は自動テスト未実装)

1. 正常系: 有効 filename / content で保存成功しステータス ok
2. 同名再保存でエラー (上書き防止)
3. 無効ファイル名でエラーメッセージ
4. 出力ディレクトリ不存在時のエラー
5. .md 付き filename で拡張子重複しない

### Copilot 利用想定フロー (参考)

1. (任意) テンプレ取得 -> 編集
2. push_markdown 呼出 JSON: {"filename":"2025-08-20-daily","content":"..."}
3. 成功レスポンスの status / message を確認し保存成功を判断

### 技術的実装方針

1. `server.rs` 内 `impl ObsidianServer` に `#[tool(description = "VaultにMarkdownファイルを保存する")] async fn push_markdown(&self, filename: String, content: String)` を追加
   - vault ロック -> `save_markdown_file` 呼出
   - 成功: JSON を文字列化 (`serde_json::json!`) し `Content::text` で返却
   - 失敗: `McpError::new(ErrorCode::INTERNAL_ERROR, msg, None)`
2. ログ出力 (log::info / log::error)
3. `Cargo.toml` の dev-dependencies 変更は不要 (自動テストを追加しないため)
4. 実装後 `cargo check` / `cargo build` でコンパイル確認 (動作検証は手動で実施)

### 後方互換性

- 新規ツール追加のみで既存ツールとの衝突なし
- Config 構造変更なし

### リスクと軽減策

| リスク                                        | 対策                                                     |
| --------------------------------------------- | -------------------------------------------------------- |
| rmcp ツールマクロの引数シリアライズ仕様不一致 | ビルドエラー内容を確認し、必要なら単一 struct 引数に変更 |
| 手動検証漏れ (自動テスト欠如)                 | 参考観点リストを用いて手動チェック                       |
| Windows パス扱い差異                          | 既存 `VaultOperations` を流用し OS 依存ロジック追加不要  |

### 完了条件

- `cargo check` / `cargo build` が成功
- tools/list に `push_markdown` が表示
- 手動検証で保存成功と代表的エラー (重複 / 無効名) が再現可能

## 実装手順サマリ

1. `push_markdown` ツール関数追加
2. ログ出力実装
3. ビルド (cargo check / build)
4. (次ステップ) 仕様書更新 / 報告書作成
