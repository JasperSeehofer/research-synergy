---
phase: 20-llm-analysis-pipeline-verification
plan: 04
subsystem: testing
tags: [integration-tests, analysis-pipeline, wiremock, ollama, axum, tfidf, llm]

# Dependency graph
requires:
  - phase: 20-llm-analysis-pipeline-verification plan: 01
    provides: StartAnalysis server function, Wave 0 test stubs
  - phase: 20-llm-analysis-pipeline-verification plan: 02
    provides: AnalysisControls UI component
  - phase: 20-llm-analysis-pipeline-verification plan: 03
    provides: result panel CTAs and SSE refetch
provides:
  - 5 passing integration tests for analysis pipeline
  - Feature-gated real Ollama test
  - Verified LLM-01 through LLM-04 programmatically
affects: []

# Tech tracking
tech-stack:
  added: [resyn-server lib target (lib.rs for test access), tower ServiceExt for in-process axum tests]
  patterns:
    - "Integration tests use connect_memory() + seed_test_db() for fully isolated in-memory DB"
    - "Wiremock MockServer for Ollama HTTP mock with OLLAMA_URL env var injection"
    - "In-process axum test via tower::ServiceExt::oneshot() without port binding"
    - "ServerFn::PATH constant for correct hash-suffixed Leptos 0.8 server function URL"
    - "resyn-server lib.rs enables integration tests to import run_analysis_pipeline directly"

key-files:
  created:
    - resyn-server/src/lib.rs
  modified:
    - resyn-server/tests/analysis_pipeline_test.rs
    - resyn-server/src/main.rs
    - resyn-server/Cargo.toml

key-decisions:
  - "Add resyn-server [lib] target to expose pub fns to integration tests — binaries have no crate for tests to import from"
  - "Use StartAnalysis::PATH (ServerFn trait constant) for axum test request URI — Leptos 0.8 generates hash-suffixed paths"
  - "VALID_ANNOTATION_JSON uses structured Method/Finding objects per LlmAnnotationRaw schema — plan's simplified schema would fail deserialization"

# Metrics
duration: 35min
completed: 2026-03-28
---

# Phase 20 Plan 04: Integration Tests for Analysis Pipeline Summary

**5 integration tests verify the analysis pipeline end-to-end: NLP-only, noop provider, wiremock Ollama, caching, and HTTP endpoint**

## Performance

- **Duration:** ~35 min
- **Completed:** 2026-03-28
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Wave 0 test stubs replaced: all 5 `#[ignore]` stubs removed, 5 full implementations added
- Added `resyn-server` lib target (`src/lib.rs`) enabling integration test access to `run_analysis_pipeline`
- Added `[features] ollama-test = []` to `resyn-server/Cargo.toml` per D-11
- Added `[dev-dependencies]` with wiremock, http, tower for integration tests
- **Test 1** `test_analysis_pipeline_noop_nlp_only`: NLP-only mode produces 3 TF-IDF analyses, 0 LLM annotations (LLM-01/D-07)
- **Test 2** `test_analysis_pipeline_noop_provider`: Noop provider produces 3 analyses + 3 annotations (LLM-01)
- **Test 3** `test_analysis_pipeline_wiremock_ollama`: Wiremock Ollama produces annotations, open_problems, method_matrix (LLM-02/03/04)
- **Test 4** `test_analysis_pipeline_caching`: Second run returns same 3 analyses, not 6 (D-02)
- **Test 5** `test_start_analysis_http`: POST to registered server fn path returns 2xx via in-process axum (D-10)
- **Test 6** `test_analysis_pipeline_real_ollama`: Feature-gated test behind `#[cfg(feature = "ollama-test")]` (D-11)
- Full test suite: 5/5 default tests pass, 0 failures, 0 ignored

## Task Commits

1. **Task 1: Cargo.toml** - `ae31453` (chore) — ollama-test feature + dev-dependencies
2. **Task 2: Integration tests** - `fbe89e6` (feat) — full test implementations + lib.rs

## Files Created/Modified

- `resyn-server/src/lib.rs` — New: `pub mod commands;` library entry for integration test access
- `resyn-server/tests/analysis_pipeline_test.rs` — Replaced Wave 0 stubs with 5 full implementations + 1 feature-gated test
- `resyn-server/src/main.rs` — Updated to use `resyn_server::commands` after lib.rs added
- `resyn-server/Cargo.toml` — Added `[features]`, `[lib]`, `[dev-dependencies]`

## Decisions Made

- **resyn-server lib target required** — Rust integration tests in `tests/` can only import from library crates, not binaries. Adding `src/lib.rs` with `pub mod commands` exposes `run_analysis_pipeline` to the test binary without code duplication.

- **StartAnalysis::PATH for axum test URL** — Leptos 0.8 generates hash-suffixed server function paths (e.g. `/api/start_analysis1516452828643397131`). Using `StartAnalysis::PATH` (via `ServerFn` trait) ensures the test uses the exact registered path. Using `/api/StartAnalysis` returns 400 "Could not find server function".

- **VALID_ANNOTATION_JSON structure** — The plan specified a simplified JSON shape (`methods: ["Monte Carlo simulation"]`), but `LlmAnnotationRaw` deserializes `methods` as `Vec<Method>` (objects with `name`/`category` fields) and `findings` as `Vec<Finding>` (objects with `text`/`strength`). Used the structured format matching `LlmAnnotationRaw` to avoid parse failure.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing resyn-server library target for integration tests**
- **Found during:** Task 2
- **Issue:** `use resyn_server::commands::analyze::run_analysis_pipeline` failed with "use of unresolved module or unlinked crate `resyn_server`" — binary crates have no importable crate root for integration tests
- **Fix:** Added `[lib]` section to Cargo.toml with `src/lib.rs`, updated `main.rs` to use `resyn_server::commands`
- **Files modified:** `resyn-server/src/lib.rs` (created), `resyn-server/src/main.rs`, `resyn-server/Cargo.toml`
- **Commit:** `fbe89e6`

**2. [Rule 1 - Bug] Leptos 0.8 hash-suffixed server function paths**
- **Found during:** Task 2, test_start_analysis_http
- **Issue:** POST to `/api/StartAnalysis` returned 400 "Could not find server function" — Leptos 0.8 registers functions at hash-suffixed paths
- **Fix:** Use `StartAnalysis::PATH` (from `ServerFn` trait) to obtain the exact registered path at compile time
- **Files modified:** `resyn-server/tests/analysis_pipeline_test.rs`
- **Commit:** `fbe89e6`

**3. [Rule 1 - Bug] VALID_ANNOTATION_JSON schema mismatch**
- **Found during:** Task 2, test_analysis_pipeline_wiremock_ollama design
- **Issue:** Plan's simplified annotation JSON (`"methods": ["Monte Carlo simulation"]`) doesn't match `LlmAnnotationRaw` which expects `Vec<Method>` (objects with `name`/`category`)
- **Fix:** Used the structured JSON format from existing ollama unit tests in `resyn-core/src/llm/ollama.rs`
- **Files modified:** `resyn-server/tests/analysis_pipeline_test.rs`

## Known Stubs

None — all tests are fully implemented with real assertions. The only non-running test is `test_analysis_pipeline_real_ollama`, which is intentionally feature-gated behind `ollama-test` and requires a running Ollama instance.

## Self-Check: PASSED
