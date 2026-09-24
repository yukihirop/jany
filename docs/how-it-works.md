# How it works

What happens between the words you type and the line on your prompt. [Back to the README](../README.md)

jany generalises [jind](https://github.com/yukihirop/jind) (jev × find) and [jurl](https://github.com/yukihirop/jurl) (jev × curl) into one binary. Each command is a **definition directory** — three files, no Rust — and adding a command is a job for an LLM skill, not a new crate.

<p align="center">
  <img src="flow.svg" alt="words → rules → all resolved? yes: assemble.sh → your prompt. no: jev (one request) → repair → assemble.sh → your prompt. Dashed boxes are the per-command definition, solid ones the host." width="880">
</p>

- **rules** — the unambiguous shapes are decided by data in `schema.toml`: paths, globs, `8080:80`, `KEY=value`, `+7d`, `>10M`, table words like `files`/`delete`/`background`. If every word resolves, jany never leaves your machine.
- **jev** — anything left over goes to [jev](https://openrouter.ai) (TypeSafe System One, via OpenRouter) in **one request**: "what is the role of each word?" plus the extra questions the schema declares (which unit? at least or at most? a typo of which table word?). jev only picks from fixed choices and returns probabilities; it never generates the command.
- **repair** — fixes what jev cannot see word by word: attach `days` to `7`, let `except` claim the names after it, pair `first_name amanda` into key and value.
- **assemble.sh** — the command's own script (stdin JSON → stdout JSON) turns the role-tagged words into `argv`. It also says how risky the result is.
- **output** — the line goes to stdout, everything else to stderr. When the words can't be turned into a command (unresolved, or below a confidence floor), jany exits non-zero and puts `jany <command> --hint` on the prompt instead, with the reason as a comment. A `dangerous` result (find `-delete`) is previewed first with a read-only run. An `unsafe` one (anything that changes state: curl `POST`/`DELETE`, `docker run`) gets a one-line note, and `autorun` leaves it on the prompt.

One jev call is 200–700 ms and under $0.0001.

## Where it is weak

- The same weaknesses as jind and jurl: a bare word (`log`, `app`) can be two roles at once, and jev is often only 0.6–0.9 sure. Write the unambiguous form (`*.log`) to skip jev.
- Commands whose vocabulary does not close — arbitrary SQL, jq programs, ffmpeg filter graphs — are not a fit. Cover the common forms and pass the rest through.
- Every word, including ones the rules already decided, is sent to jev as context. Roles that may carry secrets (curl headers, docker `KEY=value`) declare a `mask` so only a placeholder goes out.
