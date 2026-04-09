# Phase 23: Graph Analytics — Centrality & Metrics - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can explore which papers are most structurally influential using PageRank and betweenness centrality. Graph nodes resize by chosen metric, a dashboard card ranks the most influential papers, and citation queries are optimized to eliminate N+1 patterns. This phase does NOT add community detection, recommendations, or export — those are separate phases (24-26).

</domain>

<decisions>
## Implementation Decisions

### Node sizing controls
- **D-01:** "Size by" dropdown added to the existing graph controls overlay (alongside edge toggles, force mode, label mode). Options: Uniform / PageRank / Betweenness / Citations
- **D-02:** Switching metrics triggers animated lerp transition (~300ms) on node sizes, consistent with existing viewport_fit lerp pattern
- **D-03:** Hovering a node shows the active metric's raw score (e.g. "PageRank: 0.042") in the tooltip/label overlay

### Influential papers panel
- **D-04:** New (6th) dashboard card "Most Influential Papers" showing top-5 papers ranked by PageRank score
- **D-05:** Each ranked entry displays: PageRank score (formatted), paper title, publication year — consistent with existing SummaryCard density
- **D-06:** Card links to full ranking view via "View all →" pattern (matching other dashboard cards)

### Computation & caching strategy
- **D-07:** Metrics auto-compute after crawl completes (background, no user action needed), plus a manual "Recompute" button for forcing refresh
- **D-08:** Subtle progress indicator — small spinner or badge on the "Size by" dropdown until metrics are available
- **D-09:** Metrics cached in SurrealDB with corpus-fingerprint invalidation (same pattern as TF-IDF and similarity caching)
- **D-10:** When metrics haven't been computed, PageRank/Betweenness options are grayed out (disabled) in the dropdown with "Computing..." or "Not available" label. Uniform and Citations always available

### N+1 query optimization
- **D-11:** Replace `get_cited_papers()` and `get_citing_papers()` N+1 loops with single SurrealDB JOINs — in-place replacement, same function signatures, callers unchanged
- **D-12:** Claude's discretion on whether to also refactor `get_citation_graph()` BFS traversal based on performance assessment

### Claude's Discretion
- Exact PageRank score formatting (percentage vs decimal)
- Recompute button placement and styling
- Spinner/badge design for the computing indicator
- Whether to extend N+1 refactor beyond get_cited/get_citing to get_citation_graph BFS
- PageRank convergence threshold and max iterations
- Betweenness centrality algorithm variant (Brandes' as specified in GANA-02)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — GANA-01 through GANA-06 define graph analytics acceptance criteria
- `.planning/ROADMAP.md` §Phase 23 — Success criteria and phase boundary

### Existing graph code
- `resyn-core/src/data_processing/graph_creation.rs` — `create_graph_from_papers()` builds `petgraph::StableGraph` — algorithms run on this
- `resyn-core/src/database/queries.rs:178-222` — `get_cited_papers()` and `get_citing_papers()` with N+1 pattern to refactor
- `resyn-core/src/database/queries.rs:224-247` — `get_all_papers()` and `get_all_citation_edges()` for bulk loading
- `resyn-core/src/database/schema.rs` — Current DB schema (add graph_metrics table here)

### Graph controls and dashboard integration points
- `resyn-app/src/components/graph_controls.rs` — Existing controls overlay (add "Size by" dropdown here)
- `resyn-app/src/pages/dashboard.rs` — Dashboard with 5 SummaryCards (add 6th "Most Influential" card)
- `resyn-app/src/pages/graph.rs` — Graph page (node sizing logic integration)
- `resyn-app/src/graph/canvas_renderer.rs` — Canvas 2D renderer (node size from metric)
- `resyn-app/src/graph/webgl_renderer.rs` — WebGL2 renderer (node size from metric)

### Prior phase patterns
- `.planning/phases/22-paper-similarity-engine/22-CONTEXT.md` — Similarity caching with corpus fingerprint, graph controls toggle pattern, background recompute pattern

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `petgraph::StableGraph`: Already in use for citation graph — PageRank and betweenness can run directly on it
- `corpus_fingerprint` pattern: Used for TF-IDF and similarity caching — same invalidation approach for metrics
- `SummaryCard` component: Existing dashboard card component for the new "Most Influential" card
- `GraphControls` component: Has toggle buttons and `ForceMode`/`LabelMode` dropdowns — add `SizeMode` dropdown here
- `get_all_papers()` + `get_all_citation_edges()`: Bulk loaders for building adjacency without N+1

### Established Patterns
- Server fns with `#[server(..., "/api")]` for all DB queries
- `RwSignal` for UI state (force_mode, label_mode — add size_mode)
- Corpus fingerprint for cache invalidation across all analysis tables
- Dashboard cards with `SummaryCard` component, skeleton loading, and error states

### Integration Points
- Graph controls overlay: Add "Size by" dropdown (new `SizeMode` enum: Uniform, PageRank, Betweenness, Citations)
- Dashboard: Add 6th SummaryCard for "Most Influential Papers"
- Graph renderers (Canvas 2D + WebGL2): Read active SizeMode to scale node radius
- SurrealDB schema: New `graph_metrics` table with per-paper scores and corpus fingerprint
- Server fns: New endpoints for metrics computation trigger and retrieval

</code_context>

<specifics>
## Specific Ideas

- Dual trigger model: auto-compute after crawl (matching Phase 22 similarity pattern) plus manual "Recompute" button for user control
- Disabled dropdown options (not hidden) when metrics unavailable — communicates that the feature exists but needs computation
- Lerp animation for node size transitions matches existing viewport_fit animation philosophy

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 23-graph-analytics-centrality-metrics*
*Context gathered: 2026-04-09*
