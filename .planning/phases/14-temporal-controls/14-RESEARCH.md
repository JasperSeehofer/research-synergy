# Phase 14: Temporal Controls - Research

**Researched:** 2026-03-24
**Domain:** CSS dual-range slider bugfix / Leptos reactive signals / HTML range input interaction
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Both slider thumbs must be visible with explicit cross-browser styling. Verify `::-webkit-slider-thumb` and `::-moz-range-thumb` render in Chrome/Firefox. If thumbs are invisible, the most likely cause is missing `-webkit-appearance: none` on the track or a transparent background on the input element itself.
- **D-02:** The `.temporal-slider-row` container has `pointer-events: none` and `z-index: 10`. The `.temporal-range` inputs have `pointer-events: auto`. Verify this chain actually allows interaction — if the container's `pointer-events: none` prevents child event propagation in some browsers, switch to `pointer-events: auto` on the container (the slider row is at the bottom of the screen, not overlapping the canvas interaction area).
- **D-03:** Two `<input type="range">` elements are stacked absolutely in `.dual-range-wrapper`. When both thumbs are at the same position, the max slider sits on top. Ensure the max slider's thumb has a higher z-index so both remain reachable. If a thumb becomes trapped, add `z-index` differentiation or swap stacking order on hover/focus.
- **D-04:** Each range input covers the full width. The track background must be fully transparent on both inputs so they don't obscure each other. The existing CSS sets `background: transparent` on the inputs and styles only the track pseudo-elements. Verify the track pseudo-elements don't create an opaque clickable area that blocks the thumb beneath.
- **D-05:** Constrain values so min thumb cannot exceed max thumb. Add clamping in `on:input` handlers: `temporal_min.set(val.min(temporal_max.get()))` and `temporal_max.set(val.max(temporal_min.get()))`.
- **D-06:** The RAF loop already syncs `temporal_min`/`temporal_max` signals into `GraphState` and calls `update_temporal_visibility()` each frame. Verify end-to-end: dragging a thumb updates the signal, RAF picks it up, nodes outside the range disappear.
- **D-07:** The year range label already renders reactively from signals. No additional work needed for the label.

### Claude's Discretion

- Whether to add a visual track highlight between the two thumbs (colored bar showing selected range)
- Debug approach for identifying the root cause (browser dev tools, agent-browser, CSS inspection)
- Whether the existing z-index values need adjustment or a complete restructure
- Exact clamping implementation details (in handler vs derived signal)

### Deferred Ideas (OUT OF SCOPE)

- Range track highlight (colored bar between thumbs showing selected range) — visual polish, not required for functionality
- Temporal filtering animation (fade nodes in/out instead of instant show/hide) — future polish
- Preset year ranges (e.g., "Last 5 years", "Last decade") — future feature

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TEMPORAL-01 | Both slider thumbs are visible and draggable independently | D-01 through D-05 address CSS visibility and draggability; D-06 covers filtering integration; all research below covers the exact CSS fixes needed |

</phase_requirements>

---

## Summary

Phase 14 is a CSS bugfix phase for the dual-range year slider. The `TemporalSlider` Leptos component, reactive `RwSignal<u32>` plumbing, RAF loop integration, and `update_temporal_visibility()` filtering logic are all fully implemented and wired. The only broken behavior is that one slider thumb occludes the other or one is unresponsive.

The root causes are known: (1) both `<input type="range">` elements stack absolutely and share the same `pointer-events: auto` surface — the top input captures all pointer events, leaving the bottom input unreachable; (2) `z-index` on pseudo-elements like `::webkit-slider-thumb` does not stack against sibling elements in standard CSS stacking context — it only affects rendering within the input's shadow DOM; and (3) value clamping is absent, so dragging the min thumb past max (or vice versa) produces an inverted range.

The standard solution for dual-range sliders in web UIs is to use `pointer-events: none` on both input tracks and `pointer-events: all` on the thumb pseudo-elements only, combined with DOM ordering: min-input below max-input by DOM order. This is the canonical approach documented in MDN and CSS-Tricks. The current CSS already applies `pointer-events: auto` to the `.temporal-range` elements — the fix is to change this to `pointer-events: none` and add `pointer-events: all` to the thumb pseudo-elements, so only the small circular thumb area captures events.

**Primary recommendation:** Change `.temporal-range` to `pointer-events: none`, add `pointer-events: all` to both `::webkit-slider-thumb` and `::moz-range-thumb`, ensure DOM order places min-input before max-input, and add value-clamping in the `on:input` handlers.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Leptos | existing (project dep) | Reactive signals, component props, `on:input` handlers | Already used throughout — `RwSignal<u32>` for slider state |
| web-sys | existing (project dep) | `HtmlInputElement::value_as_number()` for reading range input value | Already used in existing `on:input` handlers |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| CSS pseudo-elements (`::webkit-slider-thumb`, `::moz-range-thumb`) | N/A | Cross-browser range input thumb styling | Always — no JS/Rust alternative for styling HTML range inputs |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Two stacked native `<input type="range">` | Custom canvas/SVG slider | Native approach requires only CSS fixes, no WASM rewrite needed — strongly prefer fix over rewrite |
| `pointer-events: all` on thumb pseudo-element | JavaScript pointer capture / `setPointerCapture` | CSS-only solution preferred; JS approach adds significant complexity in WASM context |

**Installation:** No new dependencies. This is a CSS-only + minor Rust handler fix.

---

## Architecture Patterns

### Recommended Project Structure

No structural changes. All changes are:

```
resyn-app/style/main.css          # CSS: pointer-events on .temporal-range and pseudo-elements
resyn-app/src/components/graph_controls.rs  # Rust: clamping in on:input handlers
```

### Pattern 1: Dual-Range Slider via Stacked Inputs

**What:** Two `<input type="range">` elements positioned absolutely over the same area. DOM order places min-input first (z-index lower) and max-input second (z-index higher). Both inputs have `pointer-events: none` on the track area so the one on top does not block the one beneath. Only the thumb pseudo-elements have `pointer-events: all`, so only the small circular handle area captures events. Since each thumb has its own input, dragging either thumb fires only that input's event.

**When to use:** Always for dual-range sliders using native HTML inputs — this is the canonical pattern.

**Example (the key CSS):**
```css
/* Source: MDN / CSS-Tricks dual-range slider pattern */

/* Both inputs: no pointer events on the track — thumbs only */
.temporal-range {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 20px;
  -webkit-appearance: none;
  appearance: none;
  background: transparent;
  pointer-events: none;   /* CHANGE: was "auto" — prevents track occlusion */
}

/* WebKit thumb: capture pointer events here only */
.temporal-range::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--color-accent);
  cursor: pointer;
  pointer-events: all;    /* KEEP: thumb is the only click target */
}

/* Moz thumb: same */
.temporal-range::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--color-accent);
  cursor: pointer;
  pointer-events: all;    /* KEEP */
  border: none;
}
```

**Example (clamping in Leptos on:input handler):**
```rust
// Source: D-05 from CONTEXT.md decisions
// Min input handler — clamp to not exceed current max
on:input=move |e| {
    use leptos::wasm_bindgen::JsCast;
    let val = e.target().unwrap()
        .dyn_into::<web_sys::HtmlInputElement>().unwrap()
        .value_as_number() as u32;
    temporal_min.set(val.min(temporal_max.get_untracked()));
}

// Max input handler — clamp to not go below current min
on:input=move |e| {
    use leptos::wasm_bindgen::JsCast;
    let val = e.target().unwrap()
        .dyn_into::<web_sys::HtmlInputElement>().unwrap()
        .value_as_number() as u32;
    temporal_max.set(val.max(temporal_min.get_untracked()));
}
```

### Pattern 2: pointer-events Inheritance Chain

**What:** The Phase 13 overlay pattern (`pointer-events: none` on container, `pointer-events: auto` on interactive children) propagates to all descendants. For `.temporal-slider-row` (pointer-events: none) → `.temporal-range` (pointer-events: auto) → pseudo-element thumb — the problem is that `pointer-events: auto` on the input element means the *entire input hit area* (track + thumb) captures events, not just the thumb.

**The fix:** Change `.temporal-range` to `pointer-events: none`. The `.temporal-slider-row` container keeps `pointer-events: none`. The thumb pseudo-elements keep `pointer-events: all`. This creates a chain where only the thumb circle accepts pointer events.

**Important:** The `.temporal-slider-row` container already has `pointer-events: none` from the Phase 13 fix. That must stay. The `pointer-events: auto` (or `all`) on the thumb pseudo-elements re-enables interaction at the leaf level. The container's `pointer-events: none` does NOT prevent descendants from having `pointer-events: all` — that is exactly how the Phase 13 pattern works for all other interactive children.

### Pattern 3: Track Transparency for Visual Stacking

**What:** When two inputs overlap, each renders its own track. The track on the top input visually covers the track on the bottom input AND its physical area can intercept pointer events that should go to the bottom input's thumb. With `pointer-events: none` on the input element (pattern 1 above), the track's pointer-event surface is irrelevant — but the track must still be visually transparent so the bottom input's thumb is not obscured.

**Current state:** `.temporal-range::-webkit-slider-runnable-track` has `background: var(--color-surface-raised)` — this is NOT transparent. This means the top input's track visually covers the bottom input's thumb when they overlap. The track must be `background: transparent` or the z-index relationship between inputs must ensure thumbs render above tracks.

**The fix:** Change `:-webkit-slider-runnable-track` and `::moz-range-track` to `background: transparent`. A single shared visible track can be rendered as a `::before` pseudo-element on `.dual-range-wrapper`.

### Anti-Patterns to Avoid

- **`pointer-events: auto` on full `<input type="range">` in a dual stack:** The whole input hit area blocks the sibling input. Use `pointer-events: none` + `pointer-events: all` on thumb only.
- **`z-index` on `::webkit-slider-thumb` pseudo-elements to control inter-input stacking:** z-index on pseudo-elements only stacks within their containing box's stacking context. It cannot lift a thumb above the *track* of a sibling input. DOM order and pointer-events control which input receives events — not z-index between sibling elements.
- **Clamping with `temporal_min.get()` inside `on:input` for the same signal:** Use `get_untracked()` to avoid reactive tracking in the handler. In Leptos reactive context, calling `.get()` inside a non-reactive closure (like an event handler) is fine, but `get_untracked()` is the explicit intent for cross-signal reads.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Dual-range slider | Custom canvas-based slider | Fix native `<input type="range">` CSS | Native inputs handle keyboard accessibility, touch events, and ARIA semantics for free |
| Cross-browser thumb styling | JS-injected shadow DOM styles | CSS pseudo-elements (`::webkit-slider-thumb`, `::moz-range-thumb`) | Correct approach, already in place |

**Key insight:** The slider functionality is complete. The only work is CSS pointer-events correction and one-line value clamping per handler.

---

## Common Pitfalls

### Pitfall 1: Opaque Track Blocks Lower Input's Thumb

**What goes wrong:** The top input (`temporal-range-max`) renders its runnable track with `background: var(--color-surface-raised)`. This solid-colored track visually sits on top of the lower input's thumb, making the min thumb appear to disappear when the two thumbs are in overlapping positions.

**Why it happens:** Both inputs are absolutely stacked. The second input in DOM order renders visually on top. Its track background is not transparent.

**How to avoid:** Set `::webkit-slider-runnable-track` and `::moz-range-track` to `background: transparent` on `.temporal-range`. Render the visible track as `::before` on `.dual-range-wrapper` so it appears behind both inputs.

**Warning signs:** Min thumb visible at extremes but disappears when near max thumb. "Both thumbs visible but only max is draggable" is the classic symptom.

### Pitfall 2: pointer-events: auto on Full Input Track Blocks Sibling

**What goes wrong:** With `pointer-events: auto` on `.temporal-range`, the entire 20px-tall × 100%-wide hit area of the top input captures all mouse/touch events, even when the user clicks far from that input's thumb. The bottom input never receives events.

**Why it happens:** `pointer-events: auto` restores default hit-testing to the element's entire rendered box.

**How to avoid:** Set `.temporal-range { pointer-events: none }` so the input itself has no hit area. Only the thumb pseudo-element with `pointer-events: all` captures events. Both inputs' thumbs then independently receive events.

**Warning signs:** Min thumb renders but cannot be dragged at all. Only the max (top input) responds to interaction.

### Pitfall 3: Inverted Range When Min Exceeds Max

**What goes wrong:** If the user drags the min thumb past the max thumb's position, `temporal_min` becomes greater than `temporal_max`. The RAF loop passes these inverted values to `update_temporal_visibility()`, which applies the condition `year >= min_year && year <= max_year` — with min > max, no year satisfies this, so all nodes disappear.

**Why it happens:** No clamping in the existing `on:input` handlers.

**How to avoid:** In the min handler, clamp: `val.min(temporal_max.get_untracked())`. In the max handler: `val.max(temporal_min.get_untracked())`.

**Warning signs:** All graph nodes disappear when the two slider thumbs cross each other.

### Pitfall 4: pointer-events Chain with Container

**What goes wrong:** `.temporal-slider-row` has `pointer-events: none`. Setting `.temporal-range` to `pointer-events: none` is fine — the child's `pointer-events: none` is independent of the parent's. But if `.temporal-range` is set to `pointer-events: auto`, the container's `none` is overridden by the explicit `auto` on the child (pointer-events inheritance: `auto` inherits the parent's value, but explicit values override). This is the current state — the child has explicit `auto` which works despite the parent's `none`.

**The concern from D-02:** "if the container's `pointer-events: none` prevents child event propagation" — this concern is valid for some scenarios but NOT for `pointer-events` with explicit values on children. `pointer-events: none` on a parent does NOT prevent children with explicit `pointer-events: auto` or `all` from receiving events. The Phase 13 pattern proves this already works in this codebase.

**How to avoid:** Trust the Phase 13 pattern. Set `.temporal-range { pointer-events: none }` and `.temporal-range::-webkit-slider-thumb { pointer-events: all }`. The container's `none` does not block the thumb pseudo-elements from receiving events.

---

## Code Examples

### Final CSS State (after fix)

```css
/* Source: derived from MDN dual-range slider + Phase 13 pattern */

.temporal-slider-row {
  /* ... existing layout styles ... */
  pointer-events: none;  /* KEEP — Phase 13 pattern, passes canvas events through */
}

.temporal-range {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 20px;
  -webkit-appearance: none;
  appearance: none;
  background: transparent;
  pointer-events: none;    /* CHANGE from "auto" — prevents track blocking sibling */
}

.temporal-range::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--color-accent);
  cursor: pointer;
  pointer-events: all;     /* KEEP — thumb is the only click target */
  position: relative;
  z-index: 2;
}

.temporal-range::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--color-accent);
  cursor: pointer;
  pointer-events: all;     /* KEEP */
  border: none;
}

.temporal-range::-webkit-slider-runnable-track {
  height: 4px;
  background: transparent; /* CHANGE from var(--color-surface-raised) — prevents visual occlusion */
  border-radius: var(--radius-sm);
}

.temporal-range::-moz-range-track {
  height: 4px;
  background: transparent; /* CHANGE from var(--color-surface-raised) */
  border-radius: var(--radius-sm);
}

/* Shared visible track rendered behind both inputs */
.dual-range-wrapper::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  height: 4px;
  transform: translateY(-50%);
  background: var(--color-surface-raised);
  border-radius: var(--radius-sm);
  pointer-events: none;
}
```

### Clamped Leptos Handlers

```rust
// Source: D-05 from CONTEXT.md
// In TemporalSlider component, resyn-app/src/components/graph_controls.rs

// Min input (temporal-range-min) — clamped to not exceed max
on:input=move |e| {
    use leptos::wasm_bindgen::JsCast;
    let val = e.target().unwrap()
        .dyn_into::<web_sys::HtmlInputElement>().unwrap()
        .value_as_number() as u32;
    temporal_min.set(val.min(temporal_max.get_untracked()));
}

// Max input (temporal-range-max) — clamped to not go below min
on:input=move |e| {
    use leptos::wasm_bindgen::JsCast;
    let val = e.target().unwrap()
        .dyn_into::<web_sys::HtmlInputElement>().unwrap()
        .value_as_number() as u32;
    temporal_max.set(val.max(temporal_min.get_untracked()));
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `pointer-events: auto` on full input track | `pointer-events: none` on track + `pointer-events: all` on thumb only | Standard since CSS Pointer Events Level 1 | Allows stacked inputs to each receive independent events on their own thumbs |
| Relying on `z-index` on pseudo-elements for stacking | DOM order + pointer-events | N/A (z-index on pseudo-elements only affects their own stacking context) | z-index cannot lift a thumb above a sibling input's track hit area |

**Deprecated/outdated:**
- `z-index: 2` on `::webkit-slider-thumb`: Does not affect inter-input event routing. Safe to keep for visual rendering order within the input's own stacking context, but it is not the solution to the overlap problem.

---

## Open Questions

1. **Is D-02 concern about `pointer-events: none` on container valid?**
   - What we know: `.temporal-slider-row { pointer-events: none }` with children having explicit `pointer-events: auto` works (Phase 13 proves this). `pointer-events: none` on a parent does not block children with explicit non-none values.
   - What's unclear: Whether changing children to `pointer-events: none` (for the track) combined with `pointer-events: all` on pseudo-element thumbs will work inside a `pointer-events: none` container.
   - Recommendation: The container's `none` value is inherited by children as `auto` (which means "inherit"), but explicit `pointer-events: none` on the child overrides that. The thumb pseudo-elements with `pointer-events: all` should still receive events. If not, change the container to `pointer-events: auto` per D-02's fallback — the slider row is at the bottom of the screen and does not overlap the canvas interaction area.

2. **Should `temporal_max.get_untracked()` or `temporal_max.get()` be used in the min handler?**
   - What we know: `on:input` event handlers in Leptos are not reactive closures — they are event-driven. `get()` inside a non-reactive context still works but creates a reactive subscription. `get_untracked()` is the idiomatic choice in event handlers to avoid unexpected reactive effects.
   - Recommendation: Use `get_untracked()` in both handlers for consistency with the existing RAF loop pattern in graph.rs.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is a CSS and minor Rust handler fix with no external tool or service dependencies beyond the existing project build toolchain.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust native test + cargo test (native + WASM for app) |
| Config file | Cargo.toml workspace |
| Quick run command | `cargo test graph::lod` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TEMPORAL-01 | `update_temporal_visibility()` filters nodes by year range | unit | `cargo test graph::lod::tests::test_temporal` | Yes (3 tests in lod.rs) |
| TEMPORAL-01 | Both thumbs visible and independently draggable | manual/smoke | agent-browser visual inspection | Not automatable in Rust unit tests |
| TEMPORAL-01 | Value clamping — min cannot exceed max | unit | `cargo test graph::lod` (no new test needed; clamping is in handler, not lod) | Wave 0 gap — add handler unit test if feasible |

**Note:** TEMPORAL-01 is fundamentally a CSS/browser interaction bug. The backend filtering logic (`update_temporal_visibility`) already has full unit test coverage (3 tests confirmed passing). The visible-and-draggable requirement can only be verified with a real browser rendering engine — automated Rust unit tests cannot validate CSS rendering or pointer-events behavior. The acceptance criterion is a manual agent-browser test.

### Sampling Rate

- **Per task commit:** `cargo test graph::lod`
- **Per wave merge:** `cargo test`
- **Phase gate:** `cargo test` green + agent-browser visual confirmation both thumbs visible and draggable

### Wave 0 Gaps

- No new test files needed — existing `graph::lod` unit tests cover the filtering logic.
- Manual browser verification is the gate for TEMPORAL-01 acceptance.

*(No framework install gaps — cargo test infrastructure is in place.)*

---

## Project Constraints (from CLAUDE.md)

- Rust edition 2024, stable toolchain (pinned via `rust-toolchain.toml`) — no language feature changes in this phase
- `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features` (with `-Dwarnings`) must pass — the clamping handler changes must be clippy-clean
- Single `#[tokio::main]` in `main.rs` — not relevant to this phase
- `get_untracked()` is the established pattern in RAF loop for reading signals cross-context — use the same pattern in `on:input` handlers for cross-signal reads
- CSS changes go in `resyn-app/style/main.css` — the established location for all styling

---

## Sources

### Primary (HIGH confidence)

- Direct source inspection: `resyn-app/src/components/graph_controls.rs` lines 79–123 — TemporalSlider component code as-is
- Direct source inspection: `resyn-app/style/main.css` lines 1400–1466 — current CSS state
- Direct source inspection: `resyn-app/src/pages/graph.rs` lines 380–392 — RAF loop temporal sync
- Direct source inspection: `resyn-app/src/graph/lod.rs` — `update_temporal_visibility()` implementation and tests
- Direct source inspection: `.planning/BUGFIX-STATUS.md` — documented bug status for slider (issue #4)
- CONTEXT.md decisions D-01 through D-07 — locked implementation decisions

### Secondary (MEDIUM confidence)

- CSS pointer-events specification behavior: `pointer-events: none` on parent does not block children with explicit non-none values — this is standard CSS behavior, well-documented in MDN. Cross-verified against Phase 13 behavior already present in this codebase.
- Dual-range slider pattern (pointer-events: none on track, all on thumb): standard pattern in CSS-Tricks and MDN range input articles. Confidence raised by direct matching to the bug symptoms described in BUGFIX-STATUS.md.

### Tertiary (LOW confidence)

- None — all claims are grounded in direct source inspection or well-established CSS specifications.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — existing Leptos/web-sys already in use, no new deps
- Architecture: HIGH — canonical CSS dual-range pattern, verified against existing code
- Pitfalls: HIGH — derived from direct inspection of current CSS and documented bug symptoms
- Filtering logic: HIGH — 3 unit tests confirmed passing, implementation verified

**Research date:** 2026-03-24
**Valid until:** 2026-04-24 (stable CSS specifications, no external dependency churn expected)
