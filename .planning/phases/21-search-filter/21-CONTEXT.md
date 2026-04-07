# Phase 21: Search & Filter - Context

**Gathered:** 2026-04-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can find papers by title, abstract, or author from anywhere in the UI and jump to them in the graph. The papers table filters its displayed rows as the user types. Search results are ranked by relevance. This phase does NOT add similarity, analytics, or export — those are separate phases (22-26).

</domain>

<decisions>
## Implementation Decisions

### Search placement & UX
- **D-01:** Global search bar in the app header/nav, persistent across all pages
- **D-02:** Results appear as a dropdown list below the search bar (top 8-10 matches with title, authors, year)
- **D-03:** Result actions are context-sensitive: on graph page pans to node + opens detail drawer, on papers page scrolls to row + opens detail drawer, elsewhere opens detail drawer
- **D-04:** Ctrl+K / Cmd+K keyboard shortcut focuses the search bar from any page

### Graph search integration
- **D-05:** Smooth lerp pan+zoom animation to center the target node (reuse existing viewport_fit lerp pattern)
- **D-06:** Matched node gets a pulse glow ring (2-3 pulses over ~2s) then fades back to normal
- **D-07:** Multi-match: viewport centers on top-ranked result, other non-matching nodes dimmed
- **D-08:** Graph does NOT pan while typing — only on explicit result selection (click or Enter). Dropdown updates live with 300ms debounce, viewport stays still until user commits to a result

### Search backend strategy
- **D-09:** SurrealDB full-text search with DEFINE ANALYZER and search index on paper table
- **D-10:** Searchable fields: title, abstract (summary), authors
- **D-11:** Relevance ranking via SurrealDB native search::score() (BM25-based)

### Papers table filtering
- **D-12:** Separate inline filter bar above the papers table (independent from global search bar)
- **D-13:** Server-side search — uses the same SurrealDB full-text search as the global bar (debounced)
- **D-14:** Matching text highlighted (bold or color) in title/author cells when filtering

### Claude's Discretion
- Debounce timing for papers table filter (suggested ~300ms to match global bar)
- Exact dropdown styling and positioning
- Pulse glow implementation details (CSS animation vs WebGL shader)
- Search result count display and "no results" empty state messaging
- Whether to show search result count badge

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above and the following project files:

### Requirements
- `.planning/REQUIREMENTS.md` — SRCH-01 through SRCH-04 define search acceptance criteria
- `.planning/ROADMAP.md` §Phase 21 — Success criteria and phase boundary

### Existing code patterns
- `resyn-app/src/pages/papers.rs` — PapersPanel component with sortable table (integration point for inline filter)
- `resyn-app/src/layout/drawer.rs` — Paper detail drawer (result click target)
- `resyn-app/src/server_fns/papers.rs` — Existing server fns pattern (GetPapers, GetPaperDetail)
- `resyn-app/src/graph/viewport_fit.rs` — Lerp animation pattern to reuse for search-to-node pan
- `resyn-core/src/database/schema.rs` — Current DB schema and index definitions (add search analyzer + index here)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `viewport_fit.rs` lerp animation: Reuse for smooth pan-to-node on search result selection
- `Drawer` component: Already supports opening with paper_id and initial_tab — search results click straight into this
- `get_papers()` server fn: Pattern for new `search_papers()` server fn
- `SelectedPaper` context signal: Can trigger drawer + graph pan from search result click

### Established Patterns
- Leptos server fns with `#[server(..., "/api")]` macro for all DB queries
- SurrealDB schema migrations in `schema.rs` with `DEFINE INDEX IF NOT EXISTS`
- Context signals (`SelectedPaper`, `DrawerOpenRequest`) for cross-component communication
- Sorting state via `RwSignal` in PapersPanel — search filter signal follows same pattern

### Integration Points
- App header/nav: New search bar component added here (global)
- `PapersPanel`: Add inline filter bar above table, modify row rendering to highlight matches
- Graph page: Listen for search-triggered pan events, add node highlight animation
- Router: Search from non-graph pages needs to set context that graph page reads on navigation

</code_context>

<specifics>
## Specific Ideas

- Graph viewport must NOT move during typing — only on deliberate result selection (user was explicit about avoiding chaotic panning)
- Context-sensitive result actions: always open drawer, plus page-specific action (pan on graph, scroll on papers table)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 21-search-filter*
*Context gathered: 2026-04-07*
