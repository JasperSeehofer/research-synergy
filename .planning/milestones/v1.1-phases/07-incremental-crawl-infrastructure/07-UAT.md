---
status: diagnosed
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
  root_cause: "paper.pdf_url is empty string for papers from arxiv-rs. Empty string flows through convert_pdf_url_to_html_url (no guard) → reqwest refuses to build request for empty URL → 'builder error'. The crawl loop's make_source() creates ArxivSource with zero rate limit but the ArxivHTMLDownloader gets an empty URL because Paper::from_arxiv_paper copies pdf_url from arxiv-rs without validation."
  artifacts:
    - path: "resyn-core/src/data_aggregation/arxiv_utils.rs"
      issue: "convert_pdf_url_to_html_url has no empty-string guard (line 14, line 71)"
    - path: "resyn-core/src/datamodels/paper.rs"
      issue: "Paper::from_arxiv_paper copies pdf_url without validating non-empty"
  missing:
    - "Guard in aggregate_references_for_arxiv_paper: return Err if pdf_url is empty"
    - "Fallback: construct HTML URL from paper ID when pdf_url is empty (https://arxiv.org/html/{id})"
  debug_session: ".planning/debug/fetch-references-empty-url.md"
- truth: "Queue status subcommand accepts --db flag and prints queue counts"
  status: failed
  reason: "User reported: error: unexpected argument '--db' found"
  severity: major
  test: 2
  root_cause: "clap subcommand dispatch closes parent arg scanner once subcommand token is consumed. CrawlSubcommand variants (Status/Clear/Retry) are unit variants with zero fields, so --db placed after subcommand name has no parser to match against. --db is only reachable when placed before the subcommand name."
  artifacts:
    - path: "resyn-server/src/commands/crawl.rs"
      issue: "CrawlSubcommand variants are unit structs (lines 34-42) — need --db field on each variant, or use clap's flatten/global arg"
  missing:
    - "Add --db field to each CrawlSubcommand variant, or use #[arg(global = true)] on CrawlArgs.db"
  debug_session: ".planning/debug/crawl-subcommand-db-flag.md"
