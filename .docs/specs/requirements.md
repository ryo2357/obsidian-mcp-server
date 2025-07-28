# 要件定義書

## はじめに

### プロジェクト概要

このプロジェクトは、Obsidian Vault を安全に操作するための Model Context Protocol（MCP）サーバーを Rust で実装します。GitHub Copilot が Obsidian Vault 内のファイルを直接操作することに対する懸念を解決し、制限された Tool を通じて安全にファイル操作を行えるシステムを提供します。

### 既存システムとの関係性

- 現在は Rust プロジェクトの基本構造（Cargo.toml、main.rs）のみが存在
- VSCode の GitHub Copilot と連携する MCP サーバーとして動作
- Obsidian Vault の外部ツールとしてファイル操作機能を提供

### 実装予定の機能概要

- MCP プロトコルに準拠したサーバー実装
- 設定ファイルベースの Vault パス管理
- 制限された Tool を通じた安全な Markdown ファイル操作
- GitHub Copilot によるファイル内容の加工・保存機能

## 要件

### 要件 1: MCP サーバー基盤の実装

**ユーザーストーリー:** 開発者として、Obsidian Vault を安全に操作するための MCP サーバーを利用したい。これにより、GitHub Copilot による直接的なファイル操作の不安を解消できる。

#### 受け入れ基準

1. WHEN システムを cargo install でインストールする THEN ~/.cargo/bin に obsidian-vault-mcp バイナリが配置される SHALL
2. WHEN VSCode で MCP サーバー設定を行う THEN github.copilot.chat.modelContextProtocol.servers 設定で obsidian-vault-mcp コマンドが実行される SHALL
3. WHEN MCP サーバーが起動する THEN RUST_LOG=info レベルでログが出力される SHALL

### 要件 2: 設定管理システム

**ユーザーストーリー:** システム利用者として、Obsidian Vault のパスを設定ファイルで管理したい。これにより、環境に応じた柔軟な Vault 操作が可能になる。

#### 受け入れ基準

1. WHEN システムが起動する THEN ~/.config/obsidian-vault-mcp.toml から設定を読み込む SHALL
2. WHEN 設定ファイルが存在しない THEN 適切なエラーメッセージを表示する SHALL
3. WHEN 設定ファイルに vault_path が記載されている THEN そのパスを Vault のルートディレクトリとして使用する SHALL

### 要件 3: 拡張可能な Tool アーキテクチャ

**ユーザーストーリー:** 開発者として、複数の Tool を実装可能な設計を利用したい。これにより、将来的な機能拡張に対応できる。

#### 受け入れ基準

1. WHEN システム設計を行う THEN 複数の Tool を登録・管理できる構造を持つ SHALL
2. WHEN 新しい Tool を追加する THEN 既存コードへの影響を最小限に抑えて追加できる SHALL
3. WHEN MCP クライアントが Tool リストを要求する THEN 利用可能な全 Tool の情報を返す SHALL

### 要件 4: Markdown ファイル保存機能

**ユーザーストーリー:** GitHub Copilot ユーザーとして、加工した Markdown ファイルを指定フォルダに保存したい。これにより、Copilot で生成・編集したコンテンツを Obsidian Vault に安全に保存できる。

#### 受け入れ基準

1. WHEN Tool が Markdown 保存要求を受信する THEN 指定されたフォルダにファイルを保存する SHALL
2. WHEN ファイル保存を行う THEN Obsidian Vault のルートディレクトリからの相対パスで指定できる SHALL
3. WHEN 保存先ディレクトリが存在しない THEN 必要に応じてディレクトリを作成する SHALL
4. WHEN ファイルが既に存在する THEN 上書き確認または新しいファイル名での保存を提供する SHALL

### 要件 5: GitHub Copilot 連携機能

**ユーザーストーリー:** GitHub Copilot ユーザーとして、コンテキストとして取り込んだファイルを Copilot で加工して保存したい。これにより、既存ファイルの内容を参考にした新しいファイルの生成が効率的に行える。

#### 受け入れ基準

1. WHEN Copilot がコンテキストファイルを指定する THEN そのファイル内容を読み込んで加工できる SHALL
2. WHEN ファイル加工処理を実行する THEN 元ファイルの内容を変更せずに新しいファイルとして保存する SHALL
3. WHEN 加工されたファイルを保存する THEN Markdown 形式で適切にフォーマットされている SHALL

## 非機能要件

### パフォーマンス要件

- ファイル読み書き操作は 1 秒以内に完了する
- 設定ファイル読み込みは 500ms 以内に完了する
- MCP プロトコル応答は 100ms 以内に返す

### セキュリティ要件

- Vault 外部への意図しないファイル書き込みを防ぐ
- 設定ファイルで指定されたディレクトリ外へのアクセスを制限する
- ファイル操作時の適切な権限チェックを実施する

### 利用性要件

- README.md に日本語での使用方法を記載する
- エラーメッセージは日本語で分かりやすく表示する
- ログ出力により動作状況を追跡可能にする

### 保守性要件

- Rust の標準的なプロジェクト構造に従う
- コードコメントは日本語で記述する
- 設定可能な部分と固定部分を明確に分離する
