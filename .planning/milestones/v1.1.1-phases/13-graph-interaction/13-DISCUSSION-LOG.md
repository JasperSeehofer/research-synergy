# Phase 13: Graph Interaction - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 13-graph-interaction
**Areas discussed:** Overlay z-index blocking, Coordinate transform verification, Force reheat on drag, Hit test tuning
**Mode:** --auto (all areas auto-selected, recommended defaults chosen)

---

## Overlay / Pointer Event Blocking

| Option | Description | Selected |
|--------|-------------|----------|
| Diagnose and fix CSS stacking | Ensure canvas receives all pointer events, overlays use pointer-events: none | ✓ |
| Add event listener to overlay instead | Forward events from overlay to canvas | |
| Remove overlays during interaction | Hide overlays when mouse enters canvas | |

**User's choice:** [auto] Diagnose and fix CSS stacking (recommended default)
**Notes:** STATE.md explicitly flagged this as primary suspect. Existing CSS shows multiple overlays with z-index 10-200 over the canvas area.

---

## Coordinate Transform Verification

| Option | Description | Selected |
|--------|-------------|----------|
| Debug logging + agent-browser verification | Log screen→world transforms, verify with automated click tests | ✓ |
| Add visual debug overlay | Draw crosshair at transformed coordinates | |
| Unit test coordinate round-trips | Add tests for known screen/world coordinate pairs | |

**User's choice:** [auto] Debug logging + agent-browser verification (recommended default)
**Notes:** Phase 12 DPR fix changed convention to CSS pixels only. Must verify screen_to_world still correct.

---

## Force Reheat on Drag

| Option | Description | Selected |
|--------|-------------|----------|
| Moderate reheat (alpha = max(current, 0.3)) | Existing behavior, tune if needed | ✓ |
| Aggressive reheat (alpha = 0.8) | Nodes rearrange significantly after drag | |
| No reheat | Only dragged node moves, rest stay still | |

**User's choice:** [auto] Moderate reheat (recommended default)
**Notes:** Existing mouseup handler already implements this. Preserve unless live testing reveals issues.

---

## Hit Test Tuning

| Option | Description | Selected |
|--------|-------------|----------|
| Keep current defaults | Node radius + 4px edge threshold, tune after verification | ✓ |
| Increase thresholds | Larger targets for easier interaction | |
| Scale-dependent thresholds | Adjust hit radius based on zoom level | |

**User's choice:** [auto] Keep current defaults (recommended default)
**Notes:** 7 existing unit tests validate hit testing logic. Only adjust if live testing shows consistent misses.

---

## Claude's Discretion

- Debugging approach and order of investigation
- Temporary debug logging (add/remove)
- Click-vs-drag threshold adjustment (currently 3px)

## Deferred Ideas

- Touch/mobile interaction support
- Node multi-select
- Right-click context menu
- Keyboard navigation
