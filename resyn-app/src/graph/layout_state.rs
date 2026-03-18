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
}

impl GraphState {
    pub fn from_graph_data(data: GraphData) -> Self {
        let node_count = data.nodes.len();
        let spread = (node_count as f64).sqrt() * 50.0;

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
                    x: r * angle.cos(),
                    y: r * angle.sin(),
                    radius: NodeState::radius_from_citations(citation_count),
                    pinned: false,
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
        }
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
        };

        let state = GraphState::from_graph_data(data);
        assert_eq!(state.nodes.len(), 3);
    }

    #[test]
    fn test_empty_graph_data_produces_empty_state() {
        let data = GraphData {
            nodes: vec![],
            edges: vec![],
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
        };
        assert_eq!(node.label(), "Doe 2023");
    }
}
