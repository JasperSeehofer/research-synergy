use serde::{Deserialize, Serialize};

pub mod barnes_hut;
pub mod forces;

// Re-export for testing and external use.
pub use barnes_hut::{barnes_hut_repulsion, QuadTree};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub x: f64,
    pub y: f64,
    pub mass: f64,
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutInput {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<(usize, usize)>,
    pub ticks: u32,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutOutput {
    pub positions: Vec<(f64, f64)>,
    pub converged: bool,
}
