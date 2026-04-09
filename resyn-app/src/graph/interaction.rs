use super::layout_state::{EdgeData, NodeState};
use super::renderer::Viewport;

/// Find the topmost node at world coordinates (wx, wy).
/// Iterates nodes in reverse order so later nodes (drawn on top) take priority.
/// Returns Some(index) if cursor is within node's radius, None otherwise.
pub fn find_node_at(nodes: &[NodeState], wx: f64, wy: f64) -> Option<usize> {
    nodes.iter().enumerate().rev().find_map(|(i, n)| {
        let dx = n.x - wx;
        let dy = n.y - wy;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq <= n.radius * n.radius {
            Some(i)
        } else {
            None
        }
    })
}

/// Find the first edge whose line segment is within `threshold` world-space pixels
/// of the point (wx, wy). Returns Some(edge index) or None.
pub fn find_edge_at(
    nodes: &[NodeState],
    edges: &[EdgeData],
    wx: f64,
    wy: f64,
    threshold: f64,
) -> Option<usize> {
    edges.iter().enumerate().find_map(|(i, e)| {
        let (x1, y1) = (nodes[e.from_idx].x, nodes[e.from_idx].y);
        let (x2, y2) = (nodes[e.to_idx].x, nodes[e.to_idx].y);
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len_sq = dx * dx + dy * dy;

        let dist = if len_sq == 0.0 {
            let ddx = wx - x1;
            let ddy = wy - y1;
            (ddx * ddx + ddy * ddy).sqrt()
        } else {
            let t = ((wx - x1) * dx + (wy - y1) * dy) / len_sq;
            let t = t.clamp(0.0, 1.0);
            let px = x1 + t * dx;
            let py = y1 + t * dy;
            let ddx = wx - px;
            let ddy = wy - py;
            (ddx * ddx + ddy * ddy).sqrt()
        };

        if dist <= threshold { Some(i) } else { None }
    })
}

/// Zoom the viewport toward the cursor position (cx, cy) in screen space.
/// delta < 0 zooms in (factor 1.1), delta >= 0 zooms out (factor 0.9).
/// Scale is clamped to [0.1, 4.0].
pub fn zoom_toward_cursor(viewport: &mut Viewport, cx: f64, cy: f64, delta: f64) {
    let factor = if delta < 0.0 { 1.1 } else { 0.9 };
    viewport.offset_x = cx - factor * (cx - viewport.offset_x);
    viewport.offset_y = cy - factor * (cy - viewport.offset_y);
    viewport.scale = (viewport.scale * factor).clamp(0.1, 4.0);
}

/// State machine for mouse/pointer interactions.
#[derive(Debug, Clone)]
pub enum InteractionState {
    Idle,
    Panning {
        start_x: f64,
        start_y: f64,
        start_offset_x: f64,
        start_offset_y: f64,
    },
    DraggingNode {
        node_idx: usize,
        offset_x: f64,
        offset_y: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_fns::graph::EdgeType;

    fn make_node(x: f64, y: f64, radius: f64) -> NodeState {
        NodeState {
            id: String::new(),
            title: String::new(),
            first_author: String::new(),
            year: String::new(),
            citation_count: 0,
            abstract_text: String::new(),
            authors: vec![],
            x,
            y,
            radius,
            target_radius: radius,
            current_radius: radius,
            pinned: false,
            bfs_depth: None,
            lod_visible: true,
            temporal_visible: true,
            is_seed: false,
            top_keywords: vec![],
            topic_dimmed: false,
        }
    }

    fn make_edge(from_idx: usize, to_idx: usize) -> EdgeData {
        EdgeData {
            from_idx,
            to_idx,
            edge_type: EdgeType::Regular,
            shared_terms: vec![],
            confidence: None,
            justification: None,
        }
    }

    // ---- screen_to_world / world_to_screen tests ----

    #[test]
    fn test_screen_to_world_at_center() {
        let vp = Viewport {
            offset_x: 400.0,
            offset_y: 300.0,
            scale: 1.0,
            css_width: 800.0,
            css_height: 600.0,
        };
        let (wx, wy) = vp.screen_to_world(400.0, 300.0);
        assert!((wx).abs() < 1e-10, "expected 0, got {wx}");
        assert!((wy).abs() < 1e-10, "expected 0, got {wy}");
    }

    #[test]
    fn test_screen_to_world_scale2() {
        let vp = Viewport {
            offset_x: 400.0,
            offset_y: 300.0,
            scale: 2.0,
            css_width: 800.0,
            css_height: 600.0,
        };
        let (wx, wy) = vp.screen_to_world(500.0, 400.0);
        assert!((wx - 50.0).abs() < 1e-10, "expected 50, got {wx}");
        assert!((wy - 50.0).abs() < 1e-10, "expected 50, got {wy}");
    }

    #[test]
    fn test_world_to_screen_round_trip() {
        let vp = Viewport {
            offset_x: 200.0,
            offset_y: 150.0,
            scale: 0.75,
            css_width: 400.0,
            css_height: 300.0,
        };
        for (wx, wy) in [
            (0.0, 0.0),
            (100.0, -50.0),
            (-200.0, 300.0),
            (1000.0, 1000.0),
        ] {
            let (sx, sy) = vp.world_to_screen(wx, wy);
            let (rx, ry) = vp.screen_to_world(sx, sy);
            assert!((rx - wx).abs() < 1e-9, "round-trip x failed: {wx} -> {rx}");
            assert!((ry - wy).abs() < 1e-9, "round-trip y failed: {wy} -> {ry}");
        }
    }

    // ---- find_node_at tests ----

    #[test]
    fn test_find_node_at_inside_radius() {
        let nodes = vec![make_node(0.0, 0.0, 10.0)];
        assert_eq!(find_node_at(&nodes, 0.0, 0.0), Some(0));
        assert_eq!(find_node_at(&nodes, 9.9, 0.0), Some(0));
    }

    #[test]
    fn test_find_node_at_outside_radius() {
        let nodes = vec![make_node(0.0, 0.0, 10.0)];
        assert_eq!(find_node_at(&nodes, 10.1, 0.0), None);
        assert_eq!(find_node_at(&nodes, 100.0, 100.0), None);
    }

    #[test]
    fn test_find_node_at_topmost_wins_overlap() {
        // Two overlapping nodes; last one (index 1) is "drawn on top"
        let nodes = vec![
            make_node(0.0, 0.0, 20.0), // index 0
            make_node(5.0, 0.0, 20.0), // index 1 — on top
        ];
        // Point (2.5, 0) is inside both; last drawn (index 1) wins
        let result = find_node_at(&nodes, 2.5, 0.0);
        assert_eq!(result, Some(1));
    }

    // ---- find_edge_at tests ----

    #[test]
    fn test_find_edge_at_on_segment() {
        let nodes = vec![make_node(0.0, 0.0, 5.0), make_node(100.0, 0.0, 5.0)];
        let edges = vec![make_edge(0, 1)];
        // Midpoint of the segment
        let result = find_edge_at(&nodes, &edges, 50.0, 0.0, 4.0);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_find_edge_at_far_from_segment() {
        let nodes = vec![make_node(0.0, 0.0, 5.0), make_node(100.0, 0.0, 5.0)];
        let edges = vec![make_edge(0, 1)];
        let result = find_edge_at(&nodes, &edges, 50.0, 50.0, 4.0);
        assert_eq!(result, None);
    }

    // ---- zoom_toward_cursor tests ----

    #[test]
    fn test_zoom_toward_cursor_preserves_world_point() {
        let mut vp = Viewport {
            offset_x: 400.0,
            offset_y: 300.0,
            scale: 1.0,
            css_width: 800.0,
            css_height: 600.0,
        };
        let (cx, cy) = (500.0, 400.0);

        // World coords under cursor before zoom
        let (wx_before, wy_before) = vp.screen_to_world(cx, cy);

        // Zoom in
        zoom_toward_cursor(&mut vp, cx, cy, -1.0);

        // World coords under cursor after zoom should be the same
        let (wx_after, wy_after) = vp.screen_to_world(cx, cy);
        assert!(
            (wx_before - wx_after).abs() < 1e-9,
            "world x changed: {wx_before} -> {wx_after}"
        );
        assert!(
            (wy_before - wy_after).abs() < 1e-9,
            "world y changed: {wy_before} -> {wy_after}"
        );
    }

    #[test]
    fn test_zoom_clamps_scale() {
        let mut vp = Viewport {
            offset_x: 400.0,
            offset_y: 300.0,
            scale: 3.9,
            css_width: 800.0,
            css_height: 600.0,
        };
        // Zoom in repeatedly — should clamp at 4.0
        for _ in 0..20 {
            zoom_toward_cursor(&mut vp, 400.0, 300.0, -1.0);
        }
        assert!(vp.scale <= 4.0, "scale exceeded 4.0: {}", vp.scale);

        let mut vp2 = Viewport {
            offset_x: 400.0,
            offset_y: 300.0,
            scale: 0.15,
            css_width: 800.0,
            css_height: 600.0,
        };
        for _ in 0..20 {
            zoom_toward_cursor(&mut vp2, 400.0, 300.0, 1.0);
        }
        assert!(vp2.scale >= 0.1, "scale below 0.1: {}", vp2.scale);
    }
}
