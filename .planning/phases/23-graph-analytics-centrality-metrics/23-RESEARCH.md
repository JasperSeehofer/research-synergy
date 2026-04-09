# Phase 23: Graph Analytics — Centrality & Metrics - Research

**Researched:** 2026-04-09
**Domain:** Graph centrality algorithms (PageRank, Brandes betweenness), SurrealDB schema migration, Leptos reactive UI patterns, node-size animation
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** "Size by" dropdown added to the existing graph controls overlay (alongside edge toggles, force mode, label mode). Options: Uniform / PageRank / Betweenness / Citations
- **D-02:** Switching metrics triggers animated lerp transition (~300ms) on node sizes, consistent with existing viewport_fit lerp pattern
- **D-03:** Hovering a node shows the active metric's raw score (e.g. "PageRank: 0.042") in the tooltip/label overlay
- **D-04:** New (6th) dashboard card "Most Influential Papers" showing top-5 papers ranked by PageRank score
- **D-05:** Each ranked entry displays: PageRank score (formatted), paper title, publication year — consistent with existing SummaryCard density
- **D-06:** Card links to full ranking view via "View all →" pattern (matching other dashboard cards)
- **D-07:** Metrics auto-compute after crawl completes (background, no user action needed), plus a manual "Recompute" button for forcing refresh
- **D-08:** Subtle progress indicator — small spinner or badge on the "Size by" dropdown until metrics are available
- **D-09:** Metrics cached in SurrealDB with corpus-fingerprint invalidation (same pattern as TF-IDF and similarity caching)
- **D-10:** When metrics haven't been computed, PageRank/Betweenness options are grayed out (disabled) in the dropdown with "Computing..." or "Not available" label. Uniform and Citations always available
- **D-11:** Replace `get_cited_papers()` and `get_citing_papers()` N+1 loops with single SurrealDB JOINs — in-place replacement, same function signatures, callers unchanged
- **D-12:** Claude's discretion on whether to also refactor `get_citation_graph()` BFS traversal based on performance assessment

### Claude's Discretion

- Exact PageRank score formatting (percentage vs decimal)
- Recompute button placement and styling
- Spinner/badge design for the computing indicator
- Whether to extend N+1 refactor beyond get_cited/get_citing to get_citation_graph BFS
- PageRank convergence threshold and max iterations
- Betweenness centrality algorithm variant (Brandes' as specified in GANA-02)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| GANA-01 | System computes PageRank on the citation graph via power iteration | petgraph 0.7.1 has `petgraph::algo::page_rank` — verified in registry source. Compatible with `StableGraph` via `IntoEdges + NodeCount + NodeIndexable` impls. |
| GANA-02 | System computes betweenness centrality via Brandes' algorithm | petgraph 0.7.1 has NO built-in betweenness. Must hand-implement Brandes' algorithm or add external crate. Research recommends hand-implementing (see Architecture Patterns). |
| GANA-03 | Graph metrics cached in SurrealDB with corpus-fingerprint invalidation | New migration 11 follows the `paper_similarity` pattern exactly. `GraphMetricsRepository` mirrors `SimilarityRepository`. |
| GANA-04 | User can size graph nodes by metric via "Size by" dropdown | New `SizeMode` enum in `layout_state.rs`. `NodeState.radius` updated each frame from metrics map. Lerp animation via frame-counted alpha blend. |
| GANA-05 | Dashboard displays "Most Influential Papers" ranking by PageRank | New server fn `get_top_pagerank_papers`. New card in `DashboardCards` component. Existing `SummaryCard` cannot display a ranked list — needs a new `RankedCard` component variant. |
| GANA-06 | N+1 citation queries replaced with single SurrealDB JOINs | SurrealDB `SELECT in.arxiv_id AS from_id, out.arxiv_id AS to_id FROM cites WHERE in = $rid` already returns all rows in one query. The inner `get_paper` loop is the N+1. Replace with `SELECT VALUE out FROM cites WHERE in = $rid FETCH out` or equivalent bulk select. |
</phase_requirements>

---

## Summary

Phase 23 adds graph centrality metrics (PageRank and betweenness centrality) to the citation graph, exposes them as a node-sizing control, and displays a ranked "Most Influential Papers" dashboard card. The main technical challenge is that petgraph 0.7.1 provides `page_rank` natively but has no betweenness centrality — this must be hand-implemented using Brandes' O(VE) algorithm operating on the existing `StableGraph`. The implementation follows the Phase 22 similarity pattern closely: compute on `StableGraph`, persist to SurrealDB with corpus-fingerprint cache invalidation, serve via Leptos server functions, and react in the UI through `RwSignal`.

The N+1 query refactor (GANA-06) is a contained database layer change. Both `get_cited_papers` and `get_citing_papers` currently fetch edge IDs in one query and then call `get_paper()` per ID in a loop. The fix replaces the loop with a bulk `SELECT` using SurrealDB's record link fetching: `SELECT out.* FROM cites WHERE in = $rid`. This returns full paper records directly without per-paper round-trips.

The dashboard "Most Influential Papers" card differs structurally from existing `SummaryCard` components (which show a single number) — it needs a ranked-list variant that can display 5 papers with scores, titles, and years. A new `InfluentialPapersCard` component (or generalised `RankedCard`) is required alongside a new server fn returning `Vec<RankedPaperEntry>`.

**Primary recommendation:** Implement PageRank via `petgraph::algo::page_rank`; implement betweenness centrality with a hand-written Brandes' algorithm in `resyn-core/src/graph_analytics/`. Persist all per-paper metrics in a single `graph_metrics` table keyed by arxiv_id. Serve metrics to the UI through `GraphNode` struct extensions and a dedicated `get_top_pagerank_papers` server fn.

---

## Standard Stack

### Core (all already in workspace)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| petgraph | 0.7.1 | `page_rank` algorithm + `StableGraph` graph structure | Already in workspace; provides `petgraph::algo::page_rank` natively — verified in crate source [VERIFIED: local registry] |
| surrealdb | 3.x | Persist `graph_metrics` table, migration 11 | Already in workspace; same UPSERT + corpus_fingerprint pattern as Phase 22 [VERIFIED: codebase] |
| leptos | 0.8 | Reactive `RwSignal<SizeMode>`, server fns, new dashboard card | Already in workspace [VERIFIED: codebase] |
| sha2 | 0.10 | Corpus fingerprint computation | Already in workspace [VERIFIED: codebase] |
| serde_json | 1.0 | Serialize metrics as JSON string for SurrealDB (mirrors `neighbors` field pattern) | Already in workspace [VERIFIED: codebase] |

### No New Dependencies Required

All required capabilities are present in the existing workspace. Betweenness centrality is hand-implemented — no external crate needed.

**Installation:** None required.

**Version verification:** petgraph 0.7.1 confirmed from local `.cargo/registry` source. [VERIFIED: local registry at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/petgraph-0.7.1/`]

---

## Architecture Patterns

### Recommended Project Structure (new files)

```
resyn-core/src/
├── graph_analytics/
│   ├── mod.rs          # pub mod pagerank; pub mod betweenness;
│   ├── pagerank.rs     # Wrapper around petgraph::algo::page_rank, returns HashMap<String, f32>
│   └── betweenness.rs  # Brandes' algorithm on StableGraph, returns HashMap<String, f32>
├── datamodels/
│   └── graph_metrics.rs  # GraphMetrics struct with arxiv_id, pagerank, betweenness, corpus_fingerprint
├── database/
│   └── schema.rs       # Add apply_migration_11 for graph_metrics table
│   └── queries.rs      # Add GraphMetricsRepository

resyn-app/src/
├── graph/
│   └── layout_state.rs  # Add SizeMode enum; add size_mode field to GraphState; update radius logic
├── server_fns/
│   └── metrics.rs       # get_top_pagerank_papers, get_metrics_status, trigger_metrics_compute
├── components/
│   └── graph_controls.rs  # Add SizeMode dropdown, computing badge
├── pages/
│   └── dashboard.rs     # Add InfluentialPapersCard (6th card)
```

### Pattern 1: PageRank via petgraph

**What:** Call `petgraph::algo::page_rank` with damping factor 0.85 and fixed iteration count (50 iterations is standard). The function returns `Vec<D>` indexed by petgraph's internal `NodeIndex`. Map back to `arxiv_id` using the node-to-paper mapping from `create_graph_from_papers`.

**When to use:** After crawl completes and graph is built from `get_all_papers()` + `get_all_citation_edges()`.

**Signature (verified from source):**
```rust
// Source: petgraph 0.7.1 src/algo/page_rank.rs
// G: NodeCount + IntoEdges + NodeIndexable
// Works with &StableGraph<Paper, f32, Directed, u32>
pub fn page_rank<G, D>(graph: G, damping_factor: D, nb_iter: usize) -> Vec<D>
```

**Usage pattern:**
```rust
// Source: verified from petgraph source and existing graph_creation.rs pattern
use petgraph::algo::page_rank;
use petgraph::stable_graph::StableGraph;

let graph = create_graph_from_papers(&papers); // existing fn
let ranks: Vec<f32> = page_rank(&graph, 0.85_f32, 50);
// Map NodeIndex -> arxiv_id -> rank
let node_ids: Vec<String> = graph.node_indices()
    .map(|idx| graph[idx].id.clone())
    .collect();
let metrics: HashMap<String, f32> = node_ids.into_iter()
    .zip(ranks.into_iter())
    .collect();
```

**CRITICAL:** `page_rank` returns scores indexed by `NodeIndex::index()` (0..N), but `StableGraph` with removed nodes may have gaps. Always use `graph.node_indices()` enumeration rather than assuming contiguous indices. [VERIFIED: petgraph source]

### Pattern 2: Brandes' Betweenness Centrality (hand-implemented)

**What:** Petgraph 0.7.1 does NOT include betweenness centrality. [VERIFIED: local registry — no `betweenness` in `src/algo/`]. Implement Brandes' O(VE) algorithm directly on `StableGraph`.

**Algorithm sketch (Brandes 2001):**

```rust
// Source: [CITED: Brandes 2001 "A Faster Algorithm for Betweenness Centrality"]
// O(VE) time for unweighted directed graphs
fn brandes_betweenness(graph: &StableGraph<Paper, f32, Directed, u32>) -> HashMap<String, f32> {
    let n = graph.node_count();
    let nodes: Vec<NodeIndex> = graph.node_indices().collect();
    let mut centrality: HashMap<NodeIndex, f32> = nodes.iter().map(|&n| (n, 0.0)).collect();

    for &s in &nodes {
        // Single-source BFS from s
        let mut stack: Vec<NodeIndex> = Vec::new();
        let mut predecessors: HashMap<NodeIndex, Vec<NodeIndex>> = HashMap::new();
        let mut sigma: HashMap<NodeIndex, f32> = nodes.iter().map(|&n| (n, 0.0)).collect();
        sigma.insert(s, 1.0);
        let mut dist: HashMap<NodeIndex, i32> = nodes.iter().map(|&n| (n, -1)).collect();
        dist.insert(s, 0);
        let mut queue = VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for w in graph.neighbors(v) {
                if dist[&w] < 0 {
                    queue.push_back(w);
                    *dist.get_mut(&w).unwrap() = dist[&v] + 1;
                }
                if dist[&w] == dist[&v] + 1 {
                    *sigma.get_mut(&w).unwrap() += sigma[&v];
                    predecessors.entry(w).or_default().push(v);
                }
            }
        }

        // Accumulate dependencies
        let mut delta: HashMap<NodeIndex, f32> = nodes.iter().map(|&n| (n, 0.0)).collect();
        while let Some(w) = stack.pop() {
            for &v in predecessors.get(&w).unwrap_or(&vec![]) {
                *delta.get_mut(&v).unwrap() +=
                    (sigma[&v] / sigma[&w]) * (1.0 + delta[&w]);
            }
            if w != s {
                *centrality.get_mut(&w).unwrap() += delta[&w];
            }
        }
    }

    // Normalize for directed graph: divide by (n-1)(n-2)
    let norm = if n > 2 { ((n - 1) * (n - 2)) as f32 } else { 1.0 };
    nodes.iter()
        .map(|&n| (graph[n].id.clone(), centrality[&n] / norm))
        .collect()
}
```

**Time complexity:** O(VE) per the Brandes paper. For a typical crawl of 50-200 papers, this is fast (< 100ms). For 500+ papers, could take 1-2 seconds — acceptable for a background compute. [ASSUMED — not benchmarked]

### Pattern 3: SurrealDB Schema — Migration 11

Add a new migration following the exact `paper_similarity` migration 10 pattern:

```rust
// Source: verified from resyn-core/src/database/schema.rs migration 10 pattern
async fn apply_migration_11(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS graph_metrics SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS arxiv_id ON graph_metrics TYPE string;
        DEFINE FIELD IF NOT EXISTS pagerank ON graph_metrics TYPE float;
        DEFINE FIELD IF NOT EXISTS betweenness ON graph_metrics TYPE float;
        DEFINE FIELD IF NOT EXISTS corpus_fingerprint ON graph_metrics TYPE string;
        DEFINE FIELD IF NOT EXISTS computed_at ON graph_metrics TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_metrics_arxiv_id ON graph_metrics FIELDS arxiv_id UNIQUE;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 11 DDL failed: {e}")))?;
    Ok(())
}
```

### Pattern 4: GraphMetricsRepository

Mirror `SimilarityRepository` exactly:

```rust
// Source: verified from resyn-core/src/database/queries.rs SimilarityRepository pattern
pub struct GraphMetricsRepository<'a> { db: &'a Db }

impl<'a> GraphMetricsRepository<'a> {
    // upsert_metrics — UPSERT type::record('graph_metrics', $id) SET ...
    // get_metrics(arxiv_id) -> Option<GraphMetrics>
    // get_all_metrics() -> Vec<GraphMetrics>
    // get_top_by_pagerank(limit) -> Vec<GraphMetrics> ordered by pagerank DESC
}
```

### Pattern 5: N+1 Query Fix (GANA-06)

**Current N+1 (get_cited_papers):**
```rust
// Line 178-198 in queries.rs — queries edge IDs then loops calling get_paper per ID
let to_ids: Vec<String> = /* one query returning IDs */;
for to_id in to_ids {
    if let Some(p) = self.get_paper(&to_id).await? { // N extra queries
        papers.push(p);
    }
}
```

**Replacement — single JOIN query:**
```rust
// Source: [VERIFIED: existing get_all_citation_edges() pattern in queries.rs:238]
// SurrealDB supports fetching related records via out.* in cites relation
let mut response = self.db
    .query("SELECT out.title, out.authors, out.summary, out.arxiv_id, \
            out.last_updated, out.published, out.pdf_url, out.comment, \
            out.doi, out.inspire_id, out.citation_count, out.source \
            FROM cites WHERE in = $rid")
    .bind(("rid", rid))
    .await?;
// Deserialize rows directly into PaperRecord structs
```

**Note on callers:** `get_cited_papers` and `get_citing_papers` are currently only used in DB tests (no callers in resyn-app or resyn-server). [VERIFIED: grep across all workspace source files]. The refactor is safe — no caller signature changes required.

### Pattern 6: SizeMode Enum and Node Radius

Add to `layout_state.rs` alongside `ForceMode` and `LabelMode`:

```rust
// Source: [VERIFIED: resyn-app/src/graph/layout_state.rs — existing enum patterns]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SizeMode {
    #[default]
    Uniform,
    PageRank,
    Betweenness,
    Citations,
}
```

**Node radius calculation:** Current `radius_from_citations` maps `citation_count -> f64` with sqrt scaling and clamp `[4.0, 18.0]`. For PageRank/Betweenness, a similar sqrt scaling on the normalized score should be used to avoid extreme size variance from outliers:

```rust
// [ASSUMED] — specific values are Claude's discretion per CONTEXT.md
pub fn radius_from_metric(score: f32, min_r: f64, max_r: f64) -> f64 {
    ((score as f64).sqrt() * (max_r - min_r) + min_r).clamp(min_r, max_r)
}
```

**Lerp animation (D-02):** Use the same frame-based lerp as `FitAnimState`. Store both `target_radius` and `current_radius` on `NodeState`, advance by factor ~0.15 per frame (300ms at 60fps ≈ 18 frames, lerp factor 0.15 reaches ~95% in 18 steps).

### Pattern 7: Server Functions

New file `resyn-app/src/server_fns/metrics.rs`:

```rust
// Pattern: mirrors similarity.rs structure exactly
#[server(GetTopPageRankPapers, "/api")]
pub async fn get_top_pagerank_papers(limit: usize) -> Result<Vec<RankedPaperEntry>, ServerFnError>

#[server(GetMetricsStatus, "/api")]
pub async fn get_metrics_status() -> Result<MetricsStatus, ServerFnError>
// Returns: Available { corpus_fingerprint }, Computing, NotAvailable

#[server(TriggerMetricsCompute, "/api")]
pub async fn trigger_metrics_compute() -> Result<String, ServerFnError>
// Spawns background tokio::spawn task, returns "Computing started"
```

### Pattern 8: Dashboard "Most Influential Papers" Card

The existing `SummaryCard` component accepts a single `number: String` value — it cannot display a ranked list. A new `InfluentialPapersCard` component is needed:

```rust
// Source: [VERIFIED: resyn-app/src/pages/dashboard.rs — SummaryCard only shows one number]
// New component with:
// - Title: "Most Influential Papers"
// - Body: <ol> with top-5 entries: score | title | year
// - Footer link: "View all →" → /metrics or /influential
```

The dashboard's `DashboardCards` component currently has 5 hardcoded `SummaryCard` calls. Extend it to load `get_top_pagerank_papers(5)` as a separate `Resource` and render the 6th card.

### Anti-Patterns to Avoid

- **Storing metrics as per-paper fields on the `paper` table:** Violates the migration-based extension pattern. All analysis results go in separate tables. Use `graph_metrics`.
- **Computing betweenness in WASM/browser:** The StableGraph is on the server side. Metrics compute only on the server (SSR/ssr feature gate). Never send the full graph to the client.
- **Using petgraph's raw `NodeIndex` as the cache key:** `NodeIndex` values depend on insertion order and are not stable across DB reloads. Always map to `arxiv_id` before persistence.
- **Assuming `page_rank` returns contiguous indices:** With `StableGraph`, indices may have gaps after node removal. Use `graph.node_indices()` enumeration. [VERIFIED: petgraph source]
- **Blocking the async executor with betweenness computation:** Use `tokio::task::spawn_blocking` for the O(VE) computation if the corpus is large (> 200 papers). [ASSUMED — conservative guidance]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| PageRank computation | Custom power iteration loop | `petgraph::algo::page_rank` | Already in workspace, handles dangling nodes (no outgoing edges), tested [VERIFIED: petgraph source] |
| Corpus fingerprint | Custom hash scheme | `sha2` via existing `AnalysisMetadata.corpus_fingerprint` pattern | Existing `compute_corpus_fingerprint()` or equivalent already used in Phase 20-22 [VERIFIED: codebase] |
| SurrealDB schema management | Ad-hoc CREATE TABLE | Migration 11 in `schema.rs` with `IF NOT EXISTS` DDL | Idempotent, versioned, tested — same pattern as migrations 1-10 [VERIFIED: codebase] |

**Betweenness is the exception:** petgraph 0.7.1 has no built-in betweenness centrality [VERIFIED: local registry]. Brandes' algorithm is well-documented and manageable (~80 lines of Rust). Writing it directly is simpler than pulling in an unfamiliar external crate for a single algorithm.

---

## Common Pitfalls

### Pitfall 1: petgraph page_rank index alignment
**What goes wrong:** `page_rank` returns `Vec<f32>` indexed 0..N. With `StableGraph`, removed nodes leave "holes" — index 0 may not be `node_indices().nth(0)`.
**Why it happens:** `StableGraph` maintains stable indices across removals; the internal storage has gaps.
**How to avoid:** Collect `graph.node_indices()` into a `Vec`, zip it with the `ranks` Vec. This is safe because `page_rank` iterates over `node_count()` items and `graph.from_index(i)` maps correctly.
**Warning signs:** Scores assigned to wrong papers (off-by-one in a graph without removals but misaligned in one with them).

### Pitfall 2: Betweenness normalization direction
**What goes wrong:** Normalized betweenness for directed graphs uses denominator `(n-1)(n-2)`. Using the undirected formula `(n-1)(n-2)/2` produces double the scores.
**Why it happens:** The citation graph is directed.
**How to avoid:** Use directed normalization: `score / ((n-1) * (n-2))` [CITED: Brandes 2001].

### Pitfall 3: SurrealDB FLEXIBLE TYPE and float storage
**What goes wrong:** `FLEXIBLE TYPE` for `graph_metrics` score fields causes unpredictable query behavior (see STATE.md note: "SurrealDB FLEXIBLE TYPE limits server-side querying").
**Why it happens:** `FLEXIBLE` was a workaround for TF-IDF vectors. Float fields should be SCHEMAFULL `TYPE float`.
**How to avoid:** Define `pagerank` and `betweenness` as `TYPE float` in the migration DDL. [VERIFIED: schema.rs DDL patterns]

### Pitfall 4: Blocking the tokio runtime with betweenness on large graphs
**What goes wrong:** Brandes' O(VE) on 500 nodes × 2000 edges in the async context blocks other requests.
**Why it happens:** The BFS + accumulation loops are CPU-bound.
**How to avoid:** Wrap the computation in `tokio::task::spawn_blocking`.
**Warning signs:** Server becomes unresponsive during metrics computation.

### Pitfall 5: Dashboard metrics card loaded in the same Suspense as existing stats
**What goes wrong:** If `get_top_pagerank_papers` is slow (not computed yet), it blocks the entire dashboard skeleton from resolving.
**Why it happens:** A single `Resource` covers all 5 existing cards; adding the 6th to the same resource couples their loading.
**How to avoid:** Load `get_top_pagerank_papers` as a separate `Resource` with its own `Suspense`. The 5 existing cards load independently from the metrics card.

### Pitfall 6: N+1 fix changes query result structure
**What goes wrong:** The refactored `get_cited_papers` query using `out.*` may return fields in a different structure than the `PaperRecord` struct expects (e.g., field names prefixed with `out.`).
**Why it happens:** SurrealDB `out.*` expands fields but the column alias depends on query form.
**How to avoid:** Use explicit column aliases matching `PaperRecord` field names (e.g., `out.arxiv_id AS arxiv_id`). Alternatively, use `SELECT * FROM paper WHERE id IN (SELECT out FROM cites WHERE in = $rid)` as a bulk IN query — proven safe from `get_all_citation_edges()` pattern. [VERIFIED: codebase queries.rs:238]

---

## Code Examples

### PageRank computation with StableGraph

```rust
// Source: verified from petgraph-0.7.1 src/algo/page_rank.rs + graph_creation.rs
use petgraph::algo::page_rank;
use resyn_core::data_processing::graph_creation::create_graph_from_papers;

pub fn compute_pagerank(papers: &[Paper]) -> HashMap<String, f32> {
    let graph = create_graph_from_papers(papers);
    if graph.node_count() == 0 {
        return HashMap::new();
    }
    let ranks: Vec<f32> = page_rank(&graph, 0.85_f32, 50);
    // node_indices() aligns with ranks vec positions (verified from NodeIndexable impl)
    graph.node_indices()
        .map(|idx| {
            let pos = graph.to_index(idx);
            (utils::strip_version_suffix(&graph[idx].id), ranks[pos])
        })
        .collect()
}
```

### SurrealDB bulk IN query (N+1 fix alternative)

```rust
// Source: verified from existing get_all_citation_edges() query pattern in queries.rs:238
// Bulk fetch papers by ID using WHERE id IN subquery
let mut response = self.db
    .query("SELECT title, authors, summary, arxiv_id, last_updated, published, \
            pdf_url, comment, doi, inspire_id, citation_count, source \
            FROM paper WHERE id IN (SELECT out FROM cites WHERE in = $rid)")
    .bind(("rid", rid))
    .await
    .map_err(|e| ResynError::Database(format!("get cited papers failed: {e}")))?;
```

### SizeMode dropdown in graph_controls.rs

```rust
// Source: [VERIFIED: existing label_mode select in graph_controls.rs:152-170]
// Add after the label mode group, mirroring exact same select pattern
<div class="graph-controls-group">
    <span class="text-label" style="...">"Size by"</span>
    <select class="form-select"
        on:change=move |e| { /* parse to SizeMode */ }>
        <option value="uniform">"Uniform"</option>
        <option value="pagerank"
            disabled=move || !metrics_available.get()>
            {move || if computing.get() { "PageRank (computing...)" } else { "PageRank" }}
        </option>
        <option value="betweenness"
            disabled=move || !metrics_available.get()>
            {move || if computing.get() { "Betweenness (computing...)" } else { "Betweenness" }}
        </option>
        <option value="citations">"Citations"</option>
    </select>
</div>
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| petgraph 0.6 (no page_rank) | petgraph 0.7+ with `page_rank` built-in | petgraph 0.7.0 release | No custom PageRank needed |
| SurrealDB v1/v2 FLEX schema | SurrealDB v3 SCHEMAFULL with explicit TYPE | Phase 19 adoption | TYPE float enforced for analytics fields |

**Deprecated/outdated:**
- `FLEXIBLE TYPE` on analytics tables: worked for TF-IDF vectors (map-shaped), inappropriate for scalar float metrics. Use `TYPE float` for `pagerank` and `betweenness` fields.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Brandes' O(VE) on 200 nodes takes < 100ms; on 500+ nodes < 2 seconds | Architecture Patterns P2 | If wrong: betweenness compute blocks server; mitigation: spawn_blocking already recommended |
| A2 | `radius_from_metric` using sqrt scaling produces visually acceptable node size variance | Architecture Patterns P6 | If wrong: planner should adjust formula; low risk, Claude's discretion per D-02 |

---

## Open Questions

1. **Does `get_citation_graph()` BFS also need refactoring (D-12)?**
   - What we know: `get_citation_graph()` does per-frontier-node N+1 queries inside a depth loop. Current callers: only used in old `main.rs`-style flows; the new server-side `get_graph_data()` uses `get_all_papers()` + `get_all_citation_edges()` instead. [VERIFIED: server_fns/graph.rs]
   - What's unclear: Whether any current execution path calls `get_citation_graph()` in production.
   - Recommendation: Defer the `get_citation_graph()` BFS refactor — it appears unused in the Leptos app path. Focus D-11 on `get_cited_papers` and `get_citing_papers` which are the specified targets.

2. **Where does "View all →" link for the Influential Papers card go?**
   - What we know: The pattern links to an existing page (e.g., `/papers`, `/gaps`). No `/influential` or `/metrics` page exists.
   - What's unclear: Whether a new page is needed or the link can be a no-op/deferred.
   - Recommendation: Link to `/papers` with a URL param like `?sort=pagerank` for now, or create a minimal `/metrics` page showing the full ranking table. This is Claude's discretion per D-06 since the card content (top-5) is the primary deliverable.

---

## Environment Availability

Step 2.6: SKIPPED — Phase 23 is a pure code change within the existing Rust workspace. No new external services, CLI tools, or databases required. SurrealDB is already in the workspace.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test (`#[test]`, `#[tokio::test]`) |
| Config file | none — cargo test discovers via `#[test]` annotations |
| Quick run command | `cargo test graph_analytics` |
| Full suite command | `cargo test --features ssr` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GANA-01 | PageRank returns non-zero scores for connected graph | unit | `cargo test pagerank` | ❌ Wave 0 |
| GANA-01 | PageRank returns empty vec for empty graph | unit | `cargo test pagerank_empty` | ❌ Wave 0 |
| GANA-02 | Betweenness returns 0 for path endpoints, positive for intermediary | unit | `cargo test betweenness` | ❌ Wave 0 |
| GANA-03 | upsert + get_metrics round-trips through SurrealDB in-memory | database | `cargo test graph_metrics_db --features ssr` | ❌ Wave 0 |
| GANA-03 | Stale metrics invalidated when corpus fingerprint changes | database | `cargo test metrics_fingerprint_invalidation --features ssr` | ❌ Wave 0 |
| GANA-04 | SizeMode enum: default is Uniform | unit | `cargo test size_mode_default` | ❌ Wave 0 |
| GANA-05 | get_top_pagerank_papers returns ordered results | database | `cargo test top_pagerank --features ssr` | ❌ Wave 0 |
| GANA-06 | get_cited_papers returns same results as before refactor | database | `cargo test get_cited_papers --features ssr` | existing test at queries.rs:1285 (update) |
| GANA-06 | get_citing_papers returns same results as before refactor | database | `cargo test get_citing_papers --features ssr` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test` (unit tests only, fast)
- **Per wave merge:** `cargo test --features ssr` (includes DB tests)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `resyn-core/src/graph_analytics/pagerank.rs` + test module — covers GANA-01
- [ ] `resyn-core/src/graph_analytics/betweenness.rs` + test module — covers GANA-02
- [ ] `resyn-core/src/database/queries.rs` — add `GraphMetricsRepository` + DB tests — covers GANA-03, GANA-05
- [ ] `resyn-app/src/graph/layout_state.rs` — add `SizeMode` test — covers GANA-04
- [ ] Update existing `get_cited_papers` test at queries.rs:1285 after refactor — covers GANA-06

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | `strip_version_suffix` on all arxiv_id inputs (existing pattern) |
| V6 Cryptography | no | sha2 for fingerprints only, not security-critical |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SurrealDB query injection via arxiv_id | Tampering | Parameterized queries with `.bind(("id", ...))` — already enforced throughout codebase [VERIFIED: queries.rs] |
| Denial of service via repeated `trigger_metrics_compute` | DoS | Check if already computing before spawning new task; return early if in-progress [ASSUMED] |

---

## Sources

### Primary (HIGH confidence)

- petgraph 0.7.1 source (`~/.cargo/registry/src/.../petgraph-0.7.1/src/algo/page_rank.rs`) — confirmed `page_rank` function signature, `StableGraph` trait impls, no betweenness in `src/algo/`
- `resyn-core/src/database/schema.rs` — confirmed migration pattern, current version at migration 10
- `resyn-core/src/database/queries.rs` — confirmed `SimilarityRepository` pattern (UPSERT, fingerprint, JSON string for complex data), N+1 pattern location (lines 178-222), `get_all_citation_edges` single-query pattern (line 238)
- `resyn-app/src/graph/layout_state.rs` — confirmed `ForceMode`, `LabelMode` enum patterns, `NodeState.radius` field, `radius_from_citations` formula
- `resyn-app/src/components/graph_controls.rs` — confirmed `<select>` dropdown pattern for label_mode (lines 152-170)
- `resyn-app/src/pages/dashboard.rs` — confirmed `SummaryCard` accepts only `number: String`, confirmed 5-card structure, separate Suspense approach needed for 6th card
- `resyn-app/src/server_fns/similarity.rs` — confirmed server fn pattern for analysis results
- `Cargo.toml` workspace — confirmed petgraph = "0.7.0", no betweenness crate dependency

### Secondary (MEDIUM confidence)

- [CITED: Brandes 2001] "A Faster Algorithm for Betweenness Centrality", Journal of Mathematical Sociology 25(2):163-177 — Brandes' O(VE) algorithm with directed normalization `(n-1)(n-2)`

### Tertiary (LOW confidence)

- None — all key claims verified from codebase or official algorithm literature

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all verified from local registry and codebase
- Architecture: HIGH — patterns verified from existing Phase 22 code
- Pitfalls: HIGH for SurrealDB/petgraph specifics (verified); MEDIUM for performance estimates (assumed)
- Algorithm correctness: HIGH (Brandes 2001 canonical reference)

**Research date:** 2026-04-09
**Valid until:** 2026-07-09 (stable — petgraph 0.7.1 is pinned, SurrealDB schema is append-only)
