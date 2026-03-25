---
phase: 16-edge-and-node-renderer-fixes
plan: "02"
subsystem: resyn-app/graph
tags: [webgl2-renderer, edge-rendering, node-rendering, seed-node, fwidth, quad-geometry]
dependency_graph:
  requires: [16-01]
  provides: [webgl2-quad-edges, webgl2-fwidth-nodes, webgl2-seed-ring, webgl2-depth-alpha]
  affects: [resyn-app/src/graph/webgl_renderer.rs]
tech_stack:
  added: []
  patterns: [quad-edge-geometry, fwidth-antialiasing, depth-based-alpha, seed-node-ring]
key_files:
  created: []
  modified:
    - resyn-app/src/graph/webgl_renderer.rs
    - resyn-app/src/graph/canvas_renderer.rs (fmt auto-fix)
    - 15 other workspace files (cargo fmt pre-existing fixes)
decisions:
  - "build_quad_edge uses world-space perpendicular offset (half_width = 0.75/scale) so existing EDGE_VERT shader needs no changes"
  - "depth_alpha_f32 mirrors Canvas 2D depth_alpha() exactly: max(from_depth, to_depth) with 0.50/0.35/0.25/0.15 thresholds"
  - "Seed outer ring uses edge shader program (triangle annulus) so no new shader or VAO needed"
metrics:
  duration: "12 minutes"
  completed: "2026-03-25"
  tasks_completed: 2
  files_modified: 18
---

# Phase 16 Plan 02: WebGL2 Renderer Fixes Summary

## One-liner

Updated WebGL2 renderer with fwidth-based node AA, bright borders, amber seed nodes with outer ring, quad triangle edge geometry (replacing GL.LINES), and #8b949e depth-based alpha matching Canvas 2D.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Update WebGL2 node shader and instance data for fwidth AA, borders, and seed nodes | 86ed9f3 | webgl_renderer.rs |
| 2 | Update WebGL2 edge rendering — quad geometry, depth alpha, color fix | 9ce524b | webgl_renderer.rs + 17 fmt-fix files |

## Decisions Made

1. `build_quad_edge` uses world-space perpendicular offset: `half_width = 0.75 / scale`. This means the existing EDGE_VERT shader (which transforms world-space positions via `world * scale + offset`) works unchanged. No new shader needed for quad edges.
2. `depth_alpha_f32` mirrors Canvas 2D `depth_alpha()` exactly: uses `max(from_depth, to_depth)` with 0.50/0.35/0.25/0.15 thresholds. Both renderers now produce identical depth-based dimming.
3. Seed outer ring reuses the edge shader program (triangle annulus built with `push_edge_vertex`). No new VAO, buffer, or shader is needed — the existing `draw_edge_pass` handles it.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Pre-existing cargo fmt failures in canvas_renderer.rs and workspace files**
- **Found during:** Task 2 verification (`cargo fmt --all -- --check`)
- **Issue:** Plan 01 and earlier changes left canvas_renderer.rs (and 16 other workspace files) with formatting inconsistencies. The plan's acceptance criteria requires `cargo fmt --all -- --check` to exit 0.
- **Fix:** Ran `cargo fmt --all` to apply rustfmt formatting across all workspace crates
- **Files modified:** canvas_renderer.rs, interaction.rs, layout_state.rs, lod.rs, renderer.rs, worker_bridge.rs, drawer.rs, graph.rs, server_fns/graph.rs, highlight.rs, llm_annotation.rs, prompt.rs, serve.rs, barnes_hut.rs, forces.rs, lib.rs
- **Commit:** 9ce524b

**2. [Rule 1 - Bug] clippy::too_many_arguments on build_quad_edge**
- **Found during:** Task 2 clippy check
- **Issue:** `build_quad_edge` has 8 arguments, exceeding the default limit of 7. Clippy with -D warnings rejects this.
- **Fix:** Added `#[allow(clippy::too_many_arguments)]` consistent with pattern already used on `draw_edge_pass`
- **Files modified:** webgl_renderer.rs
- **Commit:** 9ce524b

## Known Stubs

None — all changes are complete functional implementations with no placeholders.

## Verification

- `cargo test --workspace`: 257 tests pass (50 resyn-app + 186 resyn-core + 6 DB + 15 resyn-worker)
- `cargo fmt --all -- --check`: exits 0
- `cargo clippy -p resyn-app -- -D warnings`: exits 0
- `webgl_renderer.rs` NODE_FRAG contains `fwidth` and does NOT contain `smoothstep(0.9, 1.0, d)`
- `webgl_renderer.rs` edge draw mode is `TRIANGLES` not `LINES`
- `webgl_renderer.rs` does not contain `#404040` or `#30363d`
- Both Canvas 2D (Plan 01) and WebGL2 (this plan) use `#8b949e` edge color with same depth-alpha values

## Self-Check: PASSED

- resyn-app/src/graph/webgl_renderer.rs: FOUND
- Commit 86ed9f3: FOUND
- Commit 9ce524b: FOUND
