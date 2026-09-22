# jx ホストが持つもの / schema が持つもの (2026-09-22 時点の設計メモ)

jind 0.1.0 を `design/find/` の 3 ファイルに書き直して確かめた結果。jurl はまだ。

## ホスト (Rust、jind/jurl から流用)

流用元はそのまま: `jev/{mod,client}.rs` `color.rs` `setup.rs` `config.rs` `output.rs` (確認 UI・explain 表・`$EDITOR`)、`main.rs` の流れ、`cli.rs` (`-n` `--explain` `-y` `--no-jev`)。

新しく書くもの:

| 部品 | 内容 | 元 |
|---|---|---|
| schema ローダ | `~/.config/jx/cmd/<name>[/<sub>]/schema.toml` | — |
| Token | `text / role: Option<String> / value / amount / confidence / source / note / probs / tag` | jind `token.rs` の Role を文字列に |
| amount パーサ | `[+->< ]?数[.数]?接尾辞?` を `{n, unit, at_least}` に。単位表と曖昧接尾辞は schema | jind `rules::parse_amount` |
| 規則エンジン | `[[rules]]` を上から評価。match: `prefix / word / table / builtin(path_like, glob, existing_dir, amount, unit_word)`、when: `prev_role / prev_is_number / not_amount / has_unit / has_direction` | jind `rules::classify` |
| 質問ビルダ | `role.i` (jev=あり の roles 全部) + `[[questions]]` の when が当たる語ごとに 1 問。`{i}` `{w}` を埋める | jind `prompt::build` |
| 答えの書き戻し | `role.i` → role/confidence/probs。`applies_to` のある typo 質問は表に無い語の fixed に。amount 役割には unit/at_least を書く | jind `prompt::apply` |
| repair プリミティブ | `attach_unit`、`claim {marker, role, side, many, from, skip, require, clear}` | jind `repair.rs` |
| assemble 呼び出し | stdin に `{tokens, passthrough, defaults}`、stdout の `{argv, preview, dangerous, postprocess, error}` を読む。confidence の min はホストが取る | jind `assemble.rs` + `find.rs` の外側 |
| 確認フロー | `reject_below` / `confirm_below` / `-y`。`dangerous` なら `-y` でも確認・既定 No・preview を先に回して `preview_lines` 件見せる | jind `main.rs execute` |
| postprocess | `count_lines` | jind `Action::Count` |
| テストランナー | `jx test <name>`: cases.toml を Mock Oracle で回す。`setup.dirs` は一時ディレクトリに作って chdir | jind `interpret.rs` の Mock |

## schema (コマンドごと、`/jx-register` で LLM が書く)

- `schema.toml`: roles (jev の説明文)、amount の単位表、語テーブル、rules、questions、repair、confirm
- `assemble.sh`: 役割付きトークン → argv。言語は問わない (find は bash + jq で 84 行)
- `cases.toml`: words → argv。jev の答えはモックで書く

## find を書いて分かったこと

- jind の rules 17 分岐は全部 `[[rules]]` に落ちた。文脈条件は `prev_role` / `prev_is_number` の 2 つで足りた
- repair 3 本は `attach_unit` 1 本 + `claim` 2 本。`mark_excludes` の「実在ディレクトリの path は除外にするが `/var/log` はしない」は rules の `tag` で表した
- assemble.sh は README の例 6 本 + 衝突 1 本で jind と同じ argv。ただし **1 本目で jq の `//` が false を落とすバグを入れた** (`within an hour` が `+60` になった)。LLM が書く前提なら cases.toml は必須
- jind 固有でホストに残ったもの: `count_lines` (postprocess)、delete の preview。どちらも schema/assemble の出力で表せたので find 固有のコードはホストに無い

## まだ確かめていないこと

- jurl。`pair_key_values` (確率で交互配置を選ぶ) と `merge_joined` が `claim` で書けるか。書けなければプリミティブを 2 つ足す
- jev の Choice に `by_dimension` のような疑似 role を見せない工夫 (schema では `role = "by_dimension"` と書いたが、jev には time_amount / size_amount しか見せない)
- `-H "X: y"` のように「次の語ごと役割を変える」規則 (jurl)。`when.prev_role` だけでは書けない可能性がある
