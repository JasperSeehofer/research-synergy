---
phase: 04-cross-paper-gap-analysis
plan: "01"
subsystem: gap-finding-foundation
tags: [data-model, database, llm, surrealdb, migration]
dependency_graph:
  requires: []
  provides:
    - GapFinding type (src/datamodels/gap_finding.rs)
    - GapFindingRepository (src/database/queries.rs)
    - migration-6 (gap_finding table)
    - LlmProvider::verify_gap method
    - CONTRADICTION_SYSTEM_PROMPT / ABC_BRIDGE_SYSTEM_PROMPT
  affects:
    - src/database/schema.rs (migration 6 added)
    - src/database/queries.rs (GapFindingRepository + version assertions updated)
    - src/llm/traits.rs (trait extended with verify_gap)
    - src/llm/claude.rs (verify_gap implemented)
    - src/llm/ollama.rs (verify_gap implemented)
    - src/llm/noop.rs (verify_gap implemented)
tech_stack:
  added: []
  patterns:
    - GapFinding uses CREATE not UPSERT for history preservation
    - GapFindingRecord stores Vec<String> fields as JSON strings (SCHEMAFULL compatibility)
    - verify_gap returns raw text string (no JSON parsing — matches RESEARCH.md spec)
    - GapType::as_str() for enum-to-DB-string conversion
key_files:
  created:
    - src/datamodels/gap_finding.rs
    - src/llm/gap_prompt.rs
  modified:
    - src/datamodels/mod.rs
    - src/database/schema.rs
    - src/database/queries.rs
    - src/llm/traits.rs
    - src/llm/mod.rs
    - src/llm/claude.rs
    - src/llm/ollama.rs
    - src/llm/noop.rs
decisions:
  - GapFinding uses CREATE (not UPSERT) to preserve gap detection history across multiple analysis runs
  - paper_ids and shared_terms stored as JSON strings in SurrealDB SCHEMAFULL (consistent with LlmAnnotation lesson from Phase 3)
  - verify_gap returns raw String (not structured type) — gap verification is a yes/no judgment, no JSON parsing needed
  - NoopProvider::verify_gap returns "NO" — consistent with noop producing empty-but-valid results
  - GapType serializes as snake_case via serde rename_all (contradiction / abc_bridge)
metrics:
  duration: "6 minutes"
  completed: "2026-03-14"
  tasks_completed: 2
  files_created: 2
  files_modified: 8
---

# Phase 4 Plan 1: Gap Finding Foundation Summary

**One-liner:** GapFinding type with SurrealDB migration 6, history-preserving INSERT repository, and LlmProvider::verify_gap with contradiction/ABC-bridge prompt templates.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | GapFinding data model + migration 6 + GapFindingRepository | 4811126 | src/datamodels/gap_finding.rs, src/database/schema.rs, src/database/queries.rs |
| 2 | LlmProvider verify_gap method + gap prompt templates | 4bed9e6 | src/llm/traits.rs, src/llm/gap_prompt.rs, src/llm/claude.rs, src/llm/ollama.rs, src/llm/noop.rs |

## What Was Built

### Task 1: GapFinding Data Model + Migration 6 + GapFindingRepository

Created the complete data layer for gap findings:

- `GapType` enum with `Contradiction` and `AbcBridge` variants, `as_str()` method returning "contradiction" / "abc_bridge", full serde derive with `rename_all = "snake_case"`
- `GapFinding` struct with `gap_type`, `paper_ids`, `shared_terms`, `justification`, `confidence`, `found_at` fields
- Migration 6 defines `gap_finding` SCHEMAFULL table with appropriate field types
- `GapFindingRecord` stores `paper_ids` and `shared_terms` as JSON strings (consistent with Phase 3 lesson about SCHEMAFULL and arrays)
- `GapFindingRepository::insert_gap_finding` uses `CREATE gap_finding CONTENT $record` (auto-generated ID) — NOT UPSERT — so multiple gap detection runs for the same paper pair create separate history records
- `GapFindingRepository::get_all_gap_findings` returns all findings from the table

### Task 2: LlmProvider verify_gap + Gap Prompt Templates

Extended the LLM layer for gap verification:

- Added `verify_gap(&mut self, prompt: &str, context: &str) -> Result<String, ResynError>` to `LlmProvider` trait
- Created `src/llm/gap_prompt.rs` with two system prompt constants:
  - `CONTRADICTION_SYSTEM_PROMPT`: instructs LLM to detect genuine contradictions, respond with 1-2 sentence justification or exactly "NO"
  - `ABC_BRIDGE_SYSTEM_PROMPT`: instructs LLM to identify A→C connections via shared B concepts, respond with justification or "NO"
- Implemented `verify_gap` on all three providers:
  - `ClaudeProvider`: reuses `rate_limit_check()` + `call_api()`, returns raw text
  - `OllamaProvider`: reuses `rate_limit_check()` + `call_api()`, returns raw text
  - `NoopProvider`: returns `"NO"` — noop never confirms gaps

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated migration version assertions from 5 to 6**

- **Found during:** Task 1 verification (`cargo test test_migrate`)
- **Issue:** Six existing tests in `database/queries.rs` had hardcoded `assert_eq!(versions[0], 5)`. After adding migration 6, these all failed with `left: 6, right: 5`.
- **Fix:** Updated all six test assertions to check for version 6. The test names (`test_migrate_schema_idempotent_v5`, etc.) were preserved to not confuse future readers — they document when that migration was verified, not the current max.
- **Files modified:** `src/database/queries.rs`
- **Commit:** 4811126

## Test Results

- `cargo test gap_finding` — 7 passed (3 unit + 4 DB)
- `cargo test test_migrate` — 7 passed
- `cargo test llm` — 16 passed
- `cargo test` (full suite) — 107 passed, 0 failed

## Self-Check: PASSED
