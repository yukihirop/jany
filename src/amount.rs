//! 量のパーサ(jind `rules::parse_amount` を schema の単位表で動かすもの)。
//! `7` `7d` `+7d` `-2h` `>10M` `100MB` `<1k` → (向き, 数, 単位)。数が無ければ None。

use crate::schema::Schema;
use crate::token::Amount;

/// 単位語 → 単位キー。`case_sensitive.suffixes` にある語は大文字小文字をそのまま比べ、それ以外は小文字で比べる。
pub fn unit_of_word(schema: &Schema, w: &str) -> Option<String> {
    let a = schema.amount.as_ref()?;
    let exact = a.case_sensitive.suffixes.iter().any(|s| s == w);
    let lw = w.to_ascii_lowercase();
    for units in a.units.values() {
        for (key, syns) in units {
            let hit = if exact { syns.iter().any(|s| s == w) } else { syns.iter().any(|s| s.to_ascii_lowercase() == lw && !a.case_sensitive.suffixes.contains(s)) };
            if hit {
                return Some(key.clone());
            }
        }
    }
    None
}

pub fn parse(schema: &Schema, w: &str) -> Option<Amount> {
    let (at_least, rest) = match w.chars().next()? {
        '+' | '>' => (Some(true), &w[1..]),
        '-' | '<' => (Some(false), &w[1..]),
        _ => (None, w),
    };
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    if digits.is_empty() || !digits.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    let n: f64 = digits.parse().ok()?;
    let suffix = &rest[digits.len()..];
    let unit = if suffix.is_empty() {
        None
    } else {
        match unit_of_word(schema, suffix) {
            Some(u) => Some(u),
            // 曖昧な接尾辞(`m`)は単位なし扱いで jev に回す。それ以外の未知の接尾辞は数ではない。
            None if schema.amount.as_ref().map(|a| a.ambiguous_suffixes.iter().any(|s| s == suffix)).unwrap_or(false) => None,
            None => return None,
        }
    };
    Some(Amount { n, unit, at_least })
}
