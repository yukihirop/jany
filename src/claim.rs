//! `jany --claim -- <typed words…>`: after `jany --on`, the zsh wrapper asks this on Enter for every line.
//! Exit 0 means jany takes the line (the wrapper puts `jany ` in front of it), 1 means the shell runs it as typed.
//! It only looks at the words and the definition directories: no config, no jev, so it is cheap on every Enter.

use crate::schema;
use std::path::Path;

pub fn run(cmd_dir: &Path, typed: &[String]) -> i32 {
    if takes(cmd_dir, typed) { 0 } else { 1 }
}

/// jany takes a line when it starts with a command it has a definition for (`find`, `docker run`) and says
/// something after it, and nothing in it looks like the command's own syntax or the shell's:
/// a word starting with `-` (`find . -name x`), or a pipe, list or redirection (`find old logs > out`).
fn takes(cmd_dir: &Path, typed: &[String]) -> bool {
    if typed.iter().any(|w| w.starts_with('-') || is_operator(w)) {
        return false;
    }
    match schema::resolve(cmd_dir, typed) {
        // `find` or `docker run` alone: nothing for jany to interpret, the command itself knows what to do.
        Ok((_, used)) => used < typed.len(),
        Err(_) => false,
    }
}

/// `|`, `&&`, `;`, `>`, `2>`, `<(…)` as zsh's `${(z)…}` splits them out of a line.
fn is_operator(w: &str) -> bool {
    if w.starts_with("<(") || w.starts_with(">(") || w.starts_with("=(") {
        return true;
    }
    let rest = w.trim_start_matches(|c: char| c.is_ascii_digit());
    !rest.is_empty() && rest.chars().all(|c| "|&;<>()".contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(line: &str) -> Vec<String> {
        line.split_whitespace().map(String::from).collect()
    }

    fn examples() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("examples")
    }

    #[test]
    fn takes_words_for_a_known_command() {
        for line in ["find log files older than 7 days", "find /var/log big files", "docker run nginx on port 8080", "curl https://example.com"] {
            assert!(takes(&examples(), &words(line)), "{line}");
        }
    }

    #[test]
    fn leaves_the_rest_to_the_shell() {
        for line in [
            "",
            "ls -la",
            "git status",
            // the command alone, or its own flags
            "find",
            "docker run",
            "find . -name x",
            "curl -s https://example.com",
            // docker has no definition of its own, only `docker run`
            "docker ps",
            // pipes, lists and redirections belong to the shell
            "find old logs | wc -l",
            "find old logs && echo done",
            "find old logs ; ls",
            "find old logs > out.txt",
            "find old logs 2> err.txt",
            "find old logs &",
            "diff <(find a) b",
        ] {
            assert!(!takes(&examples(), &words(line)), "{line}");
        }
    }

    #[test]
    fn operators() {
        for w in ["|", "||", "&&", ";", ">", ">>", "2>", "&>", "|&", "<(ls)"] {
            assert!(is_operator(w), "{w}");
        }
        for w in ["7", "a|b", "logs", "*.log", "(x"] {
            assert!(!is_operator(w), "{w}");
        }
    }
}
