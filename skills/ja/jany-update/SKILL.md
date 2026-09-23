---
name: jany-update
description: 古い jany で書いた定義(schema.toml / assemble.sh / cases.toml)に、今の jany で増えた機能だけを足す。`/jany-update <name> [sub]` は既存の規則・役割・cases を書き直さず、足りないところだけ足して、`jany --test` が前と同じ本数以上通ることを確かめる。
---

# jany-update — 既存の定義を今の jany に合わせる

`/jany-register` で作った定義は、書いたときの jany の機能しか使っていない。
このスキルは**差分だけ**を足す。動いている規則は触らない。

- 今の DSL: jany-register スキルの reference.md(このスキルの隣、`../jany-register/reference.md`。**必ず先に読む**)
- 組み込み定義の最新版: `../jany-register/examples/{find,curl,docker/run}/`
- 置き場所: `~/.config/jany/cmd/<name>[/<sub>]/`(`JANY_CMD_DIR` があればそこ)

## 手順

### 1. 何が足りないかを jany に聞く

```
jany --update <name> [sub]
```

- 組み込み定義(find / curl / docker run)で手を入れていないものは、これだけで最新版に置き換わる。「updated」「up to date」と出たら終わり
- 「lacks …」と出たら、足りない機能の一覧。「you have edited …」と出たら、編集済みの組み込み定義(3 へ)
- 定義が無いと言われたら、止めてユーザーに報告する(`/jany-register` の出番)

### 2. 前の状態を記録する

```
jany --test <name> [sub]
```

通った本数・落ちた本数を控える。**最初から落ちている case は直さない**(このスキルの範囲外)。報告に書く。

### 3. 足りないものだけ足す

- reference.md を読み、定義に無いキー・節を探す。1 の「lacks」に出たものは必ず、それ以外は reference.md にあって定義に無いものを候補にする
- 足すときは reference.md の説明と examples の書き方に揃える
- **既存の `[[roles]]` `[[rules]]` `[[questions]]` `[[repair]]`、assemble.sh、既存の cases は書き直さない**。足す節が既存の役割名を参照するなら、その名前をそのまま使う
- 編集済みの組み込み定義では、`../jany-register/examples/<name>/` と定義を比べ、ユーザーの編集を残したまま、最新版で増えた部分だけを取り込む。ユーザーの編集と最新版がぶつかる箇所は書き換えず、報告に回す

足すものの例:

| 機能 | 足すもの |
|---|---|
| zsh の薄い候補 | `[[placeholders]]` を 3〜6 枠。人が言う順に並べ、裸の語がふつう入る枠に `bare = true`。`roles` は `[[roles]]` にある名前だけ |

### 4. 確かめる

```
jany --test <name> [sub]                 # 2 と同じ本数以上通ること
jany --suggest -- <name> [sub]           # placeholders を足したなら、全部の枠が出る
jany --suggest -- <name> [sub] <語 …>    # cases の words を 2〜3 本入れて、埋まった枠が消える
jany --update <name> [sub]               # もう lacks が出ない
```

cases が 2 より減ったら、足したものを戻して原因を探す。通ったことにしない。

### 5. 報告

- 足したもの(ファイルと節)。既存を変えていないこと
- 前後の `jany --test` の結果(出力ごと)
- 取り込まなかったもの・ぶつかった箇所とその理由

## 注意

- 事実と推測を分ける。確かめていないことは「確かめていない」と書く
- 頼まれていない定義は触らない。`jany --update` が複数の定義に「lacks」と言っても、やるのは頼まれた 1 つ
- ファイルを消さない。定義ディレクトリを作り直さない
