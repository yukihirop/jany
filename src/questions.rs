//! Builds the state / questions sent to jev, and writes the answers back.
//! Every candidate comes from the schema's tables. jev only picks among them and never makes up values that are not in the input.
//!
//! Question keys:
//!   role.i        pick one of the roles that have jev = (for every unresolved word, always)
//!   <key>.i       for every word where a [[questions]] when matches
//!   <key>         a scope = "command" question (one per command)
//! For roles with an amount, the host knows what `unit` and `atleast` mean (they are written back into the amount).

use crate::amount;
use crate::jev::{Answer, Answers, Question as JQ, Questions, choice, noul};
use crate::schema::{Cond, Question, Schema};
use crate::token::{Source, Token};
use serde_json::{Value, json};

pub struct Built {
    pub state: Value,
    pub questions: Questions,
}

pub fn build(schema: &Schema, tokens: &[Token]) -> Built {
    let words: Vec<String> = tokens
        .iter()
        .map(|t| t.role.as_deref().and_then(|r| schema.role(r)).and_then(|r| r.mask.clone()).unwrap_or_else(|| t.text.clone()))
        .collect();
    let mut hints = serde_json::Map::new();
    for (i, t) in tokens.iter().enumerate() {
        if let Some(r) = &t.role {
            hints.insert(i.to_string(), Value::String(r.clone()));
        }
    }

    let mut questions = Questions::new();
    let role_criteria: Vec<(&str, Option<&str>)> = schema.jev_roles().into_iter().map(|(k, d)| (k, Some(d))).collect();

    for (i, t) in tokens.iter().enumerate() {
        let w = &t.text;
        if !t.resolved() {
            questions.insert(
                format!("role.{i}"),
                choice(format!("What is the role of `tokens[{i}]` (\"{w}\") in this {} command?", schema.command.name), &role_criteria),
            );
        }
        for q in schema.questions.iter().filter(|q| q.scope.as_deref() != Some("command")) {
            if !token_when(schema, q, tokens, i) {
                continue;
            }
            let ask = fill(&q.ask, i, w, tokens.get(i.wrapping_sub(1)));
            questions.insert(format!("{}.{i}", q.key), make(schema, q, ask));
        }
    }

    for q in schema.questions.iter().filter(|q| q.scope.as_deref() == Some("command")) {
        if !command_when(q, tokens) {
            continue;
        }
        questions.insert(q.key.clone(), make(schema, q, q.ask.clone()));
    }

    let mut state = json!({
        "tool": schema.command.description,
        "tokens": words,
        "hints": hints,
        "cwd": std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default(),
    });
    if let Some(j) = &schema.jev {
        for (k, spec) in &j.state {
            let v = tokens.iter().find(|t| t.is(&spec.value_of)).map(|t| Value::String(t.value().to_string())).unwrap_or(Value::Null);
            state[k] = v;
        }
    }
    Built { state, questions }
}

fn make(schema: &Schema, q: &Question, ask: String) -> JQ {
    if q.kind == "noul" {
        return noul(ask);
    }
    let mut c: Vec<(String, Option<String>)> = Vec::new();
    match q.choices.as_deref() {
        Some("amount_units") => c.extend(schema.unit_keys().into_iter().map(|u| (u, None))),
        Some(t) if t.starts_with("table:") => {
            let min = q.min_choice_len.unwrap_or(1);
            for (key, syns) in schema.table_entries(&t[6..]) {
                for s in syns.into_iter().filter(|s| s.len() >= min) {
                    let desc = if key == "_" || key == s { None } else { Some(key.clone()) };
                    c.push((s, desc));
                }
            }
        }
        _ => {}
    }
    if let Some(n) = &q.none {
        c.push(("none".into(), Some(n.clone())));
    }
    let refs: Vec<(&str, Option<&str>)> = c.iter().map(|(k, d)| (k.as_str(), d.as_deref())).collect();
    choice(ask, &refs)
}

fn fill(ask: &str, i: usize, w: &str, prev: Option<&Token>) -> String {
    ask.replace("{i}", &i.to_string())
        .replace("{w}", w)
        .replace("{prev_i}", &i.wrapping_sub(1).to_string())
        .replace("{prev_w}", prev.map(|p| p.text.as_str()).unwrap_or(""))
}

fn token_when(schema: &Schema, q: &Question, tokens: &[Token], i: usize) -> bool {
    let t = &tokens[i];
    let c = &q.when;
    if let Some(want) = c.resolved
        && t.resolved() != want
    {
        return false;
    }
    if let Some(want) = c.has_amount
        && t.amount.is_some() != want
    {
        return false;
    }
    if let Some(true) = c.unit_missing
        && !t.amount.as_ref().map(|a| a.unit.is_none()).unwrap_or(false)
    {
        return false;
    }
    if let Some(true) = c.direction_missing
        && !t.amount.as_ref().map(|a| a.at_least.is_none()).unwrap_or(false)
    {
        return false;
    }
    if let Some(r) = &c.next_role_not
        && tokens.get(i + 1).map(|n| n.is(r)).unwrap_or(false)
    {
        return false;
    }
    if let Some(true) = c.alphabetic
        && !(!t.text.is_empty() && t.text.chars().all(|ch| ch.is_ascii_alphabetic()))
    {
        return false;
    }
    if let Some(m) = c.max_len
        && t.text.len() > m
    {
        return false;
    }
    if let Some(pat) = &c.regex
        && !schema.re(pat).is_match(&t.text)
    {
        return false;
    }
    if let Some(conds) = &c.prev_any {
        let Some(prev) = (i > 0).then(|| &tokens[i - 1]) else { return false };
        if !conds.iter().any(|cd| cond_ok(cd, prev, None)) {
            return false;
        }
    }
    true
}

fn command_when(q: &Question, tokens: &[Token]) -> bool {
    let c = &q.when;
    if let Some(r) = &c.no_role
        && tokens.iter().any(|t| t.is(r))
    {
        return false;
    }
    if let Some(true) = c.any_unresolved
        && !tokens.iter().any(|t| !t.resolved())
    {
        return false;
    }
    true
}

/// Whether a word meets the condition. The caller works out `chained` (it only matters for join).
pub fn cond_ok(c: &Cond, t: &Token, chained: Option<bool>) -> bool {
    if let Some(want) = c.resolved
        && t.resolved() != want
    {
        return false;
    }
    if let Some(r) = &c.role
        && !t.is(r)
    {
        return false;
    }
    if let Some(tag) = &c.tag
        && !t.has_tag(tag)
    {
        return false;
    }
    if let Some(tag) = &c.tag_not
        && t.has_tag(tag)
    {
        return false;
    }
    if let Some(s) = &c.source {
        let want = match s.as_str() {
            "jev" => Source::Jev,
            _ => Source::Rule,
        };
        if t.source != want {
            return false;
        }
    }
    if let Some(true) = c.chained
        && chained != Some(true)
    {
        return false;
    }
    true
}

/// Writes the answers back into the tokens. Words decided by rules keep their role,
/// but the amount's direction (`atleast`) is written even to them (`a week`).
pub fn apply(schema: &Schema, tokens: &mut [Token], answers: &Answers) {
    for (i, t) in tokens.iter_mut().enumerate() {
        let was_resolved = t.resolved();
        if !was_resolved {
            let Some(a) = answers.get(&format!("role.{i}")) else { continue };
            let Some(role) = a.choice().map(str::to_string) else { continue };
            if schema.role(&role).is_none() {
                continue;
            }
            t.role = Some(role.clone());
            t.confidence = a.certainty();
            t.source = Source::Jev;
            t.note = Some(a.top2());
            t.probs = a.probabilities().cloned();

            // If the role has a table, look the word up as-is. Otherwise fall back to the typo question's answer (applies_to).
            if let Some(table) = schema.role(&role).and_then(|r| r.table.clone()) {
                match schema.table_lookup(&table, &t.text) {
                    Some(fixed) => t.fixed = fixed,
                    None => apply_typo(schema, t, i, &role, Some(&table), answers),
                }
            } else {
                apply_typo(schema, t, i, &role, None, answers);
            }

            // Roles with an amount: a word that is not a number cannot be assembled, so lower its confidence.
            if let Some(dim) = schema.role(&role).and_then(|r| r.amount.clone()) {
                let Some(mut am) = t.amount.clone() else {
                    t.confidence = t.confidence.min(0.3);
                    continue;
                };
                if dim == "count" {
                    am.unit = None;
                    am.at_least = None;
                } else if am.unit.is_none()
                    && let Some(a2) = answers.get(&format!("unit.{i}"))
                    && let Some(u) = a2.choice().filter(|u| *u != "none")
                    && schema.unit_dimension(u).as_deref() == Some(dim.as_str())
                {
                    am.unit = Some(u.to_string());
                    t.confidence = t.confidence.min(a2.certainty());
                }
                t.amount = Some(am);
            } else if schema.amount.is_some() && t.is_unit_role(schema) {
                // The unit role: fixed is the unit key.
                match amount::unit_of_word(schema, &t.text) {
                    Some(u) => t.fixed = Some(u),
                    None => t.confidence = t.confidence.min(0.3),
                }
            }
        }

        // Direction (also for words decided by rules).
        if let Some(mut am) = t.amount.clone()
            && am.at_least.is_none()
            && t.role.as_deref().and_then(|r| schema.role(r)).and_then(|r| r.amount.as_deref()).is_some_and(|d| d != "count")
            && let Some(p) = answers.get(&format!("atleast.{i}")).and_then(|a| a.noul())
        {
            am.at_least = Some(p > 0.5);
            t.amount = Some(am);
            t.source = Source::Jev;
            let c = p.max(1.0 - p);
            t.confidence = if was_resolved { c } else { t.confidence.min(c) };
            t.prepend_note(format!("{} p={p:.2}", if p > 0.5 { "at least" } else { "at most" }));
        }

        // sets.tag (typed, etc.): tag the word if it has the applies_to role and noul > 0.5.
        for q in schema.questions.iter().filter(|q| q.sets.is_some()) {
            if q.applies_to.as_deref().is_some_and(|r| t.is(r))
                && let Some(p) = answers.get(&format!("{}.{i}", q.key)).and_then(|a| a.noul())
                && p > 0.5
            {
                t.add_tag(&q.sets.as_ref().unwrap().tag);
            }
        }
    }
}

/// Choice questions with `applies_to = role`: the answer becomes fixed. On none, lower confidence to none_conf.
fn apply_typo(schema: &Schema, t: &mut Token, i: usize, role: &str, table: Option<&str>, answers: &Answers) {
    for q in schema.questions.iter().filter(|q| q.kind == "choice" && q.applies_to.as_deref() == Some(role)) {
        let Some(a) = answers.get(&format!("{}.{i}", q.key)) else { continue };
        match a.choice() {
            Some("none") | None => {
                if let Some(c) = q.none_conf {
                    t.confidence = t.confidence.min(c);
                }
            }
            Some(k) => {
                // The answer is a table word. If there is a table, turn it into the key.
                let table = q.choices.as_deref().and_then(|c| c.strip_prefix("table:")).or(table);
                let fixed = table.and_then(|tb| schema.table_lookup(tb, k)).flatten().unwrap_or_else(|| k.to_string());
                t.fixed = Some(fixed);
                t.confidence = t.confidence.min(a.certainty());
            }
        }
        return;
    }
}

impl Token {
    fn is_unit_role(&self, schema: &Schema) -> bool {
        // Look for the unit role through the schema's attach_unit.unit_role (build and apply run before repair).
        schema.repair.iter().any(|r| matches!(r, crate::schema::Repair::AttachUnit { unit_role, .. } if self.is(unit_role)))
    }
}

/// For tests: a mock answer (cases.toml format) as an Answer.
pub fn answer_from_toml(v: &toml::Value) -> Option<Answer> {
    let t = v.as_table()?;
    if let Some(p) = t.get("noul").and_then(|x| x.as_float().or_else(|| x.as_integer().map(|n| n as f64))) {
        return Some(Answer::Noul { noul: p as f32 });
    }
    let choice = t.get("choice")?.as_str()?.to_string();
    let mut probabilities = std::collections::BTreeMap::new();
    if let Some(p) = t.get("probs").and_then(|x| x.as_table()) {
        for (k, x) in p {
            if let Some(f) = x.as_float().or_else(|| x.as_integer().map(|n| n as f64)) {
                probabilities.insert(k.clone(), f as f32);
            }
        }
    }
    let confidence = t
        .get("confidence")
        .and_then(|x| x.as_float().or_else(|| x.as_integer().map(|n| n as f64)))
        .map(|f| f as f32)
        .unwrap_or_else(|| probabilities.values().copied().fold(0.0, f32::max).max(if probabilities.is_empty() { 1.0 } else { 0.0 }));
    if probabilities.is_empty() {
        probabilities.insert(choice.clone(), confidence);
    }
    Some(Answer::Choice { choice, confidence, probabilities })
}
