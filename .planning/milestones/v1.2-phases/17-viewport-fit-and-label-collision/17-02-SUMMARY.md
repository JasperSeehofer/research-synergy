---
phase: 17-viewport-fit-and-label-collision
plan: "02"
subsystem: resyn-app/graph
tags: [labels, collision-avoidance, canvas-2d, performance, UX]
dependency_graph:
  requires: [17-01]
  provides: [label_collision_module, pill_label_rendering, measuretext_cache, hover_label_override]
  affects:
    - resyn-app/src/graph/label_collision.rs
    - resyn-app/src/graph/canvas_renderer.rs
    - resyn-app/src/graph/renderer.rs
    - resyn-app/src/pages/graph.rs
tech_stack:
  added: []
  patterns:
    - greedy-collision-avoidance
    - screen-space-label-rendering
    - measuretext-cache-at-load
    - dirty-flag-per-frame-viewport-diff
key_files:
  created:
    - resyn-app/src/graph/label_collision.rs
  modified:
    - resyn-app/src/graph/mod.rs
    - resyn-app/src/graph/canvas_renderer.rs
    - resyn-app/src/graph/renderer.rs
    - resyn-app/src/pages/graph.rs
decisions:
  - Used arc_to rounded rect path instead of round_rect_with_f64 for web-sys version compatibility
  - Renderer trait extended with default no-op set_label_cache and set_fit_anim_active methods
  - Temporary offscreen canvas used for measureText so text widths work regardless of which renderer (Canvas2D or WebGL2) was selected
  - Label cache dirty detection compares viewport scale/offset each frame (>0.0001 scale, >0.1px offset threshold)
metrics:
  duration: 4min
  completed: "2026-03-26T10:10:42Z"
  tasks: 2
  files: 5
---

# Phase 17 Plan 02: Label Collision Avoidance and Pill Rendering Summary

Priority-ordered greedy label collision avoidance with pill/badge styling, per-load measureText caching, hover override, and fit-animation suppression.

## What Was Built

**label_collision.rs** — New pure-logic module with `LabelCache` struct, `build_label_cache()` (priority sort: seed first, then descending citation count; greedy O(n) screen-space overlap test with 8px COLLISION_PAD), and `build_text_widths()` (browser-only measureText cache). Constants: `PILL_HEIGHT=20`, `PILL_H_PAD=8`, `COLLISION_PAD=8`, `LABEL_NODE_GAP=8`, `PILL_CORNER_RADIUS=4`. Verified with 7 unit tests covering all priority/collision/visibility/empty cases.

**canvas_renderer.rs** — Old naive label loop (lines 288-308) fully replaced. New screen-space label pass: resets canvas transform to DPR-scaled identity, draws pill badges using `arc_to` rounded-rect path (`rgba(13,17,23,0.85)` bg, `#30363d` border, `#cccccc` text), iterates `LabelCache.visible_indices`. Hover label override draws label for `hovered_node` even if culled. Labels suppressed when `fit_anim_active == true`. New `draw_label_pill()` helper keeps drawing code DRY.

**renderer.rs** — `Renderer` trait gains two default no-op methods: `set_label_cache()` and `set_fit_anim_active()`. Override in `Canvas2DRenderer`; WebGL2Renderer inherits no-ops. This avoids downcasting and keeps the extension backward-compatible.

**graph.rs** — `RenderState` gains `text_widths: Vec<f64>`. At graph load, a temporary offscreen canvas measures all node labels once (`build_text_widths`). RAF loop: per-frame viewport change detection (`prev_scale/offset_x/offset_y`) marks `label_cache_dirty`. When not animating and dirty, `build_label_cache()` is called and pushed to renderer via `set_label_cache()`. During fit animation, `set_label_cache(None)` + `set_fit_anim_active(true)` suppress all labels.

## Task Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create label_collision module with priority sort and greedy placement | a34840b | label_collision.rs, mod.rs |
| 2 | Replace naive label loop with collision-aware pill rendering and wire label cache lifecycle | 851a415 | canvas_renderer.rs, renderer.rs, graph.rs |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Used arc_to path instead of round_rect_with_f64**
- **Found during:** Task 2 — plan notes Pitfall 5 about web-sys binding availability
- **Issue:** `round_rect_with_f64` requires Canvas 2D Level 2 API in web-sys; plan suggested fallback if unavailable
- **Fix:** Implemented the 8-arc `arc_to` rounded-rect path directly as the primary implementation. More compatible, no compilation check needed.
- **Files modified:** resyn-app/src/graph/canvas_renderer.rs
- **Commit:** 851a415

**2. [Rule 1 - Bug] Collapsed nested if-let with && let chain for clippy**
- **Found during:** Task 2 — `cargo clippy -p resyn-app -- -D warnings`
- **Issue:** clippy::collapsible_if on `if let Some(hi) = hovered_node { if hi < len && !contains { ... } }`
- **Fix:** Rewrote as `if let Some(hi) = state.hovered_node && hi < len && !cache.contains(&hi)`
- **Files modified:** resyn-app/src/graph/canvas_renderer.rs
- **Commit:** 851a415

None beyond the above.

## Verification

- `cargo test -p resyn-app` — 64 passed (57 existing + 7 new label_collision tests)
- `cargo clippy -p resyn-app -- -D warnings` — clean
- `cargo fmt --all -- --check` — clean
- `build_label_cache` called from graph.rs RAF loop (line 541)
- `build_text_widths` called at graph load time (line 163), NOT in RAF loop
- Old naive label loop removed: no `measure_text` in `draw` method
- `arc_to` used for rounded pill corners (7 occurrences)
- `set_transform(dpr, 0, 0, dpr, 0, 0)` resets to screen space before label draw
- `hovered_node` checked for hover label override (line 345)

## Known Stubs

None — label collision is fully wired to live node data from GraphState.

## Self-Check: PASSED

- resyn-app/src/graph/label_collision.rs: EXISTS
- resyn-app/src/graph/mod.rs: contains `pub mod label_collision;`
- resyn-app/src/graph/canvas_renderer.rs: contains `rgba(13,17,23,0.85)`, `#30363d`, `#cccccc`, `arc_to`, `set_transform`, `hovered_node`, `visible_indices`, `fit_anim_active`
- resyn-app/src/pages/graph.rs: contains `build_label_cache`, `build_text_widths`, `text_widths`, `label_cache_dirty`
- Commits a34840b and 851a415: EXIST
