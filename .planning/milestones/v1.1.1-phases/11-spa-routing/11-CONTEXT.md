# Phase 11: SPA Routing - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Fix the Axum production server so that all Leptos client-side routes (`/`, `/papers`, `/gaps`, `/problems`, `/methods`, `/graph`) work on direct URL navigation and browser refresh — instead of returning 404 or serving no content.

</domain>

<decisions>
## Implementation Decisions

### SPA Fallback Strategy
- **D-01:** The Axum server must serve `index.html` for any request path that doesn't match a static file in `resyn-app/dist/`. This lets the Leptos client-side `<Router>` handle route resolution.
- **D-02:** The existing `/progress` (SSE) and `/api/{*fn_name}` (server functions) routes must continue to work — the fallback only applies to paths not matched by explicit routes or static files.

### Claude's Discretion
- Choice of `tower-http` API (`ServeDir::fallback`, `ServeFile`, or a catch-all Axum handler) is implementer's choice — all achieve the same result.
- Whether to add a `Cache-Control` header for `index.html` (no-cache) vs static assets (long-cache) is optional polish.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Server Setup
- `resyn-server/src/commands/serve.rs` — Axum router with `ServeDir` fallback (line 93 is the problem)

### Client-Side Routing
- `resyn-app/src/app.rs` — Leptos `<Router>` with all route definitions
- `resyn-app/src/layout/sidebar.rs` — Navigation links using `<A href=...>`

### Build Config
- `resyn-app/Trunk.toml` — Trunk build config, dev proxy settings

</canonical_refs>

<code_context>
## Existing Code Insights

### Root Cause
- `serve.rs:93` uses `.fallback_service(ServeDir::new("resyn-app/dist"))` — this only serves files that physically exist in `dist/`. Navigating to `/graph` looks for `dist/graph` which doesn't exist → 404.

### Reusable Assets
- `tower-http::services::ServeDir` already imported — has a `.fallback()` method or can be combined with `ServeFile`
- `tower-http::services::ServeFile` can serve `index.html` as the fallback

### Established Patterns
- Explicit Axum routes for `/progress` and `/api/{*fn_name}` are registered before the fallback — these will naturally take priority over any catch-all

### Integration Points
- Only `resyn-server/src/commands/serve.rs` needs modification
- Trunk dev server already handles SPA routing via its proxy config — no changes needed there

</code_context>

<specifics>
## Specific Ideas

No specific requirements — standard SPA fallback pattern applies.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 11-spa-routing*
*Context gathered: 2026-03-23*
