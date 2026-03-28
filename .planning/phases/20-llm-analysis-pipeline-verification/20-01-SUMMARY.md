---
phase: 20-llm-analysis-pipeline-verification
plan: 01
subsystem: api
tags: [leptos, axum, server-fn, sse, progress-events, analysis-pipeline, tfidf, llm]

# Dependency graph
requires:
  - phase: 19-data-quality-cleanup
    provides: clean paper data in SurrealDB, empty-string arXiv ID filter
provides:
  - StartAnalysis Leptos server function at /api/StartAnalysis
  - Extended ProgressEvent with analysis_stage field (backwards-compatible)
  - Analysis pipeline callable from web context without process::exit
  - Wave 0 test stubs for Plan 04 (5 ignored tests defining behavioral contracts)
affects: [20-02, 20-03, 20-04]

# Tech tracking
tech-stack:
  added: [chrono (optional SSR dep in resyn-app)]
  patterns:
    - "Analysis server function mirrors StartCrawl pattern: use_context for db/tx, tokio::spawn background task, broadcast ProgressEvent"
    - "Pipeline stages broadcast analysis_extracting/nlp/llm/gaps/complete/error events on the shared SSE channel"
    - "std::process::exit replaced with anyhow::Result propagation in web-callable pipeline functions"
    - "Wave 0 test stubs: ignored tests with exact function names that Plan 04 will un-ignore and implement"

key-files:
  created:
    - resyn-app/src/server_fns/analysis.rs
    - resyn-server/tests/analysis_pipeline_test.rs
  modified:
    - resyn-core/src/datamodels/progress.rs
    - resyn-app/src/server_fns/mod.rs
    - resyn-app/Cargo.toml
    - resyn-server/src/commands/analyze.rs
    - resyn-server/src/commands/crawl.rs
    - resyn-app/src/components/crawl_progress.rs
    - resyn-server/src/commands/serve.rs

key-decisions:
  - "StartAnalysis inlines pipeline logic using resyn-core directly rather than calling resyn-server::commands::analyze — avoids circular dependency (resyn-app cannot depend on resyn-server)"
  - "chrono added as optional SSR dep in resyn-app rather than workspace-wide — keeps WASM build clean"
  - "run_extraction and run_llm_analysis now return anyhow::Result<()>; CLI run() retains process::exit for user-facing error messages"
  - "analysis_stage field uses #[serde(default)] for backwards compatibility with existing serialized crawl events"

patterns-established:
  - "Analysis event_type pattern: analysis_{stage} prefix (analysis_extracting, analysis_nlp, analysis_llm, analysis_gaps, analysis_complete, analysis_error)"
  - "Wave 0 stubs: create ignored test file in target package early; Plan 04 un-ignores them"

requirements-completed: [LLM-01]

# Metrics
duration: 6min
completed: 2026-03-28
---

# Phase 20 Plan 01: LLM Analysis Pipeline Backend Summary

**StartAnalysis Leptos server function wired to resyn-core analysis pipeline with SSE progress events and Wave 0 test stubs**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-28T21:52:38Z
- **Completed:** 2026-03-28T21:58:22Z
- **Tasks:** 3 (Task 0, 1, 2)
- **Files modified:** 9

## Accomplishments

- Wave 0 test stubs created: 5 ignored tests in `resyn-server/tests/analysis_pipeline_test.rs` defining behavioral contracts for Plan 04
- `ProgressEvent` extended with `analysis_stage: Option<String>` field (`#[serde(default)]` for backwards compatibility)
- `StartAnalysis` Leptos server function created: spawns background analysis task, reads `RESYN_LLM_PROVIDER` env var, broadcasts stage events via SSE
- `std::process::exit` removed from web-callable pipeline functions (`run_extraction`, `run_llm_analysis`, `run_analysis_pipeline`)
- `StartAnalysis` registered in `serve.rs` at `/api/StartAnalysis`
- Full test suite passes: 268 tests, 0 failures, 5 ignored (the Wave 0 stubs)

## Task Commits

1. **Task 0: Wave 0 test stubs** - `d0c16df` (test)
2. **Task 1: ProgressEvent + StartAnalysis** - `bae1bdf` (feat)
3. **Task 2: Register StartAnalysis** - `fef67bf` (feat)

## Files Created/Modified

- `resyn-server/tests/analysis_pipeline_test.rs` — 5 ignored test stubs for Plan 04 behavioral contracts
- `resyn-core/src/datamodels/progress.rs` — Added `analysis_stage: Option<String>` with `#[serde(default)]`
- `resyn-app/src/server_fns/analysis.rs` — New: StartAnalysis server function with 4-stage pipeline background task
- `resyn-app/src/server_fns/mod.rs` — Added `pub mod analysis;`
- `resyn-app/Cargo.toml` — Added `chrono` as optional SSR dep
- `resyn-server/src/commands/analyze.rs` — `run_extraction` and `run_llm_analysis` return `anyhow::Result<()>`, `run_analysis_pipeline` propagates errors with `?`
- `resyn-server/src/commands/crawl.rs` — Added `analysis_stage: None` to 3 `ProgressEvent` struct literals
- `resyn-app/src/components/crawl_progress.rs` — Added `analysis_stage: None` to default `ProgressEvent` fallback
- `resyn-server/src/commands/serve.rs` — Registered `StartAnalysis` via `register_explicit`

## Decisions Made

- **StartAnalysis uses resyn-core directly** — The plan specified calling `resyn_server::commands::analyze::run_analysis_pipeline` from within `resyn-app`, but this would create a circular dependency (`resyn-app` ← `resyn-server` ← `resyn-app`). The server function inlines equivalent logic using `resyn-core` APIs. The pipeline stages (`run_extraction`, `run_nlp_analysis`, `run_llm_analysis`, `run_gap_analysis`) all use `resyn_core::*` exclusively, so this is straightforward.

- **chrono as optional SSR dep** — `chrono::Utc::now().to_rfc3339()` needed for `PaperAnalysis` and `AnalysisMetadata` timestamps. Added as optional dep activated only by the `ssr` feature to keep the WASM build clean.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Plan referenced resyn_server crate from resyn-app (circular dependency)**
- **Found during:** Task 1 (StartAnalysis server function)
- **Issue:** Plan specified `resyn_server::commands::analyze::run_analysis_pipeline` in the server function, but `resyn-app` cannot depend on `resyn-server` (it's the reverse: `resyn-server` depends on `resyn-app`)
- **Fix:** Inlined the pipeline logic in the server function using `resyn-core` APIs directly, mirroring what `run_analysis_pipeline` does. All the actual analysis work uses `resyn_core::*` exclusively.
- **Files modified:** `resyn-app/src/server_fns/analysis.rs`
- **Verification:** `cargo check --workspace` exits 0
- **Committed in:** `bae1bdf` (Task 1 commit)

**2. [Rule 3 - Blocking] Missing chrono dependency in resyn-app**
- **Found during:** Task 1 (StartAnalysis server function)
- **Issue:** `chrono::Utc::now().to_rfc3339()` used in analysis.rs but `chrono` not in resyn-app deps
- **Fix:** Added `chrono = { workspace = true, optional = true }` to `resyn-app/Cargo.toml`, activated via `ssr` feature
- **Files modified:** `resyn-app/Cargo.toml`
- **Verification:** `cargo check --workspace` exits 0
- **Committed in:** `bae1bdf` (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (1 bug: circular dep, 1 blocking: missing dep)
**Impact on plan:** Both auto-fixes necessary for compilation. The circular dep fix is architecturally correct — the inline approach is equivalent to calling `run_analysis_pipeline` and cleaner for the web context (no CLI-specific AnalyzeArgs struct needed).

## Issues Encountered

None beyond the auto-fixed deviations above.

## Known Stubs

None — all fields are wired to real data sources. The Wave 0 test stubs are intentional ignored tests, not data stubs.

## Next Phase Readiness

- `StartAnalysis` endpoint is live at `/api/StartAnalysis`
- SSE progress stream will emit `analysis_extracting`, `analysis_nlp`, `analysis_llm`, `analysis_gaps`, `analysis_complete` events
- Plan 02 can now add the "Analyze" button to the UI and subscribe to these events
- Plan 04 can un-ignore the Wave 0 stubs and implement them

---
*Phase: 20-llm-analysis-pipeline-verification*
*Completed: 2026-03-28*
