# Phase 2: NLP Analysis + DB Schema - Context

**Gathered:** 2026-03-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Compute TF-IDF keyword rankings per paper from extracted text, persist results to SurrealDB with corpus-aware caching, and add a DB migration for the new schema. Runs offline with no API calls. Does NOT include LLM-based semantic extraction (Phase 3) or cross-paper gap analysis (Phase 4).

</domain>

<decisions>
## Implementation Decisions

### TF-IDF Input Scope
- Text input is section-weighted with fixed weights: abstract 2x, methods 1.5x, results 1x, intro/conclusion 0.5x
- Weights are hardcoded, not CLI-configurable (can be made configurable later if needed)
- Papers with only abstract (partial extraction) are included at full weight — consistent with Phase 1's "never block on missing data" philosophy
- Text preprocessing uses standard English stop words plus a small hardcoded list of academic boilerplate terms (e.g., "paper", "study", "result", "show", "figure")

### Keyword Output & Ranking
- Top 5 keywords per paper logged at info level with TF-IDF scores (e.g., `Paper 2301.12345: quantum entanglement (0.42), lattice QCD (0.38), ...`)
- Corpus-level summary after per-paper keywords: paper count, avg keywords/paper, top corpus terms with paper counts
- NLP analysis output appears after text extraction summary, matching pipeline order: extract → analyze

### Corpus Boundary
- Corpus = all papers in the database (not just current crawl)
- TF-IDF is recomputed for all papers when corpus changes (IDF is corpus-relative)
- Corpus fingerprint (paper count + hash of arxiv_ids) stored in DB metadata table for change detection
- If corpus unchanged since last analysis, skip recomputation entirely (satisfies INFR-02)

### Vector Storage
- TF-IDF vectors stored as sparse term→score maps (only non-zero terms)
- Top-N keyword rankings stored as a separate ranked list alongside the full sparse vector for fast reads
- Separate `paper_analysis` table (not extending `text_extraction`) — clean separation of raw text vs computed NLP results
- Corpus metadata stored in its own `analysis_metadata` table (reusable for Phase 3 LLM metadata)

### Claude's Discretion
- Exact TF-IDF implementation (log-normalized TF, smooth IDF, etc.)
- N-gram handling (unigrams only vs bigrams for multi-word terms)
- Corpus fingerprint hash algorithm
- Minimum term frequency thresholds for inclusion in sparse vector
- Migration version numbering (continues from Phase 1's version 2)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TextExtractionResult` + `SectionMap` (`src/datamodels/extraction.rs`): Provides structured per-section text that feeds directly into section-weighted TF-IDF
- `ExtractionRepository` (`src/database/queries.rs`): Pattern for DB record types with `SurrealValue` derive, upsert/get/exists/get_all methods
- `migrate_schema()` (`src/database/schema.rs`): Version-guarded migration system — add migration 3 for `paper_analysis` + `analysis_metadata` tables
- `run_analysis()` (`src/main.rs`): Existing analysis pipeline entry point — extend to include NLP step after text extraction
- `strip_version_suffix()` (`src/utils.rs`): Paper ID normalization used across all layers

### Established Patterns
- `SurrealValue` derive macro for DB record types with manual `From` conversions
- `RecordId::new("table", key)` for record addressing
- `UPSERT type::record('table', $id) CONTENT $record` for idempotent writes
- `SCHEMAFULL` tables with flat fields (no nested OBJECTs) — Phase 1 decision
- Builder pattern for configurable components
- `ResynError` enum with `?` propagation

### Integration Points
- `src/database/schema.rs`: Add migration 3 (paper_analysis table) and migration 4 (analysis_metadata table)
- `src/database/queries.rs`: New `AnalysisRepository` following `ExtractionRepository` pattern
- `src/main.rs::run_analysis()`: Add NLP analysis step after text extraction loop
- `src/datamodels/`: New analysis data model module for `PaperAnalysis` struct
- `src/data_processing/` or new `src/nlp/`: TF-IDF computation logic

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 02-nlp-analysis-db-schema*
*Context gathered: 2026-03-14*
