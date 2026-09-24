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
cargo install jany
jany --skills      # installs the agent skills (--locale ja for Japanese)
```

Then run `/jany-setup` in Claude Code or Codex. It asks which shell you use, writes the `jany --init` line to its rc, and in zsh asks whether to turn on `jany --on`.

To set it up by hand, and for the dim hint, autorun, `jany --on` and the API key, see [docs/setup.md](docs/setup.md). To stop using jany, `/jany-teardown` removes what it put in place.

## Usage

```
jany <command> [words ...] [flags] [-- passthrough args]
```

`--explain` shows how each word was decided, `--hint` what you can say to a command, and words after `--` are passed through untouched. All the flags, jany's own actions (`--list`, `--test`, `--update`, `--on` …) and `config.toml` are in [docs/usage.md](docs/usage.md).

## Adding a command

```sh
/jany-register tar            # in Claude Code or Codex: create, fill, and test the definition
jany --update                 # after upgrading jany (then /jany-update <name> for the ones it lists)
```

A command is a directory of three files in `~/.config/jany/cmd/<name>/`. What goes in them, and how updating works: [docs/commands.md](docs/commands.md).

## Where it is weak

- The same weaknesses as jind and jurl: a bare word (`log`, `app`) can be two roles at once, and jev is often only 0.6–0.9 sure. Write the unambiguous form (`*.log`) to skip jev.
- Commands whose vocabulary does not close — arbitrary SQL, jq programs, ffmpeg filter graphs — are not a fit. Cover the common forms and pass the rest through.
- Every word, including ones the rules already decided, is sent to jev as context. Roles that may carry secrets (curl headers, docker `KEY=value`) declare a `mask` so only a placeholder goes out.

---

<p align="center"><sub>Sister projects: <a href="https://github.com/yukihirop/jind">jind</a> (jev × find) and <a href="https://github.com/yukihirop/jurl">jurl</a> (jev × curl) — jany is their generalisation. <code>examples/</code> holds the built-in definitions and <code>docs/HOST.md</code> the line between what the host does and what a schema does. Each module in <code>src/</code> starts with a comment on what it does. <code>docs/demo.svg</code> is real output captured through a pty by <code>docs/capture-demo.py</code> and rendered by <code>docs/make-demo.py</code>.</sub></p>
