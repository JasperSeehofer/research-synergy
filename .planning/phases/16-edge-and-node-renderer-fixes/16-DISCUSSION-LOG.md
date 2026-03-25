# Phase 16: Edge and Node Renderer Fixes - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-25
**Phase:** 16-edge-and-node-renderer-fixes
**Areas discussed:** Edge visibility, WebGL2 quad edges, Node sharpness, Seed node style

---

## Edge Visibility

### Edge color and contrast

| Option | Description | Selected |
|--------|-------------|----------|
| Subtle but visible | Muted gray #8b949e at alpha ~0.4, similar to GitHub dark theme | |
| High contrast | Lighter #c9d1d9 at alpha ~0.3, prominent web of connections | |
| Color-coded by depth | Edge color varies by BFS depth distance | ✓ |

**User's choice:** Color-coded by depth
**Notes:** User selected depth coding, then refined to single-color alpha fade in follow-up

### Depth palette

| Option | Description | Selected |
|--------|-------------|----------|
| Blue gradient | Bright blue fading through darker blues for deeper edges | |
| Warm-to-cool gradient | Amber to blue transition across depth | |
| Single color, alpha fade | All edges #8b949e, alpha decreases with depth (0.5 to 0.15) | ✓ |

**User's choice:** Single color, alpha fade
**Notes:** Preferred subtlety over multiple colors

### Arrowheads

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, subtle arrowheads | Small arrowheads at target end, same color as edge | ✓ |
| No arrowheads | Plain lines only, direction from BFS depth | |
| Arrowheads on hover only | Show on hover near edge or connected nodes | |

**User's choice:** Yes, subtle arrowheads

### Edge width

| Option | Description | Selected |
|--------|-------------|----------|
| 1.5px | Thin but visible, standard for graph visualizations | ✓ |
| 2px | Slightly thicker, more prominent | |
| You decide | Claude picks based on color/alpha | |

**User's choice:** 1.5px

---

## WebGL2 Quad Edges

### Edge softness

| Option | Description | Selected |
|--------|-------------|----------|
| Soft anti-aliased edges | Fragment shader distance-from-center falloff | ✓ |
| Hard pixel edges | Sharp pixel boundaries, no smoothing | |
| You decide | Claude picks for best result | |

**User's choice:** Soft anti-aliased edges

### Arrowhead rendering

| Option | Description | Selected |
|--------|-------------|----------|
| Separate triangle pass | Standalone triangles in second draw call | ✓ |
| Integrated quad caps | Arrowhead merged into edge quad mesh | |
| You decide | Claude picks based on complexity | |

**User's choice:** Separate triangle pass

---

## Node Sharpness

### Node border style

| Option | Description | Selected |
|--------|-------------|----------|
| Thin bright border | Lighter shade of node fill, fwidth() in WebGL | ✓ |
| Subtle dark border | Darker than fill, depth/shadow effect | |
| No border, glow falloff | Soft luminous falloff, organic feel | |

**User's choice:** Thin bright border

### Node fill style

| Option | Description | Selected |
|--------|-------------|----------|
| Flat color | Solid fill, no gradient, clean and fast | ✓ |
| Subtle radial gradient | Lighter center fading to darker edge, 3D illusion | |
| You decide | Claude picks for Canvas/WebGL consistency | |

**User's choice:** Flat color

---

## Seed Node Style

### Seed color

| Option | Description | Selected |
|--------|-------------|----------|
| Warm amber #d29922 | GitHub warning/highlight, stands out against blue nodes | ✓ |
| Bright gold #f0b429 | More saturated, very prominent | |
| Soft gold #e3b341 | Between amber and gold, GitHub star color | |

**User's choice:** Warm amber #d29922

### Outer ring style

| Option | Description | Selected |
|--------|-------------|----------|
| Solid ring, 2px gap | Transparent gap between fill and ring, planetary ring effect | ✓ |
| Pulsing glow ring | Animated glow that subtly pulses | |
| Double border | Thicker 3px outer border, no gap | |

**User's choice:** Solid ring, 2px gap

### Seed label visibility

| Option | Description | Selected |
|--------|-------------|----------|
| Always visible | Shown at all zoom levels regardless of LOD | |
| Same as other nodes | Follow standard LOD label rules | ✓ |
| You decide | Claude picks | |

**User's choice:** Same as other nodes

---

## Claude's Discretion

- Exact alpha values per BFS depth level (within 0.15-0.5 range)
- Quad edge vertex buffer layout and attribute stride
- fwidth() smoothing parameters for node border anti-aliasing
- Outer ring radius offset and thickness
- Arrowhead size scaling for 1.5px edges
- Exact brighter-shade calculation for node borders

## Deferred Ideas

None — discussion stayed within phase scope
