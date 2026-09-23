//! The rule engine. Tries the schema's [[rules]] top to bottom; the first match wins.
//! Words it cannot decide go to jev with role = None.

use crate::amount;
use crate::schema::{Rule, Schema};
use crate::token::{Amount, Token};

pub fn classify(schema: &Schema, words: &[String]) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        let w = &words[i];
        let mut t = Token::new(w.clone());
        let mut next_tok: Option<Token> = None;
        for rule in &schema.rules {
            if rule.once && out.iter().any(|p| p.is(&rule.role)) {
                continue;
            }
            let Some(hit) = matches(schema, rule, w) else { continue };
            if !when_ok(schema, rule, &out, words, i) {
                continue;
            }
            apply(schema, rule, &mut t, &hit);
            if let Some(next) = &rule.next
                && let Some(nw) = words.get(i + 1)
            {
                let mut n = Token::new(nw.clone());
                n.set_rule(&next.role);
                if let Some(f) = &next.fixed {
                    n.fixed = Some(template(f, nw, None));
                } else if let Some(table) = schema.role(&next.role).and_then(|r| r.table.as_deref())
                    && let Some(fixed) = schema.table_lookup(table, nw)
                {
                    // If the role has a table, look up the corrected value (restart always → always, arm64 → linux/arm64).
                    n.fixed = fixed;
                }
                next_tok = Some(n);
            }
            break;
        }
        out.push(t);
        i += 1;
        if let Some(n) = next_tok {
            out.push(n);
            i += 1;
        }
    }
    out
}

/// What comes with a match.
pub struct Hit {
    /// The table key (becomes fixed). None for the "_" key.
    pub fixed: Option<String>,
    pub amount: Option<Amount>,
    /// For unit_word, its unit key.
    pub unit: Option<String>,
    pub captures: Vec<String>,
}

fn matches(schema: &Schema, rule: &Rule, w: &str) -> Option<Hit> {
    let m = &rule.match_;
    let mut hit = Hit { fixed: None, amount: None, unit: None, captures: Vec::new() };
    if let Some(p) = &m.prefix {
        if !(w.starts_with(p.as_str()) && w.len() >= m.min_len.unwrap_or(1)) {
            return None;
        }
    } else if let Some(word) = &m.word {
        if !word.contains(w) {
            return None;
        }
    } else if let Some(table) = &m.table {
        hit.fixed = schema.table_lookup(table, w)?;
    } else if let Some(pat) = &m.regex {
        let caps = schema.re(pat).captures(w)?;
        hit.captures = (0..caps.len()).map(|k| caps.get(k).map(|c| c.as_str().to_string()).unwrap_or_default()).collect();
    } else if let Some(b) = &m.builtin {
        match b.as_str() {
            "path_like" => {
                if !is_path_like(w) {
                    return None;
                }
            }
            "glob" => {
                if !is_glob(w) {
                    return None;
                }
            }
            "existing_dir" => {
                if !is_existing_dir(w) {
                    return None;
                }
            }
            "amount" => hit.amount = Some(amount::parse(schema, w)?),
            "unit_word" => hit.unit = Some(amount::unit_of_word(schema, w)?),
            other => panic!("unknown builtin `{other}` (schema validation should have caught this)"),
        }
    } else {
        return None;
    }
    Some(hit)
}

fn when_ok(schema: &Schema, rule: &Rule, out: &[Token], words: &[String], i: usize) -> bool {
    let w = &words[i];
    let c = &rule.when;
    let prev = out.last();
    if let Some(true) = c.not_amount
        && amount::parse(schema, w).is_some()
    {
        return false;
    }
    if let Some(r) = &c.prev_role
        && !prev.map(|p| p.is(r)).unwrap_or(false)
    {
        return false;
    }
    if let Some(pf) = &c.prev_prefix
        && !prev.map(|p| p.text.starts_with(pf.as_str())).unwrap_or(false)
    {
        return false;
    }
    if let Some(want) = c.prev_is_number {
        let is = i > 0 && amount::parse(schema, &words[i - 1]).is_some();
        if is != want {
            return false;
        }
    }
    if let Some(pat) = &c.prev_regex
        && !prev.map(|p| schema.re(pat).is_match(&p.text)).unwrap_or(false)
    {
        return false;
    }
    if c.has_unit.is_some() || c.has_direction.is_some() {
        let Some(a) = amount::parse(schema, w) else { return false };
        if let Some(want) = c.has_unit
            && a.unit.is_some() != want
        {
            return false;
        }
        if let Some(want) = c.has_direction
            && a.at_least.is_some() != want
        {
            return false;
        }
    }
    true
}

fn apply(schema: &Schema, rule: &Rule, t: &mut Token, hit: &Hit) {
    // The rule's own amount wins; otherwise the one the builtin read.
    if let Some(ra) = &rule.amount {
        let unit = match ra.unit.as_deref() {
            Some("$unit") => hit.unit.clone(),
            Some(u) => Some(u.to_string()),
            None => None,
        };
        t.amount = Some(Amount { n: ra.n, unit, at_least: ra.at_least });
    } else if let Some(a) = &hit.amount {
        // Only for roles with an amount (and unresolved / by_dimension). The 644 in `-perm 644` is passthrough, so it gets none.
        let keeps_amount = matches!(rule.role.as_str(), "unresolved" | "by_dimension") || schema.role(&rule.role).is_some_and(|r| r.amount.is_some());
        if keeps_amount {
            t.amount = Some(a.clone());
        }
    }

    let role: Option<String> = match rule.role.as_str() {
        // No role, only the amount, and on to jev.
        "unresolved" => None,
        // Choose time_amount / size_amount from the unit's dimension.
        "by_dimension" => {
            let unit = t.amount.as_ref().and_then(|a| a.unit.clone()).or_else(|| hit.unit.clone());
            unit.and_then(|u| schema.unit_dimension(&u)).and_then(|d| schema.role_for_dimension(&d).map(str::to_string))
        }
        r => Some(r.to_string()),
    };
    if let Some(r) = role {
        t.set_rule(&r);
    }
    if let Some(f) = &rule.fixed {
        t.fixed = Some(template(f, &t.text, Some(&hit.captures)));
    } else if let Some(f) = &hit.fixed {
        t.fixed = Some(f.clone());
    } else if let Some(u) = &hit.unit
        && rule.role != "by_dimension"
    {
        // When the unit word itself is the role (unit), fixed is the unit key.
        t.fixed = Some(u.clone());
    }
    if let Some(n) = &rule.note {
        t.note = Some(n.clone());
    }
    if let Some(tag) = &rule.tag {
        t.add_tag(tag);
    }
}

/// Templates for `fixed`: "$upper" / "$lower" / "$1" / anything else is literal.
pub fn template(spec: &str, text: &str, captures: Option<&Vec<String>>) -> String {
    match spec {
        "$upper" => text.to_uppercase(),
        "$lower" => text.to_lowercase(),
        s if s.starts_with('$') => match (s[1..].parse::<usize>(), captures) {
            (Ok(n), Some(c)) => c.get(n).cloned().unwrap_or_default(),
            _ => s.to_string(),
        },
        s => s.to_string(),
    }
}

/// Words that look like a path. Bare names like `src` are checked by existing_dir.
pub fn is_path_like(w: &str) -> bool {
    w == "." || w == ".." || w == "~" || w.starts_with('/') || w.starts_with("./") || w.starts_with("../") || w.starts_with("~/")
}

pub fn is_glob(w: &str) -> bool {
    w.contains('*') || w.contains('?') || (w.contains('[') && w.contains(']'))
}

fn is_existing_dir(w: &str) -> bool {
    !w.is_empty() && !w.contains(char::is_whitespace) && std::path::Path::new(w).is_dir()
}
