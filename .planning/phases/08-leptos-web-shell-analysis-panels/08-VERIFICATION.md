---
phase: 08-leptos-web-shell-analysis-panels
verified: 2026-03-17T09:00:00Z
status: human_needed
score: 5/5 must-haves verified
human_verification:
  - test: "Open http://localhost:8080 after running trunk serve and cargo run -p resyn-server -- serve --db surrealkv://./test_data"
    expected: "All 5 panels render with dark theme; sidebar navigates between routes; Papers panel shows real paper data; Drawer opens on row click; sidebar collapse toggle works"
    why_human: "Browser rendering, visual dark-theme consistency, real-time SSE progress bar update, and interactive drawer/sort behavior cannot be verified programmatically"
  - test: "Enter a valid arXiv ID in the sidebar crawl form and click Start Crawl"
    expected: "Server returns 'Crawl started' message; SSE /progress endpoint emits ProgressEvents; progress bar begins updating in sidebar"
    why_human: "End-to-end crawl dispatch via CrawlQueue + SSE live update loop requires a running server and browser observation"
---

# Phase 8: Leptos Web Shell + Analysis Panels Verification Report

**Phase Goal:** The browser app serves the analysis pipeline's output — contradiction findings, bridge connections, open-problems, and method gaps — without touching the graph canvas
**Verified:** 2026-03-17T09:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (from ROADMAP.md Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `trunk serve` starts the app and the browser renders paper list data fetched from Axum server via Leptos server functions | ? HUMAN | WASM CSR build passes; Axum server compiles with `handle_server_fns_with_context`; `register_explicit<T>()` called for all 8 server fns in `serve.rs`; but browser rendering requires human |
| 2 | Contradiction findings and ABC-bridge connections from SurrealDB appear as labeled entries in the gap panel | ? HUMAN | `GapsPanel` fetches `get_gap_findings()` via `Resource::new`; `GapCard` renders badge (badge-contradiction / badge-bridge), confidence bar, shared terms, expandable justification; all automated checks pass |
| 3 | Open-problems panel shows problems ranked by recurrence count across the crawled corpus | ? HUMAN | `OpenProblemsPanel` fetches `get_open_problems_ranked()`; `RankedList` renders with rank number, problem text, recurrence badge; `aggregate_open_problems` has 3 passing unit tests verifying ranking correctness |
| 4 | Method-combination gap matrix renders as a heatmap showing existing vs absent method pairings | ? HUMAN | `MethodsPanel` + `Heatmap` component wired to `get_method_matrix` and `get_method_drilldown`; CSS grid with cell-empty/cell-low/cell-medium/cell-high classes; drill-down on cell click; automated build passes |
| 5 | Crawl progress bar updates in real time from the SSE endpoint established in Phase 7 | ? HUMAN | `CrawlProgress` uses `leptos_use::use_event_source::<ProgressEvent, JsonSerdeCodec>("/progress")`; `Effect::new` updates `last_event` signal on each SSE message; sidebar footer wired to `<CrawlProgress collapsed=collapsed/>`; real-time update requires running server |

**Score:** 5/5 truths — all automated checks VERIFIED, human confirmation needed for browser rendering and real-time behavior

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `resyn-core/src/datamodels/progress.rs` | ProgressEvent struct, WASM-safe (serde only) | VERIFIED | Contains `pub struct ProgressEvent` with `Serialize, Deserialize, PartialEq`; no server-only imports |
| `resyn-app/Cargo.toml` | Leptos CSR dependencies with csr/ssr feature gates | VERIFIED | `csr = ["leptos/csr"]`, `ssr = ["leptos/ssr", "dep:leptos_axum", ...]`; all Leptos 0.8 ecosystem deps present |
| `resyn-server/Cargo.toml` | Server dependencies including leptos_axum and resyn-app with ssr | VERIFIED | `resyn-app = { ..., features = ["ssr"] }`, `leptos_axum = "0.8"`, `server_fn` with axum-no-default |
| `resyn-app/Trunk.toml` | Trunk build config with proxy to Axum | VERIFIED | `[[proxy]]` rules for `/api/` and `/progress` pointing to `http://localhost:3000` |
| `resyn-app/index.html` | Trunk entry point with csr feature flag | VERIFIED | `<link data-trunk rel="rust" data-cargo-features="csr"/>` present (post Plan 07 fix) |
| `resyn-app/style/main.css` | Full CSS design system (1300+ lines) | VERIFIED | 1304 lines; `--color-bg: #0d1117` token; all component classes: dashboard-card, data-table, filter-bar, heatmap-grid, ranked-list, gap-card, crawl-form, etc. |
| `resyn-app/src/app.rs` | App component with Router and 5 routes | VERIFIED | `Router` with 5 `Route` components (`/`, `/papers`, `/gaps`, `/problems`, `/methods`); provides `SelectedPaper` and `SidebarCollapsed` context |
| `resyn-app/src/lib.rs` | WASM entry point mounting App to body | VERIFIED | `mount_to_body(app::App)` at `#[wasm_bindgen(start)]` |
| `resyn-app/src/layout/sidebar.rs` | Collapsible sidebar with 5 nav links and CrawlProgress | VERIFIED | 5 `NavItem` components with `A` router links; `<CrawlProgress collapsed=collapsed/>` in footer |
| `resyn-app/src/layout/drawer.rs` | Paper detail drawer with abstract, methods, findings | VERIFIED | `role="dialog" aria-modal="true"`; fetches `get_paper_detail` via Resource; renders title, authors, abstract, methods (tags), findings, open problems |
| `resyn-app/src/pages/dashboard.rs` | Dashboard with 5 summary cards linked to panels | VERIFIED | 5 `SummaryCard` components (Total Papers, Contradictions, ABC-Bridges, Open Problems, Method Coverage) with links to respective routes |
| `resyn-app/src/pages/papers.rs` | Sortable paper table with row click to open drawer | VERIFIED | `RwSignal<(SortColumn, SortDir)>` for sort state; `on:click` on rows calls `selected_paper.set(Some(id))`; `class="data-table"` present |
| `resyn-app/src/pages/gaps.rs` | Gap findings panel with filter controls and card list | VERIFIED | `class="filter-bar"` with Contradictions/Bridges toggles + confidence slider; `StoredValue` pattern for data; maps to `<GapCard>` |
| `resyn-app/src/pages/open_problems.rs` | Ranked open-problems list | VERIFIED | `class="ranked-list"` via `RankedList` sub-component; rank number, problem text, recurrence-badge |
| `resyn-app/src/pages/methods.rs` | Methods page with heatmap and drill-down | VERIFIED | `Resource::new` for matrix + drilldown; `RwSignal<Option<(String, String)>>` for drilldown state; "Back to overview" button |
| `resyn-app/src/components/gap_card.rs` | GapCard with badge, confidence bar, shared terms, expand | VERIFIED | badge-contradiction / badge-bridge; confidence-bar-fill; tag pills; expand/collapse via `RwSignal<bool>` |
| `resyn-app/src/components/heatmap.rs` | CSS grid heatmap with drill-down | VERIFIED | `class="heatmap-grid"` CSS grid; `cell-empty/cell-low/cell-medium/cell-high`; click fires `on_cell_click` callback for non-empty cells only |
| `resyn-app/src/components/crawl_progress.rs` | SSE-connected progress display with crawl form | VERIFIED | `use_event_source::<ProgressEvent, JsonSerdeCodec>("/progress")`; `CrawlForm` with paper ID input, depth select, source select, Action wrapping `start_crawl` |
| `resyn-app/src/server_fns/papers.rs` | get_papers, get_paper_detail, get_dashboard_stats, start_crawl | VERIFIED | All 4 functions with `#[server]` macro; SSR bodies query DB via `use_context::<Arc<Db>>()`; `start_crawl` calls `enqueue_if_absent` and spawns background tokio task |
| `resyn-app/src/server_fns/gaps.rs` | get_gap_findings server function | VERIFIED | `#[server(GetGapFindings, "/api")]`; calls `GapFindingRepository::new(&db).get_all_gap_findings()` |
| `resyn-app/src/server_fns/problems.rs` | get_open_problems_ranked server function | VERIFIED | `#[server(GetOpenProblemsRanked, "/api")]`; calls `aggregate_open_problems(&annotations)` |
| `resyn-app/src/server_fns/methods.rs` | get_method_matrix, get_method_drilldown server functions | VERIFIED | Both `#[server]` fns; call `build_method_matrix`; drilldown filters annotations by category pair |
| `resyn-server/src/commands/serve.rs` | Axum router with server fn handler and SSE proxy | VERIFIED | `handle_server_fns_with_context` at `/api/{*fn_name}`; `register_explicit<T>()` for all 8 server fns; `ServeDir` fallback; `connect(&args.db)` (not `connect_local`) |
| `resyn-core/src/analysis/aggregation.rs` | aggregate_open_problems and build_method_matrix pure functions | VERIFIED | Both functions present; `RankedProblem` and `MethodMatrix` types with `Serialize + Deserialize`; inline unit tests plus external integration tests |
| `resyn-core/tests/aggregation_tests.rs` | 6 integration tests for aggregation | VERIFIED | All 6 tests pass: empty cases, ranking, single-annotation, pair counts, alphabetical normalization |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `resyn-app/src/lib.rs` | `resyn-app/src/app.rs` | `mount_to_body(App)` | WIRED | `mount_to_body(app::App)` present at line 13 |
| `resyn-app/src/app.rs` | `resyn-app/src/pages/*.rs` | `Router Routes` | WIRED | `Route path=path!("/")` through `path!("/methods")` all wired |
| `resyn-server/src/commands/serve.rs` | `resyn-app/src/server_fns/` | `handle_server_fns_with_context` | WIRED | `register_explicit<T>()` for all 8 fns + wildcard POST at `/api/{*fn_name}` |
| `resyn-app/src/pages/dashboard.rs` | `resyn-app/src/server_fns/papers.rs` | `Resource::new` calling `get_dashboard_stats` | WIRED | `Resource::new(|| (), |_| get_dashboard_stats())` at line 8 |
| `resyn-app/src/pages/papers.rs` | `resyn-app/src/server_fns/papers.rs` | `Resource::new` calling `get_papers` | WIRED | `Resource::new(|| (), |_| get_papers())` at line 49 |
| `resyn-app/src/pages/papers.rs` | `resyn-app/src/layout/drawer.rs` | `set_selected_paper` context signal | WIRED | `selected_paper.set(Some(id))` on row click at line 111 |
| `resyn-app/src/pages/gaps.rs` | `resyn-app/src/server_fns/gaps.rs` | `Resource` calling `get_gap_findings` | WIRED | `Resource::new(|| (), |_| get_gap_findings())` at line 10 |
| `resyn-app/src/components/gap_card.rs` | `resyn-app/src/layout/drawer.rs` | `set_selected_paper` context signal | WIRED | `set_paper.set(Some(id_clone.clone()))` on paper ID button click at line 46 |
| `resyn-app/src/pages/open_problems.rs` | `resyn-app/src/server_fns/problems.rs` | `Resource` calling `get_open_problems_ranked` | WIRED | `Resource::new(|| (), |_| get_open_problems_ranked())` at line 9 |
| `resyn-app/src/pages/methods.rs` | `resyn-app/src/server_fns/methods.rs` | `Resource` calling `get_method_matrix` | WIRED | `Resource::new(|| (), |_| async { get_method_matrix().await })` at line 10 |
| `resyn-app/src/components/heatmap.rs` | `resyn-app/src/server_fns/methods.rs` | `Resource` calling `get_method_drilldown` on cell click | WIRED | `drilldown.set(Some((row, col)))` on Callback; `get_method_drilldown(cat_a, cat_b)` in drilldown_resource |
| `resyn-app/src/components/crawl_progress.rs` | `/progress` SSE endpoint | `use_event_source` | WIRED | `use_event_source::<ProgressEvent, JsonSerdeCodec>("/progress")` at line 25 |
| `resyn-app/src/server_fns/papers.rs` | `resyn-core/src/database/crawl_queue.rs` | `CrawlQueueRepository::enqueue_if_absent` | WIRED | `CrawlQueueRepository::new(&db).enqueue_if_absent(...)` at line 152; background tokio task runs full crawl loop |

### Requirements Coverage

| Requirement | Source Plan(s) | Description | Status | Evidence |
|-------------|---------------|-------------|--------|----------|
| WEB-03 | 08-01, 08-02, 08-03, 08-06, 08-07 | Leptos CSR shell with Trunk build pipeline and routing | VERIFIED | WASM build passes; Trunk.toml with `[[proxy]]`; Router with 5 routes; `data-cargo-features="csr"` in index.html |
| WEB-04 | 08-01, 08-03, 08-04 | Axum server functions exposing analysis pipeline to frontend | VERIFIED | 8 server fns with `#[server]` macro; `handle_server_fns_with_context`; `register_explicit<T>()` for cross-crate registration |
| AUI-01 | 08-05 | Gap findings rendered with contradiction edges, bridge badges | VERIFIED | GapCard has badge-contradiction / badge-bridge type badge; GapsPanel has filterable card list with type toggle; ROADMAP criteria says "labeled entries in gap panel" (not graph) |
| AUI-02 | 08-04, 08-05 | Open-problems aggregation panel ranked by recurrence frequency | VERIFIED | `aggregate_open_problems` pure function with 3 tests; `OpenProblemsPanel` renders ranked list with recurrence count badges |
| AUI-03 | 08-04, 08-06 | Method-combination gap matrix showing existing vs absent method pairings | VERIFIED | `build_method_matrix` pure function with 3 tests; `Heatmap` renders CSS grid with empty/low/medium/high cell classes; drill-down to individual method names |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `resyn-app/src/pages/papers.rs` | 325 | `// TODO: derive from LLM annotation presence (Plan 05 wires this up).` | Info | Status column uses `citation_count` as proxy for annotation presence. Column renders correctly (shows "Analyzed"/"Pending"); proxy logic is documented. Not a blocker — the plan note references a future enhancement deferred to a later phase. |

### Human Verification Required

#### 1. Full Browser Rendering Confirmation

**Test:** Run `cargo run -p resyn-server -- serve --db surrealkv://./test_data` in one terminal, then `cd resyn-app && trunk serve` in another. Open http://localhost:8080.

**Expected:**
- Dashboard loads with 5 summary cards (dark `#0d1117` background, numeric values populated from seeded database)
- Papers panel renders a sortable table; clicking a column header toggles sort direction with aria-sort attribute
- Clicking a paper row opens the detail drawer from the right with abstract, methods (if annotated), findings, open problems; backdrop click or X closes it
- Gap Findings panel loads cards with contradiction/bridge badges and confidence bars; type toggle buttons filter the list; confidence slider filters by threshold
- Open Problems panel shows a ranked list with `#N` rank numbers and recurrence count badges
- Methods panel renders the heatmap CSS grid with color-coded cells; clicking a non-empty cell shows a drill-down matrix; "Back to overview" returns
- Sidebar collapse button shrinks sidebar to icon-only rail (48px); expand returns to 240px with labels

**Why human:** Visual dark-theme consistency, responsive interactive state (sidebar toggle, drawer slide-in, sort animations), and table data rendering from real seed data cannot be verified programmatically.

#### 2. Real-Time Crawl Progress via SSE

**Test:** With the server running and test_data populated, enter arXiv ID `2503.18887` in the sidebar crawl form (depth 1, source: arXiv). Click Start Crawl.

**Expected:**
- Server returns "Crawl started for paper 2503.18887 (max depth 1)" feedback message below the form
- The SSE `/progress` endpoint begins emitting `ProgressEvent` JSON
- The progress bar in the sidebar footer updates in real time as papers are fetched

**Why human:** Verifying live SSE event delivery and the reactive progress bar update requires a running server and browser observation. The code wiring is confirmed (use_event_source + Effect updating RwSignal), but the end-to-end behavior depends on the network stack and Phase 7 SSE infrastructure.

### Automated Check Results

| Check | Result |
|-------|--------|
| `cargo build -p resyn-app --target wasm32-unknown-unknown --features csr` | PASS |
| `cargo check -p resyn-app --features ssr` | PASS |
| `cargo check -p resyn-server` | PASS |
| `cargo test -p resyn-core --test aggregation_tests` (6 tests) | PASS |
| `cargo test` (182 tests total) | PASS (176 resyn-core unit/integration + 6 aggregation) |

### Gaps Summary

No automated gaps. All 5 observable truths from ROADMAP.md are fully supported by the codebase:

1. The WASM app shell, Router, and all 5 routes are implemented and build clean.
2. Gap findings panel fetches from SurrealDB and renders cards with type badges and filter controls.
3. Open problems panel aggregates via `aggregate_open_problems` (tested) and renders a ranked list.
4. Method heatmap renders via `build_method_matrix` (tested) with drill-down.
5. SSE crawl progress wired via `leptos-use::use_event_source`; `start_crawl` dispatches a real background crawl via `CrawlQueueRepository`.

Two items require human confirmation before the phase can be formally closed: visual rendering in the browser and live SSE progress updates.

---

_Verified: 2026-03-17T09:00:00Z_
_Verifier: Claude (gsd-verifier)_
