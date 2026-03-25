---
phase: 15-force-simulation-rebalancing
plan: 02
subsystem: ui
tags: [rust, wasm, force-layout, bfs, graph-rendering, leptos]

# Dependency graph
requires:
  - phase: 15-01
    provides: NodeData with radius field, retuned force coefficients, collision force
provides:
  - BFS concentric ring initial placement (seed near origin, depth-N on Nth ring, orphans outermost)
  - Alpha full-stop convergence stopping simulation at ALPHA_MIN (D-09)
  - Drag reheat restarting stopped simulation (D-05)
  - check_alpha_convergence() testable method on GraphState
  - Viewport fit-scale based on actual node spread (not synthetic estimate)
affects:
  - 15-03 (visual checkpoint — depends on ring placement for cluster quality)
  - resyn-app graph renderer (initial layout now structurally meaningful)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "BFS ring placement: depth_counts + depth_positions hashmaps for O(n) ring assignment"
    - "Alpha full-stop: check_alpha_convergence() extracts logic to testable method, RAF loop syncs Leptos signal"
    - "Drag reheat: reheat_simulation flag pattern to set Leptos signal after dropping RefCell borrow"
    - "Viewport fit: use actual max node distance (from positions) not synthetic spread estimate"

key-files:
  created: []
  modified:
    - resyn-app/src/graph/layout_state.rs
    - resyn-app/src/pages/graph.rs

key-decisions:
  - "base_ring_spacing = 180px (1.5x IDEAL_DISTANCE=120) so nodes start beyond equilibrium for visible spreading animation"
  - "Seed node (depth-0) placed at (5+jx, 5+jy) with 10px jitter — near origin but with asymmetry"
  - "Orphan nodes assigned to max_bfs_depth+1 ring — consistently outermost without hardcoding"
  - "check_alpha_convergence() extracted to GraphState method for testability (avoids testing within Leptos RAF closure)"
  - "reheat_simulation flag used to set Leptos signal after RefCell borrow is dropped"

patterns-established:
  - "BFS ring placement: pre-compute depth_counts before map(), track depth_positions as mutable HashMap"
  - "Alpha convergence test: test the extracted method directly, not the RAF loop behavior"

requirements-completed: [FORCE-02, FORCE-03]

# Metrics
duration: 6min
completed: 2026-03-25
---

# Phase 15 Plan 02: Force Simulation Rebalancing Summary

**BFS concentric ring placement with alpha full-stop convergence so seed paper starts near center with depth rings spreading outward and simulation halts on CPU when settled**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-25T10:00:31Z
- **Completed:** 2026-03-25T10:06:32Z
- **Tasks:** 2 (Task 3 is human-verify checkpoint, not committed)
- **Files modified:** 2

## Accomplishments
- Replaced single-circle placement with BFS depth concentric rings (seed near origin, Nth ring at 180*N px)
- Added alpha full-stop: simulation halts when alpha < ALPHA_MIN (0.001), saving CPU on settled graphs
- Fixed drag reheat: `simulation_running = true` + Leptos signal sync ensures force ticks resume after full-stop
- Added 4 new tests: test_from_graph_data_seed_near_origin, test_from_graph_data_bfs_ring_placement, test_from_graph_data_orphan_outer_ring, test_alpha_stops_simulation
- Updated viewport fit-scale to use actual max node distance from positions (correct for ring layout)

## Task Commits

Each task was committed atomically:

1. **Task 1: BFS ring placement and radius propagation** - `ad79f84` (feat)
2. **Task 2: Alpha full-stop convergence, drag reheat fix, and alpha-stop test** - `4b8feaf` (feat)

**Plan metadata:** (docs commit follows)

_Note: TDD tasks have RED (compile failure) → GREEN (passing) flow for each test_

## Files Created/Modified
- `resyn-app/src/graph/layout_state.rs` - BFS ring placement in from_graph_data(), check_alpha_convergence() method, 4 new tests
- `resyn-app/src/pages/graph.rs` - Alpha full-stop in RAF loop, drag reheat fix, viewport fit-scale from node positions, simulation_running passed to attach_event_listeners

## Decisions Made
- base_ring_spacing = 180px (1.5x IDEAL_DISTANCE) so initial positions are beyond equilibrium — produces visible outward spreading animation per D-04
- Seed node at (5+jx, 5+jy) with small jitter: near origin for visual clarity but with asymmetry so force sim doesn't get stuck in perfect symmetry
- Orphan nodes use `max_bfs_depth + 1` as their ring — dynamically outermost without hardcoded ring numbers
- `check_alpha_convergence()` extracted to `GraphState` method for testability — avoids testing within WASM/Leptos RAF closure
- `reheat_simulation` flag set before `drop(s)` then `simulation_running.set(true)` called after — required pattern to avoid RefCell double-borrow with Leptos signal reactivity

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Removed unused `node_count` variable**
- **Found during:** Task 1 (after replacing placement logic)
- **Issue:** `node_count` was computed but no longer used after BFS ring placement removed the spread formula; clippy would flag it
- **Fix:** Removed the `let node_count = ...` line
- **Files modified:** resyn-app/src/graph/layout_state.rs
- **Verification:** cargo clippy clean for our changes
- **Committed in:** ad79f84 (Task 1 commit)

**2. [Rule 1 - Bug] Updated viewport fit-scale to match new ring layout**
- **Found during:** Task 1 (viewport initialization in graph.rs)
- **Issue:** graph.rs still used `(node_count as f64).sqrt() * 15.0` as `spread` for fit-scale calculation, which is wrong for ring layout (rings go to 180*max_depth px, not sqrt(n)*15 px)
- **Fix:** Replace synthetic spread with actual max node distance from positions (`nodes.iter().map(|n| distance_from_origin).max()`)
- **Files modified:** resyn-app/src/pages/graph.rs
- **Verification:** All tests pass
- **Committed in:** ad79f84 (Task 1 commit)

**3. [Rule 2 - Missing Critical] `radius: n.radius` already present (Plan 01 deviation)**
- Plan 01 already fixed the missing `radius: n.radius` in `build_layout_input` as part of its deviation. No action needed.

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical) + 1 pre-existing fix from Plan 01
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered
- `attach_event_listeners` did not receive `simulation_running` as a parameter — added it as a new parameter to enable drag reheat Leptos signal sync. This is a straightforward signature extension, not an architectural change.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- BFS ring placement provides structurally meaningful warm start for force simulation
- Alpha full-stop saves CPU on settled graphs (D-09 behavioral requirement now tested)
- Drag reheat correctly restarts stopped simulation (D-05)
- Visual checkpoint (Task 3) requires human verification: `trunk serve --open`, load graph, verify ring placement and cluster quality
- Ready for Phase 15 Plan 03 (edge rendering improvements) once visual checkpoint approved

## Self-Check: PASSED

- resyn-app/src/graph/layout_state.rs: FOUND
- resyn-app/src/pages/graph.rs: FOUND
- .planning/phases/15-force-simulation-rebalancing/15-02-SUMMARY.md: FOUND
- Commit ad79f84 (Task 1): FOUND
- Commit 4b8feaf (Task 2): FOUND
- base_ring_spacing in layout_state.rs: FOUND
- orphan_ring in layout_state.rs: FOUND
- depth_counts in layout_state.rs: FOUND
- check_alpha_convergence() in layout_state.rs: FOUND
- test_alpha_stops_simulation in layout_state.rs: FOUND
- s.graph.check_alpha_convergence() in graph.rs: FOUND
- simulation_running.set(false) in graph.rs: FOUND
- simulation_running.set(true) in graph.rs: FOUND
- s.graph.simulation_running = true in graph.rs: FOUND
- old alpha floor removed from graph.rs: CONFIRMED

---
*Phase: 15-force-simulation-rebalancing*
*Completed: 2026-03-25*
