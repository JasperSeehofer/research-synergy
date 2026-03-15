---
phase: 07-incremental-crawl-infrastructure
plan: "03"
subsystem: infra
tags: [axum, sse, tokio-stream, clap, crawl-queue, progress-streaming]

# Dependency graph
requires:
  - phase: 07-incremental-crawl-infrastructure
    provides: "07-01: CrawlQueueRepository (get_counts, clear_queue, retry_failed); 07-02: ProgressEvent, broadcast channel, crawl loop"
provides:
  - "SSE /progress endpoint via Axum serving JSON ProgressEvent stream"
  - "resyn crawl status/clear/retry subcommands for queue management"
  - "--progress[=PORT] flag starts embedded SSE server (default port 3001)"
  - "5-second keep-alive heartbeat on SSE connection"
  - "Completion event fires when crawl loop finishes"
affects:
  - "Phase 08+ — observability foundation for monitoring crawl progress"

# Tech tracking
tech-stack:
  added:
    - "axum 0.8 (workspace dep) — embedded HTTP server for SSE"
    - "tokio-stream 0.1 with sync feature — BroadcastStream adapter for SSE"
  patterns:
    - "SSE via axum::response::sse::Sse + BroadcastStream wrapping tokio broadcast::Receiver"
    - "KeepAlive::new().interval(5s) for explicit heartbeat (not KeepAlive::default)"
    - "filter_map on BroadcastStream msg.ok() handles lagged-receiver errors gracefully"
    - "Inner async fn sse_handler defined inside tokio::spawn closure to capture State type"

key-files:
  created: []
  modified:
    - "resyn-server/src/commands/crawl.rs — CrawlSubcommand enum, --progress flag, SSE spawn, subcmd dispatch"
    - "resyn-server/Cargo.toml — axum and tokio-stream deps added"
    - "Cargo.toml — axum 0.8 and tokio-stream 0.1 workspace deps"

key-decisions:
  - "axum and tokio-stream added at workspace level for future reuse (e.g., resyn-app SSR)"
  - "SSE server defined entirely inline inside tokio::spawn — avoids polluting module-level with single-use types"
  - "Subcommand dispatch returns early before paper_id validation — queue commands need only --db arg"

patterns-established:
  - "SSE pattern: BroadcastStream::new(rx).filter_map(|msg| async { msg.ok()... }) is the canonical lagged-safe SSE stream"
  - "Queue management CLI: subcmd dispatched at top of run() with early return, before crawl-specific validation"

requirements-completed:
  - CRAWL-03

# Metrics
duration: 18min
completed: "2026-03-15"
---

# Phase 7 Plan 03: SSE Progress Server and Queue Management Summary

**Axum SSE /progress endpoint and resyn crawl status/clear/retry CLI subcommands completing the crawl observability layer**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-03-15T21:08:25Z
- **Completed:** 2026-03-15T21:26:00Z
- **Tasks:** 1/2 (Task 2 is human-verify checkpoint, awaiting verification)
- **Files modified:** 3

## Accomplishments

- `resyn crawl status` prints queue counts (total/pending/fetching/done/failed) from SurrealDB
- `resyn crawl clear` deletes all queue entries via `clear_queue()`
- `resyn crawl retry` marks all failed entries as pending via `retry_failed()`
- `--progress[=PORT]` spawns an Axum SSE server on `/progress` (default port 3001)
- SSE stream wraps the existing `broadcast::Sender<ProgressEvent>` via `BroadcastStream` adapter
- 5-second keep-alive heartbeat keeps SSE connection alive during idle periods
- Completion event (`event_type: "complete"`) already fires at end of crawl loop from Plan 02

## Task Commits

Each task was committed atomically:

1. **Task 1: SSE progress server and queue management subcommands** - `ecc451b` (feat)
2. **Task 2: Verify end-to-end** - PENDING (checkpoint:human-verify)

**Plan metadata:** (to be committed after human verification)

## Files Created/Modified

- `resyn-server/src/commands/crawl.rs` — Added `CrawlSubcommand` enum, `--progress` flag, SSE server startup via `tokio::spawn`, queue management dispatch at top of `run()`
- `resyn-server/Cargo.toml` — Added `axum` and `tokio-stream` workspace deps
- `Cargo.toml` — Added `axum = "0.8"` and `tokio-stream = { version = "0.1", features = ["sync"] }` as workspace deps

## Decisions Made

- `axum` and `tokio-stream` added at workspace level so future crates (e.g., a server-rendered resyn-app variant) can reuse them without re-adding.
- SSE handler function defined inline inside `tokio::spawn` closure — avoids cluttering module scope with a function only used in one path.
- Queue management subcommand dispatch happens before `validate_arxiv_id` — these subcommands don't require a valid paper ID, only `--db`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] axum and tokio-stream were not in workspace Cargo.toml**
- **Found during:** Task 1 (SSE server implementation)
- **Issue:** Plan stated "workspace entry already added in Plan 01" but neither `axum` nor `tokio-stream` were present in workspace `Cargo.toml`
- **Fix:** Added both to `[workspace.dependencies]` in root `Cargo.toml` and to `resyn-server/Cargo.toml`
- **Files modified:** `Cargo.toml`, `resyn-server/Cargo.toml`
- **Verification:** `cargo build -p resyn-server` succeeded
- **Committed in:** `ecc451b` (Task 1 commit)

**2. [Rule 3 - Blocking] rustfmt required function return-type brace on own line**
- **Found during:** Task 1 verification (`cargo fmt --all -- --check`)
- **Issue:** `sse_handler` return type `Sse<impl Stream<...>>` was too long; rustfmt requires `{` on its own line
- **Fix:** Moved opening brace to new line per rustfmt convention
- **Files modified:** `resyn-server/src/commands/crawl.rs`
- **Verification:** `cargo fmt --all -- --check` passes
- **Committed in:** `ecc451b` (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes required for correct compilation. No scope creep.

## Issues Encountered

None beyond the auto-fixed deviations above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Full incremental crawl infrastructure complete: queue-backed BFS, parallel workers, crash recovery, SSE progress observability, queue management CLI
- Human verification of end-to-end flow (Task 2 checkpoint) required before marking phase 07 complete
- Phase 08+ can rely on `--progress` SSE stream for monitoring long crawls

---
*Phase: 07-incremental-crawl-infrastructure*
*Completed: 2026-03-15*
