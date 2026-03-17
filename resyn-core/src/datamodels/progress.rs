use serde::{Deserialize, Serialize};

/// A progress event broadcast to SSE clients.
/// WASM-safe: only depends on serde, no server-only crates.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProgressEvent {
    pub event_type: String,
    pub papers_found: u64,
    pub papers_pending: u64,
    pub papers_failed: u64,
    pub current_depth: usize,
    pub max_depth: usize,
    pub elapsed_secs: f64,
    pub current_paper_id: Option<String>,
    pub current_paper_title: Option<String>,
}
