# Architecture Patterns: v1.2 Graph Rendering Overhaul

**Domain:** Interactive citation graph visualization (Rust/WASM, Leptos CSR)
**Researched:** 2026-03-24
**Scope:** Force layout fixes, edge visibility, sharp node rendering, label collision, auto-fit viewport

---

## Current Architecture (Baseline)

The graph subsystem spans three crates and one page component. Understanding the exact data paths is prerequisite to knowing what each v1.2 fix touches.

```
resyn-app/src/pages/graph.rs  (GraphPage component)

  Leptos Effect (fires on canvas mount + data ready)
    |
    +-- GraphState::from_graph_data()  [layout_state.rs]
    |     Node positions: spiral + jitter on init
    |
    +-- make_renderer()             [renderer.rs]
    |     Canvas2D if nodes < 300
    |     WebGL2   if nodes >= 300 AND webgl2 available
    |
    +-- start_render_loop()  (RAF loop, main thread)
    |     Each frame:
    |       1. Sync Leptos signals into graph state
    |       2. run_ticks(input, ticks=1) [forces.rs]
    |          Barnes-Hut repulsion + Hooke attraction
    |          + center gravity, alpha *= 0.995/tick
    |          alpha floored at ALPHA_MIN (never stops)
    |       3. update_lod_visibility()  [lod.rs]
    |       4. update_temporal_visibility() [lod.rs]
    |       5. renderer.draw(graph, viewport)
    |
    +-- attach_event_listeners()
          mousemove -> hover + tooltip
          mousedown -> DraggingNode or Panning
          mouseup   -> click-vs-drag, select, unpin
          dblclick  -> reset viewport to center/scale=1
          wheel     -> zoom_toward_cursor()
          pointerleave -> clear hover/interaction


resyn-worker/  (WASM Web Worker, compiled separately)

  ForceLayoutWorker (gloo-worker #[reactor])
    Receives LayoutInput  ->  runs forces::run_ticks()
    Sends back LayoutOutput

  NOTE: Worker is spawned but outputs are NOT consumed.
  graph.rs comment: "waker issues with gloo-worker's
  ReactorBridge prevent outputs from being received."
  Simulation runs inline on main thread instead.


resyn-app/src/graph/  (rendering modules)

  layout_state.rs  -- NodeState, EdgeData, GraphState
    NodeState: id, title, x/y, radius, pinned,
               bfs_depth, lod_visible, temporal_visible
    GraphState: nodes[], edges[], velocities[],
                alpha, selected_node, seed_paper_id,
                temporal_min/max_year, current_scale

  renderer.rs      -- Viewport (CSS pixels), Renderer trait
    Viewport: offset_x/y, scale, css_width/height
    make_renderer(): selects Canvas2D vs WebGL2
    DPR convention: CSS pixels throughout logic;
    DPR only at canvas.set_width() and gl.viewport()

  canvas_renderer.rs -- Canvas2DRenderer
    Edge pass: #404040 @ 0.35 alpha
    Node pass: arc() circles with stroke border
    Label pass: 11px monospace at scale > 0.6
    Arrowheads: draw_arrowhead() per edge

  webgl_renderer.rs  -- WebGL2Renderer
    Edge pass: LINES draw_edge_pass()
    Arrow pass: TRIANGLES draw_edge_pass()
    Node pass: instanced TRIANGLE_FAN, 7 floats/instance
    Fragment shader: smoothstep(0.9, 1.0, d) for AA
    No text rendering (labels deferred, hook exists)

  interaction.rs  -- find_node_at, find_edge_at,
                     zoom_toward_cursor, InteractionState

  lod.rs          -- update_lod_visibility (scale bands),
                     update_temporal_visibility,
                     compute_visible_count

  worker_bridge.rs -- WorkerBridge (spawns worker,
                      send_input -- currently unused)
```

---

## v1.2 Features and Their Architectural Touch Points

### Feature 1: Force Coefficient Tuning

**Goal:** Connected nodes form visible clusters instead of collapsing to center or remaining uniformly spread.

**Current state:** REPULSION_STRENGTH = -300, ATTRACTION_STRENGTH = 0.03, CENTER_GRAVITY = 0.005, IDEAL_DISTANCE = 80, VELOCITY_DAMPING = 0.6, ALPHA_DECAY = 0.995.

**Root cause of collapse:** ATTRACTION_STRENGTH = 0.03 combined with VELOCITY_DAMPING = 0.6 creates over-damped convergence toward IDEAL_DISTANCE before the graph has spread. CENTER_GRAVITY at 0.005 may over-constrain dense sub-graphs. IDEAL_DISTANCE = 80 is reasonable for radius range [4, 18] but clusters need more breathing room.

**What to change:**

| Constant | Current | Recommended | Reason |
|---|---|---|---|
| REPULSION_STRENGTH | -300 | -500 to -800 | More spread force for 50-300 node graphs |
| ATTRACTION_STRENGTH | 0.03 | 0.01 to 0.015 | Reduce pull so edges stretch rather than collapse |
| IDEAL_DISTANCE | 80 | 120 to 150 | Wider rest length gives visual separation between clusters |
| VELOCITY_DAMPING | 0.6 | 0.7 to 0.75 | Slightly less damping lets graph explore more space |
| CENTER_GRAVITY | 0.005 | 0.002 to 0.003 | Weaker gravity lets clusters drift to natural positions |

**Files modified:**
- `resyn-worker/src/forces.rs` — change the five `pub const` values only

**Integration point:** Constants are `pub`, used only within `forces.rs::simulation_tick()`. No interface changes, no downstream signature changes. Canvas label layout and viewport code are unaffected by coefficient changes alone.

**Test impact:** `test_convergence_100_node_graph_within_5000_ticks` asserts convergence. With higher repulsion and lower attraction, convergence may take more ticks. Verify this test still passes; if not, adjust the tick limit or fine-tune coefficients. `test_attractive_force_pulls_connected_nodes_together` and `test_center_gravity_pulls_isolated_node_toward_origin` should continue to pass directionally.

---

### Feature 2: Edge Visibility Fix

**Goal:** Citation edges are visually readable against the #0d1117 background.

**Root cause:** Regular edges use `#404040` at `alpha = 0.35`. On a `#0d1117` background (RGB 13, 17, 23), a #404040 gray (64, 64, 64) at 35% opacity composites to approximately RGB (19, 21, 23) — near-invisible against the background.

**What to change:**

In `canvas_renderer.rs`:
- Line 96: `set_stroke_style_str("#404040")` -> `"#6e7681"` (GitHub dimmed color, visible on dark bg)
- Line 98-99: base visibility alpha for regular edges: 0.35 -> 0.6
- Line 197: arrowhead alpha for Regular edges: 0.35 -> 0.6

In `webgl_renderer.rs` `edge_color()` function:
- Regular branch: color -> `"#6e7681"`, alpha -> 0.6 (was 0.35)
- `both_dimmed` reduced alpha remains at 0.1 (ratio preserved)

**Files modified:**
- `resyn-app/src/graph/canvas_renderer.rs` — two alpha constants, one color constant
- `resyn-app/src/graph/webgl_renderer.rs` — `edge_color()` Regular arm

**Integration point:** Edge color and alpha are render-only. No changes to `GraphState`, `Viewport`, `interaction.rs`, `lod.rs`, or `forces.rs`. Both renderers must be changed together because `make_renderer()` selects between them at runtime based on node count; a discrepancy would produce different edge visibility at the 300-node threshold.

---

### Feature 3: Sharp Node Rendering + Seed Node Distinction

**Goal:** Nodes are crisp at all zoom levels; the seed paper has a visually distinct appearance.

**Root cause (WebGL):** The fragment shader uses `precision mediump float` and `smoothstep(0.9, 1.0, d)`. mediump has ~10 bits of mantissa, causing subtle banding. The smoothstep range of 10% of the radius spans less than 1 physical pixel at small node sizes, making the AA zone inconsistent. `TRIANGLE_FAN` with 4 vertices is a coarse circle approximation for small radii.

**Seed node:** `GraphState.seed_paper_id: Option<String>` and `NodeState.bfs_depth: Option<u32>` already exist. No new fields needed. Renderers need only a string equality check per node.

**What to change (WebGL):**
- `NODE_FRAG` shader: `precision mediump float` -> `precision highp float`
- `NODE_FRAG` shader: `smoothstep(0.9, 1.0, d)` -> `smoothstep(0.95, 1.0, d)` (tighter AA zone)
- Per-instance color logic in `draw()`: add seed detection before the dimmed/hovered/selected branch:
  ```rust
  let is_seed = state.seed_paper_id.as_deref() == Some(node.id.as_str());
  // Color priority: selected/hovered > seed > dimmed > default
  ```
  Seed color: `#f0b429` (amber, distinct from blue node default and red/yellow gap edges)

**What to change (Canvas2D):**
- `arc()` path is already exact. Sharp rendering is governed by correct DPR handling, which is already correct.
- In node draw loop (step 8): add seed detection for distinct fill_color + wider border stroke. Same priority order as WebGL.

**Files modified:**
- `resyn-app/src/graph/webgl_renderer.rs` — shader source constants `NODE_FRAG`; per-instance color logic
- `resyn-app/src/graph/canvas_renderer.rs` — fill_color selection to add seed branch
- `resyn-app/src/graph/layout_state.rs` — no changes (fields already exist)

**Integration point:** Node color selection currently uses only `is_dimmed`, `is_hovered`, `is_selected`. Seed check adds a fourth condition. Ordering: selected/hovered > seed > dimmed > default. No change to `GraphState` fields, forces, or interaction.

---

### Feature 4: Smart Label Rendering (Avoid Overlap)

**Goal:** Node labels at any zoom level are non-overlapping and prioritized by citation count.

**Current state:** `canvas_renderer.rs` renders all lod+temporal-visible node labels unconditionally when `viewport.scale > 0.6`. With 50-200 visible nodes at 1x zoom, labels overlap heavily.

**Architecture decision:** Greedy occupancy approach — project each candidate label into screen space, check against a list of already-placed bounding rects, skip if overlap. Process candidates in citation-count descending order so important papers get labels when space is tight.

**WebGL renderer:** Labels are not currently rendered by `webgl_renderer.rs`. The `node_screen_positions()` method is an existing hook but is not called anywhere. For v1.2, keep this as-is: implement collision avoidance in `canvas_renderer.rs` only. WebGL path (large graphs) already benefits from LOD reducing visible node counts significantly.

**What to add:**

In `canvas_renderer.rs`, replace the label loop (step 9, lines 259-274) with:

```rust
fn draw_labels_no_overlap(
    ctx: &CanvasRenderingContext2d,
    state: &GraphState,
    viewport: &Viewport,
)
```

Algorithm:
1. Collect `(node, screen_x, screen_y)` for all `lod_visible && temporal_visible` nodes, converting world->screen via `viewport.world_to_screen()`
2. Sort by `citation_count` descending
3. Maintain `occupied: Vec<[f64; 4]>` as `[x_min, y_min, x_max, y_max]` rects in screen space
4. For each candidate: measure `ctx.measure_text(&label)` width, compute rect at `(sx - w/2, sy + r*scale + 2, sx + w/2, sy + r*scale + 16)` in screen space (where `r*scale` is the node radius in screen pixels)
5. If any occupied rect overlaps, skip; otherwise push to occupied and draw

`Viewport` is already the third parameter of `Renderer::draw()` — no signature changes needed.

**Files modified:**
- `resyn-app/src/graph/canvas_renderer.rs` — replace label loop with `draw_labels_no_overlap()`, no other files

**Integration point:** `Viewport` is already passed to `draw()`. No new fields in `GraphState`. Label candidates use the same `lod_visible && temporal_visible` flags as the existing label loop. The only new dependency is calling `viewport.world_to_screen()` for each candidate — an existing method with no side effects.

---

### Feature 5: Auto Fit-to-Content Viewport After Layout Stabilizes

**Goal:** Viewport automatically zooms and pans to show all nodes comfortably once the force simulation settles.

**Current state:** On init, a rough fit scale is computed from initial spread (before simulation runs). This does not adjust offset, so the graph may not be centered. After `dblclick`, the viewport resets to `Viewport::new(w, h)` — scale=1.0 centered on world origin — ignoring actual node positions.

**What to add:**

1. **Bounding box utility** in `layout_state.rs`:
   ```rust
   pub fn graph_bounding_box(nodes: &[NodeState]) -> Option<(f64, f64, f64, f64)>
   // Returns Some((x_min, y_min, x_max, y_max)) for lod+temporal visible nodes
   // Returns None if no visible nodes
   ```
   Uses `n.x`, `n.y`, `n.radius` to include node borders in bounds.

2. **Viewport fit function** in `renderer.rs` as a method on `Viewport`:
   ```rust
   pub fn fit_to_bounds(
       &mut self,
       x_min: f64, y_min: f64,
       x_max: f64, y_max: f64,
       padding: f64,  // e.g. 0.1 for 10% margin
   )
   ```
   Implementation:
   - `content_w = x_max - x_min`, `content_h = y_max - y_min`
   - `scale = ((self.css_width * (1.0 - padding)) / content_w).min((self.css_height * (1.0 - padding)) / content_h).min(2.0)`
   - `offset_x = self.css_width / 2.0 - ((x_min + x_max) / 2.0) * scale`
   - `offset_y = self.css_height / 2.0 - ((y_min + y_max) / 2.0) * scale`

3. **Trigger in RAF loop** in `pages/graph.rs`:
   - Add `fit_done: bool` field to `RenderState`
   - In the frame closure, after `run_ticks`: if `s.graph.alpha < 0.1 && !s.fit_done`, call `graph_bounding_box()` + `viewport.fit_to_bounds()`, set `s.fit_done = true`
   - `RenderState` is created fresh in each Effect invocation, so `fit_done` resets automatically on data reload

4. **dblclick handler update**: replace `Viewport::new(w, h)` with a `graph_bounding_box()` + `fit_to_bounds()` call, making double-click into "fit to content" rather than "reset to 1:1"

**Files modified:**
- `resyn-app/src/graph/layout_state.rs` — add `graph_bounding_box()` function
- `resyn-app/src/graph/renderer.rs` — add `fit_to_bounds()` method on `Viewport`
- `resyn-app/src/pages/graph.rs` — add `fit_done` to `RenderState`; trigger in RAF loop; update dblclick handler

**Integration point:** `graph_bounding_box()` reads only `NodeState.x/y/radius/lod_visible/temporal_visible` — no writes. `fit_to_bounds()` writes only `viewport.offset_x/y/scale` — no effect on forces, interaction state, or rendering beyond the next frame.

---

## Component Boundaries: New vs Modified

| File | Status | What Changes |
|---|---|---|
| `resyn-worker/src/forces.rs` | Modified | Five `pub const` coefficient values only |
| `resyn-app/src/graph/canvas_renderer.rs` | Modified | Edge color/alpha; seed node color branch; replace label loop with collision-aware function |
| `resyn-app/src/graph/webgl_renderer.rs` | Modified | `NODE_FRAG` shader precision + smoothstep; `edge_color()` Regular arm; seed node color in instance loop |
| `resyn-app/src/graph/layout_state.rs` | Modified | Add `graph_bounding_box()` utility function |
| `resyn-app/src/graph/renderer.rs` | Modified | Add `fit_to_bounds()` method on `Viewport` |
| `resyn-app/src/pages/graph.rs` | Modified | Add `fit_done` to `RenderState`; trigger fit in RAF loop; update dblclick handler |
| `resyn-worker/src/barnes_hut.rs` | Unchanged | No changes needed |
| `resyn-app/src/graph/interaction.rs` | Unchanged | No changes needed |
| `resyn-app/src/graph/lod.rs` | Unchanged | No changes needed |
| `resyn-app/src/graph/worker_bridge.rs` | Unchanged | No changes needed |
| `resyn-app/src/graph/mod.rs` | Unchanged | No new public exports needed |
| `resyn-core/` | Unchanged | All changes are in rendering/layout layer |
| `resyn-server/` | Unchanged | No data model or API changes |

---

## Data Flow: How the Five Features Interact

```
Data load -> GraphState::from_graph_data()
             (positions: spiral + jitter)
                 |
                 v
RAF loop starts (alpha = 1.0)
                 |
     +-----------|----------------------------------------------+
     |  Tick N   |                                              |
     |           v                                              |
     |  forces::run_ticks() ------- FEATURE 1 ---------------> |
     |  (tuned coefficients -> visible clustering)             |
     |                                                          |
     |  lod_visibility + temporal_visibility update            |
     |                                                          |
     |  renderer.draw()                                        |
     |    +-- edge pass ----------- FEATURE 2 ---------------> |
     |    |   (brighter color, higher alpha)                   |
     |    +-- node pass ----------- FEATURE 3 ---------------> |
     |    |   (sharp AA shader, seed distinction)              |
     |    +-- label pass ---------- FEATURE 4 ---------------> |
     |        (greedy no-overlap, citation priority)           |
     |                                                          |
     |  if alpha < 0.1 && !fit_done:                          |
     |    fit_to_bounds() ---------- FEATURE 5 -------------> |
     |    fit_done = true                                      |
     +----------------------------------------------------------+
```

Features 2, 3, 4 are rendering-only and operate entirely within the draw call. Feature 1 affects only the simulation step. Feature 5 affects viewport state once after stabilization. None of these features interfere with each other's data paths.

---

## Suggested Build Order

Build order follows: (1) zero-risk coefficient changes with test validation, (2) rendering fixes that share no state, (3) coordinate utilities that other features build on, (4) wire utilities into the page, (5) label collision last when zoom and positions are stable for visual testing.

### Step 1 — Force Coefficients (resyn-worker/src/forces.rs)

**Dependencies:** None. Self-contained constant changes.
**Validation:** Run `cargo test` in resyn-worker. Visually verify nodes spread into clusters before touching renderers.
**Risk:** Low. Existing tests cover convergence, direction of forces, and alpha decay. If `test_convergence_100_node_graph_within_5000_ticks` fails, increase the tick limit or adjust the coefficient values.

### Step 2 — Edge Visibility (canvas_renderer.rs + webgl_renderer.rs)

**Dependencies:** None. Independent of Step 1.
**Rationale:** Pure constant/color changes with no logic branching. Quick visual payoff. Both files must be changed together to keep Canvas2D and WebGL2 paths in sync.
**Validation:** Test both renderer paths (node counts around 280-320 cross the WEBGL_THRESHOLD = 300 boundary).
**Risk:** Low. No new logic, only value changes.

### Step 3 — Sharp Node Rendering + Seed Distinction (canvas_renderer.rs + webgl_renderer.rs)

**Dependencies:** None on Steps 1 or 2. `seed_paper_id` already exists in `GraphState`.
**Rationale:** Shader and circle rendering changes are independent of edge and force changes. Seed distinction is a new color branch in the existing per-node render loop.
**Validation:** Verify seed node gets distinct amber color regardless of hover/selection state. Test WebGL on two browser/GPU combinations for shader precision behavior.
**Risk:** Low-Medium. Shader `precision` changes can affect behavior on different GPU vendors.

### Step 4 — Bounding Box + fit_to_bounds Utilities (layout_state.rs + renderer.rs)

**Dependencies:** None. Pure math.
**Rationale:** Build and test the utilities in isolation before wiring into the RAF loop.
**Validation:** Unit-test `graph_bounding_box()` with known node positions. Unit-test `fit_to_bounds()` with known bounds and verify `world_to_screen(center_of_bounds)` returns `(css_width/2, css_height/2)` after the fit.
**Risk:** Low. Pure functions with no side effects.

### Step 5 — Auto Fit-to-Content (pages/graph.rs)

**Dependencies:** Step 4 utilities must be complete.
**Rationale:** Wire fit into the RAF loop and dblclick handler using utilities from Step 4.
**Validation:** Load a graph, verify viewport auto-zooms once alpha < 0.1. Verify dblclick re-fits rather than jumping to scale=1. Verify reloading data (new crawl) resets and re-triggers fit.
**Risk:** Low-Medium. The alpha threshold must be calibrated: at ALPHA_DECAY = 0.995 and 60fps (1 tick/frame), alpha = 1.0 decays to 0.1 in approximately 460 frames (~7.7 seconds). This is acceptable; if it feels too slow, lower the threshold to 0.2 (reaches 0.2 in ~322 frames, ~5.4 seconds).

### Step 6 — Label Collision Avoidance (canvas_renderer.rs)

**Dependencies:** None technically; placed last so node positions and zoom are stable for visual testing.
**Rationale:** Label rendering depends on viewport scale and node screen positions — placing it after force tuning and auto-fit ensures the visible state used for testing reflects final layout quality.
**Validation:** Verify at viewport scales 0.6, 1.0, 1.5 that no two labels overlap. Verify highest-citation nodes get labels when others are evicted. Verify `ctx.measure_text()` is called within the draw loop (always in a valid context).
**Risk:** Medium. The greedy O(n^2) collision check is acceptable for label counts under 200. At 200 visible nodes with ~30 labels visible per zoom level, worst case is ~900 comparisons per frame — well within budget. If the visible label count exceeds expectations, add an early exit after placing N labels (e.g., N=50).

---

## Scalability Considerations

| Concern | At 50 nodes | At 300 nodes | At 1000 nodes |
|---|---|---|---|
| Renderer path | Canvas2D | Threshold (Canvas2D or WebGL2) | WebGL2 |
| Label collision check | Negligible | ~100 labels x 100 occupied = 10,000 comparisons; fine | LOD reduces visible to ~50 at typical zoom; fine |
| Force simulation (1 tick/frame) | <0.5ms | ~2ms Barnes-Hut O(n log n) | ~8ms; may stutter if RAF budget exceeded |
| Auto-fit trigger | Fast: small graph converges quickly | ~460 frames at alpha 0.995/tick | ~460 frames same; alpha decay is independent of n |
| Seed node lookup | O(n) string compare, negligible | O(n), negligible | O(n), negligible |

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Changing Force Constants Without Running Convergence Tests
**What:** Tuning REPULSION_STRENGTH or IDEAL_DISTANCE without re-running `test_convergence_100_node_graph_within_5000_ticks`.
**Why bad:** Over-strong repulsion may cause the 100-node test to fail. More importantly, if the visual graph never reaches near-stable positions within the auto-fit threshold window (alpha < 0.1), the auto-fit never triggers.
**Instead:** Tune constants, run all force tests, then observe visual settling time before setting the auto-fit threshold.

### Anti-Pattern 2: Triggering Auto-Fit Too Early
**What:** Using `alpha < 0.5` as the fit trigger.
**Why bad:** Nodes are still moving significantly at alpha 0.5. The viewport snaps to an intermediate layout, then nodes continue moving outside the fitted view — jarring and unhelpful.
**Instead:** Use `alpha < 0.1`. The graph is ~90% settled at this point. At 0.995^460 ≈ 0.1 and 60fps this means ~7.7 seconds, which is acceptable for an initial layout.

### Anti-Pattern 3: fit_done Outside RenderState
**What:** Storing `fit_done` as a Leptos `RwSignal<bool>` or a static/thread-local that persists across data loads.
**Why bad:** After a second data fetch (user re-crawls), the graph would never auto-fit because `fit_done` remains true from the previous run.
**Instead:** `fit_done` lives as a field in `RenderState`, which is created fresh inside the `Effect` body each time it fires on new `graph_resource` data.

### Anti-Pattern 4: Label Collision Check in World Space
**What:** Comparing label bounding boxes using world coordinates, converting `measure_text()` pixel width to world units.
**Why bad:** Requires dividing by `viewport.scale` which is error-prone and obscures intent.
**Instead:** Convert node world positions to screen space first with `viewport.world_to_screen()`, then perform all collision checks in screen space using raw `measure_text()` pixel widths. Simpler and directly matches the drawn output.

### Anti-Pattern 5: Divergent Edge Rendering in Canvas2D vs WebGL2
**What:** Changing edge color only in `canvas_renderer.rs` and forgetting `webgl_renderer.rs`.
**Why bad:** `WEBGL_THRESHOLD = 300`. A 299-node graph gets the fix; a 301-node graph does not. The discrepancy is invisible during testing with small graphs.
**Instead:** Change both renderer files together in the same commit. Both files share the same visual specification; treat them as a matched pair.

---

## Sources

- Codebase (HIGH confidence, direct read): `resyn-app/src/graph/`, `resyn-worker/src/`, `resyn-app/src/pages/graph.rs`
- DPR convention documented in: `resyn-app/src/graph/renderer.rs` header comment
- WEBGL_THRESHOLD constant: `resyn-app/src/graph/renderer.rs` line 76
- Force constants: `resyn-worker/src/forces.rs` lines 5-11
- Alpha decay schedule: 0.995^460 ≈ 0.1 (derived from ALPHA_DECAY constant)
- Inline simulation note: `resyn-app/src/pages/graph.rs` comment at line ~133
- Label collision greedy approach: standard 2D rendering pattern, no library dependency
