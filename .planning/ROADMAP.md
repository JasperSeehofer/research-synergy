# Roadmap: Research Synergy (ReSyn)

## Milestones

- ✅ **v1.0 Analysis Pipeline** — Phases 1-5 (shipped 2026-03-14)
- ✅ **v1.1 Scale & Surface** — Phases 6-10 (shipped 2026-03-22)
- ✅ **v1.1.1 Bug Fix & Polish** — Phases 11-14 (shipped 2026-03-24)
- ✅ **v1.2 Graph Rendering Overhaul** — Phases 15-17 (shipped 2026-03-26)
- ✅ **v1.3 Data Pipeline Fixes** — Phases 18-20, 999.1-999.2 (shipped 2026-04-05)
- 🚧 **v1.4 Discovery & Intelligence** — Phases 21-26 (in progress)

## Phases

<details>
<summary>✅ v1.0 Analysis Pipeline (Phases 1-5) — SHIPPED 2026-03-14</summary>

- [x] Phase 1: Text Extraction Foundation (2/2 plans) — completed 2026-03-14
- [x] Phase 2: NLP Analysis + DB Schema (2/2 plans) — completed 2026-03-14
- [x] Phase 3: Pluggable LLM Backend (3/3 plans) — completed 2026-03-14
- [x] Phase 4: Cross-Paper Gap Analysis (3/3 plans) — completed 2026-03-14
- [x] Phase 5: Visualization Enrichment (2/2 plans) — completed 2026-03-14

Full details: `.planning/milestones/v1.0-ROADMAP.md`

</details>

<details>
<summary>✅ v1.1 Scale & Surface (Phases 6-10) — SHIPPED 2026-03-22</summary>

- [x] Phase 6: Tech Debt + Workspace Restructure (2/2 plans) — completed 2026-03-15
- [x] Phase 7: Incremental Crawl Infrastructure (5/5 plans) — completed 2026-03-16
- [x] Phase 8: Leptos Web Shell + Analysis Panels (7/7 plans) — completed 2026-03-17
- [x] Phase 9: Graph Renderer (Canvas to WebGL) (5/5 plans) — completed 2026-03-18
- [x] Phase 10: Analysis UI Polish + Scale (4/4 plans) — completed 2026-03-18

Full details: `.planning/milestones/v1.1-ROADMAP.md`

</details>

<details>
<summary>✅ v1.1.1 Bug Fix & Polish (Phases 11-14) — SHIPPED 2026-03-24</summary>

- [x] Phase 11: SPA Routing (1/1 plans) — completed 2026-03-23
- [x] Phase 12: Graph Force & Rendering (1/1 plans) — completed 2026-03-23
- [x] Phase 13: Graph Interaction (1/1 plans) — completed 2026-03-23
- [x] Phase 14: Temporal Controls (1/1 plans) — completed 2026-03-24

Full details: `.planning/milestones/v1.1.1-ROADMAP.md`

</details>

<details>
<summary>✅ v1.2 Graph Rendering Overhaul (Phases 15-17) — SHIPPED 2026-03-26</summary>

- [x] Phase 15: Force Simulation Rebalancing (2/2 plans) — completed 2026-03-25
- [x] Phase 16: Edge and Node Renderer Fixes (2/2 plans) — completed 2026-03-25
- [x] Phase 17: Viewport Fit and Label Collision (2/2 plans) — completed 2026-03-26

Full details: `.planning/milestones/v1.2-ROADMAP.md`

</details>

<details>
<summary>✅ v1.3 Data Pipeline Fixes (Phases 18-20, 999.1-999.2) — SHIPPED 2026-04-05</summary>

- [x] Phase 18: arXiv Crawl Repair (2/2 plans) — completed 2026-03-28
- [x] Phase 19: Data Quality Cleanup (1/1 plans) — completed 2026-03-28
- [x] Phase 20: LLM Analysis Pipeline Verification (4/4 plans) — completed 2026-03-28
- [x] Phase 999.1: Keyword-Based Graph Labels (4/4 plans) — completed 2026-03-29
- [x] Phase 999.2: Topic Ring Node Borders (2/2 plans) — completed 2026-03-30

Full details: `.planning/milestones/v1.3-ROADMAP.md`

</details>

### 🚧 v1.4 Discovery & Intelligence (In Progress)

**Milestone Goal:** Transform ReSyn from a visualization tool into a genuine discovery engine — with full-text search, paper similarity, graph analytics, community detection, recommendations, and export.

- [x] **Phase 21: Search & Filter** - Full-text search across papers with graph viewport integration (completed 2026-04-07)
- [x] **Phase 22: Paper Similarity Engine** - Cosine similarity on TF-IDF vectors with similar-papers UI (completed 2026-04-09)
- [x] **Phase 23: Graph Analytics — Centrality & Metrics** - PageRank, betweenness centrality, and query optimization (completed 2026-04-09)
- [x] **Phase 24: Community Detection** - Louvain clustering with community summary panels (completed 2026-04-10)
- [ ] **Phase 25: Discovery Recommendations** - Scored paper recommendations combining similarity, centrality, and community signals
- [ ] **Phase 26: Export & Interop** - BibTeX, CSV, and graph JSON export
- [x] **Phase 27: Crawler Speedup** - Eliminate per-paper HTML scrapes via OpenAlex bulk reference-edge pre-ingest; wire OpenAlex API key; fix concept IDs in CLAUDE.md (completed 2026-04-22)
- [x] **Phase 29: Kuramoto-LBD v03 Corpus Build** - Bidirectional S2 crawl from 10 Feynman pair seeds; analyze + export pre-2015 graph; notebook benchmark gate (`n_eval >= 3` AND `BENCH_P10 > 0.15`) — completed 2026-05-05 with **FAIL verdict** (gate not reached: citation graph too sparse, 41 cc / 153 nodes → K_stable bisection diverges; see 29-VERIFICATION.md)
- [ ] **Phase 30: TF-IDF Semantic-Edge Graph + Downstream LBD Method** - Path C pivot (EXP-RS-11, pre-registered): c-TF-IDF cosine edges replace citation edges on the pre-2015 slice; τ sweep + connectivity precheck; kuramoto_lbd_v04 notebook to a real `BENCH_P10`. Kill gate 2026-09-30: <3 evaluable pairs or `BENCH_P10 ≤ 0.15` → kill dynamical-substrate line

## Phase Details

### Phase 21: Search & Filter
**Goal**: Users can find papers by title, abstract, or author from anywhere in the UI and jump to them in the graph
**Depends on**: Nothing (independent of other v1.4 phases)
**Requirements**: SRCH-01, SRCH-02, SRCH-03, SRCH-04
**Success Criteria** (what must be TRUE):
  1. User can type a query in a search bar and see ranked paper results filtered by title, abstract, or author
  2. Searching from the graph page pans the viewport to the matching node and briefly highlights it
  3. The papers table filters its displayed rows as the user types in the search bar
  4. Search results are ranked by relevance (not insertion order)
**Plans**: 3 plans
Plans:
- [x] 21-01-PLAN.md — SurrealDB BM25 fulltext indexes, SearchRepository, search_papers server fn
- [x] 21-02-PLAN.md — GlobalSearchBar component with dropdown, Ctrl+K, SearchPanTrigger context
- [x] 21-03-PLAN.md — Graph pan/highlight integration, papers table inline filter with match highlighting
**UI hint**: yes

### Phase 22: Paper Similarity Engine
**Goal**: Users can see which papers are most similar to any given paper, with similarity edges optionally shown on the graph
**Depends on**: Nothing (TF-IDF vectors already exist from v1.0)
**Requirements**: SIM-01, SIM-02, SIM-03, SIM-04
**Success Criteria** (what must be TRUE):
  1. The system stores top-10 most similar papers per paper in SurrealDB, computed from existing TF-IDF cosine similarity
  2. User can open the paper detail drawer and view a "Similar Papers" tab listing ranked similar papers
  3. User can toggle a similarity edge overlay on the graph that draws edges between similar papers
  4. After TF-IDF analysis completes, similarity scores are automatically recomputed for the analyzed papers
**Plans**: 3 plans
Plans:
- [x] 22-01-PLAN.md — PaperSimilarity model, compute_top_neighbors, SimilarityRepository, pipeline trigger
- [x] 22-02-PLAN.md — DrawerTab::Similar, get_similar_papers server fn, SimilarTabBody component
- [x] 22-03-PLAN.md — EdgeType::Similarity rendering, graph controls toggles, dual force model
**UI hint**: yes

### Phase 23: Graph Analytics — Centrality & Metrics
**Goal**: Users can explore which papers are most structurally influential using PageRank and betweenness centrality, and node sizes reflect the chosen metric
**Depends on**: Nothing (independent computation on existing citation graph)
**Requirements**: GANA-01, GANA-02, GANA-03, GANA-04, GANA-05, GANA-06
**Success Criteria** (what must be TRUE):
  1. System computes PageRank and betweenness centrality for all papers and caches results in SurrealDB with corpus-fingerprint invalidation
  2. User can select a "Size by" dropdown (Uniform / PageRank / Betweenness / Citations) and graph nodes resize accordingly
  3. Dashboard shows a "Most Influential Papers" ranking panel ordered by PageRank score
  4. Citation graph queries use single SurrealDB JOINs rather than per-paper N+1 lookups
**Plans**: 3 plans
Plans:
- [x] 23-01-PLAN.md — GraphMetrics data model, migration 11, GraphMetricsRepository, N+1 query fix
- [x] 23-02-PLAN.md — PageRank + Brandes betweenness computation, server fns, pipeline auto-compute
- [x] 23-03-PLAN.md — SizeMode dropdown, node radius lerp, tooltip metrics, dashboard influential card
**UI hint**: yes

### Phase 24: Community Detection
**Goal**: Users can see how the citation graph clusters into research communities and explore each community's character
**Depends on**: Phase 23 (needs graph_metrics table and PageRank for community ranking)
**Requirements**: COMM-01, COMM-02, COMM-03
**Success Criteria** (what must be TRUE):
  1. System detects communities in the citation graph via Louvain modularity optimization and stores assignments in SurrealDB
  2. User can select "Community" in the "Color by" dropdown (alongside BFS Depth and Topic) and graph nodes are colored by their community
  3. User can open a community summary panel that shows top papers, dominant keywords, and shared methods for each detected community
**Plans**: 3 plans
Plans:
- [x] 24-01-PLAN.md — CommunityAssignment model, migration 12, Louvain impl, CommunityRepository, c-TF-IDF labels
- [x] 24-02-PLAN.md — ColorMode::Community, 300ms lerp, Color by dropdown, community legend chips
- [x] 24-03-PLAN.md — DrawerTab::Community, CommunityTabBody, trigger_community_compute, post-crawl Stage 6 auto-compute
**UI hint**: yes

### Phase 25: Discovery Recommendations
**Goal**: Users receive ranked paper recommendations with explanations drawn from similarity, centrality, and community bridge signals
**Depends on**: Phase 22 (similarity neighbors), Phase 23 (centrality scores), Phase 24 (community assignments)
**Requirements**: DISC-01, DISC-02, DISC-03, DISC-04
**Success Criteria** (what must be TRUE):
  1. System generates scored recommendations for each seed paper by combining cosine similarity, PageRank, and cross-community bridge signals
  2. Each recommendation card displays a human-readable explanation of why the paper was suggested
  3. User can visit a "Discover" page, pick a seed paper, and browse recommendation cards
  4. Dashboard shows a "Suggested Reads" card with the top 3 recommendations for the current corpus
**Plans**: TBD
**UI hint**: yes

### Phase 26: Export & Interop
**Goal**: Users can export papers and graph data in standard formats for use in reference managers, spreadsheets, and downstream tools
**Depends on**: Nothing (independent of other v1.4 phases)
**Requirements**: EXPO-01, EXPO-02, EXPO-03
**Success Criteria** (what must be TRUE):
  1. User can export a single paper or a selection as a BibTeX file from the detail drawer or papers table
  2. User can export the full papers table as a CSV file containing all available metadata columns
  3. User can export the graph (nodes, edges, and computed metrics) as a JSON file from the graph page
**Plans**: TBD
**UI hint**: yes

### Phase 27: Crawler Speedup
**Goal**: Eliminate per-paper HTML scrapes for reference fetching by pre-ingesting OpenAlex citation edges into the target corpus DB; wire OpenAlex API key authentication; fix incorrect concept IDs in CLAUDE.md
**Depends on**: Nothing (independent infrastructure improvement)
**Requirements**: None (not yet mapped)
**Success Criteria** (what must be TRUE):
  1. `bulk-ingest` authenticates with `OPENALEX_API_KEY` env var instead of `--mailto` polite pool
  2. A physics filter constant (`C121864883` Statistical physics + `C26873012` Condensed matter) is available in `bulk_ingest.rs` for corpus-specific ingest runs
  3. CLAUDE.md concept ID `C2778407487` ("Statistical Physics") is corrected to the verified IDs above
  4. Running `cargo run --bin resyn -- bulk-ingest --db surrealkv://./data-physics` populates both papers and citation edges without touching any other DB path
**Plans**: 2 plans
Plans:
- [x] 27-01-PLAN.md — API key auth migration (openalex_bulk.rs + bulk_ingest.rs)
- [x] 27-02-PLAN.md — Physics filter constant + CLAUDE.md concept ID fix
**UI hint**: no

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Text Extraction Foundation | v1.0 | 2/2 | Complete | 2026-03-14 |
| 2. NLP Analysis + DB Schema | v1.0 | 2/2 | Complete | 2026-03-14 |
| 3. Pluggable LLM Backend | v1.0 | 3/3 | Complete | 2026-03-14 |
| 4. Cross-Paper Gap Analysis | v1.0 | 3/3 | Complete | 2026-03-14 |
| 5. Visualization Enrichment | v1.0 | 2/2 | Complete | 2026-03-14 |
| 6. Tech Debt + Workspace Restructure | v1.1 | 2/2 | Complete | 2026-03-15 |
| 7. Incremental Crawl Infrastructure | v1.1 | 5/5 | Complete | 2026-03-16 |
| 8. Leptos Web Shell + Analysis Panels | v1.1 | 7/7 | Complete | 2026-03-17 |
| 9. Graph Renderer (Canvas to WebGL) | v1.1 | 5/5 | Complete | 2026-03-18 |
| 10. Analysis UI Polish + Scale | v1.1 | 4/4 | Complete | 2026-03-18 |
| 11. SPA Routing | v1.1.1 | 1/1 | Complete | 2026-03-23 |
| 12. Graph Force & Rendering | v1.1.1 | 1/1 | Complete | 2026-03-23 |
| 13. Graph Interaction | v1.1.1 | 1/1 | Complete | 2026-03-23 |
| 14. Temporal Controls | v1.1.1 | 1/1 | Complete | 2026-03-24 |
| 15. Force Simulation Rebalancing | v1.2 | 2/2 | Complete | 2026-03-25 |
| 16. Edge and Node Renderer Fixes | v1.2 | 2/2 | Complete | 2026-03-25 |
| 17. Viewport Fit and Label Collision | v1.2 | 2/2 | Complete | 2026-03-26 |
| 18. arXiv Crawl Repair | v1.3 | 2/2 | Complete | 2026-03-28 |
| 19. Data Quality Cleanup | v1.3 | 1/1 | Complete | 2026-03-28 |
| 20. LLM Analysis Pipeline Verification | v1.3 | 4/4 | Complete | 2026-03-28 |
| 999.1. Keyword-Based Graph Labels | v1.3 | 4/4 | Complete | 2026-03-29 |
| 999.2. Topic Ring Node Borders | v1.3 | 2/2 | Complete | 2026-03-30 |
| 21. Search & Filter | v1.4 | 3/3 | Complete    | 2026-04-09 |
| 22. Paper Similarity Engine | v1.4 | 3/3 | Complete    | 2026-04-09 |
| 23. Graph Analytics — Centrality & Metrics | v1.4 | 3/3 | Complete    | 2026-04-09 |
| 24. Community Detection | v1.4 | 3/3 | Complete   | 2026-04-10 |
| 25. Discovery Recommendations | v1.4 | 0/? | Not started | - |
| 26. Export & Interop | v1.4 | 0/? | Not started | - |
| 27. Crawler Speedup | v1.4 | 2/2 | Complete    | 2026-04-22 |
| 28. Forward-citation crawl mode (S2) | v1.4 | 4/4 | Complete    | 2026-04-27 |
| 29. Kuramoto-LBD v03 Corpus Build | v1.4 | 1/1 | Complete (FAIL verdict — gate not reached) | 2026-05-05 |
| 30. TF-IDF Semantic-Edge Graph (EXP-RS-11) | v1.4 | 1/1 | Complete (FAIL verdict — pivot kill gate fired) | 2026-07-04 |
| 31. Dynamical LBD on full-corpus giant CC (EXP-RS-12) | v1.4 | 1/1 | Complete (MIXED — methodology fix validated; BENCH_P10=0, corpus-content gap isolated) | 2026-07-04 |
| 32. Bridge-containing corpus re-test (EXP-RS-13) | v1.4 | 0/1 | In progress (corpus built: 1400 nodes, 3/4 pairs bridged; v06 running) | - |

### Phase 28: Forward-citation crawl mode (S2)

**Goal:** Add bidirectional citation discovery to SemanticScholarSource. Implement `--bidirectional` CLI flag that fetches S2 forward-citations alongside backward-citations, writes correct-direction graph edges via a new `PaperRepository::upsert_inverse_citations_batch`, and enqueues newly discovered citing papers in the existing BFS queue.
**Requirements**: None (infrastructure improvement; no v1.4 requirement IDs map to this phase)
**Depends on:** Phase 27
**Plans:** 4/4 plans complete

Plans:
- [x] 28-01-PLAN.md — S2 fetch_citing_papers method + builder fields + wiremock tests (Wave 1, parallel with 28-02)
- [x] 28-02-PLAN.md — PaperSource trait extension + Paper.citing_papers transient field + accessor (Wave 1, parallel with 28-01)
- [x] 28-03-PLAN.md — PaperRepository::upsert_inverse_citations_batch with edge-direction integration tests (Wave 2)
- [x] 28-04-PLAN.md — Crawler wiring (CLI flags + worker block + non-S2 warn) + script + CLAUDE.md docs (Wave 3)

### Phase 29: Kuramoto-LBD v03 Corpus Build

**Goal:** Build the citation graph that the deferred Kuramoto-LBD v03 prototype notebook needs (10 Feynman pair seeds, bidirectional S2 crawl, depth 2), run analysis + export, and run the notebook as the success gate. Outcome: empirical pass/fail on the dynamical-LBD direction, gating EXP-RS-07 Sheaves-LBD.
**Requirements**: None (exploratory benchmark phase; no v1.4 requirement IDs map to this phase)
**Depends on:** Phase 28 (bidirectional crawl mode)
**Success Criteria:**
  1. `data-kuramoto` corpus built via `--bidirectional` S2 crawl from 10 seeds at depth 2
  2. `analyze` + `export-louvain-graph` produce a valid `research_synergy_pre2015.json`
  3. At least 3 of 5 evaluable Feynman pairs map to non-Other communities (`n_eval >= 3`)
  4. `kuramoto_lbd_v03.ipynb` executes without ABORT and reports `BENCH_P10 > 0.15`
**Plans:** 1 plan
Plans:
- [x] 29-01-PLAN.md — Build corpus, run analysis, export pre-2015 graph, run notebook benchmark gate (executed to FAIL verdict 2026-05-05; deviations recorded in 29-VERIFICATION.md)
**UI hint**: no

### Phase 30: TF-IDF Semantic-Edge Graph + Downstream LBD Method

**Goal:** Execute the approved Path C pivot (EXP-RS-11, pre-registered in the vault): build a TF-IDF cosine semantic-edge graph from the Phase 29 pre-2015 export, verify connectivity across τ ∈ {0.2, 0.3, 0.4, 0.5}, and run the kuramoto_lbd_v04 notebook to a real `BENCH_P10` on the shared 10-pair Feynman benchmark.
**Requirements**: None (exploratory research phase; Dynamical-LBD Gen-4 thread)
**Depends on:** Phase 29 (data-kuramoto corpus + Louvain partition, reused as-is; no new crawling)
**Success Criteria** (claims→acceptance-tests contract in 30-01-PLAN.md):
  1. `build_tfidf_graph.py` committed before real-data run; τ sweep reports (n_nodes, n_edges, n_cc, largest_cc_size, mean_degree) per τ
  2. Connectivity precheck evaluated: dynamics run only where `n_cc/N ≤ 0.05`; if no τ passes, the kill gate fires (that IS the result)
  3. `kuramoto_lbd_v04.ipynb` executes `compute_K_stable` within a 5-minute budget and produces a real `BENCH_P10`
  4. Verdict vs locked EXP-RS-11 predictions recorded in 30-VERIFICATION.md; independent falsification via `/commission --research` before acceptance
**Plans:** 1 plan
Plans:
- [x] 30-01-PLAN.md — Regenerate export, build+verify TF-IDF graph, v04 notebook benchmark, verdict (executed to FAIL verdict 2026-07-04; see 30-VERIFICATION.md)
**Outcome:** ❌ FAIL — TF-IDF cosine semantic edges make the pre-2015 corpus *more* fragmented (`n_cc/N`=0.830 @ τ=0.3) than the citation graph (0.268) at every pre-registered τ. Precheck fails at all τ; `compute_K_stable`/`BENCH_P10` correctly not reached. **Pivot kill gate FIRED** (well before the 2026-09-30 deadline). Verdict independently falsified-and-CONFIRMED via a right-sized `/commission --research` (3 converging lines; no under-connection bug, no leakage/contamination). NOTE (superseded by Phase 31 reanalysis): the "corpus too narrow" reading was wrong — the fragmentation was caused by the pre-2015 slice, not corpus size; the full corpus is well-posed.
**UI hint**: no

### Phase 31: Dynamical LBD on the full-corpus giant component (EXP-RS-12)

**Goal:** Produce the first real `BENCH_P10` for the dynamical-LBD pipeline by running the validated v03 citation-adjacency pipeline on the **well-posed** 224-node giant component of the full `data-kuramoto` corpus — testing whether Phases 29/30 failed on a methodology artifact (the unnecessary pre-2015 slice, C-1) rather than on the method itself.
**Requirements**: None (exploratory research phase; Dynamical-LBD Gen-4 thread). Pre-registration: vault § EXP-RS-12 (predictions LOCKED). Conventions: C-12 (supersedes C-1 — BENCH_P10 is date-agnostic recovery), C-13 (giant-CC citation substrate).
**Depends on:** Phase 29 corpus (`data-kuramoto` / `research_synergy_kuramoto_full.json`, reused as-is; no new crawling).
**Reanalysis basis (2026-07-04):** BENCH_P10 scoring has zero temporal logic; `dynamical-lbd.md` Criterion 3(a) requires only that the corpus "contain both literatures." The pre-2015 filter shattered the citation graph (2→41 components) without improving benchmark coverage. Full corpus: 227 nodes → giant CC 224 (1 component, λ₂>0), 34 communities, 4 evaluable pairs across communities (n_eval=4 ≥ 3).
**Success Criteria** (claims→acceptance in 31-01-PLAN.md):
  1. Giant CC is a single connected component; `compute_K_stable` returns finite K_stable ≤ 300 s
  2. `n_eval = 4 ≥ 3` in the giant CC (structural, verified)
  3. Real `BENCH_P10` produced; evaluated vs 0.15 baseline / 0.30 target — either outcome is a decisive result
  4. Real-graph bridge signal exceeds ER-null and config-model-null controls
**Plans:** 1 plan
Plans:
- [x] 31-01-PLAN.md — Build+commit v05 giant-CC citation pipeline, execute to real BENCH_P10, verdict (executed 2026-07-04; see 31-VERIFICATION.md)
**Outcome:** ⚠️ MIXED. Methodology fix VALIDATED — the 224-node giant CC is well-posed, `K_stable=14.25` converges (Phases 29/30 were connectivity artifacts, reanalysis confirmed). But locked stake P-3 FALSIFIED: `BENCH_P10=0.000`. Decisive static diagnostic: 3/4 evaluable pairs have zero inter-community citation edges (bridge literature absent from the shallow-crawled corpus); pair04's 2-edge bridge is diluted out of the global-top-10. Corpus-CONTENT gap isolated from the (solved) connectivity gap — the method was never fairly tested. Not a clean method-kill. Path forward (human decision): Phase 2 bridge-containing corpus (OpenAlex bulk-ingest) + per-pair metric, vs accept the unproven-method negative → brute-force.
**UI hint**: no
