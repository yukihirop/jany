//! Amount parser (jind's `rules::parse_amount`, driven by the schema's unit table).
//! `7` `7d` `+7d` `-2h` `>10M` `100MB` `<1k` → (direction, number, unit). None if there is no number.

use crate::schema::Schema;
use crate::token::Amount;

/// Unit word → unit key. Words in `case_sensitive.suffixes` are compared as-is; everything else is compared lowercased.
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
            // An ambiguous suffix (`m`) counts as no unit and goes to jev. Any other unknown suffix means it is not a number.
            None if schema.amount.as_ref().map(|a| a.ambiguous_suffixes.iter().any(|s| s == suffix)).unwrap_or(false) => None,
            None => return None,
        }
    };
    Some(Amount { n, unit, at_least })
}
