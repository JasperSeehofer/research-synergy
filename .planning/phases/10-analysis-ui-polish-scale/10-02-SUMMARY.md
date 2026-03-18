---
phase: 10-analysis-ui-polish-scale
plan: "02"
subsystem: ui
tags: [rust, wasm, graph, lod, temporal-filter, visibility]

# Dependency graph
requires:
  - phase: 09-graph-renderer-canvas-to-webgl
    provides: NodeState, GraphState, GraphData DTO, canvas/WebGL renderer pipeline
provides:
  - NodeState with bfs_depth, lod_visible, temporal_visible fields
  - GraphState with temporal_min_year, temporal_max_year, seed_paper_id, current_scale
  - LOD visibility functions (update_lod_visibility, update_temporal_visibility, compute_visible_count)
  - Year bounds computed from graph data at load time
affects:
  - 10-03 (RAF loop integration — consumes update_lod_visibility/update_temporal_visibility)
  - 10-04 (temporal UI controls — uses GraphState temporal fields)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pure Rust LOD logic in separate lod.rs module — no web-sys, testable natively with cargo test"
    - "Visibility state co-located on NodeState (lod_visible, temporal_visible) — renderer reads flags, doesn't compute"
    - "Year bounds eagerly computed in from_graph_data() — O(n) once, not per-frame"

key-files:
  created:
    - resyn-app/src/graph/lod.rs
  modified:
    - resyn-app/src/graph/layout_state.rs
    - resyn-app/src/graph/mod.rs
    - resyn-app/src/graph/interaction.rs
    - resyn-app/src/server_fns/graph.rs

key-decisions:
  - "LOD thresholds: LOD_LEVEL_0=0.3 (depth<=1 only), LOD_LEVEL_1=0.6 (+citations>=50), LOD_LEVEL_2=1.0 (+depth<=2 or citations>=10)"
  - "Nodes with unparseable/empty year are always temporal_visible=true — missing data stays visible"
  - "bfs_depth and seed_paper_id added to GraphNode/GraphData DTOs in this plan (Plan 01 not yet executed)"

patterns-established:
  - "Visibility flags on NodeState pattern: renderer reads lod_visible && temporal_visible; update functions mutate flags; avoids renderer logic bloat"
  - "LOD progressive reveal: seed always visible, then depth-gated, then citation-count-gated at higher scales"

requirements-completed: [SCALE-02, SCALE-03]

# Metrics
duration: 13min
completed: 2026-03-18
---

# Phase 10 Plan 02: LOD and Temporal Visibility State Summary

**LOD progressive-reveal thresholds and temporal year-range filtering as pure Rust functions on NodeState/GraphState, with 27 unit tests running natively (no WASM required)**

## Performance

- **Duration:** 13 min
- **Started:** 2026-03-18T15:12:08Z
- **Completed:** 2026-03-18T15:24:50Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Extended NodeState with `bfs_depth`, `lod_visible`, `temporal_visible` fields
- Extended GraphState with `temporal_min_year`, `temporal_max_year`, `seed_paper_id`, `current_scale`
- Year bounds computed eagerly in `from_graph_data()` — filters invalid years (non-1900–2100)
- Created `lod.rs` module with three public functions and LOD_LEVEL_0/1/2 constants
- 27 total tests (14 layout_state + 13 lod) all passing natively

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend NodeState and GraphState with LOD/temporal fields** - `28edb76` (feat)
2. **Task 2: Create LOD and temporal visibility computation module** - `e83e002` (feat)

**Plan metadata:** (see final commit)

_Note: TDD tasks both went RED-then-GREEN — tests written first, implementation second_

## Files Created/Modified
- `resyn-app/src/graph/lod.rs` - LOD/temporal visibility computation (update_lod_visibility, update_temporal_visibility, compute_visible_count + 13 tests)
- `resyn-app/src/graph/layout_state.rs` - NodeState/GraphState extended with 7 new fields + 8 new tests
- `resyn-app/src/graph/mod.rs` - Added `pub mod lod`
- `resyn-app/src/graph/interaction.rs` - Updated make_node test helper for new NodeState fields
- `resyn-app/src/server_fns/graph.rs` - Added `bfs_depth` to GraphNode, `seed_paper_id` to GraphData

## Decisions Made
- LOD threshold constants (0.3/0.6/1.0) baked in as `pub const` — readable and testable by downstream plans
- Nodes with unparseable year default to `temporal_visible=true` (missing data visible) — avoids hiding papers with missing metadata
- Seed node always visible regardless of LOD scale — ensures starting paper is always accessible

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added bfs_depth to GraphNode and seed_paper_id to GraphData DTOs**
- **Found during:** Task 1 (NodeState extension)
- **Issue:** Plan 01 hadn't been executed yet — GraphNode lacked `bfs_depth`, GraphData lacked `seed_paper_id`. Task 1 requires `bfs_depth: n.bfs_depth` in NodeState construction, which would fail without the DTO field.
- **Fix:** Added `bfs_depth: Option<u32>` to `GraphNode`, `seed_paper_id: Option<String>` to `GraphData`. Updated server fn constructor and existing tests. Set `bfs_depth: None` and `seed_paper_id: None` as defaults in server fn (BFS depth population deferred to Plan 01).
- **Files modified:** resyn-app/src/server_fns/graph.rs
- **Verification:** All 44 lib tests pass, cargo check clean
- **Committed in:** `28edb76` (part of Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking — missing DTO fields from unapplied Plan 01)
**Impact on plan:** Necessary to unblock Task 1. Plan 01 can still add BFS computation logic without conflict since defaults are None.

## Issues Encountered
None — compilation was clean after DTO field additions.

## Next Phase Readiness
- LOD and temporal visibility logic is complete and tested
- Ready for Plan 03: RAF loop integration (update_lod_visibility called per frame with current viewport scale)
- Ready for Plan 04: temporal UI slider controls (reads GraphState temporal_min_year/temporal_max_year)
- No blockers

---
*Phase: 10-analysis-ui-polish-scale*
*Completed: 2026-03-18*
