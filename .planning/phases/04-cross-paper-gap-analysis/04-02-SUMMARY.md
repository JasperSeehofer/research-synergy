---
phase: 04-cross-paper-gap-analysis
plan: "02"
subsystem: gap-analysis-algorithms
tags: [gap-analysis, nlp, similarity, contradiction, abc-bridge, petgraph, llm]
dependency_graph:
  requires:
    - GapFinding type (src/datamodels/gap_finding.rs) — from 04-01
    - LlmProvider::verify_gap — from 04-01
    - CONTRADICTION_SYSTEM_PROMPT / ABC_BRIDGE_SYSTEM_PROMPT — from 04-01
    - PaperAnalysis (src/datamodels/analysis.rs) — from 02-01
    - LlmAnnotation (src/datamodels/llm_annotation.rs) — from 03-01
    - create_graph_from_papers (src/data_processing/graph_creation.rs) — existing
  provides:
    - cosine_similarity / shared_high_weight_terms (src/gap_analysis/similarity.rs)
    - find_contradictions (src/gap_analysis/contradiction.rs)
    - find_abc_bridges (src/gap_analysis/abc_bridge.rs)
    - gap_analysis module (src/gap_analysis/mod.rs)
  affects:
    - src/lib.rs (gap_analysis module added)
    - src/main.rs (gap_analysis module declaration added)
tech_stack:
  added:
    - petgraph::algo::dijkstra (already in graph_creation, now used for distance checking)
  patterns:
    - Two-stage + LLM pipeline: structural filter -> semantic filter -> LLM verification
    - Undirected reachability via bidirectional dijkstra on a directed graph
    - Confidence as normalized shared-term count (ABC-bridge) and cosine similarity score (contradiction)
    - Graceful LLM skip: warn! on Err, skip on "NO" response
key_files:
  created:
    - src/gap_analysis/mod.rs
    - src/gap_analysis/similarity.rs
    - src/gap_analysis/contradiction.rs
    - src/gap_analysis/abc_bridge.rs
  modified:
    - src/lib.rs
    - src/main.rs
decisions:
  - Cosine similarity threshold of 0.3 for contradiction stage-1 filter (discretionary per plan)
  - Finding strength divergence: one paper must have strong/established while other has weak/preliminary
  - MIN_SHARED_TERMS = 3 for ABC-bridge filter (discretionary per plan)
  - Undirected reachability via dijkstra(A->C) + dijkstra(C->A) both directions
  - Removed pub use re-exports from mod.rs — clippy -D warnings rejects unused imports in bin target
metrics:
  duration: "8 minutes"
  completed: "2026-03-14"
  tasks_completed: 2
  files_created: 4
  files_modified: 2
---

# Phase 4 Plan 2: Gap Analysis Algorithms Summary

**One-liner:** Cosine similarity for TF-IDF vectors, two-stage contradiction detector (similarity + finding divergence + LLM), and ABC-bridge discoverer with bidirectional graph distance checking and shared-term threshold.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Similarity module + contradiction detector | 2c852e6 | src/gap_analysis/similarity.rs, src/gap_analysis/contradiction.rs, src/gap_analysis/mod.rs |
| 2 | ABC-bridge discoverer with graph distance | 9d9f49c | src/gap_analysis/abc_bridge.rs |

## What Was Built

### Task 1: Similarity Module + Contradiction Detector

**`src/gap_analysis/similarity.rs`** — Two utility functions:
- `cosine_similarity(a, b)` — computes dot product over the shorter vector's keys, divided by both magnitudes. Returns 0.0 for empty/zero-magnitude vectors.
- `shared_high_weight_terms(a, b, min_weight)` — terms present in both vectors with weight >= min_weight in both; sorted alphabetically for deterministic output.

**`src/gap_analysis/contradiction.rs`** — `find_contradictions` three-stage pipeline:
1. **TF-IDF filter**: cosine similarity >= 0.3 threshold eliminates off-topic pairs
2. **Finding strength divergence**: one paper must have "strong/established" findings while the other has "weak/preliminary" findings (or vice versa)
3. **LLM verification**: calls `provider.verify_gap(CONTRADICTION_SYSTEM_PROMPT, context)` where context includes both papers' findings; skips on "NO" response; warns and skips on LLM error
- Confidence = cosine similarity value (higher overlap = higher confidence same topic is discussed)
- Shared terms extracted via `shared_high_weight_terms(a, b, 0.1)`

### Task 2: ABC-Bridge Discoverer

**`src/gap_analysis/abc_bridge.rs`** — `find_abc_bridges` with graph distance awareness:
- Builds `arxiv_id -> NodeIndex` lookup from the petgraph `StableGraph`
- `has_direct_edge` checks both A->C and C->A directions
- `graph_distance` runs dijkstra forward (A->C) and backward (C->A), returns minimum found distance for undirected reachability on a directed graph
- Filters: no direct edge, graph distance > 1 (belt-and-suspenders), >= 3 shared high-weight terms
- `full_corpus` flag: when false, skips pairs where neither paper is in the citation graph
- LLM verification with `ABC_BRIDGE_SYSTEM_PROMPT`; graceful skip on error or "NO"
- Confidence = min(shared_terms.len() / 10.0, 1.0)

## Test Results

- `cargo test gap_analysis` — 20 passed (7 similarity, 6 contradiction, 7 abc_bridge)
- `cargo test` (full suite) — 127 passed, 0 failed
- `cargo check` — clean
- `cargo clippy -- -D warnings` — clean

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Rust 2024 pattern matching syntax in shared_high_weight_terms**

- **Found during:** Task 1 compilation
- **Issue:** `&val` pattern in `.filter(|(key, &val)| ...)` caused "reference pattern not allowed when implicitly borrowing" in Rust edition 2024
- **Fix:** Changed to `**val` dereference syntax and used `.is_some_and()` per clippy suggestion
- **Files modified:** `src/gap_analysis/similarity.rs`
- **Commit:** 9d9f49c

**2. [Rule 3 - Blocking] Removed unused pub use re-exports from mod.rs**

- **Found during:** Task 2 `cargo clippy -- -D warnings`
- **Issue:** `pub use` re-exports in `mod.rs` caused "unused import" errors in the bin target (which doesn't call these functions yet). CI enforces `-D warnings`.
- **Fix:** Removed re-exports; callers use module paths directly (e.g. `gap_analysis::contradiction::find_contradictions`)
- **Files modified:** `src/gap_analysis/mod.rs`
- **Commit:** 9d9f49c

## Self-Check: PASSED
