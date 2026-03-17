---
phase: 09-graph-renderer-canvas-to-webgl
plan: "04"
subsystem: ui
tags: [leptos, wasm, canvas2d, raf, web-sys, event-handlers, routing, css, graph-interaction]

requires:
  - phase: 09-01
    provides: Viewport, Renderer trait, GraphState, InteractionState, find_node_at, zoom_toward_cursor
  - phase: 09-02
    provides: WorkerBridge, ForceLayoutWorker, LayoutInput, LayoutOutput
  - phase: 09-03
    provides: Canvas2DRenderer implementing Renderer trait
  - phase: 08
    provides: SelectedPaper context, SidebarCollapsed context, Leptos app shell

provides:
  - GraphPage Leptos component at /graph route with full interactive canvas
  - GraphControls overlay component with edge toggles and simulation controls
  - RAF render loop polling ForceLayoutWorker via noop-waker sync poll each frame
  - Mouse event handlers: mousemove (hover/pan/drag), mousedown, mouseup, dblclick, wheel, pointerleave
  - Tooltip rendering via RwSignal<Option<TooltipData>> overlay div
  - Sidebar Graph nav entry and /graph route registration
  - Graph-specific CSS classes using existing design tokens

affects:
  - 09-05 (final integration and Trunk build configuration)

tech-stack:
  added:
    - futures = "0.3" (Stream trait, noop_waker_ref for sync RAF polling)
    - wasm-bindgen-futures = "0.4" (spawn_local if needed; added as explicit dep)
  patterns:
    - "RenderState and renderer stored in separate Rc<RefCell<...>> to avoid borrow conflicts when calling renderer.draw(&graph, &viewport)"
    - "Arc<AtomicBool> for RAF cancel flag — Rc<RefCell<bool>> is not Send+Sync but leptos::on_cleanup requires Send+Sync"
    - "noop_waker_ref + poll_next in RAF loop to synchronously drain worker bridge output each frame"
    - "Closures stored in EventListeners struct then mem::forget to keep event listeners alive for page lifetime"
    - "Effect fires on canvas mount + graph_resource resolution — canvas sized to offset_width/offset_height"

key-files:
  created:
    - resyn-app/src/pages/graph.rs
    - resyn-app/src/components/graph_controls.rs
  modified:
    - resyn-app/src/pages/mod.rs
    - resyn-app/src/components/mod.rs
    - resyn-app/src/layout/sidebar.rs
    - resyn-app/src/app.rs
    - resyn-app/style/main.css
    - resyn-app/Cargo.toml

key-decisions:
  - "RenderState and Box<dyn Renderer> split into two Rc<RefCell<...>> — Rust borrow checker cannot allow mutable borrow of renderer alongside immutable borrows of graph and viewport from the same struct"
  - "Arc<AtomicBool> for RAF cancelled flag — leptos::on_cleanup requires Send+Sync, Rc<RefCell<bool>> cannot satisfy this"
  - "Sync poll of ReactorBridge Stream using noop_waker_ref() each RAF frame — avoids spawn_local async complexity and keeps layout output processing deterministic with rendering"
  - "Event listener closures mem::forgot in EventListeners struct — single-user local tool, page-lifetime leak is acceptable"
  - "futures and wasm-bindgen-futures added as explicit Cargo.toml deps — were transitive deps only"

patterns-established:
  - "RAF loop pattern: Arc<AtomicBool> cancel + Rc<RefCell<Option<Closure>>> self-reference slot"
  - "Worker bridge sync polling: poll_next with noop_waker_ref drains available outputs without async machinery"
  - "Leptos overlay signals: RwSignal<bool> toggled by buttons, read with get_untracked() in RAF closure"

requirements-completed:
  - GRAPH-01
  - GRAPH-02

duration: 16min
completed: "2026-03-17"
---

# Phase 9 Plan 04: Graph Page Integration Summary

**Full-page interactive citation graph at /graph — Canvas 2D renderer wired to ForceLayout worker via RAF loop with pan, zoom, node click/drag/pin, edge and node tooltips, sidebar navigation, and overlay controls.**

## Performance

- **Duration:** 16 min
- **Started:** 2026-03-17T18:37:00Z
- **Completed:** 2026-03-17T18:53:22Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Created `GraphPage` component integrating Canvas2DRenderer, WorkerBridge, and all 6 mouse/pointer/wheel event handlers into one cohesive interactive page
- RAF render loop polls ForceLayoutWorker outputs synchronously each frame via noop_waker_ref, sends layout ticks when simulation is running
- Full interaction: pan via click-drag on background, zoom toward cursor via scroll, drag to pin nodes, click pinned node to unpin, click unpinned node to open drawer via SelectedPaper context, dblclick to reset viewport
- Tooltips display per UI-SPEC copywriting contract: node (title truncated at 60 chars, author, year, citation count), regular edge (A cites B), contradiction/bridge (type, confidence%, shared terms)
- GraphControls component with aria-pressed Contradiction/ABC-Bridge toggles and play/pause simulation button
- Graph nav entry after Methods in sidebar, /graph route registered in app router
- CSS: all graph-specific classes using existing design tokens, `.content-area:has(.graph-page)` padding override

## Task Commits

1. **Task 1: GraphPage component with render loop, event handlers, and worker integration** - `0e8f2ef` (feat)
2. **Task 2: GraphControls component, sidebar entry, route, and CSS** - `85f6a16` (feat)

**Plan metadata:** [pending docs commit]

## Files Created/Modified

- `resyn-app/src/pages/graph.rs` - GraphPage component: canvas NodeRef, RAF loop, 6 event handlers, tooltip overlay, loading/empty/error states
- `resyn-app/src/components/graph_controls.rs` - GraphControls: Contradiction/ABC-Bridge toggles, play/pause, zoom +/- buttons with aria attributes
- `resyn-app/src/pages/mod.rs` - Added `pub mod graph`
- `resyn-app/src/components/mod.rs` - Added `pub mod graph_controls`
- `resyn-app/src/layout/sidebar.rs` - Added Graph nav item (◉) after Methods
- `resyn-app/src/app.rs` - Added GraphPage import and /graph route
- `resyn-app/style/main.css` - Added graph-page, graph-canvas, graph-controls-overlay, graph-controls-group, graph-control-btn, graph-tooltip, graph-loading CSS classes
- `resyn-app/Cargo.toml` - Added futures = "0.3" and wasm-bindgen-futures = "0.4" as explicit deps

## Decisions Made

- **Renderer/state split:** `Box<dyn Renderer>` stored in separate `Rc<RefCell<...>>` from `RenderState` — Rust's borrow checker prevents mutably borrowing `renderer` while simultaneously borrowing `graph` and `viewport` from the same struct. Split avoids unsafe.
- **Arc<AtomicBool> for cancel:** `leptos::on_cleanup` requires `Send + Sync` on its closure. `Rc<RefCell<bool>>` is neither. `Arc<AtomicBool>` satisfies both while remaining free from actual cross-thread use.
- **Sync worker poll:** `poll_next` with `futures::task::noop_waker_ref()` drains the ReactorBridge Stream synchronously in each RAF frame. Simpler than `spawn_local` async loop; positions update atomically with rendering; avoids RefCell borrow-across-await problems.
- **futures + wasm-bindgen-futures explicit deps:** Both were transitive dependencies only (via gloo-worker and leptos). Made explicit in Cargo.toml to signal intentional use.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Renderer stored separately from RenderState**
- **Found during:** Task 1 (compile)
- **Issue:** Plan specified `renderer: Box<dyn Renderer>` inside `RenderState` struct. Calling `s.renderer.draw(&s.graph, &s.viewport)` fails borrow check — mutable borrow of `renderer` conflicts with immutable borrows of `graph` and `viewport` on the same struct.
- **Fix:** Split renderer into a separate `Rc<RefCell<Box<dyn Renderer>>>` (`renderer_rc`), keeping `RenderState` as graph state + viewport + interaction state only.
- **Files modified:** resyn-app/src/pages/graph.rs
- **Verification:** `cargo check -p resyn-app` passes cleanly
- **Committed in:** 0e8f2ef (Task 1 commit)

**2. [Rule 3 - Blocking] Arc<AtomicBool> for RAF cancel flag**
- **Found during:** Task 1 (compile)
- **Issue:** `on_cleanup(move || handle.cancel())` requires `Send + Sync`. `RafHandle { cancelled: Rc<RefCell<bool>> }` is neither.
- **Fix:** Changed to `Arc<AtomicBool>` with `store(true, Ordering::Relaxed)` / `load(Ordering::Relaxed)`.
- **Files modified:** resyn-app/src/pages/graph.rs
- **Verification:** `cargo check -p resyn-app` passes cleanly
- **Committed in:** 0e8f2ef (Task 1 commit)

**3. [Rule 3 - Blocking] Sync poll instead of async spawn_local for worker bridge**
- **Found during:** Task 1 (compile/design)
- **Issue:** Plan specified `WorkerBridge::new(callback: impl Fn(LayoutOutput))` but the actual WorkerBridge takes no callback — it exposes `ReactorBridge` as a Stream. Polling via `spawn_local` async loop required owning the bridge, conflicting with RAF loop needing to send inputs. `ReactorBridge` is not `Unpin` so `Pin::new(&mut *borrowed.bridge)` failed.
- **Fix:** Implemented `poll_bridge_sync()` using `futures::task::noop_waker_ref()` to synchronously drain available outputs in the RAF loop. `Box::pin(bridge.bridge)` stored in `Rc<RefCell<Pin<Box<ReactorBridge<...>>>>>`.
- **Files modified:** resyn-app/src/pages/graph.rs
- **Verification:** `cargo check -p resyn-app` passes cleanly
- **Committed in:** 0e8f2ef (Task 1 commit)

**4. [Rule 2 - Missing Critical] Added futures and wasm-bindgen-futures as explicit deps**
- **Found during:** Task 1 (compile)
- **Issue:** `use futures::Stream` and `futures::task::noop_waker_ref()` required but futures not in Cargo.toml (transitive only).
- **Fix:** Added `futures = "0.3"` and `wasm-bindgen-futures = "0.4"` to resyn-app/Cargo.toml.
- **Files modified:** resyn-app/Cargo.toml, Cargo.lock
- **Verification:** `cargo check -p resyn-app` passes cleanly
- **Committed in:** 0e8f2ef (Task 1 commit)

---

**Total deviations:** 4 auto-fixed (3 blocking compile issues, 1 missing dep)
**Impact on plan:** All fixes essential for correct compilation. The worker bridge approach is semantically equivalent to the callback pattern — same data flow, different WASM-compatible mechanism.

## Issues Encountered

- Pre-existing SSR build failure in `resyn-app/src/server_fns/graph.rs`: `edge_references()` method not found on `StableGraph` with current petgraph version in SSR build context. Pre-dated this plan (confirmed via git stash test). Deferred to separate fix.
- 24 existing tests (CSR feature build) pass without regressions.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- GraphPage compiles and is fully wired: server fn → GraphState → Canvas2DRenderer → ForceLayoutWorker RAF loop → event handlers → SelectedPaper drawer integration
- Plan 09-05 (Trunk build + integration testing) can proceed — all components are in place
- The pre-existing SSR build failure (`edge_references()`) should be resolved before production SSR builds

---
*Phase: 09-graph-renderer-canvas-to-webgl*
*Completed: 2026-03-17*
