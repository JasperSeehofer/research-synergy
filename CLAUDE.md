# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Research Synergy (ReSyn) is a Rust application for Literature Based Discovery (LBD). It aggregates academic papers from arXiv and InspireHEP, persists them to SurrealDB, constructs knowledge graphs from citation relationships, and visualizes them as interactive force-directed graphs.

## Build & Run Commands

```bash
cargo build                  # Debug build
cargo build --release        # Release build
cargo run                    # Run the application (default seed paper, arXiv source)
cargo run -- --paper-id 2301.12345 --max-depth 2  # Custom seed & depth
cargo run -- --source inspirehep --paper-id 2301.12345  # Use InspireHEP source
cargo run -- --db surrealkv://./data --paper-id 2301.12345  # Persist to local DB
cargo run -- --db surrealkv://./data --db-only --paper-id 2301.12345  # Load from DB only
cargo test                   # Run all tests
cargo test <test_name>       # Run a specific test
cargo test -- --nocapture    # Run tests with stdout visible
cargo check                  # Type-check without building
cargo fmt --all -- --check   # Check formatting
cargo clippy --all-targets --all-features  # Lint (CI runs with -Dwarnings)
```

## Architecture

The application pipeline: select data source → fetch seed paper → BFS crawl references → optionally persist to SurrealDB → build a directed graph → launch interactive visualization.

Eight main layers in `src/`:

- **`data_aggregation/`** — Data source abstraction and implementations:
  - `traits.rs` — `PaperSource` async trait (fetch_paper, fetch_references, source_name)
  - `arxiv_source.rs` — `ArxivSource` implementing `PaperSource`, wraps existing arXiv code
  - `inspirehep_api.rs` — `InspireHepClient` implementing `PaperSource`, InspireHEP REST API with rate limiting (350ms)
  - `arxiv_api.rs` — arXiv API queries via arxiv-rs
  - `arxiv_utils.rs` — BFS crawler (`recursive_paper_search_by_references` takes `&mut dyn PaperSource`), HTML reference parsing
  - `html_parser.rs` — `ArxivHTMLDownloader` with configurable rate limiting
  - `search_query_handler.rs` — builder-pattern search query constructor

- **`database/`** — SurrealDB persistence layer:
  - `client.rs` — Connection management (`connect`, `connect_memory`, `connect_local`)
  - `schema.rs` — Schema definitions (paper table, cites relation, indexes)
  - `queries.rs` — `PaperRepository` with upsert, get, citation graph traversal

- **`datamodels/`** — Core data structures: `Paper`, `Reference`, `Link`, `Journal`, `DataSource`. Paper includes optional `doi`, `inspire_id`, `citation_count`, `source` fields.

- **`data_processing/`** — `graph_creation.rs` converts `Vec<Paper>` into a `petgraph::StableGraph<Paper, f32, Directed>`, creating edges from citation relationships. Uses `strip_version_suffix()` from `utils.rs` for dedup.

- **`visualization/`** — `force_graph_app.rs` is the main GUI using eframe/egui with Fruchterman-Reingold layout (fdg crate). Supports pan/zoom, FPS tracking, and simulation controls. `settings.rs` holds configurable parameters.

- **`error.rs`** — `ResynError` enum with variants: `ArxivApi`, `HtmlDownload`, `HttpRequest`, `PaperNotFound`, `InvalidPaperId`, `NoArxivLink`, `InspireHepApi`, `Database`. Implements `Display`, `Error`, `From<reqwest::Error>`.

- **`validation.rs`** — arXiv paper ID validation (new format `YYMM.NNNNN` and old format `category/NNNNNNN`, both with optional version suffix). URL validation.

- **`utils.rs`** — Shared utilities: `strip_version_suffix()` for paper ID normalization, `create_http_client()` with 30-second timeout.

## Key Dependencies

- **tokio** — async runtime (single `#[tokio::main]` in `main.rs`)
- **arxiv-rs** — arXiv API client
- **surrealdb** v3 — embedded document-graph database (kv-mem for tests, kv-surrealkv for local persistence)
- **async-trait** — async trait support for `PaperSource`
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

- **arXiv source**: Each paper requires two HTTP requests — one to arXiv API (metadata) and one to the arXiv HTML page (bibliography references). Both go through `ArxivHTMLDownloader` rate limiting (default 3s).
- **InspireHEP source**: Single API call per paper returns metadata + references with direct arXiv eprint IDs (no HTML scraping needed). Rate limit: 350ms between requests.
- Reference extraction from arXiv HTML parses `<span class="ltx_bibblock">` elements. Titles extracted from `<em>` tags when present, falling back to comma-splitting.
- The BFS crawler (`recursive_paper_search_by_references`) accepts `&mut dyn PaperSource`, enabling source-agnostic crawling.
- Only arXiv-to-arXiv citation edges are followed. References to Nature/PhysRev/other journals are stored but not crawled.
- SurrealDB persistence: papers stored as `paper:⟨arxiv_id⟩` records, citations as `cites` relation edges. Schema is auto-initialized on connection.

## CLI Arguments

| Argument | Default | Description |
|---|---|---|
| `--paper-id` / `-p` | `2503.18887` | arXiv paper ID seed |
| `--max-depth` / `-d` | `3` | BFS crawl depth |
| `--rate-limit-secs` / `-r` | `3` | Rate limit between requests (seconds) |
| `--source` | `arxiv` | Data source: `arxiv` or `inspirehep` |
| `--db` | none | DB connection string (e.g. `mem://`, `surrealkv://./data`) |
| `--db-only` | false | Skip crawling, load from DB only (requires `--db`) |

## Important Notes

- arXiv rate limiting: `ArxivHTMLDownloader` enforces configurable delays (default 3s) between requests using `tokio::time::sleep`. Violating this causes request blocks.
- InspireHEP rate limiting: `InspireHepClient` enforces 350ms between requests.
- Paper IDs may have version suffixes (e.g., "2301.12345v2") — these are stripped via `utils::strip_version_suffix()` during crawl dedup, graph construction, and DB upserts.
- Rust edition 2024, stable toolchain (pinned via `rust-toolchain.toml`).
- Single async runtime: `main.rs` has the only `#[tokio::main]`. All API/HTML functions are `async fn`.
- Error handling uses `ResynError` with `?` propagation. The crawler logs warnings for individual failures and continues.
- SurrealDB `kv-mem` feature compiles as a Rust dependency — no external server needed for tests.
- CI runs fmt check, clippy with `-Dwarnings`, tests, and coverage via tarpaulin.

## Testing

- **44 tests total**: 30 unit tests + 8 integration tests + 6 database tests
  - Unit: paper, graph_creation, arxiv_utils, search_query_handler, validation, utils, inspirehep_api deserialization/conversion
  - Integration: wiremock-based arXiv HTML parsing (3), InspireHEP API mocking (5)
  - Database: SurrealDB in-memory (upsert, idempotent, exists, version dedup, citations, graph traversal)
- All DB tests use `connect_memory()` — no external DB required
- `ArxivHTMLDownloader::with_rate_limit(Duration::from_millis(0))` and `InspireHepClient::with_rate_limit(Duration::from_millis(0))` disable rate limiting in tests
