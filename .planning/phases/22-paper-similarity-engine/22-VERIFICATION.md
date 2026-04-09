---
phase: 22-paper-similarity-engine
verified: 2026-04-08T00:00:00Z
status: human_needed
score: 13/14 must-haves verified
overrides_applied: 0
gaps:
  - truth: "Similarity recomputation is skipped when corpus fingerprint is unchanged"
    status: partial
    reason: "CR-01 (from 22-REVIEW.md): fingerprint guard in analysis.rs:185-192 uses analyses[0].arxiv_id heuristic — checks only the first paper's stored record to decide whether to skip the entire corpus recompute. If the first paper has an up-to-date record but others do not (e.g., after a partial run or incremental paper addition), the entire stage is skipped, leaving those papers without similarity data. The NLP stage uses a canonical metadata key; this stage should follow the same pattern."
    artifacts:
      - path: "resyn-app/src/server_fns/analysis.rs"
        issue: "Lines 183-192: guard checks analyses[0].arxiv_id record instead of a canonical corpus_similarity metadata key. Should use analysis_repo.get_metadata(\"corpus_similarity\") following the NLP stage pattern."
    missing:
      - "Replace per-paper index-0 guard with canonical metadata key (e.g., get_metadata(\"corpus_similarity\")), write metadata after successful upsert batch, following the existing AnalysisMetadata pattern used by the NLP stage"
human_verification:
  - test: "Similarity edge overlay — visual rendering"
    expected: "Dashed amber lines appear between similar papers when 'Similarity' toggle is enabled. Lines are distinctly colored from gray citation edges, and thicker lines connect more similar papers (score-proportional thickness 1.5–4.0px)."
    why_human: "Canvas2D setLineDash rendering and color comparison between edge types requires visual inspection"
  - test: "Similarity edge overlay — toggle independence"
    expected: "Clicking 'Citations' OFF hides citation edges while similarity edges remain. Clicking 'Similarity' OFF hides similarity edges while citations remain. Both can be shown simultaneously."
    why_human: "Signal wiring verification requires live interaction"
  - test: "Force model swap animation"
    expected: "Switching Layout from 'Citation' to 'Similarity' triggers a visible re-animation as nodes rearrange to cluster similar papers. Switching back to 'Citation' re-animates to citation topology. Alpha reheat at 0.5 produces a medium-intensity animation, not a full reset."
    why_human: "Animation quality and alpha magnitude require visual inspection"
  - test: "Similar tab — ranked list display"
    expected: "Opening a paper drawer and clicking 'Similar' shows a ranked list with: similarity percentage badge, paper title, authors (et al. truncation for >2), publication year, and 2-3 shared keywords. Papers are ordered by descending similarity score."
    why_human: "UI layout, et al. truncation behavior, and ordering require visual verification"
  - test: "Similar tab — click navigation"
    expected: "Clicking a paper in the Similar tab closes the current drawer view and opens the clicked paper's drawer at the Overview tab."
    why_human: "SelectedPaper context signal dispatch and resulting UI transition require live interaction"
  - test: "Similar tab — TF-IDF waiting state"
    expected: "For a paper with no computed similarity data, the Similar tab shows a spinner and the text 'Waiting for TF-IDF analysis...'"
    why_human: "Pre-analysis state requires testing with no computed data in DB"
---

# Phase 22: Paper Similarity Engine — Verification Report

**Phase Goal:** Users can see which papers are most similar to any given paper, with similarity edges optionally shown on the graph
**Verified:** 2026-04-08
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | System stores top-10 most similar papers per paper in SurrealDB, computed from TF-IDF cosine similarity | VERIFIED | `compute_top_neighbors` in gap_analysis/similarity.rs:38, `SimilarityRepository.upsert_similarity` in queries.rs:865, migration 10 creates paper_similarity table (schema.rs:221) |
| 2 | User can open the paper detail drawer and view a "Similar Papers" tab | VERIFIED | `DrawerTab::Similar` in app.rs:20, `SimilarTabBody` in drawer.rs:297, tab button at drawer.rs:192-197 |
| 3 | User can toggle a similarity edge overlay on the graph | VERIFIED | `show_similarity: RwSignal<bool>` in graph_controls.rs:8, toggle button at graph_controls.rs:41-43, EdgeType::Similarity rendered in canvas_renderer.rs:183-210 |
| 4 | After TF-IDF analysis completes, similarity scores are automatically recomputed | VERIFIED | Stage 2.5 block in analysis.rs:174-202 calls compute_top_neighbors(analyses, 10) after NLP stage, before LLM/gap stages |
| 5 | compute_top_neighbors returns top-10 neighbors sorted by descending cosine similarity score | VERIFIED | 15 unit tests pass (test_compute_top_neighbors_sorted_descending, test_compute_top_neighbors_truncates_to_k, etc.) |
| 6 | PaperSimilarity records persist to and load from SurrealDB paper_similarity table | VERIFIED | SimilarityRepository roundtrip DB tests pass; neighbors serialized as JSON string to avoid SurrealDB FLEXIBLE array pitfall |
| 7 | Similarity is automatically recomputed after TF-IDF stage completes with a new corpus fingerprint | VERIFIED | Stage 2.5 fires after NLP completes; fingerprint sourced from analyses[0].corpus_fingerprint |
| 8 | Similarity recomputation is skipped when corpus fingerprint is unchanged | PARTIAL | CR-01 (22-REVIEW.md): guard checks analyses[0].arxiv_id record only — can incorrectly skip when other papers lack stored records, or incorrectly compute when first paper is already current |
| 9 | User can see a "Similar" tab alongside Overview and Source in paper drawer | VERIFIED | DrawerTab enum has Overview, Source, Similar variants; tab strip in drawer.rs shows all three |
| 10 | Similar tab shows ranked list with similarity %, title, authors, year, 2-3 shared keywords | VERIFIED (code) | SimilarPaperEntry struct (similarity.rs:6-15), score_pct formatted as "%.0%", authors truncated with et al., shared_terms joined — human visual verification needed |
| 11 | Clicking a similar paper opens its detail drawer | VERIFIED (code) | drawer.rs:340 calls sel.set(Some(DrawerOpenRequest { paper_id: id, initial_tab: DrawerTab::Overview, ... })) — human interaction verification needed |
| 12 | When TF-IDF not computed, Similar tab shows spinner with "Waiting for TF-IDF analysis..." | VERIFIED (code) | drawer.rs:316 contains literal "Waiting for TF-IDF analysis..." in empty-state path — human state verification needed |
| 13 | User can toggle similarity edges independently from citation edges | VERIFIED (code) | show_similarity and show_citations are independent RwSignal<bool> props in graph_controls.rs:8-9 — toggle independence needs human verification |
| 14 | Similarity edges render as dashed amber lines with thickness proportional to score | VERIFIED (code) | canvas_renderer.rs:197 "#f0a030", set_line_dash [8.0, 5.0] at line 204-206, thickness 1.5 + score * 2.5 — visual verification needed |

**Score:** 13/14 truths verified (1 partial — CR-01 fingerprint guard bug)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `resyn-core/src/datamodels/similarity.rs` | PaperSimilarity and SimilarNeighbor structs | VERIFIED | pub struct PaperSimilarity at line 14, pub struct SimilarNeighbor at line 5 |
| `resyn-core/src/gap_analysis/similarity.rs` | compute_top_neighbors function | VERIFIED | pub fn compute_top_neighbors at line 38 |
| `resyn-core/src/database/schema.rs` | Migration 10 for paper_similarity table | VERIFIED | apply_migration_10 at line 218, called at line 294 |
| `resyn-core/src/database/queries.rs` | SimilarityRepository with upsert/get methods | VERIFIED | pub struct SimilarityRepository at line 856; upsert_similarity, get_similarity, get_all_similarities all present |
| `resyn-app/src/app.rs` | DrawerTab::Similar variant | VERIFIED | DrawerTab enum contains Similar at line 20 |
| `resyn-app/src/server_fns/similarity.rs` | get_similar_papers server fn | VERIFIED | pub async fn get_similar_papers at line 19; SimilarPaperEntry struct at line 6 |
| `resyn-app/src/server_fns/mod.rs` | pub mod similarity registration | VERIFIED | line 8: pub mod similarity |
| `resyn-app/src/layout/drawer.rs` | SimilarTabBody component | VERIFIED | fn SimilarTabBody at line 297 |
| `resyn-app/src/server_fns/graph.rs` | EdgeType::Similarity variant | VERIFIED | EdgeType::Similarity present; SimilarityRepository queried at line 210; threshold 0.15 at line 215 |
| `resyn-app/src/graph/layout_state.rs` | show_similarity, show_citations, force_mode in GraphState | VERIFIED | Fields at lines 69-71; ForceMode enum at line 4 |
| `resyn-app/src/graph/canvas_renderer.rs` | Dashed amber similarity edge rendering | VERIFIED | #f0a030 at line 197; setLineDash [8.0, 5.0] at lines 204-206; if state.show_similarity at line 183 |
| `resyn-app/src/components/graph_controls.rs` | Similarity toggle and force mode selector | VERIFIED | show_similarity prop at line 8; force-mode-selector div at line 67; ForceMode::Citation/Similarity buttons |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| resyn-app/src/server_fns/analysis.rs | resyn-core::gap_analysis::similarity::compute_top_neighbors | call after NLP stage | WIRED | Stage 2.5 at analysis.rs:177, 193 |
| resyn-core/src/database/queries.rs | paper_similarity table | SimilarityRepository upsert/get | WIRED | queries.rs:865+ uses UPSERT into paper_similarity table |
| resyn-app/src/layout/drawer.rs | resyn-app/src/server_fns/similarity.rs | Resource calling get_similar_papers | WIRED | drawer.rs:8 imports get_similar_papers; drawer.rs:299 calls it via Resource::new |
| resyn-app/src/layout/drawer.rs | SelectedPaper context | sel.set on click to navigate to similar paper | WIRED | drawer.rs:340 calls sel.set(Some(DrawerOpenRequest { ... })) |
| resyn-app/src/components/graph_controls.rs | resyn-app/src/graph/layout_state.rs | show_similarity and force_mode signals | WIRED | graph_controls.rs:8-10 props; graph.rs:228, 562 sync to GraphState |
| resyn-app/src/pages/graph.rs | resyn-app/src/graph/layout_state.rs | force_mode change triggers alpha reheat | WIRED | graph.rs:554-558: current_force_mode != prev sets alpha = 0.5, simulation_running = true |
| resyn-app/src/server_fns/graph.rs | SimilarityRepository | get_graph_data loads similarity edges | WIRED | graph.rs:210-233: SimilarityRepository queried; EdgeType::Similarity emitted |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| SimilarTabBody | similar_resource (Vec<SimilarPaperEntry>) | get_similar_papers -> SimilarityRepository.get_similarity -> PaperRepository.get_paper | Yes — reads from paper_similarity table populated by analysis pipeline | FLOWING |
| get_graph_data similarity edges | edges (Vec<GraphEdge>) | SimilarityRepository.get_all_similarities -> paper_similarity table | Yes — DB query returns real stored data with 0.15 threshold | FLOWING |
| compute_top_neighbors output | Vec<PaperSimilarity> | PaperAnalysis.tfidf_vector (all analyses from DB) | Yes — pairwise cosine similarity on real TF-IDF vectors | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| similarity unit tests pass | cargo test -p resyn-core --lib similarity | 15 passed, 0 failed | PASS |
| full workspace compiles | cargo check --all-targets | Finished dev profile, 0 errors | PASS |
| EdgeType::Similarity exhaustively handled in webgl_renderer.rs | grep EdgeType::Similarity webgl_renderer.rs | Lines 252 (visible guard), 621 (color) | PASS |
| ForceMode::Similarity in build_layout_input edge selection | grep -n ForceMode::Similarity pages/graph.rs | Line 424: filter Similarity edges for simulation | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| SIM-01 | 22-01-PLAN.md | System computes pairwise cosine similarity from TF-IDF vectors, stores top-10 neighbors per paper | SATISFIED | compute_top_neighbors + SimilarityRepository + paper_similarity table, 15 tests |
| SIM-02 | 22-02-PLAN.md | User can view similar papers in a "Similar Papers" tab in the paper detail drawer | SATISFIED | DrawerTab::Similar, SimilarTabBody, get_similar_papers server fn — human visual verification pending |
| SIM-03 | 22-03-PLAN.md | User can toggle similarity edges as an overlay in the graph view | SATISFIED | show_similarity toggle, EdgeType::Similarity rendering, force mode selector — human visual verification pending |
| SIM-04 | 22-01-PLAN.md | Similarity recomputed automatically after TF-IDF analysis completes | PARTIAL | Stage 2.5 fires; fingerprint guard has CR-01 index-0 heuristic bug (see gaps) |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| resyn-app/src/server_fns/analysis.rs | 185-192 | Fingerprint guard checks analyses[0].arxiv_id record instead of canonical metadata key | Warning | Can incorrectly skip similarity recomputation for papers added after first paper's record was stored; CR-01 from 22-REVIEW.md |
| resyn-app/src/server_fns/similarity.rs | 44-62 | N+1 query: up to 10 sequential get_paper() calls per request | Info | Acceptable for embedded local DB at current scale; WR-02 from 22-REVIEW.md |
| resyn-app/src/graph/webgl_renderer.rs | (hex_to_rgb) | No bounds check before slice indexing on hex color string | Info | All current call-sites use valid 6-digit literals; latent panic if caller changes; WR-01 from 22-REVIEW.md |

### Human Verification Required

#### 1. Similarity Edge Visual Rendering

**Test:** Enable the Similarity toggle in graph controls on a graph with analyzed papers.
**Expected:** Dashed amber (#f0a030) lines appear between similar papers. Lines are visually distinct from gray citation edges. Thicker lines connect papers with higher similarity scores (thickness scales between 1.5px and 4.0px).
**Why human:** Canvas2D setLineDash rendering and amber vs. gray color distinction require visual inspection.

#### 2. Edge Toggle Independence

**Test:** With both edge types visible, click "Citations" OFF, then click it back ON. Repeat with "Similarity" toggle.
**Expected:** Each toggle independently shows/hides its edge type without affecting the other. With both ON simultaneously, the graph shows both edge layers.
**Why human:** Signal independence requires live UI interaction to verify.

#### 3. Force Model Swap Animation

**Test:** Click "Layout: Similarity" button in graph controls, then "Layout: Citation".
**Expected:** Each switch triggers a visible re-animation (nodes visibly rearrange). Similarity layout clusters similar papers together. Citation layout positions papers by citation topology. Animation is medium-intensity (alpha 0.5, not full reset).
**Why human:** Animation behavior and layout quality require visual inspection.

#### 4. Similar Tab — Ranked List

**Test:** Open a paper drawer for a paper with computed similarity data. Click the "Similar" tab.
**Expected:** Ranked list shows entries ordered by descending similarity percentage. Each entry shows: bold score badge (e.g., "87%"), paper title, authors (first author + "et al." if >2 authors), year, and shared keywords (labeled "Shared:").
**Why human:** Ranking order, et al. truncation, and visual layout require human inspection.

#### 5. Similar Tab — Click Navigation

**Test:** Click a paper in the Similar tab list.
**Expected:** The drawer navigates to show the clicked paper's Overview tab (drawer content changes to the selected paper's details).
**Why human:** SelectedPaper context dispatch and resulting drawer transition require live interaction.

#### 6. Similar Tab — Waiting State

**Test:** Open a paper drawer for a paper that has NOT been analyzed (no TF-IDF vectors computed). Click the "Similar" tab.
**Expected:** A spinner appears alongside the text "Waiting for TF-IDF analysis..." and hint text about running analysis.
**Why human:** Pre-analysis state requires testing against a DB with no similarity data for the selected paper.

### Gaps Summary

**1 gap (partial):** The fingerprint guard for similarity recomputation (analysis.rs Stage 2.5, lines 183-192) uses an index-0 heuristic instead of a canonical metadata key. It checks whether analyses[0] (the first paper in the returned list) already has a stored similarity record matching the current fingerprint. If that condition is true, the entire stage is skipped — regardless of whether all other papers have stored records. This means:

- After a partial run (where computation was interrupted), adding new papers will not trigger recomputation if the first paper alphabetically happens to have a valid stored record.
- The NLP stage (Stage 2) correctly uses `get_metadata("nlp_complete")` as a canonical guard key; similarity should follow the same pattern using `get_metadata("corpus_similarity")`.

The core functionality (computing and storing similarity when needed) works correctly. The gap affects the cache invalidation edge case defined in Plan 22-01's truth: "Similarity recomputation is skipped when corpus fingerprint is unchanged."

**Fix:** Replace the index-0 guard with a canonical metadata read (`analysis_repo.get_metadata("corpus_similarity")`), and write an `AnalysisMetadata` record with key `"corpus_similarity"` after the upsert batch completes, matching the NLP stage pattern exactly.

---

_Verified: 2026-04-08_
_Verifier: Claude (gsd-verifier)_
