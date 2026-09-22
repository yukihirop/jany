//! env + ~/.config/jany/config.toml。全部省略可。API キーは jind / jurl の設定からも借りる。
//!
//! ```toml
//! [jev]
//! api_key = "..."          # `jany setup` が書く
//! model = "typesafe/jev-1.13"
//! reject_below = 0.5       # これ未満の解釈は stdout に出さない
//!
//! [cmd.curl.defaults]      # schema の [defaults] を上書き
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
    /// コマンド名 → 設定。
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

/// コマンド定義の置き場。`JANY_CMD_DIR` があればそこ(開発中は `design/` を指す)。
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
    if cfg.jev.api_key.is_none() {
        cfg.jev.api_key = borrowed_api_key();
    }
    Ok(cfg)
}

/// ~/.config/{jurl,jind}/config.toml の [jev] api_key(同じ OpenRouter の鍵)。
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

/// aliases を展開し、値の中の `$VAR` を環境変数で置き換える。
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
