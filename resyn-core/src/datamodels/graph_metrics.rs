use serde::{Deserialize, Serialize};

/// Precomputed graph centrality metrics for a single paper.
/// Stored in the `graph_metrics` SurrealDB table.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GraphMetrics {
    pub arxiv_id: String,
    pub pagerank: f32,
    pub betweenness: f32,
    pub corpus_fingerprint: String,
    pub computed_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_metrics_serde_roundtrip() {
        let m = GraphMetrics {
            arxiv_id: "2301.12345".to_string(),
            pagerank: 0.123_456_79,
            betweenness: 42.5,
            corpus_fingerprint: "fp_abc".to_string(),
            computed_at: "2026-04-09T10:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&m).unwrap();
        let recovered: GraphMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(recovered.arxiv_id, m.arxiv_id);
        assert_eq!(recovered.corpus_fingerprint, m.corpus_fingerprint);
        assert_eq!(recovered.computed_at, m.computed_at);
        assert!((recovered.pagerank - m.pagerank).abs() < 1e-6, "pagerank mismatch");
        assert!((recovered.betweenness - m.betweenness).abs() < 1e-4, "betweenness mismatch");
    }

    #[test]
    fn test_graph_metrics_default() {
        let m = GraphMetrics::default();
        assert!(m.arxiv_id.is_empty());
        assert_eq!(m.pagerank, 0.0);
        assert_eq!(m.betweenness, 0.0);
        assert!(m.corpus_fingerprint.is_empty());
        assert!(m.computed_at.is_empty());
    }
}
