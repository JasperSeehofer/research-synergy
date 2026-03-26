use js_sys::Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use super::label_collision::LabelCache;
use super::layout_state::{EdgeData, GraphState, NodeState};
use super::renderer::{Renderer, Viewport};
use crate::server_fns::graph::EdgeType;

pub struct Canvas2DRenderer {
    ctx: CanvasRenderingContext2d,
    width: u32,
    height: u32,
    /// Cached label collision result. None = no labels drawn (e.g. during fit animation).
    label_cache: Option<LabelCache>,
    /// Whether the fit animation is currently active (suppress labels during animation).
    fit_anim_active: bool,
}

impl Canvas2DRenderer {
    pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Self {
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        Self {
            width: canvas.width(),
            height: canvas.height(),
            ctx,
            label_cache: None,
            fit_anim_active: false,
        }
    }

    pub fn set_label_cache(&mut self, cache: Option<LabelCache>) {
        self.label_cache = cache;
    }

    pub fn set_fit_anim_active(&mut self, active: bool) {
        self.fit_anim_active = active;
    }
}

impl Renderer for Canvas2DRenderer {
    fn draw(&mut self, state: &GraphState, viewport: &Viewport) {
        // 1. Reset transform and clear canvas (use actual pixel dimensions)
        self.ctx
            .set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
            .unwrap();
        self.ctx.set_fill_style_str("#0d1117");
        let canvas = self.ctx.canvas().unwrap();
        self.ctx
            .fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        // 2. Apply viewport transform
        viewport.apply(&self.ctx);

        // 3. Compute neighbor set if a node is selected
        let neighbor_set: Option<std::collections::HashSet<usize>> =
            state.selected_node.map(|sel| {
                let mut neighbors = std::collections::HashSet::new();
                neighbors.insert(sel);
                for edge in &state.edges {
                    if edge.from_idx == sel {
                        neighbors.insert(edge.to_idx);
                    } else if edge.to_idx == sel {
                        neighbors.insert(edge.from_idx);
                    }
                }
                neighbors
            });

        let is_dimmed = |idx: usize| -> bool {
            if let Some(ref neighbors) = neighbor_set {
                !neighbors.contains(&idx)
            } else {
                false
            }
        };

        let edge_both_dimmed = |e: &EdgeData| -> bool {
            if neighbor_set.is_some() {
                is_dimmed(e.from_idx) && is_dimmed(e.to_idx)
            } else {
                false
            }
        };

        // 4. Draw regular citation edges
        for edge in &state.edges {
            if edge.edge_type != EdgeType::Regular {
                continue;
            }
            let from = match state.nodes.get(edge.from_idx) {
                Some(n) => n,
                None => continue,
            };
            let to = match state.nodes.get(edge.to_idx) {
                Some(n) => n,
                None => continue,
            };

            let from_vis = from.lod_visible && from.temporal_visible;
            let to_vis = to.lod_visible && to.temporal_visible;
            let edge_vis_alpha = if from_vis && to_vis { 1.0 } else { 0.05 };

            self.ctx.save();
            self.ctx.set_stroke_style_str("#8b949e");
            self.ctx.set_line_width(1.5 / viewport.scale);
            let base_alpha = depth_alpha(edge, &state.nodes);
            let dim_alpha = if edge_both_dimmed(edge) {
                0.1
            } else {
                base_alpha
            };
            self.ctx.set_global_alpha(dim_alpha * edge_vis_alpha);
            self.ctx.begin_path();
            self.ctx.move_to(from.x, from.y);
            self.ctx.line_to(to.x, to.y);
            self.ctx.stroke();
            self.ctx.restore();
        }

        // 5. Draw contradiction edges (if show_contradictions)
        if state.show_contradictions {
            for edge in &state.edges {
                if edge.edge_type != EdgeType::Contradiction {
                    continue;
                }
                let from = match state.nodes.get(edge.from_idx) {
                    Some(n) => n,
                    None => continue,
                };
                let to = match state.nodes.get(edge.to_idx) {
                    Some(n) => n,
                    None => continue,
                };

                self.ctx.save();
                self.ctx.set_stroke_style_str("#f85149");
                self.ctx.set_line_width(2.0);
                self.ctx.set_global_alpha(1.0);
                self.ctx.begin_path();
                self.ctx.move_to(from.x, from.y);
                self.ctx.line_to(to.x, to.y);
                self.ctx.stroke();
                self.ctx.restore();
            }
        }

        // 6. Draw ABC-bridge edges (if show_bridges)
        if state.show_bridges {
            for edge in &state.edges {
                if edge.edge_type != EdgeType::AbcBridge {
                    continue;
                }
                let from = match state.nodes.get(edge.from_idx) {
                    Some(n) => n,
                    None => continue,
                };
                let to = match state.nodes.get(edge.to_idx) {
                    Some(n) => n,
                    None => continue,
                };

                self.ctx.save();
                self.ctx.set_stroke_style_str("#d29922");
                self.ctx.set_line_width(2.0);
                self.ctx.set_global_alpha(1.0);
                let dash_array = Array::new();
                dash_array.push(&JsValue::from_f64(6.0));
                dash_array.push(&JsValue::from_f64(4.0));
                self.ctx.set_line_dash(&dash_array).unwrap();
                self.ctx.begin_path();
                self.ctx.move_to(from.x, from.y);
                self.ctx.line_to(to.x, to.y);
                self.ctx.stroke();
                self.ctx.restore();
            }
        }

        // 7. Draw arrowheads on all visible edges
        for edge in &state.edges {
            let from = match state.nodes.get(edge.from_idx) {
                Some(n) => n,
                None => continue,
            };
            let to = match state.nodes.get(edge.to_idx) {
                Some(n) => n,
                None => continue,
            };

            // Skip non-visible special edges
            let visible = match edge.edge_type {
                EdgeType::Regular => true,
                EdgeType::Contradiction => state.show_contradictions,
                EdgeType::AbcBridge => state.show_bridges,
            };
            if !visible {
                continue;
            }

            // Skip if edge is fully dimmed (regular edges only)
            if edge.edge_type == EdgeType::Regular && edge_both_dimmed(edge) {
                continue;
            }

            let color = match edge.edge_type {
                EdgeType::Regular => "#8b949e",
                EdgeType::Contradiction => "#f85149",
                EdgeType::AbcBridge => "#d29922",
            };

            let alpha = match edge.edge_type {
                EdgeType::Regular => depth_alpha(edge, &state.nodes),
                _ => 1.0,
            };

            self.ctx.save();
            self.ctx.set_fill_style_str(color);
            self.ctx.set_global_alpha(alpha);
            draw_arrowhead(&self.ctx, from.x, from.y, to.x, to.y, to.radius);
            self.ctx.restore();
        }

        // 8. Draw nodes
        for (idx, node) in state.nodes.iter().enumerate() {
            self.ctx.save();

            let dimmed = is_dimmed(idx);
            let is_hovered = state.hovered_node == Some(idx);
            let is_selected = state.selected_node == Some(idx);

            let lod_alpha = if node.lod_visible { 1.0 } else { 0.03 };
            let time_alpha = if node.temporal_visible { 1.0 } else { 0.10 };
            let combined_alpha = lod_alpha * time_alpha;
            if combined_alpha < 0.01 {
                self.ctx.restore();
                continue; // Skip drawing effectively invisible nodes
            }
            self.ctx
                .set_global_alpha(combined_alpha * (if dimmed { 0.5 } else { 1.0 }));

            let fill_color = if dimmed && !is_selected && !is_hovered {
                "#2a3a4f"
            } else if is_hovered || is_selected {
                "#58a6ff"
            } else if node.is_seed {
                "#d29922"
            } else {
                "#4a9eff"
            };

            // Draw node circle
            self.ctx.begin_path();
            self.ctx
                .arc(node.x, node.y, node.radius, 0.0, std::f64::consts::TAU)
                .unwrap();
            self.ctx.set_fill_style_str(fill_color);
            self.ctx.fill();

            // Node border
            let border_color = if node.is_seed { "#e8b84b" } else { "#7cb8ff" };
            self.ctx.set_stroke_style_str(border_color);
            self.ctx.set_line_width(1.0 / viewport.scale);
            self.ctx.stroke();

            // Seed node outer planetary ring
            if node.is_seed && !dimmed {
                self.ctx.begin_path();
                let ring_radius = node.radius + 2.0 + 1.5; // 2px gap + ring center
                self.ctx
                    .arc(node.x, node.y, ring_radius, 0.0, std::f64::consts::TAU)
                    .unwrap();
                self.ctx.set_stroke_style_str("#d29922");
                self.ctx.set_line_width(3.0 / viewport.scale);
                self.ctx.stroke();
            }

            // Selected outer ring
            if is_selected {
                self.ctx.begin_path();
                self.ctx
                    .arc(
                        node.x,
                        node.y,
                        node.radius + 4.0,
                        0.0,
                        std::f64::consts::TAU,
                    )
                    .unwrap();
                self.ctx.set_stroke_style_str("#58a6ff");
                self.ctx.set_line_width(2.0);
                self.ctx.stroke();
            }

            self.ctx.restore();
        }

        // ── Screen-space label rendering ──────────────────────────────────────
        // Labels are drawn in screen space (not world space) to avoid scaling
        // with zoom. Per D-14/D-15/D-16: pill badges with rgba(13,17,23,0.85)
        // bg, #30363d border, #cccccc text.
        // Per 17-RESEARCH Pitfall 6: suppress labels during fit animation.
        if !self.fit_anim_active && viewport.scale > 0.3 {
            // Reset transform to screen space
            self.ctx.save();
            let dpr = web_sys::window().unwrap().device_pixel_ratio();
            self.ctx
                .set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0)
                .unwrap();

            self.ctx.set_font("11px monospace");

            if let Some(ref cache) = self.label_cache {
                use crate::graph::label_collision::{
                    LABEL_NODE_GAP, PILL_CORNER_RADIUS, PILL_H_PAD, PILL_HEIGHT,
                };

                for &i in &cache.visible_indices {
                    let node = &state.nodes[i];
                    let (sx, sy) = viewport.world_to_screen(node.x, node.y);
                    let text_w = cache.text_widths.get(i).copied().unwrap_or(40.0);
                    let pill_w = text_w + PILL_H_PAD * 2.0;
                    let label_x = sx - pill_w / 2.0;
                    let label_y = sy + node.radius * viewport.scale + LABEL_NODE_GAP;

                    draw_label_pill(
                        &self.ctx,
                        label_x,
                        label_y,
                        pill_w,
                        PILL_HEIGHT,
                        PILL_CORNER_RADIUS,
                        &node.label(),
                        PILL_H_PAD,
                    );
                }

                // Hover label override (D-11): if hovered node's label was culled,
                // draw it anyway so the user always sees the label for what they hover.
                if let Some(hi) = state.hovered_node
                    && hi < state.nodes.len()
                    && !cache.visible_indices.contains(&hi)
                {
                    let node = &state.nodes[hi];
                    if node.lod_visible && node.temporal_visible {
                        use crate::graph::label_collision::{
                            LABEL_NODE_GAP, PILL_CORNER_RADIUS, PILL_H_PAD, PILL_HEIGHT,
                        };
                        let (sx, sy) = viewport.world_to_screen(node.x, node.y);
                        let text_w = cache.text_widths.get(hi).copied().unwrap_or(40.0);
                        let pill_w = text_w + PILL_H_PAD * 2.0;
                        let label_x = sx - pill_w / 2.0;
                        let label_y = sy + node.radius * viewport.scale + LABEL_NODE_GAP;

                        draw_label_pill(
                            &self.ctx,
                            label_x,
                            label_y,
                            pill_w,
                            PILL_HEIGHT,
                            PILL_CORNER_RADIUS,
                            &node.label(),
                            PILL_H_PAD,
                        );
                    }
                }
            }

            self.ctx.restore();
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        if let Some(canvas) = self.ctx.canvas() {
            canvas.set_width(width);
            canvas.set_height(height);
        }
    }

    fn set_label_cache(&mut self, cache: Option<crate::graph::label_collision::LabelCache>) {
        self.label_cache = cache;
    }

    fn set_fit_anim_active(&mut self, active: bool) {
        self.fit_anim_active = active;
    }
}

/// Compute depth-based alpha for regular citation edges (D-02).
/// Uses max BFS depth of the two endpoints.
fn depth_alpha(edge: &EdgeData, nodes: &[NodeState]) -> f64 {
    let from_depth = nodes
        .get(edge.from_idx)
        .and_then(|n| n.bfs_depth)
        .unwrap_or(u32::MAX);
    let to_depth = nodes
        .get(edge.to_idx)
        .and_then(|n| n.bfs_depth)
        .unwrap_or(u32::MAX);
    let max_depth = from_depth.max(to_depth);
    match max_depth {
        0 | 1 => 0.50,
        2 => 0.35,
        3 => 0.25,
        _ => 0.15,
    }
}

/// Draw a rounded-rectangle pill label with opaque background, border, and text.
///
/// Uses `arc_to` calls for rounded corners (fallback for older web-sys bindings
/// that may not expose `round_rect_with_f64`). The Canvas 2D Level 2 `round_rect`
/// API is standard since Chrome 99 / Firefox 112 / Safari 15.4, but we use the
/// arc_to path to guarantee compatibility with any web-sys version.
#[allow(clippy::too_many_arguments)]
fn draw_label_pill(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    r: f64,
    text: &str,
    h_pad: f64,
) {
    // Background fill
    ctx.set_fill_style_str("rgba(13,17,23,0.85)");
    ctx.begin_path();
    // Rounded rect via arc_to (compatible with all web-sys versions)
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

    // Border stroke
    ctx.set_stroke_style_str("#30363d");
    ctx.set_line_width(1.0);
    ctx.stroke();

    // Text
    ctx.set_fill_style_str("#cccccc");
    ctx.fill_text(text, x + h_pad, y + 14.0).unwrap();
}

fn draw_arrowhead(
    ctx: &CanvasRenderingContext2d,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    target_radius: f64,
) {
    let angle = (y2 - y1).atan2(x2 - x1);
    // Pull arrowhead back to node border
    let tip_x = x2 - target_radius * angle.cos();
    let tip_y = y2 - target_radius * angle.sin();
    let size = 8.0_f64;
    ctx.begin_path();
    ctx.move_to(tip_x, tip_y);
    ctx.line_to(
        tip_x - size * (angle - 0.4).cos(),
        tip_y - size * (angle - 0.4).sin(),
    );
    ctx.line_to(
        tip_x - size * (angle + 0.4).cos(),
        tip_y - size * (angle + 0.4).sin(),
    );
    ctx.close_path();
    ctx.fill();
}
