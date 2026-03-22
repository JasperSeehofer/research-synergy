---
phase: 06-tech-debt-workspace-restructure
plan: 02
subsystem: infra
tags: [rust, clap, subcommands, cli, egui-removal, visualization-removal, debt-cleanup]

# Dependency graph
requires:
  - phase: 06-tech-debt-workspace-restructure
    plan: 01
    provides: "3-crate workspace with resyn-core/resyn-app/resyn-server and ssr feature gate"
provides:
  - Subcommand CLI binary: `resyn crawl`, `resyn analyze`, `resyn serve`
  - All egui/eframe/fdg/crossbeam dependencies removed from workspace
  - DEBT-02 resolved: stale stub comments removed from ollama.rs
  - DEBT-03 resolved: TODO.md deleted (superseded by ROADMAP.md)
  - WEB-05 resolved: no visualization code anywhere in workspace
affects:
  - 06-03 (CLI foundation for future subcommands)
  - 07+ (Phase 7+ builds on subcommand CLI pattern)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Clap subcommand dispatch pattern: Cli -> Commands -> per-module run() functions
    - Command module pattern: each subcommand in commands/{name}.rs with Args struct + run()
    - Shared pipeline: run_analysis_pipeline() callable from both crawl --analyze and analyze

key-files:
  created:
    - resyn-server/src/commands/mod.rs
    - resyn-server/src/commands/crawl.rs
    - resyn-server/src/commands/analyze.rs
    - resyn-server/src/commands/serve.rs
  modified:
    - resyn-server/src/main.rs (complete rewrite: minimal subcommand dispatch)
    - resyn-server/Cargo.toml (removed egui/eframe/fdg/crossbeam/egui_graphs)
    - Cargo.toml (removed egui/eframe/fdg/crossbeam/egui_graphs from workspace.dependencies)
    - resyn-core/src/llm/ollama.rs (DEBT-02: removed stale stub comments)
  deleted:
    - TODO.md (DEBT-03: superseded by .planning/ROADMAP.md)
    - resyn-server/src/visualization/ (5 files: drawers.rs, enrichment.rs, force_graph_app.rs, mod.rs, settings.rs)

key-decisions:
  - "DB argument is REQUIRED with default surrealkv://./data — no more Option<String> in CLI"
  - "TODO.md deleted entirely — .planning/ROADMAP.md is the canonical roadmap, duplicates create confusion"
  - "Analysis logic extracted into commands/analyze.rs as run_analysis_pipeline() shared by crawl --analyze"
  - "AnalyzeArgs struct is both the CLI arg type and the pipeline config type — simplifies crawl --analyze integration"

patterns-established:
  - "Subcommand dispatch: main.rs is minimal (parse CLI, match, dispatch), all logic in commands/ modules"
  - "Shared pipeline: run_analysis_pipeline(db, args, rate_limit_secs, skip_fulltext) callable from multiple subcommands"

requirements-completed: [WEB-05, DEBT-02, DEBT-03]

# Metrics
duration: 90min
completed: 2026-03-15
---

# Phase 6 Plan 02: CLI Rewrite + Egui Removal Summary

**Clap subcommand CLI (crawl/analyze/serve) replacing monolith main.rs, with egui/eframe/fdg/crossbeam removed, visualization deleted, and all remaining v1.0 tech debt resolved**

## Performance

- **Duration:** ~90 min
- **Started:** 2026-03-15T08:30:00Z
- **Completed:** 2026-03-15T09:45:00Z
- **Tasks:** 3
- **Files modified:** 14 (created/modified/deleted)

## Accomplishments

- Complete egui/eframe/fdg/crossbeam removal: 5 deps gone from workspace Cargo.toml, 5 deps gone from resyn-server/Cargo.toml, visualization directory (5 files) deleted
- Subcommand CLI with `resyn crawl` (BFS + optional --analyze), `resyn analyze` (pipeline on existing DB), `resyn serve` (placeholder)
- All 153 resyn-core tests pass, WASM compiles, clippy clean, fmt clean
- DEBT-02 resolved: stale "Stub to allow Task 1 compilation" comments removed from ollama.rs
- DEBT-03 resolved: TODO.md deleted — ROADMAP.md is now sole roadmap source

## Task Commits

Each task was committed atomically:

1. **Task 1: Delete visualization and remove egui dependencies** - `68662e9` (feat)
2. **Task 2: Rewrite CLI with subcommands** - `467e967` (feat)
3. **Task 3: Final workspace validation (fmt fix)** - `f3d1c40` (chore)

## Files Created/Modified

- `/home/jasper/Repositories/research-synergy/resyn-server/src/main.rs` - Minimal subcommand dispatch with #[command(subcommand)]
- `/home/jasper/Repositories/research-synergy/resyn-server/src/commands/mod.rs` - Module declarations
- `/home/jasper/Repositories/research-synergy/resyn-server/src/commands/crawl.rs` - CrawlArgs + BFS crawl + DB persist + optional analysis
- `/home/jasper/Repositories/research-synergy/resyn-server/src/commands/analyze.rs` - AnalyzeArgs + full extraction/NLP/LLM/gap pipeline
- `/home/jasper/Repositories/research-synergy/resyn-server/src/commands/serve.rs` - ServeArgs placeholder
- `/home/jasper/Repositories/research-synergy/resyn-server/Cargo.toml` - Removed egui/eframe/fdg/crossbeam/egui_graphs
- `/home/jasper/Repositories/research-synergy/Cargo.toml` - Removed visualization deps from workspace.dependencies

## Decisions Made

- **DB as required argument with default:** Per CONTEXT.md locked decisions, DB is now always required (with a sensible default of `surrealkv://./data`). The old `Option<String>` allowed running without a DB, but the new subcommand model assumes persistence.
- **TODO.md deleted entirely:** The file had stale Phase 1-3 v1.0 checkboxes that duplicated and contradicted ROADMAP.md. Deleting avoids confusion; ROADMAP.md is authoritative.
- **AnalyzeArgs shared between crawl and analyze:** Rather than creating a separate internal struct, `AnalyzeArgs` serves both as the clap CLI struct and the pipeline configuration. The crawl command constructs an `AnalyzeArgs` from its own fields to pass to `run_analysis_pipeline()`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `AnalysisArgs` name mismatch in crawl.rs**
- **Found during:** Task 2 (first build attempt)
- **Issue:** crawl.rs imported `AnalysisArgs` but analyze.rs defined `AnalyzeArgs` — compiler error E0432
- **Fix:** Renamed the import and struct construction in crawl.rs to use `AnalyzeArgs`
- **Files modified:** resyn-server/src/commands/crawl.rs
- **Verification:** cargo build -p resyn-server succeeded
- **Committed in:** 467e967 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug — name mismatch)
**Impact on plan:** Minor typo fix during first compilation. No architectural change needed.

## Issues Encountered

- **Disk full during compilation:** The target/debug/deps directory was 18GB, causing "No space left on device" during surrealdb_core compilation. Freed ~3GB by deleting incremental/ and stale .rlib files (old egui/research_synergy artifacts). This is an environment issue, not a plan issue.
- **Multiple concurrent cargo builds blocking on artifact lock:** Several background build processes from previous bash calls were all competing to compile surrealdb_core. Had to wait for them to sequence through before final builds could proceed.

## Next Phase Readiness

- Clean workspace: no egui/visualization code, subcommand CLI in place, all tech debt resolved
- All 153 tests pass under ssr feature; WASM target compiles cleanly
- `resyn crawl` and `resyn analyze` are ready for Phase 7 to extend with additional flags
- `resyn serve` placeholder in place for Phase 8 web server implementation

---
*Phase: 06-tech-debt-workspace-restructure*
*Completed: 2026-03-15*

## Self-Check: PASSED

All files exist and all commits verified:
- `resyn-server/src/commands/` directory with mod.rs, crawl.rs, analyze.rs, serve.rs: FOUND
- `resyn-server/src/main.rs` rewritten: FOUND
- Commit 68662e9 (Task 1 - egui removal): FOUND
- Commit 467e967 (Task 2 - CLI rewrite): FOUND
- Commit f3d1c40 (Task 3 - fmt): FOUND
