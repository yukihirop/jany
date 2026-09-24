<p align="center">
  <img src="docs/hero.svg" alt="jany — jev × any command. Say what you want, in any order. Get the command you meant, on your prompt." width="880">
</p>

<p align="center">
  <a href="https://crates.io/crates/jany"><img src="https://img.shields.io/crates/v/jany.svg" alt="crates.io"></a>
  <a href="https://crates.io/crates/jany"><img src="https://img.shields.io/crates/d/jany.svg" alt="downloads"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT"></a>
  <a href="https://github.com/yukihirop/jany/actions/workflows/ci.yml"><img src="https://github.com/yukihirop/jany/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/commands-find%20%C2%B7%20curl%20%C2%B7%20docker%20run%20%C2%B7%20yours-3b6fd1.svg" alt="find · curl · docker run · yours">
</p>

<p align="center">
  <b>English</b> · <a href="README.ja.md">日本語</a>
</p>

<p align="center">
  <b>jany</b> turns a loose pile of words — out of order, half-remembered, misspelled — into the command line you meant, and puts it on your prompt. <b>It never runs it</b> unless you opt in per command. Enter is yours.
</p>

> [!NOTE]
> **Status: experimental.** The command definitions and shell integrations may change before 1.0.

<p align="center">
  <img src="docs/demo.svg" alt="Terminal demo: 'find log files older than 7 days in logs delete --explain' showing the per-word table, the read-only preview of what would be deleted, then the find landing on the next prompt; 'curl psot localhsot 3000 users first_name amanda' becoming a POST with a JSON body; 'docker run nginx 8080:80 background named web'; 'tar extrct app.tar.gz into dist strip 1' listing the archive first" width="930">
</p>

```sh
$ jany find log files older than 7 days in logs delete
this command is destructive.
$ find logs -type f -iname '*.log' -mtime +7
  logs/old-access.log
  logs/kernel.log
  logs/system.log
  … 2 more

$ find logs -type f -iname '*.log' -mtime +7 -delete█
```

The last line is not output. It is your **next prompt, already filled in**. jany prints one shell-quoted line on stdout, and the wrapper from `jany --init zsh` puts it on the command line. Read it, edit it if you like, press Enter.

```sh
$ jany curl psot localhsot 3000 users first_name amanda      # typos, a bare port, key value as two words
$ curl -sS -X POST http://localhost:3000/users -H 'Content-Type: application/json' -H 'Accept: application/json' --data '{"first_name":"amanda"}'

$ jany docker run nginx 8080:80 background named web
$ docker run -d --name web -p 8080:80 nginx

$ jany tar extrct app.tar.gz into dist strip 1                # a definition written by the /jany-register skill
$ tar -xf app.tar.gz --strip-components 1 -C dist

$ jany find empty folders depth 2 count
$ find . -maxdepth 2 -type d -empty | wc -l
```

jany generalises [jind](https://github.com/yukihirop/jind) (jev × find) and [jurl](https://github.com/yukihirop/jurl) (jev × curl) into one binary. Each command is a **definition directory** — three files, no Rust — and adding a command is a job for an LLM skill, not a new crate.

## How it works

<p align="center">
  <img src="docs/flow.svg" alt="words → rules → all resolved? yes: assemble.sh → your prompt. no: jev (one request) → repair → assemble.sh → your prompt. Dashed boxes are the per-command definition, solid ones the host." width="880">
</p>

- **rules** — the unambiguous shapes are decided by data in `schema.toml`: paths, globs, `8080:80`, `KEY=value`, `+7d`, `>10M`, table words like `files`/`delete`/`background`. If every word resolves, jany never leaves your machine.
- **jev** — anything left over goes to [jev](https://openrouter.ai) (TypeSafe System One, via OpenRouter) in **one request**: "what is the role of each word?" plus the extra questions the schema declares (which unit? at least or at most? a typo of which table word?). jev only picks from fixed choices and returns probabilities; it never generates the command.
- **repair** — fixes what jev cannot see word by word: attach `days` to `7`, let `except` claim the names after it, pair `first_name amanda` into key and value.
- **assemble.sh** — the command's own script (stdin JSON → stdout JSON) turns the role-tagged words into `argv`. It also says how risky the result is.
- **output** — the line goes to stdout, everything else to stderr. When the words can't be turned into a command (unresolved, or below a confidence floor), jany exits non-zero and puts `jany <command> --hint` on the prompt instead, with the reason as a comment. A `dangerous` result (find `-delete`) is previewed first with a read-only run. An `unsafe` one (anything that changes state: curl `POST`/`DELETE`, `docker run`) gets a one-line note, and `autorun` leaves it on the prompt.

One jev call is 200–700 ms and under $0.0001.

## Setup

```sh
cargo install jany          # after the crates.io release
# Before the release, install from this checkout:
cargo install --path .
jany --setup                  # store your OpenRouter API key in ~/.config/jany/config.toml (0600)
echo 'eval "$(jany --init zsh)"' >> ~/.zshrc     # bash and fish too; bash is untested
# optional: aliases get the same completion and hint (zsh), also ones that name a command
printf '%s\n' "alias j='jany'" "alias jpnpm='j pnpm'" >> ~/.zshrc
```

`jany --init` does three things: prints the wrapper function, installs the built-in definitions (`find`, `curl`, `docker run`) into `~/.config/jany/cmd/`, and installs the `/jany-register` and `/jany-update` skills into `~/.agents/skills/` (linked from `~/.claude/skills/` and `~/.codex/skills/` when those exist). It never overwrites a definition that is already there. The skills are in English by default; `jany --init zsh --locale ja` installs the Japanese one (put the flag in your rc line, since `--init` rewrites the skill on every shell start).

In zsh the wrapper also shows a dim hint of what you can still say after `jany <command> ` (the definition's `[[placeholders]]`), e.g. `jany find src ` → `<file|dir> <*.log> <older than N days> <delete|count>`. It never calls jev. `[suggest] enabled = false` in `~/.config/jany/config.toml` turns it off; `JANY_SUGGEST=0` / `1` overrides that for one shell. bash and fish don't have it.

`[cmd.<name>] autorun = true` in `~/.config/jany/config.toml` lets the zsh wrapper run the line instead of putting it on the prompt, but only when the rules decided every word and the definition calls it risk `"none"`: no jev, no words after `--`, no raw `-x` flags, no preview or pipe. `jany <command> -- --help` and `-- --version` with nothing else also run. `autorun_also = ["pnpm install"]` lets lines that start with those words run even when the definition calls them `"unsafe"` (compared word by word on the final argv, so `jany pnpm install react`, which becomes `pnpm add react`, does not match; `"dangerous"` never runs). The line is shown on stderr and still goes into your history. Anything else goes on the prompt as before. Off by default, and bash / fish always put the line on the prompt.

`jany --on` (zsh only) lets you leave out `jany` in that shell until `jany --off`: on Enter, a line that starts with a command jany has a definition for and says something after it goes through jany, so `find empty folders` puts `find . -type d -empty` on the next prompt, and `jany find empty folders` is what stays in the history. A line with a word starting with `-` (`find . -name x`), a pipe, a list or a redirection runs as typed, and so does the command alone (`find`) or a subcommand jany has no definition for (`docker ps`). `command find …` or `\find …` always runs as typed. `[on] skip = ["kubectl", "docker compose"]` in `~/.config/jany/config.toml` keeps lines starting with those words (compared word by word) as typed, and gives them no dim hint. The dim hint shows for the other lines too.

<p align="center">
  <img src="docs/on.svg" alt="Flow after jany --on: Enter on a line without jany → jany takes it? (jany --claim, no jev). yes: jany is put in front, which is what stays in history → jany (rules → jev → assemble) → your prompt, or it runs right away with autorun when safe. no (a - word, a pipe or redirection, the command alone, no definition, a quoted first word, [on] skip): the line runs as typed." width="880">
</p>

`OPENROUTER_API_KEY` in the environment takes precedence; a key saved by `jind setup` or `jurl setup` is picked up too. Tested on macOS with zsh.

## Usage

```
jany <command> [words ...] [flags] [-- passthrough args]
```

| flag | |
|---|---|
| `--explain` | per-word role, confidence, and whether a rule or jev decided it (stderr) |
| `--no-jev` | offline only; unresolved words are an error |
| `--hint` | what you can say to `<command>` (its roles) and examples from its `cases.toml` (stderr) |
| `-- …` | passed through untouched (what that means is up to the command: find options, curl flags, the container command for docker run) |

jany's own actions are flags, so `<command>` is always the tool's name:

| | |
|---|---|
| `jany --list` | the definitions found, with an example each |
| `jany --test find` | run a definition's `cases.toml` (jev answers are mocked) |
| `/jany-register tar` | create and fill `~/.config/jany/cmd/tar/` with the agent skill |
| `jany --update` | after upgrading jany: replace the built-ins you have not edited, list what the others lack, and install the skills that are missing |
| `/jany-update tar` | add only what a definition lacks, with the agent skill; existing rules and cases stay |
| `jany --on` / `jany --off` | in this zsh, type `find empty folders` without `jany` (needs the wrapper) |
| `jany --init zsh\|bash\|fish` | the wrapper, plus built-ins and the skill (`--locale en\|ja`, default `en`) |
| `jany --setup` | save the API key |

## Adding a command

```sh
jany --register tar           # optional: scaffold only, without the skill
/jany-register tar            # in Claude Code or Codex: create, fill, and test the definition
jany --test tar
```

A definition lives in `~/.config/jany/cmd/<name>[/<sub>]/`:

| file | |
|---|---|
| `schema.toml` | roles (what jev may choose from), word tables, rules, questions for jev, repair steps, risk settings |
| `assemble.sh` | role-tagged tokens in, `{argv, preview, risk, pipe, error}` out; any language, bash + jq is enough |
| `cases.toml` | words → expected argv, with jev's answers written down; `jany --test` refuses answers to questions jany did not ask |

The skill reads a reference of every schema key and the two worked examples (`find`, `curl`) before writing. The rule of thumb from jind and jurl carries over: cover the 80 % you actually type, pass the rest through after `--`, and do not trust an `assemble.sh` that has no cases.

## Updating definitions

A new jany can bring new definition features (such as `[[placeholders]]`) and updated built-ins, but `jany --init` never touches a definition that is already in place. After upgrading, run:

```sh
jany --update                 # built-ins you have not edited are replaced; the rest are listed
/jany-update tar              # in Claude Code or Codex: add what tar lacks, then jany --test tar
```

`jany --update` knows a built-in is unedited when every file matches a version jany has shipped (`examples/released.txt`). An edited built-in is left alone and listed, like your own definitions; `/jany-update` merges the new parts into it and keeps your edits. `jany --update` also installs `/jany-update` itself when it is missing (in the language of the installed `/jany-register`, or `--locale en|ja`); skill files already there are left alone.

## Config (optional)

`~/.config/jany/config.toml`

```toml
[jev]
model = "typesafe/jev-1.13"
reject_below = 0.5           # below this, no command: exit non-zero and offer `--hint`

[suggest]
enabled = false              # no dim hint in zsh (JANY_SUGGEST=0/1 overrides it)

[cmd.curl.defaults]          # overrides the schema's [defaults]
content_type = "text/plain"

[cmd.find.aliases]
dl = "~/Downloads"

[cmd.pnpm]
autorun = true               # zsh: run rule-only, risk "none" lines right away
autorun_also = ["pnpm install"]   # ...and these, even when "unsafe"
```

## Where it is weak

- The same weaknesses as jind and jurl: a bare word (`log`, `app`) can be two roles at once, and jev is often only 0.6–0.9 sure. Write the unambiguous form (`*.log`) to skip jev.
- Commands whose vocabulary does not close — arbitrary SQL, jq programs, ffmpeg filter graphs — are not a fit. Cover the common forms and pass the rest through.
- Every word, including ones the rules already decided, is sent to jev as context. Roles that may carry secrets (curl headers, docker `KEY=value`) declare a `mask` so only a placeholder goes out.

---

<p align="center"><sub>Sister projects: <a href="https://github.com/yukihirop/jind">jind</a> (jev × find) and <a href="https://github.com/yukihirop/jurl">jurl</a> (jev × curl) — jany is their generalisation. <code>examples/</code> holds the built-in definitions and <code>docs/HOST.md</code> the line between what the host does and what a schema does. Each module in <code>src/</code> starts with a comment on what it does. <code>docs/demo.svg</code> is real output captured through a pty by <code>docs/capture-demo.py</code> and rendered by <code>docs/make-demo.py</code>.</sub></p>
