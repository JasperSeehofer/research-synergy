# Phase 9: Graph Renderer (Canvas to WebGL) - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Render the citation graph interactively in the browser using a full Rust/WASM pipeline. Canvas 2D for small graphs, WebGL2 for 1000+ nodes, with Barnes-Hut force layout computed in a Web Worker. Integrates into the existing Leptos app shell from Phase 8 as a new "Graph" page. No new analysis features (Phase 10). No JavaScript graph libraries.

</domain>

<decisions>
## Implementation Decisions

### Graph interactions
- Click a node opens the Phase 8 paper side drawer (reuse existing Drawer component) — graph stays visible underneath
- Hover a node shows tooltip with paper title, authors, and year (near cursor, no layout disruption)
- Drag a node pins it in place (stops force layout from moving it). Click the pinned node again to unpin
- Clicking a node highlights its direct neighbors and connected edges, dims everything else (opacity reduction on non-connected elements)
- Pan via click-drag on canvas background, zoom via scroll wheel

### Edge rendering
- Arrowheads on edge target end showing citation direction (A cites B)
- Contradiction edges: red, solid, rendered on top of regular edges
- ABC-bridge edges: orange, dashed, rendered on top of regular edges
- Special edges (contradiction + bridge) visible by default, togglable via controls (toggle buttons matching Phase 8 gap panel filter pattern)
- Regular citation edges: thin gray lines with low opacity — subtle enough to let special edges pop
- Hover tooltip on ALL edges: regular edges show "A cites B" with paper titles, special edges show gap finding summary (shared terms, confidence, justification snippet)

### Layout & convergence
- Barnes-Hut O(n log n) force layout runs in Rust/WASM Web Worker (already decided)
- Animate from random start — nodes visibly settle into position (no pre-convergence)
- Play/pause button to freeze/resume the force simulation
- Dragging a pinned node does NOT restart the simulation; unpinning restarts it for that node
- Node size scaled by citation count (more-cited papers = larger nodes)
- Node labels show "first author + year" (e.g., "Einstein 2024") — compact, readable at moderate zoom

### Canvas-to-WebGL transition
- Automatic switch by node count: Canvas 2D under threshold (~200-500 nodes), WebGL2 above
- WebGL2 support detected on init; if unavailable, fall back to Canvas 2D regardless of node count (log a warning)
- Shared `Renderer` trait implemented by both Canvas2DRenderer and WebGL2Renderer — clean runtime switching, both maintained
- Transition is transparent to the user — they just see smooth rendering

### App integration
- New "Graph" entry in the Phase 8 sidebar (after Methods, before any future additions)
- Graph page is a full-page canvas with overlay controls (edge toggles, play/pause, zoom controls)
- Paper drawer slides in from right over the graph canvas (same as paper table behavior)

### Claude's Discretion
- Exact Barnes-Hut theta parameter and force layout tuning constants
- Web Worker message protocol and serialization format
- Canvas 2D drawing implementation details (how to render arrowheads, node circles)
- WebGL2 shader programs and buffer management
- Exact node count threshold for Canvas-to-WebGL switch
- Zoom level at which labels appear/disappear
- CSS for graph controls overlay
- How to serialize/deserialize graph data between main thread and Web Worker

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Graph data layer
- `resyn-core/src/data_processing/graph_creation.rs` — `create_graph_from_papers()` builds `StableGraph<Paper, f32, Directed>` from papers; WASM-safe, no ssr gate
- `resyn-core/src/datamodels/paper.rs` — `Paper` struct with citation_count, authors, title, year fields used for node rendering
- `resyn-core/src/datamodels/gap_finding.rs` — `GapFinding` with `GapType::Contradiction` / `GapType::AbcBridge`, paper_ids, shared_terms, justification, confidence — feeds edge coloring

### Existing app shell
- `resyn-app/src/app.rs` — Leptos app root with Router, sidebar, drawer integration
- `resyn-app/src/layout/sidebar.rs` — Sidebar component with nav items (add Graph entry here)
- `resyn-app/src/layout/drawer.rs` — Paper side drawer component (reuse for node click)
- `resyn-app/src/components/gap_card.rs` — Toggle filter pattern for contradiction/bridge (reuse pattern for edge toggles)

### Build pipeline
- `resyn-app/Cargo.toml` — CSR WASM app dependencies, web-sys/wasm-bindgen already available
- `Trunk.toml` / `resyn-app/index.html` — Trunk build config for WASM app

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `create_graph_from_papers()` in `resyn-core/src/data_processing/graph_creation.rs`: builds petgraph StableGraph from papers — foundation for graph data
- `Drawer` component in `resyn-app/src/layout/drawer.rs`: paper detail side drawer — reuse directly for node click
- `GapFinding` in `resyn-core/src/datamodels/gap_finding.rs`: contradiction/bridge findings with paper_ids — maps directly to colored edge overlays
- Toggle filter pattern in gap panel (`resyn-app/src/pages/gaps.rs`): contradiction/bridge toggles — same UX pattern for edge visibility controls
- `wasm-bindgen` already in resyn-app deps — foundation for web-sys Canvas/WebGL bindings

### Established Patterns
- CSR-only with Trunk (no SSR/hydration) — graph component will be a standard Leptos component
- `ssr` feature gate on resyn-core: data_processing/graph_creation is WASM-safe (no ssr gate), always available in frontend
- Dark minimal theme with separate CSS files loaded by Trunk
- Sidebar + content layout with collapsible sidebar rail

### Integration Points
- Sidebar nav: add "Graph" item to existing sidebar component
- Router: add `/graph` route to existing Leptos router in app.rs
- Server functions: need a server fn to fetch graph data (papers + edges + gap findings) for rendering
- Paper drawer: node click triggers same drawer open logic as paper table row click

</code_context>

<specifics>
## Specific Ideas

- Graph should feel like a proper research exploration tool — click a node, see the paper, explore the neighborhood
- Edge toggles should feel consistent with the gap panel filter toggles from Phase 8
- The force layout animation from random start gives a sense of the graph structure "emerging" — engaging for researchers
- Node size by citation count gives an immediate visual sense of paper importance without needing to read labels
- "First author + year" labels are the standard academic citation shorthand — familiar to researchers

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 09-graph-renderer-canvas-to-webgl*
*Context gathered: 2026-03-17*
