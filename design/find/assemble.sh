#!/usr/bin/env bash
# jx assemble for find (jind 0.1.0 の assemble.rs + find.rs 相当)。
#
# stdin : {"tokens":[{"text","role","value","amount":{"n","unit","at_least"}|null}], "passthrough":[...], "defaults":[...]}
#         tokens はすべて role が付いている (未解決はホストが弾く)。confidence はホストが min を取るのでここでは見ない。
# stdout: {"argv":[...], "preview":[...]|null, "dangerous":bool, "postprocess":"count_lines"|null, "error":"..."|null}
#
# preview   = 実行前に見せる用の argv (delete なら -delete 抜き)。dangerous=true のときホストが先に回して最大 N 件見せる。
# postprocess = count なら find の出力行数をホストが数えて表示する。
set -euo pipefail

jq -c '
  # 向きが無ければ「以上」。jq の // は false も飛ばすので == null で見る。
  def sign: if (.at_least == null or .at_least) then "+" else "-" end;

  # 日なら -mtime、それより細かければ -mmin。週は日に。
  def time_arg:
    (.unit // "days") as $u
    | if   $u == "days"    then ["-mtime", (sign + ((.n)          | round | tostring))]
      elif $u == "weeks"   then ["-mtime", (sign + ((.n * 7)      | round | tostring))]
      elif $u == "hours"   then ["-mmin",  (sign + ((.n * 60)     | round | tostring))]
      elif $u == "minutes" then ["-mmin",  (sign + ((.n)          | round | tostring))]
      else                      ["-mtime", (sign + ((.n)          | round | tostring))] end;

  # -size は整数 + 接尾辞。小数は 1 段小さい単位に落とす (1.5G → 1536M)。
  def size_arg:
    (.unit // "MB") as $u
    | ({bytes:"c", KB:"k", MB:"M", GB:"G"}[$u] // "M") as $s
    | (.n) as $n
    | (if ($n | floor) != $n then
         (if $s == "G" then [$n*1024, "M"] elif $s == "M" then [$n*1024, "k"] elif $s == "k" then [$n*1024, "c"] else [$n, $s] end)
       else [$n, $s] end) as [$m, $suf]
    | ["-size", (sign + ($m | round | tostring) + $suf)];

  def home: env.HOME // "";
  def expand_home: if . == "~" then home elif startswith("~/") then home + .[1:] else . end;

  # 複数候補は \( a -o b \) で OR。
  def flatten_o: reduce .[] as $x ([]; if length == 0 then $x else . + ["-o"] + $x end);
  def group(f): if length == 1 then (.[0] | f) elif length > 1 then ["("] + ([.[] | f] | flatten_o) + [")"] else [] end;

  .tokens as $t
  | ($t | map(select(.role == "path") | .text | expand_home))                            as $paths
  | ($t | map(select(.role == "depth") | .amount.n | round))                                as $depths
  | ($t | map(select(.role == "type") | .value) | unique)                                   as $types
  | ($t | map(select(.role == "name_pattern") | {p: .text, ci: false})
        + map(select(.role == "extension") | {p: ("*." + (.text | ltrimstr("."))), ci: true})
        + map(select(.role == "name_word") | {p: ("*" + .text + "*"), ci: true}))           as $names
  | ($t | any(.role == "empty"))                                                            as $empty
  | ($t | map(select(.role == "time_amount") | .amount | time_arg))                        as $times
  | ($t | map(select(.role == "size_amount") | .amount | size_arg))                        as $sizes
  | ($t | map(select(.role == "exclude") | .text | ltrimstr("/") | rtrimstr("/")))          as $excl
  | ($t | map(select(.role == "passthrough") | .text))                                      as $extra
  | ($t | map(select(.role == "action") | .value) | unique)                                 as $actions
  | ($t | map(select((.role == "time_amount" or .role == "size_amount" or .role == "depth") and .amount == null) | .text)) as $no_number

  # 衝突は error で返し、ホストが表示する。
  | if ($depths | length) > 1 then {error: ("depth " + ($depths | map(tostring) | join(" and ")))}
    elif ($actions | length) > 1 then {error: ("action " + ($actions | join(" and ")))}
    elif ($no_number | length) > 0 then {error: (($no_number | join(", ")) + " has no number")}
    else
      ($actions[0] // "print") as $action
      | (["find"]
          + (if ($paths | length) == 0 then ["."] else $paths end)
          + (if ($depths | length) == 1 then ["-maxdepth", ($depths[0] | tostring)] else [] end)   # -maxdepth は式より前 (GNU が警告する)
          + ($types | group(["-type", .]))
          + ($names | group([(if .ci then "-iname" else "-name" end), .p]))
          + (if $empty then ["-empty"] else [] end)
          + ($times | add // [])
          + ($sizes | add // [])
          + ([$excl[] | ["-not", "-path", ("*/" + . + "/*"), "-not", "-path", ("*/" + .)]] | add // [])
          + $extra
          + (.defaults // [])
          + (.passthrough // [])
        ) as $base
      | (if $action == "print0" then ["-print0"] elif $action == "ls" then ["-ls"] else [] end) as $tail
      | {
          argv:        ($base + $tail + (if $action == "delete" then ["-delete"] else [] end)),
          preview:     (if $action == "delete" then ($base + $tail) else null end),
          dangerous:   ($action == "delete"),
          postprocess: (if $action == "count" then "count_lines" else null end),
          error:       null
        }
    end
'
