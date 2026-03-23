# Phase 12: Graph Force & Rendering - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Fix the force-directed graph so that: (1) nodes visibly animate and spread apart during force simulation, (2) citation edges are drawn between connected nodes, and (3) all rendering is crisp without DPR blur. This is a bugfix phase — the rendering pipeline, force simulation, and edge drawing code all exist but produce no visible output or blurry output.

</domain>

<decisions>
## Implementation Decisions

### Force Animation Debugging
- **D-01:** Claude's discretion on diagnostic approach — may use console logging, parameter tuning, code inspection, or any combination to identify why `run_ticks()` computes new positions but they don't visually appear. The force math itself is confirmed working (8 passing unit tests).

### DPR / Rendering Crispness
- **D-02:** Fix and verify DPR handling end-to-end in a single pass: canvas sizing, GL viewport, shader `u_resolution` uniforms, and `screen_to_world` / `world_to_screen` coordinate transforms must all use a consistent convention. Document the coordinate convention so Phase 13 (interaction) can rely on it.
- **D-03:** The current DPR fix (dividing `self.width`/`self.height` by `dpr` in shader uniforms) may have broken coordinate mapping — investigate whether this is correct or needs revision.

### Edge Rendering
- **D-04:** Edge rendering shares the same viewport/DPR/coordinate pipeline as node rendering. Treat it as the same root cause — fixing the rendering pipeline once should make both nodes and edges appear correctly. No separate edge-specific investigation needed unless the pipeline fix doesn't resolve edges.

### Claude's Discretion
- Force animation debugging approach (diagnostic logging, parameter tuning, code inspection — whatever is fastest)
- Whether to add temporary debug logging (remove after fix confirmed)
- Order of investigation (force first vs DPR first vs simultaneous)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Force Simulation
- `resyn-worker/src/forces.rs` — Barnes-Hut force simulation with tunable constants (REPULSION=-200, ATTRACTION=0.3, CENTER_GRAVITY=0.02, ALPHA_DECAY=0.997, DAMPING=0.4, IDEAL_DISTANCE=80)
- `resyn-app/src/graph/layout_state.rs` — `GraphState::from_graph_data()` sets initial node positions (spiral + hash jitter), velocities, alpha

### Rendering Pipeline
- `resyn-app/src/graph/webgl_renderer.rs` — WebGL2 shaders (node circle SDF + edge lines + arrowheads), DPR compensation at lines ~164-166
- `resyn-app/src/graph/renderer.rs` — `Renderer` trait, `Viewport` struct with `world_to_screen`/`screen_to_world`

### RAF Loop & Integration
- `resyn-app/src/pages/graph.rs` — RAF render loop (lines 344-362 run inline `run_ticks()`), canvas setup with DPR sizing, `ResizeObserver`, event listener attachment
- `resyn-app/src/graph/worker_bridge.rs` — Worker bridge (currently bypassed — inline layout used instead)

### Known Bug State
- `.planning/BUGFIX-STATUS.md` — Documents all known bugs, attempted fixes, and remaining issues from v1.1 session

### Verification
- `resyn-app/src/server_fns/graph.rs` — `get_graph_data()` server function (confirmed returning 375 nodes, 720 edges)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `resyn_worker::forces::run_ticks()` — Complete Barnes-Hut simulation, 8 passing tests, called inline from RAF loop
- WebGL2 shaders for nodes (circle SDF with smoothstep edge) and edges (lines + arrowhead triangles)
- `Viewport` struct handles world-to-screen and screen-to-world transforms
- `ResizeObserver` already handles canvas resize with DPR

### Established Patterns
- RAF loop runs at 60fps, one force tick per frame, checks convergence (alpha < 0.001)
- Canvas sized at `css_size * dpr` pixels, WebGL viewport set to full pixel dimensions
- Shader uniforms use `u_resolution`, `u_offset`, `u_scale` for world-to-clip transform
- LOD and temporal visibility filter nodes before render

### Integration Points
- RAF loop in `graph.rs` is the integration point — reads `GraphState`, calls `run_ticks()`, calls `renderer.draw()`
- Viewport coordinates flow: initial setup in `Effect::new` → stored in `RenderState` → passed to `renderer.draw()`
- DPR convention must be consistent across: canvas sizing, GL viewport, shader uniforms, Viewport struct

### Key Debug Info from Previous Session
- RAF loop IS running (confirmed via `document.title` debug showing frame count, sim=true, alpha=0.86)
- `run_ticks()` IS being called and computing new positions
- User reports NO visible animation and CANNOT interact with nodes
- Possible causes: positions update but rendering doesn't reflect them, or viewport transform maps all nodes to same screen position

</code_context>

<specifics>
## Specific Ideas

- Use agent-browser CLI for automated verification — take screenshots to confirm nodes moved, edges drawn, rendering crisp
- Document the DPR coordinate convention clearly so Phase 13 can build interaction on solid ground

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 12-graph-force-rendering*
*Context gathered: 2026-03-23*
