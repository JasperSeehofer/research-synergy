---
phase: 24-community-detection
plan: "01"
subsystem: resyn-core/community + resyn-app/server_fns
tags: [community-detection, louvain, ctfidf, surrealdb, server-fns, leptos]
dependency_graph:
  requires: []
  provides: [COMM-01, COMM-03]
  affects:
    - resyn-core/Cargo.toml
    - resyn-core/src/datamodels/community.rs
    - resyn-core/src/datamodels/mod.rs
    - resyn-core/src/graph_analytics/community.rs
    - resyn-core/src/graph_analytics/mod.rs
    - resyn-core/src/database/schema.rs
    - resyn-core/src/database/queries.rs
    - resyn-app/src/server_fns/community.rs
    - resyn-app/src/server_fns/mod.rs
tech_stack:
  added:
    - "single-clustering 0.6.1 (BSD-3-Clause) — Louvain community detection"
  patterns:
    - "SSR feature gate on single-clustering (WASM/rayon incompatibility)"
    - "DB-sourced citation edges via get_all_citation_edges() — papers loaded from DB have empty references field"
    - "corpus_fingerprint invalidation for stale community rows"
    - "Other bucket sentinel: community_id = u32::MAX, color_index = u32::MAX"
key_files:
  created:
    - resyn-core/src/datamodels/community.rs
    - resyn-app/src/server_fns/community.rs
  modified:
    - resyn-core/Cargo.toml
    - resyn-core/src/datamodels/mod.rs
    - resyn-core/src/graph_analytics/community.rs
    - resyn-core/src/graph_analytics/mod.rs
    - resyn-core/src/database/schema.rs
    - resyn-core/src/database/queries.rs
    - resyn-app/src/server_fns/mod.rs
decisions:
  - "single-clustering placed behind #[cfg(feature = \"ssr\")] because it pulls in rayon which is incompatible with WASM targets"
  - "Fixed RNG seed = 42 for deterministic Louvain partitions across server restarts"
  - "Communities with fewer than 3 papers bucketed into Other (community_id = u32::MAX, color_index = u32::MAX sentinel)"
  - "Citation edges loaded from DB via get_all_citation_edges() instead of paper.get_arxiv_references_ids() — papers fetched by get_all_papers() have empty references field"
  - "Auto-label uses top 1-2 c-TF-IDF keywords (never 'Community N' numeric labels) per COMM-01 requirement"
  - "trigger_community_detection is out of scope for Plan 01 — deferred to Plan 03 (auto-compute wiring)"
  - "single-clustering license is BSD-3-Clause (not MIT as initially assumed — confirmed from crate LICENSE.md)"
metrics:
  completed_date: "2026-04-11"
  tasks_completed: 3
  files_modified: 9
---

# Phase 24 Plan 01: Community Detection Backend

Deliver the compute and persistence layer for Louvain community detection: data models, SurrealDB migration 12 + CommunityRepository, the full Louvain + c-TF-IDF + hybrid-ranking pipeline in `resyn-core`, and the read-path server functions that Plans 02 and 03 consume.

## What Was Built

### Task 1: Data models, migration 12, CommunityRepository

**resyn-core/src/datamodels/community.rs** (new):
- `CommunityAssignment` — per-paper record: `arxiv_id`, `community_id`, `color_index`, `corpus_fingerprint`
- `CommunitySummary` — per-community summary: `community_id`, `label`, `color_index`, `paper_count`, `top_papers` (Vec<CommunityTopPaper>), `keywords`, `shared_methods`
- `CommunityTopPaper` — ranked paper: `arxiv_id`, `title`, `hybrid_score`, `pagerank`, `intra_degree`
- `CommunityStatus` — aggregate status: `community_count`, `paper_count`, `fingerprint`, `computed_at`
- `OTHER_COMMUNITY_ID = u32::MAX`, `OTHER_COLOR_INDEX = u32::MAX` sentinel constants

**resyn-core/src/database/schema.rs:**
- `apply_migration_12` creates `graph_communities` SCHEMAFULL table with UNIQUE index on `arxiv_id`
- Schema updated in `initialize_schema` to call migration 12

**resyn-core/src/database/queries.rs:**
- `CommunityRepository` with: `upsert_assignment`, `get_by_paper`, `list_all`, `get_by_community_id`, `get_all_for_fingerprint`, `delete_stale`
- `PaperRepository::get_all_citation_edges()` — new method returning `Vec<(String, String)>` of (from_id, to_id) citation pairs from SurrealDB `cites` relation edges

**resyn-core/Cargo.toml:**
- Added `single-clustering = { version = "0.6.1", optional = true }` with `ssr` feature gate

### Task 2: Louvain runner, c-TF-IDF, hybrid ranking

**resyn-core/src/graph_analytics/community.rs** (full implementation):

- `detect_communities(graph)` — builds undirected projection, runs Louvain with fixed seed=42 and resolution=1.0, assigns Other bucket to communities < 3 papers, ranks remaining communities by size descending to assign stable `color_index` values
- `compute_ctfidf(partition, tfidf_map)` — class-based TF-IDF: community-level term frequency × log((num_communities + 1) / (df + 1)), returns top-N terms per community; skips Other bucket
- `hybrid_score(pagerank, intra_degree)` — `pagerank * (intra_degree as f32 + 1.0)` per D-19
- `build_top_papers(assignments, pagerank_map, intra_degree_map, paper_map)` — top 5 papers per community ranked by hybrid score per D-20
- `build_community_label(keywords)` — top 1-2 c-TF-IDF keywords joined with " / " (never numeric)
- `build_graph_from_edges(papers, edges)` — builds `StableGraph` from DB-loaded papers and explicit citation edges (SSR-gated helper)
- `compute_and_store_communities(db)` — full orchestrator: load papers + citation edges + analyses + PageRank, run Louvain, bucket Other, upsert assignments with corpus-fingerprint invalidation
- `compute_community_summaries(db)` — assembles `CommunitySummary` per community from cached assignments, TF-IDF analyses, PageRank scores, and shared methods
- `load_community_status(db)` — returns `CommunityStatus` from current assignments (community_count, paper_count, fingerprint, computed_at)

### Task 3: Read-path server functions

**resyn-app/src/server_fns/community.rs** (new):
- `get_community_status()` — calls `load_community_status`; read-only, no compute triggered
- `get_all_community_summaries()` — calls `compute_community_summaries`; assembles on-demand from cached rows
- `get_community_summary(community_id)` — filters `get_all_community_summaries` for a single community
- `get_community_for_paper(arxiv_id)` — queries `CommunityRepository::get_by_paper`
- `get_community_assignments()` — returns all `CommunityAssignment` rows for the current corpus fingerprint

All server fns are SSR-feature-gated and use the shared `Arc<Db>` from Leptos context.

**resyn-app/src/server_fns/mod.rs:**
- Added `pub mod community;`

## Deviations from Plan

### Bug Fix: DB-sourced citation edges

**Found during:** Task 3 integration — `compute_and_store_communities` built the graph with `create_graph_from_papers` which reads `paper.references`, but papers loaded from DB via `get_all_papers()` have an empty `references` field (only populated during crawl).

**Fix:** Added `PaperRepository::get_all_citation_edges()` to load citation pairs directly from the `cites` relation in SurrealDB. Replaced `create_graph_from_papers` with a new `build_graph_from_edges` helper in community.rs. Applied the same fix to the intra-degree computation in `compute_community_summaries`. Updated the end-to-end orchestrator test to call `upsert_citations` so edges are present in the DB.

**Files modified:** resyn-core/src/database/queries.rs, resyn-core/src/graph_analytics/community.rs

### License correction: BSD-3-Clause (not MIT)

The `single-clustering 0.6.1` crate is licensed under **BSD-3-Clause**, not MIT. Confirmed from `~/.cargo/registry/src/.../single-clustering-0.6.1/LICENSE.md`.

### trigger_community_detection not in this plan

The commit message instructed by the orchestrator mentioned `trigger_community_detection`, but this server fn is Plan 03's responsibility (auto-compute wiring). It is intentionally absent from Plan 01.

## Tests Added

16 tests in `resyn-core/src/graph_analytics/community.rs`:

**Pure-function unit tests (10):**
- `test_community_detection_deterministic` — same seed produces same partition
- `test_community_detection_too_small` — < 3 nodes all go to Other
- `test_community_other_bucket` — sub-threshold communities bucketed correctly
- `test_community_color_index_size_rank` — largest community gets color_index 0
- `test_ctfidf_distinctive_terms` — community-distinctive terms ranked above shared terms
- `test_ctfidf_returns_top_n` — top-N truncation
- `test_ctfidf_skips_other_bucket` — Other community excluded from c-TF-IDF
- `test_hybrid_score_formula` — pagerank × (intra_degree + 1) formula
- `test_hybrid_ranking_selects_top_5` — top 5 by hybrid score
- `test_auto_label_from_ctfidf` — label uses top 1-2 keywords

**DB integration tests (6, tokio::test):**
- `test_compute_and_store_communities_end_to_end` — full orchestrator on 16-paper DB
- `test_compute_and_store_communities_replaces_stale` — fingerprint invalidation replaces old rows
- `test_compute_and_store_communities_small_graph_no_op` — < 3 papers returns Skipped
- `test_compute_community_summaries_end_to_end` — summaries assembled from cached assignments
- `test_load_community_status_states` — empty → Some(status) state transitions
- `test_load_community_for_paper` — per-paper lookup via CommunityRepository

**Result:** `cargo test -p resyn-core --lib` → 110 passed, 0 failed.

Note: `cargo test -p resyn-core` (including integration tests) fails on `tests/arxiv_text_extraction.rs` — pre-existing issue requiring `ssr` feature, unrelated to Plan 01.

## Commits

| Hash | Message |
|------|---------|
| 341d7a7 | feat(24-01): add community data models, migration 12, CommunityRepository, stub community module |
| 6dd61d9 | feat(24-01): implement Louvain runner, c-TF-IDF, hybrid ranking in community.rs |
| 274a48f | feat(24-01): add server fns for community detection (get_community_status, get_all_community_summaries, get_community_summary, get_community_for_paper, get_community_assignments) |

## Self-Check: PASSED

- resyn-core/src/datamodels/community.rs: FOUND (contains CommunitySummary, CommunityAssignment, CommunityTopPaper, CommunityStatus, OTHER_COLOR_INDEX)
- resyn-core/src/database/schema.rs: FOUND (contains apply_migration_12)
- resyn-core/src/database/queries.rs: FOUND (contains CommunityRepository, get_all_citation_edges)
- resyn-core/src/graph_analytics/community.rs: FOUND (contains detect_communities, compute_ctfidf, hybrid_score, compute_community_summaries, compute_and_store_communities)
- resyn-app/src/server_fns/community.rs: FOUND (contains get_all_community_summaries, get_community_status, get_community_for_paper, get_community_assignments)
- single-clustering behind ssr feature gate: CONFIRMED
- Fixed seed=42: CONFIRMED (LOUVAIN_SEED constant)
- Other bucket for < 3 papers: CONFIRMED (MIN_COMMUNITY_SIZE = 3)
- DB citation edges fix: CONFIRMED (get_all_citation_edges + build_graph_from_edges)
- cargo check --all-targets: PASSED (0 errors, 0 warnings in resyn-app)
- cargo test -p resyn-core --lib: 110/110 PASSED
