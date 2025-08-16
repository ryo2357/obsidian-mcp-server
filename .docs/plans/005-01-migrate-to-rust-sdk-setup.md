# 005-01 Rust SDK 移行準備 (公式 MCP rust-sdk 導入)

作成日時: 2025-08-16 15:50
参照アイディアファイル: `005_プロジェクト構造の変更.md`

## 目的

現行の自前 JSON-RPC / MCP プロトコル実装 ( `mcp/protocol.rs`, `mcp/server.rs` ) を段階的に公式 Rust SDK (`modelcontextprotocol/rust-sdk`) ベースへ移行するための初期準備を行う。リスクを最小化しつつ並行期間を設け、既存ツール (save_markdown_file / get_template_markdown / list_note_tags) の動作を保持したまま新 SDK インフラを導入する。

## スコープ (本フェーズで完了させる範囲)

- Rust SDK 依存関係の導入 (cargo add)
- 新サーバー実装用モジュールスケルトン作成 (`mcp/sdk_server/` 仮)
- 既存コードとの名称衝突回避 (型のプレフィックス戦略整理)
- 変換/互換レイヤ設計 (旧 `JsonRpcRequest` -> SDK Request 方向のブリッジ方針) の仕様確定 (実装は次フェーズ)
- ビルドが緑のままであることの確認
- 仕様書更新に向けた差分メモ (report 用メモ案内)

## 非スコープ (次以降)

- 既存サーバーロジックの SDK 置換
- ツール呼び出し経路の差し替え
- 旧 `protocol.rs` の削除

## 現行構造と課題整理

| 項目                  | 現状                    | 課題                          |
| --------------------- | ----------------------- | ----------------------------- |
| プロトコル型          | 手書き struct 群        | SDK の公式互換性/更新追従不可 |
| デコード/ディスパッチ | 手書き match + メソッド | 冗長 / エラー分類粗い         |
| ツール登録            | 手動 list + match       | 拡張毎に分岐追加が増大        |
| エラーモデル          | 独自 `McpError`         | SDK 期待エラー型との互換不明  |

## 目標アーキテクチャ (高レベル)

```
+---------------------------+
| main.rs                   |
|  - debug/config 初期化    |
|  - 選択: 旧 or 新 SDK     |
+--------------+------------+
               |
        (新) sdk_server
               |
        +------+-------------------+
        | Tool registry (SDK API) |
        | Adapter (旧Tool呼び出し)|
        +-------------------------+
```

## 実装する機能 (本フェーズ)

1. 依存追加: `modelcontextprotocol` (仮 crate 名: rust-sdk 公開時想定。実際の Cargo 名称確認が必要 / 未確定なら TODO として明記)
2. モジュール雛形:
   - `src/mcp/sdk_server/mod.rs` : 新サーバーエントリ (構造体定義のみ)
   - `src/mcp/sdk_server/adapter.rs` : 既存ツールラッパ (空 stub)
3. フィーチャーフラグ/環境切替案の記述: 環境変数 `MCP_SDK_ENABLE=1` で新経路起動 (実装は次フェーズ) — 本フェーズでは TODO コメントのみ。
4. 命名衝突ポリシー策定: 旧型は `Legacy*` へ型エイリアス導入 (削除リスクを下げる)。
5. ドキュメント (コメント) で次フェーズのエントリポイント想定を明示。

## 詳細仕様

### 依存導入方針

- コマンド: `cargo add modelcontextprotocol` (実際の crate 名確認。異なる場合は README リンク調査後修正)
- セマンティックバージョンは `^` デフォルト。互換性リスク低減のため `~` も検討 (追記コメント)。

### ディレクトリ構成 (追加分)

```
src/mcp/
  sdk_server/
    mod.rs        # 新サーバー構造体と初期化インターフェイス予定
    adapter.rs    # 既存ツール呼び出しを SDK Tool トrait に適合させる層 (stub)
```

### アダプタ設計 (仕様のみ)

| 区分            | 旧               | 新(SDK)           | アダプタ責務                                                 |
| --------------- | ---------------- | ----------------- | ------------------------------------------------------------ |
| initialize 結果 | InitializeResult | SDK Provided Type | 旧 Config 取得 → SDK Capability 組立                         |
| tools/list      | Vec<Tool> 手書き | SDK Tool registry | 旧定義を SDK Tool trait 実装へ委譲                           |
| tools/call      | match name       | SDK dispatch      | Tool trait 実装で統一; 旧レスポンス構造 -> SDK Response 変換 |

### 移行リスクと軽減策 (本フェーズ時点)

- Crate 名相違: コメントに TODO とし確定前にブロック/CI 失敗防止
- 型重複: プレフィックス & モジュール分離で回避
- ビルド失敗: 初期段階では SDK 利用をまだ行わない (use 宣言も最小限)

## 技術的実装方針

- 既存コードは編集最小限 (新規追加中心)
- 新コード内で `//!` ドキュメントコメントを用い後続フェーズタスクを TODO ラベル化
- CI / ビルド: `cargo build` が成功する状態で PR 第 1 段階

## 実装手順

1. 依存追加 (cargo add) — 実際追加は実装フェーズで行う
2. ディレクトリ `mcp/sdk_server` 追加
3. `mod.rs` に `pub struct SdkMcpServer { /* placeholder */ }` とロード関数 `pub fn new_placeholder()`
4. `adapter.rs` に Tool アダプタ設計コメントとダミー trait/struct (コンパイル通る最小)
5. 旧コードへの影響ゼロを確認

## 動作確認 (本フェーズ)

- `cargo build` 通過
- バイナリ実行パス変化なし (現行サーバー起動動作影響無し)

## 完了条件

- 新規 2 ファイル追加
- ビルド成功 (SDK crate 名未確定なら一時コメント化してビルド保持)
- 次フェーズ (005-02) で使用する設計コメントが明確

## 次フェーズ依存

- 005-02 で SdkMcpServer の本実装 (initialize, tool registry 移行)

## 参考/メモ

- 公式 SDK の API (仮): ServerBuilder, Tool trait, run_stdio() 等
- 実際の関数名差異は 005-02 着手時点で反映
