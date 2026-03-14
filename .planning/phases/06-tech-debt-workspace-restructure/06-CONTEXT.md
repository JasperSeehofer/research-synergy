# Phase 6: Tech Debt + Workspace Restructure - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Clean v1.0 tech debt (nlp export, stale stubs, stale checkboxes), split the single-crate project into a 3-crate Cargo workspace (resyn-core / resyn-app / resyn-server), establish the WASM compilation boundary with SurrealDB feature-gated behind `ssr`, remove egui/eframe/fdg dependencies, and redesign the CLI with subcommands. All 153 existing tests must continue to pass.

</domain>

<decisions>
## Implementation Decisions

### Crate layout
- 3-crate workspace: `resyn-core`, `resyn-app`, `resyn-server`
- All domain logic lives in resyn-core: datamodels, data_aggregation, database, data_processing, nlp, llm, gap_analysis, validation, utils, error
- resyn-app is the WASM frontend (minimal stub in Phase 6, fleshed out in Phase 8)
- resyn-server owns the CLI binary (`resyn`) and will become the Axum server in Phase 8
- Core re-exports petgraph types; downstream crates do not add petgraph directly

### WASM boundary
- `ssr` feature flag on resyn-core gates all server-only modules
- Behind `ssr`: data_aggregation/, database/, llm/ (reqwest + tokio + SurrealDB)
- Always available (WASM-safe): datamodels/, data_processing/, nlp/, validation/, utils/, error/
- gap_analysis/ is split: analysis logic (similarity, contradiction, abc_bridge) is always available; output/ (DB writes) is behind `ssr`
- resyn-server depends on `resyn-core = { features = ["ssr"] }`
- resyn-app depends on `resyn-core` (no ssr) — must compile to `wasm32-unknown-unknown`
- Minimal resyn-app crate created in Phase 6 to verify WASM compilation boundary

### Visualization removal
- Delete src/visualization/ entirely — no stubs, no placeholders
- Migrate reusable enrichment data types (color mappings, enrichment structs) to datamodels/ before deletion
- egui, eframe, egui_graphs, fdg dependencies removed from Cargo.toml
- Phase 9 builds the web renderer from scratch in resyn-app

### CLI redesign
- Three subcommands: `resyn crawl`, `resyn analyze`, `resyn serve` (serve is Phase 8 placeholder)
- `resyn crawl -p 2301.12345 -d 3` — fetch + persist
- `resyn analyze` — NLP + LLM + gap analysis on existing DB data
- `resyn crawl -p 2301.12345 --analyze` — crawl then analyze in one shot
- DB is required (default: `surrealkv://./data`) — no more in-memory-only path for the binary
- Skip already-analyzed papers by default; `--force` flag to re-analyze
- Binary name stays `resyn`, owned by resyn-server

### Claude's Discretion
- Exact enrichment types to migrate from visualization/ to datamodels/ (inspect and decide what's egui-specific vs reusable)
- How to handle the `resyn serve` placeholder (empty subcommand with "not yet implemented" message, or omit until Phase 8)
- Test distribution across workspace crates (most stay in core, some may move to server)
- Workspace-level Cargo.toml configuration details (shared dependencies, profiles)

</decisions>

<specifics>
## Specific Ideas

- CLI should feel like a proper subcommand tool (similar to `cargo build`, `cargo test` pattern)
- `resyn crawl -p 2301.12345 --analyze --db surrealkv://./data` as the "do everything" command
- petgraph confirmed as still the right choice for the stack — pure Rust, WASM-compatible, deeply integrated

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `nlp/` module: already exists in src/nlp/ but not exported in lib.rs — DEBT-01 fix is just adding `pub mod nlp;`
- Corpus fingerprint caching: works for skip-if-analyzed behavior in the new `resyn analyze` subcommand
- DB migration system (6 versioned migrations): carries forward unchanged into resyn-core

### Established Patterns
- Pluggable trait pattern (PaperSource, LlmProvider): stays in core, implementations behind `ssr`
- `#[cfg(feature = "...")]` pattern already used in SurrealDB's own feature flags — same pattern for `ssr`
- clap derive macros for CLI parsing: restructure from flat args to subcommand enum

### Integration Points
- src/main.rs: currently single entry point — moves to resyn-server/src/main.rs with subcommand routing
- src/lib.rs: becomes resyn-core/src/lib.rs with conditional module exports
- Cargo.toml: becomes workspace Cargo.toml + 3 crate-level Cargo.toml files
- All 153 tests: primarily test core logic, should run via `cargo test -p resyn-core --features ssr`

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-tech-debt-workspace-restructure*
*Context gathered: 2026-03-15*
