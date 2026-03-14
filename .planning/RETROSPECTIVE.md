# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — Analysis Pipeline

**Shipped:** 2026-03-14
**Phases:** 5 | **Plans:** 12 | **Sessions:** ~6

### What Was Built
- Text extraction pipeline with ar5iv HTML section parsing and abstract-only fallback
- Offline NLP module (TF-IDF with section weighting, corpus fingerprint caching)
- Pluggable LLM backend (Claude, Ollama, Noop providers) with per-paper SurrealDB caching
- Cross-paper gap analysis (contradiction detection via cosine similarity + finding divergence, ABC-bridge discovery via graph distance + shared terms)
- Enriched visualization (paper-type coloring, finding-strength sizing, edge tinting via custom TintedEdgeShape, Analysis panel with toggle/legend, hover tooltips)

### What Worked
- Phase-by-phase execution with clear dependencies kept each phase focused and testable
- DB migration system established early (Phase 1) paid off through Phases 2-5 with zero schema issues
- Corpus fingerprint caching pattern reused across NLP and gap analysis with independent invalidation
- Pure logic functions in enrichment.rs enabled TDD without GUI testing infrastructure
- ROADMAP plan checkboxes fell behind but SUMMARY.md + VERIFICATION.md provided reliable completion evidence

### What Was Inefficient
- ROADMAP plan checkboxes got stale for phases 2-5 — the GSD tooling updated plan counts but not individual plan checkboxes
- Phase 4 SUMMARY frontmatter didn't list requirements_completed for GAPS-01/GAPS-02 — executor should populate this
- Gap findings computed in Phase 4 are not wired into the visualization (Phase 5) — would need a follow-up to show contradictions/bridges visually
- Nyquist validation files exist for all phases but none are compliant — test coverage is present but VALIDATION.md wasn't filled

### Patterns Established
- SurrealDB SCHEMAFULL + JSON strings for complex fields (methods, findings, tfidf_vector) — works but limits server-side querying
- `pub trait + async_trait` for pluggable backends (PaperSource, LlmProvider)
- Corpus fingerprint guard pattern for idempotent recomputation
- `load_X_data()` async helper called before sync `launch_visualization()` — keeps GUI code sync
- TintedEdgeShape wrapper pattern when egui_graphs lacks set_color on edges

### Key Lessons
1. Establish the DB migration system in the first phase — every subsequent phase benefits from safe schema extension
2. Pure logic functions separated from rendering code enable TDD for visualization features
3. SurrealDB SCHEMAFULL + JSON strings is a pragmatic workaround but should be revisited if query patterns need server-side filtering on nested fields
4. Cross-phase data wiring should be verified at the integration level — Phase 4 gap findings being absent from Phase 5 visualization was caught by the integration checker but only at audit time

### Cost Observations
- Model mix: ~20% opus (orchestration), ~80% sonnet (execution, verification)
- Sessions: ~6 (one per phase + audit + completion)
- Notable: Parallel wave execution not exercised (all waves had 1 plan each) — would benefit from larger phases

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v1.0 | ~6 | 5 | First milestone — established migration, caching, and trait patterns |

### Cumulative Quality

| Milestone | Tests | Coverage | Zero-Dep Additions |
|-----------|-------|----------|-------------------|
| v1.0 | 153 | — | 5 (sha2, stop-words, chrono, serde_json feature, regex) |

### Top Lessons (Verified Across Milestones)

1. Migration system first, features second — pays compound interest through every subsequent phase
2. Pure logic functions before rendering code — enables testing without GUI infrastructure
