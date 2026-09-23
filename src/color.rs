//! ANSI colours. Only when the output is a terminal and NO_COLOR is not set.

use std::io::IsTerminal;

#[derive(Clone, Copy)]
pub enum C {
    Dim,
    Red,
    Green,
    Yellow,
    Blue,
    Cyan,
}

impl C {
    fn code(self) -> &'static str {
        match self {
            C::Dim => "2",
            C::Red => "31",
            C::Green => "32",
            C::Yellow => "33",
            C::Blue => "34",
            C::Cyan => "36",
        }
    }
}

pub fn stderr_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stderr().is_terminal()
}

pub fn stdout_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
}

pub fn paint(on: bool, c: C, s: &str) -> String {
    if on {
        format!("\x1b[{}m{s}\x1b[0m", c.code())
    } else {
        s.to_string()
    }
}

/// Colour by confidence band: ≥0.8 green, ≥0.5 yellow, below that red.
pub fn conf(on: bool, v: f32) -> String {
    let s = format!("{v:.2}");
    let c = if v >= 0.8 { C::Green } else if v >= 0.5 { C::Yellow } else { C::Red };
    paint(on, c, &s)
}
