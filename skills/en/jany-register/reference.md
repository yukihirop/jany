# jany command definition reference

One command definition is three files. They live in `~/.config/jany/cmd/<name>[/<sub>]/` (changeable with `JANY_CMD_DIR`).

```
schema.toml    how words get roles (rules, questions to jev, repair)
assemble.sh    role-tagged token JSON → argv JSON. Must be executable
cases.toml     words → argv tests. Run by `jany --test <name>`
```

The host's flow: **rules (schema.rules) → if any word is undecided, ask jev → write the answers back → repair → assemble.sh → one shell-quoted line on stdout**. If the rules decide everything, jev is never called (fast and free).

Subcommands are a directory hierarchy: `cmd/docker/run/schema.toml` is `jany docker run …`.

---

## schema.toml

Unknown keys are an error (`deny_unknown_fields`). Use only the keys listed below.

### [command]

```toml
[command]
name = "find"                 # display name. Need not be the binary name (assemble.sh decides the head of argv)
description = """..."""       # passed to jev as state.tool. In English: what the command is and how its words tend to break
example = "log files older than 7 days in /var/log delete"   # shown by `jany --list`
```

### [[roles]]

The roles a word can take. Only roles with `jev` are choices for jev (the description is passed to jev verbatim, so write it in English with concrete examples).

```toml
[[roles]]
key = "path"
jev = "A directory to search in (a path like src, /var/log, ~/Downloads)"
table = "types"       # optional. Look up the corrected value in this table (also used for typo questions)
amount = "time"       # optional. "time" | "size" | "count". Words with this role carry amount {n, unit, at_least}
mask = "<header>"     # optional. Replaced by this string in jev's state.tokens. Words decided by rules still go into state, so always set it on roles that may contain secrets (header, env)
```

- Roles without `jev` are only ever assigned by rules (e.g. flags, headers)
- Keep it to 10–20 roles. More roles lower jev's accuracy

### [amount]

Parsing of numeric words. `[+-><]?number[.number]?suffix?` into `{n, unit, at_least}`.

```toml
[amount]
ambiguous_suffixes = ["m"]            # suffixes that do not decide the unit (m = minutes? MB?)
case_sensitive.suffixes = ["M", "G"]  # suffixes compared case-sensitively

[amount.units.time]                   # dimension → unit key → words
minutes = ["min", "mins", "minute", "minutes"]
hours   = ["h", "hr", "hour", "hours"]
days    = ["d", "day", "days"]

[amount.units.size]
K = ["k", "kb", "kib"]
M = ["mb", "mib", "megabytes"]
```

`+7d` → `{n: 7, unit: "days", at_least: true}`, `10` → `{n: 10, unit: null, at_least: null}`.

### [tables.*]

Word → corrected value tables. Case-insensitive. The key `_` means "no corrected value (keep the word)".

```toml
[tables.types]
f = ["file", "files", "f"]
d = ["dir", "dirs", "directory", "folder", "folders", "d"]

[tables.actions]
_ = ["ls", "print"]
```

### [[rules]]

Tried top to bottom; **the first one that matches wins**. `match` looks at the word, `when` looks at the context.

```toml
[[rules]]
match = { word = ["older", "newer"] }        # exact match (one word or an array)
role  = "qualifier"

[[rules]]
match = { prefix = "-", min_len = 2 }        # prefix match (+ minimum length)
role  = "passthrough"

[[rules]]
match = { table = "types" }                  # matches a table word → fixed is the table key
role  = "type"

[[rules]]
match = { regex = '(?i)^(get|post|put|patch|delete|head|options)$' }   # regex crate. (?i) goes first
role  = "method"
fixed = "$upper"                             # "$upper" | "$lower" | "$1" (regex capture) | literal

[[rules]]
match = { builtin = "amount" }               # built-in test. See the list below
when  = { prev_role = "qualifier" }
role  = "by_dimension"                       # special role: picks the amount="time"/"size" role from the unit's dimension
amount = { n = 1, unit = "$unit", at_least = false }   # optional. The rule decides the amount ("$unit" is the key of the matched unit word)

[[rules]]
match = { word = "-H" }
role  = "flag"
next  = { role = "header", fixed = "$1" }    # also consumes the next word and gives it a role (two-word rule). Without fixed, the role's table (if any) corrects it
note  = "curl -H"                            # note shown in the --explain table

[[rules]]
match = { builtin = "existing_dir" }
role  = "path"
tag   = "existing_dir"                       # tags the word (used by repair and question conditions)
once  = true                                 # does not match if a rule has already assigned this role
```

Writing `role = "unresolved"` states explicitly "do not decide by rule (hand it to jev)".

**match** (write exactly one): `word` / `prefix` (+`min_len`) / `table` / `regex` / `builtin`

**builtin**: `path_like` (`.` `..` `~`, or starts with `/` `./` `../` `~/`) / `glob` (contains `*` `?` or `[…]`) / `existing_dir` (an existing directory relative to cwd) / `amount` (a numeric word) / `unit_word` (a word from the unit table → `fixed` is the unit key)

**when** (all must hold): `not_amount = true` / `prev_role = "<role>"` / `prev_prefix = "-"` / `prev_is_number = true` / `has_unit = true|false` / `has_direction = true|false` / `prev_regex = '...'`

An amount is attached only to roles that have `amount = "..."`, or to `by_dimension` / `unresolved` words.

To hand bare numbers to jev, put `match = { builtin = "amount" }` + `role = "unresolved"` last. That way, when jev picks an amount role such as `memory`, n is already attached (words no rule matched carry no amount).

### [[questions]]

For words the rules could not decide, the host automatically asks `role.i` (a Choice of role). Write anything else you want to ask here. For each word where `when` matches, it becomes one question `<key>.<i>`.

```toml
[[questions]]
key  = "atleast"                     # special keys for amount roles: the host writes atleast / unit back into the amount
kind = "noul"                        # "choice" | "noul" (a probability from 0 to 1)
when = { has_amount = true, direction_missing = true }
ask  = "Is `tokens[{i}]` (\"{w}\") a lower bound (at least / older than / larger than)?"

[[questions]]
key  = "unit"
kind = "choice"
when = { has_amount = true, unit_missing = true, next_role_not = "unit" }
ask  = "What unit does `tokens[{i}]` (\"{w}\") have?"
choices = "amount_units"             # every key in the unit table + none

[[questions]]
key  = "type_typo"
kind = "choice"
when = { resolved = false, alphabetic = true, max_len = 20 }
ask  = "If `tokens[{i}]` (\"{w}\") is a misspelled entry type word, which one was intended?"
choices = "table:types"              # the table's words + none
min_choice_len = 2                   # single-letter words are not offered
none = "Not an entry type"           # description of none
applies_to = "type"                  # if jev says role=type and the word is not in the table, this answer becomes fixed
none_conf = 0.3                      # answering none lowers confidence to this

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
sets = { tag = "typed" }             # tags the word if noul > 0.5

[[questions]]
key  = "get_intent"
kind = "noul"
scope = "command"                    # one question for the whole command, not per word. The answer goes into assemble.sh's stdin `answers`
when = { no_role = "method", any_unresolved = true }
ask  = "..."
```

Placeholders in `ask`: `{i}` `{w}` `{prev_i}` `{prev_w}`.

**when** (per word): `resolved` / `has_amount` / `unit_missing` / `direction_missing` / `next_role_not` / `alphabetic` / `max_len` / `regex` / `prev_any = [Cond, …]`
**when** (`scope = "command"`): `no_role` / `any_unresolved`

**Cond** (a condition on a word, used by prev_any and repair's `into` / `from`): `{ resolved, role, tag, tag_not, source = "rule"|"jev", chained }`

### [jev.state]

Adds role values to the state (so jev can see the context).

```toml
[jev.state]
method = { value_of = "method" }     # state.method = the value of the word with the method role
```

### [[repair]]

jev's answers are independent per word, so adjacency is fixed here. Applied top to bottom. There are four primitives.

```toml
[[repair]]
op = "attach_unit"                   # attaches a unit-role word to the amount just before it. If the dimensions differ, the unit wins and the role is corrected
unit_role = "unit"
amount_roles = ["time_amount", "size_amount"]

[[repair]]
op = "claim"                         # a word with the marker role takes its neighbour as role
marker = "exclude_marker"
role = "exclude"
side = "after"                       # "after" | "before" | "both" (default after)
many = false                         # true takes several consecutive words
from = ["path", { role = "path", tag = "existing_dir" }, "unresolved"]   # which words may be taken (Sel: a role name or a Cond)
skip = ["qualifier"]                 # roles to step over
require = { amount_without_unit = true }   # optional
clear = ["unit", "direction"]        # what to clear from the taken word's amount

[[repair]]
op = "join"                          # joins words whose noul answer `answer.i` > 0.5 to the previous word (from the back, chains allowed)
answer = "join"
sep = " "
into = [{ role = "value" }, { resolved = false, chained = true }]

[[repair]]
op = "pair"                          # chooses the key/value alternation using jev's probabilities (curl's field name job)
members = ["field", "value", "unresolved"]
key_probs = ["field"]
value_probs = ["value"]
key_role = "field"
value_role = "value"
key_role_if = { role = "query", any = [{ member_role = "query" }, { value_of = "method", is = "GET" }, { answer = "get_intent", gt = 0.5 }, { no_role = "method" }] }
```

### [[placeholders]]

The dim hint zsh shows after `jany <name> ` (e.g. `<path> <file|dir> <older than N days>`), so people know what they can say next. Slots are shown in the order written, and a slot disappears once a typed word takes one of its `roles`. Only rules and repair run for it (never jev), on every keystroke. Optional: without it, nothing is shown.

```toml
[[placeholders]]
text  = "<path>"
roles = ["path"]

[[placeholders]]
text  = "<*.log>"
roles = ["name_pattern", "extension", "name_word"]   # alternatives share one slot
bare  = true          # a word the rules could not decide (jev will) counts as this slot
```

- Write 3–6 slots for what people say most, in the order they usually say it. Not one per role
- Put `bare = true` on the slot that bare words usually are (find's name, docker's image, ffmpeg's input file). Otherwise `nginx` in `jany docker run nginx ` leaves `<image>` showing
- A number the rules could not decide counts as the first open slot with an `amount` role (of the same dimension when its unit is known), so `7` in `older than 7 days` fills `<older than N days>`

### [assemble] / [defaults] / [confirm]

```toml
[assemble]
script = "assemble.sh"               # relative to the schema's directory. Must be executable

[defaults]                           # goes into assemble.sh's stdin `defaults`. Overridden by [cmd.<name>.defaults] in ~/.config/jany/config.toml
args = []
content_type = "application/json"

[confirm]
preview_readonly = true              # with risk="dangerous", jany may run the preview argv to show it (true only when it is read-only)
preview_lines = 10
unsafe_note = "this method cannot be undone"   # one line printed on stderr with risk="unsafe"
```

---

## assemble.sh

JSON on stdin, JSON on stdout. Any language works (find/curl use bash + jq).

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

- `value` is the corrected value (table key / fixed) if there is one, otherwise `text`
- `source` is `"rule"` or `"jev"`
- `passthrough` is the words after `--`. Append them to the end of argv as-is
- `answers` holds only the answers to `scope = "command"` questions

**stdout**:

```json
{ "argv": ["find", "/var/log", "-type", "f"], "preview": null, "risk": "none", "pipe": null, "error": null }
```

- `argv`: the argv to run. jany shell-quotes it into one line (it never runs it)
- `preview`: with `risk = "dangerous"`, an argv that shows only the targets without destroying them (for find, the one without `-delete`)
- `risk`: `"none"` | `"unsafe"` (hard to undo: PUT/DELETE) | `"dangerous"` (destructive: rm, find -delete)
- `pipe`: an argv to connect after a `|` (`["wc", "-l"]`). null if none
- `error`: an explanation when it cannot be assembled. If present, argv is ignored

Caution (jq): `//` treats `false` as `null` too. Check booleans with `== null` / `== true`.

---

## cases.toml

```toml
[[case]]
words = ["log", "files", "older", "than", "7", "days", "in", "/var/log", "delete"]
argv  = ["find", "/var/log", "-type", "f", "-iname", "*.log", "-mtime", "+7", "-delete"]
preview = ["find", "/var/log", "-type", "f", "-iname", "*.log", "-mtime", "+7"]
risk = "dangerous"
[case.jev]                                       # answers to the questions jev is asked (Mock)
"role.0"    = { choice = "extension", confidence = 0.88 }
"role.4"    = { choice = "time_amount", confidence = 0.95 }
"atleast.4" = { noul = 0.97 }
"type_typo.0"   = { choice = "none" }

[[case]]
words = ["-perm", "644", "-newer", "x"]
argv  = ["find", ".", "-perm", "644", "-newer", "x"]
no_jev = true                                    # fails if jev is called (must be decided by rules alone)

[[case]]
words = ["src", "except", "target", "ls"]
setup = { dirs = ["src", "src/target"] }         # creates these in a temporary directory and chdirs there before running
argv  = ["find", "src", "-not", "-path", "*/target/*", "-ls"]

[[case]]
words = ["nothing"]
error = "unresolved: nothing"                    # expects an error
```

Expectation keys: `argv` `preview` `risk` (default "none") `pipe` `error` `confidence` (±0.01) `tokens` (the word sequence after joining) `state` (part of the state passed to jev, dot-separated like `"hints.0" = "path"`) `not_asked = ["unit.2"]` (keys that must not be asked) `defaults` (override) `passthrough`.

Rules of the Mock:
- If jany did not ask a key written in `[case.jev]`, that is a **failure** (so you notice when a question's `when` and the fixture drift apart)
- Keys that were asked but have no answer stay unanswered. If a role stays undecided, it is an `unresolved` error
- If jev is needed but there is no `[case.jev]`, that is a failure (the case declared it is decided by rules alone)
- `{ choice = "x", confidence = 0.9 }` / `{ choice = "x", probs = { x = 0.6, y = 0.4 } }` / `{ noul = 0.97 }`

---

## Reading errors

```
jany: schema.toml: unknown field `foo`         a key not in the schema. Only the ones listed above
jany: rule role `x` is not in [[roles]]        declare rules' roles in roles
jany: unknown builtin `x`                      path_like / glob / existing_dir / amount / unit_word
jany: could not interpret: foo                 an unresolved word. Add a rule or a role description for jev
jany: assemble.sh: No such file or directory   you forgot chmod +x
error: jev: case answers `unit.2` but jany did not ask it   the question's when does not match
```
