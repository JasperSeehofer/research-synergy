---
phase: 28-forward-citation-crawl-mode-s2
plan: "04"
subsystem: crawler-integration
tags: [bidirectional-crawl, cli-flags, worker-loop, semantic-scholar, documentation]
dependency_graph:
  requires: [28-01, 28-02, 28-03]
  provides: [bidirectional-crawl-end-to-end]
  affects: [resyn-server/src/commands/crawl.rs, resyn-core/src/data_aggregation/semantic_scholar_api.rs, scripts/crawl-feynman-seeds.sh, CLAUDE.md]
tech_stack:
  added: []
  patterns: [builder-chain-threading, worker-loop-conditional-block, warn-and-continue]
key_files:
  created:
    - scripts/crawl-feynman-seeds.sh
  modified:
    - resyn-server/src/commands/crawl.rs
    - resyn-core/src/data_aggregation/semantic_scholar_api.rs
    - CLAUDE.md
decisions:
  - "Non-S2 warn uses source_name() == semantic_scholar || last_resolving_source() == semantic_scholar for chained-source detection"
  - "bidirectional block inserted after fetch_references, before enqueue-references loop — matches plan spec"
  - "scripts/crawl-feynman-seeds.sh created from scratch (did not exist in repo prior to this plan)"
metrics:
  duration: "~60 minutes"
  completed: "2026-04-27"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 4
---

# Phase 28 Plan 04: Crawler Integration Summary

End-to-end bidirectional crawl mode wired: `--bidirectional` and `--max-forward-citations` CLI flags thread through `make_single_source` into the `SemanticScholarSource` builder, the worker loop calls `fetch_citing_papers` + `upsert_inverse_citations_batch` + enqueue for S2 sources, non-S2 sources emit a structured `tracing::warn!`, and CLAUDE.md is refreshed.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add trait impl override + CrawlArgs flags + make_single_source threading | 24e0e23 | semantic_scholar_api.rs, crawl.rs |
| 2 | Bidirectional block in worker loop + script + CLAUDE.md | ee64111 | crawl.rs, crawl-feynman-seeds.sh, CLAUDE.md |

## Key Deliverables

### Worker-loop insertion point
The bidirectional block is inserted at line ~376 in `crawl.rs`, immediately after the `fetch_references` block and before the `enqueue references` loop. The ordering is:
1. `fetch_paper` → `fetch_references` → **bidirectional block** → enqueue refs → `upsert_paper` → `upsert_citations`

### Final signature of make_single_source and make_source
```rust
fn make_single_source(
    name: &str,
    bidirectional: bool,
    max_forward_citations: usize,
) -> Box<dyn PaperSource>

fn make_source(
    source_spec: &str,
    bidirectional: bool,
    max_forward_citations: usize,
) -> Box<dyn PaperSource>
```
Both values captured from `args` before `join_set.spawn` and moved into the closure.

### Trait override on SemanticScholarSource
```rust
async fn fetch_citing_papers(&mut self, paper: &mut Paper) -> Result<(), ResynError> {
    self.fetch_citing_papers_inner(paper).await
}
```
Added inside `impl PaperSource for SemanticScholarSource`, immediately before `fn source_name`.

### last_resolving_source chain detection
The non-S2 warn check uses:
```rust
let supports = source.source_name() == "semantic_scholar"
    || source.last_resolving_source() == "semantic_scholar";
```
This correctly handles chained sources where S2 was the resolver.

### CLAUDE.md changes
- Crawl table: added `--bidirectional` and `--max-forward-citations` rows; updated `--source` row to list `semantic_scholar` and comma-chain
- Removed stale `ChainedPaperSource empty-refs bug (KNOWN, unfixed as of 2026-04-20)` paragraph
- Added `Bidirectional crawl mode (--bidirectional, S2 only)` Important Notes bullet

### Smoke test outcome
```
papers_found=4, papers_failed=8, elapsed=58s
```
8 failures were S2 429 rate-limit errors (expected without `S2_API_KEY`). 4 papers fetched and persisted. DB at `/tmp/resyn-bidir-test/` is non-empty (LOCK, manifest, sstables, vlog, wal present). No panics.

The `crawl-feynman-seeds.sh` script was created from scratch (did not exist in the repo before this plan).

## Deviations from Plan

### Auto-fixed Issues

None.

### Scope Notes

- `scripts/crawl-feynman-seeds.sh` did not exist anywhere in the repo (plan assumed it existed). Created it with the two Feynman/cond-mat seed IDs from the context, matching the plan's expected `--bidirectional` invocation pattern. Deviation: Rule 3 (blocking — the plan required a `--bidirectional` line in a script that didn't exist).
- 7 pre-existing migration test failures (`test_migrate_schema_*` in `resyn-core/src/database/queries.rs`) were observed during `cargo test --workspace`. These are not caused by plan-04 changes (our commits touch only `semantic_scholar_api.rs`, `crawl.rs`, `crawl-feynman-seeds.sh`, `CLAUDE.md`). Logged to deferred items; out of scope.

## Known Stubs

None — all code paths are wired to real implementations.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes beyond what is documented in the plan's threat model.

## Self-Check

Files exist:
- `resyn-server/src/commands/crawl.rs` — FOUND
- `resyn-core/src/data_aggregation/semantic_scholar_api.rs` — FOUND
- `scripts/crawl-feynman-seeds.sh` — FOUND
- `CLAUDE.md` — FOUND
- `.planning/phases/28-forward-citation-crawl-mode-s2/28-04-SUMMARY.md` — FOUND (this file)

Commits exist:
- 24e0e23 — FOUND
- ee64111 — FOUND
