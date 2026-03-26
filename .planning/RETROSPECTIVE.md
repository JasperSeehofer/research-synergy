# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.2 — Graph Rendering Overhaul

**Shipped:** 2026-03-26
**Phases:** 3 (15-17) | **Plans:** 6 | **Timeline:** 2 days (2026-03-25 → 2026-03-26)

### What Was Built
- Force simulation retuned with 5x stronger repulsion, collision separation, velocity clamping, and BFS concentric ring placement
- Canvas 2D and WebGL2 edge rendering with #8b949e color, depth-based alpha, and quad-based geometry for WebGL2
- Crisp node circles (fwidth AA in WebGL2, viewport-compensated borders in Canvas 2D) with amber seed node distinction
- Auto-fit viewport animation with lerp ease-out after force convergence, user interaction latch, and Fit button
- Priority-ordered label collision avoidance with pill/badge rendering and three-state convergence badge

### What Worked
- Pure logic modules (viewport_fit.rs, label_collision.rs) with no web-sys dependencies enabled native unit testing — 14 tests total without wasm-bindgen-test
- Renderer trait with default no-op methods (set_label_cache, set_fit_anim_active) avoided downcasting and kept renderers decoupled
- Consistent depth_alpha formula across Canvas 2D and WebGL2 — implemented once, mirrored exactly, no visual divergence
- Reusing edge shader program for seed outer ring saved a VAO/shader allocation and simplified WebGL2 state management
- UI-SPEC design contracts (new in v1.2) front-loaded visual decisions before execution — eliminated mid-plan design pivots

### What Was Inefficient
- SUMMARY.md one_liner fields for phases 16-17 were poorly structured — summary-extract couldn't parse them, requiring manual accomplishment extraction at milestone completion
- Phase 16-02 touched 18 files but 15 were pre-existing cargo fmt fixes — the commit conflated formatting cleanup with feature work
- WebGL2 quad edge geometry required understanding existing shader pipeline deeply — the research phase covered this but execution still took 12min (longest plan in milestone)

### Patterns Established
- Offscreen canvas for measureText at load time — works regardless of active renderer (Canvas2D or WebGL2)
- User interaction latch pattern (permanent boolean set on pan/wheel/zoom) for preventing auto-behavior re-trigger
- arc_to rounded rect path for web-sys compatibility instead of round_rect_with_f64
- Screen-space label rendering with dirty-flag per-frame viewport diff for performance

### Key Lessons
1. Pure logic modules (no web-sys) for graph algorithms enable fast native testing — this pattern should be the default for all graph-related features
2. UI-SPEC design contracts reduce execution friction by resolving visual decisions upfront — continue for all frontend phases
3. Separate formatting fixes from feature commits — a pre-phase `cargo fmt` pass prevents inflated diffs
4. SUMMARY.md one_liner quality matters for downstream tooling — ensure clear single-sentence summaries

### Cost Observations
- Model mix: ~20% opus (orchestration), ~80% sonnet (execution)
- Sessions: ~3 (Phase 15, Phase 16, Phase 17 + completion)
- Notable: Smallest feature milestone yet (3 phases, 6 plans) — focused scope enabled 2-day execution with no rework

---

## Milestone: v1.1.1 — Bug Fix & Polish

**Shipped:** 2026-03-24
**Phases:** 4 (11-14) | **Plans:** 4 | **Timeline:** 2 days (2026-03-23 → 2026-03-24)

### What Was Built
- SPA fallback routing via Axum ServeFile for all client-side routes
- Graph rendering fix: node spread reduced, VBO leak eliminated, DPR convention established
- Graph interaction restored via CSS pointer-events passthrough on overlay containers
- Dual-range temporal slider fixed with track transparency and value clamping

### What Worked
- All four bugs were CSS or trivial config issues — no deep Rust logic changes needed
- Phase 13 was a four-line CSS change that unblocked three requirements at once
- DPR coordinate convention documented in Phase 12 carried cleanly into Phase 13 pointer event work
- Auto-chain mode (yolo config) enabled fast execution across all four phases with no manual gates

### What Was Inefficient
- Force layout coefficients (Phase 12 blocker) required manual agent-browser debugging — would have been caught earlier with visual regression tests
- ROUTE-01/ROUTE-02 completed in Phase 11 but not checked off in REQUIREMENTS.md — discrepancy between SUMMARY frontmatter and tracking doc
- Several browser-verify checkpoints were auto-approved but should have been manually confirmed — visual bugs need visual confirmation

### Patterns Established
- Overlay passthrough pattern: `pointer-events: none` on overlay containers, `pointer-events: auto` on interactive children
- Dual-range slider: transparent tracks + shared `::before` pseudo-element for visible track
- `get_untracked()` for cross-signal reads in reactive handlers (avoids unnecessary reactivity chains)

### Key Lessons
1. CSS overlay stacking is the first thing to check when canvas interaction is broken — saves hours of debugging Rust event handlers
2. Dual-range sliders are a known CSS challenge — the MDN pattern (pointer-events on tracks/thumbs) should be the default starting point
3. Auto-chain is efficient for bugfix milestones but visual verification needs human eyes — auto-approve for structural tests, manual for rendering

### Cost Observations
- Model mix: ~30% opus (orchestration), ~70% sonnet (execution)
- Sessions: ~3 (Phase 11-12, Phase 13-14, completion)
- Notable: Smallest milestone yet (4 plans) — entire execution in 2 days with minimal context resets

---

## Milestone: v1.1 — Scale & Surface

**Shipped:** 2026-03-22
**Phases:** 5 (6-10) | **Plans:** 23 | **Timeline:** 4 days (2026-03-15 → 2026-03-18)

### What Was Built
- 3-crate Cargo workspace with WASM compilation boundary (SurrealDB feature-gated behind `ssr`)
- DB-backed resumable crawl queue with parallel workers, SSE progress, and CLI management
- Leptos CSR web UI — dashboard, papers table, gap findings, open problems, method heatmap, crawl launcher
- Full Rust/WASM graph renderer — Canvas 2D auto-upgrading to WebGL2, Barnes-Hut force layout in Web Worker
- Analysis provenance tracking with tabbed drawer and snippet highlighting
- LOD progressive-reveal and temporal year-range filtering for scale

### What Worked
- Feature-gating strategy (SurrealDB behind `ssr`) established in Phase 6 prevented WASM compilation issues through all subsequent phases
- Pure Rust/WASM stack paid off — no JS interop debugging, single language for entire pipeline
- Viewport/simulation types as pure math (no web-sys) enabled native test execution without wasm-bindgen-test
- Separating GraphData DTO from GraphState kept server/client concerns clean
- Aggregation helpers in resyn-core (WASM-safe, no ssr gate) enabled unit testing without Leptos infrastructure

### What Was Inefficient
- SCALE-01 (real depth test runs with profiling) not executed — infrastructure complete but testing deferred
- Multiple Leptos/WASM gotchas required iterative discovery (Callback.run() not .call(), register_explicit for server fns, data-cargo-features in index.html) — a Leptos integration spike before Phase 8 would have front-loaded these
- Worker crate build configuration (bin vs cdylib, Trunk worker module) required multiple attempts to get right
- Borrow checker conflicts with Leptos reactive system (RefCell + RwSignal, Arc<AtomicBool> for on_cleanup) added friction in renderer integration

### Patterns Established
- `#[cfg(feature = "ssr")]` gating for server-only code in shared crate
- PaperSource factory pattern (make_source()) for non-Clone trait objects across spawned tasks
- Named record IDs for idempotent SurrealDB operations (CREATE on same ID is no-op)
- Viewport as pure math struct for native-testable coordinate transforms
- GraphData DTO → GraphState conversion at client boundary
- Canvas 2D overlay for text labels over WebGL canvas (CSS absolute positioning)
- noop_waker_ref() + poll_next for synchronous worker bridge polling per RAF frame

### Key Lessons
1. Feature-gate the heaviest dependency (SurrealDB) behind a feature flag from the start of any workspace restructure — retroactive gating is much harder
2. Pure math types (Viewport, simulation_tick) that avoid web-sys enable fast native testing — design for this from day one
3. Leptos 0.8 has significant API surface differences from docs — spike early, don't discover gotchas in production phases
4. Web Worker + WASM requires careful build configuration — Trunk worker modules need bin targets, not cdylib
5. Separate DTO from mutable state at boundaries — keeps serialization clean and mutation explicit

### Cost Observations
- Model mix: ~15% opus (orchestration), ~85% sonnet (execution, verification)
- Sessions: ~8 (context resets during large phases 8 and 9)
- Notable: Phases 8 and 9 were the largest (7 and 5 plans respectively) — parallel wave execution helped but context management was the bottleneck

---

## Milestone: v1.0 — Analysis Pipeline

**Shipped:** 2026-03-14
**Phases:** 5 | **Plans:** 12 | **Sessions:** ~6

### What Was Built
- Text extraction pipeline with ar5iv HTML section parsing and abstract-only fallback
- Offline NLP module (TF-IDF with section weighting, corpus fingerprint caching)
- Pluggable LLM backend (Claude, Ollama, Noop providers) with per-paper SurrealDB caching
- Cross-paper gap analysis (contradiction detection via cosine similarity + finding divergence, ABC-bridge discovery via graph distance + shared terms)
- Enriched visualization (paper-type coloring, finding-strength sizing, edge tinting via custom TintedEdgeShape, Analysis panel with toggle/legend, hover tooltips)

### What Worked
- Phase-by-phase execution with clear dependencies kept each phase focused and testable
- DB migration system established early (Phase 1) paid off through Phases 2-5 with zero schema issues
- Corpus fingerprint caching pattern reused across NLP and gap analysis with independent invalidation
- Pure logic functions in enrichment.rs enabled TDD without GUI testing infrastructure
- ROADMAP plan checkboxes fell behind but SUMMARY.md + VERIFICATION.md provided reliable completion evidence

### What Was Inefficient
- ROADMAP plan checkboxes got stale for phases 2-5 — the GSD tooling updated plan counts but not individual plan checkboxes
- Phase 4 SUMMARY frontmatter didn't list requirements_completed for GAPS-01/GAPS-02 — executor should populate this
- Gap findings computed in Phase 4 are not wired into the visualization (Phase 5) — would need a follow-up to show contradictions/bridges visually
- Nyquist validation files exist for all phases but none are compliant — test coverage is present but VALIDATION.md wasn't filled

### Patterns Established
- SurrealDB SCHEMAFULL + JSON strings for complex fields (methods, findings, tfidf_vector) — works but limits server-side querying
- `pub trait + async_trait` for pluggable backends (PaperSource, LlmProvider)
- Corpus fingerprint guard pattern for idempotent recomputation
- `load_X_data()` async helper called before sync `launch_visualization()` — keeps GUI code sync
- TintedEdgeShape wrapper pattern when egui_graphs lacks set_color on edges

### Key Lessons
1. Establish the DB migration system in the first phase — every subsequent phase benefits from safe schema extension
2. Pure logic functions separated from rendering code enable TDD for visualization features
3. SurrealDB SCHEMAFULL + JSON strings is a pragmatic workaround but should be revisited if query patterns need server-side filtering on nested fields
4. Cross-phase data wiring should be verified at the integration level — Phase 4 gap findings being absent from Phase 5 visualization was caught by the integration checker but only at audit time

### Cost Observations
- Model mix: ~20% opus (orchestration), ~80% sonnet (execution, verification)
- Sessions: ~6 (one per phase + audit + completion)
- Notable: Parallel wave execution not exercised (all waves had 1 plan each) — would benefit from larger phases

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Plans | Timeline | Key Change |
|-----------|----------|--------|-------|----------|------------|
| v1.0 | ~6 | 5 | 12 | ~2 days | First milestone — established migration, caching, and trait patterns |
| v1.1 | ~8 | 5 | 23 | 4 days | Web migration — feature gating, WASM boundary, Leptos/WebGL stack |
| v1.1.1 | ~3 | 4 | 4 | 2 days | Bugfix milestone — CSS fixes, yolo auto-chain, minimal scope |
| v1.2 | ~3 | 3 | 6 | 2 days | Graph polish — UI-SPEC contracts, pure logic modules, renderer parity |

### Cumulative Quality

| Milestone | Rust LOC | Files | Key Dependencies Added |
|-----------|----------|-------|----------------------|
| v1.0 | 8,749 | ~40 | sha2, stop-words, chrono, regex |
| v1.1 | 15,859 | 90 | leptos, trunk, axum, tower-http, gloo-worker, web-sys, governor |
| v1.1.1 | ~16,000 | 90+ | (none — bugfix only) |
| v1.2 | ~25,000 | 90+ | (none — new modules, no new deps) |

### Top Lessons (Verified Across Milestones)

1. Migration system first, features second — pays compound interest through every subsequent phase
2. Pure logic functions before rendering code — enables testing without GUI infrastructure
3. Feature-gate heavy deps at workspace restructure time — prevents cascading WASM compilation issues
4. Separate DTOs from mutable state at boundaries — clean serialization, explicit mutation
5. Spike unfamiliar frameworks early — discovering API gotchas during production phases adds friction
6. CSS overlay stacking is the first diagnostic for canvas interaction bugs — verified across Phases 12-14
7. UI-SPEC design contracts before frontend execution — eliminates mid-plan visual decision pivots (verified v1.2)
