# Obsidian MCP Server

Obsidian 用の Model Context Protocol (MCP) サーバーです。

## 機能

- Obsidian vault のテンプレートやタグリストを取得
- Obsidian vault への Markdown ファイル保存

## 使用方法

### 通常モード

```bash
obsidian-mcp-server --vault-path /path/to/your/vault
```

### デバッグモード

開発・テスト用のデバッグモードを利用できます：

```bash
obsidian-mcp-server --debug
```

デバッグモードでは：

- `./debug-vault/` ディレクトリが自動作成されます
- サンプル Markdown ファイルが生成されます
- 追加のデバッグログが出力されます
- `--vault-path` の指定は不要です

## インストール

```bash
cargo build --release
```

## デバッグ環境の設定

### GitHub Copilot で使用する場合

```json:.vscode/mcp.json
{
  "servers": {
    "obsidian-mcp-server": {
      "type": "stdio",
      "command": "obsidian-mcp-server",

    },
  }
}
```

## 開発

### テスト実行

```bash
cargo test
```

### VSCode での検証

```json:.vscode/mcp.json
{
  "servers": {
    "obsidian-mcp-server": {
      "type": "stdio",
      "command": "./target/debug/obsidian-mcp-server",
      "args": [
        "--debug"
      ],
    },
  }
}

```

### MCP Inspector での検証

```bash
mise run inspector
```
