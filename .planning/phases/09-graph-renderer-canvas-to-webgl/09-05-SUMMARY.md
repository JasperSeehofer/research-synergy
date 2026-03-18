---
phase: 09-graph-renderer-canvas-to-webgl
plan: 05
subsystem: ui
tags: [webgl2, glsl, instanced-rendering, canvas2d, force-layout, web-sys, wasm, rust]

# Dependency graph
requires:
  - phase: 09-graph-renderer-canvas-to-webgl/09-04
    provides: GraphPage with RAF loop, event handlers, Canvas2DRenderer, and worker bridge
provides:
  - WebGL2Renderer implementing Renderer trait with instanced circle nodes and line edges
  - make_renderer factory that auto-selects WebGL2 for >300 nodes with temporary-canvas probe
  - Browser-verified full Phase 9 graph feature set
  - Bug-fixed resize, simulation jitter, zoom, button wiring, click detection, worker build
affects: [future phase 10, any rendering or graph visualization work]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "WebGL2 instanced drawing for nodes: TRIANGLE_FAN quads with per-instance position/radius/color/alpha via vertex_attrib_divisor"
    - "Temporary canvas probe for WebGL2 availability to avoid 2D context contamination on main canvas"
    - "Canvas 2D overlay for text labels on top of WebGL2 canvas (stack via CSS position: absolute)"
    - "ResizeObserver + DPR-aware canvas sizing for crisp rendering on high-DPI displays"
    - "Persist simulation alpha/velocities across RAF frames (not restarted each frame)"

key-files:
  created:
    - resyn-app/src/graph/webgl_renderer.rs
    - resyn-worker/src/bin/resyn_worker.rs
  modified:
    - resyn-app/src/graph/mod.rs
    - resyn-app/src/graph/renderer.rs
    - resyn-app/src/graph/canvas_renderer.rs
    - resyn-app/src/graph/interaction.rs
    - resyn-app/src/graph/layout_state.rs
    - resyn-app/src/pages/graph.rs
    - resyn-app/src/components/graph_controls.rs
    - resyn-worker/Cargo.toml
    - resyn-worker/src/forces.rs
    - resyn-worker/src/lib.rs

key-decisions:
  - "WebGL2 probe on a temporary canvas (document.createElement) prevents 2D context contamination on the main canvas — acquiring 2D context first makes WebGL2 return null"
  - "Instanced drawing (draw_arrays_instanced + vertex_attrib_divisor) for nodes — one draw call for all nodes regardless of count"
  - "Labels rendered on a Canvas 2D overlay canvas stacked over WebGL2 canvas via CSS absolute positioning — WebGL does not support native text"
  - "DPR-aware ResizeObserver sizing — canvas logical/physical size must track window.devicePixelRatio to prevent blurry or stretched rendering"
  - "Simulation jitter fixed by persisting alpha/velocities in layout_state across frames rather than reinitializing each RAF tick"
  - "Worker crate needs bin entry point (src/bin/resyn_worker.rs) and no cdylib — Trunk spawns it as a WASM module via worker spawn"

patterns-established:
  - "Renderer selection: make_renderer(canvas, node_count) probes WebGL2 on temp canvas, selects WebGL2Renderer if node_count > WEBGL_THRESHOLD, logs warning and falls back to Canvas2DRenderer otherwise"
  - "Edge rendering in WebGL: 2 vertices per edge in LINES draw mode; arrowheads as 3-vertex triangles in same pass"
  - "DPR canvas setup: set canvas width/height to CSS size * devicePixelRatio, then set CSS size; call gl.viewport(0,0,w,h)"

requirements-completed: [GRAPH-03, GRAPH-01, GRAPH-02, GRAPH-04]

# Metrics
duration: ~90min
completed: 2026-03-18
---

# Phase 9 Plan 05: WebGL2 Renderer and Browser Verification Summary

**WebGL2Renderer with GLSL instanced-circle nodes and line edges, automatic Canvas-to-WebGL switching via make_renderer factory, browser-verified across 17 interaction checks with 7 post-verification bug fixes committed**

## Performance

- **Duration:** ~90 min
- **Started:** 2026-03-18T (~previous session)
- **Completed:** 2026-03-18T00:15:39Z
- **Tasks:** 2 (1 auto + 1 checkpoint:human-verify)
- **Files modified:** 10

## Accomplishments

- WebGL2Renderer implementing the Renderer trait with #version 300 es GLSL shaders — instanced circle nodes (TRIANGLE_FAN, vertex_attrib_divisor), line edges, triangle arrowheads
- make_renderer factory with temporary-canvas WebGL2 probe; auto-selects WebGL2 for node_count > WEBGL_THRESHOLD (300); falls back to Canvas2DRenderer with console warning
- Full browser verification of Phase 9 across all 17 checks: graph renders, force layout converges, pan/zoom/drag, node click opens drawer, edge toggles, play/pause, +/- buttons, labels, arrowheads, empty state
- 7 post-verification bugs auto-fixed: canvas resize stretching, simulation jitter, zoom sensitivity, button wiring, white circle artifact, click-vs-drag detection, worker crate build

## Task Commits

1. **Task 1: WebGL2Renderer with instanced circles and make_renderer factory** - `a5e576e` (feat)
2. **Bug fixes from browser verification** - `eceb6d1` (fix)
3. **Task 2: Browser verification** - checkpoint approved by user (no code commit)

## Files Created/Modified

- `resyn-app/src/graph/webgl_renderer.rs` - WebGL2Renderer struct, GLSL shaders (node + edge), impl Renderer, node_screen_positions for label overlay
- `resyn-app/src/graph/mod.rs` - Added pub mod webgl_renderer; pub fn make_renderer
- `resyn-app/src/graph/renderer.rs` - make_renderer with WEBGL_THRESHOLD, temporary canvas probe, fallback warning
- `resyn-app/src/pages/graph.rs` - Use make_renderer instead of Canvas2DRenderer::new; ResizeObserver + DPR-aware sizing; fix click vs drag detection
- `resyn-app/src/graph/canvas_renderer.rs` - DPR-aware draw pass
- `resyn-app/src/graph/interaction.rs` - Click vs drag threshold fix
- `resyn-app/src/graph/layout_state.rs` - Persist alpha/velocities across frames
- `resyn-app/src/components/graph_controls.rs` - Wire +/- button click handlers via signals
- `resyn-worker/Cargo.toml` - Add bin entry, remove cdylib
- `resyn-worker/src/bin/resyn_worker.rs` - Worker bin entry point for Trunk
- `resyn-worker/src/forces.rs` - Simulation fix for jitter
- `resyn-worker/src/lib.rs` - Worker lib exports

## Decisions Made

- WebGL2 probe uses a temporary canvas created via document.createElement("canvas") — acquiring a 2D context on the main canvas before WebGL2 causes WebGL2 to return null; this is the only safe probe approach
- Instanced drawing chosen over separate draw calls per node — one draw_arrays_instanced call for all nodes scales to 1000+ nodes
- Text labels rendered via Canvas 2D overlay canvas stacked via CSS absolute positioning — no native text in WebGL; this is the standard pattern
- ResizeObserver used instead of window resize event — fires on element size change (correct for layout changes), not just window resize

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Canvas resize stretched graph on window resize**
- **Found during:** Task 2 (browser verification)
- **Issue:** Canvas dimensions not updated when container resized; graph appeared stretched or clipped
- **Fix:** Added ResizeObserver in graph.rs to detect canvas container size changes; set canvas width/height = CSS size * devicePixelRatio; call renderer.resize()
- **Files modified:** resyn-app/src/pages/graph.rs
- **Committed in:** eceb6d1

**2. [Rule 1 - Bug] Simulation jitter — nodes bounced continuously**
- **Found during:** Task 2 (browser verification)
- **Issue:** layout_state alpha/velocities reset each RAF frame instead of persisting; simulation never converged
- **Fix:** Moved simulation state outside the RAF callback closure so it persists across frames
- **Files modified:** resyn-app/src/graph/layout_state.rs, resyn-worker/src/forces.rs
- **Committed in:** eceb6d1

**3. [Rule 1 - Bug] Zoom sensitivity inconsistent across trackpad vs mouse wheel**
- **Found during:** Task 2 (browser verification)
- **Issue:** Raw wheel deltaY values differ by 100x between trackpad and mouse; made zoom unusable on trackpad
- **Fix:** Normalized wheel delta by clamping and scaling before applying to viewport zoom
- **Files modified:** resyn-app/src/pages/graph.rs
- **Committed in:** eceb6d1

**4. [Rule 1 - Bug] +/- zoom buttons had no effect**
- **Found during:** Task 2 (browser verification)
- **Issue:** GraphControls +/- buttons emitted signals but click handlers in GraphPage were not wired
- **Fix:** Connected button signals to viewport zoom update in graph.rs event handler
- **Files modified:** resyn-app/src/components/graph_controls.rs, resyn-app/src/pages/graph.rs
- **Committed in:** eceb6d1

**5. [Rule 1 - Bug] White circle artifact appeared at graph origin**
- **Found during:** Task 2 (browser verification)
- **Issue:** Pinned node indicator rendered an unintended white circle at (0,0) on init
- **Fix:** Removed pinned indicator rendering from initial draw pass
- **Files modified:** resyn-app/src/graph/canvas_renderer.rs
- **Committed in:** eceb6d1

**6. [Rule 1 - Bug] Paper drawer did not open on node click**
- **Found during:** Task 2 (browser verification)
- **Issue:** Click and drag both triggered the same handler; any mouse movement during click registered as drag and suppressed the click
- **Fix:** Added distance threshold to distinguish click from drag in interaction.rs
- **Files modified:** resyn-app/src/graph/interaction.rs
- **Committed in:** eceb6d1

**7. [Rule 3 - Blocking] Worker crate failed to build as WASM**
- **Found during:** Task 2 (browser verification)
- **Issue:** resyn-worker had cdylib crate-type but Trunk spawns it as a worker via a bin target; no bin entry point existed
- **Fix:** Added src/bin/resyn_worker.rs entry point; removed cdylib from Cargo.toml crate-type
- **Files modified:** resyn-worker/Cargo.toml, resyn-worker/src/bin/resyn_worker.rs
- **Committed in:** eceb6d1

---

**Total deviations:** 7 auto-fixed (6 Rule 1 bugs, 1 Rule 3 blocking)
**Impact on plan:** All fixes required for the feature to be functional in the browser. No scope creep — all were direct consequences of the implementation.

## Issues Encountered

None beyond the auto-fixed bugs above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 9 (Graph Renderer Canvas to WebGL) is fully complete. All 5 plans done, all 17 browser verification criteria passed.

Phase 10 (if any) has a complete, working interactive graph visualization:
- Canvas 2D renderer for small graphs, WebGL2 for large graphs (auto-selected)
- Barnes-Hut force layout in WASM Web Worker
- Full interaction: pan, zoom, hover tooltips, node click (paper drawer), node drag/pin
- Edge type overlays: contradiction (red), bridge (orange)
- Play/pause simulation controls

---
*Phase: 09-graph-renderer-canvas-to-webgl*
*Completed: 2026-03-18*
