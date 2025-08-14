# 004 メモ転送関連ツール実装計画

作成日時: 2025-08-14 16:00
参照アイディアファイル: `004_メモ転送ツールの実装.md`

## 目的

Obsidian 用メモ作成フローを Copilot + MCP サーバー経由で完結させるために、既存の save_markdown_file ツールに加えて以下 2 つの取得系ツールを追加し、テンプレート適用とタグ選定を支援する。

1. テンプレート取得ツール (get_template_markdown)
2. タグ候補取得ツール (list_note_tags)

## 実装対象機能概要

- Config 拡張
  - template_file: Option<PathBuf>
  - tag_list: Vec<String>
- MCP ツール追加
  - get_template_markdown: テンプレート markdown 文字列を返却
  - list_note_tags: 設定タグ一覧 (将来的フィルタ対応を見据えたインターフェイス)
- tools/list への 2 ツール追加
- tools/call での分岐追加
- エラーハンドリング仕様整備
- テスト (正常系 / 異常系) 追加

## 詳細仕様

### Config 拡張

| フィールド    | 型              | デフォルト | 説明                                                    |
| ------------- | --------------- | ---------- | ------------------------------------------------------- |
| template_file | Option<PathBuf> | None       | vault_dir を起点とした相対パス (例: Templates/daily.md) |
| tag_list      | Vec<String>     | ["Tips"]   | タグ候補一覧                                            |

- 既存 Config のシリアライズ/デシリアライズに自動反映 (Serde) されるようフィールド追加。
- 保存済み旧バージョンの設定ファイル (フィールド欠如) はデフォルト補完され後方互換維持。

### get_template_markdown ツール

| 項目         | 内容                                                                    |
| ------------ | ----------------------------------------------------------------------- |
| name         | get_template_markdown                                                   |
| description  | Return configured markdown template content                             |
| input_schema | 追加パラメータ無し (空オブジェクト)                                     |
| 出力         | { template_content: String, path: String, message: String }             |
| エラー条件   | template_file 未設定 / ファイル不存在 / 読込失敗 / vault 外アクセス検出 |

挙動:

1. Config.template_file が None -> message に Template not configured を含むエラー結果 (tools/call 成功レスポンス内 is_error = true) を返す。
2. Path 解決: resolved = vault_dir.join(template_file) -> 正規化後 vault 内判定。
3. ファイル読み込み (UTF-8 想定)。
4. 結果返却。

セキュリティ:

- パス正規化後に vault_dir プレフィックス検証 (既存 VaultOperations の is_path_within_vault 相当ロジックを再利用 / もしくは共通化)。

### list_note_tags ツール

| 項目                | 内容                                                           |
| ------------------- | -------------------------------------------------------------- |
| name                | list_note_tags                                                 |
| description         | Return configured tag candidates for note classification       |
| input_schema        | 将来拡張のため { filter?: string } (任意)                      |
| 出力                | { tags: Vec<String>, filtered: bool, message: String }         |
| フィルタ仕様 (初期) | filter 未実装 (与えられても無視、将来拡張で前方一致や部分一致) |

エラー条件なし (空タグでも成功)。

### 既存 save_markdown_file との連携

Copilot 側フロー:

1. get_template_markdown 呼び出し -> テンプレート本文取得 (無ければ簡易新規テンプレート生成)
2. list_note_tags で候補取得 -> 入力ノート内容から類似・関連タグ選択 (類似計算は Copilot 側で実施)
3. 必要メタ情報 (日付, タイトル, タグ) をテンプレートへ挿入
4. save_markdown_file 実行で確定保存

### エラーポリシー (tools/call wrapper 共通)

- 成功: is_error=false
- 想定内欠如 (テンプレ未設定等): is_error=true だがコード側は recoverable メッセージ
- 予期せぬ IO / パース: is_error=true

### tools/list 追加エントリ

get_template_markdown:

```
{
  "type": "object",
  "properties": {},
  "required": []
}
```

list_note_tags:

```
{
  "type": "object",
  "properties": {
    "filter": {"type": "string", "description": "Optional substring to filter tags (future use)"}
  },
  "required": []
}
```

## 技術的実装方針

1. Config フィールド追加
   - struct 拡張
   - Default 実装: template_file=None, tag_list=vec!["Tips"]
   - 既存メソッド変更不要 (将来 getter が必要なら追加)
2. 新ツールモジュール追加
   - src/mcp/tools/get_template.rs (仮) と src/mcp/tools/list_tags.rs
   - mod.rs で公開 + execute\_\* 関数エクスポート
3. McpServer 拡張
   - tools/list で新ツール 2 件追加
   - handle_call_tool にマッチ分岐追加
   - 専用ラッパーメソッド execute_get_template / execute_list_tags 実装
4. VaultOperations 既存関数の再利用
   - パス検証 (必要なら pub fn に追加 or ユーティリティ抽出) ただし現状 is_path_within_vault は pub ならそのまま利用
5. 依存追加不要 (現行標準 + anyhow / serde で十分)
6. テスト
   - get_template_markdown: 正常 (存在), 未設定, 存在しないファイル
   - list_note_tags: デフォルト, カスタム (複数タグ), filter パラメータ無視確認
   - tools/list: 新ツール名が含まれる
7. ドキュメント更新 (実装後 specs/project.md の MCP Tools セクションに 2 ツール追記)

## テスト戦略

- 単体テスト: 各 execute\_\* 関数を直接呼び出し JSON パラメータ検証
- 結合テスト (軽量): McpServer 生成 -> initialize -> tools/list -> tools/call でシナリオ確認
- エッジケース: 長いタグ, 空ファイル, 文字コード (UTF-8 以外は非対応明記)

## 想定する将来拡張

- list_note_tags の filter 実装 (大小文字非依存, Fuzzy)
- テンプレート複数管理 (template_key パラメータ)
- メタデータ抽出/挿入専用ツール

## Copilot 用プロンプト案

テンプレ取得 → 編集 → 保存の流れを誘導するためのプロンプト例。

1. テンプレ取得

```
MCP ツール get_template_markdown を実行しテンプレート本文を取得してください。テンプレートが無い場合は Empty template と出力してください。
```

2. タグ候補取得と選定

```
MCP ツール list_note_tags を実行し取得した tags から下記本文に最も関連するタグ上位 3 件を選び YAML front matter 用配列にしてください。本文: <ここに本文>
```

3. テンプレート適用

```
以下テンプレートに日時と選定タグを挿入し完成した Markdown を出力。必要ならタイトルを推測。テンプレート:
<取得したテンプレート>
```

4. 保存

```
完成テキストを基に安全なファイル名 (日付-短い英語スラッグ) を決め、拡張子抜き filename と content を save_markdown_file に渡す JSON を生成してください。
```

## 分割の要否

本件は影響範囲が限定的 (Config 追加 + ツール 2 件 + サーバー分岐) のため単一プランで実装可能と判断。

## 実装手順サマリ

1. Config 拡張 (デフォルト含む) + コンパイル確認
2. ツール 2 ファイル追加 (ロジック + テスト)
3. tools/mod.rs 追記
4. server.rs に list/call 分岐追記 + 実行関数
5. 単体テスト追加/実行
6. 結合テスト (tools/list 確認)
7. 仕様書更新 (別ステップ)

## リスクと対策

- 既存設定ファイルとの互換性: Serde のフィールド追加は後方互換 OK -> Default + Option で吸収
- パス検証抜け: 既存検証ロジックの再利用を徹底し重複実装回避
- 文字コード問題: UTF-8 前提を仕様書に明記 (非 UTF-8 はエラー)

## 完了条件

- cargo test 全件成功
- tools/list に 3 ツール (save_markdown_file, get_template_markdown, list_note_tags) 表示
- get_template_markdown / list_note_tags 正常・異常ケーステスト合格
- 新フィールド含む Config シリアライズ/デシリアライズ動作確認
