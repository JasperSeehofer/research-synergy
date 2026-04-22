use async_trait::async_trait;
use std::sync::Mutex;
use tracing::warn;

use crate::datamodels::paper::Paper;
use crate::error::ResynError;

use super::traits::PaperSource;

/// Tries each inner `PaperSource` in order; returns the first success.
///
/// `last_resolving_source()` returns the `source_name()` of whichever inner source
/// last succeeded at `fetch_paper`, so the crawl orchestrator can persist it in the queue.
pub struct ChainedPaperSource {
    sources: Vec<Box<dyn PaperSource>>,
    last_resolved_idx: Mutex<usize>,
}

impl ChainedPaperSource {
    pub fn new(sources: Vec<Box<dyn PaperSource>>) -> Self {
        Self {
            sources,
            last_resolved_idx: Mutex::new(0),
        }
    }
}

#[async_trait]
impl PaperSource for ChainedPaperSource {
    async fn fetch_paper(&self, id: &str) -> Result<Paper, ResynError> {
        let mut last_err = ResynError::PaperNotFound(id.to_string());
        for (idx, source) in self.sources.iter().enumerate() {
            match source.fetch_paper(id).await {
                Ok(paper) => {
                    *self.last_resolved_idx.lock().unwrap() = idx;
                    return Ok(paper);
                }
                Err(e) => {
                    warn!(
                        source = source.source_name(),
                        paper_id = id,
                        error = %e,
                        "ChainedSource: source failed, trying next"
                    );
                    last_err = e;
                }
            }
        }
        Err(last_err)
    }

    async fn fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError> {
        let start_idx = *self.last_resolved_idx.lock().unwrap();
        let n = self.sources.len();
        let mut last_err = ResynError::PaperNotFound(paper.id.clone());
        let mut any_ok = false;
        for i in 0..n {
            let idx = (start_idx + i) % n;
            match self.sources[idx].fetch_references(paper).await {
                Ok(_) if !paper.references.is_empty() => return Ok(()),
                Ok(_) => {
                    // Source succeeded but returned no references — arXiv HTML often does
                    // this for papers that only cite journal DOIs. Try the next source.
                    any_ok = true;
                    warn!(
                        source = self.sources[idx].source_name(),
                        paper_id = paper.id.as_str(),
                        "ChainedSource: fetch_references returned empty refs, trying next source"
                    );
                }
                Err(e) => {
                    warn!(
                        source = self.sources[idx].source_name(),
                        paper_id = paper.id.as_str(),
                        error = %e,
                        "ChainedSource: fetch_references failed, trying next"
                    );
                    last_err = e;
                }
            }
        }
        // All sources exhausted. If at least one returned Ok (paper genuinely has no
        // arXiv-linked references), propagate success with empty refs rather than an error.
        if any_ok { Ok(()) } else { Err(last_err) }
    }

    fn source_name(&self) -> &'static str {
        "chain"
    }

    fn last_resolving_source(&self) -> &'static str {
        let idx = *self.last_resolved_idx.lock().unwrap();
        self.sources
            .get(idx)
            .map(|s| s.source_name())
            .unwrap_or("chain")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datamodels::paper::Paper;
    use crate::error::ResynError;
    use async_trait::async_trait;

    struct OkSource(&'static str);
    struct ErrSource(&'static str);

    #[async_trait]
    impl PaperSource for OkSource {
        async fn fetch_paper(&self, _id: &str) -> Result<Paper, ResynError> {
            let mut p = Paper::default();
            p.id = format!("resolved-by-{}", self.0);
            Ok(p)
        }
        async fn fetch_references(&mut self, _paper: &mut Paper) -> Result<(), ResynError> {
            Ok(())
        }
        fn source_name(&self) -> &'static str {
            self.0
        }
    }

    #[async_trait]
    impl PaperSource for ErrSource {
        async fn fetch_paper(&self, id: &str) -> Result<Paper, ResynError> {
            Err(ResynError::PaperNotFound(id.to_string()))
        }
        async fn fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError> {
            Err(ResynError::PaperNotFound(paper.id.clone()))
        }
        fn source_name(&self) -> &'static str {
            self.0
        }
    }

    #[tokio::test]
    async fn test_chain_first_source_ok() {
        let chain = ChainedPaperSource::new(vec![
            Box::new(OkSource("arxiv")),
            Box::new(OkSource("inspirehep")),
        ]);
        let paper = chain.fetch_paper("2301.12345").await.unwrap();
        assert_eq!(paper.id, "resolved-by-arxiv");
        assert_eq!(chain.last_resolving_source(), "arxiv");
    }

    #[tokio::test]
    async fn test_chain_fallback_to_second() {
        let chain = ChainedPaperSource::new(vec![
            Box::new(ErrSource("arxiv")),
            Box::new(OkSource("inspirehep")),
            Box::new(OkSource("semantic_scholar")),
        ]);
        let paper = chain.fetch_paper("2301.12345").await.unwrap();
        assert_eq!(paper.id, "resolved-by-inspirehep");
        assert_eq!(chain.last_resolving_source(), "inspirehep");
    }

    #[tokio::test]
    async fn test_chain_all_fail() {
        let chain = ChainedPaperSource::new(vec![
            Box::new(ErrSource("arxiv")),
            Box::new(ErrSource("inspirehep")),
        ]);
        let result = chain.fetch_paper("2301.12345").await;
        assert!(result.is_err());
    }

    struct PopulatingSource(&'static str);

    #[async_trait]
    impl PaperSource for PopulatingSource {
        async fn fetch_paper(&self, _id: &str) -> Result<Paper, ResynError> {
            Ok(Paper::default())
        }
        async fn fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError> {
            paper.references.push(crate::datamodels::paper::Reference {
                arxiv_eprint: Some("2301.99999".to_string()),
                title: "test ref".to_string(),
                ..Default::default()
            });
            Ok(())
        }
        fn source_name(&self) -> &'static str {
            self.0
        }
    }

    #[tokio::test]
    async fn test_chain_empty_refs_falls_through_to_populating_source() {
        // OkSource returns Ok() but leaves refs empty (simulates arXiv HTML with journal-only DOIs).
        // Chain should fall through to PopulatingSource.
        let mut chain = ChainedPaperSource::new(vec![
            Box::new(OkSource("arxiv")),
            Box::new(PopulatingSource("inspirehep")),
        ]);
        let mut paper = Paper::default();
        paper.id = "2301.12345".to_string();
        chain.fetch_references(&mut paper).await.unwrap();
        assert!(!paper.references.is_empty(), "should have refs from inspirehep fallback");
    }

    #[tokio::test]
    async fn test_chain_empty_refs_all_sources_returns_ok() {
        // If every source returns Ok but with empty refs, the chain should still succeed
        // (paper genuinely has no arXiv-linked references).
        let mut chain = ChainedPaperSource::new(vec![
            Box::new(OkSource("arxiv")),
            Box::new(OkSource("inspirehep")),
        ]);
        let mut paper = Paper::default();
        paper.id = "2301.12345".to_string();
        let result = chain.fetch_references(&mut paper).await;
        assert!(result.is_ok(), "should succeed even with universally empty refs");
        assert!(paper.references.is_empty());
    }
}
