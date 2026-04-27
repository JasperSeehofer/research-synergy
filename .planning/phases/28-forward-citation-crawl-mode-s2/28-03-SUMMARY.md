---
phase: 28-forward-citation-crawl-mode-s2
plan: "03"
subsystem: database
tags: [surrealdb, surreal-relate, citation-graph, edge-direction, tdd]

requires:
  - phase: 28-01
    provides: SemanticScholarSource fetch_citing_papers inner implementation
  - phase: 28-02
    provides: Paper::citing_papers transient field and PaperSource trait extension

provides:
  - PaperRepository::upsert_inverse_citations_batch method in queries.rs
  - 5 integration tests proving edge direction is correct (citing->cited, not inverted)

affects:
  - 28-04 (crawler worker calls this method to persist forward-citation edges)

tech-stack:
  added: []
  patterns:
    - "Inverse-direction RELATE batch: RELATE $from->cites->$to where from=citing, to=cited"
    - "Skip-and-count pattern: skip unresolvable refs via debug log, count only inserted edges"
    - "Dangling-endpoint tolerance: neither citing nor cited paper needs to exist"

key-files:
  created: []
  modified:
    - resyn-core/src/database/queries.rs

key-decisions:
  - "Return count of edges actually inserted (not input slice length) — refs without arXiv IDs are silently skipped"
  - "Method placed immediately after upsert_citations_batch to group all batch-insert methods"
  - "Tests placed in new mod inverse_citations_tests inside queries.rs, matching existing in-module test pattern"
  - "cargo fmt applied to queries.rs only (other files with pre-existing fmt issues left untouched — out of scope)"

patterns-established:
  - "Direction assertion pattern: test BOTH get_citing_papers non-empty AND get_cited_papers empty to prove correct direction"
  - "Dangling-edge test: call method with non-existent cited+citing papers; verify Ok(1) — confirms no existence checks"

requirements-completed: []

duration: 28min
completed: "2026-04-27"
---

# Phase 28 Plan 03: Add upsert_inverse_citations_batch Summary

**`PaperRepository::upsert_inverse_citations_batch` — writes RELATE citing->cites->cited edges (correct forward-citation direction) with 5 integration tests proving direction cannot be silently inverted**

## Performance

- **Duration:** 28 min
- **Started:** 2026-04-27T13:57:24Z
- **Completed:** 2026-04-27T14:25:24Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- New `upsert_inverse_citations_batch(&self, cited_arxiv_id: &str, citing_papers: &[Reference]) -> Result<usize, ResynError>` method in `PaperRepository`
- Edge direction is `RELATE $from->cites->$to` where `from = citing_arxiv_id`, `to = cited_arxiv_id` — the only correct direction for forward-citation edges
- References without a resolvable arXiv ID are silently skipped (debug log, no panic, no error returned)
- Returns count of edges actually inserted (≤ input slice length)
- 5 integration tests in `mod inverse_citations_tests` — all pass; direction-inversion test would catch any implementation error

## Task Commits

1. **Task 1: Add upsert_inverse_citations_batch to PaperRepository** - `16c3dd9` (feat)
2. **Task 2: Add database integration tests proving edge direction** - `1d4a60c` (test)

## Files Created/Modified

- `resyn-core/src/database/queries.rs` — added `Reference` import, `upsert_inverse_citations_batch` method (lines 140-197), and `mod inverse_citations_tests` module (lines 2711-2845 after fmt)

## Decisions Made

- Method signature uses `&[Reference]` (not `Vec<Reference>`) to avoid allocation at call site — matches plan spec exactly
- `Reference` import added to top-level use statements in queries.rs (was missing; only `Paper` and `DataSource` were imported from paper module)
- Tests placed in-module (`mod inverse_citations_tests`) inside queries.rs, consistent with all other database test modules in that file — the plan's reference to `resyn-core/tests/database_tests.rs` was adapted because that file does not exist; database tests live inline in queries.rs
- `cargo fmt -p resyn-core` applied to queries.rs after test insertion (formatting only, no logic changes); other files with pre-existing fmt issues were restored to prevent out-of-scope changes

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Import] Added `Reference` to imports in queries.rs**
- **Found during:** Task 1 (implementing upsert_inverse_citations_batch)
- **Issue:** The method signature takes `&[Reference]` but `Reference` was not imported at the top of queries.rs (only `DataSource` and `Paper` were imported from the paper module)
- **Fix:** Changed `use crate::datamodels::paper::{DataSource, Paper};` to `use crate::datamodels::paper::{DataSource, Paper, Reference};`
- **Files modified:** resyn-core/src/database/queries.rs
- **Verification:** `cargo check -p resyn-core` succeeds with no new warnings
- **Committed in:** `16c3dd9` (Task 1 commit)

**2. [Rule 1 - Test Location Adaptation] Tests placed in-module not in external test file**
- **Found during:** Task 2 (adding integration tests)
- **Issue:** Plan referenced `resyn-core/tests/database_tests.rs` but this file does not exist. All 6 database tests mentioned in CLAUDE.md are in-module inside `queries.rs` (in `mod tests`, `mod similarity_tests`, `mod graph_metrics_tests`, `mod community_tests`). Using an external test file would require `resyn_core::` paths and `#[cfg(feature = "ssr")]` gating — inconsistent with the established pattern.
- **Fix:** Appended `mod inverse_citations_tests { ... }` to queries.rs following the exact same structure as the existing inline test modules
- **Files modified:** resyn-core/src/database/queries.rs
- **Verification:** `cargo test -p resyn-core --lib --features ssr inverse_citations_tests` shows 5 passed
- **Committed in:** `1d4a60c` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 missing import, 1 test location adaptation)
**Impact on plan:** Both fixes were essential for compilation and consistency. No scope creep.

## Issues Encountered

- **Worktree vs main repo test path confusion:** `cargo test` run from the main repo directory (`/home/jasper/Repositories/research-synergy`) compiled from main repo source files (which don't have uncommitted worktree changes). Tests only appeared after running `cargo test` from the worktree directory. This is a known gotcha with git worktrees — Cargo uses the CWD, not git HEAD.
- **`cargo fmt --all` scope:** Running `cargo fmt --all` applied formatting to many pre-existing files outside the task scope. These were reverted via `git checkout --` to keep the commit clean.
- **Pre-existing test failures (7):** `test_migrate_schema_*` tests in the existing `mod tests` module fail — confirmed pre-existing, unrelated to this plan.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. `upsert_inverse_citations_batch` operates on the existing `cites` relation table. All values reach SurrealDB via parameter binding (`.bind(("from", from_rid))` / `.bind(("to", to_rid))`), never string interpolation — T-28-11 mitigation confirmed. Direction correctness verified by test asserting `get_cited_papers(A).is_empty()` after inverse batch insert — T-28-12 mitigation confirmed.

## Next Phase Readiness

- `PaperRepository::upsert_inverse_citations_batch` is production-ready for plan 04 (crawl worker integration)
- Plan 04 calls this method immediately after `fetch_citing_papers` in the BFS worker loop to persist forward-citation edges
- The cited paper and citing paper do not need to exist before calling the method — dangling edges are accepted

## Self-Check: PASSED

- `resyn-core/src/database/queries.rs` — FOUND
- `.planning/phases/28-forward-citation-crawl-mode-s2/28-03-SUMMARY.md` — FOUND
- Commit `16c3dd9` (feat: upsert_inverse_citations_batch) — FOUND
- Commit `1d4a60c` (test: integration tests) — FOUND
- `grep -c "pub async fn upsert_inverse_citations_batch"` → 1 hit
- `grep -c 'RELATE $from->cites->$to'` → 3 hits (upsert_citations, upsert_citations_batch, new method)
- `cargo test inverse_citations_tests` → 5 passed, 0 failed
