# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Research Synergy (ReSyn) is a Rust application for Literature Based Discovery (LBD). It aggregates academic papers from arXiv, constructs knowledge graphs from citation relationships, and visualizes them as interactive force-directed graphs.

## Build & Run Commands

```bash
cargo build                  # Debug build
cargo build --release        # Release build
cargo run                    # Run the application
cargo test                   # Run all tests
cargo test <test_name>       # Run a specific test
cargo test -- --nocapture    # Run tests with stdout visible
cargo check                  # Type-check without building
```

## Architecture

The application pipeline: fetch a seed paper from arXiv → recursively aggregate referenced papers → build a directed graph → launch interactive visualization.

Four main layers in `src/`:

- **`data_aggregation/`** — arXiv API queries (`arxiv_api.rs`), recursive reference crawling with deduplication (`arxiv_utils.rs`), HTML scraping for reference metadata (`html_parser.rs`), PDF text extraction (`pdf_parser.rs`), and a builder-pattern search query constructor (`search_query_handler.rs`). `ArxivHTMLDownloader` enforces 3-second rate limiting.

- **`datamodels/`** — Core data structures: `Paper` (title, authors, summary, ID, references), `Reference` (cited paper metadata), `Link`/`Journal` (URL and source type: arXiv, Nature, PhysRev, Unknown).

- **`data_processing/`** — `graph_creation.rs` converts `Vec<Paper>` into a `petgraph::StableGraph<Paper, f32, Directed>`, creating edges from citation relationships. Handles paper ID versioning (strips "v" suffixes for dedup).

- **`visualization/`** — `force_graph_app.rs` is the main GUI using eframe/egui with Fruchterman-Reingold layout (fdg crate). Supports pan/zoom, FPS tracking, and simulation controls. `settings.rs` holds configurable parameters. `graph_app.rs` is an alternative visualization using egui_graphs.

## Key Dependencies

- **tokio** — async runtime for concurrent paper fetching
- **arxiv-rs** — arXiv API client
- **petgraph** — graph data structures
- **egui/eframe** — immediate-mode GUI
- **fdg** (git dep) — force-directed graph layout
- **reqwest** — async HTTP client
- **scraper** — HTML parsing with CSS selectors
- **lopdf** — PDF text extraction

## Data Flow Details

- Each paper requires **two HTTP requests**: one to arXiv API (metadata) and one to the arXiv HTML page (bibliography references). Both must be rate-limited.
- Reference extraction parses `<span class="ltx_bibblock">` elements from arXiv HTML. Titles are extracted from `<em>` tags when present, falling back to comma-splitting.
- `aggregate_references_for_arxiv_paper()` takes a `&mut ArxivHTMLDownloader` to enforce rate limiting on HTML downloads.
- The crawler (`recursive_paper_search_by_references`) does BFS: all papers at depth N are processed before depth N+1.
- Only arXiv-to-arXiv citation edges are followed. References to Nature/PhysRev/other journals are stored but not crawled.
- `pdf_parser.rs` exists but is **unused** in the pipeline — intended for future full-text analysis.
- `graph_app.rs` and `datamodels/graph.rs` are **unused** legacy code (replaced by petgraph + force_graph_app).

## Important Notes

- arXiv rate limiting: `ArxivHTMLDownloader` enforces 3-second delays between requests. Violating this causes request blocks. Both API calls and HTML downloads must go through the rate limiter.
- Paper IDs may have version suffixes (e.g., "2301.12345v2") — these are stripped during both crawl dedup and graph construction.
- Rust edition 2024.
- `arxiv_api.rs` functions use `#[tokio::main]` to make async functions callable synchronously from the main thread.
- Error handling is lenient: failed API calls return errors that callers handle with `unwrap_or_default()` or `continue`. The crawler keeps running when individual papers fail.
- The seed paper ID is currently hardcoded in `main.rs` (`2503.18887`).
