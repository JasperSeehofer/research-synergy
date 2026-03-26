# Phase 16: Edge and Node Renderer Fixes - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Fix edge visibility and node sharpness in both Canvas 2D and WebGL2 renderers, and add seed node distinction. Citation edges must be legible at default zoom on the dark (#0d1117) background. Node circles must have sharp, clean borders at all zoom levels. The seed paper node must be immediately distinguishable. Visual consistency between Canvas 2D (<300 nodes) and WebGL2 (300+ nodes) renderers.

</domain>

<decisions>
## Implementation Decisions

### Edge Visibility
- **D-01:** Edge color is a single muted gray `#8b949e` for all regular citation edges. No color-coding by type.
- **D-02:** Edge alpha fades by BFS depth distance: ~0.5 for depth-1 edges, decreasing to ~0.15 for deepest edges. This provides a subtle depth signal without multiple colors.
- **D-03:** Regular edge line width is 1.5px in both Canvas 2D and WebGL2 renderers.
- **D-04:** Subtle arrowheads at target end of each edge, same color as edge, showing citation direction (who-cites-whom).
- **D-05:** Contradiction edges (`#f85149`) and ABC-bridge edges (`#d29922`) retain their existing colors and always-opaque alpha. Only regular citation edges get depth-based alpha.

### WebGL2 Quad Edge Geometry
- **D-06:** Replace `GL.LINES` (1px hardware cap) with quad-based triangle geometry — two triangles per edge segment — for proper 1.5px width control.
- **D-07:** Fragment shader uses distance-from-center-of-quad for soft anti-aliased edge borders. No hard pixel boundaries.
- **D-08:** Arrowheads rendered as separate triangle pass (second draw call), not integrated into quad mesh. Matches current Canvas 2D approach for simplicity.

### Node Sharpness
- **D-09:** Node fill is flat solid color — no gradient or shading. Clean, fast, consistent between Canvas 2D and WebGL2.
- **D-10:** Thin bright border on node circles — a lighter shade of the node fill color (brighter blue ring around blue node). Crisp at all zoom levels.
- **D-11:** WebGL2 node shader uses `fwidth()` for resolution-independent anti-aliasing of both the node edge and border ring. Replaces current `smoothstep(0.9, 1.0, d)`.
- **D-12:** Canvas 2D node border updated from current dark `#30363d` (invisible on dark background) to a brighter shade matching the WebGL2 approach.
- **D-13:** Border width scaled by inverse viewport scale so it remains visually consistent (e.g., 1px screen-space) regardless of zoom level.

### Seed Node Style
- **D-14:** Seed paper node fill color: warm amber `#d29922`. Distinct from the blue `#4a9eff` of regular nodes.
- **D-15:** Seed node has a solid amber outer ring with a 2px transparent gap between the node fill circle and the ring. Planetary ring effect — unmistakable at any zoom.
- **D-16:** Seed node label follows the same LOD visibility rules as other nodes (appears at scale > 0.6). No always-on exception.
- **D-17:** `is_seed` flag added to `NodeState` struct for quick lookup during rendering, derived from `GraphState.seed_paper_id`.

### Claude's Discretion
- Exact alpha values per BFS depth level (within the 0.15-0.5 range specified)
- Quad edge vertex buffer layout and attribute stride
- fwidth() smoothing parameters for node border anti-aliasing
- Outer ring radius offset and thickness (within the "2px gap, solid ring" constraint)
- Arrowhead size scaling (currently 8.0 world units — may need adjustment for 1.5px edges)
- Exact brighter-shade calculation for node borders

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Edge Rendering
- `resyn-app/src/graph/canvas_renderer.rs` lines 77-162 — Current Canvas 2D edge rendering (regular, contradiction, bridge), arrowheads at lines 165-204, 287-312
- `resyn-app/src/graph/webgl_renderer.rs` lines 53-82 — Current WebGL2 edge shaders (EDGE_VERT, EDGE_FRAG), edge rendering at lines 233-296
- `resyn-app/src/graph/webgl_renderer.rs` lines 437-453 — Edge color/alpha function

### Node Rendering
- `resyn-app/src/graph/canvas_renderer.rs` lines 206-274 — Canvas 2D node circles, borders, labels
- `resyn-app/src/graph/webgl_renderer.rs` lines 13-51 — WebGL2 node shaders (NODE_VERT, NODE_FRAG with smoothstep)
- `resyn-app/src/graph/webgl_renderer.rs` lines 313-426 — WebGL2 instanced node rendering, instance data format

### Renderer Architecture
- `resyn-app/src/graph/renderer.rs` lines 76-102 — Renderer selection (WEBGL_THRESHOLD = 300), probe logic
- `resyn-app/src/graph/layout_state.rs` lines 41-56 — GraphState struct with seed_paper_id
- `resyn-app/src/graph/lod.rs` lines 9-31 — LOD visibility rules, seed always visible

### Requirements
- `.planning/REQUIREMENTS.md` — EDGE-01, EDGE-02, EDGE-03, NODE-01, NODE-02, NODE-03

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `canvas_renderer.rs` edge rendering loop — structure can be extended for depth-based alpha; arrowhead geometry already computed
- `webgl_renderer.rs` instanced node rendering — add `is_seed` to instance data for per-node color selection in shader
- `layout_state.rs:seed_paper_id` — already tracked, just needs to propagate as `is_seed` flag on `NodeState`
- `lod.rs` seed detection logic — already compares node ID against seed_paper_id, pattern can be reused

### Established Patterns
- Edge data format: 6 floats per vertex `[x, y, r, g, b, alpha]` — extend for quad geometry
- Node instance data: 7 floats `[x, y, radius, alpha, r, g, b]` — add `is_seed` flag (8th float)
- Color constants defined at module top as hex strings with `hex_to_rgb()` conversion
- Canvas 2D and WebGL2 renderers have parallel structure — changes to one should mirror in the other

### Integration Points
- `NodeState` struct needs `is_seed: bool` field
- `from_graph_data()` in `layout_state.rs` needs to set `is_seed` during initialization
- WebGL2 node shader (NODE_FRAG) needs border ring logic and seed color branching
- WebGL2 edge shader needs new quad vertex format with perpendicular offset
- Both renderers need BFS depth info on edges for alpha calculation — may need to pass through edge data

</code_context>

<specifics>
## Specific Ideas

- Depth-based alpha fade gives a subtle information layer without visual clutter — deeper connections naturally recede
- Seed node planetary ring effect (gap + outer ring) should be unmistakable — the first thing you notice when looking at the graph
- Anti-aliased edges and nodes should look good on both standard and HiDPI displays (fwidth handles this in WebGL)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 16-edge-and-node-renderer-fixes*
*Context gathered: 2026-03-25*
