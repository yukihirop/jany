# jx ホストが持つもの / schema が持つもの (2026-09-22 時点の設計メモ)

jind 0.1.0 を `design/find/`、jurl 0.1.2 を `design/curl/` の 3 ファイルずつに書き直して確かめた結果。

## ホスト (Rust、jind/jurl から流用)

流用元はそのまま: `jev/{mod,client}.rs` `color.rs` `setup.rs` `config.rs` `output.rs` (確認 UI・explain 表・`$EDITOR`)、`main.rs` の流れ、`cli.rs` (`-n` `--explain` `-y` `--no-jev`)。

新しく書くもの:

| 部品 | 内容 | 元 |
|---|---|---|
| schema ローダ | `~/.config/jx/cmd/<name>[/<sub>]/schema.toml` | — |
| Token | `text / role: Option<String> / value / amount / confidence / source / note / probs / tags` | jind `token.rs` の Role を文字列に。`tags` は複数 (curl の typed / kv) |
| amount パーサ | `[+->< ]?数[.数]?接尾辞?` を `{n, unit, at_least}` に。単位表と曖昧接尾辞は schema | jind `rules::parse_amount` |
| 規則エンジン | `[[rules]]` を上から評価。match: `prefix / word(1 語か配列) / table / regex / builtin(path_like, glob, existing_dir, amount, unit_word)`、when: `prev_role / prev_is_number / not_amount / has_unit / has_direction / prev_regex`。`fixed = "$upper" / "$lower" / "$1"`、`next = {role, fixed}` (次の語も消費)、`once = true` (その role が規則で付いていたら当たらない)、`tag` | jind `rules::classify`、jurl `rules::classify` + `adjacency_pass` |
| 質問ビルダ | `role.i` (jev=あり の roles 全部) + `[[questions]]` の when が当たる語ごとに 1 問。`{i}` `{w}` `{prev_i}` `{prev_w}` を埋める。when: `resolved / regex / has_amount / … / prev_any = [{…}]`。`scope = "command"` なら語ごとでなく 1 問 (curl の get_intent)。state.tokens は role の `mask` で置き換える。`[jev.state]` で `value_of = "<role>"` を足せる | jind `prompt::build`、jurl `prompt::build` |
| 答えの書き戻し | `role.i` → role/confidence/probs。`applies_to` のある typo 質問は表に無い語の fixed に (`none_conf` で none のとき confidence を落とす)。`sets = { tag }` は noul > 0.5 でタグ。amount 役割には unit/at_least を書く。command scope の答えは `answers` に残して repair / assemble へ | jind `prompt::apply`、jurl `prompt::apply` |
| repair プリミティブ | `attach_unit`、`claim {marker, role, side, many, from, skip, require, clear}`、`join {answer, sep, into}` (noul 答えで前の語に結合、後ろから、連鎖あり)、`pair {members, key_probs, value_probs, key_role, value_role, key_role_if}` (交互配置を尤度で選ぶ) | jind `repair.rs`、jurl `repair::merge_joined` / `pair_key_values` |
| assemble 呼び出し | stdin に `{tokens, passthrough, answers, defaults}` (`defaults` は schema `[defaults]` を user config で上書きした object)、stdout の `{argv, preview, risk, postprocess, error}` を読む。confidence の min はホストが取る | jind `assemble.rs` + `find.rs` の外側、jurl `assemble.rs` + `curl::argv` |
| 確認フロー | `reject_below` / `confirm_below` / `-y`。`risk = "dangerous"` なら `-y` でも確認・既定 No・preview を先に回して `preview_lines` 件見せる。`risk = "unsafe"` なら閾値だけ `confirm_below_unsafe` に上げる (-y は効く) | jind `main.rs execute`、jurl `main.rs execute` |
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

## curl を書いて分かったこと (2026-09-22)

- jurl の `classify` 17 分岐は `[[rules]]` 27 本に落ちた。足したのは `regex` match、`fixed` テンプレート (`$upper` `$lower` `$1`)、`next` (2 語規則: -H / -X / 値付きフラグ)、`once` (have_method / have_url)、`when.prev_regex` (裸のポート)。`looks_like_url` は regex 5 本 (TLD 一覧は regex に埋め込んだ)
- 予想どおり `pair_key_values` と `merge_joined` は `claim` で書けず、`pair` と `join` の 2 プリミティブを足した。どちらも jev の確率 / noul 答えを読むので、ホストが持つ
- 質問に `scope = "command"` (get_intent) と `sets.tag` (typed)、`none_conf` (typo で none のとき 0.3) が要った。role に `mask` (header を `<header>` に)
- assemble の契約を変えた: `dangerous: bool` → `risk: "none" | "unsafe" | "dangerous"` (jurl の PUT/PATCH/DELETE は「-y は効くが閾値が 0.9」で find の delete とは違う)、`defaults: [...]` → `defaults: {args, …}` (curl は `content_type` も要る)、`answers` を追加。find の 3 ファイルも合わせて直した
- assemble.sh は bash + jq 120 行。jurl の `-n --no-jev` 実出力 16 本 + interpret.rs / EXAMPLES の jev 例 7 本で同じ argv (scratch のハーネスで確認)
- **jx の規則が jurl と違うところ 1 つ**: `http://example.com/x?a=1` は jurl 0.1.2 だと `=` 分岐が URL より先に当たって未解決になる (実機で確認)。jx は `=` の左がボディのパス形のときだけ field にするので URL として通る。cases.toml に書いた
- **jurl の出力側は jx に持ち込まない** (決定 2026-09-22): `-w` のステータス行、TTY のときの `-i` + ヘッダ / JSON の色付け (`output::print_response`)。jx は argv を実行して stdout をそのまま出す。整形は `| jq`

## まだ確かめていないこと

- jev の Choice に `by_dimension` のような疑似 role を見せない工夫 (find schema では `role = "by_dimension"` と書いたが、jev には time_amount / size_amount しか見せない)
- 表 (`[tables.*]`) の大文字小文字。curl は区別しない前提で書いた (`POST` も `post` も当たる)。find の jind がどうしていたかは見ていない
- `regex` は Rust の regex crate 前提 (先読み無し)。`(?i)` は先頭に置いた (Python の re でも通る形)
- 3 つ目 (docker run / sql) で `next` `once` `pair` で足りるか
