use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, warn};

use crate::datamodels::llm_annotation::LlmAnnotation;
use crate::error::ResynError;
use crate::llm::claude::LlmAnnotationRaw;
use crate::llm::prompt::{LLM_ANNOTATION_SCHEMA, RETRY_NUDGE, SYSTEM_PROMPT};
use crate::llm::traits::LlmProvider;

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
    rate_limit: Duration,
    last_called: Option<Instant>,
}

impl OllamaProvider {
    pub fn new(client: reqwest::Client) -> Self {
        let base_url =
            std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        Self {
            client,
            base_url,
            model: "llama3.2".to_string(),
            rate_limit: Duration::from_millis(350),
            last_called: None,
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub fn with_rate_limit(mut self, duration: Duration) -> Self {
        self.rate_limit = duration;
        self
    }

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
                debug!("Ollama rate limit: sleeping for {:?}", remaining);
                sleep(remaining).await;
            }
        }
        self.last_called = Some(Instant::now());
    }

    async fn call_api(&self, system: &str, user_content: &str) -> Result<String, ResynError> {
        let url = format!("{}/api/chat", self.base_url);

        #[derive(Serialize)]
        struct OllamaMessage<'a> {
            role: &'a str,
            content: &'a str,
        }

        #[derive(Serialize)]
        struct RequestBody<'a> {
            model: &'a str,
            messages: Vec<OllamaMessage<'a>>,
            stream: bool,
            format: serde_json::Value,
            options: serde_json::Value,
        }

        let format_schema: serde_json::Value = serde_json::from_str(LLM_ANNOTATION_SCHEMA)
            .expect("LLM_ANNOTATION_SCHEMA is valid JSON");

        let body = RequestBody {
            model: &self.model,
            messages: vec![
                OllamaMessage {
                    role: "system",
                    content: system,
                },
                OllamaMessage {
                    role: "user",
                    content: user_content,
                },
            ],
            stream: false,
            format: format_schema,
            options: serde_json::json!({"temperature": 0}),
        };

        let response = self.client.post(&url).json(&body).send().await?;

        let status = response.status();
        if !status.is_success() {
            return Err(ResynError::LlmApi(format!(
                "Ollama returned status {}",
                status.as_u16()
            )));
        }

        let resp: OllamaResponse = response
            .json()
            .await
            .map_err(|e| ResynError::LlmApi(format!("Failed to parse Ollama response: {e}")))?;

        Ok(resp.message.content)
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn annotate_paper(
        &mut self,
        arxiv_id: &str,
        abstract_text: &str,
    ) -> Result<LlmAnnotation, ResynError> {
        self.rate_limit_check().await;

        let raw_text = self.call_api(SYSTEM_PROMPT, abstract_text).await?;
        debug!(arxiv_id, raw = raw_text.as_str(), "Ollama first response");

        let raw: LlmAnnotationRaw = match serde_json::from_str(&raw_text) {
            Ok(r) => r,
            Err(first_err) => {
                warn!(
                    arxiv_id,
                    err = %first_err,
                    "Ollama parse failed, retrying with nudge"
                );
                let retry_system = format!("{SYSTEM_PROMPT}\n\n{RETRY_NUDGE}");
                let retry_text = self.call_api(&retry_system, abstract_text).await?;
                debug!(arxiv_id, raw = retry_text.as_str(), "Ollama retry response");
                serde_json::from_str(&retry_text).map_err(|e| {
                    ResynError::LlmApi(format!(
                        "Ollama response could not be parsed after retry: {e}"
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
            provider: "ollama".to_string(),
            model_name: self.model.clone(),
            annotated_at: Utc::now().to_rfc3339(),
        })
    }

    async fn verify_gap(&mut self, prompt: &str, context: &str) -> Result<String, ResynError> {
        self.rate_limit_check().await;
        let text = self.call_api(prompt, context).await?;
        debug!(raw = text.as_str(), "Ollama verify_gap response");
        Ok(text)
    }

    fn provider_name(&self) -> &'static str {
        "ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const VALID_ANNOTATION_JSON: &str = r#"{
        "paper_type": "computational",
        "methods": [{"name": "Monte Carlo", "category": "computational"}],
        "findings": [{"text": "Simulation converges", "strength": "moderate_evidence"}],
        "open_problems": ["Scaling to larger systems"]
    }"#;

    fn make_ollama_response(content: &str) -> serde_json::Value {
        serde_json::json!({
            "message": {"role": "assistant", "content": content},
            "done": true
        })
    }

    fn make_provider(server: &MockServer) -> OllamaProvider {
        OllamaProvider {
            client: reqwest::Client::new(),
            base_url: server.uri(),
            model: "llama3.2".to_string(),
            rate_limit: Duration::from_millis(0),
            last_called: None,
        }
    }

    #[tokio::test]
    async fn test_ollama_sends_stream_false() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(make_ollama_response(VALID_ANNOTATION_JSON)),
            )
            .mount(&server)
            .await;

        let mut provider = make_provider(&server);
        let result = provider
            .annotate_paper("2301.12345", "Abstract text.")
            .await;

        assert!(result.is_ok(), "Expected success: {:?}", result.err());

        // Verify via request log that stream:false was sent
        let received = server.received_requests().await.unwrap();
        assert_eq!(received.len(), 1);
        let body: serde_json::Value =
            serde_json::from_slice(&received[0].body).expect("body is JSON");
        assert_eq!(body["stream"], serde_json::json!(false));
    }

    #[tokio::test]
    async fn test_ollama_sends_format_schema() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(make_ollama_response(VALID_ANNOTATION_JSON)),
            )
            .mount(&server)
            .await;

        let mut provider = make_provider(&server);
        provider
            .annotate_paper("2301.12345", "Abstract text.")
            .await
            .unwrap();

        let received = server.received_requests().await.unwrap();
        let body: serde_json::Value =
            serde_json::from_slice(&received[0].body).expect("body is JSON");
        let format = &body["format"];
        assert!(format.is_object(), "format field should be a JSON object");
        assert_eq!(
            format["type"], "object",
            "format schema should have type=object"
        );
    }

    #[tokio::test]
    async fn test_ollama_parses_valid_response() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(make_ollama_response(VALID_ANNOTATION_JSON)),
            )
            .mount(&server)
            .await;

        let mut provider = make_provider(&server);
        let annotation = provider
            .annotate_paper("2301.12345", "Abstract.")
            .await
            .unwrap();

        assert_eq!(annotation.arxiv_id, "2301.12345");
        assert_eq!(annotation.paper_type, "computational");
        assert_eq!(annotation.methods.len(), 1);
        assert_eq!(annotation.methods[0].name, "Monte Carlo");
        assert_eq!(annotation.findings.len(), 1);
        assert_eq!(annotation.provider, "ollama");
        assert_eq!(annotation.model_name, "llama3.2");
    }

    #[tokio::test]
    async fn test_ollama_retries_on_parse_failure() {
        let server = MockServer::start().await;

        // First call: malformed content
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(make_ollama_response("not valid json")),
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        // Second call (retry): valid JSON
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(make_ollama_response(VALID_ANNOTATION_JSON)),
            )
            .mount(&server)
            .await;

        let mut provider = make_provider(&server);
        let annotation = provider
            .annotate_paper("2301.12345", "Abstract.")
            .await
            .unwrap();

        assert_eq!(annotation.paper_type, "computational");
        assert_eq!(annotation.provider, "ollama");
    }
}
