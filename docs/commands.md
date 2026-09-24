# Commands

How a command definition is made, what is in it, and how to bring it up to date. [Back to the README](../README.md)

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
