use serde::{Deserialize, Serialize};

/// A single similar paper neighbor with its similarity score and shared terms.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SimilarNeighbor {
    pub arxiv_id: String,
    pub score: f32,
    pub shared_terms: Vec<String>,
}

/// Precomputed top-K similar papers for a given paper.
/// Stored in the `paper_similarity` SurrealDB table.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaperSimilarity {
    pub arxiv_id: String,
    pub neighbors: Vec<SimilarNeighbor>,
    pub corpus_fingerprint: String,
    pub computed_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_similarity_serde_roundtrip() {
        let sim = PaperSimilarity {
            arxiv_id: "2301.12345".to_string(),
            neighbors: vec![
                SimilarNeighbor {
                    arxiv_id: "2301.99999".to_string(),
                    score: 0.85,
                    shared_terms: vec!["quantum".to_string(), "entanglement".to_string()],
                },
                SimilarNeighbor {
                    arxiv_id: "2301.11111".to_string(),
                    score: 0.72,
                    shared_terms: vec!["topology".to_string()],
                },
            ],
            corpus_fingerprint: "abc123".to_string(),
            computed_at: "2026-04-08T10:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&sim).unwrap();
        let recovered: PaperSimilarity = serde_json::from_str(&json).unwrap();

        assert_eq!(recovered.arxiv_id, sim.arxiv_id);
        assert_eq!(recovered.corpus_fingerprint, sim.corpus_fingerprint);
        assert_eq!(recovered.computed_at, sim.computed_at);
        assert_eq!(recovered.neighbors.len(), sim.neighbors.len());
        for (a, b) in recovered.neighbors.iter().zip(sim.neighbors.iter()) {
            assert_eq!(a.arxiv_id, b.arxiv_id);
            assert!((a.score - b.score).abs() < 1e-6);
            assert_eq!(a.shared_terms, b.shared_terms);
        }
    }

    #[test]
    fn test_similar_neighbor_default() {
        let n = SimilarNeighbor::default();
        assert!(n.arxiv_id.is_empty());
        assert_eq!(n.score, 0.0);
        assert!(n.shared_terms.is_empty());
    }
}
