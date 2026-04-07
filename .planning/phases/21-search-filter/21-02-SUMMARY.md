---
phase: 21-search-filter
plan: "02"
subsystem: frontend + ui
tags: [search, leptos, component, dropdown, keyboard-nav, css]
dependency_graph:
  requires: [21-01-search-backend]
  provides: [GlobalSearchBar, SearchPanTrigger-context, top-bar-layout]
  affects: [resyn-app/components, resyn-app/app, resyn-app/style]
tech_stack:
  added: [leptos_use::signal_debounced, web-sys/KeyboardEvent, web-sys/HtmlInputElement, web-sys/FocusEvent]
  patterns: [Leptos Resource with debounced signal, Effect-based global event listener, context signal pattern]
key_files:
  created:
    - resyn-app/src/components/search_bar.rs
  modified:
    - resyn-app/src/app.rs
    - resyn-app/src/components/mod.rs
    - resyn-app/style/main.css
    - resyn-app/Cargo.toml
decisions:
  - "Use signal_debounced(query, 300.0) from leptos_use — avoids manual timeout/Effect for debouncing"
  - "Use on:mousedown (not on:click) on result rows — fires before blur so dropdown doesn't close before selection"
  - "200ms blur delay via set_timeout — gives mousedown time to fire before dropdown closes"
  - "Restructure content-area to flex column + content-scroll child — content-area owns top-bar, content-scroll owns page padding"
  - "Migrate graph page padding override from .content-area:has(.graph-page) to .content-scroll:has(.graph-page)"
  - "cb.forget() in Effect — global keydown listener intentionally lives for app lifetime"
metrics:
  duration: "~30 minutes"
  completed: "2026-04-07"
  tasks_completed: 2
  files_modified: 5
---

# Phase 21 Plan 02: GlobalSearchBar UI Component Summary

GlobalSearchBar component with 300ms debounced BM25 search, keyboard navigation (arrows/Enter/Escape), Ctrl+K global shortcut, and SearchPanTrigger context signal for graph page integration.

## What Was Built

**app.rs changes:**
- Added `SearchPanRequest` struct (paper_id field) and `SearchPanTrigger(RwSignal<Option<SearchPanRequest>>)` context type
- Provides `SearchPanTrigger` context in `App` component alongside existing `SelectedPaper` and `SidebarCollapsed`
- Restructured `<main class="content-area">` to contain a `<div class="top-bar">` (hosting `GlobalSearchBar`) and `<div class="content-scroll">` (hosting Routes)

**components/search_bar.rs (244 lines):**
- `GlobalSearchBar` component with `signal_debounced(query, 300.0)` triggering a `Resource` that calls `search_papers`
- Ctrl+K / Cmd+K global shortcut via `Effect::new` + `document.addEventListener("keydown", ...)` with `cb.forget()`
- Dropdown rendered only when `dropdown_open && !query.is_empty()` with `Suspense` for loading state
- "Searching..." loading state, "No papers found" + hint empty state, result rows with title + author/year
- Keyboard navigation: ArrowDown/ArrowUp move `focused_idx`, Enter selects focused (or first) result, Escape clears and closes
- `on:mousedown` (not `on:click`) on result rows to fire before blur event
- 200ms blur delay via `set_timeout` to allow click registration
- Result selection: always sets `SelectedPaper` (opens drawer); sets `SearchPanTrigger` only when `location.pathname == "/graph"` (D-08)
- Ctrl+K hint `<kbd>` badge hidden when input is focused

**main.css additions:**
- `.top-bar` — 48px min-height flex bar with surface background and bottom border
- `.global-search-wrapper` — relative positioned flex container (280–480px wide)
- `.global-search-bar` — pill input container with accent outline on focus
- `.search-kbd-hint` — styled `<kbd>` badge
- `.search-dropdown` — absolute positioned, z-index 200, `dropdown-open` animation
- `.search-result-row` — two-line row with hover + focused state, left border accent
- `.search-empty-state` / `.search-empty-hint` — empty state typography
- `.content-scroll` — flex:1 with overflow-y:auto and space-xl padding (replaces content-area padding)
- Migrated `.content-area:has(.graph-page)` override to `.content-scroll:has(.graph-page)`

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | SearchPanTrigger context + top-bar layout in app.rs | ddca701 |
| 2 | GlobalSearchBar component, mod.rs, main.css, Cargo.toml | 64e18b0 |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Migrated graph page padding override from .content-area to .content-scroll**
- **Found during:** Task 2 CSS changes
- **Issue:** Existing `.content-area:has(.graph-page) { padding: 0 }` rule targeted the old content-area which had `padding: var(--space-xl)`. After restructuring, padding lives in `.content-scroll`, so the override needed to target `.content-scroll:has(.graph-page)`.
- **Fix:** Updated the graph page padding override selector in main.css.
- **Files modified:** resyn-app/style/main.css
- **Commit:** 64e18b0

**2. [Rule 2 - Missing functionality] Added web-sys feature flags for keyboard/input/focus events**
- **Found during:** Task 2 implementation
- **Issue:** `web_sys::KeyboardEvent`, `HtmlInputElement`, and `FocusEvent` were not in the resyn-app Cargo.toml web-sys features list. The Ctrl+K shortcut requires `KeyboardEvent`.
- **Fix:** Added `KeyboardEvent`, `HtmlInputElement`, `FocusEvent` to `[dependencies.web-sys] features` in resyn-app/Cargo.toml.
- **Files modified:** resyn-app/Cargo.toml
- **Commit:** 64e18b0

## Known Stubs

None — `GlobalSearchBar` calls the real `search_papers` server fn backed by SurrealDB BM25 indexes (from Plan 01). Dropdown renders real results. `SearchPanTrigger` is set on result selection (Plan 03 will consume it for graph pan).

## Threat Flags

No new network endpoints or auth paths introduced. The `GlobalSearchBar` sends user-typed queries to the `search_papers` server fn via Leptos server fn serialization. Mitigations per threat register:
- T-21-05 (DoS): 300ms `signal_debounced` limits calls to ~3/sec; empty-string guard on server fn prevents wasteful queries (both implemented)
- T-21-06 (Ctrl+K spoofing): global shortcut only focuses an input element — no privilege escalation possible

## Self-Check: PASSED

- resyn-app/src/components/search_bar.rs — FOUND (244 lines)
- resyn-app/src/app.rs — contains SearchPanRequest, SearchPanTrigger, top-bar, content-scroll, GlobalSearchBar
- resyn-app/src/components/mod.rs — contains pub mod search_bar
- resyn-app/style/main.css — contains .top-bar, .global-search-bar, .search-dropdown, .search-result-row, .search-kbd-hint, .content-scroll
- Commit ddca701 — FOUND (Task 1)
- Commit 64e18b0 — FOUND (Task 2)
- cargo check -p resyn-app --features csr — PASSED (1 expected dead_code warning for SearchPanTrigger.paper_id, consumed by Plan 03)
