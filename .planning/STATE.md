---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Graph Rendering Overhaul
status: Ready to plan
stopped_at: Phase 17 UI-SPEC approved
last_updated: "2026-03-26T09:46:17.806Z"
progress:
  total_phases: 3
  completed_phases: 2
  total_plans: 4
  completed_plans: 4
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-24)

**Core value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph
**Current focus:** Phase 16 — edge-and-node-renderer-fixes

## Current Position

Phase: 17
Plan: Not started

## Performance Metrics

**Velocity:**

- Total plans completed: 0
- Average duration: —
- Total execution time: —

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

*Updated after each plan completion*
| Phase 15-force-simulation-rebalancing P01 | 2 | 1 tasks | 3 files |
| Phase 15-force-simulation-rebalancing P02 | 6min | 2 tasks | 2 files |
| Phase 16-edge-and-node-renderer-fixes P01 | 8min | 2 tasks | 4 files |
| Phase 16-edge-and-node-renderer-fixes P02 | 12min | 2 tasks | 18 files |

## Accumulated Context

### Decisions

(Full decision log in PROJECT.md Key Decisions table)

Recent decisions affecting v1.2:

- DPR convention: CSS pixels throughout — DPR only at canvas physical sizing and GL viewport (Phase 14)
- Dual-range slider fix pattern confirmed (Phase 14)
- [Phase 15-force-simulation-rebalancing]: REPULSION_STRENGTH set to -1500 (5x stronger than -300; vis.js uses -2000) to prevent hub node collapse
- [Phase 15-force-simulation-rebalancing]: NodeData.radius wired from NodeState.radius (citation-count-scaled 4-18px) through LayoutInput to collision force
- [Phase 15-force-simulation-rebalancing]: base_ring_spacing = 180px (1.5x IDEAL_DISTANCE) for BFS ring placement so nodes start beyond equilibrium for visible spreading animation
- [Phase 15-force-simulation-rebalancing]: check_alpha_convergence() extracted to GraphState method for testability — avoids testing within WASM/Leptos RAF closure
- [Phase 16-edge-and-node-renderer-fixes]: depth_alpha uses max BFS depth of edge endpoints for progressive hierarchy dimming
- [Phase 16-edge-and-node-renderer-fixes]: All Canvas 2D line widths divided by viewport.scale for screen-space consistency at all zoom levels
- [Phase 16-edge-and-node-renderer-fixes]: build_quad_edge uses world-space perpendicular offset (half_width = 0.75/scale) so existing EDGE_VERT shader needs no changes
- [Phase 16-edge-and-node-renderer-fixes]: depth_alpha_f32 mirrors Canvas 2D depth_alpha() exactly: max(from_depth, to_depth) with 0.50/0.35/0.25/0.15 thresholds for consistent dimming across renderers
- [Phase 16-edge-and-node-renderer-fixes]: Seed outer ring reuses edge shader program (triangle annulus) - no new VAO or shader needed

### Pending Todos

None.

### Blockers/Concerns

- Force coefficient exact values require empirical calibration during Phase 15 — reference ranges provided by research but optimal values need visual validation against real graphs
- Phase 16: WebGL quad edge geometry integration with existing arrowhead pass needs careful vertex buffer refactor (budget extra time)
- Phase 17: measureText cache is required, not optional — must be implemented at graph load time, not deferred as optimization

## Session Continuity

Last session: 2026-03-26T09:46:17.804Z
Stopped at: Phase 17 UI-SPEC approved
Resume file: .planning/phases/17-viewport-fit-and-label-collision/17-UI-SPEC.md
