---
name: jany-update
description: Bring a definition (schema.toml / assemble.sh / cases.toml) written for an older jany up to date by adding only the features jany has gained since. `/jany-update <name> [sub]` never rewrites existing rules, roles or cases; it adds what is missing and checks that `jany --test` passes at least as many cases as before.
---

# jany-update — bring an existing definition up to the current jany

A definition made with `/jany-register` only uses what jany could do when it was written.
This skill adds **only the difference**. It does not touch rules that already work.

- The current DSL: the jany-register skill's reference.md (next to this skill, `../jany-register/reference.md`; **read it first**)
- The latest built-in definitions: `../jany-register/examples/{find,curl,docker/run}/`
- Location: `~/.config/jany/cmd/<name>[/<sub>]/` (or `JANY_CMD_DIR` when set)

## Steps

### 1. Ask jany what is missing

```
jany --update <name> [sub]
```

- A built-in definition (find / curl / docker run) that you have not edited is replaced with the latest one by this alone. If it says "updated" or "up to date", you are done
- "lacks …" lists the missing features. "you have edited …" means an edited built-in (see 3)
- If it says there is no such definition, stop and tell the user (that is a job for `/jany-register`)

### 2. Record the state before

```
jany --test <name> [sub]
```

Note how many cases pass and fail. **Do not fix cases that already fail** (out of scope for this skill); mention them in the report.

### 3. Add only what is missing

- Read reference.md and look for keys and sections the definition does not have. Everything under "lacks" in 1 must be added; anything else in reference.md but not in the definition is a candidate
- Write what you add the way reference.md describes it and the examples do it
- **Do not rewrite existing `[[roles]]`, `[[rules]]`, `[[questions]]`, `[[repair]]`, assemble.sh or existing cases.** When a new section refers to roles, use the existing role names as they are
- For an edited built-in, compare it with `../jany-register/examples/<name>/` and bring in only what the latest version added, keeping the user's edits. Where the user's edit and the latest version conflict, do not rewrite; report it

Examples of what to add:

| Feature | What to add |
|---|---|
| dim hint in zsh | 3 to 6 `[[placeholders]]` slots in the order people say them, with `bare = true` on the slot a bare word usually fills. `roles` may only name roles in `[[roles]]` |

### 4. Check

```
jany --test <name> [sub]                 # at least as many pass as in 2
jany --suggest -- <name> [sub]           # after adding placeholders: every slot shows
jany --suggest -- <name> [sub] <words …> # with the words of 2 or 3 cases: the filled slots disappear
jany --update <name> [sub]               # no more "lacks"
```

If fewer cases pass than in 2, revert what you added and find out why. Do not report it as passing.

### 5. Report

- What you added (files and sections), and that nothing existing changed
- `jany --test` before and after (with the output)
- What you did not bring in, conflicts, and why

## Notes

- Keep facts and guesses apart. Say "not verified" for what you have not checked
- Do not touch definitions you were not asked about. Even if `jany --update` says several definitions "lack" something, update only the one requested
- Do not delete files. Do not recreate the definition directory
