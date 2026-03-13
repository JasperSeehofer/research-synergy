use async_trait::async_trait;

use crate::datamodels::paper::Paper;
use crate::error::ResynError;

#[async_trait]
pub trait PaperSource: Send + Sync {
    async fn fetch_paper(&self, id: &str) -> Result<Paper, ResynError>;
    async fn fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError>;
    fn source_name(&self) -> &'static str;
}
