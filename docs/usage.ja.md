# 使い方

フラグ、jany 自身の操作、`~/.config/jany/config.toml` の全部。[README に戻る](../README.ja.md)

## フラグ

```
jany <command> [words ...] [flags] [-- passthrough args]
```

| フラグ | |
|---|---|
| `--explain` | 語ごとの役割、確信度、規則と jev のどちらが決めたか(stderr) |
| `--no-jev` | オフラインのみ。決まらない語はエラー |
| `--hint` | `<command>` に何が言えるか(役割)と、その `cases.toml` から取った例(stderr) |
| `-- …` | そのまま素通し(意味はコマンド次第: find のオプション、curl のフラグ、docker run ならコンテナ内のコマンド) |

jany 自身の操作はフラグなので、`<command>` は常にツール名になる:

| | |
|---|---|
| `jany --list` | 見つかった定義と、それぞれの例 |
| `jany --test find` | 定義の `cases.toml` を回す(jev の答えはモック) |
| `/jany-register tar` | エージェントのスキルで `~/.config/jany/cmd/tar/` を作って埋める |
| `jany --update` | jany を上げた後に: 手を入れていない組み込み定義を置き換え、他の定義に足りないものを一覧し、無いスキルを置く |
| `/jany-update tar` | エージェントのスキルで、定義に足りないものだけを足す。既存の規則と cases はそのまま |
| `jany --on` / `jany --off` | この zsh で、`jany` を付けずに `find empty folders` と打てる(ラッパーが要る) |
| `jany --skills` | 最初の `--init` の前に、スキルだけを置く(次に `/jany-setup`) |
| `jany --init zsh\|bash\|fish` | ラッパーと、組み込み定義とスキル(`--locale en\|ja`、既定は `en`) |
| `jany --setup` | API キーを保存する |

## 設定(任意)

`~/.config/jany/config.toml`

```toml
[jev]
model = "typesafe/jev-1.13"
reject_below = 0.5           # これ未満ならコマンドを出さず、非ゼロで終わって --hint を勧める

[suggest]
enabled = false              # zsh の薄い候補を出さない(JANY_SUGGEST=0/1 で上書き)

[cmd.curl.defaults]          # schema の [defaults] を上書きする
content_type = "text/plain"

[cmd.find.aliases]
dl = "~/Downloads"

[cmd.pnpm]
autorun = true               # zsh: 規則だけで決まった risk "none" の行はすぐ実行する
autorun_also = ["pnpm install"]   # この語で始まる行は "unsafe" でも実行する
```
