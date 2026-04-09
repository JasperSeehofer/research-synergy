use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// A single entry in the "Similar Papers" tab list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarPaperEntry {
    pub arxiv_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub year: String,
    pub score: f32,
    pub shared_terms: Vec<String>,
}

/// Return the top similar papers for a given arXiv ID.
///
/// Returns an empty vec when no similarity data has been computed yet (D-08 spinner state).
#[server(GetSimilarPapers, "/api")]
pub async fn get_similar_papers(arxiv_id: String) -> Result<Vec<SimilarPaperEntry>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::{PaperRepository, SimilarityRepository};
        use resyn_core::utils::strip_version_suffix;
        use std::sync::Arc;

        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;

        // T-22-03: strip version suffix before DB query (prevents injection via versioned IDs)
        let clean_id = strip_version_suffix(&arxiv_id);
        let sim_repo = SimilarityRepository::new(&db);
        let paper_repo = PaperRepository::new(&db);

        let sim = sim_repo
            .get_similarity(&clean_id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        match sim {
            // No similarity data yet — triggers D-08 spinner in UI
            None => Ok(vec![]),
            Some(paper_sim) => {
                let mut entries = Vec::new();
                for neighbor in &paper_sim.neighbors {
                    // Fetch paper metadata for each neighbor
                    if let Ok(Some(paper)) = paper_repo.get_paper(&neighbor.arxiv_id).await {
                        let year = if paper.published.len() >= 4 {
                            paper.published[..4].to_string()
                        } else {
                            String::new()
                        };
                        entries.push(SimilarPaperEntry {
                            arxiv_id: neighbor.arxiv_id.clone(),
                            title: paper.title.clone(),
                            authors: paper.authors.clone(),
                            year,
                            score: neighbor.score,
                            shared_terms: neighbor.shared_terms.clone(),
                        });
                    }
                }
                Ok(entries)
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
