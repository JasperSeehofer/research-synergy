use std::collections::HashMap;

use petgraph::algo::page_rank;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::NodeIndexable;
use petgraph::Directed;

use crate::datamodels::paper::Paper;
use crate::utils::strip_version_suffix;

/// Compute PageRank for all papers in the graph.
/// Returns a map from arxiv_id (version-stripped) to PageRank score.
/// Uses damping factor 0.85 and 50 iterations (standard values).
pub fn compute_pagerank(graph: &StableGraph<Paper, f32, Directed, u32>) -> HashMap<String, f32> {
    if graph.node_count() == 0 {
        return HashMap::new();
    }
    let ranks: Vec<f32> = page_rank(graph, 0.85_f32, 50);
    graph
        .node_indices()
        .map(|idx| {
            let pos = graph.to_index(idx);
            (strip_version_suffix(&graph[idx].id), ranks[pos])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datamodels::paper::{Link, Reference};
    use crate::data_processing::graph_creation::create_graph_from_papers;

    fn make_paper(id: &str, ref_ids: &[&str]) -> Paper {
        Paper {
            title: format!("Paper {id}"),
            id: id.to_string(),
            references: ref_ids
                .iter()
                .map(|rid| Reference {
                    links: vec![Link::from_url(&format!("https://arxiv.org/abs/{rid}"))],
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn test_pagerank_empty_graph() {
        let graph = create_graph_from_papers(&[]);
        let scores = compute_pagerank(&graph);
        assert!(scores.is_empty(), "Empty graph should yield empty scores");
    }

    #[test]
    fn test_pagerank_single_node() {
        let papers = vec![make_paper("2301.00001", &[])];
        let graph = create_graph_from_papers(&papers);
        let scores = compute_pagerank(&graph);
        assert_eq!(scores.len(), 1);
        assert!(scores.contains_key("2301.00001"), "Single node should have a score");
        let score = scores["2301.00001"];
        assert!(score > 0.0, "PageRank score must be positive, got {score}");
    }

    #[test]
    fn test_pagerank_chain_middle_node_higher() {
        // A -> B -> C: B is the "hub" between A and C
        let papers = vec![
            make_paper("2301.00001", &["2301.00002"]),
            make_paper("2301.00002", &["2301.00003"]),
            make_paper("2301.00003", &[]),
        ];
        let graph = create_graph_from_papers(&papers);
        let scores = compute_pagerank(&graph);

        assert_eq!(scores.len(), 3);
        let score_a = scores["2301.00001"];
        let score_b = scores["2301.00002"];
        let score_c = scores["2301.00003"];

        // All scores must be non-zero
        assert!(score_a > 0.0, "A score must be positive");
        assert!(score_b > 0.0, "B score must be positive");
        assert!(score_c > 0.0, "C score must be positive");

        // In a citation graph A->B->C: C receives citations from B, which receives from A.
        // C has the highest PageRank as it is at the "end" of the chain (receives more rank).
        // B has higher PageRank than A (B receives from A).
        assert!(
            score_b > score_a,
            "Middle node B ({score_b}) should have higher PageRank than source A ({score_a})"
        );
    }

    #[test]
    fn test_pagerank_all_nonzero_on_connected_chain() {
        let papers = vec![
            make_paper("2301.00001", &["2301.00002"]),
            make_paper("2301.00002", &["2301.00003"]),
            make_paper("2301.00003", &[]),
        ];
        let graph = create_graph_from_papers(&papers);
        let scores = compute_pagerank(&graph);
        for (id, score) in &scores {
            assert!(*score > 0.0, "Score for {id} should be > 0, got {score}");
        }
    }
}
