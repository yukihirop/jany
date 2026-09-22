//! stderr に出すもの: `--explain` の表、jev の 1 行、preview。stdout にはコマンドの 1 行だけ。

use crate::color::{self, C, paint};
use crate::jev::Usage;
use crate::token::{Source, Token};
use std::io::{IsTerminal, Write};

pub fn explain(tokens: &[Token], jev: Option<&JevInfo>) {
    eprint!("{}", explain_text(tokens, jev, color::stderr_enabled()));
}

pub fn explain_text(tokens: &[Token], jev: Option<&JevInfo>, on: bool) -> String {
    use std::fmt::Write as _;
    let mut e = String::new();
    let w = tokens.iter().map(|t| t.text.chars().count()).max().unwrap_or(4).clamp(4, 40);
    let rw = tokens.iter().map(|t| t.role_str().len()).max().unwrap_or(4).clamp(4, 20);
    let _ = writeln!(e, "{}", paint(on, C::Dim, &format!("{:<w$}  {:<rw$} {:<5} {:<4}  note", "word", "role", "conf", "by", w = w, rw = rw)));
    for t in tokens {
        let role_s = t.role_str();
        let role_c = if t.role.is_none() { C::Red } else if t.source == Source::Jev { C::Green } else { C::Cyan };
        let role = format!("{}{}", paint(on, role_c, role_s), " ".repeat(rw.saturating_sub(role_s.len())));
        let by = match t.source {
            Source::Rule => paint(on, C::Dim, "rule"),
            Source::Jev => paint(on, C::Blue, "jev "),
        };
        let mut note = t.note.clone().unwrap_or_default();
        if let Some(f) = &t.fixed
            && f != &t.text
        {
            note = format!("→ {f}{}{note}", if note.is_empty() { "" } else { "; " });
        }
        if let Some(a) = &t.amount
            && t.resolved()
        {
            let dir = match a.at_least {
                Some(true) => "≥",
                Some(false) => "≤",
                None => "?",
            };
            note = format!("{dir} {} {}{}{note}", a.n, a.unit.as_deref().unwrap_or("?"), if note.is_empty() { "" } else { "; " });
        }
        let pad = w.saturating_sub(t.text.chars().count());
        let _ = writeln!(e, "{}{}  {} {}  {}  {}", t.text, " ".repeat(pad), role, color::conf(on, t.confidence), by, paint(on, C::Dim, &note));
    }
    let _ = match jev {
        Some(j) => writeln!(e, "{}", paint(on, C::Dim, &j.line())),
        None => writeln!(e, "{}", paint(on, C::Dim, "jev: not called (fast path)")),
    };
    e
}

pub struct JevInfo {
    pub model: String,
    pub questions: usize,
    pub ms: u128,
    pub usage: Option<Usage>,
}

impl JevInfo {
    pub fn line(&self) -> String {
        let u = self.usage.clone().unwrap_or_default();
        format!(
            "jev: {} · {} questions · {} ms · {} in / {} out tokens · ${}",
            self.model,
            self.questions,
            self.ms,
            u.input_tokens,
            u.output_tokens,
            u.cost.map(|c| format!("{c:.6}")).unwrap_or_else(|| "?".into())
        )
    }
}

/// `jx setup` の上書き確認だけに使う y/N。
pub fn confirm_no(prompt: &str) -> bool {
    if !std::io::stdin().is_terminal() {
        return false;
    }
    eprint!("{} {} ", paint(color::stderr_enabled(), C::Yellow, prompt), paint(color::stderr_enabled(), C::Dim, "[y/N]"));
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line).is_err() {
        return false;
    }
    matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

/// argv (+ pipe) を POSIX shell 用の 1 行に。
pub fn render(argv: &[String], pipe: Option<&[String]>) -> String {
    let mut s = shell_words::join(argv);
    if let Some(p) = pipe
        && !p.is_empty()
    {
        s.push_str(" | ");
        s.push_str(&shell_words::join(p));
    }
    s
}
