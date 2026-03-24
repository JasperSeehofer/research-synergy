# Roadmap: Research Synergy (ReSyn)

## Milestones

- ✅ **v1.0 Analysis Pipeline** — Phases 1-5 (shipped 2026-03-14)
- ✅ **v1.1 Scale & Surface** — Phases 6-10 (shipped 2026-03-22)
- 🚧 **v1.1.1 Bug Fix & Polish** — Phases 11-14 (in progress)

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

### 🚧 v1.1.1 Bug Fix & Polish (In Progress)

**Milestone Goal:** Fix all broken v1.1 web UI features — SPA routing, graph force layout, node interaction, temporal slider, and WebGL rendering quality.

- [x] **Phase 11: SPA Routing** - All routes load correctly on direct navigation and browser refresh (completed 2026-03-23)
- [x] **Phase 12: Graph Force & Rendering** - Force layout animates and nodes render without blur (completed 2026-03-23)
- [x] **Phase 13: Graph Interaction** - Node drag, pan, and zoom all respond to user input (completed 2026-03-23)
- [ ] **Phase 14: Temporal Controls** - Dual-range slider both thumbs visible and independently draggable

## Phase Details

### Phase 11: SPA Routing
**Goal**: Users can navigate the full app by URL without hitting 404 or blank pages
**Depends on**: Nothing (independent fix)
**Requirements**: ROUTE-01, ROUTE-02
**Success Criteria** (what must be TRUE):
  1. User can click any sidebar link (Dashboard, Papers, Graph, Gaps) and the correct page loads without a full page reload
  2. User can type `/graph` or `/papers` directly into the browser address bar and the correct page renders
  3. User can press browser refresh on any route and land back on the same page instead of a 404 or blank screen
**Plans:** 1/1 plans complete
Plans:
- [x] 11-01-PLAN.md — Add SPA fallback to Axum static file serving

### Phase 12: Graph Force & Rendering
**Goal**: The citation graph renders all edges crisply and nodes visibly spread apart during force simulation
**Depends on**: Phase 11
**Requirements**: GRAPH-01, GRAPH-02, GRAPH-03
**Success Criteria** (what must be TRUE):
  1. On graph load, nodes visibly move and spread apart — there is observable animation in the first few seconds
  2. Citation edges (lines connecting nodes) are drawn between related papers
  3. Node circles and labels render with sharp, crisp edges at all display zoom levels (no blur or DPR mismatch)
  4. The simulation settles to a stable layout rather than collapsing all nodes to a point or exploding off screen
**Plans**: 1 plan
Plans:
- [x] 12-01-PLAN.md — Fix initial spread, preallocate VBOs, verify rendering
**UI hint**: yes

### Phase 13: Graph Interaction
**Goal**: Users can explore the graph by dragging nodes and navigating the viewport
**Depends on**: Phase 12
**Requirements**: INTERACT-01, INTERACT-02, INTERACT-03
**Success Criteria** (what must be TRUE):
  1. User can click and drag an individual node to a new position and it stays there after release
  2. User can click and drag empty canvas space to pan the entire graph viewport
  3. User can scroll the mouse wheel over the graph to zoom in and out smoothly
  4. After any interaction, node positions and viewport state remain consistent (no jump or reset)
**Plans:** 1/1 plans complete
Plans:
- [x] 13-01-PLAN.md — Fix CSS pointer-events passthrough on overlay containers
**UI hint**: yes

### Phase 14: Temporal Controls
**Goal**: Users can filter the graph by publication year using the dual-range slider
**Depends on**: Phase 11
**Requirements**: TEMPORAL-01
**Success Criteria** (what must be TRUE):
  1. Both the start-year and end-year slider thumbs are visible on screen at the same time
  2. Each thumb can be dragged independently without the other thumb becoming hidden or inaccessible
  3. Moving either thumb updates the year range label and the graph filters accordingly
**Plans:** 1 plan
Plans:
- [ ] 14-01-PLAN.md — Fix dual-range slider CSS pointer-events and add value clamping
**UI hint**: yes

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
| 11. SPA Routing | v1.1.1 | 1/1 | Complete    | 2026-03-23 |
| 12. Graph Force & Rendering | v1.1.1 | 1/1 | Complete   | 2026-03-23 |
| 13. Graph Interaction | v1.1.1 | 1/1 | Complete    | 2026-03-23 |
| 14. Temporal Controls | v1.1.1 | 0/1 | Not started | - |
