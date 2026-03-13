# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Research Synergy (ReSyn) is a Rust application for Literature Based Discovery (LBD). It aggregates academic papers from arXiv, constructs knowledge graphs from citation relationships, and visualizes them as interactive force-directed graphs.

## Build & Run Commands

```bash
cargo build                  # Debug build
cargo build --release        # Release build
cargo run                    # Run the application (default seed paper)
cargo run -- --paper-id 2301.12345 --max-depth 2  # Custom seed & depth
cargo test                   # Run all tests
cargo test <test_name>       # Run a specific test
cargo test -- --nocapture    # Run tests with stdout visible
cargo check                  # Type-check without building
cargo fmt --all -- --check   # Check formatting
cargo clippy --all-targets --all-features  # Lint (CI runs with -Dwarnings)
```

## Architecture

The application pipeline: fetch a seed paper from arXiv → recursively aggregate referenced papers → build a directed graph → launch interactive visualization.

Six main layers in `src/`:

- **`data_aggregation/`** — arXiv API queries (`arxiv_api.rs`), recursive reference crawling with deduplication (`arxiv_utils.rs`), HTML scraping for reference metadata (`html_parser.rs`), and a builder-pattern search query constructor (`search_query_handler.rs`). `ArxivHTMLDownloader` enforces configurable rate limiting with `tokio::time::sleep`.

- **`datamodels/`** — Core data structures: `Paper`, `Reference`, `Link`, `Journal`. All derive `Serialize`, `Deserialize`, `Clone`, `Debug`, `Default`.

- **`data_processing/`** — `graph_creation.rs` converts `Vec<Paper>` into a `petgraph::StableGraph<Paper, f32, Directed>`, creating edges from citation relationships. Uses `strip_version_suffix()` from `utils.rs` for dedup.

- **`visualization/`** — `force_graph_app.rs` is the main GUI using eframe/egui with Fruchterman-Reingold layout (fdg crate). Supports pan/zoom, FPS tracking, and simulation controls. `settings.rs` holds configurable parameters.

- **`error.rs`** — `ResynError` enum with variants: `ArxivApi`, `HtmlDownload`, `HttpRequest`, `PaperNotFound`, `InvalidPaperId`, `NoArxivLink`. Implements `Display`, `Error`, `From<reqwest::Error>`.

- **`validation.rs`** — arXiv paper ID validation (new format `YYMM.NNNNN` and old format `category/NNNNNNN`, both with optional version suffix). URL validation.

- **`utils.rs`** — Shared utilities: `strip_version_suffix()` for paper ID normalization, `create_http_client()` with 30-second timeout.

## Key Dependencies

- **tokio** — async runtime (single `#[tokio::main]` in `main.rs`)
- **arxiv-rs** — arXiv API client
- **petgraph** — graph data structures
- **egui/eframe** — immediate-mode GUI
- **fdg** (git dep) — force-directed graph layout
- **reqwest** — async HTTP client (shared client with timeouts)
- **scraper** — HTML parsing with CSS selectors
- **clap** — CLI argument parsing
- **tracing/tracing-subscriber** — structured logging
- **serde/serde_json** — serialization for all data models
- **wiremock** (dev) — HTTP mocking for integration tests

## Data Flow Details

- Each paper requires **two HTTP requests**: one to arXiv API (metadata) and one to the arXiv HTML page (bibliography references). Both go through `ArxivHTMLDownloader` rate limiting.
- Reference extraction parses `<span class="ltx_bibblock">` elements from arXiv HTML. Titles are extracted from `<em>` tags when present, falling back to comma-splitting.
- `aggregate_references_for_arxiv_paper()` is async, takes `&mut ArxivHTMLDownloader` for rate-limited HTML downloads.
- The crawler (`recursive_paper_search_by_references`) does BFS: all papers at depth N are processed before depth N+1. It is async and takes a `&mut ArxivHTMLDownloader`.
- Only arXiv-to-arXiv citation edges are followed. References to Nature/PhysRev/other journals are stored but not crawled.

## Important Notes

- arXiv rate limiting: `ArxivHTMLDownloader` enforces configurable delays (default 3s) between requests using `tokio::time::sleep`. Violating this causes request blocks.
- Paper IDs may have version suffixes (e.g., "2301.12345v2") — these are stripped via `utils::strip_version_suffix()` during both crawl dedup and graph construction.
- Rust edition 2024, stable toolchain (pinned via `rust-toolchain.toml`).
- Single async runtime: `main.rs` has the only `#[tokio::main]`. All API/HTML functions are `async fn`.
- Error handling uses `ResynError` with `?` propagation. The crawler logs warnings for individual failures and continues.
- Seed paper ID is configurable via `--paper-id` CLI arg (default: `2503.18887`).
- CI runs fmt check, clippy with `-Dwarnings`, tests, and coverage via tarpaulin.

## Testing

- **53 tests total**: 25 unit tests (paper, graph_creation, arxiv_utils, search_query_handler, validation, utils) + 3 integration tests (wiremock-based HTML parsing, reference extraction, rate limiter)
- Integration tests use `wiremock` for HTTP mocking — no real arXiv calls
- `ArxivHTMLDownloader::with_rate_limit(Duration::from_millis(0))` disables rate limiting in tests
