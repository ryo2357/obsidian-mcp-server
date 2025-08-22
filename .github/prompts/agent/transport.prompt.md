---
mode: 'agent'
description: 'markdownファイルをObsidian Vaultに保存する'
---


- ここまでのCopilotとのchat の内容を scrap にまとめてください。
- markdownファイルを参照して、呼び出されます。
- 参照したmarkdownファイルを編集して、Obsidian Vaultに保存します。
- 編集するための情報の取得、保存のために、@obsidian-mcp-server というMCPサーバーを使用します。

## 実行手順

### フェーズ 1: 編集情報を取得

- @obsidian-mcp-server の`get_tags`と`get_template`でタグの候補とテンプレートを取得する

### フェーズ 2: markdownファイルの編集

- 参照にしたmarkdownファイルの内容を確認し、タグの候補から適切なタグを選択する。
- 選択するタグは多くても3つまでとし、タグの候補にないタグは使用しない。
- ファイルの先頭にテンプレートを挿入する。
- テンプレートに基づいて、必要な情報を埋め込む。
  - 作成日、更新日は現在の日時としてテンプレートに埋め込む
  - 選択したタグをテンプレートのタグ欄に埋め込む
- その他の内容は参照したmarkdownファイルから変更する必要はない

### フェーズ 3: ファイル名の決定

- `ファイル名は「YYYYMMDD_タイトル.md」の形式とする。
- タイトルはファイルの内容から日本語で作成する


### フェーズ 4: ファイルの転送

- @obsidian-mcp-server の `push_markdown` を使用して、編集したmarkdownファイルをObsidian Vaultにアップロードする。
