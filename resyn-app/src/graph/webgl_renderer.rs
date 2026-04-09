use wasm_bindgen::JsCast;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader,
    WebGlVertexArrayObject,
};

use super::layout_state::{GraphState, NodeState};
use super::renderer::{Renderer, Viewport};
use crate::server_fns::graph::EdgeType;

// ── Shader source strings ────────────────────────────────────────────────────

const NODE_VERT: &str = r#"#version 300 es
in vec2 a_quad;
in vec2 a_position;
in float a_radius;
in float a_alpha;
in vec3 a_color;
in float a_is_seed;

uniform vec2 u_resolution;
uniform vec2 u_offset;
uniform float u_scale;

out vec2 v_local;
out float v_alpha;
out vec3 v_color;
out float v_is_seed;

void main() {
    v_local = a_quad;
    v_alpha = a_alpha;
    v_color = a_color;
    v_is_seed = a_is_seed;
    vec2 world = a_position + a_quad * a_radius;
    vec2 screen = (world * u_scale + u_offset) / u_resolution * 2.0 - 1.0;
    gl_Position = vec4(screen.x, -screen.y, 0.0, 1.0);
}
"#;

const NODE_FRAG: &str = r#"#version 300 es
precision mediump float;
in vec2 v_local;
in float v_alpha;
in vec3 v_color;
in float v_is_seed;
out vec4 fragColor;

void main() {
    float d = length(v_local);
    float fw = fwidth(d);
    float alpha_mask = 1.0 - smoothstep(1.0 - fw, 1.0 + fw, d);
    if (alpha_mask < 0.001) discard;

    fragColor = vec4(v_color, v_alpha * alpha_mask);
}
"#;

const EDGE_VERT: &str = r#"#version 300 es
in vec2 a_position;
in vec3 a_color;
in float a_alpha;

uniform vec2 u_resolution;
uniform vec2 u_offset;
uniform float u_scale;

out vec3 v_color;
out float v_alpha;

void main() {
    vec2 screen = (a_position * u_scale + u_offset) / u_resolution * 2.0 - 1.0;
    gl_Position = vec4(screen.x, -screen.y, 0.0, 1.0);
    v_color = a_color;
    v_alpha = a_alpha;
}
"#;

const EDGE_FRAG: &str = r#"#version 300 es
precision mediump float;
in vec3 v_color;
in float v_alpha;
out vec4 fragColor;

void main() {
    fragColor = vec4(v_color, v_alpha);
}
"#;

// ── WebGL2Renderer ───────────────────────────────────────────────────────────

pub struct WebGL2Renderer {
    gl: WebGl2RenderingContext,
    node_program: WebGlProgram,
    edge_program: WebGlProgram,
    node_vao: WebGlVertexArrayObject,
    edge_vao: WebGlVertexArrayObject,
    /// Preallocated quad vertex buffer. Kept alive so the node VAO binding remains valid.
    #[allow(dead_code)]
    quad_buf: WebGlBuffer,
    instance_buf: WebGlBuffer,
    edge_buf: WebGlBuffer,
    width: u32,
    height: u32,
}

impl WebGL2Renderer {
    pub fn new(canvas: &HtmlCanvasElement) -> Self {
        let gl = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();

        let node_program = link_program(
            &gl,
            &compile_shader(&gl, WebGl2RenderingContext::VERTEX_SHADER, NODE_VERT),
            &compile_shader(&gl, WebGl2RenderingContext::FRAGMENT_SHADER, NODE_FRAG),
        );

        let edge_program = link_program(
            &gl,
            &compile_shader(&gl, WebGl2RenderingContext::VERTEX_SHADER, EDGE_VERT),
            &compile_shader(&gl, WebGl2RenderingContext::FRAGMENT_SHADER, EDGE_FRAG),
        );

        let node_vao = gl.create_vertex_array().expect("node VAO");
        let edge_vao = gl.create_vertex_array().expect("edge VAO");

        // Preallocate VBOs — reused every frame via buffer_data updates (no per-frame create)
        let quad_buf = gl.create_buffer().expect("quad buf");
        let instance_buf = gl.create_buffer().expect("instance buf");
        let edge_buf = gl.create_buffer().expect("edge buf");

        // Upload static quad vertices once (unit square for instanced node rendering)
        let quad_verts: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0];
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&quad_buf));
        unsafe {
            let view = js_sys::Float32Array::view(&quad_verts);
            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        // Set up quad attribute pointer once in the node VAO
        gl.bind_vertex_array(Some(&node_vao));
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&quad_buf));
        let a_quad = gl.get_attrib_location(&node_program, "a_quad") as u32;
        gl.enable_vertex_attrib_array(a_quad);
        gl.vertex_attrib_pointer_with_i32(a_quad, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
        gl.vertex_attrib_divisor(a_quad, 0); // shared per quad vertex
        gl.bind_vertex_array(None);

        let width = canvas.width();
        let height = canvas.height();

        gl.viewport(0, 0, width as i32, height as i32);

        Self {
            gl,
            node_program,
            edge_program,
            node_vao,
            edge_vao,
            quad_buf,
            instance_buf,
            edge_buf,
            width,
            height,
        }
    }

    /// Returns screen-space positions and labels for rendering text as a 2D
    /// overlay on top of the WebGL canvas.
    pub fn node_screen_positions(
        &self,
        state: &GraphState,
        viewport: &Viewport,
    ) -> Vec<(f64, f64, String)> {
        state
            .nodes
            .iter()
            .map(|n| {
                let (sx, sy) = viewport.world_to_screen(n.x, n.y);
                (sx, sy, n.label())
            })
            .collect()
    }
}

impl Renderer for WebGL2Renderer {
    fn draw(&mut self, state: &GraphState, viewport: &Viewport) {
        let gl = &self.gl;

        // 1. Clear with #0d1117 background
        gl.clear_color(0.051, 0.067, 0.09, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        // 2. Enable blending
        gl.enable(WebGl2RenderingContext::BLEND);
        gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        // Viewport offset/scale are in CSS coordinates. The shader maps
        // world→clip via `(world * scale + offset) / resolution * 2 - 1`,
        // so resolution must also be in CSS pixels to match.
        let dpr = web_sys::window().unwrap().device_pixel_ratio() as f32;
        let res_x = self.width as f32 / dpr;
        let res_y = self.height as f32 / dpr;
        let offset_x = viewport.offset_x as f32;
        let offset_y = viewport.offset_y as f32;
        let scale = viewport.scale as f32;

        // 3. Compute neighbor set for dimming
        let neighbor_set: Option<std::collections::HashSet<usize>> =
            state.selected_node.map(|sel| {
                let mut set = std::collections::HashSet::new();
                set.insert(sel);
                for e in &state.edges {
                    if e.from_idx == sel {
                        set.insert(e.to_idx);
                    } else if e.to_idx == sel {
                        set.insert(e.from_idx);
                    }
                }
                set
            });

        let is_dimmed = |idx: usize| -> bool {
            neighbor_set
                .as_ref()
                .map(|s| !s.contains(&idx))
                .unwrap_or(false)
        };

        // ── Draw edges ───────────────────────────────────────────────────────
        let mut edge_data: Vec<f32> = Vec::new();
        let mut arrow_data: Vec<f32> = Vec::new();

        for edge in &state.edges {
            let visible = match edge.edge_type {
                EdgeType::Regular => state.show_citations,
                EdgeType::Contradiction => state.show_contradictions,
                EdgeType::AbcBridge => state.show_bridges,
                // Similarity edges are drawn on the label canvas overlay (dashed lines
                // require Canvas2D — WebGL2 has no native set_line_dash support).
                EdgeType::Similarity => false,
            };
            if !visible {
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

            let both_dimmed =
                neighbor_set.is_some() && is_dimmed(edge.from_idx) && is_dimmed(edge.to_idx);

            let da = depth_alpha_f32(edge, &state.nodes);
            let (r, g, b, base_alpha) = edge_color(edge.edge_type.clone(), both_dimmed, da);

            let from_vis = state
                .nodes
                .get(edge.from_idx)
                .map(|n| n.lod_visible && n.temporal_visible)
                .unwrap_or(true);
            let to_vis = state
                .nodes
                .get(edge.to_idx)
                .map(|n| n.lod_visible && n.temporal_visible)
                .unwrap_or(true);
            let edge_vis_alpha = if from_vis && to_vis {
                1.0_f32
            } else {
                0.05_f32
            };
            let alpha = base_alpha * edge_vis_alpha;

            // Compute half-width in world units for 1.5px screen-space (D-03)
            let half_width = 0.75 / scale;

            // 6 vertices per quad edge (2 triangles)
            build_quad_edge(
                &mut edge_data,
                (from.x as f32, from.y as f32),
                (to.x as f32, to.y as f32),
                half_width,
                r,
                g,
                b,
                alpha,
            );

            // Arrowhead triangles (3 vertices per arrowhead)
            let skip_arrow = edge.edge_type == EdgeType::Regular && both_dimmed;
            if !skip_arrow {
                build_arrowhead(&mut arrow_data, from, to, r, g, b, alpha);
            }
        }

        draw_edge_pass(
            gl,
            &self.edge_program,
            &self.edge_vao,
            &self.edge_buf,
            &edge_data,
            res_x,
            res_y,
            offset_x,
            offset_y,
            scale,
            WebGl2RenderingContext::TRIANGLES,
        );

        // Draw arrowhead triangles using same edge program
        draw_edge_pass(
            gl,
            &self.edge_program,
            &self.edge_vao,
            &self.edge_buf,
            &arrow_data,
            res_x,
            res_y,
            offset_x,
            offset_y,
            scale,
            WebGl2RenderingContext::TRIANGLES,
        );

        // ── Draw nodes (instanced) ───────────────────────────────────────────
        // Per-instance data: x, y, radius, alpha, r, g, b, is_seed
        let mut instance_data: Vec<f32> = Vec::new();
        let node_count = state.nodes.len();

        for (idx, node) in state.nodes.iter().enumerate() {
            let dimmed = is_dimmed(idx);
            let is_hovered = state.hovered_node == Some(idx);
            let is_selected = state.selected_node == Some(idx);

            let (r, g, b) = if dimmed && !is_selected && !is_hovered {
                hex_to_rgb("#2a3a4f")
            } else if is_hovered || is_selected {
                hex_to_rgb("#58a6ff")
            } else if node.is_seed {
                hex_to_rgb("#d29922")
            } else {
                hex_to_rgb("#4a9eff")
            };

            let base_alpha = if dimmed && !is_selected && !is_hovered {
                0.5_f32
            } else {
                1.0_f32
            };
            let lod_alpha = if node.lod_visible { 1.0_f32 } else { 0.03_f32 };
            let time_alpha = if node.temporal_visible {
                1.0_f32
            } else {
                0.10_f32
            };
            let alpha = base_alpha * lod_alpha * time_alpha;

            instance_data.extend_from_slice(&[
                node.x as f32,
                node.y as f32,
                node.radius as f32,
                alpha,
                r,
                g,
                b,
                if node.is_seed { 1.0 } else { 0.0 },
            ]);
        }

        if node_count > 0 {
            gl.use_program(Some(&self.node_program));
            gl.bind_vertex_array(Some(&self.node_vao));

            // Update preallocated instance buffer with this frame's per-node data
            gl.bind_buffer(
                WebGl2RenderingContext::ARRAY_BUFFER,
                Some(&self.instance_buf),
            );
            unsafe {
                let view = js_sys::Float32Array::view(&instance_data);
                gl.buffer_data_with_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    &view,
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                );
            }

            // Stride: 8 floats per instance (x, y, radius, alpha, r, g, b, is_seed)
            let stride = 8 * 4; // bytes

            let a_position = gl.get_attrib_location(&self.node_program, "a_position") as u32;
            gl.enable_vertex_attrib_array(a_position);
            gl.vertex_attrib_pointer_with_i32(
                a_position,
                2,
                WebGl2RenderingContext::FLOAT,
                false,
                stride,
                0,
            );
            gl.vertex_attrib_divisor(a_position, 1);

            let a_radius = gl.get_attrib_location(&self.node_program, "a_radius") as u32;
            gl.enable_vertex_attrib_array(a_radius);
            gl.vertex_attrib_pointer_with_i32(
                a_radius,
                1,
                WebGl2RenderingContext::FLOAT,
                false,
                stride,
                2 * 4,
            );
            gl.vertex_attrib_divisor(a_radius, 1);

            let a_alpha = gl.get_attrib_location(&self.node_program, "a_alpha") as u32;
            gl.enable_vertex_attrib_array(a_alpha);
            gl.vertex_attrib_pointer_with_i32(
                a_alpha,
                1,
                WebGl2RenderingContext::FLOAT,
                false,
                stride,
                3 * 4,
            );
            gl.vertex_attrib_divisor(a_alpha, 1);

            let a_color = gl.get_attrib_location(&self.node_program, "a_color") as u32;
            gl.enable_vertex_attrib_array(a_color);
            gl.vertex_attrib_pointer_with_i32(
                a_color,
                3,
                WebGl2RenderingContext::FLOAT,
                false,
                stride,
                4 * 4,
            );
            gl.vertex_attrib_divisor(a_color, 1);

            let a_is_seed = gl.get_attrib_location(&self.node_program, "a_is_seed") as u32;
            gl.enable_vertex_attrib_array(a_is_seed);
            gl.vertex_attrib_pointer_with_i32(
                a_is_seed,
                1,
                WebGl2RenderingContext::FLOAT,
                false,
                stride,
                7 * 4, // byte offset: after x,y,radius,alpha,r,g,b
            );
            gl.vertex_attrib_divisor(a_is_seed, 1);

            // Set uniforms
            set_uniforms(
                gl,
                &self.node_program,
                res_x,
                res_y,
                offset_x,
                offset_y,
                scale,
            );

            // Instanced draw: 4 quad vertices, N instances
            gl.draw_arrays_instanced(
                WebGl2RenderingContext::TRIANGLE_FAN,
                0,
                4,
                node_count as i32,
            );

            gl.bind_vertex_array(None);

            // Draw seed node outer ring (D-15)
            if let Some(seed_idx) = state.nodes.iter().position(|n| n.is_seed) {
                let seed = &state.nodes[seed_idx];
                let dimmed = is_dimmed(seed_idx);
                if !dimmed {
                    // Screen-space constant gap and thickness, converted to world units
                    let gap = 2.0 / scale;
                    let ring_thickness = 3.0 / scale;
                    let (ring_r, ring_g, ring_b) = hex_to_rgb("#d29922");

                    let lod_alpha = if seed.lod_visible { 1.0_f32 } else { 0.03_f32 };
                    let time_alpha = if seed.temporal_visible {
                        1.0_f32
                    } else {
                        0.10_f32
                    };
                    let ring_alpha = lod_alpha * time_alpha;

                    // Build ring annulus from triangle segments using edge buffer
                    let mut ring_verts: Vec<f32> = Vec::new();
                    let segments = 48_u32;
                    let inner_r = (seed.radius as f32) + gap;
                    let outer_r = (seed.radius as f32) + gap + ring_thickness;
                    let cx = seed.x as f32;
                    let cy = seed.y as f32;
                    for i in 0..segments {
                        let a0 = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                        let a1 = ((i + 1) as f32) / (segments as f32) * std::f32::consts::TAU;
                        let (s0, c0) = (a0.sin(), a0.cos());
                        let (s1, c1) = (a1.sin(), a1.cos());
                        // Two triangles per segment
                        // Triangle 1: inner0, outer0, outer1
                        push_edge_vertex(
                            &mut ring_verts,
                            cx + inner_r * c0,
                            cy + inner_r * s0,
                            ring_r,
                            ring_g,
                            ring_b,
                            ring_alpha,
                        );
                        push_edge_vertex(
                            &mut ring_verts,
                            cx + outer_r * c0,
                            cy + outer_r * s0,
                            ring_r,
                            ring_g,
                            ring_b,
                            ring_alpha,
                        );
                        push_edge_vertex(
                            &mut ring_verts,
                            cx + outer_r * c1,
                            cy + outer_r * s1,
                            ring_r,
                            ring_g,
                            ring_b,
                            ring_alpha,
                        );
                        // Triangle 2: inner0, outer1, inner1
                        push_edge_vertex(
                            &mut ring_verts,
                            cx + inner_r * c0,
                            cy + inner_r * s0,
                            ring_r,
                            ring_g,
                            ring_b,
                            ring_alpha,
                        );
                        push_edge_vertex(
                            &mut ring_verts,
                            cx + outer_r * c1,
                            cy + outer_r * s1,
                            ring_r,
                            ring_g,
                            ring_b,
                            ring_alpha,
                        );
                        push_edge_vertex(
                            &mut ring_verts,
                            cx + inner_r * c1,
                            cy + inner_r * s1,
                            ring_r,
                            ring_g,
                            ring_b,
                            ring_alpha,
                        );
                    }

                    draw_edge_pass(
                        gl,
                        &self.edge_program,
                        &self.edge_vao,
                        &self.edge_buf,
                        &ring_verts,
                        res_x,
                        res_y,
                        offset_x,
                        offset_y,
                        scale,
                        WebGl2RenderingContext::TRIANGLES,
                    );
                }
            }
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.gl.viewport(0, 0, width as i32, height as i32);
    }
}

// ── Helper: edge color ───────────────────────────────────────────────────────

fn edge_color(edge_type: EdgeType, both_dimmed: bool, depth_alpha: f32) -> (f32, f32, f32, f32) {
    match edge_type {
        EdgeType::Regular => {
            let (r, g, b) = hex_to_rgb("#8b949e");
            let alpha = if both_dimmed { 0.1 } else { depth_alpha };
            (r, g, b, alpha)
        }
        EdgeType::Contradiction => {
            let (r, g, b) = hex_to_rgb("#f85149");
            (r, g, b, 1.0)
        }
        EdgeType::AbcBridge => {
            let (r, g, b) = hex_to_rgb("#d29922");
            (r, g, b, 1.0)
        }
        // Similarity edges are rendered on the label canvas overlay, not via WebGL.
        // This arm is unreachable in practice (filtered out before edge_color is called).
        EdgeType::Similarity => {
            let (r, g, b) = hex_to_rgb("#f0a030");
            (r, g, b, 0.7)
        }
    }
}

// ── Helper: depth-based alpha for regular edges ──────────────────────────────

/// Compute depth-based alpha for regular citation edges.
/// Uses max BFS depth of the two endpoints. Matches Canvas 2D depth_alpha().
fn depth_alpha_f32(edge: &super::layout_state::EdgeData, nodes: &[NodeState]) -> f32 {
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

// ── Helper: build quad edge (2 triangles = 6 vertices) ──────────────────────

/// Build world-space quad vertices for a single edge (6 vertices = 2 triangles).
/// half_width is in world units (= 0.75 / viewport_scale for 1.5px screen-space).
#[allow(clippy::too_many_arguments)]
fn build_quad_edge(
    buf: &mut Vec<f32>,
    from: (f32, f32),
    to: (f32, f32),
    half_width: f32,
    r: f32,
    g: f32,
    b: f32,
    alpha: f32,
) {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    // Perpendicular direction
    let px = -dy / len * half_width;
    let py = dx / len * half_width;

    // 4 quad corners
    let c0 = (from.0 + px, from.1 + py);
    let c1 = (from.0 - px, from.1 - py);
    let c2 = (to.0 + px, to.1 + py);
    let c3 = (to.0 - px, to.1 - py);

    // Triangle 1: c0, c1, c2
    push_edge_vertex(buf, c0.0, c0.1, r, g, b, alpha);
    push_edge_vertex(buf, c1.0, c1.1, r, g, b, alpha);
    push_edge_vertex(buf, c2.0, c2.1, r, g, b, alpha);
    // Triangle 2: c1, c3, c2
    push_edge_vertex(buf, c1.0, c1.1, r, g, b, alpha);
    push_edge_vertex(buf, c3.0, c3.1, r, g, b, alpha);
    push_edge_vertex(buf, c2.0, c2.1, r, g, b, alpha);
}

// ── Helper: push edge vertex ─────────────────────────────────────────────────

fn push_edge_vertex(buf: &mut Vec<f32>, x: f32, y: f32, r: f32, g: f32, b: f32, alpha: f32) {
    buf.extend_from_slice(&[x, y, r, g, b, alpha]);
}

// ── Helper: build arrowhead triangles ────────────────────────────────────────

fn build_arrowhead(
    buf: &mut Vec<f32>,
    from: &NodeState,
    to: &NodeState,
    r: f32,
    g: f32,
    b: f32,
    alpha: f32,
) {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let angle = dy.atan2(dx) as f32;
    let target_radius = to.radius as f32;
    let size = 8.0_f32;

    let tip_x = to.x as f32 - target_radius * angle.cos();
    let tip_y = to.y as f32 - target_radius * angle.sin();

    let wing1_x = tip_x - size * (angle - 0.4).cos();
    let wing1_y = tip_y - size * (angle - 0.4).sin();
    let wing2_x = tip_x - size * (angle + 0.4).cos();
    let wing2_y = tip_y - size * (angle + 0.4).sin();

    push_edge_vertex(buf, tip_x, tip_y, r, g, b, alpha);
    push_edge_vertex(buf, wing1_x, wing1_y, r, g, b, alpha);
    push_edge_vertex(buf, wing2_x, wing2_y, r, g, b, alpha);
}

// ── Helper: draw edge pass ───────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn draw_edge_pass(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    vao: &WebGlVertexArrayObject,
    edge_buf: &WebGlBuffer,
    data: &[f32],
    res_x: f32,
    res_y: f32,
    offset_x: f32,
    offset_y: f32,
    scale: f32,
    draw_mode: u32,
) {
    if data.is_empty() {
        return;
    }

    let vertex_count = data.len() / 6; // 6 floats per vertex: x, y, r, g, b, alpha

    gl.use_program(Some(program));
    gl.bind_vertex_array(Some(vao));

    // Update preallocated edge buffer with this pass's data
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(edge_buf));
    unsafe {
        let view = js_sys::Float32Array::view(data);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }

    let stride = 6 * 4; // 6 floats * 4 bytes

    let a_position = gl.get_attrib_location(program, "a_position") as u32;
    gl.enable_vertex_attrib_array(a_position);
    gl.vertex_attrib_pointer_with_i32(
        a_position,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        stride,
        0,
    );

    let a_color = gl.get_attrib_location(program, "a_color") as u32;
    gl.enable_vertex_attrib_array(a_color);
    gl.vertex_attrib_pointer_with_i32(
        a_color,
        3,
        WebGl2RenderingContext::FLOAT,
        false,
        stride,
        2 * 4,
    );

    let a_alpha = gl.get_attrib_location(program, "a_alpha") as u32;
    gl.enable_vertex_attrib_array(a_alpha);
    gl.vertex_attrib_pointer_with_i32(
        a_alpha,
        1,
        WebGl2RenderingContext::FLOAT,
        false,
        stride,
        5 * 4,
    );

    set_uniforms(gl, program, res_x, res_y, offset_x, offset_y, scale);

    gl.draw_arrays(draw_mode, 0, vertex_count as i32);

    gl.bind_vertex_array(None);
}

// ── Helper: set uniforms ─────────────────────────────────────────────────────

fn set_uniforms(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    res_x: f32,
    res_y: f32,
    offset_x: f32,
    offset_y: f32,
    scale: f32,
) {
    if let Some(loc) = gl.get_uniform_location(program, "u_resolution") {
        gl.uniform2f(Some(&loc), res_x, res_y);
    }
    if let Some(loc) = gl.get_uniform_location(program, "u_offset") {
        gl.uniform2f(Some(&loc), offset_x, offset_y);
    }
    if let Some(loc) = gl.get_uniform_location(program, "u_scale") {
        gl.uniform1f(Some(&loc), scale);
    }
}

// ── Helper: compile shader ───────────────────────────────────────────────────

fn compile_shader(gl: &WebGl2RenderingContext, shader_type: u32, source: &str) -> WebGlShader {
    let shader = gl.create_shader(shader_type).expect("create shader");
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    if !gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let log = gl.get_shader_info_log(&shader).unwrap_or_default();
        web_sys::console::error_1(&format!("Shader compile error: {log}").into());
    }
    shader
}

// ── Helper: link program ─────────────────────────────────────────────────────

fn link_program(
    gl: &WebGl2RenderingContext,
    vert: &WebGlShader,
    frag: &WebGlShader,
) -> WebGlProgram {
    let program = gl.create_program().expect("create program");
    gl.attach_shader(&program, vert);
    gl.attach_shader(&program, frag);
    gl.link_program(&program);
    if !gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let log = gl.get_program_info_log(&program).unwrap_or_default();
        web_sys::console::error_1(&format!("Program link error: {log}").into());
    }
    program
}

// ── Helper: hex_to_rgb ───────────────────────────────────────────────────────

pub fn hex_to_rgb(hex: &str) -> (f32, f32, f32) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
    (r, g, b)
}

// ── Helper: create_buffer (unused but part of spec) ─────────────────────────

#[allow(dead_code)]
fn create_buffer(gl: &WebGl2RenderingContext, data: &[f32]) -> WebGlBuffer {
    let buf = gl.create_buffer().expect("create buffer");
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buf));
    unsafe {
        let view = js_sys::Float32Array::view(data);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }
    buf
}
