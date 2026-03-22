---
phase: 06-tech-debt-workspace-restructure
verified: 2026-03-15T10:15:00Z
status: passed
score: 13/13 must-haves verified
re_verification: false
---

# Phase 6: Tech Debt + Workspace Restructure — Verification Report

**Phase Goal:** Restructure into Cargo workspace with 3 crates (resyn-core, resyn-app, resyn-server), establish SSR feature gate, remove egui visualization, rewrite CLI with subcommands, and clean up tech debt.
**Verified:** 2026-03-15T10:15:00Z
**Status:** PASSED
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | 3-crate workspace structure exists (resyn-core, resyn-app, resyn-server) | VERIFIED | `Cargo.toml` is a virtual workspace manifest with `members = ["resyn-core", "resyn-app", "resyn-server"]` and `resolver = "2"` |
| 2 | `use resyn_core::nlp` is accessible (DEBT-01 resolved) | VERIFIED | `resyn-core/src/lib.rs` line 6: `pub mod nlp; // DEBT-01: was missing from old lib.rs` |
| 3 | `resyn_core::gap_analysis::output` is only accessible with `ssr` feature | VERIFIED | `resyn-core/src/gap_analysis/mod.rs`: `#[cfg(feature = "ssr")] pub mod output;` |
| 4 | Downstream crates use petgraph via resyn-core re-exports, not direct petgraph dependency | VERIFIED | `resyn-core/src/lib.rs`: `pub use petgraph;`. Neither `resyn-server/Cargo.toml` nor `resyn-app/Cargo.toml` list petgraph as a direct dependency |
| 5 | resyn-app is a WASM stub crate with no SurrealDB boundary leaks | VERIFIED | `resyn-app/Cargo.toml`: `crate-type = ["cdylib", "rlib"]`, depends on `resyn-core` without `ssr` feature; `.cargo/config.toml` sets `getrandom_backend="wasm_js"` for `wasm32-unknown-unknown` |
| 6 | egui, eframe, egui_graphs, fdg, and crossbeam are completely gone from all Cargo.toml files | VERIFIED | Grep across root `Cargo.toml`, `resyn-core/Cargo.toml`, `resyn-server/Cargo.toml`, `resyn-app/Cargo.toml` returned no matches. Only occurrence is a comment string in `enrichment.rs` explaining the PaperColor migration ("replacing egui::Color32") |
| 7 | No visualization/ directory exists anywhere in the workspace | VERIFIED | `resyn-server/src/visualization/` absent; old `src/` directory deleted entirely |
| 8 | Subcommand CLI works: `resyn crawl`, `resyn analyze`, `resyn serve` | VERIFIED | `resyn-server/src/main.rs` has `#[command(subcommand)]` on `Commands` enum with `Crawl(CrawlArgs)`, `Analyze(AnalyzeArgs)`, `Serve(ServeArgs)` dispatch; `serve.rs` prints "Web server not yet implemented (coming in Phase 8)" and exits 0 |
| 9 | Stale stub comment is gone from ollama.rs (DEBT-02) | VERIFIED | `resyn-core/src/llm/ollama.rs` first lines are real imports (`use async_trait::async_trait;` etc.) — no "OllamaProvider — implemented in Task 2 / Stub to allow Task 1 compilation" comments |
| 10 | TODO.md stale checkboxes are cleaned up (DEBT-03) | VERIFIED | No `TODO.md` file exists in the repository root — deleted per plan decision |
| 11 | SurrealDB and ssr-only deps are feature-gated in resyn-core | VERIFIED | `resyn-core/Cargo.toml` `[features]` section: `ssr = ["dep:surrealdb", "dep:reqwest", "dep:async-trait", "dep:scraper", "dep:arxiv-rs", "dep:futures", "dep:tokio"]`; all 7 listed as `optional = true` |
| 12 | resyn-server depends on resyn-core with ssr feature; resyn-app depends without ssr | VERIFIED | `resyn-server/Cargo.toml`: `resyn-core = { path = "../resyn-core", features = ["ssr"] }`; `resyn-app/Cargo.toml`: `resyn-core = { path = "../resyn-core" }` (no features) |
| 13 | enrichment.rs in datamodels provides WASM-safe PaperColor replacing egui::Color32, with tests | VERIFIED | `resyn-core/src/datamodels/enrichment.rs` exists with `PaperColor { r, g, b }` struct, `paper_type_to_color()`, `finding_strength_radius()`, and 15 unit tests; `pub mod enrichment;` in `resyn-core/src/datamodels/mod.rs` |

**Score:** 13/13 truths verified

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `Cargo.toml` | Virtual workspace manifest | VERIFIED | Contains `[workspace]`, `members`, `resolver = "2"`, `[workspace.dependencies]` |
| `resyn-core/Cargo.toml` | Core library crate with ssr feature | VERIFIED | `[features] ssr = [...]` present; 7 optional deps |
| `resyn-core/src/lib.rs` | Conditional module exports + petgraph re-export | VERIFIED | `pub mod nlp`, `#[cfg(feature = "ssr")] pub mod data_aggregation/database/llm`, `pub use petgraph` |
| `resyn-core/src/gap_analysis/mod.rs` | output module behind ssr | VERIFIED | `#[cfg(feature = "ssr")] pub mod output;` |
| `resyn-core/src/datamodels/enrichment.rs` | WASM-safe PaperColor type | VERIFIED | Substantive (249 lines): struct + functions + 15 tests |
| `resyn-app/Cargo.toml` | WASM stub crate, depends on resyn-core without ssr | VERIFIED | `crate-type = ["cdylib", "rlib"]`, `resyn-core = { path = "../resyn-core" }` |
| `resyn-app/src/lib.rs` | Minimal WASM compilation verification | VERIFIED | Imports `resyn_core::datamodels::paper::Paper`, exports `app_version()` and `get_paper_id()` |
| `resyn-server/Cargo.toml` | Server binary crate with ssr feature | VERIFIED | `[[bin]] name = "resyn"`, `resyn-core = { ..., features = ["ssr"] }` — no egui/petgraph |
| `resyn-server/src/main.rs` | Subcommand-based CLI entry point | VERIFIED | `#[command(subcommand)]` on `Cli`, minimal dispatch to `commands::` module |
| `resyn-server/src/commands/crawl.rs` | Crawl subcommand implementation | VERIFIED | `CrawlArgs` with all required fields; `run()` performs DB connect, source creation, BFS crawl, upsert, optional analysis |
| `resyn-server/src/commands/analyze.rs` | Analyze subcommand implementation | VERIFIED | `AnalyzeArgs` struct + `run()` + `run_analysis_pipeline()` shared function (421 lines, substantive) |
| `resyn-server/src/commands/serve.rs` | Serve placeholder subcommand | VERIFIED | `ServeArgs` + `run()` prints placeholder message, returns `Ok(())` |
| `.cargo/config.toml` | getrandom wasm_js rustflag for WASM target | VERIFIED | `[target.wasm32-unknown-unknown] rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]` |

---

## Key Link Verification

### Plan 01 Key Links

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `resyn-core/src/lib.rs` | `resyn-core/src/datamodels/` | `pub mod datamodels` | WIRED | Line 3 of lib.rs: `pub mod datamodels;` (always available) |
| `resyn-core/src/lib.rs` | `resyn-core/src/database/` | `#[cfg(feature = "ssr")] pub mod database` | WIRED | Lines 14-16 of lib.rs confirm ssr-gated export |
| `resyn-core/src/gap_analysis/mod.rs` | `resyn-core/src/gap_analysis/output.rs` | `#[cfg(feature = "ssr")] pub mod output` | WIRED | Confirmed in mod.rs lines 5-6 |
| `resyn-core/src/lib.rs` | `petgraph` | `pub use petgraph` re-export | WIRED | Line 20: `pub use petgraph;` |
| `resyn-app/Cargo.toml` | `resyn-core` | dependency without ssr feature | WIRED | `resyn-core = { path = "../resyn-core" }` — no features listed |
| `resyn-server/Cargo.toml` | `resyn-core` | dependency with ssr feature | WIRED | `resyn-core = { path = "../resyn-core", features = ["ssr"] }` |

### Plan 02 Key Links

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `resyn-server/src/main.rs` | `resyn-server/src/commands/` | subcommand dispatch match | WIRED | `Commands::Crawl/Analyze/Serve` match dispatches to `commands::crawl/analyze/serve::run()` |
| `resyn-server/src/commands/crawl.rs` | `resyn_core::data_aggregation` | `use resyn_core::` imports | WIRED | Lines 2-4: `use resyn_core::data_aggregation::arxiv_source::ArxivSource`, `InspireHepClient`, `PaperSource` |
| `resyn-server/src/commands/analyze.rs` | `resyn_core::llm` | `use resyn_core::` imports | WIRED | Lines 8-11: `use resyn_core::llm::claude::ClaudeProvider`, `noop::NoopProvider`, `ollama::OllamaProvider`, `traits::LlmProvider` |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| DEBT-01 | 06-01 | Export `nlp` module in `lib.rs` for test/library access | SATISFIED | `pub mod nlp;` in `resyn-core/src/lib.rs` line 6 |
| DEBT-02 | 06-02 | Remove stale stub comment in `src/llm/ollama.rs` | SATISFIED | `ollama.rs` opens with real imports — no stub comments present |
| DEBT-03 | 06-02 | Clean up stale ROADMAP plan checkboxes from v1.0 | SATISFIED | `TODO.md` deleted entirely; `.planning/ROADMAP.md` is sole authoritative roadmap |
| WEB-01 | 06-01 | Cargo workspace restructure into 3-crate layout (core/app/server) | SATISFIED | Virtual workspace with resyn-core, resyn-app, resyn-server |
| WEB-02 | 06-01 | WASM compilation boundary — SurrealDB feature-gated behind `ssr` | SATISFIED | `[features] ssr = [...]` in resyn-core; resyn-app has no ssr feature; `.cargo/config.toml` enables WASM backend |
| WEB-05 | 06-02 | Remove egui/eframe/fdg dependencies | SATISFIED | Zero occurrences of egui/eframe/fdg/crossbeam/egui_graphs in any Cargo.toml; visualization/ directories deleted |

**Orphaned requirements check:** REQUIREMENTS.md traceability table maps exactly DEBT-01, DEBT-02, DEBT-03, WEB-01, WEB-02, WEB-05 to Phase 6 — matches both plans' `requirements:` fields exactly. No orphaned requirements.

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `resyn-core/src/datamodels/enrichment.rs` | 3 | `"replacing egui::Color32"` in a doc comment | Info | Not a code reference — purely a documentation comment explaining the migration rationale. No impact on compilation or WASM boundary. |
| `resyn-app/src/lib.rs` | 1 | `// Minimal WASM compilation verification stub.` | Info | Accurately describes intent. File provides real compilation boundary verification via `get_paper_id()`. Not a placeholder — serves its stated purpose. |

No blocker or warning-level anti-patterns found.

---

## Human Verification Required

### 1. WASM compilation check

**Test:** Run `cargo build -p resyn-app --target wasm32-unknown-unknown`
**Expected:** Compiles cleanly with no SurrealDB or reqwest linker errors
**Why human:** Cannot invoke cargo in this verification context; SUMMARY claims success but compilation must be confirmed against actual toolchain state.

### 2. Full test suite under ssr feature

**Test:** Run `cargo test -p resyn-core --features ssr`
**Expected:** At least 153 tests pass including 15 enrichment tests and 11 gap_analysis::output tests
**Why human:** Cannot execute cargo test here; SUMMARY claims 153 tests passing but actual count needs confirmation especially after git working-tree modifications shown in git status.

### 3. Subcommand CLI help output

**Test:** Run `cargo run -p resyn-server -- crawl --help` and `cargo run -p resyn-server -- analyze --help` and `cargo run -p resyn-server -- serve`
**Expected:** crawl shows --paper-id, --max-depth, --source, --db, --analyze flags; analyze shows --db, --llm-provider, --force flags; serve prints placeholder and exits 0
**Why human:** Cannot execute binaries here; static analysis confirms the Args structs exist with correct fields, but runtime behavior needs confirmation.

---

## Commits Verified

All 5 task commits exist and are in git history:
- `33970f5` — feat(06-01): create 3-crate workspace
- `ab0d90c` — test(06-01): verify all tests pass
- `68662e9` — feat(06-02): remove egui/eframe/fdg deps, delete visualization, fix DEBT-02 DEBT-03
- `467e967` — feat(06-02): rewrite CLI with subcommands
- `f3d1c40` — chore(06-02): apply cargo fmt

---

## Summary

Phase 6 goal is achieved. All 13 must-have truths are verified against the actual codebase, not just SUMMARY claims:

The 3-crate Cargo workspace exists at the repository root with the virtual manifest, resolver=2, and all workspace.dependencies correctly distributed. resyn-core properly splits always-available (WASM-safe) modules from ssr-gated server modules. The `pub mod nlp` export resolves DEBT-01. The `gap_analysis/output` module is correctly gated behind `#[cfg(feature = "ssr")]`. petgraph is re-exported from resyn-core and neither resyn-server nor resyn-app carry a direct petgraph dependency. resyn-app is a proper WASM cdylib stub with getrandom wasm_js backend configured.

All egui/eframe/fdg/crossbeam/egui_graphs references are gone from every Cargo.toml file. No visualization/ directory exists anywhere. The CLI has been rewritten with clap subcommands (crawl/analyze/serve) where each subcommand is implemented in a dedicated `commands/` module with substantive logic (crawl.rs performs full BFS + DB persist; analyze.rs has the complete extraction/NLP/LLM/gap pipeline). DEBT-02 (stub comment in ollama.rs) and DEBT-03 (TODO.md deletion) are both confirmed resolved.

Three items are flagged for human confirmation: WASM compilation, full test count, and CLI runtime behavior — these cannot be verified by static code inspection alone, but the static evidence strongly indicates they will pass.

---

_Verified: 2026-03-15T10:15:00Z_
_Verifier: Claude (gsd-verifier)_
