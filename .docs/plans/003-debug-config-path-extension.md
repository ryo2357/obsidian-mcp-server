# 003-debug-config-path-extension.md

作成日時: 2025-08-13 09:00
参照アイディアファイル: `003_デバッグモードの拡張.md`

## 実装する機能

デバッグモードで起動した際の設定ファイル読み込み先を専用パスに変更する機能を実装する。

## 機能の詳細な仕様

### 概要

現在のデバッグモードでは、設定ファイルのパスは通常モードと同じパス（システムの設定ディレクトリ）を使用している。デバッグモードではプロジェクトローカルの設定ファイル（`.config/config.toml`）を使用するように変更し、開発時とリリース時の設定を分離する。

### 機能要件

1. **デバッグモード用設定パス**

   - デバッグモード時は`.config/config.toml`を設定ファイルとして使用
   - 通常モード時は既存の設定パス（システム設定ディレクトリ）を継続使用

2. **設定ファイルパス取得メソッドの拡張**
   - `Config::default_config_path()`にデバッグモードフラグを追加
   - デバッグモード時は`.config/config.toml`を返す
   - 通常モード時は既存の挙動を維持

### 技術仕様

1. **Config::default_config_path()の拡張**

   - メソッドシグネチャを`default_config_path(debug: bool)`に変更
   - `debug = true`の場合は`PathBuf::from(".config/config.toml")`を返す
   - `debug = false`の場合は既存の挙動を維持

2. **main.rs でのフロー変更**
   - デバッグモード時は`Config::default_config_path(true)`を使用
   - 通常モード時は`Config::default_config_path(false)`を使用
   - 既存の`Config::load_or_default()`も同様に拡張

## 技術的な実装方針

### 変更対象ファイル

1. **src/config.rs**

   - `Config::default_config_path()`メソッドにデバッグモードフラグを追加
   - `Config::load_or_default()`メソッドも同様に拡張

2. **src/main.rs**
   - デバッグモード時の設定読み込みで`Config::default_config_path(true)`を使用
   - 通常モード時は`Config::default_config_path(false)`を使用

### 実装手順

1. **src/config.rs の修正**

   ```rust
   /// 設定ファイルのデフォルトパスを取得
   pub fn default_config_path(debug: bool) -> PathBuf {
       if debug {
           PathBuf::from(".config/config.toml")
       } else if let Some(config_dir) = dirs::config_dir() {
           config_dir
               .join("obsidian-mcp-server")
               .join("config.toml")
       } else {
           PathBuf::from("config.toml")
       }
   }

   /// 設定を読み込み、ファイルが存在しない場合はデフォルト値を使用
   pub fn load_or_default(debug: bool) -> Result<Self> {
       let config_path = Self::default_config_path(debug);
       // 既存のロジックを維持
   }
   ```

2. **main.rs での統合**

   ```rust
   if cli.debug {
       let config = Config::load_or_default(true)?;  // デバッグモード
       // 以下既存処理
   } else {
       let config = Config::load_or_default(false)?;  // 通常モード
       // 以下既存処理
   }
   ```

### セキュリティとエラーハンドリング

1. **パス検証**

   - 設定ファイルパスがプロジェクトルート以下に限定されることを確認
   - 相対パス攻撃の防止

2. **ファイル操作エラー**

   - 既存の`Config::save_to_file`のエラーハンドリングを活用
   - 設定ファイル読み書き失敗時の処理

3. **設定ファイルの検証**
   - 生成された設定ファイルの内容検証
   - 不正な設定値に対するフォールバック

### テスト方針

1. **単体テスト**

   - `Config::default_config_path(true)`がデバッグパスを返すことを確認
   - `Config::default_config_path(false)`が既存パスを返すことを確認
   - `Config::load_or_default(true)`でデバッグ設定が生成されることを確認

2. **統合テスト**
   - デバッグモードで`.config/config.toml`を使用して起動できること
   - 通常モードでの既存動作確認

### 完了条件

1. デバッグモードで`.config/config.toml`を使用して起動できること
2. 通常モードの動作に影響を与えないこと（後方互換性）
3. 単体テストと統合テストが全て通過すること
4. 既存のテストスクリプト（`test_debug.bat`）が正常動作すること

### 影響範囲

- **変更あり**: `src/config.rs`, `src/main.rs`
- **変更なし**: `src/debug.rs`, MCP 関連モジュール, vault 操作モジュール
- **新規作成**: `.config/config.toml`（デバッグモード初回起動時）

この実装により、開発時の設定とリリース時の設定を明確に分離し、デバッグ環境のセットアップがさらに自動化される。
