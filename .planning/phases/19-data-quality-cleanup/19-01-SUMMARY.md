---
phase: 19-data-quality-cleanup
plan: "01"
subsystem: data-pipeline
tags: [inspirehep, data-quality, orphan-nodes, published-dates, bug-fix]
dependency_graph:
  requires: []
  provides: [published-dates-from-inspirehep, empty-id-filter-in-bfs]
  affects: [temporal-slider, bfs-crawl-queue, graph-node-count]
tech_stack:
  added: []
  patterns: [tdd-red-green, chained-optional-unwrap-or-default, tracing-warn-on-filter]
key_files:
  created: []
  modified:
    - resyn-core/src/data_aggregation/inspirehep_api.rs
    - resyn-core/src/datamodels/paper.rs
decisions:
  - "Filter at source in get_arxiv_references_ids() rather than at BFS queue ingestion — single responsibility, easier to test"
  - "Use tracing::warn for filtered empty IDs so operators can observe orphan-prevention at runtime"
  - "earliest_date added to both fetch_paper() and fetch_literature() URL field params for consistency"
metrics:
  duration: "5min"
  completed: "2026-03-28"
  tasks_completed: 2
  files_modified: 2
requirements_satisfied: [ARXIV-02, ORPH-01, ORPH-02]
---

# Phase 19 Plan 01: Data Quality Cleanup — Published Dates and Orphan Node Fix Summary

Fixed two data quality bugs in the InspireHEP pipeline: (1) published dates now extracted from `earliest_date` field and set on all InspireHEP papers, and (2) empty-string arXiv IDs from references without `arxiv_eprint` are filtered before entering the BFS crawl queue, eliminating orphan nodes.

## Tasks Completed

### Task 1: Add published date extraction from InspireHEP earliest_date and update API field queries

Added `earliest_date: Option<String>` to `InspireMetadata` struct. Updated `convert_hit_to_paper()` to extract and set `paper.published` using the chained-optional pattern (`metadata.earliest_date.as_deref().unwrap_or_default().to_string()`). Updated both `fetch_literature()` and `fetch_paper()` URL format strings to request the `earliest_date` field. Added debug tracing log for the extraction. Extended 3 existing tests and added 1 new wiremock integration test.

**Commit:** `3e22e98`
**Files:** `resyn-core/src/data_aggregation/inspirehep_api.rs`

### Task 2: Filter empty-string IDs from get_arxiv_references_ids() to eliminate orphan nodes

Added `.filter(|id| { if id.is_empty() { tracing::warn!(...); false } else { true } })` to `Paper::get_arxiv_references_ids()`. The root cause: InspireHEP references without an `arxiv_eprint` field produce `arxiv_eprint: Some("")` after conversion, and `get_arxiv_id()` returns `Ok("")` for this case. The filter prevents empty strings from entering the BFS queue as paper IDs, which previously caused the BFS to treat `""` as a valid paper, fetch nothing, and insert a disconnected node. Added 2 new unit tests; existing test unchanged and still passes.

**Commit:** `66841d5`
**Files:** `resyn-core/src/datamodels/paper.rs`

## Verification

- `cargo test -p resyn-core --features ssr`: 204 tests pass (196 lib + 6 integration + 2 text extraction)
- `cargo clippy -p resyn-core --features ssr --all-targets -- -Dwarnings`: passes cleanly
- New tests: `test_inspirehep_fetch_paper_published`, `test_get_arxiv_references_ids_filters_empty`, `test_get_arxiv_references_ids_filters_empty_link`

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None - both fixes wire real data (earliest_date from API, filter at collection point).

## Self-Check: PASSED

Files exist:
- `resyn-core/src/data_aggregation/inspirehep_api.rs` — FOUND
- `resyn-core/src/datamodels/paper.rs` — FOUND

Commits exist:
- `3e22e98` — FOUND (feat: add published date extraction)
- `66841d5` — FOUND (fix: filter empty-string IDs)
