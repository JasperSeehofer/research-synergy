# Phase 17: Viewport Fit and Label Collision - Research

**Researched:** 2026-03-25
**Domain:** Leptos/WASM graph UI — viewport animation, label collision avoidance, Canvas 2D rendering
**Confidence:** HIGH

## Summary

Phase 17 has no external library dependencies — everything is implemented in existing Rust/WASM/Canvas 2D code. The `Viewport` struct, `check_alpha_convergence()`, and the RAF loop integration points are already in place. The work is purely additive: new state fields on `RenderState` and `GraphState`, a new label collision module, and updates to `GraphControls`.

The bounding-box fit math is straightforward: iterate visible nodes, collect min/max world coordinates, derive the scale that maps (bb_width + 20% margin) into the CSS canvas dimensions, then lerp `viewport.scale` and `viewport.offset_x/y` toward target values each RAF frame. The label collision pass is a greedy sweep-line: sort candidates by priority, place labels one at a time, skip any whose bounding box overlaps an already-placed label.

**Primary recommendation:** All implementation lives in existing files. No new crates needed. The lerp animation fits naturally into the existing `start_render_loop` closure, and the label pass replaces the naive loop at `canvas_renderer.rs:288-308`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Auto-Fit Behavior**
- D-01: Smooth animated pan-zoom transition (~0.5s lerp of scale + offset) after convergence. Not an instant snap.
- D-02: 10% viewport margin padding on each side when computing the fit bounding box.
- D-03: Bounding box computed from visible nodes only (LOD-visible AND temporal-visible). Filtered-out nodes are excluded.
- D-04: Auto-fit triggers only after initial convergence (alpha < ALPHA_MIN), not on graph load. User watches the spreading animation at default viewport, then the camera smoothly frames the result.

**User Override Latch**
- D-05: Any manual pan or zoom interaction permanently sets a `user_has_interacted` flag. Auto-fit never fires again automatically once set.
- D-06: A "Fit" button in GraphControls allows the user to manually re-trigger the same smooth fit animation at any time.
- D-07: Drag reheat (which restarts force ticks temporarily) does NOT re-trigger auto-fit and does NOT reset the latch. Drag is a deliberate spatial adjustment.
- D-08: Fit button placed in the same control group as zoom +/- buttons. Uses an expand arrows icon (Unicode).

**Label Collision Avoidance**
- D-09: Priority order: seed paper first, then descending citation count. Matches LABEL-01 requirement.
- D-10: Sparse label placement with generous padding between bounding boxes. Clean look over maximum density.
- D-11: Hovering over any node always reveals its label, even if culled by collision avoidance.
- D-12: Label collision layout is cached and only recomputed on viewport changes (zoom, pan, fit). Not recomputed every frame. measureText results cached at graph load time per STATE.md requirement.

**Label Appearance**
- D-13: All labels have uniform style regardless of priority tier. Priority only affects which labels survive collision culling.
- D-14: Labels rendered as modern pill/badge style: opaque background with thin border. Not raw floating text.
- D-15: Label pill colors: background rgba(13,17,23,0.85) (semi-transparent matching graph bg), border #30363d (subtle GitHub-style), text #cccccc. Clean, modern, doesn't compete with nodes.
- D-16: Font remains 11px monospace (consistent with current Canvas 2D labels).

**Convergence Indicator**
- D-17: Text status badge in GraphControls showing three states: "Simulating..." (while running), "Paused" (user-paused), "Settled" (naturally converged).
- D-18: Badge distinguishes user-paused from naturally converged so the user knows whether they can resume.

### Claude's Discretion
- Lerp easing function for the animated fit transition (linear, ease-out, etc.)
- Exact collision bounding box padding multiplier for "generous spacing"
- measureText cache invalidation strategy (font change, node data change, etc.)
- Hover label z-ordering and animation (fade in vs instant)
- Label pill corner radius and internal padding
- Status badge CSS styling (color, position within controls group)
- Whether collision recomputation also triggers on simulation tick during convergence animation or only after full stop
- WebGL2 label rendering path (Canvas 2D overlay text for both renderers, or separate approach)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| VIEW-01 | Graph auto-fits into viewport after force layout stabilizes | `check_alpha_convergence()` at line 404 graph.rs is the exact hook point; fit math derived from Viewport struct fields |
| VIEW-02 | Auto-fit does not re-trigger after user manually pans or zooms | Pan/zoom handlers in `attach_event_listeners` (mousemove Panning branch, wheel) are the two injection points for `user_has_interacted` flag |
| LABEL-01 | Labels rendered with priority-ordered collision avoidance (seed first, then by citation count) | `NodeState.is_seed` and `NodeState.citation_count` already present; greedy label placement is the standard O(n²) algorithm for sparse graphs |
| LABEL-02 | Convergence indicator shows stabilization status in graph controls | Three states require distinguishing natural convergence from user-pause; `simulation_running` RwSignal already exists but carries only one bit; a new `SimulationStatus` enum or second `user_paused` signal is required |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Leptos (existing) | — | Reactive signals for fit trigger, convergence status | Already used throughout the app |
| web_sys Canvas 2D (existing) | — | `fill_rect`, `stroke_rect`, `fill_text`, `measure_text`, `round_rect` | Label pill rendering |
| Rust std (existing) | — | `f64` lerp arithmetic, sort, bounding box | No new deps needed |

### No new crates required
This phase adds no new Cargo dependencies. All required functionality is in the existing stack.

## Architecture Patterns

### Recommended Project Structure
No new files required. Changes are in:
```
resyn-app/src/
├── graph/
│   ├── renderer.rs           # Add compute_fit_target() method to Viewport
│   ├── layout_state.rs       # No changes needed
│   ├── lod.rs                # Add compute_visible_bbox() function
│   ├── canvas_renderer.rs    # Replace naive label loop with collision-aware pass
│   └── label_collision.rs    # NEW: LabelCache struct, collision algorithm
├── pages/
│   └── graph.rs              # RenderState: add fit animation state + user_has_interacted
│                             # RAF loop: lerp step, convergence trigger, flag injection
│                             # attach_event_listeners: set user_has_interacted on pan/zoom
└── components/
    └── graph_controls.rs     # Add fit_count signal, simulation_status signal, Fit button, status badge
```

### Pattern 1: Bounding Box Fit Computation
**What:** Compute target viewport scale and offset that centers all visible nodes with 10% margin on each side (D-02, D-03).
**When to use:** On convergence (D-04), on Fit button press (D-06), on viewport change to invalidate label cache (D-12).

**Math (in CSS pixel / world space):**
```
world coords: min_x, max_x, min_y, max_y of visible nodes (lod_visible && temporal_visible)
bb_w = max_x - min_x   (add node radius for tight fit)
bb_h = max_y - min_y
center_wx = (min_x + max_x) / 2.0
center_wy = (min_y + max_y) / 2.0

margin_factor = 0.80  // 10% margin on each side = 80% of canvas used
target_scale = (canvas_w * margin_factor / bb_w).min(canvas_h * margin_factor / bb_h)
target_scale = target_scale.clamp(0.1, 4.0)  // respect existing zoom bounds

// Center the bounding box centroid on screen:
// screen_x = world_x * scale + offset_x  =>  canvas_w/2 = center_wx * target_scale + offset_x
target_offset_x = canvas_w / 2.0 - center_wx * target_scale
target_offset_y = canvas_h / 2.0 - center_wy * target_scale
```

**Edge case:** If zero visible nodes, skip fit computation.

### Pattern 2: Lerp Animation in RAF Loop
**What:** Each frame, move current viewport scale and offset a fraction toward target values (D-01, ~0.5s).
**When to use:** While `fit_animation_active` flag is true.

```rust
// In RenderState, add:
pub struct FitAnimState {
    pub active: bool,
    pub target_scale: f64,
    pub target_offset_x: f64,
    pub target_offset_y: f64,
}

// In RAF loop, after simulation tick:
if s.fit_anim.active {
    let t = 0.12;  // ~0.5s at 60fps with ease-out feel; tune as needed
    s.viewport.scale = lerp(s.viewport.scale, s.fit_anim.target_scale, t);
    s.viewport.offset_x = lerp(s.viewport.offset_x, s.fit_anim.target_offset_x, t);
    s.viewport.offset_y = lerp(s.viewport.offset_y, s.fit_anim.target_offset_y, t);
    // Stop when close enough
    let close = (s.viewport.scale - s.fit_anim.target_scale).abs() < 0.001
        && (s.viewport.offset_x - s.fit_anim.target_offset_x).abs() < 0.5
        && (s.viewport.offset_y - s.fit_anim.target_offset_y).abs() < 0.5;
    if close {
        s.viewport.scale = s.fit_anim.target_scale;
        s.viewport.offset_x = s.fit_anim.target_offset_x;
        s.viewport.offset_y = s.fit_anim.target_offset_y;
        s.fit_anim.active = false;
        // After animation ends, invalidate label cache (viewport changed)
        s.label_cache_dirty = true;
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 { a + (b - a) * t }
```

**Easing:** Per-frame lerp produces exponential decay (ease-out feel) naturally. No separate easing function needed.

### Pattern 3: User Interaction Latch (D-05, D-07)
**What:** A `user_has_interacted: bool` flag in `RenderState`. Set to `true` on pan or zoom. Auto-fit checks this flag before triggering.

**Injection points in `attach_event_listeners`:**

```rust
// In mousemove handler, Panning branch:
// (already sets viewport.offset_x/y — add before the offset update)
s.user_has_interacted = true;

// In wheel handler (zoom):
// (already calls zoom_toward_cursor — add before the zoom call)
s.user_has_interacted = true;

// In zoom button handler in RAF loop (zoom_in_count/zoom_out_count changes):
// (already calls zoom_toward_cursor — add before the zoom call)
s.user_has_interacted = true;

// Drag reheat path (mouseup DraggingNode, real drag): do NOT set flag (D-07)
```

**Fit button resets nothing:** The Fit button triggers a new fit animation but does not clear `user_has_interacted`. It is a manual re-trigger, not a state reset.

**Fit trigger in RAF loop:**
```rust
// After check_alpha_convergence() returns true for the first time:
if s.graph.check_alpha_convergence() {
    simulation_running.set(false);
    if !s.user_has_interacted {
        // Compute target and start fit animation
        s.fit_anim = compute_fit_target(&s.graph.nodes, &s.viewport);
    }
}
```

### Pattern 4: Label Collision Cache (D-12)
**What:** A `LabelCache` that holds the sorted priority list and the set of visible label indices. Only recomputed when `label_cache_dirty` is true.

**Dirty conditions:**
- Viewport scale changes (zoom, pan completes, fit animation step)
- Viewport offset changes
- LOD/temporal visibility changes (year slider move)
- After graph load

**Simplest implementation:** Mark dirty at the start of every frame where the viewport changed from the previous frame. Compare `(last_scale, last_offset_x, last_offset_y)` with current.

```rust
pub struct LabelCache {
    pub visible_label_indices: Vec<usize>,   // indices into nodes, in draw order
    pub text_widths: Vec<f64>,               // cached measureText per node, indexed by node idx
}
```

**measureText cache:** Built once at graph load time (or when nodes change). Maps node index to text width. Never recomputed per-frame.

```rust
// Build at graph load:
// (In Canvas2DRenderer::new or on first draw):
ctx.set_font("11px monospace");
let widths: Vec<f64> = state.nodes.iter()
    .map(|n| ctx.measure_text(&n.label()).unwrap().width())
    .collect();
```

### Pattern 5: Greedy Label Collision Avoidance (D-09, D-10)
**What:** Sort nodes by priority (seed first, then desc citation count). Iterate in order; place each label if its bounding box (label pill rect) does not overlap any already-placed label rect.

```rust
// Priority sort (done once per cache rebuild, not per frame):
let mut priority_indices: Vec<usize> = (0..nodes.len())
    .filter(|&i| nodes[i].lod_visible && nodes[i].temporal_visible)
    .collect();
priority_indices.sort_by(|&a, &b| {
    let na = &nodes[a];
    let nb = &nodes[b];
    // Seed always first
    if na.is_seed { return std::cmp::Ordering::Less; }
    if nb.is_seed { return std::cmp::Ordering::Greater; }
    nb.citation_count.cmp(&na.citation_count)
});

// Greedy placement:
let padding = 8.0;  // generous padding per D-10 (8px between label boxes)
let pill_h = 18.0;  // approximate pill height (11px font + vertical padding)
let mut placed: Vec<[f64; 4]> = Vec::new();  // [x, y, w, h] of placed labels in screen space

let mut visible_labels: Vec<usize> = Vec::new();
for &i in &priority_indices {
    let node = &nodes[i];
    let (sx, sy) = viewport.world_to_screen(node.x, node.y);
    let text_w = text_widths[i];
    let pill_w = text_w + 12.0;  // 6px horizontal padding on each side
    let label_x = sx - pill_w / 2.0;
    let label_y = sy + node.radius * viewport.scale + 6.0;  // below node in screen space

    // Check overlap with all placed labels
    let rect = [label_x - padding, label_y - padding, pill_w + padding * 2.0, pill_h + padding * 2.0];
    let overlaps = placed.iter().any(|p| {
        rect[0] < p[0] + p[2] && rect[0] + rect[2] > p[0]
        && rect[1] < p[1] + p[3] && rect[1] + rect[3] > p[1]
    });
    if !overlaps {
        placed.push([label_x, label_y, pill_w, pill_h]);
        visible_labels.push(i);
    }
}
```

**Note:** Label positions are in screen space, so the collision test is scale-aware automatically.

### Pattern 6: Label Pill Drawing (D-14, D-15, D-16)
**What:** Draw rounded rectangle background with border, then text.

```rust
// For each visible label index:
// IMPORTANT: Labels must be drawn in screen space (reset transform first)
ctx.save();
ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();

let dpr = window.device_pixel_ratio();
// DPR scale for physical pixels:
ctx.scale(dpr, dpr).unwrap();

ctx.set_font("11px monospace");
for &i in &label_cache.visible_label_indices {
    let (label_x, label_y, pill_w, pill_h) = /* from cache */;
    // Background pill
    ctx.set_fill_style_str("rgba(13,17,23,0.85)");
    ctx.begin_path();
    ctx.round_rect_with_f64(label_x, label_y, pill_w, pill_h, 4.0).unwrap();
    ctx.fill();
    // Border
    ctx.set_stroke_style_str("#30363d");
    ctx.set_line_width(1.0);
    ctx.stroke();
    // Text
    ctx.set_fill_style_str("#cccccc");
    let node = &nodes[i];
    ctx.fill_text(&node.label(), label_x + 6.0, label_y + 13.0).unwrap();
}

ctx.restore();
```

**CRITICAL:** Labels must be drawn AFTER resetting the viewport transform (after `ctx.restore()` from the world-space drawing pass). The canvas is in world space during node/edge rendering; labels go in screen space. The existing code at lines 288-308 draws labels while still in world-transform — this is a bug to fix in this phase.

**`round_rect`:** Available in `web_sys::CanvasRenderingContext2d` via the Canvas 2D Level 2 API. Available in all modern browsers (Chrome 99+, Firefox 112+, Safari 15.4+). Confidence: HIGH (standard API, 2022+).

### Pattern 7: Hover Label Override (D-11)
**What:** When `state.hovered_node == Some(i)` and `i` is NOT in `visible_label_indices`, draw the label for that node anyway — after the regular label pass. Uses the same pill style.

```rust
if let Some(hi) = state.hovered_node {
    if !label_cache.visible_label_indices.contains(&hi) {
        // Draw hover label (same pill style, computed fresh from current viewport)
    }
}
```

### Pattern 8: Convergence Status Signal (D-17, D-18)
**What:** Three states require two signals or one enum. The existing `simulation_running: RwSignal<bool>` cannot distinguish "paused by user" from "naturally settled". Add a second signal `simulation_settled: RwSignal<bool>` — set to `true` only when `check_alpha_convergence()` returns true for the first time.

```rust
// New signal in GraphPage:
let simulation_settled: RwSignal<bool> = RwSignal::new(false);

// In RAF loop on convergence:
if s.graph.check_alpha_convergence() {
    simulation_running.set(false);
    simulation_settled.set(true);  // permanently marks natural convergence
}

// Status badge logic in GraphControls:
// simulation_running=true  AND settled=false  => "Simulating..."
// simulation_running=false AND settled=false  => "Paused"
// settled=true (regardless of running)        => "Settled"
```

**Note:** After settling, user can still click play to reheat — `simulation_settled` stays true but `simulation_running` becomes true again. The badge should show "Settled" (has converged at least once) even while reheating from a drag. This is cleaner than resetting settled on reheat.

### Pattern 9: Fit Button Signal (D-06, D-08)
**What:** Same counter-increment pattern as `zoom_in_count`/`zoom_out_count`. A `fit_count: RwSignal<u32>` incremented by the Fit button; RAF loop checks for changes.

```rust
// In GraphControls:
let fit_count: RwSignal<u32>;
// ... button:
on:click=move |_| fit_count.update(|v| *v = v.wrapping_add(1))
// Unicode expand arrows: "\u{26F6}" or "\u{2922}" or "⛶" — use "⤢" (\u{2922})

// In RAF loop:
let fi = fit_count.get_untracked();
let pfi = *prev_fit.borrow();
if fi != pfi {
    *prev_fit.borrow_mut() = fi;
    // Compute fit target and start animation (ignores user_has_interacted flag)
    s.fit_anim = compute_fit_target(&s.graph.nodes, &s.viewport);
}
```

### Anti-Patterns to Avoid
- **Recomputing measureText every frame:** The existing `canvas_renderer.rs` lines 298-299 call `measure_text` in the draw loop. This phase must cache these at graph load time (STATE.md records this as a required fix, not optional optimization).
- **Drawing labels in world-transform space:** Current code draws labels while canvas transform is still set to the viewport world transform. This causes labels to scale with zoom. Labels must be drawn in screen space (after resetting transform with `ctx.set_transform(1,0,0,1,0,0)`).
- **Setting `user_has_interacted` on drag-reheat:** D-07 explicitly prohibits this. Only pan and zoom set the flag.
- **Checking label collision in world space:** Node positions are world-space; label pill sizes are screen-space (text width in pixels). Collision must happen in screen space using `world_to_screen()` converted coordinates.
- **Resetting `simulation_settled` on drag reheat:** Once settled, the status should remain "Settled". Only a full graph reload should reset it.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Smooth animation easing | Custom easing math | Per-frame lerp (exponential decay) | Already produces ease-out feel; zero additional code |
| Label text width | Custom font metrics | `ctx.measure_text()` (cached at load) | Browser has exact metrics; hand-rolling is wrong |
| Rounded rectangle | Manual arc path | `ctx.round_rect_with_f64()` | Standard Canvas 2D Level 2 API; no arc math needed |
| Complex collision tree | R-tree, spatial index | Simple O(n) `Vec<[f64;4]>` scan | Sparse labels (N < 50 typically); O(n) is fast enough at this density |

**Key insight:** For sparse citation graphs with priority culling, most nodes get no label. The final placed set is typically 5-30 labels. O(n²) collision is trivially fast.

## Common Pitfalls

### Pitfall 1: Canvas Transform State at Label Draw Time
**What goes wrong:** If labels are drawn while the world-transform (`viewport.apply()`) is still active, label text and pill backgrounds scale with zoom. At `scale=0.3`, text is tiny; at `scale=3.0`, text is enormous.
**Why it happens:** `viewport.apply()` sets a scale transform on the context. Text drawn afterward inherits it.
**How to avoid:** Call `ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)` before the label pass, then apply DPR-scale separately. Or wrap world drawing in `ctx.save()`/`ctx.restore()` and draw labels after `restore()`.
**Warning signs:** Labels grow/shrink when user zooms.

### Pitfall 2: Label Positions Computed in Wrong Space
**What goes wrong:** Computing label screen positions from world coords and then caching them — but the cache becomes stale after the fit animation moves `offset_x/y` each frame.
**Why it happens:** `world_to_screen(x, y)` depends on `viewport.offset_x`, `offset_y`, `scale`. During the lerp animation these change every frame.
**How to avoid:** Do NOT cache screen-space label positions across frames during fit animation. Either: (a) mark cache dirty on every frame during animation, or (b) store world-space node positions in the cache and convert to screen space at draw time using the current viewport.
**Recommendation:** Store only world-space positions in the cache. Convert at draw time. Cache only the sorted priority order and text widths — these don't change with viewport.

### Pitfall 3: user_has_interacted Set Too Eagerly
**What goes wrong:** Setting `user_has_interacted` in the zoom button RAF handler fires even when the Fit button is clicked, because the RAF handler processes zoom button presses unconditionally.
**Why it happens:** The RAF loop checks `zoom_in_count` changes — if a fit animation also modifies the viewport, and then a zoom press happens, the sequence is correct. But the flag must only be set when the user explicitly presses +/- or pan-drags, not when the viewport changes from Fit button.
**How to avoid:** Set `user_has_interacted = true` ONLY in the zoom-in/zoom-out button paths and in the pan mousemove/wheel handlers. The fit animation viewport changes do not set the flag.

### Pitfall 4: Fit Fires on Graph Data Reload
**What goes wrong:** The fit logic triggers on every convergence event, including when graph data is reloaded mid-session.
**Why it happens:** `check_alpha_convergence()` fires whenever `alpha < ALPHA_MIN`, including after a drag reheat that decays.
**How to avoid:** Use a `fit_has_fired_once: bool` flag in `RenderState`. Only auto-fit if `!fit_has_fired_once && !user_has_interacted`. Set `fit_has_fired_once = true` when the auto-fit animation is triggered. The Fit button bypasses this flag entirely.

### Pitfall 5: `round_rect` Not in Older `web_sys` Bindings
**What goes wrong:** `web_sys::CanvasRenderingContext2d::round_rect_with_f64` may require a recent `web_sys` version.
**Why it happens:** `round_rect` was standardized in 2022; older `web_sys` crate versions may not have the binding.
**How to avoid:** Check `Cargo.toml` for the `web-sys` version and verify the binding exists. If not, use a manual 8-arc path as fallback. Given the project's dependency on surrealdb v3 and recent Leptos, web-sys is likely recent enough.
**Verification:** Run `cargo doc --open` and search `CanvasRenderingContext2d` for `round_rect`.

### Pitfall 6: Label Cache Invalidation During Fit Animation
**What goes wrong:** Label cache rebuilt once per frame during the lerp animation (because viewport changes each frame), causing visible jitter as labels re-sort or change positions.
**Why it happens:** Greedy sort order depends on screen-space positions; if the viewport moves every frame, the "which labels fit" decision changes every frame.
**How to avoid:** During fit animation, skip label collision recomputation — draw no labels (or draw only the hovered label). Resume label rendering once `fit_anim.active = false`.

## Code Examples

### compute_fit_target function
```rust
// Source: derived from Viewport struct in renderer.rs
pub fn compute_fit_target(
    nodes: &[NodeState],
    viewport: &Viewport,
) -> Option<FitAnimState> {
    let visible: Vec<&NodeState> = nodes.iter()
        .filter(|n| n.lod_visible && n.temporal_visible)
        .collect();
    if visible.is_empty() { return None; }

    let margin_factor = 0.80_f64;  // 10% margin on each side

    let min_x = visible.iter().map(|n| n.x - n.radius).fold(f64::INFINITY, f64::min);
    let max_x = visible.iter().map(|n| n.x + n.radius).fold(f64::NEG_INFINITY, f64::max);
    let min_y = visible.iter().map(|n| n.y - n.radius).fold(f64::INFINITY, f64::min);
    let max_y = visible.iter().map(|n| n.y + n.radius).fold(f64::NEG_INFINITY, f64::max);

    let bb_w = (max_x - min_x).max(1.0);
    let bb_h = (max_y - min_y).max(1.0);
    let cx_w = (min_x + max_x) / 2.0;
    let cy_w = (min_y + max_y) / 2.0;

    let target_scale = ((viewport.css_width * margin_factor / bb_w)
        .min(viewport.css_height * margin_factor / bb_h))
        .clamp(0.1, 4.0);

    let target_offset_x = viewport.css_width / 2.0 - cx_w * target_scale;
    let target_offset_y = viewport.css_height / 2.0 - cy_w * target_scale;

    Some(FitAnimState {
        active: true,
        target_scale,
        target_offset_x,
        target_offset_y,
    })
}
```

### Label module struct
```rust
// label_collision.rs
pub struct LabelCache {
    /// Node indices in priority order that passed collision culling, indexed by position in draw order.
    pub visible_indices: Vec<usize>,
    /// Cached text widths per node index (indexed by node position in nodes slice).
    pub text_widths: Vec<f64>,
    /// Pill screen-x per visible_indices entry (world->screen at time of last rebuild).
    pub pill_x: Vec<f64>,
    /// Pill screen-y per visible_indices entry.
    pub pill_y: Vec<f64>,
    /// Pill width per visible_indices entry.
    pub pill_w: Vec<f64>,
}
```

### GraphControls additions
```rust
// New props:
fit_count: RwSignal<u32>,
simulation_settled: RwSignal<bool>,

// Fit button (in same group as zoom +/-):
<button
    class="graph-control-btn"
    on:click=move |_| fit_count.update(|v| *v = v.wrapping_add(1))
    aria-label="Fit graph to viewport"
>
    "\u{2922}"   // ⤢ expand arrows
</button>

// Status badge (in simulation control group or node count group):
<span class=move || {
    if simulation_running.get() { "sim-status-badge sim-running" }
    else if simulation_settled.get() { "sim-status-badge sim-settled" }
    else { "sim-status-badge sim-paused" }
}>
    {move || {
        if simulation_running.get() { "Simulating..." }
        else if simulation_settled.get() { "Settled" }
        else { "Paused" }
    }}
</span>
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Raw floating text (current) | Pill/badge with background | Phase 17 | Labels become legible on dense graphs |
| measureText per frame | measureText cached at load | Phase 17 | Eliminates per-frame JS bridge calls |
| All visible labels drawn | Priority-ordered collision culling | Phase 17 | Clean sparse labels rather than overlapping text |
| No auto-fit | Lerp-animated fit on convergence | Phase 17 | Graph always visible after layout |

**Deprecated/outdated:**
- Lines 288-308 of `canvas_renderer.rs` (naive label loop): replaced entirely by the label collision pass
- World-space label drawing in `viewport.apply()` transform: replaced by screen-space drawing after transform reset

## Open Questions

1. **`round_rect` binding availability in current web-sys version**
   - What we know: `round_rect` was standardized in 2022; modern browsers support it
   - What's unclear: Whether the specific `web-sys` version in `Cargo.toml` includes `round_rect_with_f64`
   - Recommendation: Verify during Wave 0 with `cargo doc`. If absent, implement as 8-arc path (4 lines + 4 quarter-circle arcs). This is a known 10-line fallback.

2. **Label position during fit animation**
   - What we know: D-12 says cache on viewport change; fit animation changes viewport every frame
   - What's unclear: Whether to suppress labels during animation or allow stale cache
   - Recommendation: Suppress label rendering during `fit_anim.active == true`. Recompute once when animation ends. This eliminates the pitfall entirely.

3. **Unicode symbol for Fit button**
   - What we know: D-08 says "expand arrows icon (Unicode)"
   - What's unclear: Which exact character renders well in monospace/system font in the graph controls
   - Recommendation: Use `⤢` (U+2922, NORTH EAST AND SOUTH WEST ARROW). Fallback: `⊞` (U+229E) or ASCII "[ ]". Test in browser.

## Environment Availability

Step 2.6: SKIPPED (no external dependencies — all changes are in existing Rust/WASM code with no new tools, services, CLIs, or runtimes required)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` (no test runner config file) |
| Config file | none (standard `cargo test`) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test -- --nocapture` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| VIEW-01 | `compute_fit_target` returns correct scale/offset for known node positions | unit | `cargo test test_compute_fit_target` | ❌ Wave 0 |
| VIEW-01 | Fit target has 10% margin (margin_factor=0.80) | unit | `cargo test test_fit_target_margin` | ❌ Wave 0 |
| VIEW-01 | Fit target returns None for empty visible nodes | unit | `cargo test test_fit_target_empty` | ❌ Wave 0 |
| VIEW-02 | `user_has_interacted` flag prevents auto-fit re-trigger | unit | `cargo test test_fit_latch` | ❌ Wave 0 |
| LABEL-01 | Priority sort: seed node always first | unit | `cargo test test_label_priority_seed_first` | ❌ Wave 0 |
| LABEL-01 | Priority sort: descending citation count after seed | unit | `cargo test test_label_priority_by_citation` | ❌ Wave 0 |
| LABEL-01 | Collision: overlapping label skipped | unit | `cargo test test_label_collision_skip` | ❌ Wave 0 |
| LABEL-01 | Collision: non-overlapping label placed | unit | `cargo test test_label_collision_place` | ❌ Wave 0 |
| LABEL-02 | `simulation_settled` set to true on alpha convergence | unit | `cargo test test_simulation_settled_on_convergence` (extend existing `test_alpha_stops_simulation`) | ✅ partially |

**Note on LABEL-01/02 tests:** `compute_fit_target` and the label collision priority/placement logic live in pure Rust (no WASM, no DOM). They are fully unit-testable. The Canvas 2D drawing steps are not unit-testable in `cargo test` — those require browser UAT.

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test -- --nocapture`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `resyn-app/src/graph/label_collision.rs` — test module for `compute_visible_bbox`, `compute_fit_target`, label priority sort, collision placement
- [ ] `resyn-app/src/pages/graph.rs` — extend existing convergence tests to cover `simulation_settled` signal logic (unit-testable via extracted helper)

*(The existing 44-test suite already covers: `check_alpha_convergence`, `NodeState.is_seed`, `lod_visible`, `temporal_visible`, `world_to_screen`, `screen_to_world` — no gaps in foundational logic)*

## Sources

### Primary (HIGH confidence)
- Direct source code read: `resyn-app/src/graph/renderer.rs` — Viewport struct, coordinate math
- Direct source code read: `resyn-app/src/graph/layout_state.rs` — NodeState fields, GraphState, check_alpha_convergence
- Direct source code read: `resyn-app/src/graph/lod.rs` — visibility predicates, compute_visible_count pattern
- Direct source code read: `resyn-app/src/pages/graph.rs` — RAF loop, interaction handlers, signal patterns
- Direct source code read: `resyn-app/src/components/graph_controls.rs` — existing component structure
- Direct source code read: `resyn-app/src/graph/canvas_renderer.rs` — current label drawing at lines 288-308
- Direct source code read: `resyn-app/src/graph/interaction.rs` — zoom_toward_cursor, InteractionState

### Secondary (MEDIUM confidence)
- Canvas 2D `round_rect` — documented in MDN as standard since Chrome 99 / Firefox 112 / Safari 15.4; web-sys binding name inferred from standard naming convention (`round_rect_with_f64`). Verify with `cargo doc` before use.
- Greedy label collision (sweep-line skip) — standard algorithm for sparse graph label placement; O(n) scan is appropriate for expected label count (< 50 placed labels).

### Tertiary (LOW confidence)
- Unicode character U+2922 (⤢) for Fit button: rendering quality depends on system font; test in browser.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all implementation uses existing code
- Architecture: HIGH — all integration points verified by direct source code read
- Pitfalls: HIGH — canvas transform pitfall (#1) verified by reading current label code in world-space; others derived from direct code analysis
- Test coverage: HIGH — existing test patterns well-established; gaps are new files only

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable codebase, no external dependencies)
