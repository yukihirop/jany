//! env + ~/.config/jany/config.toml. Everything is optional. The API key is also borrowed from jind / jurl's config.
//!
//! ```toml
//! [jev]
//! api_key = "..."          # written by `jany --setup`
//! model = "typesafe/jev-1.13"
//! reject_below = 0.5       # interpretations below this are not printed to stdout
//!
//! [suggest]
//! enabled = false          # no dim hint in zsh (JANY_SUGGEST=0/1 overrides it per shell)
//!
//! [cmd.curl.defaults]      # overrides the schema's [defaults]
//! content_type = "text/plain"
//! ```

use crate::error::JanyError;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub jev: Jev,
    pub suggest: Suggest,
    /// Command name → settings.
    pub cmd: BTreeMap<String, CmdConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Jev {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub model: String,
    pub reject_below: f32,
    pub timeout_ms: u64,
}

/// The dim hint after `jany <command> ` in zsh (`jany --suggest`).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Suggest {
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CmdConfig {
    pub defaults: toml::Table,
    pub aliases: BTreeMap<String, String>,
}

impl Default for Jev {
    fn default() -> Self {
        Jev { enabled: true, api_key: None, model: crate::jev::client::DEFAULT_MODEL.into(), reject_below: 0.5, timeout_ms: 5000 }
    }
}

impl Default for Suggest {
    fn default() -> Self {
        Suggest { enabled: true }
    }
}

pub fn config_dir() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("JANY_CONFIG_DIR") {
        return Some(PathBuf::from(p));
    }
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config").join("jany"))
}

pub fn path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("config.toml"))
}

/// Where command definitions live. `JANY_CMD_DIR` if set (points at `examples/` during development).
pub fn cmd_dir() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("JANY_CMD_DIR") {
        return Some(PathBuf::from(p));
    }
    config_dir().map(|d| d.join("cmd"))
}

pub fn load() -> Result<Config, JanyError> {
    let mut cfg = match path() {
        Some(p) if p.exists() => {
            let text = std::fs::read_to_string(&p)?;
            toml::from_str::<Config>(&text).map_err(|e| JanyError::Config(format!("{}: {e}", p.display())))?
        }
        _ => Config::default(),
    };
    if let Ok(m) = std::env::var("JEV_MODEL") {
        cfg.jev.model = m;
    }
    if let Ok(v) = std::env::var("JANY_SUGGEST") {
        cfg.suggest.enabled = v != "0";
    }
    if cfg.jev.api_key.is_none() {
        cfg.jev.api_key = borrowed_api_key();
    }
    Ok(cfg)
}

/// [jev] api_key from ~/.config/{jurl,jind}/config.toml (the same OpenRouter key).
fn borrowed_api_key() -> Option<String> {
    let home = std::env::var_os("HOME")?;
    for tool in ["jurl", "jind"] {
        let p = PathBuf::from(&home).join(".config").join(tool).join("config.toml");
        let Ok(text) = std::fs::read_to_string(p) else { continue };
        let Ok(t) = toml::from_str::<toml::Table>(&text) else { continue };
        if let Some(k) = t.get("jev").and_then(|j| j.get("api_key")).and_then(|k| k.as_str()).filter(|k| !k.is_empty()) {
            return Some(k.to_string());
        }
    }
    None
}

/// Expand aliases, replacing `$VAR` in their values with environment variables.
pub fn expand_aliases(aliases: &BTreeMap<String, String>, words: &[String]) -> Vec<String> {
    words
        .iter()
        .map(|w| match aliases.get(w) {
            Some(v) => expand_env(v),
            None => w.clone(),
        })
        .collect()
}

fn expand_env(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            let mut name = String::new();
            while let Some(&n) = chars.peek() {
                if n.is_ascii_alphanumeric() || n == '_' {
                    name.push(n);
                    chars.next();
                } else {
                    break;
                }
            }
            if name.is_empty() {
                out.push('$');
            } else {
                out.push_str(&std::env::var(&name).unwrap_or_default());
            }
        } else {
            out.push(c);
        }
    }
    out
}
