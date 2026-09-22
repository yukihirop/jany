//! `jany --test <command>`: cases.toml を Mock Oracle で回す。
//! jev の答えは cases に書いたもの。聞かれていないキーに答えたら失敗(質問設計とフィクスチャのずれに気づくため)。

use crate::color::{self, C, paint};
use crate::config::Config;
use crate::error::JanyError;
use crate::interpret;
use crate::jev::{Answers, DecisionsResponse, Oracle, Questions};
use crate::questions::answer_from_toml;
use crate::schema::Schema;
use serde::Deserialize;
use serde_json::Value;
use std::cell::RefCell;

#[derive(Debug, Deserialize)]
struct Cases {
    #[serde(default, rename = "case")]
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    words: Vec<String>,
    #[serde(default)]
    passthrough: Vec<String>,
    #[serde(default)]
    argv: Option<Vec<String>>,
    #[serde(default)]
    preview: Option<Vec<String>>,
    #[serde(default)]
    risk: Option<String>,
    #[serde(default)]
    pipe: Option<Vec<String>>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    no_jev: bool,
    #[serde(default)]
    setup: Option<Setup>,
    #[serde(default)]
    jev: Option<toml::Table>,
    #[serde(default)]
    confidence: Option<f32>,
    #[serde(default)]
    defaults: Option<toml::Table>,
    #[serde(default)]
    state: Option<toml::Table>,
    #[serde(default)]
    not_asked: Vec<String>,
    #[serde(default)]
    tokens: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
struct Setup {
    #[serde(default)]
    dirs: Vec<String>,
}

struct Mock {
    answers: Option<toml::Table>,
    seen: RefCell<Option<(Value, Questions)>>,
}

impl Oracle for Mock {
    fn decide(&self, state: Value, questions: Questions) -> Result<DecisionsResponse, JanyError> {
        let Some(table) = &self.answers else {
            return Err(JanyError::Jev("jev was called but the case has no [case.jev] (expected rules only)".into()));
        };
        for k in table.keys() {
            if !questions.contains_key(k) {
                return Err(JanyError::Jev(format!("case answers `{k}` but jany did not ask it; asked: {:?}", questions.keys().collect::<Vec<_>>())));
            }
        }
        let mut answers = Answers::new();
        for (k, v) in table {
            let a = answer_from_toml(v).ok_or_else(|| JanyError::Jev(format!("bad answer for `{k}`: {v}")))?;
            answers.insert(k.clone(), a);
        }
        *self.seen.borrow_mut() = Some((state, questions));
        Ok(DecisionsResponse { model: "mock".into(), answers, usage: None })
    }
}

pub fn run(schema: &Schema, explain: bool) -> Result<i32, JanyError> {
    let p = schema.dir.join("cases.toml");
    let text = std::fs::read_to_string(&p).map_err(|e| JanyError::Usage(format!("{}: {e}", p.display())))?;
    let cases: Cases = toml::from_str(&text).map_err(|e| JanyError::Usage(format!("{}: {e}", p.display())))?;
    let on = color::stderr_enabled();
    let cfg = Config::default();
    let cwd = std::env::current_dir()?;
    let mut ok = 0;
    let mut bad = 0;

    for (n, c) in cases.cases.iter().enumerate() {
        let label = format!("#{} {}", n + 1, shell_words::join(&c.words));
        let tmp;
        if let Some(s) = &c.setup {
            tmp = tempfile::tempdir()?;
            for d in &s.dirs {
                std::fs::create_dir_all(tmp.path().join(d))?;
            }
            std::env::set_current_dir(tmp.path())?;
        }
        let failures = run_case(schema, &cfg, c, explain);
        std::env::set_current_dir(&cwd)?;
        if failures.is_empty() {
            ok += 1;
            eprintln!("{} {label}", paint(on, C::Green, "ok  "));
        } else {
            bad += 1;
            eprintln!("{} {label}", paint(on, C::Red, "FAIL"));
            for f in failures {
                eprintln!("     {f}");
            }
        }
    }
    eprintln!("\n{ok} ok, {bad} failed");
    Ok(if bad == 0 { 0 } else { 1 })
}

fn run_case(schema: &Schema, cfg: &Config, c: &Case, explain: bool) -> Vec<String> {
    let mut fails = Vec::new();
    let mock = Mock { answers: c.jev.clone(), seen: RefCell::new(None) };
    let oracle: Option<&dyn Oracle> = if c.no_jev { None } else { Some(&mock) };
    let res = interpret::run(schema, cfg, &c.words, &c.passthrough, oracle, c.defaults.as_ref());

    // 質問の検証(呼ばれていれば)。
    if let Some((state, qs)) = mock.seen.borrow().as_ref() {
        for k in &c.not_asked {
            if qs.contains_key(k) {
                fails.push(format!("asked `{k}` but not_asked says it must not be"));
            }
        }
        if let Some(st) = &c.state {
            for (k, want) in st {
                let want = crate::assemble::toml_to_json(want);
                let got = lookup(state, k);
                if got != want {
                    fails.push(format!("state.{k}: got {got}, want {want}"));
                }
            }
        }
    }

    match res {
        Ok(r) => {
            if explain {
                crate::output::explain(&r.tokens, r.jev.as_ref());
            }
            if let Some(e) = &c.error {
                fails.push(format!("expected error `{e}`, got argv {:?}", r.out.argv));
                return fails;
            }
            if let Some(want) = &c.argv
                && r.out.argv.as_ref() != Some(want)
            {
                fails.push(format!("argv:\n       got  {}\n       want {}", shell_words::join(r.out.argv.as_deref().unwrap_or(&[])), shell_words::join(want)));
            }
            if let Some(want) = &c.preview
                && r.out.preview.as_ref() != Some(want)
            {
                fails.push(format!("preview: got {:?}, want {:?}", r.out.preview, want));
            }
            let risk = c.risk.clone().unwrap_or_else(|| "none".into());
            if r.out.risk != risk {
                fails.push(format!("risk: got {}, want {risk}", r.out.risk));
            }
            if r.out.pipe != c.pipe {
                fails.push(format!("pipe: got {:?}, want {:?}", r.out.pipe, c.pipe));
            }
            if let Some(want) = c.confidence
                && (r.out.confidence - want).abs() > 0.011
            {
                fails.push(format!("confidence: got {:.3}, want {want:.2}", r.out.confidence));
            }
            if let Some(want) = &c.tokens {
                let got: Vec<&str> = r.tokens.iter().map(|t| t.text.as_str()).collect();
                if got != want.iter().map(String::as_str).collect::<Vec<_>>() {
                    fails.push(format!("tokens: got {got:?}, want {want:?}"));
                }
            }
        }
        Err(e) => {
            let got = match &e {
                JanyError::Unresolved(s) => format!("unresolved: {}", s.trim_end_matches(" (jev disabled)")),
                JanyError::Assemble(s) => s.clone(),
                other => other.to_string(),
            };
            match &c.error {
                Some(want) if want == &got => {}
                Some(want) => fails.push(format!("error: got `{got}`, want `{want}`")),
                None => fails.push(format!("error: {got}")),
            }
        }
    }
    fails
}

/// "hints.0" のようなドット区切りで state を引く。
fn lookup(v: &Value, path: &str) -> Value {
    let mut cur = v;
    for k in path.split('.') {
        cur = match cur {
            Value::Object(m) => m.get(k).unwrap_or(&Value::Null),
            Value::Array(a) => k.parse::<usize>().ok().and_then(|i| a.get(i)).unwrap_or(&Value::Null),
            _ => &Value::Null,
        };
    }
    cur.clone()
}
