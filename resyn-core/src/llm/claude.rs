use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, warn};

use crate::datamodels::llm_annotation::{Finding, LlmAnnotation, Method};
use crate::error::ResynError;
use crate::llm::prompt::{RETRY_NUDGE, SYSTEM_PROMPT};
use crate::llm::traits::LlmProvider;

// ── Response deserialization types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ClaudeResponseContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeResponseContent>,
}

// ── Intermediate annotation shape returned by the LLM ───────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct LlmAnnotationRaw {
    pub paper_type: String,
    pub methods: Vec<Method>,
    pub findings: Vec<Finding>,
    pub open_problems: Vec<String>,
}

// ── Provider ────────────────────────────────────────────────────────────────

pub struct ClaudeProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
    rate_limit: Duration,
    last_called: Option<Instant>,
}

impl ClaudeProvider {
    /// Reads `ANTHROPIC_API_KEY` from the environment.
    pub fn new(client: reqwest::Client) -> Result<Self, ResynError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| ResynError::LlmApi("ANTHROPIC_API_KEY not set".to_string()))?;
        Ok(Self {
            client,
            api_key,
            model: "claude-haiku-4-5-20250415".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            rate_limit: Duration::from_secs(1),
            last_called: None,
        })
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub fn with_rate_limit(mut self, duration: Duration) -> Self {
        self.rate_limit = duration;
        self
    }

    /// Override base URL — used in tests to point at wiremock server.
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    async fn rate_limit_check(&mut self) {
        let now = Instant::now();
        if let Some(last_call) = self.last_called {
            let elapsed = now.duration_since(last_call);
            if elapsed < self.rate_limit {
                let remaining = self.rate_limit - elapsed;
                debug!("Claude rate limit: sleeping for {:?}", remaining);
                sleep(remaining).await;
            }
        }
        self.last_called = Some(Instant::now());
    }

    async fn call_api(&self, system: &str, user_content: &str) -> Result<String, ResynError> {
        let url = format!("{}/v1/messages", self.base_url);

        #[derive(Serialize)]
        struct Message<'a> {
            role: &'a str,
            content: &'a str,
        }

        #[derive(Serialize)]
        struct RequestBody<'a> {
            model: &'a str,
            max_tokens: u32,
            system: &'a str,
            messages: Vec<Message<'a>>,
        }

        let body = RequestBody {
            model: &self.model,
            max_tokens: 1024,
            system,
            messages: vec![Message {
                role: "user",
                content: user_content,
            }],
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ResynError::LlmApi(
                "Claude API rate limit exceeded (429)".to_string(),
            ));
        }
        if !status.is_success() {
            return Err(ResynError::LlmApi(format!(
                "Claude API returned status {}",
                status.as_u16()
            )));
        }

        let resp: ClaudeResponse = response.json().await.map_err(|e| {
            ResynError::LlmApi(format!("Failed to parse Claude response envelope: {e}"))
        })?;

        let text = resp
            .content
            .into_iter()
            .find(|c| c.content_type == "text")
            .map(|c| c.text)
            .ok_or_else(|| {
                ResynError::LlmApi("Claude response contained no text block".to_string())
            })?;

        Ok(text)
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn annotate_paper(
        &mut self,
        arxiv_id: &str,
        abstract_text: &str,
    ) -> Result<LlmAnnotation, ResynError> {
        self.rate_limit_check().await;

        // First attempt
        let raw_text = self.call_api(SYSTEM_PROMPT, abstract_text).await?;
        debug!(arxiv_id, raw = raw_text.as_str(), "Claude first response");

        let raw: LlmAnnotationRaw = match serde_json::from_str(&raw_text) {
            Ok(r) => r,
            Err(first_err) => {
                warn!(
                    arxiv_id,
                    err = %first_err,
                    "Claude parse failed, retrying with nudge"
                );
                let retry_system = format!("{SYSTEM_PROMPT}\n\n{RETRY_NUDGE}");
                let retry_text = self.call_api(&retry_system, abstract_text).await?;
                debug!(arxiv_id, raw = retry_text.as_str(), "Claude retry response");
                serde_json::from_str(&retry_text).map_err(|e| {
                    ResynError::LlmApi(format!(
                        "Claude response could not be parsed after retry: {e}"
                    ))
                })?
            }
        };

        Ok(LlmAnnotation {
            arxiv_id: arxiv_id.to_string(),
            paper_type: raw.paper_type,
            methods: raw.methods,
            findings: raw.findings,
            open_problems: raw.open_problems,
            provider: "claude".to_string(),
            model_name: self.model.clone(),
            annotated_at: Utc::now().to_rfc3339(),
        })
    }

    async fn verify_gap(&mut self, prompt: &str, context: &str) -> Result<String, ResynError> {
        self.rate_limit_check().await;
        let text = self.call_api(prompt, context).await?;
        debug!(raw = text.as_str(), "Claude verify_gap response");
        Ok(text)
    }

    fn provider_name(&self) -> &'static str {
        "claude"
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const VALID_ANNOTATION_JSON: &str = r#"{
        "paper_type": "theoretical",
        "methods": [{"name": "variational method", "category": "analytical"}],
        "findings": [{"text": "Energy gap is non-zero", "strength": "strong_evidence"}],
        "open_problems": ["Extension to 3D case"]
    }"#;

    fn make_claude_response(text: &str) -> serde_json::Value {
        serde_json::json!({
            "content": [{"type": "text", "text": text}],
            "model": "claude-haiku-4-5-20250415",
            "role": "assistant"
        })
    }

    fn make_provider(server: &MockServer) -> ClaudeProvider {
        ClaudeProvider {
            client: reqwest::Client::new(),
            api_key: "test-api-key".to_string(),
            model: "claude-haiku-4-5-20250415".to_string(),
            base_url: server.uri(),
            rate_limit: Duration::from_millis(0),
            last_called: None,
        }
    }

    #[tokio::test]
    async fn test_claude_sends_correct_headers() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("x-api-key", "test-api-key"))
            .and(header("anthropic-version", "2023-06-01"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(make_claude_response(VALID_ANNOTATION_JSON)),
            )
            .mount(&server)
            .await;

        let mut provider = make_provider(&server);
        let result = provider
            .annotate_paper("2301.12345", "A test abstract.")
            .await;

        assert!(
            result.is_ok(),
            "Expected success but got: {:?}",
            result.err()
        );
        // wiremock verifies headers by refusing to match if they're absent
    }

    #[tokio::test]
    async fn test_claude_parses_valid_response() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(make_claude_response(VALID_ANNOTATION_JSON)),
            )
            .mount(&server)
            .await;

        let mut provider = make_provider(&server);
        let annotation = provider
            .annotate_paper("2301.12345", "Abstract about quantum mechanics.")
            .await
            .unwrap();

        assert_eq!(annotation.arxiv_id, "2301.12345");
        assert_eq!(annotation.paper_type, "theoretical");
        assert_eq!(annotation.methods.len(), 1);
        assert_eq!(annotation.methods[0].name, "variational method");
        assert_eq!(annotation.findings.len(), 1);
        assert_eq!(annotation.findings[0].strength, "strong_evidence");
        assert_eq!(annotation.open_problems.len(), 1);
        assert_eq!(annotation.provider, "claude");
        assert_eq!(annotation.model_name, "claude-haiku-4-5-20250415");
        assert!(!annotation.annotated_at.is_empty());
    }

    #[tokio::test]
    async fn test_claude_handles_429() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&server)
            .await;

        let mut provider = make_provider(&server);
        let result = provider
            .annotate_paper("2301.12345", "Abstract text.")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ResynError::LlmApi(msg) => {
                assert!(msg.contains("429"), "Expected 429 in message, got: {msg}")
            }
            other => panic!("Expected LlmApi error, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_claude_retries_on_parse_failure() {
        let server = MockServer::start().await;

        // First call returns non-JSON
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(make_claude_response("this is not JSON")),
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        // Second call (retry) returns valid JSON
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(make_claude_response(VALID_ANNOTATION_JSON)),
            )
            .mount(&server)
            .await;

        let mut provider = make_provider(&server);
        let annotation = provider
            .annotate_paper("2301.12345", "Abstract text.")
            .await
            .unwrap();

        assert_eq!(annotation.paper_type, "theoretical");
        assert_eq!(annotation.provider, "claude");
    }
}
