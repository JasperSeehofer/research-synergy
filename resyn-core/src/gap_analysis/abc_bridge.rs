#[cfg(feature = "ssr")]
use super::similarity;
#[cfg(feature = "ssr")]
use crate::datamodels::llm_annotation::LlmAnnotation;
#[cfg(feature = "ssr")]
use crate::datamodels::paper::Paper;
#[cfg(feature = "ssr")]
use petgraph::Directed;
#[cfg(feature = "ssr")]
use petgraph::algo::dijkstra;
#[cfg(feature = "ssr")]
use petgraph::prelude::NodeIndex;
#[cfg(feature = "ssr")]
use petgraph::stable_graph::StableGraph;

#[cfg(feature = "ssr")]
use crate::datamodels::analysis::PaperAnalysis;
#[cfg(feature = "ssr")]
use crate::datamodels::gap_finding::{GapFinding, GapType};
#[cfg(feature = "ssr")]
use crate::llm::gap_prompt::ABC_BRIDGE_SYSTEM_PROMPT;
#[cfg(feature = "ssr")]
use crate::llm::traits::LlmProvider;
#[cfg(feature = "ssr")]
use chrono::Utc;
#[cfg(feature = "ssr")]
use std::collections::HashMap;
#[cfg(feature = "ssr")]
use tracing::warn;

/// Minimum number of shared high-weight terms to consider a potential bridge.
#[cfg(feature = "ssr")]
const MIN_SHARED_TERMS: usize = 3;

#[cfg(feature = "ssr")]
/// Computes the shortest undirected distance between two nodes in a directed graph.
///
/// Runs dijkstra from both directions and returns the minimum found distance.
/// Returns `None` if neither direction finds a path.
fn graph_distance(
    graph: &StableGraph<Paper, f32, Directed>,
    from: NodeIndex,
    to: NodeIndex,
) -> Option<u32> {
    // Forward: from -> to
    let forward = dijkstra(graph, from, Some(to), |_| 1u32);
    let forward_dist = forward.get(&to).copied();

    // Backward: to -> from
    let backward = dijkstra(graph, to, Some(from), |_| 1u32);
    let backward_dist = backward.get(&from).copied();

    match (forward_dist, backward_dist) {
        (Some(d1), Some(d2)) => Some(d1.min(d2)),
        (Some(d), None) | (None, Some(d)) => Some(d),
        (None, None) => None,
    }
}

#[cfg(feature = "ssr")]
/// Returns true if there is a direct edge between nodes a and b in either direction.
fn has_direct_edge(graph: &StableGraph<Paper, f32, Directed>, a: NodeIndex, b: NodeIndex) -> bool {
    graph.find_edge(a, b).is_some() || graph.find_edge(b, a).is_some()
}

#[cfg(feature = "ssr")]
fn build_bridge_context(
    analysis_a: &PaperAnalysis,
    analysis_c: &PaperAnalysis,
    ann_a: Option<&LlmAnnotation>,
    ann_c: Option<&LlmAnnotation>,
    shared_terms: &[String],
) -> String {
    let open_a = ann_a.map_or_else(String::new, |a| a.open_problems.join(", "));
    let open_c = ann_c.map_or_else(String::new, |c| c.open_problems.join(", "));

    format!(
        "Paper A (arxiv:{}):\nOpen problems: {}\n\nPaper C (arxiv:{}):\nOpen problems: {}\n\nShared intermediary concepts (B): {}",
        analysis_a.arxiv_id,
        if open_a.is_empty() {
            "(none)".to_string()
        } else {
            open_a
        },
        analysis_c.arxiv_id,
        if open_c.is_empty() {
            "(none)".to_string()
        } else {
            open_c
        },
        shared_terms.join(", ")
    )
}

#[cfg(feature = "ssr")]
/// Discovers ABC-bridge connections: non-citing paper pairs (A, C) connected via
/// shared high-weight keywords through intermediary papers (B).
///
/// Filters:
/// - No direct citation between A and C (graph distance must be >= 2)
/// - Must share >= 3 high-weight terms (weight >= 0.1)
/// - LLM verification (graceful skip on failure)
///
/// `full_corpus`: if false, only considers paper pairs that appear in the citation
/// graph. If true, considers all pairs from `analyses`.
pub async fn find_abc_bridges(
    analyses: &[PaperAnalysis],
    annotations: &[LlmAnnotation],
    graph: &StableGraph<Paper, f32, Directed>,
    provider: &mut dyn LlmProvider,
    full_corpus: bool,
) -> Vec<GapFinding> {
    // Build lookup: arxiv_id -> NodeIndex
    let node_map: HashMap<String, NodeIndex> = graph
        .node_indices()
        .filter_map(|idx| {
            graph.node_weight(idx).map(|paper| {
                let id = crate::utils::strip_version_suffix(&paper.id);
                (id, idx)
            })
        })
        .collect();

    // Build annotation lookup
    let annotation_map: HashMap<&str, &LlmAnnotation> = annotations
        .iter()
        .map(|a| (a.arxiv_id.as_str(), a))
        .collect();

    let mut results = Vec::new();

    for i in 0..analyses.len() {
        for j in (i + 1)..analyses.len() {
            let a = &analyses[i];
            let c = &analyses[j];

            let node_a = node_map.get(a.arxiv_id.as_str()).copied();
            let node_c = node_map.get(c.arxiv_id.as_str()).copied();

            // If not full_corpus mode, skip pairs where neither paper is in the graph
            if !full_corpus && node_a.is_none() && node_c.is_none() {
                continue;
            }

            // Check for direct citation: skip if there's a direct edge between A and C
            if let (Some(na), Some(nc)) = (node_a, node_c) {
                if has_direct_edge(graph, na, nc) {
                    continue;
                }

                // Also skip if graph distance is 1 (same as direct edge, but belt-and-suspenders)
                if let Some(dist) = graph_distance(graph, na, nc)
                    && dist <= 1
                {
                    continue;
                }
            }

            // Shared term check: need >= MIN_SHARED_TERMS
            let shared_terms =
                similarity::shared_high_weight_terms(&a.tfidf_vector, &c.tfidf_vector, 0.1);
            if shared_terms.len() < MIN_SHARED_TERMS {
                continue;
            }

            // LLM verification
            let ann_a = annotation_map.get(a.arxiv_id.as_str()).copied();
            let ann_c = annotation_map.get(c.arxiv_id.as_str()).copied();
            let context = build_bridge_context(a, c, ann_a, ann_c, &shared_terms);

            let justification = match provider
                .verify_gap(ABC_BRIDGE_SYSTEM_PROMPT, &context)
                .await
            {
                Ok(response) => {
                    if response.trim().eq_ignore_ascii_case("NO") {
                        continue;
                    }
                    response
                }
                Err(e) => {
                    warn!(
                        paper_a = a.arxiv_id.as_str(),
                        paper_c = c.arxiv_id.as_str(),
                        error = %e,
                        "LLM verification failed for ABC-bridge candidate, skipping"
                    );
                    continue;
                }
            };

            // Confidence: normalized shared term count (per RESEARCH.md recommendation)
            let confidence = (shared_terms.len() as f32 / 10.0).min(1.0);

            results.push(GapFinding {
                gap_type: GapType::AbcBridge,
                paper_ids: vec![a.arxiv_id.clone(), c.arxiv_id.clone()],
                shared_terms,
                justification,
                confidence,
                found_at: Utc::now().to_rfc3339(),
            });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datamodels::paper::{Link, Reference};
    #[cfg(feature = "ssr")]
    use crate::llm::noop::NoopProvider;

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

    #[cfg(feature = "ssr")]
    fn make_analysis(id: &str, terms: &[(&str, f32)]) -> PaperAnalysis {
        PaperAnalysis {
            arxiv_id: id.to_string(),
            tfidf_vector: terms.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            top_terms: vec![],
            top_scores: vec![],
            analyzed_at: "2026-03-14T00:00:00Z".to_string(),
            corpus_fingerprint: "test".to_string(),
        }
    }

    fn build_test_graph(papers: &[Paper]) -> StableGraph<Paper, f32, Directed> {
        crate::data_processing::graph_creation::create_graph_from_papers(papers)
    }

    fn graph_with_edge(from_id: &str, to_id: &str) -> StableGraph<Paper, f32, Directed> {
        let papers = vec![make_paper(from_id, &[to_id]), make_paper(to_id, &[])];
        build_test_graph(&papers)
    }

    fn graph_no_edge(a_id: &str, c_id: &str) -> StableGraph<Paper, f32, Directed> {
        let papers = vec![make_paper(a_id, &[]), make_paper(c_id, &[])];
        build_test_graph(&papers)
    }

    #[cfg(feature = "ssr")]
    fn many_shared_terms(n: usize) -> Vec<(&'static str, f32)> {
        let terms = [
            "quantum",
            "entanglement",
            "decoherence",
            "spin",
            "topological",
            "lattice",
            "tensor",
            "field",
            "gauge",
            "symmetry",
        ];
        terms[..n.min(terms.len())]
            .iter()
            .map(|t| (*t, 0.5_f32))
            .collect()
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_graph_distance_connected_nodes() {
        // A -> B -> C
        let papers = vec![
            make_paper("A", &["B"]),
            make_paper("B", &["C"]),
            make_paper("C", &[]),
        ];
        let graph = build_test_graph(&papers);
        let node_a = graph.node_indices().find(|&n| graph[n].id == "A").unwrap();
        let node_c = graph.node_indices().find(|&n| graph[n].id == "C").unwrap();

        let dist = graph_distance(&graph, node_a, node_c);
        assert_eq!(dist, Some(2), "A->B->C should have distance 2");
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_graph_distance_disconnected_nodes() {
        // A and C have no path between them
        let graph = graph_no_edge("A", "C");
        let node_a = graph.node_indices().find(|&n| graph[n].id == "A").unwrap();
        let node_c = graph.node_indices().find(|&n| graph[n].id == "C").unwrap();

        let dist = graph_distance(&graph, node_a, node_c);
        assert_eq!(dist, None, "Disconnected nodes should return None");
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_has_direct_edge() {
        let graph = graph_with_edge("A", "C");
        let node_a = graph.node_indices().find(|&n| graph[n].id == "A").unwrap();
        let node_c = graph.node_indices().find(|&n| graph[n].id == "C").unwrap();

        assert!(has_direct_edge(&graph, node_a, node_c));
        assert!(
            has_direct_edge(&graph, node_c, node_a),
            "Direction reversed should also detect edge"
        );
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_find_abc_bridges_excludes_direct_citations() {
        // A directly cites C — should be excluded
        let graph = graph_with_edge("2301.11111", "2301.33333");
        let terms = many_shared_terms(5);
        let a = make_analysis("2301.11111", &terms);
        let c = make_analysis("2301.33333", &terms);
        let mut provider = NoopProvider;

        let result = find_abc_bridges(&[a, c], &[], &graph, &mut provider, true).await;
        assert!(
            result.is_empty(),
            "Direct citations should be excluded from ABC-bridge results"
        );
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_find_abc_bridges_returns_empty_when_insufficient_shared_terms() {
        // Only 2 shared terms — below MIN_SHARED_TERMS (3)
        let graph = graph_no_edge("2301.11111", "2301.33333");
        let terms_a = &[("quantum", 0.5_f32), ("spin", 0.5)];
        let terms_c = &[("quantum", 0.5_f32), ("spin", 0.5)];
        let a = make_analysis("2301.11111", terms_a);
        let c = make_analysis("2301.33333", terms_c);
        let mut provider = NoopProvider;

        let result = find_abc_bridges(&[a, c], &[], &graph, &mut provider, true).await;
        assert!(
            result.is_empty(),
            "Fewer than 3 shared terms should not produce a bridge"
        );
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_find_abc_bridges_noop_provider_returns_empty() {
        // A and C don't cite each other, have >= 3 shared terms — but NoopProvider returns NO
        let graph = graph_no_edge("2301.11111", "2301.33333");
        let terms = many_shared_terms(5);
        let a = make_analysis("2301.11111", &terms);
        let c = make_analysis("2301.33333", &terms);
        let mut provider = NoopProvider;

        let result = find_abc_bridges(&[a, c], &[], &graph, &mut provider, true).await;
        // NoopProvider returns "NO" — LLM rejects the candidate
        assert!(
            result.is_empty(),
            "NoopProvider returns NO so bridge should not be confirmed"
        );
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_find_abc_bridges_returns_empty_when_no_pairs() {
        let graph = StableGraph::<Paper, f32, Directed>::new();
        let mut provider = NoopProvider;

        let result = find_abc_bridges(&[], &[], &graph, &mut provider, false).await;
        assert!(result.is_empty());
    }
}
