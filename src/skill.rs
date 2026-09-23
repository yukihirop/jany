//! Ships the `/jany-register` skill and the `jany --register <name>` scaffold.
//! The skill (`skills/<locale>/jany-register/`, en / ja) is embedded in the binary and written on every `jany --init`
//! to `~/.agents/skills/jany-register/` (a place both Claude Code and Codex can read).

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

/// The skill for one language (`skills/<locale>/jany-register/`).
macro_rules! skill_files {
    ($l:literal) => {
        &[
            ("SKILL.md", include_str!(concat!("../skills/", $l, "/jany-register/SKILL.md"))),
            ("reference.md", include_str!(concat!("../skills/", $l, "/jany-register/reference.md"))),
            ("template/schema.toml", include_str!(concat!("../skills/", $l, "/jany-register/template/schema.toml"))),
            ("template/assemble.sh", include_str!(concat!("../skills/", $l, "/jany-register/template/assemble.sh"))),
            ("template/cases.toml", include_str!(concat!("../skills/", $l, "/jany-register/template/cases.toml"))),
        ]
    };
}
const SKILL_EN: &[(&str, &str)] = skill_files!("en");
const SKILL_JA: &[(&str, &str)] = skill_files!("ja");

/// The same in every language. The examples are examples/find and examples/curl as-is.
const SKILL_EXAMPLES: &[(&str, &str)] = &[
    ("examples/find/schema.toml", include_str!("../examples/find/schema.toml")),
    ("examples/find/assemble.sh", include_str!("../examples/find/assemble.sh")),
    ("examples/find/cases.toml", include_str!("../examples/find/cases.toml")),
    ("examples/curl/schema.toml", include_str!("../examples/curl/schema.toml")),
    ("examples/curl/assemble.sh", include_str!("../examples/curl/assemble.sh")),
    ("examples/curl/cases.toml", include_str!("../examples/curl/cases.toml")),
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

/// Places only missing definitions. A directory that already has a schema.toml is left alone (even if outdated).
/// Returns the directories placed.
pub fn install_commands(cmd_dir: &Path) -> Result<Vec<String>, JanyError> {
    let mut placed = Vec::new();
    for (name, files) in COMMANDS {
        let dir = cmd_dir.join(name);
        if dir.join("schema.toml").exists() {
            continue;
        }
        std::fs::create_dir_all(&dir)?;
        for (rel, body) in *files {
            let p = dir.join(rel);
            std::fs::write(&p, body)?;
            set_executable(&p, rel.ends_with(".sh"))?;
        }
        placed.push(dir.display().to_string());
    }
    Ok(placed)
}

/// `JANY_SKILL_DIR`, or else `~/.agents/skills/jany-register`.
pub fn skill_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("JANY_SKILL_DIR") {
        return Some(PathBuf::from(d));
    }
    let home = std::env::var_os("HOME")?;
    Some(Path::new(&home).join(".agents").join("skills").join("jany-register"))
}

/// Rewrites only files whose contents differ (switching the language rewrites them). Returns the paths written and links made.
pub fn install(locale: Locale) -> Result<Vec<String>, JanyError> {
    let dir = skill_dir().ok_or_else(|| JanyError::Config("cannot determine skill dir (HOME unset)".into()))?;
    let mut changed = Vec::new();
    for (rel, body) in files(locale) {
        let p = dir.join(rel);
        if std::fs::read_to_string(&p).map(|cur| cur == *body).unwrap_or(false) {
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
            let link = skills.join("jany-register");
            if skills.is_dir() && std::fs::symlink_metadata(&link).is_err() {
                #[cfg(unix)]
                if std::os::unix::fs::symlink(&dir, &link).is_ok() {
                    changed.push(format!("{} -> {}", link.display(), dir.display()));
                }
            }
        }
    }
    Ok(changed)
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
        let body = files(locale).find(|(r, _)| *r == format!("template/{rel}")).map(|(_, b)| *b).expect("template embedded");
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
