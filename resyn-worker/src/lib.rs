use serde::{Deserialize, Serialize};

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

// Barnes-Hut force layout implementation — Plan 02.
// This crate compiles as cdylib for wasm32-unknown-unknown.
