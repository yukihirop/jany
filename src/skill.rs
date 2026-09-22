//! `/jx-register` スキルの配布と `jx register <name>` の雛形。
//! スキル本体は `skill/jx-register/` をバイナリに埋め込み、`jx init` のたびに
//! `~/.agents/skills/jx-register/` へ書く(Claude Code / Codex のどちらからも読める場所)。

use crate::error::JxError;
use std::path::{Path, PathBuf};

/// 配布するファイル。examples は design/ の find・curl そのもの。
const FILES: &[(&str, &str)] = &[
    ("SKILL.md", include_str!("../skill/jx-register/SKILL.md")),
    ("reference.md", include_str!("../skill/jx-register/reference.md")),
    ("template/schema.toml", include_str!("../skill/jx-register/template/schema.toml")),
    ("template/assemble.sh", include_str!("../skill/jx-register/template/assemble.sh")),
    ("template/cases.toml", include_str!("../skill/jx-register/template/cases.toml")),
    ("examples/find/schema.toml", include_str!("../design/find/schema.toml")),
    ("examples/find/assemble.sh", include_str!("../design/find/assemble.sh")),
    ("examples/find/cases.toml", include_str!("../design/find/cases.toml")),
    ("examples/curl/schema.toml", include_str!("../design/curl/schema.toml")),
    ("examples/curl/assemble.sh", include_str!("../design/curl/assemble.sh")),
    ("examples/curl/cases.toml", include_str!("../design/curl/cases.toml")),
];

/// 組み込みのコマンド定義。`jx init` が `~/.config/jx/cmd/<name>/` にまだ無いものだけ置く。
/// 中身は design/ の原本そのもの(examples と同じ)。
const COMMANDS: &[(&str, &[(&str, &str)])] = &[
    ("find", &[
        ("schema.toml", include_str!("../design/find/schema.toml")),
        ("assemble.sh", include_str!("../design/find/assemble.sh")),
        ("cases.toml", include_str!("../design/find/cases.toml")),
    ]),
    ("curl", &[
        ("schema.toml", include_str!("../design/curl/schema.toml")),
        ("assemble.sh", include_str!("../design/curl/assemble.sh")),
        ("cases.toml", include_str!("../design/curl/cases.toml")),
    ]),
    ("docker/run", &[
        ("schema.toml", include_str!("../design/docker/run/schema.toml")),
        ("assemble.sh", include_str!("../design/docker/run/assemble.sh")),
        ("cases.toml", include_str!("../design/docker/run/cases.toml")),
    ]),
];

/// まだ無い定義だけ置く。既に schema.toml があるディレクトリは(古くても)触らない。
/// 戻り値は置いたディレクトリ。
pub fn install_commands(cmd_dir: &Path) -> Result<Vec<String>, JxError> {
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

/// `JX_SKILL_DIR`、無ければ `~/.agents/skills/jx-register`。
pub fn skill_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("JX_SKILL_DIR") {
        return Some(PathBuf::from(d));
    }
    let home = std::env::var_os("HOME")?;
    Some(Path::new(&home).join(".agents").join("skills").join("jx-register"))
}

/// 中身が違うファイルだけ書き直す。戻り値は書いたパスと作ったリンク。
pub fn install() -> Result<Vec<String>, JxError> {
    let dir = skill_dir().ok_or_else(|| JxError::Config("cannot determine skill dir (HOME unset)".into()))?;
    let mut changed = Vec::new();
    for (rel, body) in FILES {
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
    // Claude Code / Codex がそれぞれのディレクトリしか見ない場合に備えてリンクを置く。
    // ディレクトリが既にあって、その名前がまだ無いときだけ。
    if let Some(home) = std::env::var_os("HOME") {
        for tool in [".claude", ".codex"] {
            let skills = Path::new(&home).join(tool).join("skills");
            let link = skills.join("jx-register");
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

/// `jx register <name> [sub…]`: 雛形 3 ファイルを置く。既にあれば触らない。
pub fn register(cmd_dir: &Path, names: &[String]) -> Result<i32, JxError> {
    if names.is_empty() {
        return Err(JxError::Usage("jx register <name> [sub …]  e.g. jx register docker run".into()));
    }
    for n in names {
        if n.is_empty() || n.contains('/') || n.starts_with('.') || n.starts_with('-') {
            return Err(JxError::Usage(format!("bad command name `{n}`: use plain words (docker run)")));
        }
    }
    let name = names.join(" ");
    // argv の先頭は語ごとに分ける("docker run" → "docker", "run")。
    let argv0 = names.iter().map(|n| format!("{n:?}")).collect::<Vec<_>>().join(", ");
    let dir = names.iter().fold(cmd_dir.to_path_buf(), |d, n| d.join(n));
    if dir.join("schema.toml").exists() {
        return Err(JxError::Usage(format!("{} already exists; edit it or remove it first", dir.join("schema.toml").display())));
    }
    std::fs::create_dir_all(&dir)?;
    for rel in ["schema.toml", "assemble.sh", "cases.toml"] {
        let body = FILES.iter().find(|(r, _)| *r == format!("template/{rel}")).map(|(_, b)| *b).expect("template embedded");
        let p = dir.join(rel);
        std::fs::write(&p, body.replace("__ARGV0__", &argv0).replace("__NAME__", &name))?;
        set_executable(&p, rel.ends_with(".sh"))?;
        eprintln!("wrote {}", p.display());
    }
    let skill = skill_dir().map(|d| d.display().to_string()).unwrap_or_else(|| "~/.agents/skills/jx-register".into());
    eprintln!(
        "\nnext: fill them in with the skill (Claude Code / Codex): /jx-register {name}\n      reference: {skill}/reference.md\n      then run:  jx test {name}"
    );
    Ok(0)
}
