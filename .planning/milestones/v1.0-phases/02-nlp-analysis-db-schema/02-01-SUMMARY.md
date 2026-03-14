---
phase: 02-nlp-analysis-db-schema
plan: "01"
subsystem: database/datamodels
tags: [surrealdb, migrations, data-models, tfidf, persistence]
dependency_graph:
  requires: []
  provides: [PaperAnalysis, AnalysisMetadata, AnalysisRepository, paper_analysis table, analysis_metadata table]
  affects: [src/database/queries.rs, src/database/schema.rs, src/datamodels/]
tech_stack:
  added: [sha2 = "0.10", stop-words = "0.8"]
  patterns: [SurrealDB FLEXIBLE TYPE object for sparse HashMap, parallel arrays for top_terms/top_scores, version guard migrations]
key_files:
  created:
    - src/datamodels/analysis.rs
  modified:
    - src/datamodels/mod.rs
    - src/database/schema.rs
    - src/database/queries.rs
    - Cargo.toml
decisions:
  - "SurrealDB FLEXIBLE TYPE syntax is TYPE object FLEXIBLE (FLEXIBLE comes after type, not before) — plan DDL had wrong order"
  - "AnalysisRecord uses serde_json::Value for tfidf_vector field to bridge HashMap<String,f32> into SurrealDB FLEXIBLE object"
  - "Updated existing migration tests from asserting version=2 to version=4 after adding migrations 3 and 4"
metrics:
  duration: "36 minutes"
  completed: "2026-03-14"
  tasks_completed: 2
  files_modified: 5
---

# Phase 2 Plan 1: Analysis Data Models and DB Persistence Summary

**One-liner:** SurrealDB schema migrations 3+4 with paper_analysis (FLEXIBLE TYPE object for sparse TF-IDF) and analysis_metadata tables, plus AnalysisRepository following ExtractionRepository pattern — all backed by 11 new tests.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | PaperAnalysis/AnalysisMetadata models + crate deps | 3a674b0 | src/datamodels/analysis.rs, mod.rs, Cargo.toml |
| 2 | DB migrations 3+4 and AnalysisRepository | db2553d | src/database/schema.rs, src/database/queries.rs |

## What Was Built

### Data Models (`src/datamodels/analysis.rs`)

Two structs with serde derives:

- `PaperAnalysis` — stores sparse TF-IDF vector as `HashMap<String, f32>`, keywords as parallel `top_terms: Vec<String>` + `top_scores: Vec<f32>` arrays, arxiv_id, corpus_fingerprint, analyzed_at timestamp.
- `AnalysisMetadata` — stores corpus metadata: key (e.g. "corpus_tfidf"), paper_count, corpus_fingerprint, last_analyzed timestamp.

Parallel arrays are used instead of `Vec<(String, f32)>` because SurrealDB's SurrealValue derive macro doesn't handle tuples reliably.

### DB Migrations (`src/database/schema.rs`)

- **Migration 3** — `paper_analysis` SCHEMAFULL table with `tfidf_vector TYPE object FLEXIBLE` for arbitrary term keys, parallel array fields, index on arxiv_id.
- **Migration 4** — `analysis_metadata` SCHEMAFULL table with unique index on key.
- Both use `if version < N` guards in `migrate_schema()` — fully idempotent.

### AnalysisRepository (`src/database/queries.rs`)

Follows ExtractionRepository pattern exactly:

- `AnalysisRecord` with `#[derive(SurrealValue)]` using `serde_json::Value` for tfidf_vector field
- `From<&PaperAnalysis> for AnalysisRecord` serializes HashMap to serde_json::Value via `serde_json::to_value()`
- `AnalysisRecord::to_analysis()` deserializes back via `serde_json::from_value()`
- `MetadataRecord` with SurrealValue derive for analysis_metadata table
- `AnalysisRepository<'a>`: upsert_analysis, get_analysis, analysis_exists, get_all_analyses, upsert_metadata, get_metadata
- `strip_version_suffix()` applied on all upsert operations

## Test Results

67 total tests pass (up from 56 before this plan):
- 4 new unit tests in `src/datamodels/analysis.rs` (serde roundtrips)
- 7 new DB integration tests in `src/database/queries.rs` (migration idempotency, upsert/get roundtrip, exists, get_all, metadata, version suffix stripping)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Wrong FLEXIBLE TYPE syntax in migration 3 DDL**
- **Found during:** Task 2 (test run)
- **Issue:** Plan specified `FLEXIBLE TYPE object` but SurrealDB 3.0.4 requires FLEXIBLE after the type: `TYPE object FLEXIBLE`
- **Fix:** Changed DDL from `FLEXIBLE TYPE object` to `TYPE object FLEXIBLE` (confirmed via surrealdb-core source)
- **Files modified:** src/database/schema.rs
- **Commit:** db2553d

**2. [Rule 2 - Missing update] Existing migration tests expected schema version 2**
- **Found during:** Task 2 (running all tests after adding migrations 3+4)
- **Issue:** `test_migrate_schema_creates_tables` and `test_migrate_schema_is_idempotent` asserted `versions[0] == 2`, which would fail now that migrate_schema applies 4 migrations
- **Fix:** Updated both assertions to expect version 4
- **Files modified:** src/database/queries.rs
- **Commit:** db2553d

## Self-Check: PASSED

- src/datamodels/analysis.rs: FOUND
- src/database/schema.rs: FOUND
- src/database/queries.rs: FOUND
- Commit 3a674b0: FOUND
- Commit db2553d: FOUND
