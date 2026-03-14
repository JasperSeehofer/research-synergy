---
phase: 01-text-extraction-foundation
plan: 01
subsystem: database
tags: [surrealdb, rust, chrono, data-models, migrations]

# Dependency graph
requires: []
provides:
  - TextExtractionResult, ExtractionMethod, SectionMap data models in src/datamodels/extraction.rs
  - DB migration system (migrate_schema) with schema_migrations version tracking
  - text_extraction SurrealDB table via migration v2
  - ExtractionRepository with upsert/get/exists/get_all methods in src/database/queries.rs
affects:
  - 01-02 (text extractor builds against TextExtractionResult and ExtractionRepository)
  - 01-03 (NLP pipeline uses ExtractionRepository to persist section text)

# Tech tracking
tech-stack:
  added:
    - chrono 0.4 with serde feature (timestamp generation via Utc::now().to_rfc3339())
  patterns:
    - Repository pattern: struct holding &Db, methods returning Result<T, ResynError>
    - SurrealValue derive macro for DB record types, separate from domain model
    - From<&DomainType> for Record + Record::to_domain() conversion pattern
    - Versioned migration system: schema_migrations table tracks applied version, IF NOT EXISTS DDL for idempotency

key-files:
  created:
    - src/datamodels/extraction.rs
  modified:
    - src/datamodels/mod.rs
    - src/database/schema.rs
    - src/database/queries.rs
    - src/database/client.rs
    - Cargo.toml

key-decisions:
  - "Sections stored as flat fields on text_extraction table (not nested OBJECT) for SurrealDB SCHEMAFULL compatibility"
  - "ExtractionMethod serialized as string in DB (AbstractOnly / Ar5ivHtml) matching PaperRecord's DataSource pattern"
  - "migrate_schema uses version guard (if version < N) so re-running applies only missing migrations"

patterns-established:
  - "Migration pattern: define schema_migrations SCHEMAFULL first, read current version, apply missing migrations in order, record each migration version"
  - "ExtractionRepository follows PaperRepository pattern exactly: RecordId::new(table, strip_version_suffix(id))"

requirements-completed: [INFR-03]

# Metrics
duration: 20min
completed: 2026-03-14
---

# Phase 1 Plan 01: Data Models and DB Migration Foundation Summary

**TextExtractionResult/SectionMap models + versioned SurrealDB migrations (v1 paper+cites, v2 text_extraction) + ExtractionRepository with full CRUD**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-03-14T01:42:20Z
- **Completed:** 2026-03-14T02:02:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Created TextExtractionResult with structured SectionMap (abstract, introduction, methods, results, conclusion) and from_abstract() constructor
- Replaced single-shot init_schema() with migrate_schema() that tracks schema versions in schema_migrations table and applies migrations idempotently
- Added ExtractionRepository providing upsert, get, exists, get_all operations for text_extraction records with version-suffix stripping

## Task Commits

Each task was committed atomically:

1. **Task 1: TextExtractionResult data model and chrono dependency** - `c4a6e69` (feat)
2. **Task 2: DB migration system and ExtractionRepository** - `be609d4` (feat)

**Plan metadata:** (docs commit follows)

_Note: TDD tasks — tests and implementation written together in single commits per task_

## Files Created/Modified
- `src/datamodels/extraction.rs` - ExtractionMethod, SectionMap, TextExtractionResult with from_abstract()
- `src/datamodels/mod.rs` - Added pub mod extraction
- `src/database/schema.rs` - Replaced init_schema with migrate_schema, versioned migrations v1+v2
- `src/database/queries.rs` - Added ExtractionRecord, ExtractionRepository with 4 methods + 6 new tests
- `src/database/client.rs` - Updated setup() to call migrate_schema
- `Cargo.toml` - Added chrono 0.4 with serde feature

## Decisions Made
- Sections stored as flat fields on text_extraction table (not nested OBJECT) — SurrealDB SCHEMAFULL requires explicit field definitions, nested objects would require additional DEFINE FIELD statements per subfield
- ExtractionMethod serialized as string in DB to match PaperRecord's DataSource pattern (consistent codebase convention)
- migrate_schema uses `if version < N` guards so re-running is a no-op for already-applied migrations (idempotent by design)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed clippy warnings in test code**
- **Found during:** Task 2 (ExtractionRepository)
- **Issue:** `assert_eq!(value, true)` and `format!("static string")` triggered clippy warnings (useless_format, bool comparison)
- **Fix:** Changed to `assert!(value)` and `.to_string()` respectively
- **Files modified:** src/database/queries.rs
- **Verification:** cargo clippy --all-targets --all-features reports zero warnings
- **Committed in:** be609d4 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (lint correctness)
**Impact on plan:** Minimal. Required to pass `cargo clippy --all-targets --all-features` with no warnings per CI requirements.

## Issues Encountered
- Disk full error during first test run (target/ cache from previous builds exceeded disk). Resolved with `cargo clean` (freed 15.6GB). Build cache was rebuilt from scratch adding ~5 minutes to compile time.

## Next Phase Readiness
- TextExtractionResult, ExtractionMethod, SectionMap types ready for Plan 02 (text extractor implementation)
- ExtractionRepository ready for persistence layer use in Plans 02-03
- migrate_schema pattern established; Plan 02 can add migration v3 if schema changes needed
- All 47 existing tests pass with no regression from init_schema -> migrate_schema

---
*Phase: 01-text-extraction-foundation*
*Completed: 2026-03-14*
