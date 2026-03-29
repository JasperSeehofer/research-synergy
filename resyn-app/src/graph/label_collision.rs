use super::layout_state::NodeState;
use super::renderer::Viewport;

// ── Label rendering constants (per 17-UI-SPEC.md) ─────────────────────────────

pub const PILL_HEIGHT: f64 = 20.0; // 11px font + 4.5px top/bottom
pub const PILL_H_PAD: f64 = 8.0; // horizontal padding each side
pub const COLLISION_PAD: f64 = 8.0; // clearance between placed labels
pub const LABEL_NODE_GAP: f64 = 8.0; // gap below node in screen space
pub const PILL_CORNER_RADIUS: f64 = 4.0; // round_rect radius

// ── LabelCache ────────────────────────────────────────────────────────────────

/// Cached result of label collision avoidance.
///
/// Rebuilt when `label_cache_dirty` is set (viewport change, LOD change, graph
/// load). `text_widths` are cached separately at graph load time and never
/// change with viewport changes.
pub struct LabelCache {
    /// Node indices that passed collision culling, in draw order (highest
    /// priority first).
    pub visible_indices: Vec<usize>,
    /// Cached text widths per node index (indexed by position in nodes slice).
    pub text_widths: Vec<f64>,
}

// ── build_text_widths ─────────────────────────────────────────────────────────

/// Measure the text width of each node label using the Canvas 2D API.
///
/// Must be called from a browser context with a live `CanvasRenderingContext2d`.
/// Sets font to "11px monospace" (D-16) before measuring.
///
/// This is a browser-only function. It is NOT callable from `cargo test`.
/// Widths are cached at graph load time; never recomputed per frame.
pub fn build_text_widths(ctx: &web_sys::CanvasRenderingContext2d, nodes: &[NodeState]) -> Vec<f64> {
    ctx.set_font("11px monospace");
    nodes
        .iter()
        .map(|n| ctx.measure_text(&n.label()).unwrap().width())
        .collect()
}

// ── build_label_cache ─────────────────────────────────────────────────────────

/// Run the priority-ordered greedy collision avoidance pass.
///
/// Priority order (D-09): seed paper first, then descending citation count.
/// Skips nodes where `lod_visible == false` or `temporal_visible == false`.
///
/// For each candidate (in priority order), compute the screen-space pill rect
/// and skip if it overlaps any already-placed rect (with `COLLISION_PAD`
/// expansion on each side).
///
/// Returns `LabelCache` with `visible_indices` (draw order) and
/// `text_widths` cloned from the input slice.
pub fn build_label_cache(
    nodes: &[NodeState],
    text_widths: &[f64],
    viewport: &Viewport,
) -> LabelCache {
    let mut priority_indices: Vec<usize> = (0..nodes.len())
        .filter(|&i| nodes[i].lod_visible && nodes[i].temporal_visible)
        .collect();

    // Sort: seed first, then descending citation_count (D-09)
    priority_indices.sort_by(|&a, &b| {
        let na = &nodes[a];
        let nb = &nodes[b];
        if na.is_seed {
            return std::cmp::Ordering::Less;
        }
        if nb.is_seed {
            return std::cmp::Ordering::Greater;
        }
        nb.citation_count.cmp(&na.citation_count)
    });

    // Greedy placement (D-10)
    let mut placed: Vec<[f64; 4]> = Vec::new(); // [x, y, w, h]
    let mut visible_indices: Vec<usize> = Vec::new();

    for &i in &priority_indices {
        let node = &nodes[i];
        let (sx, sy) = viewport.world_to_screen(node.x, node.y);
        let text_w = text_widths.get(i).copied().unwrap_or(40.0);
        let pill_w = text_w + PILL_H_PAD * 2.0;
        let label_x = sx - pill_w / 2.0;
        let label_y = sy + node.radius * viewport.scale + LABEL_NODE_GAP;

        // Padded rect for overlap testing
        let rx = label_x - COLLISION_PAD;
        let ry = label_y - COLLISION_PAD;
        let rw = pill_w + COLLISION_PAD * 2.0;
        let rh = PILL_HEIGHT + COLLISION_PAD * 2.0;

        let overlaps = placed
            .iter()
            .any(|p| rx < p[0] + p[2] && rx + rw > p[0] && ry < p[1] + p[3] && ry + rh > p[1]);

        if !overlaps {
            placed.push([label_x, label_y, pill_w, PILL_HEIGHT]);
            visible_indices.push(i);
        }
    }

    LabelCache {
        visible_indices,
        text_widths: text_widths.to_vec(),
    }
}

// ── Pill drawing ─────────────────────────────────────────────────────────────

/// Draw a rounded-rectangle pill label with opaque background, border, and text.
///
/// Uses `arc_to` calls for rounded corners (compatible with all web-sys versions).
#[allow(clippy::too_many_arguments)]
pub fn draw_label_pill(
    ctx: &web_sys::CanvasRenderingContext2d,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    r: f64,
    text: &str,
    h_pad: f64,
) {
    ctx.set_fill_style_str("rgba(13,17,23,0.85)");
    ctx.begin_path();
    ctx.move_to(x + r, y);
    ctx.line_to(x + w - r, y);
    ctx.arc_to(x + w, y, x + w, y + r, r).unwrap();
    ctx.line_to(x + w, y + h - r);
    ctx.arc_to(x + w, y + h, x + w - r, y + h, r).unwrap();
    ctx.line_to(x + r, y + h);
    ctx.arc_to(x, y + h, x, y + h - r, r).unwrap();
    ctx.line_to(x, y + r);
    ctx.arc_to(x, y, x + r, y, r).unwrap();
    ctx.close_path();
    ctx.fill();

    ctx.set_stroke_style_str("#30363d");
    ctx.set_line_width(1.0);
    ctx.stroke();

    ctx.set_fill_style_str("#cccccc");
    ctx.fill_text(text, x + h_pad, y + 14.0).unwrap();
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_node(
        id: &str,
        x: f64,
        y: f64,
        citation_count: u32,
        is_seed: bool,
        visible: bool,
    ) -> NodeState {
        NodeState {
            id: id.to_string(),
            title: String::new(),
            first_author: "Author".to_string(),
            year: "2023".to_string(),
            citation_count,
            abstract_text: String::new(),
            authors: vec![],
            x,
            y,
            radius: 8.0,
            pinned: false,
            bfs_depth: None,
            lod_visible: visible,
            temporal_visible: visible,
            is_seed,
            top_keywords: vec![],
        }
    }

    fn test_viewport() -> Viewport {
        Viewport {
            offset_x: 400.0,
            offset_y: 300.0,
            scale: 1.0,
            css_width: 800.0,
            css_height: 600.0,
        }
    }

    #[test]
    fn test_priority_seed_first() {
        // Seed with 0 citations should still be first
        let nodes = vec![
            make_test_node("a", 100.0, 100.0, 100, false, true),
            make_test_node("b", 200.0, 200.0, 50, false, true),
            make_test_node("seed", 300.0, 300.0, 0, true, true),
        ];
        let text_widths = vec![50.0, 60.0, 40.0];
        let cache = build_label_cache(&nodes, &text_widths, &test_viewport());
        // Seed is index 2; it should appear first in visible_indices
        assert_eq!(
            cache.visible_indices[0], 2,
            "seed node should be first in visible_indices"
        );
    }

    #[test]
    fn test_priority_by_citation_desc() {
        // 3 non-seed nodes at different positions (no overlap); order by citation desc
        let nodes = vec![
            make_test_node("a", -300.0, -300.0, 10, false, true),
            make_test_node("b", -300.0, 300.0, 50, false, true),
            make_test_node("c", 300.0, -300.0, 30, false, true),
        ];
        let text_widths = vec![50.0, 50.0, 50.0];
        let cache = build_label_cache(&nodes, &text_widths, &test_viewport());
        // Expected order: b(50), c(30), a(10) => indices [1, 2, 0]
        assert_eq!(
            cache.visible_indices,
            vec![1, 2, 0],
            "should be sorted by descending citation_count"
        );
    }

    #[test]
    fn test_collision_skip_overlapping() {
        // Two nodes at the SAME screen position — only the higher-priority one gets a label
        let nodes = vec![
            make_test_node("high", 0.0, 0.0, 100, false, true),
            make_test_node("low", 0.0, 0.0, 10, false, true),
        ];
        let text_widths = vec![50.0, 50.0];
        let cache = build_label_cache(&nodes, &text_widths, &test_viewport());
        assert_eq!(
            cache.visible_indices.len(),
            1,
            "overlapping labels: only one should pass collision test"
        );
        assert_eq!(
            cache.visible_indices[0], 0,
            "the higher-priority (100 citations) node should win"
        );
    }

    #[test]
    fn test_collision_place_non_overlapping() {
        // Two nodes far apart — both should get labels
        let nodes = vec![
            make_test_node("a", -500.0, -500.0, 100, false, true),
            make_test_node("b", 500.0, 500.0, 10, false, true),
        ];
        let text_widths = vec![50.0, 50.0];
        let cache = build_label_cache(&nodes, &text_widths, &test_viewport());
        assert_eq!(
            cache.visible_indices.len(),
            2,
            "non-overlapping nodes should both get labels"
        );
    }

    #[test]
    fn test_invisible_nodes_excluded() {
        let nodes = vec![
            make_test_node("visible", 0.0, 0.0, 100, false, true),
            make_test_node("hidden", 50.0, 50.0, 50, false, false),
        ];
        let text_widths = vec![50.0, 50.0];
        let cache = build_label_cache(&nodes, &text_widths, &test_viewport());
        assert!(
            !cache.visible_indices.contains(&1),
            "lod_visible=false node should not appear in visible_indices"
        );
    }

    #[test]
    fn test_empty_nodes() {
        let nodes: Vec<NodeState> = vec![];
        let text_widths: Vec<f64> = vec![];
        let cache = build_label_cache(&nodes, &text_widths, &test_viewport());
        assert!(
            cache.visible_indices.is_empty(),
            "empty nodes slice should produce empty visible_indices"
        );
    }

    #[test]
    fn test_build_text_widths_length() {
        // build_text_widths requires a browser Canvas context so we test the
        // shape contract using a synthetic check on the collision path instead.
        // The actual measureText is browser-only; this test validates the
        // surrounding logic by ensuring the cache carries widths per node.
        let nodes = vec![
            make_test_node("a", -300.0, -300.0, 10, false, true),
            make_test_node("b", 300.0, 300.0, 50, false, true),
            make_test_node("c", 0.0, 0.0, 30, false, true),
        ];
        let synthetic_widths = vec![45.0, 52.0, 38.0];
        let cache = build_label_cache(&nodes, &synthetic_widths, &test_viewport());
        assert_eq!(
            cache.text_widths.len(),
            3,
            "text_widths should have one entry per node"
        );
        assert!((cache.text_widths[0] - 45.0).abs() < 1e-10);
        assert!((cache.text_widths[1] - 52.0).abs() < 1e-10);
        assert!((cache.text_widths[2] - 38.0).abs() < 1e-10);
    }
}
