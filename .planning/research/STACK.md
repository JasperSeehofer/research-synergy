# Technology Stack

**Project:** Research Synergy (ReSyn) — v1.2 Graph Rendering Overhaul
**Researched:** 2026-03-24
**Scope:** Stack additions and technique recommendations for six specific rendering improvements. Does NOT re-research the validated base stack (Leptos 0.8, web-sys WebGL2, Barnes-Hut resyn-worker, petgraph, SurrealDB).
**Overall confidence:** HIGH for shader techniques and force parameter math. MEDIUM for label collision avoidance implementation details.

---

## No New Crate Dependencies Required

All six target features can be implemented as pure algorithmic and shader improvements within the existing `resyn-worker` and `resyn-app` crates. Adding crates would introduce WASM compilation risk, build complexity, and binary size overhead for problems that are solved by well-understood techniques entirely achievable in safe Rust and GLSL.

The research examined every plausible addition — `rapier2d`, `lyon`, `glam`, `fdg`, `wasm-bindgen-rayon` — and none justify inclusion for this scope.

---

## Recommended Techniques by Feature

### 1. Force Simulation Parameter Tuning

**Problem in current code (`resyn-worker/src/forces.rs`):**

The combination of `ALPHA_DECAY = 0.995`, `ATTRACTION_STRENGTH = 0.03`, and `VELOCITY_DAMPING = 0.6` produces a simulation that spreads nodes but never clusters them. Alpha decays at 0.995 per tick × 60fps = ~200 seconds to reach `ALPHA_MIN` — far too slow for a visible clustering effect in the first viewing. The 0.6 damping is extreme compared to d3-force's default of 0.4 friction (equivalent to 0.6 retained, which would be our `1 - 0.4 = 0.6`... however d3 applies `velocity *= (1 - velocityDecay)` where default decay is 0.4, meaning 0.6 velocity is retained — this matches). The real problem is that `ATTRACTION_STRENGTH = 0.03` with `IDEAL_DISTANCE = 80.0` means the spring force is barely noticeable against the Barnes-Hut repulsion.

**Recommended constant changes (in `resyn-worker/src/forces.rs`):**

| Constant | Current | Recommended | Rationale |
|----------|---------|-------------|-----------|
| `REPULSION_STRENGTH` | -300.0 | -800.0 | Must clearly dominate at short range. Barnes-Hut at THETA=0.9 approximates distant repulsion cheaply so increasing magnitude does not tank performance. |
| `ATTRACTION_STRENGTH` | 0.03 | 0.06 | Doubles spring strength so connected nodes actually cluster at `IDEAL_DISTANCE`. |
| `IDEAL_DISTANCE` | 80.0 | 120.0 | 80px at default viewport scale causes label overlap. 120px provides readable separation. |
| `VELOCITY_DAMPING` | 0.6 | 0.85 | 0.6 retention means momentum dies within 5-6 ticks. 0.85 (matching d3-force effective behavior) preserves convergence trajectory without oscillation. |
| `ALPHA_DECAY` | 0.995 | 0.9975 | 0.995/tick at 60fps ≈ 200s to `ALPHA_MIN`. 0.9975/tick ≈ 90s, matching d3-force's ~300-iteration default at reasonable frame rates. Visible clustering occurs in the first 30s. |
| `THETA` | 0.9 | 0.8 | Tighter Barnes-Hut threshold gives better accuracy on 50-400 node graphs. Performance cost is negligible when running 1 tick per RAF frame on the main thread. |

**Add a node collision force (pure Rust, O(n²) only on close pairs):**

The current simulation has no overlap prevention, so high-degree hub nodes collapse into each other. Add after the Barnes-Hut pass in `simulation_tick`:

```rust
// Node separation force — O(n²) but only fires when nodes are close.
// Only needed for visible nodes; invisible nodes are already dimmed out.
for i in 0..n {
    for j in (i + 1)..n {
        let dx = nodes[j].x - nodes[i].x;
        let dy = nodes[j].y - nodes[i].y;
        let dist2 = dx * dx + dy * dy;
        // nodes[i].radius + nodes[j].radius requires adding radius to NodeData
        let min_dist = 20.0_f64; // placeholder until radius is in NodeData
        if dist2 < min_dist * min_dist && dist2 > 1e-6 {
            let dist = dist2.sqrt();
            let overlap = (min_dist - dist) * 0.5;
            let nx = dx / dist * overlap;
            let ny = dy / dist * overlap;
            if !nodes[i].pinned { forces[i].0 -= nx; forces[i].1 -= ny; }
            if !nodes[j].pinned { forces[j].0 += nx; forces[j].1 += ny; }
        }
    }
}
```

To use actual radii, add `radius: f64` to `NodeData` in `resyn-worker/src/lib.rs` and populate it from `NodeState::radius` when building `LayoutInput`. Then replace `20.0_f64` with `nodes[i].radius + nodes[j].radius + 8.0`.

For 400 nodes this inner loop is 79,800 iterations per tick, most of which short-circuit immediately because `dist2 >= min_dist * min_dist`. Benchmark expectation: <2ms per tick. Acceptable for main-thread layout at 60fps.

**Adaptive post-drag reheat (technique, ~5 lines):**

The mouseup handler already reheats alpha to 0.3. Extend `LayoutInput` with a `post_drag_ticks: u32` field. In `simulation_tick`, if `post_drag_ticks > 0`, temporarily use `-1200.0` repulsion and decrement the counter. This is the ForceAtlas2 pattern for localized rearrangement after a manual node move. Reset to normal repulsion after 20 ticks.

**Confidence:** HIGH — d3-force parameter defaults are documented at https://d3js.org/d3-force/simulation (alphaDecay ≈ 0.0228 per tick maps to ~300 iterations; velocityDecay default 0.4). The relative magnitudes of repulsion vs attraction follow standard Fruchterman-Reingold calibration.

---

### 2. Sharp Circle Rendering in WebGL2

**Problem in current fragment shader (`resyn-app/src/graph/webgl_renderer.rs`, `NODE_FRAG`):**

```glsl
float edge = 1.0 - smoothstep(0.9, 1.0, d);
```

The blend region is fixed at 10% of the circle radius in *local quad space*, regardless of screen size. A radius-4 node at scale=1.0 maps to 4px on screen — the 10% blend is 0.4px, which is sub-pixel and causes aliasing. A radius-18 node maps to 18px — the 10% blend is 1.8px of blurring. Neither is correct.

**Recommended fix — replace with `fwidth`-based adaptive anti-aliasing:**

```glsl
#version 300 es
precision mediump float;
in vec2 v_local;
in float v_alpha;
in vec3 v_color;
in float v_is_seed;    // NEW: 0.0 = normal, 1.0 = seed node
out vec4 fragColor;

void main() {
    float d = length(v_local);
    float fw = fwidth(d);   // screen-space derivative: ~1px in screen space
    float edge = 1.0 - smoothstep(1.0 - fw, 1.0 + fw, d);

    vec3 color = v_color;

    // Seed node gold ring at node border
    if (v_is_seed > 0.5) {
        float ring_outer = 1.0;
        float ring_inner = 1.0 - 3.0 * fw;  // ~1.5px ring width
        float ring = smoothstep(ring_inner - fw, ring_inner + fw, d)
                   * (1.0 - smoothstep(ring_outer - fw, ring_outer + fw, d));
        color = mix(color, vec3(0.96, 0.65, 0.14), ring);  // #f5a623 gold
    }

    fragColor = vec4(color, v_alpha * edge);
}
```

`fwidth(d)` returns `abs(dFdx(d)) + abs(dFdy(d))` — the maximum change in the normalized distance across adjacent pixels in screen space. When the circle is small (zoomed out), `fw` is larger, giving smooth anti-aliasing. When the circle is large (zoomed in), `fw` is tiny, giving a razor-sharp edge. This is viewport-scale-adaptive with no uniforms needed.

`fwidth` is in the GLSL ES 3.00 core spec — available in all WebGL2 contexts without any extension. The existing `#version 300 es` header already enables it. `mediump` precision is sufficient because `v_local` is in [0,1] and the derivative is a screen-space quantity well within mediump range.

The `v_is_seed` varying requires:
1. Adding `a_is_seed: f32` to the per-instance data (stride expands from 7 to 8 floats)
2. Passing it through the vertex shader: `out float v_is_seed; ... v_is_seed = a_is_seed;`
3. Identifying the seed node when building instance data: compare `node.id == graph.seed_paper_id`

**Confidence:** HIGH — fwidth is GLSL ES 3.00 core spec, universally supported in WebGL2. The mathematical correctness is established across multiple shader references.

---

### 3. Edge Rendering Visibility

**Problem:** Regular edge color is `#404040` at alpha 0.35 on background `#0d1117`. Perceptual luminance contrast is ~12% — effectively invisible on non-retina displays, and still weak on retina. The `LINES` WebGL2 primitive is capped at 1px width on Chrome/ANGLE regardless of `gl.lineWidth()` calls.

**Fix 1 — Color and alpha (zero-effort, immediate improvement):**

```rust
// In edge_color() in webgl_renderer.rs:
EdgeType::Regular => {
    let (r, g, b) = hex_to_rgb("#5a6475");  // was #404040; blue-grey matches GitHub dark palette
    let alpha = if both_dimmed { 0.08 } else { 0.50 };  // was 0.1 / 0.35
    (r, g, b, alpha)
}
```

This doubles perceived contrast with a single constant change. Do this first; it may be sufficient.

**Fix 2 — Quad-based edges for sub-pixel thickness control (medium effort):**

Chrome/ANGLE enforces `gl.lineWidth(1)` regardless of input. To draw 1.5-2px edges, render each edge as a screen-aligned quad (two triangles, 6 vertices) instead of a `LINES` primitive. The vertex shader extrudes the line in screen space:

```glsl
// Edge quad vertex shader (replace current EDGE_VERT):
in vec2 a_from;       // world pos of edge start
in vec2 a_to;         // world pos of edge end
in float a_side;      // -1.0 or +1.0
in vec3 a_color;
in float a_alpha;
uniform vec2 u_resolution;
uniform vec2 u_offset;
uniform float u_scale;
uniform float u_line_half_width;  // e.g. 0.75 for 1.5px

out vec3 v_color;
out float v_alpha;

void main() {
    vec2 from_screen = (a_from * u_scale + u_offset) / u_resolution * 2.0 - 1.0;
    vec2 to_screen   = (a_to   * u_scale + u_offset) / u_resolution * 2.0 - 1.0;

    // Determine which endpoint this vertex represents based on its index (0..5)
    bool is_from = gl_VertexID % 3 == 0 || (gl_VertexID % 6 == 4);
    vec2 base = is_from ? from_screen : to_screen;

    vec2 dir = normalize(to_screen - from_screen);
    vec2 perp = vec2(-dir.y, dir.x);
    vec2 pixel_offset = perp * u_line_half_width * 2.0 / u_resolution;
    gl_Position = vec4(base + a_side * pixel_offset, 0.0, 1.0);

    v_color = a_color;
    v_alpha = a_alpha;
}
```

The data layout change: each edge becomes 6 vertices instead of 2. For 500 edges, buffer size triples from ~12KB to ~36KB — negligible. Arrowheads continue to use the existing `TRIANGLES` path unchanged.

**Recommendation:** Apply Fix 1 first (color change, 2 lines). Only implement Fix 2 if 1px lines remain unacceptably thin after testing on target hardware.

**Confidence:** HIGH for color change. MEDIUM for quad approach (well-established WebGL pattern per Cesium blog; requires vertex buffer refactor).

---

### 4. Auto-Fit Viewport After Layout Stabilization

**Problem:** Initial scale is computed from pre-simulation spread: `(css_width.min(css_height) * 0.4 / spread).min(1.0)`. After the simulation runs for 30-90 seconds, nodes may be well outside this viewport. Users must manually zoom/pan to see the full graph.

**Recommended implementation (no new crates, ~40 lines):**

Add `fit_triggered: bool` to `RenderState`. In the RAF loop, after the simulation tick, check if alpha has crossed `ALPHA_MIN * 10.0` for the first time and trigger a fit:

```rust
// In start_render_loop frame closure, after simulation tick:
if sim_running
    && !s.fit_triggered
    && s.graph.alpha <= resyn_worker::forces::ALPHA_MIN * 10.0
    && !s.graph.nodes.is_empty()
{
    fit_viewport_to_nodes(&mut s.viewport, &s.graph.nodes);
    s.fit_triggered = true;
}
```

The fit function computes the axis-aligned bounding box of all visible nodes, then sets scale and offset to center the bounding box with a 5% margin:

```rust
fn fit_viewport_to_nodes(viewport: &mut Viewport, nodes: &[NodeState]) {
    let visible: Vec<_> = nodes.iter()
        .filter(|n| n.lod_visible && n.temporal_visible)
        .collect();
    if visible.is_empty() { return; }

    let min_x = visible.iter().map(|n| n.x - n.radius).fold(f64::INFINITY, f64::min);
    let max_x = visible.iter().map(|n| n.x + n.radius).fold(f64::NEG_INFINITY, f64::max);
    let min_y = visible.iter().map(|n| n.y - n.radius).fold(f64::INFINITY, f64::min);
    let max_y = visible.iter().map(|n| n.y + n.radius).fold(f64::NEG_INFINITY, f64::max);

    let graph_w = (max_x - min_x).max(1.0);
    let graph_h = (max_y - min_y).max(1.0);

    let margin = 0.90; // 5% padding on each side
    let scale = (viewport.css_width / graph_w)
        .min(viewport.css_height / graph_h)
        * margin;
    let cx = (min_x + max_x) / 2.0;
    let cy = (min_y + max_y) / 2.0;

    viewport.scale = scale.clamp(0.1, 5.0);
    viewport.offset_x = viewport.css_width / 2.0 - cx * viewport.scale;
    viewport.offset_y = viewport.css_height / 2.0 - cy * viewport.scale;
}
```

Reset `fit_triggered = false` when new graph data is loaded. This is already handled: `Effect::new` rebuilds `RenderState` fresh on data arrival.

The existing double-click handler already resets the viewport to `Viewport::new(w, h)` — this should be updated to call `fit_viewport_to_nodes` instead so double-click becomes "fit to graph" rather than "reset to default".

**Confidence:** HIGH — AABB viewport fit is mathematically elementary and universally used (Cytoscape.js `fit()`, sigma.js `camera.animatedReset()`, Gephi zoom-to-fit). The alpha threshold for trigger timing is heuristic but well-motivated.

---

### 5. Label Collision Avoidance

**Problem:** Labels are drawn at a fixed offset below each node with no overlap checking. When nodes cluster, labels pile up and become illegible. The Canvas 2D renderer draws all labels unconditionally when `scale > 0.6`; the WebGL renderer exposes positions via `node_screen_positions()` for a separate HTML overlay.

**Recommended approach — greedy occupancy bitmap (pure Rust, WASM-safe):**

Labels are approximately 70×12 CSS pixels at 11px monospace. Use a 4px grid cell occupancy map.

```rust
// In canvas_renderer.rs, replace the label drawing loop:
fn draw_labels_with_collision(
    ctx: &CanvasRenderingContext2d,
    nodes: &[NodeState],
    viewport: &Viewport,
) {
    let cell = 4.0_f64;
    let grid_w = (viewport.css_width / cell).ceil() as usize + 1;
    let grid_h = (viewport.css_height / cell).ceil() as usize + 1;
    let mut occupied = vec![false; grid_w * grid_h];

    // Mark node circles as occupied
    for n in nodes.iter().filter(|n| n.lod_visible && n.temporal_visible) {
        let (sx, sy) = viewport.world_to_screen(n.x, n.y);
        let r = n.radius * viewport.scale;
        let x0 = ((sx - r) / cell).floor() as isize;
        let y0 = ((sy - r) / cell).floor() as isize;
        let x1 = ((sx + r) / cell).ceil() as isize;
        let y1 = ((sy + r) / cell).ceil() as isize;
        mark_rect(&mut occupied, x0, y0, x1, y1, grid_w, grid_h);
    }

    ctx.set_font("11px monospace");
    ctx.set_fill_style_str("#cccccc");

    for n in nodes.iter().filter(|n| n.lod_visible && n.temporal_visible) {
        let label = n.label();
        let lw = ctx.measure_text(&label).map(|m| m.width()).unwrap_or(50.0);
        let lh = 14.0_f64;
        let (sx, sy) = viewport.world_to_screen(n.x, n.y);
        let r = n.radius * viewport.scale;

        // Candidate positions: below, above, right, left
        let candidates = [
            (sx - lw / 2.0, sy + r + 4.0),       // below
            (sx - lw / 2.0, sy - r - lh - 2.0),  // above
            (sx + r + 4.0,  sy - lh / 2.0),       // right
            (sx - r - lw - 4.0, sy - lh / 2.0),  // left
        ];

        for (lx, ly) in candidates {
            let x0 = (lx / cell).floor() as isize;
            let y0 = (ly / cell).floor() as isize;
            let x1 = ((lx + lw) / cell).ceil() as isize;
            let y1 = ((ly + lh) / cell).ceil() as isize;

            if !any_occupied(&occupied, x0, y0, x1, y1, grid_w, grid_h) {
                ctx.fill_text(&label, lx, ly + lh - 2.0).ok();
                mark_rect(&mut occupied, x0, y0, x1, y1, grid_w, grid_h);
                break;
            }
        }
        // If all candidates are occupied, skip the label (too crowded at this scale)
    }
}
```

The `mark_rect` and `any_occupied` helpers are simple loops over grid cells with bounds clamping. At 1200×800 with cell=4, the grid is 300×200 = 60,000 bools (~60KB stack, acceptable). For 400 visible nodes with 4 candidates each, the inner check loop runs at most 400×4×(70/4×14/4) ≈ 400×4×61 ≈ 97,600 cell checks. Under 1ms on WASM at typical clock speeds.

For the **WebGL renderer** (labels rendered as HTML overlay via `node_screen_positions()`): apply the same occupancy logic in a new helper function that takes the Vec<(f64, f64, String)> from `node_screen_positions()` and returns a filtered/repositioned Vec. The overlay rendering in `graph.rs` already renders labels based on this data.

The `web_sys::TextMetrics` API is already imported in `resyn-app/Cargo.toml` (`"TextMetrics"` feature present).

**Confidence:** MEDIUM — the algorithm is well-established (Vega/Vega-Lite occupancy bitmap paper: https://idl.cs.washington.edu/files/2021-FastLabels-VIS.pdf). The WASM implementation is straightforward but involves some vec allocation per frame. Profile to verify it stays under 2ms on large graphs before enabling by default.

---

### 6. Seed Node Visual Distinction

**Problem:** `seed_paper_id` is already present in `GraphState`. Neither renderer visually distinguishes the seed node. Users cannot tell where the BFS crawl originated.

**WebGL renderer implementation:**

Expand instance data from 7 to 8 floats per instance: add `is_seed: f32` (0.0 or 1.0). The stride becomes 8×4 = 32 bytes.

In the vertex shader, add:
```glsl
in float a_is_seed;
out float v_is_seed;
// in main(): v_is_seed = a_is_seed;
```

The fragment shader uses `v_is_seed` for the gold ring as shown in Section 2.

When building instance data in `webgl_renderer.rs`:
```rust
let is_seed = state.seed_paper_id.as_deref()
    .map(|sid| if node.id == sid { 1.0_f32 } else { 0.0_f32 })
    .unwrap_or(0.0_f32);

instance_data.extend_from_slice(&[
    node.x as f32, node.y as f32, node.radius as f32,
    alpha, r, g, b,
    is_seed,  // NEW 8th component
]);
```

Update the `a_alpha` and `a_color` attribute pointer offsets accordingly (offsets increase by 0 for position/radius/alpha, unchanged; `a_color` moves from offset 4×4 to 4×4 as before; add new `a_is_seed` at offset 7×4).

**Canvas 2D renderer implementation:**

After drawing the node circle and before `ctx.restore()`:

```rust
let is_seed = state.seed_paper_id.as_deref()
    .map(|sid| node.id == sid)
    .unwrap_or(false);

if is_seed {
    self.ctx.begin_path();
    self.ctx
        .arc(node.x, node.y, node.radius + 5.0, 0.0, std::f64::consts::TAU)
        .unwrap();
    self.ctx.set_stroke_style_str("#f5a623"); // gold
    self.ctx.set_line_width(2.5);
    self.ctx.set_global_alpha(1.0);
    self.ctx.stroke();
}
```

The seed lookup is one string comparison per node per frame (400 comparisons ≈ 5µs — negligible).

**Confidence:** HIGH — purely additive rendering logic with no structural changes. The visual design (gold ring) is a common pattern in graph visualization tools to indicate a focus/root node.

---

## Implementation Order

These six features have a natural dependency ordering:

| Order | Feature | Why This Position |
|-------|---------|-------------------|
| 1 | Sharp circles (fwidth shader) | One-line shader change; zero risk; improves everything else |
| 2 | Seed node distinction | Piggybacks on shader change in step 1; add `a_is_seed` to instance data at the same time |
| 3 | Force parameter tuning | Foundational: everything looks better once nodes actually cluster |
| 4 | Edge visibility (color change) | Two-line constant change; do after layout looks correct |
| 5 | Auto-fit viewport | Meaningful only after layout stabilizes from step 3 |
| 6 | Label collision avoidance | Most code; stable layout from step 3 required for sensible label positions |

---

## Full Stack Summary

| Component | Technology | Version | Changes for v1.2 |
|-----------|-----------|---------|-----------------|
| WASM runtime | wasm-bindgen | 0.2.x | None |
| UI framework | Leptos CSR | 0.8 | None |
| Force simulation | resyn-worker custom | internal | Constant tuning + collision force + radius in NodeData |
| WebGL2 renderer | web-sys WebGL2 | 0.3.x | Fragment shader (fwidth + seed ring) + instance stride 7→8 |
| Canvas 2D renderer | web-sys Canvas 2D | 0.3.x | Label collision algorithm + seed ring drawing |
| Label measurement | web-sys TextMetrics | 0.3.x | Already available — use in collision avoidance |
| Graph data structures | petgraph | 0.7.0 | None |
| Build / CI | Trunk, cargo | existing | None |

## What NOT to Add

| Candidate | Why Not |
|-----------|---------|
| `rapier2d` | Full 2D physics for node collision is overkill; a 30-line per-frame force term is sufficient and has no WASM compilation risk |
| `lyon` (tessellation) | Needed for curved/thick paths via CPU tessellation; our edges are straight and the quad approach is trivial without a crate |
| `glam` / `nalgebra` | 2D graph math is scalar arithmetic; no matrix operations needed |
| `fdg` crate | Already replaced by custom Barnes-Hut in resyn-worker; would duplicate the simulation |
| `wasm-bindgen-rayon` | Requires SharedArrayBuffer + COOP/COEP headers; significant server config change for marginal gain when 1 tick/frame is already fast enough |
| `gloo-timers` | Not needed; settle detection is fully handled by alpha threshold comparison |
| Any JS graph library | Explicitly out of scope per PROJECT.md |

---

## Sources

- D3-force simulation alpha/cooling defaults: [https://d3js.org/d3-force/simulation](https://d3js.org/d3-force/simulation)
- ForceAtlas2 adaptive temperature: [https://medialab.sciencespo.fr/publications/Jacomy_Heymann_Venturini-Force_Atlas2.pdf](https://medialab.sciencespo.fr/publications/Jacomy_Heymann_Venturini-Force_Atlas2.pdf)
- Node collision forces in force-directed graphs: [https://tomroth.dev/fdg-collision/](https://tomroth.dev/fdg-collision/)
- fwidth anti-aliasing technique: [http://www.numb3r23.net/2015/08/17/using-fwidth-for-distance-based-anti-aliasing/](http://www.numb3r23.net/2015/08/17/using-fwidth-for-distance-based-anti-aliasing/)
- GLSL fwidth for circles: [https://rubendv.be/posts/fwidth/](https://rubendv.be/posts/fwidth/)
- WebGL LINES 1px cap (Chrome/ANGLE): [https://github.com/processing/p5.js/issues/6091](https://github.com/processing/p5.js/issues/6091)
- Instanced quad line rendering: [https://wwwtyro.net/2019/11/18/instanced-lines.html](https://wwwtyro.net/2019/11/18/instanced-lines.html)
- Label occupancy bitmap algorithm: [https://idl.cs.washington.edu/files/2021-FastLabels-VIS.pdf](https://idl.cs.washington.edu/files/2021-FastLabels-VIS.pdf)
- GLSL ES 3.00 spec (fwidth in core): GLSL ES 3.00 specification Section 8.13

---

*Stack research for: ReSyn v1.2 — Graph Rendering Overhaul*
*Researched: 2026-03-24*
