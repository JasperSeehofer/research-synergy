# Changelog

## [Unreleased] - Phase 0: Foundation

### Added
- `src/error.rs` — `ResynError` enum with typed error variants replacing panics and `unwrap_or_default()`
- `src/utils.rs` — shared `strip_version_suffix()` and `create_http_client()` (30s timeout)
- `src/validation.rs` — arXiv paper ID validation (new + old formats) and URL validation
- CLI arguments via `clap`: `--paper-id`, `--max-depth`, `--rate-limit-secs`
- Structured logging via `tracing`/`tracing-subscriber` (replaces all `println!`)
- `Serialize`/`Deserialize`/`Clone`/`Debug`/`Default` derives on all data models
- `Link` fields now `pub` (needed for serialization and future DB integration)
- 25 unit tests across paper.rs, graph_creation.rs, arxiv_utils.rs, search_query_handler.rs, validation.rs, utils.rs
- 3 integration tests using `wiremock` for HTML parsing, reference extraction, and rate limiter
- Enhanced CI: fmt check, clippy with `-Dwarnings`, `rust-cache`, `cargo-tarpaulin` coverage
- `rust-toolchain.toml` pinning stable Rust
- `.claude/commands/pre-commit.md` skill for pre-commit review checklist

### Fixed
- **Panic on malformed paper IDs**: `Paper::from_arxiv_paper` and `Reference::get_arxiv_id` now return `Result` instead of `.unwrap()`
- **Buggy version stripping in graph_creation.rs**: replaced `split("v").next().unwrap()` with shared `strip_version_suffix()`
- **Rate limiter bypass**: `get_paper_by_id()` in the crawler now goes through `ArxivHTMLDownloader` rate limiting
- **Multiple tokio runtimes**: removed `#[tokio::main]` from `arxiv_api.rs` and `html_parser.rs`, single runtime in `main.rs`
- **Blocking sleep in async context**: `ArxivHTMLDownloader` now uses `tokio::time::sleep`

### Removed
- `src/datamodels/graph.rs` — unused legacy graph implementation
- `src/visualization/graph_app.rs` — unused legacy visualization
- `src/data_aggregation/pdf_parser.rs` — unused PDF parser
- `lopdf` dependency
