# /rust-check

Run the full Rust quality check suite for Research Synergy.

## Instructions

Run these three commands in sequence and report results:

1. `cargo fmt --check` — check formatting
2. `cargo clippy -- -D warnings` — lint with warnings as errors
3. `cargo check` — type-check the project

For each step, report:
- **Pass** if it succeeds with no output
- **Warnings/Errors** with a concise summary of what needs fixing

If `cargo fmt --check` fails, suggest running `cargo fmt` to fix. If clippy or check fails, list the specific issues.

Stop at the first failure — no need to run later steps if an earlier one fails.
