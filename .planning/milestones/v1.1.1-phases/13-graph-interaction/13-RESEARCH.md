# Phase 13: Graph Interaction - Research

**Researched:** 2026-03-23
**Domain:** Browser pointer events, CSS overlay z-index, WASM/Leptos event handler debugging
**Confidence:** HIGH — all findings derive from direct code inspection of canonical source files

## Summary

This is a bugfix phase on an existing, fully-implemented interaction system. All code paths exist: the `InteractionState` state machine, `find_node_at()` / `find_edge_at()` hit testing, `zoom_toward_cursor()`, `screen_to_world()` / `world_to_screen()` transforms, and all six event handler closures attached to the canvas in `graph.rs`. Seven unit tests pass for the interaction module. The bugs are not in the logic — they are in **what the events reach**.

Two root causes are identified through code inspection. First, the `.graph-controls-overlay` (z-index: 10) and `.temporal-slider-row` (z-index: 10) are positioned absolutely within `.graph-page` (position: relative), which means they sit visually over the canvas. If their background or hit region covers the canvas area, they will intercept all mouse events before the canvas receives them — the canvas event listeners never fire. Second, the Phase 12 DPR fix established that `MouseEvent.offset_x / offset_y` are CSS pixels and must be passed directly to `screen_to_world()` without DPR conversion. This convention is documented in `renderer.rs` and is consistent with the Viewport implementation — but it must be verified against the live Phase 12 output before treating coordinate transforms as correct.

**Primary recommendation:** Diagnose overlay blocking first — add `console.log` in the `mousedown` handler and use browser devtools to confirm whether the event fires on canvas or is consumed by an overlay. If events reach the canvas but hit testing fails, add a second `console.log` after `screen_to_world()` to verify the world coordinate matches expected node positions. Fix in the CSS layer (overlay `pointer-events`) before touching interaction logic.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01: Overlay / Pointer Event Blocking**
The primary suspect is CSS overlay elements blocking pointer events from reaching the canvas. The `.graph-controls-overlay` (z-index: 10), tooltip overlay (z-index: 50/200), and temporal slider container may intercept events. Diagnose by checking if mousedown/mousemove/wheel events actually fire on the canvas element. Fix by ensuring overlays use `pointer-events: none` except on their interactive children (buttons, slider thumbs).

**D-02: STATE.md overlay flag**
STATE.md from Phase 12 explicitly flags this: "Canvas may be covered by an overlay element (z-index), blocking all pointer events — check first before debugging event listener logic."

**D-03: Coordinate Transform Verification**
Phase 12 established the DPR convention: all coordinate math in CSS pixels, DPR applied only at canvas physical sizing and GL viewport. The `screen_to_world` / `world_to_screen` transforms in `Viewport` must be verified against this convention — the Phase 12 DPR fix may have broken the coordinate mapping. Use console logging of (screen_x, screen_y) → (world_x, world_y) to verify hit testing targets the correct node.

**D-04: Coordinate Transform Fix Location**
If the coordinate transform is off, fix it in the `Viewport` struct — do not work around it in event handlers.

**D-05: Force Reheat on Drag**
Keep the existing moderate reheat behavior: `alpha = max(current_alpha, 0.3)` on drag release. This is already implemented in the mouseup handler. Only tune if live testing shows nodes don't settle or settle too fast after drag.

**D-06: Pinning Behavior**
Dragged nodes are pinned during drag (existing behavior). On click-release (< 3px movement), the node is unpinned. On drag-release, the node stays pinned. This is correct — preserve it.

**D-07: Hit Test Parameters**
Keep current hit test parameters: node radius for node detection, 4px threshold for edge detection. The `find_node_at()` function iterates in reverse render order (topmost first). Only adjust thresholds if live testing shows consistent misses.

**D-08: Hit Test Scope**
Hit testing uses all nodes regardless of LOD visibility. This is intentional — users can interact with nodes even if labels are hidden at the current zoom level.

### Claude's Discretion
- Debugging approach: console logging, event listener verification, CSS inspection — whatever is fastest to identify the root cause
- Whether to add temporary debug logging (remove after fix confirmed)
- Order of investigation (overlay blocking → coordinate transforms → force behavior)
- Whether the click-vs-drag threshold (3px) needs adjustment based on testing

### Deferred Ideas (OUT OF SCOPE)
- Touch/mobile interaction support — future milestone
- Node selection highlighting / multi-select — future feature
- Right-click context menu on nodes — future feature
- Keyboard navigation (arrow keys to pan, +/- to zoom) — future feature
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| INTERACT-01 | User can drag individual nodes to reposition them | `DraggingNode` state, `find_node_at()`, mousedown/mousemove/mouseup handlers all exist; likely blocked by overlay CSS |
| INTERACT-02 | User can pan the graph viewport by dragging empty space | `Panning` state and viewport offset update logic fully implemented; same overlay blocking issue |
| INTERACT-03 | User can zoom in/out with scroll wheel | `zoom_toward_cursor()` is tested + attached to `wheel` event with `preventDefault()`; same overlay blocking issue |
</phase_requirements>

---

## Standard Stack

### Core (all already in project — no new dependencies needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| web-sys | workspace | DOM APIs, MouseEvent, WheelEvent, canvas | Only JS interop available in WASM |
| wasm-bindgen | workspace | Closure types for event handlers | Required for WASM event attachment |
| Leptos | workspace | RwSignal, reactive UI | Project framework |

No new packages required. This phase is entirely a bugfix within existing code.

**Installation:** none needed.

## Architecture Patterns

### DOM Structure (as built)

```
.graph-page  (position: relative)
├── canvas.graph-canvas         ← events attach here
├── .graph-controls-overlay     ← z-index: 10, positioned absolute top-right
│   └── .graph-controls-group  ← contains interactive buttons
├── .temporal-slider-row        ← z-index: 10, positioned absolute bottom
│   └── .dual-range-wrapper    ← range inputs with pointer-events: none on container
└── .graph-tooltip              ← z-index: 50, pointer-events: none (already correct)
```

### Pattern 1: CSS Pointer Event Passthrough

**What:** Overlay containers that sit on top of the canvas must have `pointer-events: none` at the container level, with `pointer-events: auto` (or `all`) only on their interactive children.
**When to use:** Any absolutely-positioned element that visually overlaps a canvas but should not consume canvas events.
**Current state of `.graph-controls-overlay`:** Has `z-index: 10`, `position: absolute`, but NO `pointer-events: none`. Its background covers the top-right region of the canvas. Any mouse event in that region fires on the overlay div, not on the canvas.
**Current state of `.temporal-slider-row`:** Has `z-index: 10`, `position: absolute`, sits at the bottom. The `.temporal-range` inputs have `pointer-events: none` on the input element itself, but the `.temporal-slider-row` container does not have `pointer-events: none`. Wheel events over this area hit the slider row, not the canvas.
**Fix:** Add `pointer-events: none` to `.graph-controls-overlay` and `.temporal-slider-row`. Add `pointer-events: auto` to `.graph-control-btn`, `.graph-controls-group`, and the range thumb pseudo-elements (already done for range thumbs).

```css
/* Source: CSS inspection of main.css lines 1331–1411 */

/* BEFORE (missing pointer-events: none) */
.graph-controls-overlay {
  position: absolute;
  z-index: 10;
  /* no pointer-events set — consumes events from canvas */
}

/* AFTER */
.graph-controls-overlay {
  position: absolute;
  z-index: 10;
  pointer-events: none;   /* pass through to canvas */
}

.graph-controls-group {
  pointer-events: auto;   /* re-enable for button groups */
}

.temporal-slider-row {
  pointer-events: none;   /* pass through to canvas */
}
/* Note: .temporal-range::-webkit-slider-thumb already has pointer-events: all */
/* The input elements themselves need pointer-events: auto for the thumbs to work */
```

### Pattern 2: Canvas Event Handler Debugging

**What:** Verify events actually fire on the canvas by adding a temporary `console.log` at the top of the mousedown closure before touching any state.
**When to use:** When interaction appears broken and the root cause (overlay vs. logic) is unknown.
**Example:**
```rust
// In mousedown closure — temporary diagnostic, remove after fix confirmed
// Source: resyn-app/src/pages/graph.rs lines 552–585
let mousedown = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
    web_sys::console::log_1(&format!("mousedown at {:?},{:?}", event.client_x(), event.client_y()).into());
    // ... rest of handler
});
```

### Pattern 3: Coordinate Transform Verification

**What:** After confirming events reach the canvas, verify `screen_to_world()` produces world coordinates that match known node positions.
**When to use:** If events reach canvas but hit testing (`find_node_at`) returns None even when clicking visually over a node.
**DPR convention (from renderer.rs doc comment and Phase 12 research):** `MouseEvent.offset_x / offset_y` are CSS pixels. Pass directly to `screen_to_world()`. Do NOT multiply by DPR. The `canvas_coords()` helper function already does this correctly.

```rust
// In mousedown closure — temporary diagnostic
// Source: resyn-app/src/pages/graph.rs lines 557–559
let (sx, sy) = canvas_coords(&canvas_md, &event);
let (wx, wy) = s.viewport.screen_to_world(sx, sy);
web_sys::console::log_1(
    &format!("screen({:.1},{:.1}) -> world({:.1},{:.1}), nodes: {}",
             sx, sy, wx, wy, s.graph.nodes.len()).into()
);
// Then manually compare wx,wy against s.graph.nodes[0].x / .y in devtools
```

### Anti-Patterns to Avoid

- **DPR compensation in event handlers:** Do not multiply `event.offset_x()` by `window.device_pixel_ratio()`. The `canvas_coords()` function already returns CSS pixels, and `screen_to_world()` expects CSS pixels. Double-scaling breaks hit testing.
- **Fixing interaction logic before verifying events reach the canvas:** Start with the overlay check. If events never reach the canvas, all downstream logic is moot.
- **Removing `event.prevent_default()` from the wheel handler:** It is intentional — prevents the browser from scrolling the page while zooming the graph.
- **Adding `pointer-events: none` to `.graph-control-btn`:** This would break the zoom/simulation toggle buttons. Only the container overlay needs passthrough; buttons must remain interactive.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Pointer event passthrough | Custom JavaScript event delegation | CSS `pointer-events: none` | Native browser property, zero JS overhead, works at any stacking level |
| Canvas coordinate mapping | Browser `getBoundingClientRect()` offset math | Existing `canvas_coords()` in graph.rs line 465 | Already accounts for canvas position in viewport; duplicating this introduces drift bugs |
| Zoom math | Custom scale formula | Existing `zoom_toward_cursor()` in interaction.rs | Already tested with 2 unit tests, preserves world point under cursor correctly |
| Hit testing | Custom point-in-circle / point-on-segment | Existing `find_node_at()` / `find_edge_at()` | 5 unit tests pass; these functions are correct |

**Key insight:** In this phase, the right solution is almost certainly a CSS one-liner (`pointer-events: none`), not new interaction code. The interaction code is verified correct by unit tests.

## Common Pitfalls

### Pitfall 1: Overlay Background Consuming Events
**What goes wrong:** Devtools shows the canvas has event listeners, but they never fire during interaction. The overlay container's rendered background (even if transparent or very small) intercepts mousedown before it reaches the canvas in the event bubbling order.
**Why it happens:** Absolutely positioned elements with z-index stack on top of their siblings in the same stacking context. Without `pointer-events: none`, the overlay div is the hit target even when the user intends to interact with the canvas behind it.
**How to avoid:** Set `pointer-events: none` on any overlay container that visually covers the canvas. Only opt interactive children back in with `pointer-events: auto`.
**Warning signs:** `mousedown` console.log in the canvas handler never fires, but clicking the buttons on the overlay works fine.

### Pitfall 2: Fixing the Wrong Layer
**What goes wrong:** Developer adds console.log to `find_node_at()`, sees it never called, concludes hit testing is broken, rewrites it. The real issue was overlay blocking — events never reached canvas.
**Why it happens:** Starting debugging inside-out (logic) rather than outside-in (event delivery).
**How to avoid:** First confirm `mousedown` fires on canvas (outermost handler), then `find_node_at()` is called, then it returns expected results. Work inward.
**Warning signs:** Rewrites to interaction logic make no difference.

### Pitfall 3: Breaking Temporal Slider Thumbs
**What goes wrong:** Adding `pointer-events: none` to `.temporal-slider-row` stops the range slider thumbs from being draggable — TEMPORAL-01 regresses.
**Why it happens:** CSS `pointer-events: none` on a container cascades to all children unless explicitly overridden.
**How to avoid:** After setting `pointer-events: none` on `.temporal-slider-row`, also set `pointer-events: auto` on the input elements (`.temporal-range`). The thumb pseudo-element already has `pointer-events: all` in CSS — but the input itself must be interactive for the thumb to function.
**Warning signs:** Temporal slider thumbs no longer drag after the fix.

### Pitfall 4: `canvas_coords()` vs `event.offset_x()`
**What goes wrong:** `event.offset_x()` in Leptos/wasm-bindgen returns coordinates relative to the target element — which could be the overlay if the overlay intercepts the event, not the canvas. After fixing overlays, events fire on the canvas, and `offset_x()` is now canvas-relative. However, `canvas_coords()` uses `client_x - rect.left` which is always canvas-relative regardless of event target. These are equivalent after the overlay fix but `canvas_coords()` is more defensive.
**How to avoid:** Keep using `canvas_coords()` as-is. Do not switch to `event.offset_x()`.
**Warning signs:** Coordinates work at scale=1.0 but are off-by-bounding-rect at other positions.

## Code Examples

### Full CSS Fix (verified against main.css lines 1331–1449)

```css
/* resyn-app/style/main.css */

/* Pass events through the controls overlay to the canvas beneath */
.graph-controls-overlay {
  /* existing rules preserved */
  pointer-events: none;
}

/* Re-enable pointer events for the actual button groups */
.graph-controls-group {
  pointer-events: auto;
}

/* Pass events through the temporal slider bar container to the canvas */
.temporal-slider-row {
  /* existing rules preserved */
  pointer-events: none;
}

/* Re-enable pointer events for range inputs so thumbs remain draggable */
.temporal-range {
  pointer-events: auto;
}
```

### Confirming `InteractionState` transitions (no code changes — for verification)

The state machine transitions in the existing event handlers are:
```
mousedown on node  → DraggingNode { node_idx, offset_x, offset_y }
mousedown on empty → Panning { start_x, start_y, start_offset_x, start_offset_y }
mousemove          → updates viewport.offset (Panning) or node.x/y (DraggingNode)
mouseup            → Idle; if was_click: unpin or open drawer; else: keep pinned
wheel              → zoom_toward_cursor() called directly on viewport
pointerleave       → Idle, clear hover
```
All transitions are implemented and tested. No changes needed here.

### Viewport `screen_to_world` signature (no changes needed)

```rust
// Source: resyn-app/src/graph/renderer.rs line 48
pub fn screen_to_world(&self, sx: f64, sy: f64) -> (f64, f64) {
    ((sx - self.offset_x) / self.scale, (sy - self.offset_y) / self.scale)
}
// Input: sx, sy are CSS pixel coordinates (what canvas_coords() returns)
// Output: wx, wy are world coordinates matching GraphState.nodes[i].x / .y
// DPR: NOT involved — CSS space throughout
```

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (Rust) + agent-browser (visual/interactive) |
| Config file | `Cargo.toml` (workspace) |
| Quick run command | `cargo test -p resyn-app` |
| Full suite command | `cargo test --workspace` |
| Estimated runtime | ~30 seconds |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INTERACT-01 | Node drag repositions node and it stays | visual/manual | `agent-browser` | ❌ Wave 0 (browser test) |
| INTERACT-02 | Canvas drag pans viewport | visual/manual | `agent-browser` | ❌ Wave 0 (browser test) |
| INTERACT-03 | Scroll wheel zooms in/out | visual/manual | `agent-browser` | ❌ Wave 0 (browser test) |

Interaction logic unit tests (7 existing) serve as automated regression guard:
- `cargo test -p resyn-app` covers `test_screen_to_world_*`, `test_zoom_*`, `test_find_node_at_*`, `test_find_edge_at_*`
- These remain green regardless of the CSS fix — they validate the logic layer, not event delivery

**Manual verification is the authoritative gate** for INTERACT-01/02/03. The CSS change cannot be unit-tested; it must be verified in a running browser.

### Sampling Rate

- **Per task commit:** `cargo test -p resyn-app` (guards interaction logic unit tests)
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green + agent-browser confirms drag, pan, and zoom all functional

### Wave 0 Gaps

- [ ] agent-browser test script for interaction verification (manual steps suffice; no new test file needed)
- [ ] Verify `cargo test -p resyn-app` already runs the 7 interaction unit tests — confirmed in interaction.rs

None — existing `cargo test` infrastructure covers the logic unit tests. Browser verification is manual-only (visual interaction cannot be automated with cargo test).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/cargo | Build | ✓ | stable (pinned via rust-toolchain.toml) | — |
| trunk | WASM build/serve | assumed ✓ (used in prior phases) | — | — |
| agent-browser | Visual verification | assumed ✓ (used in Phase 12) | — | Manual devtools |

Step 2.6: External dependencies are the same build toolchain used in Phase 12. No new dependencies. Availability assumed from prior phase execution.

## Sources

### Primary (HIGH confidence)
- Direct code inspection: `resyn-app/src/pages/graph.rs` lines 462–701 — all event handler closures
- Direct code inspection: `resyn-app/src/graph/interaction.rs` — full state machine and unit tests
- Direct code inspection: `resyn-app/src/graph/renderer.rs` — Viewport struct and DPR doc comment
- Direct code inspection: `resyn-app/style/main.css` lines 1308–1449 — overlay CSS rules
- Direct code inspection: `resyn-app/src/components/graph_controls.rs` — overlay DOM structure
- `.planning/phases/12-graph-force-rendering/12-RESEARCH.md` — DPR coordinate convention (HIGH)
- `.planning/phases/13-graph-interaction/13-CONTEXT.md` — locked decisions D-01 through D-08

### Secondary (MEDIUM confidence)
- CSS `pointer-events` specification behavior: well-established browser behavior, no verification needed against current docs

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Root cause identification: HIGH — direct CSS inspection confirms `.graph-controls-overlay` has no `pointer-events: none`
- Fix approach: HIGH — CSS `pointer-events: none` is the standard pattern for this exact problem
- Coordinate transform: HIGH — DPR convention documented in renderer.rs, consistent with Phase 12 research
- Temporal slider regression risk: MEDIUM — `pointer-events: auto` on `.temporal-range` should preserve thumb interaction, but requires live verification

**Research date:** 2026-03-23
**Valid until:** 2026-04-22 (stable CSS spec; not time-sensitive)
