//! `jany setup`: OpenRouter の API キーを ~/.config/jany/config.toml の [jev] api_key に 0600 で保存し、
//! jev に 1 回テスト呼び出しして疎通を確かめる。

use crate::error::JanyError;
use crate::jev::{self, Oracle};
use serde_json::json;
use std::io::{IsTerminal, Write};
use std::time::{Duration, Instant};

pub fn run() -> Result<i32, JanyError> {
    let path = crate::config::path().ok_or_else(|| JanyError::Config("cannot determine config path (HOME unset)".into()))?;
    let existing = if path.exists() { std::fs::read_to_string(&path)? } else { String::new() };
    let mut table: toml::Table = if existing.trim().is_empty() {
        toml::Table::new()
    } else {
        toml::from_str(&existing).map_err(|e| JanyError::Config(format!("{}: {e}", path.display())))?
    };

    let has_key = table
        .get("jev")
        .and_then(|j| j.get("api_key"))
        .and_then(|k| k.as_str())
        .map(|k| !k.is_empty())
        .unwrap_or(false);
    if has_key && !crate::output::confirm_no(&format!("{} already has an api_key. overwrite?", path.display())) {
        eprintln!("kept the existing key.");
        return Ok(0);
    }

    if !std::io::stdin().is_terminal() {
        return Err(JanyError::Usage("jany setup needs a terminal to read the key".into()));
    }
    let key = rpassword::prompt_password("OpenRouter API key (input hidden): ")?;
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(JanyError::Usage("empty key, nothing saved".into()));
    }

    // 疎通確認: 1 回だけ、最小の質問。
    let model = std::env::var("JEV_MODEL").unwrap_or_else(|_| jev::client::DEFAULT_MODEL.to_string());
    let oracle = jev::client::OpenRouter { api_key: key.clone(), model: model.clone(), timeout: Duration::from_secs(10), max_retries: 1 };
    let mut q = jev::Questions::new();
    q.insert("ok".into(), jev::noul("Is the word `ping` a greeting or a connectivity check?"));
    let t0 = Instant::now();
    eprint!("checking jev via OpenRouter ... ");
    let _ = std::io::stderr().flush();
    let res = oracle.decide(json!({"word": "ping"}), q)?;
    let cost = res.usage.as_ref().and_then(|u| u.cost).map(|c| format!("${c:.6}")).unwrap_or_else(|| "$?".into());
    eprintln!("ok ({} · {} ms · {cost})", res.model, t0.elapsed().as_millis());

    let jev_tbl = table.entry("jev").or_insert_with(|| toml::Value::Table(toml::Table::new()));
    let jev_tbl = jev_tbl.as_table_mut().ok_or_else(|| JanyError::Config("[jev] is not a table".into()))?;
    jev_tbl.insert("api_key".into(), toml::Value::String(key));

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = toml::to_string_pretty(&table).map_err(|e| JanyError::Config(e.to_string()))?;
    write_private(&path, &text)?;
    eprintln!("saved to {} (mode 0600). OPENROUTER_API_KEY in the environment still takes precedence.", path.display());
    Ok(0)
}

#[cfg(unix)]
fn write_private(path: &std::path::Path, text: &str) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path)?;
    f.write_all(text.as_bytes())?;
    // 既存ファイルだった場合も 0600 に揃える。
    std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o600))
}

#[cfg(not(unix))]
fn write_private(path: &std::path::Path, text: &str) -> std::io::Result<()> {
    std::fs::write(path, text)
}
