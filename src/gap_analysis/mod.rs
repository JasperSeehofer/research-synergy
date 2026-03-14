pub mod abc_bridge;
pub mod contradiction;
pub mod similarity;

pub use contradiction::find_contradictions;
pub use similarity::{cosine_similarity, shared_high_weight_terms};
