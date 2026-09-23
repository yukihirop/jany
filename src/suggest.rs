//! `jany --suggest -- <typed words…>`: print the `[[placeholders]]` of the command that no typed word fills yet,
//! e.g. `<type> <older than N days>` after `jany find src `. The zsh wrapper shows it dimmed after the cursor.
//! Rules and repair only: jev is never called, so it is fast and free on every keystroke.

use crate::jev::Answers;
use crate::{config, output, repair, rules, schema};
use std::path::Path;

pub fn run(cmd_dir: &Path, typed: &[String]) -> i32 {
    if let Some(line) = line(cmd_dir, typed) {
        output::stdout(&format!("{line}\n"));
    }
    0
}

fn line(cmd_dir: &Path, typed: &[String]) -> Option<String> {
    // Words after `--` are passed through as-is; there is nothing left to hint.
    if typed.iter().any(|w| w == "--") {
        return None;
    }
    // jany's own flags (--explain, --no-jev) are not words of the command.
    let words: Vec<String> = typed.iter().filter(|w| !w.starts_with("--")).cloned().collect();
    let (schema, used) = schema::resolve(cmd_dir, &words).ok()?;
    if schema.placeholders.is_empty() {
        return None;
    }
    let aliases = config::load().ok().and_then(|c| c.cmd.get(&schema.command.name).map(|c| c.aliases.clone())).unwrap_or_default();
    let rest = config::expand_aliases(&aliases, &words[used..]);
    let mut tokens = rules::classify(&schema, &rest);
    repair::repair(&schema, &mut tokens, &Answers::new());

    let slots = &schema.placeholders;
    let mut filled: Vec<bool> = slots.iter().map(|p| tokens.iter().any(|t| p.roles.iter().any(|r| t.is(r)))).collect();
    // Words left for jev: guess without asking. A number goes to the first open slot with an amount role
    // of the same dimension when its unit tells (`7` in `older than 7 days`), any other word to the first
    // open slot marked `bare` (`nginx` for `<image>`).
    let has_amount = |p: &schema::Placeholder, dim: Option<&str>| {
        p.roles.iter().any(|r| schema.role(r).and_then(|r| r.amount.as_deref()).is_some_and(|d| dim.is_none_or(|want| d == want)))
    };
    for t in tokens.iter().filter(|t| !t.resolved()) {
        let dim = t.amount.as_ref().and_then(|a| a.unit.as_deref()).and_then(|u| schema.unit_dimension(u));
        let open = |i: usize, p: &schema::Placeholder| !filled[i] && if t.amount.is_some() { has_amount(p, dim.as_deref()) } else { p.bare };
        if let Some(i) = slots.iter().enumerate().position(|(i, p)| open(i, p)) {
            filled[i] = true;
        }
    }
    let left: Vec<&str> = slots.iter().zip(&filled).filter(|(_, f)| !**f).map(|(p, _)| p.text.as_str()).collect();
    (!left.is_empty()).then(|| left.join(" "))
}
