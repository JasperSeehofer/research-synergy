---
phase: 10-analysis-ui-polish-scale
plan: "03"
subsystem: ui
tags: [leptos, drawer, tabs, provenance, snippet-highlight, source-text]

# Dependency graph
requires:
  - phase: 10-01
    provides: "find_highlight_range utility, provenance fields on Finding/Method, PaperDetail with extraction field"

provides:
  - "DrawerTab enum (Overview, Source) in app.rs"
  - "DrawerOpenRequest struct replacing Option<String> in SelectedPaper context"
  - "Tabbed drawer with Overview/Source tab strip"
  - "SourceTabBody component rendering extraction sections with snippet highlighting"
  - "Gap card paper links open drawer on Source tab"
  - "CSS classes: .drawer-tab-strip, .drawer-tab, .source-section, .snippet-highlight, .abstract-only-label, .source-empty-state"

affects: [drawer, gap-card, graph-page, papers-panel]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "DrawerOpenRequest carries tab + provenance context — drawer component reads initial_tab to set RwSignal on mount"
    - "SourceSectionText sub-component handles per-section highlight: extracts snippet range via find_highlight_range, splits text into before/mark/after"
    - "filter_map pattern on sections Vec to skip None sections naturally"

key-files:
  created: []
  modified:
    - resyn-app/src/app.rs
    - resyn-app/src/layout/drawer.rs
    - resyn-app/src/components/gap_card.rs
    - resyn-app/style/main.css

key-decisions:
  - "Tab state (RwSignal<DrawerTab>) initialized from DrawerOpenRequest.initial_tab — resets to Overview when drawer is closed and reopened without Source tab request"
  - "SourceSectionText is a separate component (not inline) — required by Leptos for conditional view branching with mixed AnyView types"
  - "DrawerOpenRequest uses #[derive(Default)] with DrawerTab::Overview as default — allows graph/papers page to use ..Default::default() when no provenance context"

patterns-established:
  - "Provenance navigation: gap card sets DrawerOpenRequest { initial_tab: DrawerTab::Source, .. } to jump directly to Source tab"
  - "Snippet highlight: section key compared case-insensitively to highlight_section; find_highlight_range maps to byte offsets for text splitting"

requirements-completed: [AUI-04]

# Metrics
duration: 45min
completed: 2026-03-18
---

# Phase 10 Plan 03: Drawer Source Tab and Provenance UI Summary

**Tabbed drawer with Overview/Source tabs, extraction section display with find_highlight_range snippet highlighting, and gap card provenance click wired to Source tab**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-03-18T17:00:00Z
- **Completed:** 2026-03-18T17:45:00Z
- **Tasks:** 1
- **Files modified:** 4

## Accomplishments
- Extended `SelectedPaper` context from `Option<String>` to `Option<DrawerOpenRequest>` with tab and highlight fields
- Added `DrawerTab` enum and full drawer tab strip UI — clicking Overview/Source toggles tab state
- Created `SourceTabBody` component rendering all five extraction sections (abstract, introduction, methods, results, conclusion) with snippet highlighting via `find_highlight_range`
- Abstract-only papers show italic "Abstract only — full text unavailable" label
- Gap card paper links now open drawer directly on Source tab with `title="View source in paper"`
- Added CSS design-token-based styles for tab strip, source sections, and snippet highlight mark

## Task Commits

1. **Task 1: Tabbed drawer with Source tab and snippet highlighting** - `465010c` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified
- `resyn-app/src/app.rs` — DrawerTab enum, DrawerOpenRequest struct, SelectedPaper type updated
- `resyn-app/src/layout/drawer.rs` — Tab strip, DrawerBody restructured, SourceTabBody + SourceSectionText components, find_highlight_range integration
- `resyn-app/src/components/gap_card.rs` — Paper links use DrawerOpenRequest with DrawerTab::Source and title attribute
- `resyn-app/style/main.css` — .drawer-tab-strip, .drawer-tab, .source-section, .source-section-header, .source-section-text, .snippet-highlight, .abstract-only-label, .source-empty-state

## Decisions Made
- Tab state uses `RwSignal<DrawerTab>` initialized from `DrawerOpenRequest.initial_tab` — naturally resets when drawer is closed and reopened without provenance context because a new DrawerOpenRequest defaults to Overview
- `SourceSectionText` extracted as a named component (not inline) — Leptos view! macro requires this for conditional `into_any()` branches
- Used `filter_map` on the sections Vec to skip None sections without explicit if-let chains

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed borrow checker errors in start_render_loop (graph.rs)**
- **Found during:** Task 1 (pre-existing compilation failure)
- **Issue:** `update_lod_visibility(&mut s.graph.nodes, s.viewport.scale, &s.graph.seed_paper_id)` and `update_temporal_visibility` had simultaneous mutable+immutable borrows via `s`
- **Fix:** Extracted `lod_scale`, `seed_id`, `t_min`, `t_max` to local variables before calling the mutable borrow functions
- **Files modified:** resyn-app/src/pages/graph.rs
- **Verification:** `cargo check -p resyn-app --features csr` passes
- **Committed in:** b6b6859 (already in HEAD at task start — the fix was present in that commit)

---

**Total deviations:** 1 auto-fixed (pre-existing borrow error in graph.rs, already resolved in prior commit b6b6859)
**Impact on plan:** No scope creep. Fix was necessary for correct compilation.

## Issues Encountered
- The file `resyn-app/src/pages/graph.rs` was modified by linters/formatters between reads, causing tool "file modified since read" errors. Handled by re-reading before each edit.
- Previous commit b6b6859 had already applied `DrawerOpenRequest` to graph.rs and papers.rs, and had added CSS and fixed the borrow checker — those files were already at the correct state.

## Next Phase Readiness
- AUI-04 (provenance display) complete: clicking gap card paper links shows Source tab with section text
- Snippet highlighting ready: once findings carry `source_snippet`/`source_section` fields, the UI wiring is in place
- Ready for plan 10-04: browser verification checkpoint

---
*Phase: 10-analysis-ui-polish-scale*
*Completed: 2026-03-18*
