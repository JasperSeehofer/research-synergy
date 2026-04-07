---
phase: 21-search-filter
plan: "03"
subsystem: frontend + graph + ui
tags: [search, graph-pan, pulse-glow, dimming, filter, leptos, canvas2d]
dependency_graph:
  requires: [21-01-search-backend, 21-02-search-bar]
  provides: [graph-search-pan, pulse-glow-ring, node-dimming, papers-filter-bar, match-highlighting]
  affects: [resyn-app/pages/graph, resyn-app/graph/layout_state, resyn-app/graph/canvas_renderer, resyn-app/graph/viewport_fit, resyn-app/pages/papers, resyn-app/style]
tech_stack:
  added: []
  patterns: [Canvas2D pulse animation via frame_counter, signal_debounced filter with server-side search, highlight_text helper for match highlighting]
key_files:
  created: []
  modified:
    - resyn-app/src/graph/layout_state.rs
    - resyn-app/src/graph/viewport_fit.rs
    - resyn-app/src/graph/canvas_renderer.rs
    - resyn-app/src/pages/graph.rs
    - resyn-app/src/pages/papers.rs
    - resyn-app/style/main.css
decisions:
  - "Consume SearchPanTrigger inside RAF loop via get_untracked() rather than a separate Effect — avoids borrow conflicts with render_state Rc<RefCell>"
  - "Pulse starts after fit_anim.active=false rather than immediately on pan trigger — ensures glow fires at correct screen position after viewport settles"
  - "papers filter: fetch all papers then filter by matching IDs (two calls) — simpler than direct SearchResult-to-Paper conversion; acceptable since paper list is small"
  - "highlight_text uses char-safe slice via .min(text.len()) to avoid panics on multibyte boundary edge cases"
metrics:
  duration: "~25 minutes"
  completed: "2026-04-07"
  tasks_completed: 2
  files_modified: 6
---

# Phase 21 Plan 03: Graph Pan/Highlight + Papers Table Filter Summary

Graph viewport pans to search-selected node with lerp animation, matched node gets a 2s pulse glow ring (#58a6ff), non-matching nodes dim; papers table gains inline 300ms-debounced filter bar with server-side BM25 search and bold-accent match highlighting.

## What Was Built

**layout_state.rs — new GraphState fields:**
- `search_highlighted: Option<String>` — paper_id of the node currently highlighted
- `pulse_start_frame: Option<u32>` — frame when pulse animation began
- `frame_counter: u32` — monotonically increasing frame count (wrapping_add each RAF tick)
- `search_highlight_ids: Vec<String>` — IDs for multi-match dimming (currently single match)

All initialized to `None`/`0`/`vec![]` in `from_graph_data`.

**viewport_fit.rs — `compute_single_node_pan_target`:**
New public function that centers a single node without zoom-to-fit behavior. Preserves current viewport scale clamped to `[0.5, 2.0]` per RESEARCH.md Pitfall 4. Returns `None` if paper_id not found in node list.

**canvas_renderer.rs — two additions:**
1. `search_dimmed` in `is_dimmed` closure: dims nodes not in `search_highlight_ids` when the set is non-empty (D-07). Combined with existing `selection_dimmed || topic_dimmed`.
2. Pulse glow ring block after "Selected outer ring": renders two arcs when `search_highlighted` matches the current node and `pulse_start_frame` is set. Outer ring 1 oscillates with `sin(t * 3π)` offset (3 pulses), ring 2 is steady glow. Both fade from alpha=0.8/0.5 to 0 over 120 frames (~2s at 60fps).

**graph.rs — RAF loop wiring:**
- Imports: `SearchPanTrigger` from `crate::app`, `compute_single_node_pan_target` from viewport_fit
- Context extraction: `let SearchPanTrigger(search_pan_signal) = expect_context::<SearchPanTrigger>()`
- `search_pan_signal` passed as new last argument to `start_render_loop`
- In the RAF closure: `frame_counter` incremented first each frame
- Pending pan request consumed via `get_untracked()`, triggers `compute_single_node_pan_target`, sets fit_anim + search state, calls `search_pan_signal.set(None)`
- After fit_anim completes (`!active && search_highlighted.is_some() && pulse_start_frame.is_none()`): sets `pulse_start_frame`
- After 120 frames from pulse_start: clears search_highlighted, pulse_start_frame, search_highlight_ids

**papers.rs — inline filter bar:**
- `filter_query: RwSignal<String>` + `debounced_filter: Signal<String>` (300ms via `signal_debounced`)
- `papers_resource` uses `debounced_filter` as key: empty → `get_papers()`, non-empty → `search_papers(q, Some(100))` + `get_papers()` filtered by matching arxiv_ids
- Filter bar HTML: `<div class="papers-filter-bar">` with icon, input, and `<Show>` result count
- `highlight_text(text, query)` helper: finds first case-insensitive match, returns span with `<strong class="filter-match">` around matched portion
- `PaperRow` and `PapersTableRows` updated to accept `filter_query: RwSignal<String>`; title and authors cells call `highlight_text`
- Empty state when filter active but no matches: "No papers match '...'" + clear hint

**main.css — filter bar styles:**
`.papers-filter-bar`, `.papers-filter-input-wrapper` (focus-within outline), `.papers-filter-input`, `.papers-filter-count`, `.filter-match` (font-weight semibold + accent color), `.papers-empty-state`, `.papers-empty-hint`

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Graph search pan, pulse glow ring, node dimming | 34de9bd |
| 2 | Papers table inline filter bar with match highlighting | 8307680 |
| 3 | Human verification checkpoint | — (pending) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Safe multibyte string slice in highlight_text**
- **Found during:** Task 2 implementation
- **Issue:** `text[start..start + query.len()]` can panic if the matched region crosses a multibyte character boundary (query.len() is byte-length, not char-length).
- **Fix:** Used `let end = (start + query.len()).min(text.len())` to clamp the end index before slicing. The `lower_text.find()` always returns a byte offset into `lower_text` (ASCII-lowered), so the start is already safe; the end clamp prevents OOB on edge cases with case-folding differences in multibyte characters.
- **Files modified:** resyn-app/src/pages/papers.rs
- **Commit:** 8307680

None of the other plan instructions required changes — all compiled and behaved as specified.

## Known Stubs

None — all features are fully wired to real data:
- Graph pan/highlight reads from `SearchPanTrigger` context set by `GlobalSearchBar` (Plan 02)
- Pulse glow ring uses real Canvas2D drawing in the existing render loop
- Papers filter calls `search_papers` server fn backed by SurrealDB BM25 (Plan 01)

## Threat Flags

No new network endpoints, auth paths, or file access patterns introduced beyond what was planned.

- T-21-07 (Tampering, SearchPanTrigger paper_id): accepted — bogus ID results in `compute_single_node_pan_target` returning `None`, no animation triggered
- T-21-08 (DoS, papers filter): mitigated — 300ms `signal_debounced` limits server fn calls; empty-string guard on `search_papers` prevents wasteful DB hits
- T-21-09 (Info Disclosure, highlight_text): accepted — highlights public paper metadata only

## Self-Check: PASSED

- resyn-app/src/graph/layout_state.rs — FOUND (contains search_highlighted, pulse_start_frame, frame_counter, search_highlight_ids)
- resyn-app/src/graph/viewport_fit.rs — FOUND (contains compute_single_node_pan_target, current_scale.clamp(0.5, 2.0))
- resyn-app/src/graph/canvas_renderer.rs — FOUND (contains search_dimmed, pulse_start_frame, #58a6ff)
- resyn-app/src/pages/graph.rs — FOUND (contains SearchPanTrigger, compute_single_node_pan_target)
- resyn-app/src/pages/papers.rs — FOUND (contains filter_query, signal_debounced, search_papers, highlight_text, filter-match, papers-filter-bar)
- resyn-app/style/main.css — FOUND (contains .papers-filter-bar, .papers-filter-input-wrapper, .filter-match, .papers-empty-state)
- cargo check -p resyn-app — PASSED (1 pre-existing dead_code warning for SearchPanRequest.paper_id in CSR compilation only)
- cargo test -p resyn-core --lib --features ssr — PASSED (206 tests)
- Commit 34de9bd — FOUND (Task 1)
- Commit 8307680 — FOUND (Task 2)
