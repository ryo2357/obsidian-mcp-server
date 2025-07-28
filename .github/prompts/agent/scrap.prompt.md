---
mode: 'agent'
description: 'chatをscrapする'
---


ここまでのCopilotとのchat の内容を scrap にまとめてください。

chat 内で重複・修正した内容はまとめる際に反映してください

## scrap 作成時の留意点

- scrap は Markdown 形式で記述します。
- scrap のファイル名は、以下のルールに従ってください。
  - chat の話題をファイル名に含めるようにします。
  - `.github/vibe/scrap` ディレクトリに保存します
  - ファイル名の先頭には、3桁の連番を付けます。
    - 既存のPlanファイルを確認し、連番が欠けないように次の番号を割り当ててください。
  - 作成日は下記のようにファイルの内容に記述します。
    作成日: YYYY-MM-DD
