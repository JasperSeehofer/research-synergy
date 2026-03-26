use super::layout_state::NodeState;

/// Animation state for viewport fit lerp.
pub struct FitAnimState {
    pub active: bool,
    pub target_scale: f64,
    pub target_offset_x: f64,
    pub target_offset_y: f64,
}

impl Default for FitAnimState {
    fn default() -> Self {
        Self {
            active: false,
            target_scale: 1.0,
            target_offset_x: 0.0,
            target_offset_y: 0.0,
        }
    }
}

/// Linear interpolation between `a` and `b` at factor `t`.
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Compute the target viewport state to fit all visible nodes (lod_visible &&
/// temporal_visible) with a 10% margin on each side (margin_factor = 0.80).
///
/// Returns `None` if there are no visible nodes.
pub fn compute_fit_target(
    nodes: &[NodeState],
    css_width: f64,
    css_height: f64,
) -> Option<FitAnimState> {
    let visible: Vec<&NodeState> = nodes
        .iter()
        .filter(|n| n.lod_visible && n.temporal_visible)
        .collect();

    if visible.is_empty() {
        return None;
    }

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for n in &visible {
        let left = n.x - n.radius;
        let right = n.x + n.radius;
        let top = n.y - n.radius;
        let bottom = n.y + n.radius;
        if left < min_x {
            min_x = left;
        }
        if right > max_x {
            max_x = right;
        }
        if top < min_y {
            min_y = top;
        }
        if bottom > max_y {
            max_y = bottom;
        }
    }

    let margin_factor = 0.80_f64;
    let bb_w = (max_x - min_x).max(1.0);
    let bb_h = (max_y - min_y).max(1.0);
    let center_wx = (min_x + max_x) / 2.0;
    let center_wy = (min_y + max_y) / 2.0;

    let target_scale = (css_width * margin_factor / bb_w)
        .min(css_height * margin_factor / bb_h)
        .clamp(0.1, 4.0);
    let target_offset_x = css_width / 2.0 - center_wx * target_scale;
    let target_offset_y = css_height / 2.0 - center_wy * target_scale;

    Some(FitAnimState {
        active: true,
        target_scale,
        target_offset_x,
        target_offset_y,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::layout_state::NodeState;

    fn make_node(
        x: f64,
        y: f64,
        radius: f64,
        lod_visible: bool,
        temporal_visible: bool,
    ) -> NodeState {
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
            pinned: false,
            bfs_depth: Some(0),
            lod_visible,
            temporal_visible,
            is_seed: false,
        }
    }

    #[test]
    fn test_fit_target_basic() {
        // 3 nodes at known positions, radius 0
        // (-100,0), (100,0), (0,50) with radius=0
        // bb_w = 200, bb_h = 50
        // center_wx = 0, center_wy = 25
        // viewport 800x600
        // margin: 640/200 = 3.2, 480/50 = 9.6 => target_scale = 3.2 (min)
        // target_offset_x = 400 - 0 * 3.2 = 400
        // target_offset_y = 300 - 25 * 3.2 = 220
        let nodes = vec![
            make_node(-100.0, 0.0, 0.0, true, true),
            make_node(100.0, 0.0, 0.0, true, true),
            make_node(0.0, 50.0, 0.0, true, true),
        ];
        let result = compute_fit_target(&nodes, 800.0, 600.0).unwrap();
        assert!(result.active);
        assert!(
            (result.target_scale - 3.2).abs() < 1e-9,
            "expected scale 3.2, got {}",
            result.target_scale
        );
        assert!(
            (result.target_offset_x - 400.0).abs() < 1e-9,
            "expected offset_x 400.0, got {}",
            result.target_offset_x
        );
        assert!(
            (result.target_offset_y - 220.0).abs() < 1e-9,
            "expected offset_y 220.0, got {}",
            result.target_offset_y
        );
    }

    #[test]
    fn test_fit_target_margin() {
        // Verify margin_factor=0.80 is applied (not 1.0)
        // Single node at origin with radius 0, viewport 100x100
        // bb_w = 1.0 (max(0, 1.0)), bb_h = 1.0
        // without margin: scale = 100 / 1.0 = 100 => clamped to 4.0
        // with margin 0.80: scale = 80 / 1.0 = 80 => clamped to 4.0
        // We verify it uses margin by checking: for a larger bounding box (200x200):
        // viewport 1000x800, bb_w=200, bb_h=200
        // with margin 0.80: min(800/200, 640/200) = min(4.0, 3.2) = 3.2
        // without margin: min(1000/200, 800/200) = min(5.0, 4.0) = 4.0 (clamped)
        let nodes = vec![
            make_node(-100.0, -100.0, 0.0, true, true),
            make_node(100.0, 100.0, 0.0, true, true),
        ];
        let result = compute_fit_target(&nodes, 1000.0, 800.0).unwrap();
        // bb_w = 200, bb_h = 200
        // with margin 0.80: min(1000*0.80/200, 800*0.80/200) = min(4.0, 3.2) = 3.2
        assert!(
            (result.target_scale - 3.2).abs() < 1e-9,
            "expected scale 3.2 with margin, got {}",
            result.target_scale
        );
    }

    #[test]
    fn test_fit_target_empty() {
        // Zero visible nodes returns None
        let nodes: Vec<NodeState> = vec![];
        let result = compute_fit_target(&nodes, 800.0, 600.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_fit_target_filters_invisible() {
        // lod_visible=false or temporal_visible=false are excluded from bounding box
        let nodes = vec![
            make_node(-1000.0, -1000.0, 0.0, false, true), // lod_visible=false, should be excluded
            make_node(-100.0, 0.0, 0.0, true, true),
            make_node(100.0, 0.0, 0.0, true, false), // temporal_visible=false, should be excluded
            make_node(0.0, 50.0, 0.0, true, true),
        ];
        // Only nodes at (-100,0) and (0,50) are visible
        let result = compute_fit_target(&nodes, 800.0, 600.0).unwrap();
        // bb: min_x=-100, max_x=0, min_y=0, max_y=50 => bb_w=100, bb_h=50
        // center_wx=-50, center_wy=25
        // scale = min(800*0.80/100, 600*0.80/50) = min(6.4, 9.6) = 6.4 => clamped to 4.0
        assert!(
            (result.target_scale - 4.0).abs() < 1e-9,
            "expected scale 4.0 (clamped), got {}",
            result.target_scale
        );
    }

    #[test]
    fn test_fit_target_clamp() {
        // Extremely small bounding box (single node) clamps scale to 4.0 max
        let nodes = vec![make_node(0.0, 0.0, 0.5, true, true)];
        // bb_w = 1.0, bb_h = 1.0 (due to max(_, 1.0) after radius included: bb_w = 2*0.5=1.0)
        // scale = min(800*0.80/1.0, 600*0.80/1.0) = 480 => clamped to 4.0
        let result = compute_fit_target(&nodes, 800.0, 600.0).unwrap();
        assert!(
            (result.target_scale - 4.0).abs() < 1e-9,
            "scale must be clamped to 4.0 max, got {}",
            result.target_scale
        );
    }

    #[test]
    fn test_fit_target_includes_radius() {
        // Bounding box extends by node radius on each side
        // Node at (0,0) with radius=10: bounding box should be [-10,-10] to [10,10]
        // bb_w=20, bb_h=20, viewport 800x600
        // scale = min(800*0.80/20, 600*0.80/20) = min(32, 24) = 24 => clamped to 4.0
        let nodes = vec![make_node(0.0, 0.0, 10.0, true, true)];
        // Without radius, bb_w = bb_h = 1.0 (clamped), scale would be huge
        // With radius, bb_w = bb_h = 20.0, scale = 24.0 => clamped to 4.0
        let result = compute_fit_target(&nodes, 800.0, 600.0).unwrap();
        assert!(
            result.target_scale <= 4.0,
            "scale must be clamped to 4.0 max"
        );
        // Now verify with two nodes spread so radius matters
        // Node A at (-100,0) radius=5: left edge = -105
        // Node B at (100,0) radius=5: right edge = 105
        // bb_w = 210 (not 200 without radius)
        let nodes2 = vec![
            make_node(-100.0, 0.0, 5.0, true, true),
            make_node(100.0, 0.0, 5.0, true, true),
        ];
        let result2 = compute_fit_target(&nodes2, 800.0, 600.0).unwrap();
        // bb_w = 210, bb_h = 10 (2*radius, since both at y=0)
        // scale = min(800*0.80/210, 600*0.80/10) = min(~3.047, 48) = ~3.047
        let expected_scale = (800.0 * 0.80 / 210.0_f64).min(600.0 * 0.80 / 10.0);
        assert!(
            (result2.target_scale - expected_scale).abs() < 1e-6,
            "radius included in bounding box: expected {}, got {}",
            expected_scale,
            result2.target_scale
        );
    }

    #[test]
    fn test_lerp() {
        assert!((lerp(10.0, 20.0, 0.5) - 15.0).abs() < 1e-9);
        assert!((lerp(10.0, 20.0, 0.0) - 10.0).abs() < 1e-9);
        assert!((lerp(10.0, 20.0, 1.0) - 20.0).abs() < 1e-9);
    }
}
