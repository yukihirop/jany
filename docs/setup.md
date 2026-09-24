# Setup

Everything here is what `/jany-setup` does for you, plus the parts you can turn on later. [Back to the README](../README.md)

## By hand

```sh
cargo install jany            # or, from a checkout: cargo install --path .
jany --setup                  # store your OpenRouter API key in ~/.config/jany/config.toml (0600)
echo 'eval "$(jany --init zsh)"' >> ~/.zshrc     # bash and fish too; bash is untested
# optional: aliases get the same completion and hint (zsh), also ones that name a command
printf '%s\n' "alias j='jany'" "alias jpnpm='j pnpm'" >> ~/.zshrc
```

`jany --init` does three things: prints the wrapper function, installs the built-in definitions (`find`, `curl`, `docker run`) into `~/.config/jany/cmd/`, and installs the `/jany-setup`, `/jany-register` and `/jany-update` skills into `~/.agents/skills/` (linked from `~/.claude/skills/` and `~/.codex/skills/` when those exist). It never overwrites a definition that is already there. The skills are in English by default; `jany --init zsh --locale ja` installs the Japanese one (put the flag in your rc line, since `--init` rewrites the skill on every shell start).

`jany --skills` installs only the skills, for `/jany-setup` before the first `--init`. `/jany-setup` never reads your API key; without one it asks you to run `jany --setup`.

## Dim hint (zsh)

In zsh the wrapper also shows a dim hint of what you can still say after `jany <command> ` (the definition's `[[placeholders]]`), e.g. `jany find src ` → `<file|dir> <*.log> <older than N days> <delete|count>`. It never calls jev. `[suggest] enabled = false` in `~/.config/jany/config.toml` turns it off; `JANY_SUGGEST=0` / `1` overrides that for one shell. bash and fish don't have it.

## autorun (zsh)

`[cmd.<name>] autorun = true` in `~/.config/jany/config.toml` lets the zsh wrapper run the line instead of putting it on the prompt, but only when the rules decided every word and the definition calls it risk `"none"`: no jev, no words after `--`, no raw `-x` flags, no preview or pipe. `jany <command> -- --help` and `-- --version` with nothing else also run. `autorun_also = ["pnpm install"]` lets lines that start with those words run even when the definition calls them `"unsafe"` (compared word by word on the final argv, so `jany pnpm install react`, which becomes `pnpm add react`, does not match; `"dangerous"` never runs). The line is shown on stderr and still goes into your history. Anything else goes on the prompt as before. Off by default, and bash / fish always put the line on the prompt.

## jany --on (zsh)

`jany --on` (zsh only) lets you leave out `jany` in that shell until `jany --off`: on Enter, a line that starts with a command jany has a definition for and says something after it goes through jany, so `find empty folders` puts `find . -type d -empty` on the next prompt, and `jany find empty folders` is what stays in the history. A line with a word starting with `-` (`find . -name x`), a pipe, a list or a redirection runs as typed, and so does the command alone (`find`) or a subcommand jany has no definition for (`docker ps`). `command find …` or `\find …` always runs as typed. `[on] skip = ["kubectl", "docker compose"]` in `~/.config/jany/config.toml` keeps lines starting with those words (compared word by word) as typed, and gives them no dim hint. The dim hint shows for the other lines too.

<p align="center">
  <img src="on.svg" alt="Flow after jany --on: Enter on a line without jany → jany takes it? (jany --claim, no jev). yes: jany is put in front, which is what stays in history → jany (rules → jev → assemble) → your prompt, or it runs right away with autorun when safe. no (a - word, a pipe or redirection, the command alone, no definition, a quoted first word, [on] skip): the line runs as typed." width="880">
</p>

## API key

`OPENROUTER_API_KEY` in the environment takes precedence; a key saved by `jind setup` or `jurl setup` is picked up too. Tested on macOS with zsh.
