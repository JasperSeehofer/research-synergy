# Roadmap: Research Synergy (ReSyn)

## Milestones

- ✅ **v1.0 Analysis Pipeline** — Phases 1-5 (shipped 2026-03-14)
- 🚧 **v1.1 Scale & Surface** — Phases 6-10 (in progress)

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

### 🚧 v1.1 Scale & Surface (In Progress)

**Milestone Goal:** Make ReSyn usable at real research scale (depth 10+) and move gap insights from stdout into the primary interface, migrating to a Leptos web UI with full Rust/WASM graph rendering.

- [x] **Phase 6: Tech Debt + Workspace Restructure** — Clean debt, split into 3-crate workspace, establish WASM boundary (completed 2026-03-15)
- [x] **Phase 7: Incremental Crawl Infrastructure** — DB-backed resumable crawl queue with progress reporting and parallel fetching (completed 2026-03-15)
- [ ] **Phase 8: Leptos Web Shell + Analysis Panels** — CSR Leptos app, Axum server functions, gap analysis panels
- [ ] **Phase 9: Graph Renderer (Canvas to WebGL)** — Rust/WASM Canvas 2D renderer, Barnes-Hut force layout, WebGL upgrade
- [ ] **Phase 10: Analysis UI Polish + Scale** — Provenance tracking, section-aware LLM, scale testing, LOD, temporal filter

## Phase Details

### Phase 6: Tech Debt + Workspace Restructure
**Goal**: The codebase is cleanly split into a 3-crate Cargo workspace with SurrealDB feature-gated behind `ssr`, all 153 existing tests pass, and small v1.0 debt items are resolved before any migration work begins
**Depends on**: Nothing (first v1.1 phase)
**Requirements**: DEBT-01, DEBT-02, DEBT-03, WEB-01, WEB-02, WEB-05
**Success Criteria** (what must be TRUE):
  1. `cargo test` passes across all workspace crates with the same 153 tests from v1.0
  2. `cargo build --target wasm32-unknown-unknown -p app` compiles without SurrealDB linker errors
  3. `use resyn_core::nlp` is accessible from test and library contexts (DEBT-01 resolved)
  4. egui, eframe, egui_graphs, and fdg dependencies are removed from Cargo.toml (WEB-05)
  5. Stale stub comment in `src/llm/ollama.rs` and stale ROADMAP plan checkboxes are gone
**Plans:** 2/2 plans complete
Plans:
- [ ] 06-01-PLAN.md — Workspace skeleton, source migration, ssr feature gate, WASM boundary verification
- [ ] 06-02-PLAN.md — Visualization removal, egui dep cleanup, CLI subcommand rewrite, tech debt fixes

### Phase 7: Incremental Crawl Infrastructure
**Goal**: High-depth crawls survive crashes and can be monitored in real time, with a DB-backed queue replacing the in-memory BFS frontier
**Depends on**: Phase 6
**Requirements**: CRAWL-01, CRAWL-02, CRAWL-03, CRAWL-04
**Success Criteria** (what must be TRUE):
  1. A depth-5 crawl interrupted mid-run resumes from the last checkpoint on restart with no duplicate fetches
  2. Running `--progress` with `--db` shows a live progress stream (papers found, queue depth) via SSE, readable via `curl`
  3. A depth-3 crawl with `--parallel` completes faster than sequential while respecting the global arXiv 3s rate limit across all concurrent tasks
  4. Papers already in the DB are skipped without network requests when resuming a completed crawl
**Plans:** 5/5 plans complete
Plans:
- [ ] 07-01-PLAN.md — DB migration, CrawlQueueRepository, governor rate limiter
- [ ] 07-02-PLAN.md — Queue-driven crawl loop with parallel workers
- [ ] 07-03-PLAN.md — SSE progress server, queue management subcommands, end-to-end verification
- [ ] 07-04-PLAN.md — Gap closure: fix empty pdf_url fallback for reference fetching (UAT Test 1)
- [ ] 07-05-PLAN.md — Gap closure: fix --db flag on crawl subcommands (UAT Tests 2-5)

### Phase 8: Leptos Web Shell + Analysis Panels
**Goal**: The browser app serves the analysis pipeline's output — contradiction findings, bridge connections, open-problems, and method gaps — without touching the graph canvas
**Depends on**: Phase 6
**Requirements**: WEB-03, WEB-04, AUI-01, AUI-02, AUI-03
**Success Criteria** (what must be TRUE):
  1. `trunk serve` starts the app and the browser renders paper list data fetched from the Axum server via Leptos server functions
  2. Contradiction findings and ABC-bridge connections from SurrealDB appear as labeled entries in the gap panel
  3. Open-problems panel shows problems ranked by recurrence count across the crawled corpus
  4. Method-combination gap matrix renders as a heatmap showing existing vs absent method pairings
  5. Crawl progress bar updates in real time from the SSE endpoint established in Phase 7
**Plans:** 5 plans
Plans:
- [ ] 08-01-PLAN.md — Workspace deps, Trunk config, CSS design system, Leptos app shell with Router, Axum serve command
- [ ] 08-02-PLAN.md — Server functions (all data access), Dashboard summary cards, Papers sortable table, Paper detail drawer
- [ ] 08-03-PLAN.md — Gap findings panel with filterable cards, Open problems ranked list
- [ ] 08-04-PLAN.md — Method-combination heatmap with drill-down, Crawl progress SSE + launcher form
- [ ] 08-05-PLAN.md — Final verification: automated compilation checks + human browser verification

### Phase 9: Graph Renderer (Canvas to WebGL)
**Goal**: The citation graph renders interactively in the browser using a full Rust/WASM pipeline — Canvas 2D initially, WebGL2 when scale demands it — with Barnes-Hut force layout computed in a Web Worker
**Depends on**: Phase 8
**Requirements**: GRAPH-01, GRAPH-02, GRAPH-03, GRAPH-04
**Success Criteria** (what must be TRUE):
  1. The citation graph renders in the browser canvas with pan, zoom, and node-hover interactions matching the retired egui feature set
  2. Clicking a node opens the paper detail sidebar populated with title, authors, abstract, and analysis results
  3. Contradiction edges render in red and ABC-bridge edges render in orange/dashed, overlaid on the same graph
  4. A 1000-node graph maintains interactive frame rate (30+ fps) under the WebGL2 renderer with Barnes-Hut force layout
  5. Force layout converges and stops visibly oscillating within 10 seconds on a 500-node graph
**Plans**: TBD

### Phase 10: Analysis UI Polish + Scale
**Goal**: The full analysis surface is complete — provenance traces findings back to source text, section-aware LLM extraction improves annotation quality, and the app is verified usable at 1000+ node scale with temporal and LOD controls
**Depends on**: Phase 9
**Requirements**: AUI-04, DEBT-04, SCALE-01, SCALE-02, SCALE-03
**Success Criteria** (what must be TRUE):
  1. Clicking a gap finding in the panel highlights the source text segment in a paper sidebar view (provenance tracking)
  2. Re-running analysis on a paper with detected section boundaries produces LLM annotations that reference specific sections (e.g., "Methods", "Results")
  3. Depth-2, depth-3, and depth-5 real crawl runs complete with performance metrics recorded and no regressions vs v1.0
  4. A 1000+ node graph switches to clustered/LOD rendering automatically, keeping the canvas readable
  5. The temporal filter slider narrows the visible graph to papers published within the selected year range
**Plans**: TBD

## Progress

**Execution Order:** 6 → 7 → 8 → 9 → 10

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Text Extraction Foundation | v1.0 | 2/2 | Complete | 2026-03-14 |
| 2. NLP Analysis + DB Schema | v1.0 | 2/2 | Complete | 2026-03-14 |
| 3. Pluggable LLM Backend | v1.0 | 3/3 | Complete | 2026-03-14 |
| 4. Cross-Paper Gap Analysis | v1.0 | 3/3 | Complete | 2026-03-14 |
| 5. Visualization Enrichment | v1.0 | 2/2 | Complete | 2026-03-14 |
| 6. Tech Debt + Workspace Restructure | 2/2 | Complete   | 2026-03-15 | - |
| 7. Incremental Crawl Infrastructure | 5/5 | Complete   | 2026-03-16 | - |
| 8. Leptos Web Shell + Analysis Panels | v1.1 | 0/5 | Planning complete | - |
| 9. Graph Renderer (Canvas to WebGL) | v1.1 | 0/? | Not started | - |
| 10. Analysis UI Polish + Scale | v1.1 | 0/? | Not started | - |
