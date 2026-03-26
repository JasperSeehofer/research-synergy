# Research Synergy (ReSyn)

## What This Is

A Rust-powered Literature Based Discovery tool that aggregates academic papers from arXiv and InspireHEP, builds citation graphs, extracts structured insights via NLP and LLM analysis, surfaces cross-paper research gaps, and presents everything through an interactive Leptos web UI with full Rust/WASM graph rendering.

## Core Value

Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph.

## Requirements

### Validated

- ✓ Fetch paper metadata from arXiv API — existing
- ✓ Fetch paper metadata from InspireHEP API — existing
- ✓ BFS crawl citation references to configurable depth — existing
- ✓ Parse arXiv HTML bibliography for reference extraction — existing
- ✓ Persist papers and citation edges to SurrealDB — existing
- ✓ Load citation graph from database (db-only mode) — existing
- ✓ Build directed citation graph with petgraph — existing
- ✓ Interactive force-directed graph visualization with pan/zoom — existing
- ✓ Paper ID validation (new and old arXiv formats) — existing
- ✓ Rate limiting for arXiv (3s) and InspireHEP (350ms) — existing
- ✓ Pluggable data source architecture via PaperSource trait — existing
- ✓ Version suffix deduplication across all layers — existing
- ✓ Extract full text from arXiv HTML (ar5iv) with section detection — v1.0
- ✓ Graceful abstract-only fallback when full text unavailable — v1.0
- ✓ DB migration system for safe schema evolution — v1.0
- ✓ CLI flags for analysis pipeline control — v1.0
- ✓ Offline TF-IDF keyword extraction with corpus fingerprint caching — v1.0
- ✓ Analysis results cached in SurrealDB per paper — v1.0
- ✓ Pluggable LLM backend via trait (Claude, Ollama, Noop) — v1.0
- ✓ Structured semantic annotations (methods, findings, open problems) via LLM — v1.0
- ✓ Cross-paper contradiction detection — v1.0
- ✓ ABC-bridge discovery (hidden connections via shared intermediaries) — v1.0
- ✓ Citation graph nodes colored/sized by analysis dimensions — v1.0
- ✓ Toggle between raw citation view and enriched view — v1.0
- ✓ 3-crate Cargo workspace with WASM compilation boundary — v1.1
- ✓ DB-backed resumable crawl queue with parallel workers — v1.1
- ✓ SSE crawl progress reporting — v1.1
- ✓ Leptos CSR web UI with Trunk build pipeline — v1.1
- ✓ Axum server functions exposing analysis pipeline to frontend — v1.1
- ✓ Gap findings rendered in graph (contradiction edges, bridge badges) — v1.1
- ✓ Open-problems aggregation panel ranked by recurrence frequency — v1.1
- ✓ Method-combination gap matrix showing existing vs absent pairings — v1.1
- ✓ Full Rust/WASM Canvas 2D + WebGL2 graph renderer — v1.1
- ✓ Barnes-Hut O(n log n) force layout in Web Worker — v1.1
- ✓ Analysis provenance tracking (click finding → see source text) — v1.1
- ✓ Section-aware LLM extraction using detected section boundaries — v1.1
- ✓ LOD progressive-reveal for 1000+ node graphs — v1.1
- ✓ Temporal filtering by publication year — v1.1

- ✓ SPA fallback routing — all client-side routes work on direct navigation and refresh — v1.1.1
- ✓ Force-directed graph animation with visible node spreading — v1.1.1
- ✓ Graph edges rendered crisply between connected nodes — v1.1.1
- ✓ Node drag, viewport pan, scroll zoom all functional — v1.1.1
- ✓ Dual-range temporal slider thumbs independently draggable — v1.1.1
- ✓ Force coefficients retuned for visible citation cluster spreading — v1.2 Phase 15
- ✓ BFS concentric ring placement for simulation warm start — v1.2 Phase 15
- ✓ Alpha full-stop convergence halts CPU when simulation settles — v1.2 Phase 15
- ✓ Citation edges visible on dark background (#8b949e with depth-based alpha) — v1.2 Phase 16
- ✓ Node circles crisp at all zoom levels (fwidth AA in WebGL2, viewport-compensated Canvas 2D) — v1.2 Phase 16
- ✓ Seed paper visually distinct (amber fill, bright border, planetary ring) — v1.2 Phase 16
- ✓ WebGL2 quad-based edge geometry replacing GL.LINES — v1.2 Phase 16
- ✓ Auto-fit viewport after force layout converges with lerp animation — v1.2 Phase 17
- ✓ User interaction latch prevents auto-fit re-trigger after pan/zoom — v1.2 Phase 17
- ✓ Priority-ordered label collision avoidance (seed first, then citation count) — v1.2 Phase 17
- ✓ Three-state convergence badge (Simulating/Paused/Settled) — v1.2 Phase 17

### Active

(No active milestone — run `/gsd:new-milestone` to define next)

### Out of Scope

- Real-time collaborative analysis — single-user tool for now
- Citation prediction / paper recommendation — focus is on gap surfacing, not suggesting new papers
- Full-text indexing / search engine — analysis is structured extraction, not free-text search
- Non-arXiv PDF sources — only papers reachable through existing data sources
- Fine-tuning custom models — use off-the-shelf LLM APIs with prompt engineering
- LaTeX source parsing — ar5iv HTML is simpler and sufficient
- SSR / server-side rendering — CSR-only, single-user local tool
- JavaScript graph libraries (sigma.js, d3) — full Rust/WASM stack preferred
- Multi-user collaboration — single-user research tool

## Current State

**Shipped:** v1.2 Graph Rendering Overhaul (2026-03-26)

ReSyn is a 3-crate Cargo workspace (resyn-core/resyn-app/resyn-server) with ~25,000 LOC Rust across 90+ files. The full pipeline runs through a Leptos CSR web UI served by Axum, with interactive Canvas 2D / WebGL2 graph rendering powered by Barnes-Hut force layout in a WASM Web Worker. The graph renderer now produces visually clear force-directed layouts with retuned coefficients, visible edges on dark backgrounds, crisp anti-aliased nodes, seed node distinction, auto-fit viewport animation, and collision-free priority-ordered labels.

**Stack:** Rust (edition 2024), Leptos 0.8 (CSR), Trunk, Axum, SurrealDB v3 (embedded), petgraph, web-sys (Canvas 2D + WebGL2), gloo-worker, reqwest, tokio.

## Constraints

- **Language**: Rust — maintain consistency with existing codebase
- **Rendering**: Full Rust/WASM — no JavaScript graph libraries
- **Database**: SurrealDB — extend existing schema rather than introducing new storage
- **API costs**: LLM calls batched/cached; re-runs skip already-analyzed papers
- **Rate limits**: Respect arXiv (3s) and InspireHEP (350ms) rate limits
- **Offline capability**: NLP extraction works fully offline; LLM analysis requires API access

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Hybrid NLP + LLM analysis | NLP for structure/cost efficiency, LLM for semantic depth | ✓ Good — clear separation of concerns |
| Pluggable backend traits (PaperSource, LlmProvider) | Future-proofs provider choice | ✓ Good — extensible pattern |
| SurrealDB for all storage | Extend existing schema; graph queries natural for cross-paper analysis | ✓ Good — 6+ migrations, crawl queue, analysis cache |
| ar5iv HTML as primary full-text source | Best structure preservation | ✓ Good — section detection works |
| DB migration system (version guards) | Idempotent, re-runnable, no data loss | ✓ Good — replaced init_schema cleanly |
| GapFinding uses CREATE not UPSERT | History preservation: multiple runs create separate records | ✓ Good — enables temporal gap tracking |
| Full Rust/WASM graph stack | No JS deps, single language, compile-time safety | ✓ Good — Canvas 2D + WebGL2 both work |
| CSR-only (Trunk, not cargo-leptos) | Single-user local tool, no SSR complexity | ✓ Good — simpler build, no hydration issues |
| Barnes-Hut in Web Worker | O(n log n) layout off main thread | ✓ Good — interactive at 1000+ nodes |
| SurrealDB feature-gated behind `ssr` | WASM compilation boundary | ✓ Good — clean separation |
| Named record IDs for crawl queue | Idempotent enqueue (CREATE on same ID is no-op) | ✓ Good — natural dedup |
| SurrealDB FLEXIBLE TYPE for complex fields | JSON strings for methods/findings/tfidf_vector | ⚠ Revisit — works but limits server-side querying |
| ServeFile SPA fallback | Single-line change, no new deps | ✓ Good — all client routes resolve |
| CSS pointer-events overlay passthrough | No Rust interaction logic changes needed | ✓ Good — clean separation of concerns |
| DPR convention: CSS pixels throughout | DPR only at canvas physical sizing and GL viewport | ✓ Good — documented for future phases |
| Dual-range slider: pointer-events:none on track | Canonical MDN pattern for stacked range inputs | ✓ Good — both thumbs independently draggable |
| REPULSION_STRENGTH = -1500 (5x stronger) | vis.js uses -2000; prevents hub node collapse | ✓ Good — visible cluster separation |
| BFS ring placement with 180px spacing | 1.5x IDEAL_DISTANCE; nodes start beyond equilibrium | ✓ Good — organized warm start |
| Alpha full-stop convergence | Halt CPU simulation when settled | ✓ Good — zero idle CPU usage |
| depth_alpha using max BFS depth of endpoints | Progressive hierarchy dimming | ✓ Good — consistent Canvas 2D and WebGL2 |
| Quad-based WebGL2 edges | Replace 1px-capped GL.LINES | ✓ Good — visible edges at all zoom levels |
| fwidth AA for WebGL2 nodes | Resolution-independent anti-aliasing | ✓ Good — crisp circles at all sizes |
| Seed outer ring reuses edge shader program | Triangle annulus, no new VAO/shader | ✓ Good — minimal GPU overhead |
| Viewport fit margin_factor=0.80 with lerp t=0.12 | 10% margin each side, ~0.5s ease-out | ✓ Good — smooth animation |
| User interaction latch for auto-fit | Permanent latch on pan/wheel/zoom | ✓ Good — respects user viewport |
| Offscreen canvas for measureText cache | Load-time text width measurement | ✓ Good — works for both renderers |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

Last updated: 2026-03-26

---
*Last updated: 2026-03-26 after v1.2 milestone*
