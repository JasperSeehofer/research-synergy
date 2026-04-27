---
phase: 28-forward-citation-crawl-mode-s2
plan: "01"
subsystem: api
tags: [rust, semantic-scholar, reqwest, wiremock, pagination, citation-graph, bidirectional]

# Dependency graph
requires: []
provides:
  - "SemanticScholarSource.fetch_citing_papers_inner: paginates S2 /citations endpoint, enforces max_forward_citations cap, silently swallows 404"
  - "SemanticScholarSource builder fields: bidirectional (bool), max_forward_citations (usize) with with_bidirectional() and with_max_forward_citations() methods"
  - "S2CitationsPage / S2CitationItem deserializer structs mirroring S2RefsPage / S2RefItem"
  - "convert_s2_paper_to_ref private helper extracted from convert_s2_refs body (shared by references and citations converters)"
  - "5 wiremock integration tests covering happy path, 404 silent, pagination, cap, and disabled-mode short-circuit"
affects:
  - 28-forward-citation-crawl-mode-s2/28-02 (adds paper.citing_papers field + get_citing_arxiv_ids — required for tests to compile)
  - 28-forward-citation-crawl-mode-s2/28-03 (wires fetch_citing_papers_inner into PaperSource trait + DB persistence)
  - 28-forward-citation-crawl-mode-s2/28-04 (CLI flags + worker loop integration)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "fetch_citing_papers_inner: inherent method (not trait method) for wave-1 parallel safety — plan 03 delegates trait impl to this method"
    - "convert_s2_paper_to_ref: private helper shared by references and citations converters — eliminates code duplication"
    - "bidir_source_with test helper: mirrors source_with but sets bidirectional=true and configurable cap"
    - "TDD RED phase: tests written before Paper.citing_papers field exists; compile error is expected and gated by plan-02 merge"

key-files:
  created: []
  modified:
    - resyn-core/src/data_aggregation/semantic_scholar_api.rs
    - resyn-core/tests/semantic_scholar_integration.rs

key-decisions:
  - "fetch_citing_papers_inner named as inherent method (not fetch_citing_papers) to avoid name collision with plan-02 trait method before merge"
  - "paper.citing_papers assignment written unconditionally (not cfg-gated); wave-1 orchestrator merges plan-01 + plan-02 before any wave-2 work begins"
  - "convert_s2_paper_to_ref extracted as private helper per CONTEXT discretion bullet #1 — avoids duplicating S2Paper->Reference logic across references and citations converters"
  - "rate_limit_check() called inside fetch_citing_papers_inner (consistent with fetch_references behavior)"

patterns-established:
  - "S2 endpoint pagination pattern: loop with offset cursor, break on next: None or cap reached, truncate accumulator to cap"
  - "404 silent-skip: break from loop (not early return) so caller receives Ok(()) with empty result"

requirements-completed: []

# Metrics
duration: 11min
completed: 2026-04-27
---

# Phase 28 Plan 01: Forward-citation S2 plumbing Summary

**SemanticScholarSource extended with bidirectional builder fields, S2CitationsPage/S2CitationItem deserializers, convert_s2_paper_to_ref helper, and fetch_citing_papers_inner method paginating the S2 /citations endpoint with cap enforcement and 404 silent-skip**

## Performance

- **Duration:** 11 min
- **Started:** 2026-04-27T09:17:04Z
- **Completed:** 2026-04-27T09:28:10Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Extended `SemanticScholarSource` with `bidirectional: bool` and `max_forward_citations: usize` fields (defaults false/500), plus `with_bidirectional()` and `with_max_forward_citations()` builder methods
- Added `S2CitationsPage` / `S2CitationItem` deserializer structs mirroring the existing `S2RefsPage` / `S2RefItem` shape
- Extracted `convert_s2_paper_to_ref` private helper from `convert_s2_refs` body; added `convert_s2_citations` delegating to the same helper — zero logic duplication between references and citations converters
- Implemented `fetch_citing_papers_inner` as a `pub async fn` on the inherent impl: paginates `/paper/arXiv:{id}/citations`, enforces `max_forward_citations` cap, silently swallows 404
- Added 5 wiremock integration tests covering happy path (2 items, 1 arXiv-tagged), 404 silent, pagination (2 pages), cap truncation (5 items → 3), and disabled-mode short-circuit (no HTTP calls)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add builder fields, deserializers, and fetch_citing_papers_inner method** - `2469a34` (feat)
2. **Task 2: Add wiremock integration tests for /citations endpoint** - `e674d33` (test)

_Note: Both tasks are TDD. Task 1 = GREEN implementation; Task 2 = RED tests. Tests compile and pass after plan-02 adds `Paper::citing_papers` and `Paper::get_citing_arxiv_ids()`._

## Files Created/Modified

- `resyn-core/src/data_aggregation/semantic_scholar_api.rs` — +141 lines / -37 lines: struct fields, builder methods, deserializer structs, private helpers, `fetch_citing_papers_inner` method
- `resyn-core/tests/semantic_scholar_integration.rs` — +138 lines: `bidir_source_with` helper, 4 JSON fixtures, 5 async test functions

## Lines Added to semantic_scholar_api.rs

| Category | Lines |
|----------|-------|
| Struct fields (bidirectional, max_forward_citations) | +2 |
| Constructor initializers (new + from_env, both fields) | +4 |
| Builder methods (with_bidirectional, with_max_forward_citations) | +16 |
| S2CitationsPage + S2CitationItem structs | +12 |
| convert_s2_paper_to_ref helper + refactored convert_s2_refs | +32 |
| convert_s2_citations helper | +6 |
| fetch_citing_papers_inner method | +63 |
| Doc comments | +6 |

**convert_s2_paper_to_ref extracted:** YES — extracted as a `fn convert_s2_paper_to_ref(s2: &S2Paper) -> Reference` private helper on the inherent impl. `convert_s2_refs` now delegates to it; new `convert_s2_citations` also delegates to it.

## Decisions Made

- `fetch_citing_papers_inner` (not `fetch_citing_papers`) chosen as the inherent method name to avoid a name collision with the trait method that plan-03 adds; plan-03's trait impl delegates to this inner method
- `paper.citing_papers` assignment not cfg-gated: the wave-1 orchestrator merges plan-01 and plan-02 together before any wave-2 work begins, so the compile-error window is closed automatically
- `rate_limit_check()` called at the top of `fetch_citing_papers_inner` consistent with `fetch_references` behavior

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `cargo check -p resyn-core` (without `--features ssr`) reported no errors for `paper.citing_papers` because `reqwest` and the HTTP code paths are behind the `ssr` feature flag. The compile error only surfaces when running `cargo test` (which enables all features for the integration test crate). This is expected behavior — confirmed the RED state by running the integration test directly.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan-02 must add `Paper::citing_papers: Vec<Reference>` (with `#[serde(default, skip_serializing)]`) and `Paper::get_citing_arxiv_ids(&self) -> Vec<String>` before the 5 new integration tests compile
- Once plan-01 and plan-02 are merged, plan-03 can wire `fetch_citing_papers_inner` into the `PaperSource` trait default impl and add `upsert_inverse_citations_batch` to the database layer
- No blockers for plan-02 (independent parallel execution in wave-1)

---
*Phase: 28-forward-citation-crawl-mode-s2*
*Completed: 2026-04-27*

## Self-Check: PASSED

- `resyn-core/src/data_aggregation/semantic_scholar_api.rs` — exists, confirmed
- `resyn-core/tests/semantic_scholar_integration.rs` — exists, confirmed
- Task 1 commit `2469a34` — verified in git log
- Task 2 commit `e674d33` — verified in git log
