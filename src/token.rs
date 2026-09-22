//! 語 1 つに付く情報。役割は schema の文字列(jind/jurl の enum をデータにしたもの)。

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Rule,
    Jev,
}

/// 量。`+7d` は規則で全部決まる。`7` は jev が unit と向きを答える。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Amount {
    pub n: f64,
    pub unit: Option<String>,
    /// Some(true) = 以上、Some(false) = 以下、None = 未定。
    pub at_least: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Token {
    pub text: String,
    pub role: Option<String>,
    pub confidence: f32,
    pub source: Source,
    /// 補正後の値(`files` → `f`、`psot` → `POST`)。無ければ text。
    pub fixed: Option<String>,
    pub amount: Option<Amount>,
    pub tags: Vec<String>,
    /// `--explain` に出す一言。
    pub note: Option<String>,
    /// jev が返した役割ごとの確率(規則で決めたものは無し)。repair の根拠に使う。
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

    /// note の先頭に一言足す(`old → new (why); 以前の note`)。
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
