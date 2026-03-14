# Project Research Summary

**Project:** Research Synergy (ReSyn) — Text Analysis + 3D Visualization Milestone
**Domain:** Literature-Based Discovery (LBD) — hybrid NLP + LLM analysis on citation graph
**Researched:** 2026-03-14
**Confidence:** MEDIUM-HIGH

## Executive Summary

ReSyn is adding a text analysis and multidimensional visualization capability on top of an already-shipped citation graph tool. The existing foundation (SurrealDB persistence, arXiv/InspireHEP sources, force-directed 2D graph, petgraph, egui) is stable and does not need revisiting. The new milestone follows a well-established pattern for LBD systems: extract structured per-paper annotations first, then run cross-paper comparison to surface genuine research gaps. The recommended approach pairs lightweight offline NLP (TF-IDF keyword extraction, section boundary detection) with pluggable LLM calls for semantic interpretation, keeping API costs proportional to value and the pipeline usable without an internet connection.

The core architectural decision is that both new capability tracks — text analysis and 3D visualization — are additive extensions that share the existing `Paper` domain model and SurrealDB instance. The trait-based abstraction already proven with `PaperSource` should be replicated for `TextExtractor` and `LlmBackend`. No changes are required to the crawl or BFS layers. The text extraction path must implement a fallback cascade (LaTeX source → ar5iv HTML → abstract-only) because LaTeX availability is not guaranteed across the citation graph; this cascade decision must be made before any analysis logic is written.

The main risks are operational: arXiv rate limiting must be extended to new HTTP fetchers or the analysis phase will be throttled mid-run; SurrealDB schema additions require a migration strategy before any new fields land or existing populated databases become unreliable; and the LLM trait must be designed with capability-aware methods from the start or it will leak provider semantics into call sites. The 3D layout algorithm also needs benchmarking before committing to a library — the existing Fruchterman-Reingold implementation is already O(n²) and adding a third dimension on 200+ nodes will freeze the GUI.

## Key Findings

### Recommended Stack

The new milestone adds targeted libraries on top of the existing stack. For full-text extraction: `flate2` + `tar` for arXiv LaTeX source archives, with the existing `scraper` crate extended for ar5iv HTML, and `pdf-extract` as a last-resort fallback. For NLP: `keyword_extraction` (pure Rust, offline TF-IDF/RAKE/TextRank) and `fastembed` (ONNX-based local sentence embeddings, no Python). For LLM routing: `genai` 0.5, which covers Claude, OpenAI, Ollama, Gemini, and others behind one async API. For 3D visualization: wgpu and glam are already transitive dependencies through eframe; the `egui-wgpu` `CallbackTrait` pattern enables injecting a custom render pass into an existing egui panel without restructuring the application. There is one dependency conflict to resolve: `genai` 0.5 uses `reqwest` 0.13 while the project uses 0.12; Cargo should compile both as different major versions but this must be verified during implementation.

**Core technologies:**
- `flate2` + `tar`: arXiv LaTeX source extraction — pure Rust, zero external dependencies, stable arXiv endpoint
- `scraper` (existing): ar5iv HTML full-text sections — already used for references; CSS class structure confirmed
- `pdf-extract` 0.10: PDF fallback — active development; variable quality on math-heavy PDFs, use only as last resort
- `keyword_extraction` 1.x: offline TF-IDF/RAKE/TextRank — no API cost, works offline, multiple algorithms
- `fastembed` 5.x: local sentence embeddings — ONNX-based, no Python, 384-dim vectors, SurrealDB integration documented
- `genai` 0.5: multi-provider LLM client — single crate for all providers, actively maintained, reqwest version conflict needs verification
- `egui-wgpu` + `wgpu` + `glam`: 3D rendering inside existing egui app — official egui demo (`custom3d_wgpu.rs`) confirms the pattern

### Expected Features

**Must have (table stakes):**
- Abstract-level LLM extraction (methods, findings, open problems) stored as structured JSON — `Paper.summary` already populated; zero new data fetching needed
- Analysis result caching and idempotency — re-runs must not re-bill LLM API for already-analyzed papers
- Pluggable LLM backend — users have different API access; hard-coding one provider is a dealbreaker
- Keyword/TF-IDF extraction — expected by every LBD tool; works offline, no API cost
- Graceful degradation when full text unavailable — ar5iv HTML exists for ~70% of papers; abstract-only fallback must never block the pipeline
- Incremental analysis (new papers only) — corpus grows across sessions; full re-analysis per session is unacceptable

**Should have (competitive differentiators):**
- Cross-paper gap detection (contradictions, unexplored method combinations) — the core LBD value; no mainstream citation tool does this
- Open-problems aggregation across the graph — synthesizes what papers collectively admit they have not solved
- Method-combination gap matrix — which method pairings appear vs which are conspicuously absent
- Graph enrichment with analysis dimensions — color/size nodes by paper type, primary method; makes the existing graph immediately more informative
- Full-text extraction via ar5iv HTML — deeper analysis than abstract-only
- Analysis provenance tracking — stores source segment for each extracted claim, enabling user trust and spot-checking

**Defer (v2+):**
- ABC-model bridge discovery — requires all per-paper analysis data to be meaningful before it produces value
- Temporal evolution view — useful but not required for gap surfacing
- 3D visualization — significant egui/wgpu effort; deliver 2D graph enrichment first

### Architecture Approach

Both new capability tracks (text analysis pipeline, 3D visualization) are additive modules layered on top of the existing `Vec<Paper>` canonical state produced by the crawl. Neither requires changes to `data_aggregation/`, `data_processing/`, or the BFS crawler. The text analysis pipeline introduces four new module directories (`text_extraction/`, `nlp_analysis/`, `llm_backend/`, `gap_analysis/`) and extends `database/` with a `paper_analysis` table, vector index on `tfidf_vector`, and a `gap_finding` table. The 3D visualization introduces `visualization_3d/` (wgpu renderer, orbit camera, scatter app) as a separate window launched via `--view 3d`. The `LlmBackend` trait mirrors the existing `PaperSource` trait pattern exactly; the `TextExtractor` trait follows the same structure. SurrealDB's native HNSW vector index is used for embedding similarity search, keeping everything in a single database.

**Major components:**
1. `text_extraction/` — fetch and parse full paper text (ar5iv HTML, LaTeX source, abstract fallback) via `TextExtractor` trait
2. `nlp_analysis/` — offline mechanical extraction: TF-IDF vectors, section detection, keyword ranking
3. `llm_backend/` — semantic extraction via `LlmBackend` trait (Claude, OpenAI, Ollama implementations)
4. `gap_analysis/` — cross-paper contradiction detection, method gap matrix, PCA projection to 3D coordinates
5. `database/` (extended) — `paper_analysis` table, HNSW vector index, `gap_finding` table, `AnalysisRepository`
6. `visualization_3d/` — wgpu-based 3D scatter plot with orbit camera, gap highlighting, egui sidebar

### Critical Pitfalls

1. **LaTeX source unavailability for older papers** — implement a strict fallback cascade (LaTeX → ar5iv HTML → abstract-only) and store `extraction_method` enum on every `paper_analysis` record before writing any analysis logic; never treat abstract-only extraction as equivalent to full-text
2. **SurrealDB schema drift without migrations** — adopt `surrealdb-migrations` crate before adding any new schema fields; the current auto-init pattern breaks when type constraints are tightened or fields renamed against existing data
3. **LLM abstraction that leaks provider semantics** — design `LlmBackend` with capability-reporting methods (`max_context_tokens`, `supports_structured_output`) and accept a `PromptConfig` struct rather than raw strings; get the trait signature right before implementing two providers
4. **Rate limiting not extended to new text extraction fetchers** — refactor the existing `ArxivHTMLDownloader` rate limiter into a shared `RateLimiter` struct before writing any new HTTP fetcher code; source archive downloads warrant 5s intervals
5. **O(n²) 3D layout freezing the GUI** — benchmark the current 2D Fruchterman-Reingold implementation at 200+ nodes before selecting a 3D layout algorithm; prefer pre-computing layout offline and storing positions in SurrealDB

## Implications for Roadmap

Based on the feature dependency graph from FEATURES.md and the build order from ARCHITECTURE.md, the milestone divides naturally into five phases. Each phase produces a working, testable deliverable. Phases 1-3 can be validated without the full pipeline integrated.

### Phase 1: Text Extraction Foundation

**Rationale:** All analysis depends on having extracted text. The fallback cascade and `extraction_method` field must be defined here or they cannot be retrofitted without touching every downstream consumer. Rate limiting infrastructure must also be refactored in this phase to avoid Pitfall 7 repeating across new fetchers.
**Delivers:** `TextExtractionResult` structs per paper with populated `section_map`, `raw_text`, and `extraction_method`; shared `RateLimiter` struct; extended `error.rs`
**Addresses:** Abstract-level availability, graceful degradation, full-text via ar5iv HTML
**Avoids:** Pitfall 1 (LaTeX unavailability), Pitfall 2 (LaTeX artifacts in prose), Pitfall 7 (rate limiting gap), Pitfall 11 (arXiv HTML structure changes)

### Phase 2: NLP Analysis + DB Schema

**Rationale:** Pure offline processing; fast iteration without API keys or network. Schema migration infrastructure must be established here before LLM fields are added. This phase produces the TF-IDF vectors that the embedding similarity search (Phase 4) depends on.
**Delivers:** Per-paper keyword rankings, TF-IDF vectors, section maps persisted to SurrealDB `paper_analysis` table; migration infrastructure in place
**Uses:** `keyword_extraction` crate, SurrealDB `paper_analysis` schema, `surrealdb-migrations`
**Implements:** `nlp_analysis/` module, `AnalysisRepository::upsert_nlp`, `NlpResult` domain model
**Avoids:** Pitfall 3 (schema drift)

### Phase 3: Pluggable LLM Backend

**Rationale:** LLM calls receive pre-processed structured sections from Phase 2, not raw text. The trait signature must be finalized before two provider implementations are written to avoid Pitfall 4. Caching schema must include `analysis_schema_version` from the start.
**Delivers:** Semantic extraction (methods, findings, open problems, datasets) per paper; LLM result caching with schema versioning; Claude, OpenAI, and Ollama implementations; `NoopBackend` for offline mode
**Uses:** `genai` 0.5, `tiktoken-rs` for token counting, `fastembed` for local embeddings
**Implements:** `llm_backend/` module with capability-aware `LlmBackend` trait
**Avoids:** Pitfall 4 (leaky LLM abstraction), Pitfall 5 (stale cache), Pitfall 10 (context window overflow), Pitfall 12 (parallel calls without backpressure)

### Phase 4: Cross-Paper Gap Analysis

**Rationale:** Requires both NLP vectors (Phase 2) and LLM semantic extractions (Phase 3) to produce meaningful output. Entity normalization must be designed before the gap detection algorithm is implemented or false-positive rates make results unusable. This is the primary differentiating feature.
**Delivers:** `Vec<GapFinding>` with contradiction detection, method gap matrix, open-problems aggregation; HNSW vector index on SurrealDB; PCA-projected 3D coordinates per paper
**Uses:** `fastembed`, `ndarray`, `linfa-reduction` (PCA), SurrealDB vector search
**Implements:** `gap_analysis/` module: `embeddings.rs`, `cross_paper.rs`, `projection.rs`, `gap_finding` DB table
**Avoids:** Pitfall 8 (entity ambiguity)

### Phase 5: 3D Visualization

**Rationale:** Depends on Phase 4 for 3D coordinates; can be developed in parallel against mock data once the `GapFinding` and coordinate output shapes are defined. Must benchmark layout performance before committing to a rendering approach.
**Delivers:** Interactive 3D scatter plot with orbit camera, gap highlighting, egui sidebar with paper metadata on hover; `--view 2d|3d` CLI flag; existing 2D visualization untouched
**Uses:** `wgpu` (existing transitive dep), `egui-wgpu` `CallbackTrait`, `glam`
**Implements:** `visualization_3d/` module
**Avoids:** Pitfall 6 (projection as ground truth — show algorithm/params in UI), Pitfall 9 (O(n²) layout freeze — benchmark and consider pre-computed layout)

### Phase Ordering Rationale

- Phase 1 before everything because `extraction_method` is a discriminator used by every downstream layer; retrofitting it is expensive.
- Phase 2 before Phase 3 because the LLM receives structured sections, not raw text — this is architecturally enforced.
- Phase 3 before Phase 4 because gap analysis compares `findings` and `methods` fields that only exist after LLM extraction.
- Phase 5 can be parallelized with Phase 4 once output data shapes are agreed, but depends on Phase 4 for real data.
- The `surrealdb-migrations` infrastructure lands in Phase 2 so that Phase 3 and Phase 4 schema additions are handled safely.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 3 (LLM Backend):** `genai` 0.5 reqwest version conflict needs hands-on resolution; structured output enforcement varies significantly by provider and model version; verify before designing the `PromptConfig` schema
- **Phase 4 (Gap Analysis):** Entity normalization strategy for the physics/HEP domain is non-trivial; InspireHEP keyword taxonomy as normalization target needs feasibility validation; SurrealDB HNSW index performance at 200+ papers needs benchmarking
- **Phase 5 (3D Visualization):** 3D layout algorithm selection requires benchmarking the existing 2D Fruchterman-Reingold at scale; `linfa-reduction` PCA at corpus size needs verification

Phases with standard patterns (skip research-phase):
- **Phase 1 (Text Extraction):** ar5iv HTML structure and arXiv LaTeX endpoint are well-documented; `flate2` + `tar` pattern is standard; `scraper` already in use
- **Phase 2 (NLP Analysis):** TF-IDF and section detection are established; `keyword_extraction` crate API is straightforward; SurrealDB schema patterns follow existing code

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Core library choices are well-documented with official sources; one unresolved dependency conflict (genai + reqwest versions) drops from HIGH to MEDIUM-HIGH overall |
| Features | HIGH | Feature set grounded in recent LBD literature (2025); dependency graph is explicit and logical; table stakes vs. differentiators are clearly separated |
| Architecture | MEDIUM-HIGH | Module structure is clear and consistent with existing codebase patterns; 3D architecture relies on egui-wgpu CallbackTrait pattern confirmed by official demo; UMAP/linfa-reduction maturity at LOW confidence |
| Pitfalls | HIGH | Pitfalls are specific, grounded in cited sources, and actionable; each has a concrete prevention strategy tied to a specific phase |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **`genai` reqwest version conflict:** Must verify Cargo compiles both 0.12 and 0.13 side-by-side or determine migration cost. Unblock during Phase 3 setup before writing provider implementations.
- **`keyword_extraction` API validation:** Community adoption is modest; validate the actual API against the feature flag interface before committing the NLP analysis module design.
- **SurrealDB HNSW vector index at corpus scale:** Official docs confirm the feature; real-world performance at 200-paper corpus with 384-dim or 512-dim vectors needs a quick benchmark before designing the similarity search queries in Phase 4.
- **ar5iv HTML coverage for physics papers:** The ~70% figure is domain-averaged; HEP-specific coverage (older papers, conference proceedings) may be lower. Add `extraction_method` telemetry in Phase 1 and evaluate before designing Phase 3 prompts that depend on full-text sections.
- **InspireHEP keyword taxonomy as normalization target:** Feasibility for entity normalization in Phase 4 needs validation; if the taxonomy is accessible via API it simplifies Pitfall 8 significantly.

## Sources

### Primary (HIGH confidence)
- [ar5iv HTML5 service](https://ar5iv.labs.arxiv.org/) — full-text structure, section CSS classes confirmed
- [arXiv HTML accessibility initiative](https://arxiv.org/html/2402.08954v1) — arXiv HTML rendering pipeline via LaTeXML
- [arXiv LaTeX source e-print download](https://info.arxiv.org/help/view.html) — LaTeX tar.gz endpoint documented
- [SurrealDB vector embeddings + HNSW docs](https://surrealdb.com/docs/surrealdb/models/vector) — native vector index confirmed
- [egui custom3d_wgpu demo](https://github.com/emilk/egui/blob/main/crates/egui_demo_app/src/apps/custom3d_wgpu.rs) — CallbackTrait pattern confirmed
- [fastembed-rs GitHub](https://github.com/Anush008/fastembed-rs) — v5.12.0, ONNX-based, no Python
- [rust-genai GitHub](https://github.com/jeremychone/rust-genai) — v0.5, multi-provider confirmed

### Secondary (MEDIUM confidence)
- [A Hybrid Approach to LBD — MDPI 2025](https://www.mdpi.com/2076-3417/15/16/8785) — NLP + LLM pipeline patterns
- [Leveraging LLMs for Enhancing LBD — MDPI 2025](https://www.mdpi.com/2504-2289/8/11/146) — entity ambiguity and extraction challenges
- [Structured information extraction from scientific text — Nature Communications 2024](https://www.nature.com/articles/s41467-024-45563-x) — LLM schema design for academic extraction
- [surrealdb-migrations crate](https://github.com/Odonno/surrealdb-migrations) — migration tooling exists and is Rust-native
- [keyword_extraction — lib.rs](https://lib.rs/crates/keyword_extraction) — API confirmed, community adoption modest
- [Stop Misusing t-SNE and UMAP (2025)](https://arxiv.org/html/2506.08725v2) — projection distortion pitfalls

### Tertiary (LOW confidence)
- [fast-umap Rust UMAP](https://github.com/eugenehp/fast-umap) — depends on burn ML framework; not recommended as primary option
- [linfa-reduction PCA](https://crates.io/crates/linfa-reduction) — confirmed on crates.io; real-world performance at corpus scale unverified

---
*Research completed: 2026-03-14*
*Ready for roadmap: yes*
