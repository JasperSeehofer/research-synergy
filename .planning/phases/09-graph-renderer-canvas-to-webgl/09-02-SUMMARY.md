---
phase: 09-graph-renderer-canvas-to-webgl
plan: 02
subsystem: resyn-worker / resyn-app
tags: [barnes-hut, force-layout, web-worker, wasm, gloo-worker, trunk]
dependency_graph:
  requires: ["09-01"]
  provides: ["09-03", "09-04"]
  affects: ["resyn-worker", "resyn-app"]
tech_stack:
  added: ["barnes-hut quadtree", "gloo-worker reactor", "futures SinkExt/StreamExt"]
  patterns: ["O(n log n) repulsion via QuadTree", "gloo-worker reactor pattern", "velocity Verlet integration"]
key_files:
  created:
    - resyn-worker/src/barnes_hut.rs
    - resyn-worker/src/forces.rs
  modified:
    - resyn-worker/src/lib.rs
    - resyn-app/index.html
    - resyn-app/Cargo.toml
    - resyn-app/src/graph/worker_bridge.rs
decisions:
  - "simulation_tick takes &mut [NodeData] + parallel &mut [(f64,f64)] velocity slice — avoids SimNode wrapper type leaking into public API"
  - "scope.send(output).await with SinkExt import — gloo-worker ReactorScope implements Sink, not a plain method"
  - "WorkerBridge exposes send_input (not send) matching ReactorBridge API; Spawnable trait must be in scope for spawner()"
  - "ReactorBridge<ForceLayoutWorker> stored as pub field in WorkerBridge — callers can poll it as Stream for responses"
metrics:
  duration_seconds: 496
  completed_date: "2026-03-17"
  tasks_completed: 2
  files_created: 2
  files_modified: 4
---

# Phase 9 Plan 2: Barnes-Hut Force Layout Worker Summary

**One-liner:** Barnes-Hut O(n log n) quadtree with alpha-decay force simulation in gloo-worker reactor, wired to Trunk via index.html worker link tag.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Barnes-Hut quadtree and force simulation | 03aaeb6 | resyn-worker/src/barnes_hut.rs, resyn-worker/src/forces.rs, resyn-worker/src/lib.rs |
| 2 | Wire gloo-worker reactor and Trunk build integration | 783cc68 | resyn-worker/src/lib.rs, resyn-app/index.html, resyn-app/src/graph/worker_bridge.rs, resyn-app/Cargo.toml |

## What Was Built

**Barnes-Hut QuadTree (`barnes_hut.rs`):**
- `QuadTree::build(positions, masses)` — computes bounding rect, inserts all nodes with O(n log n) subdivision
- `QuadTree::insert` — stores leaf nodes with x/y/mass to enable correct CoM-weighted subdivision
- `barnes_hut_repulsion(tree, x, y, mass, theta)` — applies opening-angle criterion; skips self-interaction at leaf nodes
- Repulsion constant: -30.0; distance clamped to 1.0 minimum to prevent division by zero

**Force Simulation (`forces.rs`):**
- `simulation_tick(nodes, vel, edges, alpha)` — one integration step: BH repulsion + Hooke's edge attraction + center gravity
- `run_ticks(input) -> LayoutOutput` — runs up to `input.ticks` iterations, short-circuits on convergence
- Alpha decay: 0.92 per tick; convergence threshold: 0.001; velocity damping: 0.6
- 14 unit tests covering all force components, pinned nodes, convergence, and inverse-square law

**gloo-worker Reactor (`lib.rs`):**
- `#[reactor] ForceLayoutWorker` — async loop receiving `LayoutInput`, calling `run_ticks`, sending `LayoutOutput`
- Uses `futures::{SinkExt, StreamExt}` — ReactorScope implements both Sink (for send) and Stream (for receive)

**WorkerBridge (`resyn-app/src/graph/worker_bridge.rs`):**
- Wraps `ReactorBridge<ForceLayoutWorker>` — typed bridge using `Spawnable::spawner().spawn("./resyn_worker.js")`
- `send()` delegates to `bridge.send_input()` — callers poll `bridge` as Stream to receive `LayoutOutput`

**Trunk Integration (`resyn-app/index.html`):**
- Added `<link data-trunk rel="rust" href="../resyn-worker/Cargo.toml" data-type="worker" data-bin="resyn_worker"/>`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] QuadTree insert used back-calculated CoM for existing node position**
- **Found during:** Task 1 RED test run — `test_barnes_hut_repulsion_nonzero_for_separated_nodes` panicked with NaN
- **Issue:** Original `QuadTree` stored `node_idx: Option<usize>` without position, requiring error-prone back-calculation from CoM to relocate existing node during subdivision
- **Fix:** Replaced `node_idx: Option<usize>` with `leaf: Option<LeafNode>` storing `{idx, x, y, mass}` directly — no position back-calculation needed
- **Files modified:** resyn-worker/src/barnes_hut.rs
- **Commit:** 03aaeb6 (folded into Task 1 implementation commit)

**2. [Rule 1 - Bug] gloo-worker scope.send() requires SinkExt + .await**
- **Found during:** Task 2 WASM check — `scope.send(output)` compiled without SinkExt but produced unused-future warning
- **Issue:** ReactorScope implements Sink, so `send()` returns a Future that must be `.await`ed; also needs `use futures::SinkExt`
- **Fix:** Added `use futures::SinkExt` import and `.await` to `scope.send(output).await`
- **Files modified:** resyn-worker/src/lib.rs
- **Commit:** 783cc68

**3. [Rule 1 - Bug] WorkerBridge required Spawnable trait import and send_input (not send)**
- **Found during:** Task 2 cargo check — `ForceLayoutWorker::spawner()` not found, `bridge.send()` not found
- **Issue:** `spawner()` comes from `gloo_worker::Spawnable` trait (must be imported); ReactorBridge method is `send_input()` not `send()`
- **Fix:** Added `use gloo_worker::Spawnable`, changed `bridge.send()` to `bridge.send_input()`; also added `gloo-worker` dep to resyn-app/Cargo.toml
- **Files modified:** resyn-app/src/graph/worker_bridge.rs, resyn-app/Cargo.toml
- **Commit:** 783cc68

**4. [Rule 2 - Cleanup] Removed unused SimState struct**
- **Found during:** Task 1 — leftover struct from initial design
- **Fix:** Deleted the `SimState` struct; velocity tracking uses parallel `vel: &mut [(f64, f64)]` slice instead
- **Files modified:** resyn-worker/src/forces.rs
- **Commit:** 03aaeb6 (folded into Task 1 implementation)

## Verification Results

```
cargo test -p resyn-worker -- --nocapture
  14 passed; 0 failed (barnes_hut: 6 tests, forces: 8 tests)

cargo check -p resyn-worker --target wasm32-unknown-unknown
  Finished dev profile [unoptimized + debuginfo]

cargo check -p resyn-app
  Finished dev profile [unoptimized + debuginfo]
```

## Must-Have Truths Verified

- [x] Barnes-Hut single step produces non-zero repulsive forces for two separated nodes (`test_barnes_hut_repulsion_nonzero_for_separated_nodes`)
- [x] Force layout converges (alpha < 0.001) within 500 ticks for a 100-node graph (`test_convergence_100_node_graph_within_500_ticks`)
- [x] Worker crate compiles as cdylib for wasm32-unknown-unknown (`cargo check -p resyn-worker --target wasm32-unknown-unknown`)
- [x] Trunk index.html includes the worker link tag with data-type=worker

## Self-Check: PASSED

Files verified:
- resyn-worker/src/barnes_hut.rs: contains `pub struct QuadTree` — FOUND
- resyn-worker/src/forces.rs: contains `pub fn simulation_tick`, `pub fn run_ticks`, `pub const THETA`, `pub const ALPHA_MIN` — FOUND
- resyn-worker/src/lib.rs: contains `#[reactor]`, `pub async fn ForceLayoutWorker`, `forces::run_ticks` — FOUND
- resyn-app/index.html: contains `data-type="worker"`, `resyn-worker/Cargo.toml` — FOUND
- resyn-app/src/graph/worker_bridge.rs: contains `pub struct WorkerBridge`, `ForceLayoutWorker::spawner` — FOUND

Commits verified:
- 03aaeb6: feat(09-02): implement Barnes-Hut quadtree and force simulation — FOUND
- 783cc68: feat(09-02): wire gloo-worker reactor and Trunk build integration — FOUND
