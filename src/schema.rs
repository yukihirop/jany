//! The shape of `~/.config/jany/cmd/<name>[/<sub>]/schema.toml`. Meanings are in docs/HOST.md.
//! This only reads it; interpretation lives in rules / questions / repair.

use crate::error::JanyError;
use regex::Regex;
use serde::Deserialize;
use std::collections::{hash_map::Entry, BTreeMap, HashMap};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Schema {
    pub command: Command,
    #[serde(default)]
    pub roles: Vec<Role>,
    #[serde(default)]
    pub amount: Option<AmountSpec>,
    #[serde(default)]
    pub tables: BTreeMap<String, toml::Table>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub questions: Vec<Question>,
    #[serde(default)]
    pub jev: Option<JevSection>,
    #[serde(default)]
    pub repair: Vec<Repair>,
    pub assemble: AssembleSpec,
    #[serde(default)]
    pub defaults: toml::Table,
    #[serde(default)]
    pub confirm: Confirm,
    /// Dim hints after `jany <name> ` in zsh (`jany --suggest`), in the order written.
    #[serde(default)]
    pub placeholders: Vec<Placeholder>,

    /// The directory it was loaded from (where assemble.sh is).
    #[serde(skip)]
    pub dir: PathBuf,
    #[serde(skip)]
    regexes: HashMap<String, Regex>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub example: Option<String>,
}

/// One slot of the dim hint, e.g. `<path>`. Hidden once a word has any of `roles`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placeholder {
    pub text: String,
    pub roles: Vec<String>,
    /// A word the rules could not decide (jev will) fills this slot, e.g. the bare `nginx` for `<image>`.
    #[serde(default)]
    pub bare: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Role {
    pub key: String,
    /// Description shown in jev's Choice. Without it, only rules can assign the role.
    #[serde(default)]
    pub jev: Option<String>,
    #[serde(default)]
    pub table: Option<String>,
    /// "time" | "size" | "count": words with this role carry an amount.
    #[serde(default)]
    pub amount: Option<String>,
    /// Replaced by this string in jev's state.tokens (so secrets are not sent).
    #[serde(default)]
    pub mask: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AmountSpec {
    #[serde(default)]
    pub ambiguous_suffixes: Vec<String>,
    /// dimension → unit key → words.
    #[serde(default)]
    pub units: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    #[serde(default)]
    pub case_sensitive: CaseSensitive,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CaseSensitive {
    #[serde(default)]
    pub suffixes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    #[serde(rename = "match")]
    pub match_: Match,
    #[serde(default)]
    pub when: When,
    pub role: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub amount: Option<RuleAmount>,
    /// "$upper" | "$lower" | "$1" | anything else is literal.
    #[serde(default)]
    pub fixed: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub next: Option<Next>,
    #[serde(default)]
    pub once: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Match {
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub min_len: Option<usize>,
    #[serde(default)]
    pub word: Option<OneOrMany>,
    #[serde(default)]
    pub table: Option<String>,
    #[serde(default)]
    pub regex: Option<String>,
    #[serde(default)]
    pub builtin: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    pub fn contains(&self, w: &str) -> bool {
        match self {
            OneOrMany::One(s) => s == w,
            OneOrMany::Many(v) => v.iter().any(|s| s == w),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct When {
    #[serde(default)]
    pub not_amount: Option<bool>,
    #[serde(default)]
    pub prev_role: Option<String>,
    #[serde(default)]
    pub prev_prefix: Option<String>,
    #[serde(default)]
    pub prev_is_number: Option<bool>,
    #[serde(default)]
    pub has_unit: Option<bool>,
    #[serde(default)]
    pub has_direction: Option<bool>,
    #[serde(default)]
    pub prev_regex: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleAmount {
    pub n: f64,
    /// A unit key, or "$unit" (the key of the matched unit word).
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub at_least: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Next {
    pub role: String,
    #[serde(default)]
    pub fixed: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub key: String,
    /// "choice" | "noul"
    pub kind: String,
    #[serde(default)]
    pub when: QWhen,
    pub ask: String,
    /// "amount_units" | "table:<name>"
    #[serde(default)]
    pub choices: Option<String>,
    #[serde(default)]
    pub min_choice_len: Option<usize>,
    #[serde(default)]
    pub none: Option<String>,
    #[serde(default)]
    pub applies_to: Option<String>,
    #[serde(default)]
    pub none_conf: Option<f32>,
    #[serde(default)]
    pub sets: Option<Sets>,
    /// "command" asks once instead of per word.
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct QWhen {
    #[serde(default)]
    pub resolved: Option<bool>,
    #[serde(default)]
    pub has_amount: Option<bool>,
    #[serde(default)]
    pub unit_missing: Option<bool>,
    #[serde(default)]
    pub direction_missing: Option<bool>,
    #[serde(default)]
    pub next_role_not: Option<String>,
    #[serde(default)]
    pub alphabetic: Option<bool>,
    #[serde(default)]
    pub max_len: Option<usize>,
    #[serde(default)]
    pub regex: Option<String>,
    #[serde(default)]
    pub prev_any: Option<Vec<Cond>>,
    // for scope = "command"
    #[serde(default)]
    pub no_role: Option<String>,
    #[serde(default)]
    pub any_unresolved: Option<bool>,
}

/// A condition on a word's state (used by prev_any / repair.join.into).
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Cond {
    #[serde(default)]
    pub resolved: Option<bool>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub tag_not: Option<String>,
    /// "rule" | "jev"
    #[serde(default)]
    pub source: Option<String>,
    /// join: the previous word was itself answered as join.
    #[serde(default)]
    pub chained: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sets {
    pub tag: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct JevSection {
    #[serde(default)]
    pub state: BTreeMap<String, StateSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateSpec {
    pub value_of: String,
}

/// A role selector: "path" or { role = "path", tag = "existing_dir" }.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Sel {
    Role(String),
    Cond(Cond),
}

impl Sel {
    pub fn cond(&self) -> Cond {
        match self {
            Sel::Role(r) => Cond { role: Some(r.clone()), ..Default::default() },
            Sel::Cond(c) => c.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Repair {
    AttachUnit {
        unit_role: String,
        amount_roles: Vec<String>,
    },
    Claim {
        marker: String,
        role: String,
        /// "after" | "before" | "both"
        #[serde(default = "default_side")]
        side: String,
        #[serde(default)]
        many: bool,
        from: Vec<Sel>,
        #[serde(default)]
        skip: Vec<String>,
        #[serde(default)]
        require: Option<Require>,
        #[serde(default)]
        clear: Vec<String>,
    },
    Join {
        answer: String,
        #[serde(default = "default_sep")]
        sep: String,
        into: Vec<Cond>,
    },
    Pair {
        members: Vec<Sel>,
        key_probs: Vec<String>,
        value_probs: Vec<String>,
        key_role: String,
        value_role: String,
        #[serde(default)]
        key_role_if: Option<KeyRoleIf>,
    },
}

fn default_side() -> String {
    "after".into()
}
fn default_sep() -> String {
    " ".into()
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Require {
    #[serde(default)]
    pub amount_without_unit: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyRoleIf {
    pub role: String,
    pub any: Vec<PairCond>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairCond {
    #[serde(default)]
    pub member_role: Option<String>,
    #[serde(default)]
    pub value_of: Option<String>,
    #[serde(default)]
    pub is: Option<String>,
    #[serde(default)]
    pub answer: Option<String>,
    #[serde(default)]
    pub gt: Option<f32>,
    #[serde(default)]
    pub no_role: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssembleSpec {
    pub script: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Confirm {
    #[serde(default)]
    pub preview_readonly: bool,
    #[serde(default = "default_preview_lines")]
    pub preview_lines: usize,
    #[serde(default)]
    pub unsafe_note: Option<String>,
}

fn default_preview_lines() -> usize {
    10
}

impl Schema {
    pub fn load(dir: &Path) -> Result<Schema, JanyError> {
        let p = dir.join("schema.toml");
        let text = std::fs::read_to_string(&p).map_err(|e| JanyError::Schema(format!("{}: {e}", p.display())))?;
        let mut s: Schema = toml::from_str(&text).map_err(|e| JanyError::Schema(format!("{}: {e}", p.display())))?;
        // Make it absolute, because cases chdir.
        s.dir = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        s.compile_regexes()?;
        s.validate()?;
        Ok(s)
    }

    fn compile_regexes(&mut self) -> Result<(), JanyError> {
        let mut pats: Vec<String> = Vec::new();
        for r in &self.rules {
            pats.extend(r.match_.regex.clone());
            pats.extend(r.when.prev_regex.clone());
        }
        for q in &self.questions {
            pats.extend(q.when.regex.clone());
        }
        for p in pats {
            if let Entry::Vacant(entry) = self.regexes.entry(p) {
                let re = Regex::new(entry.key()).map_err(|e| JanyError::Schema(format!("regex `{}`: {e}", entry.key())))?;
                entry.insert(re);
            }
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), JanyError> {
        /// The builtins that matches() in rules.rs knows.
        const BUILTINS: &[&str] = &["path_like", "glob", "existing_dir", "amount", "unit_word"];
        let known: Vec<&str> = self.roles.iter().map(|r| r.key.as_str()).collect();
        for r in &self.rules {
            if r.role != "by_dimension" && r.role != "unresolved" && !known.contains(&r.role.as_str()) {
                return Err(JanyError::Schema(format!("rule role `{}` is not in [[roles]]", r.role)));
            }
            if let Some(t) = &r.match_.table
                && !self.tables.contains_key(t)
            {
                return Err(JanyError::Schema(format!("rule table `{t}` is not in [tables]")));
            }
            if let Some(b) = &r.match_.builtin
                && !BUILTINS.contains(&b.as_str())
            {
                return Err(JanyError::Schema(format!("unknown builtin `{b}` (one of {})", BUILTINS.join(", "))));
            }
        }
        for p in &self.placeholders {
            if let Some(r) = p.roles.iter().find(|r| !known.contains(&r.as_str())) {
                return Err(JanyError::Schema(format!("placeholder `{}`: role `{r}` is not in [[roles]]", p.text)));
            }
        }
        Ok(())
    }

    pub fn re(&self, pat: &str) -> &Regex {
        &self.regexes[pat]
    }

    pub fn role(&self, key: &str) -> Option<&Role> {
        self.roles.iter().find(|r| r.key == key)
    }

    /// A table's (key, words) in order. Key "_" means no fixed value.
    pub fn table_entries(&self, name: &str) -> Vec<(String, Vec<String>)> {
        self.tables
            .get(name)
            .map(|t| {
                t.iter()
                    .map(|(k, v)| (k.clone(), v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Looks a word up in a table (case-insensitive). On a hit, Some(fixed); fixed is None for key "_".
    pub fn table_lookup(&self, name: &str, w: &str) -> Option<Option<String>> {
        let lw = w.to_ascii_lowercase();
        for (k, syns) in self.table_entries(name) {
            if syns.iter().any(|s| s.to_ascii_lowercase() == lw) {
                return Some(if k == "_" { None } else { Some(k) });
            }
        }
        None
    }

    /// The dimension of a unit key ("time" / "size").
    pub fn unit_dimension(&self, unit: &str) -> Option<String> {
        let a = self.amount.as_ref()?;
        a.units.iter().find(|(_, units)| units.contains_key(unit)).map(|(d, _)| d.clone())
    }

    /// Every unit key (in dimension order).
    pub fn unit_keys(&self) -> Vec<String> {
        self.amount.as_ref().map(|a| a.units.values().flat_map(|u| u.keys().cloned()).collect()).unwrap_or_default()
    }

    /// The role whose amount has that dimension.
    pub fn role_for_dimension(&self, dim: &str) -> Option<&str> {
        self.roles.iter().find(|r| r.amount.as_deref() == Some(dim)).map(|r| r.key.as_str())
    }

    pub fn jev_roles(&self) -> Vec<(&str, &str)> {
        self.roles.iter().filter_map(|r| r.jev.as_deref().map(|d| (r.key.as_str(), d))).collect()
    }
}

/// `jany <name> [<sub> ...]`: finds the deepest matching directory under cmd_dir.
/// Returns (schema, number of words consumed).
pub fn resolve(cmd_dir: &Path, words: &[String]) -> Result<(Schema, usize), JanyError> {
    let Some(first) = words.first() else {
        return Err(JanyError::Usage("nothing to do. try: jany find log files older than 7 days".into()));
    };
    if first.is_empty() || first.contains('/') || first.starts_with('.') {
        return Err(JanyError::NoCommand(first.clone(), cmd_dir.display().to_string()));
    }
    let mut dir = cmd_dir.join(first);
    if !dir.is_dir() {
        return Err(JanyError::NoCommand(first.clone(), cmd_dir.display().to_string()));
    }
    let mut used = 1;
    for w in &words[1..] {
        // Subcommands are plain names only. Joining "/var/log" or ".." would point somewhere else.
        if w.is_empty() || w.contains('/') || w.starts_with('.') {
            break;
        }
        let sub = dir.join(w);
        if sub.is_dir() {
            dir = sub;
            used += 1;
        } else {
            break;
        }
    }
    if !dir.join("schema.toml").exists() {
        return Err(JanyError::NoCommand(words[..used].join(" "), cmd_dir.display().to_string()));
    }
    Ok((Schema::load(&dir)?, used))
}
