//! Calls the assemble script: role-tagged tokens on its stdin, argv back from its stdout.
//! The host does not know what the command means. Only the minimum confidence is taken here.

use crate::error::JanyError;
use crate::jev::Answers;
use crate::schema::Schema;
use crate::token::Token;
use serde::Deserialize;
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Deserialize)]
pub struct Assembled {
    #[serde(default)]
    pub argv: Option<Vec<String>>,
    #[serde(default)]
    pub preview: Option<Vec<String>>,
    /// "none" | "unsafe" | "dangerous"
    #[serde(default = "default_risk")]
    pub risk: String,
    #[serde(default)]
    pub pipe: Option<Vec<String>>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(skip)]
    pub confidence: f32,
}

fn default_risk() -> String {
    "none".into()
}

/// Checks that every word has a role, then calls the script.
pub fn assemble(schema: &Schema, tokens: &[Token], passthrough: &[String], answers: &Answers, defaults: &toml::Table) -> Result<Assembled, JanyError> {
    let unresolved: Vec<&str> = tokens.iter().filter(|t| !t.resolved()).map(|t| t.text.as_str()).collect();
    if !unresolved.is_empty() {
        return Err(JanyError::Unresolved(unresolved.join(", ")));
    }
    let confidence = tokens.iter().map(|t| t.confidence).fold(1.0f32, f32::min);

    let toks: Vec<Value> = tokens
        .iter()
        .map(|t| {
            json!({
                "text": t.text, "role": t.role, "value": t.value(), "tags": t.tags,
                "amount": t.amount, "confidence": t.confidence, "source": t.source, "note": t.note,
            })
        })
        .collect();
    let ans: serde_json::Map<String, Value> = answers
        .iter()
        .filter(|(k, _)| !k.contains('.'))
        .map(|(k, a)| (k.clone(), a.noul().map(|p| json!(p)).unwrap_or_else(|| json!(a.choice()))))
        .collect();
    let input = json!({
        "tokens": toks,
        "passthrough": passthrough,
        "answers": ans,
        "defaults": toml_to_json(&toml::Value::Table(defaults.clone())),
    });

    let script = schema.dir.join(&schema.assemble.script);
    let mut child = Command::new(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| JanyError::Assemble(format!("{}: {e}", script.display())))?;
    child.stdin.take().unwrap().write_all(input.to_string().as_bytes())?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err(JanyError::Assemble(format!("{} exited with {}", script.display(), out.status)));
    }
    let mut a: Assembled = serde_json::from_slice(&out.stdout).map_err(|e| JanyError::Assemble(format!("{}: bad output: {e}", script.display())))?;
    if let Some(e) = a.error.take() {
        return Err(JanyError::Assemble(e));
    }
    if a.argv.is_none() {
        return Err(JanyError::Assemble(format!("{}: neither argv nor error", script.display())));
    }
    a.confidence = confidence;
    Ok(a)
}

/// Layers the user config's [cmd.<name>.defaults] over the schema's [defaults].
pub fn merge_defaults(schema: &toml::Table, user: &toml::Table) -> toml::Table {
    let mut d = schema.clone();
    for (k, v) in user {
        d.insert(k.clone(), v.clone());
    }
    d
}

pub fn toml_to_json(v: &toml::Value) -> Value {
    match v {
        toml::Value::String(s) => json!(s),
        toml::Value::Integer(i) => json!(i),
        toml::Value::Float(f) => json!(f),
        toml::Value::Boolean(b) => json!(b),
        toml::Value::Datetime(d) => json!(d.to_string()),
        toml::Value::Array(a) => Value::Array(a.iter().map(toml_to_json).collect()),
        toml::Value::Table(t) => Value::Object(t.iter().map(|(k, v)| (k.clone(), toml_to_json(v))).collect()),
    }
}
