---
phase: 08-leptos-web-shell-analysis-panels
plan: "04"
subsystem: ui
tags: [leptos, wasm, csr, ssr, server-functions, surrealdb, aggregation, testing]

# Dependency graph
requires:
  - phase: 08-03
    provides: 4 server fn module stubs, Axum serve command with handle_server_fns_with_context

provides:
  - 7 Leptos server functions compiling for both WASM (csr stub) and native (ssr body)
  - Pure aggregation helpers in resyn-core (no ssr gate, fully unit-testable)
  - Dashboard with 5 linked summary cards using Resource + Suspense + skeleton fallback
  - Sortable papers table with 4 sortable columns and row-click to detail drawer
  - Paper detail drawer fetching PaperDetail (paper + annotation) with full content rendering
  - 6 integration tests for aggregate_open_problems and build_method_matrix

affects:
  - 08-05
  - 08-06

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "SSR-only imports gated inside #[cfg(feature = \"ssr\")] blocks inside server fn bodies (not at module level)"
    - "Aggregation helpers moved to resyn-core/src/analysis/aggregation.rs — pure functions, WASM-safe, no ssr gate"
    - "Resource::new(|| (), |_| server_fn()) + Suspense + skeleton fallback for all data panels"
    - "RwSignal<(SortColumn, SortDir)> for client-side table sorting without server refetch"
    - "Callback<web_sys::MouseEvent> for drawer close — use leptos::web_sys import"

key-files:
  created:
    - resyn-core/src/analysis/mod.rs
    - resyn-core/src/analysis/aggregation.rs
    - resyn-core/tests/aggregation_tests.rs
  modified:
    - resyn-core/src/lib.rs
    - resyn-app/src/server_fns/papers.rs
    - resyn-app/src/server_fns/gaps.rs
    - resyn-app/src/server_fns/problems.rs
    - resyn-app/src/server_fns/methods.rs
    - resyn-app/src/pages/dashboard.rs
    - resyn-app/src/pages/papers.rs
    - resyn-app/src/layout/drawer.rs

key-decisions:
  - "Aggregation helpers (aggregate_open_problems, build_method_matrix) placed in resyn-core/src/analysis/aggregation.rs not in resyn-app — keeps them WASM-safe and testable without Leptos feature gates"
  - "SSR-only imports (database modules, LlmAnnotation, build_method_matrix fn) moved inside #[cfg(feature = \"ssr\")] blocks — top-level imports cause unused-import warnings on WASM build"
  - "Status column in papers table is non-sortable — SortColumn enum has 4 variants (Title, Authors, Year, Citations) only; Status column has no clickable header"
  - "year_from_published returns &str borrowed from paper — must call .to_string() in PaperRow component where paper is moved into closure"

patterns-established:
  - "Pattern: resyn-core/src/analysis/ module for pure aggregation/analysis functions that need to be unit-tested without ssr feature"
  - "Pattern: #[cfg(feature = \"ssr\")] block encloses all SSR imports AND logic inside server fn body"

requirements-completed: [WEB-04, AUI-02, AUI-03]

# Metrics
duration: 18min
completed: 2026-03-17
---

# Phase 8 Plan 04: Server Functions, Dashboard, Papers Table, Paper Drawer Summary

**7 Leptos server functions wired to SurrealDB, dashboard with live summary cards, sortable papers table with drawer, and 6 unit tests for pure aggregation logic**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-03-17T07:04:16Z
- **Completed:** 2026-03-17T07:22:15Z
- **Tasks:** 3
- **Files modified:** 11 (3 created, 8 modified)

## Accomplishments

- 7 server functions (get_papers, get_paper_detail, get_dashboard_stats, start_crawl, get_gap_findings, get_open_problems_ranked, get_method_matrix, get_method_drilldown) fully implemented with SSR DB access and WASM stub compilation
- Pure aggregation helpers extracted to `resyn-core/src/analysis/aggregation.rs` (aggregate_open_problems + build_method_matrix), no feature gate required, 6 integration tests passing
- Dashboard replaced from static placeholder to Resource-based with Suspense skeleton, 5 linked summary cards with live counts from DB
- Papers table fully sortable client-side by 4 columns, row click opens drawer via SelectedPaper context
- Paper drawer fetches PaperDetail from server and renders title, authors, abstract, methods (tags), findings, open problems with skeleton loading state
- 182 total tests pass (176 existing + 6 new), zero regressions

## Task Commits

1. **Task 1: Server functions + aggregation helpers** - `906ca74` (feat)
2. **Task 2: Dashboard + papers table + paper drawer** - `237408b` (feat)
3. **Task 3: Integration tests for aggregation** - `906752e` (test)

## Files Created/Modified

- `resyn-core/src/analysis/aggregation.rs` — Pure aggregate_open_problems() and build_method_matrix() with inline unit tests
- `resyn-core/src/analysis/mod.rs` — Module declaration
- `resyn-core/src/lib.rs` — Added `pub mod analysis`
- `resyn-core/tests/aggregation_tests.rs` — 6 integration tests (empty, ranking, single-annotation, pair counts, normalization)
- `resyn-app/src/server_fns/papers.rs` — get_papers, get_paper_detail, get_dashboard_stats, start_crawl (stub)
- `resyn-app/src/server_fns/gaps.rs` — get_gap_findings
- `resyn-app/src/server_fns/problems.rs` — get_open_problems_ranked
- `resyn-app/src/server_fns/methods.rs` — get_method_matrix, get_method_drilldown
- `resyn-app/src/pages/dashboard.rs` — Resource + Suspense + 5 summary cards + empty state
- `resyn-app/src/pages/papers.rs` — Sortable table + row click + skeleton + error banner
- `resyn-app/src/layout/drawer.rs` — Full drawer with Resource fetch + content rendering + skeleton

## Decisions Made

- Aggregation helpers in resyn-core (not resyn-app) so they're testable without Leptos feature gates
- All SSR-only imports placed inside `#[cfg(feature = "ssr")]` blocks (not module-level), avoiding unused-import warnings on WASM build
- Status column non-sortable — matches the UI spec where Status column has no sort affordance
- `year_from_published` returns owned `String` in PaperRow because the paper is moved into the view closure

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Moved SSR-only imports from module level to inside cfg(ssr) blocks**
- **Found during:** Task 1 (server function implementation)
- **Issue:** Top-level `use resyn_core::database::...` imports caused "unused import" warnings on WASM build since the database code is ssr-only
- **Fix:** Moved all SSR-dependent imports inside `#[cfg(feature = "ssr")]` blocks within server fn bodies
- **Files modified:** methods.rs, problems.rs
- **Verification:** `cargo check -p resyn-app --features csr --target wasm32-unknown-unknown` passes clean
- **Committed in:** 906ca74 (Task 1 commit)

**2. [Rule 1 - Bug] `<A class=...>` not valid in leptos_router 0.8**
- **Found during:** Task 2 (dashboard implementation)
- **Issue:** `leptos_router::components::A` does not accept `class` as a prop — compiler error "trait bounds not satisfied"
- **Fix:** Replaced `<A class="...">` with plain `<a class="...">` for dashboard card links
- **Files modified:** resyn-app/src/pages/dashboard.rs
- **Verification:** WASM build passes
- **Committed in:** 237408b (Task 2 commit)

**3. [Rule 1 - Bug] `year_from_published` borrowed string from moved paper**
- **Found during:** Task 2 (PaperRow component)
- **Issue:** `year_from_published(&paper.published)` returns `&str` borrowed from paper, but paper is moved into the `view!` macro closure, causing lifetime error
- **Fix:** Called `.to_string()` to produce owned `String`
- **Files modified:** resyn-app/src/pages/papers.rs
- **Verification:** WASM build passes
- **Committed in:** 237408b (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (all Rule 1 — compile bugs)
**Impact on plan:** All fixes necessary for compilation correctness. No scope creep.

## Issues Encountered

None beyond the auto-fixed compile errors above.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Server functions ready: all 7 compile for both targets, registered via handle_server_fns_with_context in Plan 03
- Dashboard reads DashboardStats from DB via get_dashboard_stats server fn
- Papers table shows sorted papers from get_papers, drawer shows PaperDetail from get_paper_detail
- aggregate_open_problems and build_method_matrix ready for use in Plans 05 (Gaps panel) and remaining analysis panels
- `cargo build -p resyn-app --target wasm32-unknown-unknown --features csr` PASSES
- `cargo check -p resyn-app --features ssr` PASSES
- `cargo test -p resyn-core --test aggregation_tests` PASSES (6/6)
- `cargo test` PASSES (182/182)

## Self-Check: PASSED

- resyn-core/src/analysis/aggregation.rs: FOUND
- resyn-core/src/analysis/mod.rs: FOUND
- resyn-core/tests/aggregation_tests.rs: FOUND
- resyn-app/src/server_fns/papers.rs: FOUND (contains #[server)
- resyn-app/src/server_fns/gaps.rs: FOUND (contains #[server)
- resyn-app/src/server_fns/problems.rs: FOUND (contains #[server)
- resyn-app/src/server_fns/methods.rs: FOUND (contains #[server)
- resyn-app/src/pages/dashboard.rs: FOUND (contains summary-card)
- resyn-app/src/pages/papers.rs: FOUND (contains data-table)
- Commit 906ca74: FOUND
- Commit 237408b: FOUND
- Commit 906752e: FOUND

---
*Phase: 08-leptos-web-shell-analysis-panels*
*Completed: 2026-03-17*
