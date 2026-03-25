# Phase 16: Edge and Node Renderer Fixes - Research

**Researched:** 2026-03-25
**Domain:** WebGL2 GLSL shaders, Canvas 2D rendering, Rust/WASM graph visualization
**Confidence:** HIGH

## Summary

Phase 16 fixes two rendering layers in parallel: the WebGL2 renderer (used above 300 nodes) and the Canvas 2D renderer (used below 300 nodes). The current code has three concrete problems: (1) regular citation edges use color `#404040` at 0.35 alpha — essentially invisible on the `#0d1117` dark background; (2) WebGL2 uses `GL.LINES` which is hardware-capped at 1px and cannot be widened; (3) node borders use `#30363d` which is also invisible on the dark background, and the node fragment shader uses a coarse `smoothstep(0.9, 1.0, d)` AA region that blurs heavily at low zoom.

The fixes are self-contained code changes to two renderer files and one data model file. There are no new dependencies, no API calls, and no external services involved. The primary technical challenge is the WebGL2 quad edge geometry refactor — replacing a per-edge `[x,y,r,g,b,alpha]` vertex pair with a `[x,y,perp_x,perp_y,r,g,b,alpha]` quad (4 vertices per edge, 2 triangles) and updating the vertex shader to apply the perpendicular offset in screen space.

**Primary recommendation:** Tackle changes in strict dependency order — data model first (`is_seed` on `NodeState`), then Canvas 2D renderer (simpler, immediate visual feedback), then WebGL2 shaders (most complex, can be validated against the Canvas 2D result).

## Standard Stack

This phase uses no new libraries. All implementation is within existing code.

### Core (Already in use)
| Component | Version | Purpose | Notes |
|-----------|---------|---------|-------|
| web_sys WebGl2RenderingContext | (via web-sys crate) | WebGL2 API bindings | Already used |
| web_sys CanvasRenderingContext2d | (via web-sys crate) | Canvas 2D API | Already used |
| GLSL ES 3.00 | `#version 300 es` | Shader language for WebGL2 | `fwidth()` requires ES 3.00 |
| wasm-bindgen | (locked in Cargo.toml) | JS-Rust interop | Already used |

### No New Dependencies Required

All changes are shader source strings (GLSL), Rust struct field additions, and arithmetic changes to buffer-building loops. The `fwidth()` GLSL built-in is part of GLSL ES 3.00 and already available because the shaders declare `#version 300 es`.

## Architecture Patterns

### File Structure (Affected Files)

```
resyn-app/src/graph/
├── layout_state.rs     # Add is_seed: bool to NodeState, set in from_graph_data()
├── canvas_renderer.rs  # Edge color/width/alpha, border color, seed ring
└── webgl_renderer.rs   # NODE_FRAG (fwidth AA + border + seed), EDGE_VERT/FRAG (quad geometry)
```

### Pattern 1: BFS-Depth Alpha for Canvas 2D Edges

The `EdgeData` struct already has `from_idx` and `to_idx`. `NodeState` already has `bfs_depth: Option<u32>`. During the edge rendering loop in `canvas_renderer.rs`, the depth-based alpha can be computed by looking up `state.nodes[edge.from_idx].bfs_depth` and `state.nodes[edge.to_idx].bfs_depth` and taking the max (the deepest end of the edge determines how faint it appears).

```rust
// Depth-based alpha for regular citation edges
fn depth_alpha(edge: &EdgeData, nodes: &[NodeState]) -> f64 {
    let from_depth = nodes.get(edge.from_idx).and_then(|n| n.bfs_depth).unwrap_or(u32::MAX);
    let to_depth = nodes.get(edge.to_idx).and_then(|n| n.bfs_depth).unwrap_or(u32::MAX);
    let max_depth = from_depth.max(to_depth);
    match max_depth {
        0 => 0.5,
        1 => 0.5,
        2 => 0.35,
        3 => 0.25,
        _ => 0.15,
    }
}
```

This is Claude's discretion on exact values within the 0.15–0.5 range. Both renderers must use the same function or equivalent logic.

### Pattern 2: WebGL2 Quad Edge Geometry

The current `EDGE_VERT` shader takes `a_position` (world xy) and transforms it directly. For quad edges, each edge needs two vertices with a perpendicular offset applied in screen space so the quad stays screen-space-wide regardless of zoom.

The standard approach for screen-space-width lines in WebGL2 is:
1. Pack the CPU-side perpendicular vector (in world coords) into the vertex
2. In the vertex shader, transform both the position AND the perpendicular into clip space, then add the clip-space offset scaled to screen pixels

The simpler correct approach for this codebase: compute the perpendicular in world space, pass it as `a_perp: vec2`, and in the vertex shader compute the screen-space half-width by dividing the desired pixel width by `u_resolution` in each axis, then add it before or after the transform. Because the existing transform is `(world * scale + offset) / resolution * 2 - 1`, the perpendicular in NDC space is `perp_world * scale / resolution * 2`.

```glsl
// New EDGE_VERT for quad geometry
#version 300 es
in vec2 a_position;   // world-space center of edge at this vertex
in vec2 a_perp;       // normalized perpendicular direction (world-space)
in float a_side;      // +1.0 or -1.0 (which side of the quad)
in vec3 a_color;
in float a_alpha;
in float a_half_width_px; // half width in CSS pixels (= 0.75 for 1.5px)

uniform vec2 u_resolution;
uniform vec2 u_offset;
uniform float u_scale;

out vec3 v_color;
out float v_alpha;
out float v_t;        // -1.0 to +1.0 across width (for fragment AA)

void main() {
    v_color = a_color;
    v_alpha = a_alpha;
    v_t = a_side;

    // Center point in NDC
    vec2 center_screen = (a_position * u_scale + u_offset) / u_resolution * 2.0 - 1.0;
    // Perpendicular offset in NDC = perp_world * scale / resolution * 2
    // (perp_world is normalized in world space)
    vec2 perp_ndc = a_perp * u_scale / u_resolution * 2.0;
    // Apply half-width in screen pixels converted to NDC
    // half_width_px in CSS px → in NDC = half_width_px / resolution * 2
    vec2 offset_ndc = (a_half_width_px / u_resolution) * 2.0 * normalize(perp_ndc);
    // Actually simpler: just scale the perp to the desired pixel width
    vec2 offset = a_side * (a_half_width_px / u_resolution) * 2.0
                  * sign(perp_ndc) * vec2(abs(perp_ndc.y), abs(perp_ndc.x));
    gl_Position = vec4(center_screen.x + ..., -center_screen.y + ..., 0.0, 1.0);
}
```

Note: The shader math above is illustrative. The exact formula is left to Claude's discretion (see below). The key insight is that the perpendicular offset must be applied AFTER the world-to-screen transform so it stays at a fixed screen-space pixel width. The CPU side must emit 4 vertices per edge (2 triangles = 6 indices, or use TRIANGLE_STRIP with 4 vertices).

**Simpler alternative approach (recommended):** Since arrowheads are already triangle geometry with per-vertex positions computed on CPU, the same strategy works for quad edges — compute all 4 corner positions in world space using the perpendicular vector, then let the shader transform them identically to the current arrowhead vertices. This keeps the shader simple and avoids per-pixel NDC arithmetic. The half-width in world units is `0.75 / viewport.scale` (to maintain constant screen-space width).

```rust
// CPU-side quad vertex generation (world-space approach)
fn build_quad_edge(
    buf: &mut Vec<f32>,
    from: (f32, f32), to: (f32, f32),
    half_width: f32,  // world units = 0.75 / scale
    r: f32, g: f32, b: f32, alpha: f32,
) {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let len = (dx*dx + dy*dy).sqrt().max(0.001);
    let px = -dy / len * half_width;  // perpendicular
    let py =  dx / len * half_width;

    // 4 quad corners: from+perp, from-perp, to+perp, to-perp
    // Two triangles: (0,1,2) and (1,3,2) using indices, or 6 verts
    let corners = [
        (from.0 + px, from.1 + py),
        (from.0 - px, from.1 - py),
        (to.0   + px, to.1   + py),
        (to.0   - px, to.1   - py),
    ];
    // Triangle 1: corners[0], corners[1], corners[2]
    // Triangle 2: corners[1], corners[3], corners[2]
    for &(x, y) in &[corners[0], corners[1], corners[2],
                     corners[1], corners[3], corners[2]] {
        push_edge_vertex(buf, x, y, r, g, b, alpha);
    }
}
```

This world-space approach means the existing EDGE_VERT shader needs NO changes — the quad vertices are already in world space, just like arrowhead vertices. The fragment shader also needs no changes. This is the lowest-risk path to quad edges.

The `half_width` in world units = `0.75 / viewport.scale` means the quad width is always 1.5px on screen regardless of zoom. The scale is already available in the draw loop (`viewport.scale as f32`).

**Vertex count change:** Current: 2 floats×vertices per edge. New: 6×vertices (6 verts for 2 triangles per edge segment, same 6 floats per vert). This is a 3× buffer size increase for edges, which is acceptable.

**Draw mode change:** Current `WebGl2RenderingContext::LINES` → `WebGl2RenderingContext::TRIANGLES`.

### Pattern 3: Node fwidth() Anti-Aliasing

Current `NODE_FRAG` uses `smoothstep(0.9, 1.0, d)` which is a fixed 10%-of-radius AA region. At small node sizes (radius=4px), this is a large fraction and blurs visibly. At large zoom the AA region doesn't adapt.

`fwidth(d)` returns the rate of change of `d` per pixel in screen space, which is exactly 1/(screen_radius_in_pixels). Using `fwidth` for the AA band means the band is always 1–2 pixels wide regardless of radius or zoom.

```glsl
// New NODE_FRAG with fwidth AA + border ring + seed color
#version 300 es
precision mediump float;
in vec2 v_local;
in float v_alpha;
in vec3 v_color;
in float v_is_seed;   // 1.0 = seed node, 0.0 = regular
out vec4 fragColor;

void main() {
    float d = length(v_local);

    // Outer ring for seed nodes (d in [0.85, 1.0] with 2px gap → ring outside node)
    // Ring is drawn as separate geometry pass — not in this shader (D-08 pattern)

    // Anti-aliased node fill
    float fw = fwidth(d);
    float edge_aa = 1.0 - smoothstep(1.0 - fw, 1.0, d);
    if (edge_aa < 0.001) discard;

    // Border ring: thin bright band near node edge
    // Border at d in [border_inner, 1.0]
    float border_inner = 1.0 - 2.0 * fw; // ~2px border in screen space
    float border_blend = smoothstep(border_inner - fw, border_inner, d);

    // Brighter border color: mix fill color toward white
    vec3 border_color = mix(v_color, v_color + vec3(0.3, 0.3, 0.3), 1.0);
    // Clamp to avoid overshooting
    border_color = clamp(border_color, 0.0, 1.0);

    vec3 final_color = mix(v_color, border_color, border_blend);
    fragColor = vec4(final_color, v_alpha * edge_aa);
}
```

Note: The exact brighter shade calculation is Claude's discretion. Adding a fixed `vec3(0.3)` to the fill color is a simple approximation — alternatively, multiply the fill color by 1.5 and clamp. Both give a visually lighter ring that matches the fill hue.

### Pattern 4: Seed Node Outer Ring

Decision D-15 specifies a solid outer ring with a 2px transparent gap. This is best implemented as a **separate rendering pass** after all regular nodes (so it draws on top):

For Canvas 2D:
```rust
// After regular node fill and border, for seed nodes:
if node.is_seed {
    let gap = 2.0;
    let ring_r = node.radius + gap + 2.0; // 2px ring, 2px gap
    ctx.begin_path();
    ctx.arc(node.x, node.y, ring_r, 0.0, TAU).unwrap();
    ctx.set_stroke_style_str("#d29922");
    ctx.set_line_width(2.0 / viewport.scale); // screen-space constant
    ctx.stroke();
}
```

For WebGL2, the outer ring can be an additional instanced draw pass using the same quad-based circle geometry, or rendered as a triangle-fan ring using the existing node VAO with a slightly larger radius and a different color. The simplest approach: add a second instance draw for the seed node only with `radius = seed_radius + gap + ring_thickness/2`, using the ring color, and relying on the `fwidth` AA to create a clean circle edge. The gap is achieved by skipping pixels where `d < (seed_radius + gap) / (seed_radius + gap + ring_thickness/2)` in the fragment shader — or by using two draw calls with different radii and `discard` for the gap.

**Recommended simplest approach for WebGL2 seed ring:** After the main instanced node draw, issue a second `draw_arrays_instanced` with a single instance (the seed node), a modified `instance_data` entry with radius = `seed_outer_r` and color = amber, and a modified `NODE_FRAG` that draws only the ring (discard interior). This requires either a separate shader program or a uniform flag.

The lowest-friction option: pass `v_is_seed` through the instance data as an 8th float, and handle the ring entirely in `NODE_FRAG` using `v_is_seed` to switch between modes (fill vs ring). The ring can be drawn by discarding all pixels where `d < ring_inner_norm`, where `ring_inner_norm = (seed_radius + gap) / outer_ring_radius`.

### Pattern 5: NodeState.is_seed Field

`NodeState` needs `is_seed: bool`. It must be set in `from_graph_data()` using the same pattern already in `lod.rs`:

```rust
// In NodeState construction inside from_graph_data():
is_seed: data.seed_paper_id.as_ref().map(|sid| sid == &n.id).unwrap_or(false),
```

This requires `data.seed_paper_id` to be available during node construction. It is — `GraphData` already has `seed_paper_id: Option<String>` (confirmed in `layout_state.rs` line 188: `seed_paper_id: data.seed_paper_id`).

The field must be added to all `NodeState` construction sites — including in test helper functions in `layout_state.rs` and `lod.rs`.

### Pattern 6: Canvas 2D Line Width Scaling for Borders

Decision D-13 requires border width to be constant in screen space. In Canvas 2D, the transform matrix includes scale, so `ctx.set_line_width(w)` draws at `w * scale` pixels on screen. To get 1px screen-space, set `line_width = 1.0 / viewport.scale`.

The `viewport` is passed to `Canvas2DRenderer::draw()` as `&Viewport`, so `viewport.scale` is available in the drawing loop.

### Anti-Patterns to Avoid

- **Mixing NDC and CSS pixels in GLSL:** The current shader transforms world→clip via `(world * scale + offset) / resolution * 2 - 1`. All offset math must stay in the same space. The world-space quad approach (Pattern 2 recommended) sidesteps this entirely.
- **smoothstep with fixed endpoints for AA:** Current `smoothstep(0.9, 1.0, d)` is viewport-blind. Replace with `fwidth(d)` endpoints.
- **Forgetting the arrowhead color for regular edges:** Current Canvas 2D arrowheads use `"#404040"` — this must change to `"#8b949e"` alongside the edge stroke color.
- **Failing to update existing tests when adding `is_seed`:** The `make_node` helpers in `layout_state.rs` and `lod.rs` tests will not compile after adding `is_seed` to `NodeState` without being updated (Rust struct literal completeness requirement).
- **Using TRIANGLE_FAN for quads:** Current node rendering uses `TRIANGLE_FAN` with 4 vertices. For edge quads, `TRIANGLE_FAN` does not produce a correct rectangle from `[-1,-1], [1,-1], [1,1], [-1,1]` if vertices are wound differently. Use `TRIANGLES` with 6 vertices (two explicit triangles) or `TRIANGLE_STRIP` with 4 vertices — both are correct.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Screen-space line width | Custom NDC offset math in vertex shader | World-space quad vertices computed on CPU | Simpler, no shader changes needed, arrowheads already use this pattern |
| Resolution-independent circle AA | Fixed smoothstep band | GLSL `fwidth()` (built-in ES 3.00) | fwidth automatically adapts to zoom/radius, 1–2px band always |
| Color brightening for borders | Custom HSL computation | Add constant vec3 offset + clamp | No need for color space conversion for a subtle highlight |

**Key insight:** The arrowhead geometry is already world-space triangles rendered through the unmodified edge shader. Quad edges can use exactly the same pattern — CPU computes world positions, shader does the same transform. This eliminates the need for any shader changes for edges.

## Runtime State Inventory

Step 2.5: SKIPPED — this is a rendering enhancement phase, not a rename or migration. No stored data, live service config, OS registrations, secrets, or build artifacts contain rendering parameters.

## Common Pitfalls

### Pitfall 1: is_seed Struct Literal Completeness
**What goes wrong:** Adding `is_seed: bool` to `NodeState` causes compile errors in every test file that constructs a `NodeState` struct literal directly.
**Why it happens:** Rust requires all fields in struct literals unless `..default` is used and `Default` is derived.
**How to avoid:** Search for all `NodeState {` literal constructions in test helpers and update them simultaneously. There are `make_node` helpers in `layout_state.rs` (lines 212, 223, 236) and `lod.rs` (line 53) that must be updated.
**Warning signs:** `cargo check` immediately shows "missing field `is_seed`".

### Pitfall 2: arrowhead alpha inconsistency
**What goes wrong:** Edge line color is updated to `#8b949e` but arrowhead code still uses `"#404040"` (Canvas 2D line 191-195) or `edge_color()` in WebGL2 still returns the old color.
**Why it happens:** Arrowheads and edge lines are drawn in separate passes — easy to update one and forget the other.
**How to avoid:** Update `edge_color()` in `webgl_renderer.rs` and the color match in Canvas 2D arrowhead loop together.

### Pitfall 3: WebGL2 vertex count mismatch after quad refactor
**What goes wrong:** `draw_arrays` called with wrong vertex count after changing from 2 vertices/edge to 6 vertices/edge.
**Why it happens:** `draw_edge_pass` computes `vertex_count = data.len() / 6` — this is the number of (x,y,r,g,b,alpha) vertices. With 6 vertices per edge, `data.len() = edges × 6 × 6 floats = edges × 36`. The formula `data.len() / 6` = `edges × 6` = correct vertex count. No change needed to `draw_edge_pass`. However, the draw mode must change from `LINES` to `TRIANGLES`.
**Warning signs:** Visible line artifacts, or half the expected edges visible, or GL error in browser console.

### Pitfall 4: Canvas 2D line_width not accounting for viewport scale
**What goes wrong:** Node border appears thicker when zoomed in, thinner when zoomed out.
**Why it happens:** Canvas 2D transforms include scale, so line_width is in pre-transform (world) units.
**How to avoid:** Set `ctx.set_line_width(1.0 / viewport.scale)` for screen-space-constant borders. The seed ring outer stroke also needs this treatment.

### Pitfall 5: fwidth() precision mode conflict
**What goes wrong:** `fwidth()` produces zero or undefined results.
**Why it happens:** `fwidth` requires standard or high precision. Current `NODE_FRAG` declares `precision mediump float` — this is fine for `fwidth` in WebGL2/GLSL ES 3.00 but worth verifying. If artifacts appear, change to `highp`.
**Warning signs:** Node borders disappear entirely or appear as pure hard cutoffs.

### Pitfall 6: Seed ring gap implementation complexity
**What goes wrong:** The "2px gap" planetary ring effect is overengineered.
**Why it happens:** Attempting to implement the gap in the fragment shader with normalized coordinates across two different radius values is easy to get wrong.
**How to avoid:** The simplest correct implementation is: draw the seed node fill normally (nothing changes), then draw a second stroke-only circle at `radius + gap + ring_half_width` using Canvas 2D `stroke()` or a WebGL2 circle outline with the ring color. The background `#0d1117` fills the gap naturally — no special discard needed.

## Code Examples

### Current vs New Edge Color (Canvas 2D)

```rust
// CURRENT (invisible on dark background):
self.ctx.set_stroke_style_str("#404040");
self.ctx.set_line_width(1.0);
self.ctx.set_global_alpha(0.35);

// NEW (D-01, D-02, D-03):
self.ctx.set_stroke_style_str("#8b949e");
self.ctx.set_line_width(1.5 / viewport.scale);  // D-03, D-13 for screen-space consistency
let alpha = depth_alpha(edge, &state.nodes) * edge_vis_alpha;
self.ctx.set_global_alpha(dim_alpha * alpha);
```

### Current vs New Node Border (Canvas 2D)

```rust
// CURRENT (invisible on dark background):
self.ctx.set_stroke_style_str("#30363d");
self.ctx.set_line_width(1.0);

// NEW (D-10, D-12, D-13):
// Compute a brighter shade: e.g., "#58a6ff" lightened toward white = "#7bbdff"
// Or just use a fixed lighter color per node fill color.
let border_color = brighter(fill_color);  // helper: add ~0x30 to each channel, clamp
self.ctx.set_stroke_style_str(border_color);
self.ctx.set_line_width(1.0 / viewport.scale);  // 1px screen-space
```

### Current vs New NODE_FRAG (WebGL2)

```glsl
// CURRENT (blurry AA):
void main() {
    float d = length(v_local);
    if (d > 1.0) discard;
    float edge = 1.0 - smoothstep(0.9, 1.0, d);
    fragColor = vec4(v_color, v_alpha * edge);
}

// NEW (D-11 fwidth AA + D-10 border ring):
void main() {
    float d = length(v_local);
    float fw = fwidth(d);
    float alpha_mask = 1.0 - smoothstep(1.0 - fw, 1.0 + fw, d);
    if (alpha_mask < 0.001) discard;
    float border_inner = 1.0 - 2.5 * fw;
    float border_blend = smoothstep(border_inner, 1.0 - fw, d);
    vec3 border_color = clamp(v_color * 1.6, 0.0, 1.0);
    vec3 final_color = mix(v_color, border_color, border_blend);
    fragColor = vec4(final_color, v_alpha * alpha_mask);
}
```

### Instance Data Stride Change (WebGL2 Nodes)

```rust
// CURRENT: 7 floats per instance [x, y, radius, alpha, r, g, b]
let stride = 7 * 4;

// NEW: 8 floats per instance [x, y, radius, alpha, r, g, b, is_seed]
let stride = 8 * 4;
instance_data.extend_from_slice(&[
    node.x as f32, node.y as f32, node.radius as f32, alpha, r, g, b,
    if node.is_seed { 1.0 } else { 0.0 },
]);
```

### Quad Edge Buffer Size

```rust
// CURRENT: 2 verts × 6 floats = 12 floats per edge
push_edge_vertex(&mut edge_data, from.x as f32, from.y as f32, r, g, b, alpha);
push_edge_vertex(&mut edge_data, to.x as f32, to.y as f32, r, g, b, alpha);

// NEW: 6 verts × 6 floats = 36 floats per edge (world-space quad, no shader change)
build_quad_edge(
    &mut edge_data,
    (from.x as f32, from.y as f32),
    (to.x as f32, to.y as f32),
    0.75 / scale,  // half-width in world units = 0.75px screen-space
    r, g, b, alpha,
);
// draw mode: TRIANGLES (was LINES)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| smoothstep fixed AA band | fwidth()-based adaptive AA | This phase | Crisp nodes at all zoom levels and DPR |
| GL.LINES (1px cap) | Quad triangles (arbitrary width) | This phase | Proper 1.5px edges on all GPUs |
| Invisible dark borders | Brighter fill-derived border | This phase | Visible node delineation |
| All edges same alpha | Depth-based alpha fade | This phase | Depth signal without color encoding |

**Known limitation of GL.LINES:** The WebGL specification states that the `lineWidth()` function is deprecated and implementations may cap line width at 1px. This is why the quad geometry approach is the standard in production WebGL renderers.

## Open Questions

1. **Arrowhead size at 1.5px edges**
   - What we know: Current `size = 8.0_f64` (world units) was calibrated for 1px-effective edges. D-46 says arrowhead size may need adjustment.
   - What's unclear: Whether 8.0 world units looks visually balanced with 1.5px edge width at default zoom.
   - Recommendation: Keep 8.0 initially; adjust in Claude's discretion if the visual result looks unbalanced. The planner should budget one verification step for this.

2. **Seed ring exact dimensions**
   - What we know: "2px gap, solid ring" (D-15). Ring thickness is Claude's discretion.
   - What's unclear: Whether the ring should scale with node radius (larger seed = larger ring) or be a fixed screen-space thickness.
   - Recommendation: Fixed 2px screen-space ring (consistent visual weight). Planner should spec as `ring_thickness = 2.0 / viewport.scale` world units.

3. **WebGL2 seed ring as separate draw vs shader branch**
   - What we know: D-08 says arrowheads use a separate triangle pass for simplicity.
   - What's unclear: Whether the outer seed ring should follow the same pattern (separate geometry pass) or be handled inside NODE_FRAG via `v_is_seed`.
   - Recommendation: Shader branch in NODE_FRAG (single draw call). The ring is drawn by expanding the effective radius in the shader when `v_is_seed > 0.5`, discarding the gap region. This avoids a second draw call entirely. See Pattern 4 above.

## Environment Availability

Step 2.6: No external dependencies for this phase. All changes are to shader source strings and Rust rendering code compiled to WASM. `cargo check` and `cargo test` are sufficient.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Build | Yes | see rust-toolchain.toml | — |
| cargo test | Validation | Yes | stable | — |
| WebGL2 in browser | Visual testing | Browser-dependent | — | Canvas 2D fallback already exists |

## Validation Architecture

nyquist_validation is enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` (no external test runner) |
| Config file | none |
| Quick run command | `cargo test -p resyn-app -- --nocapture` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| EDGE-01 | Edge color `#8b949e` replaces `#404040` in both renderers | unit | `cargo test -p resyn-app edge_color` | No — Wave 0 |
| EDGE-02 | Quad geometry replaces GL.LINES: `build_quad_edge` emits 6 vertices per edge | unit | `cargo test -p resyn-app build_quad_edge` | No — Wave 0 |
| EDGE-03 | Depth-based alpha: depth-1 gets higher alpha than depth-3 in both renderers | unit | `cargo test -p resyn-app depth_alpha` | No — Wave 0 |
| NODE-01 | NODE_FRAG uses `fwidth()` (verified via shader source string inspection) | unit | `cargo test -p resyn-app node_frag_contains_fwidth` | No — Wave 0 |
| NODE-02 | Border line_width is `1.0 / viewport.scale` not fixed `1.0` | unit | `cargo test -p resyn-app node_border_scales_with_viewport` | No — Wave 0 |
| NODE-03 | `NodeState.is_seed = true` for node matching `seed_paper_id` | unit | `cargo test -p resyn-app is_seed_set_for_seed_paper` | No — Wave 0 |

Note: Shader source string tests (EDGE-01, NODE-01) can assert on string content of `EDGE_FRAG`, `NODE_FRAG` constants. Geometry tests (EDGE-02) can call `build_quad_edge` and assert vertex count. Alpha tests (EDGE-03) can call `depth_alpha` directly. Scale tests (NODE-02) can be integration-style checking the computed value. `is_seed` test (NODE-03) is a unit test on `from_graph_data()` — the same pattern as existing `test_graph_state_seed_paper_id_propagates`.

### Sampling Rate
- **Per task commit:** `cargo check --workspace`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green (255+ tests) before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `resyn-app/src/graph/tests/edge_rendering_tests.rs` — covers EDGE-01, EDGE-02, EDGE-03
- [ ] `resyn-app/src/graph/tests/node_rendering_tests.rs` — covers NODE-01, NODE-02, NODE-03

Alternatively: add tests to existing `#[cfg(test)]` blocks inside `webgl_renderer.rs`, `canvas_renderer.rs`, and `layout_state.rs` — matching the project's current inline test style.

## Sources

### Primary (HIGH confidence)
- Direct code inspection of `resyn-app/src/graph/canvas_renderer.rs` — current edge/node rendering, colors, alpha values
- Direct code inspection of `resyn-app/src/graph/webgl_renderer.rs` — GLSL source strings, vertex buffer format, draw calls
- Direct code inspection of `resyn-app/src/graph/layout_state.rs` — NodeState struct, EdgeData, from_graph_data()
- Direct code inspection of `resyn-app/src/graph/lod.rs` — is_seed pattern for seed detection
- CONTEXT.md decisions D-01 through D-17 — locked implementation choices
- REQUIREMENTS.md EDGE-01 through NODE-03 — requirement definitions
- GLSL ES 3.00 specification — `fwidth()` availability (standard built-in in ES 3.00, declared by existing `#version 300 es`)

### Secondary (MEDIUM confidence)
- WebGL2 `GL.LINES` 1px cap: documented in WebGL specification and widely documented in WebGL rendering tutorials; confirmed by the fact that this is exactly why the codebase note in CONTEXT.md specifies quad geometry as the fix

### Tertiary (LOW confidence)
- Exact `fwidth` behavior with `mediump` precision: generally reliable in practice but not exhaustively tested across GPU vendors in this codebase

## Project Constraints (from CLAUDE.md)

- **No external graph libraries** — full Rust/WASM stack (out-of-scope, not relevant to this phase)
- **DPR convention:** CSS pixels throughout — DPR only at canvas physical sizing and GL viewport. Confirmed in renderer.rs header comment. Border line_width calculations must use `viewport.scale` not `viewport.scale * dpr`.
- **Single async runtime** — not relevant to rendering
- **Rust edition 2024, stable toolchain** — use edition-compatible syntax
- **CI runs clippy with -Dwarnings** — no `#[allow(dead_code)]` on new code; the `quad_buf` has one now but it is an existing exception
- **`cargo fmt --all`** must pass — format new code
- **44 tests baseline** — new tests should not break existing 44; adding is_seed to NodeState will require updating test helper structs in layout_state.rs and lod.rs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, verified existing code
- Architecture: HIGH — patterns derived from direct code inspection
- Pitfalls: HIGH — derived from actual code structure (struct literal completeness, vertex count math)
- Shader patterns: MEDIUM — fwidth behavior is well-established GLSL ES 3.00 but not tested in this specific browser/GPU combination

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable codebase, no fast-moving dependencies)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Edge color is a single muted gray `#8b949e` for all regular citation edges. No color-coding by type.
- **D-02:** Edge alpha fades by BFS depth distance: ~0.5 for depth-1 edges, decreasing to ~0.15 for deepest edges.
- **D-03:** Regular edge line width is 1.5px in both Canvas 2D and WebGL2 renderers.
- **D-04:** Subtle arrowheads at target end of each edge, same color as edge, showing citation direction.
- **D-05:** Contradiction edges (`#f85149`) and ABC-bridge edges (`#d29922`) retain their existing colors and always-opaque alpha.
- **D-06:** Replace `GL.LINES` with quad-based triangle geometry — two triangles per edge segment.
- **D-07:** Fragment shader uses distance-from-center-of-quad for soft anti-aliased edge borders.
- **D-08:** Arrowheads rendered as separate triangle pass (second draw call), not integrated into quad mesh.
- **D-09:** Node fill is flat solid color — no gradient or shading.
- **D-10:** Thin bright border on node circles — a lighter shade of the node fill color.
- **D-11:** WebGL2 node shader uses `fwidth()` for resolution-independent anti-aliasing. Replaces current `smoothstep(0.9, 1.0, d)`.
- **D-12:** Canvas 2D node border updated from current dark `#30363d` to a brighter shade matching WebGL2.
- **D-13:** Border width scaled by inverse viewport scale so it remains 1px screen-space at all zoom levels.
- **D-14:** Seed paper node fill color: warm amber `#d29922`.
- **D-15:** Seed node has a solid amber outer ring with a 2px transparent gap between fill circle and ring.
- **D-16:** Seed node label follows same LOD visibility rules as other nodes (scale > 0.6). No always-on exception.
- **D-17:** `is_seed` flag added to `NodeState` struct, derived from `GraphState.seed_paper_id`.

### Claude's Discretion
- Exact alpha values per BFS depth level (within the 0.15–0.5 range specified)
- Quad edge vertex buffer layout and attribute stride
- fwidth() smoothing parameters for node border anti-aliasing
- Outer ring radius offset and thickness (within the "2px gap, solid ring" constraint)
- Arrowhead size scaling (currently 8.0 world units — may need adjustment for 1.5px edges)
- Exact brighter-shade calculation for node borders

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| EDGE-01 | Regular citation edges visible at-a-glance on the dark (#0d1117) background | Change edge color from `#404040` (barely distinguishable from background) to `#8b949e` with depth-based alpha 0.15–0.5 |
| EDGE-02 | WebGL2 edges rendered via quad-based triangle geometry instead of 1px-capped LINES primitive | Replace `GL.LINES` call in `draw_edge_pass()` with world-space quad vertices (6 verts/edge) and `TRIANGLES` draw mode |
| EDGE-03 | Edge color and alpha consistent between Canvas 2D and WebGL2 renderers | Share `depth_alpha()` logic; both renderers use same color `#8b949e`; arrowhead colors updated together |
| NODE-01 | Node circles sharp at all sizes using resolution-independent anti-aliasing (fwidth in WebGL2) | Replace `smoothstep(0.9, 1.0, d)` in `NODE_FRAG` with `fwidth(d)`-based endpoints |
| NODE-02 | Node borders crisp at all zoom levels (line width scaled by inverse viewport scale) | Set `line_width = 1.0 / viewport.scale` in Canvas 2D; use `fwidth` in WebGL2 for same effect |
| NODE-03 | Seed paper node visually distinct with gold/amber color and outer ring | Add `is_seed: bool` to `NodeState`; amber `#d29922` fill; separate outer ring pass in both renderers |
</phase_requirements>
