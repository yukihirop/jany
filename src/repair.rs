//! jev answers each word independently, so relations between neighbours are fixed here. Four primitives:
//!   attach_unit  attach a unit word to the amount before it (jind attach_units)
//!   claim        change the role of the word next to a marker (jind mark_depth / mark_excludes)
//!   join         join a word jev called "a continuation of the previous word" onto it (jurl merge_joined)
//!   pair         choose the key value key value alternation from jev's probabilities (jurl pair_key_values)

use crate::jev::Answers;
use crate::questions::cond_ok;
use crate::schema::{KeyRoleIf, Repair, Schema, Sel};
use crate::token::{Source, Token};

pub fn repair(schema: &Schema, tokens: &mut Vec<Token>, answers: &Answers) {
    for r in &schema.repair {
        match r {
            Repair::AttachUnit { unit_role, amount_roles } => attach_unit(schema, tokens, unit_role, amount_roles),
            Repair::Claim { marker, role, side, many, from, skip, require, clear } => {
                claim(tokens, marker, role, side, *many, from, skip, require.as_ref().and_then(|r| r.amount_without_unit).unwrap_or(false), clear)
            }
            Repair::Join { answer, sep, into } => join(tokens, answers, answer, sep, into),
            Repair::Pair { members, key_probs, value_probs, key_role, value_role, key_role_if } => {
                pair(tokens, answers, members, key_probs, value_probs, key_role, value_role, key_role_if.as_ref())
            }
        }
    }
}

/// `7 days` / `10 MB`: a unit word attaches to the amount before it. If its dimension disagrees with jev's role, the unit (a rule) wins.
fn attach_unit(schema: &Schema, tokens: &mut [Token], unit_role: &str, amount_roles: &[String]) {
    for i in 1..tokens.len() {
        if !tokens[i].is(unit_role) {
            continue;
        }
        let Some(u) = tokens[i].fixed.clone() else { continue };
        let Some(dim) = schema.unit_dimension(&u) else { continue };
        let Some(want) = schema.role_for_dimension(&dim).map(str::to_string) else { continue };
        let prev = &mut tokens[i - 1];
        let Some(mut am) = prev.amount.clone() else { continue };
        if !amount_roles.iter().any(|r| prev.is(r)) {
            continue;
        }
        if !prev.is(&want) {
            let old = prev.role_str().to_string();
            prev.prepend_note(format!("{old} → {want} (unit says so)"));
            prev.role = Some(want);
            if prev.source == Source::Jev {
                // The unit settled the role, so what remains is how sure the direction is, not jev's role probability.
                prev.confidence = prev.confidence.max(0.8);
            }
        }
        am.unit = Some(u);
        prev.amount = Some(am);
    }
}

/// Gives `role` to the word next to a marker (after / before / both = after first). With many, as long as they continue; roles in skip are stepped over.
#[allow(clippy::too_many_arguments)]
fn claim(tokens: &mut [Token], marker: &str, role: &str, side: &str, many: bool, from: &[Sel], skip: &[String], amount_without_unit: bool, clear: &[String]) {
    let n = tokens.len();
    let accepts = |t: &Token| -> bool {
        if amount_without_unit && !t.amount.as_ref().map(|a| a.unit.is_none()).unwrap_or(false) {
            return false;
        }
        from.iter().any(|s| {
            let c = s.cond();
            // "unresolved" means a word without a role.
            if c.role.as_deref() == Some("unresolved") { !t.resolved() } else { cond_ok(&c, t, None) }
        })
    };
    let mut i = 0;
    while i < n {
        if !tokens[i].is(marker) {
            i += 1;
            continue;
        }
        let marker_text = tokens[i].text.clone();
        let mut claimed_any = false;
        let mut last = i;
        let dirs: Vec<isize> = match side {
            "before" => vec![-1],
            "both" => vec![1, -1],
            _ => vec![1],
        };
        'dirs: for d in dirs {
            let mut j = i as isize + d;
            while j >= 0 && (j as usize) < n {
                let t = &tokens[j as usize];
                if skip.iter().any(|s| t.is(s)) {
                    j += d;
                    continue;
                }
                if t.is(role) && many {
                    last = j as usize;
                    j += d;
                    continue;
                }
                if !accepts(t) {
                    break;
                }
                let t = &mut tokens[j as usize];
                if !t.is(role) {
                    let old = t.role_str().to_string();
                    t.prepend_note(format!("{old} → {role} (next to \"{marker_text}\")"));
                }
                t.role = Some(role.to_string());
                if t.source == Source::Rule {
                    t.confidence = 1.0;
                }
                if let Some(am) = t.amount.as_mut() {
                    if clear.iter().any(|c| c == "unit") {
                        am.unit = None;
                    }
                    if clear.iter().any(|c| c == "direction") {
                        am.at_least = None;
                    }
                }
                claimed_any = true;
                last = j as usize;
                if !many {
                    break 'dirs;
                }
                j += d;
            }
            if claimed_any {
                break;
            }
        }
        i = if side == "before" { i + 1 } else { last.max(i) + 1 };
    }
}

/// Joins words jev called "a continuation of the previous word's value" (answer.i > 0.5) onto the previous word. Scans from the back, so `a b c` also becomes one.
/// cur must be a word jev decided; prev must match one of `into`.
fn join(tokens: &mut Vec<Token>, answers: &Answers, answer: &str, sep: &str, into: &[crate::schema::Cond]) {
    let p_of = |i: usize| answers.get(&format!("{answer}.{i}")).and_then(|a| a.noul());
    let mut i = tokens.len();
    while i > 1 {
        i -= 1;
        let Some(p) = p_of(i) else { continue };
        if p <= 0.5 || tokens[i].source != Source::Jev {
            continue;
        }
        let prev_chained = p_of(i - 1).map(|q| q > 0.5).unwrap_or(false);
        if !into.iter().any(|c| cond_ok(c, &tokens[i - 1], Some(prev_chained))) {
            continue;
        }
        let cur = tokens.remove(i);
        let prev = &mut tokens[i - 1];
        prev.text = format!("{}{sep}{}", prev.text, cur.text);
        prev.fixed = None;
        prev.tags.clear();
        prev.confidence = prev.confidence.min(p);
        // pair recomputes confidence from probs, so carry join's p into the "is a value" probability too.
        if let Some(r) = prev.role.clone()
            && let Some(m) = prev.probs.as_mut()
        {
            let v = m.entry(r).or_insert(1.0);
            *v = v.min(p);
        }
        prev.prepend_note(format!("joined \"{}\" p={p:.2}", cur.text));
    }
}

/// For each run of consecutive members, compare the likelihood of "starts with a key" and "starts with a value" and take the better.
#[allow(clippy::too_many_arguments)]
fn pair(tokens: &mut [Token], answers: &Answers, members: &[Sel], key_probs: &[String], value_probs: &[String], key_role: &str, value_role: &str, key_role_if: Option<&KeyRoleIf>) {
    let is_member = |t: &Token| members.iter().any(|s| cond_ok(&s.cond(), t, None));
    let key_roles: Vec<&str> = key_probs.iter().map(String::as_str).collect();
    let n = tokens.len();
    let mut i = 0;
    while i < n {
        if !is_member(&tokens[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < n && is_member(&tokens[i]) {
            i += 1;
        }
        let run = start..i;
        if run.len() < 2 {
            continue;
        }
        let key_role = key_role_if.filter(|k| k.any.iter().any(|c| pair_cond(c, tokens, &run, answers))).map(|k| k.role.as_str()).unwrap_or(key_role);

        // Likelihood of the two arrangements. Words without probabilities (decided by rules) count as 1.0 / 1e-3.
        let p = |t: &Token, key: bool| -> f32 {
            match &t.probs {
                Some(m) => {
                    let k: f32 = key_probs.iter().map(|r| m.get(r).copied().unwrap_or(0.0)).sum();
                    let v: f32 = value_probs.iter().map(|r| m.get(r).copied().unwrap_or(0.0)).sum();
                    (if key { k } else { v }).max(1e-3)
                }
                None => {
                    let is_key = t.role.as_deref().is_some_and(|r| key_roles.contains(&r));
                    if is_key == key { 1.0 } else { 1e-3 }
                }
            }
        };
        let mut p_key_first = 1.0f32;
        let mut p_val_first = 1.0f32;
        for (k, t) in tokens[run.clone()].iter().enumerate() {
            p_key_first *= p(t, k % 2 == 0);
            p_val_first *= p(t, k % 2 == 1);
        }
        let key_first = p_key_first >= p_val_first;
        let conf = p_key_first.max(p_val_first) / (p_key_first + p_val_first);

        for (k, idx) in run.clone().enumerate() {
            let is_key = (k % 2 == 0) == key_first;
            let t = &mut tokens[idx];
            let new_role = if is_key { key_role } else { value_role };
            let switched = !t.is(new_role);
            if switched {
                let old = t.role_str().to_string();
                t.prepend_note(format!("{old} → {new_role} (paired)"));
            }
            t.role = Some(new_role.to_string());
            if let Some(m) = &t.probs {
                let role_p: f32 = if is_key { key_probs.iter().map(|r| m.get(r).copied().unwrap_or(0.0)).sum() } else { value_probs.iter().map(|r| m.get(r).copied().unwrap_or(0.0)).sum() };
                // A word whose role the arrangement changed gets the arrangement's confidence; an unchanged one gets min(role p, arrangement).
                t.confidence = if switched { conf } else { role_p.min(conf) };
            }
        }
    }
}

fn pair_cond(c: &crate::schema::PairCond, tokens: &[Token], run: &std::ops::Range<usize>, answers: &Answers) -> bool {
    if let Some(r) = &c.member_role {
        return tokens[run.clone()].iter().any(|t| t.is(r));
    }
    if let Some(role) = &c.value_of {
        return tokens.iter().find(|t| t.is(role)).map(|t| Some(t.value()) == c.is.as_deref()).unwrap_or(false);
    }
    if let Some(a) = &c.answer {
        if let Some(r) = &c.no_role
            && tokens.iter().any(|t| t.is(r))
        {
            return false;
        }
        return answers.get(a).and_then(|x| x.noul()).map(|p| p > c.gt.unwrap_or(0.5)).unwrap_or(false);
    }
    false
}
