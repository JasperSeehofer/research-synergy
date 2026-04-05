---
phase: 20-llm-analysis-pipeline-verification
plan: 02
subsystem: ui
tags: [leptos, components, sse, analysis-pipeline, css, dashboard, sidebar]

# Dependency graph
requires:
  - phase: 20-01
    provides: StartAnalysis server function, CheckLlmConfigured server function, analysis SSE events
provides:
  - AnalysisControls Leptos component with Run Analysis button and LLM warning banner
  - Analysis progress display in sidebar (Extracting text / Running TF-IDF / Annotating papers / Finding gaps)
  - Post-crawl inline prompt "Analysis available — Run now" in sidebar footer
  - CSS classes: warning-banner, analysis-prompt, analysis-prompt-btn, analysis-stage-label
affects: [20-03, 20-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "AnalysisControls uses its own SSE subscription — multiple lightweight connections to the same broadcast are fine in Leptos"
    - "is_analysis_running derived signal: starts_with('analysis_') && != analysis_complete && != analysis_error"
    - "Post-crawl prompt gated on event_type == 'complete' — hides automatically when analysis SSE events begin"
    - "CheckLlmConfigured loaded as Resource with () key — fetches once on mount"

key-files:
  created:
    - resyn-app/src/components/analysis_controls.rs
  modified:
    - resyn-app/src/server_fns/analysis.rs
    - resyn-app/src/components/mod.rs
    - resyn-app/src/pages/dashboard.rs
    - resyn-app/style/main.css
    - resyn-server/src/commands/serve.rs
    - resyn-app/src/components/crawl_progress.rs

key-decisions:
  - "AnalysisControls has its own SSE subscription rather than receiving last_event as a prop — self-contained component, no prop drilling needed"
  - "Analysis branch checks is_analysis_running() before is_running() in CrawlProgress — analysis events take priority in the sidebar display"

patterns-established:
  - "analysis_controls.rs pattern: own SSE subscription, check_llm_configured Resource, disabled button during analysis"
  - "CrawlProgress if-chain: collapsed → is_analysis_running → is_running → idle"

requirements-completed: [LLM-01]

# Metrics
duration: 7min
completed: 2026-03-28
---

# Phase 20 Plan 02: Analysis UI Controls Summary

**Analysis trigger UI: Run Analysis button on dashboard, analysis progress in sidebar, post-crawl prompt, LLM warning banner**

## Performance

- **Duration:** ~7 min
- **Started:** 2026-03-28T21:59:44Z
- **Completed:** 2026-03-28T22:06:04Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- `AnalysisControls` component created: LLM warning banner + Run Analysis button with disabled state during analysis
- `check_llm_configured` server function added to `server_fns/analysis.rs` and registered in `serve.rs`
- `AnalysisControls` wired into Dashboard below summary cards
- CSS added: `.warning-banner` (amber left border, muted background), `.analysis-prompt`, `.analysis-prompt-btn`, `.analysis-stage-label`
- `CrawlProgress` extended: analysis progress branch with stage labels (Extracting text / Running TF-IDF / Annotating papers / Finding gaps)
- Post-crawl inline prompt added: "Analysis available — Run now" shown after `event_type == "complete"` crawl event
- Full workspace compiles: `cargo check --workspace` exits 0

## Task Commits

1. **Task 1: AnalysisControls component and dashboard wiring** — `5167ed5` (feat)
2. **Task 2: Analysis progress and post-crawl prompt in sidebar** — `900d8b1` (feat)

## Files Created/Modified

- `resyn-app/src/components/analysis_controls.rs` — New: AnalysisControls with LLM warning banner and Run Analysis button
- `resyn-app/src/server_fns/analysis.rs` — Added `check_llm_configured` server function
- `resyn-app/src/components/mod.rs` — Added `pub mod analysis_controls;`
- `resyn-app/src/pages/dashboard.rs` — Imported and rendered `<AnalysisControls/>` after DashboardCards
- `resyn-app/style/main.css` — Added `.warning-banner`, `.analysis-prompt`, `.analysis-prompt-btn`, `.analysis-stage-label`
- `resyn-server/src/commands/serve.rs` — Added `register_explicit::<analysis::CheckLlmConfigured>()`
- `resyn-app/src/components/crawl_progress.rs` — Added `is_analysis_running`, analysis progress branch, post-crawl prompt

## Decisions Made

- **AnalysisControls has its own SSE subscription** — The plan noted that receiving `last_event` as a prop was also viable, but the own-subscription approach keeps the component self-contained and avoids prop threading through the sidebar layout.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Known Stubs

None — all components are wired to real server functions and SSE events. The analysis progress bar uses `width:50%` as a fixed value because the `analysis_*` events do not carry completion percentage data (papers_found / papers_pending are the available metrics, shown as text).

## Next Phase Readiness

- Plan 03 can add empty-state CTAs to gaps/problems/methods panels
- Plan 04 can un-ignore Wave 0 test stubs and implement behavioral tests
- The full UI flow is now: crawl → "Analysis available — Run now" prompt → Run Analysis button → sidebar shows analysis progress → panels auto-refetch on `analysis_complete`

---
*Phase: 20-llm-analysis-pipeline-verification*
*Completed: 2026-03-28*
