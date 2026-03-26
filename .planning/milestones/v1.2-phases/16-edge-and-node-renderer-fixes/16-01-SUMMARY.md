---
phase: 16-edge-and-node-renderer-fixes
plan: "01"
subsystem: resyn-app/graph
tags: [canvas-renderer, node-state, edge-colors, seed-node, lod, interaction]
dependency_graph:
  requires: []
  provides: [NodeState.is_seed, canvas-edge-colors, canvas-node-borders, canvas-seed-ring]
  affects: [resyn-app/src/graph/layout_state.rs, resyn-app/src/graph/canvas_renderer.rs]
tech_stack:
  added: []
  patterns: [depth-based-alpha, viewport-scale-compensation, seed-node-distinction]
key_files:
  created: []
  modified:
    - resyn-app/src/graph/layout_state.rs
    - resyn-app/src/graph/canvas_renderer.rs
    - resyn-app/src/graph/lod.rs
    - resyn-app/src/graph/interaction.rs
decisions:
  - "depth_alpha function uses max BFS depth of edge endpoints (not average) for alpha selection"
  - "Seed ring uses node.radius + 3.5px offset (2px gap + 1.5 center) with 3px line width"
  - "All line widths divided by viewport.scale for screen-space consistency at all zoom levels"
metrics:
  duration: "8 minutes"
  completed: "2026-03-25"
  tasks_completed: 2
  files_modified: 4
---

# Phase 16 Plan 01: Edge and Node Renderer Fixes Summary

## One-liner

Added `is_seed` to NodeState data model and updated Canvas 2D renderer with #8b949e citation edges using BFS-depth-based alpha, bright viewport-compensated node borders (#7cb8ff/#e8b84b), and amber seed node (#d29922) with outer planetary ring.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add is_seed to NodeState and update all test helpers | ad581b7 | layout_state.rs, lod.rs, interaction.rs |
| 2 | Update Canvas 2D renderer — edges, nodes, seed ring | 96dfd17 | canvas_renderer.rs |

## Decisions Made

1. `depth_alpha` uses the max BFS depth of the two edge endpoints (not average), so edges connecting deeper nodes are progressively dimmer — matches the visual intent of showing structure hierarchy.
2. Seed outer planetary ring offset: `node.radius + 2.0 + 1.5` (2px visual gap + 1.5px ring center), drawn with `3.0 / viewport.scale` line width so it remains 3px screen-space at all zoom levels.
3. All line widths (`1.5 / viewport.scale` for edges, `1.0 / viewport.scale` for borders, `3.0 / viewport.scale` for rings) follow the DPR convention established in Phase 14: CSS pixels throughout, compensation only at canvas physical sizing.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all changes are complete functional implementations with no placeholders.

## Verification

- `cargo test -p resyn-app`: 50 tests pass (including 2 new is_seed tests)
- `cargo check -p resyn-app`: no warnings
- `canvas_renderer.rs` contains no references to old colors `#404040` or `#30363d`
- `layout_state.rs` has `is_seed: bool` on NodeState and `is_seed` computed in `from_graph_data`
- `depth_alpha` function exists and is called in both edge rendering loop and arrowhead pass

## Self-Check: PASSED

- resyn-app/src/graph/layout_state.rs: FOUND
- resyn-app/src/graph/canvas_renderer.rs: FOUND
- resyn-app/src/graph/lod.rs: FOUND
- resyn-app/src/graph/interaction.rs: FOUND
- Commit ad581b7: FOUND
- Commit 96dfd17: FOUND
