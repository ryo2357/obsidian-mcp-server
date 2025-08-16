# 005-02 Rust SDK 置換コア実装 (initialize / tools 基本移行)

作成日時: 2025-08-16 15:50
参照アイディアファイル: `005_プロジェクト構造の変更.md`
依存コーディング計画: `005-01-migrate-to-rust-sdk-setup.md`

## 目的

公式 Rust SDK を用いて `initialize` フローと `tools/list` / `tools/call` のコア動作を移行し、既存ツール (save_markdown_file / get_template_markdown / list_note_tags) を SDK 上で動作させる。旧実装は温存し、環境変数で切替可能にする。

## スコープ

- SdkMcpServer に initialize 実装
- Tool 登録 (既存 3 ツール) の SDK 化
- 旧ツールロジック呼び出し用アダプタ (005-01 stub 拡張)
- 環境変数フラグ `MCP_SDK_ENABLE` による main 起動分岐
- エラーハンドリング: anyhow -> SDK エラー型マッピング
- 基本テスト (initialize -> tools/list -> tools/call 1 サイクル)

## 非スコープ

- 旧コード削除
- 拡張ツールや追加機能
- 高度な並列実行・キャンセルサポート

## 方針

| 項目        | 方針                                                                                |
| ----------- | ----------------------------------------------------------------------------------- |
| 切替方式    | `std::env::var("MCP_SDK_ENABLE").is_ok()` で分岐                                    |
| ログ        | 既存 logger 再利用                                                                  |
| Config 取得 | 旧経路と同一 (差異なし)                                                             |
| Tool 実装   | Adapter で `execute(&self, params: serde_json::Value) -> Result<serde_json::Value>` |
| 非同期      | SDK の async API に合わせ `tokio` 活用                                              |

## 詳細仕様

### SdkMcpServer (暫定)

```
pub struct SdkMcpServer {
  config: Config,
  tools: Vec<Box<dyn SdkToolTrait + Send + Sync>>,
}
```

- new(config) -> Self
- register_builtin_tools() 内で 3 ツール登録
- run_stdio() ラッパを提供 (SDK の実装を呼ぶ)

### Tool Adapter

```
pub trait LegacyToolAdapter {
  fn name(&self) -> &'static str;
  fn description(&self) -> &'static str;
  fn schema(&self) -> serde_json::Value; // JSON Schema
  async fn call(&self, server_ctx: &SdkContext, params: serde_json::Value) -> anyhow::Result<serde_json::Value>;
}
```

→ SDK が要求する trait に対してブリッジ実装

### 既存ツール対応

| ツール                | 追加処理                  | 備考                    |
| --------------------- | ------------------------- | ----------------------- |
| save_markdown_file    | vault_dir の取得 (Config) | 旧 VaultOperations 流用 |
| get_template_markdown | Config.template_file 参照 | エラー分類維持          |
| list_note_tags        | Config.tag_list 参照      | filter 無視             |

### エラー変換

- anyhow::Error -> SDK Error (コード 500 相当 / data に string 化メッセージ)
- Recoverable (テンプレ未設定等) は 200 成功レスポンス内 field で is_error=true (現行踏襲)

## 実装手順

1. `sdk_server/mod.rs` に SdkMcpServer フィールド & new 実装
2. adapter.rs に LegacyToolAdapter と 3 ツール struct 実装
3. 既存コードからロジック抽出 (重複最小化) — 共通関数を `tools/common.rs` (必要なら) に追加
4. main.rs に環境変数分岐追加
5. テスト: 環境変数セット時に SDK パスで initialize/ツール呼び出し成功

## テスト計画

- 単体: 各 Adapter.call の正常/代表的異常
- 結合: end-to-end (env 有 / 無) 2 パス
- 旧パス回帰: env 未設定で以前通り

## リスク

- SDK API 変更 → コンパイル失敗: 早期段階で pin バージョン
- 非同期ランタイム二重起動: main 側で統一 (tokio::main の重複避け)

## 完了条件

- フラグ ON で 3 ツール利用可能
- OFF で旧実装継続
- cargo test 全成功 (新規テスト含む)

## 次フェーズ (005-03)

- 旧実装の段階的削除と仕様書更新反映
