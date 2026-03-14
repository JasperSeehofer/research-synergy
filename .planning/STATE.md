---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: planning
stopped_at: Completed 03-pluggable-llm-backend 03-03-PLAN.md
last_updated: "2026-03-14T12:48:57.053Z"
last_activity: 2026-03-14 — Roadmap created, 12/12 v1 requirements mapped to 5 phases
progress:
  total_phases: 5
  completed_phases: 3
  total_plans: 7
  completed_plans: 7
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-14)

**Core value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph
**Current focus:** Phase 1 — Text Extraction Foundation

## Current Position

Phase: 1 of 5 (Text Extraction Foundation)
Plan: 0 of TBD in current phase
Status: Ready to plan
Last activity: 2026-03-14 — Roadmap created, 12/12 v1 requirements mapped to 5 phases

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

**Velocity:**
- Total plans completed: 0
- Average duration: —
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**
- Last 5 plans: —
- Trend: —

*Updated after each plan completion*
| Phase 01-text-extraction-foundation P01 | 20 | 2 tasks | 6 files |
| Phase 01-text-extraction-foundation P02 | 94min | 2 tasks | 3 files |
| Phase 02-nlp-analysis-db-schema P01 | 36 | 2 tasks | 5 files |
| Phase 02-nlp-analysis-db-schema P02 | 5 | 2 tasks | 4 files |
| Phase 03-pluggable-llm-backend P01 | 17 | 2 tasks | 11 files |
| Phase 03-pluggable-llm-backend PP02 | 12 | 2 tasks | 4 files |
| Phase 03-pluggable-llm-backend P03 | 12 | 1 tasks | 1 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Pre-roadmap: Hybrid NLP + LLM analysis (NLP for structure, LLM for semantic depth)
- Pre-roadmap: Pluggable LLM backend via trait (mirrors existing PaperSource pattern)
- Pre-roadmap: Extend SurrealDB schema with migrations rather than auto-init DDL
- Pre-roadmap: ar5iv HTML as primary full-text source; LaTeX source deferred to v2
- [Phase 01-text-extraction-foundation]: Sections stored as flat fields on text_extraction table (not nested OBJECT) for SurrealDB SCHEMAFULL compatibility
- [Phase 01-text-extraction-foundation]: ExtractionMethod serialized as string in DB (AbstractOnly / Ar5ivHtml) matching PaperRecord DataSource pattern
- [Phase 01-text-extraction-foundation]: migrate_schema uses version guards (if version < N) so re-running applies only missing migrations — idempotent by design
- [Phase 01-text-extraction-foundation]: Delimiter-guarded Roman numeral stripping: only strip I/II/... when followed by delimiter to avoid consuming word-starts
- [Phase 01-text-extraction-foundation]: Abstract text extracted from .ltx_para children (not all text) to exclude the 'Abstract' heading element in LaTeXML output
- [Phase 01-text-extraction-foundation]: run_analysis() extracted as async helper reused from both db_only and normal flows
- [Phase 02-nlp-analysis-db-schema]: SurrealDB FLEXIBLE TYPE syntax is TYPE object FLEXIBLE (FLEXIBLE comes after type name)
- [Phase 02-nlp-analysis-db-schema]: AnalysisRecord uses serde_json::Value for tfidf_vector to bridge HashMap into SurrealDB FLEXIBLE object
- [Phase 02-nlp-analysis-db-schema]: Unigrams only for TF-IDF (bigrams deferred to v2 per CONTEXT.md)
- [Phase 02-nlp-analysis-db-schema]: Stop word filtering at compute_weighted_tf() stage, not tokenize() — keeps tokenize() a pure utility
- [Phase 02-nlp-analysis-db-schema]: Corpus fingerprint skip guard uses let-chain syntax (Rust edition 2024) to collapse nested if-let
- [Phase 03-pluggable-llm-backend]: Methods/findings stored as JSON strings (TYPE string) in SurrealDB SCHEMAFULL — avoids nested-object field enforcement; consistent with tfidf_vector lesson
- [Phase 03-pluggable-llm-backend]: LlmProvider uses &mut self for rate-limit state mutation in future providers (mirrors InspireHepClient)
- [Phase 03-pluggable-llm-backend]: reqwest json feature added as feature flag on existing dep to enable .json() method — no new crate
- [Phase 03-pluggable-llm-backend]: LlmAnnotationRaw defined once in claude.rs as pub(crate), reused by ollama.rs — single source of truth for LLM output shape
- [Phase 03-pluggable-llm-backend]: with_base_url builder pattern on both providers enables wiremock injection without env var manipulation
- [Phase 03-pluggable-llm-backend]: NoopProvider is a unit struct without new() — constructed directly as NoopProvider literal
- [Phase 03-pluggable-llm-backend]: LLM step disabled by default — only runs when --llm-provider is explicitly specified (Option<String>)

### Pending Todos

None yet.

### Blockers/Concerns

- Phase 3: `genai` 0.5 uses reqwest 0.13 vs project reqwest 0.12 — verify Cargo compiles both before designing provider implementations
- Phase 4: SurrealDB HNSW vector index performance at 200+ papers with 384-dim vectors is unverified — benchmark early
- Phase 4: Entity normalization strategy for HEP domain (InspireHEP keyword taxonomy) needs feasibility check before implementing contradiction detection

## Session Continuity

Last session: 2026-03-14T12:43:33.831Z
Stopped at: Completed 03-pluggable-llm-backend 03-03-PLAN.md
Resume file: None
