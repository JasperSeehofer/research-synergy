---
phase: 08-leptos-web-shell-analysis-panels
plan: "07"
subsystem: ui
tags: [leptos, wasm, trunk, browser-verification, csr, surrealkv, server-fn-registration]

requires:
  - phase: 08-04
    provides: server functions (get_papers, get_dashboard_stats, start_crawl, etc.)
  - phase: 08-05
    provides: Gap Findings and Open Problems panels
  - phase: 08-06
    provides: Method heatmap component and SSE crawl progress
provides:
  - Human-verified confirmation that all 5 panels render correctly in the browser
  - Trunk CSR build with correct data-cargo-features="csr" declaration in index.html
  - Explicit server function registration via register_explicit (cross-crate boundary fix)
  - DB connection using connect() instead of connect_local() (eliminated double-prefix bug)
  - Visible sidebar toggle button (background-color fix)
  - Seeded test database (test_data/) with real papers for pipeline verification
affects:
  - 09-graph-visualization
  - any plan using trunk serve or resyn-server serve

tech-stack:
  added:
    - server_fn 0.8 with axum-no-default feature (explicit server fn registration)
  patterns:
    - Trunk requires data-cargo-features="csr" in index.html link tag (not just Cargo.toml feature gate)
    - Server functions must be registered via register_explicit<T>() at server startup when auto-registration
      fails across crate boundaries (resyn-app -> resyn-server)
    - connect() handles any connection string; connect_local() prepends surrealkv:// prefix (avoid with --db arg)

key-files:
  created: []
  modified:
    - resyn-app/index.html
    - resyn-app/style/main.css
    - resyn-server/Cargo.toml
    - resyn-server/src/commands/serve.rs

key-decisions:
  - "Trunk passes Cargo features via data-cargo-features attribute in index.html, not only via Cargo.toml features"
  - "register_explicit<T>() required for all server fns at serve startup — inventory-based auto-registration fails across crate boundaries in this workspace setup"
  - "connect() is the correct function for serve command; connect_local() was double-prefixing the surrealkv:// scheme"

patterns-established:
  - "Browser verification plan: Task 1 runs full automated checks + data seed; Task 2 is human-verify checkpoint"

requirements-completed:
  - WEB-03
  - AUI-01
  - AUI-02
  - AUI-03

duration: 45min
completed: 2026-03-17
---

# Phase 8 Plan 07: Browser Verification Summary

**Leptos CSR web shell verified in browser with real data — all 5 panels render with dark theme, navigation, and interactions working; 4 production bugs fixed during verification**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-03-17 (continuation after Task 1 commit b3fa044)
- **Completed:** 2026-03-17
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- All 5 panels (Dashboard, Papers, Gap Findings, Open Problems, Methods) verified rendering correctly in the browser with dark theme
- Papers panel showed real data from seeded database (non-zero paper count, sortable table, working drawer)
- Sidebar collapse/expand working correctly; navigation between all routes confirmed
- 4 production bugs identified and fixed: Trunk feature flag, invisible sidebar toggle, broken server fn registration, double-prefixed DB connection string

## Task Commits

Each task was committed atomically:

1. **Task 1: Final compilation, automated verification, and seed test data** - `b3fa044` (fix)
2. **Task 2: Browser verification (post-verification fixes)** - `e3bb048` (fix)

**Plan metadata:** (this commit — docs: complete plan)

## Files Created/Modified

- `resyn-app/index.html` — Added `data-cargo-features="csr"` so Trunk passes CSR feature to WASM build
- `resyn-app/style/main.css` — Added `background-color` to `.sidebar-toggle` (button was invisible)
- `resyn-server/Cargo.toml` — Added `server_fn` dependency for explicit server fn registration
- `resyn-server/src/commands/serve.rs` — Added `register_explicit<T>()` calls for all 8 server fns; switched `connect_local` to `connect`

## Decisions Made

- Used `register_explicit<T>()` from `server_fn::axum` for all server functions — the inventory-based auto-registration that Leptos 0.8 normally uses does not propagate correctly when server fns are defined in `resyn-app` and served from `resyn-server` (separate crates). Explicit registration at startup is the reliable cross-crate pattern.
- `connect()` (not `connect_local()`) is correct for the serve command when the user supplies a full `surrealkv://./test_data` string. `connect_local()` prepends the scheme prefix, producing `surrealkv://surrealkv://./test_data`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Trunk not passing CSR feature to WASM compilation**
- **Found during:** Task 2 (browser verification)
- **Issue:** `trunk serve` compiled the WASM binary without the `csr` feature, so Leptos components were excluded and the page was blank
- **Fix:** Added `<link data-trunk rel="rust" data-cargo-features="csr"/>` to `resyn-app/index.html`
- **Files modified:** resyn-app/index.html
- **Verification:** Page renders after fix
- **Committed in:** e3bb048

**2. [Rule 1 - Bug] Sidebar toggle button invisible against page background**
- **Found during:** Task 2 (browser verification)
- **Issue:** `.sidebar-toggle` CSS had no background-color, making the button invisible against the dark `#0d1117` background
- **Fix:** Added `background-color: var(--color-surface-raised)` to `.sidebar-toggle` rule
- **Files modified:** resyn-app/style/main.css
- **Verification:** Button visible in browser after fix
- **Committed in:** e3bb048

**3. [Rule 1 - Bug] Server functions returning 404 — auto-registration failed across crate boundaries**
- **Found during:** Task 2 (browser verification)
- **Issue:** All server function calls returned 404. Leptos inventory-based auto-registration does not work when server fns are defined in `resyn-app` but served from `resyn-server`
- **Fix:** Added `server_fn` dependency and registered all 8 server functions explicitly via `register_explicit::<T>()` in `serve.rs`
- **Files modified:** resyn-server/Cargo.toml, resyn-server/src/commands/serve.rs
- **Verification:** Server fn calls succeed, panels load data
- **Committed in:** e3bb048

**4. [Rule 1 - Bug] DB connection failed with double-prefixed connection string**
- **Found during:** Task 2 (browser verification)
- **Issue:** `connect_local("surrealkv://./test_data")` produced `surrealkv://surrealkv://./test_data`, causing a connection error
- **Fix:** Switched to `connect(&args.db)` which accepts any connection string as-is
- **Files modified:** resyn-server/src/commands/serve.rs
- **Verification:** Server connects to DB successfully
- **Committed in:** e3bb048

---

**Total deviations:** 4 auto-fixed (4 Rule 1 bugs)
**Impact on plan:** All 4 fixes were required for the application to run at all. No scope creep.

## Issues Encountered

All 4 issues were discovered during browser verification and fixed before final sign-off. The core application logic (components, server fn implementations, routing) was correct — the bugs were all in the build/serve plumbing layer.

## User Setup Required

None - no external service configuration required. Test data can be seeded with:
```bash
cargo run -p resyn-server -- crawl --paper-id 2503.18887 --max-depth 1 --db surrealkv://./test_data
```

## Next Phase Readiness

- All 5 analysis panels are verified working end-to-end with real data from the database
- Dark theme is consistent across all panels
- The complete Phase 8 web shell is ready; Phase 9 (graph visualization) can begin
- Blocker from STATE.md (Canvas 2D + Leptos NodeRef pattern validation) should be spiked before full Phase 9 implementation

---
*Phase: 08-leptos-web-shell-analysis-panels*
*Completed: 2026-03-17*
