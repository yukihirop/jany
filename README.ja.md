<p align="center">
  <img src="docs/hero.svg" alt="jany — jev × any command. Say what you want, in any order. Get the command you meant, on your prompt." width="880">
</p>

<p align="center">
  <a href="https://crates.io/crates/jany"><img src="https://img.shields.io/crates/v/jany.svg" alt="crates.io"></a>
  <a href="https://crates.io/crates/jany"><img src="https://img.shields.io/crates/d/jany.svg" alt="downloads"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT"></a>
  <a href="https://github.com/yukihirop/jany/actions/workflows/ci.yml"><img src="https://github.com/yukihirop/jany/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/commands-find%20%C2%B7%20curl%20%C2%B7%20docker%20run%20%C2%B7%20yours-3b6fd1.svg" alt="find · curl · docker run · yours">
</p>

<p align="center">
  <a href="README.md">English</a> · <b>日本語</b>
</p>

<p align="center">
  <b>jany</b> は、順番がばらばらで、うろ覚えで、typo まじりの語の並びを、意図したコマンド行に直してプロンプトに置く。<b>実行はしない</b>(コマンドごとに on にしたときを除く)。Enter を押すのはあなた。
</p>

> [!NOTE]
> **ステータス: 実験的。** コマンド定義とシェル連携は 1.0 までに変わることがある。

<p align="center">
  <img src="docs/demo.svg" alt="ターミナルのデモ: 'find log files older than 7 days in logs delete --explain' で語ごとの表と削除対象の read-only プレビューが出て、find が次のプロンプトに載る。'curl psot localhsot 3000 users first_name amanda' が JSON ボディ付きの POST になる。'docker run nginx 8080:80 background named web'。'tar extrct app.tar.gz into dist strip 1' はまずアーカイブの中身を一覧する" width="930">
</p>

```sh
$ jany find log files older than 7 days in logs delete
this command is destructive.
$ find logs -type f -iname '*.log' -mtime +7
  logs/old-access.log
  logs/kernel.log
  logs/system.log
  … 2 more

$ find logs -type f -iname '*.log' -mtime +7 -delete█
```

最後の行は出力ではない。**入力済みの次のプロンプト**だ。jany は shell-quote した 1 行を stdout に出し、`jany --init zsh` のラッパーがそれを入力行に置く。読んで、必要なら直して、Enter を押す。

```sh
$ jany curl psot localhsot 3000 users first_name amanda      # typo、裸のポート番号、2 語に分かれた key value
$ curl -sS -X POST http://localhost:3000/users -H 'Content-Type: application/json' -H 'Accept: application/json' --data '{"first_name":"amanda"}'

$ jany docker run nginx 8080:80 background named web
$ docker run -d --name web -p 8080:80 nginx

$ jany tar extrct app.tar.gz into dist strip 1                # /jany-register スキルが書いた定義
$ tar -xf app.tar.gz --strip-components 1 -C dist

$ jany find empty folders depth 2 count
$ find . -maxdepth 2 -type d -empty | wc -l
```

jany は [jind](https://github.com/yukihirop/jind)(jev × find)と [jurl](https://github.com/yukihirop/jurl)(jev × curl)を 1 本のバイナリに汎化したもの。コマンドごとに **定義ディレクトリ**(ファイル 3 つ、Rust は書かない)があり、コマンドを足すのは新しい crate ではなく LLM スキルの仕事になる。

## 仕組み

<p align="center">
  <img src="docs/flow.svg" alt="語 → 規則 → 全部決まった? はい: assemble.sh → プロンプト。いいえ: jev(1 リクエスト)→ repair → assemble.sh → プロンプト。破線の箱はコマンドごとの定義、実線はホスト。" width="880">
</p>

- **規則** — 迷いようのない形は `schema.toml` のデータで決まる: パス、glob、`8080:80`、`KEY=value`、`+7d`、`>10M`、`files` / `delete` / `background` のような表の語。全部の語が決まれば、jany は手元から外に出ない。
- **jev** — 残った語は [jev](https://openrouter.ai)(TypeSafe System One、OpenRouter 経由)に **1 リクエスト** で聞く: 「それぞれの語の役割は何か」と、schema が宣言した追加の質問(単位はどれか、以上か以下か、どの表の語の typo か)。jev は決められた選択肢から選んで確率を返すだけで、コマンドを生成はしない。
- **repair** — 語ごとの jev には見えないことを直す: `days` を `7` に付ける、`except` の後ろの名前を除外に取る、`first_name amanda` を key と value の組にする。
- **assemble.sh** — コマンド自身のスクリプト(stdin JSON → stdout JSON)が、役割の付いた語から `argv` を組む。結果がどれだけ危ないかもここで返す。
- **出力** — コマンド行は stdout、それ以外は stderr。語を解釈できない、または確信度が下限を下回ってコマンドを組めないときは、非ゼロで終わり、代わりに `jany <command> --hint` を理由のコメント付きでプロンプトに置く。`dangerous` な結果(find の `-delete`)は先に read-only で実行してプレビューする。`unsafe` なもの(curl の `DELETE`、docker の `--privileged`)には 1 行の注意が付く。

jev の呼び出しは 1 回 200〜700 ms、$0.0001 未満。

## セットアップ

```sh
cargo install jany          # crates.io に出た後
# 出る前は、このチェックアウトから入れる:
cargo install --path .
jany --setup                  # OpenRouter の API キーを ~/.config/jany/config.toml(0600)に保存
echo 'eval "$(jany --init zsh --locale ja)"' >> ~/.zshrc     # bash と fish もある。bash は未検証
# 任意: `j` を `jany` の個人用エイリアスにし、補完も同じにする
printf '%s\n' "alias j='jany'" 'compdef _jany_complete j' >> ~/.zshrc
```

`jany --init` は 3 つのことをする: ラッパー関数を出力する、組み込みの定義(`find`、`curl`、`docker run`)を `~/.config/jany/cmd/` に置く、`/jany-register` と `/jany-update` のスキルを `~/.agents/skills/` に置く(`~/.claude/skills/` と `~/.codex/skills/` があればそこからリンクする)。すでにある定義は上書きしない。スキルは既定で英語版。日本語版は `--locale ja` で置く。`--init` はシェルを開くたびにスキルを書き直すので、フラグは rc の行に書いておく(上の例のように)。

zsh では、`jany <command> ` の後ろに、まだ言えることを薄く出す(定義の `[[placeholders]]`)。例: `jany find src ` → `<file|dir> <*.log> <older than N days> <delete|count>`。jev は呼ばない。`~/.config/jany/config.toml` に `[suggest] enabled = false` と書くと消える。`JANY_SUGGEST=0` / `1` はそのシェルだけ上書きする。bash と fish には無い。

`~/.config/jany/config.toml` に `[cmd.<name>] autorun = true` を書くと、zsh のラッパーが行を入力行に置かずにそのまま実行する。ただし、全部の語が規則で決まり、定義が risk `"none"` を返したときだけ(jev 無し、`--` の後ろ無し、生の `-x` フラグ無し、preview / pipe 無し)。ほかに何も無い `jany <command> -- --help` / `-- --version` も実行する。`autorun_also = ["pnpm install"]` と書くと、その語で始まる行は定義が `"unsafe"` と言っても実行する(最終的な argv の先頭を語単位で比べるので、`pnpm add react` になる `jany pnpm install react` は当たらない。`"dangerous"` は常に実行しない)。行は stderr に出し、履歴にも残る。それ以外は今までどおり入力行に置く。既定は off。bash / fish は常に入力行。

環境変数の `OPENROUTER_API_KEY` が優先される。`jind setup` や `jurl setup` で保存したキーも拾う。macOS の zsh で確認している。

## 使い方

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
| `jany --init zsh\|bash\|fish` | ラッパーと、組み込み定義とスキル(`--locale en\|ja`、既定は `en`) |
| `jany --setup` | API キーを保存する |

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

## 弱いところ

- jind と jurl と同じ弱さがある: 裸の語(`log`、`app`)は 2 つの役割のどちらにもなりえて、jev の確信度は 0.6〜0.9 にとどまることが多い。迷いようのない形(`*.log`)で書けば jev を飛ばせる。
- 語彙が閉じないコマンド(任意の SQL、jq のプログラム、ffmpeg のフィルタグラフ)には向かない。よく使う形だけ載せて、残りは素通しする。
- 規則で決まった語も含め、全部の語が文脈として jev に送られる。秘密を含みうる役割(curl のヘッダ、docker の `KEY=value`)は `mask` を宣言し、プレースホルダだけが外に出るようにしている。

---

<p align="center"><sub>姉妹プロジェクト: <a href="https://github.com/yukihirop/jind">jind</a>(jev × find)と <a href="https://github.com/yukihirop/jurl">jurl</a>(jev × curl)。jany はこの 2 つを汎化したもの。<code>examples/</code> に組み込みの定義が、<code>docs/HOST.md</code> にホストがすることと schema がすることの境界がある。<code>src/</code> の各モジュールは、何をするかのコメントから始まる。<code>docs/demo.svg</code> は <code>docs/capture-demo.py</code> が pty 越しに取った実際の出力を <code>docs/make-demo.py</code> で描いたもの。</sub></p>
