# Phase 7: Incremental Crawl Infrastructure - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Make high-depth crawls (depth 5+) resilient and observable. Replace the in-memory BFS frontier with a DB-backed crawl queue that survives crashes, add live progress reporting via SSE, and enable parallel reference fetching within rate limits. The existing `resyn crawl` subcommand gains incremental behavior by default plus new flags for parallelism and progress.

</domain>

<decisions>
## Implementation Decisions

### Queue & resumability
- Queue-driven resume: `resyn crawl` checks DB queue first — if pending entries exist, resume from where it left off; if not, start fresh from seed
- Separate `crawl_queue` SurrealDB table (not fields on paper records) — clean separation of crawl orchestration from paper data
- Minimal queue entries: paper_id, depth_level, status (pending/fetching/done/failed), timestamps — no provenance tracking
- Failed entries get one automatic retry on next resume; if they fail again, permanently marked failed
- Always incremental by default — every `resyn crawl` is resumable, no special flag needed

### Progress reporting
- Dual output: structured progress logs always go to stderr via tracing; `--progress[=PORT]` starts an SSE server (default localhost:3001)
- SSE payload: papers_found, papers_pending, papers_failed, current_depth, max_depth, elapsed_seconds + current_paper_id, current_paper_title, last_event type
- Event frequency: fire on every state change (paper fetched/queued/failed) + 5s heartbeat if nothing changed (keeps connections alive)
- Completion event: final SSE event with type=complete, total stats, duration, failure count — then stream closes

### Parallelism model
- Global token bucket rate limiter: 1 token per 3s for arXiv, 1 per 350ms for InspireHEP — shared across all concurrent tasks
- Auto-tuned concurrency: starts at 4 tasks, adjusts based on observed latency/error rates
- Paper-level task granularity: each task claims a pending paper from the DB queue atomically — papers at the same depth run in parallel
- Always enqueue all discovered references regardless of max_depth; filter at dequeue time — keeps a complete picture of the graph frontier for future "expand crawl" use

### CLI surface
- `resyn crawl -p <id> -d <depth>` — always incremental, resumes from queue if pending entries exist
- `--parallel[=N]` — enable parallel mode; no value = auto-tune, value = override concurrency
- `--progress[=PORT]` — start SSE server; no value = default port 3001, value = custom port
- Queue management subcommands:
  - `resyn crawl status` — show queue summary (pending/done/failed counts)
  - `resyn crawl clear` — reset the crawl queue
  - `resyn crawl retry` — mark all failed entries as pending for retry

### Claude's Discretion
- Token bucket implementation (roll own vs crate like `governor`)
- Auto-tune algorithm specifics (how to detect backpressure, adjustment strategy)
- DB migration for `crawl_queue` table schema details
- SSE server implementation (lightweight Axum or tokio-based)
- Atomic queue claim mechanism in SurrealDB (UPDATE ... WHERE status = 'pending' LIMIT 1 or similar)

</decisions>

<specifics>
## Specific Ideas

- The crawl should feel like a long-running daemon that you can kill and restart without losing work
- `curl -N localhost:3001/progress` should "just work" with readable output — success criterion from roadmap
- Queue management commands (`status`, `clear`, `retry`) give visibility into what the crawler is doing without needing to query SurrealDB directly
- Enqueuing all references (even beyond max_depth) means a user can later run `resyn crawl -d 10` on the same seed and it picks up where depth-5 left off

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `recursive_paper_search_by_references` in `resyn-core/src/data_aggregation/arxiv_utils.rs`: current BFS implementation — will be replaced/refactored but contains the fetch+parse logic to reuse
- `PaperSource` trait (`fetch_paper`, `fetch_references`): stays as the fetch interface — parallel tasks call through this
- `PaperRepository` in `resyn-core/src/database/queries.rs`: existing upsert_paper/upsert_citations — queue entries can check "already in paper table" to skip re-fetch
- DB migration system (6 existing migrations): add migration 7+ for `crawl_queue` table
- `ArxivHTMLDownloader.with_rate_limit()` and `InspireHepClient` rate limiting: will be replaced by the global token bucket

### Established Patterns
- SurrealDB embedded via `kv-surrealkv` feature: queue table uses same connection
- clap derive macros for CLI: extend CrawlArgs with new fields, add subcommand enum for status/clear/retry
- Tracing for structured logging: progress events naturally fit as tracing events at info level

### Integration Points
- `resyn-server/src/commands/crawl.rs`: main entry point — refactor `run()` to use queue-based crawl loop instead of calling `recursive_paper_search_by_references`
- `resyn-core/src/database/`: new `crawl_queue.rs` module for queue operations (claim, enqueue, mark_done, mark_failed, counts)
- `resyn-core/src/data_aggregation/`: rate limiter module shared across parallel tasks
- SSE server: lightweight Axum instance in resyn-server, receives progress events via tokio broadcast channel

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-incremental-crawl-infrastructure*
*Context gathered: 2026-03-15*
