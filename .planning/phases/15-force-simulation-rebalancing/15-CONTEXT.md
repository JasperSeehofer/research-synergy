# Phase 15: Force Simulation Rebalancing - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Retune the Barnes-Hut force simulation coefficients so citation graph nodes form visible organic clusters reflecting citation structure, instead of collapsing to a central blob. Add BFS depth ring initial placement and collision separation force. Simulation should fully stop after convergence.

</domain>

<decisions>
## Implementation Decisions

### Coefficient Tuning
- **D-01:** Keep force coefficients as compile-time `pub const` values in `forces.rs`. Runtime configurability deferred to CONFIG-01 (future requirement).
- **D-02:** Target organic cluster layout (like Connected Papers) — connected papers form loose clusters with clear spacing between groups. Not a structured radial tree.
- **D-03:** Add collision separation force that pushes overlapping nodes apart based on their radii. Requires adding `radius: f64` to `NodeData` in `resyn-worker/src/lib.rs` (LayoutInput schema change).
- **D-04:** Smooth animation quality matters — the spreading-out process should look natural and satisfying, not jittering or popping. Tune damping/alpha for visual quality during transition.
- **D-05:** Drag reheat: local rearrangement only. Keep current behavior (`alpha = max(0.3, current)`). No full graph re-simulation on drag.

### BFS Depth Ring Placement
- **D-06:** Seed paper placed at a slight offset from center (0,0) to break symmetry and help force sim converge faster.
- **D-07:** Nodes arranged in concentric rings by BFS depth. Depth-0 (seed) near center, depth-1 in first ring, depth-2 in second ring, etc.
- **D-08:** Nodes without `bfs_depth` (orphans not reachable from seed) placed in the outermost ring, scattered. Simulation handles their final positioning.

### Convergence Behavior
- **D-09:** Simulation fully stops when alpha drops below threshold. No continuous idle forces. Saves CPU. Drag reheat restarts simulation temporarily.
- **D-10:** Target convergence time: 15-20 seconds for a ~350 node graph. Moderate speed — smooth spreading animation with time to watch clusters form.

### Claude's Discretion
- Exact coefficient values (repulsion, attraction, damping, ideal distance, alpha decay) — research provides ranges, Claude calibrates empirically
- Ring spacing formula for BFS depth placement
- Alpha threshold for full stop convergence
- Collision force strength and distance calculations

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Force Simulation
- `resyn-worker/src/forces.rs` — Current force constants and simulation_tick() implementation
- `resyn-worker/src/barnes_hut.rs` — Barnes-Hut quadtree for O(n log n) repulsion
- `resyn-worker/src/lib.rs` — LayoutInput/LayoutOutput/NodeData structs (NodeData needs radius field)

### Graph State & Initialization
- `resyn-app/src/graph/layout_state.rs` — `from_graph_data()` handles initial node placement (lines 59-115), `NodeState` has `bfs_depth: Option<u32>`
- `resyn-app/src/pages/graph.rs` — RAF loop calling `run_ticks()` (lines 350-367), alpha floor logic

### Research
- `.planning/research/FEATURES.md` — Force coefficient analysis table with reference values from vis.js and d3-force
- `.planning/research/STACK.md` — Recommended coefficient ranges and collision force approach
- `.planning/research/PITFALLS.md` — Force tuning pitfalls, graph density interaction warnings

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `forces.rs` — Complete simulation tick with Barnes-Hut repulsion, Hooke's law springs, center gravity. Only constants need changing + collision force added.
- `layout_state.rs:from_graph_data()` — Already receives `bfs_depth` on each node. Initial placement logic isolated here.
- 8 existing force tests — convergence, attraction, repulsion, pinned nodes, alpha decay tests. Must pass after coefficient changes.

### Established Patterns
- Force constants are `pub const` at module top — simple to modify
- `NodeData` struct in `resyn-worker/src/lib.rs` carries per-node data to simulation. Adding `radius: f64` follows existing pattern (has `mass: f64`).
- `build_layout_input()` in `graph.rs` maps `GraphState` to `LayoutInput` — will need to pass node radius here too.

### Integration Points
- `resyn-worker/src/lib.rs` — `NodeData` struct needs `radius` field for collision force
- `resyn-app/src/pages/graph.rs:356` — `build_layout_input()` needs to include radius from `NodeState`
- `resyn-app/src/graph/layout_state.rs:60-115` — `from_graph_data()` needs new BFS ring placement logic replacing current radial jitter
- `resyn-app/src/pages/graph.rs:366` — Alpha floor logic (`output.alpha.max(ALPHA_MIN)`) must change to allow full stop

</code_context>

<specifics>
## Specific Ideas

- User wants the graph to look like Connected Papers — organic clusters, not a rigid radial tree
- Animation quality is important: the spreading-out process should be satisfying to watch
- 15-20 second convergence for 350 nodes — not too fast, not too slow

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 15-force-simulation-rebalancing*
*Context gathered: 2026-03-24*
