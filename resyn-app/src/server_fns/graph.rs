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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Regular,
    Contradiction,
    AbcBridge,
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
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub seed_paper_id: Option<String>,
}

#[server(GetGraphData, "/api")]
pub async fn get_graph_data() -> Result<GraphData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::data_processing::graph_creation::create_graph_from_papers;
        use resyn_core::database::queries::{GapFindingRepository, PaperRepository};
        use resyn_core::datamodels::gap_finding::GapType;
        use resyn_core::petgraph::visit::{EdgeRef, IntoEdgeReferences};
        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;

        let papers = PaperRepository::new(&db)
            .get_all_papers()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let graph = create_graph_from_papers(&papers);

        let nodes: Vec<GraphNode> = graph
            .node_weights()
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
                    bfs_depth: None,
                }
            })
            .collect();

        // Build regular citation edges from petgraph
        let mut edges: Vec<GraphEdge> = Vec::new();

        for edge_ref in graph.edge_references() {
            let from = &graph[edge_ref.source()];
            let to = &graph[edge_ref.target()];
            edges.push(GraphEdge {
                from: from.id.clone(),
                to: to.id.clone(),
                edge_type: EdgeType::Regular,
                shared_terms: vec![],
                confidence: None,
                justification: None,
            });
        }

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

        Ok(GraphData { nodes, edges, seed_paper_id: None })
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
                },
                GraphNode {
                    id: "2301.22222".to_string(),
                    title: "Paper B".to_string(),
                    authors: vec!["Doe, Jane".to_string()],
                    year: "2022".to_string(),
                    citation_count: None,
                    abstract_text: "Abstract B".to_string(),
                    bfs_depth: None,
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
        };
        let json = serde_json::to_string(&node).unwrap();
        let decoded: GraphNode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, "2301.11111");
        assert_eq!(decoded.authors.len(), 2);
        assert_eq!(decoded.citation_count, Some(100));
    }
}
