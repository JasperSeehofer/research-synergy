# Phase 14: Temporal Controls - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Fix the dual-range year slider so that both the start-year and end-year thumbs are visible, independently draggable, and moving either thumb updates the graph's temporal filter. The `TemporalSlider` component and all filtering logic already exist — this is a CSS/interaction bugfix, not new feature work.

</domain>

<decisions>
## Implementation Decisions

### Thumb visibility
- **D-01:** Both slider thumbs must be visible with explicit cross-browser styling. The existing CSS defines `::-webkit-slider-thumb` and `::-moz-range-thumb` with 16x16px circular thumbs in `var(--color-accent)` — verify these render in both Chrome/Firefox. If thumbs are invisible, the most likely cause is missing `-webkit-appearance: none` on the track or a transparent background on the input element itself.
- **D-02:** The `.temporal-slider-row` container has `pointer-events: none` (from Phase 13 fix) and `z-index: 10`. The `.temporal-range` inputs have `pointer-events: auto`. Verify this chain actually allows interaction — if the container's `pointer-events: none` prevents child event propagation in some browsers, switch to `pointer-events: auto` on the container (the slider row is at the bottom of the screen, not overlapping the canvas interaction area).

### Thumb overlap and draggability
- **D-03:** Two `<input type="range">` elements are stacked absolutely in `.dual-range-wrapper`. When both thumbs are at the same position, the max slider (second input) sits on top. Ensure the max slider's thumb has a higher z-index so both remain reachable. If a thumb becomes trapped behind the other, add `z-index` differentiation or swap stacking order on hover/focus.
- **D-04:** Each range input covers the full width (`width: 100%`). The track background must be fully transparent on both inputs so they don't obscure each other — the existing CSS sets `background: transparent` on the inputs and styles only the track pseudo-elements. Verify the track pseudo-elements don't create an opaque clickable area that blocks the thumb beneath.
- **D-05:** Constrain values so min thumb cannot exceed max thumb and vice versa. Add clamping in the `on:input` handlers: `temporal_min.set(val.min(temporal_max.get()))` and `temporal_max.set(val.max(temporal_min.get()))`.

### Graph filtering integration
- **D-06:** The RAF loop in `graph.rs` already syncs `temporal_min`/`temporal_max` signals into `GraphState` and calls `update_temporal_visibility()` each frame. Live filtering during drag is the existing behavior via `on:input` — no `on:change` needed. Verify this works end-to-end: dragging a thumb updates the signal, the RAF loop picks it up, and nodes outside the range disappear.
- **D-07:** The year range label (`"{min} – {max}"`) already renders reactively from the signals. No additional work needed for the label.

### Claude's Discretion
- Whether to add a visual track highlight between the two thumbs (colored bar showing selected range)
- Debug approach for identifying the root cause (browser dev tools, agent-browser, CSS inspection)
- Whether the existing z-index values need adjustment or a complete restructure
- Exact clamping implementation details (in handler vs derived signal)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Temporal Slider Component
- `resyn-app/src/components/graph_controls.rs` lines 79–123 — `TemporalSlider` component: two stacked `<input type="range">` with `on:input` handlers updating `RwSignal<u32>` for min/max year
- `resyn-app/src/components/graph_controls.rs` lines 3–77 — `GraphControls` component receiving `temporal_min`/`temporal_max`/`year_bounds` props (currently unused with `let _ =`)

### CSS Styling
- `resyn-app/style/main.css` lines 1400–1466 — `.temporal-slider-row` (absolute bottom, z-index 10, pointer-events none), `.dual-range-wrapper`, `.temporal-range` (pointer-events auto), thumb/track pseudo-elements for webkit and moz

### Graph Page Integration
- `resyn-app/src/pages/graph.rs` lines 70–72 — Signal initialization (`temporal_min=2000`, `temporal_max=2026`, `year_bounds=(2000, 2026)`)
- `resyn-app/src/pages/graph.rs` lines 116–120 — Year bounds sync from `GraphState` after data load
- `resyn-app/src/pages/graph.rs` lines 380–392 — RAF loop syncing temporal signals into `GraphState` and calling `update_temporal_visibility()`
- `resyn-app/src/pages/graph.rs` lines 204–212 — Component mounting: `GraphControls` and `TemporalSlider` in DOM

### Temporal Filtering Logic
- `resyn-app/src/graph/layout_state.rs` lines 52–53 — `temporal_min_year`/`temporal_max_year` fields on `GraphState`
- `resyn-app/src/graph/lod.rs` — `update_temporal_visibility()` function filtering nodes by year range

### Phase 13 Context (pointer-events fix)
- `.planning/phases/13-graph-interaction/13-CONTEXT.md` — D-01/D-02: pointer-events none on overlay containers, pointer-events auto on interactive children

### Known Bug State
- `.planning/STATE.md` Blockers section — "Slider fix attempted (z-index, transparent tracks) — status unclear, may just need browser test"

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TemporalSlider` component: fully wired with two range inputs, reactive signals, and year label display
- `update_temporal_visibility()` in `lod.rs`: filters node visibility by year range — already called in RAF loop
- `GraphState.temporal_min_year` / `temporal_max_year`: state fields already synced from signals each frame
- Year bounds detection in `layout_state.rs`: extracts min/max year from paper data on graph load

### Established Patterns
- Overlay containers use `pointer-events: none` with `pointer-events: auto/all` on interactive children (Phase 13 pattern)
- `RwSignal<u32>` for reactive slider state, synced via `get_untracked()` in RAF loop
- CSS pseudo-elements for cross-browser range input styling (webkit + moz)

### Integration Points
- Temporal signals created in `graph.rs` component scope → passed as props to `TemporalSlider` and `GraphControls`
- RAF loop reads signals each frame → updates `GraphState` → calls `update_temporal_visibility()` → renders
- `GraphControls` receives temporal props but currently discards them with `let _ =` — not used in controls panel

</code_context>

<specifics>
## Specific Ideas

- The bug may be purely CSS — the slider was reportedly attempted fixed with z-index and transparent tracks but never verified in a browser
- Use agent-browser or browser dev tools to visually confirm thumb rendering before changing code
- Phase 13's pointer-events fix may have inadvertently affected the slider — verify the `pointer-events: none` on `.temporal-slider-row` doesn't prevent thumb interaction

</specifics>

<deferred>
## Deferred Ideas

- Range track highlight (colored bar between thumbs showing selected range) — visual polish, not required for functionality
- Temporal filtering animation (fade nodes in/out instead of instant show/hide) — future polish
- Preset year ranges (e.g., "Last 5 years", "Last decade") — future feature

</deferred>

---

*Phase: 14-temporal-controls*
*Context gathered: 2026-03-24*
