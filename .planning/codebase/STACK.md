# Technology Stack

**Analysis Date:** 2026-03-14

## Languages

**Primary:**
- Rust (stable edition 2024) - Core application logic, all async operations, data aggregation, graph processing, visualization

## Runtime

**Environment:**
- Tokio 1.44.1 - Async runtime with full features enabled. Single `#[tokio::main]` in `src/main.rs` is the only entry point

**Package Manager:**
- Cargo - Standard Rust package manager
- Lockfile: Present (Cargo.lock)

## Frameworks

**Core:**
- Tokio 1.44.1 - Async runtime for all async functions across data aggregation, database operations, and HTTP requests
- Clap 4.x - CLI argument parsing with derive macros for command-line interface in `src/main.rs`

**GUI/Visualization:**
- eframe 0.31.1 - Native windowed GUI application framework
- egui 0.31.1 - Immediate-mode UI library for interactive graph visualization
- egui_graphs 0.25.0 - Graph visualization widgets with event support

**Data & Serialization:**
- serde 1.x - Serialization/deserialization framework with derive macros
- serde_json 1.0 - JSON serialization for all data models

**Graph Processing:**
- petgraph 0.7.0 - Graph data structures with StableGraph<Paper, f32, Directed> for citation networks
  - Features: graphmap, stable_graph, matrix_graph, serde-1
- fdg (git dependency: https://github.com/grantshandy/fdg) - Force-directed graph layout algorithm (Fruchterman-Reingold) for visualization

**Testing:**
- tokio-test 0.4 - Testing utilities for async code
- wiremock 0.6 - HTTP mocking for integration tests of API endpoints

**Build/Dev:**
- Rust toolchain (stable) - Pinned via `rust-toolchain.toml`

## Key Dependencies

**Critical:**
- arxiv-rs 0.1.5 - arXiv API client library used in `src/data_aggregation/arxiv_api.rs` for fetching paper metadata
- reqwest 0.12.15 - Async HTTP client with 30-second timeout configured in `src/utils.rs:create_http_client()`. Shared across all HTTP requests to arXiv HTML pages and InspireHEP API
- scraper 0.23.1 - HTML parsing with CSS selectors for extracting bibliography from arXiv HTML pages in `src/data_aggregation/html_parser.rs`

**Database:**
- surrealdb 3.x - Embedded document-graph database with two features:
  - `kv-mem` - In-memory backend for testing (no external DB required)
  - `kv-surrealkv` - File-based persistence using SurrealKV engine for local storage
  - Connection types: `mem://` (in-memory), `surrealkv://./path` (local file-based)
  - Used in `src/database/client.rs` with namespace/database "resyn"

**Async & Concurrency:**
- async-trait 0.1 - Async trait support for `PaperSource` trait in `src/data_aggregation/traits.rs`
- futures 0.3.31 - Future utilities and combinators
- crossbeam 0.8.4 - Concurrency utilities
- tokio::time - Built-in sleep/duration for rate limiting in `ArxivHTMLDownloader` and `InspireHepClient`

**Logging & Tracing:**
- tracing 0.1 - Structured logging framework
- tracing-subscriber 0.3 - Subscriber implementation for tracing output (initialized in `src/main.rs`)

**Utilities:**
- rand 0.9.0 - Random number generation
- anyhow 1.0.97 - Flexible error handling

## Configuration

**Environment:**
- No .env files required for core operation
- Configurable via CLI arguments:
  - `--paper-id` (default: 2503.18887) - arXiv paper ID seed
  - `--max-depth` (default: 3) - BFS crawl depth for reference discovery
  - `--rate-limit-secs` (default: 3) - Rate limit between arXiv HTML requests
  - `--source` (default: arxiv) - Data source: "arxiv" or "inspirehep"
  - `--db` - Optional database connection string (e.g., "surrealkv://./data")
  - `--db-only` - Load graph from database only (requires --db)

**Build:**
- `Cargo.toml` - Project manifest with dependencies
- `rust-toolchain.toml` - Rust stable channel pinned

**Runtime Configuration:**
- HTTP client: 30-second timeout configured globally in `src/utils.rs`
- arXiv rate limiting: Default 3 seconds between requests (configurable via CLI)
- InspireHEP rate limiting: 350ms between requests (hardcoded in `src/data_aggregation/inspirehep_api.rs`)
- SurrealDB: Namespace and database "resyn" auto-initialized on connection in `src/database/client.rs`

## Platform Requirements

**Development:**
- Rust stable toolchain (2024 edition)
- System dependencies for GUI (Linux):
  - libxkbcommon-dev
  - libgtk-3-dev
  - libatk1.0-dev
  - libglib2.0-dev
  - libpango1.0-dev
  - libgdk-pixbuf2.0-dev
- See `.github/workflows/rust.yml` for full system dependency list

**Production:**
- Standalone Rust binary (no runtime dependencies beyond system GUI libs)
- Optional: Database file location for `surrealkv://` persistence

## Feature Flags

**Enabled by default:**
- petgraph: graphmap, stable_graph, matrix_graph, serde-1
- egui_graphs: events
- serde: derive
- clap: derive

**Conditional (test-only):**
- tokio: full (enables all tokio features for tests)
- wiremock: HTTP mocking only in dev-dependencies

---

*Stack analysis: 2026-03-14*
