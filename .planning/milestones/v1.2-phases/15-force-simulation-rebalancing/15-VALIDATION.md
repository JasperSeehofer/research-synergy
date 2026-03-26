---
phase: 15
slug: force-simulation-rebalancing
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-24
---

# Phase 15 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test framework (cargo test) |
| **Config file** | none (standard cargo test) |
| **Quick run command** | `cargo test -p resyn-worker forces` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p resyn-worker forces`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 15-01-01 | 01 | 0 | FORCE-01 | unit (new) | `cargo test -p resyn-worker forces::tests::test_collision_force_separates_overlapping_nodes` | W0 | pending |
| 15-02-01 | 02 | 0 | FORCE-03 | unit (new) | `cargo test -p resyn-app layout_state::tests::test_from_graph_data_bfs_ring_placement` | W0 | pending |
| 15-02-02 | 02 | 0 | FORCE-03 | unit (new) | `cargo test -p resyn-app layout_state::tests::test_from_graph_data_orphan_outer_ring` | W0 | pending |
| 15-02-03 | 02 | 0 | FORCE-03 | unit (new) | `cargo test -p resyn-app layout_state::tests::test_from_graph_data_seed_near_origin` | W0 | pending |
| 15-02-04 | 02 | 0 | FORCE-02 | unit (new) | `cargo test -p resyn-app layout_state::tests::test_alpha_stops_simulation` | W0 | pending |
| 15-02-05 | 02 | 1 | FORCE-01 | unit (regression) | `cargo test -p resyn-worker forces::tests::test_convergence_100_node_graph_within_5000_ticks` | exists | pending |
| 15-02-06 | 02 | 1 | FORCE-02 | unit (regression) | `cargo test -p resyn-worker forces::tests::test_repulsion_moves_close_nodes_apart` | exists | pending |
| 15-02-07 | 02 | 1 | FORCE-02 | unit (regression) | `cargo test -p resyn-worker forces::tests::test_attractive_force_pulls_connected_nodes_together` | exists | pending |
| 15-02-08 | 02 | 1 | FORCE-02 | unit (regression) | `cargo test -p resyn-worker forces::tests::test_simulation_tick_alpha_decays` | exists | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

- [ ] `resyn-worker/src/forces.rs` — add `test_collision_force_separates_overlapping_nodes`: two overlapping nodes with radii, run 1 tick, verify they moved apart
- [ ] `resyn-app/src/graph/layout_state.rs` — add `test_from_graph_data_bfs_ring_placement`: depth-0 node closer to origin than depth-1 nodes
- [ ] `resyn-app/src/graph/layout_state.rs` — add `test_from_graph_data_orphan_outer_ring`: orphan node (bfs_depth=None) farther from origin than any depth-N node
- [ ] `resyn-app/src/graph/layout_state.rs` — add `test_from_graph_data_seed_near_origin`: seed node (depth-0) x,y both < 20.0
- [ ] `resyn-app/src/graph/layout_state.rs` — add `test_alpha_stops_simulation`: create GraphState, set alpha below ALPHA_MIN, call check_alpha_convergence(), verify simulation_running=false

*Note: All existing 14 resyn-worker tests (8 forces + 6 barnes_hut) must continue to pass. After adding `radius: f64` to `NodeData`, update `make_node()` helper in forces.rs tests to include `radius: 8.0`.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Organic cluster layout resembles Connected Papers | FORCE-01 | Visual layout quality requires human judgment | Load a 350-node citation graph; verify nodes form loose clusters with clear spacing between groups |
| Spreading animation looks natural and satisfying | D-04 | Animation smoothness is subjective | Watch the graph load animation; verify no jittering or popping; spreading should feel smooth |
| Convergence in 15-20 seconds | D-10 | Wall-clock timing at target graph size | Load a ~350 node graph; time from load to stable layout |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
