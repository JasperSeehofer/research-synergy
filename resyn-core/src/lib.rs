// Always available (WASM-safe)
pub mod data_processing;
pub mod datamodels;
pub mod error;
pub mod gap_analysis;
pub mod nlp; // DEBT-01: was missing from old lib.rs
pub mod utils;
pub mod validation; // analysis submodules (similarity, contradiction, abc_bridge) always available;
// output submodule gated behind ssr — see gap_analysis/mod.rs

// Server-only (behind ssr feature)
#[cfg(feature = "ssr")]
pub mod data_aggregation;
#[cfg(feature = "ssr")]
pub mod database;
#[cfg(feature = "ssr")]
pub mod llm;

// Re-export petgraph types so downstream crates use these instead of depending on petgraph directly
pub use petgraph;
