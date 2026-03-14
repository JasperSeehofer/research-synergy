# Architecture

**Analysis Date:** 2026-03-14

## Pattern Overview

**Overall:** Layered pipeline with abstraction-based data source pluggability

**Key Characteristics:**
- Data source abstraction enables arXiv and InspireHEP implementations without duplication
- BFS crawler operates source-agnostically via `PaperSource` trait
- Single async runtime (`#[tokio::main]` in `src/main.rs`) with all I/O operations async
- Graph construction normalizes paper IDs via version suffix stripping before deduplication
- Error handling uses `ResynError` enum with `?` propagation; crawler logs warnings and continues

## Layers

**Data Aggregation Layer:**
- Purpose: Fetch papers and their references from external sources
- Location: `src/data_aggregation/`
- Contains: Trait definitions (`traits.rs`), source implementations (`arxiv_source.rs`, `inspirehep_api.rs`), HTTP clients (`html_parser.rs`, `arxiv_api.rs`), BFS crawler (`arxiv_utils.rs`), utilities (`search_query_handler.rs`)
- Depends on: `datamodels`, `error`, `utils`, `validation`
- Used by: `main.rs` for data collection

**Data Models Layer:**
- Purpose: Core domain objects used throughout pipeline
- Location: `src/datamodels/paper.rs`
- Contains: `Paper`, `Reference`, `Link`, `Journal`, `DataSource` enum with serde support
- Depends on: `arxiv-rs` for `Arxiv` type conversion
- Used by: All layers (aggregation, processing, database, visualization)

**Database Layer:**
- Purpose: SurrealDB persistence and citation graph queries
- Location: `src/database/`
- Contains: Connection management (`client.rs`), schema initialization (`schema.rs`), repository pattern (`queries.rs`)
- Depends on: `datamodels`, `error`, `utils`, `surrealdb` v3
- Used by: `main.rs` for optional persistence and db-only mode

**Data Processing Layer:**
- Purpose: Transform collected papers into graph structure
- Location: `src/data_processing/graph_creation.rs`
- Contains: `create_graph_from_papers()` builds `petgraph::StableGraph<Paper, f32, Directed>` with citation edges
- Depends on: `datamodels`, `utils`
- Used by: `main.rs` before visualization

**Visualization Layer:**
- Purpose: Interactive force-directed graph display
- Location: `src/visualization/`
- Contains: Main GUI app (`force_graph_app.rs`), Fruchterman-Reingold force layout driver, node/edge drawers (`drawers.rs`), simulation settings (`settings.rs`)
- Depends on: `egui/eframe`, `egui_graphs`, `fdg` (git dep), `petgraph`
- Used by: `main.rs` as final display

**Error Handling Layer:**
- Purpose: Centralized error type with source trace capability
- Location: `src/error.rs`
- Contains: `ResynError` enum (ArxivApi, HtmlDownload, HttpRequest, PaperNotFound, InvalidPaperId, NoArxivLink, InspireHepApi, Database)
- Depends on: `reqwest::Error` (implements From<>)
- Used by: All error-producing modules

**Validation Layer:**
- Purpose: Input validation before processing
- Location: `src/validation.rs`
- Contains: `validate_arxiv_id()` for new (`YYMM.NNNNN`) and old (`category/NNNNNNN`) formats with optional version suffix; `validate_url()` for HTTP(S)
- Depends on: `error`
- Used by: `main.rs` early to gate execution

**Utilities Layer:**
- Purpose: Cross-cutting helper functions
- Location: `src/utils.rs`
- Contains: `strip_version_suffix()` normalizes paper IDs (e.g., `2301.12345v2` → `2301.12345`); `create_http_client()` with 30s timeout
- Depends on: None
- Used by: `graph_creation`, `database/queries`, `data_aggregation`

## Data Flow

**Standard Crawl → Persist → Visualize:**

1. User invokes CLI with paper ID, max depth, source, optional DB endpoint
2. `main.rs` validates paper ID via `validation::validate_arxiv_id()`
3. Connects to DB if `--db` provided (initializes schema)
4. Creates source-specific `Box<dyn PaperSource>` (ArxivSource or InspireHepClient)
5. BFS crawler (`data_aggregation::arxiv_utils::recursive_paper_search_by_references`) explores references up to max_depth
   - Per-paper: fetch_paper() → fetch_references() → extract arxiv reference IDs → queue new papers
   - Version suffixes stripped during dedup to prevent duplicate crawls
   - Crawler logs warnings for individual failures, continues with others
6. Returned `Vec<Paper>` upserted to DB (per-paper records, then citation edges) if DB connected
7. Graph construction calls `create_graph_from_papers()` to build directed citation graph
   - Version dedup again: only one node per base paper ID
   - Citation edges created only for papers both in graph
8. Graph passed to visualization engine (initialized with Fruchterman-Reingold layout)
9. Interactive display with pan, zoom, FPS monitoring

**DB-Only Mode:**

1. Skip crawling entirely
2. Load citation graph from DB via `PaperRepository::get_citation_graph()`
3. Reconstruct papers from DB records
4. Pass to visualization

**State Management:**

- `Vec<Paper>` is the canonical state during crawl
- Version dedup via `utils::strip_version_suffix()` ensures single canonical ID per paper
- Graph node dedup: `HashMap<String, NodeIndex>` maps normalized IDs to graph nodes
- Visited paper tracking during BFS prevents re-fetching
- DB acts as read-only snapshot for db-only mode (no runtime state mutation)
- GUI state (pan, zoom, FPS, simulation paused) maintained in `DemoApp` struct

## Key Abstractions

**PaperSource Trait:**
- Purpose: Decouple crawl logic from data source implementation
- Examples: `ArxivSource` (`src/data_aggregation/arxiv_source.rs`), `InspireHepClient` (`src/data_aggregation/inspirehep_api.rs`)
- Pattern: Async trait with three methods: `fetch_paper(&self, id: &str) -> Result<Paper>`, `fetch_references(&mut self, paper: &mut Paper) -> Result<()>`, `source_name(&self) -> &'static str`
- Enables: BFS crawler in `arxiv_utils.rs` accepts `&mut dyn PaperSource` to work with any implementation

**Paper Domain Model:**
- Purpose: Unified representation of academic papers
- Metadata: title, authors, summary, ID, publication dates, PDF URL, comment
- Optional fields: doi, inspire_id, citation_count, source (Arxiv/InspireHep/Merged)
- Methods: `get_arxiv_references_ids()` filters references to only arxiv links, `from_arxiv_paper()` conversion constructor
- Serialization: Full serde support for DB/JSON

**ResynError Enum:**
- Purpose: Typed error handling with source chain for reqwest::Error
- Variants: ArxivApi, HtmlDownload, HttpRequest, PaperNotFound, InvalidPaperId, NoArxivLink, InspireHepApi, Database
- Display impl shows human-readable messages
- From<reqwest::Error> enables ? propagation from HTTP calls

**StableGraph<Paper, f32, Directed>:**
- Purpose: Citation network representation
- Nodes: Full Paper objects (allows visualization to access metadata)
- Edges: Weighted as 1.0 (significance currently uniform)
- Direction: Paper A → Paper B means A cites B
- Dedup: Normalized paper IDs prevent multi-node papers with version variations

## Entry Points

**CLI Entry:**
- Location: `src/main.rs` line 58 (`#[tokio::main] async fn main()`)
- Triggers: `cargo run [args]`
- Responsibilities: Parse CLI args, validate paper ID, connect DB, create paper source, invoke crawler, persist, launch GUI

**Library Entry:**
- Location: `src/lib.rs` exposes all public modules
- Used by: Tests, potential downstream crates

**Visualization Entry:**
- Location: `src/main.rs` line 147 (`fn launch_visualization()`)
- Triggers: After crawl or DB load completes
- Responsibilities: Call graph builder, create DemoApp, run eframe event loop

## Error Handling

**Strategy:** Typed errors with Try operator chaining; crawler graceful degradation

**Patterns:**
- HTTP failures propagate via `ResynError::HttpRequest` with reqwest::Error source chain
- Paper not found (404 or parse failure) → `ResynError::PaperNotFound`, logged as warning, BFS continues
- Invalid paper ID caught early via `validate_arxiv_id()` before any I/O
- DB connection errors exit fast (no partial graphs written)
- DB upsert failures per-paper logged but don't block visualization (graceful degradation)
- Crawler loop accumulates successful papers and continues despite individual reference failures

## Cross-Cutting Concerns

**Logging:** `tracing` crate with `tracing_subscriber::fmt::init()` in `main.rs`
- Info: Crawl progress (depth, paper count), DB connection, graph stats, visualization launch
- Debug: HTTP requests, rate limiting events, graph edge creation
- Warn: Per-paper failures during crawl (not found, parse errors)
- Error: Validation failures, DB connection failures, early-exit conditions

**Validation:** Input validation happens before I/O
- Paper IDs: `validate_arxiv_id()` supports both arXiv formats with optional version suffix
- URLs: `validate_url()` ensures http(s) scheme
- CLI args: clap validates types and defaults

**Authentication:** None required for arXiv or InspireHEP (public APIs)

---

*Architecture analysis: 2026-03-14*
