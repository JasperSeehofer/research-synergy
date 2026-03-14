# Coding Conventions

## Code Style

- **Rust Edition 2024** with stable toolchain (pinned via `rust-toolchain.toml`)
- Enforced by CI: `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -Dwarnings`
- Uses `let` chains (e.g., `if let Some(element) = ... && let Some(child) = ...`) — an Edition 2024 feature

## Naming Patterns

- **Types:** PascalCase — `Paper`, `PaperSource`, `ResynError`, `PaperRepository`, `ArxivHTMLDownloader`
- **Functions:** snake_case — `fetch_paper`, `strip_version_suffix`, `aggregate_references_for_arxiv_paper`
- **Fields:** snake_case — `pdf_url`, `last_updated`, `citation_count`, `arxiv_eprint`
- **Test functions:** `test_` prefix — `test_upsert_and_get_paper`, `test_valid_arxiv_ids`
- **Modules:** snake_case matching domain — `data_aggregation`, `graph_creation`, `html_parser`

## Error Handling

- Central `ResynError` enum in `src/error.rs` with domain-specific variants:
  `ArxivApi`, `HtmlDownload`, `HttpRequest`, `PaperNotFound`, `InvalidPaperId`, `NoArxivLink`, `InspireHepApi`, `Database`
- Implements `Display`, `Error`, and `From<reqwest::Error>` for `?` propagation
- Functions return `Result<T, ResynError>` — uses `?` operator throughout
- Error context added via `map_err` with descriptive messages:
  ```rust
  .map_err(|e| ResynError::Database(format!("upsert paper failed: {e}")))?;
  ```
- BFS crawler logs warnings for individual failures and continues:
  ```rust
  Err(e) => { warn!(paper_id, error = %e, "Failed to fetch paper"); continue; }
  ```

## Logging

- Uses `tracing` crate with `tracing-subscriber` (initialized once in `main.rs`)
- Structured logging with field syntax:
  ```rust
  info!(depth, count = referenced_paper_ids.len(), "Processing depth level");
  warn!(paper_id, error = %e, "Failed to fetch paper");
  ```
- Log levels: `info` for progress, `warn` for recoverable failures, `error` for fatal issues, `debug` for skip reasons

## Async Patterns

- Single `#[tokio::main]` in `main.rs` — all async flows through this runtime
- Async trait support via `async-trait` crate on `PaperSource`
- Rate limiting via `tokio::time::sleep` with configurable `Duration`
- Rate limiting is disableable for tests: `.with_rate_limit(Duration::from_millis(0))`

## Data Model Patterns

- `Paper` is the central type — used across all layers
- `#[derive(Default)]` on data models for easy construction in tests
- Extensive use of `Clone` — data models are cloned between layers
- `serde::Serialize`/`Deserialize` on all data models for JSON and DB serialization
- Optional fields use `Option<T>`: `doi`, `inspire_id`, `citation_count`, `comment`

## Builder/Configuration Pattern

- Rate-limited clients use builder pattern: `ArxivHTMLDownloader::new(client).with_rate_limit(duration)`
- `InspireHepClient::new(client).with_base_url(url).with_rate_limit(duration)` — chainable configuration
- `SearchQueryHandler` uses builder pattern for query construction

## Module Organization

- Each domain layer is a directory module with `mod.rs`
- Public API exposed through `lib.rs` for integration test access
- `main.rs` uses private `mod` declarations with `#[allow(dead_code)]`
- Unit tests live inline in `#[cfg(test)] mod tests` blocks within each source file
- Integration tests in `tests/` directory import through `lib.rs`

## ID Normalization

- Paper IDs consistently stripped of version suffixes via `utils::strip_version_suffix()`
- Applied at: BFS dedup, graph construction, DB upserts, DB record IDs
- DB records keyed as `paper:⟨stripped_arxiv_id⟩`
