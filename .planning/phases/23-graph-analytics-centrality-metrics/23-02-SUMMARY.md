---
phase: 23-graph-analytics-centrality-metrics
plan: "02"
subsystem: graph-analytics
tags: [pagerank, betweenness, centrality, server-fns, analysis-pipeline]
dependency_graph:
  requires: [23-01]
  provides: [graph-analytics-computation, metrics-server-fns, auto-compute-pipeline]
  affects: [resyn-core/graph_analytics, resyn-app/server_fns/metrics, resyn-app/server_fns/analysis]
tech_stack:
  added: [petgraph::algo::page_rank, Brandes betweenness algorithm]
  patterns: [spawn_blocking for CPU-bound work, corpus fingerprint caching, SSR feature-gated server fns]
key_files:
  created:
    - resyn-core/src/graph_analytics/mod.rs
    - resyn-core/src/graph_analytics/pagerank.rs
    - resyn-core/src/graph_analytics/betweenness.rs
    - resyn-app/src/server_fns/metrics.rs
  modified:
    - resyn-core/src/lib.rs
    - resyn-app/src/server_fns/mod.rs
    - resyn-app/src/server_fns/analysis.rs
decisions:
  - "Reuse resyn_core::nlp::tfidf::corpus_fingerprint for metrics fingerprint — avoids adding sha2 dep to resyn-app"
  - "Stage 5 metrics compute placed before analysis_complete broadcast — ensures metrics ready when UI polls"
  - "betweenness runs in spawn_blocking — isolates O(VE) CPU work from async runtime per T-23-05"
metrics:
  duration: "~15 minutes"
  completed_date: "2026-04-09"
  tasks_completed: 2
  tasks_total: 2
  files_created: 4
  files_modified: 3
  tests_added: 10
---

# Phase 23 Plan 02: Graph Analytics Computation + Server Functions Summary

PageRank (petgraph::algo::page_rank, 0.85 damping / 50 iterations) and Brandes betweenness centrality (O(VE), directed normalization (n-1)(n-2)) with three server functions and silent auto-compute wired into the analysis pipeline.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | PageRank + Betweenness computation module with tests | 87fdc85 | resyn-core/src/graph_analytics/{mod,pagerank,betweenness}.rs, resyn-core/src/lib.rs |
| 2 | Metrics server functions + analysis pipeline auto-compute | 88d4ca0 | resyn-app/src/server_fns/metrics.rs, mod.rs, analysis.rs |

## What Was Built

### Task 1: graph_analytics module

- `resyn-core/src/graph_analytics/pagerank.rs` — `compute_pagerank` wraps `petgraph::algo::page_rank` with damping factor 0.85 and 50 iterations. Returns `HashMap<String, f32>` keyed by version-stripped arxiv_id.
- `resyn-core/src/graph_analytics/betweenness.rs` — `compute_betweenness` implements Brandes' algorithm. BFS-based O(VE) traversal with back-propagation pass. Normalizes by `(n-1)(n-2)` for directed graphs. Returns `HashMap<String, f32>`.
- `resyn-core/src/lib.rs` — registered `pub mod graph_analytics` (always available, WASM-safe; no SSR gate needed since algorithms only use petgraph).
- 10 tests covering: empty graph, single node, chain ordering (B > A in PageRank chain; B highest betweenness in A→B→C), disconnected all-zeros, normalized range [0,1], (n-1)(n-2) normalization constant verification, 2-node edge case.

### Task 2: Server functions + pipeline wiring

- `resyn-app/src/server_fns/metrics.rs`:
  - `RankedPaperEntry` — arxiv_id, title, year, pagerank for UI ranked list
  - `MetricsStatus` — `Ready { corpus_fingerprint }` or `NotAvailable`
  - `GetTopPageRankPapers` — fetches top-N by pagerank, joins with paper metadata
  - `GetMetricsStatus` — cheap 1-row probe to check if metrics exist
  - `TriggerMetricsCompute` — spawns background task calling `compute_and_store_metrics`
  - `compute_and_store_metrics` — full computation: loads papers, builds graph, runs PageRank on async thread, runs betweenness in `spawn_blocking`, upserts all results to `graph_metrics` table with corpus fingerprint
- `resyn-app/src/server_fns/mod.rs` — added `pub mod metrics`
- `resyn-app/src/server_fns/analysis.rs` — Stage 5 silent metrics compute injected before `analysis_complete` broadcast, non-fatal (errors logged but pipeline continues)

## Verification

- `cargo check --all-targets` — passes cleanly (1m 30s)
- `cargo test -p resyn-core --lib` — 101 tests pass (91 existing + 10 new graph_analytics)
- `cargo test -p resyn-core --lib graph_analytics` — 10/10 pass

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing `NodeIndexable` trait import for `to_index`**
- **Found during:** Task 1 RED compilation
- **Issue:** `graph.to_index(idx)` failed — `NodeIndexable` trait not in scope
- **Fix:** Added `use petgraph::visit::NodeIndexable` to pagerank.rs
- **Files modified:** resyn-core/src/graph_analytics/pagerank.rs
- **Commit:** 87fdc85 (fixed before commit)

**2. [Rule 1 - Bug] Unused import `IntoNeighbors` in betweenness.rs**
- **Found during:** Task 1 RED compilation
- **Issue:** `use petgraph::visit::IntoNeighbors` was unused — compiler warning (clippy -Dwarnings would fail CI)
- **Fix:** Removed unused import; `graph.neighbors(v)` works without explicit trait import
- **Files modified:** resyn-core/src/graph_analytics/betweenness.rs
- **Commit:** 87fdc85 (fixed before commit)

**3. [Rule 2 - Missing dependency] No sha2 in resyn-app for corpus fingerprint**
- **Found during:** Task 2 implementation
- **Issue:** Plan called for `Sha256::digest` directly in metrics.rs, but `sha2` is not a resyn-app dependency
- **Fix:** Reused `resyn_core::nlp::tfidf::corpus_fingerprint` — same function, avoids new dep, consistent with NLP/similarity fingerprint approach
- **Files modified:** resyn-app/src/server_fns/metrics.rs
- **Commit:** 88d4ca0

## Known Stubs

None. All server functions are fully implemented and wired.

## Threat Surface Scan

No new network endpoints or auth paths introduced beyond what the plan's threat model already covers. `trigger_metrics_compute`, `get_metrics_status`, and `get_top_pagerank_papers` are standard Leptos server functions following the same pattern as existing endpoints. All threats documented in plan (T-23-03, T-23-04, T-23-05) are mitigated as specified.

## Self-Check: PASSED
