# Phase 17: Viewport Fit and Label Collision - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Auto-fit the graph viewport after force layout converges so all nodes are visible without manual pan/zoom. Add priority-ordered label collision avoidance so node labels are readable without overlap. Wire a convergence status indicator into the graph controls. Add a manual "Fit" button for re-triggering viewport fit on demand.

</domain>

<decisions>
## Implementation Decisions

### Auto-Fit Behavior
- **D-01:** Smooth animated pan-zoom transition (~0.5s lerp of scale + offset) after convergence. Not an instant snap.
- **D-02:** 10% viewport margin padding on each side when computing the fit bounding box.
- **D-03:** Bounding box computed from visible nodes only (LOD-visible AND temporal-visible). Filtered-out nodes are excluded.
- **D-04:** Auto-fit triggers only after initial convergence (alpha < ALPHA_MIN), not on graph load. User watches the spreading animation at default viewport, then the camera smoothly frames the result.

### User Override Latch
- **D-05:** Any manual pan or zoom interaction permanently sets a `user_has_interacted` flag. Auto-fit never fires again automatically once set.
- **D-06:** A "Fit" button in GraphControls allows the user to manually re-trigger the same smooth fit animation at any time.
- **D-07:** Drag reheat (which restarts force ticks temporarily) does NOT re-trigger auto-fit and does NOT reset the latch. Drag is a deliberate spatial adjustment.
- **D-08:** Fit button placed in the same control group as zoom +/- buttons. Uses an expand arrows icon (Unicode).

### Label Collision Avoidance
- **D-09:** Priority order: seed paper first, then descending citation count. Matches LABEL-01 requirement.
- **D-10:** Sparse label placement with generous padding between bounding boxes. Clean look over maximum density.
- **D-11:** Hovering over any node always reveals its label, even if culled by collision avoidance.
- **D-12:** Label collision layout is cached and only recomputed on viewport changes (zoom, pan, fit). Not recomputed every frame. measureText results cached at graph load time per STATE.md requirement.

### Label Appearance
- **D-13:** All labels have uniform style regardless of priority tier. Priority only affects which labels survive collision culling.
- **D-14:** Labels rendered as modern pill/badge style: opaque background with thin border. Not raw floating text.
- **D-15:** Label pill colors: background rgba(13,17,23,0.85) (semi-transparent matching graph bg), border #30363d (subtle GitHub-style), text #cccccc. Clean, modern, doesn't compete with nodes.
- **D-16:** Font remains 11px monospace (consistent with current Canvas 2D labels).

### Convergence Indicator
- **D-17:** Text status badge in GraphControls showing three states: "Simulating..." (while running), "Paused" (user-paused), "Settled" (naturally converged).
- **D-18:** Badge distinguishes user-paused from naturally converged so the user knows whether they can resume.

### Claude's Discretion
- Lerp easing function for the animated fit transition (linear, ease-out, etc.)
- Exact collision bounding box padding multiplier for "generous spacing"
- measureText cache invalidation strategy (font change, node data change, etc.)
- Hover label z-ordering and animation (fade in vs instant)
- Label pill corner radius and internal padding
- Status badge CSS styling (color, position within controls group)
- Whether collision recomputation also triggers on simulation tick during convergence animation or only after full stop
- WebGL2 label rendering path (Canvas 2D overlay text for both renderers, or separate approach)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Viewport & Auto-Fit
- `resyn-app/src/graph/renderer.rs` lines 21-75 -- Viewport struct (offset_x/y, scale, css_width/height, world_to_screen, screen_to_world, apply)
- `resyn-app/src/pages/graph.rs` lines 388-406 -- RAF loop: convergence check at line 404, simulation_running.set(false) on convergence
- `resyn-app/src/graph/layout_state.rs` lines 210-220 -- check_alpha_convergence() method

### Label Rendering (Current)
- `resyn-app/src/graph/canvas_renderer.rs` lines 288-308 -- Canvas 2D label drawing (naive, all visible nodes, scale > 0.6, 11px monospace, measureText per frame)
- `resyn-app/src/graph/webgl_renderer.rs` lines 182-197 -- node_screen_positions() returning (sx, sy, label) tuples for overlay text

### LOD & Visibility
- `resyn-app/src/graph/lod.rs` lines 1-44 -- LOD visibility rules, update_lod_visibility, update_temporal_visibility, compute_visible_count
- `resyn-app/src/graph/layout_state.rs` lines 1-57 -- NodeState struct (id, citation_count, is_seed, lod_visible, temporal_visible, radius), GraphState struct

### Controls
- `resyn-app/src/components/graph_controls.rs` -- GraphControls component (play/pause, zoom +/-, visible count), TemporalSlider
- `resyn-app/src/graph/interaction.rs` -- find_node_at, find_edge_at, zoom_toward_cursor

### Requirements
- `.planning/REQUIREMENTS.md` -- VIEW-01, VIEW-02, LABEL-01, LABEL-02

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Viewport` struct already has all fields needed for fit computation (offset_x/y, scale, css_width/height, world_to_screen)
- `check_alpha_convergence()` in GraphState -- exact hook point for triggering auto-fit
- `compute_visible_count()` in lod.rs -- pattern for iterating visible nodes (can be extended to compute bounding box)
- `GraphControls` component -- ready to accept new props for fit button and convergence status
- `node_screen_positions()` in WebGL2Renderer -- already computes screen-space label positions

### Established Patterns
- Viewport transform: `world_to_screen(wx, wy) = (wx * scale + offset_x, wy * scale + offset_y)` -- reverse this for fit
- LOD visibility filtering: `node.lod_visible && node.temporal_visible` -- same predicate for fit bounding box
- GraphControls receives `RwSignal` props -- add new signals for fit trigger and convergence state
- Canvas 2D labels use `ctx.measure_text()` for width -- must cache per STATE.md note

### Integration Points
- `graph.rs` RAF loop line 404 -- after convergence detected, trigger auto-fit animation
- `graph.rs` mouse/wheel handlers -- set `user_has_interacted` flag on pan/zoom events
- `GraphControls` component -- add Fit button and convergence status badge
- `canvas_renderer.rs` label section (lines 288-308) -- replace naive loop with collision-aware rendering
- `NodeState` or new struct -- label collision cache (sorted priority list + visible label indices)

</code_context>

<specifics>
## Specific Ideas

- Labels as modern pill/badges with opaque background + thin border -- user specifically wants a clean, modern tooltip look rather than raw floating text
- Generous label spacing -- user prefers sparse, clean labels over maximum density
- Three-state convergence badge (Simulating / Paused / Settled) -- user wants to distinguish user-paused from naturally converged
- Fit button as icon next to zoom +/- -- consistent with existing minimalist control style

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 17-viewport-fit-and-label-collision*
*Context gathered: 2026-03-25*
