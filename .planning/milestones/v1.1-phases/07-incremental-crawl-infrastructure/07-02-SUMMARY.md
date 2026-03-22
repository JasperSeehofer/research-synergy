---
phase: 07-incremental-crawl-infrastructure
plan: 02
subsystem: resyn-server/crawl
tags: [crawl, queue, parallel, rate-limiter, progress-events]
dependency_graph:
  requires: ["07-01"]
  provides: ["queue-driven-crawl", "progress-events", "parallel-workers"]
  affects: ["resyn-server/src/commands/crawl.rs"]
tech_stack:
  added: ["tokio::task::JoinSet", "tokio::sync::Semaphore", "tokio::sync::broadcast"]
  patterns: ["semaphore-bounded-parallelism", "actor-per-task-source", "crash-recovery-reset"]
key_files:
  created: []
  modified:
    - resyn-server/src/commands/crawl.rs
    - resyn-server/Cargo.toml
decisions:
  - "PaperSource is not Clone — each spawned task creates its own instance via make_source() factory"
  - "fetch_references() takes &mut Paper and mutates paper.references in-place; use paper.get_arxiv_references_ids() to extract arXiv IDs after the call"
  - "Default concurrency is 4 (per CONTEXT.md user decision), overridable via --parallel=N"
  - "Semaphore::acquire_owned before spawn (not inside task) — bounds total in-flight tasks in the main loop"
  - "One automatic retry via retry_failed() after main loop drains, guarded by retried bool flag"
metrics:
  duration_minutes: 12
  tasks_completed: 2
  files_modified: 2
  completed_date: "2026-03-15"
---

# Phase 7 Plan 2: Queue-Driven Parallel Crawl Loop Summary

Replaced the in-memory BFS crawl (`recursive_paper_search_by_references`) with a DB queue-driven loop using parallel tokio workers bounded by a Semaphore, governor-based shared rate limiter, and crash recovery on startup.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add --parallel flag and futures dep | de10a02 | resyn-server/Cargo.toml |
| 2 | Queue-driven crawl loop with parallel workers | 37e7a71 | resyn-server/src/commands/crawl.rs |

## What Was Built

### Queue-Driven Crawl Architecture

The `run()` function in `crawl.rs` now follows this flow:

1. **Connect to DB** and initialize `CrawlQueueRepository` + `PaperRepository`
2. **Crash recovery**: `reset_stale_fetching()` resets any interrupted `fetching` entries back to `pending`
3. **Resume check**: if `pending_count() > 0`, skip seed enqueue (resuming); otherwise enqueue seed at depth 0
4. **Create shared resources**: `SharedRateLimiter` (governor), `Semaphore(concurrency)`, broadcast channel for `ProgressEvent`
5. **Main loop**: `claim_next_pending()` → skip if beyond `max_depth` or paper already in DB → acquire semaphore permit → spawn task
6. **Worker task**: `make_source()` factory creates fresh `PaperSource`, calls `wait_for_token()`, `fetch_paper()`, `fetch_references()`, enqueues all arXiv refs, upserts paper+citations, `mark_done()` or `mark_failed()`
7. **Drain loop**: when no pending entries, wait for `JoinSet` to drain, then attempt one automatic `retry_failed()`
8. **Final stats**: log papers_found, papers_failed, elapsed_secs; broadcast `"complete"` ProgressEvent

### Key Design Decisions

**PaperSource factory pattern**: `fetch_references(&mut self, paper: &mut Paper)` takes `&mut self`, making `PaperSource` unsuitable for sharing across tasks even with Arc. Each spawned task calls `make_source(&source_name)` to get its own zero-rate-limit instance (rate limiting is handled by the shared governor limiter instead).

**Actual PaperSource API**: The plan's interface documentation was outdated. The real trait has:
- `fetch_paper(&self, id: &str) -> Result<Paper, ResynError>` (immutable self)
- `fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError>` (mutates paper.references in-place)
- `get_arxiv_references_ids()` on `Paper` to extract arXiv IDs after the call

**Semaphore acquisition before spawn**: The permit is acquired in the main loop before `tokio::spawn`, not inside the task. This ensures the main loop naturally blocks at `concurrency` outstanding tasks, preventing unbounded queue claiming.

### ProgressEvent Struct

Defined at the top of `crawl.rs` (pub, for Plan 03 SSE consumer):

```rust
pub struct ProgressEvent {
    pub event_type: String,        // "paper_fetched", "paper_failed", "complete"
    pub papers_found: u64,
    pub papers_pending: u64,
    pub papers_failed: u64,
    pub current_depth: usize,
    pub max_depth: usize,
    pub elapsed_secs: f64,
    pub current_paper_id: Option<String>,
    pub current_paper_title: Option<String>,
}
```

Broadcast via `tokio::sync::broadcast::channel::<ProgressEvent>(256)`. The `tx.send()` errors are ignored — no receivers (no SSE client connected) is normal.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected PaperSource trait API mismatch**
- **Found during:** Task 2 implementation (cargo check)
- **Issue:** Plan documented `fetch_references(&mut self, id: &str) -> Result<Vec<Reference>, ResynError>` but the actual trait signature is `fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError>`. The `Reference` struct also has no `arxiv_id` field — arXiv IDs are extracted via `paper.get_arxiv_references_ids()` which filters `paper.references` for links with `Journal::Arxiv`.
- **Fix:** Used `source.fetch_references(&mut paper).await` then `paper.get_arxiv_references_ids()` to enumerate refs for enqueuing.
- **Files modified:** resyn-server/src/commands/crawl.rs
- **Commit:** 37e7a71

## Verification

```
cargo build -p resyn-server    -> Finished dev profile
cargo clippy --all-targets --all-features -- -D warnings  -> Finished (no warnings)
cargo fmt --all -- --check     -> Clean
cargo test                     -> 169 passed, 0 failed
```

## Self-Check: PASSED

- resyn-server/src/commands/crawl.rs: exists, 280 lines, contains `claim_next_pending`
- resyn-server/Cargo.toml: contains `futures = { workspace = true }`
- Commit de10a02: present
- Commit 37e7a71: present
- All 169 tests passing
