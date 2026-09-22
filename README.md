<p align="center">
  <b>jx</b> — jev × any command. Say what you want, in any order. Get the command line you meant, on your prompt.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT"></a>
</p>

```sh
$ jx find log files older than 7 days in /var/log delete
this command is destructive.
$ find /var/log -type f -iname '*.log' -mtime +7
  /var/log/system.log.3
  /var/log/install.log.1
  … 40 more

$ find /var/log -type f -iname '*.log' -mtime +7 -delete█
```

The last line is not output. It is your **next prompt, already filled in**. jx never runs anything: it prints one shell-quoted line, and the wrapper from `jx --init zsh` puts it on the command line. You read it, edit it if you like, and press Enter.

```sh
$ jx curl psot localhsot 3000 users first_name amanda
$ curl -sS -X POST http://localhost:3000/users -H 'Content-Type: application/json' -H 'Accept: application/json' --data '{"first_name":"amanda"}'

$ jx docker run nginx 8080:80 background named web
$ docker run -d --name web -p 8080:80 nginx

$ jx find empty folders depth 2 count
$ find . -maxdepth 2 -type d -empty | wc -l
```

jx generalises [jind](https://github.com/yukihirop/jind) (jev × find) and [jurl](https://github.com/yukihirop/jurl) (jev × curl) into one binary. Each command is a **definition directory** — three files, no Rust — and adding a command is a job for an LLM skill, not a new crate.

## How it works

```
words ──► rules ──► all resolved? ──yes──► assemble.sh ──► one line on stdout ──► your prompt
                        │ no
                        ▼
                  jev (one request) ──► repair ──┘
```

- **rules** — the unambiguous shapes are decided by data in `schema.toml`: paths, globs, `8080:80`, `KEY=value`, `+7d`, `>10M`, table words like `files`/`delete`/`background`. If every word resolves, jx never leaves your machine.
- **jev** — anything left over goes to [jev](https://openrouter.ai) (TypeSafe System One, via OpenRouter) in **one request**: "what is the role of each word?" plus the extra questions the schema declares (which unit? at least or at most? a typo of which table word?). jev only picks from fixed choices and returns probabilities; it never generates the command.
- **repair** — fixes what jev cannot see word by word: attach `days` to `7`, let `except` claim the names after it, pair `first_name amanda` into key and value.
- **assemble.sh** — the command's own script (stdin JSON → stdout JSON) turns the role-tagged words into `argv`. It also says how risky the result is.
- **output** — the line goes to stdout, everything else to stderr. Below a confidence floor jx prints nothing and exits non-zero. A `dangerous` result (find `-delete`) is previewed first with a read-only run. An `unsafe` one (curl `DELETE`, docker `--privileged`) gets a one-line note.

One jev call is 200–700 ms and under $0.0001.

## Setup

```sh
cargo install --path .      # not on crates.io yet
jx --setup                  # store your OpenRouter API key in ~/.config/jx/config.toml (0600)
echo 'eval "$(jx --init zsh)"' >> ~/.zshrc     # bash and fish too; bash is untested
```

`jx --init` does three things: prints the wrapper function, installs the built-in definitions (`find`, `curl`, `docker run`) into `~/.config/jx/cmd/`, and installs the `/jx-register` skill into `~/.agents/skills/` (linked from `~/.claude/skills/` and `~/.codex/skills/` when those exist). It never overwrites a definition you have edited.

`OPENROUTER_API_KEY` in the environment takes precedence; a key saved by `jind setup` or `jurl setup` is picked up too. Tested on macOS with zsh.

## Usage

```
jx <command> [words ...] [flags] [-- passthrough args]
```

| flag | |
|---|---|
| `--explain` | per-word role, confidence, and whether a rule or jev decided it (stderr) |
| `--no-jev` | offline only; unresolved words are an error |
| `-- …` | passed through untouched (what that means is up to the command: find options, curl flags, the container command for docker run) |

jx's own actions are flags, so `<command>` is always the tool's name:

| | |
|---|---|
| `jx --list` | the definitions found, with an example each |
| `jx --test find` | run a definition's `cases.toml` (jev answers are mocked) |
| `jx --register tar` | scaffold `~/.config/jx/cmd/tar/` |
| `jx --init zsh\|bash\|fish` | the wrapper, plus built-ins and the skill |
| `jx --setup` | save the API key |

## Adding a command

```sh
jx --register tar           # three template files
/jx-register tar            # in Claude Code or Codex: fill them in, run the cases
jx --test tar
```

A definition lives in `~/.config/jx/cmd/<name>[/<sub>]/`:

| file | |
|---|---|
| `schema.toml` | roles (what jev may choose from), word tables, rules, questions for jev, repair steps, risk settings |
| `assemble.sh` | role-tagged tokens in, `{argv, preview, risk, pipe, error}` out; any language, bash + jq is enough |
| `cases.toml` | words → expected argv, with jev's answers written down; `jx --test` refuses answers to questions jx did not ask |

The skill reads a reference of every schema key and the two worked examples (`find`, `curl`) before writing. The rule of thumb from jind and jurl carries over: cover the 80 % you actually type, pass the rest through after `--`, and do not trust an `assemble.sh` that has no cases.

## Config (optional)

`~/.config/jx/config.toml`

```toml
[jev]
model = "typesafe/jev-1.13"
reject_below = 0.5           # below this, print nothing and exit non-zero

[cmd.curl.defaults]          # overrides the schema's [defaults]
content_type = "text/plain"

[cmd.find.aliases]
dl = "~/Downloads"
```

## Where it is weak

- The same weaknesses as jind and jurl: a bare word (`log`, `app`) can be two roles at once, and jev is often only 0.6–0.9 sure. Write the unambiguous form (`*.log`) to skip jev.
- Commands whose vocabulary does not close — arbitrary SQL, jq programs, ffmpeg filter graphs — are not a fit. Cover the common forms and pass the rest through.
- Every word, including ones the rules already decided, is sent to jev as context. Roles that may carry secrets (curl headers, docker `KEY=value`) declare a `mask` so only a placeholder goes out.

---

<p align="center"><sub><code>design/</code> holds the built-in definitions and <code>design/HOST.md</code> the line between what the host does and what a schema does. Each module in <code>src/</code> starts with a comment on what it does. The jev wire format follows eg-jev's <code>packages/recipes/src/lib/{openrouter,questions}.ts</code>.</sub></p>
