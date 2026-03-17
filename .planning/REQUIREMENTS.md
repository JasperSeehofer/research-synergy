# Requirements: Research Synergy (ReSyn)

**Defined:** 2026-03-15
**Core Value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph

## v1.1 Requirements

Requirements for the Scale & Surface milestone. Each maps to roadmap phases.

### Tech Debt

- [x] **DEBT-01**: Export `nlp` module in `lib.rs` for test/library access
- [x] **DEBT-02**: Remove stale stub comment in `src/llm/ollama.rs`
- [x] **DEBT-03**: Clean up stale ROADMAP plan checkboxes from v1.0
- [ ] **DEBT-04**: Section-aware LLM extraction using detected section boundaries

### Crawl Infrastructure

- [x] **CRAWL-01**: DB-backed crawl queue replacing in-memory BFS frontier for resumability
- [x] **CRAWL-02**: Crash recovery — resume interrupted crawls from last checkpoint
- [x] **CRAWL-03**: Crawl progress reporting via SSE (papers found, queue depth, estimated time)
- [x] **CRAWL-04**: Parallel reference fetching where rate limits allow

### Web Migration

- [x] **WEB-01**: Cargo workspace restructure into 3-crate layout (core/app/server)
- [x] **WEB-02**: WASM compilation boundary — SurrealDB feature-gated behind `ssr`
- [x] **WEB-03**: Leptos CSR shell with Trunk build pipeline and routing
- [x] **WEB-04**: Axum server functions exposing analysis pipeline to frontend
- [x] **WEB-05**: Remove egui/eframe/fdg dependencies

### Graph Visualization

- [x] **GRAPH-01**: Canvas 2D renderer via web-sys with Web Worker force layout (full Rust/WASM)
- [x] **GRAPH-02**: Pan/zoom/hover interactions matching current egui feature set
- [ ] **GRAPH-03**: WebGL2 upgrade via web-sys for 1000+ node rendering (full Rust)
- [x] **GRAPH-04**: Barnes-Hut O(n log n) force layout in Rust/WASM replacing fdg

### Analysis UI

- [x] **AUI-01**: Gap findings rendered in graph (contradiction edges, bridge badges)
- [x] **AUI-02**: Open-problems aggregation panel ranked by recurrence frequency
- [x] **AUI-03**: Method-combination gap matrix showing existing vs absent method pairings
- [ ] **AUI-04**: Analysis provenance tracking — click a finding, see source text segment

### Scale & Polish

- [ ] **SCALE-01**: Real test runs at depth 2, 3, 5 with performance profiling
- [ ] **SCALE-02**: Node clustering / level-of-detail for 1000+ node graphs
- [ ] **SCALE-03**: Temporal filtering by publication year

## Future Requirements

Deferred to v1.2+. Tracked but not in current roadmap.

### Visualization

- **VIZ-01**: 3D multidimensional projection of paper embeddings (PCA/UMAP)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Real-time collaborative analysis | Single-user tool for now |
| Citation prediction / paper recommendation | Focus is on gap surfacing, not suggesting new papers |
| Full-text indexing / search engine | Analysis is structured extraction, not free-text search |
| Non-arXiv PDF sources | Only papers reachable through existing data sources |
| Fine-tuning custom models | Use off-the-shelf LLM APIs with prompt engineering |
| LaTeX source parsing | ar5iv HTML is simpler and sufficient |
| SSR / server-side rendering | CSR-only — single-user local tool, no SEO/TTFB needs |
| JavaScript graph libraries (sigma.js, d3) | Full Rust/WASM stack preferred — use web-sys WebGL2 bindings |
| Multi-user collaboration | Single-user research tool |
| Auto-expanding graph by similarity | Scope creep — explicit crawl control only |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| DEBT-01 | Phase 6 | Complete |
| DEBT-02 | Phase 6 | Complete |
| DEBT-03 | Phase 6 | Complete |
| WEB-01 | Phase 6 | Complete |
| WEB-02 | Phase 6 | Complete |
| WEB-05 | Phase 6 | Complete |
| CRAWL-01 | Phase 7 | Complete |
| CRAWL-02 | Phase 7 | Complete |
| CRAWL-03 | Phase 7 | Complete |
| CRAWL-04 | Phase 7 | Complete |
| WEB-03 | Phase 8 | Complete |
| WEB-04 | Phase 8 | Complete |
| AUI-01 | Phase 8 | Complete |
| AUI-02 | Phase 8 | Complete |
| AUI-03 | Phase 8 | Complete |
| GRAPH-01 | Phase 9 | Complete |
| GRAPH-02 | Phase 9 | Complete |
| GRAPH-03 | Phase 9 | Pending |
| GRAPH-04 | Phase 9 | Complete |
| AUI-04 | Phase 10 | Pending |
| DEBT-04 | Phase 10 | Pending |
| SCALE-01 | Phase 10 | Pending |
| SCALE-02 | Phase 10 | Pending |
| SCALE-03 | Phase 10 | Pending |

**Coverage:**
- v1.1 requirements: 24 total
- Mapped to phases: 24
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-15*
*Last updated: 2026-03-15 — traceability populated after roadmap creation*
