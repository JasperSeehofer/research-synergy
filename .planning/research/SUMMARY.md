# Project Research Summary

**Project:** Research Synergy (ReSyn) — v1.1 "Scale & Surface" Milestone
**Domain:** Literature Based Discovery (LBD) — Leptos web migration, WebGL graph visualization, incremental crawling, enriched gap analysis UI
**Researched:** 2026-03-15
**Confidence:** HIGH (stack choices, architecture patterns), MEDIUM (WASM/JS interop specifics, Barnes-Hut convergence tuning)

## Executive Summary

ReSyn v1.1 is a migration and surface-area milestone: the egui desktop binary becomes a Leptos-powered browser app, the force-directed graph moves to WebGL to handle 1000+ nodes, and the BFS crawler gains persistent resumability via a SurrealDB-backed queue. The underlying analysis pipeline (TF-IDF, LLM annotation, contradiction detection, ABC-bridge) is unchanged and already produces `GapFinding` records in SurrealDB — v1.1 wires that data into the primary UI rather than routing it to stdout. The key insight from research is that ReSyn's differentiator is analysis, not graph visualization: competitors (Connected Papers, Litmaps, ResearchRabbit) are citation-map tools; ReSyn uses citation maps as scaffolding for structural gap discovery. None of those competitors surface contradictions, ABC-bridge connections, or open-problems aggregations.

The recommended approach is a Cargo workspace split into three crates: `resyn-core` (existing 8-module native library), `app` (Leptos isomorphic crate, compiles to both WASM and native), and `server` (Axum binary). Leptos server functions provide a type-safe, compile-time-enforced boundary between browser WASM and native-only code (SurrealDB, reqwest). Graph rendering uses sigma.js 3.0 + graphology 0.26 in a JavaScript Web Worker, with ForceAtlas2 for Barnes-Hut O(n log n) layout — avoiding the JS-WASM boundary overhead of a pure-Rust WebGL path. The incremental crawl queue requires only a new SurrealDB schema migration and no new Rust crates.

The single most dangerous risk is the WASM compilation boundary: SurrealDB embedded (kv-surrealkv, kv-mem) uses native OS primitives that will not compile to wasm32-unknown-unknown. This must be addressed in the workspace restructure — before any Leptos component code is written — by feature-gating SurrealDB behind an `ssr` Cargo feature and confining all DB access to `#[server]` functions. A secondary risk is force layout convergence at scale: naive Barnes-Hut without a cooling schedule oscillates indefinitely at 1000+ nodes. Both risks are preventable at the design stage with well-understood mitigations documented in official sources.

## Key Findings

### Recommended Stack

The v1.1 stack replaces egui/eframe/fdg with Leptos 0.8 (CSR mode via Trunk) + Axum 0.8 on the server side, and sigma.js 3.0 + graphology 0.26 + graphology-layout-forceatlas2 0.10 for graph visualization in the browser. CSR with Trunk is preferred over SSR cargo-leptos because ReSyn is a single-user local tool: SEO, TTFB, and hydration complexity are irrelevant, and CSR iteration cycles are faster. The Axum server is necessary regardless — it exposes the analysis pipeline as a REST API and serves the Trunk-built WASM bundle via `tower-http::ServeDir`. Incremental crawling requires no new crate: only a `crawl_queue` SurrealDB table and control-flow changes in `arxiv_utils.rs`.

**Core technologies:**
- `leptos 0.8` (CSR via Trunk): Reactive Rust/WASM frontend — fine-grained reactivity, production-ready (1.8M+ downloads), Rust all the way through
- `axum 0.8` + `tower-http 0.6`: REST API server and static bundle serving — Tokio-native, Leptos's recommended server, zero friction with existing runtime
- `sigma.js 3.0` + `graphology 0.26`: WebGL graph renderer + graph data model — required pairing; WebGL-native, handles 1000+ nodes at interactive framerates with built-in spatial indexing
- `graphology-layout-forceatlas2 0.10`: Barnes-Hut O(n log n) force layout in Web Worker — replaces fdg's O(n²) Fruchterman-Reingold; designed specifically for academic network graphs (Gephi heritage)
- `leptos_axum 0.8`: Server function routing — typed RPC stubs auto-generated for client; must match leptos version exactly (same 0.8.x patch)
- `js-sys 0.3` + `web-sys 0.3`: WASM-to-JS boundary glue for sigma.js interop and canvas access
- Tailwind CSS 4: UI styling — Trunk integrates Tailwind CLI automatically; no Node.js required
- `trunk` (CLI, not crate): WASM build tool — handles wasm-pack, asset bundling, live reload; installed once via `cargo install trunk`

**Key version constraints:** leptos + leptos_axum must share the same 0.8.x patch; axum 0.8 requires tower-http 0.6 (not 0.5); sigma.js 3.x requires graphology 0.25+.

**Removals:** egui, eframe, egui_graphs, fdg, crossbeam — none compile cleanly to WASM and are replaced entirely.

### Expected Features

ReSyn v1.1 targets researchers running depth-10+ crawls where the existing egui binary becomes impractical (no browser access, graph degrades above 300 nodes, gap findings buried in stdout).

**Must have (table stakes for v1.1 launch):**
- Tech debt cleanup: wire existing gap findings into egui before migration; remove stale stubs — ensures no analysis data is lost in the transition
- Incremental/resumable crawl with DB-backed queue — depth 10+ at 3s rate limits takes 30+ min; a crash without checkpoint means restarting from zero
- Leptos web migration (shell + routing) — foundational; every other web feature depends on this
- Graph rendering in browser (canvas initially, WebGL when needed) — the migration is not usable without it
- Crawl progress via SSE — depth-10 crawl appears frozen without visibility; depth-5+ is the realistic threshold
- Paper detail sidebar — click-a-node interaction is the minimum expected graph interaction
- Gap findings surfaced in graph (contradiction edges red, bridge badges orange) — the core v1.1 promise: move gap insights into the primary interface

**Should have (v1.1 polish, add after core works):**
- WebGL renderer upgrade — swap canvas for WebGL when 1000+ node performance is observed to degrade; don't optimize prematurely
- Open-problems aggregation panel — data already in SurrealDB from v1.0 `LlmAnnotation`; high analytical value, low implementation risk
- Method-combination gap matrix — heatmap of untried method pairings; analytically unique vs all competitors
- Temporal filtering slider — `published` field already exists on `Paper`; straightforward Leptos signal
- URL-addressable graph state — Leptos router query params (`?paper=2301.12345&depth=3`); enables sharing

**Defer (v1.2+):**
- Node clustering / level-of-detail — valuable at 1000+ nodes but requires real usage data to tune clustering heuristics
- Analysis provenance (source text segments with click-to-highlight) — schema additions + extraction re-runs; defer until pipeline stabilizes post-migration
- Section-aware LLM extraction — risks cache-invalidating existing `LlmAnnotation` records; defer until extraction pipeline is stable
- 3D visualization — analytically inferior to 2D WebGL with good LOD; no demonstrated demand

**Anti-features to reject:** multi-user collaboration, full-text keyword search (becomes a search engine, not a gap analyzer), auto-expanding by similarity, live paper alerts.

### Architecture Approach

The v1.1 architecture is a Cargo workspace of three crates sharing a single embedded SurrealDB instance. The non-negotiable boundary: `crates/resyn-core/` contains all native-only code and never targets WASM; `crates/app/` is isomorphic (WASM + native) and accesses resyn-core exclusively through Leptos `#[server]` functions (compile-time enforced — the Leptos macro system strips server function bodies from the WASM binary); `crates/server/` is the thin Axum binary that wires these together and owns SSE streaming endpoints. Graph visualization lives in a JavaScript Web Worker (`public/graph-worker.js`) running Barnes-Hut via ForceAtlas2 and posting position arrays to the Leptos canvas component via postMessage — this keeps force computation off the browser main thread without requiring `SharedArrayBuffer` COOP/COEP headers.

**Major components:**
1. `crates/resyn-core/` — existing 8-module analysis library, server-only, no WASM targets; extended with `CrawlRepository` + `crawl_queue` schema (migration 7)
2. `crates/app/` — Leptos components (`graph_canvas`, `gap_panel`, `open_problems`, `method_matrix`, `crawl_progress`) + server functions (`GetGraphData`, `GetGapFindings`, `StartCrawl`); compiles to both WASM and native
3. `crates/server/` — Axum entry point, `CrawlService` (tokio task loop + broadcast::Sender), SSE endpoint, leptos_axum routing, static file serving via tower-http ServeDir
4. `public/graph-worker.js` — ForceAtlas2 Barnes-Hut simulation; postMessage API: `{ nodes, edges }` in, `{ positions }` out each tick; sigma.js handles WebGL rendering on the canvas
5. `crawl_queue` SurrealDB table (migration 7) — persistent BFS frontier with `pending | in_progress | done | failed` states; atomic claim via `UPDATE ... WHERE status = 'pending' LIMIT 1`

**Key patterns to follow:**
- Server functions only for SurrealDB access — compile-time enforced, not a convention
- SSE for crawl progress events (lightweight); server function for graph payload (single typed response with HTTP compression)
- Atomic queue item claim prevents duplicate concurrent fetches without external locking
- Offline force layout precomputation stored in SurrealDB as primary path; interactive simulation as enhancement

**Anti-patterns to avoid:**
- Importing resyn-core directly in app crate (WASM linker failures)
- Running force simulation on browser main thread (UI jank at 500+ nodes)
- Rewriting BFS from scratch (extract inner logic, reuse PaperSource trait dispatch)
- Using SSE for the graph data payload (wrong tool; use single server function response)

### Critical Pitfalls

1. **SurrealDB embedded not WASM-compilable** — feature-gate surrealdb as `optional = true` behind `ssr` Cargo feature on day one of workspace restructure; verify with `cargo leptos build --release` before writing any Leptos component code; warning signs: compile errors mentioning `mio`, `timerfd`, or OS-level I/O

2. **Leptos SSR hydration mismatches** — access browser APIs only inside `Effect::new(|| { ... })`, never in component body; avoid `cfg!()` inside view macros for structural branches; add `<tbody>` explicitly in all table elements; set up an SSR integration test before shipping the first component

3. **JS-WASM boundary overhead in render loop** — use `js_sys::Float32Array` backed by Rust `Vec<f32>` for node positions (pointer, not copy); batch all GPU buffer updates in a single `gl.buffer_sub_data` call; never call `JSON.stringify` in the animation frame path; warning: frame rate degrades linearly with node count before GPU is saturated

4. **Force layout oscillation at scale** — implement a cooling schedule (geometrically decreasing temperature per tick); apply ForceAtlas2 anti-swinging (braking force when displacement direction reverses); precompute layout offline and store positions in SurrealDB as the primary path; warning: graph visibly wiggles without settling after 10+ seconds

5. **Duplicate crawl writes under concurrency** — design the DB queue state machine before writing any crawler code; use `INSERT IF NOT EXISTS` when enqueueing; share a single `Arc<RateLimiter>` across all concurrent fetch tasks (per-task rate limiting violates the global arXiv rate limit); on restart, reclaim all `in_progress` items back to `pending`

6. **`usize`/`isize` crossing the 32-bit WASM boundary** — all server function argument and return types must use fixed-width integers (`u32`, `i64`, never `usize`); convert petgraph `NodeIndex` (wraps `usize`) to `u32` at the API boundary; add a CI grep lint on `#[server]` function signatures — explicitly called out in Leptos 0.8.0 release notes

## Implications for Roadmap

The feature dependency graph is unambiguous about ordering: workspace restructure is a prerequisite for everything; incremental crawl can be validated CLI-only before any Leptos work; gap analysis panels are data-ready and can be built in parallel with the graph renderer once the Leptos shell exists.

### Phase 1: Workspace Restructure and WASM Boundary Setup

**Rationale:** The single-crate structure is incompatible with WASM compilation of any frontend code. This is the foundational change every other phase depends on. Establishing it with zero behavior change — all 44 existing tests pass — is the validation signal before any new work begins. Pitfalls 1 and 6 must be addressed here or they will corrupt every subsequent phase.
**Delivers:** Cargo workspace (`resyn-core`, `app`, `server` crates); `ssr`/`hydrate` Cargo feature flags; SurrealDB feature-gated behind `ssr`; server function type audit (no `usize`); tech debt cleanup (gap findings wired in egui, stale stubs removed); CI confirms green before migration begins
**Addresses:** Tech debt cleanup (P1)
**Avoids:** Pitfall 1 (SurrealDB WASM compile failure), Pitfall 6 (usize boundary — establish API types here)

### Phase 2: Incremental Crawl with DB-Backed Queue

**Rationale:** The crawl queue is pure server logic with no UI dependency. Validating it independently via CLI before adding Leptos complexity reduces risk surface. A depth-10 crawl that cannot resume is a blocker for real research use regardless of UI quality. Rate limiting under concurrency (Pitfall 5) must be designed here before parallel fetch tasks are introduced.
**Delivers:** `crawl_queue` SurrealDB table (migration 7); `CrawlRepository` (claim/enqueue/mark_done); `CrawlService` (tokio task loop); CLI `--incremental` flag; SSE endpoint (curl-accessible even before UI); shared `Arc<RateLimiter>` across fetch tasks
**Addresses:** Resumable crawl (P1 table stakes)
**Avoids:** Pitfall 5 (duplicate writes — state machine designed here under no UI pressure)

### Phase 3: Leptos Web Shell + Gap Analysis Panels

**Rationale:** The analysis panels (gap findings, open problems, method matrix) are data-ready — `GapFinding` and `LlmAnnotation` records already exist in SurrealDB from v1.0. Building these as the first Leptos UI validates SSR/hydration, server functions, and Axum integration with low rendering complexity — before tackling the harder graph canvas work. This produces immediate user-visible value: the gap analysis surface, which is the core differentiator.
**Delivers:** Leptos app skeleton; Axum serving WASM bundle; URL routing with query params; paper list from SurrealDB via server function; gap findings panel (contradictions + bridges); open-problems aggregation panel; method-combination gap matrix; paper detail sidebar; crawl progress bar wired to SSE
**Uses:** `leptos 0.8`, `leptos_axum 0.8`, `axum 0.8`, Tailwind CSS 4
**Addresses:** Leptos web migration (P1), paper detail sidebar (P1), crawl progress UI (P1), open-problems panel (P2), method matrix (P2), URL-addressable state (P2)
**Avoids:** Pitfall 2 (hydration mismatch — SSR integration test established here before shipping any component)

### Phase 4: Graph Renderer in Browser (Canvas, then WebGL)

**Rationale:** The graph canvas component is the most technically complex piece of v1.1. Starting with Canvas 2D in the Web Worker validates the postMessage architecture before committing to WebGL complexity. The WebGL upgrade via sigma.js is a contained change when 500+ node performance becomes an observed problem — the Leptos component is unchanged; only the worker internals change.
**Delivers:** `graph_canvas.rs` Leptos component wrapping `<canvas>`; `graph-worker.js` with ForceAtlas2 Barnes-Hut force simulation; node-click events wired to paper detail sidebar; contradiction edges (red) and ABC-bridge edges (orange/dashed) overlaid on graph; temporal filtering slider
**Uses:** sigma.js 3.0, graphology 0.26, graphology-layout-forceatlas2 0.10, js-sys, web-sys
**Addresses:** Graph rendering in browser (P1), gap findings in graph (P1), WebGL renderer upgrade (P2), temporal filtering (P2)
**Avoids:** Pitfall 3 (JS-WASM boundary overhead — memory model defined before render loop); Pitfall 4 (force layout oscillation — cooling schedule and offline precompute from the start)

### Phase 5: Polish and v1.2 Preparation

**Rationale:** Node clustering/LOD and analysis provenance are deferred because clustering heuristics require real usage data from real-scale crawls, and provenance storage risks cache-invalidating existing `LlmAnnotation` records. After v1.1 is stable, these are the highest-value additions.
**Delivers:** Node clustering / level-of-detail at 1000+ nodes; analysis provenance (source text segments with click-to-highlight); WASM binary size optimization (`wasm-opt`, `opt-level = 'z'`); verify release binary < 5MB
**Addresses:** P3 features from prioritization matrix

### Phase Ordering Rationale

- **Workspace first:** The WASM build boundary must exist before any code crosses it. Zero feature change with all tests green is the only safe starting point.
- **Crawl before UI:** Incremental crawl is independently testable and represents the highest user-visible risk (lost work). Validating it before Leptos complexity means bugs are easier to isolate.
- **Panels before graph:** Gap analysis panels are data-ready and test the server function pattern end-to-end with low rendering complexity. The graph renderer is the hardest piece — build it last when the rest of the stack is validated.
- **Canvas before full WebGL:** The Web Worker + postMessage + draw call architecture is the same for both. Canvas 2D validates the architecture; WebGL is a contained upgrade inside the worker.
- **Pitfall sequencing:** Pitfalls 1 and 6 (WASM boundary) addressed in Phase 1; Pitfall 5 (duplicate writes) in Phase 2; Pitfall 2 (hydration) in Phase 3; Pitfalls 3 and 4 (rendering performance) in Phase 4. Each phase addresses its pitfalls before they can affect downstream phases.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2 (Incremental Crawl):** Validate SurrealDB embedded `UPDATE ... LIMIT 1` atomicity semantics in practice — official concurrency docs confirm embedded operations are serialized, but the specific claim queue pattern with concurrent tokio tasks needs a concurrency test before committing the design. MEDIUM confidence source.
- **Phase 3 (Leptos Shell):** Tailwind CSS 4 + Trunk integration — the community guide used as source is MEDIUM confidence. Run a spike on `LEPTOS_TAILWIND_VERSION` config before building out the full component library. One session, not a full research pass.
- **Phase 4 (Graph Renderer):** Two items need spikes before full implementation: (1) sigma.js + Leptos NodeRef canvas initialization in `use_effect` — confirmed conceptually in community discussion but not in official docs; (2) ForceAtlas2 cooling schedule parameters for citation graph topology (hub-and-spoke patterns typical in physics arXiv may require tuned `theta` and `scalingRatio`). A 500-node real graph benchmark is the validation target.

Phases with well-documented standard patterns (skip research):
- **Phase 1 (Workspace Restructure):** Cargo workspace + leptos_axum Cargo feature flags are extensively documented in official Leptos docs and the start-axum template. No research needed.
- **Phase 5 (Polish):** WASM binary size optimization has official Leptos guidance with specific flags. Provenance storage is a schema extension of known SurrealDB patterns.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Leptos 0.8, Axum 0.8, sigma.js 3.0 all verified via official sources. Version compatibility matrix confirmed. JS interop approach (wasm-bindgen + NodeRef) confirmed via Leptos book and community discussion. CSR vs SSR decision well-reasoned for single-user tool. |
| Features | MEDIUM-HIGH | P1 feature set grounded in explicit feature dependency analysis. Competitor comparison from Effortless Academic 2025 is MEDIUM confidence but directionally consistent with known tool capabilities. Anti-feature decisions are conservative and well-justified. |
| Architecture | HIGH | Cargo workspace structure follows official leptos-rs/start-axum template. SSE pattern from official Axum docs. Web Worker postMessage pattern from MDN. SurrealDB concurrency from official SDK docs. Server function boundary is compile-time enforced — not a convention. |
| Pitfalls | MEDIUM-HIGH | WASM boundary pitfalls (1, 2, 6) are from official Leptos docs with explicit callouts in 0.8.0 release notes. Force layout convergence (Pitfall 4) and JS-WASM overhead (Pitfall 3) from MEDIUM confidence sources (Rust WASM book, academic papers). Crawl queue concurrency (Pitfall 5) from SurrealDB official concurrency docs. |

**Overall confidence:** HIGH for the workspace restructure and server architecture; MEDIUM for the graph rendering details (sigma.js/Leptos integration specifics) and crawl queue atomicity guarantees. These gaps are bounded and addressable with targeted spikes rather than full research passes.

### Gaps to Address

- **SurrealDB `UPDATE ... LIMIT 1` atomicity in embedded mode:** Official concurrency docs confirm embedded SurrealDB serializes ops on the same handle, but the specific claim queue pattern under concurrent tokio tasks needs a concurrency test (not a research pass) before Phase 2 commits the design.
- **sigma.js + Leptos NodeRef integration:** The pattern is documented generically in the Leptos book; sigma.js-specific initialization in `use_effect` is confirmed in a community discussion thread (MEDIUM). A working spike precedes full Phase 4 implementation.
- **ForceAtlas2 convergence for citation graph topology:** Physics arXiv citation graphs have hub-and-spoke structure (a few papers with 100+ citations alongside many with 2-5). ForceAtlas2 cooling parameters from the original Gephi paper may need tuning. The Phase 4 spike validates convergence on a real 500-node graph before committing the architecture.
- **Tailwind CSS 4 + Trunk integration:** The `LEPTOS_TAILWIND_VERSION` Trunk config approach is from a community source. Validate in a 30-minute spike before styling Phase 3 components.

## Sources

### Primary (HIGH confidence)
- [Leptos Book](https://book.leptos.dev/) — server functions, hydration bugs, WASM binary size, wasm-bindgen integration
- [Leptos 0.8.0 Release Notes](https://github.com/leptos-rs/leptos/releases/tag/v0.8.0) — breaking changes, usize/isize pitfall explicitly called out
- [leptos-rs/start-axum template](https://github.com/leptos-rs/start-axum) — workspace structure reference implementation
- [leptos_axum docs.rs](https://docs.rs/leptos_axum/latest/leptos_axum/) — Axum integration, route list generation, server function registration
- [sigma.js GitHub v3.0.2](https://github.com/jacomyal/sigma.js/) — WebGL renderer, graphology requirement, API surface
- [sigma.js v3.0 release announcement](https://www.ouestware.com/2024/03/21/sigma-js-3-0-en/) — breaking changes from v2, production status
- [graphology-layout-forceatlas2 docs](https://graphology.github.io/standard-library/layout-forceatlas2.html) — Barnes-Hut parameters, Web Worker support
- [axum::response::Sse docs.rs](https://docs.rs/axum/latest/axum/response/sse/) — SSE streaming pattern, keep-alive
- [Web Workers postMessage MDN](https://developer.mozilla.org/en-US/docs/Web/API/Worker/postMessage) — structured clone semantics, OffscreenCanvas
- [SurrealDB SDK Concurrency Docs](https://surrealdb.com/docs/sdk/rust/concepts/concurrency) — embedded engine concurrency model
- [Rust WASM Book: Game of Life](https://rustwasm.github.io/book/game-of-life/implementing.html) — typed array / linear memory patterns for JS-WASM boundary overhead avoidance

### Secondary (MEDIUM confidence)
- [Leptos canvas/NodeRef community discussion](https://github.com/leptos-rs/leptos/discussions/2245) — NodeRef pattern for canvas confirmed; sigma.js specifics not covered
- [Leptos + SurrealDB + Axum example](https://github.com/oxide-byte/rust-berlin-leptos) — real integration reference (Leptos 0.6; patterns still applicable)
- [Webcola + WASM graph layout case study](https://cprimozic.net/blog/speeding-up-webcola-with-webassembly/) — JS-WASM boundary overhead in graph rendering quantified empirically
- [ForceAtlas2 paper](https://medialab.sciencespo.fr/publications/Jacomy_Heymann_Venturini-Force_Atlas2.pdf) — anti-swinging mechanism and cooling schedule design
- [Barnes-Hut algorithm reference](https://arborjs.org/docs/barnes-hut) — theta parameter and convergence behavior
- [Litmaps vs ResearchRabbit vs Connected Papers 2025](https://effortlessacademic.com/litmaps-vs-researchrabbit-vs-connected-papers-the-best-literature-review-tool-in-2025/) — competitor feature analysis
- [Graph visualization performance comparison (PMC 2025)](https://pmc.ncbi.nlm.nih.gov/articles/PMC12061801/) — canvas vs WebGL performance at scale
- [leptos_use use_event_source](https://leptos-use.rs/network/use_event_source.html) — SSE client subscription pattern

### Tertiary (MEDIUM-LOW confidence)
- [Leptos 0.8 + Tailwind 4 + DaisyUI 5 guide](https://8vi.cat/leptos-0-8-tailwind4-daisyui5-for-easy-websites/) — Tailwind + Trunk integration via `LEPTOS_TAILWIND_VERSION`; community source, needs validation
- [LogRocket: Migrating JS frontend to Leptos](https://blog.logrocket.com/migrating-javascript-frontend-leptos-rust-framework/) — practical migration experience; version not specified

---
*Research completed: 2026-03-15*
*Ready for roadmap: yes*
