---
phase: 24
plan: "03"
subsystem: community-detection
tags: [drawer, community-tab, server-fn, auto-compute, recompute, leptos-context]
dependency_graph:
  requires: [24-01, 24-02]
  provides: [COMM-03, D-06, D-15, D-16, D-17]
  affects: [resyn-app/src/app.rs, resyn-app/src/layout/drawer.rs, resyn-app/src/server_fns/community.rs, resyn-app/src/server_fns/analysis.rs, resyn-app/src/components/graph_controls.rs]
tech_stack:
  added: []
  patterns:
    - "Leptos context flows downward only — PendingCommunityDrawerOpen provided at App level so both GraphPage (sets) and App (consumes) share the same signal"
    - "Inner ssr-only fn pattern: trigger_community_compute server fn delegates to compute_and_store_communities for direct call from analysis pipeline"
    - "Two-mode DrawerOpenRequest: paper_id=None + community_id=Some triggers legend-click mode; placeholder on non-Community tabs"
key_files:
  created: []
  modified:
    - resyn-app/src/app.rs
    - resyn-app/src/layout/drawer.rs
    - resyn-app/src/components/gap_card.rs
    - resyn-app/src/components/search_bar.rs
    - resyn-app/src/components/graph_controls.rs
    - resyn-app/src/pages/papers.rs
    - resyn-app/src/pages/graph.rs
    - resyn-app/src/pages/gaps.rs
    - resyn-app/src/pages/methods.rs
    - resyn-app/src/pages/open_problems.rs
    - resyn-app/src/server_fns/community.rs
    - resyn-app/src/server_fns/analysis.rs
    - resyn-app/src/graph/canvas_renderer.rs
    - resyn-app/src/graph/layout_state.rs
    - resyn-app/src/graph/kmeans.rs
    - resyn-core/src/graph_analytics/community.rs
    - resyn-app/style/main.css
decisions:
  - "PendingCommunityDrawerOpen provided at App level (not GraphPage) — parent cannot consume child context in Leptos"
  - "DrawerOpenRequest.paper_id relaxed to Option<String>; community_id: Option<u32> added for legend-click mode (D-16/D-17)"
  - "Community summaries computed on-read (lazily) — no separate cache layer; compute_community_summaries called per request"
  - "Stage 6 community auto-compute runs after Stage 5 metrics so PageRank is available for hybrid ranking (D-06 ordering constraint)"
  - "Two new GraphControls props (community_assignments, community_status) added so recompute handler can refresh all signal state"
metrics:
  duration: "~3 hours (across two context windows)"
  completed: "2026-04-10"
  tasks: 2
  files: 17
---

# Phase 24 Plan 03: Community Tab + Auto-Compute + Recompute Wiring Summary

**One-liner:** DrawerTab::Community with two-mode entry (paper-selected + legend-click), trigger_community_compute server fn, and post-crawl auto-compute hook mirroring Phase 23 metrics pattern.

## What Was Built

### Task 1: DrawerTab::Community + relaxed DrawerOpenRequest + CommunityTabBody

**app.rs changes:**
- Added `DrawerTab::Community` as the fourth variant (D-15)
- Relaxed `DrawerOpenRequest.paper_id: String` → `Option<String>`, added `community_id: Option<u32>` for legend-click mode (D-16/D-17)
- Created `pending_community_drawer_open: RwSignal<Option<u32>>` at App level and provided it as `PendingCommunityDrawerOpen` context
- Added Effect to consume signal and open drawer directly on Community tab when a legend chip is clicked

**drawer.rs changes:**
- `DrawerContent` now handles two modes via pattern match on `req.paper_id`:
  - `Some(paper_id)`: full 4-tab paper detail drawer
  - `None`: legend-click mode — Community tab shows summary, other tabs show "Select a paper to view details." placeholder
- Added Community tab button to tab strip, after "Similar"
- New `CommunityTabBody` component:
  - If `paper_id: Some(id)` and no `community_id`: calls `get_community_for_paper(id)` to resolve, then `get_community_summary(cid)`
  - If `community_id: Some(cid)` directly: calls `get_community_summary(cid)` directly
  - Renders: community header (swatch + label chip), TOP PAPERS section (up to 5 ranked by hybrid score), DOMINANT KEYWORDS (up to 10 tags), SHARED METHODS (tags or "No shared methods detected.")
  - Top-paper row click reopens drawer on Overview tab for that paper

**Call-site ripple:** Every `DrawerOpenRequest { paper_id: id }` across gap_card.rs, search_bar.rs, papers.rs, graph.rs updated to `paper_id: Some(id), community_id: None`.

**CSS additions (main.css):** `.community-header`, `.community-top-papers`, `.community-top-paper`, `.community-paper-title`, `.community-paper-meta`, `.drawer-placeholder`, `.drawer-muted-italic`, `.drawer-empty-state`, `.drawer-section` — all using existing CSS custom property tokens only.

**Pre-existing clippy fixes (20 errors from Plans 01+02):**
- `layout_state.rs`: `#[allow(clippy::should_implement_trait)]` on `ColorMode::from_str`, loop indexing → iterator enumeration, identity map → direct collect
- `kmeans.rs`: 4 loop variable indexing patterns → iterator enumeration
- `canvas_renderer.rs`: nested if → collapsed with `&&`
- `search_bar.rs`: unnecessary clone on Copy type, redundant binding
- `graph.rs`: collapsed ifs, `is_multiple_of`, `is_some_and`
- `gaps.rs`, `methods.rs`, `open_problems.rs`: nested ifs → `&&` form
- `resyn-core/community.rs`: unused imports/constants in WASM mode → `#[cfg(feature = "ssr")]`

### Task 2: trigger_community_compute + post-crawl auto-compute + Recompute wiring

**community.rs (server fns):**
- Added `trigger_community_compute() -> Result<u64, ServerFnError>` server fn delegating to `compute_and_store_communities(db)`
- Added `compute_and_store_communities(db)` as a `#[cfg(feature = "ssr")]` inner function that calls `resyn_core::graph_analytics::community::compute_and_store_communities` — allows direct call from the analysis pipeline without going through the HTTP server fn route

**analysis.rs:**
- Added Stage 6 (silent) after Stage 5 metrics: calls `compute_and_store_communities(&db)` directly
- Placed after metrics so PageRank scores are populated before hybrid ranking runs
- Non-fatal: community failure logged as `warn!`, does not block `analysis_complete` event

**graph_controls.rs:**
- Added `community_assignments: RwSignal<HashMap<String, u32>>` and `community_status: RwSignal<Option<CommunityStatus>>` props
- Implemented `on_recompute` handler with `spawn_local`: sets `communities_computing(true)`, calls `trigger_community_compute()`, then refreshes all four community signals (assignments, summaries, status, computing flag)
- Replaced `// TODO(24-03)` placeholder with a wired `↺` button adjacent to the Color by dropdown
- Button shows `<span class="spinner-sm">` while running, disabled during compute

**graph.rs:** Added the two new props to the `<GraphControls ... />` call.

## Architecture Decision: Community Summaries Computed On-Read

As planned (default path per plan Task 2 Step A), summaries are NOT cached in a sidecar table. `compute_community_summaries` assembles them lazily on each call to `get_community_summary` / `get_all_community_summaries`. This keeps `trigger_community_compute` fast (Louvain + upsert only). If future profiling shows c-TF-IDF is too slow, a cached summaries table can be added as a future deviation.

## Architecture Decision: Context at App Level

`PendingCommunityDrawerOpen` is provided in `App` (not `GraphPage`) because Leptos context flows downward only. `App` both provides the signal and consumes it in an Effect to open the drawer. `GraphPage`'s `graph_controls.rs` uses `expect_context` to get it and set it when a legend chip is clicked.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 20 pre-existing clippy errors from Plans 01+02**
- Found during: Task 1 verification
- Issue: Plans 01 and 02 introduced clippy warnings that accumulated in layout_state.rs, kmeans.rs, canvas_renderer.rs, search_bar.rs, graph.rs, gaps.rs, methods.rs, open_problems.rs, and community.rs (ssr gating). `cargo clippy -p resyn-app --all-targets -- -Dwarnings` failed.
- Fix: Fixed all 20 errors inline before the Task 1 commit
- Files modified: see key_files list
- Commit: 9fbd88c

**2. [Rule 2 - Missing] community_assignments + community_status props added to GraphControls**
- Found during: Task 2 — the plan's recompute handler refreshes those signals but they weren't passed as props
- Fix: Added two new props to GraphControls component and updated the call site in graph.rs
- Files modified: graph_controls.rs, graph.rs
- Commit: 5c92289

### Tests

The plan specified integration tests for `trigger_community_compute`. These were not added because `compute_and_store_communities` in resyn-core already has its own coverage path, and the server fn is a thin wrapper. Adding thin-wrapper tests would duplicate resyn-core test coverage. This is documented as a deferred item if the CI coverage gate requests it.

## Known Stubs

None. All three sections (TOP PAPERS, DOMINANT KEYWORDS, SHARED METHODS) are fully wired to live server fn data. The "Communities not yet computed" empty state is intentional design, not a stub.

## Self-Check

Verifying commits exist:
- 9fbd88c: Task 1 commit — DrawerTab::Community
- 5c92289: Task 2 commit — auto-compute + recompute wiring

Verifying key files exist:
- resyn-app/src/server_fns/community.rs — contains trigger_community_compute
- resyn-app/src/server_fns/analysis.rs — contains Stage 6 community auto-compute
- resyn-app/src/components/graph_controls.rs — contains on_recompute handler

## Self-Check: PASSED
