//! `jany --complete -- <typed words…>`: print completion candidates, one `candidate<TAB>description` per line.
//! The shell side (the functions `jany --init` prints) only lists the result; it does not know what the candidates are.

use crate::schema::{OneOrMany, Schema};
use std::collections::BTreeSet;
use std::path::Path;

/// jany's own actions. They are flags so they never clash with <command>.
const ACTIONS: &[(&str, &str)] = &[
    (
        "--init",
        "print the shell wrapper (and install built-ins + skill)",
    ),
    ("--skills", "install only the agent skills (then /jany-setup)"),
    ("--list", "show the command definitions found"),
    ("--test", "run a definition's cases.toml"),
    ("--register", "scaffold a new command definition"),
    ("--setup", "save your OpenRouter API key"),
    ("--explain", "show how each word was classified"),
    ("--no-jev", "never call jev; unresolved words are an error"),
    ("--help", ""),
    ("--version", ""),
];

pub fn run(cmd_dir: &Path, typed: &[String]) -> i32 {
    let mut out: Vec<(String, String)> = Vec::new();
    // The final word is the shell's current partial word. Resolve the command
    // from the words before it, then filter candidates by that partial word.
    let (mut context, mut prefix) = match typed.split_last() {
        Some((last, rest)) => (rest, last.as_str()),
        None => (&[][..], ""),
    };
    // An exact command name is already complete even without a trailing space.
    let typed_words: Vec<String> = typed
        .iter()
        .filter(|w| !w.starts_with('-'))
        .cloned()
        .collect();
    // If the partial word is a flag (`find --h`), it is the prefix; do not override it.
    if !typed_words.is_empty()
        && !prefix.starts_with('-')
        && crate::schema::resolve(cmd_dir, &typed_words)
            .map(|(_, used)| used == typed_words.len())
            .unwrap_or(false)
    {
        context = typed;
        prefix = "";
    }
    let words: Vec<&String> = context.iter().filter(|w| !w.starts_with('-')).collect();

    // After --test / --register / --init, offer that action's arguments and stop.
    match context
        .iter()
        .rev()
        .find(|w| w.starts_with("--"))
        .map(String::as_str)
    {
        Some("--locale") => {
            // Nothing more after `--locale en`.
            if context.last().map(String::as_str) != Some("--locale") {
                return print(out);
            }
            for (l, d) in [("en", "English skill"), ("ja", "Japanese skill")] {
                if l.starts_with(prefix) {
                    out.push((l.into(), d.into()));
                }
            }
            return print(out);
        }
        Some("--init") => {
            // Once the shell is typed, only --locale is left.
            let shell_given = context.last().is_some_and(|w| !w.starts_with('-'));
            let cands: &[(&str, &str)] = if shell_given {
                &[("--locale", "language of the /jany-register skill (en, ja)")]
            } else {
                &[("zsh", "shell wrapper"), ("bash", "shell wrapper"), ("fish", "shell wrapper")]
            };
            for (s, d) in cands {
                if s.starts_with(prefix) {
                    out.push((s.to_string(), d.to_string()));
                }
            }
            return print(out);
        }
        Some(flag @ ("--test" | "--register")) => {
            // Returning `docker run` as one candidate makes the shell insert `docker\ run` (one argument),
            // so offer one level at a time: only the children of the directory named so far.
            let after = context.iter().skip_while(|w| w.as_str() != flag).skip(1).filter(|w| !w.starts_with('-'));
            let dir = after.fold(cmd_dir.to_path_buf(), |d, w| d.join(w.as_str()));
            for (name, description) in root_commands(&dir) {
                if name.starts_with(prefix) {
                    out.push((name, description));
                }
            }
            return print(out);
        }
        _ => {}
    }

    // No command name yet: definition names and actions.
    let resolved = crate::schema::resolve(
        cmd_dir,
        &context
            .iter()
            .filter(|w| !w.starts_with('-'))
            .cloned()
            .collect::<Vec<_>>(),
    );
    let Ok((schema, used)) = resolved else {
        // At the root, return only the first word of hierarchical commands.
        // Returning `docker run` here makes shells insert it as `docker\ run`,
        // which is one argument rather than the two words jany resolves.
        if context.is_empty() {
            for (name, description) in root_commands(cmd_dir) {
                if name.starts_with(prefix) {
                    out.push((name, description));
                }
            }
            for (f, d) in ACTIONS {
                if f.starts_with(prefix) {
                    out.push((f.to_string(), d.to_string()));
                }
            }
            return print(out);
        }
        // A namespace such as `docker` has no schema of its own; expose its
        // schema-bearing child directories (`run`) before the root list.
        let dir = context
            .iter()
            .filter(|w| !w.starts_with('-'))
            .fold(cmd_dir.to_path_buf(), |d, w| d.join(w.as_str()));
        if !context.is_empty() && dir.is_dir() {
            if let Ok(rd) = std::fs::read_dir(&dir) {
                for e in rd
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().join("schema.toml").exists())
                {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with(prefix) {
                        out.push((name, "subcommand".into()));
                    }
                }
            }
            return print(out);
        }
        for name in commands(cmd_dir) {
            if name.starts_with(prefix) {
                let ex = Schema::load(&cmd_dir.join(name.replace(' ', "/")))
                    .ok()
                    .map(|s| s.command.example.unwrap_or_default())
                    .unwrap_or_default();
                out.push((name, ex));
            }
        }
        for (f, d) in ACTIONS {
            if f.starts_with(prefix) {
                out.push((f.to_string(), d.to_string()));
            }
        }
        return print(out);
    };

    // Subcommands (hierarchies like cmd/docker/run).
    let dir = words[..used]
        .iter()
        .fold(cmd_dir.to_path_buf(), |d, w| d.join(w.as_str()));
    if let Ok(rd) = std::fs::read_dir(&dir) {
        let mut subs: Vec<_> = rd
            .filter_map(|e| e.ok())
            .filter(|e| e.path().join("schema.toml").exists())
            .collect();
        subs.sort_by_key(|e| e.file_name());
        for e in subs {
            out.push((
                e.file_name().to_string_lossy().to_string(),
                "subcommand".into(),
            ));
        }
    }

    // Words the command knows: rule words and table words.
    let already: BTreeSet<&str> = words.iter().map(|w| w.as_str()).collect();
    let mut vocab: BTreeSet<String> = BTreeSet::new();
    for r in &schema.rules {
        match &r.match_.word {
            Some(OneOrMany::One(w)) => {
                vocab.insert(w.clone());
            }
            Some(OneOrMany::Many(v)) => vocab.extend(v.iter().cloned()),
            None => {}
        }
        if let Some(t) = &r.match_.table {
            for (_, syns) in schema.table_entries(t) {
                vocab.extend(syns);
            }
        }
    }
    for (_, syns) in schema.tables.keys().flat_map(|t| schema.table_entries(t)) {
        vocab.extend(syns);
    }
    for w in vocab {
        if w.len() >= 2 && !already.contains(w.as_str()) && w.starts_with(prefix) {
            out.push((w, String::new()));
        }
    }
    for (f, d) in [
        ("--explain", "show how each word was classified"),
        ("--no-jev", "offline only"),
        ("--hint", "what you can say, with examples"),
        ("--", "pass the rest through untouched"),
    ] {
        if f.starts_with(prefix) {
            out.push((f.into(), d.into()));
        }
    }
    print(out)
}

/// Names that have a definition (space-separated, like "docker run").
fn commands(cmd_dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(dir: &Path, prefix: &str, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut names: Vec<_> = rd
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        names.sort_by_key(|e| e.file_name());
        for e in names {
            let name = e.file_name().to_string_lossy().to_string();
            let full = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix} {name}")
            };
            if e.path().join("schema.toml").exists() {
                out.push(full.clone());
            }
            walk(&e.path(), &full, out);
        }
    }
    walk(cmd_dir, "", &mut out);
    out
}

/// Root-level candidates. A directory without its own schema is a namespace
/// candidate (for example `docker`), not the flattened `docker run` command.
fn root_commands(cmd_dir: &Path) -> Vec<(String, String)> {
    let Ok(rd) = std::fs::read_dir(cmd_dir) else {
        return Vec::new();
    };
    let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).collect();
    entries.sort_by_key(|e| e.file_name());
    entries
        .into_iter()
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let description = if e.path().join("schema.toml").exists() {
                Schema::load(&e.path())
                    .ok()
                    .and_then(|s| s.command.example)
                    .unwrap_or_default()
            } else {
                "subcommand".into()
            };
            (name, description)
        })
        .collect()
}

fn print(out: Vec<(String, String)>) -> i32 {
    let mut seen = BTreeSet::new();
    let mut text = String::new();
    for (c, d) in out {
        if seen.insert(c.clone()) {
            text.push_str(&format!("{c}\t{d}\n"));
        }
    }
    crate::output::stdout(&text);
    0
}
