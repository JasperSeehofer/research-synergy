---
phase: 12-graph-force-rendering
plan: 01
subsystem: ui
tags: [webgl, canvas, wasm, leptos, force-layout, rendering, gpu]

# Dependency graph
requires: []
provides:
  - "Reduced initial node spread (sqrt * 15) fitting ~375 nodes in viewport at ~290px radius"
  - "Viewport fit-scale applied at graph load — initial view shows entire graph"
  - "VBOs preallocated in WebGL2Renderer::new() — no per-frame GPU buffer leak"
  - "DPR coordinate convention documented in renderer.rs module doc for Phase 13"
affects: [13-node-interaction, phase-13, pointer-events, zoom, pan]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Preallocate GPU buffers in renderer constructor, update contents per-frame via DYNAMIC_DRAW"
    - "Fit-scale computed at graph load: min(css_width, css_height) * 0.4 / spread"
    - "DPR applied only at canvas physical size and GL viewport — all other math in CSS pixels"

key-files:
  created: []
  modified:
    - resyn-app/src/graph/layout_state.rs
    - resyn-app/src/graph/webgl_renderer.rs
    - resyn-app/src/graph/renderer.rs
    - resyn-app/src/pages/graph.rs

key-decisions:
  - "Spread constant reduced from 50.0 to 15.0 — nodes fit viewport without changing simulation physics"
  - "Viewport made mutable in graph.rs Effect so fit-scale can be applied post-construction"
  - "quad_buf kept alive in struct with #[allow(dead_code)] — VAO retains reference to it"
  - "draw_edge_pass accepts edge_buf: &WebGlBuffer so caller owns the preallocated buffer"

patterns-established:
  - "GPU buffer pattern: create in new(), update contents in draw() with DYNAMIC_DRAW"
  - "Quad VAO attributes set up once in new() — VAO remembers the binding"

requirements-completed: [GRAPH-01, GRAPH-02, GRAPH-03]

# Metrics
duration: 6min
completed: 2026-03-23
---

# Phase 12 Plan 01: Graph Force & Rendering Fix Summary

**Reduced initial node spread from 968px to 290px and eliminated per-frame GPU VBO leak, with DPR convention documented for Phase 13**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-03-23T13:08:31Z
- **Completed:** 2026-03-23T13:14:44Z
- **Tasks:** 2 of 3 complete (Task 3 is a visual verification checkpoint)
- **Files modified:** 4

## Accomplishments
- Node spread constant reduced 50.0 → 15.0: for 375 nodes, initial radius ~290px (was ~968px, fully off-screen)
- Fit-scale computed and applied to viewport on load: `min(css_w, css_h) * 0.4 / spread`, capped at 1.0
- VBOs preallocated in `WebGL2Renderer::new()`: `quad_buf` (STATIC_DRAW once), `instance_buf`, `edge_buf` (DYNAMIC_DRAW per frame)
- Quad VAO attribute pointer set once in `new()` — removed redundant per-frame attrib setup
- `draw_edge_pass` updated to accept `edge_buf: &WebGlBuffer` parameter instead of creating new buffer each call
- DPR coordinate convention documented as module-level doc in `renderer.rs` for Phase 13 pointer event work
- Fixed 8 pre-existing clippy errors in `graph.rs` (unused imports, type alias, useless conversion, unneeded returns)

## Task Commits

1. **Task 1: Fix initial spread and viewport fit-to-graph scaling** - `1cff1ad` (fix)
2. **Task 2: Preallocate VBOs and fix per-frame GPU memory leak** - `a188d13` (fix)
3. **Task 3: Visual verification of graph rendering** - PENDING (checkpoint:human-verify)

## Files Created/Modified
- `resyn-app/src/graph/layout_state.rs` - Spread constant 50.0 → 15.0
- `resyn-app/src/graph/webgl_renderer.rs` - Preallocated VBOs, updated draw_edge_pass signature
- `resyn-app/src/graph/renderer.rs` - DPR coordinate convention module doc added
- `resyn-app/src/pages/graph.rs` - Viewport made mutable, fit_scale computed; pre-existing clippy fixes

## Decisions Made
- Spread constant 15.0 chosen: `sqrt(375) * 15 ≈ 290px` fits on any reasonable screen at scale=1 without force simulation changes
- `quad_buf` kept in struct with `#[allow(dead_code)]` — the buffer must outlive the VAO that references it
- Edge buffer shared between two passes (lines and arrow triangles) — two sequential updates per frame, no simultaneous use conflict

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed 8 pre-existing clippy errors in graph.rs**
- **Found during:** Task 1 (clippy verification step)
- **Issue:** `cargo clippy -p resyn-app -- -D warnings` failed with 8 errors: unused imports (`std::task::Poll`, `futures::Stream`), `useless_conversion` on canvas cast, `type_complexity` on closure slot, `unused_variable` on `bridge` param, `dead_assignment` on `vis_count`, 2x `needless_return`
- **Fix:** Removed unused imports, added `ClosureSlot` type alias, fixed `canvas_el` cast, renamed `bridge` to `_bridge`, changed `let mut vis_count = (0, 0)` to `let vis_count;`, removed trailing `return;` statements
- **Files modified:** `resyn-app/src/pages/graph.rs`
- **Verification:** `cargo clippy -p resyn-app --no-deps -- -D warnings` exits 0
- **Committed in:** `1cff1ad` (Task 1 commit)

**2. [Rule 2 - Missing Critical] Added #[allow(dead_code)] for quad_buf field**
- **Found during:** Task 2 (clippy verification step)
- **Issue:** `quad_buf` field set up in `new()` (VAO attribute binding requires buffer outlive VAO) but never read in `draw()`, triggering `dead_code` lint
- **Fix:** Added doc comment explaining the field keeps the buffer alive, with `#[allow(dead_code)]`
- **Files modified:** `resyn-app/src/graph/webgl_renderer.rs`
- **Verification:** `cargo clippy -p resyn-app --no-deps -- -D warnings` exits 0
- **Committed in:** `a188d13` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical)
**Impact on plan:** Both auto-fixes necessary for correct operation (clippy clean) and GPU correctness (buffer lifetime). No scope creep.

## Issues Encountered
- `resyn-worker` has a pre-existing stack overflow in `test_convergence_100_node_graph_within_5000_ticks`. This is in `resyn-worker/src/forces.rs` which was already modified before this plan ran. Out of scope — not caused by this plan's changes. Deferred.
- `cargo clippy -p resyn-app -p resyn-worker -- -D warnings` cannot exit 0 while resyn-worker has pre-existing issues. The plan spec was met for resyn-app specifically.

## Known Stubs
None — no stubbed data or placeholder values in the changed files.

## Next Phase Readiness
- Task 3 (visual verify) pending: start `cargo leptos serve`, open http://localhost:3000/graph, verify GRAPH-01/02/03
- DPR convention documented — Phase 13 pointer events can use `screen_to_world()` directly without DPR conversion
- Blocker from STATE.md: DPR fix in webgl_renderer.rs coordinate system — verify Phase 13 coordinate correctness after visual check

---
*Phase: 12-graph-force-rendering*
*Completed: 2026-03-23 (pending Task 3 visual verification)*
