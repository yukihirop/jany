#!/usr/bin/env bash
# jany assemble: __NAME__
# stdin:  {tokens:[{text,role,value,tags,amount,confidence,source,note}], passthrough:[...], answers:{...}, defaults:{args:[...]}}
# stdout: {argv:[...], preview:null|[...], risk:"none"|"unsafe"|"dangerous", pipe:null|[...], error:null|"..."}
# 注意: jq の `//` は false も落とす。真偽値は `== true` で見る。
set -euo pipefail

jq -c '
  def toks(r): [.tokens[] | select(.role == r)];
  def vals(r): [toks(r)[] | .value];

  (.defaults.args // []) as $defaults
  | .passthrough as $pass
  | vals("passthrough") as $flags

  # ここで argv を組む。組めない組み合わせは error に理由を書く(argv を捏造しない)。
  | {
      argv: ([__ARGV0__] + $defaults + $flags + $pass),
      preview: null,
      risk: "none",
      pipe: null,
      error: null
    }
'
