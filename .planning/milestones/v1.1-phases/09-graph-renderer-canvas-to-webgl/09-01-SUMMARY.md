---
phase: 09-graph-renderer-canvas-to-webgl
plan: 01
subsystem: ui
tags: [rust, wasm, leptos, web-sys, petgraph, canvas, webgl, serde, leptos-server-fn]

# Dependency graph
requires:
  - phase: 08-leptos-web-shell-analysis-panels
    provides: server fn registration pattern and Arc<Db> context injection
  - phase: 06-workspace-restructure
    provides: resyn-core workspace split with WASM-safe types
provides:
  - resyn-worker crate (cdylib+rlib) with LayoutInput/LayoutOutput types for Barnes-Hut WASM worker
  - GraphData/GraphNode/GraphEdge/EdgeType serde DTO types
  - GetGraphData server function reading from SurrealDB via PaperRepository and GapFindingRepository
  - Renderer trait (draw/resize) and Viewport transform helpers (screen_to_world, world_to_screen)
  - GraphState::from_graph_data converting GraphData to positioned NodeState array with radius formula
  - find_node_at, find_edge_at, zoom_toward_cursor interaction functions
  - InteractionState enum for pan/drag/idle state machine
affects:
  - 09-02 (Barnes-Hut WASM worker implements LayoutInput/LayoutOutput)
  - 09-03 (Canvas2DRenderer implements Renderer trait)
  - 09-04 (WebGL2Renderer implements Renderer trait)
  - 09-05 (Leptos component wires everything together)

# Tech tracking
tech-stack:
  added:
    - resyn-worker crate (new workspace member, cdylib+rlib for WASM)
    - web-sys 0.3 with Canvas2D, WebGL2, pointer/mouse/wheel events
    - js-sys 0.3
    - gloo-worker 0.5 (futures feature) for Web Worker bridge
  patterns:
    - TDD: test-first for serde round-trips, math functions, and hit-test logic
    - GraphData DTO pattern: server types separate from layout state types
    - EdgeType overlay: citation edges from petgraph + gap finding edges from GapFindingRepository merged into one GraphData response
    - radius_from_citations formula: clamp(sqrt(count+1)*2.5, 4.0, 18.0)
    - InteractionState enum for type-safe mouse state machine
    - Viewport as pure-math struct (no web-sys dependency) enabling unit testing

key-files:
  created:
    - resyn-worker/Cargo.toml
    - resyn-worker/src/lib.rs
    - resyn-app/src/graph/mod.rs
    - resyn-app/src/graph/renderer.rs
    - resyn-app/src/graph/layout_state.rs
    - resyn-app/src/graph/interaction.rs
    - resyn-app/src/graph/worker_bridge.rs
    - resyn-app/src/server_fns/graph.rs
  modified:
    - Cargo.toml (added resyn-worker to workspace members)
    - resyn-app/Cargo.toml (added web-sys, js-sys, resyn-worker deps)
    - resyn-app/src/lib.rs (added pub mod graph)
    - resyn-app/src/server_fns/mod.rs (added pub mod graph)
    - resyn-server/src/commands/serve.rs (registered GetGraphData)

key-decisions:
  - "Viewport struct is pure math (no web-sys dependency) so all transform tests can run natively without wasm-bindgen-test"
  - "Renderer::apply() takes &web_sys::CanvasRenderingContext2d — only Canvas2DRenderer uses it; WebGLRenderer ignores it (Plan 04)"
  - "worker_bridge.rs is a stub with #[allow(unused_imports)] — Plan 02 fills in the gloo-worker reactor"
  - "GraphState::from_graph_data uses spiral/ring initial placement: r = sqrt(i/n) * spread to avoid all nodes at origin"

patterns-established:
  - "Viewport as pure-math struct: offset_x/y and scale with screen<->world helpers, no web-sys in struct — allows native unit tests"
  - "GraphData DTO separate from GraphState: server returns serializable DTO; client converts to mutable simulation state"
  - "EdgeType merges Regular (petgraph citations) + Contradiction/AbcBridge (GapFindingRepository) in one server fn"

requirements-completed: [GRAPH-01, GRAPH-02, GRAPH-04]

# Metrics
duration: 35min
completed: 2026-03-17
---

# Phase 9 Plan 01: Graph Renderer Scaffold Summary

**GraphData server fn, Renderer trait, Viewport transforms, and resyn-worker cdylib scaffold — 24 tests pass, WASM target compiles**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-03-17T00:00:00Z
- **Completed:** 2026-03-17T00:35:00Z
- **Tasks:** 2 (Tasks 1 and 2)
- **Files modified:** 14

## Accomplishments

- Created resyn-worker crate with LayoutInput/LayoutOutput types, compiles for wasm32-unknown-unknown
- Established GraphData/GraphNode/GraphEdge/EdgeType serde DTO types with 4 serialization tests
- Built GraphState::from_graph_data with spiral initial placement and radius_from_citations formula
- Defined Renderer trait and Viewport with screen_to_world/world_to_screen round-trip tested helpers
- Implemented find_node_at (reverse-order hit-test), find_edge_at (point-to-segment), zoom_toward_cursor (cursor-anchored)
- Registered GetGraphData server fn in resyn-server; merges citation edges with gap findings

## Task Commits

Each task was committed atomically:

1. **Task 1 + 2: Graph module scaffold, types, worker crate, Renderer trait, interaction module** - `160fea2` (feat)

_Note: Tasks 1 and 2 were committed together because interaction.rs (Task 2) is declared in graph/mod.rs (Task 1), making them compile-interdependent._

**Plan metadata:** (docs commit — see below)

## Files Created/Modified

- `resyn-worker/Cargo.toml` - New crate config with cdylib+rlib and gloo-worker dep
- `resyn-worker/src/lib.rs` - LayoutInput/LayoutOutput/NodeData types
- `resyn-app/src/graph/mod.rs` - Module declarations for renderer, layout_state, interaction, worker_bridge
- `resyn-app/src/graph/renderer.rs` - Viewport struct, Renderer trait, WEBGL_THRESHOLD=300 constant
- `resyn-app/src/graph/layout_state.rs` - NodeState, EdgeData, GraphState with from_graph_data and radius formula
- `resyn-app/src/graph/interaction.rs` - find_node_at, find_edge_at, zoom_toward_cursor, InteractionState enum
- `resyn-app/src/graph/worker_bridge.rs` - Stub placeholder for Plan 02 gloo-worker bridge
- `resyn-app/src/server_fns/graph.rs` - GraphData DTO types, GetGraphData server fn with serde tests
- `Cargo.toml` - Added resyn-worker to workspace members
- `resyn-app/Cargo.toml` - Added web-sys (19 features), js-sys, resyn-worker deps
- `resyn-app/src/lib.rs` - Added pub mod graph
- `resyn-app/src/server_fns/mod.rs` - Added pub mod graph
- `resyn-server/src/commands/serve.rs` - Registered GetGraphData server fn

## Decisions Made

- Viewport is a plain struct with no web-sys dependency — all transform tests run natively without wasm-bindgen-test
- Renderer::apply() takes &web_sys::CanvasRenderingContext2d — Canvas2DRenderer uses it; WebGLRenderer ignores it
- worker_bridge.rs is a compile-stub — Plan 02 fills in the gloo-worker reactor implementation
- GraphState::from_graph_data uses sqrt-based radial placement (not random) to avoid degenerate initial layouts

## Deviations from Plan

None — plan executed exactly as written with one minor addition:

**[Rule 2 - Warning cleanup] Removed unused imports before CI would flag them**
- `GraphEdge` import in layout_state.rs was unused (edges are built via iterator without explicit type annotation)
- `LayoutOutput` import in worker_bridge.rs was unused (stub only uses LayoutInput in the comment import)
- Fixed with targeted `use` statement cleanup and `#[allow(unused_imports)]` on the intentional stub import

## Issues Encountered

None — all compilation succeeded on first attempt. The Viewport struct being pure-math (no web-sys) was the key design choice that enabled native test execution.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Plan 02 (Barnes-Hut WASM worker): LayoutInput/LayoutOutput types ready; worker_bridge.rs stub ready to replace
- Plan 03 (Canvas2DRenderer): Renderer trait defined; GraphState and Viewport contracts established
- Plan 04 (WebGL2Renderer): Renderer trait ready; WEBGL_THRESHOLD=300 constant defined
- Plan 05 (Leptos GraphView component): GraphData server fn registered; GetGraphData callable from Leptos Suspense

---
*Phase: 09-graph-renderer-canvas-to-webgl*
*Completed: 2026-03-17*
