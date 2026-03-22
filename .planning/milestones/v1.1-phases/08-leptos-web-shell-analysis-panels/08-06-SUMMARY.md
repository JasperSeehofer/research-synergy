---
phase: 08-leptos-web-shell-analysis-panels
plan: "06"
subsystem: ui
tags: [leptos, wasm, sse, crawl-queue, heatmap, leptos-use, css-grid]

requires:
  - phase: 08-03
    provides: CSS design system (main.css) with heatmap and crawl form classes
  - phase: 08-04
    provides: server fns for get_method_matrix, get_method_drilldown, start_crawl stub; CrawlQueue infrastructure
provides:
  - CSS grid heatmap component with drill-down for method category co-occurrence matrix
  - Methods page wired to get_method_matrix and get_method_drilldown server functions
  - SSE-connected CrawlProgress component using leptos-use use_event_source
  - Real start_crawl server function dispatching via CrawlQueueRepository
affects:
  - 09-graph-visualization
  - future plans using SSE or crawl queue

tech-stack:
  added:
    - tokio (optional, ssr-gated in resyn-app for background crawl task)
    - tracing (optional, ssr-gated in resyn-app)
    - PartialEq derive on ProgressEvent (required by leptos-use use_event_source)
  patterns:
    - SSE consumed via leptos-use use_event_source::<T, JsonSerdeCodec> with Effect updating RwSignal
    - Background crawl spawned in start_crawl server fn; server fn returns immediately
    - Leptos Action wrapping server fn for form submission with pending/result state
    - Heatmap normalizes (row, col) key to alphabetical order to match pair_counts HashMap

key-files:
  created:
    - resyn-app/src/components/heatmap.rs
    - resyn-app/src/components/crawl_progress.rs
  modified:
    - resyn-app/src/pages/methods.rs
    - resyn-app/src/layout/sidebar.rs
    - resyn-app/src/server_fns/papers.rs
    - resyn-app/src/components/mod.rs
    - resyn-app/Cargo.toml
    - resyn-core/src/datamodels/progress.rs

key-decisions:
  - "ProgressEvent gains PartialEq derive (leptos-use use_event_source requires PartialEq on the decoded type)"
  - "tokio and tracing added as optional ssr-only deps in resyn-app (start_crawl body uses tokio::spawn and tracing::warn)"
  - "start_crawl spawns background tokio task with own source factory and CrawlQueueRepository instances (PaperSource is not Clone)"
  - "Heatmap normalizes (row, col) lookup to alphabetical order matching MethodMatrix::pair_counts key convention"

patterns-established:
  - "SSE pattern: use_event_source::<T, JsonSerdeCodec>(url) + Effect updating RwSignal on msg.data"
  - "Server fn action pattern: Action::new wrapping async server fn, is_pending/value signals drive UI state"
  - "Background task pattern: start_crawl returns immediately after seeding queue, background tokio::spawn runs loop"

requirements-completed: [AUI-03, WEB-03]

duration: 32min
completed: 2026-03-17
---

# Phase 8 Plan 06: Heatmap + Crawl Progress + Real start_crawl Summary

**CSS grid method heatmap with drill-down, SSE-connected crawl progress component with leptos-use, and start_crawl wired to CrawlQueueRepository with background tokio task**

## Performance

- **Duration:** 32 min
- **Started:** 2026-03-17T07:40:00Z
- **Completed:** 2026-03-17T08:12:00Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Method heatmap renders as CSS grid with dynamic column count, rotated column headers, row labels, and color-coded cells (empty/low/medium/high) based on co-occurrence count
- Methods page wires to get_method_matrix and get_method_drilldown server functions with drill-down state as RwSignal, loading/error/empty states, and back-to-overview navigation
- CrawlProgress component connects to `/progress` SSE endpoint via leptos-use use_event_source, shows running state (progress bar, stats, current paper) and idle state (last summary + crawl form)
- Sidebar footer uses CrawlProgress component replacing the static placeholder
- start_crawl server function validates arXiv ID, seeds CrawlQueueRepository, and spawns independent background crawl loop using the same pattern as resyn-server commands/crawl.rs

## Task Commits

1. **Task 1: Method heatmap component + Methods page with drill-down** - `ef984a9` (feat)
2. **Task 2: Crawl progress SSE component + crawl launcher form + sidebar integration + wire start_crawl** - `839436f` (feat)

## Files Created/Modified

- `resyn-app/src/components/heatmap.rs` — CSS grid Heatmap component for MethodMatrix with click-to-drilldown
- `resyn-app/src/components/crawl_progress.rs` — SSE-connected CrawlProgress and CrawlForm components
- `resyn-app/src/pages/methods.rs` — Full MethodsPanel with heatmap Resource, drill-down state, Suspense
- `resyn-app/src/layout/sidebar.rs` — CrawlProgress replaces CrawlProgressFooter placeholder
- `resyn-app/src/server_fns/papers.rs` — start_crawl wired to real CrawlQueueRepository + background task
- `resyn-app/src/components/mod.rs` — heatmap and crawl_progress modules enabled
- `resyn-app/Cargo.toml` — tokio + tracing added as optional ssr-gated deps
- `resyn-core/src/datamodels/progress.rs` — ProgressEvent gains PartialEq derive

## Decisions Made

- **ProgressEvent + PartialEq**: leptos-use `use_event_source` bounds `T: PartialEq` for change detection; added the derive to resyn-core's ProgressEvent.
- **tokio/tracing in resyn-app**: The `start_crawl` SSR body needs `tokio::spawn` and `tracing::warn`. Added as `optional = true` with `ssr` feature gate; no WASM cost.
- **Background crawl task**: Server function returns immediately after seeding queue. Background loop mirrors `resyn-server/src/commands/crawl.rs` but creates its own PaperSource instances per task (PaperSource is not Clone, per Phase 07-02 decision).
- **Heatmap key normalization**: `MethodMatrix::pair_counts` stores keys in alphabetical order (a <= b). The Heatmap component normalizes `(row, col)` lookup to match this convention so every cell correctly reads its count.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] MethodMatrix uses pair_counts HashMap, not cells Vec**
- **Found during:** Task 1 (Heatmap component)
- **Issue:** Plan interface spec showed `cells: Vec<MatrixCell>` but the actual resyn-core struct uses `pair_counts: HashMap<(String, String), u32>`. The plan spec was aspirational, the actual aggregation.rs never had MatrixCell.
- **Fix:** Implemented heatmap directly against the real MethodMatrix type with pair_counts lookup and alphabetical key normalization.
- **Files modified:** resyn-app/src/components/heatmap.rs
- **Verification:** WASM build succeeds
- **Committed in:** ef984a9 (Task 1 commit)

**2. [Rule 2 - Missing Critical] ProgressEvent needs PartialEq for use_event_source**
- **Found during:** Task 2 (CrawlProgress component)
- **Issue:** leptos-use use_event_source requires `T: PartialEq`; ProgressEvent didn't derive it.
- **Fix:** Added `PartialEq` to `#[derive(...)]` on ProgressEvent in resyn-core.
- **Files modified:** resyn-core/src/datamodels/progress.rs
- **Verification:** WASM build succeeds
- **Committed in:** 839436f (Task 2 commit)

**3. [Rule 2 - Missing Critical] use_event_source message is UseEventSourceMessage<T, C>, not T**
- **Found during:** Task 2 (CrawlProgress component)
- **Issue:** leptos-use SSE message signal yields `Option<UseEventSourceMessage<T,C>>` with `.data: T`; direct destructuring to T fails.
- **Fix:** Access `msg.data` in the Effect that updates last_event.
- **Files modified:** resyn-app/src/components/crawl_progress.rs
- **Verification:** SSR check passes
- **Committed in:** 839436f (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (1 wrong types bug, 2 missing critical derives/API usage)
**Impact on plan:** All auto-fixes required for correctness. No scope creep.

## Issues Encountered

- tokio/tracing not available in resyn-app SSR compilation (not in Cargo.toml) — resolved by adding them as optional ssr-gated deps, following the same pattern as leptos_axum.

## Next Phase Readiness

- Methods panel, crawl progress, and start_crawl are all fully wired and building cleanly
- Phase 9 (graph visualization) can proceed with the full web shell in place
- No blockers

---
*Phase: 08-leptos-web-shell-analysis-panels*
*Completed: 2026-03-17*
