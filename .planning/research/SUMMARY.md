# Project Research Summary

**Project:** Research Synergy (ReSyn) — v1.2 Graph Rendering Overhaul
**Domain:** Rust/WASM WebGL2 + Canvas 2D force-directed citation graph visualization
**Researched:** 2026-03-24
**Confidence:** HIGH

## Executive Summary

ReSyn v1.2 is a targeted rendering overhaul of an existing, working Leptos/WASM citation graph visualizer. All six target improvements are achievable through algorithmic and shader changes within the existing `resyn-worker` and `resyn-app` crates — no new dependencies are required. Every pitfall identified traces to misapplied constants, incomplete implementations of already-designed features, or missing 5–30 line additions. The existing code already stores the necessary data (`seed_paper_id`, `converged`, `bfs_depth`, `lod_visible`, `temporal_visible`) — this milestone is primarily a wiring and calibration effort, not new design.

The recommended approach is a sequenced six-step implementation ordered by dependency. Force coefficient rebalancing must come first: the current blob-collapse layout failure makes all other visual fixes impossible to validate. After the simulation produces spread clusters, the five rendering improvements (edge visibility, node sharpness, seed distinction, auto-fit, label collision avoidance) are independently implementable with one constraint — label collision avoidance should be tested against a stable spread layout. The only cross-cutting coordination requirement is that edge color constants must change in both `canvas_renderer.rs` and `webgl_renderer.rs` simultaneously, because the `WEBGL_THRESHOLD = 300` renderer switch would otherwise create a visible discrepancy at graph sizes near that boundary.

The primary implementation risks are force coefficient overshoot (two failure modes: collapse when attraction overwhelms repulsion in hub-heavy graphs, scatter when repulsion is raised without a velocity cap) and DPR coordinate misalignment for labels on HiDPI displays. Both are preventable with targeted tests: force changes must pass validation against both sparse-chain and dense-mesh topologies; label rendering must be confirmed at DPR=2 before sign-off. There is no architectural risk: the scope is bounded to six files across two crates, zero interface changes, and zero new public API surfaces.

## Key Findings

### Recommended Stack

No new crate dependencies are required for any of the six target features. All are implemented via constant tuning, GLSL shader modifications, and pure-Rust algorithmic additions within the existing stack. The research examined every plausible addition — `rapier2d`, `lyon`, `glam`, `fdg`, `wasm-bindgen-rayon` — and none justify inclusion for this scope.

**Core technologies:**
- `resyn-worker` Barnes-Hut force simulation: constant retuning + optional node-separation collision force — no interface changes, no new crates
- `web-sys WebGL2` (`webgl_renderer.rs`): fragment shader `fwidth`-based AA + instance stride 7→8 floats to carry `is_seed` varying
- `web-sys Canvas 2D` (`canvas_renderer.rs`): edge color/alpha fix, seed ring draw, label collision occupancy algorithm, auto-fit viewport trigger
- `petgraph` 0.7.0, Leptos 0.8, Trunk, wasm-bindgen: all unchanged throughout this milestone

**Critical version note:** `fwidth` is GLSL ES 3.00 core — available in all WebGL2 contexts without any extension. The existing `#version 300 es` header already enables it. No `OES_standard_derivatives` extension needed.

### Expected Features

All six target improvements are table-stakes fixes for an already-built visualizer, not new capabilities. The data infrastructure for each is in place.

**Must have (table stakes — currently broken or missing):**
- Visible citation edges — `#404040` at 0.35 alpha on `#0d1117` composites to near-background; fix to `#6e7681` at 0.6 alpha
- Nodes spread into clusters — force coefficients cause blob collapse; rebalance repulsion/attraction/damping constants
- Sharp node circles at all zoom levels — fixed 1px Canvas 2D border ignores scale; WebGL smoothstep delta is fixed, not screen-adaptive; use `fwidth` in WebGL2 and `1.0/viewport.scale` in Canvas 2D
- Seed node visual distinction — `seed_paper_id` exists in `GraphState` but is never read by renderers; add gold ring + amber fill
- Non-overlapping labels — labels drawn unconditionally, pile up at medium zoom; replace loop with greedy AABB occupancy algorithm
- Auto-fit viewport on load — graph may be outside viewport after simulation stabilises; trigger fit when `alpha < 0.1`

**Should have (after table-stakes fixes are stable):**
- BFS depth rings as initial node placement — `bfs_depth` field already on `NodeState`; better warm-start reduces simulation steps for readable layout
- Simulation convergence indicator UI — `converged: bool` already returned from worker, unwired; one-line status label change
- Runtime force parameter controls — only justified after default coefficient values are empirically validated through real use

**Defer to v2+:**
- Edge bundling, curved/bezier edges, 3D rendering, per-edge weights, undo/redo for node drag, any JavaScript graph library (all explicit anti-features; out of scope per PROJECT.md)
- Runtime force parameter sliders — premature until defaults are proven

### Architecture Approach

The v1.2 changes touch exactly six files across two crates. No new public API surfaces, no data model changes, no crate exports, and no changes to the simulation-on-main-thread architecture (the gloo-worker waker issue that causes worker outputs to go unconsumed is unchanged — the inline simulation path remains the active one).

**Components and their v1.2 changes:**
1. `resyn-worker/src/forces.rs` — five `pub const` coefficient values only; no interface changes
2. `resyn-app/src/graph/canvas_renderer.rs` — edge color/alpha constants; seed node color branch; label loop replaced with `draw_labels_no_overlap()`
3. `resyn-app/src/graph/webgl_renderer.rs` — `NODE_FRAG` shader (fwidth AA + seed ring varyings); `edge_color()` Regular arm; instance stride 7→8
4. `resyn-app/src/graph/layout_state.rs` — add `graph_bounding_box()` utility function
5. `resyn-app/src/graph/renderer.rs` — add `fit_to_bounds()` method on `Viewport`
6. `resyn-app/src/pages/graph.rs` — add `fit_done: bool` to `RenderState`; wire fit trigger in RAF loop; update dblclick handler to call fit rather than reset to 1:1

Unchanged: `barnes_hut.rs`, `interaction.rs`, `lod.rs`, `worker_bridge.rs`, all of `resyn-core/`, all of `resyn-server/`.

### Critical Pitfalls

1. **WebGL2 LINES primitive is always 1px — color tuning is useless before this is fixed** — Chrome/ANGLE enforces `lineWidth = 1.0` regardless of `gl.line_width()` input; this is a WebGL spec-level constraint. Replace `LINES` with quad triangle geometry (two triangles per edge, same approach already used for arrowheads). Apply this before any edge color changes or the color improvements will be invisible.

2. **Fixed smoothstep delta produces fuzzy large nodes** — `smoothstep(0.9, 1.0, d)` in the fragment shader maps to 3.6 physical pixels of feathering on an 18px-radius node at DPR=2. Replace with `float fw = fwidth(d); smoothstep(1.0 - fw, 1.0 + fw, d)` for screen-adaptive anti-aliasing. Two-line shader change; `fwidth` is WebGL2 core.

3. **Force coefficient imbalance collapses or scatters** — Two failure modes: (a) collapse when sum of attractive forces across many edges overwhelms weak repulsion in hub-heavy graphs; (b) scatter when repulsion is raised too aggressively without a per-tick velocity cap. Always validate coefficient changes against both a sparse-chain topology and a dense-mesh topology. Add `vel = vel.clamp(-MAX_VEL, MAX_VEL)` where `MAX_VEL = IDEAL_DISTANCE / 2` to prevent scatter.

4. **Auto-fit computed before simulation stabilises** — Fitting at T=0 calibrates to initial jitter spread, not converged positions. Defer fit until `alpha < 0.1` (approximately 460 frames at 60fps with `ALPHA_DECAY = 0.995`). Store `fit_done: bool` in `RenderState` (created fresh per Leptos Effect invocation) so it resets automatically on data reload.

5. **DPR coordinate mismatch for Canvas 2D labels on HiDPI displays** — `world_to_screen()` returns CSS-pixel coordinates. Label drawing code must not apply an additional DPR transform; labels will appear offset by `radius * (DPR - 1)` on retina displays. Test at DPR=2 via Chrome DevTools device emulation before sign-off.

## Implications for Roadmap

Research is unusually directive for this milestone. Dependency ordering is clear, file boundaries are identified, and all six features have implementation specifications with line-count estimates. No phase requires a `/gsd:research-phase` pass — all patterns are well-documented. A three-phase grouping is recommended.

### Phase 1: Force Simulation Rebalancing

**Rationale:** Blob collapse makes every other visual feature untestable. Label collision geometry is meaningless against a collapsed graph. Auto-fit is pointless when the settled AABB is near-zero. Force tuning is the prerequisite for the entire milestone — it must pass the existing convergence test suite before any rendering work begins.
**Delivers:** Nodes spread into visible clusters; hub-degree papers separated; simulation converges within ~30 seconds of load; node-separation collision force prevents overlap at high degree
**Addresses:** REPULSION_STRENGTH (raise from -300 to -500—-800), IDEAL_DISTANCE (raise from 80 to 120—150), VELOCITY_DAMPING (raise from 0.6 to 0.7—0.85), ATTRACTION_STRENGTH (lower from 0.03 to 0.01—0.015), CENTER_GRAVITY (reduce to 0.002—0.003)
**Avoids:** Pitfall 3 (collapse/scatter) — validate with sparse chain AND dense mesh test cases; run `test_convergence_100_node_graph_within_5000_ticks` after any coefficient change
**Files:** `resyn-worker/src/forces.rs` only

### Phase 2: Renderer Fixes (Edges, Nodes, Seed)

**Rationale:** These three rendering fixes are mutually independent and share no state, but both renderers must be changed together to prevent a visible discrepancy at the 300-node threshold. Grouping them enforces dual-renderer discipline and delivers all three improvements in one cohesive visual pass.
**Delivers:** Citation edges visible at default zoom on dark background; crisp node borders at all zoom levels; seed paper identified with gold ring and amber fill
**Addresses:** Edge color/alpha fix (both renderers), WebGL quad-triangle edge geometry (replace LINES), Canvas 2D `line_width = 1.0 / viewport.scale` for zoom-correct borders, WebGL fragment shader `fwidth` AA, seed instance data expansion (stride 7→8), seed ring in Canvas 2D arc path
**Avoids:** Pitfall 1 (fixed smoothstep blurriness — use fwidth, not precision change); Pitfall 2 (LINES 1px cap — switch to quad triangles before tuning color); Pitfall 5 (divergent renderers — change both files in same commit); Pitfall 9 (seed distinction is visual only — no force modification)
**Files:** `canvas_renderer.rs`, `webgl_renderer.rs`

### Phase 3: Viewport Fit and Label Collision Avoidance

**Rationale:** Auto-fit requires a spread layout (Phase 1) and the bounding box utilities built in this phase. Label collision avoidance is meaningful only with stable node positions. Both features share the `Viewport` coordinate math and benefit from being implemented together — `graph_bounding_box()` and `world_to_screen()` are shared building blocks.
**Delivers:** Graph auto-fits to viewport when simulation converges; node labels visible without overlap at medium zoom, priority-ordered by citation count; double-click becomes "fit to content" rather than "reset to 1:1 at origin"
**Addresses:** `graph_bounding_box()` in `layout_state.rs`; `fit_to_bounds()` on `Viewport`; `fit_done: bool` in `RenderState`; RAF-loop trigger at `alpha < 0.1`; greedy AABB occupancy algorithm in `canvas_renderer.rs` with `measureText` width cache; WebGL renderer labels remain deferred (LOD already reduces node count for large graphs)
**Avoids:** Pitfall 4 (premature fit — trigger only at `alpha < 0.1`, never T=0); Pitfall 5 (DPR label mismatch — CSS-pixel convention throughout, test at DPR=2); Pitfall 6 (O(n²) collision — use occupancy bitmap + priority ordering + `measureText` cache from day one)
**Files:** `layout_state.rs`, `renderer.rs`, `pages/graph.rs`, `canvas_renderer.rs`

### Phase Ordering Rationale

- Phase 1 before everything: blob collapse blocks visual validation of all rendering changes; force tests are the earliest possible signal
- Phase 2 before Phase 3: sharp nodes and correct edge geometry are prerequisites for accurate label bounding box calculations and for meaningful visual acceptance of auto-fit results
- Phase 3 last: label collision testing requires stable node positions (Phase 1); auto-fit trigger calibration depends on observed convergence timing from Phase 1
- The deferred differentiators (BFS depth rings, convergence indicator UI) are non-blocking additions: convergence indicator is a one-line status label that can be appended to Phase 3 with no additional planning

### Research Flags

No phases require a `/gsd:research-phase` call. All patterns are documented and implementation-ready.

Phases with standard, well-documented patterns:
- **Phase 1 (Force Coefficients):** D3-force defaults, ForceAtlas2 paper, and vis.js barnesHut reference values provide concrete numeric targets. Direct code inspection confirms the exact constants. HIGH confidence.
- **Phase 2 (Renderer Fixes):** `fwidth` is GLSL ES 3.00 core spec; quad-line approach is an established WebGL pattern; seed instance data layout is a stride expansion. All verified against official sources. HIGH confidence.
- **Phase 3 (Viewport + Labels):** Greedy AABB label placement is published algorithm (Vega/Vega-Lite); AABB fit math is elementary; alpha-threshold trigger is standard pattern. MEDIUM-HIGH confidence.

The one area requiring empirical calibration during implementation (not a research gap): force coefficient exact values. Research provides reference ranges but optimal values for ReSyn's hub-heavy citation graph topology must be validated against the convergence test suite and visual inspection. Plan for one tuning iteration after initial values are applied.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | No new crates; all decisions from direct code inspection + verified spec documents; zero dependency risk |
| Features | HIGH | All six features diagnosed from direct source inspection; implementation paths specified to file and line level |
| Architecture | HIGH | File-level touch points identified; integration points mapped; data flows confirmed against actual source; unchanged components confirmed |
| Pitfalls | HIGH (WebGL/DPR) / MEDIUM (force coefficients) | LINES width limit and fwidth: verified against WebGL2 spec and webgl2fundamentals. Force coefficient ranges: D3/vis.js/ForceAtlas2 community consensus — exact optimal values need empirical validation |

**Overall confidence:** HIGH

### Gaps to Address

- **Force coefficient exact values require empirical calibration:** Research provides reference ranges (REPULSION -500 to -1800 across multiple sources, with -800 as a conservative starting point; IDEAL_DISTANCE 120–150; VELOCITY_DAMPING 0.7–0.85). Optimal values for ReSyn's specific citation graph density and degree distribution must be confirmed with real graphs. Plan one iteration of visual validation after Phase 1 constants are applied.
- **`measureText` cache is required, not optional:** PITFALLS.md explicitly flags `ctx.measure_text()` as expensive per-frame across the wasm-bindgen bridge. The label collision implementation must cache widths at graph load time in a `HashMap<String, f64>`. This is implementation-level but must not be deferred as an optimization.
- **WebGL quad edge geometry integration with existing arrowhead pass:** Switching from `LINES` to quad triangles (6 vertices per edge vs 2) requires a vertex buffer refactor. The established pattern is documented (wwwtyro.net instanced lines), but the concrete integration with the existing `TRIANGLES` arrowhead pass in `webgl_renderer.rs` needs care to avoid draw call reorganisation complexity. Budget extra time for this sub-task within Phase 2.

## Sources

### Primary (HIGH confidence — direct code inspection or official spec)
- Direct source read: `resyn-worker/src/forces.rs`, `resyn-app/src/graph/webgl_renderer.rs`, `canvas_renderer.rs`, `renderer.rs`, `layout_state.rs`, `pages/graph.rs` — confirmed all current constants, shader source, instance layout, data structures
- GLSL ES 3.00 specification Section 8.13 — `fwidth` in core spec, no extension needed in WebGL2
- WebGL2 Anti-Patterns (webgl2fundamentals.org) — LINES width limit confirmed
- WebGL2 Cross Platform Issues (webgl2fundamentals.org) — line width clamping cross-platform confirmed
- Khronos WebGL Wiki: HandlingHighDPI — DPR coordinate transformation patterns

### Secondary (MEDIUM confidence — cross-referenced community sources)
- D3-force simulation documentation (d3js.org) — alpha decay, velocity decay defaults; alphaDecay 0.0228/tick ≈ 300 iterations
- vis.js Physics Documentation — barnesHut defaults: gravitationalConstant -2000, springLength 95, springConstant 0.04
- ForceAtlas2 paper (PLOS One) — adaptive temperature, anti-swinging, cooling schedule rationale
- Label occupancy bitmap algorithm — Vega/Vega-Lite Fast Labels paper (idl.cs.washington.edu/files/2021-FastLabels-VIS.pdf)
- wwwtyro.net instanced lines — quad-based edge rendering approach
- fwidth anti-aliasing technique (numb3r23.net, rubendv.be) — screen-adaptive smoothstep pattern

### Tertiary (MEDIUM confidence — visual tool observation)
- Connected Papers, Litmaps, VOSviewer feature baseline — live observation + 2025 comparison articles confirming seed distinction patterns and auto-fit behaviour

---
*Research completed: 2026-03-24*
*Ready for roadmap: yes*
