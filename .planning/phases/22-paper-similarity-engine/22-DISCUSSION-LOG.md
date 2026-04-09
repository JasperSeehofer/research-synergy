# Phase 22: Paper Similarity Engine - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 22-paper-similarity-engine
**Areas discussed:** Similar Papers tab presentation, Similarity edge overlay visuals, Computation trigger & freshness, Graph interaction with similarity mode

---

## Similar Papers tab presentation

| Option | Description | Selected |
|--------|-------------|----------|
| Ranked list with scores | Show similarity score, title, authors, year. Clicking navigates. | |
| Ranked list with explanation | Same as above, plus 2-3 shared keywords explaining WHY similar (uses existing `shared_high_weight_terms`). | ✓ |
| Compact cards | Mini cards per similar paper with title, score badge, and shared terms. More visual but more space. | |

**User's choice:** Ranked list with explanation
**Notes:** Shared keywords come nearly free since `shared_high_weight_terms()` already exists in `gap_analysis/similarity.rs`.

---

## Similarity edge overlay visuals

| Option | Description | Selected |
|--------|-------------|----------|
| Dashed lines, distinct color | Dashed amber/orange edges, thickness scales with similarity score. Clear visual separation from solid gray citation edges. | ✓ |
| Dotted lines, subtle | Thin dotted lines in muted color. Less visual noise but harder to spot. | |
| Gradient opacity | Different hue with opacity mapped to score. Clean but could blur with citations. | |

**User's choice:** Dashed lines, distinct color
**Notes:** None

### Follow-up: Minimum similarity threshold

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed threshold | Claude picks sensible default (top-5 or score > 0.3). Keeps graph clean. | ✓ |
| User-adjustable slider | Threshold slider in graph controls for real-time adjustment. | |
| You decide | Claude picks whatever works best. | |

**User's choice:** Fixed threshold for now
**Notes:** User explicitly said "for now" — slider could be added in future iteration.

---

## Computation trigger & freshness

| Option | Description | Selected |
|--------|-------------|----------|
| Silent recompute | Background update after TF-IDF. Tab shows current data or "Run analysis first" empty state. | ✓ (modified) |
| Toast notification | Silent recompute plus brief toast when done. | |
| Progress indicator | Spinner or progress bar during recomputation. | |

**User's choice:** Silent recompute, but with spinner + message in the Similar Papers tab while waiting for TF-IDF vectors to exist
**Notes:** User wanted a spinner with message specifically in the tab when TF-IDF hasn't been run yet, not during recomputation itself.

---

## Graph interaction with similarity mode

| Option | Description | Selected |
|--------|-------------|----------|
| Overlay only | Similarity edges on top of citation edges. Both visible simultaneously. No interaction changes. | |
| Focus mode | Clicking a node highlights similarity neighbors, dims everything else. Citations stay visible but faded. | |
| Dual-layer toggle | Citation and similarity edges independently hideable. | ✓ (extended) |

**User's choice:** Dual-layer toggle AND force model swap
**Notes:** User specifically requested that toggling to similarity-only mode should change the force simulation so similar papers cluster together instead of using citation structure. This was an unprompted addition — the user sees this as two fundamentally different spatial views of the corpus.

### Follow-up: Force model when both visible

| Option | Description | Selected |
|--------|-------------|----------|
| Citation forces only | Default layout stays citation-based. Similarity edges are visual overlay. Similarity-only mode swaps forces. | ✓ |
| Blended forces | Both attract, weighted. Could get chaotic. | |
| Primary toggle | One force model at a time, user picks which. | |

**User's choice:** Citation forces only (default), similarity forces when citation edges hidden
**Notes:** Clean two-mode approach avoids muddy blended layouts.

---

## Claude's Discretion

- Exact amber/orange color value for similarity edges
- Dash pattern details
- Fixed similarity threshold value
- Spinner styling and message copy
- Similar papers list item layout
- Force simulation parameters for similarity layout

## Deferred Ideas

None — discussion stayed within phase scope
