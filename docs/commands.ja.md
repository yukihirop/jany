# コマンドの定義

コマンドの定義の作り方、中身、新しくするやり方。[README に戻る](../README.ja.md)

## コマンドを足す

```sh
jany --register tar           # 任意: 雛形だけ置く(スキルは使わない)
/jany-register tar            # Claude Code か Codex で: 定義を作り、埋めて、テストする
jany --test tar
```

定義は `~/.config/jany/cmd/<name>[/<sub>]/` に置く:

| ファイル | |
|---|---|
| `schema.toml` | 役割(jev が選べるもの)、語の表、規則、jev への質問、repair の手順、危険度の設定 |
| `assemble.sh` | 役割の付いたトークンを受け取り、`{argv, preview, risk, pipe, error}` を返す。言語は問わない。bash + jq で足りる |
| `cases.toml` | 語 → 期待する argv と、書き下した jev の答え。jany が聞いていない質問への答えは `jany --test` が受け付けない |

スキルは書く前に、schema の全キーのリファレンスと 2 つの実例(`find`、`curl`)を読む。jind と jurl の経験則はそのまま使える: 実際に打つ 8 割を載せ、残りは `--` の後ろで素通しし、cases の無い `assemble.sh` は信用しない。

## 定義を新しくする

新しい jany では定義の機能(`[[placeholders]]` など)や組み込み定義が増えることがある。ただし `jany --init` は、すでにある定義には触らない。jany を上げたら次を実行する:

```sh
jany --update                 # 手を入れていない組み込み定義は置き換わり、残りは一覧に出る
/jany-update tar              # Claude Code か Codex で: tar に足りないものを足し、jany --test tar まで回す
```

組み込み定義に手が入っていないかは、全ファイルが jany の出した版のどれかと一致するかで判断する(`examples/released.txt`)。編集済みの組み込み定義はそのまま残し、あなたの定義と同じく一覧に出す。`/jany-update` は、あなたの編集を残したまま新しい部分を取り込む。`/jany-update` スキル自体が無ければ `jany --update` が置く(言語は置いてある `/jany-register` に合わせる。`--locale en|ja` でも選べる)。すでにあるスキルのファイルは触らない。
