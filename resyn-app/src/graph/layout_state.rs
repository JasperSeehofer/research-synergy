use crate::server_fns::graph::{EdgeType, GraphData};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ForceMode {
    #[default]
    Citation,
    Similarity,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LabelMode {
    #[default]
    AuthorYear,
    Keywords,
    Off,
}

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
    pub is_seed: bool,
    pub top_keywords: Vec<(String, f32)>,
    pub topic_dimmed: bool,
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
    pub show_similarity: bool,
    pub show_citations: bool,
    pub force_mode: ForceMode,
    pub simulation_running: bool,
    pub temporal_min_year: u32,
    pub temporal_max_year: u32,
    pub seed_paper_id: Option<String>,
    pub current_scale: f64,
    pub label_mode: LabelMode,
    pub palette: Vec<crate::server_fns::graph::PaletteEntry>,
    /// The paper_id of the search-highlighted node (if any). Set on search result selection.
    pub search_highlighted: Option<String>,
    /// Frame number when pulse animation started. None = no pulse active.
    pub pulse_start_frame: Option<u32>,
    /// Total frames drawn (monotonically increasing counter for pulse timing).
    pub frame_counter: u32,
    /// Set of paper_ids that match the current search (for multi-match dimming per D-07).
    pub search_highlight_ids: Vec<String>,
}

impl GraphState {
    pub fn from_graph_data(data: GraphData) -> Self {
        // Simple deterministic hash for reproducible jitter (no rand dependency).
        fn hash_jitter(seed: u64) -> f64 {
            let h = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (h >> 33) as f64 / (u32::MAX as f64) - 0.5 // range [-0.5, 0.5]
        }

        // Pre-compute BFS depth groups for ring placement (D-07).
        let max_bfs_depth = data
            .nodes
            .iter()
            .filter_map(|n| n.bfs_depth)
            .max()
            .unwrap_or(0);
        let orphan_ring = max_bfs_depth + 1;

        // Count nodes at each depth for angular spacing within rings.
        let mut depth_counts: std::collections::HashMap<u32, usize> =
            std::collections::HashMap::new();
        for n in &data.nodes {
            let depth = n.bfs_depth.unwrap_or(orphan_ring);
            *depth_counts.entry(depth).or_insert(0) += 1;
        }
        // Track position within each depth ring during iteration.
        let mut depth_positions: std::collections::HashMap<u32, usize> =
            std::collections::HashMap::new();

        // Ring spacing: 1.5x IDEAL_DISTANCE (120) = 180px between depth rings.
        // Nodes start beyond equilibrium distance to produce visible spreading animation (D-04).
        let base_ring_spacing: f64 = 400.0;

        let nodes: Vec<NodeState> = data
            .nodes
            .into_iter()
            .enumerate()
            .map(|(i, n)| {
                let depth = n.bfs_depth.unwrap_or(orphan_ring);
                let pos_in_ring = *depth_positions.entry(depth).or_insert(0);
                *depth_positions.get_mut(&depth).unwrap() += 1;
                let count_at_depth = *depth_counts.get(&depth).unwrap_or(&1);

                let (x, y) = if depth == 0 {
                    // Seed node: slight offset from origin to break symmetry (D-06).
                    let jx = hash_jitter(i as u64 * 2) * 10.0;
                    let jy = hash_jitter(i as u64 * 2 + 1) * 10.0;
                    (5.0 + jx, 5.0 + jy)
                } else {
                    // Depth-N nodes: concentric ring placement (D-07).
                    let ring_radius = base_ring_spacing * depth as f64;
                    let angle =
                        2.0 * std::f64::consts::PI * (pos_in_ring as f64) / (count_at_depth as f64);
                    // 15% radial jitter to avoid perfect circle (gives force sim asymmetry).
                    let radial_jitter = ring_radius * 0.15 * hash_jitter(i as u64 * 3);
                    (
                        (ring_radius + radial_jitter) * angle.cos(),
                        (ring_radius + radial_jitter) * angle.sin(),
                    )
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
                let is_seed = data
                    .seed_paper_id
                    .as_ref()
                    .map(|sid| sid == &n.id)
                    .unwrap_or(false);
                NodeState {
                    id: n.id,
                    title: n.title,
                    first_author,
                    year,
                    citation_count,
                    abstract_text: n.abstract_text,
                    authors: n.authors,
                    x,
                    y,
                    radius: NodeState::radius_from_citations(citation_count),
                    pinned: false,
                    bfs_depth: n.bfs_depth,
                    lod_visible: true,
                    temporal_visible: true,
                    is_seed,
                    top_keywords: n.top_keywords,
                    topic_dimmed: false,
                }
            })
            .collect();

        // Build node id-to-index map for edge resolution
        let id_to_idx: std::collections::HashMap<&str, usize> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.as_str(), i))
            .collect();

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
            alpha: 0.2,
            selected_node: None,
            hovered_node: None,
            hovered_edge: None,
            show_contradictions: true,
            show_bridges: true,
            show_similarity: false,
            show_citations: true,
            force_mode: ForceMode::Citation,
            simulation_running: true,
            temporal_min_year: year_min,
            temporal_max_year: year_max,
            seed_paper_id: data.seed_paper_id,
            current_scale: 1.0,
            label_mode: LabelMode::default(),
            palette: data.palette,
            search_highlighted: None,
            pulse_start_frame: None,
            frame_counter: 0,
            search_highlight_ids: vec![],
        }
    }

    /// Check if the simulation has converged (alpha below threshold).
    /// Returns `true` if the simulation was stopped (i.e., it just converged).
    /// Called after each force tick to implement D-09 full-stop behavior.
    pub fn check_alpha_convergence(&mut self) -> bool {
        if self.alpha < resyn_worker::forces::ALPHA_MIN {
            self.simulation_running = false;
            true
        } else {
            false
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
            top_keywords: vec![],
        }
    }

    fn make_node_with_depth(id: &str, bfs_depth: Option<u32>) -> GraphNode {
        GraphNode {
            id: id.to_string(),
            title: format!("Paper {id}"),
            authors: vec!["Smith, John".to_string()],
            year: "2023".to_string(),
            citation_count: Some(0),
            abstract_text: "Abstract".to_string(),
            bfs_depth,
            top_keywords: vec![],
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
            top_keywords: vec![],
        }
    }

    #[test]
    fn test_node_state_bfs_depth_some_propagates() {
        let mut node = make_node("A", Some(0));
        node.bfs_depth = Some(2);
        let data = GraphData {
            nodes: vec![node],
            edges: vec![],
            seed_paper_id: None,
            palette: vec![],
        };
        let state = GraphState::from_graph_data(data);
        assert_eq!(state.nodes[0].bfs_depth, Some(2));
    }

    #[test]
    fn test_node_state_bfs_depth_none_propagates() {
        let node = make_node("A", Some(0));
        let data = GraphData {
            nodes: vec![node],
            edges: vec![],
            seed_paper_id: None,
            palette: vec![],
        };
        let state = GraphState::from_graph_data(data);
        assert_eq!(state.nodes[0].bfs_depth, None);
    }

    #[test]
    fn test_node_state_lod_visible_defaults_true() {
        let node = make_node("A", Some(0));
        let data = GraphData {
            nodes: vec![node],
            edges: vec![],
            seed_paper_id: None,
            palette: vec![],
        };
        let state = GraphState::from_graph_data(data);
        assert!(state.nodes[0].lod_visible);
    }

    #[test]
    fn test_node_state_temporal_visible_defaults_true() {
        let node = make_node("A", Some(0));
        let data = GraphData {
            nodes: vec![node],
            edges: vec![],
            seed_paper_id: None,
            palette: vec![],
        };
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
            palette: vec![],
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
            palette: vec![],
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
            palette: vec![],
        };
        let state = GraphState::from_graph_data(data);
        assert_eq!(state.seed_paper_id, Some("seed-id".to_string()));
    }

    #[test]
    fn test_is_seed_set_for_seed_paper() {
        let data = GraphData {
            nodes: vec![make_node("seed-id", Some(10)), make_node("other", Some(5))],
            edges: vec![],
            seed_paper_id: Some("seed-id".to_string()),
            palette: vec![],
        };
        let state = GraphState::from_graph_data(data);
        assert!(state.nodes[0].is_seed, "seed node should have is_seed=true");
        assert!(
            !state.nodes[1].is_seed,
            "non-seed node should have is_seed=false"
        );
    }

    #[test]
    fn test_is_seed_false_when_no_seed_id() {
        let data = GraphData {
            nodes: vec![make_node("A", Some(0))],
            edges: vec![],
            seed_paper_id: None,
            palette: vec![],
        };
        let state = GraphState::from_graph_data(data);
        assert!(!state.nodes[0].is_seed);
    }

    #[test]
    fn test_graph_state_current_scale_initialized_to_1() {
        let data = GraphData {
            nodes: vec![],
            edges: vec![],
            seed_paper_id: None,
            palette: vec![],
        };
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
            palette: vec![],
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
            palette: vec![],
        };

        let state = GraphState::from_graph_data(data);
        assert_eq!(state.nodes.len(), 0);
        assert_eq!(state.edges.len(), 0);
        assert_eq!(state.selected_node, None);
    }

    #[test]
    fn test_from_graph_data_node_radius_uses_citation_count() {
        let data = GraphData {
            nodes: vec![
                make_node("2301.11111", Some(0)),
                make_node("2301.22222", Some(3)),
            ],
            edges: vec![],
            seed_paper_id: None,
            palette: vec![],
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
            palette: vec![],
        };

        let state = GraphState::from_graph_data(data);
        assert_eq!(state.edges.len(), 1);
        assert_eq!(state.edges[0].from_idx, 0);
        assert_eq!(state.edges[0].to_idx, 1);
    }

    #[test]
    fn test_alpha_stops_simulation() {
        // Verify D-09: simulation_running becomes false when alpha < ALPHA_MIN.
        let data = GraphData {
            nodes: vec![make_node("A", Some(0))],
            edges: vec![],
            seed_paper_id: None,
            palette: vec![],
        };
        let mut state = GraphState::from_graph_data(data);
        assert!(state.simulation_running, "simulation should start running");

        // Alpha above threshold — simulation stays running.
        state.alpha = 0.01;
        let converged = state.check_alpha_convergence();
        assert!(!converged, "should not converge when alpha > ALPHA_MIN");
        assert!(
            state.simulation_running,
            "simulation should still be running"
        );

        // Alpha below threshold — simulation stops.
        state.alpha = 0.0005; // Below ALPHA_MIN (0.001)
        let converged = state.check_alpha_convergence();
        assert!(converged, "should converge when alpha < ALPHA_MIN");
        assert!(
            !state.simulation_running,
            "simulation_running should be false after alpha drops below ALPHA_MIN"
        );
    }

    #[test]
    fn test_from_graph_data_seed_near_origin() {
        let data = GraphData {
            nodes: vec![
                make_node_with_depth("seed", Some(0)),
                make_node_with_depth("child1", Some(1)),
                make_node_with_depth("child2", Some(1)),
            ],
            edges: vec![],
            seed_paper_id: Some("seed".to_string()),
            palette: vec![],
        };
        let state = GraphState::from_graph_data(data);
        let seed = &state.nodes[0];
        assert!(
            seed.x.abs() < 20.0 && seed.y.abs() < 20.0,
            "seed node should be near origin; x={}, y={}",
            seed.x,
            seed.y
        );
    }

    #[test]
    fn test_from_graph_data_bfs_ring_placement() {
        let data = GraphData {
            nodes: vec![
                make_node_with_depth("seed", Some(0)),
                make_node_with_depth("d1a", Some(1)),
                make_node_with_depth("d1b", Some(1)),
                make_node_with_depth("d2a", Some(2)),
            ],
            edges: vec![],
            seed_paper_id: Some("seed".to_string()),
            palette: vec![],
        };
        let state = GraphState::from_graph_data(data);
        let seed_dist = (state.nodes[0].x.powi(2) + state.nodes[0].y.powi(2)).sqrt();
        let d1a_dist = (state.nodes[1].x.powi(2) + state.nodes[1].y.powi(2)).sqrt();
        let d1b_dist = (state.nodes[2].x.powi(2) + state.nodes[2].y.powi(2)).sqrt();
        assert!(
            seed_dist < d1a_dist,
            "seed (depth-0) should be closer to origin than depth-1; seed_dist={}, d1a_dist={}",
            seed_dist,
            d1a_dist
        );
        assert!(
            seed_dist < d1b_dist,
            "seed (depth-0) should be closer to origin than depth-1; seed_dist={}, d1b_dist={}",
            seed_dist,
            d1b_dist
        );
    }

    #[test]
    fn test_from_graph_data_orphan_outer_ring() {
        let data = GraphData {
            nodes: vec![
                make_node_with_depth("seed", Some(0)),
                make_node_with_depth("d1", Some(1)),
                make_node_with_depth("orphan", None),
            ],
            edges: vec![],
            seed_paper_id: Some("seed".to_string()),
            palette: vec![],
        };
        let state = GraphState::from_graph_data(data);
        let d1_dist = (state.nodes[1].x.powi(2) + state.nodes[1].y.powi(2)).sqrt();
        let orphan_dist = (state.nodes[2].x.powi(2) + state.nodes[2].y.powi(2)).sqrt();
        assert!(
            orphan_dist > d1_dist,
            "orphan should be farther from origin than depth-1; orphan_dist={}, d1_dist={}",
            orphan_dist,
            d1_dist
        );
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
            is_seed: false,
            top_keywords: vec![],
            topic_dimmed: false,
        };
        assert_eq!(node.label(), "Doe 2023");
    }

    #[test]
    fn test_label_mode_default_is_author_year() {
        assert_eq!(LabelMode::default(), LabelMode::AuthorYear);
    }

    #[test]
    fn test_from_graph_data_populates_top_keywords() {
        let mut node = make_node("A", Some(0));
        node.top_keywords = vec![("quantum".to_string(), 0.9)];
        let data = GraphData {
            nodes: vec![node],
            edges: vec![],
            seed_paper_id: None,
            palette: vec![],
        };
        let state = GraphState::from_graph_data(data);
        assert_eq!(state.nodes[0].top_keywords.len(), 1);
        assert_eq!(state.nodes[0].top_keywords[0].0, "quantum");
        assert!((state.nodes[0].top_keywords[0].1 - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_graph_state_palette_propagation() {
        use crate::server_fns::graph::{GraphData, PaletteEntry};
        let data = GraphData {
            nodes: vec![],
            edges: vec![],
            seed_paper_id: None,
            palette: vec![PaletteEntry {
                keyword: "test".to_string(),
                r: 0x56,
                g: 0xc7,
                b: 0x6b,
                slot_index: 0,
            }],
        };
        let state = GraphState::from_graph_data(data);
        assert_eq!(state.palette.len(), 1);
        assert_eq!(state.palette[0].keyword, "test");
    }

    #[test]
    fn test_node_state_topic_dimmed_defaults_false() {
        use crate::server_fns::graph::{GraphData, GraphNode};
        let data = GraphData {
            nodes: vec![GraphNode {
                id: "test".to_string(),
                title: "Test".to_string(),
                authors: vec!["Author".to_string()],
                year: "2025".to_string(),
                citation_count: Some(5),
                abstract_text: "".to_string(),
                bfs_depth: Some(0),
                top_keywords: vec![],
            }],
            edges: vec![],
            seed_paper_id: Some("test".to_string()),
            palette: vec![],
        };
        let state = GraphState::from_graph_data(data);
        assert!(!state.nodes[0].topic_dimmed);
    }
}
