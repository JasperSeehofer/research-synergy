use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub year: String,
    pub citation_count: Option<u32>,
    pub abstract_text: String,
    pub bfs_depth: Option<u32>,
    #[serde(default)]
    pub top_keywords: Vec<(String, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Regular,
    Contradiction,
    AbcBridge,
    Similarity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
    pub shared_terms: Vec<String>,
    pub confidence: Option<f32>,
    pub justification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteEntry {
    pub keyword: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub slot_index: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub seed_paper_id: Option<String>,
    #[serde(default)]
    pub palette: Vec<PaletteEntry>,
}

#[server(GetGraphData, "/api")]
pub async fn get_graph_data() -> Result<GraphData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::{
            AnalysisRepository, GapFindingRepository, PaperRepository,
        };
        use resyn_core::datamodels::gap_finding::GapType;
        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;

        let paper_repo = PaperRepository::new(&db);

        // Load all papers from DB
        let papers = paper_repo
            .get_all_papers()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // Load all TF-IDF analyses and build a fast lookup map (no N+1 queries).
        let analysis_repo = AnalysisRepository::new(&db);
        let analyses = analysis_repo
            .get_all_analyses()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        let analysis_map: std::collections::HashMap<String, Vec<(String, f32)>> = analyses
            .into_iter()
            .map(|a| {
                let keywords: Vec<(String, f32)> = a
                    .top_terms
                    .into_iter()
                    .zip(a.top_scores.into_iter())
                    .take(5)
                    .collect();
                (a.arxiv_id, keywords)
            })
            .collect();

        // Build paper ID set for fast lookup
        let paper_id_set: std::collections::HashSet<String> =
            papers.iter().map(|p| p.id.clone()).collect();

        // Query citation edges directly from DB `cites` relations.
        // Use get_citation_edges helper which returns (from_id, to_id) pairs.
        let citation_pairs = paper_repo
            .get_all_citation_edges()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // Build adjacency list for BFS depth computation
        let mut adjacency: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        let mut edge_set: Vec<(String, String)> = Vec::new();
        for (from_id, to_id) in &citation_pairs {
            if paper_id_set.contains(from_id) && paper_id_set.contains(to_id) {
                adjacency
                    .entry(from_id.clone())
                    .or_default()
                    .push(to_id.clone());
                // Also add reverse direction for undirected BFS
                adjacency
                    .entry(to_id.clone())
                    .or_default()
                    .push(from_id.clone());
                edge_set.push((from_id.clone(), to_id.clone()));
            }
        }

        // Find seed paper: the paper with the most outgoing citations (crawl root).
        // Ties broken by first in the list.
        let mut out_degree: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for (from, _) in &edge_set {
            *out_degree.entry(from.as_str()).or_insert(0) += 1;
        }
        let seed_paper_id = papers
            .iter()
            .max_by_key(|p| out_degree.get(p.id.as_str()).copied().unwrap_or(0))
            .map(|p| p.id.clone());

        // BFS depth from seed
        let mut depths: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        if let Some(ref seed_id) = seed_paper_id {
            use std::collections::VecDeque;
            let mut queue = VecDeque::new();
            queue.push_back((seed_id.clone(), 0u32));
            depths.insert(seed_id.clone(), 0);
            while let Some((node_id, depth)) = queue.pop_front() {
                if let Some(neighbors) = adjacency.get(&node_id) {
                    for neighbor in neighbors {
                        if !depths.contains_key(neighbor) {
                            depths.insert(neighbor.clone(), depth + 1);
                            queue.push_back((neighbor.clone(), depth + 1));
                        }
                    }
                }
            }
        }

        let nodes: Vec<GraphNode> = papers
            .iter()
            .map(|p| {
                let year = if p.published.len() >= 4 {
                    p.published[..4].to_string()
                } else {
                    String::new()
                };
                GraphNode {
                    id: p.id.clone(),
                    title: p.title.clone(),
                    authors: p.authors.clone(),
                    year,
                    citation_count: p.citation_count,
                    abstract_text: p.summary.clone(),
                    bfs_depth: depths.get(&p.id).copied(),
                    top_keywords: analysis_map.get(&p.id).cloned().unwrap_or_default(),
                }
            })
            .collect();

        // Build regular citation edges from DB relations
        let mut edges: Vec<GraphEdge> = edge_set
            .into_iter()
            .map(|(from, to)| GraphEdge {
                from,
                to,
                edge_type: EdgeType::Regular,
                shared_terms: vec![],
                confidence: None,
                justification: None,
            })
            .collect();

        // Overlay gap findings as special edges
        let findings = GapFindingRepository::new(&db)
            .get_all_gap_findings()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        for finding in findings {
            if finding.paper_ids.len() >= 2 {
                let edge_type = match finding.gap_type {
                    GapType::Contradiction => EdgeType::Contradiction,
                    GapType::AbcBridge => EdgeType::AbcBridge,
                };
                edges.push(GraphEdge {
                    from: finding.paper_ids[0].clone(),
                    to: finding.paper_ids[1].clone(),
                    edge_type,
                    shared_terms: finding.shared_terms,
                    confidence: Some(finding.confidence),
                    justification: Some(finding.justification),
                });
            }
        }

        // Load similarity edges from DB (D-06: fixed threshold 0.15)
        {
            use resyn_core::database::queries::SimilarityRepository;
            let sim_repo = SimilarityRepository::new(&db);
            let all_sims = sim_repo.get_all_similarities().await.unwrap_or_default();
            let node_ids: std::collections::HashSet<&str> =
                nodes.iter().map(|n| n.id.as_str()).collect();
            let threshold = 0.15_f32; // D-06: fixed minimum threshold

            for sim in &all_sims {
                if !node_ids.contains(sim.arxiv_id.as_str()) {
                    continue;
                }
                for neighbor in &sim.neighbors {
                    if neighbor.score < threshold {
                        continue;
                    }
                    if !node_ids.contains(neighbor.arxiv_id.as_str()) {
                        continue;
                    }
                    // Avoid duplicate edges (A->B and B->A): only emit if source < target lexically
                    if sim.arxiv_id < neighbor.arxiv_id {
                        edges.push(GraphEdge {
                            from: sim.arxiv_id.clone(),
                            to: neighbor.arxiv_id.clone(),
                            edge_type: EdgeType::Similarity,
                            shared_terms: neighbor.shared_terms.clone(),
                            confidence: Some(neighbor.score),
                            justification: None,
                        });
                    }
                }
            }
        }

        // Compute corpus palette from TF-IDF variance (D-01, D-04)
        let palette = {
            use resyn_core::database::queries::PaletteRepository;
            let palette_repo = PaletteRepository::new(&db);

            // D-04: Corpus fingerprint = paper count. New crawl changes paper count,
            // which invalidates the cached palette. Re-analysis alone does not change
            // paper count, so palette stays stable within a session.
            let current_fingerprint = format!("paper_count:{}", nodes.len());
            let cached_fingerprint = palette_repo.get_corpus_fingerprint().await.unwrap_or(None);

            let fingerprint_matches = cached_fingerprint
                .as_ref()
                .map(|fp| fp == &current_fingerprint)
                .unwrap_or(false);

            if fingerprint_matches {
                // Cached palette is still valid for this corpus
                let cached = palette_repo.get_palette().await.unwrap_or_default();
                cached
                    .into_iter()
                    .map(|(keyword, r, g, b, slot_index)| PaletteEntry {
                        keyword,
                        r,
                        g,
                        b,
                        slot_index,
                    })
                    .collect::<Vec<_>>()
            } else if !analysis_map.is_empty() {
                // Corpus changed (new crawl) or no cache yet — recompute
                // Find top-8 keywords by inter-paper TF-IDF variance
                let mut keyword_scores: std::collections::HashMap<String, Vec<f32>> =
                    std::collections::HashMap::new();
                for keywords in analysis_map.values() {
                    for (kw, score) in keywords {
                        keyword_scores.entry(kw.clone()).or_default().push(*score);
                    }
                }

                // Compute variance for each keyword (inter-paper variance = discriminative power)
                let mut keyword_variances: Vec<(String, f64)> = keyword_scores
                    .iter()
                    .filter(|(_, scores)| scores.len() >= 2)
                    .map(|(kw, scores)| {
                        let n = scores.len() as f64;
                        let mean = scores.iter().map(|s| *s as f64).sum::<f64>() / n;
                        let variance = scores
                            .iter()
                            .map(|s| (*s as f64 - mean).powi(2))
                            .sum::<f64>()
                            / n;
                        (kw.clone(), variance)
                    })
                    .collect();

                keyword_variances
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                // Pre-computed OKLCH palette colors (8 slots, from UI-SPEC)
                const PALETTE_COLORS: &[[u8; 3]] = &[
                    [0xe8, 0xa3, 0x4b], // slot 0: amber
                    [0xb8, 0xcc, 0x52], // slot 1: yellow-green
                    [0x56, 0xc7, 0x6b], // slot 2: green
                    [0x40, 0xc9, 0xa4], // slot 3: teal
                    [0x58, 0xb4, 0xe8], // slot 4: cyan-blue
                    [0x88, 0x84, 0xe0], // slot 5: blue-violet
                    [0xcc, 0x6f, 0xd6], // slot 6: magenta
                    [0xe0, 0x60, 0x80], // slot 7: rose
                ];

                let entries: Vec<PaletteEntry> = keyword_variances
                    .iter()
                    .take(8)
                    .enumerate()
                    .map(|(i, (kw, _))| {
                        let [r, g, b] = PALETTE_COLORS[i];
                        PaletteEntry {
                            keyword: kw.clone(),
                            r,
                            g,
                            b,
                            slot_index: i as u8,
                        }
                    })
                    .collect();

                // Cache to DB with corpus fingerprint (fire-and-forget, don't block graph load)
                let db_entries: Vec<(String, u8, u8, u8, u8, String)> = entries
                    .iter()
                    .map(|e| {
                        (
                            e.keyword.clone(),
                            e.r,
                            e.g,
                            e.b,
                            e.slot_index,
                            current_fingerprint.clone(),
                        )
                    })
                    .collect();
                let _ = palette_repo.upsert_palette(&db_entries).await;

                entries
            } else {
                Vec::new()
            }
        };

        Ok(GraphData {
            nodes,
            edges,
            seed_paper_id,
            palette,
        })
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_data_round_trips_serde() {
        let data = GraphData {
            nodes: vec![
                GraphNode {
                    id: "2301.11111".to_string(),
                    title: "Paper A".to_string(),
                    authors: vec!["Smith, John".to_string()],
                    year: "2023".to_string(),
                    citation_count: Some(42),
                    abstract_text: "Abstract A".to_string(),
                    bfs_depth: Some(1),
                    top_keywords: vec![],
                },
                GraphNode {
                    id: "2301.22222".to_string(),
                    title: "Paper B".to_string(),
                    authors: vec!["Doe, Jane".to_string()],
                    year: "2022".to_string(),
                    citation_count: None,
                    abstract_text: "Abstract B".to_string(),
                    bfs_depth: None,
                    top_keywords: vec![],
                },
            ],
            edges: vec![GraphEdge {
                from: "2301.11111".to_string(),
                to: "2301.22222".to_string(),
                edge_type: EdgeType::Regular,
                shared_terms: vec!["quantum".to_string()],
                confidence: Some(0.9),
                justification: Some("Because".to_string()),
            }],
            seed_paper_id: Some("2301.11111".to_string()),
            palette: vec![],
        };

        let json = serde_json::to_string(&data).unwrap();
        let decoded: GraphData = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.nodes.len(), 2);
        assert_eq!(decoded.edges.len(), 1);
        assert_eq!(decoded.nodes[0].id, "2301.11111");
        assert_eq!(decoded.nodes[1].citation_count, None);
        assert_eq!(decoded.edges[0].edge_type, EdgeType::Regular);
    }

    #[test]
    fn test_edge_type_contradiction_serializes() {
        let et = EdgeType::Contradiction;
        let json = serde_json::to_string(&et).unwrap();
        assert_eq!(json, "\"contradiction\"");
    }

    #[test]
    fn test_edge_type_abc_bridge_serializes() {
        let et = EdgeType::AbcBridge;
        let json = serde_json::to_string(&et).unwrap();
        assert_eq!(json, "\"abc_bridge\"");
    }

    #[test]
    fn test_graph_node_all_fields_serialize() {
        let node = GraphNode {
            id: "2301.11111".to_string(),
            title: "Test Paper".to_string(),
            authors: vec!["Author A".to_string(), "Author B".to_string()],
            year: "2023".to_string(),
            citation_count: Some(100),
            abstract_text: "This is the abstract.".to_string(),
            bfs_depth: Some(2),
            top_keywords: vec![],
        };
        let json = serde_json::to_string(&node).unwrap();
        let decoded: GraphNode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, "2301.11111");
        assert_eq!(decoded.authors.len(), 2);
        assert_eq!(decoded.citation_count, Some(100));
    }

    #[test]
    fn test_graph_node_top_keywords_serde_default() {
        // JSON without top_keywords field should deserialize with empty vec (backward compat)
        let json = r#"{"id":"2301.11111","title":"T","authors":[],"year":"2023","citation_count":null,"abstract_text":"","bfs_depth":null}"#;
        let node: GraphNode = serde_json::from_str(json).unwrap();
        assert!(
            node.top_keywords.is_empty(),
            "missing top_keywords field should default to empty vec"
        );
    }

    #[test]
    fn test_graph_node_top_keywords_round_trip() {
        let node = GraphNode {
            id: "2301.11111".to_string(),
            title: "Test".to_string(),
            authors: vec![],
            year: "2023".to_string(),
            citation_count: None,
            abstract_text: String::new(),
            bfs_depth: None,
            top_keywords: vec![("Monte Carlo".to_string(), 0.85)],
        };
        let json = serde_json::to_string(&node).unwrap();
        let decoded: GraphNode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.top_keywords.len(), 1);
        assert_eq!(decoded.top_keywords[0].0, "Monte Carlo");
        assert!((decoded.top_keywords[0].1 - 0.85).abs() < 1e-6);
    }

    #[test]
    fn test_palette_entry_serde_round_trip() {
        let entry = PaletteEntry {
            keyword: "monte_carlo".to_string(),
            r: 0x56,
            g: 0xc7,
            b: 0x6b,
            slot_index: 0,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: PaletteEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.keyword, "monte_carlo");
        assert_eq!(deserialized.r, 0x56);
    }

    #[test]
    fn test_graph_data_palette_defaults_empty() {
        // Backward compat: palette field absent in JSON should default to empty vec
        let json = r#"{"nodes":[],"edges":[],"seed_paper_id":null}"#;
        let data: GraphData = serde_json::from_str(json).unwrap();
        assert!(data.palette.is_empty());
    }

    #[test]
    fn test_palette_computation_selects_top_by_variance() {
        // Simulate analysis_map data
        let mut keyword_scores: std::collections::HashMap<String, Vec<f32>> =
            std::collections::HashMap::new();
        // "high_var" keyword: appears in 3 papers with very different scores
        keyword_scores.insert("high_var".to_string(), vec![0.9, 0.1, 0.5]);
        // "low_var" keyword: appears in 3 papers with similar scores
        keyword_scores.insert("low_var".to_string(), vec![0.5, 0.5, 0.5]);
        // "ubiquitous" keyword: appears everywhere with same score (zero variance)
        keyword_scores.insert("ubiquitous".to_string(), vec![0.3, 0.3, 0.3]);

        let mut keyword_variances: Vec<(String, f64)> = keyword_scores
            .iter()
            .filter(|(_, scores)| scores.len() >= 2)
            .map(|(kw, scores)| {
                let n = scores.len() as f64;
                let mean = scores.iter().map(|s| *s as f64).sum::<f64>() / n;
                let variance = scores
                    .iter()
                    .map(|s| (*s as f64 - mean).powi(2))
                    .sum::<f64>()
                    / n;
                (kw.clone(), variance)
            })
            .collect();
        keyword_variances
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        assert_eq!(keyword_variances[0].0, "high_var"); // Highest variance first
        assert!(keyword_variances[0].1 > keyword_variances[1].1);
    }
}
