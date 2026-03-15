use async_trait::async_trait;

use crate::datamodels::llm_annotation::LlmAnnotation;
use crate::error::ResynError;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn annotate_paper(
        &mut self,
        arxiv_id: &str,
        abstract_text: &str,
    ) -> Result<LlmAnnotation, ResynError>;

    /// Verify whether a potential gap (contradiction or ABC-bridge) is genuine.
    ///
    /// `prompt` is the system prompt (e.g. CONTRADICTION_SYSTEM_PROMPT or ABC_BRIDGE_SYSTEM_PROMPT).
    /// `context` is the user content describing the papers and shared terms.
    ///
    /// Returns the raw justification string from the LLM, or "NO" if no real gap exists.
    /// No JSON parsing — plain text response per RESEARCH.md spec.
    async fn verify_gap(&mut self, prompt: &str, context: &str) -> Result<String, ResynError>;

    fn provider_name(&self) -> &'static str;
}
