//! 規則エンジン。schema の [[rules]] を上から順に試し、最初に当たったものが勝つ。
//! 決められない語は role = None のまま jev へ。

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
                    // 役割に表があれば補正値を引く(restart always → always、arm64 → linux/arm64)。
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

/// 当たったときの付随情報。
pub struct Hit {
    /// 表のキー(fixed になる)。"_" の表は None。
    pub fixed: Option<String>,
    pub amount: Option<Amount>,
    /// unit_word のとき、その単位キー。
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
    // amount は規則の指定が優先、無ければ builtin が読んだもの。
    if let Some(ra) = &rule.amount {
        let unit = match ra.unit.as_deref() {
            Some("$unit") => hit.unit.clone(),
            Some(u) => Some(u.to_string()),
            None => None,
        };
        t.amount = Some(Amount { n: ra.n, unit, at_least: ra.at_least });
    } else if let Some(a) = &hit.amount {
        // 量を持つ役割(と未解決 / by_dimension)にだけ付ける。`-perm 644` の 644 は passthrough なので付けない。
        let keeps_amount = matches!(rule.role.as_str(), "unresolved" | "by_dimension") || schema.role(&rule.role).is_some_and(|r| r.amount.is_some());
        if keeps_amount {
            t.amount = Some(a.clone());
        }
    }

    let role: Option<String> = match rule.role.as_str() {
        // 役割は付けず amount だけ持たせて jev へ。
        "unresolved" => None,
        // 単位の次元で time_amount / size_amount を選ぶ。
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
        // 単位語そのものが役割(unit)のとき、fixed は単位キー。
        t.fixed = Some(u.clone());
    }
    if let Some(n) = &rule.note {
        t.note = Some(n.clone());
    }
    if let Some(tag) = &rule.tag {
        t.add_tag(tag);
    }
}

/// `fixed` のテンプレート: "$upper" / "$lower" / "$1" / それ以外はそのまま。
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

/// 見た目でパスと分かるもの。`src` のような裸の名前は existing_dir で見る。
pub fn is_path_like(w: &str) -> bool {
    w == "." || w == ".." || w == "~" || w.starts_with('/') || w.starts_with("./") || w.starts_with("../") || w.starts_with("~/")
}

pub fn is_glob(w: &str) -> bool {
    w.contains('*') || w.contains('?') || (w.contains('[') && w.contains(']'))
}

fn is_existing_dir(w: &str) -> bool {
    !w.is_empty() && !w.contains(char::is_whitespace) && std::path::Path::new(w).is_dir()
}
