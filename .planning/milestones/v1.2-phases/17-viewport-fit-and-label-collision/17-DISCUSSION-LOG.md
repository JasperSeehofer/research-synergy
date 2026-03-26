# Phase 17: Viewport Fit and Label Collision - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md -- this log preserves the alternatives considered.

**Date:** 2026-03-25
**Phase:** 17-viewport-fit-and-label-collision
**Areas discussed:** Auto-fit behavior, User override latch, Label collision strategy, Convergence indicator, Fit button placement & style, Label font & appearance

---

## Auto-fit Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Smooth animated pan-zoom | Lerp scale + offset over ~0.5s. More polished feel. | ✓ |
| Instant snap | Set scale/offset in one frame. Simple. | |
| You decide | Claude picks. | |

**User's choice:** Smooth animated pan-zoom
**Notes:** None

### Fit Padding

| Option | Description | Selected |
|--------|-------------|----------|
| Generous (10% margin) | 10% of viewport on each side. | ✓ |
| Tight (5% margin) | 5% padding. | |
| You decide | Claude picks. | |

**User's choice:** Generous (10% margin)

### Fit Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Visible nodes only | Bounding box from LOD + temporal visible nodes only. | ✓ |
| All nodes always | Includes hidden nodes. | |

**User's choice:** Visible nodes only

### Initial Fit Trigger

| Option | Description | Selected |
|--------|-------------|----------|
| Only after convergence | Nodes start in BFS rings at default viewport. Auto-fit fires once simulation stabilizes. | ✓ |
| Both on load and convergence | Frame BFS ring layout immediately, then re-fit after convergence. | |
| You decide | Claude picks. | |

**User's choice:** Only after convergence

---

## User Override Latch

| Option | Description | Selected |
|--------|-------------|----------|
| Permanent latch + re-center button | Any manual pan/zoom sets flag. Fit button for manual re-trigger. | ✓ |
| Permanent latch, no button | Once user interacts, auto-fit gone. +/- and drag only. | |
| Reset on new graph load | Latch resets on new seed paper. | |

**User's choice:** Permanent latch + re-center button

### Reheat Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| No -- drag reheat never re-fits | Drag is deliberate. Latch stays set. | ✓ |
| Yes -- re-fit after reheat converges | Treats reheat like mini-simulation. | |
| You decide | Claude picks. | |

**User's choice:** No -- drag reheat never re-fits

---

## Label Collision Strategy

### Priority Order

| Option | Description | Selected |
|--------|-------------|----------|
| Seed first, then citation count | Most-cited papers labeled first after seed. Matches LABEL-01. | ✓ |
| Seed first, then BFS depth + citations | Structural proximity prioritized. | |
| You decide | Claude picks. | |

**User's choice:** Seed first, then citation count

### Label Density

| Option | Description | Selected |
|--------|-------------|----------|
| Show as many as fit | Greedy placement, maximize info density. | |
| Sparse -- generous spacing | Extra padding between boxes. Cleaner. | ✓ |
| You decide | Claude picks. | |

**User's choice:** Sparse -- generous spacing

### Hover Label

| Option | Description | Selected |
|--------|-------------|----------|
| Yes -- hover always shows label | Hovering reveals label regardless of collision state. | ✓ |
| No -- labels only via collision system | Hover highlights but no forced label. | |
| You decide | Claude picks. | |

**User's choice:** Yes -- hover always shows label

### Label Performance

| Option | Description | Selected |
|--------|-------------|----------|
| Cache + invalidate on viewport change | Compute once, recompute on zoom/pan. measureText cached. | ✓ |
| Every frame | Simplest but expensive. | |
| You decide | Claude picks. | |

**User's choice:** Cache + invalidate on viewport change

---

## Fit Button Placement & Style

### Placement

| Option | Description | Selected |
|--------|-------------|----------|
| Next to zoom +/- buttons | Same control group. Three viewport controls together. | ✓ |
| Separate group | Own control group. More distinct. | |
| You decide | Claude places where it fits. | |

**User's choice:** Next to zoom +/- buttons

### Style

| Option | Description | Selected |
|--------|-------------|----------|
| Icon: expand arrows | Unicode expand icon. Minimalist. | ✓ |
| Text: 'Fit' | Plain text button. | |
| You decide | Claude picks. | |

**User's choice:** Icon: expand arrows

---

## Label Font & Appearance

### Priority Visual Distinction

| Option | Description | Selected |
|--------|-------------|----------|
| All labels same style | Uniform style. Priority only affects culling. | ✓ |
| Tiered: seed bold, rest normal | Seed label bolder. | |
| Tiered: seed + top-cited brighter | Multi-tier brightness. | |

**User's choice:** All labels same style, BUT with modern pill/badge styling (opaque background + thin border)
**Notes:** User specifically requested a modern tooltip look: opaque background with thin border for clean appearance.

### Label Pill Colors

| Option | Description | Selected |
|--------|-------------|----------|
| Dark semi-transparent bg + subtle border | bg: rgba(13,17,23,0.85), border: #30363d, text: #cccccc | ✓ |
| Solid dark bg + brighter border | bg: #161b22, border: #8b949e | |
| You decide | Claude picks. | |

**User's choice:** Dark semi-transparent bg + subtle border

---

## Convergence Indicator

### Indicator Style

| Option | Description | Selected |
|--------|-------------|----------|
| Text status badge | 'Simulating...' / 'Settled'. Minimal. | ✓ |
| Alpha progress bar | Thin bar showing alpha decay. | |
| Animated spinner | Spinning icon, checkmark when done. | |
| You decide | Claude picks. | |

**User's choice:** Text status badge

### Pause State Distinction

| Option | Description | Selected |
|--------|-------------|----------|
| Three states: Simulating / Paused / Settled | Full information. | ✓ |
| Two states: Running / Settled | Simpler. | |

**User's choice:** Three states

---

## Claude's Discretion

- Lerp easing function, collision padding multiplier, measureText cache invalidation, hover label animation, pill corner radius, status badge CSS, WebGL2 label path

## Deferred Ideas

None
