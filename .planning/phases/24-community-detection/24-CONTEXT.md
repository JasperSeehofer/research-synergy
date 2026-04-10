# Phase 24: Community Detection - Context

**Gathered:** 2026-04-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can see how the citation graph clusters into research communities and explore each community's character. The system runs Louvain modularity optimization on the citation graph, stores per-paper community assignments in SurrealDB, introduces a new "Color by" dropdown in the graph controls (with Community / BFS Depth / Topic modes), and adds a community summary view as a new drawer tab. This phase does NOT add recommendations, export, or further graph analytics — those are Phases 25–26.

</domain>

<decisions>
## Implementation Decisions

### Louvain algorithm & execution
- **D-01:** Use an existing Rust crate for Louvain (e.g. `graphalgs` or similar petgraph-compatible crate). Researcher evaluates candidates in RESEARCH.md and locks one choice; only fall back to in-house implementation if no mature crate cleanly accepts `petgraph::StableGraph`
- **D-02:** Run Louvain on the **undirected** projection of the citation graph (standard formulation; most intuitive topical clusters)
- **D-03:** **Fixed RNG seed** for Louvain's randomized node ordering — successive recomputations on the same corpus yield identical community IDs, so node colors stay stable across page reloads and recomputes
- **D-04:** Communities with fewer than 3 papers are **bucketed into a single "Other" community** and rendered in a neutral gray color. Keeps the legend short on noisy corpora

### Storage & caching
- **D-05:** New `graph_communities` table (name TBD by planner — e.g. `community_assignments`) storing `{paper_id, community_id, corpus_fingerprint}`. Follows the Phase 22/23 caching convention — invalidated by `corpus_fingerprint` changes
- **D-06:** Auto-compute after crawl completes (matching Phase 23 metrics pattern), plus a manual "Recompute" trigger. Use the same "Computing..." disabled-option UX pattern as Phase 23 when community data isn't yet available

### Color by dropdown — new node-color system
- **D-07:** New `ColorMode` enum introduced with **three modes: Community, BFS Depth, Topic** — all shipped in this phase. This is the first time node fills are modal (currently nodes use a fixed blue/seed-yellow palette), so existing `canvas_renderer.rs` and `webgl_renderer.rs` fill logic must be refactored to read `ColorMode`
- **D-08:** "Color by" dropdown added to `graph_controls.rs`, placed alongside the existing "Size by" dropdown from Phase 23. Managed via a new `RwSignal<ColorMode>` state, parallel to `size_mode`
- **D-09:** **Default mode on first load: Community** — showcases the new feature immediately
- **D-10:** `Color by = Community` and the existing **Topic Rings toggle remain independent** — users can have both on simultaneously (rings draw around nodes; fills come from ColorMode)
- **D-11:** **Categorical palette** for communities — use a well-known 10-color categorical palette (Tableau 10 / D3 schemeCategory10 or closest GitHub-dark-compatible equivalent). Cycle for >10 communities. "Other" bucket uses a neutral gray outside the categorical set
- **D-12:** **Lerp color transitions (~300ms)** when switching ColorMode, consistent with Phase 23's Size by animated lerp pattern
- **D-13:** BFS Depth color mode: color nodes by BFS distance from the seed paper (depth 0 = seed, depth 1, 2, ...). Use a monotonic scale (e.g. warm→cool)
- **D-14:** Topic color mode: reuse existing topic grouping (the same grouping that `Topic Rings` uses) to color node fills. Topic groups are derived from the existing topic-ring data pipeline — no new extraction

### Community summary panel placement
- **D-15:** New **`DrawerTab::Community`** variant added to the existing `DrawerTab` enum (currently Overview / Source / Similar from Phase 22). This is the fourth tab
- **D-16:** **Two entry points** for opening a community summary:
  1. **Paper-selected mode:** When a paper is selected, the Community tab is available and shows **that paper's** community summary
  2. **Legend-click mode:** Clicking a community chip in the graph-controls legend opens the drawer directly on the Community tab showing that community's summary
- **D-17:** When entry is via legend click and **no paper is selected**, the drawer opens on the Community tab with **community-level content only** — no paper title/author header. Overview / Source / Similar tabs show a "select a paper to view" placeholder (or are disabled). The drawer is still "drawer-shaped" but scoped to the community
- **D-18:** Legend of community chips is rendered inside the graph controls overlay (similar to existing "Topic Colors legend" pattern in `graph_controls.rs`), appearing when `ColorMode::Community` is active

### Community summary content
- **D-19:** **Top papers ranked by hybrid score:** `PageRank × community-internal degree`. Reuses Phase 23 PageRank scores and combines with how well-connected each paper is *within* the community. Surfaces papers that are both globally influential and locally central
- **D-20:** **Show top 5 papers per community** (matches Phase 23 "Most Influential Papers" card convention)
- **D-21:** **Dominant keywords via c-TF-IDF (class-based TF-IDF):** treat each community as a single pseudo-document, compute TF-IDF at the community level to surface terms **distinctive** to that community relative to others. Uses existing per-paper TF-IDF vectors from Phase 22 as input
- **D-22:** **Show top 10 keywords per community**
- **D-23:** **"Shared methods" field:** reuse the existing `shared_high_weight_terms()` helper from `gap_analysis/similarity.rs` (Phase 22) aggregated across community members to surface method-like recurring terms. No new extraction pipeline — if the existing helper doesn't yield useful output on communities, planner may reduce scope
- **D-24:** **Community auto-label:** each community is named after its **top 1–2 c-TF-IDF keywords** (e.g. "quantum decoherence"). Displayed in the legend chip and drawer tab header. No numeric "Community N" labels — keywords are the name

### Claude's Discretion
- Exact Louvain crate choice (researcher locks in RESEARCH.md after crate evaluation)
- Fixed RNG seed value
- Exact categorical palette hex values and mapping to communities by size rank
- Drawer tab layout details for community summary (card order, typography, spacing)
- "Select a paper to view" placeholder copy for orphan-community drawer state
- Exact lerp curve parameters and whether color lerping shares the same animation clock as size lerping
- Implementation strategy for c-TF-IDF (pure aggregation vs. existing `nlp/tfidf.rs` extension)
- How many keywords `shared_high_weight_terms()` should surface per community and how to present them alongside dominant keywords
- Whether community membership is also visible on the paper detail drawer's Overview tab (e.g. a "Community: quantum decoherence" chip)
- Fallback behavior for graphs too small to produce meaningful communities (e.g. <10 nodes)
- Louvain resolution / modularity tuning parameters

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & roadmap
- `.planning/REQUIREMENTS.md` — COMM-01, COMM-02, COMM-03 define community detection acceptance criteria
- `.planning/ROADMAP.md` §Phase 24 — Success criteria and phase boundary

### Graph & analytics code (algorithm integration)
- `resyn-core/src/data_processing/graph_creation.rs` — `create_graph_from_papers()` builds the `petgraph::StableGraph` — Louvain runs on this (undirected projection)
- `resyn-core/src/database/queries.rs` — `get_all_papers()` and `get_all_citation_edges()` bulk loaders (reuse for community compute)
- `resyn-core/src/database/schema.rs` — Current DB schema (add community assignments table here)
- `resyn-core/src/gap_analysis/similarity.rs` — `shared_high_weight_terms()` helper (reused for D-23)
- `resyn-core/src/nlp/tfidf.rs` — TF-IDF computation; c-TF-IDF for community keywords builds on this (D-21)
- `resyn-core/src/datamodels/analysis.rs` — `PaperAnalysis.tfidf_vector` (sparse HashMap<String, f32>) and `corpus_fingerprint` — the inputs for c-TF-IDF aggregation

### Graph controls & renderer integration points
- `resyn-app/src/components/graph_controls.rs` — Existing controls overlay; add "Color by" dropdown here alongside the Phase 23 "Size by" dropdown; add community legend here (see existing "Topic Colors legend" pattern ~line 252)
- `resyn-app/src/graph/layout_state.rs` — Add `ColorMode` enum next to existing `ForceMode`, `LabelMode`, `SizeMode`
- `resyn-app/src/graph/canvas_renderer.rs:262-298` — Node fill logic (currently fixed palette) — refactor to read `ColorMode` and community assignments; add color lerp state
- `resyn-app/src/graph/webgl_renderer.rs` — WebGL2 renderer mirror of the same node fill refactor
- `resyn-app/src/pages/graph.rs:92-93` — Topic ring signals and signal wiring pattern — add `ColorMode` signal here; keep `show_topic_rings` independent (D-10)

### Drawer integration points
- `resyn-app/src/app.rs:16` — `DrawerTab` enum (currently Overview, Source, Similar) — add `Community` variant here (D-15)
- `resyn-app/src/layout/drawer.rs` — Drawer component with tab system; handle "no paper selected" state for legend-click entry (D-17)
- `resyn-app/src/server_fns/analysis.rs` — Analysis server fns; add community compute trigger hook and query endpoints

### Prior phase patterns (must match)
- `.planning/phases/22-paper-similarity-engine/22-CONTEXT.md` — `DrawerTab` extension precedent, corpus_fingerprint caching, drawer tab pattern
- `.planning/phases/23-graph-analytics-centrality-metrics/23-CONTEXT.md` — Controls dropdown placement, auto-compute + manual Recompute, lerp animation, disabled-option "Computing..." UX, PageRank (input for D-19)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `petgraph::StableGraph`: Already the working graph type — Louvain crate must accept it (or we build adjacency from edges)
- `corpus_fingerprint` pattern: Established in Phase 22 (similarity) and Phase 23 (metrics) — same invalidation flow for communities
- Phase 23 `PageRank` scores in `graph_metrics` table: Direct input to the D-19 hybrid ranking
- `PaperAnalysis.tfidf_vector`: Existing sparse per-paper vectors power c-TF-IDF (D-21) without new extraction
- `shared_high_weight_terms()` in `gap_analysis/similarity.rs`: Reused for "shared methods" (D-23)
- `DrawerTab` enum: Extensible pattern — Phase 22 added `Similar`, Phase 24 adds `Community`
- `SizeMode` dropdown in `graph_controls.rs`: Direct template for the new `ColorMode` dropdown
- "Topic Colors legend" block in `graph_controls.rs` (~line 252): Template for the new community legend

### Established Patterns
- Server fns with `#[server(..., "/api")]` for all DB queries
- `RwSignal<T>` for cross-component UI state; context-based wiring
- Lerp-based smooth transitions for mode changes (Phase 23 `Size by`)
- Dashboard / drawer consume cached analysis tables, never trigger compute on read
- Corpus fingerprint invalidation across all analysis tables (TF-IDF, similarity, metrics, now communities)

### Integration Points
- New `ColorMode::{Community, BfsDepth, Topic}` enum in `layout_state.rs`
- New `color_mode: RwSignal<ColorMode>` signal in `pages/graph.rs`, plumbed into both renderers
- Node fill pipeline in Canvas 2D and WebGL2 renderers must consult `ColorMode` + community assignments
- New drawer tab `DrawerTab::Community` + drawer state handling for "no paper selected" (legend-click entry)
- New SurrealDB table for community assignments; new server fns for compute-trigger + fetch
- Community legend rendered in graph controls when `ColorMode::Community` is active

### Greenfield (nothing to reuse)
- Louvain algorithm itself — no modularity / community detection code anywhere in `resyn-core/` today
- c-TF-IDF aggregation logic — existing TF-IDF is per-paper only
- Color lerp animation — Phase 23 established size lerp, but color interpolation is new

</code_context>

<specifics>
## Specific Ideas

- Determinism via fixed RNG seed is a UX-critical choice: without it, community IDs (and therefore colors) would shuffle on every recompute, breaking visual memory across user sessions
- Introducing `ColorMode` is a bigger refactor than Phase 23's `SizeMode` because node fills are currently hardcoded; this is the first time fills become data-driven. Planner should treat the renderer refactor as a first-class task, not a side-effect
- c-TF-IDF (class-based TF-IDF) is specifically better than plain aggregated TF-IDF for community labeling because it surfaces distinctive terms rather than just popular ones — "quantum" might be popular across every community in a quantum-physics corpus; c-TF-IDF would still pick the terms that differentiate clusters
- The "hybrid: PageRank × community-internal degree" ranking is deliberately chosen to blend global prestige with local centrality — papers that are famous overall AND structurally central within their cluster
- Auto-naming communities by top keywords (D-24) avoids the "Community 1, Community 2" problem where users have no mental handle on which cluster is which

</specifics>

<deferred>
## Deferred Ideas

- User-adjustable Louvain resolution parameter (slider to control community granularity) — v1.4 ships with a fixed default; tuning is a future iteration
- Community comparison view ("how do Community A and Community B differ?") — belongs to a later discovery feature, not Phase 24
- Community membership history / evolution as corpus grows — interesting but out of scope
- Cross-community bridge detection as a standalone surfacing — Phase 25 DISC-04 already covers bridge signals in recommendations
- Richer "methods" extraction beyond `shared_high_weight_terms()` (e.g. NER for equation/algorithm names) — deferred until the simple approach is validated
- Community-scoped search/filter within Phase 21 search — potential v1.5 addition

</deferred>

---

*Phase: 24-community-detection*
*Context gathered: 2026-04-10*
