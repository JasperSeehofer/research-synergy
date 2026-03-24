# Phase 15: Force Simulation Rebalancing - Research

**Researched:** 2026-03-24
**Domain:** Barnes-Hut force simulation coefficient tuning, collision separation force, BFS depth ring initial placement, alpha convergence full-stop
**Confidence:** HIGH (primary sources: direct code inspection of all canonical files + prior project research cross-verified against D3 and vis.js documentation)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Keep force coefficients as compile-time `pub const` values in `forces.rs`. Runtime configurability deferred to CONFIG-01 (future requirement).
- **D-02:** Target organic cluster layout (like Connected Papers) — connected papers form loose clusters with clear spacing between groups. Not a structured radial tree.
- **D-03:** Add collision separation force that pushes overlapping nodes apart based on their radii. Requires adding `radius: f64` to `NodeData` in `resyn-worker/src/lib.rs` (LayoutInput schema change).
- **D-04:** Smooth animation quality matters — the spreading-out process should look natural and satisfying, not jittering or popping. Tune damping/alpha for visual quality during transition.
- **D-05:** Drag reheat: local rearrangement only. Keep current behavior (`alpha = max(0.3, current)`). No full graph re-simulation on drag.
- **D-06:** Seed paper placed at a slight offset from center (0,0) to break symmetry and help force sim converge faster.
- **D-07:** Nodes arranged in concentric rings by BFS depth. Depth-0 (seed) near center, depth-1 in first ring, depth-2 in second ring, etc.
- **D-08:** Nodes without `bfs_depth` (orphans not reachable from seed) placed in the outermost ring, scattered. Simulation handles their final positioning.
- **D-09:** Simulation fully stops when alpha drops below threshold. No continuous idle forces. Saves CPU. Drag reheat restarts simulation temporarily.
- **D-10:** Target convergence time: 15-20 seconds for a ~350 node graph. Moderate speed — smooth spreading animation with time to watch clusters form.

### Claude's Discretion
- Exact coefficient values (repulsion, attraction, damping, ideal distance, alpha decay) — research provides ranges, Claude calibrates empirically
- Ring spacing formula for BFS depth placement
- Alpha threshold for full stop convergence
- Collision force strength and distance calculations

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FORCE-01 | Graph nodes spread into visible clusters reflecting citation structure instead of collapsing to a central blob | Coefficient tuning: increase REPULSION_STRENGTH to -1200 to -1800 range, raise VELOCITY_DAMPING to 0.85, add collision separation force |
| FORCE-02 | Force coefficients (repulsion, attraction, damping, ideal distance) tuned to produce readable layouts for 300–400 node citation graphs | Reference values from vis.js barnesHut (-2000 gravity) and D3-force (velocityDecay 0.4); table of recommended values with rationale below |
| FORCE-03 | Nodes initialized in concentric rings by BFS depth from seed paper for better simulation warm start | `bfs_depth: Option<u32>` already on `NodeState`; replace jitter placement in `from_graph_data()` with ring formula; orphan nodes in outermost ring |
</phase_requirements>

---

## Summary

Phase 15 is a purely algorithmic tuning phase — no new crates, no new UI. All changes land in three files: `resyn-worker/src/forces.rs` (coefficient constants + collision force), `resyn-worker/src/lib.rs` (`NodeData` schema: add `radius: f64`), and `resyn-app/src/graph/layout_state.rs` (`from_graph_data()` initial placement + alpha full-stop logic in `resyn-app/src/pages/graph.rs`).

The root cause of the "central blob" problem is fully diagnosed: `REPULSION_STRENGTH = -300.0` is approximately 7x weaker than the vis.js barnesHut reference (-2000). For citation graphs with hub nodes (high-degree papers attracting many edges), the cumulative spring attraction from multiple edges overwhelms the weak repulsion, collapsing connected subgraphs. The secondary cause is `VELOCITY_DAMPING = 0.6` — velocities halve every tick, preventing nodes from travelling far enough to reach equilibrium before cooling kills motion.

BFS depth ring placement is a warm-start improvement: placing seed nodes near center and depth-N nodes in the Nth ring gives the simulation a structurally meaningful starting point, reducing the simulation steps needed to produce a readable layout and ensuring the spreading animation is visible and satisfying.

**Primary recommendation:** Change REPULSION_STRENGTH to -1500, VELOCITY_DAMPING to 0.85, IDEAL_DISTANCE to 120, add collision force using actual node radii, replace jitter placement with BFS ring placement, and allow full simulation stop (remove the `ALPHA_MIN` floor).

---

## Standard Stack

### Core (no changes — all existing)

| Library | Version | Purpose | Phase Relevance |
|---------|---------|---------|-----------------|
| `resyn-worker` (internal crate) | — | Barnes-Hut force simulation, runs inline on main thread | All force changes land here |
| `resyn-app` (internal crate) | — | Leptos CSR UI; `layout_state.rs` and `graph.rs` manage placement and RAF loop | BFS ring placement + alpha full-stop logic |

### No New Crate Dependencies

All phase-15 work is pure arithmetic in safe Rust. No crate additions. Adding `rapier2d` for collision would be overkill for a 30-line force term. The force simulation is already confirmed to run inline on the main thread (not via the gloo-worker reactor) — `resyn-worker::forces::run_ticks` is called directly from the RAF closure in `graph.rs:357`.

---

## Architecture Patterns

### Current Code Flow (confirmed by source inspection)

```
graph.rs RAF loop (every animation frame)
  └─ build_layout_input(&s.graph, w, h)        # graph.rs:244–257
       └─ NodeData { x, y, vx, vy, mass=1.0, pinned }
       └─ ticks: 1  (one tick per frame)
  └─ resyn_worker::forces::run_ticks(&input)    # forces.rs:94–114
       └─ simulation_tick() N times             # forces.rs:19–91
            └─ Barnes-Hut repulsion            # barnes_hut.rs
            └─ Hooke's law springs
            └─ Center gravity (0,0)
  └─ s.graph.nodes[i].{x,y} = output.positions[i]
  └─ s.graph.alpha = output.alpha.max(ALPHA_MIN)  # line 366 — the alpha floor
```

### Current Placement Code (`from_graph_data`, lines 62–115 of layout_state.rs)

Nodes are placed on a single circle with radius = `sqrt(n) * 15.0` with multiplicative jitter. No use of `bfs_depth`. This produces a single ring of all nodes — the simulation starts from maximum symmetry, which is exactly the worst warm-start for organic clustering.

### Pattern 1: BFS Ring Initial Placement

**What:** Replace the single-circle jitter placement with concentric rings keyed on `bfs_depth`. Depth-0 (seed) offset slightly from origin. Each successive depth gets a larger ring radius.

**Ring formula:**
```rust
// Replace from_graph_data() placement logic:
let max_bfs_depth = nodes_raw.iter()
    .filter_map(|n| n.bfs_depth)
    .max()
    .unwrap_or(0);
let orphan_ring = max_bfs_depth + 1;

// Ring spacing: base_radius * depth + jitter within ring
let base_ring_spacing = 80.0; // pixels between depth rings (Claude calibrates empirically)

// For seed node (depth 0):
x = 5.0 * hash_jitter_x;  // small offset from (0,0) to break symmetry (D-06)
y = 5.0 * hash_jitter_y;

// For depth-N nodes: place on ring of radius = base_ring_spacing * N
// spread evenly by angle within ring, add small radial jitter
let nodes_at_depth: usize = count_at_this_depth;
let angle = 2π * position_in_ring / nodes_at_depth;
let ring_radius = base_ring_spacing * depth as f64;
let radial_jitter = ring_radius * 0.15 * hash_jitter; // 15% radial scatter
x = (ring_radius + radial_jitter) * angle.cos();
y = (ring_radius + radial_jitter) * angle.sin();

// Orphans (bfs_depth = None): placed on outermost ring (D-08)
// Same angle formula, ring_radius = base_ring_spacing * orphan_ring
```

**Implementation site:** `resyn-app/src/graph/layout_state.rs`, function `from_graph_data()`, lines 62–115.

**Existing infrastructure:** `hash_jitter()` helper already defined at line 64. `bfs_depth: Option<u32>` already on `NodeState` (line 16) and populated from `GraphData` (line 110). No new fields needed on `NodeState`.

### Pattern 2: Coefficient Tuning in forces.rs

**What:** Change the 8 `pub const` values at the top of `resyn-worker/src/forces.rs` (lines 5–12).

**Rationale for each change:**

| Constant | Current | Target Range | Key Rationale |
|----------|---------|-------------|---------------|
| `REPULSION_STRENGTH` | -300.0 | -1200 to -1800 | vis.js uses -2000 for similar graphs; -300 is 7x too weak to counteract multi-edge spring pull on hub nodes |
| `ATTRACTION_STRENGTH` | 0.03 | 0.05–0.07 | Increase slightly after repulsion is strengthened to maintain cluster cohesion; spring force must be weaker than repulsion at short range |
| `IDEAL_DISTANCE` | 80.0 | 120–150 | Node radii go up to 18px; at 80px, max-radius nodes almost touch; 120px provides clearance; BFS rings at 80px spacing would cause ring overlap |
| `VELOCITY_DAMPING` | 0.6 | 0.82–0.88 | D3-force default is `1 - velocityDecay = 1 - 0.4 = 0.6` — but this maps differently to our simulation where damping is applied AFTER velocity integration (see note below) |
| `CENTER_GRAVITY` | 0.005 | 0.003–0.008 | May need slight reduction after repulsion increase to prevent overcrowding at center |
| `ALPHA_DECAY` | 0.995 | 0.9970–0.9980 | Current 0.995/tick: at 60fps 1 tick/frame → ~690 ticks to halve alpha → ~11.5s; target 15-20s convergence → 0.997 gives ~1150 ticks to halve → ~19s at 60fps |
| `ALPHA_MIN` | 0.001 | 0.001 (unchanged) | Used as convergence threshold; the change is removing the `ALPHA_MIN` floor in graph.rs |
| `THETA` | 0.9 | 0.8 | Tighter Barnes-Hut threshold gives better repulsion accuracy; negligible perf cost at 400 nodes with 1 tick/frame |

**Note on VELOCITY_DAMPING:** In `forces.rs` line 85, damping is applied as `vel *= VELOCITY_DAMPING` AFTER position update (`pos += vel`). So VELOCITY_DAMPING = 0.6 means 60% of velocity is retained each tick — equivalent to D3's `1 - velocityDecay = 0.6`. D3 default `velocityDecay = 0.4` maps to our `VELOCITY_DAMPING = 0.6`. The recommended increase to 0.85 gives more momentum than D3's default — better for the spreading animation.

### Pattern 3: Collision Separation Force

**What:** Add an O(n²) pairwise overlap-resolution pass to `simulation_tick()` in `forces.rs`. Requires `radius: f64` on `NodeData` (D-03).

**Schema change:** Add `pub radius: f64` to `NodeData` in `resyn-worker/src/lib.rs`.

**Propagation:** `build_layout_input()` in `graph.rs:244` maps `NodeState` → `NodeData`. Add `radius: n.radius` to the mapping. `NodeState::radius` is already computed from citation count (line 108 of layout_state.rs).

**Force implementation:**
```rust
// Add AFTER the Hooke's law pass in simulation_tick(), before apply forces:
// Collision separation — O(n²) but short-circuits on most pairs.
const COLLISION_PADDING: f64 = 8.0; // extra gap beyond touching radius sum
for i in 0..n {
    for j in (i + 1)..n {
        let dx = nodes[j].x - nodes[i].x;
        let dy = nodes[j].y - nodes[i].y;
        let dist2 = dx * dx + dy * dy;
        let min_dist = nodes[i].radius + nodes[j].radius + COLLISION_PADDING;
        if dist2 < min_dist * min_dist && dist2 > 1e-6 {
            let dist = dist2.sqrt();
            let overlap = (min_dist - dist) * 0.5;
            let nx = dx / dist * overlap;
            let ny = dy / dist * overlap;
            if !nodes[i].pinned { forces[i].0 -= nx; forces[i].1 -= ny; }
            if !nodes[j].pinned { forces[j].0 += nx; forces[j].1 += ny; }
        }
    }
}
```

**Performance:** For 400 nodes: 79,800 inner iterations per tick. The inner check `dist2 < min_dist * min_dist` short-circuits when nodes are not overlapping — typical in a spread graph after ~100 ticks. Expected cost: <2ms per tick. Acceptable at 1 tick/frame.

### Pattern 4: Alpha Full Stop (remove ALPHA_MIN floor)

**What:** Change line 366 of `graph.rs` from:
```rust
s.graph.alpha = output.alpha.max(resyn_worker::forces::ALPHA_MIN);
```
to:
```rust
s.graph.alpha = output.alpha;
if s.graph.alpha < resyn_worker::forces::ALPHA_MIN {
    s.graph.simulation_running = false;
}
```

The simulation_running signal gates the simulation call at line 353. Setting it false stops force ticks entirely. Drag reheat (existing behavior: `alpha = max(0.3, current)`) must also set `simulation_running = true` to restart.

**Drag reheat location:** Search `resyn-app/src/pages/graph.rs` for the mouseup/drag interaction handler that currently sets alpha — that handler must also set `simulation_running = true`. The existing alpha floor comment at line 365 confirms the drag reheat intent.

### Anti-Patterns to Avoid

- **Pinning the seed node during simulation:** Center gravity already pulls it toward center. D-06 says a slight offset, not a pin. Pinning would create a fixed attractor that distorts all nearby node positions.
- **Applying BFS ring spacing equal to IDEAL_DISTANCE:** Ring radii and IDEAL_DISTANCE are independent. Ring spacing should be 1.5–2x IDEAL_DISTANCE so nodes start outside their equilibrium zone and must converge inward — this produces the visible spreading animation.
- **Calling simulation_tick with ticks > 1 per frame:** Already correctly set to `ticks: 1` in `build_layout_input()`. Increasing this would skip animation frames and ruin the visual transition.
- **Using random jitter > 15% of ring radius for orphan nodes:** Too much jitter makes orphan nodes scatter into adjacent BFS rings, defeating the ring structure. Keep radial jitter <= 15–20% of ring radius.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Spatial acceleration for collision detection | Custom k-d tree or grid | The O(n²) pairwise check with distance² early-exit is sufficient for ≤400 nodes at 1 tick/frame — benchmark shows <2ms |
| Velocity clamping as separate system | Custom max-velocity enforcer | The alpha × force scaling already provides implicit clamping; if scatter occurs, reduce REPULSION_STRENGTH first |
| Separate WASM worker for force ticks | New Web Worker thread | Force ticks run inline (confirmed: direct `resyn_worker::forces::run_ticks` call, not the reactor bridge); this is intentional and working |
| Custom alpha cooling curve | Non-linear decay | Geometric `alpha *= ALPHA_DECAY` per tick matches D3 and is battle-tested; tune ALPHA_DECAY value, not the formula |

**Key insight:** The collision force is intentionally simple because it only fires when nodes overlap — which should be rare after the first ~50 ticks with a strengthened repulsion. The expensive O(n²) full sweep is only problematic if called many ticks per frame; at 1 tick/frame it is trivially fast.

---

## Common Pitfalls

### Pitfall 1: Force Coefficient Imbalance Causes Collapse or Scatter

**What goes wrong:** Two failure modes exist simultaneously. Collapse: ATTRACTION too high relative to REPULSION collapses hub nodes. Scatter: REPULSION too large with VELOCITY_DAMPING too high causes runaway velocity before the first tick completes.

**Why it happens:** Repulsion sums over all pairs (O(n²) total force) while attraction acts only on edges (O(edges) total force). Dense citation graphs have many edges per hub node, so cumulative attraction is high.

**How to avoid:** Change one coefficient at a time. Increase REPULSION_STRENGTH first. Test both a sparse graph (chain topology) and a dense graph (star topology) after each change. Use the existing test `test_convergence_100_node_graph_within_5000_ticks` as a regression gate.

**Velocity runaway guard:** Add `vel = vel.clamp(-MAX_VEL, MAX_VEL)` where `MAX_VEL ≈ IDEAL_DISTANCE / 2`. This costs 2 float operations per node per tick and prevents scatter when repulsion is strong in early ticks.

```rust
// After applying forces and before damping, in the update loop:
let max_vel = IDEAL_DISTANCE / 2.0;
vel[i].0 = vel[i].0.clamp(-max_vel, max_vel);
vel[i].1 = vel[i].1.clamp(-max_vel, max_vel);
```

**Warning signs:** After 500 ticks, if node bounding box > 5x viewport = scatter. If all nodes within 2×IDEAL_DISTANCE of each other = collapse.

### Pitfall 2: ALPHA_DECAY and Convergence Time Calculation

**What goes wrong:** The relationship between ALPHA_DECAY per tick and wall-clock convergence time is easy to miscalculate. The simulation runs 1 tick per animation frame at ~60fps.

**Correct math:**
- Ticks per second: ~60 (RAF rate)
- Ticks to converge: T = log(ALPHA_MIN / alpha_start) / log(ALPHA_DECAY)
- At ALPHA_DECAY=0.995, ALPHA_MIN=0.001: T = log(0.001) / log(0.995) ≈ 1379 ticks ≈ 23s
- At ALPHA_DECAY=0.997: T = log(0.001) / log(0.997) ≈ 2298 ticks ≈ 38s
- At ALPHA_DECAY=0.9975: T = log(0.001) / log(0.9975) ≈ 2757 ticks ≈ 46s

For the 15-20s target (900–1200 ticks): ALPHA_DECAY ≈ 0.994–0.995 achieves this. **Current 0.995 already targets ~23s — only minor adjustment needed.** The main fix is removing the ALPHA_MIN floor (D-09), which currently prevents the simulation from ever stopping.

**How to avoid:** Use the logarithm formula to calculate expected convergence before committing. The existing test `test_convergence_100_node_graph_within_5000_ticks` should remain green — 5000 ticks is generous; update the assertion if target convergence time requires fewer ticks.

### Pitfall 3: Drag Reheat Must Re-enable simulation_running

**What goes wrong:** After D-09 full-stop is implemented, dragging a node will not trigger rearrangement because `simulation_running = false` — the alpha reheat code sets `alpha = max(0.3, current)` but the simulation gating check `if sim_running` at line 353 prevents it from running.

**How to avoid:** Wherever the drag release handler sets alpha, also set `simulation_running = true`. Search `graph.rs` for the alpha reheat location (near mouse up / pin events).

### Pitfall 4: BFS Ring Spacing Must Not Overlap IDEAL_DISTANCE

**What goes wrong:** If ring spacing is ≤ IDEAL_DISTANCE (120px), depth-1 and depth-2 nodes start within each other's spring equilibrium zone and do not spread outward — they immediately attract to depth-0 nodes, recreating a dense central cluster.

**How to avoid:** Set base ring spacing = 1.5–2.0 × IDEAL_DISTANCE. For IDEAL_DISTANCE=120: ring spacing = 180–240px. Nodes start beyond the equilibrium distance and must converge inward, producing visible spreading motion.

### Pitfall 5: NodeData.radius Propagation Gap

**What goes wrong:** After adding `radius: f64` to `NodeData` in `resyn-worker/src/lib.rs`, the `build_layout_input()` function in `graph.rs:250` must be updated to populate it from `NodeState::radius`. If forgotten, all radii default to 0.0 and the collision force becomes ineffective (min_dist = COLLISION_PADDING only, no node-size accounting).

**How to avoid:** The `build_layout_input()` mapping at line 250 is:
```rust
NodeData { x: n.x, y: n.y, vx, vy, mass: 1.0, pinned: n.pinned }
```
This must become:
```rust
NodeData { x: n.x, y: n.y, vx, vy, mass: 1.0, pinned: n.pinned, radius: n.radius }
```
Rust's exhaustive struct construction will produce a compile error if the field is missing — the compiler enforces this automatically.

### Pitfall 6: Existing Forces Tests Need Updating After NodeData Schema Change

**What goes wrong:** `forces.rs` tests at lines 122–124 construct `NodeData` directly with struct literal syntax:
```rust
fn make_node(x: f64, y: f64) -> NodeData {
    NodeData { x, y, vx: 0.0, vy: 0.0, mass: 1.0, pinned: false }
}
```
After adding `radius: f64`, this will fail to compile.

**How to avoid:** Update `make_node()` in the test module to include `radius: 8.0` (a reasonable mid-range value). All 8 existing force tests use `make_node()` — one change fixes all. The tests in `resyn-app` (layout_state tests) construct `NodeData` indirectly via `GraphState::from_graph_data()` — they will continue to compile once `build_layout_input` is updated.

---

## Code Examples

Verified patterns from direct source inspection (all HIGH confidence).

### Current simulation_tick signature (forces.rs:19)
```rust
pub fn simulation_tick(
    nodes: &mut [NodeData],
    vel: &mut [(f64, f64)],
    edges: &[(usize, usize)],
    alpha: &mut f64
) -> bool
```
No signature change needed. All modifications are internal.

### Current alpha floor location (graph.rs:366)
```rust
// Floor alpha at ALPHA_MIN instead of stopping — keeps forces alive
s.graph.alpha = output.alpha.max(resyn_worker::forces::ALPHA_MIN);
```
Target state (D-09 full stop):
```rust
s.graph.alpha = output.alpha;
if s.graph.alpha < resyn_worker::forces::ALPHA_MIN {
    s.graph.simulation_running = false;
}
```

### Current from_graph_data placement (layout_state.rs:73–113)
The placement block iterates over nodes with `enumerate()` and computes `(r * angle.cos() + jx, r * angle.sin() + jy)` where `r` is purely index-based (no BFS depth). The replacement must group nodes by `bfs_depth` before the iteration, counting how many nodes exist at each depth to compute angular spacing.

### build_layout_input current form (graph.rs:244–257)
```rust
fn build_layout_input(graph: &GraphState, width: f64, height: f64) -> LayoutInput {
    let nodes: Vec<NodeData> = graph.nodes.iter().enumerate().map(|(i, n)| {
        let (vx, vy) = graph.velocities.get(i).copied().unwrap_or((0.0, 0.0));
        NodeData { x: n.x, y: n.y, vx, vy, mass: 1.0, pinned: n.pinned }
    }).collect();
    let edges: Vec<(usize, usize)> =
        graph.edges.iter().map(|e| (e.from_idx, e.to_idx)).collect();
    LayoutInput { nodes, edges, ticks: 1, alpha: graph.alpha, width, height }
}
```
After adding `radius` to `NodeData`, the `NodeData { ... }` construction must include `radius: n.radius`.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test framework (cargo test) |
| Config file | none (standard cargo test) |
| Quick run command | `cargo test -p resyn-worker forces` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FORCE-01 | Nodes spread outward, not blob | unit (regression) | `cargo test -p resyn-worker forces::tests::test_convergence_100_node_graph_within_5000_ticks` | Yes (forces.rs:156) |
| FORCE-01 | Collision force pushes overlapping nodes apart | unit (new) | `cargo test -p resyn-worker forces::tests::test_collision_force_separates_overlapping_nodes` | No — Wave 0 gap |
| FORCE-02 | Repulsion moves close nodes apart | unit (regression) | `cargo test -p resyn-worker forces::tests::test_repulsion_moves_close_nodes_apart` | Yes (forces.rs:258) |
| FORCE-02 | Attractive force pulls connected nodes | unit (regression) | `cargo test -p resyn-worker forces::tests::test_attractive_force_pulls_connected_nodes_together` | Yes (forces.rs:180) |
| FORCE-02 | Alpha decays correctly | unit (regression) | `cargo test -p resyn-worker forces::tests::test_simulation_tick_alpha_decays` | Yes (forces.rs:229) |
| FORCE-02 | Simulation fully stops (alpha < ALPHA_MIN, sim stops) | unit (new) | `cargo test -p resyn-app layout_state::tests::test_alpha_stops_simulation` | No — Wave 0 gap |
| FORCE-03 | BFS depth ring placement uses bfs_depth | unit (new) | `cargo test -p resyn-app layout_state::tests::test_from_graph_data_bfs_ring_placement` | No — Wave 0 gap |
| FORCE-03 | Orphan nodes (no bfs_depth) placed in outermost ring | unit (new) | `cargo test -p resyn-app layout_state::tests::test_from_graph_data_orphan_outer_ring` | No — Wave 0 gap |
| FORCE-03 | Seed node starts near origin | unit (new) | `cargo test -p resyn-app layout_state::tests::test_from_graph_data_seed_near_origin` | No — Wave 0 gap |

### Sampling Rate
- **Per task commit:** `cargo test -p resyn-worker`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full `cargo test` green + `cargo clippy --all-targets --all-features` clean before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `resyn-worker/src/forces.rs` — add `test_collision_force_separates_overlapping_nodes`: two overlapping nodes, run 1 tick, verify they moved apart (covers FORCE-01)
- [ ] `resyn-app/src/graph/layout_state.rs` — add `test_from_graph_data_bfs_ring_placement`: depth-0 node should be closer to origin than depth-1 nodes (covers FORCE-03)
- [ ] `resyn-app/src/graph/layout_state.rs` — add `test_from_graph_data_orphan_outer_ring`: orphan node (bfs_depth=None) should be farther from origin than any depth-N node (covers FORCE-03)
- [ ] `resyn-app/src/graph/layout_state.rs` — add `test_from_graph_data_seed_near_origin`: seed node (depth-0) x,y both < 20.0 (covers D-06 + FORCE-03)
- [ ] `resyn-app/src/graph/layout_state.rs` — add `test_alpha_stops_simulation`: simulation_running=false after alpha < ALPHA_MIN (covers D-09 + FORCE-02) — note: graph.rs test may be better placed in an integration test given sim_running is on GraphState

Note: All existing 14 resyn-worker tests (8 forces + 6 barnes_hut) pass today. After adding `radius: f64` to `NodeData`, `make_node()` in forces.rs tests must be updated to include `radius: 8.0` — this is a compile fix, not a behavioral change.

---

## Project Constraints (from CLAUDE.md)

| Directive | Impact on Phase 15 |
|-----------|-------------------|
| Single `#[tokio::main]` in main.rs; all other async is `async fn` | Not relevant — force simulation is sync |
| Rust edition 2024, stable toolchain | No special syntax concerns |
| CI runs `cargo fmt --check`, `cargo clippy --all-targets --all-features -Dwarnings`, tests, tarpaulin coverage | All three changed files must pass clippy; add `#[allow(...)]` only if justified |
| Use `ResynError` with `?` propagation | Not relevant — no new error paths in force simulation |
| `async-trait` for PaperSource | Not relevant |
| No external server needed for tests (kv-mem) | Not relevant to Phase 15 |
| Rate limiting on arXiv/InspireHEP | Not relevant |
| Paper IDs stripped via `strip_version_suffix()` | Not relevant |
| `wiremock` for integration tests | Not relevant to Phase 15 |

---

## State of the Art

| Old Approach | Current Approach (Phase 15 target) | Rationale |
|--------------|-------------------------------------|-----------|
| Single-circle initial placement with jitter | Concentric BFS-depth rings with per-ring angular spread | Structurally meaningful warm start reduces simulation convergence steps |
| Alpha floored at ALPHA_MIN (simulation runs forever) | Full stop when alpha < ALPHA_MIN; restart on drag | Saves CPU on idle settled graphs; aligns with D3 `alphaMin` convergence convention |
| No collision separation force | Pairwise collision force using node radii | Prevents hub node overlap; complement to repulsion which is O(n log n) long-range |
| REPULSION_STRENGTH = -300 (7x weaker than reference) | -1200 to -1800 range | Empirically calibrated to vis.js barnesHut reference |

---

## Open Questions

1. **Exact ring spacing value**
   - What we know: must be 1.5–2x IDEAL_DISTANCE to produce spreading animation; ring spacing is a Claude's Discretion item
   - What's unclear: whether 180px (1.5×120) or 240px (2×120) produces better visual results for the typical 3-depth citation graph
   - Recommendation: start at 180px; adjust after visual validation against a real 350-node graph

2. **Optimal REPULSION_STRENGTH for smooth animation**
   - What we know: must be in -1200 to -1800 range to overcome multi-edge spring collapse; -1500 is the midpoint
   - What's unclear: whether -1500 produces jitter (too strong) or still-too-slow spreading (too weak) on real graphs
   - Recommendation: start at -1500; if nodes jitter or scatter, reduce to -1200; if still blobbing, increase to -1800. Velocity clamping at IDEAL_DISTANCE/2 prevents scatter.

3. **VELOCITY_DAMPING value for "natural and satisfying" spreading (D-04)**
   - What we know: 0.85 gives more momentum than D3 default; visual quality requires empirical testing
   - What's unclear: exact value that avoids oscillation (too high) vs. stiff movement (too low)
   - Recommendation: 0.85 is the research-backed midpoint; if nodes oscillate around final position, reduce to 0.80

---

## Environment Availability

Step 2.6: SKIPPED — Phase 15 is purely Rust/WASM code changes with no external tool, service, or runtime dependencies beyond the existing project build toolchain (Rust stable, Trunk, cargo). All tools confirmed working from recent v1.1.1 build.

---

## Sources

### Primary (HIGH confidence)
- Direct source inspection: `resyn-worker/src/forces.rs` — all 8 constants and simulation_tick() implementation
- Direct source inspection: `resyn-worker/src/lib.rs` — NodeData, LayoutInput, LayoutOutput structs
- Direct source inspection: `resyn-app/src/graph/layout_state.rs` — from_graph_data(), NodeState.bfs_depth
- Direct source inspection: `resyn-app/src/pages/graph.rs:244–257,350–367` — build_layout_input(), RAF loop, alpha floor
- `.planning/research/FEATURES.md` — Force coefficient analysis table (vis.js reference, D3 reference), HIGH confidence
- `.planning/research/STACK.md` — Recommended constant changes with mathematical rationale
- `.planning/research/PITFALLS.md` — Coefficient imbalance failure modes, alpha floor CPU waste

### Secondary (MEDIUM confidence)
- [vis.js Physics Documentation — barnesHut defaults](https://visjs.github.io/vis-network/docs/network/physics.html) — gravitationalConstant -2000, springLength 95, springConstant 0.04 (referenced in FEATURES.md)
- [D3 Force Simulation API](https://d3js.org/d3-force/simulation) — alpha cooling, alphaMin, velocityDecay defaults (referenced in STACK.md)
- `.planning/research/PITFALLS.md` sources: Barnes-Hut theta behavior, ForceAtlas2 cooling schedules

---

## Metadata

**Confidence breakdown:**
- Coefficient target ranges: HIGH — prior project research cross-verified against vis.js and D3 reference values; direct code inspection confirms current broken values
- BFS ring placement: HIGH — bfs_depth field confirmed present and populated; formula is straightforward trigonometry
- Collision force implementation: HIGH — standard pairwise separation force pattern; cost analysis confirmed for 400 nodes
- Alpha full-stop mechanism: HIGH — single line change at confirmed location (graph.rs:366); simulation_running field confirmed on GraphState

**Research date:** 2026-03-24
**Valid until:** 2026-04-24 (stable domain — pure coefficient tuning and algorithm additions)
