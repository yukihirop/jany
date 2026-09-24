---
name: jany-setup
description: Set jany up for the first time. Asks which shell to use it in, writes the `jany --init` line to that shell's rc, and places the built-in definitions and skills. In zsh it also asks whether to write `jany --on` (type commands without `jany`) to the rc. Invoke with `/jany-setup`, or when asked to "set up jany" or "configure jany".
---

# jany-setup — first-time setup of jany

jany is a CLI that turns loosely ordered words into a command line and puts it on the shell's prompt. Putting it on the prompt is the job of a shell wrapper function, so the rc needs a `jany --init <shell>` line. This skill writes it.

**Ask with AskUserQuestion** (the shell and `jany --on`). Do not decide for the user.
Read the rc before changing it. If the same line is already there, do not add it again.

## Steps

### 1. Is jany installed?

```
jany --version
```

If not, stop and point to `cargo install jany` (this skill does not install it).

### 2. Ask for the shell

Look at the login shell with `echo $SHELL`, make it the recommendation, and ask with AskUserQuestion. The options are these three.

| Shell | rc | Line to write |
|---|---|---|
| zsh | `~/.zshrc` | `eval "$(jany --init zsh)"` |
| bash | `~/.bashrc` | `eval "$(jany --init bash)"` |
| fish | `~/.config/fish/config.fish` | `jany --init fish \| source` |

- Put the recommended option first and end its label with "(Recommended)"
- Say in the descriptions: the dim hint, `jany --on` and autorun are zsh only. The bash wrapper has not been tested by the author

### 3. Write the rc

- If the rc is a symlink, change the file it points to (check with `ls -l`). If there is no rc, create it (for fish, `~/.config/fish/` too)
- If there is already a line containing `jany --init`, do not add one. If it differs (another shell, another `--locale`), leave it and mention it in the report
- Otherwise append:

```
# jany: put the assembled command on the next prompt
<the line from the table in 2>
```

### 4. Run it once to place the definitions and skills

The rc line takes effect in the next shell. Place the definitions and skills now:

```
jany --init <shell> >/dev/null
```

Use the `jany: installed …` lines on stderr in the report. Also check the rc's syntax (`zsh -n ~/.zshrc`, `bash -n ~/.bashrc`, `fish -n ~/.config/fish/config.fish`).

### 5. In zsh, ask about `jany --on`

Only for zsh. Ask with AskUserQuestion whether every new shell should start with `jany --on`.

- "Write it to the rc (Recommended)": type `find empty folders` without `jany`. On Enter, a line that starts with a command jany has a definition for and has words after it goes through jany. A line with a word starting with `-`, a pipe or a redirection runs as typed, and so does the command alone (`find`) or a subcommand with no definition (`docker ps`). **Say that everyday lines without a `-`, like `pnpm install`, also go through jany**
- "Don't write it": type `jany --on` in a shell when you want it; `jany --off` stops it

For "Write it to the rc", add this right after the line from 3 (unless it is already there). The notice it prints on every start is discarded:

```
jany --on 2>/dev/null
```

bash and fish have no `jany --on`. Do not ask; just say so in the report.

### 6. Is there an API key?

When the rules cannot decide a word, jany calls jev (OpenRouter) once. Do not read or print the key. Only check whether one exists:

```
[ -n "$OPENROUTER_API_KEY" ] && echo env
grep -l '^api_key' ~/.config/jany/config.toml ~/.config/jurl/config.toml ~/.config/jind/config.toml 2>/dev/null
```

If neither finds one, ask the user to type `! jany --setup` (the key is read hidden from a terminal, which the agent cannot do). Never have them paste the key into the conversation.

### 7. Report

- The shell chosen, the rc changed (the real path) and the lines added. For lines not added, why
- Whether `jany --on` went into the rc
- The definitions and skills placed (output of 4)
- Whether there is an API key. If not, `! jany --setup`
- It does not apply to the current shell yet: `exec zsh` (`exec bash`, `exec fish`) starts a new one
- An example to try: `jany find log files older than 7 days` (with on: `find log files older than 7 days`) → `find . -type f -iname '*.log' -mtime +7` on the next prompt
- Next: to add a command you type often, `/jany-register <name>`
