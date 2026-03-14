# Phase 6: Tech Debt + Workspace Restructure - Research

**Researched:** 2026-03-15
**Domain:** Cargo workspace restructure, WASM compilation boundary, feature flags, clap subcommands
**Confidence:** HIGH

## Summary

Phase 6 converts a single-crate Rust project into a 3-crate Cargo workspace (`resyn-core`, `resyn-app`, `resyn-server`) while gate-keeping all server-only dependencies (SurrealDB, reqwest, tokio full, async-trait) behind an `ssr` feature flag on `resyn-core`. This is non-negotiable: SurrealDB embeds native crypto/TLS libraries (`ring`, `rustls`) that cannot target `wasm32-unknown-unknown`, so the crate boundary must exist before any WASM code is written. The remaining work is minor v1.0 tech debt: expose the `nlp` module, remove a stale stub comment in `ollama.rs`, and clean up `TODO.md`.

One critical finding: the 15 tests inside `src/visualization/enrichment.rs` (`paper_type_to_color`, `finding_strength_radius`) depend on `egui::Color32`. When `src/visualization/` is deleted as decided, these tests must be rewritten using a WASM-safe color type in `resyn-core/src/datamodels/`. The logic (string-to-color mapping, radius multipliers) is reusable domain knowledge; the `Color32` type is not. The planner must account for this migration so the 153-test baseline is maintained.

**Primary recommendation:** Create the workspace skeleton first, gate SurrealDB behind `ssr`, verify `cargo build -p resyn-app --target wasm32-unknown-unknown` compiles before touching anything else. Treat WASM compilation as the phase gating criterion.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Crate layout**
- 3-crate workspace: `resyn-core`, `resyn-app`, `resyn-server`
- All domain logic lives in resyn-core: datamodels, data_aggregation, database, data_processing, nlp, llm, gap_analysis, validation, utils, error
- resyn-app is the WASM frontend (minimal stub in Phase 6, fleshed out in Phase 8)
- resyn-server owns the CLI binary (`resyn`) and will become the Axum server in Phase 8
- Core re-exports petgraph types; downstream crates do not add petgraph directly

**WASM boundary**
- `ssr` feature flag on resyn-core gates all server-only modules
- Behind `ssr`: data_aggregation/, database/, llm/ (reqwest + tokio + SurrealDB)
- Always available (WASM-safe): datamodels/, data_processing/, nlp/, validation/, utils/, error/
- gap_analysis/ is split: analysis logic (similarity, contradiction, abc_bridge) is always available; output/ (DB writes) is behind `ssr`
- resyn-server depends on `resyn-core = { features = ["ssr"] }`
- resyn-app depends on `resyn-core` (no ssr) — must compile to `wasm32-unknown-unknown`
- Minimal resyn-app crate created in Phase 6 to verify WASM compilation boundary

**Visualization removal**
- Delete src/visualization/ entirely — no stubs, no placeholders
- Migrate reusable enrichment data types (color mappings, enrichment structs) to datamodels/ before deletion
- egui, eframe, egui_graphs, fdg dependencies removed from Cargo.toml
- Phase 9 builds the web renderer from scratch in resyn-app

**CLI redesign**
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

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DEBT-01 | Export `nlp` module in `lib.rs` for test/library access | Confirmed: `src/lib.rs` has no `pub mod nlp;` — single-line fix |
| DEBT-02 | Remove stale stub comment in `src/llm/ollama.rs` | Confirmed: line 1 reads `// OllamaProvider — implemented in Task 2 / Stub to allow Task 1 compilation` |
| DEBT-03 | Clean up stale ROADMAP plan checkboxes from v1.0 | Confirmed: `TODO.md` contains v1.0 phase plan with stale unchecked items referencing Phase 1–3 milestones now superseded by v1.1 roadmap |
| WEB-01 | Cargo workspace restructure into 3-crate layout (core/app/server) | Workspace.dependencies table pattern documented; resolver="2" required; all 153 lib tests must move to resyn-core |
| WEB-02 | WASM compilation boundary — SurrealDB feature-gated behind `ssr` | SurrealDB confirmed WASM-incompatible (ring/rustls); feature gate pattern documented; 15 enrichment tests need Color32 replacement |
| WEB-05 | Remove egui/eframe/fdg dependencies | 5 deps to remove: egui, eframe, egui_graphs, fdg, crossbeam (crossbeam used only in force_graph_app.rs) |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Cargo workspace | built-in | Multi-crate project layout | Native Cargo feature, no extra tooling |
| `#[cfg(feature = "ssr")]` | stable | Gate server-only modules | Standard Rust conditional compilation |
| clap 4 derive | `4` (existing) | CLI subcommands | Already used; derive API supports `#[command(subcommand)]` enum |
| resolver = "2" | Cargo 1.53+ | Prevent cross-target feature unification | Required to stop WASM and native builds from unifying features |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `[workspace.dependencies]` | Cargo 1.64+ | Shared dep versions across crates | Prevents version drift between workspace members |
| `dep:` prefix in features | Rust 1.60+ | Prevent implicit feature from optional dep name | Use when optional dep name would pollute feature namespace |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `cfg(feature = "ssr")` | `cfg(target_arch = "wasm32")` | Target-based cfg is fragile: tests run on x86, not wasm. Feature flag is explicit and testable |
| Workspace virtual manifest | Root crate as workspace | Virtual manifest is cleaner for 3 peer crates, no root-crate confusion |

**Installation:** No new dependencies. This is purely a restructure of existing dependencies.

## Architecture Patterns

### Workspace File Layout
```
Cargo.toml                     # workspace root — [workspace] + [workspace.dependencies]
Cargo.lock                     # single shared lock file
resyn-core/
  Cargo.toml                   # [package] + [dependencies] with workspace=true
  src/
    lib.rs                     # conditional pub mod declarations
    datamodels/                # always available (WASM-safe)
    data_processing/           # always available
    nlp/                       # always available (DEBT-01 fix: add pub mod nlp;)
    validation/                # always available
    utils/                     # always available
    error/                     # always available
    gap_analysis/
      mod.rs                   # pub mod similarity; pub mod contradiction; pub mod abc_bridge;
                               # #[cfg(feature="ssr")] pub mod output;
    data_aggregation/          # behind ssr
    database/                  # behind ssr
    llm/                       # behind ssr
resyn-server/
  Cargo.toml                   # bin crate, resyn-core features=["ssr"]
  src/
    main.rs                    # subcommand routing
    commands/
      crawl.rs
      analyze.rs
      serve.rs
resyn-app/
  Cargo.toml                   # lib crate, wasm32 target, resyn-core (no ssr)
  src/
    lib.rs                     # minimal stub — Phase 6 just verifies compilation
```

### Pattern 1: Workspace Root Cargo.toml
**What:** Virtual manifest with shared dep versions; no `[package]` section.
**When to use:** 3 peer crates with no "main" crate owning the workspace.
**Example:**
```toml
# Source: https://doc.rust-lang.org/cargo/reference/workspaces.html
[workspace]
members = ["resyn-core", "resyn-app", "resyn-server"]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.44.1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
reqwest = { version = "0.12.15", features = ["json"] }
surrealdb = { version = "3", features = ["kv-mem", "kv-surrealkv"] }
petgraph = { version = "0.7.0", default-features = false, features = ["graphmap", "stable_graph", "matrix_graph", "serde-1"] }
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
stop-words = "0.8"
async-trait = "0.1"
futures = "0.3.31"
anyhow = "1.0.97"
rand = "0.9.0"
scraper = "0.23.1"
arxiv-rs = "0.1.5"
wiremock = "0.6"
tokio-test = "0.4"

[profile.release]
opt-level = 3
```

### Pattern 2: Feature-Gated Modules in resyn-core
**What:** `lib.rs` conditionally exposes server-only modules behind `ssr` feature.
**When to use:** Any module that imports SurrealDB, reqwest, tokio, or async-trait.
**Example:**
```toml
# resyn-core/Cargo.toml
[package]
name = "resyn-core"
version = "0.1.0"
edition = "2024"

[features]
ssr = [
    "dep:surrealdb",
    "dep:reqwest",
    "dep:async-trait",
    "dep:scraper",
    "dep:arxiv-rs",
    "dep:futures",
]

[dependencies]
# Always available (WASM-safe)
serde.workspace = true
serde_json.workspace = true
petgraph.workspace = true
chrono.workspace = true
sha2.workspace = true
stop-words.workspace = true
tracing.workspace = true
rand.workspace = true
anyhow.workspace = true

# Behind ssr feature
surrealdb = { workspace = true, optional = true }
reqwest = { workspace = true, optional = true }
async-trait = { workspace = true, optional = true }
scraper = { workspace = true, optional = true }
arxiv-rs = { workspace = true, optional = true }
futures = { workspace = true, optional = true }

[dev-dependencies]
wiremock.workspace = true
tokio-test.workspace = true
tokio.workspace = true
```

```rust
// resyn-core/src/lib.rs
pub mod datamodels;
pub mod data_processing;
pub mod nlp;           // DEBT-01: was missing, add here
pub mod validation;
pub mod utils;
pub mod error;
pub mod gap_analysis;  // always available (analysis logic only)

#[cfg(feature = "ssr")]
pub mod data_aggregation;

#[cfg(feature = "ssr")]
pub mod database;

#[cfg(feature = "ssr")]
pub mod llm;
```

### Pattern 3: Clap Subcommand Enum
**What:** Replace flat `Cli` struct with enum-based subcommands.
**When to use:** When commands have distinct argument sets and dispatch logic.
**Example:**
```rust
// Source: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
#[derive(Parser)]
#[command(name = "resyn", about = "Research Synergy - Literature Based Discovery")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch papers and persist to database
    Crawl(CrawlArgs),
    /// Run NLP + LLM + gap analysis on DB contents
    Analyze(AnalyzeArgs),
    /// Start the web server (Phase 8)
    Serve(ServeArgs),
}

#[derive(Args)]
struct CrawlArgs {
    #[arg(short, long, default_value = "2503.18887")]
    paper_id: String,
    #[arg(short = 'd', long, default_value_t = 3)]
    max_depth: usize,
    #[arg(short, long, default_value_t = 3)]
    rate_limit_secs: u64,
    #[arg(long, default_value = "arxiv")]
    source: String,
    #[arg(long, default_value = "surrealkv://./data")]
    db: String,
    /// Run analyze pipeline after crawl
    #[arg(long)]
    analyze: bool,
}

#[derive(Args)]
struct AnalyzeArgs {
    #[arg(long, default_value = "surrealkv://./data")]
    db: String,
    #[arg(long)]
    llm_provider: Option<String>,
    #[arg(long)]
    llm_model: Option<String>,
    #[arg(long)]
    force: bool,
    #[arg(long)]
    full_corpus: bool,
    #[arg(long)]
    verbose: bool,
}

#[derive(Args)]
struct ServeArgs {}
```

### Pattern 4: gap_analysis `output` Module Behind `ssr`
**What:** `gap_analysis/mod.rs` conditionally includes the output module.
**Why:** The CONTEXT.md says `output/` (DB writes) is behind `ssr`. But inspection shows `output.rs` only formats strings — no DB access. It's safe to keep it always-available. The planner should verify this.
**Example:**
```rust
// resyn-core/src/gap_analysis/mod.rs
pub mod abc_bridge;
pub mod contradiction;
pub mod output;     // string formatting only — WASM-safe
pub mod similarity;
// Note: DB writes for gap findings happen in resyn-server, not in this module
```

### Anti-Patterns to Avoid
- **Putting SurrealDB in the always-available section:** Even `use surrealdb` anywhere in a non-feature-gated path will cause WASM linker errors. Must be 100% behind `#[cfg(feature = "ssr")]`.
- **Using `cfg(not(target_arch = "wasm32"))` instead of feature flags:** Tests run on x86_64, so `cfg(not(target_arch = "wasm32"))` would include DB code in test builds. Feature flag is the right boundary.
- **Feature unification via workspace-level builds:** Running `cargo test` from workspace root may unify features across crates. Use `cargo test -p resyn-core --features ssr` for core tests, `cargo build -p resyn-app --target wasm32-unknown-unknown` for WASM checks.
- **Forgetting `resolver = "2"` in workspace Cargo.toml:** Without it, Cargo 1.x may unify features across dev/normal/build contexts, pulling in native deps during WASM compilation.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Conditional compilation | Custom build.rs scripts | `#[cfg(feature = "ssr")]` | Native Rust; no external tooling |
| Shared dep versions | Copy-paste version strings | `[workspace.dependencies]` + `workspace = true` | Prevents silent version divergence |
| CLI subcommand dispatch | Manual arg matching | clap 4 derive `#[command(subcommand)]` | Type-safe, already used in project |

**Key insight:** The workspace restructure is purely mechanical file movement + Cargo.toml editing. No new algorithms, no new libraries. The complexity is in getting the feature boundaries exactly right on the first pass.

## Common Pitfalls

### Pitfall 1: SurrealDB in WASM via Transitive Dependency
**What goes wrong:** `resyn-core` conditionally expiles `database/` behind `ssr`, but another always-available module (e.g., an error variant, a struct field) imports a SurrealDB type. The WASM build fails with linker errors about `ring` or native TLS.
**Why it happens:** Rust's conditional compilation is per-item, not per-file. One stray `use surrealdb::...` outside a `#[cfg(feature = "ssr")]` block contaminates the always-available path.
**How to avoid:** After structuring, immediately run `cargo build -p resyn-app --target wasm32-unknown-unknown`. Fix every error before proceeding. This is the phase's gating check.
**Warning signs:** Linker errors mentioning `ring`, `rustls`, `openssl`, or `native-tls`.

### Pitfall 2: Enrichment Tests Break When visualization/ is Deleted
**What goes wrong:** 15 tests in `src/visualization/enrichment.rs` test `paper_type_to_color` (returns `egui::Color32`) and `finding_strength_radius`. Deleting the file removes these tests.
**Why it happens:** The logic is domain knowledge (paper type → color semantic), but the return type is an egui-specific type.
**How to avoid:** Before deleting, migrate the domain logic to `resyn-core/src/datamodels/`. Return a simple `[u8; 3]` (RGB tuple) or a new `PaperColor { r, g, b }` struct — no egui dependency. Re-write the 15 tests against the new type. Phase 9 maps to `egui::Color32` in the rendering layer.
**Warning signs:** Test count drops below 153 after the visualization delete.

### Pitfall 3: `cargo test` at Workspace Root Fails Without `--features ssr`
**What goes wrong:** `resyn-core`'s tests for DB, LLM, and data_aggregation are behind `ssr`. Running `cargo test` at workspace root may not enable ssr for `resyn-core`, causing compilation errors or skipped tests.
**Why it happens:** Feature flags are per-crate. Workspace-level `cargo test` runs each crate with its default features.
**How to avoid:** Test commands must be:
  - `cargo test -p resyn-core --features ssr` — all core tests (153)
  - `cargo test -p resyn-server` — server integration tests
  - `cargo build -p resyn-app --target wasm32-unknown-unknown` — WASM gate check
**Warning signs:** Test count appears low (< 153) when running from workspace root.

### Pitfall 4: `tokio` in resyn-core WASM-Safe Code
**What goes wrong:** `tokio` is used in always-available modules (e.g., a struct that has `tokio::time::Instant`). WASM targets don't support tokio's OS-level thread/timer primitives.
**Why it happens:** `tokio = { features = ["full"] }` pulls in all OS integrations.
**How to avoid:** Audit always-available modules for `tokio` imports. Only use `tokio` in `ssr`-gated modules. If timing is needed in WASM-safe code, use `std::time::Instant`.
**Warning signs:** `tokio::` anywhere in `datamodels/`, `data_processing/`, `nlp/`, `validation/`, `utils/`, `error/`.

### Pitfall 5: `crossbeam` Needs Removal Too
**What goes wrong:** `crossbeam` is in Cargo.toml but is only used in `force_graph_app.rs`. The CONTEXT.md lists egui/eframe/fdg for removal but not crossbeam explicitly.
**Why it happens:** `crossbeam` is listed as a regular (non-optional) dependency.
**How to avoid:** When deleting visualization/, also remove `crossbeam` from Cargo.toml. It has no other users.
**Warning signs:** `cargo build` warnings about unused dependency after visualization/ removal.

## Code Examples

### resyn-app Minimal Stub (WASM Compilation Gate)
```toml
# resyn-app/Cargo.toml
[package]
name = "resyn-app"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
resyn-core = { path = "../resyn-core" }
```

```rust
// resyn-app/src/lib.rs
// Phase 6: minimal stub to verify WASM compilation boundary.
// Full Leptos frontend added in Phase 8.
use resyn_core::datamodels::paper::Paper;

pub fn app_version() -> &'static str {
    "0.1.0"
}
```

### Running WASM Compilation Check
```bash
cargo build -p resyn-app --target wasm32-unknown-unknown
```
Expected: compiles with no errors. Any linker error about `ring`, `rustls`, or SurrealDB internals means the `ssr` boundary is leaking.

### Running Core Tests With SSR
```bash
cargo test -p resyn-core --features ssr
# Expected: 153 tests pass
```

### Migrating PaperColor (replaces egui::Color32)
```rust
// resyn-core/src/datamodels/enrichment.rs
use serde::{Deserialize, Serialize};

/// RGB color for paper type display. WASM-safe (no egui dependency).
/// Phase 9 maps this to egui::Color32 in the rendering layer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PaperColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PaperColor {
    pub const GRAY_UNANALYZED: Self = PaperColor { r: 140, g: 140, b: 140 };
    pub const DEFAULT: Self = PaperColor { r: 128, g: 128, b: 128 };
}

pub fn paper_type_to_color(paper_type: &str) -> PaperColor {
    match paper_type.to_lowercase().as_str() {
        "theoretical"   => PaperColor { r: 100, g: 140, b: 200 },
        "experimental"  => PaperColor { r: 90,  g: 170, b: 110 },
        "review"        => PaperColor { r: 200, g: 160, b: 70  },
        "computational" => PaperColor { r: 150, g: 100, b: 190 },
        _               => PaperColor::GRAY_UNANALYZED,
    }
}

pub fn finding_strength_radius(findings: &[crate::datamodels::llm_annotation::Finding], base: f32) -> f32 {
    let multiplier = findings.iter()
        .map(|f| match f.strength.as_str() {
            "strong_evidence"   => 3.0_f32,
            "moderate_evidence" => 2.0_f32,
            "weak_evidence"     => 1.5_f32,
            _                   => 1.0_f32,
        })
        .fold(1.0_f32, f32::max);
    base * multiplier
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single crate `research_synergy` | 3-crate workspace | This phase | Enables WASM compilation |
| Flat CLI args (all in one struct) | Subcommand enum pattern | This phase | Cleaner UX, extensible for Phase 8 |
| `egui::Color32` for paper colors | `PaperColor { r, g, b }` | This phase | Removes egui from core |
| `resolver = "1"` (implicit) | `resolver = "2"` in workspace | This phase | Prevents feature unification across targets |

**Deprecated/outdated:**
- `fdg` git dependency: removed in this phase (no replacement in Phase 6, Phase 9 builds Barnes-Hut in WASM)
- `crossbeam`: used only in force_graph_app.rs, removed with visualization/
- `egui_graphs = { version = "0.25.0", features = ["events"] }`: removed
- `eframe = "0.31.1"`: removed
- `egui = "0.31.1"`: removed

## Open Questions

1. **gap_analysis/output.rs: ssr-gated or always-available?**
   - What we know: CONTEXT.md says `output/` is behind `ssr`. Inspection shows `output.rs` only formats `String` — no DB access, no tokio, no SurrealDB.
   - What's unclear: Whether the intent was "DB write outputs" (behind ssr) or "all gap_analysis output" (behind ssr).
   - Recommendation: Keep output.rs always-available (it only formats strings). The DB persistence calls are in main.rs and move to resyn-server. This preserves 11 tests in output.rs without requiring ssr feature.

2. **`resyn serve` placeholder: include or defer?**
   - What we know: CONTEXT.md marks this as Claude's discretion.
   - What's unclear: Whether a "not yet implemented" stub helps or creates confusion.
   - Recommendation: Include `resyn serve` as a stub that prints "Server not yet implemented (Phase 8)" and exits 0. Keeps the CLI contract consistent.

3. **wiremock in resyn-core dev-dependencies under ssr**
   - What we know: wiremock tests exist in `llm/ollama.rs`, `llm/claude.rs`, `data_aggregation/text_extractor.rs`, `gap_analysis/abc_bridge.rs`, `gap_analysis/contradiction.rs` — all behind `ssr`.
   - What's unclear: Whether wiremock needs to be a conditional dev-dep or always available.
   - Recommendation: Put `wiremock` and `tokio-test` as regular dev-dependencies in resyn-core (dev-deps don't affect WASM compilation of the lib crate). Tests that require ssr modules will naturally only compile when `--features ssr` is set.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) — no external framework |
| Config file | none (Cargo.toml controls test targets) |
| Quick run command | `cargo test -p resyn-core --features ssr` |
| Full suite command | `cargo test -p resyn-core --features ssr && cargo test -p resyn-server && cargo build -p resyn-app --target wasm32-unknown-unknown` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DEBT-01 | `use resyn_core::nlp` accessible from test context | unit | `cargo test -p resyn-core --features ssr -- nlp` | ✅ (existing nlp tests) |
| DEBT-02 | Stub comment gone from ollama.rs | compile check | `cargo build -p resyn-core --features ssr` | ✅ |
| DEBT-03 | TODO.md stale checkboxes removed | manual | inspect TODO.md | ✅ |
| WEB-01 | All 153 tests pass in workspace | unit | `cargo test -p resyn-core --features ssr` | ✅ (existing tests move to core) |
| WEB-02 | resyn-app compiles to wasm32 | compile | `cargo build -p resyn-app --target wasm32-unknown-unknown` | ❌ Wave 0 (new crate) |
| WEB-05 | egui/eframe/fdg not in any Cargo.toml | compile + grep | `cargo build -p resyn-core` (no egui dep = compiles) | ✅ after deletion |

### Sampling Rate
- **Per task commit:** `cargo test -p resyn-core --features ssr`
- **Per wave merge:** `cargo test -p resyn-core --features ssr && cargo test -p resyn-server && cargo build -p resyn-app --target wasm32-unknown-unknown`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `resyn-core/Cargo.toml` — crate does not yet exist
- [ ] `resyn-app/Cargo.toml` + `resyn-app/src/lib.rs` — WASM stub crate does not yet exist
- [ ] `resyn-server/Cargo.toml` + `resyn-server/src/main.rs` — server crate does not yet exist
- [ ] Root `Cargo.toml` — must be converted from package manifest to workspace manifest
- [ ] `resyn-core/src/datamodels/enrichment.rs` — replaces visualization/enrichment.rs color logic

## Sources

### Primary (HIGH confidence)
- Official Cargo Docs: https://doc.rust-lang.org/cargo/reference/workspaces.html — workspace.dependencies, resolver="2", virtual manifests
- Official Cargo Features Docs: https://doc.rust-lang.org/cargo/reference/features.html — optional deps, cfg(feature), feature unification
- clap docs.rs: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html — subcommand derive pattern
- Source code inspection: `src/lib.rs`, `src/main.rs`, `src/visualization/enrichment.rs`, `src/llm/ollama.rs`, `TODO.md`, `Cargo.toml`

### Secondary (MEDIUM confidence)
- SurrealDB GitHub issue #3321: https://github.com/surrealdb/surrealdb/issues/3321 — confirms ring/rustls WASM incompatibility
- nickb.dev feature unification pitfall: https://nickb.dev/blog/cargo-workspace-and-the-feature-unification-pitfall/ — workspace feature unification warning

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all patterns from official Cargo/Rust docs, verified against existing Cargo.toml
- Architecture: HIGH — crate layout from CONTEXT.md, file moves verified by source code inspection
- Pitfalls: HIGH (1, 3, 4) from official docs and existing bug reports; MEDIUM (2, 5) from source code inspection

**Research date:** 2026-03-15
**Valid until:** 2026-09-15 (stable Cargo features, workspace patterns are stable; clap 4 is stable)
