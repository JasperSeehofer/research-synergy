---
phase: 28-forward-citation-crawl-mode-s2
plan: "02"
subsystem: resyn-core
tags: [trait-extension, datamodel, serde, tdd]
dependency_graph:
  requires: []
  provides:
    - PaperSource::fetch_citing_papers default-impl method (consumed by plan 01 SemanticScholarSource override and plan 04 crawler wiring)
    - Paper::citing_papers transient field (consumed by plan 01 S2 impl and plan 03 DB persistence)
    - Paper::get_citing_arxiv_ids accessor (consumed by plan 04 BFS enqueue)
  affects:
    - resyn-core/src/data_aggregation/traits.rs
    - resyn-core/src/datamodels/paper.rs
    - resyn-core/src/data_aggregation/openalex_bulk.rs
    - resyn-core/src/database/queries.rs
tech_stack:
  added: []
  patterns:
    - async-trait default no-op method (safe-by-construction EoP mitigation)
    - serde transient field pattern (#[serde(default, skip_serializing)])
key_files:
  created: []
  modified:
    - resyn-core/src/data_aggregation/traits.rs
    - resyn-core/src/datamodels/paper.rs
    - resyn-core/src/data_aggregation/openalex_bulk.rs
    - resyn-core/src/database/queries.rs
decisions:
  - "citing_papers_tests placed as separate #[cfg(test)] mod (not merged into existing tests mod) for clear separation of new behavior"
  - "openalex_bulk.rs and queries.rs Paper initializers updated with ..Default::default() (Rule 1 fix) to be forward-compatible with future fields"
  - "Trait-impl override for SemanticScholarSource deferred to plan 04 per chosen approach in plan frontmatter — keeps wave-1 plans file-disjoint"
metrics:
  duration: "~25 min"
  completed: "2026-04-27"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 4
---

# Phase 28 Plan 02: Trait + Datamodel Contract Surfaces Summary

Extended the `PaperSource` trait with a default-impl `fetch_citing_papers` no-op and added `Paper::citing_papers` transient field with `get_citing_arxiv_ids` accessor, fully gated by serde skip-serializing to prevent DB/export leakage.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend PaperSource trait with default-impl fetch_citing_papers | ed9a3c4 | resyn-core/src/data_aggregation/traits.rs |
| 2 (RED) | Add failing tests for Paper::citing_papers | 83c44ee | resyn-core/src/datamodels/paper.rs |
| 2 (GREEN) | Implement citing_papers field + get_citing_arxiv_ids | 72d32ac | resyn-core/src/datamodels/paper.rs, openalex_bulk.rs, queries.rs |

## Implementation Details

### Field Placement

`citing_papers: Vec<Reference>` placed at the **end of the `Paper` struct** (after `source: DataSource`), following plan spec. The field is annotated `#[serde(default, skip_serializing)]`:
- `default` — allows deserialization from DB/JSON that lacks the field (backward compat)
- `skip_serializing` — ensures the field never appears in `serde_json::to_*` output (T-28-08 mitigation verified by test)

### Method Placement

`get_citing_arxiv_ids` placed **immediately after `get_arxiv_references_ids`** in the `impl Paper` block. Exact mirror: same `filter_map(|r| r.get_arxiv_id().ok())` + empty-ID filter with `tracing::warn!`.

### Test Module Naming

Used a **separate `#[cfg(test)] mod citing_papers_tests`** module (not merged into the existing `tests` module). This keeps concerns separated and makes the 4 new tests easy to target with `cargo test -- citing_papers`.

### Trait Override Deferral

Per the **chosen approach** in the plan frontmatter (line 144-149): the `SemanticScholarSource` trait-impl override (`async fn fetch_citing_papers`) is NOT added in this plan. It will be wired in **plan 04** (wave 3) after plan 01's inherent method ships. This keeps wave-1 plans completely file-disjoint.

### All Existing Trait Implementors Compile Without Modification

Verified: `ArxivSource`, `InspireHepClient`, `ChainedPaperSource`, and `SemanticScholarSource` all inherit the no-op default and required zero changes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed missing `citing_papers` field in two Paper struct initializers**
- **Found during:** Task 2 GREEN phase (clippy `-Dwarnings` with `--all-features`)
- **Issue:** `openalex_bulk.rs:126` and `queries.rs:73` used explicit struct syntax for `Paper` initialization. Adding the new field caused `E0063: missing field` compile errors.
- **Fix:** Added `..Default::default()` to both initializers. This is future-proof — any subsequent new `Paper` fields with `Default` impls will not break these sites again.
- **Files modified:** `resyn-core/src/data_aggregation/openalex_bulk.rs`, `resyn-core/src/database/queries.rs`
- **Commit:** 72d32ac

## Known Stubs

None. The `citing_papers` field defaults to `Vec::new()` — but this is by design (it is populated transiently by `fetch_citing_papers` during crawl, not stored). Plan 01 wires the S2 implementation; plan 04 wires the crawler invocation.

## Deferred Items

Pre-existing clippy `-Dwarnings` failures (not caused by this plan):
- `resyn-core/src/graph_analytics/community.rs` — 4 unused constant/import warnings + `contains_key` followed by `insert` lint
- `resyn-core/src/data_aggregation/openalex_bulk.rs:76` — collapsible `if let` lint
- `resyn-core/src/data_aggregation/chained_source.rs` — fmt diff

These were present before this plan and are out of scope per the scope boundary rule.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The `citing_papers` field is explicitly excluded from DB serialization via `#[serde(skip_serializing)]` — T-28-08 verified by test.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| resyn-core/src/data_aggregation/traits.rs | FOUND |
| resyn-core/src/datamodels/paper.rs | FOUND |
| .planning/phases/28-forward-citation-crawl-mode-s2/28-02-SUMMARY.md | FOUND |
| commit ed9a3c4 (trait extension) | FOUND |
| commit 83c44ee (RED tests) | FOUND |
| commit 72d32ac (GREEN implementation) | FOUND |
| fetch_citing_papers in traits.rs | 1 hit |
| pub citing_papers field at line 35 | FOUND |
| pub fn get_citing_arxiv_ids at line 85 | FOUND |
| 4 citing_papers_tests pass | VERIFIED (117 total lib tests pass) |
