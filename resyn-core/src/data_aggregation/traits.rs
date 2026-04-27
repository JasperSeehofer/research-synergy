use async_trait::async_trait;

use crate::datamodels::paper::Paper;
use crate::error::ResynError;

#[async_trait]
pub trait PaperSource: Send + Sync {
    async fn fetch_paper(&self, id: &str) -> Result<Paper, ResynError>;
    async fn fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError>;
    fn source_name(&self) -> &'static str;
    /// For chained sources, returns the inner source that resolved the last fetch.
    /// Default: same as `source_name()`.
    fn last_resolving_source(&self) -> &'static str {
        self.source_name()
    }
    /// Fetch papers that cite `paper` and store them in `paper.citing_papers`.
    /// Default: no-op — only `SemanticScholarSource` overrides this in plan 01.
    async fn fetch_citing_papers(&mut self, _paper: &mut Paper) -> Result<(), ResynError> {
        Ok(())
    }
}
