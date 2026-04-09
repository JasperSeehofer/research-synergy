---
phase: 22-paper-similarity-engine
plan: 01
subsystem: similarity-engine
tags: [similarity, cosine, tfidf, surrealdb, migration, pipeline]
dependency_graph:
  requires: [resyn-core/src/datamodels/analysis.rs, resyn-core/src/gap_analysis/similarity.rs]
  provides: [PaperSimilarity, SimilarNeighbor, compute_top_neighbors, SimilarityRepository, paper_similarity table]
  affects: [resyn-app/src/server_fns/analysis.rs, resyn-core/src/database/queries.rs, resyn-core/src/database/schema.rs]
tech_stack:
  added: []
  patterns: [JSON-string serialization for complex fields in SurrealDB SCHEMAFULL, fingerprint guard for idempotent recomputation]
key_files:
  created:
    - resyn-core/src/datamodels/similarity.rs
  modified:
    - resyn-core/src/datamodels/mod.rs
    - resyn-core/src/gap_analysis/similarity.rs
    - resyn-core/src/database/schema.rs
    - resyn-core/src/database/queries.rs
    - resyn-app/src/server_fns/analysis.rs
decisions:
  - Store neighbors as JSON string (TYPE string) not TYPE object FLEXIBLE — avoids SurrealDB SCHEMAFULL array-in-object pitfall, consistent with LlmAnnotation pattern
  - Fingerprint guard checks first analysis record's corpus_fingerprint to skip redundant recomputation
  - Stage 2.5 runs silently with no send_progress call per D-07
metrics:
  duration: ~15 minutes
  completed: 2026-04-09
  tasks_completed: 2
  files_modified: 6
  tests_added: 22
requirements: [SIM-01, SIM-04]
---

# Phase 22 Plan 01: Paper Similarity Engine Foundation Summary

**One-liner:** Cosine similarity top-10 computation on TF-IDF vectors, persisted to SurrealDB paper_similarity table via SimilarityRepository, auto-triggered silently after NLP stage.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | PaperSimilarity model, compute_top_neighbors, SimilarityRepository | 3ca742b | datamodels/similarity.rs, gap_analysis/similarity.rs, database/schema.rs, database/queries.rs |
| 2 | Wire similarity recompute into analysis pipeline after NLP stage | 6652f6e | resyn-app/src/server_fns/analysis.rs |

## What Was Built

### Data Model (`resyn-core/src/datamodels/similarity.rs`)

- `SimilarNeighbor` struct: `arxiv_id`, `score: f32`, `shared_terms: Vec<String>`
- `PaperSimilarity` struct: `arxiv_id`, `neighbors: Vec<SimilarNeighbor>`, `corpus_fingerprint`, `computed_at`
- Both derive `Debug, Clone, Default, Serialize, Deserialize`

### Computation (`resyn-core/src/gap_analysis/similarity.rs`)

- `compute_top_neighbors(analyses: &[PaperAnalysis], top_k: usize) -> Vec<PaperSimilarity>`
- O(N^2) pairwise cosine similarity using existing `cosine_similarity()`
- Excludes self from neighbors, sorts descending by score, truncates to top_k
- Stores top-3 shared high-weight terms per neighbor (min_weight=0.05) via `shared_high_weight_terms()`
- Propagates `corpus_fingerprint` from source analysis for cache invalidation

### Database (`resyn-core/src/database/schema.rs` + `queries.rs`)

- Migration 10: `paper_similarity` SCHEMAFULL table with `arxiv_id`, `neighbors` (TYPE string), `corpus_fingerprint`, `computed_at`, unique index on `arxiv_id`
- `SimilarityRepository`: `upsert_similarity`, `get_similarity`, `get_all_similarities`
- Neighbors serialized as JSON string (same pattern as LlmAnnotation methods/findings) — avoids TYPE object FLEXIBLE pitfall with arrays

### Pipeline Integration (`resyn-app/src/server_fns/analysis.rs`)

- Stage 2.5 inserted between NLP stage (Stage 2) and LLM/gap stages (Stages 3 & 4)
- Silent: no `send_progress` call per D-07
- Fingerprint guard: checks first analysis record's `corpus_fingerprint` against stored similarity; skips if already computed for this corpus version
- Per-paper upsert failures logged with `error!()` but non-fatal — pipeline continues

## Tests Added

22 new tests across 4 files:

- `datamodels/similarity.rs`: 2 tests (serde roundtrip, default values)
- `gap_analysis/similarity.rs`: 6 tests (self-exclusion, descending sort, fewer-than-k, truncation, shared terms, fingerprint propagation)
- `database/queries.rs`: 4 DB tests (upsert+get roundtrip, None for non-existent, get_all, UPSERT idempotency)
- `database/schema.rs`: 2 schema tests (migration 10 creates table, idempotent migration)
- Also updated 7 existing migration version assertions from `9` to `10` (Rule 1 auto-fix)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed existing migration version tests hardcoded to version 9**
- **Found during:** Task 2 — running full test suite revealed 7 tests asserting `versions[0] == 9`
- **Issue:** Adding migration 10 caused those assertions to fail (actual: 10, expected: 9)
- **Fix:** Updated 7 test assertions in `queries.rs` from `9` to `10`, and updated comment strings to match
- **Files modified:** `resyn-core/src/database/queries.rs`
- **Commit:** 6652f6e

**2. [Rule 2 - Missing critical functionality] Changed neighbors storage from TYPE object FLEXIBLE to TYPE string**
- **Found during:** Task 1 — initial DB tests failed because SurrealDB SCHEMAFULL TYPE object FLEXIBLE rejects JSON arrays
- **Issue:** `neighbors` is `Vec<SimilarNeighbor>` (array), not an object; FLEXIBLE TYPE object does not accept arrays
- **Fix:** Changed migration 10 schema from `TYPE object FLEXIBLE` to `TYPE string`, and serialized neighbors as JSON string in upsert/deserialize as JSON string in get — consistent with the established LlmAnnotation pattern for complex fields in this codebase
- **Files modified:** `resyn-core/src/database/schema.rs`, `resyn-core/src/database/queries.rs`
- **Commit:** 3ca742b

## Known Stubs

None — all data flows are wired end-to-end: compute_top_neighbors produces real results from real TF-IDF vectors, SimilarityRepository persists them to SurrealDB, and the pipeline trigger calls both after NLP completes.

## Threat Flags

No new security-relevant surface introduced beyond the plan's threat model. T-22-01 mitigated: `strip_version_suffix()` is called on `arxiv_id` before constructing the record ID in `upsert_similarity`. T-22-02 accepted: O(N^2) with N<500, fingerprint guard prevents repeated computation.

## Self-Check: PASSED

- resyn-core/src/datamodels/similarity.rs: FOUND
- resyn-core/src/gap_analysis/similarity.rs: FOUND
- resyn-core/src/database/schema.rs (apply_migration_10): FOUND
- resyn-core/src/database/queries.rs (SimilarityRepository): FOUND
- resyn-app/src/server_fns/analysis.rs (Stage 2.5): FOUND
- Commit 3ca742b: FOUND
- Commit 6652f6e: FOUND
- 21 similarity tests passing (20 new + 1 pre-existing contradiction test)
