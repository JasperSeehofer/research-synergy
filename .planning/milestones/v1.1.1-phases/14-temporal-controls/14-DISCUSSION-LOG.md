# Phase 14: Temporal Controls - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-24
**Phase:** 14-temporal-controls
**Areas discussed:** Thumb visibility, Thumb overlap/draggability, Graph filtering feedback
**Mode:** auto (all decisions auto-selected)

---

## Thumb Visibility

| Option | Description | Selected |
|--------|-------------|----------|
| Cross-browser thumb styling | Ensure explicit width/height/background on both webkit and moz pseudo-elements | ✓ |
| JavaScript-rendered thumbs | Replace native range inputs with custom JS-drawn thumbs | |

**User's choice:** [auto] Cross-browser thumb styling (recommended default)
**Notes:** Existing CSS already defines both pseudo-element variants. Primary risk is that `-webkit-appearance: none` or `pointer-events` chain may be breaking thumb rendering.

---

## Thumb Overlap/Draggability

| Option | Description | Selected |
|--------|-------------|----------|
| Z-index differentiation + value clamping | Higher z-index on max thumb, clamp min/max values in handlers | ✓ |
| Single combined range with two handles | Replace two inputs with a custom dual-handle component | |

**User's choice:** [auto] Z-index differentiation + value clamping (recommended default)
**Notes:** The stacked absolute-position approach is already implemented. Fix is likely CSS z-index tuning and adding min/max clamping to prevent thumbs from crossing.

---

## Graph Filtering Feedback

| Option | Description | Selected |
|--------|-------------|----------|
| Live update during drag | on:input handler fires continuously, RAF loop picks up changes each frame | ✓ |
| Update on release | on:change handler fires only when user releases thumb | |

**User's choice:** [auto] Live update during drag (recommended default — already implemented)
**Notes:** Existing `on:input` handlers and RAF loop sync already provide live filtering. Just needs verification that the pipeline works end-to-end.

---

## Claude's Discretion

- Track highlight between thumbs (visual polish)
- Debug approach (browser dev tools vs agent-browser)
- Z-index restructuring details
- Clamping implementation (handler vs derived signal)

## Deferred Ideas

- Range track highlight (colored bar between thumbs)
- Temporal filtering animation (fade in/out)
- Preset year ranges ("Last 5 years", "Last decade")
