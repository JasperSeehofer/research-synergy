---
phase: 08-leptos-web-shell-analysis-panels
plan: "03"
subsystem: ui
tags: [leptos, axum, wasm, csr, ssr, leptos_axum, server-functions, surrealdb]

# Dependency graph
requires:
  - phase: 08-02
    provides: Trunk config, CSS design system, 5 page stub files, server_fns/mod.rs skeleton

provides:
  - 4 server function submodule stubs (papers, gaps, problems, methods) compiling for both csr and ssr
  - Axum serve command with --db, --port args and handle_server_fns_with_context registration
  - Static file fallback via tower-http ServeDir pointing at resyn-app/dist

affects:
  - 08-04
  - 08-05

# Tech tracking
tech-stack:
  added: []
  patterns:
    - handle_server_fns_with_context with Arc<Db> context injection via /api/{*fn_name} POST wildcard route
    - ServeDir fallback for production static file serving from resyn-app/dist

key-files:
  created:
    - resyn-app/src/server_fns/papers.rs
    - resyn-app/src/server_fns/gaps.rs
    - resyn-app/src/server_fns/problems.rs
    - resyn-app/src/server_fns/methods.rs
  modified:
    - resyn-app/src/server_fns/mod.rs
    - resyn-server/src/commands/serve.rs

key-decisions:
  - "Axum wildcard route uses /api/{*fn_name} not /api/*fn_name — Axum 0.8 path syntax requires braces for named wildcard segments"
  - "Server fn submodules are comment-only stubs — no #[server] macros yet to keep WASM build clean; implementations deferred to Plan 04"
  - "handle_server_fns_with_context injects Arc<Db> via provide_context — server functions will use use_context::<Arc<Db>>() in their bodies"

patterns-established:
  - "Pattern: Leptos server fn wildcard handler at /api/{*fn_name} with Arc<Db> context injection"
  - "Pattern: ServeDir fallback for dist/ — dev uses Trunk proxy, production uses Axum static serving"

requirements-completed: [WEB-04]

# Metrics
duration: 2min
completed: 2026-03-17
---

# Phase 8 Plan 03: Page Stubs + Server Function Scaffolding + Axum Serve Command Summary

**4 server function module stubs wired into resyn-app, and Axum serve command with leptos_axum::handle_server_fns_with_context registering all server functions with Arc<Db> context injection**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-03-17T06:57:39Z
- **Completed:** 2026-03-17T07:00:00Z
- **Tasks:** 2
- **Files modified:** 6 (4 created, 2 modified)

## Accomplishments

- `resyn-app/src/server_fns/{papers,gaps,problems,methods}.rs` created as comment-only stubs documenting the server functions to be implemented in Plan 04
- `resyn-app/src/server_fns/mod.rs` updated to declare all 4 submodules (previously had them commented out)
- `resyn-server/src/commands/serve.rs` rewritten with full Axum server: `--db` arg, `--port` arg, `connect_local` + `Arc<Db>`, `handle_server_fns_with_context` wildcard POST route at `/api/{*fn_name}`, `ServeDir` fallback for production static files
- All 5 page stubs were already present from Plan 02 with correct titles and empty-state messages
- 172 existing tests pass — zero regressions

## Task Commits

1. **Task 1: Server function module stubs** - `d0aedc4` (feat)
2. **Task 2: Axum serve command with server fn registration** - `2ebd6a8` (feat)

## Files Created/Modified

- `resyn-app/src/server_fns/papers.rs` — Comment stub documenting get_papers, get_paper_detail, get_dashboard_stats, start_crawl
- `resyn-app/src/server_fns/gaps.rs` — Comment stub documenting get_gap_findings
- `resyn-app/src/server_fns/problems.rs` — Comment stub documenting get_open_problems_ranked
- `resyn-app/src/server_fns/methods.rs` — Comment stub documenting get_method_matrix, get_method_drilldown
- `resyn-app/src/server_fns/mod.rs` — Updated to declare all 4 submodules (was fully commented out)
- `resyn-server/src/commands/serve.rs` — Rewritten: ServeArgs with --db/--port, connect_local, Arc<Db>, handle_server_fns_with_context, ServeDir fallback

## Decisions Made

- Axum wildcard route uses `/api/{*fn_name}` syntax (Axum 0.8 named wildcard) — the research example showed `/api/*fn_name` but that's Axum 0.7 syntax; braces are required in 0.8
- Server function stubs are comment-only (no `#[server]` macros) — keeps both the WASM and SSR builds clean; actual `#[server]` fns deferred to Plan 04 where DB types will be needed
- `handle_server_fns_with_context` provides `Arc<Db>` — server fn bodies will call `use_context::<Arc<Db>>().expect("DB not provided")` in Plan 04

## Deviations from Plan

None — plan executed exactly as written. Plan 02 had already created all 5 page files, so Task 1 focused only on the server function stubs. Task 2 went smoothly on first attempt.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- `cargo build -p resyn-app --target wasm32-unknown-unknown --features csr` — PASSES
- `cargo check -p resyn-server` — PASSES
- `cargo test` — 172 tests pass, zero regressions
- Server function modules ready for Plan 04 implementation (papers, gaps, problems, methods)
- Axum serve command ready — `resyn serve` will start on port 3000 with SurrealDB connection

## Self-Check: PASSED

- resyn-app/src/server_fns/papers.rs: FOUND
- resyn-app/src/server_fns/gaps.rs: FOUND
- resyn-app/src/server_fns/problems.rs: FOUND
- resyn-app/src/server_fns/methods.rs: FOUND
- resyn-app/src/server_fns/mod.rs: FOUND (updated)
- resyn-server/src/commands/serve.rs: FOUND (rewritten)
- Commit d0aedc4: FOUND
- Commit 2ebd6a8: FOUND

---
*Phase: 08-leptos-web-shell-analysis-panels*
*Completed: 2026-03-17*
