# 001-02 Obsidian vault 操作ツールの実装

作成日時: 2025-07-29 14:30
参照アイディアファイル: .docs/ideas/001\_プロジェクトの概要.md

## 概要

Obsidian vault 内の指定フォルダに markdown ファイルを保存する機能を実装します。GitHub Copilot からのコンテキストを受け取り、適切に加工して markdown ファイルとして保存する機能を提供します。

## 前提条件

- 001-01 のプロジェクト基盤構築が完了していること
- MCP サーバーの基本機能が動作していること
- ツール登録フレームワークが整備されていること

## 実装内容

### 1. Obsidian vault 操作ツールの実装

#### ファイル構造

```text
src/tools/
├── mod.rs
├── registry.rs          # 既存
├── obsidian/
│   ├── mod.rs
│   ├── save_markdown.rs  # markdownファイル保存ツール
│   └── vault_utils.rs    # vault操作ユーティリティ
```

#### SaveMarkdownTool の実装

```rust
pub struct SaveMarkdownTool {
    vault_path: PathBuf,
}

// 機能:
// - 指定されたフォルダパスにmarkdownファイルを保存
// - ファイル名の自動生成（日時ベース or 指定）
// - 既存ファイルの上書き確認
// - vault内での適切なパス検証
```

### 2. ツール仕様の詳細設計

#### 入力パラメータ

```json
{
  "type": "object",
  "properties": {
    "folder_path": {
      "type": "string",
      "description": "vault内の保存先フォルダパス"
    },
    "filename": {
      "type": "string",
      "description": "ファイル名（拡張子なし、省略時は自動生成）"
    },
    "content": {
      "type": "string",
      "description": "保存するmarkdownコンテンツ"
    },
    "overwrite": {
      "type": "boolean",
      "description": "既存ファイルの上書き許可",
      "default": false
    }
  },
  "required": ["folder_path", "content"]
}
```

#### 出力形式

```json
{
  "success": true,
  "file_path": "vault_path/folder/filename.md",
  "message": "ファイルが正常に保存されました"
}
```

### 3. vault 操作ユーティリティの実装

#### VaultValidator

```rust
pub struct VaultValidator {
    vault_path: PathBuf,
}

// 機能:
// - vault内パスの検証
// - フォルダ存在確認
// - パストラバーサル攻撃の防止
// - 許可されたパスかの確認
```

#### FileHandler

```rust
pub struct FileHandler;

// 機能:
// - ファイル名の自動生成
// - 安全なファイル書き込み
// - 既存ファイルの確認
// - ファイル権限の適切な設定
```

### 4. エラーハンドリングの拡張

#### Obsidian 固有のエラー型

```rust
#[derive(Debug, thiserror::Error)]
pub enum ObsidianError {
    #[error("Invalid vault path: {0}")]
    InvalidVaultPath(String),

    #[error("Folder not found: {0}")]
    FolderNotFound(String),

    #[error("File already exists: {0}")]
    FileAlreadyExists(String),

    #[error("Path traversal detected: {0}")]
    PathTraversalDetected(String),

    #[error("Vault access denied: {0}")]
    VaultAccessDenied(String),
}
```

### 5. ツールの登録と初期化

#### ツールレジストリへの登録

````rust
pub fn register_obsidian_tools(registry: &mut ToolRegistry, config: &Config) -> Result<()> {
    let save_markdown = SaveMarkdownTool::new(config.vault_path.clone())?;
    registry.register(Box::new(save_markdown))?;
    Ok(())
}
## ビルド・動作確認

### ユニットテスト

```bash
cargo test tools::obsidian
````

### 統合テスト

- vault 操作の正常系テスト
- エラーケースのテスト
- セキュリティテスト（パストラバーサル等）

### 手動テスト

MCP クライアントを使用した実際の動作確認:

1. ツール一覧の取得確認
2. markdown ファイル保存機能の確認
3. エラーハンドリングの確認

## 期待される成果

- markdown ファイル保存機能が正常動作
- vault 内の安全なファイル操作が実現
- 適切なエラーハンドリングとログ出力
- 次フェーズでのインストール・設定準備完了

## セキュリティ考慮事項

- パストラバーサル攻撃の防止
- vault 外へのアクセス制限
- ファイルサイズ制限
- 許可されたフォルダ内での操作制限
