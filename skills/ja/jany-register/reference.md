# jany コマンド定義リファレンス

1 つのコマンド定義は 3 ファイル。置き場所は `~/.config/jany/cmd/<name>[/<sub>]/`(`JANY_CMD_DIR` で変更可)。

```
schema.toml    語 → 役割 の決め方(規則、jev への質問、repair)
assemble.sh    役割付きトークン JSON → argv JSON。実行権限が要る
cases.toml     words → argv のテスト。`jany --test <name>` が回す
```

ホストの流れ: **規則(schema.rules)→ 決まらない語があれば jev に質問 → 答えを書き戻す → repair → assemble.sh → shell-quote した 1 行を stdout**。全部規則で決まれば jev は呼ばれない(速い・無料)。

サブコマンドはディレクトリ階層: `cmd/docker/run/schema.toml` は `jany docker run …`。

---

## schema.toml

未知のキーはエラー(`deny_unknown_fields`)。以下にあるキーだけ使う。

### [command]

```toml
[command]
name = "find"                 # 表示名。バイナリ名でなくてよい(argv の先頭は assemble.sh が決める)
description = """..."""       # jev に state.tool として渡す。何のコマンドか・語がどう崩れるかを英語で
example = "log files older than 7 days in /var/log delete"   # `jany --list` に出る
```

### [[roles]]

語に付く役割。`jev` があるものだけが jev の選択肢になる(説明文はそのまま jev に渡すので英語で具体例つき)。

```toml
[[roles]]
key = "path"
jev = "A directory to search in (a path like src, /var/log, ~/Downloads)"
table = "types"       # 任意。補正値をこの表から引く(typo 質問にも使う)
amount = "time"       # 任意。"time" | "size" | "count"。この役割の語は amount {n, unit, at_least} を持つ
mask = "<header>"     # 任意。jev の state.tokens でこの文字列に置き換える。規則で決まった語も state には入るので、秘密を含みうる役割 (header, env) には必ず付ける
```

- `jev` の無い役割は規則でしか付かない(例: フラグ、ヘッダ)
- 役割は 10〜20 個に抑える。多いと jev の精度が落ちる

### [amount]

数値語のパース。`[+-><]?数[.数]?接尾辞?` を `{n, unit, at_least}` に。

```toml
[amount]
ambiguous_suffixes = ["m"]            # 単位を決めない接尾辞(m = minutes? MB?)
case_sensitive.suffixes = ["M", "G"]  # 大文字小文字を区別する接尾辞

[amount.units.time]                   # 次元 → 単位キー → 語
minutes = ["min", "mins", "minute", "minutes"]
hours   = ["h", "hr", "hour", "hours"]
days    = ["d", "day", "days"]

[amount.units.size]
K = ["k", "kb", "kib"]
M = ["mb", "mib", "megabytes"]
```

`+7d` → `{n: 7, unit: "days", at_least: true}`、`10` → `{n: 10, unit: null, at_least: null}`。

### [tables.*]

語 → 補正値の表。大文字小文字は区別しない。キー `_` は「補正値なし(語そのまま)」。

```toml
[tables.types]
f = ["file", "files", "f"]
d = ["dir", "dirs", "directory", "folder", "folders", "d"]

[tables.actions]
_ = ["ls", "print"]
```

### [[rules]]

上から順に試し、**最初に当たったものが勝つ**。`match` で語を見て、`when` で文脈を見る。

```toml
[[rules]]
match = { word = ["older", "newer"] }        # 完全一致(1 語か配列)
role  = "qualifier"

[[rules]]
match = { prefix = "-", min_len = 2 }        # 前方一致(+ 最小長)
role  = "passthrough"

[[rules]]
match = { table = "types" }                  # 表の語に一致 → fixed に表のキー
role  = "type"

[[rules]]
match = { regex = '(?i)^(get|post|put|patch|delete|head|options)$' }   # regex crate。(?i) は先頭
role  = "method"
fixed = "$upper"                             # "$upper" | "$lower" | "$1"(regex のキャプチャ)| 文字通り

[[rules]]
match = { builtin = "amount" }               # 組み込み判定。下の一覧
when  = { prev_role = "qualifier" }
role  = "by_dimension"                       # 特別な role: 単位の次元から amount="time"/"size" の役割を選ぶ
amount = { n = 1, unit = "$unit", at_least = false }   # 任意。規則側で amount を決める("$unit" は当たった単位語のキー)

[[rules]]
match = { word = "-H" }
role  = "flag"
next  = { role = "header", fixed = "$1" }    # 次の語も消費して role を付ける(2 語規則)。fixed 無しで role に table があれば表で補正
note  = "curl -H"                            # --explain の表に出るメモ

[[rules]]
match = { builtin = "existing_dir" }
role  = "path"
tag   = "existing_dir"                       # 語にタグを付ける(repair や質問の条件に使う)
once  = true                                 # この role が既に規則で付いていたら当たらない
```

`role = "unresolved"` を書くと「規則で決めない(jev に回す)」を明示できる。

**match**(1 つだけ書く): `word` / `prefix`(+`min_len`) / `table` / `regex` / `builtin`

**builtin**: `path_like`(`.` `..` `~` か `/` `./` `../` `~/` で始まる)/ `glob`(`*` `?` か `[…]` を含む)/ `existing_dir`(cwd に実在するディレクトリ)/ `amount`(数値語)/ `unit_word`(単位表の語 → `fixed` に単位キー)

**when**(全部満たすとき): `not_amount = true` / `prev_role = "<role>"` / `prev_prefix = "-"` / `prev_is_number = true` / `has_unit = true|false` / `has_direction = true|false` / `prev_regex = '...'`

amount は `amount = "..."` を持つ役割か `by_dimension` / `unresolved` の語にだけ付く。

裸の数を jev に回すときは `match = { builtin = "amount" }` + `role = "unresolved"` を最後に置く。こうすると jev が `memory` のような amount 役割を選んだときに n が付いている(規則が当たらない語は amount を持たない)。

### [[questions]]

規則で決まらなかった語に対して、`role.i`(役割を選ぶ Choice)はホストが自動で聞く。それ以外に聞きたいことを書く。語ごとに `when` が当たれば `<key>.<i>` として 1 問。

```toml
[[questions]]
key  = "atleast"                     # amount 役割向けの特別なキー: atleast / unit はホストが amount に書き戻す
kind = "noul"                        # "choice" | "noul"(0〜1 の確率)
when = { has_amount = true, direction_missing = true }
ask  = "Is `tokens[{i}]` (\"{w}\") a lower bound (at least / older than / larger than)?"

[[questions]]
key  = "unit"
kind = "choice"
when = { has_amount = true, unit_missing = true, next_role_not = "unit" }
ask  = "What unit does `tokens[{i}]` (\"{w}\") have?"
choices = "amount_units"             # 単位表の全キー + none

[[questions]]
key  = "type_typo"
kind = "choice"
when = { resolved = false, alphabetic = true, max_len = 20 }
ask  = "If `tokens[{i}]` (\"{w}\") is a misspelled entry type word, which one was intended?"
choices = "table:types"              # 表の語 + none
min_choice_len = 2                   # 1 文字の語は候補にしない
none = "Not an entry type"           # none の説明文
applies_to = "type"                  # jev が role=type と言い、表に無い語なら、この答えを fixed に
none_conf = 0.3                      # none と答えたら confidence をここまで落とす

[[questions]]
key  = "join"
kind = "noul"
when = { resolved = false, prev_any = [{ role = "value" }, { resolved = false }] }
ask  = "Should `tokens[{i}]` (\"{w}\") be joined to the previous word `tokens[{prev_i}]` (\"{prev_w}\")?"

[[questions]]
key  = "typed"
kind = "noul"
when = { resolved = false }
ask  = "..."
sets = { tag = "typed" }             # noul > 0.5 ならタグを付ける

[[questions]]
key  = "get_intent"
kind = "noul"
scope = "command"                    # 語ごとでなく 1 問。答えは assemble.sh の stdin `answers` に入る
when = { no_role = "method", any_unresolved = true }
ask  = "..."
```

`ask` の穴: `{i}` `{w}` `{prev_i}` `{prev_w}`。

**when**(語ごと): `resolved` / `has_amount` / `unit_missing` / `direction_missing` / `next_role_not` / `alphabetic` / `max_len` / `regex` / `prev_any = [Cond, …]`
**when**(`scope = "command"`): `no_role` / `any_unresolved`

**Cond**(prev_any / repair の `into` / `from` で使う語の条件): `{ resolved, role, tag, tag_not, source = "rule"|"jev", chained }`

### [jev.state]

state に役割の値を足す(jev が文脈を見られるように)。

```toml
[jev.state]
method = { value_of = "method" }     # state.method = method 役割の語の value
```

### [[repair]]

jev の答えは語ごとに独立なので、隣接関係をここで直す。上から順に適用。プリミティブは 4 つ。

```toml
[[repair]]
op = "attach_unit"                   # unit 役割の語を直前の amount に付ける。次元が違えば役割を単位側に直す
unit_role = "unit"
amount_roles = ["time_amount", "size_amount"]

[[repair]]
op = "claim"                         # marker 役割の語が隣の語を role として取る
marker = "exclude_marker"
role = "exclude"
side = "after"                       # "after" | "before" | "both"(既定 after)
many = false                         # true なら連続する複数語
from = ["path", { role = "path", tag = "existing_dir" }, "unresolved"]   # 取ってよい語の条件(Sel: role 名か Cond)
skip = ["qualifier"]                 # 読み飛ばす役割
require = { amount_without_unit = true }   # 任意
clear = ["unit", "direction"]        # 取った語の amount から消すもの

[[repair]]
op = "join"                          # noul 答え `answer.i` > 0.5 の語を前の語に結合(後ろから、連鎖あり)
answer = "join"
sep = " "
into = [{ role = "value" }, { resolved = false, chained = true }]

[[repair]]
op = "pair"                          # key/value の交互配置を jev の確率で選ぶ(curl の field name job)
members = ["field", "value", "unresolved"]
key_probs = ["field"]
value_probs = ["value"]
key_role = "field"
value_role = "value"
key_role_if = { role = "query", any = [{ member_role = "query" }, { value_of = "method", is = "GET" }, { answer = "get_intent", gt = 0.5 }, { no_role = "method" }] }
```

### [[placeholders]]

zsh で `jany <name> ` の後ろに薄く出す候補(例: `<path> <file|dir> <older than N days>`)。次に何を言えるかを見せて、迷わないようにする。枠は書いた順に出て、打った語がどれかの `roles` を取ると消える。キーを打つたびに規則と repair だけを回す(jev は呼ばない)。任意。無ければ何も出ない。

```toml
[[placeholders]]
text  = "<path>"
roles = ["path"]

[[placeholders]]
text  = "<*.log>"
roles = ["name_pattern", "extension", "name_word"]   # 択一の役割は 1 つの枠にまとめる
bare  = true          # 規則で決まらない語 (jev に回る語) はこの枠を埋めたとみなす
```

- 枠はよく言うものを 3〜6 個、人が言う順に書く。役割ごとに 1 つ作らない
- 裸の語がふつう入る枠(find の名前、docker の image、ffmpeg の入力ファイル)に `bare = true` を付ける。付けないと `jany docker run nginx ` で `<image>` が出たままになる
- 規則で決まらない数は、`amount` を持つ役割の最初の空き枠を埋めたとみなす(単位が分かれば同じ次元の枠)。`older than 7 days` の `7` は `<older than N days>` を埋める

### [assemble] / [defaults] / [confirm]

```toml
[assemble]
script = "assemble.sh"               # schema と同じディレクトリからの相対。実行権限が要る

[defaults]                           # assemble.sh の stdin `defaults` に入る。~/.config/jany/config.toml の [cmd.<name>.defaults] で上書きされる
args = []
content_type = "application/json"

[confirm]
preview_readonly = true              # risk="dangerous" のとき preview argv を jany が実行して見せてよい(read-only のときだけ true)
preview_lines = 10
unsafe_note = "this method cannot be undone"   # risk="unsafe" のとき stderr に出す一言
```

---

## assemble.sh

stdin に JSON、stdout に JSON。言語は問わない(find/curl は bash + jq)。

**stdin**:

```json
{
  "tokens": [
    { "text": "+7d", "role": "time_amount", "value": "+7d", "tags": [],
      "amount": { "n": 7, "unit": "days", "at_least": true },
      "confidence": 1.0, "source": "rule", "note": null }
  ],
  "passthrough": ["-perm", "644"],
  "answers": { "get_intent": 0.9 },
  "defaults": { "args": [] }
}
```

- `value` は補正値(表のキー / fixed)があればそれ、無ければ `text`
- `source` は `"rule"` か `"jev"`
- `passthrough` は `--` の後ろの語。argv の末尾にそのまま付ける
- `answers` は `scope = "command"` の質問の答えだけ

**stdout**:

```json
{ "argv": ["find", "/var/log", "-type", "f"], "preview": null, "risk": "none", "pipe": null, "error": null }
```

- `argv`: 実行される argv。jany はこれを shell-quote して 1 行にする(実行はしない)
- `preview`: `risk = "dangerous"` のとき、破壊せずに対象だけ見せる argv(find なら `-delete` を外したもの)
- `risk`: `"none"` | `"unsafe"`(取り消しにくい: PUT/DELETE)| `"dangerous"`(破壊的: rm、find -delete)
- `pipe`: 後ろに `|` で繋ぐ argv(`["wc", "-l"]`)。無ければ null
- `error`: 組み立てられないときの説明。あれば argv は無視される

注意(jq): `//` は `false` も `null` 扱いにする。真偽値は `== null` / `== true` で見る。

---

## cases.toml

```toml
[[case]]
words = ["log", "files", "older", "than", "7", "days", "in", "/var/log", "delete"]
argv  = ["find", "/var/log", "-type", "f", "-iname", "*.log", "-mtime", "+7", "-delete"]
preview = ["find", "/var/log", "-type", "f", "-iname", "*.log", "-mtime", "+7"]
risk = "dangerous"
[case.jev]                                       # jev が聞く質問への答え(Mock)
"role.0"    = { choice = "extension", confidence = 0.88 }
"role.4"    = { choice = "time_amount", confidence = 0.95 }
"atleast.4" = { noul = 0.97 }
"type_typo.0"   = { choice = "none" }

[[case]]
words = ["-perm", "644", "-newer", "x"]
argv  = ["find", ".", "-perm", "644", "-newer", "x"]
no_jev = true                                    # jev を呼んだら失敗(規則だけで決まるべき)

[[case]]
words = ["src", "except", "target", "ls"]
setup = { dirs = ["src", "src/target"] }         # 一時ディレクトリにこれを作って chdir してから回す
argv  = ["find", "src", "-not", "-path", "*/target/*", "-ls"]

[[case]]
words = ["nothing"]
error = "unresolved: nothing"                    # エラーを期待する
```

期待値のキー: `argv` `preview` `risk`(既定 "none")`pipe` `error` `confidence`(±0.01)`tokens`(結合後の語の並び)`state`(jev に渡す state の一部、`"hints.0" = "path"` のようにドット区切り)`not_asked = ["unit.2"]`(聞かれてはいけないキー)`defaults`(上書き)`passthrough`。

Mock の掟:
- `[case.jev]` に書いたキーを jany が聞かなかったら **失敗**(質問の `when` とフィクスチャのずれに気づくため)
- 聞かれたのに答えが無いキーは未回答のまま。役割が決まらなければ `unresolved` エラー
- `[case.jev]` が無いのに jev が必要になったら失敗(規則だけで決まると宣言したことになる)
- `{ choice = "x", confidence = 0.9 }` / `{ choice = "x", probs = { x = 0.6, y = 0.4 } }` / `{ noul = 0.97 }`

---

## エラーの読み方

```
jany: schema.toml: unknown field `foo`         schema に無いキー。上の一覧にあるものだけ
jany: rule role `x` is not in [[roles]]        rules の role は roles に宣言する
jany: unknown builtin `x`                      path_like / glob / existing_dir / amount / unit_word
jany: could not interpret: foo                 未解決語。規則か jev の役割説明を足す
jany: assemble.sh: No such file or directory   chmod +x を忘れている
error: jev: case answers `unit.2` but jany did not ask it   質問の when が当たっていない
```
