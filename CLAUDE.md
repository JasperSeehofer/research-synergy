# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Research Synergy (ReSyn) is a Rust application for Literature Based Discovery (LBD). It aggregates academic papers from arXiv and InspireHEP, persists them to SurrealDB, constructs knowledge graphs from citation relationships, and visualizes them as interactive force-directed graphs.

## Build & Run Commands

The workspace has multiple binaries. Always use `cargo run --bin resyn` with the appropriate subcommand — bare `cargo run` is ambiguous.

```bash
cargo build                  # Debug build (all crates)
cargo build --release        # Release build
cargo test                   # Run all tests
cargo test <test_name>       # Run a specific test
cargo test -- --nocapture    # Run tests with stdout visible
cargo check                  # Type-check without building
cargo fmt --all -- --check   # Check formatting
cargo clippy --all-targets --all-features  # Lint (CI runs with -Dwarnings)
```

**Subcommands (`cargo run --bin resyn -- <subcommand> [args]`):**

```bash
# Crawl
cargo run --bin resyn -- crawl --paper-id 2503.18887 --db surrealkv://./data
cargo run --bin resyn -- crawl --paper-id 2301.12345 --max-depth 2 --db surrealkv://./data
cargo run --bin resyn -- crawl --source inspirehep --paper-id 2301.12345 --db surrealkv://./data

# Analyze (NLP + optionally LLM + gap analysis)
cargo run --bin resyn -- analyze --db surrealkv://./data
cargo run --bin resyn -- analyze --db surrealkv://./data --llm-provider claude

# Export Louvain community graph to JSON (for external tooling, e.g. Kuramoto-LBD notebook)
cargo run --bin resyn -- export-louvain-graph --db surrealkv://./data --output graph.json
cargo run --bin resyn -- export-louvain-graph --db surrealkv://./data \
    --output research_synergy_pre2015.json \
    --published-before 2014-12-31 \
    --tfidf-top-n 50

# Serve web UI
cargo run --bin resyn -- serve --db surrealkv://./data

# Bulk-ingest papers from OpenAlex REST API (Phase RS-08)
# Default filter: ML+stat.ML+NeuralNet papers hosted on arXiv (~1.5M works)
cargo run --release --bin resyn -- bulk-ingest --db surrealkv://./data-openalex
# Custom filter (ML + statistical physics boundary):
cargo run --release --bin resyn -- bulk-ingest --db surrealkv://./data-openalex \
    --filter "primary_location.source.id:S4306400194,concepts.id:C154945302|C121332964|C41008148|C2778407487"

# Frontend (separate from the backend binary)
cd resyn-app && trunk serve
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

- **`datamodels/`** — Core data structures: `Paper`, `Reference`, `Link`, `Journal`, `DataSource`. Paper includes optional `doi`, `inspire_id`, `citation_count`, `source` fields. `community_graph.rs` holds the export-only types `CommunityGraph`, `ExportedNode`, `ExportedEdge`, `LouvainParams` used by `export-louvain-graph`.

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

## CLI Subcommands

Run `cargo run --bin resyn -- <subcommand> --help` for full argument lists. Key arguments per subcommand:

**`crawl`**

| Argument | Default | Description |
|---|---|---|
| `--paper-id` / `-p` | `2503.18887` | arXiv seed paper ID |
| `--max-depth` / `-d` | `3` | BFS crawl depth |
| `--rate-limit-secs` / `-r` | `3` | Rate limit between requests (seconds) |
| `--source` | `arxiv` | Data source: `arxiv` or `inspirehep` |
| `--db` | `surrealkv://./data` | DB connection string |

**`analyze`**

| Argument | Default | Description |
|---|---|---|
| `--db` | `surrealkv://./data` | DB connection string |
| `--llm-provider` | none | LLM for semantic extraction: `claude`, `ollama`, `noop` |
| `--force` | false | Re-analyze already-analyzed papers |

**`export-louvain-graph`** — exports the Louvain community graph to JSON for external tooling

| Argument | Default | Description |
|---|---|---|
| `--db` | `surrealkv://./data` | DB connection string |
| `--output` | *(required)* | Output JSON file path |
| `--published-before` | none | ISO-8601 date cutoff e.g. `2014-12-31` (inclusive, lexicographic) |
| `--tfidf-top-n` | `50` | Max TF-IDF terms per node |

Output schema: `{louvain_params, corpus_fingerprint, nodes: [{id, community_id, tfidf_vec}], communities: [{community_id, size, tfidf_vec}], edges: [{src, dst, weight}]}`. "Other" community papers (community_id = u32::MAX-1) are excluded. Edge weight is `1.0` (uniform). The `communities` field carries per-community c-TF-IDF vectors for EXP-RS-07 (Sheaves-LBD); old consumers ignore it via serde default. Requires communities to be computed first (`analyze` runs community detection). See `resyn-core/src/datamodels/community_graph.rs` for the full type definitions.

**Typical Kuramoto-LBD v03 workflow:**

```bash
# 1. Crawl a corpus (one-time)
cargo run --bin resyn -- crawl --paper-id <seed> --db surrealkv://./data --max-depth 3

# 2. Run analysis (NLP + community detection)
cargo run --bin resyn -- analyze --db surrealkv://./data

# 3. Export for the Python notebook
cargo run --bin resyn -- export-louvain-graph \
    --db surrealkv://./data \
    --output professional-vault/prototypes/data/research_synergy_pre2015.json \
    --published-before 2014-12-31 \
    --tfidf-top-n 50

# 4. Run kuramoto_lbd_v03.ipynb in professional-vault/prototypes/
```

## Important Notes

- arXiv rate limiting: `ArxivHTMLDownloader` enforces configurable delays (default 3s) between requests using `tokio::time::sleep`. Violating this causes request blocks.
- InspireHEP rate limiting: `InspireHepClient` enforces 350ms between requests.
- **`ChainedPaperSource` empty-refs bug (KNOWN, unfixed as of 2026-04-20):** `chained_source.rs` `fetch_references` stops at the first source that returns `Ok(())`, even if `paper.references` is empty. arXiv HTML returns empty refs for physics papers that only cite journal DOIs → chain stops, S2/InspireHEP fallback never fires. Workaround: use `--source inspirehep` or `--source semantic_scholar` directly for physics seeds. Fix: propagate only if `paper.references` is non-empty, otherwise try next source.
- **OpenAlex bulk ingest (`bulk-ingest` subcommand):** Ingests arXiv-indexed papers in bulk from the OpenAlex REST API (polite pool, ~10 req/s). Skips per-paper HTTP calls entirely. `upsert_citations_batch` does not check target paper existence (dangling edges OK). `arxiv_id()` extracts arXiv IDs from both `10.48550/arxiv.*` DOIs and `locations[].landing_page_url` matching `arxiv.org/abs/`. Concept IDs: `C154945302`=ML, `C121332964`=stat.ML, `C41008148`=Neural Networks, `C2778407487`=Statistical Physics.
- Paper IDs may have version suffixes (e.g., "2301.12345v2") — these are stripped via `utils::strip_version_suffix()` during crawl dedup, graph construction, and DB upserts.
- Rust edition 2024, stable toolchain (pinned via `rust-toolchain.toml`).
- Single async runtime: `main.rs` has the only `#[tokio::main]`. All API/HTML functions are `async fn`.
- Error handling uses `ResynError` with `?` propagation. The crawler logs warnings for individual failures and continues.
- SurrealDB `kv-mem` feature compiles as a Rust dependency — no external server needed for tests.
- CI runs fmt check, clippy with `-Dwarnings`, tests, and coverage via tarpaulin.

## Testing

- **46 tests total**: 32 unit tests + 8 integration tests + 6 database tests
  - Unit: paper, graph_creation, arxiv_utils, search_query_handler, validation, utils, inspirehep_api deserialization/conversion
  - Integration: wiremock-based arXiv HTML parsing (3), InspireHEP API mocking (5)
  - Database: SurrealDB in-memory (upsert, idempotent, exists, version dedup, citations, graph traversal)
- All DB tests use `connect_memory()` — no external DB required
- `ArxivHTMLDownloader::with_rate_limit(Duration::from_millis(0))` and `InspireHepClient::with_rate_limit(Duration::from_millis(0))` disable rate limiting in tests
