//! jev(TypeSafe System One)への問い合わせ。ワイヤ形式は eg-jev の
//! `packages/recipes/src/lib/questions.ts` / `openrouter.ts` に合わせている。

pub mod client;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    Choice { instructions: String, criteria: BTreeMap<String, Option<String>> },
    Noul { instructions: String },
}

pub fn choice(instructions: impl Into<String>, criteria: &[(&str, Option<&str>)]) -> Question {
    Question::Choice {
        instructions: instructions.into(),
        criteria: criteria.iter().map(|(k, v)| (k.to_string(), v.map(str::to_string))).collect(),
    }
}

pub fn noul(instructions: impl Into<String>) -> Question {
    Question::Noul { instructions: instructions.into() }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
    Choice {
        choice: String,
        #[serde(default)]
        confidence: f32,
        #[serde(default)]
        probabilities: BTreeMap<String, f32>,
    },
    Noul {
        noul: f32,
    },
    // Score は jind では使わないが、返ってきても壊れないように受ける。
    Score {
        #[serde(default)]
        score: f32,
        #[serde(default)]
        confidence: f32,
    },
}

impl Answer {
    /// 型を問わず「確からしさ」を 1 つの数にする(eg-jev の pTrue と同じ発想。Noul には confidence が無い)。
    pub fn certainty(&self) -> f32 {
        match self {
            Answer::Choice { confidence, .. } | Answer::Score { confidence, .. } => *confidence,
            Answer::Noul { noul } => noul.max(1.0 - noul),
        }
    }

    pub fn choice(&self) -> Option<&str> {
        match self {
            Answer::Choice { choice, .. } => Some(choice),
            _ => None,
        }
    }

    pub fn probabilities(&self) -> Option<&BTreeMap<String, f32>> {
        match self {
            Answer::Choice { probabilities, .. } => Some(probabilities),
            _ => None,
        }
    }

    pub fn noul(&self) -> Option<f32> {
        match self {
            Answer::Noul { noul } => Some(*noul),
            _ => None,
        }
    }

    /// `--explain` 用: 上位 2 候補。
    pub fn top2(&self) -> String {
        match self {
            Answer::Choice { probabilities, .. } => {
                let mut v: Vec<(&String, &f32)> = probabilities.iter().collect();
                v.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
                v.iter().take(2).map(|(k, p)| format!("{k} {:.2}", p)).collect::<Vec<_>>().join(", ")
            }
            Answer::Noul { noul } => format!("p(yes) {noul:.2}"),
            Answer::Score { score, .. } => format!("score {score:.2}"),
        }
    }
}

pub type Questions = BTreeMap<String, Question>;
pub type Answers = BTreeMap<String, Answer>;

#[derive(Debug, Clone, Serialize)]
pub struct DecisionsRequest {
    pub model: String,
    pub state: Value,
    pub questions: Questions,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DecisionsResponse {
    #[serde(default)]
    pub model: String,
    pub answers: Answers,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cost: Option<f64>,
}

/// 実機(client) とテストのモックを差し替えるための境界。
pub trait Oracle {
    fn decide(&self, state: Value, questions: Questions) -> Result<DecisionsResponse, crate::error::JanyError>;
}
