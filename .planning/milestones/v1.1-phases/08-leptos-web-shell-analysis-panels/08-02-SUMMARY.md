---
phase: 08-leptos-web-shell-analysis-panels
plan: "02"
subsystem: ui
tags: [leptos, wasm, csr, trunk, css, routing, layout, sidebar, drawer]

# Dependency graph
requires:
  - phase: 08-01
    provides: resyn-app Cargo.toml with Leptos 0.8 deps, csr/ssr feature gates
provides:
  - Trunk.toml with proxy config to Axum (port 3000)
  - Complete CSS design system (all tokens from UI-SPEC)
  - App component with Router and 5 routes
  - Collapsible sidebar with nav links and crawl form
  - Paper detail drawer with skeleton loading state
  - Placeholder pages for all 5 panels
  - Empty component and server_fn module stubs
affects:
  - 08-03
  - 08-04
  - 08-05

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Leptos Callback uses .run() not .call() in 0.8 (reactive_graph Callable trait)
    - NavItem props must exactly match what is passed at call site — unused props cause build errors
    - web_sys re-exported via leptos::web_sys for WASM targets

key-files:
  created:
    - resyn-app/Trunk.toml
    - resyn-app/index.html
    - resyn-app/style/main.css
    - resyn-app/src/app.rs
    - resyn-app/src/layout/mod.rs
    - resyn-app/src/layout/sidebar.rs
    - resyn-app/src/layout/drawer.rs
    - resyn-app/src/pages/mod.rs
    - resyn-app/src/pages/dashboard.rs
    - resyn-app/src/pages/papers.rs
    - resyn-app/src/pages/gaps.rs
    - resyn-app/src/pages/open_problems.rs
    - resyn-app/src/pages/methods.rs
    - resyn-app/src/components/mod.rs
    - resyn-app/src/server_fns/mod.rs
  modified:
    - resyn-app/src/lib.rs

key-decisions:
  - "Leptos 0.8 Callback uses .run() not .call() — the Callable trait in reactive_graph exposes run() and try_run()"
  - "NavItem collapsed prop removed (CSS handles rail/expanded via class on parent) — avoids unused prop build errors"
  - "web_sys accessed via leptos::web_sys re-export in WASM context — no separate dep needed in Cargo.toml"

requirements-completed: [WEB-03]

# Metrics
duration: 8min
completed: 2026-03-17
---

# Phase 8 Plan 02: App Shell Summary

**Trunk config, complete CSS design system, and Leptos CSR app shell with Router, collapsible sidebar, paper detail drawer, and 5 placeholder pages — all compiling to WASM**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-03-17T06:45:31Z
- **Completed:** 2026-03-17T06:53:31Z
- **Tasks:** 1
- **Files modified:** 16 (1 modified, 15 created)

## Accomplishments

- `Trunk.toml` created with `[[proxy]]` to Axum at port 3000 (`/api/` and `/progress`)
- `index.html` minimal Trunk entry point linking `style/main.css`
- `style/main.css` — complete dark-theme CSS: all design tokens (colors, spacing, typography, layout), app shell, sidebar (expanded + rail), drawer, nav items, crawl form, cards, badges, tags, tables, dashboard cards, confidence bar, filter controls (toggle + slider), heatmap cells, ranked list, gap card expand/collapse, progress bar, skeleton shimmer animation, spinner, error banner, empty state
- `lib.rs` rewritten with `mount_to_body(App)`, module declarations (app, layout, pages, components, server_fns)
- `app.rs` — `App` component with `Router`, 5 routes (`/`, `/papers`, `/gaps`, `/problems`, `/methods`), `SelectedPaper(RwSignal<Option<String>>)` and `SidebarCollapsed(RwSignal<bool>)` provided as context
- `layout/sidebar.rs` — collapsible sidebar with 5 nav links (`A` components from leptos_router), chevron toggle, rail/expanded CSS class swap via signal, crawl form with paper ID input + depth select + source select + Start Crawl button
- `layout/drawer.rs` — paper detail drawer slides in from right, backdrop click-to-close, skeleton loading content
- `pages/`: Dashboard (5 summary cards), Papers (table with sortable column headers), Gaps (filter bar with toggles + confidence slider), OpenProblems, Methods — all as placeholder components
- `components/mod.rs` and `server_fns/mod.rs` — empty stubs for Plan 03
- 172 existing tests pass — zero regressions

## Task Commits

1. **Task 1: Trunk config + CSS design system + Leptos app shell with Router and Layout** - `af9168c` (feat)

## Files Created/Modified

- `resyn-app/Trunk.toml` — Trunk build config + proxy rules
- `resyn-app/index.html` — Trunk entry point
- `resyn-app/style/main.css` — Full CSS design system (~580 lines)
- `resyn-app/src/lib.rs` — WASM entry point (rewritten)
- `resyn-app/src/app.rs` — App component with Router and 5 routes
- `resyn-app/src/layout/mod.rs` — Layout module declarations
- `resyn-app/src/layout/sidebar.rs` — Collapsible sidebar
- `resyn-app/src/layout/drawer.rs` — Paper detail drawer
- `resyn-app/src/pages/mod.rs` — Page module declarations
- `resyn-app/src/pages/dashboard.rs` — Dashboard placeholder
- `resyn-app/src/pages/papers.rs` — Papers panel placeholder
- `resyn-app/src/pages/gaps.rs` — Gaps panel placeholder (with filter signals)
- `resyn-app/src/pages/open_problems.rs` — Open Problems placeholder
- `resyn-app/src/pages/methods.rs` — Methods panel placeholder
- `resyn-app/src/components/mod.rs` — Component stubs
- `resyn-app/src/server_fns/mod.rs` — Server function stubs

## Decisions Made

- `Callback::run()` not `call()` — in Leptos 0.8, the `Callback` type implements the `Callable` trait from `reactive_graph`, which exposes `run()` and `try_run()` methods
- Sidebar CSS class handling via parent — the `NavItem` component does not need a `collapsed` prop; the parent `<nav class=sidebar_class>` drives the rail/expanded toggle purely via CSS
- `web_sys::MouseEvent` referenced as `leptos::web_sys::MouseEvent` in WASM context (re-exported by Leptos)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] web_sys not directly importable without re-export path**
- **Found during:** Task 1 (WASM compilation)
- **Issue:** `web_sys::MouseEvent` in drawer.rs caused `E0433: unresolved module` — `web_sys` needs to be accessed via `leptos::web_sys`
- **Fix:** Added `use leptos::web_sys;` import at top of drawer.rs
- **Files modified:** `resyn-app/src/layout/drawer.rs`
- **Commit:** af9168c (fixed before commit)

**2. [Rule 1 - Bug] Callback::call() does not exist in Leptos 0.8**
- **Found during:** Task 1 (WASM compilation)
- **Issue:** `on_close.call(e)` — the `Callable` trait uses `.run()` not `.call()`
- **Fix:** Changed to `on_close.run(e)`
- **Files modified:** `resyn-app/src/layout/drawer.rs`
- **Commit:** af9168c (fixed before commit)

**3. [Rule 1 - Bug] NavItem with unused `collapsed` prop caused prop builder errors**
- **Found during:** Task 1 (WASM compilation)
- **Issue:** `NavItem` had a `collapsed: RwSignal<bool>` prop that was renamed to `_collapsed` (unused), but the call sites still passed `collapsed=collapsed`, causing `E0599: no method named 'collapsed'`
- **Fix:** Removed the prop entirely from `NavItem` — CSS handles visibility via the parent `<nav>` class
- **Files modified:** `resyn-app/src/layout/sidebar.rs`
- **Commit:** af9168c (fixed before commit)

## Issues Encountered

None beyond the 3 auto-fixed compilation errors above.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- `cargo build -p resyn-app --target wasm32-unknown-unknown --features csr` — PASSES
- `cargo test` — 172 tests pass, zero regressions
- App shell structure ready for Plan 03 (page implementations + server functions)
- CSS design system complete — all component styles defined and ready to use

## Self-Check: PASSED

- resyn-app/Trunk.toml: FOUND
- resyn-app/style/main.css: FOUND
- resyn-app/src/app.rs: FOUND
- resyn-app/src/layout/sidebar.rs: FOUND
- resyn-app/src/layout/drawer.rs: FOUND
- Commit af9168c: FOUND

---
*Phase: 08-leptos-web-shell-analysis-panels*
*Completed: 2026-03-17*
