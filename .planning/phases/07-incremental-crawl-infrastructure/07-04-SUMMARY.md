---
phase: 07-incremental-crawl-infrastructure
plan: "04"
subsystem: data_aggregation
tags: [arxiv, html-parsing, url-construction, bug-fix, tdd]

# Dependency graph
requires:
  - phase: 07-incremental-crawl-infrastructure
    provides: arxiv_utils.rs with aggregate_references_for_arxiv_paper function
provides:
  - aggregate_references_for_arxiv_paper with empty pdf_url guard and ID-based fallback
  - Three new unit tests documenting and verifying fallback behavior
affects: [UAT Test 1, crawl pipeline, any caller of aggregate_references_for_arxiv_paper]

# Tech tracking
tech-stack:
  added: []
  patterns: [call-site guard over function-level guard — fallback uses paper.id available only at call site not inside converter]

key-files:
  created: []
  modified:
    - resyn-core/src/data_aggregation/arxiv_utils.rs

key-decisions:
  - "Fallback guard placed at call site in aggregate_references_for_arxiv_paper (not inside convert_pdf_url_to_html_url) — paper.id is only available at the call site"
  - "convert_pdf_url_to_html_url left unchanged — empty-in/empty-out is correct for its contract; callers are responsible for providing valid input"

patterns-established:
  - "ID-based HTML URL construction: https://arxiv.org/html/{paper.id} as canonical fallback"

requirements-completed: [CRAWL-01, CRAWL-02]

# Metrics
duration: 3min
completed: 2026-03-16
---

# Phase 7 Plan 04: Empty pdf_url Guard Summary

**ID-based fallback in `aggregate_references_for_arxiv_paper` prevents reqwest builder errors when arxiv-rs returns empty pdf_url, unblocking UAT Test 1 and the five dependent reference-fetching tests**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-16T18:17:41Z
- **Completed:** 2026-03-16T18:20:42Z
- **Tasks:** 1 (TDD: 2 commits — test + feat)
- **Files modified:** 1

## Accomplishments
- Eliminated silent reference-fetching failures caused by empty `paper.pdf_url` from arxiv-rs
- When `pdf_url` is empty, HTML URL is now constructed as `https://arxiv.org/html/{paper.id}`
- When `pdf_url` is non-empty, existing `convert_pdf_url_to_html_url` path is unchanged (no regression)
- Added 3 new unit tests (172 total passing, up from 169)

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Add failing tests for empty pdf_url fallback** - `c5a5df6` (test)
2. **Task 1 GREEN: Guard empty pdf_url with ID-based fallback** - `e0d6967` (feat)

_Note: TDD task had two commits (test → feat) as expected_

## Files Created/Modified
- `resyn-core/src/data_aggregation/arxiv_utils.rs` - Added 5-line guard in `aggregate_references_for_arxiv_paper` + 3 new unit tests

## Decisions Made
- Guard placed at call site (not inside `convert_pdf_url_to_html_url`) because `paper.id` is only available where `Paper` is in scope, not inside the URL converter
- `convert_pdf_url_to_html_url` left with empty-in/empty-out contract — the fix is a caller-level responsibility since the converter has no access to `paper.id`

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

One discovery during execution: the `data_aggregation` module is behind the `ssr` feature gate in `lib.rs`, so tests must be run with `--features ssr` to be included. The plan's verification command (`cargo test -p resyn-core -- arxiv_utils`) would silently find 0 tests without this flag. Used `cargo test -p resyn-core --features ssr` for targeted runs, `cargo test --all-features` for full suite verification.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- UAT Test 1 ("Application fetches seed paper and its references without errors") is now unblocked
- The five UAT tests that depend on Test 1 are now unblocked
- Gap closure for CRAWL-01 and CRAWL-02 requirements is complete
- Ready to advance to Phase 8

---
*Phase: 07-incremental-crawl-infrastructure*
*Completed: 2026-03-16*
