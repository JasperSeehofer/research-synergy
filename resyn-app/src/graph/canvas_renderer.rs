use js_sys::Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use super::layout_state::{EdgeData, GraphState, NodeState};
use super::renderer::{Renderer, Viewport};
use crate::server_fns::graph::EdgeType;

pub struct Canvas2DRenderer {
    ctx: CanvasRenderingContext2d,
    width: u32,
    height: u32,
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
        }
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

        // 9. Draw labels (only if viewport.scale > 0.6, and only for LOD+temporal visible nodes)
        if viewport.scale > 0.6 {
            self.ctx.set_font("11px monospace");
            self.ctx.set_fill_style_str("#cccccc");

            for node in &state.nodes {
                if !node.lod_visible || !node.temporal_visible {
                    continue;
                }
                let label = node.label();
                let metrics = self.ctx.measure_text(&label).unwrap();
                let text_half_width = metrics.width() / 2.0;
                self.ctx
                    .fill_text(
                        &label,
                        node.x - text_half_width,
                        node.y + node.radius + 14.0,
                    )
                    .unwrap();
            }
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
