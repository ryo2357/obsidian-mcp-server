# Obsidian MCP Server

Obsidian 用の Model Context Protocol (MCP) サーバーです。

## 機能

- Obsidian vault への Markdown ファイル保存
- MCP 準拠の JSON-RPC 通信

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

### 同期モード（テスト用）

```bash
obsidian-mcp-server --sync
```

## インストール

```bash
cargo build --release
```

## 利用可能なツール

### save_markdown_file

Markdown ファイルを指定されたディレクトリに保存します。

**パラメータ:**

- `filename`: ファイル名（.md 拡張子は自動付与）
- `content`: Markdown コンテンツ

**例:**

```json
{
  "name": "save_markdown_file",
  "arguments": {
    "filename": "my-note",
    "content": "# My Note\n\nThis is a test note."
  }
}
```

## デバッグ環境の設定

### GitHub Copilot で使用する場合

**デバッグモード:**

```json
{
  "mcpServers": {
    "obsidian-mcp-server": {
      "command": "obsidian-mcp-server",
      "args": ["--debug"]
    }
  }
}
```

**本番モード:**

```json
{
  "mcpServers": {
    "obsidian-mcp-server": {
      "command": "obsidian-mcp-server",
      "args": ["--vault-path", "/path/to/production/vault"]
    }
  }
}
```

## 開発

### テスト実行

```bash
cargo test
```

### デバッグモードテスト

```bash
cargo run -- --debug --sync
```

## リファクタリング履歴

- ワイルドカードインポート（`use *`）を明示的なインポートに変更
- デバッグ機能を `src/debug.rs` に集約
- `--debug` フラグによるデバッグ環境の自動構築
