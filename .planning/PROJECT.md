# Research Synergy (ReSyn)

## What This Is

A Rust-powered Literature Based Discovery tool that aggregates academic papers from arXiv and InspireHEP, builds citation graphs, extracts structured insights via NLP and LLM analysis, surfaces cross-paper research gaps (contradictions and hidden connections), and visualizes enriched citation networks as interactive force-directed graphs.

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
- ✓ CLI flags for analysis pipeline control (--analyze, --skip-fulltext, --llm-provider, --llm-model, --full-corpus, --verbose) — v1.0
- ✓ Offline TF-IDF keyword extraction with corpus fingerprint caching — v1.0
- ✓ Analysis results cached in SurrealDB per paper — v1.0
- ✓ Pluggable LLM backend via trait (Claude, Ollama, Noop) — v1.0
- ✓ Structured semantic annotations (methods, findings, open problems) via LLM — v1.0
- ✓ Cross-paper contradiction detection — v1.0
- ✓ ABC-bridge discovery (hidden connections via shared intermediaries) — v1.0
- ✓ Citation graph nodes colored/sized by analysis dimensions — v1.0
- ✓ Toggle between raw citation view and enriched view — v1.0

### Active

- [ ] Wire gap findings into graph visualization (edges/badges for contradictions and bridges)
- [ ] Section-aware LLM extraction using detected section boundaries
- [ ] Incremental/resumable crawling with DB-backed crawl queue for high-depth runs
- [ ] Leptos web migration replacing egui desktop GUI
- [ ] WebGL/Canvas graph renderer with Barnes-Hut O(n log n) force layout
- [ ] Analysis provenance tracking (click a finding, see source text segment)
- [ ] Open-problems aggregation panel ranked by recurrence frequency
- [ ] Method-combination gap matrix showing existing vs absent method pairings
- [ ] Temporal filtering by publication year
- [ ] Node clustering / level-of-detail for 1000+ node graphs

### Out of Scope

- Real-time collaborative analysis — single-user tool for now
- Citation prediction / paper recommendation — focus is on gap surfacing, not suggesting new papers
- Full-text indexing / search engine — analysis is structured extraction, not free-text search
- Non-arXiv PDF sources — only papers reachable through existing data sources
- Fine-tuning custom models — use off-the-shelf LLM APIs with prompt engineering
- LaTeX source parsing — ar5iv HTML is simpler and sufficient; LaTeX parsing in Rust is high complexity for marginal gain
- 3D paper embedding projections (PCA/UMAP) — deferred to v1.2+ after web platform stabilizes

## Current Milestone: v1.1 Scale & Surface

**Goal:** Make ReSyn usable at real research scale (depth 10+) and move gap insights from stdout into the primary interface, migrating to a Leptos web UI.

**Target features:**
- Tech debt cleanup (nlp export, stale stubs, gap findings wiring)
- Section-aware LLM extraction
- Incremental/resumable high-depth crawling with progress reporting
- Full Leptos web migration replacing egui
- Enriched web visualization (gap findings, provenance, open-problems panel, method matrix)
- Scale testing and UX polish at 1000+ nodes

## Context

ReSyn is a brownfield Rust project with 8,749 LOC across ~40 source files in 10 modules. The full pipeline (crawl → extract text → NLP keywords → LLM annotations → gap analysis → enriched visualization) is operational with 153 tests.

The v1.0 milestone delivered the complete analysis pipeline on top of the existing citation graph foundation. Key architectural patterns established:
- **Data source trait** (`PaperSource`) for pluggable paper sources
- **LLM provider trait** (`LlmProvider`) for pluggable semantic extraction backends
- **DB migration system** with versioned schema changes (6 migrations)
- **Corpus fingerprint caching** to skip redundant NLP and gap analysis recomputation
- **One-frame-lag enrichment** pattern for visualization overlay without restructuring the render loop

Tech stack: Rust (edition 2024), SurrealDB v3 (embedded), petgraph, egui/eframe, reqwest, tokio.

## Constraints

- **Language**: Rust — maintain consistency with existing codebase
- **Database**: SurrealDB — extend existing schema rather than introducing new storage
- **API costs**: LLM calls batched/cached; re-runs skip already-analyzed papers
- **Rate limits**: Respect arXiv (3s) and InspireHEP (350ms) rate limits
- **Offline capability**: NLP extraction works fully offline; LLM analysis requires API access

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Hybrid NLP + LLM analysis | NLP for structure/cost efficiency, LLM for semantic depth | ✓ Good — clear separation of concerns |
| Pluggable LLM backend via trait | Same pattern as PaperSource; future-proofs provider choice | ✓ Good — Claude, Ollama, Noop all work |
| SurrealDB for analysis storage | Extend existing schema; graph queries natural for cross-paper analysis | ✓ Good — migrations work, 6 versioned |
| ar5iv HTML as primary full-text source | Best structure preservation, already partially scraped | ✓ Good — section detection works |
| DB migration system (version guards) | Idempotent, re-runnable, no data loss | ✓ Good — replaced init_schema cleanly |
| Corpus fingerprint caching | Avoid redundant NLP recomputation on unchanged corpus | ✓ Good — independent fingerprints for NLP and gap analysis |
| TintedEdgeShape custom DisplayEdge | Edge::set_color() absent in egui_graphs 0.25.0 | ✓ Good — required workaround, clean implementation |
| GapFinding uses CREATE not UPSERT | History preservation: multiple runs create separate records | ✓ Good — enables temporal gap tracking |
| SurrealDB FLEXIBLE TYPE for complex fields | JSON strings for methods/findings/tfidf_vector | ⚠ Revisit — works but limits server-side querying |

---
*Last updated: 2026-03-15 after v1.1 milestone start*
