use serde::{Deserialize, Serialize};

/// Sentinel color_index used for the "Other" bucket (D-04).
pub const OTHER_COLOR_INDEX: u32 = u32::MAX;

/// Assignment of a single paper to a community.
/// Persisted in the `graph_communities` SurrealDB table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommunityAssignment {
    pub arxiv_id: String,
    pub community_id: u32,
    pub corpus_fingerprint: String,
}

/// A top paper within a community, ranked by hybrid score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommunityTopPaper {
    pub arxiv_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<i32>,
    pub hybrid_score: f32,
}

/// Summary of a single community for display in the UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommunitySummary {
    pub community_id: u32,
    /// Top 1-2 c-TF-IDF keywords (D-24); "Other" for the sentinel bucket.
    pub label: String,
    pub size: usize,
    /// 0 = largest non-Other community; u32::MAX = "Other" bucket.
    pub color_index: u32,
    /// Up to 5 papers ranked by hybrid score (D-20).
    pub top_papers: Vec<CommunityTopPaper>,
    /// Up to 10 c-TF-IDF distinctive keywords (D-22).
    pub dominant_keywords: Vec<(String, f32)>,
    /// Shared method terms from shared_high_weight_terms aggregated across members (D-23).
    pub shared_methods: Vec<String>,
}

/// Status of community detection computation relative to the current corpus.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CommunityStatus {
    /// true iff assignments exist for the current corpus_fingerprint.
    pub ready: bool,
    pub fingerprint: Option<String>,
    /// Number of distinct non-Other communities.
    pub count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_community_assignment_serde_roundtrip() {
        let assignment = CommunityAssignment {
            arxiv_id: "2301.12345".to_string(),
            community_id: 3,
            corpus_fingerprint: "fp_abc123".to_string(),
        };

        let json = serde_json::to_string(&assignment).expect("serialize failed");
        let recovered: CommunityAssignment =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(recovered.arxiv_id, assignment.arxiv_id);
        assert_eq!(recovered.community_id, assignment.community_id);
        assert_eq!(recovered.corpus_fingerprint, assignment.corpus_fingerprint);
        assert_eq!(recovered, assignment);
    }

    #[test]
    fn test_community_summary_serde_roundtrip() {
        let summary = CommunitySummary {
            community_id: 1,
            label: "quantum entanglement".to_string(),
            size: 12,
            color_index: 0,
            top_papers: vec![CommunityTopPaper {
                arxiv_id: "2301.12345".to_string(),
                title: "Quantum Circuits".to_string(),
                authors: vec!["Alice".to_string(), "Bob".to_string()],
                year: Some(2023),
                hybrid_score: 2.5,
            }],
            dominant_keywords: vec![
                ("quantum".to_string(), 0.9),
                ("entanglement".to_string(), 0.7),
            ],
            shared_methods: vec!["tensor networks".to_string(), "variational methods".to_string()],
        };

        let json = serde_json::to_string(&summary).expect("serialize failed");
        let recovered: CommunitySummary = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(recovered.community_id, summary.community_id);
        assert_eq!(recovered.label, summary.label);
        assert_eq!(recovered.size, summary.size);
        assert_eq!(recovered.color_index, summary.color_index);
        assert_eq!(recovered.top_papers.len(), summary.top_papers.len());
        assert_eq!(recovered.dominant_keywords.len(), summary.dominant_keywords.len());
        assert_eq!(recovered.shared_methods, summary.shared_methods);
        assert_eq!(recovered, summary);
    }
}
