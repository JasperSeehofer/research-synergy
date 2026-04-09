---
phase: 23-graph-analytics-centrality-metrics
plan: 01
subsystem: resyn-core/database
tags: [graph-analytics, centrality, surrealldb, migration, repository, n+1-fix]
dependency_graph:
  requires: []
  provides: [GraphMetrics, GraphMetricsRepository, graph_metrics-table, optimized-citation-queries]
  affects: [resyn-core/src/database/queries.rs, resyn-core/src/database/schema.rs, resyn-core/src/datamodels]
tech_stack:
  added: []
  patterns: [repository-pattern, surrealldb-upsert, migration-versioning]
key_files:
  created:
    - resyn-core/src/datamodels/graph_metrics.rs
  modified:
    - resyn-core/src/datamodels/mod.rs
    - resyn-core/src/database/schema.rs
    - resyn-core/src/database/queries.rs
decisions:
  - "Store pagerank/betweenness as f64 in SurrealDB bind (TYPE float = f64) then cast to f32 on read — matches SurrealDB's float type precision"
  - "N+1 get_cited_papers/get_citing_papers replaced with IN subquery — no BFS get_citation_graph refactor (unused in Leptos path per D-12)"
  - "GraphMetricsRepository follows SimilarityRepository pattern: serde_json::Value deserialization with .as_f64() cast to f32"
metrics:
  duration: ~20 minutes
  completed: 2026-04-09
  tasks_completed: 2
  files_changed: 4
---

# Phase 23 Plan 01: GraphMetrics Data Layer Summary

GraphMetrics struct, SurrealDB migration 11, GraphMetricsRepository with upsert/get/get_all/get_top_by_pagerank, and N+1 citation query refactor to single IN-subquery.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | GraphMetrics data model + Migration 11 + GraphMetricsRepository | 22efcdb, c247f3d | graph_metrics.rs (new), mod.rs, schema.rs, queries.rs |
| 2 | Refactor N+1 citation queries to single SurrealDB JOINs | c247f3d | queries.rs |

## What Was Built

### Task 1: GraphMetrics Data Layer

**`resyn-core/src/datamodels/graph_metrics.rs`** — New file. `GraphMetrics` struct with `arxiv_id: String`, `pagerank: f32`, `betweenness: f32`, `corpus_fingerprint: String`, `computed_at: String`. Derives `Debug, Clone, Default, Serialize, Deserialize, PartialEq`. Includes 2 unit tests (serde roundtrip, default values).

**`resyn-core/src/database/schema.rs`** — Added `apply_migration_11` creating `graph_metrics` SCHEMAFULL table with `TYPE float` for `pagerank` and `betweenness`, `TYPE string` for other fields, and a UNIQUE index on `arxiv_id`. Updated `migrate_schema` to call migration 11. Added `test_migration_11_creates_graph_metrics_table` test.

**`resyn-core/src/database/queries.rs`** — Added `GraphMetricsRepository<'a>` with:
- `upsert_metrics`: UPSERT using `type::record('graph_metrics', $id)`, binds floats as `f64`, calls `strip_version_suffix`
- `get_metrics`: SELECT from record ID, parses floats via `.as_f64() as f32`
- `get_all_metrics`: SELECT * FROM graph_metrics
- `get_top_by_pagerank(limit)`: ORDER BY pagerank DESC LIMIT $limit

Added `graph_metrics_tests` module with 7 DB tests (roundtrip, None-for-missing, get_all, top_by_pagerank order, upsert idempotency, version suffix stripping). Updated all schema version assertions from `10` to `11`.

### Task 2: N+1 Citation Query Refactor

Replaced 2-step N+1 pattern in `get_cited_papers` and `get_citing_papers` with single-query IN-subquery approach:

- `get_cited_papers`: `SELECT ... FROM paper WHERE id IN (SELECT VALUE out FROM cites WHERE in = $rid)`
- `get_citing_papers`: `SELECT ... FROM paper WHERE id IN (SELECT VALUE in FROM cites WHERE out = $rid)`

No N+1 loop (`for to_id in to_ids { self.get_paper(&to_id) }`) remains. `get_citation_graph` BFS was intentionally not refactored per plan D-12 (unused in Leptos app path).

## Verification

```
cargo test -p resyn-core --features ssr --lib
running 229 tests
test result: ok. 229 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Graph-metrics specific:
- `test_graph_metrics_default` — ok
- `test_graph_metrics_serde_roundtrip` — ok
- `test_metrics_upsert_get_roundtrip` — ok
- `test_metrics_get_nonexistent_returns_none` — ok
- `test_metrics_get_all` — ok
- `test_metrics_get_top_by_pagerank` — ok
- `test_metrics_upsert_idempotent` — ok
- `test_metrics_version_suffix_stripped` — ok
- `test_migration_11_creates_graph_metrics_table` — ok

## Deviations from Plan

None — plan executed exactly as written.

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns introduced. All queries use parameterized `.bind()` — no string interpolation. `strip_version_suffix()` applied in `upsert_metrics` per T-23-01 mitigation.

## Known Stubs

None — this plan is data-layer only (struct, migration, repository). No UI rendering path.

## Self-Check: PASSED

Files created:
- resyn-core/src/datamodels/graph_metrics.rs — FOUND
- .planning/phases/23-graph-analytics-centrality-metrics/23-01-SUMMARY.md — (this file)

Commits:
- 22efcdb — FOUND (GraphMetrics model, migration 11)
- c247f3d — FOUND (GraphMetricsRepository, N+1 refactor, version assertions)
