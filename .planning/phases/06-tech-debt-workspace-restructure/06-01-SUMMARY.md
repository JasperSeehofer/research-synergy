---
phase: 06-tech-debt-workspace-restructure
plan: 01
subsystem: infra
tags: [rust, cargo-workspace, wasm, surreald, feature-gates, petgraph]

# Dependency graph
requires: []
provides:
  - 3-crate Cargo workspace (resyn-core, resyn-app, resyn-server)
  - ssr feature gate separating SurrealDB/reqwest/tokio from WASM-safe code
  - WASM-compilable resyn-app stub crate
  - resyn-core exporting pub mod nlp (DEBT-01 resolved)
  - gap_analysis/output gated behind ssr
  - petgraph re-exported from resyn-core
  - PaperColor WASM-safe enrichment type in datamodels
affects:
  - 06-02 (CLI rewrite depends on this workspace structure)
  - all subsequent phases (all build on resyn-core)

# Tech tracking
tech-stack:
  added:
    - cargo workspace resolver=2
    - wasm-bindgen 0.2 (resyn-app)
    - getrandom wasm_js feature (WASM compatibility)
    - .cargo/config.toml with getrandom_backend="wasm_js" rustflag
  patterns:
    - ssr feature gate pattern for SurrealDB/reqwest conditional compilation
    - #[cfg(feature = "ssr")] on imports, functions, test helpers
    - pub use petgraph re-export from resyn-core for downstream crates
    - PaperColor struct as WASM-safe replacement for egui::Color32

key-files:
  created:
    - Cargo.toml (virtual workspace manifest)
    - resyn-core/Cargo.toml (core library with ssr feature)
    - resyn-core/src/lib.rs (conditional module exports + petgraph re-export)
    - resyn-core/src/datamodels/enrichment.rs (PaperColor WASM-safe type)
    - resyn-core/src/gap_analysis/mod.rs (output gated behind ssr)
    - resyn-app/Cargo.toml (WASM stub crate)
    - resyn-app/src/lib.rs (minimal WASM verification)
    - resyn-server/Cargo.toml (server binary crate)
    - resyn-server/src/main.rs (adapted from old src/main.rs)
    - .cargo/config.toml (getrandom wasm_js rustflag)
  modified:
    - resyn-core/src/error.rs (HttpRequest variant gated behind ssr)
    - resyn-core/src/utils.rs (create_http_client gated behind ssr)
    - resyn-core/src/datamodels/paper.rs (from_arxiv_paper gated behind ssr)
    - resyn-core/src/gap_analysis/abc_bridge.rs (LLM functions gated behind ssr)
    - resyn-core/src/gap_analysis/contradiction.rs (LLM functions gated behind ssr)

key-decisions:
  - "tokio added as ssr-gated optional dependency in resyn-core (needed by data_aggregation, llm)"
  - "getrandom wasm_js backend configured via .cargo/config.toml rustflags + wasm_js feature in resyn-app"
  - "error.rs HttpRequest variant gated behind ssr to make ResynError WASM-compatible"
  - "gap_analysis abc_bridge/contradiction LLM-dependent functions gated behind ssr (pure graph fns stay always-available)"
  - "visualization module copied to resyn-server (temporary until plan 02 removes it)"

patterns-established:
  - "SSR feature gate: #[cfg(feature = ssr)] on all SurrealDB/reqwest/tokio-dependent code"
  - "Downstream crates reference petgraph via resyn_core::petgraph not direct dep"
  - "WASM boundary verified: cargo build -p resyn-app --target wasm32-unknown-unknown"

requirements-completed: [WEB-01, WEB-02, DEBT-01]

# Metrics
duration: 52min
completed: 2026-03-15
---

# Phase 6 Plan 01: Workspace Restructure Summary

**3-crate Cargo workspace with resyn-core/resyn-app/resyn-server, SurrealDB feature-gated behind `ssr`, and WASM compilation verified for wasm32-unknown-unknown target**

## Performance

- **Duration:** 52 min
- **Started:** 2026-03-15T06:37:34Z
- **Completed:** 2026-03-15T07:29:32Z
- **Tasks:** 2
- **Files modified:** 55 (created/renamed/deleted)

## Accomplishments

- Created 3-crate workspace: resyn-core (library), resyn-app (WASM stub), resyn-server (binary)
- All 153 resyn-core tests pass under `--features ssr` with no regression
- WASM boundary verified: resyn-app compiles to wasm32-unknown-unknown cleanly with no SurrealDB linker errors
- DEBT-01 resolved: `pub mod nlp` exported from resyn-core/src/lib.rs
- 15 enrichment tests migrated to WASM-safe PaperColor type in datamodels/enrichment.rs
- 11 gap_analysis::output tests pass behind ssr feature gate (per user decision)
- petgraph re-exported from resyn-core; no direct petgraph dependency in resyn-server

## Task Commits

Each task was committed atomically:

1. **Task 1: Create workspace skeleton and move source to resyn-core** - `33970f5` (feat)
2. **Task 2: Verify all tests pass in workspace** - `ab0d90c` (test)

## Files Created/Modified

- `/home/jasper/Repositories/research-synergy/Cargo.toml` - Virtual workspace manifest with resolver=2 and all workspace.dependencies
- `/home/jasper/Repositories/research-synergy/resyn-core/Cargo.toml` - Core library with ssr feature gating 7 deps
- `/home/jasper/Repositories/research-synergy/resyn-core/src/lib.rs` - Conditional module exports + petgraph re-export
- `/home/jasper/Repositories/research-synergy/resyn-core/src/datamodels/enrichment.rs` - PaperColor WASM-safe struct with 15 migrated tests
- `/home/jasper/Repositories/research-synergy/resyn-core/src/gap_analysis/mod.rs` - output module behind #[cfg(feature = "ssr")]
- `/home/jasper/Repositories/research-synergy/resyn-app/Cargo.toml` - WASM cdylib crate with getrandom wasm_js
- `/home/jasper/Repositories/research-synergy/resyn-server/Cargo.toml` - Server binary with eframe (temporary, plan 02 removes)
- `/home/jasper/Repositories/research-synergy/resyn-server/src/main.rs` - Adapted from old src/main.rs using resyn_core:: imports
- `/home/jasper/Repositories/research-synergy/.cargo/config.toml` - getrandom_backend="wasm_js" rustflag for wasm32 target

## Decisions Made

- **tokio as ssr-gated dep:** tokio is used by data_aggregation html_parser, text_extractor, inspirehep_api and llm modules — all ssr-only — so gating it makes sense.
- **getrandom wasm_js via .cargo/config.toml:** The getrandom v0.3 change requires explicit wasm_js backend. Used the rustflag approach (`.cargo/config.toml`) combined with the `wasm_js` feature in resyn-app for correct compilation.
- **error.rs HttpRequest variant gated:** ResynError must be WASM-safe since it's in the always-available `error` module. The HttpRequest(reqwest::Error) variant is only needed for ssr code paths.
- **gap_analysis LLM functions gated behind ssr:** abc_bridge.rs and contradiction.rs have `find_*` functions that take `&mut dyn LlmProvider` (ssr-only trait). Pure graph helper functions (graph_distance, has_direct_edge, findings_diverge) also gated since they're only called from ssr-gated functions.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] tokio missing from resyn-core ssr deps**
- **Found during:** Task 1 (first build attempt)
- **Issue:** html_parser.rs, text_extractor.rs, inspirehep_api.rs, claude.rs, ollama.rs use tokio::time::sleep but tokio was not listed in the ssr feature deps
- **Fix:** Added tokio as optional ssr-gated dependency in resyn-core/Cargo.toml
- **Files modified:** resyn-core/Cargo.toml
- **Committed in:** 33970f5 (Task 1 commit)

**2. [Rule 1 - Bug] error.rs uses reqwest::Error in always-available module**
- **Found during:** Task 1 (WASM build attempt)
- **Issue:** ResynError::HttpRequest(reqwest::Error) caused WASM compilation failure since reqwest is ssr-only
- **Fix:** Gated HttpRequest variant and its Display/Error/From impls behind #[cfg(feature = "ssr")]
- **Files modified:** resyn-core/src/error.rs
- **Committed in:** 33970f5 (Task 1 commit)

**3. [Rule 1 - Bug] paper.rs imports arxiv::Arxiv (ssr-only crate) unconditionally**
- **Found during:** Task 1 (WASM build attempt)
- **Issue:** arxiv-rs is an ssr-only dep but paper.rs imported Arxiv and Paper::from_arxiv_paper unconditionally
- **Fix:** Gated use arxiv::Arxiv import and from_arxiv_paper method behind #[cfg(feature = "ssr")]
- **Files modified:** resyn-core/src/datamodels/paper.rs
- **Committed in:** 33970f5 (Task 1 commit)

**4. [Rule 1 - Bug] utils.rs create_http_client uses reqwest unconditionally**
- **Found during:** Task 1 (WASM build attempt)
- **Issue:** create_http_client() returns reqwest::Client but reqwest is ssr-only
- **Fix:** Gated function and Duration import behind #[cfg(feature = "ssr")]
- **Files modified:** resyn-core/src/utils.rs
- **Committed in:** 33970f5 (Task 1 commit)

**5. [Rule 1 - Bug] gap_analysis modules use LlmProvider from ssr-gated llm module**
- **Found during:** Task 1 (WASM build attempt)
- **Issue:** contradiction.rs and abc_bridge.rs import crate::llm::traits::LlmProvider which is behind ssr
- **Fix:** Gated all LLM-dependent imports, functions, and their tests behind #[cfg(feature = "ssr")]
- **Files modified:** resyn-core/src/gap_analysis/contradiction.rs, resyn-core/src/gap_analysis/abc_bridge.rs
- **Committed in:** 33970f5 (Task 1 commit)

**6. [Rule 3 - Blocking] getrandom v0.3 WASM compilation requires wasm_js backend**
- **Found during:** Task 1 (WASM build attempt)
- **Issue:** getrandom v0.3 requires explicit wasm_js feature for wasm32 target (breaking change from v0.2)
- **Fix:** Added .cargo/config.toml with getrandom_backend="wasm_js" rustflag, added getrandom with wasm_js feature and wasm-bindgen to resyn-app/Cargo.toml
- **Files modified:** .cargo/config.toml, resyn-app/Cargo.toml, Cargo.toml (workspace.dependencies)
- **Committed in:** 33970f5 (Task 1 commit)

---

**Total deviations:** 6 auto-fixed (2 missing ssr gates for new deps, 4 WASM boundary violations in existing code)
**Impact on plan:** All auto-fixes necessary for WASM compilation. Each was a direct consequence of the feature-gating work the plan specified. No scope creep.

## Issues Encountered

- The plan listed gap_analysis modules as "always available" but their primary public functions (find_contradictions, find_abc_bridges) take LlmProvider which is ssr-only. Resolution: gated only the LLM-dependent functions behind ssr while the module itself remains always-importable for the pure structural types.

## Next Phase Readiness

- 3-crate workspace fully operational — plan 02 can rewrite the CLI
- WASM target verified — plan 03 can add Leptos frontend
- All 153 tests passing — no regression from the restructure
- Temporary visualization code in resyn-server/src/visualization/ ready for deletion in plan 02

---
*Phase: 06-tech-debt-workspace-restructure*
*Completed: 2026-03-15*
