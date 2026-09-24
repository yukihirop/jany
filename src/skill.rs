//! Ships the `/jany-setup`, `/jany-register` and `/jany-update` skills, the `jany --register <name>` scaffold,
//! and `jany --update`, which brings the built-in definitions up to date.
//! The skills (`skills/<locale>/jany-{setup,register,update}/`, en / ja) are embedded in the binary and written on every
//! `jany --init` (and by `jany --skills`, before the first `--init`) to `~/.agents/skills/`
//! (a place both Claude Code and Codex can read).

use crate::error::JanyError;
use std::path::{Path, PathBuf};

/// The skill's language, chosen with `jany --init <shell> --locale ja`. Defaults to en.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    #[default]
    En,
    Ja,
}

impl Locale {
    pub const ALL: &[&str] = &["en", "ja"];

    pub fn parse(s: &str) -> Result<Locale, JanyError> {
        match s {
            "en" => Ok(Locale::En),
            "ja" => Ok(Locale::Ja),
            other => Err(JanyError::Usage(format!("unknown locale `{other}` ({})", Locale::ALL.join(", ")))),
        }
    }
}

/// The skills for one language (`skills/<locale>/jany-{setup,register,update}/`).
/// Paths are relative to the skills root (`~/.agents/skills/`).
macro_rules! skill_files {
    ($l:literal) => {
        &[
            ("jany-register/SKILL.md", include_str!(concat!("../skills/", $l, "/jany-register/SKILL.md"))),
            ("jany-register/reference.md", include_str!(concat!("../skills/", $l, "/jany-register/reference.md"))),
            ("jany-register/template/schema.toml", include_str!(concat!("../skills/", $l, "/jany-register/template/schema.toml"))),
            ("jany-register/template/assemble.sh", include_str!(concat!("../skills/", $l, "/jany-register/template/assemble.sh"))),
            ("jany-register/template/cases.toml", include_str!(concat!("../skills/", $l, "/jany-register/template/cases.toml"))),
            ("jany-update/SKILL.md", include_str!(concat!("../skills/", $l, "/jany-update/SKILL.md"))),
            ("jany-setup/SKILL.md", include_str!(concat!("../skills/", $l, "/jany-setup/SKILL.md"))),
        ]
    };
}
const SKILL_EN: &[(&str, &str)] = skill_files!("en");
const SKILL_JA: &[(&str, &str)] = skill_files!("ja");

/// The same in every language. The examples are the built-in definitions as-is
/// (`/jany-update` also merges from them into a built-in the user has edited).
const SKILL_EXAMPLES: &[(&str, &str)] = &[
    ("jany-register/examples/find/schema.toml", include_str!("../examples/find/schema.toml")),
    ("jany-register/examples/find/assemble.sh", include_str!("../examples/find/assemble.sh")),
    ("jany-register/examples/find/cases.toml", include_str!("../examples/find/cases.toml")),
    ("jany-register/examples/curl/schema.toml", include_str!("../examples/curl/schema.toml")),
    ("jany-register/examples/curl/assemble.sh", include_str!("../examples/curl/assemble.sh")),
    ("jany-register/examples/curl/cases.toml", include_str!("../examples/curl/cases.toml")),
    ("jany-register/examples/docker/run/schema.toml", include_str!("../examples/docker/run/schema.toml")),
    ("jany-register/examples/docker/run/assemble.sh", include_str!("../examples/docker/run/assemble.sh")),
    ("jany-register/examples/docker/run/cases.toml", include_str!("../examples/docker/run/cases.toml")),
];

/// The files to ship (the skill + examples).
fn files(locale: Locale) -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    let body = match locale {
        Locale::En => SKILL_EN,
        Locale::Ja => SKILL_JA,
    };
    body.iter().chain(SKILL_EXAMPLES)
}

/// Built-in command definitions. `jany --init` places the ones not yet in `~/.config/jany/cmd/<name>/`.
/// Their contents are the originals in examples/.
const COMMANDS: &[(&str, &[(&str, &str)])] = &[
    ("find", &[
        ("schema.toml", include_str!("../examples/find/schema.toml")),
        ("assemble.sh", include_str!("../examples/find/assemble.sh")),
        ("cases.toml", include_str!("../examples/find/cases.toml")),
    ]),
    ("curl", &[
        ("schema.toml", include_str!("../examples/curl/schema.toml")),
        ("assemble.sh", include_str!("../examples/curl/assemble.sh")),
        ("cases.toml", include_str!("../examples/curl/cases.toml")),
    ]),
    ("docker/run", &[
        ("schema.toml", include_str!("../examples/docker/run/schema.toml")),
        ("assemble.sh", include_str!("../examples/docker/run/assemble.sh")),
        ("cases.toml", include_str!("../examples/docker/run/cases.toml")),
    ]),
];

/// Places only missing definitions. A directory that already has a schema.toml is left alone (even if outdated;
/// `jany --update` replaces it when the user has not edited it).
/// Returns the directories placed.
pub fn install_commands(cmd_dir: &Path) -> Result<Vec<String>, JanyError> {
    let mut placed = Vec::new();
    for (name, files) in COMMANDS {
        let dir = cmd_dir.join(name);
        if dir.join("schema.toml").exists() {
            continue;
        }
        install_files(&dir, files)?;
        placed.push(dir.display().to_string());
    }
    Ok(placed)
}

/// The names of the shipped skills, each a directory under the skills root.
const SKILLS: &[&str] = &["jany-register", "jany-update", "jany-setup"];

/// `JANY_SKILL_DIR`, or else `~/.agents/skills/jany-register`.
pub fn skill_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("JANY_SKILL_DIR") {
        return Some(PathBuf::from(d));
    }
    let home = std::env::var_os("HOME")?;
    Some(Path::new(&home).join(".agents").join("skills").join("jany-register"))
}

/// Rewrites only files whose contents differ (switching the language rewrites them). Returns the paths written and links made.
/// The other skills go next to jany-register (`JANY_SKILL_DIR=/x/jany-register` puts jany-update in `/x/jany-update`).
pub fn install(locale: Locale) -> Result<Vec<String>, JanyError> {
    install_skills(locale, false)
}

/// Places only the skill files that are not there yet (e.g. /jany-update for someone who set jany up before it
/// existed), leaving the rest as they are. Without `--locale`, follows the language of the installed jany-register.
pub fn install_missing(locale: Option<Locale>) -> Result<Vec<String>, JanyError> {
    let locale = locale.unwrap_or_else(|| {
        let installed = skill_dir().and_then(|d| std::fs::read_to_string(d.join("SKILL.md")).ok()).unwrap_or_default();
        if installed.chars().any(|c| ('\u{3040}'..='\u{30ff}').contains(&c)) { Locale::Ja } else { Locale::En }
    });
    install_skills(locale, true)
}

fn install_skills(locale: Locale, only_missing: bool) -> Result<Vec<String>, JanyError> {
    let dir = skill_dir().ok_or_else(|| JanyError::Config("cannot determine skill dir (HOME unset)".into()))?;
    let root = dir.parent().map(Path::to_path_buf).unwrap_or_default();
    let skill_path = |name: &str| if name == "jany-register" { dir.clone() } else { root.join(name) };
    let mut changed = Vec::new();
    for (rel, body) in files(locale) {
        let (name, rest) = rel.split_once('/').expect("skill file paths start with the skill name");
        let p = skill_path(name).join(rest);
        if (only_missing && p.exists()) || std::fs::read_to_string(&p).map(|cur| cur == *body).unwrap_or(false) {
            continue;
        }
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&p, body)?;
        set_executable(&p, rel.ends_with(".sh"))?;
        changed.push(p.display().to_string());
    }
    // Link from Claude Code's / Codex's own directories in case they only look there,
    // but only when the directory exists and has nothing by that name yet.
    if let Some(home) = std::env::var_os("HOME") {
        for tool in [".claude", ".codex"] {
            let skills = Path::new(&home).join(tool).join("skills");
            for name in SKILLS {
                let (target, link) = (skill_path(name), skills.join(name));
                if skills.is_dir() && std::fs::symlink_metadata(&link).is_err() {
                    #[cfg(unix)]
                    if std::os::unix::fs::symlink(&target, &link).is_ok() {
                        changed.push(format!("{} -> {}", link.display(), target.display()));
                    }
                }
            }
        }
    }
    Ok(changed)
}

/// Hashes of every shipped version of the built-in files (`examples/released.txt`).
const RELEASED: &str = include_str!("../examples/released.txt");

/// FNV-1a 64. Stable across Rust versions, unlike `DefaultHasher`, so the hashes can be kept in a file.
fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3))
}

/// Whether `body` is a version of `<command>/<file>` that jany has shipped.
fn released(command: &str, file: &str, body: &[u8]) -> bool {
    let key = format!("{command}/{file}");
    let hash = format!("{:016x}", fnv64(body));
    RELEASED.lines().filter_map(|l| l.split_once(' ')).any(|(k, h)| k == key && h.trim() == hash)
}

/// What a definition written for an older jany lacks. `/jany-update` adds these.
pub fn missing_features(schema: &crate::schema::Schema) -> Vec<&'static str> {
    let mut m = Vec::new();
    if schema.placeholders.is_empty() {
        m.push("[[placeholders]] (the dim hint after `jany <command> ` in zsh)");
    }
    m
}

/// `jany --update [name …]`: brings the built-in definitions up to date and tells what the others lack.
/// A built-in is replaced only when every one of its files is a shipped version (the user has not edited it);
/// an edited one is left alone and pointed at `/jany-update`. `defs` are all definitions found (`jany --list`).
pub fn update(cmd_dir: &Path, names: &[String], defs: &[String]) -> Result<i32, JanyError> {
    let wanted = names.join(" ");
    if !wanted.is_empty() && !defs.contains(&wanted) && !COMMANDS.iter().any(|(c, _)| c.replace('/', " ") == wanted) {
        return Err(JanyError::Usage(format!("no command definition `{wanted}` (see jany --list)")));
    }
    let mut todo = Vec::new();
    for (command, files) in COMMANDS {
        let name = command.replace('/', " ");
        if !wanted.is_empty() && name != wanted {
            continue;
        }
        let dir = cmd_dir.join(command);
        if !dir.join("schema.toml").exists() {
            install_files(&dir, files)?;
            eprintln!("{name}: installed {}", dir.display());
            continue;
        }
        let current: Vec<(&str, Option<Vec<u8>>)> = files.iter().map(|(rel, _)| (*rel, std::fs::read(dir.join(rel)).ok())).collect();
        if files.iter().zip(&current).all(|((_, body), (_, cur))| cur.as_deref() == Some(body.as_bytes())) {
            eprintln!("{name}: up to date");
            continue;
        }
        let edited: Vec<&str> = current
            .iter()
            .filter(|(rel, cur)| cur.as_ref().is_some_and(|b| !released(command, rel, b)))
            .map(|(rel, _)| *rel)
            .collect();
        if edited.is_empty() {
            install_files(&dir, files)?;
            eprintln!("{name}: updated {}", dir.display());
        } else {
            eprintln!("{name}: you have edited {}; left alone", edited.join(", "));
            todo.push(name);
        }
    }
    // The user's own definitions: jany cannot rewrite them, only say what is new.
    for name in defs {
        if COMMANDS.iter().any(|(c, _)| c.replace('/', " ") == *name) || (!wanted.is_empty() && *name != wanted) {
            continue;
        }
        let Ok(schema) = crate::schema::Schema::load(&cmd_dir.join(name.replace(' ', "/"))) else {
            eprintln!("{name}: does not load (jany --test {name})");
            continue;
        };
        let missing = missing_features(&schema);
        if missing.is_empty() {
            eprintln!("{name}: up to date");
        } else {
            eprintln!("{name}: lacks {}", missing.join(", "));
            todo.push(name.clone());
        }
    }
    for name in &todo {
        eprintln!("  → in Claude Code or Codex: /jany-update {name}");
    }
    Ok(0)
}

fn install_files(dir: &Path, files: &[(&str, &str)]) -> Result<(), JanyError> {
    std::fs::create_dir_all(dir)?;
    for (rel, body) in files {
        let p = dir.join(rel);
        std::fs::write(&p, body)?;
        set_executable(&p, rel.ends_with(".sh"))?;
    }
    Ok(())
}

fn set_executable(p: &Path, on: bool) -> std::io::Result<()> {
    #[cfg(unix)]
    if on {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(p)?.permissions();
        perm.set_mode(perm.mode() | 0o111);
        std::fs::set_permissions(p, perm)?;
    }
    Ok(())
}

/// `jany --register <name> [sub…]`: places the three scaffold files. Leaves existing ones alone.
pub fn register(cmd_dir: &Path, names: &[String], locale: Locale) -> Result<i32, JanyError> {
    if names.is_empty() {
        return Err(JanyError::Usage("jany --register <name> [sub …]  e.g. jany --register docker run".into()));
    }
    for n in names {
        if n.is_empty() || n.contains('/') || n.starts_with('.') || n.starts_with('-') {
            return Err(JanyError::Usage(format!("bad command name `{n}`: use plain words (docker run)")));
        }
    }
    let name = names.join(" ");
    // The head of argv is split per word ("docker run" → "docker", "run").
    let argv0 = names.iter().map(|n| format!("{n:?}")).collect::<Vec<_>>().join(", ");
    let dir = names.iter().fold(cmd_dir.to_path_buf(), |d, n| d.join(n));
    if dir.join("schema.toml").exists() {
        return Err(JanyError::Usage(format!("{} already exists; edit it or remove it first", dir.join("schema.toml").display())));
    }
    std::fs::create_dir_all(&dir)?;
    for rel in ["schema.toml", "assemble.sh", "cases.toml"] {
        let body = files(locale).find(|(r, _)| *r == format!("jany-register/template/{rel}")).map(|(_, b)| *b).expect("template embedded");
        let p = dir.join(rel);
        std::fs::write(&p, body.replace("__ARGV0__", &argv0).replace("__NAME__", &name))?;
        set_executable(&p, rel.ends_with(".sh"))?;
        eprintln!("wrote {}", p.display());
    }
    let skill = skill_dir().map(|d| d.display().to_string()).unwrap_or_else(|| "~/.agents/skills/jany-register".into());
    eprintln!(
        "\nnext: fill them in with the skill (Claude Code / Codex): /jany-register {name}\n      reference: {skill}/reference.md\n      then run:  jany --test {name}"
    );
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every built-in shipped now must be in examples/released.txt, or `jany --update` would take it for an edited one.
    #[test]
    fn current_builtins_are_listed_as_released() {
        let missing: Vec<String> = COMMANDS
            .iter()
            .flat_map(|(c, files)| files.iter().map(move |(rel, body)| (c, rel, body)))
            .filter(|(c, rel, body)| !released(c, rel, body.as_bytes()))
            .map(|(c, rel, body)| format!("{c}/{rel} {:016x}", fnv64(body.as_bytes())))
            .collect();
        assert!(missing.is_empty(), "append to examples/released.txt:\n{}", missing.join("\n"));
    }

    #[test]
    fn fnv64_matches_the_reference_values() {
        assert_eq!(fnv64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv64(b"a"), 0xaf63_dc4c_8601_ec8c);
    }
}
