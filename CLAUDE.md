# jany(旧 jx)— 引き継ぎ(2026-09-22)

jind(jev × find、`../jind`)と jurl(jev × curl、`../jurl`)を汎化した 1 本の CLI。`jany find …` `jany curl …` `jany docker run …` `jany sql …` のように、コマンドごとの定義を `~/.config/jany/cmd/<name>/` から読み込んで、同じ流れ(規則 → jev → repair → 組み立て → 確認 → 実行)で動かす。動機は「jind, jurl, jocker, jql … とコマンドが増えるたびにバイナリを書くのが面倒」。コマンドの追加は `jany --register <name>` + Claude Code / Codex のスキル(`/jany-register <name>`)で定義ファイルを生成させる想定。

## いまの状態

- **`design/` → `examples/`、`HOST.md` → `docs/`**(2026-09-22、ユーザー選択): 「design」はコードが無かった頃の名残。`examples/{find,curl,docker/run}` は組み込み定義の原本で、バイナリに埋め込まれ、スキルの examples と `--init` の配布元を兼ねる。あなたの定義(`~/.config/jany/cmd/tar` など)はリポジトリに入れない

- **2026-09-22 に `jx` → `jany` に改名**(crates.io に `jx` `jv` が既にあった。`jany` = jev × any、空きを確認して /ask で選択)。crate / バイナリ / `~/.config/jany` / `JANY_*` / `/jany-register` / `JanyError` を一括置換。GitHub リポジトリ名とローカルのディレクトリ名は別途

- **Rust ホストが動く**(2026-09-22)。`src/` 16 ファイル。`JANY_CMD_DIR=examples cargo run -- test find` 8/8、`test curl` 30/30。実機で jev を呼んで `find empty folders depth 2 count` → `find . -maxdepth 2 -type d -empty | wc -l` が stdout に出ることを確認。リモートは `https://github.com/yukihirop/jany`(2026-09-22 に private で作成。2026-09-23 時点で public、切り替えた日は確かめていない)。README / LICENSE あり
- `src/` の由来: `jev/{mod,client}.rs` `color.rs` `setup.rs` `config.rs` は jind からほぼコピー。`rules.rs` `questions.rs` `repair.rs` `amount.rs` `schema.rs` `assemble.rs` `interpret.rs` `testrun.rs` `init.rs` は schema 駆動で書き直したもの。jind/jurl は参考であって依存ではない
- `jany --init zsh` は**実機で確認済み**(2026-09-22、ユーザー): `.zshrc`(実体 `~/dotfiles/.zshrc`)に `eval "$(jany --init zsh)"` を入れ、`jany find log files older than 7 days` → 次のプロンプトに `find . -type f -iname '*.log' -mtime +7` が載った。`jany --list` などjany 自身の操作はラッパーが素通しする。bash の `\e[5n` トリックは `bash -n` で構文だけ確認、fish は手元に無く未確認
- **`/jany-register` スキルと `jany --register`**(2026-09-22): スキル本体は `skills/{en,ja}/jany-register/{SKILL.md, reference.md, template/}` にあり(2026-09-23 に en / ja の 2 つにした。`jany --init <shell> --locale ja` / `jany --register … --locale ja` で選ぶ。既定は en。ja が原本で en はその訳なので、直すときは両方直す)、`include_str!` でバイナリに埋め込む(examples は `examples/find` `examples/curl` そのもの)。`jany --init <shell>` がラッパーを stdout に出すついでに `~/.agents/skills/jany-register/` に書き(`JANY_SKILL_DIR` で変更可、中身が同じなら何もしない)、`~/.claude/skills/` `~/.codex/skills/` が既にあってその名前が無ければ symlink を置く。**Codex が `~/.agents/skills` を読むかは確かめていない**(symlink はそのための保険)。`jany --register <name> [sub]` は雛形 3 ファイルを置くだけ(既存があれば止まる)。**組み込みの 3 定義(find / curl / docker run)も `jany --init` が `~/.config/jany/cmd/` に置く**(`examples/` を `include_str!` で埋め込み。`schema.toml` が既にあるディレクトリは触らないので、examples/ を直したら `jany --init` では更新されない。手で消すかコピーする)。`examples/` はリポジトリ内の原本で、実行時に読むのは `~/.config/jany/cmd/`
- **`examples/docker/run/`**(2026-09-22): スキルの手順で書いた 3 つ目の定義。cases 16/16。書いて分かったことは `docs/HOST.md`「docker run を書いて分かったこと」(ホストに足したのは `next` の table 補正 1 つ)
- clippy の警告は 0 件(CI が `-D warnings` で回す)。`src/` は `.rs` が 23 ファイル(`jev/` の 2 つを含む)
- **`src/` のコメントは英語**(2026-09-23、ユーザー指示)。新しく書くコメントも英語にする。会話・CLAUDE.md・docs/HOST.md・skills/ja は日本語のまま
- stdout への書き出しは `output::stdout()` を通す(2026-09-23)。`print!` はパイプが閉じると panic する(`jany --list | head -1`)
- **`jany <cmd> --hint`**(2026-09-23): `[[roles]]` の `jev` 説明と cases.toml の words(error / setup / defaults 付きを除く先頭 8 本)を stderr に出す。`src/hint.rs`。**変換に失敗したとき**(Unresolved / LowConfidence / Assemble)は stderr にエラー、stdout に `jany find --hint  # could not interpret: edtied, wthin` を出して非ゼロ終了し、ラッパーは非ゼロでも stdout があれば入力行に置く(2026-09-23、ユーザー選択)。コメント部は語ごとに shell-quote し、jany の引数解析は `#` の語で止まる(zsh 既定の interactivecomments off 対策、`zsh -f` で確認済み)
- **zsh の薄い候補**(2026-09-23、ユーザー選択: 「次に来る役割のプレースホルダ」、既定で有効): schema の `[[placeholders]]`(`text` / `roles` / `bare`)を、`jany --suggest -- <語>` が規則 + repair だけで評価し(jev は呼ばない)、埋まっていない枠を返す。`src/suggest.rs`。zsh ラッパーが `add-zle-hook-widget line-pre-redraw` で `POSTDISPLAY` に fg=8 で出す。`jany ` か jany の alias で始まり末尾が空白の行だけ、`config.toml` の `[suggest] enabled = false` で消え、`JANY_SUGGEST=0/1` がシェル単位で上書きする(2026-09-23、ユーザー選択「両方」。0 はラッパーが見て jany を起動しない)。`zpty` で表示・alias・ls で出ないこと・Enter で消えることを確認した(実端末の目視はしていない)。bash / fish は無し。examples の 3 定義に書いた。**`~/.config/jany/cmd/` の組み込み定義は `jany --init` では更新されない**。`jany --update` で入れ替える(次の項)
- **`jany --update` と `/jany-update` スキル**(2026-09-23、ユーザー指示「jany-update コマンドでやる」、方針は「両方やる」): `jany --update [name]` は、手つかずの組み込み定義を置き換える。手つかずかどうかは全ファイルのハッシュが `examples/released.txt` にあるかで判定する(git 履歴の全版から作った 31 行。**examples/ を直したら `cargo test` が落ちて足す行を出す。行は消さない**)。編集済みの組み込み定義と自作の定義には、足りない機能(`skill::missing_features`)と `/jany-update <name>` を出す。スキルは `skills/{en,ja}/jany-update/SKILL.md` の 1 ファイルで、reference.md は隣の `../jany-register/` を読む。`--init` が jany-register の隣(`JANY_SKILL_DIR` の親)に置き(`--update` も無いファイルだけ置く。言語は `--locale`、無ければ置いてある jany-register/SKILL.md に仮名があれば ja。2026-09-23、ユーザー指示)、`~/.claude/skills` などへの symlink も張る。スキルの examples に docker/run を足した。scratch(実定義のコピー)で `--update` の置き換え・編集検出・2 回目の up to date を確認済み。**`/jany-update` をスキルとして呼んで試してはいない**
- **`[cmd.<name>] autorun`**(2026-09-23、ユーザー選択: コマンドごと / `--` の後ろがあれば実行しない / 実行は zsh ラッパーの eval。4 問とも推奨案): config.toml で on にしたコマンドは、規則だけで決まり(jev 無し)・`risk = "none"`・`--` の後ろも `passthrough` 役割の語も無し・preview / pipe 無し、のときだけ jany が終了コード 3 を返し、ラッパーが `print -s` で履歴に入れて `eval` する。条件は `src/interpret.rs` の `Run::autorun_safe`。合図はラッパーが付ける `JANY_CAN_RUN=1` があるときだけ(bash / fish は付けないので常に入力行)。jany 自身は spawn しない。**`risk = "none"` が「確認なしで走ってよい」の意味を持つようになった**ので、状態を変えるものは `unsafe` にする(reference.md en/ja に追記)。scratch の zsh -f -i で run の実行・unsafe と `--` で止まること・履歴を確認。**実端末では試していない**。同日、語が 0 個で `--` の後ろが `--help` / `--version` の 1 語だけなら実行してよいことにした(ユーザー選択、推奨案。`jpnpm -- --version` が入力行に置かれたため)。さらに `autorun_also = ["pnpm install"]`(argv の先頭を語単位で比べる。unsafe まで、dangerous は不可)を足した(ユーザー選択、推奨案。`[cmd.pnpm.install]` の形は、jany が install を知らない・`install react` が add に化ける・サブコマンド定義と紛らわしい、で退けた)。curl(POST が none)と docker run(起動が none)の risk は見直していないので、この 2 つは autorun を on にしない方がよい
- **alias でも薄い候補と補完**(2026-09-23、ユーザー選択。`feat/autorun` に同乗): zsh ラッパーの `_jany_words` が行の先頭の alias を最大 5 段展開する(`jpnpm='j pnpm'` → `jany pnpm`)。薄い候補と `_jany_complete` がこれを使い、`_jany_compdef_aliases` が最初の precmd で jany に行き着く alias 全部に `compdef` を張る(rc の alias はラッパーの後に書かれるため)。`zsh -f` で展開・`--suggest`・`_jany_complete` を直接呼ぶ形で確認。**実端末で Tab を押して確かめてはいない**(zpty での駆動は読み取りが止まって断念)
- **`jany --on` / `--off`**(2026-09-24、ユーザー指示「mahojin の --on / --off を参考に」、選択は推奨案「jany を付けずに打てる」): mahojin(`~/RustProjects/mahojin/src/shell.rs`)と同じ形。zsh ラッパーの `jany()` が引数 1 個の `--on` / `--off` だけを横取りして `_JANY_ON` を立てる/消す。`accept-line` を `_jany_accept_line` で包み(元が user widget なら `_jany_accept_line_orig` に退避して呼ぶ)、on の間は Enter で `jany --claim -- <語>`(`src/claim.rs`、終了コード 0/1、config も jev も読まない)が 0 を返した行の頭に `jany ` を足す。履歴には `jany find …` が残る。引き受けるのは「定義のあるコマンドで始まり、その後に語がある」行だけで、`-` で始まる語・`|` `&&` `;` `>` `2>` `<(…)`・コマンド単体(`find`)・定義の無いサブコマンド(`docker ps`)・先頭の語がクォートされた行(`\find` `'find'` `command find`)は素通し。on の間は薄い候補も `find src ` に出る(`-` の語がある行は出さない)。バイナリまで届いた `--on` / `--off`(ラッパー無し、bash / fish)は「zsh のラッパーで使う」と言って 64 で終わる。`script` で擬似端末を作り `zsh -f -i` に打鍵を流して、書き換え・素通し・`--off`・履歴・薄い候補を確認した(zpty は今回も読み取りで止まった)。**実端末では試していない**。**`pnpm install` `kubectl get pods` のように `-` の無い普段の打ち方も jany に回る**(autorun が効かなければ入力行に置かれてもう 1 回 Enter)。除外リスト(mahojin の `chant_skip` 相当)は作っていない。同じ日に、新しい clippy で `src/questions.rs:136` が `nonminimal_bool` で落ちるようになっていたので直した
- **組み込みの risk を autorun に合わせた**(2026-09-23、ユーザー選択「risk を直してから 0.4.0」): curl は GET / HEAD / OPTIONS 以外を `unsafe`(POST も)、docker run は常に `unsafe`(起動そのものが状態を変える)。`unsafe_note` も言い直し、cases 31 本に `risk = "unsafe"` を足し、released.txt に 6 行足した。手元の `~/.config/jany/cmd/{curl,docker/run}` は `jany --update` で入れ替わる(手を入れていなければ)
- **リリースは main へのマージで走る**(2026-09-23、ユーザー指示「バンプして main に merge したらすぐ release」): `.github/workflows/ci.yml` の `release` ジョブが、`check` が通った main への push で `Cargo.toml` の版が crates.io に無ければ `cargo publish`、`v<版>` タグと GitHub Release を作る。各段は先に有無を見るので、失敗しても re-run で続きから。認証は crates.io の Trusted Publishing(`rust-lang/crates-io-auth-action@v1`、トークンを secret に置かない)。**crates.io の jany の Settings → Trusted Publishing に `yukihirop` / `jany` / `ci.yml` を登録しておく必要がある**(登録したかは確かめていない)。手順は `release/<版>` ブランチで版を上げて PR → マージ
- 2026-09-22 に `~/JavaScriptProjects/jany` から `~/RustProjects/jany` へ移動した(ホストを Rust にすると決めたため)
- `examples/find/` は jind 0.1.0 を定義ファイル 3 つに書き直したもの。`assemble.sh` は実際に動かして jind の README の例 6 本 + 衝突 1 本で同じ argv が出ることを確認済み
- `examples/curl/` は jurl 0.1.2 を同じ 3 つに書き直したもの(2026-09-22)。`assemble.sh` は jurl の `-n --no-jev` 実出力 16 本 + interpret.rs / EXAMPLES の jev 例 7 本で同じ argv。規則は Python で最小エンジンを書いて `jurl --explain` の role 列と 18 本一致(scratch、リポジトリには入れていない)。DSL に足したものは `docs/HOST.md`「curl を書いて分かったこと」

## 決まっていること(2026-09-22、ユーザーが選択)

| 論点 | 決定 | 理由 |
|---|---|---|
| ホストの言語 | Rust | jind/jurl の `jev/{mod,client}.rs` `output.rs` `color.rs` `setup.rs` `config.rs` をほぼそのまま流用できる(diff は数十行) |
| コマンド固有の組み立て | 外部スクリプト(stdin JSON → stdout JSON) | `/jany-register` で LLM に書かせやすく、jany の再ビルドが不要。find は bash + jq で 84 行 |
| 規則・repair | schema.toml のデータ + ホスト組み込みプリミティブ | 「全部規則で決まれば jev を呼ばない」判断をホストがするため、規則をスクリプトにしない |
| 最初に載せるもの | jind(find)→ jurl(curl)→ docker run / sql の順 | 既存のテストと実機で通した例が正解になる |
| サブコマンド | ディレクトリ階層(`cmd/docker/run/`) | jev の Choice に見せる役割を 10〜20 個に抑える |

## 構成(予定)

```
~/.config/jany/cmd/<name>[/<sub>]/
  schema.toml    roles(jev に見せる説明文)、amount の単位表、語テーブル、rules、questions、repair、confirm
  assemble.sh    役割付きトークン JSON → {argv, preview, risk, pipe, error}。実行権限が要る(jany は直接 exec する)
  cases.toml     words → argv のテスト。jev の答えは Mock で書く。`jany --test <name>` が回す
```

ホストが持つ部品と schema が持つものの境界は `docs/HOST.md` に表でまとめてある。要点:

- ホスト組み込みの match: `prefix / word / table / builtin(path_like, glob, existing_dir, amount, unit_word)`、when: `prev_role / prev_is_number / not_amount / has_unit / has_direction`
- repair プリミティブ: `attach_unit`、`claim {marker, role, side, many, from, skip, require, clear}`。jind の 3 本はこれで書けた
- `risk = "dangerous"` を assemble が返したら `-y` でも必ず確認・既定 No・`preview` argv を先に回して `preview_lines` 件見せる(jind の delete の安全弁を一般化したもの)。`risk = "unsafe"`(jurl の PUT/PATCH/DELETE)は閾値を `confirm_below_unsafe` に上げるだけで `-y` は効く
- **jany はコマンドを実行しない**(決定 2026-09-22、ユーザー): 未知のコマンドを扱うので、Y の後に spawn せず、shell-quote した 1 行を stdout に出し、`eval "$(jany --init zsh)"` のラッパー(`print -z`)がシェルの入力行に置く。Enter は人が押す。`-y` / `confirm_below` / `[Y/n/e]` / `$EDITOR` は無し、`reject_below` だけ残す。`postprocess = "count_lines"` は `pipe = ["wc", "-l"]` に(`find … | wc -l` を入力行に載せる)。`risk = "dangerous"` の preview だけ read-only の実行として残す(schema に `preview_readonly = true` を書かせる)。詳細は `docs/HOST.md`「出力」行

## 次にやること

1. ~~jurl を `examples/curl/` に書き直す~~ 済(2026-09-22)。予想どおり `pair` / `join` の 2 プリミティブ、2 語規則は `next`、have_method/have_url は `once`、Header は role の `mask` で表した
2. ~~jurl の出力側を jany に持ち込むか~~ 決定(2026-09-22、ユーザー): **持ち込まない**。jany curl は curl の argv を作って実行し stdout をそのまま出す。整形は `| jq`。jurl 本体は残るので機能が消えるわけではない
3. ~~Rust ホストを書く~~ 済(2026-09-22)。find 8 + curl 30 の cases が通る。直したこと: cases が chdir するので `Schema.dir` は canonicalize、サブコマンド解決は `/` や `.` を含む語で止める(`jany find /var/log …` が `examples/find//var/log` を探しに行った)、find の cases 2 本を直した(delete に `risk`/`preview` が無かった、"mp4" は英字だけでないので typo 質問は聞かれない = jind `prompt.rs:93` と同じ)
4. `jany --init zsh` を対話シェルで試す(`eval "$(jany --init zsh)"` を .zshrc に入れて `jany find …` → 入力行に載るか)
5. ~~`/jany-register` スキルを書き、docker run で試す~~ 済(2026-09-22)。ただし試したのは「Claude Code がスキルの手順に沿って自分で書く」であって、`/jany-register docker run` をスキルとして呼んだわけではない。**未検証: 実際に `jany --init zsh` を本物の HOME で実行してスキルが Claude Code / Codex に見えるか、`/jany-register <name>` で一発で通る定義が出るか**
6. ~~インストールと実機確認~~ 済(2026-09-22)。`cargo install --path .`、`.zshrc` に `eval "$(jany --init zsh)"`、`~/.config/jany/cmd/{find,curl,docker/run}` と `~/.agents/skills/jany-register`(+ `~/.claude/skills` `~/.codex/skills` の symlink)ができ、Claude Code のスキル一覧に `jany-register` が出た。Codex 側は未確認
7. 4 つ目のコマンド(ユーザーがよく打つもの)で `/jany-register <name>` を本当に呼び、一発で cases が通る定義が出るか試す。出なければスキルの手順か reference.md を直す

## 分かっていること・注意

- 規則で決まった語も jev の state.tokens に入る。秘密を含みうる役割(curl の header、docker の env)には `mask` を付ける
- `jany --test` の Mock は「cases に書いた答えのうち jany が聞かなかったキーがあれば失敗」にしてある。質問の when とフィクスチャのずれに気づくため。逆に聞かれたのに答えが無いキーは未回答のまま(未解決になれば error で落ちる)
- jq の `//` は `false` を「無い」扱いにする。`assemble.sh` の初版で `within an hour` が `-mmin +60` になった(正しくは `-60`)。**LLM が書く assemble.sh は cases.toml 無しで信用しない**
- assemble の契約は curl で変えた: `dangerous: bool` → `risk: "none"|"unsafe"|"dangerous"`、`defaults: [...]` → `defaults: {args, ...}`、stdin に `answers`(command scope の質問の答え)を追加。find の 3 ファイルも合わせて直してある
- jurl 0.1.2 のバグを 1 つ見つけた: `http://example.com/x?a=1` が規則で未解決になる(`=` 分岐が URL より先)。jany の schema では直っている。jurl 側は直していない
- `cargo package` の後、`target/debug/jany` の依存情報が `target/package/jany-0.2.0/src/…` を指したまま残り、`src/` を直しても `cargo build` が Fresh と言って作り直さなかった(2026-09-23)。`cargo clean -p jany` で直った
- `repair.attach_unit` は単位の次元ごとに役割を 1 つに決め直す(`src/repair.rs:35` の `role_for_dimension`)。同じ次元の役割が複数あるコマンド(ffmpeg の start / end / duration)では使えず、ffmpeg の定義では assemble.sh で単位を付けた。ホストを直すかは未決
- jev の弱さは汎化しても直らない: 裸の語が extension か name_word かで 0.6〜0.9 に割れる(jind CLAUDE.md「弱いところ」)。SQL の table/column でも同じことが起きる見込み(未検証)
- 役割集合が閉じないコマンド(任意の SQL、jq の式、ffmpeg のフィルタ)は対象外。jind が `-mtime` だけに割り切ったのと同じ姿勢で「よく使う 8 割 + `--` 素通し」にする
- `sql` はバイナリ名ではないので、assemble.sh が argv の先頭(`psql -c` など)も決める。jany の「コマンド名」は schema の名前であってバイナリ名ではない
- 3 つ目・4 つ目のコマンドで repair プリミティブが足りなくなり Rust を触ることは織り込む

## 一次資料

- jind: `../jind`(`CLAUDE.md` に構成と実機で確認した挙動)。crates.io 0.1.0
- jurl: `../jurl`(`EXAMPLES.md` あり)。crates.io 0.1.2
- jev の API / モデル ID: `~/JavaScriptProjects/eg-jev`(`packages/recipes/src/lib/{openrouter,questions}.ts`)

## 作業ルール(ユーザーから)

- 日本語で答える。事実と推測を分ける。確かめていないことは「確かめていない」と書く
- 公開・publish・削除・push・リポジトリの移動は必ず事前に確認
- 方針の分岐は `/ask` で推奨付きで聞く(これまで 6 問すべて推奨案が選ばれている)
- commit 末尾に `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>`
