use super::layout_state::NodeState;

// LOD zoom-level thresholds (configurable constants)
pub const LOD_LEVEL_0: f64 = 0.3; // seed + depth-1 only
pub const LOD_LEVEL_1: f64 = 0.6; // + high-citation (>=50)
pub const LOD_LEVEL_2: f64 = 1.0; // + medium-citation (>=10) and depth<=2
// Above 1.0: all nodes visible

pub fn update_lod_visibility(nodes: &mut [NodeState], scale: f64, seed_paper_id: &Option<String>) {
    for node in nodes.iter_mut() {
        let depth = node.bfs_depth.unwrap_or(u32::MAX);
        let cites = node.citation_count;
        let is_seed = seed_paper_id
            .as_ref()
            .map(|sid| sid == &node.id)
            .unwrap_or(false);
        node.lod_visible = if is_seed {
            true // seed always visible
        } else if scale < LOD_LEVEL_0 {
            depth <= 1
        } else if scale < LOD_LEVEL_1 {
            depth <= 1 || cites >= 50
        } else if scale < LOD_LEVEL_2 {
            depth <= 2 || cites >= 10
        } else {
            true
        };
    }
}

pub fn update_temporal_visibility(nodes: &mut [NodeState], min_year: u32, max_year: u32) {
    for node in nodes.iter_mut() {
        let year: u32 = node.year.parse().unwrap_or(0);
        node.temporal_visible = year == 0 || (year >= min_year && year <= max_year);
    }
}

pub fn compute_visible_count(nodes: &[NodeState]) -> (usize, usize) {
    let visible = nodes
        .iter()
        .filter(|n| n.lod_visible && n.temporal_visible)
        .count();
    (visible, nodes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: &str, bfs_depth: Option<u32>, citation_count: u32, year: &str) -> NodeState {
        NodeState {
            id: id.to_string(),
            title: String::new(),
            first_author: String::new(),
            year: year.to_string(),
            citation_count,
            abstract_text: String::new(),
            authors: vec![],
            x: 0.0,
            y: 0.0,
            radius: 4.0,
            target_radius: 4.0,
            current_radius: 4.0,
            pinned: false,
            bfs_depth,
            lod_visible: true,
            temporal_visible: true,
            is_seed: false,
            top_keywords: vec![],
            topic_dimmed: false,
        }
    }

    // --- LOD tests ---

    #[test]
    fn test_lod_scale_0_1_seed_node_visible() {
        // scale < LOD_LEVEL_0 (0.3): seed node is always visible
        let mut nodes = vec![make_node("seed", Some(0), 0, "2020")];
        let seed = Some("seed".to_string());
        update_lod_visibility(&mut nodes, 0.1, &seed);
        assert!(
            nodes[0].lod_visible,
            "seed node must be visible at low scale"
        );
    }

    #[test]
    fn test_lod_scale_0_1_depth_2_not_visible() {
        // scale < LOD_LEVEL_0 (0.3): depth-2 non-seed not visible
        let mut nodes = vec![make_node("deep", Some(2), 100, "2020")];
        let seed = Some("seed".to_string());
        update_lod_visibility(&mut nodes, 0.1, &seed);
        assert!(
            !nodes[0].lod_visible,
            "depth-2 node should not be visible at scale 0.1"
        );
    }

    #[test]
    fn test_lod_scale_0_1_depth_1_visible() {
        // scale < LOD_LEVEL_0 (0.3): depth-1 node is visible
        let mut nodes = vec![make_node("d1", Some(1), 0, "2020")];
        let seed = Some("seed".to_string());
        update_lod_visibility(&mut nodes, 0.1, &seed);
        assert!(
            nodes[0].lod_visible,
            "depth-1 node should be visible at scale 0.1"
        );
    }

    #[test]
    fn test_lod_scale_0_4_high_citation_visible() {
        // LOD_LEVEL_0 (0.3) <= scale < LOD_LEVEL_1 (0.6): depth-3 with >=50 citations visible
        let mut nodes = vec![make_node("hc", Some(3), 100, "2020")];
        let seed = Some("seed".to_string());
        update_lod_visibility(&mut nodes, 0.4, &seed);
        assert!(
            nodes[0].lod_visible,
            "high-citation node should be visible at scale 0.4"
        );
    }

    #[test]
    fn test_lod_scale_0_4_low_citation_deep_not_visible() {
        // LOD_LEVEL_0 (0.3) <= scale < LOD_LEVEL_1 (0.6): depth-3 with <50 citations not visible
        let mut nodes = vec![make_node("lc", Some(3), 5, "2020")];
        let seed = Some("seed".to_string());
        update_lod_visibility(&mut nodes, 0.4, &seed);
        assert!(
            !nodes[0].lod_visible,
            "low-citation depth-3 node should not be visible at scale 0.4"
        );
    }

    #[test]
    fn test_lod_scale_0_8_medium_citation_depth_2_visible() {
        // LOD_LEVEL_1 (0.6) <= scale < LOD_LEVEL_2 (1.0): depth-2 with >=10 citations visible
        let mut nodes = vec![make_node("mc", Some(2), 15, "2020")];
        let seed = Some("seed".to_string());
        update_lod_visibility(&mut nodes, 0.8, &seed);
        assert!(
            nodes[0].lod_visible,
            "medium-citation depth-2 node should be visible at scale 0.8"
        );
    }

    #[test]
    fn test_lod_scale_0_8_low_citation_deep_not_visible() {
        // LOD_LEVEL_1 (0.6) <= scale < LOD_LEVEL_2 (1.0): depth-4 with <10 citations not visible
        let mut nodes = vec![make_node("lc4", Some(4), 5, "2020")];
        let seed = Some("seed".to_string());
        update_lod_visibility(&mut nodes, 0.8, &seed);
        assert!(
            !nodes[0].lod_visible,
            "low-citation depth-4 node should not be visible at scale 0.8"
        );
    }

    #[test]
    fn test_lod_scale_1_5_all_visible() {
        // scale >= LOD_LEVEL_2 (1.0): all nodes visible regardless of depth or citations
        let mut nodes = vec![
            make_node("a", Some(5), 0, "2020"),
            make_node("b", Some(10), 1, "2018"),
            make_node("c", None, 0, "2015"),
        ];
        let seed: Option<String> = None;
        update_lod_visibility(&mut nodes, 1.5, &seed);
        assert!(
            nodes[0].lod_visible,
            "node a should be visible at scale 1.5"
        );
        assert!(
            nodes[1].lod_visible,
            "node b should be visible at scale 1.5"
        );
        assert!(
            nodes[2].lod_visible,
            "node c should be visible at scale 1.5"
        );
    }

    // --- Temporal tests ---

    #[test]
    fn test_temporal_year_in_range_visible() {
        let mut nodes = vec![make_node("a", None, 0, "2020")];
        update_temporal_visibility(&mut nodes, 2018, 2022);
        assert!(
            nodes[0].temporal_visible,
            "year 2020 should be visible in [2018, 2022]"
        );
    }

    #[test]
    fn test_temporal_year_out_of_range_not_visible() {
        let mut nodes = vec![make_node("a", None, 0, "2015")];
        update_temporal_visibility(&mut nodes, 2018, 2022);
        assert!(
            !nodes[0].temporal_visible,
            "year 2015 should not be visible in [2018, 2022]"
        );
    }

    #[test]
    fn test_temporal_unparseable_year_always_visible() {
        // Empty/unparseable year should always be visible (year == 0 => visible)
        let mut nodes = vec![make_node("a", None, 0, "")];
        update_temporal_visibility(&mut nodes, 2018, 2022);
        assert!(
            nodes[0].temporal_visible,
            "node with empty year should always be visible"
        );
    }

    // --- Visible count tests ---

    #[test]
    fn test_compute_visible_count_correct() {
        let nodes = vec![
            NodeState {
                id: "a".to_string(),
                title: String::new(),
                first_author: String::new(),
                year: "2020".to_string(),
                citation_count: 0,
                abstract_text: String::new(),
                authors: vec![],
                x: 0.0,
                y: 0.0,
                radius: 4.0,
                target_radius: 4.0,
                current_radius: 4.0,
                pinned: false,
                bfs_depth: None,
                lod_visible: true,
                temporal_visible: true,
                is_seed: false,
                top_keywords: vec![],
                topic_dimmed: false,
            },
            NodeState {
                id: "b".to_string(),
                title: String::new(),
                first_author: String::new(),
                year: "2019".to_string(),
                citation_count: 0,
                abstract_text: String::new(),
                authors: vec![],
                x: 0.0,
                y: 0.0,
                radius: 4.0,
                target_radius: 4.0,
                current_radius: 4.0,
                pinned: false,
                bfs_depth: None,
                lod_visible: false,
                temporal_visible: true,
                is_seed: false,
                top_keywords: vec![],
                topic_dimmed: false,
            },
            NodeState {
                id: "c".to_string(),
                title: String::new(),
                first_author: String::new(),
                year: "2018".to_string(),
                citation_count: 0,
                abstract_text: String::new(),
                authors: vec![],
                x: 0.0,
                y: 0.0,
                radius: 4.0,
                target_radius: 4.0,
                current_radius: 4.0,
                pinned: false,
                bfs_depth: None,
                lod_visible: true,
                temporal_visible: false,
                is_seed: false,
                top_keywords: vec![],
                topic_dimmed: false,
            },
        ];
        let (visible, total) = compute_visible_count(&nodes);
        assert_eq!(total, 3);
        assert_eq!(
            visible, 1,
            "only node 'a' has both lod_visible and temporal_visible=true"
        );
    }
}
