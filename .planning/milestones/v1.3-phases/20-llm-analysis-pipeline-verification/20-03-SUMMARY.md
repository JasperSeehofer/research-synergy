---
phase: 20-llm-analysis-pipeline-verification
plan: 03
subsystem: ui
tags: [leptos, sse, use_event_source, analysis-pipeline, empty-state, refetch]

# Dependency graph
requires:
  - phase: 20-llm-analysis-pipeline-verification plan 01
    provides: StartAnalysis server function at /api/StartAnalysis, analysis_complete SSE event type
provides:
  - Gap findings panel with CTA empty state and SSE auto-refetch
  - Open problems panel with CTA empty state and SSE auto-refetch
  - Methods panel with CTA empty state and SSE auto-refetch
affects: [20-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "SSE refetch pattern: use_event_source on /progress, Effect checks event_type == 'analysis_complete', calls resource.refetch()"
    - "CTA empty state: empty-state-heading + empty-state-body + btn-primary Run Analysis button via Action"
    - "Action::new wraps start_analysis server fn; on:click handler uses block form { action.dispatch(()); } to discard ActionAbortHandle"

key-files:
  created: []
  modified:
    - resyn-app/src/pages/gaps.rs
    - resyn-app/src/pages/open_problems.rs
    - resyn-app/src/pages/methods.rs

key-decisions:
  - "on:click handler uses block form { action.dispatch(()); } not expression form — Rust requires discarding ActionAbortHandle return value"
  - "analysis_action for methods.rs defined at MethodsPanel top-level (not inside Ok branch) so it can be captured by the empty state closure"

patterns-established:
  - "CTA empty-state pattern for result panels: use_event_source + Effect refetch + Action wrapping start_analysis"

requirements-completed: [LLM-02, LLM-03, LLM-04]

# Metrics
duration: 5min
completed: 2026-03-28
---

# Phase 20 Plan 03: Result Panel CTA Empty States and SSE Refetch Summary

**Three result panels (gaps, open problems, methods) now show guided CTA empty states and auto-refetch on analysis_complete SSE event**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-28T21:59:44Z
- **Completed:** 2026-03-28T22:04:52Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- All three result panels subscribe to `/progress` SSE and call `resource.refetch()` when `event_type == "analysis_complete"`
- Empty states replaced with CTA design: heading "No analysis results yet" + body copy per UI-SPEC + "Run Analysis" btn-primary button
- `start_analysis()` server function dispatched directly from each panel's empty state button
- Plan 01 merge brought worktree up to date (fast-forward from 9dc6bf2 to 376ac0c)

## Task Commits

1. **Task 1: Update gaps panel with CTA empty state and SSE refetch** - `3362da8` (feat)
2. **Task 2: Update open problems and methods panels with CTA empty state and SSE refetch** - `9d61de9` (feat)

## Files Created/Modified

- `resyn-app/src/pages/gaps.rs` — Added SSE subscription, refetch on analysis_complete, CTA empty state replacing generic empty state
- `resyn-app/src/pages/open_problems.rs` — Added SSE subscription, refetch on analysis_complete, CTA empty state with exact UI-SPEC copy
- `resyn-app/src/pages/methods.rs` — Added SSE subscription, refetch on analysis_complete, CTA empty state for overview heatmap; analysis_action hoisted to component top-level

## Decisions Made

- **Block form for on:click dispatch** — `Action::dispatch()` returns `ActionAbortHandle`. Leptos on:click expects `()` as expression result, so the handler must use `{ action.dispatch(()); }` block form to discard the handle. This is a Rust type system requirement, not a logic choice.

- **analysis_action placement in methods.rs** — Created at the `MethodsPanel` component level (before the drilldown conditional) rather than inside the `Ok(matrix)` branch. This ensures the closure can be captured by the empty state view without ownership issues.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ActionAbortHandle type mismatch in on:click handler**
- **Found during:** Task 1 (gaps panel)
- **Issue:** `on:click=move |_| analysis_action.dispatch(())` fails because `dispatch()` returns `ActionAbortHandle`, not `()`. Leptos event handler closure must return `()`.
- **Fix:** Changed to block form `on:click=move |_| { analysis_action.dispatch(()); }` across all three files
- **Files modified:** resyn-app/src/pages/gaps.rs, resyn-app/src/pages/open_problems.rs, resyn-app/src/pages/methods.rs
- **Verification:** `cargo check --workspace` exits 0
- **Committed in:** 3362da8, 9d61de9

---

**Total deviations:** 1 auto-fixed (Rule 1 bug: type mismatch in dispatch call)
**Impact on plan:** Minor fix required for compilation. The block form is the correct idiomatic Rust pattern when return value is not needed.

## Issues Encountered

- Worktree was branched before phase 20 commits — merged main (fast-forward) to get plan 01 work (`resyn-app/src/server_fns/analysis.rs`, extended `ProgressEvent`, updated `mod.rs`)

## Known Stubs

None — all three panels are wired to real SSE endpoint and real `start_analysis()` server function.

## Next Phase Readiness

- All three result panels now auto-refresh when analysis completes
- D-04 and D-05 defects resolved: panels no longer show stale empty states
- Plan 04 (integration tests and E2E verification) can now test the full pipeline end-to-end

---
*Phase: 20-llm-analysis-pipeline-verification*
*Completed: 2026-03-28*
