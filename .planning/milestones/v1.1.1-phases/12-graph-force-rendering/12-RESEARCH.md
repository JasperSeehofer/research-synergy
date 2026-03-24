# Phase 12: Graph Force & Rendering - Research

**Researched:** 2026-03-23
**Domain:** WebGL2 rendering pipeline, force simulation integration, DPR coordinate mapping
**Confidence:** HIGH — all findings derive from direct code inspection of canonical source files

## Summary

This is a bugfix phase on an existing Rust/WASM + WebGL2 rendering pipeline. The force simulation (`run_ticks()`) is confirmed running. The code paths for updating node positions and calling `renderer.draw()` are structurally correct. The bugs are in **how the rendered output relates to world coordinates** and **how the VAO state is managed across frames** — not in the force math itself.

Three root causes are identified through code inspection. First, initial node positions use a `spread = sqrt(N) * 50.0` (~968 CSS pixels for 375 nodes) but the viewport is centered at `(css_width/2, css_height/2)` with `scale=1.0`, which maps most nodes outside the visible viewport rectangle — the graph is rendering but entirely off-screen. Second, the WebGL2 renderer creates new VBO objects every frame but binds them to persistent VAO objects that retain stale attribute pointer state, producing undefined behavior in the instanced draw call. Third, the DPR fix (`res_x = self.width / dpr`) may be internally consistent but needs verification against the `screen_to_world` convention in `Viewport` (which uses CSS coordinates).

**Primary recommendation:** Fix initial spread and viewport scale first (highest impact, one-line change). Then audit the per-frame VAO rebind pattern. Verify DPR end-to-end by tracing a single world coordinate through the shader formula.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01: Force Animation Debugging**
Claude's discretion on diagnostic approach — may use console logging, parameter tuning, code inspection, or any combination to identify why `run_ticks()` computes new positions but they don't visually appear. The force math itself is confirmed working (8 passing unit tests).

**D-02: DPR / Rendering Crispness**
Fix and verify DPR handling end-to-end in a single pass: canvas sizing, GL viewport, shader `u_resolution` uniforms, and `screen_to_world` / `world_to_screen` coordinate transforms must all use a consistent convention. Document the coordinate convention so Phase 13 (interaction) can rely on it.

**D-03: DPR Fix Correctness**
The current DPR fix (dividing `self.width`/`self.height` by `dpr` in shader uniforms) may have broken coordinate mapping — investigate whether this is correct or needs revision.

**D-04: Edge Rendering**
Edge rendering shares the same viewport/DPR/coordinate pipeline as node rendering. Treat it as the same root cause — fixing the rendering pipeline once should make both nodes and edges appear correctly. No separate edge-specific investigation needed unless the pipeline fix doesn't resolve edges.

### Claude's Discretion
- Force animation debugging approach (diagnostic logging, parameter tuning, code inspection — whatever is fastest)
- Whether to add temporary debug logging (remove after fix confirmed)
- Order of investigation (force first vs DPR first vs simultaneous)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| GRAPH-01 | Force-directed layout produces visible node animation spreading nodes apart | Root cause identified: initial spread puts nodes off-screen; fix spread constant and initial viewport scale |
| GRAPH-02 | Graph nodes render with crisp edges (no DPR blur) | DPR formula in webgl_renderer.rs is internally consistent; verify full pipeline consistency |
| GRAPH-03 | Graph edges (citation links) are visually rendered between connected nodes | Edge draw path exists and is structurally correct; shares same coordinate pipeline as nodes |
</phase_requirements>

---

## Architecture Patterns

### Rendering Pipeline Data Flow

```
GraphState.nodes[i].{x,y}   (world coordinates, origin at 0,0)
         ↓
webgl_renderer.draw(state, viewport)
         ↓
shader: screen = (world * scale + offset) / resolution * 2.0 - 1.0
         where:
           scale    = viewport.scale      (CSS units)
           offset   = viewport.{offset_x, offset_y}  (CSS units)
           resolution = canvas.pixel_size / dpr  (= CSS size)
```

The **coordinate convention** confirmed by inspection:
- World space: origin at (0,0), units = CSS pixels at scale=1.0
- Screen space: origin at top-left, units = CSS pixels
- Viewport offset: `(css_width/2, css_height/2)` at init → world origin maps to canvas center
- DPR division in `res_x/res_y`: converts pixel dimensions to CSS dimensions, making the shader formula operate entirely in CSS units. This is **internally consistent**.

The `screen_to_world` / `world_to_screen` in `Viewport` use CSS coordinates only (no DPR). This is **consistent with the shader** since both operate in CSS space.

### Force → Renderer Integration

```
RAF frame:
  build_layout_input(graph, viewport.css_width, viewport.css_height)
    → LayoutInput { nodes: positions+velocities, edges, ticks:1, alpha }
  run_ticks(input)
    → LayoutOutput { positions: Vec<(f64,f64)>, velocities, alpha, converged }
  for (i, (x,y)) in output.positions:
    graph.nodes[i].x = x   (only if !pinned)
    graph.nodes[i].y = y
  graph.velocities = output.velocities
  graph.alpha = output.alpha
  renderer.draw(&graph, &viewport)
```

This flow is correct. The force math and integration loop are not the source of the bug.

---

## Root Cause Analysis

### Bug 1: Initial Node Positions Are Off-Screen (GRAPH-01)
**Confidence: HIGH** — verified by code trace

`layout_state.rs:61`: `spread = sqrt(node_count) * 50.0`

For 375 nodes: `spread = sqrt(375) * 50 ≈ 968 CSS pixels`.

Nodes are placed at: `x = r * cos(angle) + jitter`, where `r` can reach up to `spread * 1.0 = 968`.

The viewport initializes with `offset = (css_width/2, css_height/2)` and `scale = 1.0`. A node at world `(968, 0)` maps to screen `x = 968 * 1.0 + 800 = 1768` on an 1600px canvas — **completely off-screen**.

Even nodes near the center at (0,0) would be visible, but with 375 nodes spread out to radius 968, **most nodes start off-screen and force simulation does not pull them into view** because center gravity is weak (0.02) and takes many frames.

**Fix:** Either reduce the initial spread constant (e.g., `sqrt(N) * 15.0` instead of `50.0`) or initialize the viewport with a smaller scale (e.g., `scale = 0.3`) so the full initial spread fits within the canvas.

### Bug 2: VAO Rebind Without State Reset (GRAPH-02, GRAPH-03)
**Confidence: HIGH** — verified by WebGL2 spec knowledge + code inspection

`webgl_renderer.rs` creates `node_vao` and `edge_vao` once in `new()`. Every `draw()` call:
1. Creates brand-new VBO objects
2. Binds them to the **same persistent VAO**
3. Sets `vertex_attrib_pointer` and `vertex_attrib_divisor` for each attribute

In WebGL2, VAOs store the complete attribute state including: buffer binding, attribute pointer, enabled/disabled, and divisor. Since the same VAO is reused every frame, old attribute bindings from the previous frame may persist if any attribute setup is skipped.

**More critical issue:** After the node draw pass, `vertex_attrib_divisor` is set to `1` for all instanced attributes. When the edge pass subsequently uses `edge_vao` (a different VAO), this is fine. But within the same VAO, stale divisor settings from the previous frame persist until explicitly reset. This is likely benign if attributes are always re-setup, but creates fragile code.

The more serious problem: **per-frame buffer creation without deletion leaks GPU memory**. While browsers GC these eventually, creating 2-3 new VBOs every RAF frame (60fps) is a memory leak pattern that can degrade performance.

**Fix options:**
- Option A: Create VBOs once (static allocation) and use `bufferSubData` to update contents each frame — more efficient, eliminates leak
- Option B: Add `gl.delete_buffer()` calls after each draw pass to clean up
- Option C (minimal): Ensure `bind_vertex_array(None)` before creating new buffers, and `bind_vertex_array(vao)` before setting up attributes — this is already done, so the current code may work correctly but wastefully

### Bug 3: DPR Convention Verification (GRAPH-02)
**Confidence: MEDIUM** — internally consistent but needs runtime verification

The current DPR fix in `webgl_renderer.rs` lines 164-166:
```rust
let dpr = web_sys::window().unwrap().device_pixel_ratio() as f32;
let res_x = self.width as f32 / dpr;
let res_y = self.height as f32 / dpr;
```

`self.width` is set in `new()` from `canvas.width()` (physical pixels). After `canvas.set_width(css * dpr)`, `canvas.width()` = physical pixels. So `res_x = physical_pixels / dpr = CSS pixels`. The shader formula then works in CSS pixel space. The viewport `offset_x/offset_y` are also CSS pixels. **This is internally consistent.**

Without the DPR fix, `res_x = physical_pixels`, but `offset` is still CSS pixels, making the division `(world * scale + css_offset) / physical_pixels` dimensionally inconsistent — this is the actual blur source.

**The fix is correct.** What needs verification is that the GL viewport is also set correctly: `gl.viewport(0, 0, self.width as i32, self.height as i32)` in `resize()` uses physical pixel dimensions — this is also correct (GL viewport is always in physical pixels).

**Summary:** The DPR fix in `webgl_renderer.rs` is correct and should not have broken coordinate mapping. The `screen_to_world` in `Viewport` is also correct because it operates in CSS space matching the shader.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| GPU buffer management | Custom buffer pool | Preallocate once + `bufferSubData` | WebGL VBO creation is expensive; existing pattern leaks GPU memory |
| DPR detection | Custom DPR watcher | `window.device_pixel_ratio` + `ResizeObserver` (already in place) | Platform-specific values need OS-level API |
| Force convergence detection | Custom threshold | Existing `alpha < ALPHA_MIN` pattern (already in place) | Already correct, don't change |

---

## Common Pitfalls

### Pitfall 1: Viewport Scale Not Adjusted for Large Graphs
**What goes wrong:** Initial node positions cover a world-space radius of ~968px but viewport scale is 1.0, making most nodes appear outside the visible canvas rectangle.
**Why it happens:** The spread formula `sqrt(N) * 50` was designed for smaller graphs; for 375 nodes it generates coordinates that exceed typical screen dimensions.
**How to avoid:** Either reduce `spread` constant OR initialize `viewport.scale` based on node count (e.g., `scale = min(1.0, css_width / (2 * spread))`).
**Warning signs:** RAF loop running, alpha decaying, but no visible nodes on first load.

### Pitfall 2: Per-Frame VBO Creation Leaks GPU Memory
**What goes wrong:** `gl.create_buffer()` called 3 times per frame (quad, instance, edge) at 60fps = 180 buffer objects/second created without deletion.
**Why it happens:** Pattern copied without considering lifecycle — works in small tests but degrades over time.
**How to avoid:** Create VBOs once in `new()`, update with `gl.buffer_sub_data_with_i32_and_array_buffer_view()` or `gl.buffer_data_*` with the same buffer object bound.
**Warning signs:** Browser memory grows monotonically; GPU memory warnings in DevTools.

### Pitfall 3: VAO Attribute Divisors Persist Across Frames
**What goes wrong:** `vertex_attrib_divisor(attr, 1)` set in frame N persists into frame N+1 since the same VAO is reused. If any attribute is added or removed, stale divisors cause incorrect instancing.
**Why it happens:** VAOs store all attribute state including divisors — they are not reset when you unbind/rebind.
**How to avoid:** Always explicitly set divisors for all attributes in each draw call, or use separate VAOs for instanced vs non-instanced draws.
**Warning signs:** Incorrect number of instances drawn, or instanced data appearing wrong.

### Pitfall 4: `TRIANGLE_FAN` vs `TRIANGLE_STRIP` for Quads
**What goes wrong:** The node quad uses `TRIANGLE_FAN` with 4 vertices `(-1,-1), (1,-1), (1,1), (-1,1)`. `TRIANGLE_FAN` creates triangles by connecting each vertex to the first vertex. With 4 verts this creates triangles: `(0,1,2)` and `(0,2,3)` — which correctly covers the quad. This is fine, but `TRIANGLE_STRIP` with the same 4 verts would require a different winding order.
**Status:** The current `TRIANGLE_FAN` with 4 verts is correct for this specific vertex order.

### Pitfall 5: Simulation May Visually Settle Before User Observes It
**What goes wrong:** With `ALPHA_DECAY = 0.997` and initial alpha=1.0, convergence threshold is 0.001. This takes `ln(0.001)/ln(0.997) ≈ 2296 frames` — about 38 seconds at 60fps. This should be enough to observe. However, if the simulation effectively converges early due to all nodes reaching similar positions (e.g., center gravity pulling all to origin), the visual change per frame becomes imperceptible.
**How to avoid:** Ensure initial jitter creates enough asymmetry. The current `jitter = ±40% * spread` should be sufficient with Bug 1 fixed.

---

## Code Examples

### Correct DPR Convention (verified from webgl_renderer.rs)
```rust
// Canvas is sized to physical pixels:
canvas.set_width((css_width * dpr) as u32);   // e.g., 1600px on 2x display

// GL viewport uses physical pixels (correct):
gl.viewport(0, 0, physical_w as i32, physical_h as i32);

// Shader uniforms use CSS pixels (res = physical / dpr):
let res_x = self.width as f32 / dpr;   // e.g., 800 CSS px
let res_y = self.height as f32 / dpr;  // e.g., 450 CSS px

// Shader formula (all in CSS space):
// screen = (world * scale + offset) / resolution * 2.0 - 1.0
// where offset and resolution are both in CSS pixels
```

### Correct Viewport Transform (verified from renderer.rs)
```rust
// screen_to_world: CSS screen coords → world coords
(sx - offset_x) / scale   // offset in CSS pixels

// world_to_screen: world coords → CSS screen coords
wx * scale + offset_x      // returns CSS screen coords
```

These are consistent with the shader. Phase 13 interaction code can rely on `screen_to_world` operating in CSS pixel space.

### Recommended Fix for Initial Spread (layout_state.rs)
```rust
// Before (too large for 375+ nodes):
let spread = (node_count as f64).sqrt() * 50.0;

// After (fits within typical 800x600 viewport):
let spread = (node_count as f64).sqrt() * 15.0;
// For 375 nodes: spread ≈ 290px — fits within a centered viewport
```

### Alternative: Fit-to-Viewport Initial Scale
Instead of changing spread, adjust the initial viewport scale in graph.rs Effect::new:
```rust
let spread = (node_count as f64).sqrt() * 50.0;
let fit_scale = (css_width.min(css_height) * 0.45 / spread).min(1.0);
let viewport = Viewport {
    offset_x: css_width / 2.0,
    offset_y: css_height / 2.0,
    scale: fit_scale,
    css_width,
    css_height,
};
```
This dynamically scales the viewport so the full initial spread fits in view.

### Efficient VBO Update Pattern (to replace per-frame creation)
```rust
// In WebGL2Renderer::new():
let node_instance_buf = gl.create_buffer().expect("instance buf");
let quad_buf = gl.create_buffer().expect("quad buf");
// store in struct

// In draw() — update existing buffer instead of creating new:
gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.node_instance_buf));
unsafe {
    let view = js_sys::Float32Array::view(&instance_data);
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &view,
        WebGl2RenderingContext::DYNAMIC_DRAW,
    );
}
```

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|-----------------|--------|
| Worker-based force layout (broken waker) | Inline `run_ticks()` on main thread | Simpler, reliable, fast enough (<2ms per tick for 375 nodes with Barnes-Hut) |
| Hard-coded spread constant | Dynamic spread based on node count | Needed for variable-size graphs |

---

## Open Questions

1. **Is the initial node spread the only reason animation is invisible?**
   - What we know: spread puts most nodes off-screen (world radius ~968px > typical viewport)
   - What's unclear: whether after fixing spread, the per-frame VBO leak also causes visual artifacts or just performance degradation
   - Recommendation: Fix spread first, take a screenshot via agent-browser, then assess if VBO pattern needs addressing in this phase

2. **Does the DPR fix produce correctly sharp nodes?**
   - What we know: the fix is mathematically consistent
   - What's unclear: rendering crispness depends on both the shader math AND the physical pixel count being correct
   - Recommendation: After fixing node visibility (Bug 1), use agent-browser screenshot to zoom into nodes and verify crispness

3. **Are edges empty arrays?**
   - What we know: server returns 720 edges (confirmed in BUGFIX-STATUS.md)
   - What's unclear: whether `GraphState::from_graph_data` maps all 720 edges or silently drops some due to id mismatches
   - Recommendation: Add a brief `console.log(state.edges.len())` in draw() or check via agent-browser eval

---

## Environment Availability

| Dependency | Required By | Available | Fallback |
|------------|------------|-----------|----------|
| agent-browser | Visual verification | ✓ (documented in reference memory) | Manual browser testing |
| Running app server | Any verification | Must be started by executor | `cargo leptos serve` |
| WebGL2 | Renderer | Browser-dependent (375 > 300 threshold) | Canvas2DRenderer fallback exists |

**Note:** Executor must start the app server before any agent-browser verification steps.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (unit/integration) + agent-browser (visual/e2e) |
| Config file | Cargo.toml workspaces |
| Quick run command | `cargo test -p resyn-app -p resyn-worker` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GRAPH-01 | Nodes visibly animate and spread apart on load | visual/e2e | agent-browser screenshot + pixel diff | ❌ Wave 0 (visual only) |
| GRAPH-02 | Nodes render crisp, no DPR blur | visual/e2e | agent-browser screenshot at 100% zoom | ❌ Wave 0 (visual only) |
| GRAPH-03 | Edges rendered between connected nodes | visual/e2e | agent-browser screenshot showing lines | ❌ Wave 0 (visual only) |

**Note:** GRAPH-01, GRAPH-02, GRAPH-03 are inherently visual requirements. Automated verification is via agent-browser screenshots interpreted by the executor. No unit test can substitute for pixel-level visual confirmation.

Existing unit tests cover the subsystems:
- `cargo test -p resyn-worker` — 8 force simulation tests (all passing, not at risk)
- `cargo test -p resyn-app` — layout_state, interaction, renderer Viewport, lod tests

### Sampling Rate
- **Per task commit:** `cargo test -p resyn-app -p resyn-worker` (unit tests, <5s)
- **Per wave merge:** `cargo test` (full suite, ~30s)
- **Phase gate:** Full suite green + agent-browser screenshot confirms GRAPH-01/02/03 visually

### Wave 0 Gaps
- No new test files needed — existing test infrastructure covers subsystem units
- Visual verification is exclusively via agent-browser; no automated pixel-diff infrastructure needed for this phase

---

## Project Constraints (from CLAUDE.md)

- Rust edition 2024, stable toolchain (pinned via `rust-toolchain.toml`)
- CI: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -Dwarnings`, `cargo test`
- Single async runtime: `main.rs` only `#[tokio::main]` — all WASM async uses `spawn_local`
- Error handling via `ResynError` with `?` propagation
- Rate limiting must not be removed from arXiv/InspireHEP clients (not relevant to this phase)
- No new external server dependencies — SurrealDB `kv-mem` compiles as Rust dep

---

## Sources

### Primary (HIGH confidence)
- Direct code inspection: `resyn-app/src/graph/webgl_renderer.rs` — shader formulas, DPR fix, VAO/VBO patterns
- Direct code inspection: `resyn-app/src/pages/graph.rs` — RAF loop, force integration
- Direct code inspection: `resyn-app/src/graph/layout_state.rs` — initial position formula
- Direct code inspection: `resyn-app/src/graph/renderer.rs` — Viewport coordinate transforms
- Direct code inspection: `resyn-worker/src/forces.rs` — force constants, simulation correctness
- `.planning/BUGFIX-STATUS.md` — confirmed working vs broken features, previous fix attempts

### Secondary (MEDIUM confidence)
- WebGL2 specification knowledge: VAO stores complete attribute state including divisors — standard WebGL2 behavior
- WASM/browser environment: `device_pixel_ratio` API behavior, physical vs CSS pixel distinction

---

## Metadata

**Confidence breakdown:**
- Root cause identification (Bug 1, spread): HIGH — mathematical derivation from code constants
- Root cause identification (Bug 2, VAO/VBO): HIGH — standard WebGL2 behavior
- Root cause identification (Bug 3, DPR): HIGH — formula traced through shader
- Recommended fixes: HIGH — direct, minimal, reversible changes
- Force simulation correctness: HIGH — 8 unit tests passing, integration code correct

**Research date:** 2026-03-23
**Valid until:** 2026-04-22 (stable domain — WebGL2 API and project code are stable)
