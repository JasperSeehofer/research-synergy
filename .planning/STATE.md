---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: planning
stopped_at: Completed 02-nlp-analysis-db-schema 02-02-PLAN.md
last_updated: "2026-03-14T11:04:00.339Z"
last_activity: 2026-03-14 — Roadmap created, 12/12 v1 requirements mapped to 5 phases
progress:
  total_phases: 5
  completed_phases: 2
  total_plans: 4
  completed_plans: 4
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

### Pending Todos

None yet.

### Blockers/Concerns

- Phase 3: `genai` 0.5 uses reqwest 0.13 vs project reqwest 0.12 — verify Cargo compiles both before designing provider implementations
- Phase 4: SurrealDB HNSW vector index performance at 200+ papers with 384-dim vectors is unverified — benchmark early
- Phase 4: Entity normalization strategy for HEP domain (InspireHEP keyword taxonomy) needs feasibility check before implementing contradiction detection

## Session Continuity

Last session: 2026-03-14T11:04:00.329Z
Stopped at: Completed 02-nlp-analysis-db-schema 02-02-PLAN.md
Resume file: None
