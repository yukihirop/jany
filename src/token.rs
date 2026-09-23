//! What is attached to one word. Roles are the schema's strings (jind/jurl's enums turned into data).

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Rule,
    Jev,
}

/// An amount. `+7d` is fully decided by rules; for `7`, jev answers the unit and the direction.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Amount {
    pub n: f64,
    pub unit: Option<String>,
    /// Some(true) = at least, Some(false) = at most, None = undecided.
    pub at_least: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Token {
    pub text: String,
    pub role: Option<String>,
    pub confidence: f32,
    pub source: Source,
    /// The corrected value (`files` → `f`, `psot` → `POST`). text if none.
    pub fixed: Option<String>,
    pub amount: Option<Amount>,
    pub tags: Vec<String>,
    /// A short note shown by `--explain`.
    pub note: Option<String>,
    /// jev's probability per role (none for words decided by rules). Used as evidence by repair.
    pub probs: Option<BTreeMap<String, f32>>,
}

impl Token {
    pub fn new(text: impl Into<String>) -> Self {
        Token { text: text.into(), role: None, confidence: 0.0, source: Source::Rule, fixed: None, amount: None, tags: Vec::new(), note: None, probs: None }
    }

    pub fn resolved(&self) -> bool {
        self.role.is_some()
    }

    pub fn is(&self, role: &str) -> bool {
        self.role.as_deref() == Some(role)
    }

    pub fn set_rule(&mut self, role: &str) {
        self.role = Some(role.to_string());
        self.confidence = 1.0;
        self.source = Source::Rule;
    }

    pub fn value(&self) -> &str {
        self.fixed.as_deref().unwrap_or(&self.text)
    }

    pub fn has_tag(&self, t: &str) -> bool {
        self.tags.iter().any(|x| x == t)
    }

    pub fn add_tag(&mut self, t: &str) {
        if !self.has_tag(t) {
            self.tags.push(t.to_string());
        }
    }

    /// Prepends a note (`old → new (why); previous note`).
    pub fn prepend_note(&mut self, s: String) {
        self.note = Some(match self.note.take() {
            Some(n) if !n.is_empty() => format!("{s}; {n}"),
            _ => s,
        });
    }

    pub fn role_str(&self) -> &str {
        self.role.as_deref().unwrap_or("?")
    }
}
