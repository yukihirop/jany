# Usage

All the flags, jany's own actions and `~/.config/jany/config.toml`. [Back to the README](../README.md)

## Flags

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
| `jany --skills` | only the skills, before the first `--init` (then `/jany-setup`) |
| `jany --init zsh\|bash\|fish` | the wrapper, plus built-ins and the skill (`--locale en\|ja`, default `en`) |
| `jany --setup` | save the API key |

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
