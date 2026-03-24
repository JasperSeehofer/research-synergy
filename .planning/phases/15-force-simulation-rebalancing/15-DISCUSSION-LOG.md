# Phase 15: Force Simulation Rebalancing - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-24
**Phase:** 15-force-simulation-rebalancing
**Areas discussed:** Coefficient tuning approach, BFS depth ring placement, Convergence behavior

---

## Coefficient Tuning Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Compile-time constants | Keep as pub const in forces.rs. Tune empirically by rebuilding. Runtime sliders deferred. | ✓ |
| Runtime-configurable now | Pass coefficients through LayoutInput for faster iteration | |

**User's choice:** Compile-time constants
**Notes:** Runtime configurability explicitly deferred to CONFIG-01 future requirement.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Organic clusters | Connected papers form loose clusters with clear spacing. Natural feel like Connected Papers. | ✓ |
| Radial tree structure | Seed at center, citation depth radiating outward in rings. More structured. | |
| You decide | Claude picks whatever produces the most readable layout | |

**User's choice:** Organic clusters

---

| Option | Description | Selected |
|--------|-------------|----------|
| Allow overlap | Simpler — nodes can visually overlap when densely connected | |
| Collision separation | Add collision force pushing overlapping nodes apart based on radii. Requires NodeData schema change. | ✓ |

**User's choice:** Collision separation

---

| Option | Description | Selected |
|--------|-------------|----------|
| Smooth animation matters | Spreading-out should look natural and satisfying. Worth tuning damping/alpha. | ✓ |
| Final state matters most | Settled layout is primary concern, animation quality secondary. | |

**User's choice:** Smooth animation matters

---

| Option | Description | Selected |
|--------|-------------|----------|
| Local rearrangement | Reheat alpha slightly so nearby nodes adjust. Current behavior (alpha = max(0.3, current)). | ✓ |
| Full reheat | Reset alpha to 1.0 to fully re-simulate the entire graph. | |

**User's choice:** Local rearrangement

---

## BFS Depth Ring Placement

| Option | Description | Selected |
|--------|-------------|----------|
| Center (0,0) | Seed at origin, depth-1 in first ring, depth-2 in second ring. Classic radial tree. | |
| Slight offset | Seed slightly off-center to break symmetry and help force sim converge faster. | ✓ |

**User's choice:** Slight offset

---

| Option | Description | Selected |
|--------|-------------|----------|
| Outer ring | Place orphan nodes in outermost ring, scattered. Simulation handles final positioning. | ✓ |
| Random scatter | Place randomly across full layout area. | |

**User's choice:** Outer ring

---

## Convergence Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Full stop | Simulation stops when alpha drops below threshold. Drag reheat restarts. Saves CPU. | ✓ |
| Gentle idle (current) | Alpha floors at ALPHA_MIN, forces keep running with negligible movement. | |
| You decide | Claude picks based on UX for drag interactions and CPU usage. | |

**User's choice:** Full stop

---

| Option | Description | Selected |
|--------|-------------|----------|
| 5-10 seconds | Fast convergence. May sacrifice animation smoothness. | |
| 15-20 seconds | Moderate. Smooth spreading animation with time to watch clusters form. | ✓ |
| 30+ seconds | Slow, very smooth. Full spreading process unfolds gradually. | |

**User's choice:** 15-20 seconds

---

## Claude's Discretion

- Exact coefficient values (repulsion, attraction, damping, ideal distance, alpha decay)
- Ring spacing formula for BFS depth placement
- Alpha threshold for full stop convergence
- Collision force strength and distance calculations

## Deferred Ideas

None — discussion stayed within phase scope
