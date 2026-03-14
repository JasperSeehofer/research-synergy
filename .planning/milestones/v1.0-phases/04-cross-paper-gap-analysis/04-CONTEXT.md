# Phase 4: Cross-Paper Gap Analysis - Context

**Gathered:** 2026-03-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Surface contradictions between papers and hidden ABC-bridge connections across the citation graph. Gap findings are stored as structured records in SurrealDB and printed to stdout. Does NOT include visualization enrichment (Phase 5), method-combination gap matrices (v2), or open-problem aggregation (v2).

</domain>

<decisions>
## Implementation Decisions

### Contradiction Detection
- Two-stage pipeline: TF-IDF keyword overlap identifies same-topic paper pairs, then finding strength divergence (from Phase 3 `Finding.strength`) narrows candidates, then LLM confirms actual contradictions
- TF-IDF overlap threshold: Claude's discretion — pick a sensible default, hardcoded but easy to tune later
- LLM verification reuses existing `LlmProvider` trait and `--llm-provider` CLI flag — same provider for both annotation and gap analysis, different prompt template
- Matches Phase 1's "never block on missing data" philosophy — if LLM verification fails for a pair, skip and continue

### ABC-Bridge Discovery
- B intermediary = shared high-weight TF-IDF keywords between papers A and C where A and C don't directly cite each other
- Default scope: papers within the citation graph (connected by some path)
- `--full-corpus` CLI flag expands scope to all papers in SurrealDB, including disconnected papers from separate crawls
- LLM verification generates a human-readable justification explaining the A↔C connection via B (satisfies success criterion #4)
- Minimum graph distance for "non-obvious": Claude's discretion

### Gap Output Format
- Table format grouped by type (Contradictions section, then ABC Bridges section)
- Columns: Type | Papers | Shared Terms | Justification
- Justification truncated to ~60 chars in table; `--verbose` flag shows full justifications below the table
- Summary count line after table: "Gap analysis: N contradictions, M ABC-bridges found across P papers"
- Stdout + DB persistence only — no file export in v1

### Gap Persistence
- Separate `gap_finding` SCHEMAFULL table (follows Phase 2/3 pattern of separate tables per analysis type)
- Fields: type (contradiction/abc_bridge), paper_ids, shared_terms, justification, confidence, found_at
- Corpus fingerprint caching — skip gap analysis if corpus unchanged since last run
- History preserved with timestamps — old findings kept when corpus changes and new findings are added, not deleted

### Claude's Discretion
- TF-IDF similarity threshold for same-topic detection
- Minimum graph distance for ABC-bridge "non-obvious" qualification
- LLM prompt templates for contradiction verification and ABC-bridge justification
- Confidence scoring approach for gap findings
- Migration version numbering (continues from Phase 3's version 5)
- Table rendering implementation (manual formatting vs crate)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `PaperAnalysis.tfidf_vector` (`src/datamodels/analysis.rs`): Sparse term→score maps for TF-IDF overlap computation between paper pairs
- `LlmAnnotation.findings` (`src/datamodels/llm_annotation.rs`): `Finding { text, strength }` — strength divergence drives contradiction detection
- `LlmAnnotation.methods` (`src/datamodels/llm_annotation.rs`): `Method { name, category }` — available for future method-pairing gaps (v2)
- `LlmProvider` trait (`src/llm/traits.rs`): Reuse for contradiction verification and ABC-bridge justification LLM calls
- `create_graph_from_papers()` (`src/data_processing/graph_creation.rs`): petgraph `StableGraph` with citation edges — use for graph distance computation
- `AnalysisRepository` / `LlmAnnotationRepository` (`src/database/queries.rs`): Pattern for new `GapFindingRepository`
- `migrate_schema()` (`src/database/schema.rs`): Version-guarded migration system — add migration for `gap_finding` table
- `run_analysis()` (`src/main.rs`): Pipeline entry point — gap analysis slots in after LLM annotation step

### Established Patterns
- Separate SCHEMAFULL tables per analysis type (text_extraction, paper_analysis, llm_annotation → gap_finding)
- `SurrealValue` derive for DB record types with manual `From` conversions
- Version-guarded migrations: `if version < N` for each migration
- Builder pattern for rate-limited clients
- Flat fields (no nested OBJECTs) in SCHEMAFULL tables — store complex data as JSON strings
- Pipeline summary style: one-line count at end of each analysis stage

### Integration Points
- `src/main.rs`: Add `--full-corpus` and `--verbose` flags to `Cli` struct
- `src/main.rs`: New `run_gap_analysis()` after `run_llm_analysis()` in pipeline
- `src/database/schema.rs`: Migration for `gap_finding` table
- `src/database/queries.rs`: New `GapFindingRepository`
- `src/datamodels/`: New `gap_finding.rs` module
- New `src/gap_analysis/` module: contradiction detector, ABC-bridge discoverer, gap output formatter

</code_context>

<specifics>
## Specific Ideas

- Two-stage detection is cost-conscious: TF-IDF narrows candidates cheaply, LLM only verifies the interesting pairs — keeps API costs proportional to real findings
- User expressed long-term interest in semantic synonym detection and non-obvious links (future enhancement beyond TF-IDF keyword matching — possibly embeddings or LLM-based topic matching in v2)

</specifics>

<deferred>
## Deferred Ideas

- Semantic synonym/embedding-based topic matching — future enhancement for deeper non-obvious link detection beyond TF-IDF
- Method-combination gap matrix (GAPS-04, v2)
- Open-problem aggregation across graph (GAPS-03, v2)
- File export (--output gaps.json) — revisit if users need programmatic access beyond DB queries

</deferred>

---

*Phase: 04-cross-paper-gap-analysis*
*Context gathered: 2026-03-14*
