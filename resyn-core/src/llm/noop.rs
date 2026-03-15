use async_trait::async_trait;
use chrono::Utc;
use tracing::debug;

use crate::datamodels::llm_annotation::LlmAnnotation;
use crate::error::ResynError;
use crate::llm::prompt::SYSTEM_PROMPT;
use crate::llm::traits::LlmProvider;

pub struct NoopProvider;

#[async_trait]
impl LlmProvider for NoopProvider {
    async fn annotate_paper(
        &mut self,
        arxiv_id: &str,
        abstract_text: &str,
    ) -> Result<LlmAnnotation, ResynError> {
        let prompt = format!("{SYSTEM_PROMPT}\n\nPaper ID: {arxiv_id}\nAbstract: {abstract_text}");
        debug!(
            arxiv_id,
            prompt = prompt.as_str(),
            "NoopProvider constructed prompt"
        );

        Ok(LlmAnnotation {
            arxiv_id: arxiv_id.to_string(),
            paper_type: "unknown".to_string(),
            methods: vec![],
            findings: vec![],
            open_problems: vec![],
            provider: "noop".to_string(),
            model_name: "noop".to_string(),
            annotated_at: Utc::now().to_rfc3339(),
        })
    }

    async fn verify_gap(&mut self, _prompt: &str, _context: &str) -> Result<String, ResynError> {
        // Noop never confirms gaps — returns "NO" consistent with producing
        // empty-but-valid results for all operations.
        Ok("NO".to_string())
    }

    fn provider_name(&self) -> &'static str {
        "noop"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_provider_returns_valid_annotation() {
        let mut provider = NoopProvider;
        let result = provider
            .annotate_paper("2301.12345", "This paper studies quantum entanglement.")
            .await
            .unwrap();

        assert_eq!(result.arxiv_id, "2301.12345");
        assert_eq!(result.paper_type, "unknown");
        assert!(result.methods.is_empty());
        assert!(result.findings.is_empty());
        assert!(result.open_problems.is_empty());
        assert_eq!(result.provider, "noop");
        assert_eq!(result.model_name, "noop");
        assert!(!result.annotated_at.is_empty());
    }

    #[test]
    fn test_noop_provider_name() {
        let provider = NoopProvider;
        assert_eq!(provider.provider_name(), "noop");
    }
}
