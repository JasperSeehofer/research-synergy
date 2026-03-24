# Phase 13: Graph Interaction - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Fix graph interaction so that: (1) users can click and drag individual nodes to reposition them, (2) users can pan the viewport by dragging empty canvas space, and (3) users can zoom in/out with the scroll wheel. This is a bugfix phase — all interaction code (event handlers, state machine, hit testing, viewport transforms) already exists but is reported non-functional.

</domain>

<decisions>
## Implementation Decisions

### Overlay / Pointer Event Blocking
- **D-01:** The primary suspect is CSS overlay elements blocking pointer events from reaching the canvas. The `.graph-controls-overlay` (z-index: 10), tooltip overlay (z-index: 50/200), and temporal slider container may intercept events. Diagnose by checking if mousedown/mousemove/wheel events actually fire on the canvas element. Fix by ensuring overlays use `pointer-events: none` except on their interactive children (buttons, slider thumbs).
- **D-02:** STATE.md from Phase 12 explicitly flags this: "Canvas may be covered by an overlay element (z-index), blocking all pointer events — check first before debugging event listener logic."

### Coordinate Transform Verification
- **D-03:** Phase 12 established the DPR convention: all coordinate math in CSS pixels, DPR applied only at canvas physical sizing and GL viewport. The `screen_to_world` / `world_to_screen` transforms in `Viewport` must be verified against this convention — the Phase 12 DPR fix may have broken the coordinate mapping. Use console logging of (screen_x, screen_y) → (world_x, world_y) to verify hit testing targets the correct node.
- **D-04:** If the coordinate transform is off, fix it in the `Viewport` struct — do not work around it in event handlers.

### Force Reheat on Drag
- **D-05:** Keep the existing moderate reheat behavior: `alpha = max(current_alpha, 0.3)` on drag release. This is already implemented in the mouseup handler. Only tune if live testing shows nodes don't settle or settle too fast after drag.
- **D-06:** Dragged nodes are pinned during drag (existing behavior). On click-release (< 3px movement), the node is unpinned. On drag-release, the node stays pinned. This is correct — preserve it.

### Hit Test Tuning
- **D-07:** Keep current hit test parameters: node radius for node detection, 4px threshold for edge detection. The `find_node_at()` function iterates in reverse render order (topmost first). Only adjust thresholds if live testing shows consistent misses.
- **D-08:** Hit testing uses all nodes regardless of LOD visibility. This is intentional — users can interact with nodes even if labels are hidden at the current zoom level.

### Claude's Discretion
- Debugging approach: console logging, event listener verification, CSS inspection — whatever is fastest to identify the root cause
- Whether to add temporary debug logging (remove after fix confirmed)
- Order of investigation (overlay blocking → coordinate transforms → force behavior)
- Whether the click-vs-drag threshold (3px) needs adjustment based on testing

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Interaction Logic
- `resyn-app/src/graph/interaction.rs` — `InteractionState` enum (Idle/Panning/DraggingNode), `find_node_at()`, `find_edge_at()`, `zoom_toward_cursor()` — 7 unit tests
- `resyn-app/src/graph/renderer.rs` — `Viewport` struct with `screen_to_world()` / `world_to_screen()` transforms, scale clamping [0.1, 4.0]

### Event Handlers
- `resyn-app/src/pages/graph.rs` lines 462–701 — All mouse event closures: mousemove (491–547), mousedown (552–585), mouseup (587–635), wheel (654–674), dblclick (640–652), pointerleave (676–686)
- `resyn-app/src/pages/graph.rs` lines 31–40 — `RenderState` struct holding `GraphState`, `Viewport`, `InteractionState`, drag tracking fields

### CSS / Overlay Structure
- `resyn-app/style/main.css` lines 1331–1449 — `.graph-controls-overlay` (z-index: 10), tooltip (z-index: 200, pointer-events: none), temporal slider (pointer-events: none on container, pointer-events: all on thumbs)
- `resyn-app/src/components/graph_controls.rs` — Control buttons overlay div

### Force Simulation
- `resyn-worker/src/forces.rs` — Current constants: REPULSION=-300, ATTRACTION=0.03, CENTER_GRAVITY=0.005, ALPHA_DECAY=0.995, ALPHA_MIN=0.001, DAMPING=0.6
- `resyn-app/src/graph/layout_state.rs` — `GraphState` node positions, velocities, pinning

### Phase 12 Context
- `.planning/phases/12-graph-force-rendering/12-CONTEXT.md` — DPR convention decisions (D-02, D-03) that constrain coordinate handling here

### Known Bug State
- `.planning/STATE.md` — Blockers section documents overlay z-index concern, coordinate transform concern, and force tuning status

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `InteractionState` state machine: fully implemented with Idle/Panning/DraggingNode states and 7 passing tests
- `find_node_at()` / `find_edge_at()`: hit testing with reverse-order iteration and distance thresholds
- `zoom_toward_cursor()`: preserves world coords under cursor, scale clamping [0.1, 4.0]
- `Viewport::screen_to_world()` / `world_to_screen()`: coordinate transforms using CSS pixel space
- All mouse event handlers: closures already attached to canvas in `graph.rs`

### Established Patterns
- Closure-based event listeners captured in `Rc<RefCell<RenderState>>` for shared mutable access
- State machine transitions in event handlers: mousedown sets state, mousemove acts on state, mouseup resets to Idle
- Pinning: drag pins node, click unpins, tracked via `was_already_pinned` flag
- RAF loop reads `RenderState` each frame — no explicit re-render trigger needed after interaction changes

### Integration Points
- Event handlers modify `RenderState.viewport` (pan/zoom) and `RenderState.graph.nodes[idx]` (drag) directly
- RAF loop in `graph.rs` reads modified state each frame — changes are automatically rendered
- Force simulation `run_ticks()` respects pinned nodes (skips velocity update for pinned)
- LOD visibility does NOT gate interaction — hit testing uses all nodes

</code_context>

<specifics>
## Specific Ideas

- Check overlay blocking FIRST — this is the most likely root cause since all interaction code exists and is well-tested
- Use browser dev tools or agent-browser to verify events actually reach the canvas element
- If events do reach canvas, add console.log in mousedown handler to verify `find_node_at()` returns expected results
- Document any coordinate convention changes so Phase 14 (temporal controls) can rely on them

</specifics>

<deferred>
## Deferred Ideas

- Touch/mobile interaction support — future milestone
- Node selection highlighting / multi-select — future feature
- Right-click context menu on nodes — future feature
- Keyboard navigation (arrow keys to pan, +/- to zoom) — future feature

</deferred>

---

*Phase: 13-graph-interaction*
*Context gathered: 2026-03-23*
