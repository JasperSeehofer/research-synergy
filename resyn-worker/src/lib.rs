use futures::{SinkExt, StreamExt};
use gloo_worker::reactor::{reactor, ReactorScope};
use serde::{Deserialize, Serialize};

pub mod barnes_hut;
pub mod forces;

// Re-export for testing and external use.
pub use barnes_hut::{barnes_hut_repulsion, QuadTree};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub mass: f64,
    pub pinned: bool,
    pub radius: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutInput {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<(usize, usize)>,
    pub ticks: u32,
    pub alpha: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutOutput {
    pub positions: Vec<(f64, f64)>,
    pub velocities: Vec<(f64, f64)>,
    pub alpha: f64,
    pub converged: bool,
}

/// gloo-worker reactor — receives LayoutInput messages and responds with LayoutOutput.
/// This function runs inside a Web Worker thread, keeping the UI responsive.
#[reactor]
pub async fn ForceLayoutWorker(mut scope: ReactorScope<LayoutInput, LayoutOutput>) {
    while let Some(input) = scope.next().await {
        let output = forces::run_ticks(&input);
        let _ = scope.send(output).await;
    }
}
