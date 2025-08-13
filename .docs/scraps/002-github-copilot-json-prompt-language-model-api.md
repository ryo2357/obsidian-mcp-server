# 002 GitHub Copilot JSON プロンプトと Language Model API 活用方法

作成日時: 2025-08-13 21:00

## 概要

GitHub Copilot での JSON 形式プロンプトの活用方法と、Language Model API のアクセス方法について検討した。Obsidian テンプレート機能の MCP 実装において、構造化プロンプトによるタグ生成の最適な実装方針を明確化した。

## 検討内容

### JSON 形式プロンプトの活用可能性

#### GitHub Copilot チャットでの利用

- JSON 形式のプロンプトは直接入力・送信可能
- 構造化されたプロンプトにより明確な指示とコンテキストを提供
- 期待する出力形式を明示的に指定できる

#### 推奨 JSON 構造

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

### Language Model API の実装方式

#### 1. VS Code 拡張機能からのアクセス

**特徴**: VS Code 内で動作する拡張機能が GitHub Copilot の Language Model API を使用

**実装例**:

```typescript
const models = await vscode.lm.selectChatModels({
  vendor: "copilot",
  family: "gpt-3.5-turbo",
});

const request = vscode.LanguageModelChatRequest.create(
  [vscode.LanguageModelChatMessage.User("JSONプロンプトを処理してください")],
  {}
);

const response = await model.sendRequest(request, {}, token);
```

#### 2. MCP サーバー側からのアクセス

**特徴**: MCP サーバー（独立したプロセス）から OpenAI API 等にアクセス

**実装例**:

```rust
async fn generate_tags_with_llm(
    template_content: &str,
    description: &str,
    available_tags: &[String],
) -> Result<serde_json::Value, Box<dyn std::error::Error>>
```

### 実装方針の比較

#### 選択肢 1: MCP サーバー側で LLM API 呼び出し

- **メリット**: 完全に自動化、独立性が高い
- **デメリット**: OpenAI API キーが必要、追加コスト

#### 選択肢 2: VS Code 側で Language Model API 使用

- **メリット**: GitHub Copilot の機能を活用、追加コスト不要
- **デメリット**: VS Code 環境に依存

#### 選択肢 3: ハイブリッド方式（推奨）

1. MCP サーバーはデータ処理のみ（テンプレート読み込み、タグリスト提供）
2. VS Code 側で GitHub Copilot の Language Model API を使用してタグ選択
3. 結果を MCP サーバーに送信してファイル作成

### スラッシュコマンドでの活用方法

現在のプロンプトファイル形式を拡張して、JSON 構造を組み込むことが可能：

````markdown
---
mode: "generate"
description: "テンプレートベースのタグ生成"
---

以下の JSON プロンプト形式でタグ選択を行ってください：

```json
{
  "context": {
    "template": "[選択されたテンプレート内容]",
    "available_tags": ["rust", "programming", "mcp", "obsidian"],
    "note_description": "[ユーザーの説明]"
  }
}
```
````

要件:

- 選択されたコンテキストに基づいて適切なタグを選択
- JSON 形式で結果を出力
- 選択理由を必ず含める

```markdown
## 結論

- GitHub Copilot チャットでは JSON 形式プロンプトが効果的に使用可能
- Language Model API は VS Code 拡張機能として実装するのが最適
- ハイブリッド方式により、既存の GitHub Copilot 環境を活用しながら MCP サーバーの機能拡張が可能
- 構造化プロンプトにより一貫性のある高品質なタグ選択とメタデータ生成が実現可能

## 次のステップ

- VS Code 拡張機能開発による Language Model API 活用の実装検討
- 既存 MCP サーバーとの連携方式の詳細設計
- JSON 構造化プロンプトを活用したタグ生成機能の実装
```
