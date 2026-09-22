//! OpenRouter の Decisions router: POST https://openrouter.ai/api/alpha/decisions
//! (eg-jev `openrouter.ts` の decide() と同じ body / headers / 再試行)。

use super::{DecisionsRequest, DecisionsResponse, Oracle, Questions};
use crate::error::JxError;
use serde_json::Value;
use std::time::Duration;

pub const ENDPOINT: &str = "https://openrouter.ai/api/alpha/decisions";
pub const DEFAULT_MODEL: &str = "typesafe/jev-1.13";

pub struct OpenRouter {
    pub api_key: String,
    pub model: String,
    pub timeout: Duration,
    pub max_retries: u32,
}

impl Oracle for OpenRouter {
    fn decide(&self, state: Value, questions: Questions) -> Result<DecisionsResponse, JxError> {
        let body = DecisionsRequest { model: self.model.clone(), state, questions };
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(self.timeout))
            .http_status_as_error(false)
            .build()
            .into();
        let mut attempt = 0;
        loop {
            let res = agent
                .post(ENDPOINT)
                .header("Authorization", &format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .header("HTTP-Referer", "https://github.com/yukihirop/jx")
                .header("X-Title", "jx")
                .send_json(&body);
            let mut res = match res {
                Ok(r) => r,
                Err(e) => return Err(JxError::Jev(format!("request failed: {e}"))),
            };
            let status = res.status().as_u16();
            if (200..300).contains(&status) {
                return res
                    .body_mut()
                    .read_json::<DecisionsResponse>()
                    .map_err(|e| JxError::Jev(format!("bad response: {e}")));
            }
            let retryable = status == 429 || status >= 500;
            let text = res.body_mut().read_to_string().unwrap_or_default();
            if !retryable || attempt >= self.max_retries {
                return Err(JxError::Jev(format!("HTTP {status}: {}", text.chars().take(500).collect::<String>())));
            }
            let delay = Duration::from_millis(500 * 2u64.pow(attempt));
            attempt += 1;
            std::thread::sleep(delay);
        }
    }
}
