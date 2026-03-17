# Phase 8: Leptos Web Shell + Analysis Panels - Research

**Researched:** 2026-03-17
**Domain:** Leptos 0.8 CSR frontend + Axum server functions backend
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Sidebar + content layout with a Dashboard as the landing/overview page
- Sidebar sections: Dashboard, Papers, Gaps, Open Problems, Methods
- Sidebar is collapsible to an icon-only rail (thin vertical strip with icons, tooltips on hover)
- Dashboard shows summary cards (total papers, contradiction count, bridge count, open problems count, method coverage %). Each card links to its panel
- Dark minimal theme — dark background, muted colors, clean typography (VS Code / Linear aesthetic)
- Separate CSS file(s) loaded by Trunk — no inline styles, no CSS-in-Rust
- Paper list: sortable table with columns: title, authors, year, citation count, analysis status
- Row click opens a side drawer (slides in from right) showing paper detail: abstract, methods, findings, open problems
- Gap findings: card per finding with type badge, confidence color bar, shared terms as tags, expand for justification
- Paper IDs on cards are clickable — opens paper side drawer (cross-panel navigation)
- Filtering controls: toggle buttons for Contradictions/Bridges + confidence threshold slider
- Open-problems: ranked list by recurrence count (from LlmAnnotation.open_problems)
- Method heatmap axes: method categories (from LlmAnnotation.methods[].category)
- Method heatmap cell color: sequential blue scheme (dark blue = 0, bright cyan = many papers). Empty cells = dark gray with subtle marker
- Click a cell to drill down into sub-matrix of individual method names within those two categories
- Crawl progress at bottom of sidebar: full stats when expanded, spinning indicator + percentage when collapsed, idle shows last summary
- Web UI can start a crawl — form with paper ID, depth, source fields. Server function triggers crawl logic on backend
- Leptos server functions call resyn-core's PaperRepository and gap_analysis functions
- SSE crawl progress consumed from the existing /progress endpoint (Phase 7)

### Claude's Discretion
- Leptos component structure and file organization within resyn-app
- Trunk configuration details (index.html, asset pipeline)
- Axum server setup in resyn-server (router, state, server function integration)
- Exact CSS class naming and organization
- Loading states and skeleton screens
- Error display patterns
- Open-problems panel exact layout (cards vs table rows)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| WEB-03 | Leptos CSR shell with Trunk build pipeline and routing | Trunk 0.21.14 + leptos 0.8 csr feature + leptos_router 0.8; Trunk.toml proxy routes API calls to Axum server |
| WEB-04 | Axum server functions exposing analysis pipeline to frontend | leptos_axum 0.8.8 handle_server_fns_with_context; resyn-server adds server fn routes; resyn-app defines #[server] fns with csr feature |
| AUI-01 | Gap findings rendered (contradiction cards, bridge cards) in analysis panel | GapFindingRepository::get_all_gap_findings() feeds server fn; card component with GapType badge and confidence bar |
| AUI-02 | Open-problems aggregation panel ranked by recurrence frequency | get_all_annotations() → aggregate open_problems strings → sort by count; display as ranked list |
| AUI-03 | Method-combination gap matrix showing existing vs absent method pairings | get_all_annotations() → collect Method.category pairs → build NxN matrix; canvas or CSS grid heatmap |
</phase_requirements>

## Summary

Phase 8 builds the Leptos CSR frontend (`resyn-app`) and wires up the Axum server (`resyn-server/src/commands/serve.rs`) to serve Leptos server functions backed by SurrealDB via `resyn-core`. The browser app renders five panels — Dashboard, Papers, Gaps, Open Problems, Methods — and consumes live crawl progress from the existing SSE endpoint.

The architecture is split: `trunk serve` builds and serves the WASM/JS bundle on port 8080; the Axum binary (`resyn serve`) runs on port 3000 and handles server function HTTP calls. During development, `Trunk.toml` `[[proxy]]` forwards `/api/` requests from port 8080 to port 3000, avoiding CORS issues. In production, both are served from the same origin by having Axum also serve the static WASM/JS dist files.

The critical constraint is the existing `ssr` feature gate on `resyn-core`: `resyn-app` must NOT enable `ssr` (only `csr`), while `resyn-server` enables `ssr`. Server functions defined in `resyn-app` with `#[server]` compile to HTTP stubs on the WASM side and to real function bodies (calling the DB) on the server side — but only if they are also registered with the Axum router in `resyn-server`.

**Primary recommendation:** Use leptos 0.8 `csr` feature in resyn-app, leptos `ssr` + `leptos_axum` in resyn-server, define `#[server]` functions in a shared `app` crate (or directly in resyn-app with conditional compilation), register them via `handle_server_fns_with_context` in Axum, and proxy `/api/` through Trunk during development.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| leptos | 0.8.17 | Reactive CSR UI framework | Project decision; fine-grained reactivity, compile-time checked templates |
| leptos_router | 0.8.12 | Client-side routing | Ships with leptos; path!() macro, nested routes, use_navigate |
| leptos_axum | 0.8.8 | Server function integration with Axum | Official Leptos integration; handle_server_fns_with_context |
| leptos-use | 0.18.3 | Utility hooks including use_event_source | Recommended replacement for leptos_sse; handles SSE with auto-reconnect |
| wasm-bindgen | 0.2.114 | WASM/JS bridge | Required by leptos csr; already in resyn-app |
| console_error_panic_hook | 0.1.7 | Panic messages in browser console | Standard debugging tool for WASM |
| trunk | 0.21.14 (install) | WASM bundler / dev server | Project decision; CSR build pipeline |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| leptos_meta | 0.8 | `<Title>`, `<Stylesheet>` components | Setting page title and loading CSS via Leptos |
| codee | latest | Codec for use_event_source | Required by leptos-use for SSE message decoding |
| tower-http | 0.6 | Static file serving + CORS for Axum | Serve WASM bundle in production from Axum; already workspace dep |
| tower | 0.5 | Middleware layers for Axum | Already workspace dep |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Trunk (CSR) | cargo-leptos (SSR/hydrate) | Project decision is CSR-only; cargo-leptos adds SSR complexity |
| leptos-use use_event_source | raw web-sys EventSource | leptos-use handles reactivity integration, reconnect, cleanup automatically |
| CSS files via Trunk | Tailwind | Project uses plain CSS; adding Tailwind is Claude's discretion |

**Installation for resyn-app:**
```bash
# In resyn-app/Cargo.toml — add these dependencies
# (leptos with csr feature, leptos_router, leptos_meta, leptos-use, wasm-bindgen, console_error_panic_hook)
```

**Install trunk (dev tooling):**
```bash
cargo install trunk@0.21.14
rustup target add wasm32-unknown-unknown
```

**Version verification (confirmed 2026-03-17):**
- leptos: 0.8.17 (released 2026-03-01)
- leptos_router: 0.8.12
- leptos_axum: 0.8.8
- leptos-use: 0.18.3 (released 2026-02-26, requires leptos = "0.8")
- trunk: 0.21.14 stable (0.22.0-beta.1 exists but is beta)

## Architecture Patterns

### Recommended Project Structure
```
resyn-app/
├── src/
│   ├── lib.rs               # App root, mount_to_body, server fn registrations
│   ├── app.rs               # App() component, Router, top-level layout
│   ├── layout/
│   │   ├── mod.rs
│   │   ├── sidebar.rs       # Collapsible sidebar with nav + crawl progress
│   │   └── drawer.rs        # Paper detail side drawer (slides in)
│   ├── pages/
│   │   ├── mod.rs
│   │   ├── dashboard.rs     # Summary cards
│   │   ├── papers.rs        # Sortable paper table
│   │   ├── gaps.rs          # Gap finding cards with filter controls
│   │   ├── open_problems.rs # Ranked open-problems list
│   │   └── methods.rs       # Method heatmap + drill-down
│   ├── server_fns/
│   │   ├── mod.rs
│   │   ├── papers.rs        # get_papers(), get_paper_detail(), start_crawl()
│   │   ├── gaps.rs          # get_gap_findings()
│   │   ├── problems.rs      # get_open_problems_ranked()
│   │   └── methods.rs       # get_method_matrix()
│   └── components/
│       ├── mod.rs
│       ├── gap_card.rs      # GapFinding card with badge, confidence bar, tags
│       ├── heatmap.rs       # Method category heatmap cell grid
│       └── crawl_progress.rs # SSE-connected progress display + crawl form
├── index.html               # Minimal HTML; trunk injects WASM loader
├── style/
│   └── main.css             # Dark theme CSS
└── Trunk.toml               # Build config + proxy to Axum

resyn-server/src/commands/
└── serve.rs                 # Axum router: server fn route + static file fallback
```

### Pattern 1: CSR App Setup with Trunk
**What:** The resyn-app lib.rs mounts the Leptos app to the document body. Trunk's `index.html` is minimal — Trunk injects the WASM/JS loader script automatically.
**When to use:** Entry point for all CSR Leptos apps with Trunk.
**Example:**
```rust
// resyn-app/src/lib.rs
// Source: https://book.leptos.dev/getting_started/index.html
use leptos::prelude::*;
use leptos::mount::mount_to_body;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
```

```html
<!-- resyn-app/index.html — minimal, trunk adds scripts -->
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
    <link data-trunk rel="css" href="style/main.css"/>
  </head>
  <body></body>
</html>
```

### Pattern 2: Server Functions — Definition and Feature Gating
**What:** `#[server]` macro generates an HTTP stub (WASM side) and a real async function body (server side). The body can call DB repositories — but only when compiled with the `ssr` feature (which resyn-app must NOT enable).
**When to use:** Any data-fetching from SurrealDB that the frontend needs.
**Example:**
```rust
// resyn-app/src/server_fns/gaps.rs
// Source: https://book.leptos.dev/server/25_server_functions.html
use leptos::prelude::*;
use resyn_core::datamodels::gap_finding::GapFinding;

#[server(GetGapFindings, "/api")]
pub async fn get_gap_findings() -> Result<Vec<GapFinding>, ServerFnError> {
    // This body only compiles server-side (ssr feature)
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::client::connect_local;
        use resyn_core::database::queries::GapFindingRepository;
        // DB access here
        todo!()
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
```

**Critical:** The `#[server]` macro in Leptos 0.8 accepts `(Name, "/api_prefix")`. The prefix `/api` matches the Trunk proxy backend path.

### Pattern 3: Axum Server — Registering Server Functions
**What:** The Axum server must register server function routes. In a CSR-only setup (no SSR rendering), use `handle_server_fns_with_context` on a route that catches all server function paths.
**When to use:** resyn-server/src/commands/serve.rs.
**Example:**
```rust
// resyn-server/src/commands/serve.rs
// Source: https://docs.rs/leptos_axum/latest/leptos_axum/fn.handle_server_fns_with_context.html
use axum::{Router, routing::post};
use leptos_axum::handle_server_fns_with_context;

pub async fn run(args: ServeArgs) -> anyhow::Result<()> {
    let db = resyn_core::database::client::connect_local(&args.db).await?;
    let db = std::sync::Arc::new(db);

    let app = Router::new()
        // Server functions POST to /api/<ServerFnName>
        .route("/api/*fn_name", post({
            let db = db.clone();
            move |req| handle_server_fns_with_context(
                move || {
                    provide_context(db.clone());
                },
                req,
            )
        }))
        // SSE endpoint is already at /progress from Phase 7
        .route("/progress", get(crawl_progress_handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

### Pattern 4: Trunk Proxy Configuration
**What:** During development, `trunk serve` runs on port 8080. API calls from the WASM app to `/api/*` are proxied to the Axum server on port 3000. This avoids CORS configuration.
**When to use:** Development only. Production: Axum serves the dist/ folder too.
**Example:**
```toml
# resyn-app/Trunk.toml
# Source: https://raw.githubusercontent.com/trunk-rs/trunk/main/Trunk.toml (official example)

[serve]
port = 8080

[[proxy]]
# Forward /api/* to Axum server functions
backend = "http://localhost:3000/api/"
rewrite = "/api/"

[[proxy]]
# Forward /progress SSE to Axum
backend = "http://localhost:3000/progress"
ws = false
```

### Pattern 5: Reactive Data Loading with Resource
**What:** `Resource` wraps async server function calls into the reactive graph. Suspense boundaries handle loading states.
**When to use:** All data panels — papers, gaps, open problems, method matrix.
**Example:**
```rust
// Source: https://book.leptos.dev/async/10_resources.html
use leptos::prelude::*;

#[component]
fn GapsPanel() -> impl IntoView {
    let findings = Resource::new(|| (), |_| get_gap_findings());

    view! {
        <Suspense fallback=|| view! { <p>"Loading findings..."</p> }>
            {move || findings.get().map(|result| match result {
                Ok(fs) => fs.into_iter().map(|f| view! { <GapCard finding=f/> }).collect_view(),
                Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_view(),
            })}
        </Suspense>
    }
}
```

### Pattern 6: SSE Crawl Progress with leptos-use
**What:** `use_event_source` from leptos-use wraps browser EventSource in a reactive signal. Reconnects automatically on error.
**When to use:** Crawl progress section at sidebar bottom.
**Example:**
```rust
// Source: https://leptos-use.rs/network/use_event_source.html
use leptos::prelude::*;
use leptos_use::{use_event_source, UseEventSourceReturn};
use codee::string::JsonSerdeCodec;
use resyn_core::commands::crawl::ProgressEvent; // WASM-safe since it's serde only

#[component]
fn CrawlProgress() -> impl IntoView {
    let UseEventSourceReturn { ready_state, message, .. } =
        use_event_source::<ProgressEvent, JsonSerdeCodec>("/progress");

    view! {
        <div class="crawl-progress">
            {move || message.get().map(|ev| view! {
                <progress value={ev.papers_found} max={ev.papers_found + ev.papers_pending}/>
                <span>{ev.papers_found} " / " {ev.papers_found + ev.papers_pending}</span>
            })}
        </div>
    }
}
```

**Note:** `ProgressEvent` is in `resyn-server/src/commands/crawl.rs` behind `ssr`. It must be moved to a WASM-safe location (resyn-core datamodels) so resyn-app can import it.

### Pattern 7: Method Heatmap — CSS Grid Approach
**What:** A CSS grid where rows and columns are method categories. Each cell `<div>` gets a background color from a sequential blue scale based on paper count. No external chart library — pure CSS.
**When to use:** AUI-03 method-combination matrix.
**Example:**
```rust
// Pseudo-code structure — actual colors set via CSS custom properties
fn cell_color_class(count: u32) -> &'static str {
    match count {
        0 => "cell-empty",
        1 => "cell-low",
        2..=5 => "cell-medium",
        _ => "cell-high",
    }
}

// CSS: .cell-empty { background: #1e1e2e; border: 1px dashed #444; }
//      .cell-low { background: #1e4a8a; }
//      .cell-high { background: #00cfff; }
```

### Anti-Patterns to Avoid
- **`ssr` feature in resyn-app:** Adding `ssr` to resyn-app Cargo.toml pulls in SurrealDB which fails WASM compilation. The boundary exists for this reason.
- **Direct DB access from server functions without context:** Server functions need DB injected via `provide_context` in the Axum handler — do not open a new DB connection per call.
- **Defining server functions in resyn-server:** Server functions must be defined in a crate compiled for both targets (WASM for the stub, native for the body). Define them in resyn-app with conditional compilation, or create a shared `app` crate.
- **Hardcoded port 3000 in WASM:** Server function URLs use relative paths (`/api/`) — they resolve against the page's origin. The Trunk proxy forwards them. Never hardcode backend URLs in WASM code.
- **Calling a server function directly from main Rust (not spawn_local):** Server functions are async — call them inside `Resource`, `Action`, or `spawn_local`. Blocking on them will panic.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SSE reconnection logic | Custom EventSource wrapper | leptos-use use_event_source | Handles ready state, errors, reconnect, WASM cleanup |
| Reactive async data | Manual fetch + state management | Leptos Resource + Suspense | Correct reactive graph integration, automatic re-fetching |
| Client-side routing | Window.location manipulation | leptos_router Route/A/use_navigate | Handles nested routes, history API, link resolution |
| Server function registration | Manual axum routes per function | leptos_axum handle_server_fns_with_context | Dispatches all #[server] functions from one wildcard route |
| WASM build pipeline | Custom webpack/wasm-pack scripts | Trunk + index.html | Handles wasm-bindgen, JS loader generation, dev server, proxy |
| Method matrix aggregation | Custom matrix data structure | Vec<(category, category, count)> + HashMap | Simple; no special graph library needed for NxN category matrix |

**Key insight:** Leptos's `#[server]` macro + `leptos_axum::handle_server_fns_with_context` eliminates the need to manually define REST endpoints. The macro generates both the HTTP stub (WASM) and the handler (server). The wildcard route on Axum dispatches to whichever server function matches by name.

## Common Pitfalls

### Pitfall 1: ProgressEvent Not WASM-Safe
**What goes wrong:** `ProgressEvent` is currently defined in `resyn-server/src/commands/crawl.rs`. The resyn-app WASM crate cannot import from resyn-server (it would pull in tokio, axum, etc.). Compilation fails.
**Why it happens:** The struct lives in the server binary crate, not in the shared resyn-core.
**How to avoid:** Move `ProgressEvent` to `resyn-core/src/datamodels/progress.rs` (no ssr gate needed — it's pure serde). Then both resyn-app and resyn-server can import it.
**Warning signs:** `error[E0432]: unresolved import resyn_server` in resyn-app build.

### Pitfall 2: Server Function Definition Crate Mismatch
**What goes wrong:** Server functions defined in resyn-app (compiled to WASM with `csr`) need their body to run server-side. In a single-crate setup, this requires conditional compilation inside the function body. If not carefully guarded, SSR-only types leak into the WASM build.
**Why it happens:** The `#[server]` macro generates different code per feature, but imports inside the body must still type-check for both targets.
**How to avoid:** Gate all SSR imports inside server function bodies with `#[cfg(feature = "ssr")]`. Use `use resyn_core::database::...` only behind that gate. The outer function signature uses only WASM-safe types (from resyn-core without ssr, serde types, etc.).
**Warning signs:** `error: can't find crate for surrealdb` during `cargo build --target wasm32-unknown-unknown`.

### Pitfall 3: Trunk Proxy Not Forwarding to Axum
**What goes wrong:** `trunk serve` starts on port 8080, server functions 404.
**Why it happens:** Axum server not running, or Trunk.toml proxy `rewrite` prefix doesn't match the `#[server]` macro's path prefix.
**How to avoid:** The server function macro `#[server(GetPapers, "/api")]` generates an endpoint at `/api/GetPapers`. The `[[proxy]]` must use `backend = "http://localhost:3000/api/"` and `rewrite = "/api/"`. Run `resyn serve` in a separate terminal before `trunk serve`.
**Warning signs:** Browser network tab shows 404 for POST `/api/GetPapers`.

### Pitfall 4: Method Heatmap Sparse Matrix Overwhelming
**What goes wrong:** If there are 20+ method categories, the heatmap becomes a 20×20 grid that's unreadable.
**Why it happens:** Displaying all categories simultaneously without filtering.
**How to avoid:** Cap the heatmap to the top N categories by paper count (e.g., top 10). Provide a "show more" expansion. The drill-down sub-matrix shows individual methods only for the two selected categories.
**Warning signs:** Heatmap renders but most cells are empty/zero.

### Pitfall 5: Sidebar CSS Transition State Not Reactive
**What goes wrong:** Sidebar collapse state managed with a JS class toggle doesn't integrate with Leptos reactivity — server functions called from sidebar components re-render incorrectly.
**Why it happens:** Mixing imperative DOM manipulation with Leptos's reactive graph.
**How to avoid:** Store sidebar collapse state as a `RwSignal<bool>`. Use `move || if collapsed.get() { "sidebar-collapsed" } else { "sidebar-expanded" }` in class attributes. All child components read this signal reactively.

### Pitfall 6: Server Function Context Not Provided
**What goes wrong:** Server function body calls `use_context::<Arc<Db>>()` but it returns `None`, causing a panic or `ServerFnError`.
**Why it happens:** `handle_server_fns_with_context` was not given the context provider closure, or the Axum state injection is missing.
**How to avoid:** Always pass a closure to `handle_server_fns_with_context` that calls `provide_context(db.clone())`. Verify with a minimal server function that returns the context or an error.

## Code Examples

Verified patterns from official sources:

### Cargo.toml for resyn-app (CSR)
```toml
# Source: https://book.leptos.dev/getting_started/index.html
# (adapted for workspace; csr feature required)
[dependencies]
resyn-core = { path = "../resyn-core" }  # NO ssr feature!
leptos = { version = "0.8", features = ["csr"] }
leptos_router = { version = "0.8" }
leptos_meta = { version = "0.8" }
leptos-use = { version = "0.18" }
codee = "0.3"
wasm-bindgen = { workspace = true }
getrandom = { workspace = true, features = ["wasm_js"] }
console_error_panic_hook = "0.1"
```

### Cargo.toml for resyn-server additions
```toml
# Add to resyn-server/Cargo.toml
leptos = { version = "0.8", features = ["ssr"] }
leptos_axum = { version = "0.8" }
resyn-app = { path = "../resyn-app" }  # to register server functions
tower-http = { workspace = true, features = ["fs", "cors"] }
```

### Trunk.toml
```toml
# Source: https://raw.githubusercontent.com/trunk-rs/trunk/main/Trunk.toml (canonical example)
[serve]
port = 8080

[[proxy]]
backend = "http://localhost:3000/api/"
rewrite = "/api/"

[[proxy]]
backend = "http://localhost:3000/progress"
```

### Leptos Router Setup (CSR)
```rust
// Source: https://github.com/leptos-rs/start-axum-workspace/blob/main/app/src/lib.rs
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes, A},
    path,
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Layout>
                <Routes fallback=|| "Not found".into_view()>
                    <Route path=path!("/") view=Dashboard/>
                    <Route path=path!("/papers") view=PapersPanel/>
                    <Route path=path!("/gaps") view=GapsPanel/>
                    <Route path=path!("/problems") view=OpenProblemsPanel/>
                    <Route path=path!("/methods") view=MethodsPanel/>
                </Routes>
            </Layout>
        </Router>
    }
}
```

### Open Problems Aggregation (Server-Side)
```rust
// Server function body — aggregates open_problems strings across all annotations
#[cfg(feature = "ssr")]
async fn aggregate_open_problems(db: &Arc<Db>) -> Result<Vec<(String, usize)>, ResynError> {
    let repo = AnnotationRepository::new(db);
    let annotations = repo.get_all_annotations().await?;
    let mut counts: std::collections::HashMap<String, usize> = Default::default();
    for ann in &annotations {
        for problem in &ann.open_problems {
            *counts.entry(problem.clone()).or_default() += 1;
        }
    }
    let mut ranked: Vec<_> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(ranked)
}
```

### Method Matrix Aggregation (Server-Side)
```rust
// For AUI-03: collect method category pairs per paper, build NxN count matrix
#[cfg(feature = "ssr")]
async fn build_method_matrix(db: &Arc<Db>) -> Result<MethodMatrix, ResynError> {
    let repo = AnnotationRepository::new(db);
    let annotations = repo.get_all_annotations().await?;
    let mut pair_counts: std::collections::HashMap<(String, String), u32> = Default::default();
    for ann in &annotations {
        let cats: Vec<_> = ann.methods.iter().map(|m| m.category.clone()).collect();
        for i in 0..cats.len() {
            for j in i..cats.len() {
                let key = if cats[i] <= cats[j] {
                    (cats[i].clone(), cats[j].clone())
                } else {
                    (cats[j].clone(), cats[i].clone())
                };
                *pair_counts.entry(key).or_default() += 1;
            }
        }
    }
    Ok(MethodMatrix { pair_counts })
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| leptos_sse crate | leptos-use use_event_source | ~2024 | leptos_sse is deprecated; leptos-use is the maintained path |
| create_resource() | Resource::new() | Leptos 0.7 → 0.8 | API change; old function still works but new API preferred |
| cargo-leptos for all setups | Trunk for CSR, cargo-leptos for SSR | Ongoing | CSR is simpler; no hydration or SSR complexity |
| Trunk 0.20 | Trunk 0.21.14 (stable) | 2025 | 0.22 is beta; use 0.21.14 |

**Deprecated/outdated:**
- `leptos_sse` crate: deprecated, maintainer recommends leptos-use use_event_source
- `create_resource()`: works but `Resource::new()` is the 0.8 idiomatic API
- `handle_server_fns()`: the non-context variant; prefer `handle_server_fns_with_context` for DB injection

## Open Questions

1. **ProgressEvent move to resyn-core**
   - What we know: Currently in resyn-server/src/commands/crawl.rs, not accessible from resyn-app WASM
   - What's unclear: Whether Phase 7 code depends on it being in crawl.rs (likely just local use)
   - Recommendation: Move to `resyn-core/src/datamodels/progress.rs` (no feature gate needed — pure serde). This is required before resyn-app can use it.

2. **Server function registration: single-crate vs shared `app` crate**
   - What we know: Official templates use a 3-crate workspace (app/frontend/server). This project has resyn-app (frontend) and resyn-server. There's no shared `app` crate yet.
   - What's unclear: Whether to add a 4th crate or use conditional compilation within resyn-app
   - Recommendation: Keep resyn-app as the shared crate. Define `#[server]` functions in `resyn-app/src/server_fns/`. resyn-server depends on resyn-app to ensure server functions are registered. The `ssr` feature on resyn-app enables the server-side bodies.

3. **Production serving**
   - What we know: For production, both static files and API must come from one origin to avoid CORS
   - What's unclear: Whether to add tower-http fs middleware to serve dist/ from Axum in production
   - Recommendation: Add `tower_http::services::ServeDir` as a fallback in the Axum router pointing at `../resyn-app/dist/`. This is the production path; Trunk proxy covers dev.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust built-in) |
| Config file | none — inline #[test] and #[cfg(test)] |
| Quick run command | `cargo test -p resyn-server serve` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WEB-03 | trunk serve compiles resyn-app to WASM without error | compile check | `cargo build -p resyn-app --target wasm32-unknown-unknown` | ❌ Wave 0 — add to CI or Makefile |
| WEB-04 | Server functions return data from DB | integration | `cargo test -p resyn-server server_fn` | ❌ Wave 0 |
| AUI-01 | gap findings aggregated correctly from DB | unit | `cargo test -p resyn-core gap_finding_repository` | ❌ Wave 0 |
| AUI-02 | open problems ranked by recurrence | unit | `cargo test -p resyn-core aggregate_open_problems` | ❌ Wave 0 |
| AUI-03 | method matrix pair counts correct | unit | `cargo test -p resyn-core build_method_matrix` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo check -p resyn-app --target wasm32-unknown-unknown && cargo check -p resyn-server`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`; manual browser check of `trunk serve`

### Wave 0 Gaps
- [ ] `resyn-core/src/datamodels/progress.rs` — move ProgressEvent here; covers WEB-03/crawl progress SSE
- [ ] Unit test for `aggregate_open_problems()` aggregation logic — covers AUI-02
- [ ] Unit test for `build_method_matrix()` pair counting — covers AUI-03
- [ ] Unit test for `GapFindingRepository::get_all_gap_findings()` (already exists in queries.rs tests — verify)
- [ ] `cargo build -p resyn-app --target wasm32-unknown-unknown` must pass as CI gate for WEB-03

## Sources

### Primary (HIGH confidence)
- https://crates.io/crates/leptos — version 0.8.17 confirmed 2026-03-17
- https://crates.io/crates/leptos_axum — version 0.8.8 confirmed
- https://crates.io/crates/leptos-use — version 0.18.3 confirmed, leptos = "0.8" dependency verified from Cargo.toml
- https://raw.githubusercontent.com/trunk-rs/trunk/main/Trunk.toml — canonical Trunk.toml with [[proxy]] format
- https://github.com/leptos-rs/start-axum-workspace — official workspace template (frontend/app/server crate pattern)
- https://leptos-use.rs/network/use_event_source.html — use_event_source API
- https://docs.rs/leptos_axum/latest/leptos_axum/ — handle_server_fns_with_context signature
- resyn-core source — PaperRepository, GapFindingRepository, LlmAnnotation structs verified directly

### Secondary (MEDIUM confidence)
- https://book.leptos.dev/server/25_server_functions.html — server function mechanism description
- https://book.leptos.dev/getting_started/index.html — Trunk CSR setup steps
- https://docs.rs/leptos_router/latest/leptos_router/ — Router API summary

### Tertiary (LOW confidence)
- None — all critical findings verified with official sources or crate source code

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified against crates.io registry 2026-03-17
- Architecture: HIGH — patterns derived from official templates and docs; Trunk proxy format from canonical Trunk.toml
- Pitfalls: HIGH — derived from actual codebase constraints (ssr feature gate) and known Leptos patterns
- SSE pattern: HIGH — leptos-use deprecation of leptos_sse is documented on the leptos_sse GitHub repo

**Research date:** 2026-03-17
**Valid until:** 2026-04-17 (leptos 0.8 is actively maintained; patch releases frequent but non-breaking)
