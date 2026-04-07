---
phase: 21-search-filter
plan: "01"
subsystem: database + server-fns
tags: [search, fulltext, bm25, surrealdb, migration, leptos]
dependency_graph:
  requires: []
  provides: [SearchRepository, search_papers-server-fn, migration-9-bm25-indexes]
  affects: [resyn-core/database, resyn-app/server_fns]
tech_stack:
  added: []
  patterns: [SurrealDB BM25 fulltext search, parameterized bind queries, serde_json::Value deserialization]
key_files:
  created: []
  modified:
    - resyn-core/src/database/schema.rs
    - resyn-core/src/database/queries.rs
    - resyn-app/src/server_fns/papers.rs
decisions:
  - "Use serde_json::Value intermediate for SurrealDB response deserialization — avoids SurrealValue trait bound requirement"
  - "Bind query as owned String to satisfy 'static lifetime requirement on .bind()"
  - "Score weighting: title*2.0 + summary*1.5 + authors*1.0 — title matches rank highest per D-11"
metrics:
  duration: "~25 minutes"
  completed: "2026-04-07"
  tasks_completed: 2
  files_modified: 3
---

# Phase 21 Plan 01: Search Backend — BM25 Indexes + SearchRepository + Server Fn Summary

SurrealDB BM25 fulltext search on paper title/summary/authors with SearchRepository query layer and Leptos search_papers server fn, backed by 6 passing DB integration tests.

## What Was Built

**Migration 9 (schema.rs):** Defines `paper_analyzer` (blank+class tokenizers, lowercase+ascii filters) and three BM25 fulltext indexes: `idx_paper_fts_title`, `idx_paper_fts_summary`, `idx_paper_fts_authors`. Uses `IF NOT EXISTS` per project migration convention. Dispatched in `migrate_schema` after the existing `version < 8` block.

**SearchRepository (queries.rs):** New public struct with `search_papers(&self, query: &str, limit: usize)` method. Uses multi-field `@0@`/`@1@`/`@2@` predicates with `search::score(N)` weighted combination. Parameterized via `.bind(("query", query_owned))` — no string interpolation (mitigates T-21-01). Empty-string guard returns `Ok(vec![])` without DB hit (mitigates T-21-02). Deserializes via `Vec<serde_json::Value>` to avoid `SurrealValue` trait bound issues.

**SearchResult + search_papers server fn (papers.rs):** `SearchResult` struct with `arxiv_id`, `title`, `authors`, `year` (first 4 chars of `published`), `score`. Server fn accepts `query: String, limit: Option<usize>` (defaults to 10). Delegates to `SearchRepository::new(&db).search_papers()`.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Migration 9 + SearchRepository + 6 DB tests (TDD) | 1458bf6 |
| 2 | SearchResult + search_papers server fn + version assertion fixes | 01d6e13 |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated 7 hardcoded schema version assertions from 8 to 9**
- **Found during:** Task 2 full test suite run
- **Issue:** Existing tests checked `schema_migrations` version == 8, which was correct before migration 9. After adding migration 9, all `connect("mem://")` calls produce version 9.
- **Fix:** Updated 7 assert_eq! calls and associated comments in `queries.rs` test blocks (test_migrate_schema_creates_tables, test_migrate_schema_is_idempotent, test_migrate_schema_applies_all_migrations, test_migrate_schema_idempotent_from_v2, test_migrate_schema_applies_migration_5, test_migrate_schema_idempotent_v5, test_migrate_schema_creates_gap_finding_table).
- **Files modified:** resyn-core/src/database/queries.rs
- **Commit:** 01d6e13

**2. [Rule 1 - Bug] Used owned String bind instead of &str for query parameter**
- **Found during:** Task 1 compilation
- **Issue:** `.bind(("query", query))` where `query: &str` fails with E0521 (borrowed data escapes method body — bind requires `'static`).
- **Fix:** Added `let query_owned = query.to_string()` before the query chain and bound the owned value.
- **Files modified:** resyn-core/src/database/queries.rs
- **Commit:** 1458bf6

**3. [Rule 1 - Bug] Used serde_json::Value deserialization instead of direct struct deserialization**
- **Found during:** Task 1 compilation
- **Issue:** `response.take::<Vec<SearchResultRow>>(0)` requires `SearchResultRow: SurrealValue` (a surrealdb-internal derive), which `#[derive(Serialize, Deserialize)]` alone does not satisfy.
- **Fix:** Deserialize as `Vec<serde_json::Value>` and map to `SearchResultRow` manually — consistent with existing `PaletteRepository::get_palette()` pattern in the same file.
- **Files modified:** resyn-core/src/database/queries.rs
- **Commit:** 1458bf6

## Known Stubs

None — `SearchRepository` and `search_papers` server fn are fully wired and return real data from SurrealDB.

## Threat Flags

No new network endpoints, auth paths, or file access patterns introduced beyond what was planned. The `search_papers` server fn exposes paper metadata (title, authors, year) which is public academic data (T-21-03, accepted per plan).

## Test Results

```
cargo test -p resyn-core --lib --features ssr -- test_search
running 6 tests
test database::queries::tests::test_search_papers_no_match ... ok
test database::queries::tests::test_search_papers_empty_query ... ok
test database::queries::tests::test_search_papers_by_author ... ok
test database::queries::tests::test_search_papers_result_order ... ok
test database::queries::tests::test_search_papers_returns_ranked_results ... ok
test database::queries::tests::test_search_papers_title_scores_higher ... ok
test result: ok. 6 passed; 0 failed

cargo test -p resyn-core --lib --features ssr
test result: ok. 206 passed; 0 failed

cargo check -p resyn-app --features ssr
Finished `dev` profile
```

Note: Pre-existing `resyn-core/tests/arxiv_text_extraction.rs` integration test fails due to `#[cfg(feature = "ssr")]`-gated `data_aggregation` module not being available in that test binary context. This is pre-existing before this plan (verified by git stash test).

## Self-Check: PASSED

- resyn-core/src/database/schema.rs — FOUND
- resyn-core/src/database/queries.rs — FOUND
- resyn-app/src/server_fns/papers.rs — FOUND
- .planning/phases/21-search-filter/21-01-SUMMARY.md — FOUND
- Commit 1458bf6 — FOUND
- Commit 01d6e13 — FOUND
