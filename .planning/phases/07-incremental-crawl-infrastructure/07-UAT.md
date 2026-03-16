---
status: complete
phase: 07-incremental-crawl-infrastructure
source: [07-01-SUMMARY.md, 07-02-SUMMARY.md, 07-03-SUMMARY.md]
started: 2026-03-16T12:00:00Z
updated: 2026-03-16T18:05:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start Smoke Test
expected: Kill any running resyn process. Remove `./test_data` if it exists. Run `cargo run --bin resyn -- crawl -p 2503.18887 -d 1 --db surrealkv://./test_data`. Application boots, connects to DB, enqueues seed, fetches paper and references, completes with summary log showing papers_found > 0.
result: issue
reported: "1 paper is found but I get these warnings: WARN resyn_core::data_aggregation::html_parser: HTML download failed for : builder error / WARN resyn::commands::crawl: Failed to fetch references paper_id=\"2503.18887\" error=HTML download error: : builder error"
severity: major

### 2. Queue Status Command
expected: After a completed crawl, run `cargo run --bin resyn -- crawl status --db surrealkv://./test_data`. Prints queue counts showing total, pending, fetching, done, and failed — with done > 0.
result: issue
reported: "error: unexpected argument '--db' found"
severity: major

### 3. Crawl Resume After Interrupt
expected: Start a depth-2 crawl: `cargo run --bin resyn -- crawl -p 2503.18887 -d 2 --db surrealkv://./test_data`. Ctrl+C mid-crawl. Re-run the same command. Log shows "Resuming crawl from existing queue" (or similar resume message) and does not re-fetch already-completed papers.
result: skipped
reason: Blocked by Test 1 — reference fetching broken, cannot test crawl behavior

### 4. Queue Clear Command
expected: Run `cargo run --bin resyn -- crawl clear --db surrealkv://./test_data`. Then run `crawl status`. Queue shows 0 entries across all categories.
result: skipped
reason: Blocked by Test 2 — --db flag not recognized by subcommands

### 5. Queue Retry Command
expected: After a crawl that has failed entries (or manually after clear+partial crawl), run `cargo run --bin resyn -- crawl retry --db surrealkv://./test_data`. Failed entries are marked back to pending. Status confirms pending count increased.
result: skipped
reason: Blocked by Test 2 — --db flag not recognized by subcommands

### 6. SSE Progress Streaming
expected: Start crawl with `--progress`: `cargo run --bin resyn -- crawl -p 2503.18887 -d 1 --db surrealkv://./test_data --progress`. In another terminal: `curl -N http://localhost:3001/progress`. JSON SSE events stream with fields like papers_found, papers_pending, current_paper_id. Stream ends with an event where event_type is "complete".
result: skipped
reason: Blocked by Test 1 — reference fetching broken, crawl produces no meaningful progress events

### 7. Parallel Workers
expected: Run with parallel flag: `cargo run --bin resyn -- crawl -p 2503.18887 -d 2 --db surrealkv://./test_data --parallel=2`. Log timestamps show overlapping paper fetches bounded at 2 concurrent workers. Crawl completes successfully.
result: skipped
reason: Blocked by Test 1 — reference fetching broken, cannot test parallel crawl

## Summary

total: 7
passed: 0
issues: 2
pending: 0
skipped: 5

## Gaps

- truth: "Application fetches seed paper and its references without errors"
  status: failed
  reason: "User reported: 1 paper is found but HTML download failed for references with 'builder error' — fetch_references fails for seed paper 2503.18887"
  severity: major
  test: 1
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
- truth: "Queue status subcommand accepts --db flag and prints queue counts"
  status: failed
  reason: "User reported: error: unexpected argument '--db' found"
  severity: major
  test: 2
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
