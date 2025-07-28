# 最終統合・テスト・パッケージングプラン

## 基本情報

- 作成日: 2025-07-28
- 参照仕様書セクション: 要件 1（MCP サーバー基盤の実装）、非機能要件（全体）
- 参照設計書セクション: 実装順序と統合方針、全コンポーネント統合
- プラン進捗: 5/5（最終統合・テスト・パッケージング）
- 前提条件: 001-01〜001-04 の全機能が実装済み

## 概要と目標

### 実装する機能の詳細説明

このフェーズでは、実装した全機能の統合テスト、VSCode との実際の連携確認、パフォーマンス最適化、および本番環境での利用に向けたパッケージング作業を行います。プロジェクトの完成と実際の使用開始を目標とします。

### 達成すべき具体的な目標

1. VSCode + GitHub Copilot との実際の連携テスト
2. エラーケースの網羅的なテスト
3. パフォーマンス要件の確認と最適化
4. パッケージング・インストール機能の実装
5. ドキュメントの整備と使用方法の確立

### requirements.md との対応関係

- 要件 1-1: cargo install での obsidian-vault-mcp バイナリ配置
- 要件 1-2: VSCode MCP サーバー設定対応
- 非機能要件: パフォーマンス、セキュリティ、利用性、保守性の全面確認

## 実装手順

### ステップ 1: Cargo.toml の最終調整とパッケージング準備

1. Cargo.toml の本番向け設定

   ```toml
   [package]
   name = "obsidian-vault-mcp"
   version = "0.1.0"
   edition = "2021"
   authors = ["Your Name <your.email@example.com>"]
   description = "Obsidian Vault を安全に操作するための Model Context Protocol サーバー"
   license = "MIT OR Apache-2.0"
   repository = "https://github.com/your-username/obsidian-mcp-server"
   keywords = ["obsidian", "mcp", "model-context-protocol", "vault"]
   categories = ["command-line-utilities", "development-tools"]
   readme = "README.md"

   [[bin]]
   name = "obsidian-vault-mcp"
   path = "src/main.rs"

   [dependencies]
   tokio = { version = "1.0", features = ["full"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   toml = "0.8"
   log = "0.4"
   env_logger = "0.11"
   anyhow = "1.0"
   thiserror = "1.0"
   dirs = "5.0"
   async-trait = "0.1"

   [profile.release]
   opt-level = 3
   lto = true
   codegen-units = 1
   panic = "abort"
   strip = true
   ```

### ステップ 2: VSCode 統合テスト環境の構築

1. VSCode 設定ファイルの作成 `.vscode/settings.json`

   ```json
   {
     "github.copilot.chat.modelContextProtocol.servers": {
       "obsidian-vault-mcp": {
         "command": "cargo",
         "args": ["run", "--release"],
         "env": {
           "RUST_LOG": "info"
         }
       }
     }
   }
   ```

2. テスト用の設定ファイル作成スクリプト

   ```bash
   # tests/setup_test_environment.ps1
   # Windows PowerShell 用のテスト環境構築スクリプト

   # テスト用 Vault ディレクトリ作成
   $testVaultPath = "$env:TEMP\obsidian-test-vault"
   if (Test-Path $testVaultPath) {
       Remove-Item -Recurse -Force $testVaultPath
   }
   New-Item -ItemType Directory -Path $testVaultPath -Force

   # テスト用設定ファイル作成
   $configDir = "$env:USERPROFILE\.config"
   if (-not (Test-Path $configDir)) {
       New-Item -ItemType Directory -Path $configDir -Force
   }

   $configContent = @"
   vault_path = "$testVaultPath"
   allowed_extensions = ["md", "txt"]
   max_file_size = 1048576
   log_level = "debug"
   "@

   $configPath = "$configDir\obsidian-vault-mcp.toml"
   Set-Content -Path $configPath -Value $configContent -Encoding UTF8

   # テスト用ファイル作成
   $notesDir = "$testVaultPath\notes"
   New-Item -ItemType Directory -Path $notesDir -Force

   $sampleContent = @"
   # サンプルノート

   これはテスト用のサンプルファイルです。

   ## セクション1

   - 項目1
   - 項目2
   - 項目3

   ## セクション2

   この内容は GitHub Copilot での参照テストに使用されます。
   "@

   Set-Content -Path "$notesDir\sample.md" -Value $sampleContent -Encoding UTF8

   Write-Host "テスト環境が構築されました:"
   Write-Host "Vault パス: $testVaultPath"
   Write-Host "設定ファイル: $configPath"
   ```

### ステップ 3: 統合テストスイートの作成

1. tests/integration_tests.rs の作成

   ```rust
   use std::process::{Command, Stdio};
   use std::io::{Write, BufRead, BufReader};
   use std::time::Duration;
   use tokio::time::timeout;
   use serde_json::{json, Value};

   #[tokio::test]
   async fn test_mcp_server_lifecycle() {
       // サーバー起動
       let mut child = Command::new("cargo")
           .args(&["run", "--release"])
           .stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .spawn()
           .expect("MCPサーバーの起動に失敗");

       let stdin = child.stdin.take().expect("標準入力の取得に失敗");
       let stdout = child.stdout.take().expect("標準出力の取得に失敗");

       // 初期化テスト
       let init_request = json!({
           "jsonrpc": "2.0",
           "id": 1,
           "method": "initialize",
           "params": {}
       });

       let response = send_request_and_get_response(stdin, stdout, init_request).await;
       assert!(response["result"]["protocolVersion"].is_string());
       assert_eq!(response["result"]["serverInfo"]["name"], "obsidian-vault-mcp");

       child.kill().expect("プロセス終了に失敗");
   }

   #[tokio::test]
   async fn test_tools_functionality() {
       let mut child = start_mcp_server().await;
       let (stdin, stdout) = get_stdio(&mut child);

       // ツールリスト取得
       let tools_request = json!({
           "jsonrpc": "2.0",
           "id": 2,
           "method": "tools/list"
       });

       let response = send_request_and_get_response(stdin, stdout, tools_request).await;
       let tools = response["result"]["tools"].as_array().unwrap();

       assert_eq!(tools.len(), 3);
       let tool_names: Vec<&str> = tools.iter()
           .map(|t| t["name"].as_str().unwrap())
           .collect();

       assert!(tool_names.contains(&"save_markdown"));
       assert!(tool_names.contains(&"read_markdown"));
       assert!(tool_names.contains(&"list_files"));

       child.kill().expect("プロセス終了に失敗");
   }

   #[tokio::test]
   async fn test_file_operations() {
       let mut child = start_mcp_server().await;
       let (stdin, stdout) = get_stdio(&mut child);

       // ファイル保存テスト
       let save_request = json!({
           "jsonrpc": "2.0",
           "id": 3,
           "method": "tools/call",
           "params": {
               "name": "save_markdown",
               "arguments": {
                   "file_path": "test/integration_test.md",
                   "content": "# 統合テスト\n\nこのファイルは統合テストで作成されました。"
               }
           }
       });

       let save_response = send_request_and_get_response(stdin, stdout, save_request).await;
       assert_eq!(save_response["result"]["content"][0]["text"].as_str().unwrap().contains("success\": true"), true);

       // ファイル読み込みテスト
       let read_request = json!({
           "jsonrpc": "2.0",
           "id": 4,
           "method": "tools/call",
           "params": {
               "name": "read_markdown",
               "arguments": {
                   "file_path": "test/integration_test.md"
               }
           }
       });

       let read_response = send_request_and_get_response(stdin, stdout, read_request).await;
       let content = read_response["result"]["content"][0]["text"].as_str().unwrap();
       assert!(content.contains("統合テスト"));

       child.kill().expect("プロセス終了に失敗");
   }

   async fn start_mcp_server() -> std::process::Child {
       Command::new("cargo")
           .args(&["run", "--release"])
           .stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .spawn()
           .expect("MCPサーバーの起動に失敗")
   }

   fn get_stdio(child: &mut std::process::Child) -> (&mut std::process::ChildStdin, &mut std::process::ChildStdout) {
       let stdin = child.stdin.as_mut().expect("標準入力の取得に失敗");
       let stdout = child.stdout.as_mut().expect("標準出力の取得に失敗");
       (stdin, stdout)
   }

   async fn send_request_and_get_response(
       stdin: &mut std::process::ChildStdin,
       stdout: &mut std::process::ChildStdout,
       request: Value
   ) -> Value {
       let request_str = request.to_string() + "\n";
       stdin.write_all(request_str.as_bytes()).expect("リクエスト送信に失敗");
       stdin.flush().expect("フラッシュに失敗");

       let mut reader = BufReader::new(stdout);
       let mut response_line = String::new();
       reader.read_line(&mut response_line).expect("レスポンス読み込みに失敗");

       serde_json::from_str(&response_line).expect("JSONパースに失敗")
   }
   ```

### ステップ 4: パフォーマンステストと最適化

1. benches/performance_tests.rs の作成

   ```rust
   use criterion::{black_box, criterion_group, criterion_main, Criterion};
   use std::time::Instant;
   use obsidian_mcp_server::config::Config;
   use obsidian_mcp_server::vault::VaultHandler;

   fn benchmark_config_loading(c: &mut Criterion) {
       c.bench_function("config_loading", |b| {
           b.iter(|| {
               let config_path = Config::default_config_path();
               black_box(Config::load_from_file(config_path))
           });
       });
   }

   fn benchmark_path_validation(c: &mut Criterion) {
       let config = Config::default();
       let vault_handler = VaultHandler::new(std::path::PathBuf::from("./test-vault"), config);

       c.bench_function("path_validation", |b| {
           b.iter(|| {
               black_box(vault_handler.validate_path("notes/test.md"))
           });
       });
   }

   async fn benchmark_file_operations(c: &mut Criterion) {
       let config = Config::default();
       let vault_handler = VaultHandler::new(std::path::PathBuf::from("./test-vault"), config);

       c.bench_function("file_save_1kb", |b| {
           let content = "x".repeat(1024);
           b.iter(|| {
               black_box(vault_handler.save_file("bench/test_1kb.md", &content, true))
           });
       });

       c.bench_function("file_save_100kb", |b| {
           let content = "x".repeat(100 * 1024);
           b.iter(|| {
               black_box(vault_handler.save_file("bench/test_100kb.md", &content, true))
           });
       });
   }

   criterion_group!(benches, benchmark_config_loading, benchmark_path_validation);
   criterion_main!(benches);
   ```

2. Cargo.toml にベンチマーク設定追加

   ```toml
   [dev-dependencies]
   criterion = "0.5"

   [[bench]]
   name = "performance_tests"
   harness = false
   ```

### ステップ 5: ドキュメント整備

1. README.md の完成版作成

   ````markdown
   # Obsidian Vault MCP Server

   Obsidian Vault を安全に操作するための Model Context Protocol（MCP）サーバーです。GitHub Copilot が Obsidian Vault 内のファイルを制限された Tool を通じて安全に操作できます。

   ## 特徴

   - MCP プロトコル準拠のサーバー実装
   - Vault 外部への意図しないアクセスを防ぐセキュリティ機能
   - GitHub Copilot との自然な連携
   - 設定ファイルベースの柔軟な Vault 管理

   ## インストール

   ### Cargo からのインストール

   ```bash
   cargo install obsidian-vault-mcp
   ```
   ````

   ### ソースからのビルド

   ```bash
   git clone https://github.com/your-username/obsidian-mcp-server.git
   cd obsidian-mcp-server
   cargo build --release
   cargo install --path .
   ```

   ## 設定

   ### 1. 設定ファイルの作成

   `~/.config/obsidian-vault-mcp.toml` を作成:

   ```toml
   vault_path = "/path/to/your/obsidian/vault"
   allowed_extensions = ["md", "txt"]
   max_file_size = 1048576  # 1MB
   log_level = "info"
   ```

   ### 2. VSCode の設定

   VSCode の設定ファイル（settings.json）に以下を追加:

   ```json
   {
     "github.copilot.chat.modelContextProtocol.servers": {
       "obsidian-vault-mcp": {
         "command": "obsidian-vault-mcp"
       }
     }
   }
   ```

   ## 使用方法

   ### 利用可能な Tool

   1. **save_markdown** - Markdown ファイルの保存
   2. **read_markdown** - Markdown ファイルの読み込み
   3. **list_files** - ファイル一覧の取得

   ### GitHub Copilot での使用例

   1. **新しいファイルの作成**:

      ```text
      @obsidian-vault save_markdown "notes/new_article.md" "# 新しい記事\n\n内容をここに書く"
      ```

   2. **既存ファイルの参照**:

      ```text
      @obsidian-vault read_markdown "notes/existing_note.md"
      ```

   3. **ファイル一覧の確認**:

      ```text
      @obsidian-vault list_files {"extension": "md", "directory": "notes"}
      ```

   ## セキュリティ

   - すべてのファイル操作は設定された Vault 内に制限
   - パス走査攻撃を防ぐ厳密なパス検証
   - 設定可能なファイルサイズ制限
   - 許可された拡張子のみ操作可能

   ## ライセンス

   MIT または Apache-2.0 のデュアルライセンス

   ## 貢献

   Issue や Pull Request は GitHub リポジトリまで

   ```markdown

   ```

### ステップ 6: 最終ビルドとリリース準備

1. リリース用ビルドの実行

   ```bash
   # 最終ビルド
   cargo build --release

   # テスト実行
   cargo test

   # ベンチマーク実行
   cargo bench

   # パッケージング確認
   cargo package --dry-run

   # インストールテスト
   cargo install --path . --force
   ```

2. 動作確認テスト

   ```bash
   # 設定ファイル作成とテスト環境構築
   .\tests\setup_test_environment.ps1

   # 実際の VSCode 連携テスト
   # 1. VSCode を開く
   # 2. GitHub Copilot Chat で以下を実行:
   #    @obsidian-vault list_files
   #    @obsidian-vault read_markdown "notes/sample.md"
   #    @obsidian-vault save_markdown "notes/test_output.md" "# テスト結果\n\n正常に動作しています。"

   # ログ確認
   RUST_LOG=debug obsidian-vault-mcp
   ```

3. パフォーマンス要件の確認

   ```bash
   # ファイル読み書き 1秒以内確認
   # 設定ファイル読み込み 500ms以内確認
   # MCP レスポンス 100ms以内確認
   cargo bench
   ```

## 品質保証

### 最終チェックリスト

#### 機能要件確認

- [ ] 要件 1-1: cargo install での obsidian-vault-mcp バイナリ配置
- [ ] 要件 1-2: VSCode MCP サーバー設定対応
- [ ] 要件 1-3: RUST_LOG=info レベルでのログ出力
- [ ] 要件 2-1: ~/.config/obsidian-vault-mcp.toml からの設定読み込み
- [ ] 要件 2-2: 設定ファイル不在時の適切なエラーメッセージ
- [ ] 要件 2-3: vault_path 設定値の Vault ルートディレクトリ利用
- [ ] 要件 3-1: 複数 Tool の登録・管理構造
- [ ] 要件 3-2: 新 Tool 追加時の既存コード影響最小化
- [ ] 要件 3-3: MCP クライアントへの Tool リスト提供
- [ ] 要件 4-1: 指定フォルダへの Markdown ファイル保存
- [ ] 要件 4-2: Vault ルートからの相対パス指定
- [ ] 要件 4-3: 保存先ディレクトリの自動作成
- [ ] 要件 4-4: ファイル重複時の上書き確認・新名前保存
- [ ] 要件 5-1: コンテキストファイル指定・読み込み・加工
- [ ] 要件 5-2: 元ファイル保護での新ファイル保存
- [ ] 要件 5-3: Markdown 形式での適切なフォーマット保存

#### 非機能要件確認

- [ ] パフォーマンス: ファイル操作 1 秒以内、設定読み込み 500ms 以内、MCP レスポンス 100ms 以内
- [ ] セキュリティ: Vault 外部アクセス防止、権限チェック、パス検証
- [ ] 利用性: 日本語 README、日本語エラーメッセージ、ログ出力
- [ ] 保守性: 標準プロジェクト構造、日本語コメント、設定分離

#### VSCode 統合確認

- [ ] GitHub Copilot Chat での Tool 呼び出し成功
- [ ] ファイル保存・読み込み・一覧取得の正常動作
- [ ] エラーケースでの適切なメッセージ表示
- [ ] MCP プロトコル準拠の確認

#### パッケージング確認

- [ ] cargo package の成功
- [ ] cargo install の成功
- [ ] リリースビルドの動作確認
- [ ] ドキュメントの完全性

### 最終リリース作業

1. バージョンタグ作成
2. GitHub リリース作成
3. crates.io への公開（準備できた場合）
4. ドキュメント公開

## プロジェクト完了確認

このプランの完了により、以下が達成されます：

1. **完全に動作する MCP サーバー**

   - Obsidian Vault の安全な操作
   - GitHub Copilot との自然な連携
   - 拡張可能なアーキテクチャ

2. **本番利用可能な品質**

   - 包括的なテスト
   - セキュリティ機能
   - パフォーマンス要件の満足

3. **保守可能なコードベース**

   - 適切なドキュメント
   - 標準的なプロジェクト構造
   - 拡張性を考慮した設計

4. **ユーザーフレンドリーな利用環境**
   - 簡単なインストール
   - 明確な使用方法
   - 効果的な VSCode 統合
