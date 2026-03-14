---
phase: 03-pluggable-llm-backend
plan: "01"
subsystem: llm
tags: [llm, trait, datamodels, database, migration, noop]
dependency_graph:
  requires: [src/datamodels/analysis.rs, src/database/queries.rs, src/database/schema.rs, src/error.rs]
  provides: [src/datamodels/llm_annotation.rs, src/llm/traits.rs, src/llm/noop.rs, src/llm/prompt.rs, src/database/llm_annotation_repository]
  affects: [src/datamodels/mod.rs, src/lib.rs, src/main.rs]
tech_stack:
  added: [async-trait (existing), chrono::Utc (existing), serde_json string serialization for nested objects]
  patterns: [SurrealValue derive with JSON string fields for SCHEMAFULL nested-object workaround, async_trait LlmProvider with &mut self for state mutation]
key_files:
  created:
    - src/datamodels/llm_annotation.rs
    - src/llm/mod.rs
    - src/llm/traits.rs
    - src/llm/noop.rs
    - src/llm/prompt.rs
  modified:
    - src/error.rs
    - src/datamodels/mod.rs
    - src/database/schema.rs
    - src/database/queries.rs
    - src/lib.rs
    - src/main.rs
decisions:
  - "Methods and findings stored as JSON strings (TYPE string) in SurrealDB SCHEMAFULL — avoids nested-object field enforcement pitfall; consistent with Phase 2 tfidf_vector lesson"
  - "LlmProvider uses &mut self (not &self) to allow rate-limit state mutation in future providers (mirrors InspireHepClient)"
  - "NoopProvider logs full constructed prompt at debug level, not just arxiv_id"
metrics:
  duration_minutes: 17
  tasks_completed: 2
  files_created: 5
  files_modified: 6
  tests_added: 14
  completed_date: "2026-03-14"
---

# Phase 3 Plan 1: LLM Foundation (Types, Trait, Noop, Migration, Repository) Summary

**One-liner:** LlmAnnotation data models, LlmProvider async trait, NoopProvider, SurrealDB migration 5, and LlmAnnotationRepository — full foundation for pluggable LLM backend.

## What Was Built

### Data Models (`src/datamodels/llm_annotation.rs`)

Three new structs with full serde support:
- `Finding { text: String, strength: String }` — individual paper finding with evidence strength
- `Method { name: String, category: String }` — method used in paper with category classification
- `LlmAnnotation { arxiv_id, paper_type, methods, findings, open_problems, provider, model_name, annotated_at }` — top-level annotation container

### LLM Module (`src/llm/`)

- `traits.rs` — `LlmProvider` async trait with `Send + Sync` bounds, `&mut self` for rate-limit state, `annotate_paper()` and `provider_name()` methods
- `noop.rs` — `NoopProvider` returning empty-collection annotation with `paper_type: "unknown"`, logging constructed prompt at `debug!` level
- `prompt.rs` — `SYSTEM_PROMPT`, `RETRY_NUDGE`, and `LLM_ANNOTATION_SCHEMA` (JSON Schema for Ollama `format` field) constants

### Error Variant (`src/error.rs`)

Added `ResynError::LlmApi(String)` with Display: `"LLM API error: {msg}"`.

### Database Migration (`src/database/schema.rs`)

Migration 5 creates `llm_annotation` SCHEMAFULL table. Methods and findings stored as `TYPE string` (JSON-encoded) rather than `TYPE array` — workaround for SurrealDB SCHEMAFULL nested-object field enforcement.

### Repository (`src/database/queries.rs`)

`LlmAnnotationRepository<'a>` with:
- `upsert_annotation` — UPSERT with version-suffix stripping
- `get_annotation` — SELECT by arxiv_id (version-stripped)
- `annotation_exists` — boolean check
- `get_all_annotations` — full table scan

## Tests Added

| Test | Location | What it verifies |
|------|----------|-----------------|
| `test_llm_annotation_serde_roundtrip` | llm_annotation | Full struct serializes/deserializes identically |
| `test_finding_serde_roundtrip` | llm_annotation | Finding struct roundtrip |
| `test_method_serde_roundtrip` | llm_annotation | Method struct roundtrip |
| `test_llm_annotation_empty_vecs_serde` | llm_annotation | Empty collections serialize without error |
| `test_noop_provider_returns_valid_annotation` | noop | All fields correct, annotated_at non-empty |
| `test_noop_provider_name` | noop | Returns "noop" |
| `test_annotation_schema_is_valid_json` | prompt | LLM_ANNOTATION_SCHEMA parses as valid JSON |
| `test_llm_api_error_display` | error | Display format correct |
| `test_migrate_schema_applies_migration_5` | queries | Schema version 5 after connect |
| `test_migrate_schema_idempotent_v5` | queries | Running twice stays at version 5 |
| `test_annotation_upsert_and_get` | queries | All fields persist and retrieve correctly |
| `test_annotation_exists` | queries | false before upsert, true after |
| `test_get_all_annotations` | queries | 3 upserted → 3 returned |
| `test_annotation_version_suffix_dedup` | queries | Upsert v2, get bare ID succeeds |

Total: 92 tests passing (14 new).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] SurrealDB SCHEMAFULL TYPE array rejects nested object fields**
- **Found during:** Task 2 (initial UPSERT test failure)
- **Issue:** Defining `methods` and `findings` as `TYPE array` in SCHEMAFULL still enforces schema on nested object elements (e.g., `findings[0].strength` not declared). UPSERT silently failed (Rust returned Ok, but SurrealDB rejected content).
- **Fix:** Changed schema to `TYPE string` for methods and findings; serialize/deserialize as JSON strings in `LlmAnnotationRecord`. This is consistent with the Phase 2 `tfidf_vector` lesson (stored as `TYPE object FLEXIBLE` there, here we use `TYPE string` since array FLEXIBLE is not valid syntax).
- **Files modified:** `src/database/schema.rs`, `src/database/queries.rs`
- **Commit:** e1d38ca

## Self-Check: PASSED

All key files confirmed present. Both task commits exist in git log.
