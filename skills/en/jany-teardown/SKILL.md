---
name: jany-teardown
description: Clean up what jany put in place when you stop using it. The `jany --init` / `jany --on` lines, aliases and `compdef` in the rc that point at jany, `~/.config/jany/` (definitions, config, API key), the skills in `~/.agents/skills/` and their symlinks, and the binary. Asks what to remove with AskUserQuestion, and shows the list for confirmation before removing anything. Invoke with `/jany-teardown`, or when asked to "uninstall jany" or "stop using jany".
---

# jany-teardown — clean up jany

Remove what `/jany-setup` and `jany --init` put in place, in the reverse order.

**Ask what to remove with AskUserQuestion.** It cannot be undone, so before removing anything, show the list of targets (paths and lines) and confirm once more.
Your own definitions and the API key are moved aside by default, not removed.

## What jany puts in place

| What | Where | What was put there |
|---|---|---|
| rc lines | `~/.zshrc` / `~/.bashrc` / `~/.config/fish/config.fish` | `eval "$(jany --init …)"` (fish: `jany --init fish \| source`), `jany --on`, aliases and `compdef` pointing at jany |
| Definitions and config | `~/.config/jany/` (or `JANY_CONFIG_DIR` / `JANY_CMD_DIR` when set) | `cmd/<name>/` (the built-in find / curl / docker run and your own definitions), `config.toml` (may hold `[jev] api_key`) |
| Skills | `~/.agents/skills/jany-{setup,register,update,teardown}/` (or the parent of `JANY_SKILL_DIR` when set) | the skills themselves |
| Symlinks | `~/.claude/skills/`, `~/.codex/skills/` | symlinks to the skills above |
| Binary | `~/.cargo/bin/jany` | `cargo install jany` |

`~/.config/{jind,jurl}/` belongs to `jind` / `jurl`, not jany. Leave it alone.

## Steps

### 1. Find

Only collect where things are; remove nothing:

```
command -v jany; jany --version
echo "$JANY_CONFIG_DIR $JANY_CMD_DIR $JANY_SKILL_DIR"
grep -n 'jany' ~/.zshrc ~/.bashrc ~/.config/fish/config.fish 2>/dev/null
jany --list
ls -la ~/.agents/skills ~/.claude/skills ~/.codex/skills 2>/dev/null | grep jany
```

- If an rc is a symlink, look at the file it points to (`ls -l`)
- From the `grep` output, pick only jany's lines: the `jany --init` line, the `jany --on` line, aliases whose value is `jany` or leads to an alias that does (`alias j='jany'`, `alias jpnpm='j pnpm'`), `compdef _jany_complete …`, and comment lines like `# jany: …`. **Do not pick other lines that merely contain jany (a path, an argument to another command).** Leave doubtful lines alone and mention them in the report
- **`grep 'jany'` misses aliases that reach jany through another alias** (`alias jpnpm='j pnpm'` has no "jany" in it). With each alias name found (`j` and so on), search `grep -nE "^alias [^=]+=['\"]?j( |['\"])" <rc>`, and repeat until no new name turns up. Pick `compdef … <name>` for those names too
- Your own definitions: those in `jany --list` other than `find` / `curl` / `docker run`
- API key: `grep -c '^api_key' ~/.config/jany/config.toml`. **Never read or print the key**
- Symlinks: only those that point to jany's skills (check with `readlink`)

### 2. Ask what to clean up

Ask with AskUserQuestion (multiSelect), with what step 1 found.

- "rc lines (Recommended)": the lines picked in step 1. **Unless these go, `jany --init` puts the skills back on every shell start.** Required when removing the binary (otherwise every new shell prints `jany: command not found`)
- "Skills and symlinks (Recommended)"
- "Definitions and config (~/.config/jany)": say the names of your own definitions and whether there is an API key
- "Binary (cargo uninstall jany)"

If "Definitions and config" is chosen, ask how with AskUserQuestion:

- "Move aside (Recommended)": move `~/.config/jany` to `~/jany-backup-<YYYYMMDD>/`, which can be moved back. Say the API key stays there too
- "Delete": your own definitions and the API key go too

### 3. Show the list and confirm

For what was chosen, list every rc line to remove (file, line number, content) and every path to move or delete, and ask with AskUserQuestion: "Clean up as listed" or "Stop". On "Stop", end without changing anything.

### 4. Clean up

In this order (removing the skills or the binary first would make the rc lines fail in the next shell):

1. **rc lines**: copy the rc to `<rc>.jany-teardown.bak` first. Remove only the lines shown in step 3. Then check the syntax (`zsh -n ~/.zshrc`, `bash -n ~/.bashrc`, `fish -n ~/.config/fish/config.fish`). If it fails, restore the backup and stop
2. **Definitions and config**: `mv` to move aside, `rm -rf` to delete. Only the paths confirmed in step 1
3. **Skills and symlinks**: the symlinks first (only those pointing to jany's skills), then the skills. **Remove this skill (jany-teardown) last**
4. **Binary**: `cargo uninstall jany`. A jany outside `~/.cargo/bin` (Homebrew and so on) is not removed; report where it is

### 5. Check

```
grep -n 'jany' <the rc changed>       # the lines shown in step 3 are gone
ls ~/.config/jany ~/.agents/skills/jany-* 2>&1
command -v jany
```

### 6. Report

- The rc lines removed (file and content) and where the rc backup is
- Where things were moved, or the paths deleted
- The skills and symlinks removed
- Whether the binary was removed
- What was kept (what was not chosen, doubtful lines left alone)
- The current shell still has jany's functions: `exec zsh` (`exec bash`, `exec fish`) starts a new one
- If moved aside, how to bring it back: `mv ~/jany-backup-<YYYYMMDD> ~/.config/jany`
