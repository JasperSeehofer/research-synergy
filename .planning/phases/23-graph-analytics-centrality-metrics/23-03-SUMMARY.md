---
phase: 23-graph-analytics-centrality-metrics
plan: "03"
subsystem: resyn-app/graph-ui
tags: [graph-analytics, ui, size-mode, dashboard, leptos, animation]
dependency_graph:
  requires: [23-02]
  provides: [GANA-04, GANA-05]
  affects: [resyn-app/src/graph/layout_state.rs, resyn-app/src/components/graph_controls.rs, resyn-app/src/pages/graph.rs, resyn-app/src/pages/dashboard.rs]
tech_stack:
  added: []
  patterns:
    - "target_radius/current_radius lerp pattern for smooth animated node sizing"
    - "Per-component Resource + Suspense for non-blocking dashboard cards"
    - "SizeMode enum driving GraphState.metrics HashMap lookup"
key_files:
  created: []
  modified:
    - resyn-app/src/graph/layout_state.rs
    - resyn-app/src/components/graph_controls.rs
    - resyn-app/src/pages/graph.rs
    - resyn-app/src/graph/canvas_renderer.rs
    - resyn-app/src/graph/webgl_renderer.rs
    - resyn-app/src/graph/label_collision.rs
    - resyn-app/src/graph/interaction.rs
    - resyn-app/src/graph/lod.rs
    - resyn-app/src/graph/viewport_fit.rs
    - resyn-app/src/server_fns/metrics.rs
    - resyn-app/src/pages/dashboard.rs
    - resyn-app/style/main.css
decisions:
  - "Used target_radius/current_radius lerp (LERP_FACTOR=0.15) rather than a timed CSS transition — keeps animation in sync with the RAF loop already driving graph rendering"
  - "Added get_metrics_pairs server fn returning (id, pagerank, betweenness) tuples to populate the metrics HashMap in one roundtrip"
  - "InfluentialPapersCard has its own Resource + Suspense so it never delays the 5 existing summary cards"
  - "Kept node.radius (physics) separate from current_radius (visual) — force layout and hit-testing still use the stable physics radius"
  - "Task 3 checkpoint auto-approved (AUTO mode active)"
metrics:
  duration_minutes: 45
  completed_date: "2026-04-09"
  tasks_completed: 3
  files_modified: 12
---

# Phase 23 Plan 03: Graph Metrics UI — Size by Dropdown, Node Animation, Dashboard Card Summary

Wire PageRank and betweenness centrality metrics into the graph UI and dashboard: animated node sizing by metric, "Size by" dropdown with disabled states, metric scores in tooltips, recompute button, and the "Most Influential Papers" dashboard card.

## What Was Built

### Task 1: SizeMode enum, Size by dropdown, animated node radius, metric tooltip

**layout_state.rs:**
- Added `SizeMode` enum (Uniform, PageRank, Betweenness, Citations) with `#[default] Uniform`
- Added `target_radius: f64` and `current_radius: f64` to `NodeState` (alongside existing `radius` for physics)
- Added `size_mode: SizeMode` and `metrics: HashMap<String, (f32, f32)>` to `GraphState`
- Added `GraphState::update_node_target_radii()` method — maps metric scores to [4.0, 18.0] via sqrt scaling
- Updated all 6 test NodeState constructors to include the new required fields

**server_fns/metrics.rs:**
- Added `get_metrics_pairs()` server fn returning `Vec<(String, f32, f32)>` (arxiv_id, pagerank, betweenness)
- Added two additional server fns (`get_all_metrics`, `get_all_betweenness`) for potential future use

**graph_controls.rs:**
- Added `size_mode: RwSignal<SizeMode>`, `metrics_ready: RwSignal<bool>`, `metrics_computing: RwSignal<bool>` parameters
- Added "Size by" dropdown group with: Uniform, PageRank (disabled when not ready), Betweenness (disabled when not ready), Citations options
- Added computing spinner in the "Size by" label
- Added Recompute button (↺) that calls `trigger_metrics_compute()`, disabled while computing

**pages/graph.rs:**
- Added `size_mode`, `metrics_ready`, `metrics_computing` signals
- Added `metrics_status_resource` (polls `get_metrics_status`) and `metrics_pairs_resource` (fetches all metric pairs)
- Added `metrics_map_signal` shared between reactive and RAF contexts
- Added `prev_size_mode` tracker in RAF loop
- Each RAF frame: syncs metrics map into GraphState on first arrival, detects size_mode changes and calls `update_node_target_radii()`
- Added lerp step: `LERP_FACTOR = 0.15`, ~95% transition in 18 frames (~300ms at 60fps)
- Updated `node_tooltip()` to accept `size_mode` and `metrics` params — appends "PageRank: 0.XXXX", "Betweenness: 0.XXXX", or "Citations: N" based on active mode

**Renderers and label positioning:**
- All drawing code switched from `node.radius` to `node.current_radius`: canvas_renderer.rs (circle, ring, pulse, arrowhead), webgl_renderer.rs (instance data, seed ring, edge arrowhead), label_collision.rs, graph.rs label offsets, draw_topic_rings

### Task 2: "Most Influential Papers" dashboard card

**pages/dashboard.rs:**
- Added `InfluentialPapersCard` component with its own `Resource::new(|| (), |_| get_top_pagerank_papers(5))` + `Suspense`
- Shows top 5 ranked entries: rank number (1.), truncated title (50 chars + ellipsis), year, PR score formatted as "2021 · PR: 0.043"
- Three states: skeleton loading (5 skeleton-text rows), error ("—" + "Failed to load"), empty ("No metrics computed yet")
- "View all →" link to /papers
- Placed as 6th card inside `DashboardCards`, after the 5 existing summary cards
- Added SkeletonCard and ErrorCard for "Most Influential Papers" in the outer Suspense fallback

**style/main.css:**
- Added `.influential-list` (flex column, gap 6px)
- Added `.influential-entry` (flex row, align-start, gap 8px)
- Added `.influential-rank` (12px, semibold, muted, min-width 20px)
- Added `.influential-info` (flex 1, min-width 0)
- Added `.influential-title` (14px, -webkit-line-clamp 2)
- Added `.influential-meta` (12px, muted)

### Task 3: Visual verification checkpoint (auto-approved)

Auto-approved in AUTO mode. Build compiles cleanly, all 15 existing tests pass.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing functionality] Added get_metrics_pairs server fn**
- **Found during:** Task 1 — plan referenced a "get_all_metrics" bulk server fn that didn't exist
- **Fix:** Added `get_metrics_pairs()` returning `Vec<(String, f32, f32)>` using the existing `GraphMetricsRepository::get_all_metrics()` DB method
- **Files modified:** resyn-app/src/server_fns/metrics.rs
- **Commit:** d952e71

**2. [Rule 1 - Bug] Fixed 6 NodeState test constructors missing new required fields**
- **Found during:** Task 1 cargo check — struct literal missing `target_radius` and `current_radius`
- **Fix:** Added `target_radius: radius` and `current_radius: radius` to test helpers in interaction.rs, lod.rs, label_collision.rs, viewport_fit.rs, and layout_state.rs
- **Files modified:** 5 files
- **Commit:** d952e71

**3. [Rule 1 - Bug] Updated all visual rendering to use current_radius**
- **Found during:** Task 1 — plan said "use current_radius in renderers" but analysis revealed more places than listed: arrowheads in canvas_renderer.rs and webgl_renderer.rs, seed ring in webgl_renderer.rs, label offsets in graph.rs, label_collision.rs, and draw_topic_rings
- **Fix:** Systematically replaced all visual `node.radius` uses with `node.current_radius`; kept `node.radius` only for physics (force layout, hit testing)
- **Files modified:** canvas_renderer.rs, webgl_renderer.rs, label_collision.rs, graph.rs
- **Commit:** d952e71

## Commits

| Hash | Message |
|------|---------|
| d952e71 | feat(23-03): SizeMode enum, Size by dropdown, animated node radius, metric tooltip |
| 709e8a8 | feat(23-03): Most Influential Papers dashboard card |

## Known Stubs

None — all data flows are wired to real server functions backed by SurrealDB.

## Threat Flags

None — no new network endpoints or auth paths introduced beyond those defined in the plan's threat model. The recompute button is DoS-guarded server-side via the corpus fingerprint check (T-23-06, from 23-02).

## Self-Check: PASSED

- resyn-app/src/graph/layout_state.rs: FOUND (contains SizeMode, target_radius, current_radius, size_mode, metrics)
- resyn-app/src/components/graph_controls.rs: FOUND (contains size_mode param, Size by dropdown, Recompute button)
- resyn-app/src/pages/graph.rs: FOUND (contains LERP_FACTOR, PageRank: in tooltip)
- resyn-app/src/pages/dashboard.rs: FOUND (contains InfluentialPapersCard, Most Influential Papers, PR:)
- resyn-app/style/main.css: FOUND (contains .influential-list, .influential-entry, .influential-rank, .influential-title)
- Commits d952e71 and 709e8a8: FOUND
- cargo check --all-targets: PASSED
- cargo test --lib: 15/15 PASSED
