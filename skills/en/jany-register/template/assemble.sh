#!/usr/bin/env bash
# jany assemble: __NAME__
# stdin:  {tokens:[{text,role,value,tags,amount,confidence,source,note}], passthrough:[...], answers:{...}, defaults:{args:[...]}}
# stdout: {argv:[...], preview:null|[...], risk:"none"|"unsafe"|"dangerous", pipe:null|[...], error:null|"..."}
# Caution: jq's `//` drops false too. Check booleans with `== true`.
set -euo pipefail

jq -c '
  def toks(r): [.tokens[] | select(.role == r)];
  def vals(r): [toks(r)[] | .value];

  (.defaults.args // []) as $defaults
  | .passthrough as $pass
  | vals("passthrough") as $flags

  # Build argv here. For combinations that cannot be built, put the reason in error (do not invent an argv).
  | {
      argv: ([__ARGV0__] + $defaults + $flags + $pass),
      preview: null,
      risk: "none",
      pipe: null,
      error: null
    }
'
