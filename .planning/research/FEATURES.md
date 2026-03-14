# Feature Research

**Domain:** Literature Based Discovery (LBD) — Leptos web UI, WebGL graph visualization, incremental crawling, enriched gap analysis views
**Project:** Research Synergy (ReSyn) — v1.1 "Scale & Surface" milestone
**Researched:** 2026-03-15
**Confidence:** MEDIUM–HIGH
**Baseline:** v1.0 delivered full analysis pipeline: BFS crawl, arXiv/InspireHEP sources, SurrealDB, petgraph, egui force-directed viz with TF-IDF, LLM annotations, contradiction detection, ABC-bridge, graph enrichment overlays. This file covers ONLY new features for v1.1.

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features a researcher using this tool at depth 10+ expects. Missing any makes the tool feel broken or unusable at real scale.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Web UI accessible in browser | egui is a desktop app binary; researchers expect shareable, bookmark-able, OS-agnostic access | HIGH | Full Leptos migration — significant but scoped. `leptos_axum` + `wasm-pack` is the standard pairing; existing Axum server work reuses the current tokio runtime |
| Crawl progress visibility | Depth-10 crawls take minutes; blind wait is unacceptable; user needs to know it's working | MEDIUM | SSE stream from server to browser; Leptos `use_event_source` or `leptos_sse` crate handles client side; server pushes paper-fetched events as they happen |
| Resumable crawl after interruption | At depth 10+ with rate limits, a crash means 30+ min lost; users expect retry-from-checkpoint | MEDIUM | DB-backed queue of pending arXiv IDs with `status: pending|in_progress|done`; on restart, pick up `pending` rows. SurrealDB graph queries already track what's been fetched |
| Gap findings visible in graph (not just stdout) | v1.0 routes contradictions and bridges to stdout; the graph is where users are looking | MEDIUM | Edge/badge overlays on the force graph; requires wiring existing `GapFinding` records from SurrealDB into the renderer |
| Responsive graph at 1000+ nodes | Citation graphs at depth 10 easily exceed 1000 nodes; canvas-based rendering degrades below usable FPS | HIGH | WebGL rendering path required; Barnes-Hut O(n log n) force layout for simulation; canvas is acceptable up to ~500 nodes, WebGL above that |
| Paper detail panel / sidebar | Clicking a node and seeing its data is fundamental; currently requires egui hover tooltip | LOW | React-like side panel in Leptos; populate from server function fetching paper + analysis from SurrealDB |
| URL-addressable graph state | Users need to share a specific graph view (seed paper, depth) via URL | LOW | Leptos router with query params: `?paper=2301.12345&depth=3&source=arxiv` — standard Leptos routing |

### Differentiators (Competitive Advantage)

Features beyond what Connected Papers, Litmaps, and ResearchRabbit offer. These are where ReSyn wins at depth 10+ research.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Open-problems aggregation panel | Ranked list of what the corpus collectively admits is unsolved; no existing tool surfaces this | MEDIUM | Aggregate `open_problems` from existing `LlmAnnotation` records; cluster by semantic similarity; count recurrence; render as sorted list with source paper links. Data already exists in SurrealDB |
| Method-combination gap matrix | Visual matrix of method pairings that appear vs conspicuously absent; shows where no one has tried combining approaches | MEDIUM | Build method vocabulary from `methods` field in `LlmAnnotation`; compute pairwise co-occurrence; render as heatmap with absent cells highlighted |
| Analysis provenance: click a finding, see source text | Trust and spot-checking: researcher must be able to verify an extracted claim against the original text | MEDIUM | Store `source_segment` + char offsets alongside extracted fields (schema already has stub); render in detail panel with scroll-to-highlight |
| Contradiction edges on graph | Visual encoding of which paper pairs have detected contradictions; no existing citation tool shows this | MEDIUM | Load `GapFinding` records with type `contradiction` from SurrealDB; render as dashed red edges between conflicting nodes in the force graph |
| ABC-bridge edges on graph | Visual encoding of hidden-connection pairs; exclusive to ReSyn among citation tools | MEDIUM | Same pattern as contradiction edges; load `GapFinding` type `bridge`; render as dashed green edges or node badges |
| Temporal filtering slider | Filter the graph to papers before/after a year; see how the field evolved | MEDIUM | `published` field already on `Paper` model; slider signal in Leptos that filters the rendered node set; no new data fetching |
| Node clustering / level-of-detail at scale | At 1000+ nodes, the graph is unreadable without grouping; cluster by primary method or paper type | HIGH | Barnes-Hut naturally handles layout at scale; visual clustering requires grouping nodes by extracted dimension and rendering cluster hulls. This is the hardest UX problem in this milestone |
| Section-aware LLM extraction | Using section boundaries rather than full-text blob improves extraction precision; existing pipeline treats text as a flat string | MEDIUM | Section detection via heading patterns already partially implemented; route `methods_section`, `results_section` etc. as separate prompts |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Real-time collaborative multi-user graphs | "Can multiple researchers work on this simultaneously?" | Requires auth, session management, conflict resolution, WebSocket per-session state — disproportionate scope for a single-user tool | Accept single-user; make graph state shareable via URL params so researchers can reproduce each other's views |
| Full-text keyword search across corpus | "I want to search for papers mentioning 'renormalization group'" | This becomes a search engine, duplicating Semantic Scholar; focus shifts from gap analysis to retrieval | Expose structured SurrealDB filter on extracted `methods` / `keywords` field; not free-text search |
| Auto-expanding the graph based on similarity | "Add papers similar to these even if not cited" | Escapes the citation-graph anchor; user loses control of what's in the corpus; recommendation problem, not gap analysis | Keep corpus user-defined; let depth parameter control expansion |
| 3D force graph | "The 3D view looks more impressive" | 3D force graphs are visually compelling but analytically confusing; camera orientation becomes a cognitive burden; WebXR dependency adds build complexity | 2D WebGL with good zoom, pan, and LOD is analytically superior; defer 3D to v1.2+ if demand exists |
| Exporting to LaTeX / PDF citation report | "I want a formatted literature review" | Document generation is a separate product; ReSyn's value is in the visual analysis, not report writing | Export to JSON/CSV of gap findings for users who want to integrate with their own reporting workflow |
| Live paper alerts ("new paper in my graph's topic") | "Notify me when a new arXiv paper cites X" | Requires a persistent server, polling arXiv API on schedule, user accounts and email — operational complexity far beyond current scope | Out of scope until v2 when multi-user / persistence layer matures |

---

## Feature Dependencies

```
[Leptos web migration]
    └── provides UI shell for all other web features
          ├──requires──> [Axum backend server]
          │                   └── already exists (tokio runtime, SurrealDB connection)
          ├──enables──> [URL-addressable graph state]
          ├──enables──> [Paper detail sidebar]
          └──enables──> [Crawl progress UI]

[Crawl progress UI]
    └──requires──> [Incremental/resumable crawl (DB-backed queue)]
                        └──requires──> [SurrealDB crawl_queue table] (new schema)

[WebGL graph renderer]
    └──requires──> [Graph data serialized to JSON for JS boundary]
    └──requires──> [Leptos web migration] (rendered in browser)
    └──enables──> [Node clustering / LOD at 1000+ nodes]
    └──enables──> [Temporal filtering slider]

[Gap findings in graph]
    └──requires──> [WebGL or Canvas graph renderer in browser]
    └──requires──> [GapFinding records in SurrealDB] (already exists from v1.0)
          ├──enables──> [Contradiction edges]
          └──enables──> [ABC-bridge edges]

[Open-problems panel]
    └──requires──> [LlmAnnotation.open_problems in SurrealDB] (exists from v1.0)
    └──requires──> [Leptos web migration]

[Method-combination gap matrix]
    └──requires──> [LlmAnnotation.methods in SurrealDB] (exists from v1.0)
    └──requires──> [Leptos web migration]

[Analysis provenance]
    └──requires──> [source_segment field stored at extraction time] (stub exists, needs population)
    └──requires──> [Paper detail sidebar]

[Section-aware LLM extraction]
    └──requires──> [Full-text extraction] (exists from v1.0)
    └──requires──> [NLP section detection] (exists from v1.0)
    └──enhances──> [Analysis provenance]
```

### Dependency Notes

- **WebGL graph renderer requires Leptos migration first:** The graph renderer runs in a browser canvas/WebGL context; it cannot be ported until the Leptos shell exists to mount it into.
- **Gap findings wiring requires v1.0 gap analysis data:** The `GapFinding` SurrealDB records already exist; this is a frontend wiring task, not a data generation task. Dependency is on the renderer, not on re-running analysis.
- **Open-problems panel and method matrix are data-ready:** Both depend only on `LlmAnnotation` records already in SurrealDB and the Leptos shell. They can be built in parallel with the graph renderer.
- **Incremental crawl is independent of web UI:** The DB-backed queue can be implemented and tested via CLI before the web UI exists; progress SSE is the only web-dependent part.
- **Section-aware extraction conflicts with existing extraction:** Changing how extraction runs requires care not to invalidate cached `LlmAnnotation` records. Use a version field or separate `extraction_version` on the record.

---

## MVP Definition

### Launch With (v1.1 milestone)

Minimum set that makes ReSyn usable at real research scale in a browser.

- [ ] **Tech debt cleanup** — wire existing gap findings into egui visualization before migration; remove stale stubs. Ensures nothing is lost in migration.
- [ ] **Incremental/resumable crawl with DB queue** — depth 10+ requires this; without it, a rate-limit error means starting over. Essential for real use.
- [ ] **Leptos web migration (shell + routing)** — replace egui binary with a browser-accessible app. The foundational change everything else depends on.
- [ ] **Graph renderer in browser** — even canvas-based at first; the graph must render in the browser for the migration to be usable.
- [ ] **Crawl progress via SSE** — visibility into depth-10 crawl; without this, the UI appears frozen.
- [ ] **Paper detail sidebar** — click a node, see its data; this is the most basic graph interaction expectation.
- [ ] **Gap findings surfaced in graph** — contradiction and bridge edges/badges; this is the core v1.1 promise ("move gap insights into primary interface").

### Add After Core Works (v1.1 polish)

Features that add significant value once the foundation is stable.

- [ ] **WebGL renderer upgrade** — replace canvas with WebGL when 1000+ node performance becomes an observed problem. Don't optimize prematurely.
- [ ] **Open-problems aggregation panel** — data exists; wire into sidebar/panel. High value, low risk.
- [ ] **Method-combination gap matrix** — heatmap of method pairings; medium effort, high analytical value.
- [ ] **Temporal filtering slider** — `published` field already exists; filtering signal in Leptos is straightforward.
- [ ] **URL-addressable graph state** — Leptos router query params; enables sharing.

### Future Consideration (v1.2+)

Defer until v1.1 is stable and user feedback is available.

- [ ] **Node clustering / LOD** — valuable at 1000+ nodes but complex; needs real usage data to tune the clustering heuristics.
- [ ] **Analysis provenance (source text segments)** — requires schema additions and extraction re-runs; do after migration stabilizes.
- [ ] **Section-aware LLM extraction** — improves extraction quality but risks cache invalidation; defer until extraction pipeline is stable post-migration.
- [ ] **3D visualization** — defer; 2D WebGL with good LOD is analytically superior.

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Tech debt / gap wiring cleanup | HIGH | LOW | P1 |
| Incremental/resumable crawl | HIGH | MEDIUM | P1 |
| Leptos web migration (shell) | HIGH | HIGH | P1 |
| Graph rendering in browser (canvas) | HIGH | MEDIUM | P1 |
| Crawl progress SSE | HIGH | MEDIUM | P1 |
| Paper detail sidebar | HIGH | LOW | P1 |
| Gap findings in graph (edges/badges) | HIGH | MEDIUM | P1 |
| WebGL renderer upgrade | MEDIUM | HIGH | P2 |
| Open-problems aggregation panel | HIGH | LOW | P2 |
| Method-combination gap matrix | MEDIUM | MEDIUM | P2 |
| Temporal filtering slider | MEDIUM | LOW | P2 |
| URL-addressable graph state | MEDIUM | LOW | P2 |
| Node clustering / LOD | MEDIUM | HIGH | P3 |
| Analysis provenance (source segments) | MEDIUM | MEDIUM | P3 |
| Section-aware LLM extraction | MEDIUM | MEDIUM | P3 |

**Priority key:**
- P1: Must have for v1.1 launch — core migration and scale promises
- P2: Should have, add when P1 is stable
- P3: Nice to have, target v1.2

---

## Competitor Feature Analysis

This milestone's features are informed by what ResearchRabbit, Connected Papers, and Litmaps do — and where ReSyn intentionally diverges.

| Feature | ResearchRabbit | Connected Papers | Litmaps | ReSyn v1.1 Approach |
|---------|---------------|-----------------|---------|---------------------|
| Graph visualization | Force graph, custom canvas | Force graph, D3 SVG | Timeline + citation graph | WebGL force graph, same data model |
| Node interaction | Click for paper detail | Click for sidebar | Click for detail panel | Click for Leptos sidebar with full analysis |
| Gap / contradiction surfacing | None | None | None | Contradiction edges + bridge badges (differentiator) |
| Open problems aggregation | None | None | None | Panel ranked by recurrence (differentiator) |
| Method matrix | None | None | None | Heatmap of pairings (differentiator) |
| Temporal filtering | Year-axis layout | None | X-axis is publication year | Slider filters node set |
| Progress on large crawls | N/A (uses their DB) | N/A | N/A | SSE stream with paper-by-paper progress |
| Resumable crawl | N/A | N/A | N/A | DB-backed queue with checkpoint |
| Offline/local LLM | No | No | No | Ollama backend (differentiator) |

**Key competitive observation (MEDIUM confidence, source: Effortless Academic 2025 comparison):** None of the three major competitors surface structural research gaps, contradictions, or ABC-bridge connections. They are citation-map tools; ReSyn is an analysis tool that uses citation maps as scaffolding. This distinction should drive all UX decisions — the graph is the context, not the product.

---

## Sources

- [Litmaps vs ResearchRabbit vs Connected Papers 2025 — The Effortless Academic](https://effortlessacademic.com/litmaps-vs-researchrabbit-vs-connected-papers-the-best-literature-review-tool-in-2025/)
- [ResearchRabbit features page](https://www.researchrabbit.ai/features)
- [ResearchRabbit 2025 revamp review — Aaron Tay](https://aarontay.substack.com/p/researchrabbits-2025-revamp-iterative)
- [Leptos official documentation — leptos.dev](https://www.leptos.dev/)
- [leptos_axum integration — docs.rs](https://docs.rs/leptos_axum/latest/leptos_axum/)
- [Leptos 0.8 + Axum starter template — GitHub](https://github.com/leptos-rs/start-axum)
- [Leptos reactive stores (0.7+) — book.leptos.dev](https://book.leptos.dev/15_global_state.html)
- [Leptos SSE / use_event_source — leptos-use.rs](https://leptos-use.rs/network/use_event_source.html)
- [cosmos.gl GPU-accelerated force graph — OpenJS Foundation](https://openjsf.org/blog/introducing-cosmos-gl)
- [Cytoscape.js WebGL renderer preview (Jan 2025)](https://blog.js.cytoscape.org/2025/01/13/webgl-preview/)
- [Graph visualization performance comparison (canvas vs WebGL) — Memgraph blog](https://memgraph.com/blog/you-want-a-fast-easy-to-use-and-popular-graph-visualization-tool)
- [PMC study: graph viz library performance 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12061801/)
- [Speeding up graph layout with Rust + WASM — cprimozic.net](https://cprimozic.net/blog/speeding-up-webcola-with-webassembly/)
- [Leptos + SurrealDB + Axum reference project — GitHub](https://github.com/oxide-byte/rust-berlin-leptos)
- [SurrealDB + Axum official docs](https://surrealdb.com/docs/sdk/rust/frameworks/axum)
- [SSE for long-running task progress — auth0.com](https://auth0.com/blog/developing-real-time-web-applications-with-server-sent-events/)
- [Provenance visualization in analysis tools — arXiv 2505.11784 (2025)](https://arxiv.org/html/2505.11784v1)

---
*Feature research for: ReSyn v1.1 — Leptos web migration, WebGL graph, incremental crawling, enriched gap analysis UI*
*Researched: 2026-03-15*
