# 001 Obsidian テンプレート機能の MCP 実装検討

作成日時: 2025-08-12 21:00

## 概要

Obsidian のテンプレートファイル（\_frontmatter.md）を MCP サーバーから読み込み、GitHub Copilot と連携してメタデータ（特にタグ）を自動設定し、新しい Markdown メモを作成する機能の実装について検討した。

## 検討内容

### 現状分析

- 既存の obsidian-mcp-server プロジェクトには Markdown ファイル保存機能（save_markdown_file）が実装済み
- Obsidian テンプレート（\_frontmatter.md）には Templater 形式のプレースホルダーが含まれている
  - `<% tp.file.creation_date("YYYYMMDDTHHmmssSS") %>` 形式
  - id、created、updated フィールドに日時情報を自動挿入

### 実装方針の検討

#### GitHub Copilot での実装可能性

結論: 完全に実装可能

現在のプロジェクト構造を活かして以下の機能を追加することで実現可能:

1. テンプレート読み込み機能
2. タグサジェスト機能
3. テンプレート適用機能

#### タグサジェストの最適なアプローチ

ユーザー提案の「タグ配列ハードコーディング + GitHub Copilot 選択」方式を採用:

1. 事前定義タグ配列を MCP ツールで管理
2. カテゴリ別分類（技術系、作業系、プロジェクト系、状態系）
3. GitHub Copilot による関連性判定と選択
4. 選択理由の説明も含めて出力

#### 推奨プロンプト設計

```json
{
  "context": {
    "template": "[テンプレート内容]",
    "available_tags": ["rust", "programming", "idea", "todo", "project"],
    "note_description": "[ユーザーの説明]"
  },
  "task": "メモ作成のためのメタデータ生成",
  "output": {
    "selected_tags": ["関連性の高いタグ3-5個"],
    "title": "適切なタイトル",
    "reasoning": "選択理由"
  }
}
```

### 技術実装計画

#### 新規 MCP ツール

1. read_template - テンプレートファイル読み込み
2. get_predefined_tags - 事前定義タグリスト取得
3. create_note_from_template - テンプレート適用メモ作成

#### テンプレート処理エンジン

- Templater 形式プレースホルダーの解析と置換
- 日時フォーマット対応（YYYYMMDDTHHmmssSS、YYYY-MM-DDTHH:mm:ss）
- ユニーク ID 生成機能

#### タグ管理システム

カテゴリ別事前定義タグ:

- 技術系: rust, programming, mcp, obsidian
- 作業系: todo, idea, meeting, note
- プロジェクト系: project, development, documentation
- 状態系: draft, review, complete

#### セキュリティ考慮

- テンプレートファイルパス検証
- vault 外アクセス制限
- プレースホルダー処理の安全性確保

### MCP ツールチェーン設計

1. `get_predefined_tags` でタグリストを取得
2. `read_template` でテンプレートを読み込み
3. GitHub Copilot にタグ選択とメタデータ生成を依頼
4. `create_note_from_template` で最終的なメモを作成

### 期待される効果

- Obsidian での一貫したメモフォーマット
- 適切なタグ付けによる検索性向上
- テンプレート活用による作業効率化
- GitHub Copilot との連携による知的なメタデータ生成

## 結論

提案された機能は技術的に実装可能であり、既存の MCP サーバー基盤を活用して効率的に開発できる。特にタグサジェスト機能における AI 活用により、手動作業の大幅な削減と品質向上が期待できる。
