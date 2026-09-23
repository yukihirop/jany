//! The rules → (jev) → repair → assemble flow. Used by both main and `jany --test`.

use crate::assemble::{self, Assembled};
use crate::config::Config;
use crate::error::JanyError;
use crate::jev::{self, Answers, Oracle};
use crate::output::JevInfo;
use crate::questions;
use crate::repair;
use crate::rules;
use crate::schema::Schema;
use crate::token::Token;
use std::time::Instant;

pub struct Run {
    pub tokens: Vec<Token>,
    pub jev: Option<JevInfo>,
    pub out: Assembled,
}

/// With `oracle` = None, jev is never called (any unresolved word is Unresolved).
pub fn run(schema: &Schema, cfg: &Config, words: &[String], passthrough: &[String], oracle: Option<&dyn Oracle>, defaults_override: Option<&toml::Table>) -> Result<Run, JanyError> {
    let cmd_cfg = cfg.cmd.get(&schema.command.name).cloned().unwrap_or_default();
    let words = crate::config::expand_aliases(&cmd_cfg.aliases, words);
    let mut tokens = rules::classify(schema, &words);
    let mut info = None;
    let mut answers = Answers::new();

    if tokens.iter().any(|t| !t.resolved()) {
        let Some(oracle) = oracle else {
            let bad: Vec<&str> = tokens.iter().filter(|t| !t.resolved()).map(|t| t.text.as_str()).collect();
            return Err(JanyError::Unresolved(format!("{} (jev disabled)", bad.join(", "))));
        };
        let built = questions::build(schema, &tokens);
        let n = built.questions.len();
        let t0 = Instant::now();
        let res = oracle.decide(built.state, built.questions)?;
        info = Some(JevInfo { model: res.model.clone(), questions: n, ms: t0.elapsed().as_millis(), usage: res.usage.clone() });
        answers = res.answers;
        questions::apply(schema, &mut tokens, &answers);
    }
    repair::repair(schema, &mut tokens, &answers);

    let defaults = assemble::merge_defaults(&schema.defaults, defaults_override.unwrap_or(&cmd_cfg.defaults));
    let out = assemble::assemble(schema, &tokens, passthrough, &answers, &defaults)?;
    Ok(Run { tokens, jev: info, out })
}

pub fn oracle_from_config(cfg: &Config) -> Result<jev::client::OpenRouter, JanyError> {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .or_else(|| cfg.jev.api_key.clone())
        .ok_or_else(|| JanyError::Jev("no API key. run `jany --setup` or set OPENROUTER_API_KEY (needed to interpret ambiguous words)".into()))?;
    Ok(jev::client::OpenRouter {
        api_key,
        model: cfg.jev.model.clone(),
        timeout: std::time::Duration::from_millis(cfg.jev.timeout_ms),
        max_retries: 3,
    })
}

impl Run {
    /// Whether the line may run without a look on the prompt (`[cmd.<name>] autorun = true`):
    /// rules alone decided every word, the definition calls it risk "none", nothing needs a preview
    /// or a pipe, and nothing went through unread (words after `--`, or raw flags the rules only
    /// tagged with the `passthrough` role).
    pub fn autorun_safe(&self, passthrough: &[String]) -> bool {
        self.jev.is_none()
            && self.tokens.iter().all(|t| t.source == crate::token::Source::Rule && !t.is("passthrough"))
            && passthrough.is_empty()
            && self.out.risk == "none"
            && self.out.preview.is_none()
            && self.out.pipe.is_none()
            && self.out.argv.as_ref().is_some_and(|a| !a.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Source;

    fn run(roles: &[(&str, Source)], risk: &str) -> Run {
        let tokens = roles
            .iter()
            .map(|(r, s)| {
                let mut t = Token::new("w");
                t.set_rule(r);
                t.source = *s;
                t
            })
            .collect();
        let out = Assembled { argv: Some(vec!["pnpm".into(), "run".into(), "dev".into()]), preview: None, risk: risk.into(), pipe: None, error: None, confidence: 1.0 };
        Run { tokens, jev: None, out }
    }

    #[test]
    fn autorun_only_when_rules_decided_a_safe_line() {
        assert!(run(&[("action", Source::Rule), ("script", Source::Rule)], "none").autorun_safe(&[]));
        assert!(!run(&[("action", Source::Rule)], "unsafe").autorun_safe(&[]));
        assert!(!run(&[("action", Source::Rule)], "dangerous").autorun_safe(&[]));
        assert!(!run(&[("action", Source::Rule), ("package", Source::Jev)], "none").autorun_safe(&[]));
        assert!(!run(&[("action", Source::Rule), ("passthrough", Source::Rule)], "none").autorun_safe(&[]));
        assert!(!run(&[], "none").autorun_safe(&["--help".into()]));
        let mut piped = run(&[("action", Source::Rule)], "none");
        piped.out.pipe = Some(vec!["wc".into(), "-l".into()]);
        assert!(!piped.autorun_safe(&[]));
    }
}
