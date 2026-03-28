/// Public library interface for resyn-server, primarily used by integration tests.
///
/// The binary (`main.rs`) exposes CLI commands; this lib re-exports the
/// `commands` module so integration tests in `tests/` can call public functions
/// such as `run_analysis_pipeline` directly.
pub mod commands;
