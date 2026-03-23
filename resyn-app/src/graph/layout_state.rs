use crate::server_fns::graph::{EdgeType, GraphData};

#[derive(Debug, Clone)]
pub struct NodeState {
    pub id: String,
    pub title: String,
    pub first_author: String,
    pub year: String,
    pub citation_count: u32,
    pub abstract_text: String,
    pub authors: Vec<String>,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub pinned: bool,
    pub bfs_depth: Option<u32>,
    pub lod_visible: bool,
    pub temporal_visible: bool,
}

impl NodeState {
    pub fn radius_from_citations(count: u32) -> f64 {
        (((count + 1) as f64).sqrt() * 2.5).clamp(4.0, 18.0)
    }

    pub fn label(&self) -> String {
        format!("{} {}", self.first_author, self.year)
    }
}

#[derive(Debug, Clone)]
pub struct EdgeData {
    pub from_idx: usize,
    pub to_idx: usize,
    pub edge_type: EdgeType,
    pub shared_terms: Vec<String>,
    pub confidence: Option<f32>,
    pub justification: Option<String>,
}

pub struct GraphState {
    pub nodes: Vec<NodeState>,
    pub edges: Vec<EdgeData>,
    pub velocities: Vec<(f64, f64)>,
    pub alpha: f64,
    pub selected_node: Option<usize>,
    pub hovered_node: Option<usize>,
    pub hovered_edge: Option<usize>,
    pub show_contradictions: bool,
    pub show_bridges: bool,
    pub simulation_running: bool,
    pub temporal_min_year: u32,
    pub temporal_max_year: u32,
    pub seed_paper_id: Option<String>,
    pub current_scale: f64,
}

impl GraphState {
    pub fn from_graph_data(data: GraphData) -> Self {
        let node_count = data.nodes.len();
        let spread = (node_count as f64).sqrt() * 15.0;

        // Simple deterministic hash for reproducible jitter (no rand dependency).
        fn hash_jitter(seed: u64) -> f64 {
            let h = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (h >> 33) as f64 / (u32::MAX as f64) - 0.5 // range [-0.5, 0.5]
        }

        let nodes: Vec<NodeState> = data
            .nodes
            .into_iter()
            .enumerate()
            .map(|(i, n)| {
                let angle = if node_count > 0 {
                    2.0 * std::f64::consts::PI * (i as f64) / (node_count as f64)
                } else {
                    0.0
                };
                let r = if node_count > 0 {
                    spread * (i as f64 / node_count as f64).sqrt()
                } else {
                    0.0
                };
                // Add jitter so the force simulation has asymmetry to work with.
                let jx = hash_jitter(i as u64 * 2) * spread * 0.8;
                let jy = hash_jitter(i as u64 * 2 + 1) * spread * 0.8;
                let first_author = n
                    .authors
                    .first()
                    .map(|a| a.split_whitespace().last().unwrap_or(a).to_string())
                    .unwrap_or_default();
                let year = if n.year.len() >= 4 {
                    n.year[..4].to_string()
                } else {
                    String::new()
                };
                let citation_count = n.citation_count.unwrap_or(0);
                NodeState {
                    id: n.id,
                    title: n.title,
                    first_author,
                    year,
                    citation_count,
                    abstract_text: n.abstract_text,
                    authors: n.authors,
                    x: r * angle.cos() + jx,
                    y: r * angle.sin() + jy,
                    radius: NodeState::radius_from_citations(citation_count),
                    pinned: false,
                    bfs_depth: n.bfs_depth,
                    lod_visible: true,
                    temporal_visible: true,
                }
            })
            .collect();

        // Build node id-to-index map for edge resolution
        let id_to_idx: std::collections::HashMap<&str, usize> =
            nodes.iter().enumerate().map(|(i, n)| (n.id.as_str(), i)).collect();

        let edges: Vec<EdgeData> = data
            .edges
            .into_iter()
            .filter_map(|e| {
                let from_idx = *id_to_idx.get(e.from.as_str())?;
                let to_idx = *id_to_idx.get(e.to.as_str())?;
                Some(EdgeData {
                    from_idx,
                    to_idx,
                    edge_type: e.edge_type,
                    shared_terms: e.shared_terms,
                    confidence: e.confidence,
                    justification: e.justification,
                })
            })
            .collect();

        let velocities = vec![(0.0, 0.0); nodes.len()];

        // Compute year bounds from valid years (1900–2100)
        let year_values: Vec<u32> = nodes
            .iter()
            .filter_map(|n| n.year.parse::<u32>().ok())
            .filter(|&y| y > 1900 && y < 2100)
            .collect();
        let year_min = year_values.iter().copied().min().unwrap_or(2000);
        let year_max = year_values.iter().copied().max().unwrap_or(2026);

        Self {
            nodes,
            edges,
            velocities,
            alpha: 1.0,
            selected_node: None,
            hovered_node: None,
            hovered_edge: None,
            show_contradictions: true,
            show_bridges: true,
            simulation_running: true,
            temporal_min_year: year_min,
            temporal_max_year: year_max,
            seed_paper_id: data.seed_paper_id,
            current_scale: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_fns::graph::{EdgeType, GraphEdge, GraphNode};

    fn make_node(id: &str, citation_count: Option<u32>) -> GraphNode {
        GraphNode {
            id: id.to_string(),
            title: format!("Paper {id}"),
            authors: vec!["Smith, John".to_string()],
            year: "2023".to_string(),
            citation_count,
            abstract_text: "Abstract".to_string(),
            bfs_depth: None,
        }
    }

    fn make_node_with_year(id: &str, year: &str) -> GraphNode {
        GraphNode {
            id: id.to_string(),
            title: format!("Paper {id}"),
            authors: vec!["Smith, John".to_string()],
            year: year.to_string(),
            citation_count: Some(0),
            abstract_text: "Abstract".to_string(),
            bfs_depth: None,
        }
    }

    #[test]
    fn test_node_state_bfs_depth_some_propagates() {
        let mut node = make_node("A", Some(0));
        node.bfs_depth = Some(2);
        let data = GraphData { nodes: vec![node], edges: vec![], seed_paper_id: None };
        let state = GraphState::from_graph_data(data);
        assert_eq!(state.nodes[0].bfs_depth, Some(2));
    }

    #[test]
    fn test_node_state_bfs_depth_none_propagates() {
        let node = make_node("A", Some(0));
        let data = GraphData { nodes: vec![node], edges: vec![], seed_paper_id: None };
        let state = GraphState::from_graph_data(data);
        assert_eq!(state.nodes[0].bfs_depth, None);
    }

    #[test]
    fn test_node_state_lod_visible_defaults_true() {
        let node = make_node("A", Some(0));
        let data = GraphData { nodes: vec![node], edges: vec![], seed_paper_id: None };
        let state = GraphState::from_graph_data(data);
        assert!(state.nodes[0].lod_visible);
    }

    #[test]
    fn test_node_state_temporal_visible_defaults_true() {
        let node = make_node("A", Some(0));
        let data = GraphData { nodes: vec![node], edges: vec![], seed_paper_id: None };
        let state = GraphState::from_graph_data(data);
        assert!(state.nodes[0].temporal_visible);
    }

    #[test]
    fn test_graph_state_year_bounds_computed_correctly() {
        let data = GraphData {
            nodes: vec![
                make_node_with_year("A", "2018"),
                make_node_with_year("B", "2021"),
                make_node_with_year("C", "2015"),
            ],
            edges: vec![],
            seed_paper_id: None,
        };
        let state = GraphState::from_graph_data(data);
        assert_eq!(state.temporal_min_year, 2015);
        assert_eq!(state.temporal_max_year, 2021);
    }

    #[test]
    fn test_graph_state_year_bounds_ignores_empty_years() {
        let data = GraphData {
            nodes: vec![
                make_node_with_year("A", "2018"),
                make_node_with_year("B", ""),
                make_node_with_year("C", "invalid"),
                make_node_with_year("D", "2020"),
            ],
            edges: vec![],
            seed_paper_id: None,
        };
        let state = GraphState::from_graph_data(data);
        assert_eq!(state.temporal_min_year, 2018);
        assert_eq!(state.temporal_max_year, 2020);
    }

    #[test]
    fn test_graph_state_seed_paper_id_propagates() {
        let data = GraphData {
            nodes: vec![make_node("seed-id", Some(10))],
            edges: vec![],
            seed_paper_id: Some("seed-id".to_string()),
        };
        let state = GraphState::from_graph_data(data);
        assert_eq!(state.seed_paper_id, Some("seed-id".to_string()));
    }

    #[test]
    fn test_graph_state_current_scale_initialized_to_1() {
        let data = GraphData { nodes: vec![], edges: vec![], seed_paper_id: None };
        let state = GraphState::from_graph_data(data);
        assert!((state.current_scale - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_radius_from_citations_formula() {
        // At 0 citations: sqrt(1) * 2.5 = 2.5, clamped to 4.0
        assert_eq!(NodeState::radius_from_citations(0), 4.0);

        // At 3 citations: sqrt(4) * 2.5 = 5.0
        assert!((NodeState::radius_from_citations(3) - 5.0).abs() < 1e-10);

        // At 8 citations: sqrt(9) * 2.5 = 7.5
        assert!((NodeState::radius_from_citations(8) - 7.5).abs() < 1e-10);

        // At 47 citations: sqrt(48) * 2.5 ≈ 17.32, under max
        let r47 = NodeState::radius_from_citations(47);
        assert!(r47 < 18.0);
        assert!(r47 > 4.0);

        // At very high citations: should clamp to 18.0
        let r_high = NodeState::radius_from_citations(10000);
        assert_eq!(r_high, 18.0);
    }

    #[test]
    fn test_from_graph_data_correct_node_count() {
        let data = GraphData {
            nodes: vec![
                make_node("2301.11111", Some(10)),
                make_node("2301.22222", Some(5)),
                make_node("2301.33333", None),
            ],
            edges: vec![],
            seed_paper_id: None,
        };

        let state = GraphState::from_graph_data(data);
        assert_eq!(state.nodes.len(), 3);
    }

    #[test]
    fn test_empty_graph_data_produces_empty_state() {
        let data = GraphData {
            nodes: vec![],
            edges: vec![],
            seed_paper_id: None,
        };

        let state = GraphState::from_graph_data(data);
        assert_eq!(state.nodes.len(), 0);
        assert_eq!(state.edges.len(), 0);
        assert_eq!(state.selected_node, None);
    }

    #[test]
    fn test_from_graph_data_node_radius_uses_citation_count() {
        let data = GraphData {
            nodes: vec![make_node("2301.11111", Some(0)), make_node("2301.22222", Some(3))],
            edges: vec![],
            seed_paper_id: None,
        };

        let state = GraphState::from_graph_data(data);
        assert_eq!(state.nodes[0].radius, 4.0); // clamped minimum
        assert!((state.nodes[1].radius - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_from_graph_data_edges_resolved_by_id() {
        let data = GraphData {
            nodes: vec![make_node("A", Some(0)), make_node("B", Some(0))],
            edges: vec![GraphEdge {
                from: "A".to_string(),
                to: "B".to_string(),
                edge_type: EdgeType::Regular,
                shared_terms: vec![],
                confidence: None,
                justification: None,
            }],
            seed_paper_id: None,
        };

        let state = GraphState::from_graph_data(data);
        assert_eq!(state.edges.len(), 1);
        assert_eq!(state.edges[0].from_idx, 0);
        assert_eq!(state.edges[0].to_idx, 1);
    }

    #[test]
    fn test_node_label() {
        let node = NodeState {
            id: "test".to_string(),
            title: "Title".to_string(),
            first_author: "Doe".to_string(),
            year: "2023".to_string(),
            citation_count: 0,
            abstract_text: String::new(),
            authors: vec![],
            x: 0.0,
            y: 0.0,
            radius: 4.0,
            pinned: false,
            bfs_depth: None,
            lod_visible: true,
            temporal_visible: true,
        };
        assert_eq!(node.label(), "Doe 2023");
    }
}
