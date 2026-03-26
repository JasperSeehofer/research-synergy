# Milestones

## v1.2 Graph Rendering Overhaul (Shipped: 2026-03-26)

**Phases completed:** 3 phases (15-17), 6 plans
**Stats:** 50 commits, 60 files changed, +9,036/-334 lines
**Timeline:** 2 days (2026-03-25 → 2026-03-26)

**Delivered:** Overhauled the citation graph renderer with retuned force simulation, visible edges, crisp nodes, seed node distinction, auto-fit viewport, and collision-free labels.

**Key accomplishments:**

- Force coefficients retuned 5x stronger with collision separation and BFS ring placement so citation clusters visibly spread instead of collapsing to a blob
- Canvas 2D and WebGL2 edges rendered with #8b949e color and depth-based alpha, WebGL2 upgraded to quad-based geometry replacing 1px GL.LINES
- Node circles crisp at all zoom levels (fwidth AA in WebGL2, viewport-compensated borders in Canvas 2D) with amber seed node distinction
- Auto-fit viewport with lerp animation after force layout converges, user interaction latch prevents re-trigger
- Priority-ordered label collision avoidance with pill/badge rendering (seed first, then citation count) and three-state convergence badge

---

## v1.1.1 Bug Fix & Polish (Shipped: 2026-03-24)

**Phases completed:** 4 phases, 4 plans, 5 tasks

**Key accomplishments:**

- Axum ServeDir fallback to index.html via ServeFile, enabling client-side Leptos Router for all routes on direct navigation and refresh
- Reduced initial node spread from 968px to 290px and eliminated per-frame GPU VBO leak, with DPR convention documented for Phase 13
- CSS pointer-events passthrough on overlay containers unblocks node drag (INTERACT-01), viewport pan (INTERACT-02), and scroll zoom (INTERACT-03) — four-line CSS change, no Rust modifications
- Dual-range temporal slider fixed with pointer-events passthrough, transparent track backgrounds, and value clamping via get_untracked()

---

## v1.1 Scale & Surface (Shipped: 2026-03-22)

**Phases completed:** 5 phases (6-10), 23 plans, 36 tasks
**Stats:** 100 commits, 177 files changed, +27,467/-4,549 lines, 15,859 total Rust LOC across 90 files
**Timeline:** 4 days (2026-03-15 → 2026-03-18)

**Delivered:** Migrated ReSyn from a CLI/egui desktop app to a Leptos CSR web application with full Rust/WASM graph rendering, DB-backed resumable crawling, and interactive analysis panels.

**Key accomplishments:**

- 3-crate Cargo workspace (resyn-core/app/server) with SurrealDB feature-gated behind `ssr` and WASM compilation boundary
- DB-backed resumable crawl queue with parallel workers, SSE progress reporting, and CLI management subcommands
- Leptos CSR web UI with dashboard, papers table, gap findings panel, open problems, method heatmap, and crawl launcher
- Full Rust/WASM graph renderer — Canvas 2D with auto-upgrade to WebGL2, Barnes-Hut force layout in Web Worker
- Analysis provenance tracking (click finding → see source text with snippet highlighting) via tabbed drawer
- LOD progressive-reveal and temporal year-range filtering for 1000+ node graph scale

**Known gaps:**

- SCALE-01: Real depth 2/3/5 test runs with performance profiling not executed (UI infrastructure complete but profiling deferred)

---

## v1.0 Analysis Pipeline (Shipped: 2026-03-14)

**Phases completed:** 5 phases, 12 plans, 6 tasks

**Key accomplishments:**

- Full text extraction from ar5iv HTML with section detection and abstract-only graceful degradation
- Offline TF-IDF keyword extraction with corpus fingerprint caching and section-weighted scoring
- Pluggable LLM backend (Claude, Ollama, Noop) with per-paper caching via SurrealDB
- Cross-paper contradiction detection and ABC-bridge discovery with LLM-verified justifications
- Enriched citation graph visualization with paper-type coloring, finding-strength sizing, edge tinting, and hover tooltips

**Stats:** 22 feat commits, 32 files modified, 5,528 lines added, 8,749 total Rust LOC
**Git range:** `c4a6e69..HEAD` (feat(01-01) → feat(05-02))

**Known tech debt:**

- `nlp` module not exported in `lib.rs` (only accessible from binary)
- Phase 4 SUMMARY frontmatter missing `requirements_completed` for GAPS-01/GAPS-02
- Gap findings not wired into visualization layer (stdout only)
- ROADMAP plan checkboxes stale for phases 2-5
- Stale stub comment in `src/llm/ollama.rs:2`

---
