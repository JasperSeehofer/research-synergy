---
phase: 07-incremental-crawl-infrastructure
plan: 01
subsystem: database
tags: [surrealdb, governor, rate-limiter, crawl-queue, token-bucket, migration]

# Dependency graph
requires: []
provides:
  - CrawlQueueRepository: enqueue_if_absent, claim_next_pending (atomic), mark_done, mark_failed, reset_stale_fetching, retry_failed, get_counts, clear_queue, has_completed_paper, pending_count
  - Migration 7: crawl_queue table with UNIQUE index on (paper_id, seed_paper_id)
  - SharedRateLimiter: Arc-wrapped governor token bucket with make_arxiv_limiter (3s) and make_inspirehep_limiter (350ms)
affects:
  - 07-02 (crawl loop uses CrawlQueueRepository and SharedRateLimiter)
  - 07-03 (SSE server reads queue status via get_counts)

# Tech tracking
tech-stack:
  added:
    - governor 0.10 (token bucket rate limiter, ssr-gated)
  patterns:
    - Named record IDs for SurrealDB idempotent INSERT: `CREATE crawl_queue:⟨key⟩ CONTENT {...}`
    - LET + UPDATE ONLY $var for atomic claim-and-mark in single SurrealDB query
    - IF $var != NONE THEN ... END to guard UPDATE ONLY when no entry exists
    - `<string>time::now()` cast to match option<string> schema fields
    - count-first-then-update pattern for reset/retry operations (avoids update-returns-zero issue)
    - Arc<RateLimiter> for shared token budget across cloned handles

key-files:
  created:
    - resyn-core/src/database/crawl_queue.rs
    - resyn-core/src/data_aggregation/rate_limiter.rs
  modified:
    - resyn-core/src/database/schema.rs
    - resyn-core/src/database/mod.rs
    - resyn-core/src/database/queries.rs
    - resyn-core/src/data_aggregation/mod.rs
    - resyn-core/Cargo.toml
    - Cargo.toml

key-decisions:
  - "Named record IDs used for idempotent enqueue — CREATE on same ID is a SurrealDB no-op"
  - "LET + UPDATE ONLY $entry_id (not WHERE id = $id) for atomic claim — WHERE-based update after SELECT does not persist in embedded SurrealDB"
  - "time::now() cast to <string> because schema declared claimed_at as option<string>"
  - "count_by_status before update for reset_stale_fetching and retry_failed — avoids returning stale count from UPDATE"
  - "governor until_ready() used for wait_for_token — cleaner than manual check+sleep loop"

patterns-established:
  - "Pattern: SurrealDB atomic claim — LET $id = (SELECT ... LIMIT 1)[0].id; IF $id != NONE THEN UPDATE ONLY $id SET ... END"
  - "Pattern: Idempotent enqueue — CREATE $rid CONTENT {...} where $rid is RecordId::new(table, composite_key)"
  - "Pattern: Rate limiter factory — make_X_limiter() returns SharedRateLimiter (Arc), call wait_for_token() before each request"

requirements-completed: [CRAWL-01, CRAWL-02, CRAWL-04]

# Metrics
duration: 71min
completed: 2026-03-15
---

# Phase 7 Plan 01: Migration 7 + CrawlQueueRepository + SharedRateLimiter Summary

**SurrealDB crawl queue (11 ops, atomic claim via LET+UPDATE ONLY) and governor token-bucket rate limiter (Arc-shared, factory functions for arXiv/InspireHEP)**

## Performance

- **Duration:** 71 min
- **Started:** 2026-03-15T19:30:00Z
- **Completed:** 2026-03-15T20:42:00Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Atomic crawl queue with idempotent enqueue (named record IDs), depth-ordered claim, crash recovery reset, and full CRUD — all 11 tests passing against in-memory SurrealDB
- SharedRateLimiter wrapping governor 0.10 token bucket, Arc-clonable for sharing across async tasks, with factory functions for arXiv (3s) and InspireHEP (350ms) intervals — 5 tests passing
- Discovered and documented SurrealDB embedded quirks: WHERE-based UPDATE after SELECT doesn't persist; `UPDATE ONLY $let_var` (not `UPDATE ... WHERE id = $bound_var`) is the correct atomic claim pattern

## Task Commits

1. **Task 1: Migration 7 + CrawlQueueRepository** - `de2e3d3` (feat)
2. **Task 2: Governor-based SharedRateLimiter** - `77e0967` (feat)

## Files Created/Modified

- `resyn-core/src/database/crawl_queue.rs` - CrawlQueueRepository with 11 methods and 11 tests
- `resyn-core/src/database/schema.rs` - Migration 7: crawl_queue table with UNIQUE index
- `resyn-core/src/database/mod.rs` - Added pub mod crawl_queue
- `resyn-core/src/database/queries.rs` - Updated migration version assertions (6 -> 7)
- `resyn-core/src/data_aggregation/rate_limiter.rs` - SharedRateLimiter, factory functions, wait_for_token
- `resyn-core/src/data_aggregation/mod.rs` - Added ssr-gated pub mod rate_limiter
- `resyn-core/Cargo.toml` - Added governor optional dep in ssr feature
- `Cargo.toml` - Added governor 0.10 to workspace deps

## Decisions Made

- Named record IDs (`crawl_queue:⟨paper_id_seed_id⟩`) for idempotent enqueue — `CREATE` on existing ID is a no-op in SurrealDB
- `UPDATE ONLY $entry_id` (LET variable) instead of `UPDATE ... WHERE id = $bound_var` for atomic claim — binding a `RecordId` to UPDATE WHERE doesn't persist in embedded SurrealDB
- `<string>time::now()` cast required because schema field is `option<string>` but governor's `time::now()` returns a datetime type
- `count_by_status` before update in reset/retry operations — avoids relying on UPDATE's affected-count return
- `governor::until_ready()` for wait_for_token — cleaner async wait vs. manual check+sleep loop

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed pre-existing abc_bridge.rs test compilation errors**
- **Found during:** Task 1 (building test suite)
- **Issue:** `Paper`, `StableGraph`, `Directed` not in scope in abc_bridge test module, blocking all tests from compiling
- **Fix:** Added missing imports `use crate::datamodels::paper::{Link, Paper, Reference}; use petgraph::Directed; use petgraph::stable_graph::StableGraph;`
- **Files modified:** resyn-core/src/gap_analysis/abc_bridge.rs
- **Verification:** All abc_bridge tests compile and pass
- **Committed in:** de2e3d3 (Task 1 commit)

**2. [Rule 1 - Bug] Updated schema migration version assertions in queries tests**
- **Found during:** Task 1 (full test suite after adding migration 7)
- **Issue:** 7 existing tests asserted `versions[0] == 6` but migration 7 was now the latest, causing failures
- **Fix:** Updated all version assertions and comments from 6 to 7 in queries.rs
- **Files modified:** resyn-core/src/database/queries.rs
- **Verification:** All 7 schema migration tests pass
- **Committed in:** de2e3d3 (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 1 - bug fixes)
**Impact on plan:** Pre-existing compilation errors and version assertions; no scope creep.

## Issues Encountered

- `UPDATE crawl_queue SET ... WHERE id = $bound_var` (binding a RecordId) silently succeeds but doesn't persist — discovered empirically after extensive debugging. The fix is `UPDATE ONLY $let_var` where `$let_var` comes from a `LET` statement in the same multi-statement query.
- `time::now()` in SurrealDB returns a datetime type, not a string — required `<string>time::now()` cast to match `option<string>` schema field.
- `UPDATE ONLY NONE` throws an error when queue is empty — fixed with `IF $entry_id != NONE THEN ... END` guard.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- CrawlQueueRepository ready for use in Plan 02 (crawl loop)
- SharedRateLimiter ready for injection into BFS crawler in Plan 02
- Queue status counts (`get_counts`) ready for SSE streaming in Plan 03
- No blockers

## Self-Check: PASSED

- resyn-core/src/database/crawl_queue.rs: FOUND
- resyn-core/src/data_aggregation/rate_limiter.rs: FOUND
- .planning/phases/07-incremental-crawl-infrastructure/07-01-SUMMARY.md: FOUND
- commit de2e3d3: FOUND
- commit 77e0967: FOUND

---
*Phase: 07-incremental-crawl-infrastructure*
*Completed: 2026-03-15*
