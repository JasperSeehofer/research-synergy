# Feature Landscape: Graph Rendering Overhaul (v1.2)

**Domain:** Force-directed citation graph visualization — fixing and enhancing existing Rust/WASM renderer
**Project:** Research Synergy (ReSyn) — v1.2 Graph Rendering Overhaul milestone
**Researched:** 2026-03-24
**Confidence:** HIGH (based on direct code inspection + cross-referenced research)

---

## Context: What Already Exists

All of the following are implemented and working in v1.1.1. This milestone fixes problems within them, not replaces them.

- Canvas 2D renderer (`canvas_renderer.rs`) with node circles, edge lines, arrowheads
- WebGL2 renderer (`webgl_renderer.rs`) as alternate backend
- Barnes-Hut force simulation in Web Worker (`resyn-worker/src/forces.rs`)
- LOD progressive reveal and temporal year-range filtering
- Hover/selection highlighting with neighbor dimming
- Contradiction (`#f85149`) and ABC-bridge (`#d29922`) edge types with toggle visibility
- Node radius scaled by citation count (4–18px via `radius_from_citations`)
- Labels rendered below nodes when `viewport.scale > 0.6`
- `seed_paper_id` field in `GraphState` (stored, never used in renderer)
- `converged: bool` in `LayoutOutput` (returned from worker, not acted on in UI)

## Identified Problems in Existing Code

Diagnosed from direct source inspection — not hypothetical.

**Force coefficient imbalance (the root cause of blob collapse):**
Current values: `REPULSION_STRENGTH = -300.0`, `IDEAL_DISTANCE = 80.0`, `ATTRACTION_STRENGTH = 0.03`, `VELOCITY_DAMPING = 0.6`, `ALPHA_DECAY = 0.995`.

The vis.js barnesHut reference implementation uses gravitationalConstant -2000 for similar node sizes. At -300, repulsion is ~7x weaker than the reference. For a citation graph with hub nodes (high-citation papers attracting many edges), the spring attraction from multiple edges overwhelms the weak repulsion, causing dense clusters to collapse into a blob. Additionally, `VELOCITY_DAMPING = 0.6` kills momentum too aggressively — velocities halve every tick, which prevents nodes from reaching equilibrium positions.

**Edge invisibility:** Regular edges use `#404040` at alpha `0.35` against the `#0d1117` background. The effective luminance contrast is approximately 10:1 in isolation but the 0.35 alpha brings it down to near-invisible. Contradiction (`#f85149`) and bridge (`#d29922`) edges at full alpha are fine; regular citation edges are not.

**Node border degradation at zoom:** `set_line_width(1.0)` is an absolute CSS-pixel value. When `viewport.scale` is > 2 (user zoomed in), the rendered border is 1 CSS pixel = correct. But the canvas transform has already scaled the coordinate system, so 1.0 in the transformed context becomes `1/scale` screen pixels — effectively thinner than 1px at high zoom. Fix: pass `1.0 / viewport.scale` to maintain apparent 1px border.

**Seed node not distinguished:** `seed_paper_id: Option<String>` is stored in `GraphState` and populated from server data. The renderer never checks it — the seed paper gets the same `#4a9eff` fill as all other non-selected nodes.

**Label pile-up at medium zoom:** Labels are drawn unconditionally for all `lod_visible && temporal_visible` nodes when `viewport.scale > 0.6`. No bounding box collision check. In dense clusters at zoom ~0.7, label text overlaps into unreadable stacks.

**No auto-fit on load:** Nodes are positioned by initial jitter from `from_graph_data` then spread by simulation. After simulation stabilizes, the viewport remains at initial zoom/offset. User must manually pan and zoom to find the graph. The `converged: bool` from the worker is returned but the bridge callback does not trigger any viewport adjustment.

---

## Table Stakes

Features users expect. Missing = graph feels broken.

| Feature | Why Expected | Complexity | Current State | Notes |
|---------|--------------|------------|---------------|-------|
| Visible edges between nodes | A graph without visible edges is just dots | Low | Broken — `#404040` at 0.35 alpha on `#0d1117` is near-invisible | Color and alpha change only |
| Nodes spread into clusters, not a blob | Core promise of force-directed layout | Medium | Broken — repulsion too weak for hub-heavy citation graphs | Coefficient rebalancing in `forces.rs` |
| Sharp node circles at all zoom levels | Expected of any canvas graph renderer | Low | Degraded — fixed 1px border ignores zoom scale | Scale `line_width` by `1.0 / viewport.scale` |
| Seed node visually distinct | User needs an anchor — "where did I start?" | Low | Not implemented — field exists, never rendered | Gold ring + distinct fill color |
| Labels readable without overlap | Overlapping labels are worse than no labels | Medium | Broken — no collision avoidance at medium zoom | Greedy AABB skip, priority-ordered |
| Auto-fit viewport after load | Graph should be visible without manual pan/zoom | Low | Not implemented — user must hunt for graph | Compute AABB on convergence, apply to viewport |

## Differentiators

Features that improve usability beyond the basics. Not strictly required, but high value.

| Feature | Value Proposition | Complexity | Depends On | Notes |
|---------|-------------------|------------|------------|-------|
| BFS depth rings as initial placement | Citation graphs are tree-structured; placing nodes in concentric rings by BFS depth gives simulation a better warm start and produces more readable final layouts | Medium | `bfs_depth` field already on `NodeState` | Replace current radial jitter in `from_graph_data` with ring placement; reduces simulation steps needed for good layout |
| Simulation convergence indicator | Users should know when the layout is stable vs still animating | Low | `converged` already in `LayoutOutput`, `simulation_running` already in `GraphState` | Show "Stabilizing..." / "Layout stable" text in controls; one-line UI change after convergence detection is wired |
| Configurable force parameters at runtime | Advanced users can tune repulsion for their specific graph shape | High | Coefficient fix | Sliders for `REPULSION_STRENGTH` and `IDEAL_DISTANCE`; requires making constants runtime-configurable; defer until defaults are validated |
| LOD label reveal tied to zoom region | Only show labels for nodes in the current viewport region, not all visible nodes | Medium | Label collision fix | Cull labels for nodes outside viewport AABB before running collision check; reduces work and clutter when panned in |

## Anti-Features

Features to explicitly NOT build in this milestone.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Edge bundling | Obscures individual citation paths; expensive to implement in Canvas 2D | Keep straight lines; they work for graphs under 500 nodes |
| Curved / bezier edges | Separate tangent computation per tick; no analytical benefit for citation edges | Straight lines with arrowheads are sufficient |
| 3D graph rendering | Unnecessary cognitive overhead; camera orientation adds complexity without benefit | Stay 2D; PROJECT.md already lists this as out-of-scope |
| Label always-on (all nodes) | 200+ overlapping labels are unreadable at any zoom | Keep LOD/zoom gating, add collision skip within visible set |
| Custom node shapes for types | Triangles or diamonds for different paper types add visual noise | Distinguish via color and ring, not shape |
| Per-edge weight controls | Citation graphs are unweighted; weight UI is busywork | All citation edges equal weight |
| Undo/redo for node drag | Users don't need drag history | Accept drag as permanent until new graph loads |
| JavaScript graph libraries | Explicitly out of scope (PROJECT.md "Out of Scope" section) | Full Rust/WASM stack |

---

## Feature Dependencies

```
Force coefficient fix
  └── enables: cluster spreading (all other fixes are untestable against a blob)
  └── enables: meaningful auto-fit (fitting a blob is pointless)
  └── enables: meaningful label collision (labels only overlap when nodes are spread)

Edge visibility fix
  └── independent — change color/alpha in canvas_renderer.rs

Sharp node borders
  └── independent — scale line_width by 1.0 / viewport.scale

Seed node distinction
  └── independent rendering change — seed_paper_id already in GraphState
  └── more useful after: coefficient fix (seed is invisible when graph is a blob)

Auto-fit viewport
  └── requires: convergence signal wired from worker_bridge to viewport state
  └── converged: bool already returned in LayoutOutput (not acted on yet)

Label collision avoidance
  └── requires: force coefficient fix (collision only useful with spread layout)
  └── independent of: auto-fit, seed distinction, edge visibility

BFS depth rings (differentiator)
  └── requires: bfs_depth field on NodeState (already exists)
  └── modifies: from_graph_data initial placement only (not simulation logic)

Convergence indicator (differentiator)
  └── requires: convergence signal wired from worker to GraphState.simulation_running
  └── simulation_running field already exists in GraphState
```

---

## Force Coefficient Analysis

Evidence-based recommendations derived from code inspection and reference implementations.

| Parameter | Current Value | Problem | Recommended Value | Rationale |
|-----------|--------------|---------|-------------------|-----------|
| `REPULSION_STRENGTH` | -300.0 | 7x weaker than vis.js barnesHut reference (-2000); insufficient to prevent hub collapse | -1200 to -1800 | Start at -1200; increase if graph still compresses. Hub nodes with many edges need strong repulsion to counteract multi-edge spring pull |
| `IDEAL_DISTANCE` | 80.0 | Too short; node radii range 4–18px; at 80px nodes at max radius have edges touching their borders | 120–150 | 6–8x max node radius (18px) gives adequate edge-to-node clearance |
| `ATTRACTION_STRENGTH` | 0.03 | Low but reasonable; re-evaluate after repulsion fix | 0.03–0.05 | May need slight increase to maintain cluster cohesion after repulsion is strengthened |
| `VELOCITY_DAMPING` | 0.6 | Too aggressive; velocities halve every tick, nodes cannot reach equilibrium before cooling kills motion | 0.85 | Standard range 0.8–0.95; 0.85 allows nodes to travel further per tick toward equilibrium |
| `CENTER_GRAVITY` | 0.005 | Appropriate; keeps graph centered | 0.005–0.01 | Increase only if graph drifts outside viewport after repulsion increase |
| `ALPHA_DECAY` | 0.995 | Very slow decay (690 ticks to halve alpha); acceptable if tick batch size is appropriate | 0.99–0.995 | Depends on ticks-per-message in `LayoutInput.ticks`; verify batch size is 10–50 ticks per message |

Source for reference values: vis.js barnesHut defaults (gravitationalConstant: -2000, springLength: 95, springConstant: 0.04). Confidence: MEDIUM — WebSearch result, vis.js docs URL confirmed but not directly fetched.

---

## Edge Rendering Requirements

**Current:** `#404040` stroke at alpha `0.35` on `#0d1117` background.

**Target:** Edges must be visible at-a-glance without competing with special edge colors.

| Edge Type | Current | Problem | Recommended |
|-----------|---------|---------|-------------|
| Regular citation | `#404040` @ 0.35 | Near-invisible | `#5a6a7a` @ 0.55 (slate-blue gray, matches node color palette) |
| Regular (dimmed neighbor) | any @ 0.10 | Fine | Keep 0.08–0.10 |
| Regular (LOD/temporal hidden) | any @ 0.05 | Fine | Keep 0.05 |
| Contradiction | `#f85149` @ 1.0 | Working | No change |
| ABC-bridge | `#d29922` @ 1.0 dashed | Working | No change |

Arrowheads on regular edges: currently use same `#404040` color at 0.35 alpha — update to match edge line color.

---

## Node Rendering Requirements

**Sharp circles at zoom:**
- Canvas 2D `arc()` is browser-antialiased — circles are already smooth
- Problem is border `set_line_width(1.0)` — this is in the transformed coordinate space
- When viewport scale > 1 (zoomed in), 1.0 in transformed space = `1/scale` screen pixels (too thin)
- Fix: call `set_line_width(1.0 / viewport.scale)` before stroking node circles
- DPR is already handled at canvas physical sizing per PROJECT.md convention; no change needed there

**Seed node distinction:**
- Check `state.seed_paper_id == Some(node.id)` in the node rendering loop
- If seed: use fill color `#f5c542` (gold/amber) instead of `#4a9eff`
- Draw an additional outer ring: `arc(node.x, node.y, node.radius + 5.0)` with `#f5c542` stroke at 0.8 alpha, `line_width = 2.0 / viewport.scale`
- Seed node in hovered/selected state: use `#58a6ff` (same as current) — interaction highlighting overrides seed distinction
- Research precedent: Connected Papers green for seed, Litmaps green for seed — bright distinguishing color is standard pattern (MEDIUM confidence, from search results)

---

## Label Collision Avoidance

**Greedy AABB skip algorithm (O(n) amortized):**

This is the standard approach used by charting libraries. O(n) per frame for the typical case.

1. Sort visible nodes by priority: seed node first, then by `citation_count` descending
2. Maintain `Vec<(f64, f64, f64, f64)>` of placed label AABBs (xmin, ymin, xmax, ymax)
3. For each node in priority order:
   - Compute label AABB: text centered at `(node.x, node.y + node.radius + 12.0)`, width from `ctx.measure_text()`, height fixed at 14px
   - Check if AABB overlaps any placed AABB (iterate the placed list)
   - If overlap: skip this node's label
   - If no overlap: draw label, append AABB to placed list
4. Only run when `viewport.scale > 0.6` (existing threshold)

**Complexity note:** In the worst case (all nodes in view, dense graph) this is O(n²) comparison, but in practice: at `scale = 0.6` with 200 nodes, about 40–60 labels will fit without overlap, so the placed list stays short and comparisons are fast. Acceptable for Canvas 2D at 60fps.

**Priority rationale:** Always show the seed paper label and top-cited hubs. Leaf nodes (low citation, deep BFS) are suppressed first.

---

## Auto-Fit Viewport

**Trigger:** When worker returns `converged: true` for the first time after a graph load.

**Implementation:**
1. Detect convergence: in `worker_bridge` callback, when `output.converged` is true and this is the first convergence for current graph data, emit a fit-viewport signal
2. Compute AABB over all `lod_visible && temporal_visible` nodes, including `radius` in bounds: `xmin = node.x - node.radius`, `xmax = node.x + node.radius`, etc.
3. Scale calculation: `scale = min(canvas_width / (bbox_width + 100.0), canvas_height / (bbox_height + 100.0))` — 50px padding each side
4. Offset calculation: center the bounding box in the canvas
5. Apply to `Viewport` — either instant or with a smooth interpolation over ~20 animation frames

**Constraints:**
- Do not re-trigger on temporal filter changes (user has taken over zoom context)
- Do not re-trigger if user has manually panned/zoomed since last fit
- If graph has 0 visible nodes (all filtered out), skip fit

**Source:** D3 force simulation "end" event pattern; vasturiano force-graph `zoomToFit()` + `getBoundingBox()` API pattern. Confidence: MEDIUM.

---

## MVP Build Order

Build in this sequence — each step unblocks the next and is independently testable:

1. **Force coefficients** (`forces.rs` constants) — must fix first; all other visual improvements are meaningless against a blob layout. Testable with existing convergence test.
2. **Edge visibility** (`canvas_renderer.rs` color + alpha) — one-line change; immediate visual validation at current zoom level.
3. **Sharp node borders** (`canvas_renderer.rs` line_width) — one-line change; visible at high zoom.
4. **Seed node distinction** (`canvas_renderer.rs` fill + ring) — small renderer change; validates that `seed_paper_id` flows through correctly.
5. **Auto-fit viewport** (`worker_bridge.rs` + `layout_state.rs`) — requires wiring convergence signal to viewport; tests that graph is visible on load.
6. **Label collision avoidance** (`canvas_renderer.rs` sort + AABB check) — do last; layout must be stable before collision geometry is meaningful.

Defer:
- BFS depth rings (initial placement improvement) — only if coefficient fix alone is insufficient
- Runtime force parameter controls — after defaults are empirically validated through real use
- Convergence indicator UI — low effort but not blocking; add in same PR as auto-fit

---

## Competitor Feature Baseline

What citation graph tools do for the six target features, for reference.

| Feature | Connected Papers | Litmaps | VOSviewer | ReSyn v1.2 Target |
|---------|-----------------|---------|-----------|-------------------|
| Layout quality | Good clusters, no blob | Timeline-axis hybrid | Good, tuned for bibliometric networks | Fix coefficients to match Connected Papers quality |
| Edge visibility | Visible light-gray on white | Visible on white | Visible | Dark-mode appropriate contrast |
| Node rendering | Circle, smooth | Circle | Circle | Already smooth via Canvas 2D arc() |
| Seed node | Green distinct node | Highlighted | No concept | Gold/amber ring + fill |
| Label overlap | Suppressed at low zoom, visible at high | Always shown with zoom-linked font | Suppressed at low zoom | Greedy AABB skip, priority-ordered |
| Auto-fit on load | Yes — always fits | Yes — always fits | Yes | Fit on first convergence |

Source: Live tool observation + 2025 comparison articles. Confidence: MEDIUM.

---

## Sources

- [Force-Directed Drawing Algorithms, Kobourov (Brown University)](https://cs.brown.edu/people/rtamassi/gdhandbook/chapters/force-directed.pdf) — coefficient recommendations c1=2, c2=1, c3=1; repulsion/attraction balance
- [vis.js Physics Documentation — barnesHut defaults](https://visjs.github.io/vis-network/docs/network/physics.html) — gravitationalConstant -2000, springLength 95, springConstant 0.04
- [D3 Force Simulation API](https://d3js.org/d3-force/simulation) — alpha cooling, end event, alphaMin convergence threshold
- [Fields, Bridges, Foundations: Researcher Citation Graph Study (arXiv 2405.07267)](https://arxiv.org/html/2405.07267v1) — six researcher patterns for citation graph exploration; Fields/Bridges/Foundations taxonomy
- [Minimizing Overlapping Labels in Interactive Visualizations (Towards Data Science)](https://towardsdatascience.com/minimizing-overlapping-labels-in-interactive-visualizations-b0eabd62ef0/) — greedy O(n) AABB label placement algorithm
- [Litmaps vs Connected Papers 2025 comparison](https://effortlessacademic.com/litmaps-vs-researchrabbit-vs-connected-papers-the-best-literature-review-tool-in-2025/) — competitive feature baseline
- [Connected Papers 2025 review](https://skywork.ai/skypage/ko/Connected-Papers:-My-Deep-Dive-into-the-Visual-Research-Tool-(2025-Review)/1972566882891395072) — seed node green color pattern
- [vasturiano force-graph (GitHub)](https://github.com/vasturiano/force-graph) — zoomToFit() API, getBoundingBox() pattern for auto-fit
- [Force-Directed Graph Layouts Revisited: T-Distribution (arXiv 2303.03964)](https://arxiv.org/abs/2303.03964) — bounded short-range force for better neighborhood preservation

---
*Feature research for: ReSyn v1.2 — Graph Rendering Overhaul*
*Researched: 2026-03-24*
*Supersedes: v1.1 feature research (2026-03-15)*
