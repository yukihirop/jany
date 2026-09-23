---
name: jany-register
description: Add a command definition (schema.toml / assemble.sh / cases.toml) to jany. `/jany-register <name> [sub]` maps the words for the "common 80%" of that command onto roles, and keeps fixing the definition until every case passes `jany --test`.
---

# jany-register — add one command to jany

jany turns "a loosely ordered run of words → one command line", driven by three definition files per command.
This skill writes those three files and gets them to the point where `jany --test <name>` passes.

- Every DSL key and the assemble contract: [reference.md](reference.md) (**read it first**. The schema rejects unknown keys)
- Worked examples: `examples/find/` (jind equivalent: amount, claim, dangerous/preview, pipe), `examples/curl/` (jurl equivalent: next, once, regex, join, pair, mask, scope=command)
- Location: `~/.config/jany/cmd/<name>[/<sub>]/` (or `JANY_CMD_DIR` if set). This skill creates the scaffold itself

## Steps

### 1. Decide the scope (research before asking)

- Read the binary's `--help` / man page for `<name>` and pick the **commonly used 80% of options**. Do not cover everything (the same way jind settled for `-mtime` only). Anything left out is passed through after `--`
- Commands whose role set is open-ended (arbitrary SQL statements, jq expressions, ffmpeg filters) are out of scope. Do not force them into roles
- Write down **at least 10** word sequences people would actually type. Include the ways they break (wrong order, typos, units split off, glued words like `8080:80`). These become the cases
- If there are subcommands, use a hierarchy: `/jany-register docker run` → `cmd/docker/run/`

### 2. Place the scaffold

Use `JANY_CMD_DIR` as the definition root if set, otherwise `~/.config/jany/cmd/`. Join `<name> [sub]` as path components and create the target directory. If a `schema.toml` already exists there, do not overwrite it; stop at that point.

Copy this skill's `template/schema.toml`, `template/assemble.sh` and `template/cases.toml` into the target directory as the scaffold. Replace `__NAME__` with the command name (the subcommands joined with spaces) and `__ARGV0__` with the JSON array items for the head of assemble's argv. Make `assemble.sh` executable. Use your environment's normal editing tools for the file operations.

Do not call `jany --register`. This skill alone takes the definition from scaffold to finished.

### 3. Write schema.toml

Think about it in this order:

1. **roles**: 10–20 roles that words can take. Write each `jev` description in English with concrete examples (jev chooses by looking at these alone). Give roles that may contain secrets a `mask`
2. **tables**: word → corrected value (`file`/`files`/`f` → `f`). These also become the candidates for typo correction
3. **amount**: if there are numeric words, a unit table. Dimensions are `time` / `size` / `count`
4. **rules**: tried top to bottom. Start with the certain ones (flags, words containing `=` or `:`, paths, globs, table words). **Words decided by rules never call jev**, so make every unambiguous word a rule. Two words that form one thing (`-p 8080:80`) use `next`
5. **questions**: what to ask jev about words the rules could not decide. `role.i` is automatic. Typo correction, units, direction, joining (`join`), and so on
6. **repair**: adjacency (attach a unit to its number, let a marker take its neighbour, alternate key/value)
7. **confirm / defaults**: `preview_readonly` if destructive, `unsafe_note` if hard to undo

### 4. Write assemble.sh

- stdin JSON → stdout JSON. bash + jq is fine (imitate the examples). `chmod +x`
- The head of argv (the binary name) is decided here too
- For destructive operations return `risk = "dangerous"` plus a `preview` that shows the targets without destroying anything. For things that are hard to undo, `"unsafe"`
- For combinations that cannot be assembled, put the reason in `error` (do not invent an argv)
- **jq's `//` drops `false`**. Check booleans with `== true`

### 5. Write cases.toml and run it

```
jany --test <name> [sub]
jany --test <name> [sub] --explain      # show the per-word role table
```

- Turn every word sequence from step 1 into a case. Cases decided by rules alone get `no_jev = true`
- Write jev's answers as a Mock in `[case.jev]`. **Answering a key that was not asked is a failure**, so check what gets asked with `--explain` before writing them
- The cases' `words` are shown to people as-is as the examples of `jany <name> --hint` (cases with `error` / `setup` / `defaults`, and anything after the first 8, are not shown). **Put the common phrasings first**, in the word order people would type. The `jev` descriptions in `[[roles]]` are also shown by `--hint`
- Fix schema / assemble until everything passes. **Do not trust an assemble.sh without cases** (when an LLM writes one, mistakes like the `//` trap creep in)
- Once they pass, try 2–3 on the real thing: `jany <name> …` (this actually calls jev and needs `OPENROUTER_API_KEY`). Check that the line printed on stdout is what you meant. jany never runs it, so this is safe

### 6. Report

- Options you covered / options you left out, and why
- The number of cases and the result (how many passed / failed, with the output)
- Words where jev's answer is likely to split (a bare word torn between two roles, etc.)

## Cautions

- Keep facts and guesses apart. Check the binary's option names against `--help`. Say "not verified" for anything you did not verify
- Do not register commands nobody asked for
- Do not overwrite existing definitions under `~/.config/jany/cmd/`. If a `schema.toml` already exists, leave it unchanged and report it to the user
