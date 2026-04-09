use std::collections::{HashMap, VecDeque};

use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::Directed;

use crate::datamodels::paper::Paper;
use crate::utils::strip_version_suffix;

/// Compute betweenness centrality for all nodes using Brandes' algorithm.
/// Returns normalized scores in [0, 1] for directed graphs: score / ((n-1)(n-2)).
pub fn compute_betweenness(
    graph: &StableGraph<Paper, f32, Directed, u32>,
) -> HashMap<String, f32> {
    let n = graph.node_count();
    if n == 0 {
        return HashMap::new();
    }
    let nodes: Vec<NodeIndex<u32>> = graph.node_indices().collect();
    let mut centrality: HashMap<NodeIndex<u32>, f32> =
        nodes.iter().map(|&n| (n, 0.0)).collect();

    for &s in &nodes {
        let mut stack: Vec<NodeIndex<u32>> = Vec::new();
        let mut predecessors: HashMap<NodeIndex<u32>, Vec<NodeIndex<u32>>> = HashMap::new();
        let mut sigma: HashMap<NodeIndex<u32>, f32> =
            nodes.iter().map(|&n| (n, 0.0)).collect();
        *sigma.get_mut(&s).unwrap() = 1.0;
        let mut dist: HashMap<NodeIndex<u32>, i32> =
            nodes.iter().map(|&n| (n, -1)).collect();
        *dist.get_mut(&s).unwrap() = 0;
        let mut queue = VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for w in graph.neighbors(v) {
                if dist[&w] < 0 {
                    queue.push_back(w);
                    *dist.get_mut(&w).unwrap() = dist[&v] + 1;
                }
                if dist[&w] == dist[&v] + 1 {
                    *sigma.get_mut(&w).unwrap() += sigma[&v];
                    predecessors.entry(w).or_default().push(v);
                }
            }
        }

        let mut delta: HashMap<NodeIndex<u32>, f32> =
            nodes.iter().map(|&n| (n, 0.0)).collect();
        while let Some(w) = stack.pop() {
            if let Some(preds) = predecessors.get(&w) {
                for &v in preds {
                    let d = (sigma[&v] / sigma[&w]) * (1.0 + delta[&w]);
                    *delta.get_mut(&v).unwrap() += d;
                }
            }
            if w != s {
                *centrality.get_mut(&w).unwrap() += delta[&w];
            }
        }
    }

    // Normalize for directed graph: divide by (n-1)(n-2)
    let norm = if n > 2 { ((n - 1) * (n - 2)) as f32 } else { 1.0 };
    nodes
        .iter()
        .map(|&idx| {
            (
                strip_version_suffix(&graph[idx].id),
                centrality[&idx] / norm,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_processing::graph_creation::create_graph_from_papers;
    use crate::datamodels::paper::{Link, Reference};

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
    fn test_betweenness_empty_graph() {
        let graph = create_graph_from_papers(&[]);
        let scores = compute_betweenness(&graph);
        assert!(scores.is_empty(), "Empty graph should yield empty scores");
    }

    #[test]
    fn test_betweenness_disconnected_nodes_all_zero() {
        // Three nodes, no edges — betweenness should be 0 for all
        let papers = vec![
            make_paper("2301.00001", &[]),
            make_paper("2301.00002", &[]),
            make_paper("2301.00003", &[]),
        ];
        let graph = create_graph_from_papers(&papers);
        let scores = compute_betweenness(&graph);
        for (id, score) in &scores {
            assert!(
                (*score).abs() < 1e-6,
                "Disconnected node {id} should have betweenness ~0, got {score}"
            );
        }
    }

    #[test]
    fn test_betweenness_chain_middle_highest() {
        // A -> B -> C: B is on every shortest path from A to C
        let papers = vec![
            make_paper("2301.00001", &["2301.00002"]),
            make_paper("2301.00002", &["2301.00003"]),
            make_paper("2301.00003", &[]),
        ];
        let graph = create_graph_from_papers(&papers);
        let scores = compute_betweenness(&graph);

        let score_a = scores["2301.00001"];
        let score_b = scores["2301.00002"];
        let score_c = scores["2301.00003"];

        // B is on the path A->B->C, so B has highest betweenness
        assert!(
            score_b > score_a,
            "Middle node B ({score_b}) should have higher betweenness than A ({score_a})"
        );
        assert!(
            score_b > score_c,
            "Middle node B ({score_b}) should have higher betweenness than C ({score_c})"
        );
    }

    #[test]
    fn test_betweenness_normalized_range() {
        // Scores should be in [0, 1] for normalized directed betweenness
        let papers = vec![
            make_paper("2301.00001", &["2301.00002"]),
            make_paper("2301.00002", &["2301.00003"]),
            make_paper("2301.00003", &["2301.00001"]),
        ];
        let graph = create_graph_from_papers(&papers);
        let scores = compute_betweenness(&graph);

        for (id, score) in &scores {
            assert!(
                *score >= 0.0 && *score <= 1.0,
                "Score for {id} out of [0,1] range: {score}"
            );
        }
    }

    #[test]
    fn test_betweenness_normalization_constant() {
        // For n=3 directed graph: norm = (3-1)*(3-2) = 2*1 = 2
        // This verifies the (n-1)(n-2) formula is applied
        let papers = vec![
            make_paper("2301.00001", &["2301.00002"]),
            make_paper("2301.00002", &["2301.00003"]),
            make_paper("2301.00003", &[]),
        ];
        let graph = create_graph_from_papers(&papers);
        assert_eq!(graph.node_count(), 3);
        let scores = compute_betweenness(&graph);
        // Raw betweenness of B in chain: 1.0 (one path through it)
        // Normalized: 1.0 / ((3-1)*(3-2)) = 1.0/2 = 0.5
        let score_b = scores["2301.00002"];
        assert!(
            (score_b - 0.5).abs() < 1e-5,
            "Expected B betweenness ~0.5, got {score_b}"
        );
    }

    #[test]
    fn test_betweenness_two_nodes() {
        // With n=2: norm = 1 (special case, n<=2)
        let papers = vec![
            make_paper("2301.00001", &["2301.00002"]),
            make_paper("2301.00002", &[]),
        ];
        let graph = create_graph_from_papers(&papers);
        let scores = compute_betweenness(&graph);
        assert_eq!(scores.len(), 2);
        // With only 2 nodes, neither can be "between" two other nodes
        for (id, score) in &scores {
            assert!(
                (*score).abs() < 1e-6,
                "2-node graph: {id} betweenness should be 0, got {score}"
            );
        }
    }
}
