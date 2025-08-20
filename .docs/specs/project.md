# Obsidian MCP Server プロジェクト仕様書

作成日時: 2025-07-31 12:00
更新日時: 2025-08-20 12:30

## プロジェクト概要

Obsidian Vault を操作するための Model Context Protocol (MCP) サーバーの実装。

Rust で記述され、`rmcp` クレートを用いた MCP サーバーフレームワーク上で動作する。

現行実装は `rmcp` クレートを利用した最小サーバー構成。

## コマンドライン引数

```rust
#[derive(Parser)]
struct Cli {
        /// デバッグモードで実行
        #[arg(short,long)]
        debug: bool,
}
```

利用可能なオプション:

- `--debug`: デバッグモードで実行（debug-vault/ を自動使用）

現行 CLI は `--debug` フラグのみを受け付ける最小構成へ簡素化された。（旧: `--config`, `--vault-dir`, `--sync` は削除。設定ファイル位置は固定ロジック化。）

### 起動時設定ロードフロー

1. `--debug` 指定時: `DebugConfig` で `./debug-vault` と `./.config/config.toml` を使用しサンプルノート生成。ログは `./.config/logs`。
2. 通常時: `dirs::config_dir()/obsidian-mcp-server/config.toml` を生成/読み込みし、ログは同ディレクトリ配下 `logs/`。

## プロジェクト構造（現行）

```text
src/
├── main.rs          # エントリポイント / CLI / ロガー初期化 / サービス起動
├── server.rs        # rmcp ベースの ObsidianServer 実装（ツール登録含む）
├── config.rs        # アプリ設定 (vault_dir, template_file, tag_list)
├── debug.rs         # デバッグモード環境構築
├── logger.rs        # flexi_logger 初期化
├── error.rs         # 共通エラー型（AppResult）
├── vault.rs         # VaultOperations（ファイル保存/検証ユーティリティ）
└── （将来追加予定のモジュールは適宜拡張）
```

（旧構成に関する記述は省略）

## データ構造

### Config (`src/config.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
        vault_dir: Option<PathBuf>,
        template_file: Option<PathBuf>,
        #[serde(default)]
        tag_list: Vec<String>,
}
```

- `vault_dir`: Vault ルートパス（必須: 実行前に設定される想定）
- `template_file`: テンプレートファイル相対パス（任意）
- `tag_list`: タグ候補（デフォルト: `["Tips"]`）

### VaultOperations (`src/vault.rs`)

ファイル保存とパス検証ユーティリティ。

主メソッド:

- `save_markdown_file`（ファイル存在/境界/名前検証。
  - `.md` 自動付与
  - 既存ファイル上書き防止
  - Windows 予約語 / 危険文字 / ".." 排除）
- `is_path_within_vault`
- `validate_filename`

### MCP サーバー (`src/server.rs`)

`rmcp` の `tool_router` マクロでツール登録。

```rust
pub struct ObsidianServer {
    config: Arc<Mutex<Config>>,
    tool_router: ToolRouter<ObsidianServer>,
}
```

提供ツール:

| ツール   | 説明                     | 出力                                                |
| -------- | ------------------------ | --------------------------------------------------- |
| get_tags | `Config.tag_list` を列挙 | `CallToolResult` (各タグを text Content として返却) |

`initialize` ハンドラは `ProtocolVersion::V_2024_11_05` と tools capability を返却。

### エラーハンドリング

アプリ全体: `type AppResult<T> = anyhow::Result<T>`。

ツール: `Result<CallToolResult, McpError>`。

### ログ出力機能

`flexi_logger` によるファイルロギング。サイズ 10MB ローテーション、最新 5 ファイル保持。

### Markdown ファイル保存ユーティリティ

`VaultOperations::save_markdown_file` による安全な保存処理（ファイル名検証・パストラバーサル防止・重複防止）。

### 設定管理

- `load_or_default` がファイル非存在時にデフォルト生成
- デバッグ / 通常で設定ファイル & ログ格納パスを分離

### デバッグ機能

`DebugConfig` により demo Vault (`./debug-vault`) とサンプルノートを自動生成。

## コーディング規約（抜粋）

- ワイルドカードインポート禁止
- Vault 外アクセス禁止（パス正規化 & prefix チェック）
- ファイル名バリデーション必須

## 依存関係（主要）

- anyhow / serde / serde_json / tokio / toml / dirs / clap / chrono / flexi_logger / log / once_cell

開発: tempfile

## 実装済み機能サマリ

- rmcp ベースサーバー (`get_tags` ツール)
- デバッグ環境自動構築（サンプルノート生成）
- ファイルロギング（サイズローテーション）
- Markdown 保存ユーティリティ（API 化前段階）

## 今後の拡張予定

1. `get_tags` のフィルタ / メタ情報拡張
2. Markdown 保存ツール（push_markdown）公開化
3. テンプレート取得ツール追加
4. 設定ホットリロード
5. Vault 内検索（一覧 / 全文）
6. メタデータ操作（frontmatter 編集）
