#!/usr/bin/env bash
# jx assemble for docker run。
#
# stdin : {"tokens":[{"text","role","value","amount":{"n","unit","at_least"}|null,...}], "passthrough":[...], "answers":{}, "defaults":{"args":[...]}}
# stdout: {"argv":[...], "preview":null, "risk":"none"|"unsafe", "pipe":null, "error":"..."|null}
#
# argv の並び: docker run [switches] [--name] [-p …] [-v …] [-e …] [--env-file] [--network] [-w] [-u] [-m] [--cpus]
#              [--restart] [--platform] [passthrough role の生フラグ] [defaults.args] IMAGE [cmd …] [-- の後ろ]
# `--` の後ろはコンテナ内コマンドとして image の後ろに置く (docker run のオプションではない)。
set -euo pipefail

jq -c '
  def toks(r): [.tokens[] | select(.role == r)];
  def vals(r): [toks(r)[] | .value];
  def each(flag; r): [vals(r)[] | [flag, .]] | add // [];

  # 8080 だけなら 8080:8080
  def port_arg: if test(":") then . else . + ":" + . end;

  # memory: amount があれば n + unit (512m)。単位無しは m とみなす。
  def mem_arg: if .amount != null then ((.amount.n | if . == floor then floor else . end | tostring) + (.amount.unit // "m")) else .value end;
  def cpus_arg: if .amount != null then (.amount.n | if . == floor then floor else . end | tostring) else .value end;

  def expand_home: if . == "~" then env.HOME elif startswith("~/") then env.HOME + .[1:] else . end;
  def vol_arg: (split(":") | .[0] |= expand_home) | join(":");

  (vals("image")) as $images
  | (vals("flag") | unique) as $flags
  | (vals("container_name")) as $names
  | (vals("passthrough")) as $raw
  | (.defaults.args // []) as $defaults
  | (.passthrough) as $after

  | if ($images | length) == 0 then
      {argv: null, preview: null, risk: "none", pipe: null, error: "no image: say which image to run (nginx, redis:7, ubuntu)"}
    elif ($images | length) > 1 then
      {argv: null, preview: null, risk: "none", pipe: null, error: ("more than one image: " + ($images | join(", ")))}
    elif ($names | length) > 1 then
      {argv: null, preview: null, risk: "none", pipe: null, error: ("more than one name: " + ($names | join(", ")))}
    else
      {
        argv: (
          ["docker", "run"]
          + ([$flags[] | select(. != "-it")] )
          + (if ($flags | index("-it")) != null then ["-it"] else [] end)
          + (if ($names | length) == 1 then ["--name", $names[0]] else [] end)
          + ([vals("port")[] | ["-p", port_arg]] | add // [])
          + ([vals("volume")[] | ["-v", vol_arg]] | add // [])
          + each("-e"; "env")
          + each("--env-file"; "env_file")
          + each("--network"; "network")
          + each("-w"; "workdir")
          + each("-u"; "user")
          + ([toks("memory")[] | ["-m", mem_arg]] | add // [])
          + ([toks("cpus")[] | ["--cpus", cpus_arg]] | add // [])
          + each("--restart"; "restart")
          + each("--platform"; "platform")
          + $raw
          + $defaults
          + [$images[0]]
          + vals("cmd")
          + $after
        ),
        preview: null,
        risk: (if ($flags | index("--privileged")) != null then "unsafe" else "none" end),
        pipe: null,
        error: null
      }
    end
'
