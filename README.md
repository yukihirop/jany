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

The last line in the demo is not output: it is your next prompt, already filled in. Read it, edit it if you like, and press Enter yourself.

## Get started

| | |
|---|---|
| **Start** | `cargo install jany && jany --skills`, then **`/jany-setup`** in Claude Code or Codex |
| **Use** | `jany find log files older than 7 days` |
| **Add a command** | **`/jany-register tar`** |
| **After upgrading** | `jany --update`, then `/jany-update <name>` for the ones it lists |
| **Stop** | **`/jany-teardown`** |

`/jany-setup` asks which shell you use and writes the `jany --init` line to its rc; `/jany-teardown` removes what jany put in place. `jany --skills --locale ja` installs the Japanese skills.

## More

- [Examples](docs/examples.md): what you type and the line you get, with the destructive-command preview
- [How it works](docs/how-it-works.md): rules, jev, repair, assemble.sh, and where it is weak
- [Setup](docs/setup.md): by hand, the dim hint, autorun, `jany --on`, the API key, stopping jany
- [Usage](docs/usage.md): flags, jany's own actions, `config.toml`
- [Commands](docs/commands.md): what a definition is made of, adding and updating one

---

<p align="center"><sub>Sister projects: <a href="https://github.com/yukihirop/jind">jind</a> (jev × find) and <a href="https://github.com/yukihirop/jurl">jurl</a> (jev × curl) — jany is their generalisation. <code>examples/</code> holds the built-in definitions and <code>docs/HOST.md</code> the line between what the host does and what a schema does. Each module in <code>src/</code> starts with a comment on what it does. <code>docs/demo.svg</code> is real output captured through a pty by <code>docs/capture-demo.py</code> and rendered by <code>docs/make-demo.py</code>.</sub></p>
