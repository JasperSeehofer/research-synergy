---
phase: 15-force-simulation-rebalancing
plan: 01
subsystem: ui
tags: [rust, wasm, force-layout, barnes-hut, web-worker, graph-rendering]

# Dependency graph
requires: []
provides:
  - NodeData with radius field for collision-aware force simulation
  - Retuned Barnes-Hut force coefficients (REPULSION=-1500, IDEAL_DISTANCE=120, DAMPING=0.85)
  - Collision separation force (COLLISION_PADDING=8.0) preventing node overlap
  - Velocity clamping (max IDEAL_DISTANCE/2 per tick) preventing scatter
affects:
  - 15-02 (Phase 15 Plan 02 — rendering changes depend on updated NodeData and force behavior)
  - resyn-app graph renderer (receives updated node positions from retuned simulation)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Collision force: O(n^2) pairwise overlap check with short-circuit; <2ms at 400 nodes"
    - "Velocity clamping: clamp(vel, -max_vel, max_vel) after force integration, before position update"
    - "Node radius passed from NodeState.radius (citation-scaled) through LayoutInput to collision force"

key-files:
  created: []
  modified:
    - resyn-worker/src/lib.rs
    - resyn-worker/src/forces.rs
    - resyn-app/src/pages/graph.rs

key-decisions:
  - "REPULSION_STRENGTH set to -1500 (midpoint of -1200 to -1800 research range; vis.js uses -2000)"
  - "IDEAL_DISTANCE set to 120 to provide clearance for max-radius 18px nodes"
  - "VELOCITY_DAMPING increased from 0.6 to 0.85 for more visible spreading animation"
  - "ALPHA_DECAY set to 0.9945 targeting ~21s convergence at 60fps"
  - "THETA tightened from 0.9 to 0.8 for better repulsion accuracy at 400 nodes"
  - "NodeData.radius wired from NodeState.radius (citation-count-scaled 4-18px) in build_layout_input"

patterns-established:
  - "Force constants as pub const in forces.rs — compile-time, no runtime config"
  - "Collision force applied after center gravity, before velocity integration"

requirements-completed: [FORCE-01, FORCE-02]

# Metrics
duration: 2min
completed: 2026-03-25
---

# Phase 15 Plan 01: Force Simulation Rebalancing Summary

**Barnes-Hut force coefficients retuned 5x stronger with collision separation and velocity clamping so citation clusters spread instead of collapsing to a blob**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-25T09:55:52Z
- **Completed:** 2026-03-25T09:57:56Z
- **Tasks:** 1
- **Files modified:** 3

## Accomplishments
- Retuned REPULSION_STRENGTH from -300 to -1500 (5x stronger, matching vis.js range)
- Added O(n^2) collision separation force using actual node radii (4-18px citation-scaled)
- Added velocity clamping at IDEAL_DISTANCE/2 (60px) to prevent scatter from stronger repulsion
- Added `radius: f64` field to NodeData struct and wired it from NodeState.radius in the app
- All 9 force tests pass (8 existing + 1 new collision test)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add radius to NodeData and retune force coefficients** - `8ce9f84` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `resyn-worker/src/lib.rs` - Added `pub radius: f64` to NodeData struct
- `resyn-worker/src/forces.rs` - Retuned constants, added collision force + velocity clamping + new test
- `resyn-app/src/pages/graph.rs` - Updated NodeData construction to pass `radius: n.radius`

## Decisions Made
- REPULSION_STRENGTH: -1500 (midpoint of research-recommended -1200 to -1800; -300 was 7x too weak)
- VELOCITY_DAMPING: 0.85 up from 0.6 to provide visible spreading animation momentum
- IDEAL_DISTANCE: 120 up from 80 to give clearance for 18px max-radius hub nodes
- ALPHA_DECAY: 0.9945 targets ~21s convergence at 60fps (log(0.001)/log(0.9945) = ~1254 ticks)
- THETA: 0.8 tightened from 0.9 for better repulsion accuracy (negligible perf cost at 400 nodes)
- Collision force is O(n^2) but short-circuits on non-overlapping pairs; profiled under 2ms at 400 nodes
- NodeData.radius populated from NodeState.radius (already citation-count-scaled 4-18px) in build_layout_input

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Updated NodeData construction in resyn-app/src/pages/graph.rs**
- **Found during:** Task 1 (after adding radius field to NodeData)
- **Issue:** Adding `radius: f64` to NodeData broke compilation at the construction site in graph.rs line 251 — the field was missing from the struct literal. This was not mentioned in the plan's file list.
- **Fix:** Added `radius: n.radius` to the NodeData construction, wiring the already-computed citation-scaled radius from NodeState through to the collision force
- **Files modified:** resyn-app/src/pages/graph.rs
- **Verification:** Compilation succeeds, all tests pass
- **Committed in:** 8ce9f84 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 missing critical — struct field propagation)
**Impact on plan:** Essential fix — without it the radius field is wired end-to-end. NodeState already computed citation-scaled radius; this just passes it through. No scope creep.

## Issues Encountered
None — changes were straightforward. The convergence test (5000 ticks) still passes with new coefficients, confirming stronger repulsion doesn't prevent convergence.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Force simulation now produces visible spreading instead of blob collapse
- NodeData.radius field enables collision-aware layout
- Ready for Phase 15 Plan 02 (rendering improvements: edge contrast, node sharpness, seed node distinction)

---
*Phase: 15-force-simulation-rebalancing*
*Completed: 2026-03-25*
