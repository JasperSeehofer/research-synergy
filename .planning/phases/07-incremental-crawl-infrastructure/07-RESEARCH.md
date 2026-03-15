# Phase 7: Incremental Crawl Infrastructure - Research

**Researched:** 2026-03-15
**Domain:** Rust async concurrency, SurrealDB queue patterns, SSE progress reporting, token bucket rate limiting
**Confidence:** HIGH (core stack verified via official docs; one MEDIUM area noted below)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Queue-driven resume: `resyn crawl` checks DB queue first — if pending entries exist, resume from where it left off; if not, start fresh from seed
- Separate `crawl_queue` SurrealDB table (not fields on paper records) — clean separation of crawl orchestration from paper data
- Minimal queue entries: paper_id, depth_level, status (pending/fetching/done/failed), timestamps — no provenance tracking
- Failed entries get one automatic retry on next resume; if they fail again, permanently marked failed
- Always incremental by default — every `resyn crawl` is resumable, no special flag needed
- Dual output: structured progress logs always go to stderr via tracing; `--progress[=PORT]` starts an SSE server (default localhost:3001)
- SSE payload: papers_found, papers_pending, papers_failed, current_depth, max_depth, elapsed_seconds + current_paper_id, current_paper_title, last_event type
- Event frequency: fire on every state change (paper fetched/queued/failed) + 5s heartbeat if nothing changed
- Completion event: final SSE event with type=complete, total stats, duration, failure count — then stream closes
- Global token bucket rate limiter: 1 token per 3s for arXiv, 1 per 350ms for InspireHEP — shared across all concurrent tasks
- Auto-tuned concurrency: starts at 4 tasks, adjusts based on observed latency/error rates
- Paper-level task granularity: each task claims a pending paper from the DB queue atomically
- Always enqueue all discovered references regardless of max_depth; filter at dequeue time
- CLI: `resyn crawl -p <id> -d <depth>` always incremental; `--parallel[=N]`; `--progress[=PORT]`
- Queue management: `resyn crawl status`, `resyn crawl clear`, `resyn crawl retry`

### Claude's Discretion

- Token bucket implementation (roll own vs crate like `governor`)
- Auto-tune algorithm specifics (how to detect backpressure, adjustment strategy)
- DB migration for `crawl_queue` table schema details
- SSE server implementation (lightweight Axum or tokio-based)
- Atomic queue claim mechanism in SurrealDB (UPDATE ... WHERE status = 'pending' LIMIT 1 or similar)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CRAWL-01 | DB-backed crawl queue replacing in-memory BFS frontier for resumability | SurrealDB migration 7 adds `crawl_queue` table; `PaperRepository` pattern extended to `CrawlQueueRepository` |
| CRAWL-02 | Crash recovery — resume interrupted crawls from last checkpoint | Queue claim via BEGIN/COMMIT transaction; `fetching` entries reset to `pending` on startup |
| CRAWL-03 | Crawl progress reporting via SSE (papers found, queue depth, estimated time) | Axum 0.8 `Sse<impl Stream>` + tokio `broadcast` channel from crawl loop to handler |
| CRAWL-04 | Parallel reference fetching where rate limits allow | `tokio::sync::Semaphore` for concurrency cap + `governor` 0.10 for shared token bucket rate limit |
</phase_requirements>

---

## Summary

Phase 7 replaces the in-memory BFS loop (`recursive_paper_search_by_references`) with a DB-backed crawl queue stored in SurrealDB, enabling crash recovery and resumption. Three orthogonal problems are layered on top: (1) atomic queue claiming from concurrent tokio tasks, (2) live progress reporting via SSE, and (3) parallel fetching within rate limits.

The key constraint is the embedded `kv-surrealkv` storage model. SurrealDB's Rust SDK is designed so that the `Surreal<Any>` client can be cloned and shared across tasks using `Arc<Surreal<Any>>` — this is the officially documented pattern. The embedded engine serializes writes internally, so concurrent tasks issuing separate BEGIN/COMMIT transactions get correct ACID isolation. The critical insight for queue claiming is that SurrealDB's UPDATE statement does NOT support a LIMIT clause, so atomic claim must use a different pattern: a named `LET` + conditional UPDATE within a single transaction (see Architecture Patterns below).

For rate limiting, `governor` 0.10 is the standard Rust crate — it implements GCRA (equivalent to a leaky bucket), is `Arc`-clonable and thread-safe, and does not require an external `nonzero_ext` macro in modern editions (you can use `NonZeroU32::new(1).unwrap()` or the `nonzero!` macro from `nonzero_ext`). For SSE, Axum 0.8 has built-in `axum::response::sse::Sse` support gated on the `tokio` feature which is on by default.

**Primary recommendation:** Add `axum = "0.8"` and `governor = "0.10"` to resyn-server; implement `CrawlQueueRepository` in resyn-core/database; refactor `crawl.rs` to a queue-driven loop; wire progress via `tokio::sync::broadcast` channel to an Axum SSE endpoint started on `--progress`.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `axum` | 0.8.8 | SSE HTTP server for `--progress` | Official tokio-rs framework; built-in `sse` module; already-dep tokio runtime |
| `governor` | 0.10.4 | Shared token bucket rate limiter | Standard GCRA implementation; `Arc`-clonable; thread-safe CAS updates; no mutex overhead |
| `tokio::sync::broadcast` | (tokio 1.44, already dep) | Fan-out progress events from crawl loop to SSE clients | Built into tokio; MPMC semantics needed for multiple curl clients watching `/progress` |
| `tokio::sync::Semaphore` | (tokio 1.44, already dep) | Bound concurrent crawl task count | Built into tokio; `acquire_owned()` moves permit across task boundary |
| `surrealdb` BEGIN/COMMIT | (surrealdb 3, already dep) | Atomic queue claim transaction | Required because UPDATE has no LIMIT; must use transaction to claim + mark atomically |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `futures-util` | 0.3 (already dep via `futures`) | Build `Stream` for Axum SSE from `broadcast::Receiver` | Needed for `BroadcastStream` or manual stream adapter |
| `tokio-stream` | 0.1 | `BroadcastStream` wrapper | Alternative to manual `Stream` impl for broadcast channels; optional |
| `nonzero_ext` | 0.3 | `nonzero!` macro for governor Quota | Optional quality-of-life; can use `NonZeroU32::new(1).unwrap()` instead |
| `chrono` | 0.4 (already dep) | RFC3339 timestamps for queue entries | Already in workspace |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `governor` | Roll own token bucket with `tokio::time::sleep` + `Arc<Mutex<Instant>>` | Roll-own is simpler to understand but does not handle burst correctly and requires careful sleep math |
| Axum SSE server | Plain `tokio::net::TcpListener` + manual HTTP SSE response | Much more boilerplate; Axum adds ~100ms startup with far less code |
| `tokio::sync::broadcast` | `tokio::sync::watch` | `watch` only keeps latest value; `broadcast` lets all SSE clients see all events |

**Installation (additions to workspace):**

```toml
# workspace Cargo.toml [workspace.dependencies]
axum = "0.8"
tokio-stream = "0.1"

# resyn-server/Cargo.toml [dependencies]
axum = { workspace = true }
tokio-stream = { workspace = true }

# resyn-core/Cargo.toml [dependencies] under ssr feature
governor = "0.10"
```

---

## Architecture Patterns

### Recommended Module Structure

```
resyn-core/src/database/
├── client.rs         # existing — Db type alias stays
├── mod.rs            # add pub mod crawl_queue
├── queries.rs        # existing PaperRepository
├── schema.rs         # add apply_migration_7() for crawl_queue table
└── crawl_queue.rs    # NEW: CrawlQueueRepository

resyn-core/src/data_aggregation/
├── arxiv_utils.rs    # BFS logic extracted from recursive_paper_search
├── rate_limiter.rs   # NEW: shared governor-based RateLimiter wrapper
└── ...               # existing files unchanged

resyn-server/src/commands/
├── crawl.rs          # refactored: queue-driven loop + --parallel + --progress
└── crawl_queue_cmds.rs  # NEW or inline: status/clear/retry subcommands
```

### Pattern 1: SurrealDB Atomic Queue Claim

The UPDATE statement in SurrealDB has no LIMIT clause. The correct atomic claim pattern uses a transaction: SELECT one pending record id, then UPDATE only that specific record by its ID.

```rust
// Source: SurrealDB Rust SDK transaction docs https://surrealdb.com/docs/sdk/rust/concepts/transaction
// Pattern: BEGIN + SELECT id + UPDATE by id + COMMIT = atomic claim
async fn claim_next_pending(db: &Db) -> Result<Option<QueueEntry>, ResynError> {
    let mut response = db
        .query("
            BEGIN;
            LET $entry = (SELECT id, paper_id, depth_level FROM crawl_queue
                          WHERE status = 'pending'
                          ORDER BY depth_level ASC, created_at ASC
                          LIMIT 1)[0];
            IF $entry != NONE {
                UPDATE $entry.id SET status = 'fetching', claimed_at = time::now();
            };
            RETURN $entry;
            COMMIT;
        ")
        .await
        .map_err(|e| ResynError::Database(format!("claim_next_pending failed: {e}")))?;

    let entries: Vec<QueueEntry> = response.take(3)?; // index 3 = RETURN result
    Ok(entries.into_iter().next())
}
```

**Why this works:** The embedded surrealkv engine provides ACID transactions; the SELECT + UPDATE within BEGIN/COMMIT is an atomic unit. Concurrent tasks calling this will serialize at the storage layer — only one gets each record.

**Crash recovery:** On startup, reset all `fetching` entries back to `pending`:
```rust
db.query("UPDATE crawl_queue SET status = 'pending', claimed_at = NONE
          WHERE status = 'fetching'").await?;
```

### Pattern 2: Governor Shared Rate Limiter

```rust
// Source: https://docs.rs/governor/latest/governor/_guide/index.html
use governor::{Quota, RateLimiter, clock::DefaultClock, state::direct::NotKeyed, state::InMemoryState};
use std::num::NonZeroU32;
use std::sync::Arc;

type SharedLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

fn make_rate_limiter(requests_per_interval: u32, interval: Duration) -> SharedLimiter {
    let quota = Quota::with_period(interval)
        .expect("non-zero interval")
        .allow_burst(NonZeroU32::new(1).unwrap());
    Arc::new(RateLimiter::direct(quota))
}

// arXiv: 1 request per 3 seconds
let arxiv_limiter = make_rate_limiter(1, Duration::from_secs(3));

// InspireHEP: 1 request per 350ms
let inspire_limiter = make_rate_limiter(1, Duration::from_millis(350));

// In each worker task — await until token available:
loop {
    match limiter.check() {
        Ok(_) => break,
        Err(not_until) => {
            tokio::time::sleep(not_until.wait_time_from(DefaultClock::default().now())).await;
        }
    }
}
```

All concurrent worker tasks share the same `Arc<RateLimiter>`. The CAS-based state update means no mutex contention.

### Pattern 3: Parallel Workers with Semaphore

```rust
// Source: tokio docs https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html
use tokio::sync::Semaphore;
use std::sync::Arc;

let sem = Arc::new(Semaphore::new(concurrency)); // e.g. 4

loop {
    let entry = claim_next_pending(&db).await?;
    let Some(entry) = entry else {
        // Queue empty — wait for any in-flight tasks to finish then check again
        break;
    };

    let permit = Arc::clone(&sem).acquire_owned().await.unwrap();
    let db = db.clone();
    let limiter = Arc::clone(&rate_limiter);
    let tx = progress_tx.clone();

    tokio::spawn(async move {
        let _permit = permit; // dropped when task completes, releasing slot
        // rate-limit wait
        // fetch paper
        // enqueue discovered references
        // send progress event to tx
    });
}
```

### Pattern 4: Axum SSE from broadcast Channel

```rust
// Source: https://docs.rs/axum/latest/axum/response/sse/
use axum::{Router, routing::get, response::sse::{Event, KeepAlive, Sse}};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use futures_util::StreamExt;

#[derive(Clone, serde::Serialize)]
struct ProgressEvent { /* papers_found, papers_pending, etc. */ }

async fn sse_handler(
    State(tx): State<broadcast::Sender<ProgressEvent>>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let rx = tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|msg| async move {
            msg.ok().and_then(|e| {
                serde_json::to_string(&e).ok()
                    .map(|data| Ok(Event::default().data(data)))
            })
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// Start SSE server on --progress port, passing Sender as shared state:
let app = Router::new()
    .route("/progress", get(sse_handler))
    .with_state(progress_tx);
tokio::spawn(async move {
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await.unwrap();
    axum::serve(listener, app).await.unwrap();
});
```

### Pattern 5: Clap Subcommand for queue management

Extend `Commands` enum with a nested subcommand:

```rust
#[derive(Subcommand, Debug)]
enum CrawlSubcommand {
    // Default behavior (no subcommand) = crawl run
    Status,
    Clear,
    Retry,
}

// In CrawlArgs:
#[command(subcommand)]
pub subcmd: Option<CrawlSubcommand>,
```

`resyn crawl` (no subcmd) = run crawl. `resyn crawl status` = show queue summary.

### Anti-Patterns to Avoid

- **Storing queue state in-memory only:** The entire point of CRAWL-01/02 is that the queue survives crashes. Never use `Vec<String>` as the BFS frontier alongside DB persistence.
- **Creating a new SurrealDB connection per worker task:** Known to cause `SurrealKV panics` and lock errors. Create one connection, wrap in `Arc`, clone the handle for each task — this is the officially documented pattern.
- **`UPDATE crawl_queue SET status = 'fetching' WHERE status = 'pending' LIMIT 1`:** This syntax is invalid in SurrealDB. Must use BEGIN/SELECT id/UPDATE by id/COMMIT pattern.
- **Sleeping a fixed duration for rate limiting instead of using governor:** Fixed sleeps don't correctly handle burst recovery and require careful math when tasks finish at different times.
- **Spawning unlimited tokio tasks:** Without a Semaphore, a depth-5 crawl with many references will spawn thousands of tasks simultaneously, exhausting memory and overwhelming the rate limiter.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Token bucket rate limiting | Custom `Arc<Mutex<Instant>>` sleep loop | `governor` 0.10 | GCRA is mathematically correct; hand-rolled leaky buckets have off-by-one errors in burst handling and drift under load |
| HTTP SSE response | Raw `Content-Type: text/event-stream` + manual chunked encoding | `axum::response::sse::Sse` | SSE has edge cases (reconnect, keep-alive, event ID, `\n\n` encoding) that Axum handles |
| Stream from broadcast receiver | Manual `poll_next` impl | `tokio_stream::wrappers::BroadcastStream` | Handles lagged receiver errors correctly; implements `Stream` trait properly |
| ACID queue transactions | `#[tokio::test]` + `Arc<Mutex<Vec>>` | SurrealDB BEGIN/COMMIT | Queue must survive process restarts; in-memory not acceptable |

**Key insight:** The "just sleep" approach to rate limiting does not compose correctly across multiple concurrent tasks — governor's CAS-based state handles this without locks.

---

## Common Pitfalls

### Pitfall 1: SurrealDB concurrent access with embedded engine

**What goes wrong:** Creating multiple `connect()` calls in different tasks causes SurrealKV panics (`no locks available` or IO errors).
**Why it happens:** The embedded engine uses file locking; multiple open handles to the same file fight over the lock.
**How to avoid:** Create ONE connection in `main`/`run()`, wrap it in `Arc`, pass cloned `Arc<Db>` to all workers.
**Warning signs:** Intermittent panics in `resyn crawl --parallel` that don't occur sequentially.

### Pitfall 2: `fetching` entries left behind after crash

**What goes wrong:** If the process crashes while tasks are fetching, entries remain in `fetching` state forever and are never retried.
**Why it happens:** No cleanup on startup.
**How to avoid:** On every `resyn crawl` startup (before queue loop begins), reset `fetching → pending`:
```sql
UPDATE crawl_queue SET status = 'pending' WHERE status = 'fetching';
```
**Warning signs:** Queue shrinks on each run but papers at certain depths are never fetched.

### Pitfall 3: `broadcast::Receiver` lagging when SSE client is slow

**What goes wrong:** If no client connects or a client is slow, `broadcast::Sender` fills its buffer and old messages are dropped; subsequent `Receiver::recv()` returns `Err(RecvError::Lagged)`.
**Why it happens:** `tokio::sync::broadcast` has a fixed-size ring buffer (default 1024).
**How to avoid:** `BroadcastStream` handles `Lagged` by logging and continuing; the stream does not terminate. On the crawl loop side, use `tx.send()` (which drops if no receivers) — don't use `tx.send().unwrap()`.
**Warning signs:** `tokio_stream::wrappers::BroadcastStream` emits a tracing warning on lag.

### Pitfall 4: Enqueue-dedup race condition

**What goes wrong:** Two workers both discover the same paper reference and both INSERT it into the queue simultaneously, producing two `pending` entries for the same paper ID.
**Why it happens:** Both workers read references, both do an INSERT before the other's INSERT commits.
**How to avoid:** Define `crawl_queue` with a UNIQUE index on `(paper_id, seed_paper_id)` or use `CREATE crawl_queue:⟨paper_id⟩` (record ID = paper_id) so the second insert is a no-op (SurrealDB `CREATE` is idempotent on named record IDs).
**Warning signs:** `paper_id` appears twice in `crawl_queue`, causing duplicate network fetches.

### Pitfall 5: Migration 7 must run before concurrent workers start

**What goes wrong:** If the crawl loop starts before `migrate_schema()` completes, workers attempt to INSERT into a nonexistent table.
**Why it happens:** Async task spawning before migration completes.
**How to avoid:** `migrate_schema()` is already called synchronously in `run()` before the loop — maintain this ordering. Do not move migration into background task.

---

## Code Examples

### Migration 7: crawl_queue Table

```rust
// Source: existing schema.rs pattern — add apply_migration_7
async fn apply_migration_7(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query("
        DEFINE TABLE IF NOT EXISTS crawl_queue SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS paper_id ON crawl_queue TYPE string;
        DEFINE FIELD IF NOT EXISTS seed_paper_id ON crawl_queue TYPE string;
        DEFINE FIELD IF NOT EXISTS depth_level ON crawl_queue TYPE int;
        DEFINE FIELD IF NOT EXISTS status ON crawl_queue TYPE string;
        DEFINE FIELD IF NOT EXISTS retry_count ON crawl_queue TYPE int DEFAULT 0;
        DEFINE FIELD IF NOT EXISTS created_at ON crawl_queue TYPE string;
        DEFINE FIELD IF NOT EXISTS claimed_at ON crawl_queue TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS completed_at ON crawl_queue TYPE option<string>;
        DEFINE INDEX IF NOT EXISTS idx_queue_paper_seed
            ON crawl_queue FIELDS paper_id, seed_paper_id UNIQUE;
        DEFINE INDEX IF NOT EXISTS idx_queue_status
            ON crawl_queue FIELDS status;
    ")
    .await
    .map_err(|e| ResynError::Database(format!("migration 7 DDL failed: {e}")))?;
    Ok(())
}
```

### Queue Status Summary

```rust
// Source: SurrealDB query pattern from queries.rs
pub async fn get_queue_counts(&self) -> Result<QueueCounts, ResynError> {
    let mut response = self.db
        .query("
            SELECT count() AS total,
                   count(status = 'pending') AS pending,
                   count(status = 'fetching') AS fetching,
                   count(status = 'done') AS done,
                   count(status = 'failed') AS failed
            FROM crawl_queue GROUP ALL
        ")
        .await?;
    let counts: Vec<QueueCounts> = response.take(0)?;
    Ok(counts.into_iter().next().unwrap_or_default())
}
```

### Enqueue references (idempotent via named record ID)

```rust
// Using SurrealDB record ID = paper_id makes INSERT idempotent
// "CREATE" on an existing ID is a no-op in SurrealDB
pub async fn enqueue_if_absent(&self, paper_id: &str, seed_id: &str, depth: usize)
    -> Result<(), ResynError>
{
    let paper_id = strip_version_suffix(paper_id);
    // Use INSERT ... ON DUPLICATE KEY IGNORE equivalent:
    self.db
        .query("
            INSERT INTO crawl_queue (paper_id, seed_paper_id, depth_level, status, created_at)
            VALUES ($paper_id, $seed, $depth, 'pending', time::now())
            ON DUPLICATE KEY IGNORE
        ")
        .bind(("paper_id", &paper_id))
        .bind(("seed", seed_id))
        .bind(("depth", depth))
        .await
        .map_err(|e| ResynError::Database(format!("enqueue failed: {e}")))?;
    Ok(())
}
```

**Note:** SurrealDB 3.x supports `INSERT ... ON DUPLICATE KEY IGNORE`. If this syntax is unavailable in embedded kv-surrealkv, fall back to `CREATE crawl_queue:⟨paper_id⟩ CONTENT {...}` where the record ID deduplication is the idempotency mechanism. This needs a quick spike to confirm exact syntax.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `recursive_paper_search_by_references` in-memory BFS | DB-backed queue with atomic claim | Phase 7 | Survives crashes; supports parallel workers |
| Per-source rate limiting in `ArxivHTMLDownloader` | Global shared `governor` token bucket | Phase 7 | Single rate limit source of truth across all tasks |
| Single-threaded sequential crawl | `tokio::spawn` pool bounded by `Semaphore` | Phase 7 | Parallel fetching within rate limits |
| `rate_limit_secs` CLI arg controlling per-source sleep | `governor` Quota with fixed per-source interval | Phase 7 | `--rate-limit-secs` arg can be removed or deprecated |

**Note on `rate_limit_secs` CLI arg:** Currently `CrawlArgs` has `rate_limit_secs: u64`. In Phase 7, the per-source rate limits are constants (3s arXiv, 350ms InspireHEP). The existing arg could be kept as an override for arXiv only (power users may want to slow down further), or removed. This is Claude's discretion.

---

## Open Questions

1. **`INSERT ... ON DUPLICATE KEY IGNORE` availability in SurrealDB 3 embedded**
   - What we know: The statement exists in SurrealDB 3 remote server docs
   - What's unclear: Whether kv-surrealkv embedded engine supports the same SQL dialect
   - Recommendation: Wave 0 spike — attempt the statement in a `#[tokio::test]` against `connect_memory()`, fall back to `CREATE crawl_queue:⟨paper_id⟩ CONTENT {...}` (record-ID-based dedup) if not supported

2. **`RETURN` index from nested BEGIN transaction**
   - What we know: SurrealDB transactions return results indexed by statement position
   - What's unclear: Exact index of the `RETURN` statement result when inside BEGIN/COMMIT with conditional IF blocks
   - Recommendation: Spike test to verify `.take(3)` vs `.take(4)` etc. before committing to claim function implementation

3. **`BroadcastStream` availability in `tokio-stream`**
   - What we know: `tokio_stream::wrappers::BroadcastStream` exists in tokio-stream 0.1
   - What's unclear: Whether it's behind a feature flag
   - Recommendation: Check `tokio-stream` Cargo.toml; alternative is a manual `Stream` impl using `poll_fn`

4. **Auto-tune concurrency algorithm**
   - What we know: Start at 4 concurrent tasks (locked decision)
   - What's unclear: Specific metrics and thresholds for adjustment (Claude's discretion)
   - Recommendation: Simple approach — track a rolling window of last-N fetch latencies; if P90 latency > 2x baseline, reduce by 1; if error rate == 0 for 10 successive fetches, increase by 1. Cap at some upper bound (e.g., 8 for arXiv).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness + `#[tokio::test]` |
| Config file | none (cargo-native) |
| Quick run command | `cargo test crawl_queue -- --nocapture` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CRAWL-01 | `enqueue_if_absent` inserts and deduplicates | unit | `cargo test test_queue_enqueue_dedup -- --nocapture` | Wave 0 |
| CRAWL-01 | `claim_next_pending` returns pending entry and marks it fetching | unit | `cargo test test_queue_claim -- --nocapture` | Wave 0 |
| CRAWL-01 | `claim_next_pending` returns None when queue is empty | unit | `cargo test test_queue_claim_empty -- --nocapture` | Wave 0 |
| CRAWL-02 | `reset_stale_fetching` resets fetching→pending on startup | unit | `cargo test test_queue_reset_stale -- --nocapture` | Wave 0 |
| CRAWL-02 | Crawl resumed after queue has pending entries skips completed papers | integration | `cargo test test_crawl_resume -- --nocapture` | Wave 0 |
| CRAWL-03 | Progress broadcast emits event on paper fetched | unit | `cargo test test_progress_event -- --nocapture` | Wave 0 |
| CRAWL-04 | Parallel crawl with N workers respects semaphore (max N concurrent) | unit | `cargo test test_parallel_concurrency -- --nocapture` | Wave 0 |
| CRAWL-04 | Rate limiter blocks second token within interval | unit | `cargo test test_rate_limiter_blocks -- --nocapture` | Wave 0 |

All DB tests use `connect_memory()` — no external DB required. Wiremock is available for mocking HTTP calls in CRAWL-02 integration test.

### Sampling Rate

- **Per task commit:** `cargo test crawl_queue -- --nocapture`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `resyn-core/src/database/crawl_queue.rs` — covers CRAWL-01, CRAWL-02 (new module, new tests)
- [ ] `resyn-core/src/data_aggregation/rate_limiter.rs` — covers CRAWL-04 rate limiter unit test
- [ ] `governor = "0.10"` and `axum = "0.8"` added to Cargo.toml before any implementation

---

## Sources

### Primary (HIGH confidence)

- `https://docs.rs/governor/latest/governor/_guide/index.html` — Quota construction, RateLimiter direct pattern, version 0.10.4
- `https://docs.rs/axum/latest/axum/response/sse/` — Sse struct, Event, KeepAlive API; version 0.8.8
- `https://docs.rs/axum/latest/axum/` — axum version 0.8.8; `tokio` feature enables SSE (on by default)
- `https://surrealdb.com/docs/sdk/rust/concepts/transaction` — BEGIN/COMMIT Rust SDK pattern; atomic multi-statement transactions
- `https://surrealdb.com/docs/surrealql/statements/update` — Confirmed: UPDATE has NO LIMIT clause
- `https://surrealdb.com/docs/sdk/rust/concepts/concurrency` — Arc<Surreal<Any>> clone pattern for concurrent tasks; officially documented
- `https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html` — `acquire_owned()` for cross-task permits; fairness guarantee

### Secondary (MEDIUM confidence)

- `https://github.com/tokio-rs/axum/discussions/1670` — broadcast channel + Axum SSE integration pattern (community-verified, matches official SSE docs)
- SurrealKV concurrent access pitfall — from GitHub issue #5233 and community reports; verified against official concurrent pattern recommendation

### Tertiary (LOW confidence)

- INSERT ON DUPLICATE KEY IGNORE availability in kv-surrealkv embedded — referenced in SurrealDB 3 docs but not specifically tested for embedded engine; flagged as Open Question 1

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — governor, axum, tokio all verified via official docs with specific versions
- Architecture patterns: HIGH (claim transaction pattern, SSE pattern) / MEDIUM (INSERT dedup exact syntax)
- Pitfalls: HIGH — SurrealKV concurrent connection issue from official concurrency docs + issue tracker; others derived from the architecture

**Research date:** 2026-03-15
**Valid until:** 2026-04-15 (stable crates; governor and axum are mature)
