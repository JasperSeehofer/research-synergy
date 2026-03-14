# Codebase Structure

## Directory Layout

```
research-synergy/
├── src/
│   ├── main.rs                      # Entry point — CLI parsing, orchestration, visualization launch
│   ├── lib.rs                       # Public module re-exports for integration tests
│   ├── error.rs                     # ResynError enum (all error variants)
│   ├── validation.rs                # arXiv ID and URL validation
│   ├── utils.rs                     # Shared utilities (version stripping, HTTP client)
│   ├── data_aggregation/            # Data source layer
│   │   ├── mod.rs                   # Module declarations
│   │   ├── traits.rs                # PaperSource async trait
│   │   ├── arxiv_source.rs          # ArxivSource — PaperSource impl wrapping arXiv API + HTML
│   │   ├── arxiv_api.rs             # arXiv API queries via arxiv-rs crate
│   │   ├── arxiv_utils.rs           # BFS crawler + HTML reference extraction
│   │   ├── html_parser.rs           # ArxivHTMLDownloader with rate limiting
│   │   ├── inspirehep_api.rs        # InspireHepClient — PaperSource impl for InspireHEP REST API
│   │   └── search_query_handler.rs  # Builder-pattern search query constructor
│   ├── database/                    # Persistence layer
│   │   ├── mod.rs                   # Module declarations
│   │   ├── client.rs                # Connection management (memory, local, remote)
│   │   ├── schema.rs                # Schema definitions (paper table, cites relation, indexes)
│   │   └── queries.rs               # PaperRepository — CRUD + graph traversal
│   ├── datamodels/                  # Core data structures
│   │   ├── mod.rs                   # Module declarations
│   │   └── paper.rs                 # Paper, Reference, Link, Journal, DataSource
│   ├── data_processing/             # Graph construction
│   │   ├── mod.rs                   # Module declarations
│   │   └── graph_creation.rs        # Vec<Paper> → petgraph::StableGraph
│   └── visualization/               # GUI layer
│       ├── mod.rs                   # Module declarations
│       ├── force_graph_app.rs       # DemoApp — eframe/egui with Fruchterman-Reingold layout
│       ├── drawers.rs               # UI drawing helpers (sliders, debug sections)
│       └── settings.rs              # Configurable GUI parameters
├── tests/
│   ├── html_parsing.rs              # Integration tests: wiremock-based arXiv HTML parsing (3 tests)
│   └── inspirehep_integration.rs    # Integration tests: InspireHEP API mocking (5 tests)
├── Cargo.toml                       # Dependencies, features, metadata
├── Cargo.lock                       # Locked dependency versions
├── rust-toolchain.toml              # Pinned Rust edition 2024, stable toolchain
├── CLAUDE.md                        # Claude Code project instructions
├── CHANGELOG.md                     # Release notes
├── README.md                        # Project documentation
├── IDEAS.md                         # Feature ideas
├── TODO.md                          # Pending tasks
└── .gitignore                       # Git ignore rules
```

## Key Locations

| What | Where |
|---|---|
| Entry point | `src/main.rs` |
| Library root | `src/lib.rs` |
| Error types | `src/error.rs` |
| Data source trait | `src/data_aggregation/traits.rs` |
| BFS crawler | `src/data_aggregation/arxiv_utils.rs:recursive_paper_search_by_references` |
| Core data model | `src/datamodels/paper.rs` |
| DB repository | `src/database/queries.rs` |
| Graph construction | `src/data_processing/graph_creation.rs` |
| GUI application | `src/visualization/force_graph_app.rs` |
| Integration tests | `tests/` |
| Unit tests | Inline `#[cfg(test)] mod tests` in source files |

## Naming Conventions

- **Files:** `snake_case.rs` throughout
- **Modules:** Organized by domain layer (`data_aggregation`, `database`, `datamodels`, `data_processing`, `visualization`)
- **Types:** `PascalCase` — `Paper`, `PaperSource`, `ResynError`, `ArxivSource`, `InspireHepClient`
- **Functions:** `snake_case` — `fetch_paper`, `strip_version_suffix`, `create_graph_from_papers`
- **Constants:** `UPPER_SNAKE_CASE` — `EVENTS_LIMIT`
- **Test functions:** `test_` prefix — `test_upsert_and_get_paper`, `test_valid_arxiv_ids`

## Where to Add New Code

| Adding | Location |
|---|---|
| New data source | Implement `PaperSource` trait in `src/data_aggregation/`, add to `main.rs` match |
| New data model | `src/datamodels/paper.rs` or new file in `src/datamodels/` |
| New DB query | `src/database/queries.rs` on `PaperRepository` |
| New error variant | `src/error.rs` — add to `ResynError` enum + `Display` + optional `From` |
| New CLI argument | `src/main.rs` on `Cli` struct |
| New integration test | `tests/` directory as new `.rs` file |
| New unit test | Inline `#[cfg(test)]` module in the relevant source file |

## Module Dependency Flow

```
main.rs
  ├── data_aggregation (traits, arxiv_source, inspirehep_api)
  │     └── datamodels, error, utils
  ├── database (client, schema, queries)
  │     └── datamodels, error, utils
  ├── data_processing (graph_creation)
  │     └── datamodels, utils
  ├── visualization (force_graph_app, settings, drawers)
  ├── validation
  │     └── error
  └── utils
```

No circular dependencies. Each layer depends down toward `datamodels` and `error`.
