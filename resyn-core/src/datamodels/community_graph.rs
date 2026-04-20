use serde::{Deserialize, Serialize};

/// Parameters used for the Louvain run that produced the community assignments.
/// Embedded in the export for reproducibility verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LouvainParams {
    pub seed: u64,
    pub resolution: f64,
    pub min_community_size: usize,
}

/// A single paper node in the exported community graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportedNode {
    /// arXiv paper ID (version suffix stripped).
    pub id: String,
    /// Louvain community ID. The "Other" bucket (u32::MAX - 1) is excluded from exports.
    pub community_id: u32,
    /// TF-IDF terms sorted by score descending, truncated to `tfidf_top_n`.
    pub tfidf_vec: Vec<(String, f32)>,
}

/// A single directed citation edge in the exported graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportedEdge {
    /// arXiv ID of the citing paper.
    pub src: String,
    /// arXiv ID of the cited paper.
    pub dst: String,
    /// Edge weight (currently 1.0 — uniform coupling for Kuramoto v03).
    pub weight: f32,
}

/// Per-community c-TF-IDF profile for the sheaf-LBD prototype (EXP-RS-07).
/// Not used by Kuramoto-LBD (ignored by the Python loader there).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportedCommunity {
    pub community_id: u32,
    /// Number of papers assigned to this community (in-scope only).
    pub size: usize,
    /// c-TF-IDF terms sorted by score descending, truncated to `tfidf_top_n`.
    pub tfidf_vec: Vec<(String, f32)>,
}

/// Exported Louvain community graph for external consumption (e.g. Kuramoto-LBD notebook).
///
/// Schema: `{louvain_params, corpus_fingerprint, nodes: [{id, community_id, tfidf_vec}], communities: [{community_id, size, tfidf_vec}], edges: [{src, dst, weight}]}`
///
/// "Other" communities are excluded. Only papers with both a community assignment and a
/// TF-IDF vector are included as nodes. Edges are included only when both endpoints are
/// in the node set and (when `published_before` is set) both pass the date filter.
/// The `communities` field carries per-community c-TF-IDF vectors for sheaf-LBD (EXP-RS-07);
/// older consumers (Kuramoto v03) ignore it via serde default.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommunityGraph {
    pub louvain_params: LouvainParams,
    /// Corpus fingerprint of the Louvain run (hash of sorted arXiv IDs).
    pub corpus_fingerprint: String,
    pub nodes: Vec<ExportedNode>,
    /// Per-community c-TF-IDF profiles, sorted by community_id ascending.
    /// Missing in exports produced before EXP-RS-07; deserializes as empty vec.
    #[serde(default)]
    pub communities: Vec<ExportedCommunity>,
    pub edges: Vec<ExportedEdge>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_community_graph_serde_roundtrip() {
        let graph = CommunityGraph {
            louvain_params: LouvainParams {
                seed: 42,
                resolution: 1.0,
                min_community_size: 3,
            },
            corpus_fingerprint: "abc123".to_string(),
            nodes: vec![
                ExportedNode {
                    id: "2301.12345".to_string(),
                    community_id: 0,
                    tfidf_vec: vec![("quantum".to_string(), 0.9), ("spin".to_string(), 0.5)],
                },
                ExportedNode {
                    id: "2301.99999".to_string(),
                    community_id: 1,
                    tfidf_vec: vec![("neural".to_string(), 0.8)],
                },
            ],
            communities: vec![
                ExportedCommunity {
                    community_id: 0,
                    size: 1,
                    tfidf_vec: vec![("quantum".to_string(), 1.5), ("spin".to_string(), 0.8)],
                },
                ExportedCommunity {
                    community_id: 1,
                    size: 1,
                    tfidf_vec: vec![("neural".to_string(), 1.2)],
                },
            ],
            edges: vec![ExportedEdge {
                src: "2301.12345".to_string(),
                dst: "2301.99999".to_string(),
                weight: 1.0,
            }],
        };
        let json = serde_json::to_string(&graph).unwrap();
        let recovered: CommunityGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered, graph);
    }

    #[test]
    fn test_empty_community_graph_serializes() {
        let graph = CommunityGraph {
            louvain_params: LouvainParams {
                seed: 42,
                resolution: 1.0,
                min_community_size: 3,
            },
            corpus_fingerprint: String::new(),
            nodes: vec![],
            communities: vec![],
            edges: vec![],
        };
        let json = serde_json::to_string(&graph).unwrap();
        let recovered: CommunityGraph = serde_json::from_str(&json).unwrap();
        assert!(recovered.nodes.is_empty());
        assert!(recovered.communities.is_empty());
        assert!(recovered.edges.is_empty());
    }

    #[test]
    fn test_community_graph_backward_compat_no_communities_field() {
        // Old JSON produced before EXP-RS-07 has no "communities" key.
        // serde(default) must make it deserialize as empty vec.
        let old_json = r#"{
            "louvain_params": {"seed": 42, "resolution": 1.0, "min_community_size": 3},
            "corpus_fingerprint": "abc123",
            "nodes": [],
            "edges": []
        }"#;
        let recovered: CommunityGraph = serde_json::from_str(old_json).unwrap();
        assert!(
            recovered.communities.is_empty(),
            "missing 'communities' key should deserialize as empty vec"
        );
    }
}
