# Phase 8: Leptos Web Shell + Analysis Panels - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the Leptos CSR web app that serves the analysis pipeline's output — contradiction findings, ABC-bridge connections, open-problems, and method-combination gaps — plus a paper list and live crawl progress. No graph canvas (Phase 9). The app runs via `trunk serve` and communicates with an Axum backend through Leptos server functions backed by resyn-core.

</domain>

<decisions>
## Implementation Decisions

### App structure & navigation
- Sidebar + content layout with a Dashboard as the landing/overview page
- Sidebar sections: Dashboard, Papers, Gaps, Open Problems, Methods
- Sidebar is collapsible to an icon-only rail (thin vertical strip with icons, tooltips on hover)
- Dashboard shows summary cards (total papers, contradiction count, bridge count, open problems count, method coverage %). Each card links to its panel

### Visual style
- Dark minimal theme — dark background, muted colors, clean typography (VS Code / Linear aesthetic)
- Reduced visual noise for long research analysis sessions
- Separate CSS file(s) loaded by Trunk — no inline styles, no CSS-in-Rust

### Paper list panel
- Sortable table with columns: title, authors, year, citation count, analysis status
- Clickable column headers to sort
- Row click opens a side drawer (slides in from right) showing paper detail: abstract, methods, findings, open problems
- Table stays visible underneath the drawer

### Gap findings panel
- Card per finding — each card shows: type badge (Contradiction / ABC-Bridge), paper titles, confidence color-coded bar, shared terms as tags
- Justification text shown on card expand
- Paper IDs on cards are clickable — clicking opens the paper side drawer (cross-panel navigation)
- Confidence bar: horizontal bar with color gradient (red = low, green = high), numeric value beside it
- Filtering controls: toggle buttons for Contradictions/Bridges + confidence threshold slider

### Open-problems panel
- Problems ranked by recurrence count across the crawled corpus (from LlmAnnotation.open_problems)
- Display as a ranked list with recurrence count

### Method-combination heatmap
- Axes show method categories (from LlmAnnotation.methods[].category) — coarser, more readable
- Cell color: sequential blue scheme (dark blue = 0, bright cyan = many papers). Empty cells are distinct dark gray with a subtle marker (small icon or dashed border)
- Click a cell to drill down into a sub-matrix of individual method names within those two categories
- Empty cells indicate research gaps — marked subtly, not overwhelming

### Crawl progress
- Located at bottom of sidebar, always visible
- When sidebar expanded: full stats — progress bar + papers found/pending/failed + current depth/max depth + elapsed time + current paper title
- When sidebar collapsed: minimal — spinning indicator + percentage
- When idle (no crawl running): show last crawl summary (total papers, duration, failures)
- Web UI can start a crawl — form with paper ID, depth, source fields. Server function triggers crawl logic on backend

### Data layer
- Leptos server functions call resyn-core's PaperRepository and gap_analysis functions — reuse existing query and analysis logic, no duplicate DB access
- SSE crawl progress consumed from the existing /progress endpoint (Phase 7)

### Claude's Discretion
- Leptos component structure and file organization within resyn-app
- Trunk configuration details (index.html, asset pipeline)
- Axum server setup in resyn-server (router, state, server function integration)
- Exact CSS class naming and organization
- Loading states and skeleton screens
- Error display patterns
- Open-problems panel exact layout (cards vs table rows)

</decisions>

<specifics>
## Specific Ideas

- Sidebar collapse should feel smooth — icon-only rail with tooltips, not a jarring layout shift
- Dashboard summary cards should be the first thing you see — quick glance at the state of your research analysis
- Gap finding cards should make it easy to jump between a finding and the involved papers (clickable paper links to drawer)
- Method heatmap at category level first, then drill to specific methods — avoids overwhelming sparse matrices
- Crawl progress section doubles as a crawl launcher from the browser — full browser-based workflow without needing CLI

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `GapFinding` (resyn-core/src/datamodels/gap_finding.rs): Contradiction/AbcBridge findings with paper_ids, shared_terms, justification, confidence — directly feeds gap panel
- `LlmAnnotation` (resyn-core/src/datamodels/llm_annotation.rs): methods (name + category), findings, open_problems — feeds open-problems panel and method heatmap
- `PaperRepository` (resyn-core/src/database/queries.rs): upsert, get, citation graph traversal — server functions call through this
- `CrawlQueueRepository` (resyn-core/src/database/crawl_queue.rs): queue status queries — feeds crawl progress section
- `ProgressEvent` struct (resyn-server/src/commands/crawl.rs): SSE event type with all progress fields — consumed by frontend

### Established Patterns
- `ssr` feature gate on resyn-core: server-only modules (data_aggregation, database, llm) gated behind ssr; WASM-safe modules always available
- resyn-app depends on resyn-core (no ssr) — only WASM-safe types available in frontend
- resyn-server depends on resyn-core with ssr — full DB and API access
- Axum already a workspace dependency, used for SSE server in crawl.rs
- tokio broadcast channel pattern for SSE events

### Integration Points
- `resyn-app/src/lib.rs`: currently a stub — becomes the Leptos CSR app root
- `resyn-server/src/commands/serve.rs`: placeholder — becomes the Axum server hosting Leptos server functions
- SSE endpoint at `/progress` in crawl.rs — frontend connects to consume progress events
- `resyn-server/Cargo.toml`: already has axum, tokio-stream dependencies

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 08-leptos-web-shell-analysis-panels*
*Context gathered: 2026-03-17*
