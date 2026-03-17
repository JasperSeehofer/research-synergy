---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Scale & Surface
status: completed
stopped_at: Completed 08-07-PLAN.md
last_updated: "2026-03-17T16:50:25.526Z"
last_activity: "2026-03-17 — Plan 08-07 complete: browser verification of all 5 panels with real data"
progress:
  total_phases: 5
  completed_phases: 3
  total_plans: 14
  completed_plans: 14
  percent: 95
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-15)

**Core value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph
**Current focus:** Phase 8 — Leptos Web Shell + Analysis Panels (COMPLETE)

## Current Position

Phase: 8 of 10 (Phase 8: Leptos Web Shell + Analysis Panels) — COMPLETE
Plan: 7 of 7 in current phase (08-07 complete)
Status: Phase complete, ready to advance to Phase 9
Last activity: 2026-03-17 — Plan 08-07 complete: browser verification of all 5 panels with real data

Progress: [█████████░] 95% (v1.1 — all 7 Phase 08 plans done)

## Accumulated Context

### Decisions

- [v1.0] Hybrid NLP + LLM analysis pattern works well; clear separation of concerns
- [v1.0] Pluggable trait pattern (PaperSource, LlmProvider) is the right extensibility model
- [v1.0] DB migration system (6 migrations) handles schema evolution cleanly
- [v1.1] Full Rust/WASM graph stack chosen — web-sys Canvas2D/WebGL2, NO JavaScript graph libraries (sigma.js, d3)
- [v1.1] CSR-only (Trunk, not cargo-leptos) — single-user local tool, no SSR/hydration complexity needed
- [v1.1] SurrealDB must be feature-gated behind `ssr` from day one of workspace restructure (Pitfall 1)
- [v1.1] Barnes-Hut force layout implemented in Rust/WASM Web Worker, not JS ForceAtlas2
- [06-01] tokio added as ssr-gated dep in resyn-core (used by data_aggregation, llm modules)
- [06-01] getrandom wasm_js backend via .cargo/config.toml rustflags + wasm_js feature for WASM compat
- [06-01] error.rs HttpRequest variant gated behind ssr to make ResynError WASM-safe
- [06-01] gap_analysis LLM-dependent functions gated behind ssr; pure graph fns always available
- [06-02] DB argument is REQUIRED (default surrealkv://./data) — no more Option<String> in CLI
- [06-02] TODO.md deleted entirely — .planning/ROADMAP.md is sole canonical roadmap
- [06-02] AnalyzeArgs struct serves as both CLI arg type and pipeline config type for crawl --analyze
- [Phase 07-01]: Named record IDs for idempotent SurrealDB enqueue (CREATE on same ID is a no-op)
- [Phase 07-01]: UPDATE ONLY $let_var (not WHERE id = $bound_var) required for atomic claim in SurrealDB embedded
- [Phase 07-02]: PaperSource is not Clone — each spawned task must create its own instance via make_source() factory
- [Phase 07-02]: fetch_references(&mut self, paper: &mut Paper) mutates paper.references in-place; use paper.get_arxiv_references_ids() to extract arXiv IDs
- [Phase 07-02]: Semaphore::acquire_owned before spawn (not inside task) — bounds total in-flight tasks naturally in main loop
- [Phase 07]: axum and tokio-stream added at workspace level for future reuse by other crates
- [Phase 07]: SSE handler defined inline in tokio::spawn to avoid module-level scope pollution
- [Phase 07]: Queue management subcommands dispatch before paper_id validation — they only need --db arg
- [Phase 07-incremental-crawl-infrastructure]: Each CrawlSubcommand variant owns its --db arg; clap subcommand context stops parent arg parsing so each variant needs its own field
- [Phase 07-incremental-crawl-infrastructure]: Empty pdf_url guard placed at call site in aggregate_references_for_arxiv_paper (not in convert_pdf_url_to_html_url) — paper.id only available at call site
- [Phase 08-leptos-web-shell-analysis-panels]: ProgressEvent gains Deserialize derive when moved to resyn-core (server only had Serialize; WASM client needs both)
- [Phase 08-leptos-web-shell-analysis-panels]: tower-http added to resyn-server for static file serving and CORS (required for Leptos web shell)
- [Phase 08]: Leptos 0.8 Callback uses .run() not .call() — reactive_graph Callable trait exposes run() and try_run()
- [Phase 08]: Sidebar collapse state drives CSS via parent nav class — NavItem does not need collapsed prop
- [Phase 08]: Axum wildcard route /api/{*fn_name} with handle_server_fns_with_context injects Arc<Db> context for all server functions
- [Phase 08]: Aggregation helpers (aggregate_open_problems, build_method_matrix) placed in resyn-core/src/analysis/aggregation.rs — WASM-safe, no ssr gate, unit-testable without Leptos
- [Phase 08]: SSR-only imports in server fn bodies must be inside #[cfg(feature = "ssr")] block — top-level imports cause unused-import warnings on WASM build
- [Phase 08]: ProgressEvent gains PartialEq derive — leptos-use use_event_source requires PartialEq on the decoded type
- [Phase 08]: start_crawl server fn spawns background tokio::spawn with own PaperSource factory (not Clone) — server fn returns immediately after queue seeding
- [Phase 08]: StoredValue used to share immutable loaded Vec<GapFinding> across two closures in Suspense reactive branch
- [Phase 08]: Sub-component pattern required for <For> inside view! macro match arms — move keyword causes Leptos parser errors when used directly in each= attribute inside a match arm
- [Phase 08-07]: Trunk requires data-cargo-features="csr" in index.html link tag — not just Cargo.toml feature gate
- [Phase 08-07]: register_explicit<T>() required for all server fns at startup — inventory auto-registration fails across crate boundaries (resyn-app -> resyn-server)
- [Phase 08-07]: connect() accepts any connection string as-is; connect_local() prepends surrealkv:// prefix (use connect() when user supplies full string)

### Pending Todos

None yet.

### Blockers/Concerns

- Phase 6: Verify `UPDATE ... LIMIT 1` atomicity in embedded SurrealDB under concurrent tokio tasks before committing crawl queue design (MEDIUM confidence gap from research)
- Phase 9: sigma.js integration is research-recommended but user-overridden — use web-sys WebGL2 bindings; spike needed to validate Canvas 2D + Leptos NodeRef pattern before full implementation

## Session Continuity

Last session: 2026-03-17T09:00:00.000Z
Stopped at: Completed 08-07-PLAN.md
Resume file: None
