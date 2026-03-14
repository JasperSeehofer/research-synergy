# Pitfalls Research

**Domain:** Rust LBD app — adding Leptos web UI, WebGL/Barnes-Hut graph rendering, and incremental crawling to existing egui/SurrealDB stack
**Researched:** 2026-03-15
**Confidence:** MEDIUM — Leptos/WASM specifics verified via official docs; Barnes-Hut implementation and crawl queue patterns from community sources; SurrealDB concurrent access from official docs

---

## Critical Pitfalls

### Pitfall 1: SurrealDB Embedded Engine Not Compilable to WASM

**What goes wrong:**
The existing codebase uses SurrealDB v3 with the `kv-surrealkv` feature for local persistence and `kv-mem` for tests. Both features depend on native OS primitives (mio, file I/O, OS threads). When cargo-leptos compiles the client WASM target, it attempts to compile all workspace dependencies including SurrealDB, which fails with linker errors or panics because `mio` and file-backed storage are not available in the WASM sandbox.

**Why it happens:**
cargo-leptos builds two binaries: a native server binary (SSR) and a WASM client binary (CSR/hydration). Dependencies that are fine on the server side will silently pull into the WASM build unless explicitly gated. SurrealDB's embedded storage backends are not WASM-compatible. The `kv-mem` feature uses `tokio` internals that include thread-based primitives; the Leptos book explicitly notes that crates with `mio` cannot compile to WASM.

**How to avoid:**
- Gate all SurrealDB imports behind `#[cfg(not(target_arch = "wasm32"))]` or behind a Cargo feature that is only enabled in the `ssr` feature set.
- Move all database access exclusively into server functions (`#[server]`) — these are compiled out of the WASM binary automatically by Leptos.
- In `Cargo.toml`, mark surrealdb as an optional dependency enabled only by `ssr`:
  ```toml
  [features]
  ssr = ["dep:surrealdb", "leptos/ssr", "leptos_axum"]
  hydrate = ["leptos/hydrate"]
  ```
- Never import `crate::database` from any module that is also used in client-side component code.

**Warning signs:**
- Compilation error mentioning `mio`, `timerfd`, or OS-level I/O when building WASM target
- `cargo leptos build` succeeds but `cargo leptos serve` fails on the WASM pass
- Any `use crate::database::*` appearing in a file without a `cfg(not(target_arch = "wasm32"))` guard

**Phase to address:** Web migration phase — define the server/client boundary before writing any Leptos components.

---

### Pitfall 2: Leptos SSR Hydration Mismatch Causing Silent Breakage

**What goes wrong:**
The server renders HTML and sends it to the browser. The WASM-compiled client then "hydrates" the page by attaching event listeners and reactive state to the existing DOM. If the server renders different HTML than the client would render — even by one attribute or one text node — hydration fails silently in release mode (the UI appears frozen, some events don't fire) or panics in debug mode.

**Why it happens:**
Three common causes in this codebase specifically:
1. Calling `js_sys` or `web_sys` browser APIs during component rendering on the server side (they don't exist server-side).
2. Using `cfg!(target_arch = "wasm32")` inside a component's view macro — the server renders the server branch, the client renders the client branch, and they diverge.
3. Missing `<tbody>` in table elements — browsers auto-insert `<tbody>`, creating a node the client view didn't produce.

**How to avoid:**
- Access browser APIs only inside `Effect::new(|| { ... })` — effects are client-only, never run during SSR.
- Never use `cfg!` inside view macros for structural changes. Use `<Show when=...>` with a reactive signal instead.
- Explicitly include `<tbody>` when rendering any table.
- Validate rendered HTML output with W3C Validator during development.
- Enable Leptos's console-error-panic-hook in WASM for clear panic messages during development.

**Warning signs:**
- UI renders correctly on first load but interactive elements (buttons, graph clicks) don't respond
- Browser console shows hydration mismatch warnings
- Works in CSR-only mode but breaks with SSR enabled

**Phase to address:** Web migration phase — set up an SSR integration test early that renders a component and checks the DOM structure.

---

### Pitfall 3: JS-WASM Boundary Overhead Killing Graph Render Performance

**What goes wrong:**
The WebGL graph renderer requires per-frame data: node positions, edge coordinates, color arrays. If this data is serialized across the JS-WASM boundary every animation frame (via wasm-bindgen's default string/JSON serialization), the overhead dominates render time and caps frame rate far below what the GPU could achieve. At 500 nodes and 60 FPS, even 1ms of serialization overhead per frame becomes 60ms/s of wasted CPU time.

**Why it happens:**
wasm-bindgen's default behavior converts Rust types to JS-compatible types by copying and serializing. Developers use `#[wasm_bindgen]` to expose functions that accept `Vec<f32>` and return `JsValue`, not realizing that each call allocates and copies the entire data structure into the JS heap.

**How to avoid:**
- Keep all graph position data as typed arrays allocated directly in the WASM linear memory (`js_sys::Float32Array` backed by a `Vec<f32>` in Rust) and pass a view/pointer to WebGL buffer uploads rather than copying.
- Call WebGL methods from Rust via `web_sys` directly — avoid the JS intermediary for hot paths.
- The simulation loop (Barnes-Hut force calculation + Verlet integration) runs entirely in Rust. Only the final positions array is transferred to WebGL via a typed array buffer view.
- Batch all GPU buffer updates in a single `gl.buffer_sub_data` call per frame rather than one call per node.

**Warning signs:**
- Frame rate degrades linearly with node count even before GPU is saturated
- Chrome DevTools shows large "Script" time in the frame flame graph, not "GPU" time
- `JSON.stringify` or `JSON.parse` appears in the call stack during animation frames

**Phase to address:** WebGL renderer phase — establish the memory model before writing any rendering code.

---

### Pitfall 4: Force Layout Oscillation and Convergence Failure at Scale

**What goes wrong:**
Barnes-Hut force simulation with a fixed timestep oscillates indefinitely rather than converging when the graph has nodes with widely different degree (e.g., a hub paper with 200 citations alongside isolated papers with 2). The hub node accumulates large force vectors and overshoots its equilibrium position each tick. With a fixed learning rate / timestep, the layout never stabilizes and the graph visibly jitters.

**Why it happens:**
Fruchterman-Reingold and naive Barnes-Hut both require a cooling schedule — the effective force magnitude must decrease over simulation time. Implementing the quadtree without the cooling schedule produces oscillating behavior. At 1000+ nodes the problem becomes visible to users immediately.

**How to avoid:**
- Implement a temperature/cooling schedule: start with high temperature (large step size) and reduce it geometrically each tick. Stop the simulation when `max_displacement < threshold` rather than running for a fixed number of ticks.
- Apply the ForceAtlas2 anti-swinging mechanism: track each node's last displacement direction and apply a braking force when the direction reverses.
- Run the simulation to convergence offline (during the crawl pipeline), store final positions in SurrealDB, and load them for visualization. This decouples layout quality from frame budget.
- Use the `theta` parameter in Barnes-Hut (typically 0.5-1.0) — a higher theta gives faster but less accurate repulsion. Start at 0.8 and let users tune.
- Recompute the quadtree only every 3-5 ticks rather than every tick during the cooling phase.

**Warning signs:**
- Graph visibly wiggles without settling after 10+ seconds
- Nodes cluster into two or three dense groups with empty space between (local minimum)
- CPU usage stays at 100% indefinitely during layout

**Phase to address:** WebGL renderer phase, during initial force layout implementation.

---

### Pitfall 5: Incremental Crawl Queue Producing Duplicate DB Writes Under Concurrency

**What goes wrong:**
The existing BFS crawler (`recursive_paper_search_by_references`) uses an in-memory `HashSet<String>` as its visited set. When incremental crawling resumes from a database-backed queue, concurrent tokio tasks can race on the visited check: task A and task B both read the queue, both see paper X as unvisited, both fetch it, both attempt to upsert. SurrealDB upserts are idempotent for identical records, but if fetched metadata differs slightly (e.g., citation counts changed between fetches), the second write overwrites the first. More problematically, both tasks spawn child tasks for paper X's references, duplicating the entire subtree traversal.

**Why it happens:**
The in-memory visited set is not shared between async tasks — each spawned task has its own copy (or the set is not passed across task boundaries at all in the resumed state). The database queue does not implement "claim before process" semantics.

**How to avoid:**
- Replace in-memory visited set with a SurrealDB `crawl_queue` table with states: `pending`, `claimed`, `done`, `failed`.
- Claim a batch of items with a single atomic SurrealDB transaction: `UPDATE crawl_queue SET status = 'claimed' WHERE status = 'pending' LIMIT N`. Unclaimed items remain available for retry.
- Never re-add a paper to the queue without checking for an existing record in any non-pending state. Use `INSERT IGNORE` / `CREATE ... IF NOT EXISTS` semantics.
- Maintain a write-through in-memory `DashMap` for hot deduplication within a single run. The DB is ground truth for cross-session deduplication.
- On process restart, mark all `claimed` items as `pending` again (safe because claiming was atomic and SurrealDB upserts are idempotent for the final result).

**Warning signs:**
- Paper count in DB grows on each resumed run even though depth hasn't increased
- Two concurrent tasks fetching the same paper ID appear in logs within the same run
- Analysis results are stored twice for the same paper with different timestamps

**Phase to address:** Incremental crawling phase — design the queue state machine before writing any resumable logic.

---

### Pitfall 6: isize/usize Crossing the 32-bit WASM / 64-bit Server Boundary

**What goes wrong:**
Server functions serialize their return types via serde and transmit them to the browser WASM client. Any type that uses `usize` or `isize` (Rust pointer-sized integers) serializes as 64-bit on the server but deserializes into a 32-bit context in WASM. For values that fit in 32 bits this is silent; for values that exceed `u32::MAX` (e.g., a very large paper count or offset), deserialization silently truncates or panics.

**Why it happens:**
Graph structures, database query results, and pagination offsets naturally use `usize`. The existing codebase uses `usize` pervasively in petgraph indices. When these types appear in data structs passed through server functions, the boundary mismatch becomes a latent bug.

**How to avoid:**
- All data types crossing server function boundaries must use fixed-width integer types: `u32`, `i32`, `u64`, `i64` — never `usize` or `isize`.
- Convert petgraph `NodeIndex` (which wraps `usize`) to `u32` before serializing for the client; reconstruct on the server side.
- Add a CI lint that fails if any server function argument or return type contains `usize` or `isize` directly.
- The Leptos documentation explicitly calls this out as a known pitfall for pointer-sized types across architectures.

**Warning signs:**
- Data appears correct on the server but is wrong or zero in the browser
- Node count displays correctly for small graphs but wraps around for large ones
- serde deserialization errors appearing in the browser console for integer fields

**Phase to address:** Web migration phase — audit all data types used in server function signatures before writing client components.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|---|---|---|---|
| CSR-only (no SSR) Leptos to skip hydration complexity | Eliminates hydration bugs, simpler mental model, faster to ship | No indexable content; no TTFB benefit; forecloses future SSR without rewrite | Acceptable for v1.1 — this is a single-user research tool, not a public-facing web app |
| Spawn blocking task for SurrealDB writes from Leptos server function | Quick to implement, avoids async-in-async issues | Wastes tokio worker threads; can cause deadlocks if the pool is exhausted by write-heavy crawls | Never — use `tokio::task::spawn_blocking` correctly with its own thread pool |
| Store graph node positions in Leptos reactive signals | Simple to wire to UI, positions update reactively | Fine-grained signal per node means O(n) signal subscriptions; triggers O(n) reactive updates every simulation tick | Never for n > 50 nodes — use a single `Signal<PositionBuffer>` wrapping a flat array |
| Use `serde_json` for server function payloads with large graph data | Zero additional dependencies | Large graphs with 1000+ nodes serialize to megabytes of JSON per request | Never — use binary encoding (MessagePack via `rmp-serde`) for graph transfer or paginate |
| Global `Arc<Mutex<petgraph>>` shared between server function handlers | One graph object in memory, easy to reason about | Mutex contention on every graph read; blocks all concurrent requests during layout computation | Never — immutable graph (crawl is complete before serving); clone on read or use RwLock |

---

## Integration Gotchas

Common mistakes when connecting components of this specific system.

| Integration | Common Mistake | Correct Approach |
|---|---|---|
| Leptos server functions + SurrealDB | Attempting to use `surrealdb::Surreal` as `AppState` via Axum `Extension` without wrapping in `Arc` | Wrap the SurrealDB client in `Arc<Surreal<Any>>` in Axum state; the client is already internally cheaply cloneable |
| cargo-leptos + SurrealDB `kv-surrealkv` feature | `kv-surrealkv` feature enabled unconditionally in Cargo.toml causes WASM build failure | Feature-gate behind `ssr`: `surrealdb = { features = ["kv-surrealkv"], optional = true }` |
| WebGL + Leptos canvas element | Using Leptos's `node_ref` to get the canvas works, but calling `getContext("webgl2")` in component body (not in Effect) means it runs on the server during SSR and panics | Get WebGL context inside `Effect::new(|| { ... })` only |
| Barnes-Hut + petgraph | petgraph's `StableGraph` uses `NodeIndex(usize)` — Barnes-Hut quadtree needs spatial indices as `usize` internally but these should not cross WASM boundary | Keep Barnes-Hut and petgraph entirely server-side or in WASM-local Rust code; never expose raw `NodeIndex` to server function API |
| Incremental crawl + Leptos progress streaming | Using a `tokio::mpsc` channel to stream progress to a Leptos server-sent-events endpoint works, but the SSE connection drops when the Leptos server restarts during `cargo leptos serve` hot-reload | Implement reconnection with `EventSource` on the client; SSE is inherently resumable |
| arXiv rate limiting + concurrent incremental crawl | Multiple `tokio::spawn` tasks each with their own `Instant::now()` clock for rate limiting — the rate limit is per-task, not per-client | Share a single `Arc<RateLimiter>` (e.g., `governor` crate) across all tasks to enforce the global rate |

---

## Performance Traps

Patterns that work at small scale but fail as graph size grows.

| Trap | Symptoms | Prevention | When It Breaks |
|---|---|---|---|
| Reactive signal per graph node for position | Layout update causes full page re-render | Single `RwSignal<Vec<[f32; 2]>>` for all positions; update as a batch | ~50 nodes |
| Full graph serialized in server function response | Page load freezes; large network payload visible in DevTools | Paginate or stream; send only visible subgraph initially | ~200 nodes (JSON > 1MB) |
| Synchronous quadtree rebuild every Barnes-Hut tick | Layout freezes UI thread in WASM | Rebuild quadtree every 3-5 ticks; run simulation on a Web Worker | ~300 nodes |
| O(n²) edge rendering in WebGL (line per edge, draw call per edge) | GPU time dominates frame budget | Batch all edges into a single VBO; one draw call for all edges | ~500 edges |
| SurrealDB `SELECT * FROM paper` for graph query | Memory spike loading all papers including full_text | Use projection: `SELECT id, title, arxiv_id, citation_count` — never select full_text for visualization queries | ~100 papers with full text |
| In-memory `HashSet` for visited set in resumed crawl | Duplicate fetches and DB writes on resume | Persistent DB-backed claim queue with atomic state transitions | First resume after crash |

---

## Security Mistakes

Domain-specific security issues for a research tool with web UI.

| Mistake | Risk | Prevention |
|---|---|---|
| Embedding arXiv/InspireHEP API keys (if any) in WASM binary | Keys exposed in browser DevTools; scraped from binary | All API calls run server-side only, inside `#[server]` functions; never in WASM |
| Accepting arbitrary `paper_id` from client in server function without validation | arXiv ID `../../etc/passwd` style path traversal if used in file paths; SSRF if used in direct URL construction | Validate paper ID against existing regex in `validation.rs` before any use in server functions; this validator already handles both arXiv ID formats |
| Exposing full SurrealDB query errors to client | Internal schema details, table names, field names leak | Map SurrealDB errors to opaque `ServerFnError` before returning from server functions |
| Storing LLM API keys in SurrealDB (accessible via DB-only mode) | Keys in DB are readable by anyone with file system access to `./data` | Store API keys in environment variables only; reference via `std::env::var` in server functions |

---

## UX Pitfalls

Common user experience mistakes specific to graph visualization and gap analysis tools.

| Pitfall | User Impact | Better Approach |
|---|---|---|
| Force layout runs on page load, blocking interaction for 5-10 seconds | User sees frozen blank canvas, unclear if app is working | Show a "computing layout" spinner; stream positions as they stabilize; allow interaction before convergence |
| Graph node click opens full paper text in a panel that replaces the graph | User loses their navigational context | Side-panel or drawer that overlays the graph; graph stays visible |
| Gap findings listed as raw strings with no paper provenance | User can't verify findings; loses trust | Every finding links to the specific sentence/section it came from (provenance tracking requirement) |
| Temporal filter slider with no visual feedback on graph | User sets year range but cannot see which nodes are affected | Dim/grey out nodes outside the filter range; don't remove them (preserves context) |
| Barnes-Hut `theta` parameter exposed in UI without explanation | Users set theta=0 (exact N-body), freezing the browser | Either hide advanced parameters behind an "expert" toggle, or clamp theta >= 0.5 |
| Method-combination matrix showing all pairs including trivially absent ones | Table with 10,000 cells is unreadable | Filter to only show method pairs where at least one paper uses each method individually but no paper combines them |

---

## "Looks Done But Isn't" Checklist

Things that appear complete in development but are missing for production.

- [ ] **Leptos SSR hydration**: Works locally with `cargo leptos watch` — verify with `cargo leptos build --release` that the WASM binary compiles without SurrealDB dependencies pulling in.
- [ ] **WebGL renderer**: Renders correctly at 50 nodes — verify with a synthetic 1000-node graph that frame rate stays above 30 FPS.
- [ ] **Incremental crawl resume**: Resumes correctly after clean process exit — verify that resume after a `kill -9` (unclean exit with `claimed` items in queue) correctly reclaims and processes those items.
- [ ] **Server function types**: No `usize` in any server function argument or return type — verify with a grep CI check.
- [ ] **Rate limiting under concurrency**: Single-task rate limit looks correct — verify with 10 concurrent crawl tasks that arXiv is not hit faster than 1 req/3s combined.
- [ ] **Gap findings in web UI**: Findings render in dev — verify they load correctly from DB in `--db-only` mode without triggering a crawl.
- [ ] **WASM binary size**: Development build is acceptably sized — verify `wasm-opt`-compressed release binary is under 5MB; add `opt-level = 'z'` profile if not.
- [ ] **Provenance links**: "Click to see source" feature is wired — verify that clicking a gap finding actually scrolls to the source text segment, not just the paper.

---

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---|---|---|
| SurrealDB pulls into WASM build (Pitfall 1) | MEDIUM | Feature-gate surrealdb as `optional = true`; move all DB code into server functions; rebuild |
| Hydration mismatch discovered late (Pitfall 2) | MEDIUM | Temporarily switch to CSR-only (`leptos/hydrate` without `ssr`) to isolate the mismatch; bisect which component causes it; re-enable SSR once fixed |
| Force layout never converges (Pitfall 4) | MEDIUM | Implement offline pre-computation as the primary path; interactive simulation becomes optional enhancement |
| Duplicate writes discovered in resumed crawl (Pitfall 5) | HIGH | Requires schema migration to add `crawl_queue` table; existing data is safe but crawl must restart to rebuild queue state |
| `isize`/`usize` boundary bug found in production (Pitfall 6) | LOW | Mechanical refactor — change field types, update serde derives, redeploy; no data migration needed |
| WASM binary too large for reasonable load time | LOW | Add `wasm-opt` pass; switch to `miniserde`; audit dependencies with `cargo bloat --release --crates` |

---

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---|---|---|
| SurrealDB not WASM-compilable (Pitfall 1) | Phase: Web migration setup — establish cargo feature boundaries first | `cargo leptos build --release` succeeds without error |
| Hydration mismatch (Pitfall 2) | Phase: Web migration — add SSR integration test before shipping first component | Test renders a component server-side and validates DOM structure |
| JS-WASM boundary overhead (Pitfall 3) | Phase: WebGL renderer — define memory model in design doc before writing render loop | Frame rate benchmark at 500 nodes >= 30 FPS |
| Force layout oscillation (Pitfall 4) | Phase: WebGL renderer — implement cooling schedule and offline precompute path | 1000-node graph converges to stable layout within 30 seconds |
| Duplicate crawl writes (Pitfall 5) | Phase: Incremental crawling — design DB queue schema before writing crawler code | Simulated crash-and-resume test produces identical paper set as clean run |
| isize/usize boundary (Pitfall 6) | Phase: Web migration setup — add CI grep lint on server function types | CI fails if `usize` appears in any `#[server]` function signature |
| Reactive signal per node (performance trap) | Phase: WebGL renderer — define signal architecture before connecting to Leptos | Layout update measured at < 2ms per frame in DevTools |
| Rate limiting under concurrency | Phase: Incremental crawling — share single Arc<RateLimiter> | Integration test: 10 concurrent tasks produce no more than 1 req/3s to arXiv mock |

---

## Sources

- [Leptos Book: Hydration Bugs](https://book.leptos.dev/ssr/24_hydration_bugs.html) — official Leptos hydration pitfall documentation; HIGH confidence
- [Leptos Book: Optimizing WASM Binary Size](https://book.leptos.dev/deployment/binary_size.html) — official binary size guidance; HIGH confidence
- [Leptos Book: Server Functions](https://book.leptos.dev/server/25_server_functions.html) — server function behavior and feature flag requirements; HIGH confidence
- [Leptos 0.8.0 Release Notes](https://github.com/leptos-rs/leptos/releases/tag/v0.8.0) — breaking changes including usize/isize pointer-width pitfall explicitly called out; HIGH confidence
- [leptos_axum crate docs](https://docs.rs/leptos_axum/latest/leptos_axum/) — Axum integration patterns; HIGH confidence
- [SurrealDB SDK Concurrency Docs](https://surrealdb.com/docs/sdk/rust/concepts/concurrency) — SurrealDB concurrency model; HIGH confidence
- [SurrealKV MVCC / Snapshot Isolation](https://surrealdb.com/blog/vart-a-persistent-data-structure-for-snapshot-isolation) — SurrealDB transaction isolation guarantees; HIGH confidence
- [Barnes-Hut Algorithm Reference](https://arborjs.org/docs/barnes-hut) — Barnes-Hut theta parameter and convergence behavior; MEDIUM confidence
- [ForceAtlas2 Paper](https://medialab.sciencespo.fr/publications/Jacomy_Heymann_Venturini-Force_Atlas2.pdf) — anti-swinging and cooling schedule design; MEDIUM confidence
- [GraphWaGu: GPU Force Layout](https://stevepetruzza.io/pubs/graphwagu-2022.pdf) — GPU-accelerated graph layout at scale; MEDIUM confidence
- [Webcola + WASM graph layout case study](https://cprimozic.net/blog/speeding-up-webcola-with-webassembly/) — JS-WASM boundary overhead in graph rendering; MEDIUM confidence
- [Rust WASM Book: Implementing Conway's Game of Life](https://rustwasm.github.io/book/game-of-life/implementing.html) — typed array / linear memory patterns for avoiding JS-WASM copy overhead; HIGH confidence
- [LogRocket: Migrating JS frontend to Leptos](https://blog.logrocket.com/migrating-javascript-frontend-leptos-rust-framework/) — practical migration experience; MEDIUM confidence

---
*Pitfalls research for: ReSyn v1.1 — Leptos web migration, WebGL/Barnes-Hut graph rendering, incremental crawling*
*Researched: 2026-03-15*
