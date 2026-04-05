---
phase: 20-llm-analysis-pipeline-verification
verified: 2026-03-28T22:30:00Z
status: passed
score: 10/10 must-haves verified
re_verification: false
---

# Phase 20: LLM Analysis Pipeline Verification Report

**Phase Goal:** Users can trigger LLM analysis from the web UI and view all analysis results (gap findings, open problems, method heatmap) populated with real data
**Verified:** 2026-03-28
**Status:** PASSED
**Re-verification:** No — initial verification (previous verification file did not exist; note in prompt acknowledged a prior informal check)

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|---------|
| 1  | StartAnalysis server function exists and is registered at /api | VERIFIED | `resyn-app/src/server_fns/analysis.rs` l.30 `#[server(StartAnalysis, "/api")]`; registered in `serve.rs` l.53 |
| 2  | Calling /api/StartAnalysis returns 200 immediately while analysis runs in background | VERIFIED | `test_start_analysis_http` passes — 1/1 test ok in 0.27s |
| 3  | SSE /progress emits analysis stage events while analysis runs | VERIFIED | `analysis.rs` sends `analysis_extracting`, `analysis_nlp`, `analysis_llm`, `analysis_gaps`, `analysis_complete`, `analysis_error` via broadcast channel |
| 4  | NLP-only mode (RESYN_LLM_PROVIDER unset) produces TF-IDF results and completes without error | VERIFIED | `test_analysis_pipeline_noop_nlp_only` passes: 3 analyses, 0 annotations |
| 5  | Dashboard shows Run Analysis button wired to start_analysis() | VERIFIED | `dashboard.rs` l.39 renders `<AnalysisControls/>`; `analysis_controls.rs` l.45 dispatches `start_analysis()` |
| 6  | LLM warning banner shown when RESYN_LLM_PROVIDER is not set | VERIFIED | `analysis_controls.rs` l.54-57 renders `.warning-banner` div when `check_llm_configured()` returns `false` |
| 7  | Gap findings panel shows CTA empty state and auto-refetches on analysis_complete | VERIFIED | `gaps.rs` l.15-24 SSE subscription + `findings.refetch()`; l.121-138 CTA with exact UI-SPEC copy |
| 8  | Open problems panel shows CTA empty state and auto-refetches on analysis_complete | VERIFIED | `open_problems.rs` l.14-23 SSE subscription + `problems.refetch()`; l.46-63 CTA |
| 9  | Methods panel shows CTA empty state and auto-refetches on analysis_complete | VERIFIED | `methods.rs` l.15-24 SSE subscription + `matrix_resource.refetch()`; l.110-122 CTA |
| 10 | Automated tests verify pipeline produces gap findings, open problems, and method matrix with wiremock Ollama | VERIFIED | `test_analysis_pipeline_wiremock_ollama` passes: 3 annotations, non-empty open_problems, non-empty method_matrix.categories |

**Score:** 10/10 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `resyn-app/src/server_fns/analysis.rs` | StartAnalysis + CheckLlmConfigured server functions | VERIFIED | Both functions present, 340 lines, real pipeline logic wired |
| `resyn-core/src/datamodels/progress.rs` | ProgressEvent with analysis_stage field | VERIFIED | `analysis_stage: Option<String>` at l.19 with `#[serde(default)]` |
| `resyn-app/src/components/analysis_controls.rs` | AnalysisControls component with Run Analysis button and LLM warning banner | VERIFIED | 89 lines, `AnalysisControls` component, `warning-banner`, `btn-primary`, `start_analysis` call |
| `resyn-app/src/pages/dashboard.rs` | Dashboard renders AnalysisControls | VERIFIED | l.39 `<AnalysisControls/>` inside Ok(s) arm |
| `resyn-app/src/components/crawl_progress.rs` | Analysis progress display and post-crawl prompt | VERIFIED | l.49-59 `is_analysis_running`, l.97-128 analysis progress branch, l.198-213 post-crawl prompt |
| `resyn-app/src/pages/gaps.rs` | CTA empty state + SSE refetch | VERIFIED | l.15-24 SSE + refetch, l.121-138 CTA with "No analysis results yet" / "Run analysis to see gap findings here." |
| `resyn-app/src/pages/open_problems.rs` | CTA empty state + SSE refetch | VERIFIED | l.14-23 SSE + refetch, l.46-63 CTA with exact UI-SPEC copy |
| `resyn-app/src/pages/methods.rs` | CTA empty state + SSE refetch | VERIFIED | l.15-24 SSE + refetch, l.110-122 CTA with "Run analysis to see the method heatmap here." |
| `resyn-server/tests/analysis_pipeline_test.rs` | 5 passing integration tests + 1 feature-gated | VERIFIED | 5/5 tests pass; `#[cfg(feature = "ollama-test")]` gates real Ollama test |
| `resyn-server/Cargo.toml` | ollama-test feature flag | VERIFIED | `[features] ollama-test = []` |
| `resyn-server/src/lib.rs` | Library target for test imports | VERIFIED | Exists, exposes `pub mod commands` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `analysis_controls.rs` | `server_fns/analysis.rs` | `start_analysis()` call | VERIFIED | l.6 import, l.45 dispatch |
| `analysis_controls.rs` | `server_fns/analysis.rs` | `check_llm_configured()` resource | VERIFIED | l.6 import, l.17 Resource::new |
| `dashboard.rs` | `analysis_controls.rs` | `<AnalysisControls/>` render | VERIFIED | l.3 import, l.39 render |
| `gaps.rs` | `/progress` SSE | `use_event_source` + `findings.refetch()` on `analysis_complete` | VERIFIED | l.15-24 |
| `open_problems.rs` | `/progress` SSE | `use_event_source` + `problems.refetch()` on `analysis_complete` | VERIFIED | l.14-23 |
| `methods.rs` | `/progress` SSE | `use_event_source` + `matrix_resource.refetch()` on `analysis_complete` | VERIFIED | l.15-24 |
| `gaps.rs` | `server_fns/analysis.rs` | `start_analysis()` from CTA button | VERIFIED | l.123 `crate::server_fns::analysis::start_analysis()` |
| `open_problems.rs` | `server_fns/analysis.rs` | `start_analysis()` from CTA button | VERIFIED | l.48 `crate::server_fns::analysis::start_analysis()` |
| `methods.rs` | `server_fns/analysis.rs` | `start_analysis()` from CTA button | VERIFIED | l.27 `crate::server_fns::analysis::start_analysis()` |
| `analysis.rs` (server fn) | broadcast channel | `use_context::<broadcast::Sender<ProgressEvent>>()` | VERIFIED | l.41-42 |
| `serve.rs` | `analysis::StartAnalysis` | `register_explicit::<analysis::StartAnalysis>()` | VERIFIED | serve.rs l.53 |
| `serve.rs` | `analysis::CheckLlmConfigured` | `register_explicit::<analysis::CheckLlmConfigured>()` | VERIFIED | serve.rs l.54 |
| `analysis_pipeline_test.rs` | `run_analysis_pipeline` | direct call via resyn-server lib target | VERIFIED | l.9 import, l.96/134/208 calls |
| `test_start_analysis_http` | `/api/StartAnalysis` (hash-suffixed) | `tower::ServiceExt::oneshot()` in-process axum | VERIFIED | l.346 oneshot, test passes |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `gaps.rs` | `findings` (Resource) | `get_gap_findings()` server fn → `GapFindingRepository::get_all_gap_findings()` | Yes — DB query | FLOWING |
| `open_problems.rs` | `problems` (Resource) | `get_open_problems_ranked()` server fn → `LlmAnnotationRepository` + `aggregate_open_problems` | Yes — DB query + aggregation | FLOWING |
| `methods.rs` | `matrix_resource` (Resource) | `get_method_matrix()` server fn → `LlmAnnotationRepository` + `build_method_matrix` | Yes — DB query + aggregation | FLOWING |
| `analysis_controls.rs` | `llm_configured` (Resource) | `check_llm_configured()` → `std::env::var("RESYN_LLM_PROVIDER")` | Yes — live env var | FLOWING |
| `analysis.rs` (server fn) | pipeline stages | `PaperRepository::get_all_papers()`, `ExtractionRepository`, `AnalysisRepository`, `LlmAnnotationRepository`, `GapFindingRepository` | Yes — all DB queries, no static returns | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| NLP-only pipeline completes, produces 3 TF-IDF analyses | `cargo test --package resyn-server test_analysis_pipeline_noop_nlp_only` | ok (0.34s) | PASS |
| Noop provider pipeline produces 3 analyses + 3 annotations | `cargo test --package resyn-server test_analysis_pipeline_noop_provider` | ok | PASS |
| Wiremock Ollama pipeline produces annotations + open problems + method matrix | `cargo test --package resyn-server test_analysis_pipeline_wiremock_ollama` | ok | PASS |
| Caching: second run leaves analysis count at 3 (not 6) | `cargo test --package resyn-server test_analysis_pipeline_caching` | ok | PASS |
| POST /api/StartAnalysis returns 2xx via in-process axum | `cargo test --package resyn-server test_start_analysis_http` | ok (0.27s) | PASS |
| Full workspace test suite — no regressions | `cargo test --workspace` | 288 passed, 0 failed, 0 ignored | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| LLM-01 | Plans 01, 02, 04 | User can trigger LLM analysis from web UI and see it complete | SATISFIED | `StartAnalysis` server fn registered; button wired in dashboard; 5 integration tests pass including NLP-only and wiremock Ollama |
| LLM-02 | Plans 03, 04 | User can view gap findings (contradictions, ABC-bridges) after analysis | SATISFIED | `gaps.rs` CTA empty state + SSE refetch; `test_analysis_pipeline_wiremock_ollama` queries gap findings without error |
| LLM-03 | Plans 03, 04 | User can view open problems panel with results ranked by recurrence | SATISFIED | `open_problems.rs` CTA + refetch; wiremock test asserts `aggregate_open_problems` non-empty |
| LLM-04 | Plans 03, 04 | User can view method heatmap showing existing vs absent pairings | SATISFIED | `methods.rs` CTA + refetch; wiremock test asserts `build_method_matrix.categories` non-empty |

All four required IDs (LLM-01 through LLM-04) are covered. No orphaned requirements found — REQUIREMENTS.md marks all four as Phase 20 / Complete.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `crawl_progress.rs` | 124 | `width:50%` hardcoded in analysis progress bar | Info | Analysis progress bar does not animate based on actual completion percentage — acceptable per SUMMARY note: analysis_* events do not carry a completion percentage, so fixed 50% is a documented limitation, not a stub |
| `analyze.rs` | 49 | `std::process::exit(1)` in CLI `run()` function | Info | Confirmed this is in the CLI-only `run()` path which is only called from `main.rs`, not from any web-callable function. All web-callable functions (`run_analysis_pipeline`, `run_extraction`, `run_llm_analysis`) return `anyhow::Result` — no `process::exit` in those paths |

No blockers found. One informational note on the progress bar, one informational note confirming the sole remaining `process::exit` is CLI-only.

### Human Verification Required

#### 1. LLM Warning Banner Visibility

**Test:** Run `resyn-server` without `RESYN_LLM_PROVIDER` set; navigate to Dashboard
**Expected:** Amber warning banner appears above the Run Analysis button: "LLM provider not configured — showing NLP-only results. Set RESYN_LLM_PROVIDER for full analysis."
**Why human:** Server-side env var check + SSR render requires a running server to verify visually

#### 2. Run Analysis Button Disabled State

**Test:** Click Run Analysis; observe button state while SSE events arrive
**Expected:** Button reads "Analysis running..." and is disabled during `analysis_extracting` through `analysis_gaps` events; re-enables on `analysis_complete` or `analysis_error`
**Why human:** Real-time reactive UI state requires a browser with a live SSE stream to verify

#### 3. Post-Crawl Prompt in Sidebar

**Test:** Run a crawl to completion; observe sidebar footer
**Expected:** "Analysis available — Run now" link appears below the crawl summary; clicking "Run now" triggers analysis
**Why human:** Requires a running server receiving real SSE `complete` event

#### 4. Panels Auto-Refresh After Analysis

**Test:** With empty gap/problems/methods panels, run analysis; observe panels
**Expected:** After `analysis_complete` SSE event, all three panels automatically refetch and display populated data without a page reload
**Why human:** Requires live SSE stream with data in DB to observe reactive refetch

### Gaps Summary

No gaps found. All automated checks pass. The phase goal is fully achieved:

- The backend (`StartAnalysis` server function) is wired, registered, and proven by 5 integration tests including in-process HTTP verification.
- The UI entry points (dashboard Run Analysis button, LLM warning banner, sidebar analysis progress, post-crawl prompt) are all substantive — not placeholder stubs.
- All three result panels (gaps, open problems, methods) have SSE-triggered auto-refetch wired to `analysis_complete` and CTA empty states with the exact copywriting from the UI-SPEC.
- The `process::exit` removal from web-callable pipeline paths is confirmed. The one remaining `exit` call is in the CLI-only `run()` function.
- Full workspace: 288 tests, 0 failures.

---

_Verified: 2026-03-28_
_Verifier: Claude (gsd-verifier)_
