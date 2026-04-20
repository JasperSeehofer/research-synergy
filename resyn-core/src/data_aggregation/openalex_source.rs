use async_trait::async_trait;

use crate::database::client::Db;
use crate::database::queries::PaperRepository;
use crate::datamodels::paper::{Paper, Reference};
use crate::error::ResynError;

use super::traits::PaperSource;

/// A `PaperSource` backed by a local OpenAlex bulk-ingest DB.
/// All lookups are in-process SurrealKV reads — no HTTP calls.
pub struct OpenAlexSource {
    db: Db,
}

impl OpenAlexSource {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PaperSource for OpenAlexSource {
    async fn fetch_paper(&self, id: &str) -> Result<Paper, ResynError> {
        let repo = PaperRepository::new(&self.db);
        repo.get_paper(id)
            .await?
            .ok_or_else(|| ResynError::PaperNotFound(id.to_string()))
    }

    async fn fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError> {
        let repo = PaperRepository::new(&self.db);
        let cited = repo.get_cited_papers(&paper.id).await?;
        paper.references = cited
            .into_iter()
            .map(|p| Reference {
                arxiv_eprint: Some(p.id),
                title: p.title,
                author: p.authors.join(", "),
                ..Default::default()
            })
            .collect();
        Ok(())
    }

    fn source_name(&self) -> &'static str {
        "openalex"
    }
}
