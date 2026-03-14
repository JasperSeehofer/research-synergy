# Technology Stack

**Project:** Research Synergy (ReSyn) — v1.1 Scale & Surface Milestone
**Researched:** 2026-03-15
**Scope:** New libraries only for v1.1. Existing stack (tokio, petgraph, surrealdb, reqwest, scraper, serde, egui/eframe, clap, tracing, fastembed, keyword_extraction, genai, sha2, chrono) is not re-researched.
**Confidence:** HIGH (Leptos, sigma.js), MEDIUM (JS interop approach, incremental crawl queue patterns)

---

## Context: What Changes in v1.1

The v1.1 milestone replaces the egui desktop GUI with a Leptos web UI and introduces:
1. **Leptos** — replaces egui/eframe as the UI framework (web, WASM)
2. **Sigma.js + Graphology** — replaces fdg/egui_graphs for graph rendering (WebGL, 1000+ nodes)
3. **ForceAtlas2 via graphology-layout-forceatlas2** — replaces fdg force layout (Barnes-Hut O(n log n))
4. **Axum** — HTTP server to expose existing Rust analysis pipeline as a REST API
5. **SurrealDB crawl queue schema** — no new crate; extend existing schema with queue records
6. **Trunk** — WASM build tool for the Leptos frontend

The existing CLI binary continues to work. The web UI is a parallel serving mode activated via a new `--web` flag or a separate binary.

---

## New Dependencies for This Milestone

### Web Framework (Frontend)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `leptos` | 0.8.x (latest: 0.8.17) | Reactive Rust/WASM frontend framework | Fine-grained reactivity compiles to WASM; Rust all the way through — no JS framework to learn. Mature: 1.8M+ downloads, production-ready. CSR mode (Trunk) is simpler for a tool that talks to a local Axum server; no SSR hydration complexity needed for a single-user research tool. |
| `leptos_axum` | 0.8.x (matches leptos) | Axum integration for server functions | Required for type-safe server → client communication if using Leptos server functions. Also provides static file serving for the WASM bundle. |
| `trunk` | latest (CLI, not crate dep) | WASM build and dev server | The standard build tool for Leptos CSR. Handles: wasm-pack invocation, asset bundling, live reload, Tailwind integration. Installed once: `cargo install trunk`. Not a Cargo dependency. |

**CSR vs SSR decision:**

Use **CSR with Trunk** — not full SSR with cargo-leptos.

Rationale:
- ReSyn is a single-user local tool, not a public web app. SEO is irrelevant. Initial load time for a local tool is irrelevant.
- CSR has faster iteration (only recompile the frontend WASM, not a server+client dual binary).
- The Axum server is already needed to expose the analysis pipeline as a REST API. Serving the static WASM bundle from Axum is trivial (`tower_http::ServeDir`).
- Full SSR (cargo-leptos) would require restructuring the entire project into a workspace with separate server/client crates and a dual-compilation target. Disproportionate complexity for a local-first tool.

Architecture: `cargo run -- --web` starts an Axum server on `localhost:3000`, serves the pre-built Leptos WASM SPA as static files, and exposes REST API endpoints for the analysis pipeline. During development, Trunk's dev server proxies API calls to Axum.

---

### HTTP Server (Backend for Web UI)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `axum` | 0.8.x | REST API server exposing analysis pipeline to Leptos frontend | The standard Rust web framework for async APIs; Tokio-native; integrates with leptos_axum for static file serving. Already the Leptos-recommended server. Zero friction with existing tokio runtime. |
| `tower-http` | 0.6.x | Static file serving, CORS, request logging | Provides `ServeDir` middleware for serving the Trunk-built WASM bundle. Also provides `CorsLayer` for the Trunk dev server proxy. Required alongside axum. |
| `serde_json` (existing) | 1.x | JSON serialization for REST API responses | Already in use. REST endpoints return `Paper`, `GapFinding`, `AnalysisResult` structs as JSON via axum's `Json` extractor/responder. |

---

### Graph Visualization (Frontend JS, loaded via Trunk)

These are **JavaScript / TypeScript npm packages**, not Rust crates. They are loaded via a `<script>` tag in `index.html` or bundled by a JavaScript bundler invoked via Trunk's asset pipeline.

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `sigma` | 3.0.x (latest: 3.0.2) | WebGL graph renderer for 1000+ node citation graphs | WebGL-based rendering; handles large graphs (thousands of nodes/edges) at interactive framerates where Canvas/SVG fail. Built-in spatial index for hover/click hit testing. Actively maintained; used in production by Gephi Lite. The correct tool for this scale. |
| `graphology` | 0.26.x | Graph data structure for sigma.js | sigma.js requires graphology as its graph data model. Handles directed/undirected graphs; node/edge attribute storage; provides the import/export interface. No alternative — sigma.js only accepts graphology graphs. |
| `graphology-layout-forceatlas2` | 0.10.x | Barnes-Hut O(n log n) force layout | ForceAtlas2 with Barnes-Hut approximation handles 1000+ nodes where naive O(n²) fdg layouts stall. Runs in a Web Worker (non-blocking). The algorithm is designed for network visualization (Gephi heritage). Directly replaces the existing fdg Fruchterman-Reingold layout. |

**Integration with Leptos:**

sigma.js is a JavaScript library. Integration with Leptos WASM follows this pattern:
1. A Leptos `<canvas>` element is created with a `NodeRef`.
2. `use_effect` initializes the sigma.js `Sigma` instance on the canvas element after mount.
3. `wasm-bindgen` / `js-sys` are used to call sigma.js and graphology APIs from Rust. The `#[wasm_bindgen]` attribute imports JS constructors and methods.
4. Graph data (nodes, edges, colors from gap analysis) is serialized to a `JsValue` and passed to graphology's `Graph` constructor via `js-sys`.
5. Layout computation (ForceAtlas2 web worker) runs independently; sigma.js renders whatever positions graphology currently holds.

This is the **established pattern** for Leptos + WebGL visualization. Leptos discussions confirm canvas access via `NodeRef` works, and the Leptos book documents wasm-bindgen integration explicitly.

**Alternative considered and rejected:** Rendering sigma.js purely from TypeScript/JS with Leptos calling it via postMessage. More decoupled but adds a TypeScript build step and complicates the data serialization boundary. The direct wasm-bindgen approach keeps everything in Rust.

---

### WASM Build Support (Rust additions)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `wasm-bindgen` | 0.2.x (managed by Trunk) | Rust ↔ JS boundary glue | Required for all Rust/WASM web apps. Trunk manages the exact version. No manual Cargo.toml entry needed for basic use; leptos already pulls it in. Add explicitly only if calling custom JS libraries (sigma.js interop). |
| `js-sys` | 0.3.x (via wasm-bindgen) | Rust bindings for core JS types (Array, Object, Function) | Required to build graphology Graph objects and call sigma.js constructors from Rust. Added explicitly: `js-sys = "0.3"` with `wasm-bindgen` feature. |
| `web-sys` | 0.3.x (via wasm-bindgen) | Rust bindings for DOM/Web APIs (canvas, events) | Required for canvas access, requestAnimationFrame, event handling in the Leptos graph component. leptos already depends on web-sys; may need additional features enabled (e.g., `HtmlCanvasElement`, `WebGl2RenderingContext`). |
| `console_error_panic_hook` | 0.1.x | Panic messages forwarded to browser console | Development quality-of-life. Panic messages from Rust show as readable strings in browser devtools instead of `RuntimeError: unreachable`. Standard practice for Rust WASM. |
| `gloo` | 0.11.x | High-level WASM utilities (timers, events, storage) | Optional. Provides ergonomic wrappers around web-sys for animation frames and event listeners in the graph canvas. Use if the raw web-sys APIs become verbose. leptos-use covers many of these use cases already. |

---

### CSS / Styling

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Tailwind CSS | 4.x | Utility-first CSS for Leptos UI | Standard pairing with Leptos; Trunk downloads and runs the Tailwind CLI automatically via `LEPTOS_TAILWIND_VERSION` config. No Node.js required. Leptos 0.8 + Tailwind 4 is a documented combination. Suitable for the gap analysis panels, node inspector, control sidebars. |

---

### Incremental Crawling (No New Crates — Schema Extension Only)

Incremental/resumable crawling does **not** require a new crate. It requires:

1. **A `crawl_queue` SurrealDB table** (new migration, extends existing schema):
   - Records: `{ paper_id: String, depth: u32, status: "pending" | "in_progress" | "done" | "failed", enqueued_at, processed_at }`
   - The BFS frontier is persisted here instead of an in-memory `VecDeque`.
   - On restart, `SELECT * FROM crawl_queue WHERE status = "pending"` resumes the crawl.

2. **tokio::sync::Semaphore** (already in tokio, no new dep) — limits concurrent in-flight requests to respect rate limits when processing queue entries concurrently.

3. **tokio::time::sleep** (existing) — per-request rate limiting already implemented in `ArxivHTMLDownloader` and `InspireHepClient`. Incremental crawl reuses this.

Pattern: Replace the existing `recursive_paper_search_by_references` VecDeque BFS with a DB-backed queue processor:
```
loop:
  SELECT batch of pending items from crawl_queue
  if empty: break
  for each item (with semaphore permit):
    mark in_progress
    fetch + process paper
    enqueue new references as pending (if not already crawled or enqueued)
    mark done
```

No new Rust crates needed. This is a query + control-flow change, not a library change.

---

### Supporting Library Changes

| Library | Change | Reason |
|---------|--------|--------|
| `egui`, `eframe`, `egui_graphs`, `fdg` | **Remove** from default binary target | Replaced by Leptos + sigma.js. Keep behind a `--features desktop` flag during transition if needed, then fully remove. eframe and egui are desktop-only and do not compile to WASM. |
| `crossbeam` | **Review need** | Currently used for graph data sharing between render thread and analysis. With Leptos web, replace with Leptos signals or `RwSignal`. May be removable. |
| `rand` | **Keep** | Used in analysis pipeline; also needed for graph layout seed positions in WASM context (use `getrandom` with `wasm` feature flag if WASM target has issues). |

---

## Installation

```toml
# Cargo.toml additions for v1.1

[dependencies]
# Web framework (server side)
axum = "0.8"
tower-http = { version = "0.6", features = ["fs", "cors"] }

# Leptos frontend (CSR)
leptos = { version = "0.8", features = ["csr"] }
leptos_axum = "0.8"

# WASM JS interop (for sigma.js calls)
js-sys = "0.3"
web-sys = { version = "0.3", features = [
  "HtmlCanvasElement",
  "Element",
  "Window",
  "Document",
] }
console_error_panic_hook = "0.1"

# Optional: ergonomic WASM utilities
gloo = "0.11"

[target.'cfg(target_arch = "wasm32")'.dependencies]
# Only compile WASM-specific deps for the frontend target
wasm-bindgen = "0.2"
```

```bash
# CLI tools (install once)
cargo install trunk
rustup target add wasm32-unknown-unknown

# npm packages (in a /frontend or project root package.json)
npm install sigma graphology graphology-layout-forceatlas2
```

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Web framework | Leptos 0.8 | Yew | Leptos has finer-grained reactivity and better SSR/CSR flexibility; Yew uses a virtual DOM (less efficient). Leptos is now more active. |
| Web framework | Leptos 0.8 | Dioxus | Dioxus targets desktop + web but has less mature web ecosystem. Leptos has better Axum integration docs and more examples. |
| Graph renderer | sigma.js + graphology | Cytoscape.js | Cytoscape.js WebGL support is via optional plugin; sigma.js is WebGL-native. sigma.js has smaller bundle size for pure graph rendering. |
| Graph renderer | sigma.js + graphology | D3.js force simulation | D3 uses Canvas/SVG (not WebGL); SVG degrades badly above 500 nodes. D3's API is imperative and hard to integrate with Leptos reactive model. |
| Graph renderer | sigma.js + graphology | Pure Rust WebGL (web-sys) | Writing WebGL shaders + hit testing + spatial indexing from scratch in Rust is months of work. sigma.js provides all of this battle-tested. |
| Force layout | ForceAtlas2 (graphology) | fdg (existing Fruchterman-Reingold) | fdg runs on desktop/native; does not compile to WASM cleanly (git dep, no WASM target validated). ForceAtlas2 has Barnes-Hut O(n log n) needed for 1000+ nodes. |
| Force layout | ForceAtlas2 (graphology) | d3-force (JS) | d3-force is O(n²) without Barnes-Hut by default. ForceAtlas2 is purpose-built for network graphs and produces better academic citation graph layouts. |
| HTTP server | Axum | Actix-web | Both are valid. Axum is Leptos's default recommendation; uses tower middleware (tower-http already needed); integrates cleanly with tokio ecosystem. |
| CSS | Tailwind CSS | Plain CSS / CSS Modules | Tailwind pairs naturally with Leptos class attributes; Trunk handles Tailwind CLI automatically. Suitable for control panels and data tables in the UI. |
| Build tool | Trunk (CSR) | cargo-leptos (SSR) | cargo-leptos optimizes for SSR hydration. ReSyn is a local single-user tool; SSR adds complexity (dual compilation, server rendering) with no benefit for this use case. |
| Incremental crawl | SurrealDB queue table | External queue (Redis, SQLite) | Existing SurrealDB is already embedded and running; adding another store violates the single-database constraint in the project constraints. |

---

## What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `cargo-leptos` build tool | Designed for SSR dual-compilation; overkill for local CSR SPA | `trunk` for CSR builds |
| `Bevy` | ECS game engine; embedding in a web app requires bevy_egui and full ECS restructure | sigma.js for graph rendering |
| `three-d` | OpenGL-based; conflicts with WASM targets | sigma.js WebGL |
| `wasm-pack` directly | Trunk invokes wasm-pack internally; calling it separately creates duplicate build artifacts | Let Trunk manage wasm-pack |
| React / Vue / Svelte | Foreign language from existing Rust codebase; loses type safety at the Rust/JS boundary | Leptos (Rust/WASM) |
| `graphql` | Adds schema/resolver complexity for a single-user tool with simple data access needs | Axum REST JSON endpoints |
| `getrandom` explicitly | Only needed if `rand` fails on WASM target due to missing entropy source; fix by adding `wasm` feature to rand instead | `rand = { version = "0.9", features = ["wasm"] }` if needed |

---

## Version Compatibility Notes

| Concern | Detail |
|---------|--------|
| leptos + leptos_axum versions must match | Both must be the same 0.8.x patch. Mismatching causes compile errors due to shared internal traits. |
| wasm-bindgen version managed by Trunk | Trunk downloads a compatible wasm-bindgen CLI that matches the Cargo.lock version. Do not pin wasm-bindgen separately unless debugging binding issues. |
| axum 0.8 requires tower-http 0.6 | tower-http 0.5 is incompatible with axum 0.8's tower types. Use 0.6. |
| sigma.js 3.x requires graphology 0.25+ | sigma 3.0.x expects graphology 0.25.4+. Install both together: `npm install sigma graphology`. |
| rand on WASM target | rand 0.9 uses `getrandom` under the hood. If the WASM build fails with entropy errors, add `getrandom = { version = "0.2", features = ["js"] }` as a dev/WASM dependency. |
| eframe/egui removed | eframe depends on winit and wgpu, which do not compile cleanly to wasm32-unknown-unknown without significant configuration. Removing eframe simplifies the WASM build target. |

---

## Sources

- [Leptos crates.io — version 0.8.17 confirmed](https://crates.io/crates/leptos)
- [Leptos book — Getting Started (CSR vs SSR)](https://book.leptos.dev/getting_started/index.html)
- [Leptos book — Integrating with JavaScript: wasm-bindgen, web_sys, and HtmlElement](https://book.leptos.dev/web_sys.html)
- [Leptos GitHub — start-axum template](https://github.com/leptos-rs/start-axum)
- [leptos_axum docs.rs](https://docs.rs/leptos_axum/latest/leptos_axum/)
- [sigma.js GitHub — version 3.0.2](https://github.com/jacomyal/sigma.js/)
- [sigma.js Quickstart Guide](https://www.sigmajs.org/docs/quickstart/)
- [sigma.js v3.0 release announcement](https://www.ouestware.com/2024/03/21/sigma-js-3-0-en/)
- [graphology npm — version 0.26.0](https://graphology.github.io/)
- [graphology-layout-forceatlas2 — Barnes-Hut docs](https://graphology.github.io/standard-library/layout-forceatlas2.html)
- [Leptos canvas/WebGL discussion — confirmed NodeRef pattern](https://github.com/leptos-rs/leptos/discussions/2245)
- [Leptos + Tailwind 4 + DaisyUI 5 guide](https://8vi.cat/leptos-0-8-tailwind4-daisyui5-for-easy-websites/) — MEDIUM confidence (community source)
- [tokio::sync::Semaphore docs — rate limiting pattern](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html)
- [cargo-leptos GitHub](https://github.com/leptos-rs/cargo-leptos)
- [wasm-bindgen guide](https://rustwasm.github.io/docs/wasm-bindgen/)

---

*Stack research for: ReSyn v1.1 — Leptos web migration, WebGL graph, Barnes-Hut layout, incremental crawl*
*Researched: 2026-03-15*
