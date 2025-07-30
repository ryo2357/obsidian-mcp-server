# 001 clap 依存関係に関する質疑応答

作成日時: 2025-07-30 15:00

## 概要

001-01 プロジェクト基盤構築計画における`clap`依存関係の必要性について質疑応答を行った。ユーザーから「VSCode で MCP を起動するときに引数が必要？」という疑問が提起され、clap の具体的な用途と使用パターンについて説明した。

## 質問内容

ユーザーからの質問：

- CLI 引数処理はどういう機能に必要？
- VSCode で MCP を起動するときに引数が必要？

## 回答内容

### clap が必要な理由

1. **デバッグ・開発時のローカル実行**

   ```bash
   cargo run -- --help
   cargo run -- --debug
   cargo run -- --config ./custom-config.toml
   ```

2. **設定ファイルの初期化（001-03 で実装予定）**

   ```bash
   obsidian-vault-mcp --init-config
   obsidian-vault-mcp --validate-config
   ```

3. **テスト・診断機能**

   ```bash
   obsidian-vault-mcp test-vault
   obsidian-vault-mcp validate-config
   ```

### VSCode での MCP 起動について

**通常の起動（引数なし）**：

```json
{
  "github.copilot.chat.modelContextProtocol.servers": {
    "obsidian-vault": {
      "command": "obsidian-vault-mcp"
    }
  }
}
```

**デバッグやカスタム設定が必要な場合（引数あり）**：

```json
{
  "github.copilot.chat.modelContextProtocol.servers": {
    "obsidian-vault": {
      "command": "obsidian-vault-mcp",
      "args": ["--config", "/path/to/custom/config.toml"],
      "env": {
        "RUST_LOG": "debug"
      }
    }
  }
}
```

### 実際の使用パターン

1. **通常運用**: VSCode から引数なしで起動
2. **初期設定**: `--init-config`で設定ファイル作成
3. **開発・デバッグ**: `--debug`や`--config`でカスタム動作
4. **診断**: `test-vault`や`validate-config`でトラブルシューティング

## 結論

`clap`は主に開発・設定・診断のための CLI 機能を提供する。VSCode からの通常起動では基本的に使用されないが、柔軟な運用のために重要な機能である。

## 関連ファイル

- `.docs/plans/001-01-project-setup.md`: 基盤構築計画
- `.docs/plans/001-03-deploy-setup.md`: CLI 機能の詳細実装計画
