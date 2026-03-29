# Roadmap: Research Synergy (ReSyn)

## Milestones

- ✅ **v1.0 Analysis Pipeline** — Phases 1-5 (shipped 2026-03-14)
- ✅ **v1.1 Scale & Surface** — Phases 6-10 (shipped 2026-03-22)
- ✅ **v1.1.1 Bug Fix & Polish** — Phases 11-14 (shipped 2026-03-24)
- ✅ **v1.2 Graph Rendering Overhaul** — Phases 15-17 (shipped 2026-03-26)
- 🚧 **v1.3 Data Pipeline Fixes** — Phases 18-20 (in progress)

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

### v1.3 Data Pipeline Fixes (In Progress)

**Milestone Goal:** Fix the broken arXiv crawl pipeline, eliminate orphan nodes in InspireHEP crawls, and verify the LLM analysis pipeline works end-to-end in the web UI.

- [x] **Phase 18: arXiv Crawl Repair** — Fix HTML reference parser to extract arXiv IDs from plain text, restoring edge-comparable crawl output (completed 2026-03-28)
- [x] **Phase 19: Data Quality Cleanup** — Diagnose and eliminate orphan nodes in InspireHEP crawls; backfill missing publication dates for all crawled papers (completed 2026-03-28)
- [x] **Phase 20: LLM Analysis Pipeline Verification** — Restore end-to-end LLM analysis in the web UI with all result panels functional (completed 2026-03-28)

## Phase Details

### Phase 18: arXiv Crawl Repair
**Goal**: Users can run an arXiv crawl and get a densely connected citation graph with the same edge coverage as InspireHEP for the same seed paper
**Depends on**: Phase 17 (v1.2 complete)
**Requirements**: ARXIV-01, ARXIV-03
**Success Criteria** (what must be TRUE):
  1. User can crawl a seed paper via arXiv source and sees citation edges stored for references that only appear as plain text in the HTML bibliography (no hyperlink)
  2. User can compare an arXiv crawl and an InspireHEP crawl for the same seed paper and observes comparable edge density (not a fraction of InspireHEP)
  3. The arXiv HTML parser extracts arXiv IDs from `arXiv:YYMM.NNNNN` patterns in reference text, not only from `<a>` tags
**Plans**: 2 plans

Plans:
- [x] 18-01-PLAN.md — Add regex dependency, implement text-based arXiv ID/DOI extraction, update get_arxiv_id() fallback
- [x] 18-02-PLAN.md — Integration test with real HTML fixture validating edge density

### Phase 19: Data Quality Cleanup
**Goal**: Users see a fully connected citation graph after any crawl, with published dates present on all papers so temporal filtering works
**Depends on**: Phase 18
**Requirements**: ARXIV-02, ORPH-01, ORPH-02
**Success Criteria** (what must be TRUE):
  1. User can inspect a specific disconnected node from a previous InspireHEP crawl and identify the root cause (missing edge, ID mismatch, dedup error, or crawl boundary)
  2. User runs a depth-2+ InspireHEP crawl and sees zero orphan nodes in the resulting graph — every node has at least one visible edge
  3. User can run the temporal year-range slider and see papers filter in/out — all crawled papers have non-null published dates including those fetched via reference parsing
**Plans**: 1 plan

Plans:
- [x] 19-01-PLAN.md — InspireHEP published date extraction + empty-ID orphan elimination

### Phase 20: LLM Analysis Pipeline Verification
**Goal**: Users can trigger LLM analysis from the web UI and view all analysis results (gap findings, open problems, method heatmap) populated with real data
**Depends on**: Phase 19
**Requirements**: LLM-01, LLM-02, LLM-03, LLM-04
**Success Criteria** (what must be TRUE):
  1. User can click the analysis trigger in the web UI, watch it run, and see it complete without errors
  2. User can view the gap findings panel and see contradiction edges and ABC-bridge badges rendered for papers where contradictions and bridges were detected
  3. User can view the open problems panel and see problems ranked by recurrence frequency across the crawled paper set
  4. User can view the method heatmap and see a populated matrix distinguishing existing method combinations from absent ones
**Plans**: 4 plans

Plans:
- [x] 20-01-PLAN.md — Backend wiring: StartAnalysis server fn, ProgressEvent extension, process::exit removal
- [x] 20-02-PLAN.md — Dashboard UI: analysis controls, LLM warning banner, sidebar progress display
- [x] 20-03-PLAN.md — Result panel updates: CTA empty states and SSE-triggered refetch for all panels
- [x] 20-04-PLAN.md — Integration tests: wiremock Ollama pipeline verification

## Backlog

### Phase 999.1: Keyword-Based Graph Labels (BACKLOG)

**Goal:** Replace author/year labels with TF-IDF keyword pills so users can visually identify paper topics at a glance. Two tiers: (1) per-node keyword pills showing top 2 keywords with score-based opacity, (2) cluster-level floating labels with dashed convex hull borders showing dominant topic per region. Label mode dropdown (Keywords / Author-Year / Off) in graph controls.
**Requirements:** KGL-01, KGL-02, KGL-03, KGL-04, KGL-05, KGL-06
**Plans:** 4/4 plans complete

Plans:
- [x] 999.1-01-PLAN.md — Data pipeline: extend GraphNode/NodeState with top_keywords, define LabelMode enum, server function AnalysisRepository join
- [x] 999.1-02-PLAN.md — K-means clustering module: Lloyd's algorithm, convex hull, auto_k heuristic, dominant keyword aggregation
- [x] 999.1-03-PLAN.md — Label mode dropdown + per-node keyword pill rendering with score-based opacity
- [x] 999.1-04-PLAN.md — Cluster rendering (convex hull borders, LOD transition) + extended hover tooltip with top-5 keywords

### Phase 999.2: Topic Ring Node Borders (BACKLOG)

**Goal:** Encode each node's top-3 TF-IDF keywords as colored arc segments on its border ring, creating a visual topic fingerprint. Global corpus keywords each get a fixed color (variance-ranked). Arc length proportional to normalized TF-IDF score. Includes a keyword-to-color legend panel with click-to-filter interaction, and an independent "Topic Rings" toggle.
**Requirements:** D-01, D-02, D-03, D-04, D-05, D-06, D-07, D-08, D-09, D-10, D-11, D-12, D-13, D-14, D-15
**Plans:** 1/2 plans executed

Plans:
- [x] 999.2-01-PLAN.md — Data pipeline: PaletteEntry type, DB migration 8, PaletteRepository CRUD, palette variance computation, GraphState/NodeState extensions
- [ ] 999.2-02-PLAN.md — Canvas2D arc rendering, Topic Rings toggle, Topic Colors legend with click-to-filter, CSS styles

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
| 18. arXiv Crawl Repair | v1.3 | 2/2 | Complete    | 2026-03-28 |
| 19. Data Quality Cleanup | v1.3 | 1/1 | Complete    | 2026-03-28 |
| 20. LLM Analysis Pipeline Verification | v1.3 | 4/4 | Complete    | 2026-03-28 |
