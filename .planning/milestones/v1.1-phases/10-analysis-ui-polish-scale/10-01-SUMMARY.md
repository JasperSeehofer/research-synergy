---
phase: 10-analysis-ui-polish-scale
plan: 01
subsystem: api
tags: [rust, serde, llm, petgraph, provenance, lod, bfs]

requires:
  - phase: 08-leptos-web-shell-analysis-panels
    provides: LlmAnnotation, PaperDetail, GraphData DTOs and server functions

provides:
  - Finding and Method structs with source_section/source_snippet provenance fields (backward-compatible serde)
  - SYSTEM_PROMPT instructs LLM to return source references per finding/method
  - build_section_aware_user_message assembles TextExtractionResult sections with truncation
  - LLM_ANNOTATION_SCHEMA updated with source_section/source_snippet in findings and methods items
  - find_highlight_range fuzzy snippet matching in analysis/highlight.rs
  - PaperDetail includes TextExtractionResult extraction field for provenance drawer
  - GraphNode includes bfs_depth for LOD visibility; GraphData includes seed_paper_id

affects:
  - 10-02 (LOD visibility uses bfs_depth from GraphData)
  - 10-03 (provenance drawer reads extraction and source_section/snippet fields from PaperDetail)
  - 10-04 (section-aware LLM pipeline uses build_section_aware_user_message)

tech-stack:
  added: []
  patterns:
    - "Backward-compatible serde evolution: #[serde(default)] on new Option<T> struct fields"
    - "BFS depth from petgraph using VecDeque over NodeIndex with HashMap for visited tracking"
    - "Section-aware LLM messages: push_section helper with MAX_CHARS truncation"
    - "Fuzzy snippet highlight: normalize whitespace + word-overlap sliding window fallback"

key-files:
  created:
    - resyn-core/src/analysis/highlight.rs
  modified:
    - resyn-core/src/datamodels/llm_annotation.rs
    - resyn-core/src/llm/prompt.rs
    - resyn-core/src/analysis/mod.rs
    - resyn-app/src/server_fns/papers.rs
    - resyn-app/src/server_fns/graph.rs
    - resyn-core/src/datamodels/enrichment.rs
    - resyn-core/src/gap_analysis/contradiction.rs
    - resyn-core/src/analysis/aggregation.rs
    - resyn-core/tests/aggregation_tests.rs
    - resyn-core/src/database/queries.rs

key-decisions:
  - "ExtractionRepository (not TextExtractionRepository) is the correct name in queries.rs"
  - "BFS depth computed server-side from petgraph NodeIndex, first node = seed"
  - "#[serde(default)] enables zero-migration backward compat for existing DB records without provenance fields"

requirements-completed: [DEBT-04, AUI-04]

duration: 25min
completed: 2026-03-18
---

# Phase 10 Plan 01: Data Models, LLM Prompt, and DTO Extensions Summary

**Provenance fields on Finding/Method with section-aware LLM prompt, BFS depth on GraphNode, and TextExtractionResult in PaperDetail**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-03-18T15:10:06Z
- **Completed:** 2026-03-18T15:35:00Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments
- Extended `Finding` and `Method` structs with `source_section` and `source_snippet` optional fields using `#[serde(default)]` for zero-migration backward compatibility
- Replaced `SYSTEM_PROMPT` with section-aware version instructing LLM to return source references, updated `LLM_ANNOTATION_SCHEMA` with new properties in findings/methods items
- Created `analysis/highlight.rs` with `find_highlight_range` (exact + whitespace-normalized match, word-overlap sliding window fallback)
- Added `build_section_aware_user_message` with `MAX_CHARS = 3000` truncation for section assembly
- Extended `PaperDetail` with `extraction: Option<TextExtractionResult>` queried via `ExtractionRepository`
- Extended `GraphNode` with `bfs_depth: Option<u32>` and `GraphData` with `seed_paper_id: Option<String>`, computed via BFS on petgraph NodeIndex

## Task Commits

1. **Task 1: Extend data models, LLM prompt, and highlight utility** - `5200465` (feat)
2. **Task 2: Extend PaperDetail and GraphData DTOs** - `34c2b2c` (feat)

## Files Created/Modified
- `resyn-core/src/analysis/highlight.rs` - New: find_highlight_range, normalize_whitespace, 3 unit tests
- `resyn-core/src/datamodels/llm_annotation.rs` - Added source_section/source_snippet to Finding and Method, 3 new tests
- `resyn-core/src/llm/prompt.rs` - Section-aware SYSTEM_PROMPT, updated schema, build_section_aware_user_message, 4 new tests
- `resyn-core/src/analysis/mod.rs` - Added `pub mod highlight`
- `resyn-app/src/server_fns/papers.rs` - PaperDetail.extraction field, ExtractionRepository query in get_paper_detail
- `resyn-app/src/server_fns/graph.rs` - GraphNode.bfs_depth, GraphData.seed_paper_id, BFS computation, updated tests
- `resyn-core/src/datamodels/enrichment.rs` - Fixed Finding struct literals with ..Default::default()
- `resyn-core/src/gap_analysis/contradiction.rs` - Fixed Finding struct literal with ..Default::default()
- `resyn-core/src/analysis/aggregation.rs` - Fixed Finding/Method struct literals
- `resyn-core/tests/aggregation_tests.rs` - Fixed Method/Finding struct literals

## Decisions Made
- `ExtractionRepository` is the correct repository name (not `TextExtractionRepository` as in the plan's interface doc) — checked against queries.rs at execution time
- BFS depth computed at server-side graph construction time using petgraph's `node_indices()` for mapping
- `#[serde(default)]` eliminates need for DB migration — old records with missing provenance fields deserialize cleanly with `None`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed all struct literals that omitted new Optional fields**
- **Found during:** Task 1 (extending Finding/Method structs)
- **Issue:** Adding non-default fields to Finding and Method caused E0063 compile errors in ~10 files with existing struct literals
- **Fix:** Added `..Default::default()` to all existing Finding/Method struct initialization sites across enrichment.rs, aggregation.rs, contradiction.rs, queries.rs, aggregation_tests.rs
- **Files modified:** resyn-core/src/datamodels/enrichment.rs, resyn-core/src/analysis/aggregation.rs, resyn-core/src/gap_analysis/contradiction.rs, resyn-core/src/database/queries.rs, resyn-core/tests/aggregation_tests.rs
- **Verification:** `cargo test -p resyn-core` passes with 75 tests
- **Committed in:** `5200465` (Task 1 commit)

**2. [Rule 3 - Blocking] Corrected repository name from TextExtractionRepository to ExtractionRepository**
- **Found during:** Task 2 (extending PaperDetail)
- **Issue:** Plan's interface doc referenced `TextExtractionRepository` but actual queries.rs struct is `ExtractionRepository`
- **Fix:** Used correct name `ExtractionRepository` in papers.rs import and call site
- **Files modified:** resyn-app/src/server_fns/papers.rs
- **Verification:** `cargo check --workspace` passes
- **Committed in:** `34c2b2c` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 bug/compile-error, 1 blocking naming mismatch)
**Impact on plan:** Both fixes essential to compile. No scope creep.

## Issues Encountered
- graph.rs had already been updated by a previous commit (`417486b`) with `bfs_depth`, `seed_paper_id`, and updated tests before this plan executed. Only the BFS computation logic and papers.rs needed new work.

## Next Phase Readiness
- bfs_depth and seed_paper_id available in GraphData for LOD visibility (10-02)
- source_section/source_snippet ready for provenance drawer UI (10-03)
- build_section_aware_user_message ready for section-aware LLM pipeline integration (10-04)
- All workspace tests green (75 resyn-core + 44 resyn-app lib)

---
*Phase: 10-analysis-ui-polish-scale*
*Completed: 2026-03-18*
