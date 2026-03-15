// Minimal WASM compilation verification stub.
// Importing from resyn-core confirms the WASM boundary compiles cleanly.
use resyn_core::datamodels::paper::Paper;

pub fn app_version() -> &'static str {
    "0.1.0"
}

// Ensure Paper is recognized as used for the WASM boundary check.
pub fn get_paper_id(paper: &Paper) -> &str {
    &paper.id
}
