# 実装報告書: rmcp ベース構造への移行（部分）

作成日時: 2025-08-20 12:35
参照コーディング計画: （手動実装 / 計画ファイルなし）

## 実装概要

直近コミット群で以下を実施:

1. 旧 `src/mcp/` ディレクトリ（自前 JSON-RPC + 手動ツール実装）を削除 / 段階的撤去開始。
2. `rmcp` クレートを用いた新サーバー `ObsidianServer` を `src/server.rs` に実装。
3. 最小ツール `get_tags` を `tool_router` マクロで登録し `Config.tag_list` を返却。
4. CLI を簡素化し `--debug` フラグのみを残し、設定/パス指定系オプションを除去。
5. `main.rs` を刷新し rmcp サービス (`serve(stdio())`) 起動フローへ変更。
6. 旧 Vault モジュールのうち共通ユーティリティを `vault.rs` に統合（`VaultOperations`）。
7. 仕様書を現行構造へ全面更新（旧ツール群 -> 再導入計画セクション追加 / 移行ステータステーブル追加）。

## 成果物

- 新規: `src/server.rs` （rmcp サーバー + get_tags ツール）
- 更新: `src/main.rs` （CLI / 起動ロジック）
- 更新: `src/config.rs` （構造は維持、利用方法変更）
- 更新: `src/vault.rs` （operations の統合）
- 削除: 旧 `src/mcp/` 配下ファイル一式
- 更新: 仕様書 `.docs/specs/project.md`
- 新規: 本報告書

## 仕様書との差異と理由

| 項目           | 旧仕様書                                 | 実装結果                | 差異理由                                |
| -------------- | ---------------------------------------- | ----------------------- | --------------------------------------- |
| CLI オプション | `--config --vault-dir --sync --debug`    | `--debug` のみ          | 設定自動化と最小 MVP 化のため段階的削減 |
| MCP ツール     | save_markdown / get_template / list_tags | get_tags のみ           | rmcp 移行初期段階。段階導入方針         |
| 通信層説明     | JSON-RPC 手動処理                        | rmcp サービス           | フレームワーク移行                      |
| Vault 操作     | operations.rs 下構造                     | `vault.rs` 単一ファイル | 階層簡素化（モジュール削減）            |

## 今後の課題 / 次ステップ

1. `push_markdown` ツール再実装（VaultOperations をラップ）。
2. `get_template` ツール再実装（パス検証 + 読込 + Content 化）。
3. `get_tags` にフィルタ引数（前方一致 / 部分一致）追加。
4. エラー型の統一（anyhow -> domain 変換ヘルパ）。
5. Vault 検索機能（ファイル列挙 / grep ライク検索）。
6. 設定ホットリロード（ファイル更新監視）。

## 備考

- セキュリティ（パストラバーサル / 予約語検証）ロジックは `VaultOperations` に継承済み。
- ロギングは既存設計を流用し、新サーバーでも標準入出力を占有せず安全。
