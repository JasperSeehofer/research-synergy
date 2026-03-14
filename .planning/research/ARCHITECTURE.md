# Architecture Research

**Domain:** Leptos web UI migration + WebGL graph renderer + incremental crawl — integration with existing Rust LBD app
**Researched:** 2026-03-15
**Confidence:** MEDIUM (Leptos patterns HIGH; WebGL/WASM graph specifics MEDIUM; crawl queue pattern MEDIUM)

---

## Standard Architecture

### System Overview

The v1.1 migration replaces the egui/eframe desktop binary with a full-stack Leptos web application served by Axum. The existing 8-module Rust core (data_aggregation, database, datamodels, data_processing, nlp, llm, gap_analysis, visualization) becomes the server-side library. The browser receives a WASM bundle that renders graph and panels using Leptos signals plus a Canvas/WebGL renderer.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         BROWSER (WASM)                               │
│  ┌────────────────┐  ┌──────────────────┐  ┌──────────────────────┐ │
│  │  Leptos UI     │  │  Canvas Renderer │  │  Side Panels         │ │
│  │  (signals,     │  │  (graph layout   │  │  (gap findings,      │ │
│  │   server fns)  │  │   WebGL nodes)   │  │   open problems,     │ │
│  └───────┬────────┘  └────────┬─────────┘  │   method matrix)     │ │
│          │ server fn calls    │ graph data │                      │ │
│          │ (typed RPC)        │ (JSON)     └──────────────────────┘ │
└──────────┼────────────────────┼─────────────────────────────────────┘
           │ HTTP / SSE         │
┌──────────┼────────────────────┼─────────────────────────────────────┐
│          │     AXUM SERVER    │                                      │
│  ┌───────▼────────┐  ┌────────▼────────┐  ┌──────────────────────┐ │
│  │ leptos_axum    │  │  /api/graph     │  │  /api/crawl/progress │ │
│  │ (SSR + hydrate)│  │  (graph JSON)   │  │  (SSE stream)        │ │
│  └───────┬────────┘  └────────┬────────┘  └──────────┬───────────┘ │
│          │                   │                        │             │
│  ┌───────▼────────────────────▼────────────────────────▼──────────┐ │
│  │                     RESYN CORE LIBRARY                          │ │
│  │  data_aggregation │ database │ datamodels │ data_processing     │ │
│  │  nlp │ llm │ gap_analysis │ (visualization dropped)            │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                │                                    │
│                    ┌───────────▼──────────┐                         │
│                    │  SurrealDB (embedded) │                         │
│                    │  + crawl_queue table  │                         │
│                    └──────────────────────┘                         │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Status |
|-----------|----------------|--------|
| `crates/app/` | Leptos components, server functions, routing | New |
| `crates/server/` | Axum entry point, leptos_axum integration, SSE endpoints | New |
| `crates/resyn-core/` | Existing 8 modules moved to shared library crate | Restructured |
| Canvas/WebGL renderer | Force-directed graph at 1000+ nodes in browser | New (JS or WASM) |
| `crawl_queue` DB table | Persistent BFS frontier for resumable crawling | New (DB migration 7) |
| `CrawlService` | Async tokio task spawning BFS workers with queue | New in server |

---

## Recommended Project Structure

The single-crate `research_synergy` binary becomes a cargo workspace. `cargo-leptos` handles the dual server/WASM build.

```
research-synergy/
├── Cargo.toml                    # workspace root
├── Cargo.lock
├── leptos.toml                   # cargo-leptos config
│
├── crates/
│   ├── resyn-core/               # existing logic, server-only
│   │   ├── Cargo.toml            # no wasm32 targets; surrealdb, reqwest, etc.
│   │   └── src/
│   │       ├── lib.rs            # re-exports all existing modules
│   │       ├── data_aggregation/ # unchanged
│   │       ├── database/         # +migration 7 (crawl_queue)
│   │       ├── datamodels/       # +CrawlItem, +api response types
│   │       ├── data_processing/
│   │       ├── nlp/
│   │       ├── llm/
│   │       ├── gap_analysis/
│   │       └── [error, utils, validation]
│   │
│   ├── app/                      # Leptos isomorphic crate (compiles to WASM + native)
│   │   ├── Cargo.toml            # leptos, serde; no native-only deps
│   │   └── src/
│   │       ├── lib.rs            # App component, router
│   │       ├── components/
│   │       │   ├── graph_canvas.rs   # <canvas> wrapper; posts messages to worker
│   │       │   ├── gap_panel.rs      # contradiction + ABC-bridge list
│   │       │   ├── open_problems.rs  # ranked open-problems panel
│   │       │   ├── method_matrix.rs  # method-combination gap matrix
│   │       │   └── crawl_progress.rs # SSE-driven progress bar
│   │       └── server_fns/
│   │           ├── papers.rs     # get_graph_data(), get_gap_findings()
│   │           ├── crawl.rs      # start_crawl(), get_crawl_status()
│   │           └── analysis.rs   # trigger_analysis()
│   │
│   └── server/                   # native binary crate
│       ├── Cargo.toml            # axum, tokio, leptos_axum, resyn-core
│       └── src/
│           ├── main.rs           # Axum router, leptos_axum::generate_route_list
│           ├── api/
│           │   ├── graph.rs      # GET /api/graph → GraphResponse JSON
│           │   └── crawl_sse.rs  # GET /api/crawl/progress → SSE stream
│           └── services/
│               └── crawl_service.rs  # CrawlService (tokio task + queue)
│
├── public/                       # static assets served by Axum
│   └── graph-worker.js           # Web Worker wrapping force layout JS (or WASM)
│
└── src/                          # REMOVED — content migrated to crates/
```

### Structure Rationale

- **crates/resyn-core/**: Isolates all code with native-only dependencies (SurrealDB embedded, reqwest, scraper). Nothing here compiles to WASM. This is the clean boundary.
- **crates/app/**: The isomorphic Leptos crate. Must compile to both `wasm32-unknown-unknown` (browser) and the server's native target. Server functions live here and are called transparently from components.
- **crates/server/**: Thin binary wiring Axum to Leptos SSR and providing custom SSE/REST endpoints the app crate cannot own (because app must be WASM-compatible).
- **public/graph-worker.js**: The force layout runs in a dedicated Web Worker off the main thread, eliminating UI jank during simulation ticks. This is a JS file, not Rust/WASM, for maximum browser compatibility.

---

## Architectural Patterns

### Pattern 1: Server Functions for Data Access

**What:** Leptos `#[server]` functions annotated in `crates/app/` compile to typed RPC stubs on the client and full Rust implementations on the server. The client calls them like regular async functions.

**When to use:** All data fetching that requires SurrealDB access: loading the graph, gap findings, analysis results.

**Trade-offs:** Eliminates manual REST API maintenance. The cost is that server functions must be registered with the Axum router via `leptos_axum::generate_route_list`. Breaking the WASM compilation boundary (e.g., accidentally importing surrealdb in app crate) fails at compile time, which is good.

**Example:**
```rust
// crates/app/src/server_fns/papers.rs
#[server(GetGraphData, "/api")]
pub async fn get_graph_data(paper_id: String, depth: usize) -> Result<GraphResponse, ServerFnError> {
    // server-only: use resyn-core
    use resyn_core::database::queries::PaperRepository;
    let db = use_context::<Arc<Db>>().ok_or_else(|| ServerFnError::new("no db"))?;
    let repo = PaperRepository::new(&db);
    let (papers, edges) = repo.get_citation_graph(&paper_id, depth).await?;
    Ok(GraphResponse::from(papers, edges))
}
```

### Pattern 2: SSE for Crawl Progress Streaming

**What:** Crawl progress (papers fetched, queue depth, current BFS level) streams from server to browser via Server-Sent Events. The Axum handler streams a `tokio::sync::broadcast` channel; the Leptos component subscribes with `leptos_use::use_event_source` or a direct `EventSource` JS binding.

**When to use:** Long-running operations where the user needs progress feedback: crawls at depth 5+, full analysis pipeline runs.

**Trade-offs:** SSE is one-directional and simpler than WebSocket. Sufficient here because the browser only needs to receive progress updates, not send messages during a crawl. Axum's `axum::response::Sse` handles keep-alive automatically.

**Example:**
```rust
// crates/server/src/api/crawl_sse.rs
pub async fn crawl_progress_sse(
    State(tx): State<Arc<broadcast::Sender<CrawlEvent>>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = tx.subscribe();
    let stream = BroadcastStream::new(rx).map(|msg| {
        Ok(Event::default().json_data(msg.unwrap()).unwrap())
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

### Pattern 3: Web Worker + postMessage for Force Layout

**What:** The graph force simulation runs in a dedicated Web Worker (`public/graph-worker.js`). The Leptos graph component (`graph_canvas.rs`) posts the adjacency list to the worker on mount, then receives updated node positions at each simulation tick via `postMessage`. The component writes those positions to a `<canvas>` via `web_sys`.

**When to use:** Any computation that would block the browser main thread for >16ms. Force simulation with 500+ nodes at 60fps qualifies.

**Trade-offs:** JS Web Workers are universally supported. The alternative — running Barnes-Hut in a Rust/WASM thread — requires `SharedArrayBuffer` and cross-origin isolation headers (`COOP`/`COEP`), adding server configuration overhead. The JS worker approach is simpler to deploy. For rendering, a Canvas 2D context (via `web_sys::CanvasRenderingContext2d`) handles up to ~1000 nodes smoothly; switching the draw calls to WebGL2 is a contained change inside the worker when needed.

**Example structure:**
```
graph_canvas.rs (Leptos component)
    on_mount: create Worker("graph-worker.js")
              post { nodes, edges } to worker
    on_message: receive { positions } from worker
                draw nodes + edges to <canvas>

graph-worker.js
    on_message({ nodes, edges }): initialize Barnes-Hut simulation
    setInterval(60fps): tick simulation, postMessage({ positions })
```

### Pattern 4: DB-Backed Crawl Queue (Incremental BFS)

**What:** A `crawl_queue` table in SurrealDB replaces the in-memory `Vec<String>` BFS frontier in `recursive_paper_search_by_references`. Each queue item has a status (`pending` | `in_progress` | `done` | `failed`), the paper ID, the BFS depth level, and a `session_id`. Workers claim items atomically using a SurrealQL `UPDATE ... WHERE status = 'pending' LIMIT 1`.

**When to use:** Any crawl at depth 4+, or when the user wants to resume an interrupted crawl without refetching already-done papers.

**Trade-offs:** Atomic row claim in SurrealDB requires a `RETURN BEFORE` pattern to ensure only one worker claims a given item. Single-embedded SurrealDB instance is not multi-process, so concurrent writes from multiple tokio tasks to the same DB handle are safe (SurrealDB Rust SDK serializes ops on the embedded handle). The persistent queue means re-running the crawl command resumes from where it left off rather than starting over.

**DB Schema (migration 7):**
```surql
DEFINE TABLE IF NOT EXISTS crawl_queue SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS session_id ON crawl_queue TYPE string;
DEFINE FIELD IF NOT EXISTS paper_id   ON crawl_queue TYPE string;
DEFINE FIELD IF NOT EXISTS depth      ON crawl_queue TYPE int;
DEFINE FIELD IF NOT EXISTS status     ON crawl_queue TYPE string;  -- pending|in_progress|done|failed
DEFINE FIELD IF NOT EXISTS enqueued_at ON crawl_queue TYPE string;
DEFINE FIELD IF NOT EXISTS claimed_at  ON crawl_queue TYPE option<string>;
DEFINE INDEX IF NOT EXISTS idx_crawl_status ON crawl_queue FIELDS session_id, status;
```

---

## Data Flow

### Leptos Request Flow (graph load)

```
Browser: user visits /?paper=2503.18887&depth=3
    ↓ Axum SSR renders App component
    ↓ Leptos Resource<GetGraphData> suspends
    ↓ Server function runs on server: PaperRepository::get_citation_graph()
    ↓ Returns GraphResponse { nodes: Vec<NodeData>, edges: Vec<EdgeData> }
    ↓ HTML streamed to browser with embedded JSON
    ↓ WASM hydrates, graph_canvas.rs reads signal
    ↓ posts { nodes, edges } to Web Worker
    ↓ Worker runs force layout ticks
    ↓ postMessage positions back → canvas draw
```

### Incremental Crawl Flow

```
User triggers crawl (via server fn start_crawl)
    ↓ CrawlService::start(session_id, paper_id, max_depth)
    ↓ Insert seed into crawl_queue (status=pending, depth=0)
    ↓ Spawn tokio task loop:
        while pending items exist:
            claim item (UPDATE crawl_queue SET status='in_progress' WHERE status='pending' LIMIT 1)
            fetch paper via PaperSource trait
            upsert paper + citations to DB
            enqueue all arXiv references at depth+1 (if depth < max_depth, not already done/in_progress)
            mark item done
            broadcast CrawlEvent { fetched, queued, depth } via broadcast::Sender
    ↓ SSE endpoint streams CrawlEvents to browser
    ↓ crawl_progress.rs Leptos component updates progress bar
```

### Gap Findings Data Flow (existing → UI)

```
Existing gap_analysis module writes GapFinding records to SurrealDB (unchanged)
    ↓
GetGapFindings server fn: GapFindingRepository::get_all_gap_findings()
    ↓
Leptos signal in gap_panel.rs
    ↓ rendered as sorted list (contradictions, then ABC-bridges)
    ↓ click finding → provenance text segment shown (future: source text lookup)
```

---

## Integration Points: New vs Modified vs Deleted

### New Components

| Component | Location | Interfaces |
|-----------|----------|------------|
| Leptos App crate | `crates/app/` | Depends on `resyn-core` (server side only via server fns) |
| Axum Server binary | `crates/server/` | Wraps leptos_axum; imports resyn-core |
| `graph_canvas.rs` | `crates/app/src/components/` | Communicates with Web Worker via postMessage |
| `gap_panel.rs` | `crates/app/src/components/` | Reads `GetGapFindings` server fn |
| `open_problems.rs` | `crates/app/src/components/` | Reads `GetGapFindings`, ranks by recurrence |
| `method_matrix.rs` | `crates/app/src/components/` | Reads `GetGapFindings` + paper annotations |
| `crawl_progress.rs` | `crates/app/src/components/` | Subscribes to SSE `/api/crawl/progress` |
| `CrawlService` | `crates/server/src/services/` | Spawns tokio task; holds `broadcast::Sender` |
| SSE endpoint | `crates/server/src/api/crawl_sse.rs` | Streams from `broadcast::Receiver` |
| `crawl_queue` table | `database/schema.rs` migration 7 | SurrealDB; read/written by CrawlService |
| `CrawlRepository` | `database/queries.rs` | claim_item, enqueue, mark_done |
| Web Worker | `public/graph-worker.js` | JS Barnes-Hut simulation; postMessage API |

### Modified Components

| Component | Change | Reason |
|-----------|--------|--------|
| `data_aggregation/arxiv_utils.rs` | `recursive_paper_search_by_references` replaced by queue-backed version | Incremental crawl |
| `database/schema.rs` | Add migration 7 for `crawl_queue` | Persistent queue |
| `database/queries.rs` | Add `CrawlRepository` | Queue operations |
| `datamodels/` | Add `GraphResponse`, `NodeData`, `EdgeData`, `CrawlEvent` | API response types (must be Serde + no native-only deps) |
| `Cargo.toml` → `Cargo.toml` (workspace) | Convert to workspace; add crates/app, crates/server, crates/resyn-core | Workspace migration |
| `src/main.rs` | Replaced by `crates/server/src/main.rs` | Axum replaces tokio::main + eframe::run_native |

### Deleted Components

| Component | Replacement |
|-----------|-------------|
| `src/visualization/` (entire module) | Leptos `graph_canvas.rs` component + Web Worker |
| `eframe`, `egui`, `egui_graphs`, `fdg`, `crossbeam` deps | Removed from resyn-core; add `leptos`, `leptos_axum`, `axum`, `web-sys` |
| `src/main.rs` CLI binary | Replaced by `crates/server/src/main.rs` with Axum |

---

## Build Order

Build order respects the dependency chain: core library → DB layer → server functions → UI components → WebGL renderer.

### Step 1: Workspace restructure (no behavior change)

Move `src/` into `crates/resyn-core/src/`. Create workspace `Cargo.toml`. Add minimal `crates/server/src/main.rs` that calls the existing CLI pipeline. All 153 tests still pass. No feature change.

**Deliverable:** Green CI on workspace structure. Confirms no regressions before UI work begins.

### Step 2: Incremental crawl queue (DB migration 7 + CrawlRepository + CrawlService)

Add `crawl_queue` schema, `CrawlRepository` (claim, enqueue, mark_done), and `CrawlService` (tokio task loop). Replace the in-memory BFS in `arxiv_utils.rs`. Add SSE endpoint. Test with a depth-5 crawl that can be interrupted and resumed.

**Deliverable:** CLI `--incremental` flag that uses the persistent queue; progress visible via `curl /api/crawl/progress`.

**Why before Leptos UI:** The crawl service is pure server logic. Validate it independently before adding UI complexity.

### Step 3: Leptos app skeleton + server functions for existing data

Scaffold `crates/app/` with `cargo leptos new --git leptos-rs/start-axum`. Wire up `GetGraphData` and `GetGapFindings` server functions that return JSON for data already in DB. Render a static HTML table of papers to verify SSR + hydration works.

**Deliverable:** Browser shows paper list from SurrealDB via Leptos server function. No graph rendering yet.

### Step 4: Gap analysis UI panels (gap_panel, open_problems, method_matrix)

Implement the three sidebar panels consuming existing gap findings data. These are pure reactive Leptos components reading server function results — no new backend work. Style with Tailwind CSS (cargo-leptos integrates it natively).

**Deliverable:** Browser shows gap findings, open-problems ranking, method matrix. This is the primary v1.1 value-add surface.

### Step 5: Canvas graph renderer + Web Worker

Add `graph_canvas.rs` wrapping a `<canvas>` element. Implement `graph-worker.js` with a Barnes-Hut force simulation (use the existing `d3-force` npm package via a bundled script, or a pure-JS implementation). Wire node positions back to canvas draw calls via postMessage. Add node-click events to show paper detail in a side panel.

**Deliverable:** Interactive force-directed graph in browser. Validates that the worker approach achieves 60fps at 300-node graphs.

### Step 6: WebGL upgrade (conditional)

If Canvas 2D performance is inadequate at the target node count (>500 nodes, <30fps), upgrade the draw calls inside `graph-worker.js` to WebGL2 using `OffscreenCanvas`. The Leptos component is unchanged; only the worker internals change.

**Deliverable:** Graph maintains 60fps at 1000 nodes.

### Step 7: Gap findings wired into graph visualization

Add visual overlays: contradiction edges (red), ABC-bridge edges (orange), badge counts on nodes. The graph worker receives gap findings alongside graph data and emits colored draw commands.

**Deliverable:** Gap findings visible in graph without navigating to a separate panel.

---

## Anti-Patterns

### Anti-Pattern 1: Importing resyn-core directly in app crate

**What people do:** Add `resyn-core` as a dependency of `crates/app/`, then call SurrealDB/reqwest functions from Leptos component code.

**Why it's wrong:** `crates/app/` compiles to WASM. SurrealDB embedded and reqwest use native OS syscalls that have no WASM polyfill. The build fails with linker errors, and the fix is non-trivial.

**Do this instead:** All resyn-core calls live in `#[server]` functions in `crates/app/src/server_fns/`. Server functions are stripped from the WASM bundle automatically by the Leptos macro system. The compile-time boundary is the primary safety mechanism.

### Anti-Pattern 2: Running force layout on the browser main thread

**What people do:** Implement the force simulation in the Leptos component's `create_effect` or `use_interval`, ticking every 16ms.

**Why it's wrong:** JavaScript (and WASM) on the main thread blocks rendering. 500-node Barnes-Hut ticks take 5–20ms each, causing visible jank and dropped frames.

**Do this instead:** Delegate all simulation ticks to a Web Worker. The Leptos component only receives position arrays via `postMessage` and issues canvas draw calls — both are fast (<1ms).

### Anti-Pattern 3: Rewriting BFS from scratch for the queue-backed version

**What people do:** Discard `recursive_paper_search_by_references` entirely and write a new queue-driven BFS that duplicates the PaperSource trait dispatch and rate-limiting logic.

**Why it's wrong:** The existing BFS already handles visited-set deduplication, version-suffix stripping, and `PaperSource` trait dispatch correctly. Duplicating it introduces divergence and doubles test surface.

**Do this instead:** Extract the inner fetch-and-enqueue logic from `recursive_paper_search_by_references` into a `process_crawl_item(item: CrawlItem, source: &mut dyn PaperSource, db: &Db)` function. The queue-backed loop calls this function; the existing in-memory BFS (kept for tests) also calls it. Single implementation, two drivers.

### Anti-Pattern 4: SSE for the graph data payload

**What people do:** Stream the graph JSON through the SSE crawl progress endpoint rather than through a server function.

**Why it's wrong:** SSE is a unidirectional text stream designed for incremental events, not for returning a large structured payload. Graph JSON (potentially 1MB+ for 1000 nodes) should be a single HTTP response, benefiting from compression and browser caching.

**Do this instead:** `GetGraphData` server function returns the full graph as a single typed response. SSE carries only lightweight progress events (paper fetched, queue depth, current depth level). Keep concerns separated.

### Anti-Pattern 5: Replacing the Leptos SSR app with a pure SPA

**What people do:** Configure Leptos in CSR-only mode (no SSR), serve a blank `index.html`, and load everything via client-side data fetching.

**Why it's wrong:** CSR-only means the initial page load shows nothing until WASM downloads (~500KB+ gzipped) and executes. SSR with hydration gives an immediately-useful HTML page (paper list, gap count) while the WASM finishes loading. For a research tool opened infrequently, first-load UX matters.

**Do this instead:** Use the standard `cargo leptos` SSR+hydration mode with `leptos_axum`. The cost is a slightly more complex build setup, which cargo-leptos fully manages.

---

## Integration Boundaries

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| arXiv API | Unchanged via `ArxivSource` in resyn-core | Rate limit 3s, no change |
| InspireHEP API | Unchanged via `InspireHepClient` in resyn-core | Rate limit 350ms, no change |
| Claude / Ollama | Unchanged via `LlmProvider` trait in resyn-core | Called from server only |
| ar5iv HTML | Unchanged via `Ar5ivExtractor` in resyn-core | No WASM boundary contact |

### Internal Module Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `app` ↔ `resyn-core` | Server functions only (no direct import from WASM) | Compile-time enforced |
| `server` ↔ `resyn-core` | Direct Rust imports (both native) | Standard |
| `graph_canvas.rs` ↔ `graph-worker.js` | `postMessage` (structured clone) | Types: `{ nodes, edges }` in, `{ positions }` out |
| `CrawlService` ↔ SSE endpoint | `tokio::sync::broadcast` channel | Unbounded capacity; buffer 256 events |
| `CrawlService` ↔ SurrealDB | `CrawlRepository` (claim/enqueue/mark_done) | Atomic claim via SurrealQL `UPDATE ... LIMIT 1` |
| Server functions ↔ SurrealDB | Via `Arc<Db>` in Axum state, extracted with `use_context()` | Leptos pattern for sharing server state |

---

## Scalability Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 50–300 nodes (typical crawl depth 3) | Canvas 2D renderer in Web Worker is sufficient |
| 300–1000 nodes (depth 5–7) | Switch Web Worker draw calls to WebGL2 via OffscreenCanvas |
| 1000+ nodes (depth 8+) | Add level-of-detail: render clusters at far zoom, expand on zoom-in; node clustering by citation community |
| Crawl queue concurrency | Single embedded SurrealDB allows safe concurrent tokio tasks; scale to ~4 parallel fetch tasks before hitting arXiv rate limits |

### Scaling Priorities

1. **First bottleneck:** Force layout CPU time at >500 nodes. Fix: Web Worker already decouples from main thread; switch to Barnes-Hut O(n log n) in the worker.
2. **Second bottleneck:** Canvas draw calls at >1000 nodes (~60ms per frame). Fix: WebGL2 instanced rendering via OffscreenCanvas; batch nodes into typed arrays.

---

## Sources

- [Leptos documentation — server functions](https://book.leptos.dev/) — HIGH confidence (official)
- [leptos_axum crate docs](https://docs.rs/leptos_axum/latest/leptos_axum/) — HIGH confidence (official)
- [cargo-leptos build tool](https://github.com/leptos-rs/cargo-leptos) — HIGH confidence (official)
- [Leptos start-axum workspace template](https://github.com/leptos-rs/start-axum) — HIGH confidence (official)
- [Leptos 0.8.0 release — WebSocket server functions](https://github.com/leptos-rs/leptos/releases/tag/v0.8.0) — HIGH confidence (official)
- [axum::response::Sse](https://docs.rs/axum/latest/axum/response/sse/) — HIGH confidence (official)
- [leptos_use use_event_source](https://leptos-use.rs/network/use_event_source.html) — MEDIUM confidence (WebSearch, official leptos-use docs)
- [Canvas 2D vs WebGL performance benchmark 2025](https://www.svggenie.com/blog/svg-vs-canvas-vs-webgl-performance-2025) — MEDIUM confidence (WebSearch)
- [Graph visualization performance comparison (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC12061801/) — MEDIUM confidence (academic, peer-reviewed)
- [Web Workers postMessage API](https://developer.mozilla.org/en-US/docs/Web/API/Worker/postMessage) — HIGH confidence (MDN official)
- [wgpu cross-platform Rust graphics](https://wgpu.rs/) — HIGH confidence (official); noted as alternative to Canvas/WebGL if full WASM renderer needed
- [SurrealDB Rust SDK concurrency](https://surrealdb.com/docs/sdk/rust/concepts/concurrency) — HIGH confidence (official)
- [Leptos + SurrealDB + Axum example](https://github.com/oxide-byte/rust-berlin-leptos) — MEDIUM confidence (community, Leptos 0.6)

---

*Architecture research for: ReSyn v1.1 — Leptos web UI, WebGL graph, incremental crawling*
*Researched: 2026-03-15*
