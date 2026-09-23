//! `jany <command> --hint`: prints what you can say to that command (its roles) and examples (cases.toml) on stderr.
//! The examples are cases.toml's words as-is, so only phrasings that pass the tests are shown.

use crate::color::{self, C, paint};
use crate::output;
use crate::schema::Schema;

/// How many examples to show. For the rest, only the count is given.
const MAX_EXAMPLES: usize = 8;

pub fn run(schema: &Schema) -> i32 {
    eprint!("{}", text(schema, color::stderr_enabled()));
    0
}

pub fn text(schema: &Schema, on: bool) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    let name = &schema.command.name;

    let roles = schema.jev_roles();
    if !roles.is_empty() {
        let _ = writeln!(s, "{}", paint(on, C::Dim, &format!("what you can say to `jany {name}`:")));
        let w = roles.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
        for (k, d) in roles {
            let _ = writeln!(s, "  {}{}  {d}", paint(on, C::Cyan, k), " ".repeat(w - k.len()));
        }
    }

    let examples = examples(schema);
    if !examples.is_empty() {
        let _ = writeln!(s, "\n{}", paint(on, C::Dim, "examples (from cases.toml):"));
        for (words, cmd) in examples.iter().take(MAX_EXAMPLES) {
            let _ = writeln!(s, "  jany {name} {words}");
            if let Some(cmd) = cmd {
                let _ = writeln!(s, "    {}", paint(on, C::Dim, &format!("→ {cmd}")));
            }
        }
        if examples.len() > MAX_EXAMPLES {
            let _ = writeln!(s, "{}", paint(on, C::Dim, &format!("  … {} more in {}", examples.len() - MAX_EXAMPLES, schema.dir.join("cases.toml").display())));
        }
    }
    s
}

/// The line put on the prompt when a command could not be built:
/// `jany find --hint  # could not interpret: edtied, wthin`. Every word after `#` is shell-quoted,
/// so even with interactive comments off (zsh's default) it only reaches jany as words it ignores.
pub fn retry_line(command: &[String], e: &crate::error::JanyError) -> String {
    use crate::error::JanyError;
    let why = match e {
        JanyError::Unresolved(s) => format!("could not interpret: {}", s.trim_end_matches(" (jev disabled)")),
        JanyError::LowConfidence(c, _) => format!("not sure enough (confidence {c:.2})"),
        other => other.to_string(),
    };
    let mut argv = vec!["jany".to_string()];
    argv.extend(command.iter().cloned());
    argv.push("--hint".into());
    let comment: Vec<String> = why.split_whitespace().map(|w| shell_words::quote(w).into_owned()).collect();
    format!("{}  # {}", shell_words::join(&argv), comment.join(" "))
}

/// For an intermediate name without a definition (`jany docker --hint`), the definitions under it with one example each. None if it is not such a name.
pub fn subcommands(cmd_dir: &std::path::Path, words: &[String]) -> Option<String> {
    if words.is_empty() || words.iter().any(|w| w.is_empty() || w.contains('/') || w.starts_with('.')) {
        return None;
    }
    let dir = words.iter().fold(cmd_dir.to_path_buf(), |d, w| d.join(w));
    let mut subs: Vec<_> = std::fs::read_dir(&dir).ok()?.filter_map(|e| e.ok()).filter(|e| e.path().join("schema.toml").exists()).collect();
    if subs.is_empty() {
        return None;
    }
    subs.sort_by_key(|e| e.file_name());
    let on = color::stderr_enabled();
    let name = words.join(" ");
    let mut s = format!("{}\n", paint(on, C::Dim, &format!("`jany {name}` needs one more word:")));
    for e in subs {
        let sub = e.file_name().to_string_lossy().to_string();
        let ex = Schema::load(&e.path()).ok().and_then(|x| x.command.example).map(|x| paint(on, C::Dim, &format!("  e.g. jany {name} {sub} {x}"))).unwrap_or_default();
        s.push_str(&format!("  {}{ex}\n", paint(on, C::Cyan, &sub)));
    }
    s.push_str(&paint(on, C::Dim, &format!("then: jany {name} <sub> --hint\n")));
    Some(s)
}

/// (words to type, resulting command). Cases that expect an error, set up directories, or override defaults
/// would not come out the same on your machine, so they are left out. No examples if cases.toml is missing or unreadable.
fn examples(schema: &Schema) -> Vec<(String, Option<String>)> {
    let Ok(text) = std::fs::read_to_string(schema.dir.join("cases.toml")) else { return Vec::new() };
    let Ok(t) = toml::from_str::<toml::Table>(&text) else { return Vec::new() };
    let strs = |v: Option<&toml::Value>| -> Option<Vec<String>> { v?.as_array()?.iter().map(|x| x.as_str().map(str::to_string)).collect() };
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    for c in t.get("case").and_then(|c| c.as_array()).into_iter().flatten() {
        let Some(c) = c.as_table() else { continue };
        if ["error", "setup", "defaults"].iter().any(|k| c.contains_key(*k)) {
            continue;
        }
        let Some(words) = strs(c.get("words")) else { continue };
        let mut line = shell_words::join(&words);
        if let Some(p) = strs(c.get("passthrough")).filter(|p| !p.is_empty()) {
            line = format!("{line} -- {}", shell_words::join(&p));
        }
        if out.iter().any(|(l, _)| l == &line) {
            continue;
        }
        let pipe = strs(c.get("pipe"));
        let cmd = strs(c.get("argv")).map(|a| output::render(&a, pipe.as_deref()));
        out.push((line, cmd));
    }
    out
}
