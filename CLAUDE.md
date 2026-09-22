# jx — 引き継ぎ(2026-09-22)

jind(jev × find、`../jind`)と jurl(jev × curl、`../jurl`)を汎化した 1 本の CLI。`jx find …` `jx curl …` `jx docker run …` `jx sql …` のように、コマンドごとの定義を `~/.config/jx/cmd/<name>/` から読み込んで、同じ流れ(規則 → jev → repair → 組み立て → 確認 → 実行)で動かす。動機は「jind, jurl, jocker, jql … とコマンドが増えるたびにバイナリを書くのが面倒」。コマンドの追加は `jx register <name>` + Claude Code / Codex のスキル(`/jx-register <name>`)で定義ファイルを生成させる想定。

## いまの状態

- コードはまだ無い。`PLAN.md`(最初のメモ)と `design/`(設計草案)だけ。コミット 1 件(`91bf0a7`)、リモート無し。`CLAUDE.md` と `design/curl/` は未コミット
- 2026-09-22 に `~/JavaScriptProjects/jx` から `~/RustProjects/jx` へ移動した(ホストを Rust にすると決めたため)
- `design/find/` は jind 0.1.0 を定義ファイル 3 つに書き直したもの。`assemble.sh` は実際に動かして jind の README の例 6 本 + 衝突 1 本で同じ argv が出ることを確認済み
- `design/curl/` は jurl 0.1.2 を同じ 3 つに書き直したもの(2026-09-22)。`assemble.sh` は jurl の `-n --no-jev` 実出力 16 本 + interpret.rs / EXAMPLES の jev 例 7 本で同じ argv。規則は Python で最小エンジンを書いて `jurl --explain` の role 列と 18 本一致(scratch、リポジトリには入れていない)。DSL に足したものは `design/HOST.md`「curl を書いて分かったこと」

## 決まっていること(2026-09-22、ユーザーが選択)

| 論点 | 決定 | 理由 |
|---|---|---|
| ホストの言語 | Rust | jind/jurl の `jev/{mod,client}.rs` `output.rs` `color.rs` `setup.rs` `config.rs` をほぼそのまま流用できる(diff は数十行) |
| コマンド固有の組み立て | 外部スクリプト(stdin JSON → stdout JSON) | `/jx-register` で LLM に書かせやすく、jx の再ビルドが不要。find は bash + jq で 84 行 |
| 規則・repair | schema.toml のデータ + ホスト組み込みプリミティブ | 「全部規則で決まれば jev を呼ばない」判断をホストがするため、規則をスクリプトにしない |
| 最初に載せるもの | jind(find)→ jurl(curl)→ docker run / sql の順 | 既存のテストと実機で通した例が正解になる |
| サブコマンド | ディレクトリ階層(`cmd/docker/run/`) | jev の Choice に見せる役割を 10〜20 個に抑える |

## 構成(予定)

```
~/.config/jx/cmd/<name>[/<sub>]/
  schema.toml    roles(jev に見せる説明文)、amount の単位表、語テーブル、rules、questions、repair、confirm
  assemble.sh    役割付きトークン JSON → {argv, preview, dangerous, postprocess, error}
  cases.toml     words → argv のテスト。jev の答えは Mock で書く。`jx test <name>` が回す
```

ホストが持つ部品と schema が持つものの境界は `design/HOST.md` に表でまとめてある。要点:

- ホスト組み込みの match: `prefix / word / table / builtin(path_like, glob, existing_dir, amount, unit_word)`、when: `prev_role / prev_is_number / not_amount / has_unit / has_direction`
- repair プリミティブ: `attach_unit`、`claim {marker, role, side, many, from, skip, require, clear}`。jind の 3 本はこれで書けた
- `risk = "dangerous"` を assemble が返したら `-y` でも必ず確認・既定 No・`preview` argv を先に回して `preview_lines` 件見せる(jind の delete の安全弁を一般化したもの)。`risk = "unsafe"`(jurl の PUT/PATCH/DELETE)は閾値を `confirm_below_unsafe` に上げるだけで `-y` は効く
- `postprocess = "count_lines"`(jind の `count`)

## 次にやること

1. ~~jurl を `design/curl/` に書き直す~~ 済(2026-09-22)。予想どおり `pair` / `join` の 2 プリミティブ、2 語規則は `next`、have_method/have_url は `once`、Header は role の `mask` で表した
2. ~~jurl の出力側を jx に持ち込むか~~ 決定(2026-09-22、ユーザー): **持ち込まない**。jx curl は curl の argv を作って実行し stdout をそのまま出す。整形は `| jq`。jurl 本体は残るので機能が消えるわけではない
3. DSL が固まったら Rust ホストを書く。`cargo init`、jind/jurl から共通ファイルをコピー、`design/find` と `design/curl` の cases が通るまで。regex は regex crate、TOML は toml crate
4. `/jx-register` スキルを書き、docker run で LLM 生成を試す

## 分かっていること・注意

- jq の `//` は `false` を「無い」扱いにする。`assemble.sh` の初版で `within an hour` が `-mmin +60` になった(正しくは `-60`)。**LLM が書く assemble.sh は cases.toml 無しで信用しない**
- assemble の契約は curl で変えた: `dangerous: bool` → `risk: "none"|"unsafe"|"dangerous"`、`defaults: [...]` → `defaults: {args, ...}`、stdin に `answers`(command scope の質問の答え)を追加。find の 3 ファイルも合わせて直してある
- jurl 0.1.2 のバグを 1 つ見つけた: `http://example.com/x?a=1` が規則で未解決になる(`=` 分岐が URL より先)。jx の schema では直っている。jurl 側は直していない
- jev の弱さは汎化しても直らない: 裸の語が extension か name_word かで 0.6〜0.9 に割れる(jind CLAUDE.md「弱いところ」)。SQL の table/column でも同じことが起きる見込み(未検証)
- 役割集合が閉じないコマンド(任意の SQL、jq の式、ffmpeg のフィルタ)は対象外。jind が `-mtime` だけに割り切ったのと同じ姿勢で「よく使う 8 割 + `--` 素通し」にする
- `sql` はバイナリ名ではないので、assemble.sh が argv の先頭(`psql -c` など)も決める。jx の「コマンド名」は schema の名前であってバイナリ名ではない
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
