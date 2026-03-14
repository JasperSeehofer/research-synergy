# External Integrations

**Analysis Date:** 2026-03-14

## APIs & External Services

**Academic Data Sources:**
- **arXiv REST API** - Fetches paper metadata and references
  - SDK/Client: `arxiv-rs 0.1.5` crate
  - Implementation: `src/data_aggregation/arxiv_api.rs` with `get_papers()` and `get_paper_by_id()` functions
  - Query builder: `ArxivQueryBuilder` from arxiv-rs
  - Rate limiting: Enforced via `ArxivHTMLDownloader` at 3 seconds per request (configurable)
  - Also retrieves bibliography HTML from arXiv HTML pages via direct HTTP requests

- **InspireHEP REST API** - Alternative academic data source for High Energy Physics
  - Base URL: `https://inspirehep.net/api`
  - Implementation: `src/data_aggregation/inspirehep_api.rs` with `InspireHepClient` struct
  - Endpoint: `/literature?q=arxiv:{id}&fields=references,titles,authors,abstracts,arxiv_eprints,dois,citation_count`
  - SDK/Client: Implemented directly using `reqwest::Client`
  - Auth: None required (public API)
  - Rate limiting: 350ms hardcoded between requests (enforced via `rate_limit_check()` method)
  - Returns: JSON response with paper metadata and references in single call (no HTML scraping needed)
  - Conversion logic: `convert_hit_to_paper()` and `convert_references()` methods handle deserialization

**HTML Scraping:**
- arXiv HTML pages (https://arxiv.org/abs/{id})
  - Implementation: `src/data_aggregation/html_parser.rs` with `ArxivHTMLDownloader` struct
  - Parser: `scraper` crate with CSS selectors
  - Target: `<span class="ltx_bibblock">` elements for bibliography
  - Rate limiting: Default 3 seconds between requests (configurable via `with_rate_limit()`)

## Data Storage

**Databases:**
- **SurrealDB 3.x** - Embedded document-graph database
  - Connection: Managed in `src/database/client.rs`
    - In-memory: `mem://` (used in tests)
    - Local file-based: `surrealkv://./data` (persists to local directory)
    - Any URL: `surrealdb::engine::any::connect(endpoint)` supports any SurrealDB endpoint
  - Client: Surreal<Any> type alias defined in `src/database/client.rs`
  - Namespace: "resyn"
  - Database: "resyn"
  - Schema: Auto-initialized on connection via `init_schema()` in `src/database/schema.rs`
  - Tables:
    - `paper` table: Stores papers with arXiv ID as record key (`paper:⟨arxiv_id⟩`)
    - `cites` relation: Citation edges between papers
    - Indexes: Created for efficient citation queries
  - Repositories: `src/database/queries.rs` with `PaperRepository` struct for upsert, get, and graph traversal operations

**File Storage:**
- Local filesystem only (no cloud storage integration)
  - SurrealKV files stored in `--db surrealkv://./path` directory
  - No object storage or CDN integration

**Caching:**
- None implemented (graph built fresh in memory each run or loaded from DB)

## Authentication & Identity

**Auth Provider:**
- None required - All external APIs are public/unauthenticated
  - arXiv: Public API, no authentication
  - InspireHEP: Public API, no authentication

## Monitoring & Observability

**Error Tracking:**
- None (no external error tracking service)
- Custom error enum: `ResynError` in `src/error.rs` with variants:
  - ArxivApi, HtmlDownload, HttpRequest, PaperNotFound, InvalidPaperId, NoArxivLink, InspireHepApi, Database

**Logs:**
- Structured logging via `tracing` crate
- Subscriber: `tracing-subscriber` (fmt style)
- Initialization: `tracing_subscriber::fmt::init()` in `src/main.rs`
- Log levels: info!, warn!, error!, debug! used throughout
- No external log aggregation service

## CI/CD & Deployment

**Hosting:**
- Not configured for production hosting
- Application: Desktop GUI application meant for local execution

**CI Pipeline:**
- GitHub Actions (`.github/workflows/rust.yml`)
  - Triggers: On push to main, on pull requests to main
- Jobs:
  - **check**: Rust formatting (`cargo fmt --all -- --check`) and linting (`cargo clippy --all-targets --all-features` with `-Dwarnings`)
  - **test**: `cargo test --verbose` (runs all 44 tests)
  - **coverage**: Tarpaulin coverage report uploaded to Codecov (fail_ci_if_error: false)
  - **build**: Release binary build (`cargo build --verbose`)
- System dependencies: Installed for GUI support (libxkbcommon-dev, libgtk-3-dev, etc.)
- Caching: Rust cache via Swatinem/rust-cache@v2

**Deployment:**
- Not configured for automatic deployment
- Manual binary distribution as standalone executable

## Environment Configuration

**Required env vars:**
- None - All configuration via CLI arguments or hardcoded defaults
- Optional database location: Passed via `--db` CLI argument

**Secrets location:**
- None - No secrets required for public API access
- No .env files needed

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None - One-way data ingestion from arXiv and InspireHEP

## HTTP Client Configuration

**Shared HTTP Client:**
- Created in `src/utils.rs:create_http_client()`
- Timeout: 30 seconds globally
- Used for:
  - arXiv API requests via arxiv-rs
  - arXiv HTML page downloads for bibliography parsing
  - InspireHEP API requests
- Reused across all requests to avoid connection pool overhead

## Rate Limiting

**arXiv Source:**
- HTML downloads: Default 3 seconds per request (configurable via `--rate-limit-secs` CLI arg)
- Enforced via `ArxivHTMLDownloader::rate_limit_check()` using `tokio::time::sleep`
- Prevents request blocking from arXiv servers

**InspireHEP Source:**
- API requests: 350ms hardcoded between calls
- Enforced via `InspireHepClient::rate_limit_check()` using `tokio::time::sleep`
- No external rate limit configuration available

## Data Flow Architecture

**Paper Source Abstraction:**
- Trait: `PaperSource` in `src/data_aggregation/traits.rs`
- Async trait with methods: `fetch_paper()`, `fetch_references()`, `source_name()`
- Implementations:
  - `ArxivSource` wraps `ArxivHTMLDownloader` (two HTTP calls per paper: metadata + HTML bibliography)
  - `InspireHepClient` (single HTTP call per paper: metadata + references in JSON)
- BFS Crawler: `recursive_paper_search_by_references()` in `src/data_aggregation/arxiv_utils.rs` accepts `&mut dyn PaperSource`

**Graph Construction:**
- Directed graph: `petgraph::StableGraph<Paper, f32, Directed>`
- Created from `Vec<Paper>` in `src/data_processing/graph_creation.rs`
- Edge weights: Float values (currently unused)
- Only arXiv-to-arXiv citations are followed; other references stored but not crawled

**Persistence Flow:**
- Optional: If `--db` specified, papers persisted to SurrealDB after crawl
- Load flow: `--db-only` skips crawling, loads papers and edges from database via `PaperRepository::get_citation_graph()`

---

*Integration audit: 2026-03-14*
