---
phase: 22-paper-similarity-engine
plan: 02
subsystem: similarity-ui
tags: [similarity, leptos, drawer, server-fn, ui]
dependency_graph:
  requires: [resyn-core/src/datamodels/similarity.rs, resyn-core/src/database/queries.rs (SimilarityRepository), resyn-app/src/app.rs (DrawerTab)]
  provides: [DrawerTab::Similar, get_similar_papers server fn, SimilarTabBody component]
  affects: [resyn-app/src/app.rs, resyn-app/src/layout/drawer.rs, resyn-app/src/server_fns/mod.rs]
tech_stack:
  added: []
  patterns: [Leptos Resource for async server fn, Suspense fallback pattern, match on DrawerTab for tab body dispatch]
key_files:
  created:
    - resyn-app/src/server_fns/similarity.rs
  modified:
    - resyn-app/src/app.rs
    - resyn-app/src/layout/drawer.rs
    - resyn-app/src/server_fns/mod.rs
    - resyn-app/style/main.css
decisions:
  - Removed SimilarPaperEntry unused import from drawer.rs — type inferred through Resource result, no explicit annotation needed
  - Switched tab body from if/else chain to match on DrawerTab — cleaner exhaustive pattern as enum grows
  - Used paper.published[..4] for year extraction matching existing PapersPanel pattern
metrics:
  duration: ~20 minutes
  completed: 2026-04-09
  tasks_completed: 1
  files_modified: 5
  tests_added: 0
requirements: [SIM-02]
---

# Phase 22 Plan 02: Similar Papers Drawer Tab Summary

**One-liner:** Similar tab in paper detail drawer showing ranked cosine-similarity list with score %, metadata, shared keywords, and TF-IDF waiting spinner when not yet computed.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | DrawerTab::Similar, get_similar_papers server fn, and SimilarTabBody component | f947d9a | app.rs, server_fns/similarity.rs, server_fns/mod.rs, layout/drawer.rs, style/main.css |

## What Was Built

### DrawerTab enum (`resyn-app/src/app.rs`)

Added `Similar` as a third variant alongside `Overview` and `Source`. Existing `DrawerOpenRequest` struct accepts `Similar` as `initial_tab` with no changes needed.

### Server Function (`resyn-app/src/server_fns/similarity.rs`)

- `SimilarPaperEntry` struct: `arxiv_id`, `title`, `authors: Vec<String>`, `year`, `score: f32`, `shared_terms: Vec<String>`
- `get_similar_papers(arxiv_id: String) -> Result<Vec<SimilarPaperEntry>, ServerFnError>`
  - Applies `strip_version_suffix()` before DB query (T-22-03 mitigation)
  - Returns empty vec when no similarity data exists (triggers D-08 waiting state in UI)
  - Enriches each neighbor from `SimilarityRepository` with paper metadata from `PaperRepository`
  - Extracts year as first 4 chars of `paper.published`

### Drawer UI (`resyn-app/src/layout/drawer.rs`)

- Added "Similar" tab button to the tab strip (matches existing `drawer-tab` / `drawer-tab active` pattern)
- Refactored tab body dispatch from `if/else` to `match` on `DrawerTab` — exhaustive, cleaner
- `SimilarTabBody` component:
  - Uses `Resource::new` + `Suspense` for async loading
  - **Empty state (D-08):** spinner + "Waiting for TF-IDF analysis..." + hint text
  - **Populated state (D-01):** ranked list of `similar-paper-item` cards with score badge, title, authors (et al. truncation for >2), year, shared terms
  - **Click (D-02):** sets `SelectedPaper` context to open the clicked paper's drawer at `DrawerTab::Overview`
  - **Error state:** plain muted text fallback

### CSS (`resyn-app/style/main.css`)

New "Similar Papers Tab" section with:
- `.similar-waiting-state` — centered flex column with spinner
- `.similar-papers-list` — vertical stack
- `.similar-paper-item` — clickable card with hover background bleed, border-bottom separator
- `.similar-paper-header` — flex row: accent-colored score badge + title
- `.similar-paper-meta` — label-size authors + year row
- `.similar-shared-terms` — italic shared terms with semibold "Shared:" label

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused `SimilarPaperEntry` import from drawer.rs**
- **Found during:** Task 1 — `cargo check` reported unused import warning
- **Issue:** Import was included per plan spec but type is inferred through `Resource` result; no explicit annotation needed in drawer component
- **Fix:** Removed `SimilarPaperEntry` from the import line, keeping only `get_similar_papers`
- **Files modified:** `resyn-app/src/layout/drawer.rs`
- **Commit:** f947d9a

**2. [Rule 2 - Missing critical functionality] Replaced if/else tab dispatch with exhaustive match**
- **Found during:** Task 1 — adding a third `DrawerTab` variant to a non-exhaustive `if/else` chain is a correctness hazard
- **Issue:** The original `if active_tab.get() == DrawerTab::Overview { ... } else { ... }` would silently render the Source tab body for any unknown tab variant
- **Fix:** Changed to `match active_tab.get() { DrawerTab::Overview => ..., DrawerTab::Source => ..., DrawerTab::Similar => ... }` — compiler enforces exhaustiveness
- **Files modified:** `resyn-app/src/layout/drawer.rs`
- **Commit:** f947d9a

## Known Stubs

None — the Similar tab is fully wired: `get_similar_papers` reads from the `paper_similarity` table populated by Plan 01's pipeline, enriches with paper metadata from `PaperRepository`, and returns real scored entries. The waiting spinner is intentional UX for the pre-analysis state, not a stub.

## Threat Flags

No new security surface beyond the plan's threat model. T-22-03 mitigated: `strip_version_suffix()` applied to `arxiv_id` before DB query in `get_similar_papers`.

## Self-Check: PASSED

- resyn-app/src/app.rs (DrawerTab::Similar): FOUND
- resyn-app/src/server_fns/similarity.rs (get_similar_papers, SimilarPaperEntry): FOUND
- resyn-app/src/server_fns/mod.rs (pub mod similarity): FOUND
- resyn-app/src/layout/drawer.rs (SimilarTabBody, "Waiting for TF-IDF analysis...", DrawerTab::Similar): FOUND
- resyn-app/style/main.css (.similar-waiting-state, .similar-paper-item): FOUND
- Commit f947d9a: FOUND
- cargo check --all-targets: clean (0 warnings, 0 errors)
- cargo test: 344 passed, 0 failed
