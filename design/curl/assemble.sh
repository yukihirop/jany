#!/usr/bin/env bash
# jany assemble for curl (jurl 0.1.2 の assemble.rs + body.rs + url.rs + curl.rs::argv 相当)。
#
# stdin : {"tokens":[{"text","role","value","tags":[...],"confidence",...}], "passthrough":[...], "answers":{...},
#          "defaults":{"content_type":"application/json","args":[...]}}
#         tokens はすべて role が付いている。value は fixed があればそれ、無ければ text (ホストが埋める)。
#         repair.pair 済みなので field_key の次は field_value、裸の query の次も field_value。
# stdout: {"argv":[...], "preview":null, "risk":"none"|"unsafe", "pipe":null, "error":"..."|null}
#
# risk = "unsafe" は PUT / PATCH / DELETE。ホストは入力行に置く前に stderr に一言注意を出すだけ (実行はしない)。preview は使わない。
set -euo pipefail

jq -c '
  def val: .value // .text;
  def has_tag($t): ((.tags // []) | index($t)) != null;
  # `:=` / typed の値は JSON リテラル。読めなければ文字列 (jurl body::parse_value)。
  def parse_value($typed): . as $s | if $typed then (try ($s | fromjson) catch $s) else $s end;

  # --- body path `a.b[0].c` / `a/b` / `a[b]` → jq の path 配列 (jurl body::segments) -------------
  def segments:
    [scan("\\[([^\\]]*)\\]|([^.\\[\\]/]+)")
     | if .[0] != null then (if (.[0] | test("^[0-9]+$")) then (.[0] | tonumber) else .[0] end)
       elif .[1] != null then .[1] else empty end
     | select(. != "")];
  def build_body($fields):
    reduce $fields[] as $f (null; ($f.k | segments) as $p | if ($p | length) == 0 then . else setpath($p; $f.v) end);

  # form 用: ネストを profile[first_name]=job に平坦化 (jurl body::flatten)
  def flatten_form($prefix):
    if type == "object" then to_entries[] | (.key as $k | .value | flatten_form(if $prefix == "" then $k else $prefix + "[" + $k + "]" end))
    elif type == "array" then to_entries[] | (.key as $i | .value | flatten_form($prefix + "[" + ($i | tostring) + "]"))
    elif type == "string" then [$prefix, .]
    elif . == null then [$prefix, ""]
    else [$prefix, tostring] end;
  def scalar: if type == "string" then . else tojson end;

  # --- URL (jurl url.rs) -------------------------------------------------------------------------
  def parse_url:
    (if test("://") then capture("^(?<scheme>[^:]+)://(?<rest>.*)$") | .scheme |= ascii_downcase
     else {scheme: null, rest: .} end)
    | (if (.rest | startswith(":")) then .rest = "localhost" + .rest else . end)
    | (.rest | capture("^(?<hostport>[^/]*)(?<path>/[^?]*)?(\\?(?<q>.*))?$")) as $m
    | ($m.hostport | if test(":[0-9]+$") then capture("^(?<host>.*):(?<port>[0-9]+)$") | .port |= tonumber else {host: ., port: null} end) as $hp
    | { scheme, host: $hp.host, port: $hp.port, path: ($m.path // ""),
        query: [($m.q // "") | split("&")[] | select(. != "") | capture("^(?<k>[^=]*)(=(?<v>.*))?$") | [.k, (.v // "")]] };
  def push_path($seg):
    ($seg | ltrimstr("/") | rtrimstr("/")) as $s
    | if $s == "" then . else .path = (.path + (if (.path | endswith("/")) then "" else "/" end) + $s) end;
  def is_private_ipv4:
    (split(".") | map(tonumber? // -1)) as $p
    | ($p | length) == 4 and ($p[0] == 127 or $p[0] == 10 or ($p[0] == 172 and $p[1] >= 16 and $p[1] <= 31) or ($p[0] == 192 and $p[1] == 168));
  # 決定 (jurl 2026-09-22): https 既定、localhost / *.local / *.test / *.internal / 私有 IP だけ http
  def default_scheme:
    (.host | ascii_downcase) as $h
    | if $h == "localhost" or ($h | endswith(".localhost")) or ($h | endswith(".local")) or ($h | endswith(".test"))
         or ($h | endswith(".internal")) or $h == "0.0.0.0" or ($h | is_private_ipv4) then "http" else "https" end;
  def encode: @uri | gsub("%20"; "+");        # jurl url::encode (空白は +)
  def render_url:
    (.scheme // default_scheme) + "://" + .host
    + (if .port != null then ":" + (.port | tostring) else "" end)
    + (if .path == "" then "/" else (if (.path | startswith("/")) then "" else "/" end) + .path end)
    + (if (.query | length) > 0 then "?" + ([.query[] | (.[0] | encode) + "=" + (.[1] | encode)] | join("&")) else "" end);

  # --- トークン → 要素 ------------------------------------------------------------------------------
  .tokens as $t | ($t | length) as $n | .defaults as $d
  | ($t | map(select(.role == "method") | val))                       as $methods
  | ($t | map(select(.role == "url") | val))                          as $urls
  | ($t | map(select(.role == "port") | .text | tonumber? // empty))  as $ports
  | ($t | map(select(.role == "url_path") | .text))                   as $paths
  | ($t | map(select(.role == "content_type") | val))                 as $cts
  | ($t | map(select(.role == "header") | .text | capture("^(?<k>[^:]*):?(?<v>.*)$") | [(.k | ltrimstr(" ") | rtrimstr(" ")), (.v | ltrimstr(" ") | rtrimstr(" "))])) as $headers
  | ($t | map(select(.role == "body_file") | .text[1:]))              as $body_files
  | ($t | map(select(.role == "curl_flag" or .role == "curl_flag_value") | .text)) as $curl_extra
  | ([range(0; $n) as $i | $t[$i] as $x
      | if $x.role == "field" then
          (if ($x | has_tag("typed")) then ($x.text | capture("^(?<k>[^:]*):=(?<v>.*)$") | {kind: "field", k, v: (.v | parse_value(true))})
           else ($x.text | capture("^(?<k>[^=]*)=(?<v>.*)$") | {kind: "field", k, v}) end)
        elif $x.role == "field_key" then
          ($t[$i + 1] // null) as $nx
          | if $nx != null and $nx.role == "field_value"
            then {kind: "field", k: ($x | val), v: ($nx | val | parse_value($nx | has_tag("typed"))), conf: $nx.confidence}
            else {kind: "field", k: ($x | val), v: ""} end
        elif $x.role == "query" then
          if ($x.text | contains("==")) then ($x.text | capture("^(?<k>[^=]*)==(?<v>.*)$") | {kind: "query", k, v})
          else ($t[$i + 1] // null) as $nx
            | if $nx != null and $nx.role == "field_value" then {kind: "query", k: ($x | val), v: ($nx | val)}
              else {kind: "query", k: ($x | val), v: ""} end
          end
        elif $x.role == "field_value" then
          ($t[$i - 1] // null) as $pv
          | if $i > 0 and $pv != null and ($pv.role == "field_key" or ($pv.role == "query" and ($pv.text | contains("==") | not))) then empty
            else {kind: "error", msg: ("value \"" + $x.text + "\" has no key")} end
        else empty end]) as $items
  | ($items | map(select(.kind == "field")))  as $fields
  | ($items | map(select(.kind == "query")))  as $queries
  | ($items | map(select(.kind == "error") | .msg)) as $value_errors

  # --- 衝突 (jurl assemble の conflicts / Unresolved) ------------------------------------------------
  | ([ (if ($methods | length) > 1 then "method " + ($methods | join(" and ")) else empty end),
       (if ($urls | length) > 1 then "url " + ($urls | join(" and ")) else empty end),
       (if ($cts | length) > 1 then "content-type " + ($cts | join(" and ")) else empty end),
       ($value_errors[]),
       (if ($t | map(select(.role == "port") | .text | tonumber) | any(. > 65535)) then "port out of range" else empty end)
     ]) as $conflicts
  | if ($conflicts | length) > 0 then {argv: null, preview: null, risk: "none", pipe: null, error: ($conflicts | join("; "))}
    elif ($urls | length) == 0 then {argv: null, preview: null, risk: "none", pipe: null, error: "no url"}
    else
      # --- URL を組む -------------------------------------------------------------------------------
      ($urls[0] | parse_url
        | (if .port == null and ($ports | length) > 0 then .port = $ports[0] else . end)
        | reduce $paths[] as $p (.; push_path($p))
        | .query += [$queries[] | [.k, .v]]
        | render_url) as $url

      # --- メソッド: 省略時はボディがあれば POST、無ければ GET (jurl 決定 2026-09-22) -----------------
      | (($fields | length) > 0 or ($body_files | length) > 0) as $has_body
      | ($methods[0] // (if $has_body then "POST" else "GET" end)) as $method

      # --- ボディと Content-Type -----------------------------------------------------------------------
      | ($cts[0] // $d.content_type // "application/json") as $ct
      | (if ($body_files | length) > 0 then ["--data-binary", "@" + $body_files[0]]
         elif ($fields | length) == 0 then []
         elif ($ct | startswith("multipart/")) then [$fields[] | "-F", (.k + "=" + (.v | scalar))]
         elif $ct == "application/x-www-form-urlencoded" then [build_body($fields) | flatten_form("") | "--data-urlencode", (.[0] + "=" + .[1])]
         else ["--data", (build_body($fields) | tojson)] end) as $body_args
      | (if $has_body then $ct elif ($cts | length) > 0 then $cts[0] else null end) as $content_type
      | ($headers | map(.[0] | ascii_downcase)) as $hnames
      | (if $content_type == null then []
         else (if ($hnames | index("content-type")) == null then ["-H", "Content-Type: " + $content_type] else [] end)
            + (if $content_type == "application/json" and ($hnames | index("accept")) == null then ["-H", "Accept: application/json"] else [] end)
         end) as $ct_args

      | { argv: (["curl", "-sS"] + ($d.args // []) + $curl_extra
                 + (if $method != "GET" then ["-X", $method] else [] end)
                 + [$url] + $ct_args
                 + [$headers[] | "-H", (.[0] + ": " + .[1])]
                 + $body_args + (.passthrough // [])),
          preview: null,
          risk: (if ($method | IN("PUT", "PATCH", "DELETE")) then "unsafe" else "none" end),
          pipe: null,
          error: null }
    end
'
