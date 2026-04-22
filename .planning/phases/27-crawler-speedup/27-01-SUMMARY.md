---
phase: 27-crawler-speedup
plan: "01"
subsystem: bulk-ingest
tags: [openalex, api-key, authentication, bulk-ingest, security]

dependency_graph:
  requires: []
  provides:
    - "OpenAlexBulkLoader with api_key field and Authorization: Bearer header"
    - "BulkIngestArgs with --api-key / OPENALEX_API_KEY env var and hard-fail"
    - "DEFAULT_FILTER_PHYSICS constant for cond-mat/stat-phys corpus"
  affects:
    - resyn-core/src/data_aggregation/openalex_bulk.rs
    - resyn-server/src/commands/bulk_ingest.rs
    - Cargo.toml (workspace)

tech_stack:
  added:
    - "clap env feature (added to workspace Cargo.toml)"
  patterns:
    - "clap env= attribute for env-var-backed CLI args"
    - "Option<String> + hard-fail guard pattern for required env-var credentials"
    - "Authorization: Bearer header injection (never in URL)"

key_files:
  modified:
    - resyn-core/src/data_aggregation/openalex_bulk.rs
    - resyn-server/src/commands/bulk_ingest.rs
    - Cargo.toml

decisions:
  - "D-01: --api-key backed by OPENALEX_API_KEY env var; hard-fail if absent — no deprecation shim"
  - "D-02: Hard-fail with actionable URL on missing key rather than silent unauthenticated fallback"
  - "D-03: api_key never appears in URL query params or tracing fields — only in Authorization header"
  - "[Rule 3 auto-fix] Added clap env feature to workspace Cargo.toml — required for env= attribute"

metrics:
  duration: "9m"
  completed: "2026-04-22T15:37:56Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 3
---

# Phase 27 Plan 01: OpenAlex API Key Migration Summary

Migrated OpenAlex bulk-ingest authentication from deprecated mailto polite pool to Bearer token API key. Without this change, bulk-ingest silently drops papers after ~20 pages due to a 100-credit daily cap introduced with OpenAlex's Feb 2026 deprecation of unauthenticated polite-pool access.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Replace mailto with api_key in OpenAlexBulkLoader | 004e960 | resyn-core/src/data_aggregation/openalex_bulk.rs |
| 2 | Migrate BulkIngestArgs to --api-key with hard-fail | 9856a7e | resyn-server/src/commands/bulk_ingest.rs, Cargo.toml |

## What Was Built

**Task 1 — openalex_bulk.rs:**
- `OpenAlexBulkLoader.mailto: String` replaced with `api_key: String`
- `fetch_page` URL no longer includes `?mailto=` parameter
- `Authorization: Bearer {api_key}` header added to every request
- Static `User-Agent` header retained for courtesy identification
- `api_key` never passed to any tracing call (T-27-02 mitigation)

**Task 2 — bulk_ingest.rs + Cargo.toml:**
- `DEFAULT_MAILTO` constant removed entirely
- `--mailto` arg removed; replaced with `--api-key` / `OPENALEX_API_KEY` env var (`Option<String>`)
- `DEFAULT_FILTER_PHYSICS` constant added for cond-mat/stat-phys corpus (`C26873012|C121864883`)
- Hard-fail guard in `run()`: exits 1 with actionable error if `api_key` absent or empty
- `OpenAlexBulkLoader::new(client, &api_key)` callsite updated
- `clap env` feature added to workspace `Cargo.toml` (required for `env=` attribute)
- 2 unit tests added: `test_api_key_flag_parsed`, `test_no_api_key_is_none`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] clap `env` feature not in workspace Cargo.toml**
- **Found during:** Task 2 — GREEN phase compile
- **Issue:** `#[arg(env = "OPENALEX_API_KEY")]` requires clap's `env` feature, which was absent from the workspace dependency declaration. Compiler error: `no method named env found for struct Arg`.
- **Fix:** Added `"env"` to the `features` list in the workspace `clap = { version = "4", features = ["derive", "env"] }` entry.
- **Files modified:** Cargo.toml
- **Commit:** 9856a7e

## TDD Gate Compliance

Task 2 followed TDD:
- RED: Tests added referencing `api_key` field before implementation; compile failed with `no field api_key on type BulkIngestArgs` — gate confirmed.
- GREEN: Implementation landed; `cargo test -p resyn-server` passed all tests including the 2 new unit tests.
- REFACTOR: Not needed — implementation was clean.

## Test Results

```
cargo test -p resyn-server
  test commands::bulk_ingest::tests::test_api_key_flag_parsed ... ok
  test commands::bulk_ingest::tests::test_no_api_key_is_none ... ok
  (+ 5 existing integration tests pass)

cargo test -p resyn-core --lib --features ssr -- data_aggregation::openalex_bulk
  6 tests pass (unchanged)
```

## Security Notes

All T-27-0x threats from the plan's threat model are mitigated:
- T-27-01: `?mailto=` removed from URL; key only in `Authorization` header
- T-27-02: `api_key` value never passed to `tracing::info!` or `tracing::error!`
- T-27-03: `env = "OPENALEX_API_KEY"` means key never needs to appear in `ps aux` output

## Known Stubs

None. No placeholder data paths introduced.

## Threat Flags

None. No new network endpoints, auth paths, or schema changes beyond the planned migration.

## Self-Check

- [x] `resyn-core/src/data_aggregation/openalex_bulk.rs` — modified, verified
- [x] `resyn-server/src/commands/bulk_ingest.rs` — modified, verified
- [x] `Cargo.toml` — modified, verified
- [x] Commit 004e960 exists: `git log --oneline | grep 004e960` — FOUND
- [x] Commit 9856a7e exists: `git log --oneline | grep 9856a7e` — FOUND

## Self-Check: PASSED
