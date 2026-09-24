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

デモの最後の行は出力ではない。入力済みの次のプロンプトだ。読んで、必要なら直して、Enter は自分で押す。

## はじめ方

| | |
|---|---|
| **始める** | `cargo install jany && jany --skills --locale ja` のあと、Claude Code か Codex で **`/jany-setup`** |
| **使う** | `jany find log files older than 7 days` |
| **コマンドを足す** | **`/jany-register tar`** |
| **jany を上げたら** | `jany --update`。一覧に出た定義は `/jany-update <name>` |
| **やめる** | **`/jany-teardown`** |

`/jany-setup` はどのシェルで使うかを聞いて、その rc に `jany --init` の行を書く。`/jany-teardown` は jany が置いたものを片付ける。

## もっと詳しく

- [例](docs/examples.ja.md): 打つ語と、出てくる 1 行。壊すコマンドのプレビューも
- [仕組み](docs/how-it-works.ja.md): 規則、jev、repair、assemble.sh、弱いところ
- [セットアップ](docs/setup.ja.md): 手での設定、薄い候補、autorun、`jany --on`、API キー、やめ方
- [使い方](docs/usage.ja.md): フラグ、jany 自身の操作、`config.toml`
- [コマンドの定義](docs/commands.ja.md): 定義の中身、足し方と新しくし方

---

<p align="center"><sub>姉妹プロジェクト: <a href="https://github.com/yukihirop/jind">jind</a>(jev × find)と <a href="https://github.com/yukihirop/jurl">jurl</a>(jev × curl)。jany はこの 2 つを汎化したもの。<code>examples/</code> に組み込みの定義が、<code>docs/HOST.md</code> にホストがすることと schema がすることの境界がある。<code>src/</code> の各モジュールは、何をするかのコメントから始まる。<code>docs/demo.svg</code> は <code>docs/capture-demo.py</code> が pty 越しに取った実際の出力を <code>docs/make-demo.py</code> で描いたもの。</sub></p>
