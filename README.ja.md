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
- **出力** — コマンド行は stdout、それ以外は stderr。語を解釈できない、または確信度が下限を下回ってコマンドを組めないときは、非ゼロで終わり、代わりに `jany <command> --hint` を理由のコメント付きでプロンプトに置く。`dangerous` な結果(find の `-delete`)は先に read-only で実行してプレビューする。`unsafe` なもの(状態を変えるもの: curl の `POST` / `DELETE`、`docker run`)には 1 行の注意が付き、`autorun` でも入力行に置く。

jev の呼び出しは 1 回 200〜700 ms、$0.0001 未満。

## セットアップ

```sh
cargo install jany
jany --skills --locale ja      # エージェント用のスキルを置く
```

あとは Claude Code か Codex で `/jany-setup` を呼ぶ。どのシェルで使うかを聞いて、その rc に `jany --init` の行を書き、zsh なら `jany --on` にするかも聞く。

手で設定するとき、薄い候補・autorun・`jany --on`・API キーのことは [docs/setup.ja.md](docs/setup.ja.md) にある。jany をやめるときは `/jany-teardown` が置いたものを片付ける。

## 使い方

```
jany <command> [words ...] [flags] [-- passthrough args]
```

`--explain` で語ごとにどう決まったか、`--hint` でそのコマンドに何が言えるかを出す。`--` の後ろはそのまま素通し。フラグの全部、jany 自身の操作(`--list`、`--test`、`--update`、`--on` など)、`config.toml` は [docs/usage.ja.md](docs/usage.ja.md) にある。

## コマンドを足す

```sh
/jany-register tar            # Claude Code か Codex で: 定義を作って埋め、テストまで通す
jany --update                 # jany を上げた後に(一覧に出た定義は /jany-update <name>)
```

コマンドは `~/.config/jany/cmd/<name>/` にある 3 ファイルのディレクトリ。中身と、新しくするやり方は [docs/commands.ja.md](docs/commands.ja.md) にある。

## 弱いところ

- jind と jurl と同じ弱さがある: 裸の語(`log`、`app`)は 2 つの役割のどちらにもなりえて、jev の確信度は 0.6〜0.9 にとどまることが多い。迷いようのない形(`*.log`)で書けば jev を飛ばせる。
- 語彙が閉じないコマンド(任意の SQL、jq のプログラム、ffmpeg のフィルタグラフ)には向かない。よく使う形だけ載せて、残りは素通しする。
- 規則で決まった語も含め、全部の語が文脈として jev に送られる。秘密を含みうる役割(curl のヘッダ、docker の `KEY=value`)は `mask` を宣言し、プレースホルダだけが外に出るようにしている。

---

<p align="center"><sub>姉妹プロジェクト: <a href="https://github.com/yukihirop/jind">jind</a>(jev × find)と <a href="https://github.com/yukihirop/jurl">jurl</a>(jev × curl)。jany はこの 2 つを汎化したもの。<code>examples/</code> に組み込みの定義が、<code>docs/HOST.md</code> にホストがすることと schema がすることの境界がある。<code>src/</code> の各モジュールは、何をするかのコメントから始まる。<code>docs/demo.svg</code> は <code>docs/capture-demo.py</code> が pty 越しに取った実際の出力を <code>docs/make-demo.py</code> で描いたもの。</sub></p>
