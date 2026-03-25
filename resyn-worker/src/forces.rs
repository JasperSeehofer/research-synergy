//! Force simulation using Barnes-Hut for repulsion.

use crate::{LayoutInput, LayoutOutput, NodeData, barnes_hut};

pub const REPULSION_STRENGTH: f64 = -5000.0;
pub const ATTRACTION_STRENGTH: f64 = 0.008;
pub const CENTER_GRAVITY: f64 = 0.0;
pub const ALPHA_DECAY: f64 = 0.995;
pub const ALPHA_MIN: f64 = 0.001;
pub const THETA: f64 = 0.8;
pub const IDEAL_DISTANCE: f64 = 400.0;
pub const VELOCITY_DAMPING: f64 = 0.7;
/// Depth multiplier for ideal distance between nodes at different BFS depths.
/// Edges spanning N depths get ideal distance = IDEAL_DISTANCE * (1 + DEPTH_DISTANCE_FACTOR * N).
pub const DEPTH_DISTANCE_FACTOR: f64 = 0.5;

/// Run one tick of the force simulation.
///
/// `nodes` holds position/mass/pinned state and is mutated in place.
/// Velocities are maintained via a parallel `vel` slice.
/// Returns `true` if the simulation has converged (alpha < ALPHA_MIN).
pub fn simulation_tick(
    nodes: &mut [NodeData],
    vel: &mut [(f64, f64)],
    edges: &[(usize, usize)],
    alpha: &mut f64,
) -> bool {
    let n = nodes.len();
    if n == 0 {
        *alpha *= ALPHA_DECAY;
        return *alpha < ALPHA_MIN;
    }

    // Build quadtree from current positions.
    let positions: Vec<(f64, f64)> = nodes.iter().map(|nd| (nd.x, nd.y)).collect();
    let masses: Vec<f64> = nodes.iter().map(|nd| nd.mass).collect();
    let tree = barnes_hut::QuadTree::build(&positions, &masses);

    // Accumulate forces.
    let mut forces: Vec<(f64, f64)> = vec![(0.0, 0.0); n];

    // 1. Barnes-Hut repulsion.
    for i in 0..n {
        if nodes[i].pinned {
            continue;
        }
        let (fx, fy) = barnes_hut::barnes_hut_repulsion(
            &tree,
            nodes[i].x,
            nodes[i].y,
            nodes[i].mass,
            THETA,
            REPULSION_STRENGTH,
        );
        forces[i].0 += fx;
        forces[i].1 += fy;
    }

    // 2. Attractive forces (degree-normalized).
    // Divide by sqrt(degree) so hub nodes don't collapse their neighborhood.
    let mut degree = vec![0u32; n];
    for &(a, b) in edges {
        if a < n && b < n {
            degree[a] += 1;
            degree[b] += 1;
        }
    }
    for &(a, b) in edges {
        if a >= n || b >= n {
            continue;
        }
        let dx = nodes[b].x - nodes[a].x;
        let dy = nodes[b].y - nodes[a].y;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
        // Depth-aware ideal distance: edges spanning multiple BFS depths get
        // longer rest lengths, naturally separating depth layers into rings.
        // Cap depth_diff at 6 to avoid huge distances from orphan nodes (u32::MAX).
        let depth_diff = (nodes[a].bfs_depth.abs_diff(nodes[b].bfs_depth)).min(6) as f64;
        let ideal = IDEAL_DISTANCE * (1.0 + DEPTH_DISTANCE_FACTOR * depth_diff);
        // One-sided spring: only attract when farther than ideal distance.
        // When closer, repulsion alone handles separation. This removes the
        // net inward bias that contracts connected components into a blob.
        let stretch = (dist - ideal).max(0.0);
        let deg_factor = ((degree[a].max(1) * degree[b].max(1)) as f64).sqrt();
        let force_mag = ATTRACTION_STRENGTH * stretch / deg_factor;
        let fx = force_mag * dx / dist;
        let fy = force_mag * dy / dist;

        if !nodes[a].pinned {
            forces[a].0 += fx;
            forces[a].1 += fy;
        }
        if !nodes[b].pinned {
            forces[b].0 -= fx;
            forces[b].1 -= fy;
        }
    }

    // Apply forces with alpha floor. All forces use max(alpha, 0.005) so the
    // equilibrium layout is maintained even at convergence. This prevents
    // repeated reheats from slowly compacting the graph — residual force at
    // 0.005 keeps the repulsion/attraction balance active.
    let effective_alpha = alpha.max(0.005);
    let max_vel = IDEAL_DISTANCE / 2.0;
    for i in 0..n {
        if nodes[i].pinned {
            continue;
        }
        vel[i].0 += forces[i].0 * effective_alpha;
        vel[i].1 += forces[i].1 * effective_alpha;
        // Magnitude-based velocity clamping — preserves direction, prevents scatter.
        // Per-axis clamping would create 45° artifacts (X-shape) when both axes saturate.
        let speed2 = vel[i].0 * vel[i].0 + vel[i].1 * vel[i].1;
        if speed2 > max_vel * max_vel {
            let scale = max_vel / speed2.sqrt();
            vel[i].0 *= scale;
            vel[i].1 *= scale;
        }
        nodes[i].x += vel[i].0;
        nodes[i].y += vel[i].1;
        vel[i].0 *= VELOCITY_DAMPING;
        vel[i].1 *= VELOCITY_DAMPING;
    }

    // 4. Collision resolution — hard constraint, resolves actual overlap only.
    // Runs AFTER position integration. Nudges overlapping node pairs apart by
    // exactly half the overlap distance. No padding, no force scaling — this is
    // a geometric constraint, not a force, so it doesn't distort the layout.
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = nodes[j].x - nodes[i].x;
            let dy = nodes[j].y - nodes[i].y;
            let dist2 = dx * dx + dy * dy;
            let min_dist = nodes[i].radius + nodes[j].radius;
            if dist2 < min_dist * min_dist && dist2 > 1e-6 {
                let dist = dist2.sqrt();
                let half_overlap = (min_dist - dist) * 0.5;
                let nx = dx / dist * half_overlap;
                let ny = dy / dist * half_overlap;
                if !nodes[i].pinned {
                    nodes[i].x -= nx;
                    nodes[i].y -= ny;
                }
                if !nodes[j].pinned {
                    nodes[j].x += nx;
                    nodes[j].y += ny;
                }
            }
        }
    }

    *alpha *= ALPHA_DECAY;
    *alpha < ALPHA_MIN
}

/// Run `ticks` iterations of the force simulation and return positions + convergence flag.
pub fn run_ticks(input: &LayoutInput) -> LayoutOutput {
    let mut nodes: Vec<NodeData> = input.nodes.clone();
    let mut vel: Vec<(f64, f64)> = input.nodes.iter().map(|nd| (nd.vx, nd.vy)).collect();
    let mut alpha = input.alpha;
    let mut converged = false;

    for _ in 0..input.ticks {
        converged = simulation_tick(&mut nodes, &mut vel, &input.edges, &mut alpha);
        if converged {
            break;
        }
    }

    let positions = nodes.iter().map(|nd| (nd.x, nd.y)).collect();
    let velocities = vel;
    LayoutOutput {
        positions,
        velocities,
        alpha,
        converged,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeData;

    fn make_node(x: f64, y: f64) -> NodeData {
        NodeData {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            mass: 1.0,
            pinned: false,
            radius: 8.0,
            bfs_depth: 0,
        }
    }

    fn make_vel(n: usize) -> Vec<(f64, f64)> {
        vec![(0.0, 0.0); n]
    }

    #[test]
    fn test_simulation_tick_pinned_nodes_no_movement() {
        let mut nodes = vec![
            NodeData {
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                mass: 1.0,
                pinned: true,
                radius: 8.0,
                bfs_depth: 0,
            },
            NodeData {
                x: 50.0,
                y: 50.0,
                vx: 0.0,
                vy: 0.0,
                mass: 1.0,
                pinned: true,
                radius: 8.0,
                bfs_depth: 0,
            },
        ];
        let mut vel = make_vel(2);
        let edges = vec![(0, 1)];
        let mut alpha = 1.0;
        simulation_tick(&mut nodes, &mut vel, &edges, &mut alpha);
        assert!(
            (nodes[0].x - 0.0).abs() < 1e-10,
            "pinned node x should not change"
        );
        assert!(
            (nodes[0].y - 0.0).abs() < 1e-10,
            "pinned node y should not change"
        );
        assert!(
            (nodes[1].x - 50.0).abs() < 1e-10,
            "pinned node x should not change"
        );
        assert!(
            (nodes[1].y - 50.0).abs() < 1e-10,
            "pinned node y should not change"
        );
    }

    #[test]
    fn test_convergence_returns_true_when_alpha_below_threshold() {
        let mut nodes = vec![make_node(0.0, 0.0), make_node(10.0, 0.0)];
        let mut vel = make_vel(2);
        let edges = vec![];
        let mut alpha = ALPHA_MIN * 0.5; // Already below threshold.
        let converged = simulation_tick(&mut nodes, &mut vel, &edges, &mut alpha);
        assert!(
            converged,
            "should report converged when alpha is below ALPHA_MIN"
        );
    }

    #[test]
    fn test_convergence_100_node_graph_within_5000_ticks() {
        // Generate 100 nodes in a grid pattern.
        let mut node_data = Vec::new();
        for i in 0..10 {
            for j in 0..10 {
                node_data.push(make_node(i as f64 * 20.0, j as f64 * 20.0));
            }
        }
        // Connect in a chain.
        let edges: Vec<(usize, usize)> = (0..99).map(|i| (i, i + 1)).collect();

        let input = LayoutInput {
            nodes: node_data,
            edges,
            ticks: 5000,
            alpha: 1.0,
            width: 800.0,
            height: 600.0,
        };
        let output = run_ticks(&input);
        assert!(
            output.converged,
            "100-node graph should converge within 5000 ticks"
        );
    }

    #[test]
    fn test_attractive_force_pulls_connected_nodes_together() {
        // Two nodes far apart, connected by an edge. After ticks, they should move closer.
        let input = LayoutInput {
            nodes: vec![make_node(0.0, 0.0), make_node(500.0, 0.0)],
            edges: vec![(0, 1)],
            ticks: 50,
            alpha: 1.0,
            width: 1000.0,
            height: 1000.0,
        };
        let initial_dist = 500.0_f64;
        let output = run_ticks(&input);
        let (x0, _) = output.positions[0];
        let (x1, _) = output.positions[1];
        let final_dist = (x1 - x0).abs();
        assert!(
            final_dist < initial_dist,
            "connected nodes should move closer; initial={}, final={}",
            initial_dist,
            final_dist
        );
    }

    #[test]
    fn test_isolated_node_stays_put_without_forces() {
        // Single isolated node — no edges, no repulsion neighbors. With center
        // gravity at zero, it should remain near its starting position.
        let input = LayoutInput {
            nodes: vec![make_node(1000.0, 1000.0)],
            edges: vec![],
            ticks: 10,
            alpha: 1.0,
            width: 800.0,
            height: 600.0,
        };
        let output = run_ticks(&input);
        let (x, y) = output.positions[0];
        assert!(
            (x - 1000.0).abs() < 1.0,
            "isolated node should stay near start; final x={}",
            x
        );
        assert!(
            (y - 1000.0).abs() < 1.0,
            "isolated node should stay near start; final y={}",
            y
        );
    }

    #[test]
    fn test_simulation_tick_alpha_decays() {
        let mut nodes = vec![make_node(0.0, 0.0), make_node(100.0, 0.0)];
        let mut vel = make_vel(2);
        let edges = vec![];
        let mut alpha = 1.0;
        simulation_tick(&mut nodes, &mut vel, &edges, &mut alpha);
        assert!(
            (alpha - ALPHA_DECAY).abs() < 1e-10,
            "alpha should decay by ALPHA_DECAY per tick, expected {}, got {}",
            ALPHA_DECAY,
            alpha
        );
    }

    #[test]
    fn test_run_ticks_positions_length_matches_input() {
        let input = LayoutInput {
            nodes: vec![
                make_node(0.0, 0.0),
                make_node(100.0, 0.0),
                make_node(50.0, 100.0),
            ],
            edges: vec![(0, 1), (1, 2)],
            ticks: 10,
            alpha: 1.0,
            width: 800.0,
            height: 600.0,
        };
        let output = run_ticks(&input);
        assert_eq!(
            output.positions.len(),
            3,
            "output positions should match input node count"
        );
    }

    #[test]
    fn test_repulsion_moves_close_nodes_apart() {
        // Two nodes very close — pure repulsion should push them apart.
        let input = LayoutInput {
            nodes: vec![make_node(0.0, 0.0), make_node(1.0, 0.0)],
            edges: vec![],
            ticks: 5,
            alpha: 1.0,
            width: 800.0,
            height: 600.0,
        };
        let output = run_ticks(&input);
        let (x0, _) = output.positions[0];
        let (x1, _) = output.positions[1];
        let final_dist = (x1 - x0).abs();
        assert!(
            final_dist > 1.0,
            "repulsion should push close nodes apart; final dist={}",
            final_dist
        );
    }

    #[test]
    fn test_collision_force_separates_overlapping_nodes() {
        // Two nodes with radius 8.0 at the same position — collision should push them apart.
        let mut nodes = vec![
            NodeData {
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                mass: 1.0,
                pinned: false,
                radius: 8.0,
                bfs_depth: 0,
            },
            NodeData {
                x: 5.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                mass: 1.0,
                pinned: false,
                radius: 8.0,
                bfs_depth: 0,
            },
        ];
        let mut vel = make_vel(2);
        let edges = vec![];
        let mut alpha = 1.0;
        simulation_tick(&mut nodes, &mut vel, &edges, &mut alpha);
        // Nodes should have moved apart (distance > initial 5.0).
        let final_dist = (nodes[1].x - nodes[0].x).abs();
        assert!(
            final_dist > 5.0,
            "overlapping nodes should separate; initial=5.0, final={}",
            final_dist
        );
    }
}
