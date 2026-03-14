# Phase 1: Text Extraction Foundation - Context

**Gathered:** 2026-03-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Fetch and parse paper full text from ar5iv HTML with structured section extraction. Graceful fallback to abstract-only when HTML unavailable. Add `--analyze` and `--skip-fulltext` CLI flags. Introduce DB migration infrastructure for schema changes. Does NOT include NLP processing, LLM extraction, or cross-paper analysis.

</domain>

<decisions>
## Implementation Decisions

### Extraction Structure
- Extract into structured named sections: abstract, introduction, methods/approach, results/discussion, conclusion
- Each section is a separate field (not a flat text blob) — enables section-aware LLM prompting in later phases
- Metadata per extraction: `extraction_method` (ar5iv_html / abstract_only) + completeness map (which sections were found vs missing)
- All four section categories extracted: abstract, intro+conclusion, methods, results/discussion

### Pipeline Integration
- `--analyze` flag triggers text extraction as a post-crawl step: crawl → persist → **analyze** → visualize
- Extraction runs after crawl, as a second pass over persisted papers — not during crawl
- `--analyze` requires `--db` (persistence needed for caching and avoiding redundant work)
- Analysis runs on already-persisted papers from DB, enabling re-extraction without re-crawling

### Fallback Behavior
- Summary at end of extraction: "12/30 papers used abstract-only (no ar5iv HTML available)" — not per-paper warnings
- Best-effort section extraction: extract whatever sections exist, mark missing ones as None
- `--skip-fulltext` forces abstract-only extraction for all papers (fast mode for testing/debugging)
- Immediate fallback on ar5iv HTTP errors (500, timeout) — no retry, fall back to abstract-only
- Papers flagged as `partial` when using abstract-only continue through the pipeline without error

### Claude's Discretion
- DB schema design: separate table + relation vs embedded fields on paper record
- Rate limiter strategy: share existing ArxivHTMLDownloader rate limiter or new fetcher with same politeness rules
- Section detection approach: CSS selectors for ar5iv HTML heading patterns
- Text cleaning level: handling of LaTeX artifacts, math notation, reference markers
- Migration tool choice: surrealdb-migrations crate vs custom version tracking
- InspireHEP paper extraction: how to handle papers that came from InspireHEP but have arXiv eprints

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ArxivHTMLDownloader` (`src/data_aggregation/html_parser.rs`): Rate-limited HTML fetcher with builder pattern — can be extended or used as template for ar5iv fetcher
- `scraper` crate: Already a dependency, used for CSS selector-based HTML parsing in reference extraction
- `reqwest::Client` shared via `utils::create_http_client()` with 30s timeout
- `Paper.summary` field: Abstract already populated from both arXiv and InspireHEP — available as fallback text

### Established Patterns
- Builder pattern for rate-limited clients: `::new(client).with_rate_limit(duration)`
- `ResynError` enum for typed errors with `?` propagation and `map_err` context
- `tracing` structured logging: `info!` for progress, `warn!` for recoverable failures
- `#[derive(Default, Clone, Serialize, Deserialize)]` on data models
- DB record IDs as `paper:⟨stripped_arxiv_id⟩` via `strip_version_suffix()`

### Integration Points
- `main.rs`: New `--analyze` and `--skip-fulltext` flags on `Cli` struct (clap derive)
- `main.rs`: New pipeline step after DB persist, before visualization
- `src/database/schema.rs`: Schema extension for extraction results (needs migration system)
- `src/database/queries.rs`: New repository methods for extraction result CRUD
- `src/data_aggregation/`: New module for text extraction (parallel to existing source modules)
- `src/lib.rs`: Export new modules for integration test access

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

*Phase: 01-text-extraction-foundation*
*Context gathered: 2026-03-14*
