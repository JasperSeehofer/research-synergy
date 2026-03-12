# /test

Run tests for Research Synergy.

## Instructions

Run `cargo test` with the provided arguments. If the user passes a filter (e.g., `/test search_query`), run `cargo test <filter> -- --nocapture`. If no argument is given, run `cargo test`.

Report results concisely:
- Number of tests passed/failed/ignored
- For failures: show the test name and the key assertion or panic message
- Skip the full compilation output — only show test results
