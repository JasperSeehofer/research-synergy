---
phase: 08-leptos-web-shell-analysis-panels
plan: "05"
subsystem: ui
tags: [leptos, wasm, csr, gap-findings, open-problems, filter-bar, ranked-list]

# Dependency graph
requires:
  - phase: 08-04
    provides: get_gap_findings and get_open_problems_ranked server fns, GapFinding/RankedProblem types

provides:
  - GapCard component with type badge, clickable paper ID links, shared term tags, confidence bar, expand/collapse justification
  - Gap Findings page with Contradictions/Bridges type toggle filter and confidence threshold slider
  - Open Problems page with ranked list sorted by recurrence count and recurrence badges

affects:
  - cross-panel navigation: paper IDs in gap cards open the paper detail drawer via SelectedPaper context

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "StoredValue::new(all_findings) to share loaded data between two closures inside a Suspense reactive branch"
    - "Sub-component pattern for <For> loops inside match arms to avoid Leptos view! macro move/borrow conflicts"
    - "GapType filter via match in .filter() closure — no intermediate signal needed, just re-reads filter RwSignals"

key-files:
  created:
    - resyn-app/src/components/gap_card.rs
  modified:
    - resyn-app/src/components/mod.rs
    - resyn-app/src/pages/gaps.rs
    - resyn-app/src/pages/open_problems.rs

key-decisions:
  - "StoredValue used (not RwSignal) to share immutable loaded Vec<GapFinding> across two closures — avoids Clone overhead on re-render"
  - "RankedList extracted as sub-component so <For each=move ||> is not directly inside a view! macro match arm (prevents Leptos parser errors on the move keyword)"
  - "RankedProblem field is .problem not .text — verified from aggregation.rs source"

# Metrics
duration: 14min
completed: 2026-03-17
---

# Phase 8 Plan 05: Gap Findings Panel + Open Problems Panel Summary

**Gap findings panel with type/confidence filter bar and expandable gap cards; open problems panel with recurrence-ranked list**

## Performance

- **Duration:** ~14 min
- **Started:** 2026-03-17T07:26:13Z
- **Completed:** 2026-03-17T07:40:17Z
- **Tasks:** 2
- **Files modified:** 4 (1 created, 3 modified)

## Accomplishments

- `GapCard` component built with: type badge (contradiction = red, bridge = amber), clickable paper ID buttons that set `SelectedPaper` context to open the detail drawer, shared terms rendered as `.tag` pills, confidence bar with gradient fill and percentage label, expand/collapse toggle for justification text via local `RwSignal<bool>`
- Gap Findings page replaces placeholder: `Resource::new(|| (), |_| get_gap_findings())` + `Suspense` + 3-skeleton fallback; filter bar with Contradictions/Bridges toggles (`.filter-toggle.active`) and confidence threshold range slider; derived filtering via `StoredValue` pattern; three distinct empty states (no data, filter mismatch) and error banner per UI-SPEC copywriting
- Open Problems page replaces placeholder: `Resource::new(|| (), |_| get_open_problems_ranked())` + `Suspense` + 5-skeleton fallback; `RankedList` sub-component renders `<ul class="ranked-list">` with rank number, problem text, and recurrence badge; loading/error/empty states per UI-SPEC copywriting
- 182 tests pass (no regressions)
- WASM build: `cargo build -p resyn-app --target wasm32-unknown-unknown --features csr` PASSES

## Task Commits

1. **Task 1: GapCard component + Gap Findings page** - `0cc3bac` (feat)
2. **Task 2: Open Problems ranked list page** - `7898787` (feat)

## Files Created/Modified

- `resyn-app/src/components/gap_card.rs` — GapCard component: badge, paper links, tags, confidence bar, expand/collapse
- `resyn-app/src/components/mod.rs` — Enabled `pub mod gap_card` (heatmap and crawl_progress already present)
- `resyn-app/src/pages/gaps.rs` — Full Gap Findings page with filter bar, Resource fetch, derived filtering
- `resyn-app/src/pages/open_problems.rs` — Full Open Problems page with ranked list

## Decisions Made

- `StoredValue` used (not `RwSignal`) to share immutable loaded `Vec<GapFinding>` across two closures — avoids unnecessary reactive tracking on a value that never changes after load
- `RankedList` extracted as a sub-component so `<For each=move ||>` is not placed directly inside a `view!` macro match arm, which causes the Leptos parser to misinterpret the `move` keyword
- `RankedProblem.problem` (not `.text`) — verified from `resyn-core/src/analysis/aggregation.rs`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Move/borrow conflict: `all_findings` moved into `filtered` closure then used in `empty_state` check**
- **Found during:** Task 1 (gaps.rs implementation)
- **Issue:** `all_findings` was moved into the `filtered = move ||` closure and then borrowed in a second `move ||` closure checking `all_findings.is_empty()`
- **Fix:** Wrapped loaded findings in `StoredValue::new(all_findings)` so both closures can call `.get_value()` independently
- **Files modified:** resyn-app/src/pages/gaps.rs
- **Commit:** 0cc3bac

**2. [Rule 1 - Bug] Leptos parser error: `each=move ||` inside match arm in view! macro**
- **Found during:** Task 2 (open_problems.rs implementation)
- **Issue:** `<For each=move || items.clone().into_iter().enumerate()...>` directly inside a `view!` match arm caused the Leptos macro parser to error (`expected identifier, found keyword move`)
- **Fix:** Extracted `RankedList` sub-component; `<For>` lives in the sub-component's top-level `view!` block where the `move` keyword is unambiguous
- **Files modified:** resyn-app/src/pages/open_problems.rs
- **Commit:** 7898787

**3. [Rule 1 - Bug] Wrong field name `text` on `RankedProblem`**
- **Found during:** Task 2 (open_problems.rs implementation)
- **Issue:** Used `problem.text` but `RankedProblem` struct has field `problem: String` not `text`
- **Fix:** Changed to `problem.problem`
- **Files modified:** resyn-app/src/pages/open_problems.rs
- **Commit:** 7898787

---

**Total deviations:** 3 auto-fixed (all Rule 1 — compile bugs)
**Impact on plan:** All fixes necessary for compilation correctness. No scope creep.

## Issues Encountered

The IDE linter (rust-analyzer/editor) repeatedly uncommented `pub mod crawl_progress` and `pub mod heatmap` in `components/mod.rs` after each write. Both files existed from Plan 06 and compiled correctly, so this had no negative effect — it simply exposed those modules earlier than this plan intended. The WASM build continued to pass throughout.

## User Setup Required

None.

## Next Phase Readiness

- Gap findings panel: fully functional with filter bar, card list, cross-panel drawer navigation
- Open problems panel: fully functional ranked list with recurrence counts
- Both panels match UI-SPEC copywriting for all three states (loading/error/empty)
- `cargo build -p resyn-app --target wasm32-unknown-unknown --features csr` PASSES
- `cargo test` PASSES (182/182)

## Self-Check: PASSED

- resyn-app/src/components/gap_card.rs: FOUND
- resyn-app/src/pages/gaps.rs: FOUND (contains StoredValue, filter-toggle)
- resyn-app/src/pages/open_problems.rs: FOUND (contains ranked-list, RankedList)
- Commit 0cc3bac: FOUND
- Commit 7898787: FOUND

---
*Phase: 08-leptos-web-shell-analysis-panels*
*Completed: 2026-03-17*
