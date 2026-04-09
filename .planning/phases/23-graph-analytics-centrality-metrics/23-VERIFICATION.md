---
phase: 23-graph-analytics-centrality-metrics
verified: 2026-04-09T00:00:00Z
status: human_needed
score: 9/9
overrides_applied: 0
human_verification:
  - test: "Navigate to the Graph page, observe the 'Size by' dropdown in graph controls"
    expected: "Dropdown has 4 options: Uniform, PageRank, Betweenness, Citations. PageRank and Betweenness show '(computing...)' or are disabled/grayed when metrics have not been computed yet."
    why_human: "Disabled state and spinner rendering are visual DOM states that cannot be verified without a browser"
  - test: "After a crawl completes, switch between Uniform, PageRank, Betweenness, and Citations in the Size by dropdown"
    expected: "Graph nodes animate smoothly to new sizes (~300ms transition) as the metric changes — lerp interpolation visible"
    why_human: "Animation smoothness is a temporal/visual property not verifiable by static analysis"
  - test: "Hover a graph node while PageRank is selected in the Size by dropdown"
    expected: "Tooltip shows the paper's PageRank score formatted as 'PageRank: 0.XXXX'"
    why_human: "Tooltip rendering on hover requires real browser interaction"
  - test: "Click the recompute button (looping arrow icon) next to the Size by dropdown"
    expected: "Button shows a spinner briefly while computation runs, then returns to the arrow icon"
    why_human: "Spinner state transition requires live browser observation"
  - test: "Navigate to the Dashboard page"
    expected: "A 6th card 'Most Influential Papers' appears showing up to 5 ranked entries, each with rank number, truncated title, year, and PR score. A 'View all ->' link appears at the bottom."
    why_human: "Card layout, truncation, and rank formatting require visual confirmation"
---

# Phase 23: Graph Analytics — Centrality & Metrics Verification Report

**Phase Goal:** Users can explore which papers are most structurally influential using PageRank and betweenness centrality, and node sizes reflect the chosen metric
**Verified:** 2026-04-09
**Status:** human_needed — all automated checks pass; 5 visual/interactive behaviors require human confirmation
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | System computes PageRank and betweenness centrality for all papers and caches results in SurrealDB with corpus-fingerprint invalidation | VERIFIED | `compute_pagerank` in `resyn-core/src/graph_analytics/pagerank.rs` wraps `petgraph::algo::page_rank`. `compute_betweenness` in `betweenness.rs` implements Brandes O(VE). `compute_and_store_metrics` in `metrics.rs` upserts to `graph_metrics` table with corpus fingerprint from `resyn_core::nlp::tfidf::corpus_fingerprint`. 9 DB tests pass (migration 11, upsert/get roundtrip, idempotency, version-stripping, get_top_by_pagerank). |
| 2 | User can select a "Size by" dropdown (Uniform / PageRank / Betweenness / Citations) and graph nodes resize accordingly | VERIFIED (code) + human needed | `SizeMode` enum with 4 variants exists in `layout_state.rs`. `graph_controls.rs` has the dropdown with disabled states and spinner. `graph.rs` has `LERP_FACTOR = 0.15` in RAF loop. `update_node_target_radii()` maps metric scores to radii. Renderers use `current_radius`. Visual animation requires human confirmation. |
| 3 | Dashboard shows a "Most Influential Papers" ranking panel ordered by PageRank score | VERIFIED (code) + human needed | `InfluentialPapersCard` in `dashboard.rs` uses its own `Resource::new(|| (), \|_\| get_top_pagerank_papers(5))` + `Suspense`. Shows rank number, title (truncated at 50 chars), year, PR score (`2021 · PR: 0.043` format). "View all →" link to /papers. CSS classes in `main.css`. Visual layout requires human confirmation. |
| 4 | Citation graph queries use single SurrealDB JOINs rather than per-paper N+1 lookups | VERIFIED | `get_cited_papers` uses `FROM paper WHERE id IN (SELECT VALUE out FROM cites WHERE in = $rid)`. `get_citing_papers` uses `FROM paper WHERE id IN (SELECT VALUE in FROM cites WHERE out = $rid)`. No `self.get_paper` call inside a loop present. |

**Score:** 9/9 individual truths verified (automated); 5 items require human visual confirmation

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `resyn-core/src/datamodels/graph_metrics.rs` | GraphMetrics struct with pagerank, betweenness, corpus_fingerprint, computed_at | VERIFIED | `pub struct GraphMetrics` with all required `f32`/`String` fields, `Default + Serialize + Deserialize + PartialEq` |
| `resyn-core/src/database/schema.rs` | Migration 11 creating graph_metrics SCHEMAFULL table with TYPE float fields | VERIFIED | `apply_migration_11` at line 234, `DEFINE TABLE IF NOT EXISTS graph_metrics SCHEMAFULL`, `TYPE float` for pagerank and betweenness |
| `resyn-core/src/database/queries.rs` | GraphMetricsRepository with upsert/get/get_all/get_top_by_pagerank | VERIFIED | `pub struct GraphMetricsRepository<'a>` at line 1094, all 4 methods present, `ORDER BY pagerank DESC LIMIT $limit` |
| `resyn-core/src/graph_analytics/pagerank.rs` | compute_pagerank wrapping petgraph::algo::page_rank | VERIFIED | `pub fn compute_pagerank`, `page_rank(graph, 0.85_f32, 50)`, `strip_version_suffix` applied |
| `resyn-core/src/graph_analytics/betweenness.rs` | Brandes betweenness centrality on StableGraph | VERIFIED | `pub fn compute_betweenness`, `((n - 1) * (n - 2)) as f32` directed normalization |
| `resyn-app/src/server_fns/metrics.rs` | GetTopPageRankPapers, GetMetricsStatus, TriggerMetricsCompute server fns | VERIFIED | All 3 `#[server(...)]` macros present plus `compute_and_store_metrics` shared fn, `spawn_blocking` for betweenness, corpus fingerprint via `resyn_core::nlp::tfidf::corpus_fingerprint` |
| `resyn-app/src/graph/layout_state.rs` | SizeMode enum, target_radius/current_radius on NodeState | VERIFIED | `pub enum SizeMode` with Uniform/PageRank/Betweenness/Citations at line 20, `target_radius: f64` at line 41, `current_radius: f64` at line 43, `size_mode: SizeMode` on `GraphState` |
| `resyn-app/src/components/graph_controls.rs` | Size by dropdown with disabled states and recompute button | VERIFIED | `size_mode: RwSignal<SizeMode>` parameter, "Size by" group, `prop:disabled=move \|\| !metrics_ready.get()`, `title="Recompute centrality metrics"`, `trigger_metrics_compute` call |
| `resyn-app/src/pages/dashboard.rs` | InfluentialPapersCard component showing top 5 ranked papers | VERIFIED | `fn InfluentialPapersCard()` at line 103, its own Resource + Suspense, `get_top_pagerank_papers(5)`, `PR:` format string, "View all →" |
| `resyn-app/style/main.css` | .influential-list and related CSS classes | VERIFIED | `.influential-list` at line 2019, `.influential-entry`, `.influential-rank`, `.influential-title`, `.influential-meta` all present |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `resyn-core/src/database/queries.rs` | `resyn-core/src/datamodels/graph_metrics.rs` | `use crate::datamodels::graph_metrics::GraphMetrics` | WIRED | Import confirmed; GraphMetricsRepository uses GraphMetrics as return type |
| `resyn-core/src/database/schema.rs` | graph_metrics table | `apply_migration_11` DDL | WIRED | `DEFINE TABLE IF NOT EXISTS graph_metrics SCHEMAFULL` in migration 11 |
| `resyn-core/src/graph_analytics/pagerank.rs` | `petgraph::algo::page_rank` | direct call | WIRED | `page_rank(graph, 0.85_f32, 50)` at line 18 |
| `resyn-app/src/server_fns/metrics.rs` | `GraphMetricsRepository::new` | use in server fn | WIRED | `GraphMetricsRepository::new(&db)` in all 3 server functions |
| `resyn-app/src/server_fns/analysis.rs` | `compute_and_store_metrics` | Stage 5 silent call | WIRED | `use crate::server_fns::metrics::compute_and_store_metrics` at line 365, called at line 366 |
| `resyn-app/src/components/graph_controls.rs` | `resyn-app/src/graph/layout_state.rs` | `RwSignal<SizeMode>` | WIRED | `size_mode: RwSignal<SizeMode>` parameter, `size_mode.set(match ...)` in on:change |
| `resyn-app/src/pages/graph.rs` | `resyn-app/src/server_fns/metrics.rs` | `get_metrics_status` Resource | WIRED | `use crate::server_fns::metrics::{get_metrics_pairs, get_metrics_status, MetricsStatus}` at line 23 |
| `resyn-app/src/pages/dashboard.rs` | `resyn-app/src/server_fns/metrics.rs` | `get_top_pagerank_papers` Resource | WIRED | `use crate::server_fns::metrics::get_top_pagerank_papers` at line 4, `Resource::new(...)` at line 104 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|--------------------|--------|
| `InfluentialPapersCard` | `top_papers` Resource | `get_top_pagerank_papers(5)` → `GraphMetricsRepository::get_top_by_pagerank` → `SELECT * FROM graph_metrics ORDER BY pagerank DESC LIMIT $limit` | Yes — SurrealDB query, not static | FLOWING |
| `graph_controls.rs` size_mode dropdown | `metrics_ready` signal | `get_metrics_status` → `GraphMetricsRepository::get_top_by_pagerank(1)` as probe | Yes — live DB check | FLOWING |
| `pages/graph.rs` node sizing | `GraphState.metrics` HashMap | `get_metrics_pairs()` → `get_all_metrics()` → `SELECT * FROM graph_metrics` | Yes — full DB read | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| GraphMetrics serde roundtrip | `cargo test -p resyn-core --lib graph_metrics` | 2/2 pass | PASS |
| Graph analytics algorithms (PageRank + Betweenness) | `cargo test -p resyn-core --lib graph_analytics` | 10/10 pass | PASS |
| GraphMetricsRepository DB operations (SSR) | `cargo test -p resyn-core --features ssr --lib graph_metrics` | 9/9 pass | PASS |
| Full test suite | `cargo test -p resyn-core --features ssr --lib` | 239/239 pass | PASS |
| Full compilation | `cargo check --all-targets` | 0 errors | PASS |
| Node size animation (lerp) | Requires running browser | — | SKIP (needs human) |
| Disabled state for PageRank/Betweenness when no metrics | Requires running browser | — | SKIP (needs human) |
| Metric score in node tooltip | Requires running browser | — | SKIP (needs human) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| GANA-01 | 23-02 | System computes PageRank on the citation graph via power iteration | SATISFIED | `compute_pagerank` wraps `petgraph::algo::page_rank` with 0.85 damping / 50 iterations; 3 PageRank tests pass |
| GANA-02 | 23-02 | System computes betweenness centrality via Brandes' algorithm | SATISFIED | `compute_betweenness` implements Brandes O(VE) with directed normalization (n-1)(n-2); 7 betweenness tests pass |
| GANA-03 | 23-01, 23-02 | Graph metrics cached in SurrealDB with corpus-fingerprint invalidation | SATISFIED | `graph_metrics` SCHEMAFULL table via migration 11; `compute_and_store_metrics` checks `first.corpus_fingerprint == fingerprint` before recomputing |
| GANA-04 | 23-03 | User can size graph nodes by metric via "Size by" dropdown | SATISFIED (code) | `SizeMode` enum, dropdown with 4 options, lerp animation, `update_node_target_radii()`; visual confirmation pending |
| GANA-05 | 23-03 | Dashboard displays "Most Influential Papers" ranking by PageRank | SATISFIED (code) | `InfluentialPapersCard` with top-5, rank, title, year, PR score; visual confirmation pending |
| GANA-06 | 23-01 | N+1 citation queries replaced with single SurrealDB JOINs | SATISFIED | `get_cited_papers` and `get_citing_papers` use `IN (SELECT VALUE ...)` subquery — no loop calling `self.get_paper` |

All 6 GANA requirements from REQUIREMENTS.md traceability table are covered. No orphaned requirements.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| — | — | — | — | None found |

No TODOs, FIXMEs, placeholder returns, or hardcoded empty values found in phase-modified files. All server functions have real implementations, not stubs.

### Human Verification Required

#### 1. Size by dropdown visual states

**Test:** Start the app with `cargo run --bin resyn-server -- --db surrealkv://./data`. Navigate to the Graph page. Inspect the "Size by" dropdown in graph controls.
**Expected:** Dropdown visible with 4 options (Uniform, PageRank, Betweenness, Citations). Before metrics are computed, PageRank and Betweenness show "(computing...)" label and are non-selectable/grayed out.
**Why human:** Disabled state and label rendering are DOM-level visual properties not verifiable by static code analysis.

#### 2. Animated node size transitions

**Test:** After a crawl completes and metrics are computed, switch between dropdown options (Uniform → PageRank → Betweenness → Citations).
**Expected:** Graph nodes smoothly animate to their new sizes over approximately 300ms — no instant jump, visible lerp interpolation.
**Why human:** Animation smoothness is a temporal visual property; LERP_FACTOR=0.15 code exists but perceptual quality requires live observation.

#### 3. Metric score in node tooltip

**Test:** With PageRank selected in the Size by dropdown, hover over any graph node.
**Expected:** Tooltip shows the paper's PageRank score on a line formatted as "PageRank: 0.XXXX".
**Why human:** Tooltip rendering on hover requires browser interaction; tooltip content code exists in `node_tooltip()` but cannot be triggered statically.

#### 4. Recompute button spinner state

**Test:** Click the recompute button (↺ icon) next to the Size by dropdown.
**Expected:** Button shows a spinner briefly while computation runs (server-side is background-spawned so should return quickly), then returns to the ↺ icon. Button disabled while computing.
**Why human:** Spinner appearance and state transition require live browser observation.

#### 5. "Most Influential Papers" dashboard card

**Test:** Navigate to the Dashboard page.
**Expected:** A 6th card titled "Most Influential Papers" appears after the 5 existing summary cards. Shows up to 5 ranked entries with: rank number (e.g., "1."), truncated title, year, PR score (e.g., "2021 · PR: 0.043"). "View all →" link at the bottom of the card.
**Why human:** Card layout, ranking order display, and truncation behavior require visual confirmation.

### Gaps Summary

No automated gaps found. All phase artifacts exist, are substantive, and are wired to real data sources. The 239-test suite passes cleanly. `cargo check --all-targets` reports 0 errors.

The only outstanding items are visual/interactive UI behaviors in Plan 03 that require a human to confirm in a running browser instance. The Plan 03 checkpoint task (Task 3) was auto-approved during execution; this verification restores the human gate.

---

_Verified: 2026-04-09_
_Verifier: Claude (gsd-verifier)_
