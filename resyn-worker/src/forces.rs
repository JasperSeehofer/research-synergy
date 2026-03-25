//! Force simulation using Barnes-Hut for repulsion.

use crate::{barnes_hut, LayoutInput, LayoutOutput, NodeData};

pub const REPULSION_STRENGTH: f64 = -1500.0;
pub const ATTRACTION_STRENGTH: f64 = 0.06;
pub const CENTER_GRAVITY: f64 = 0.005;
pub const ALPHA_DECAY: f64 = 0.9945;
pub const ALPHA_MIN: f64 = 0.001;
pub const THETA: f64 = 0.8;
pub const IDEAL_DISTANCE: f64 = 120.0;
pub const VELOCITY_DAMPING: f64 = 0.85;

/// Run one tick of the force simulation.
///
/// `nodes` holds position/mass/pinned state and is mutated in place.
/// Velocities are maintained via a parallel `vel` slice.
/// Returns `true` if the simulation has converged (alpha < ALPHA_MIN).
pub fn simulation_tick(nodes: &mut [NodeData], vel: &mut [(f64, f64)], edges: &[(usize, usize)], alpha: &mut f64) -> bool {
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
        let (fx, fy) = barnes_hut::barnes_hut_repulsion(&tree, nodes[i].x, nodes[i].y, nodes[i].mass, THETA, REPULSION_STRENGTH);
        forces[i].0 += fx;
        forces[i].1 += fy;
    }

    // 2. Attractive forces (Hooke's law along edges).
    for &(a, b) in edges {
        if a >= n || b >= n {
            continue;
        }
        let dx = nodes[b].x - nodes[a].x;
        let dy = nodes[b].y - nodes[a].y;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
        let stretch = dist - IDEAL_DISTANCE;
        let force_mag = ATTRACTION_STRENGTH * stretch;
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

    // 3. Center gravity — pull toward (0, 0).
    for i in 0..n {
        if nodes[i].pinned {
            continue;
        }
        forces[i].0 -= CENTER_GRAVITY * nodes[i].x;
        forces[i].1 -= CENTER_GRAVITY * nodes[i].y;
    }

    // 4. Collision separation — pushes overlapping nodes apart (per D-03).
    // O(n^2) but short-circuits on non-overlapping pairs; <2ms for 400 nodes at 1 tick/frame.
    const COLLISION_PADDING: f64 = 8.0;
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
                if !nodes[i].pinned {
                    forces[i].0 -= nx;
                    forces[i].1 -= ny;
                }
                if !nodes[j].pinned {
                    forces[j].0 += nx;
                    forces[j].1 += ny;
                }
            }
        }
    }

    // Apply forces and update positions.
    let max_vel = IDEAL_DISTANCE / 2.0;
    for i in 0..n {
        if nodes[i].pinned {
            continue;
        }
        vel[i].0 += forces[i].0 * *alpha;
        vel[i].1 += forces[i].1 * *alpha;
        // Velocity clamping prevents scatter when repulsion is strong in early ticks
        vel[i].0 = vel[i].0.clamp(-max_vel, max_vel);
        vel[i].1 = vel[i].1.clamp(-max_vel, max_vel);
        nodes[i].x += vel[i].0;
        nodes[i].y += vel[i].1;
        vel[i].0 *= VELOCITY_DAMPING;
        vel[i].1 *= VELOCITY_DAMPING;
    }

    *alpha *= ALPHA_DECAY;
    *alpha < ALPHA_MIN
}

/// Run `ticks` iterations of the force simulation and return positions + convergence flag.
pub fn run_ticks(input: &LayoutInput) -> LayoutOutput {
    let mut nodes: Vec<NodeData> = input.nodes.clone();
    let mut vel: Vec<(f64, f64)> = input
        .nodes
        .iter()
        .map(|nd| (nd.vx, nd.vy))
        .collect();
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
    LayoutOutput { positions, velocities, alpha, converged }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeData;

    fn make_node(x: f64, y: f64) -> NodeData {
        NodeData { x, y, vx: 0.0, vy: 0.0, mass: 1.0, pinned: false, radius: 8.0 }
    }

    fn make_vel(n: usize) -> Vec<(f64, f64)> {
        vec![(0.0, 0.0); n]
    }

    #[test]
    fn test_simulation_tick_pinned_nodes_no_movement() {
        let mut nodes = vec![
            NodeData { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, mass: 1.0, pinned: true, radius: 8.0 },
            NodeData { x: 50.0, y: 50.0, vx: 0.0, vy: 0.0, mass: 1.0, pinned: true, radius: 8.0 },
        ];
        let mut vel = make_vel(2);
        let edges = vec![(0, 1)];
        let mut alpha = 1.0;
        simulation_tick(&mut nodes, &mut vel, &edges, &mut alpha);
        assert!((nodes[0].x - 0.0).abs() < 1e-10, "pinned node x should not change");
        assert!((nodes[0].y - 0.0).abs() < 1e-10, "pinned node y should not change");
        assert!((nodes[1].x - 50.0).abs() < 1e-10, "pinned node x should not change");
        assert!((nodes[1].y - 50.0).abs() < 1e-10, "pinned node y should not change");
    }

    #[test]
    fn test_convergence_returns_true_when_alpha_below_threshold() {
        let mut nodes = vec![make_node(0.0, 0.0), make_node(10.0, 0.0)];
        let mut vel = make_vel(2);
        let edges = vec![];
        let mut alpha = ALPHA_MIN * 0.5; // Already below threshold.
        let converged = simulation_tick(&mut nodes, &mut vel, &edges, &mut alpha);
        assert!(converged, "should report converged when alpha is below ALPHA_MIN");
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
        assert!(output.converged, "100-node graph should converge within 5000 ticks");
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
    fn test_center_gravity_pulls_isolated_node_toward_origin() {
        // Single isolated node far from origin — center gravity should pull it in.
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
            x < 1000.0,
            "node should be pulled toward origin in x; final x={}",
            x
        );
        assert!(
            y < 1000.0,
            "node should be pulled toward origin in y; final y={}",
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
            nodes: vec![make_node(0.0, 0.0), make_node(100.0, 0.0), make_node(50.0, 100.0)],
            edges: vec![(0, 1), (1, 2)],
            ticks: 10,
            alpha: 1.0,
            width: 800.0,
            height: 600.0,
        };
        let output = run_ticks(&input);
        assert_eq!(output.positions.len(), 3, "output positions should match input node count");
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
        assert!(final_dist > 1.0, "repulsion should push close nodes apart; final dist={}", final_dist);
    }

    #[test]
    fn test_collision_force_separates_overlapping_nodes() {
        // Two nodes with radius 8.0 at the same position — collision should push them apart.
        let mut nodes = vec![
            NodeData { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, mass: 1.0, pinned: false, radius: 8.0 },
            NodeData { x: 5.0, y: 0.0, vx: 0.0, vy: 0.0, mass: 1.0, pinned: false, radius: 8.0 },
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
