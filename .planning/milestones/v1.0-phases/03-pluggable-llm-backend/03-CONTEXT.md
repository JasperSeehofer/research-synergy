# Phase 3: Pluggable LLM Backend - Context

**Gathered:** 2026-03-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Each paper receives structured semantic annotations (methods, findings, open problems) extracted by an LLM. The backend is swappable via `--llm-provider` CLI flag (claude, ollama, noop). Results are cached so re-runs never re-bill API costs for already-analyzed papers. Does NOT include cross-paper gap analysis (Phase 4) or visualization enrichment (Phase 5).

</domain>

<decisions>
## Implementation Decisions

### Annotation Schema
- Findings represented as structured objects with text + strength indicator (e.g., `{text: "...", strength: "strong_evidence"}`) — enables Phase 4 contradiction detection to weigh conflicting claims
- Paper type extracted as a fixed enum classification (e.g., experimental, theoretical, review, computational) — feeds VIS-01 color-by-type
- Methods granularity: Claude's discretion based on Phase 4 gap analysis needs (likely structured with category tags for cross-paper comparison)
- Open problems: Claude's discretion on format
- Results stored in a new `llm_annotation` table — separate from `paper_analysis` (TF-IDF), consistent with Phase 2 decision of separate tables per analysis type

### Provider Design
- API keys via environment variables only (ANTHROPIC_API_KEY, OLLAMA_URL, etc.) — no config files, no secrets in CLI flags
- `--llm-model` CLI flag for model selection within a provider (e.g., `--llm-provider claude --llm-model claude-sonnet-4-20250514`), each provider has a sensible default
- Ollama defaults to `http://localhost:11434`, overridable via `OLLAMA_URL` env var
- HTTP approach (direct reqwest vs SDK crate): Claude's discretion — note STATE.md blocker about genai 0.5 / reqwest 0.13 conflict
- LLM trait mirrors `PaperSource` pattern: `#[async_trait]`, `Send + Sync` (carried forward from PROJECT.md decision)

### Noop Provider
- Produces empty collections + noop marker: `methods: [], findings: [], open_problems: [], paper_type: "unknown", provider: "noop"`
- Persists to DB — tests the full pipeline end-to-end including DB writes
- Runs instantly with no simulated delay
- Logs the constructed prompt at debug level — useful for prompt engineering without API costs

### Error & Retry Policy
- Single paper LLM failure (network, rate limit): skip and continue — matches Phase 1's "never block on missing data" philosophy
- Parse failure (malformed LLM response): retry once with stricter "respond ONLY with valid JSON" nudge, log raw response at debug level for offline prompt improvement, then skip
- Rate limiting: respect provider-specific rate limits with configurable delays baked into each provider (no CLI flag needed)
- End-of-run summary: "LLM analysis: 28/30 papers annotated (2 skipped), provider: claude, model: claude-sonnet-4-20250514" — matches NLP analysis summary style

### Claude's Discretion
- Methods field granularity and structure (likely structured with category for Phase 4 compatibility)
- Open problems format
- HTTP client approach (direct reqwest vs SDK crate) — consider reqwest version compatibility
- Exact prompt template design for annotation extraction
- Migration version numbering (continues from Phase 2's version 4)
- Caching strategy details (likely per-paper check like Phase 2's corpus fingerprint pattern, but per-paper since LLM calls are expensive)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `PaperSource` trait (`src/data_aggregation/traits.rs`): Pattern to mirror — 3 methods, `#[async_trait]`, `Send + Sync`
- `AnalysisRepository` (`src/database/queries.rs`): Pattern for the new `LlmAnnotationRepository` — upsert/get/exists/get_all
- `PaperAnalysis` struct (`src/datamodels/analysis.rs`): Template for `LlmAnnotation` struct design
- `analysis_metadata` table: Already exists, reusable for LLM-specific metadata (corpus fingerprint per provider)
- `run_nlp_analysis()` (`src/main.rs`): Shows where LLM analysis step slots in — after NLP, before visualization
- `ResynError` enum (`src/error.rs`): Add `LlmApi` variant for LLM-specific errors
- `reqwest` 0.12: Already a dependency — direct API calls avoid adding new HTTP client deps

### Established Patterns
- `#[async_trait]` for async trait methods
- Builder pattern for rate-limited clients: `::new(client).with_rate_limit(duration)`
- `SurrealValue` derive for DB record types with manual `From` conversions
- `migrate_schema()` version guards: `if version < N` for each migration
- `tracing` for structured logging: `info!` progress, `warn!` recoverable, `error!` fatal, `debug!` verbose
- Flat SCHEMAFULL tables (no nested OBJECTs) — but may need FLEXIBLE for LLM response fields

### Integration Points
- `src/main.rs`: Add `--llm-provider` and `--llm-model` flags to `Cli` struct (clap derive)
- `src/main.rs`: New `run_llm_analysis()` after `run_nlp_analysis()` in pipeline
- `src/database/schema.rs`: Migration 5 for `llm_annotation` table
- `src/database/queries.rs`: New `LlmAnnotationRepository`
- `src/datamodels/`: New `llm_annotation.rs` module
- New `src/llm/` module: trait definition, claude provider, ollama provider, noop provider
- `src/error.rs`: New `LlmApi` error variant

</code_context>

<specifics>
## Specific Ideas

- Parse failure logging: raw LLM response logged at debug level to enable offline prompt improvement
- Noop provider as a development/testing tool: debug-level prompt logging lets users inspect what would be sent without API costs

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 03-pluggable-llm-backend*
*Context gathered: 2026-03-14*
