# Domain Pitfalls

**Domain:** Rust/WASM WebGL2 + Canvas 2D force-directed graph rendering overhaul
**Researched:** 2026-03-24
**Confidence:** HIGH for WebGL line/AA fundamentals (MDN + WebGL2Fundamentals verified); HIGH for DPR coordinate space (Khronos wiki + WebGL2Fundamentals verified); MEDIUM for force coefficient tuning (academic literature + community patterns); MEDIUM for label collision in Canvas/WebGL hybrid

---

## Critical Pitfalls

These mistakes require rewrites or cause visually broken output that is hard to debug.

---

### Pitfall 1: Fixed smoothstep Delta Produces Blurry Nodes at Large Radii

**What goes wrong:**
The current fragment shader uses `smoothstep(0.9, 1.0, d)` with a hardcoded delta of 0.1 across the [-1, 1] quad. At small node radii (4 px CSS) this delta corresponds to ~0.4 physical pixels — adequate. At large radii (18 px CSS at DPR 2 = 36 physical pixels), that same 0.1 in unit-quad space corresponds to 3.6 physical pixels of feathering, producing a visibly fuzzy halo. The issue is scale-invariant smoothstep applied to scale-variant nodes.

**Why it happens:**
The quad coordinate `v_local` ranges over [-1, 1] regardless of the node's physical pixel size. The smoothstep transition width in quad-space maps to a different number of physical pixels depending on the rendered radius. Big nodes get blurry; small nodes get aliased hard edges.

**Consequences:**
Seed node (typically high-citation, large radius) looks most blurry — exactly the node that should stand out most. Visual quality degrades precisely where it matters.

**Prevention:**
Replace the fixed delta with `fwidth(d)` (or `fwidth(length(v_local))` in the fragment shader), which returns the screen-space derivative of the distance field — one physical pixel of transition regardless of radius. Pattern:
```glsl
float d = length(v_local);
float anti_alias = fwidth(d);
float edge = 1.0 - smoothstep(1.0 - anti_alias, 1.0, d);
fragColor = vec4(v_color, v_alpha * edge);
```
`fwidth` requires no WebGL extension in WebGL2; it is part of the core spec. Verify `OES_standard_derivatives` is not needed — in WebGL2 it is available unconditionally.

**Detection:**
Zoom in on a large-radius node at DPR 2. If the edge looks wider than one physical pixel it is blurry. Zoom out — small nodes should have crisp (but not jagged) edges.

**Phase:** Graph rendering overhaul phase — fix before visual acceptance testing.

---

### Pitfall 2: WebGL2 LINES Primitive is Effectively 1 Physical Pixel — Not Configurable

**What goes wrong:**
The current edge renderer uses `gl.draw_arrays(LINES, ...)`. `gl.line_width()` exists in the API but the WebGL2 specification allows implementations to clamp it to 1.0. In Chrome and Firefox on all platforms, lines are always 1 pixel wide regardless of the value passed to `gl.line_width()`. On the dark `#0d1117` background at 0.35 alpha, 1-pixel lines at `#404040` render at effective luminance near zero — invisible without close inspection.

**Why it happens:**
This is a WebGL spec-level constraint, not a usage bug. The `LINES` primitive was effectively deprecated for anything beyond debugging tools. The [WebGL2 Anti-Patterns](https://webgl2fundamentals.org/webgl/lessons/webgl-anti-patterns.html) documentation explicitly states maximum line thickness is 1.0.

**Consequences:**
All citation edges are invisible or near-invisible on the dark background regardless of the color value. The graph appears to have no structure. This is the single most visible bug at v1.2.

**Prevention:**
Replace `LINES` with geometry-shader-equivalent quad triangles: for each edge, emit two triangles forming a screen-space quad of configurable width along the edge direction. This is the standard approach and has no hardware limitation. Width of 1.5–2 CSS pixels is adequate for citation edges; 2.5 CSS pixels for semantic edges (contradiction/bridge).

For the arrowhead triangles already drawn with `TRIANGLES`, this pattern is already established in the codebase — extend the same approach to the edge body.

**Detection:**
Open the graph on a dark background. If edges are invisible at default zoom, the `LINES` primitive width limit is the cause. Inspect the WebGL draw calls in the browser; any `lineWidth` call above 1.0 is a no-op signal.

**Phase:** Graph rendering overhaul phase — change before any other edge visibility tuning; edge color choice is irrelevant until width is fixed.

---

### Pitfall 3: Force Coefficient Imbalance Collapses Graph or Scatters Nodes to Infinity

**What goes wrong:**
Two failure modes:

**Mode A — Collapse:** ATTRACTION_STRENGTH too high relative to REPULSION_STRENGTH causes all nodes to cluster at one point. The graph looks like a blob. With IDEAL_DISTANCE=80 and ATTRACTION_STRENGTH=0.03, any pair of connected nodes closer than 80 px experiences a net attractive force. For a dense citation graph (many edges per node), the sum of attractive forces overwhelms repulsion, collapsing connected subgraphs.

**Mode B — Scatter:** REPULSION_STRENGTH too large or VELOCITY_DAMPING too high causes nodes to fly off-screen before the simulation cools. The initial layout spread is `sqrt(n) * 15` world units; at 100 nodes that is 150 world units. With REPULSION=-300, two nodes at distance 1 unit (possible in early ticks) receive force magnitude 300 — which with alpha=1.0 and VELOCITY_DAMPING=0.6 produces runaway velocity.

**Why it happens:**
The three force constants (REPULSION, ATTRACTION, CENTER_GRAVITY) are not independently tunable; they interact through the alpha decay schedule. The same absolute constant values produce different behavior at different graph densities because repulsion sums over all pairs (O(n²) force total) while attraction only acts on edges (O(edges) force total). A sparse graph behaves differently from a dense one at identical constants.

**Consequences:**
Graph either shows no structure (blob) or nodes disappear off-screen (scatter). Both are invisible to normal debugging until you see the rendered output.

**Prevention:**
Normalise forces by graph density before tuning:
- After any coefficient change, test with both a sparse graph (chain topology, 20 nodes) and a dense graph (complete graph, 20 nodes) and verify both converge.
- Scale IDEAL_DISTANCE with approximate graph diameter. For a BFS-crawled citation graph at depth 3, 80 px is reasonable. If spread looks too tight, raise IDEAL_DISTANCE (not REPULSION).
- VELOCITY_DAMPING=0.6 is aggressive. Values below 0.5 allow oscillation to persist longer; values above 0.85 converge too slowly. The canonical D3 default is 0.6 alpha-decay equivalent — keep within 0.5–0.7.
- Clamp velocity per tick to prevent runaway: `vel = vel.clamp(-MAX_VEL, MAX_VEL)` where MAX_VEL ≈ IDEAL_DISTANCE / 2. This prevents the scatter failure mode without affecting convergence quality.

**Detection:**
After 500 ticks from a fresh layout, compute the bounding box of all node positions. If it is larger than 5x the viewport, scatter is occurring. If all nodes are within 2x IDEAL_DISTANCE of each other, collapse is occurring. Both are testable in the existing forces test suite.

**Phase:** Graph rendering overhaul phase — test coefficient changes with both topology types before committing.

---

### Pitfall 4: Auto-Fit Viewport Computed Before Simulation Stabilises

**What goes wrong:**
If `fit_scale` is computed from node positions at time T=0 (the initial circular layout with jitter), the scale is calibrated to the initial spread. After the simulation runs for 200+ frames, connected clusters have pulled together and the bounding box is much smaller — the user sees an over-zoomed-out view with all nodes packed in a small region. Conversely, if the graph scatters before the viewport fits, the fit catches the scattered state and the user sees an over-zoomed-in view with most nodes off-screen.

**Current state:** The codebase already does `viewport.scale = (css_width.min(css_height) * 0.4 / spread).min(1.0)` at T=0 using the initial mathematical spread estimate. This is computed before any simulation ticks run, so it may not match the settled bounding box.

**Why it happens:**
The force simulation is non-deterministic in convergence time. There is no signal from the simulation that it has "settled enough to fit". Computing fit at T=0 is deterministic but wrong for the converged state.

**Consequences:**
On graph load the user sees nodes far too small (if initial spread was wide) or too large (if graph is small). Either requires manual zoom to find the actual content.

**Prevention:**
Implement a deferred fit: track whether an initial auto-fit has been applied. On the first frame after simulation alpha drops below a threshold (e.g., 0.1 — 70% cooled), compute the actual bounding box from current node positions, add 10% padding, and interpolate the viewport to fit. Do not re-run auto-fit on subsequent frames unless explicitly triggered by a "fit to screen" button.

The bounding box computation is O(n) — safe to run inline in the RAF loop.

```rust
// In the RAF tick, after running simulation:
if graph.alpha < 0.1 && !has_auto_fit {
    let bbox = compute_bounding_box(&graph.nodes);
    viewport.fit_to_bbox(bbox, padding_fraction: 0.1);
    has_auto_fit = true;
}
```

**Detection:**
Load a fresh 50-node graph. After 5 seconds observe whether all nodes are visible in the default viewport without any user zoom action. If zoom-in or zoom-out is needed to see all nodes, auto-fit is misfiring.

**Phase:** Graph rendering overhaul phase — implement deferred fit as part of the viewport work, not as a premature optimization.

---

### Pitfall 5: Label Rendering on Canvas 2D Overlay Misaligned When DPR != 1

**What goes wrong:**
Labels are rendered in Canvas 2D over the WebGL canvas. The `node_screen_positions` method on `WebGL2Renderer` returns screen positions using `viewport.world_to_screen(n.x, n.y)` — which computes positions in CSS pixel space. If the Canvas 2D overlay uses a different coordinate system (e.g., physical pixels via a `setTransform(dpr, 0, 0, dpr, 0, 0)` call), labels appear shifted by a factor of `dpr` relative to node centers.

**Current state:** `Viewport::apply()` multiplies by DPR inside the Canvas 2D transform: `ctx.set_transform(self.scale * dpr, ..., self.offset_x * dpr, self.offset_y * dpr)`. If labels are drawn in a separate pass that also calls `apply()`, label origin coordinates would need to be in world space. If labels use the screen positions from `node_screen_positions` (CSS pixels) and the Canvas 2D context has a DPR-scaled transform applied, they will be placed at `css_x * dpr` — double-scaled.

**Why it happens:**
The DPR convention documented in `renderer.rs` says "CSS pixels throughout; DPR only at canvas physical sizing and GL viewport." However, `Viewport::apply()` for Canvas 2D sets a transform that includes DPR, making the Canvas 2D coordinate system physical-pixel-space, not CSS-pixel-space. Any code that mixes `world_to_screen` output (CSS) with a DPR-scaled Canvas 2D context will be off by a factor of DPR.

**Consequences:**
On HiDPI displays (DPR=2), labels appear 2x too far from node centers. On standard displays (DPR=1) the bug is invisible, making it easy to miss in development.

**Prevention:**
Choose one of two consistent approaches and document it explicitly:
- **Approach A (CSS-only):** Canvas 2D for labels does NOT apply DPR in its transform. Draw labels using CSS pixel coordinates. Text appears at the correct visual position; the OS composites the canvas at the right physical size.
- **Approach B (Physical):** Canvas 2D applies DPR transform. Convert all CSS-pixel screen positions to physical pixels before drawing (`css_pos * dpr`). The `world_to_screen` helper must not be used for this path, or a separate `world_to_physical_screen` must be added.

The convention already documents Approach A as the intended pattern. Verify that any new label-rendering code uses `world_to_screen` output directly without further DPR multiplication.

**Detection:**
Render on a HiDPI display (or emulate DPR=2 in Chrome DevTools device toolbar). Check if label text center aligns with node center. If labels are displaced to bottom-right by exactly `radius * (dpr - 1)` pixels, the double-DPR bug is present.

**Phase:** Graph rendering overhaul phase, during label collision work — DPR alignment must be correct before label positions can be meaningfully tested.

---

## Moderate Pitfalls

These produce visible problems but do not require architecture changes to fix.

---

### Pitfall 6: Label Collision Check Running Every Frame is O(n²)

**What goes wrong:**
A naive label collision avoidance algorithm compares every label bounding box against every other, which is O(n²) per frame. At 300 nodes with 60 FPS, that is 300² × 60 = 5.4M comparisons per second in WASM. This is measurably slow and the LOD system already hides labels below a zoom threshold — the cost is paid even when no labels should be shown.

**Prevention:**
- Only run collision detection when the label set changes: on zoom level change (which triggers LOD updates), not every tick.
- Use a simple spatial grid (divide screen into cells of label-width size) for O(n) average-case collision checks instead of O(n²). For the expected label count (< 100 visible at any zoom level), a greedy greedy linear sweep sorted by priority (seed > high-citation > low-citation) is sufficient.
- Do not render labels below a minimum zoom threshold; this is already implemented via LOD. Ensure the collision check is gated behind the same threshold.
- Cache the last computed visible label set in a `Vec<usize>` and only recompute on viewport change.

**Phase:** Graph rendering overhaul phase — implement collision avoidance with the spatial grid from the start; do not ship the O(n²) version as a "we'll fix it later" shortcut.

---

### Pitfall 7: Alpha Floor Keeps Simulation Hot and Wastes CPU After Convergence

**What goes wrong:**
The RAF loop forces `alpha = output.alpha.max(ALPHA_MIN)` — alpha never drops below 0.001. This is intentional for responsiveness (dragging a node should cause rearrangement), but it means Barnes-Hut runs every frame indefinitely, even when the graph has visually settled. On a 400-node graph, a Barnes-Hut tick takes ~2 ms; at 60 FPS this is 120 ms/s of wasted CPU time on an idle settled graph.

**Prevention:**
Distinguish between "simulation running due to user interaction" and "simulation running to convergence". After alpha drops to ALPHA_MIN naturally and no user interaction has occurred for N seconds, pause the RAF tick's simulation call (not the RAF loop itself — rendering must continue for interaction feedback). Resume on any drag, pin, or toggle event.

Do not gate the entire RAF loop on simulation state — the renderer must continue to respond to viewport pan/zoom even when forces are off.

**Phase:** Graph rendering overhaul phase — implement the idle pause as part of `simulation_running` state logic; it is already exposed as a `RwSignal<bool>`.

---

### Pitfall 8: Renderer Switch Threshold at 300 Nodes Causes Context Re-creation on Resize

**What goes wrong:**
`make_renderer` chooses Canvas 2D or WebGL2 based on `node_count` at creation time. If the graph has exactly 300 nodes and the user resizes the browser causing `setup_resize_observer` to fire, the resize handler calls `renderer.resize()` but does not re-evaluate which renderer to use. The renderer type is fixed at creation. This is fine. But if `make_renderer` were ever called again on the same canvas after the context was already set to "2d", `get_context("webgl2")` would return null — the browser does not allow switching context types on an existing canvas.

**Prevention:**
Never call `make_renderer` after the initial setup. The ResizeObserver handler correctly calls `renderer.resize()` only. Verify that no code path calls `make_renderer` on graph data reload (e.g., if the server function is re-fetched). The `graph_resource` Effect currently creates the renderer once inside the Effect body — this is correct since Effects re-run when their dependencies change. If the graph resource refreshes (e.g., a second `get_graph_data()` call), the Effect will re-run and create a new renderer on the same canvas element, which will fail if the canvas already has a "2d" context.

The safest fix: store the renderer in a long-lived ref outside the Effect and only create it once, or gate renderer creation on canvas context not yet being claimed.

**Phase:** Graph rendering overhaul phase — validate that hot-reload of graph data (server function re-call) does not trigger renderer re-creation.

---

### Pitfall 9: Center Gravity Constant Fights the Auto-Fit Viewport Offset

**What goes wrong:**
CENTER_GRAVITY pulls all nodes toward (0, 0) in world space. The `Viewport` centers at (0, 0) via `offset = (css_width/2, css_height/2)` — so world origin maps to the viewport center. This is consistent. But if the seed node is pinned or given a strong attractor force to appear at the center, and other nodes are simultaneously being repelled, the graph can develop a stable configuration where high-degree nodes cluster around the seed but isolated papers drift to the edge of the graph. The auto-fit viewport then has to accommodate both the tight cluster and the distant outliers, producing a scale so small the cluster is illegible.

**Prevention:**
Do not pin the seed node during the simulation unless the user explicitly drags and pins it. A visual seed distinction (ring, color, larger radius) is sufficient. The center gravity constant already provides implicit pull-to-center; adding explicit seed pinning doubles up on centering forces and breaks layout balance.

If seed visual distinction is wanted: change seed node appearance (stroke, color accent) without modifying its force behavior.

**Phase:** Graph rendering overhaul phase — seed distinction is a pure visual change; isolate it from force tuning.

---

## Minor Pitfalls

---

### Pitfall 10: Edge Alpha 0.05 for LOD-Hidden Nodes is Not Zero — Hidden Edges Still Consume GPU

**What goes wrong:**
The current code sets `edge_vis_alpha = 0.05` for edges where either endpoint is LOD-hidden or temporally filtered. These edges are still uploaded to the GPU and drawn as geometry; only the alpha value changes. At 1000 edges with most nodes LOD-hidden, the GPU still processes ~1000 line draw calls (or triangle draw calls after the LINES fix), each contributing a near-transparent fragment. The visual result is correct (invisible), but the GPU cost is real.

**Prevention:**
Skip edges entirely if both endpoints have `lod_visible = false` or `temporal_visible = false`. The conditional skip for `from_vis && to_vis` already partially does this; verify it fully excludes from the edge buffer, not just reduces alpha. Skip inserting into `edge_data` entirely rather than inserting with alpha=0.05.

**Phase:** Graph rendering overhaul phase — improve alongside the edge visibility fix; minimal additional effort.

---

### Pitfall 11: `measureText` in Canvas 2D for Label Collision Bounding Boxes is Expensive

**What goes wrong:**
`ctx.measure_text(label)` requires a browser layout/text measurement call. If called every frame for every visible label (potentially 100+ labels), it is a JavaScript call per label per frame. In WASM this goes through a wasm-bindgen bridge call per invocation, with associated overhead.

**Prevention:**
Cache label widths in a `HashMap<String, f64>` keyed on label string. Labels are "First Author YYYY" — the set of unique labels is small and fixed for a given graph load. Measure once at graph initialization, not every frame.

**Phase:** Graph rendering overhaul phase — implement the cache alongside label rendering, not as a follow-up optimization.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Fix edge visibility | LINES width limit (Pitfall 2) | Switch to quad triangles first; do not tune edge colors until geometry is fixed |
| Fix node anti-aliasing | Fixed smoothstep delta (Pitfall 1) | Use `fwidth(d)` in fragment shader; test at radius=4 and radius=18 |
| Tune force coefficients | Collapse vs scatter failure modes (Pitfall 3) | Always test sparse chain topology AND dense mesh topology after any coefficient change |
| Auto-fit viewport | Fitting before simulation settles (Pitfall 4) | Defer auto-fit until alpha < 0.1; do not fit at T=0 |
| Add label rendering | DPR coordinate mismatch (Pitfall 5) | Confirm CSS-pixel convention before writing any label positioning code |
| Add label collision | O(n²) collision check (Pitfall 6) | Use spatial grid + priority ordering; gate on zoom level from day one |
| Seed node distinction | Center gravity interaction (Pitfall 9) | Visual change only; no force modification |
| Force simulation idle | Alpha floor CPU waste (Pitfall 7) | Pause simulation ticks when alpha = ALPHA_MIN and no interaction for 3s |

---

## Integration Gotchas

Specific to combining the Canvas 2D and WebGL2 renderers in one page.

| Integration Point | Common Mistake | Correct Approach |
|---|---|---|
| Labels over WebGL nodes | Drawing label text in a Canvas 2D context that shares the WebGL canvas | Use a separate overlay `<canvas>` for 2D text, positioned absolutely over the WebGL canvas; or use a `<div>` overlay for labels and position with CSS transforms |
| Viewport coordinate handoff | Passing `world_to_screen` CSS-pixel output to a DPR-scaled Canvas 2D context | Either ensure Canvas 2D context has no DPR transform when receiving CSS-pixel positions, or convert explicitly with `* dpr` |
| Resize affecting both renderers | ResizeObserver updating WebGL viewport but not Canvas 2D overlay dimensions | Update both canvas sizes in the same ResizeObserver callback; they must always match |
| Force tick duration variance | Barnes-Hut tick taking >16ms on a large initial graph | Cap max ticks per frame to 1 (already done); consider skipping the force tick on frames where the previous frame exceeded 14ms budget |
| Shader compilation cost | WebGL programs compiled synchronously in `WebGL2Renderer::new()` — can take 50–200ms on first load | Pre-warm shaders during Suspense fallback period; `new()` is called inside the Effect which fires after canvas mount, during the loading state — this timing is acceptable |

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|--------------|----------------|
| LINES width invisible (Pitfall 2) | LOW | Mechanical: replace LINES draw call with triangle quad geometry for each edge; 1–2 hours |
| Smoothstep blurry nodes (Pitfall 1) | LOW | 2-line shader change; replace fixed delta with `fwidth(d)` |
| Force coefficient collapse/scatter (Pitfall 3) | MEDIUM | Iterative: add velocity cap first (prevents scatter), then tune coefficients incrementally |
| Auto-fit at wrong time (Pitfall 4) | LOW | Add boolean flag + alpha threshold check in RAF loop; no architectural change |
| DPR label misalignment (Pitfall 5) | LOW | Audit every Canvas 2D drawing path; confirm transform convention; 1 hour |
| O(n²) label collision discovered late | MEDIUM | Requires rewrite of collision logic; add spatial grid; 2–4 hours |
| Renderer recreated on graph reload (Pitfall 8) | MEDIUM | Move renderer into stable ref outside the reactive Effect; requires refactor of graph page state |

---

## "Looks Done But Isn't" Checklist

- [ ] **Edge visibility**: Edges visible at default zoom on `#0d1117` background — check on a machine where `lineWidth > 1` might work (Firefox Linux with software renderer) to avoid false positives
- [ ] **Node sharpness**: Check seed node (largest radius) at DPR=2 — not just DPR=1 where the bug is invisible
- [ ] **Auto-fit**: Load a 100-node graph cold; confirm all nodes visible without user zoom after 3 seconds
- [ ] **Label alignment**: Test on a HiDPI display or with Chrome DevTools DPR emulation set to 2.0
- [ ] **Force stability**: Leave graph running for 60 seconds; confirm nodes are not still visibly moving after 30 seconds
- [ ] **Canvas 2D fallback**: Verify Canvas 2D renderer produces visually consistent results vs WebGL2 for a 50-node graph (both renderers must match)
- [ ] **Resize**: Resize browser window; confirm both renderers resize correctly and labels (if any) remain aligned
- [ ] **CPU on idle**: After simulation settles, confirm CPU usage drops below 5% in DevTools

---

## Sources

- [WebGL2 Anti-Patterns — webgl2fundamentals.org](https://webgl2fundamentals.org/webgl/lessons/webgl-anti-patterns.html) — LINES width limit explicitly documented; HIGH confidence
- [HandlingHighDPI — Khronos WebGL Wiki](https://www.khronos.org/webgl/wiki/HandlingHighDPI) — DPR coordinate transformation patterns; HIGH confidence
- [MDN WebGL Best Practices](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices) — general WebGL2 guidance; HIGH confidence
- [Drawing anti-aliased circles in OpenGL — rubendv.be](https://rubendv.be/posts/fwidth/) — `fwidth()` for resolution-independent AA; MEDIUM confidence
- [glsl-aastep — glslify/glsl-aastep on GitHub](https://github.com/glslify/glsl-aastep) — `fwidth`-based smoothstep AA utility; MEDIUM confidence
- [ForceAtlas2 paper — PLOS One](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0098679) — anti-swinging, cooling schedules, gravity force rationale; MEDIUM confidence
- [Barnes-Hut theta and convergence — Barnes-Hut Approximation by Jeff Heer](https://jheer.github.io/barnes-hut/) — theta parameter behavior; MEDIUM confidence
- [D3 force simulation documentation](https://d3js.org/d3-force/simulation) — alpha decay, velocity decay conventions; MEDIUM confidence
- [Force-directed graph drawing — Wikipedia](https://en.wikipedia.org/wiki/Force-directed_graph_drawing) — cooling schedule and energy function fundamentals; MEDIUM confidence
- [WebGL2 Cross Platform Issues — webgl2fundamentals.org](https://webgl2fundamentals.org/webgl/lessons/webgl-cross-platform-issues.html) — line width cross-platform limitations confirmed; HIGH confidence
- Direct code inspection: `resyn-worker/src/forces.rs`, `resyn-app/src/graph/webgl_renderer.rs`, `resyn-app/src/graph/renderer.rs`, `resyn-app/src/pages/graph.rs` — HIGH confidence (source of truth)

---

*Pitfalls research for: ReSyn v1.2 — Graph Rendering Overhaul (force tuning, WebGL AA, edge visibility, label collision, auto-fit viewport, DPR consistency)*
*Researched: 2026-03-24*
