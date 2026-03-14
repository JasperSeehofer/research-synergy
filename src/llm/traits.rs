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

    fn provider_name(&self) -> &'static str;
}
