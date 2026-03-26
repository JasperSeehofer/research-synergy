---
phase: 17-viewport-fit-and-label-collision
plan: "01"
subsystem: resyn-app/graph
tags: [viewport, animation, UX, convergence, WASM]
dependency_graph:
  requires: []
  provides: [viewport_fit_module, auto_fit_on_convergence, fit_button, convergence_badge]
  affects: [resyn-app/src/pages/graph.rs, resyn-app/src/components/graph_controls.rs]
tech_stack:
  added: []
  patterns: [lerp-animation, counter-signal-for-events, user-interaction-latch]
key_files:
  created:
    - resyn-app/src/graph/viewport_fit.rs
  modified:
    - resyn-app/src/graph/mod.rs
    - resyn-app/src/pages/graph.rs
    - resyn-app/src/components/graph_controls.rs
    - resyn-app/style/main.css
decisions:
  - Collapsed nested if-let with && let syntax per clippy collapsible_if lint
  - user_has_interacted set in 4 places (zoom-in, zoom-out, panning mousemove, wheel) — more than minimum 3
metrics:
  duration: 8min
  completed: "2026-03-26T10:04:19Z"
  tasks: 2
  files: 5
---

# Phase 17 Plan 01: Viewport Auto-Fit and Convergence Badge Summary

Viewport fit math, RAF lerp animation with user interaction latch, Fit button, and three-state convergence status badge.

## What Was Built

**viewport_fit.rs** — New pure-logic module with `compute_fit_target()` (10% margin, scale clamped 0.1–4.0, filters invisible nodes), `FitAnimState` struct, and `lerp()` utility. Verified with 7 unit tests.

**graph.rs** — `RenderState` gains `fit_anim`, `user_has_interacted`, `fit_has_fired_once`, `label_cache_dirty` fields. RAF loop: fit button detection (counter diff pattern), convergence auto-fit (fires once when user has not interacted), lerp animation step (t=0.12 exponential decay). `user_has_interacted` set on pan, wheel, and both zoom buttons. `simulation_settled` signal set on convergence.

**graph_controls.rs** — Two new props (`fit_count`, `simulation_settled`). Fit button (U+2922) added to simulation group. Three-state badge (`sim-status-badge`) shows Simulating.../Paused/Settled based on `simulation_running` and `simulation_settled`.

**main.css** — `.sim-status-badge`, `.sim-running`, `.sim-paused`, `.sim-settled` CSS classes. Settled state uses `var(--color-success)`.

## Task Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create viewport_fit module | 26850c8 | viewport_fit.rs, mod.rs |
| 2 | Wire auto-fit, latch, fit button, convergence badge | 0d9081f | graph.rs, graph_controls.rs, main.css |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Collapsed nested if into if-let chain for clippy compliance**
- **Found during:** Task 2 — `cargo clippy -p resyn-app -- -D warnings`
- **Issue:** clippy::collapsible_if triggered on `if !cond { if let Some(x) = ... }` pattern
- **Fix:** Rewrote as `if !cond && let Some(x) = ...` using let-chains (edition 2024)
- **Files modified:** resyn-app/src/pages/graph.rs

None beyond the above.

## Verification

- `cargo test -p resyn-app` — 57 passed (50 existing + 7 new viewport_fit tests)
- `cargo clippy -p resyn-app -- -D warnings` — clean
- `cargo fmt --all -- --check` — clean
- `compute_fit_target` called from 2 sites: fit button (line 415) and convergence (line 445)
- `user_has_interacted = true` in 4 locations: zoom-in, zoom-out, panning, wheel — NOT in DraggingNode
- `simulation_settled` signal flows from graph.rs to GraphControls via prop

## Known Stubs

None — all signals are wired to live data.

## Self-Check: PASSED

- resyn-app/src/graph/viewport_fit.rs: EXISTS
- resyn-app/src/graph/mod.rs: contains `pub mod viewport_fit;`
- resyn-app/src/pages/graph.rs: contains `fit_anim`, `user_has_interacted`, `fit_has_fired_once`, `simulation_settled.set(true)`, `let t = 0.12`
- resyn-app/src/components/graph_controls.rs: contains `fit_count`, `simulation_settled`, `Fit graph to viewport`, `\u{2922}`, `Simulating...`, `Settled`, `Paused`, `sim-status-badge`
- resyn-app/style/main.css: contains `.sim-status-badge`, `.sim-status-badge.sim-settled`, `var(--color-success)`
- Commits 26850c8 and 0d9081f: EXIST
