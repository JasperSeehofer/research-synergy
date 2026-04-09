# Phase 22: Paper Similarity Engine - Research

**Researched:** 2026-04-09
**Domain:** Rust/WASM similarity computation, SurrealDB persistence, Leptos frontend integration, Canvas2D/WebGL2 rendering, force simulation
**Confidence:** HIGH

## Summary

This is primarily a wiring phase, not an algorithmic one. The core cosine similarity algorithm (`cosine_similarity()`) and shared-keyword extraction (`shared_high_weight_terms()`) already exist in `resyn-core/src/gap_analysis/similarity.rs` and are tested. TF-IDF vectors (`tfidf_vector: HashMap<String, f32>`) are already stored in SurrealDB per paper in the `paper_analysis` table. The main work is: (1) a new SurrealDB table and migration for precomputed neighbor lists, (2) a new `SimilarityRepository` following existing patterns, (3) wiring similarity computation into the analysis pipeline after NLP completes, (4) a `DrawerTab::Similar` variant with a Similar tab UI, and (5) similarity edge rendering in both Canvas2D and WebGL2 renderers plus a second force model.

The trickiest decision is the dual-force-model swap (D-11): the worker currently takes an `edges: Vec<(usize, usize)>` list and runs the same force algorithm regardless of topology. Switching to similarity-based layout means populating those edges from similarity neighbors instead of citations when similarity mode is active — a clean substitution that requires no new force physics, just different edge lists.

**Primary recommendation:** New `paper_similarity` SurrealDB table keyed by `arxiv_id` holding `top_neighbors: Vec<(String, f32)>` (top-10 by score). Compute server-side after NLP stage using `get_all_analyses()`, run pairwise similarity, persist. Serve via new server fn. Render as dashed amber edges in both renderers. Force swap by replacing the edge list passed to the worker when similarity-only mode is toggled.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Ranked list showing similarity score (percentage), title, authors, year, plus 2-3 shared keywords explaining WHY papers are similar (using existing `shared_high_weight_terms()`)
- **D-02:** Clicking a similar paper navigates to it (opens its detail drawer)
- **D-03:** New `DrawerTab::Similar` variant added to existing `DrawerTab` enum (currently has Overview, Source)
- **D-04:** Dashed lines in warm color (orange/amber) to clearly distinguish from solid gray citation edges
- **D-05:** Edge thickness scales with similarity score (higher similarity = thicker)
- **D-06:** Fixed minimum similarity threshold for displaying edges (Claude picks sensible default, e.g. top-5 neighbors or score > 0.3) — no user-adjustable slider for v1.4
- **D-07:** Silent background recompute — similarity updates automatically after TF-IDF analysis completes, no toast or progress bar
- **D-08:** When TF-IDF vectors don't exist yet, the "Similar Papers" tab shows a spinner with "Waiting for TF-IDF analysis..." message
- **D-09:** Once vectors exist, similarity data is just available — no freshness indicators needed
- **D-10:** Dual-layer toggle — citation edges and similarity edges can be independently shown/hidden via graph controls
- **D-11:** Two distinct force models: citation topology (default) vs content similarity. Toggling to similarity-only mode swaps the force simulation so similar papers cluster together
- **D-12:** When both edge types are visible, citation forces drive layout (similarity edges are visual overlay only)
- **D-13:** Switching force models triggers a re-simulation with visible animation

### Claude's Discretion
- Exact amber/orange color value for similarity edges
- Dash pattern (length, gap) for similarity edges
- Fixed similarity threshold value for edge display
- Spinner styling and "waiting for TF-IDF" message copy
- Similar papers list item layout details
- Force simulation parameters for similarity-based layout

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SIM-01 | System computes pairwise cosine similarity from existing TF-IDF vectors and stores top-10 neighbors per paper | `cosine_similarity()` exists; new `paper_similarity` table migration + `SimilarityRepository::upsert_similarity()` |
| SIM-02 | User can view similar papers in a "Similar Papers" tab in the paper detail drawer | `DrawerTab::Similar` variant; new tab in `drawer.rs`; new server fn `get_similar_papers(arxiv_id)` |
| SIM-03 | User can toggle similarity edges as an overlay in the graph view | New `show_similarity` signal in `graph_controls.rs`; `EdgeType::Similarity` variant; Canvas2D + WebGL2 rendering |
| SIM-04 | Similarity is recomputed automatically after TF-IDF analysis completes | Hook into `start_analysis()` after NLP stage completes, same fingerprint-check pattern |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| surrealdb | v3 (already in Cargo.toml) | Persist `paper_similarity` table | Already the project DB; embedded, no server |
| leptos | already in Cargo.toml | Reactive frontend, new drawer tab | Project framework |
| web-sys (Canvas2D) | already in Cargo.toml | Dashed amber edge rendering | Existing Canvas2D renderer pattern |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde_json | already in Cargo.toml | Serialize neighbor list as JSON in FLEXIBLE field | Existing pattern for complex types in SurrealDB |
| chrono | already in Cargo.toml | Timestamp `computed_at` field | Existing pattern |

**No new dependencies required.** [VERIFIED: codebase grep]

## Architecture Patterns

### Recommended Project Structure (new files/changes only)

```
resyn-core/src/
├── gap_analysis/similarity.rs       EXISTING — cosine_similarity(), shared_high_weight_terms()
├── datamodels/similarity.rs         NEW — PaperSimilarity struct (top_neighbors)
├── database/
│   ├── schema.rs                    MODIFY — add migration 10 for paper_similarity table
│   └── queries.rs                   MODIFY — add SimilarityRepository

resyn-app/src/
├── app.rs                           MODIFY — add DrawerTab::Similar
├── layout/drawer.rs                 MODIFY — add Similar tab strip button + SimilarTabBody component
├── server_fns/
│   ├── analysis.rs                  MODIFY — trigger similarity recompute after NLP stage
│   └── similarity.rs                NEW — get_similar_papers(arxiv_id) server fn
├── components/graph_controls.rs     MODIFY — add show_similarity + show_citations toggles + force mode
├── graph/
│   ├── layout_state.rs              MODIFY — add show_similarity, show_citations, similarity_edges to GraphState
│   ├── canvas_renderer.rs           MODIFY — add similarity edge draw pass with dashes
│   └── webgl_renderer.rs            MODIFY — add similarity edge draw pass with dashes
└── pages/graph.rs                   MODIFY — wire new signals, force model swap on mode change
```

### Pattern 1: SurrealDB Table + Repository (migration pattern)

**What:** Add a new SCHEMAFULL table via a numbered migration, accessed via a typed repository struct.
**When to use:** Every new persisted data type in this project.
**Example (from existing `apply_migration_3`):**

```rust
// Source: resyn-core/src/database/schema.rs
async fn apply_migration_10(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS paper_similarity SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS arxiv_id ON paper_similarity TYPE string;
        DEFINE FIELD IF NOT EXISTS neighbors ON paper_similarity TYPE object FLEXIBLE;
        DEFINE FIELD IF NOT EXISTS corpus_fingerprint ON paper_similarity TYPE string;
        DEFINE FIELD IF NOT EXISTS computed_at ON paper_similarity TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_similarity_arxiv_id ON paper_similarity FIELDS arxiv_id UNIQUE;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 10 DDL failed: {e}")))?;
    Ok(())
}
```

**Note on `neighbors` field:** Use `TYPE object FLEXIBLE` (same as `tfidf_vector` in `paper_analysis`) to store `Vec<(String, f32)>` serialized as JSON. The `SurrealValue` derive won't work for `Vec<(String, f32)>` directly — serialize to `serde_json::Value` first, same as `AnalysisRecord::tfidf_vector`. [VERIFIED: resyn-core/src/database/queries.rs:422-440]

### Pattern 2: Similarity Computation (O(N²) acceptable for typical corpus sizes)

**What:** Load all `PaperAnalysis` records, compute pairwise cosine similarity, keep top-10 per paper.
**When to use:** After TF-IDF stage completes, with fingerprint guard.

```rust
// Source: resyn-core/src/gap_analysis/similarity.rs
// Pattern: iterate all analyses, compute similarity for each pair, keep top-K
fn compute_top_neighbors(analyses: &[PaperAnalysis], top_k: usize) -> Vec<PaperSimilarity> {
    analyses.iter().map(|target| {
        let mut scored: Vec<(String, f32)> = analyses.iter()
            .filter(|other| other.arxiv_id != target.arxiv_id)
            .map(|other| {
                let score = cosine_similarity(&target.tfidf_vector, &other.tfidf_vector);
                (other.arxiv_id.clone(), score)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        PaperSimilarity {
            arxiv_id: target.arxiv_id.clone(),
            top_neighbors: scored,
            corpus_fingerprint: target.corpus_fingerprint.clone(),
            computed_at: chrono::Utc::now().to_rfc3339(),
        }
    }).collect()
}
```

**Performance note:** O(N²) pairwise similarity for N papers. At N=200 papers, this is 40,000 comparisons with sparse vectors — fast in microseconds. At N=2000 this may take ~4 seconds. For v1.4 (LBD use case with typically <500 papers), no optimization needed. [ASSUMED — no profiling done in this session]

### Pattern 3: EdgeType Extension (adding similarity edge variant)

**What:** Add `EdgeType::Similarity` to the existing enum used by both renderers.
**When to use:** Any new graph edge type.

```rust
// Source: resyn-app/src/server_fns/graph.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Regular,
    Contradiction,
    AbcBridge,
    Similarity,  // NEW — amber dashed overlay
}
```

Both Canvas2D renderer (`canvas_renderer.rs`) and WebGL2 renderer (`webgl_renderer.rs`) have existing `EdgeType`-matched draw passes. Add a new pass for `EdgeType::Similarity` following the `AbcBridge` pattern (dashed, colored). [VERIFIED: resyn-app/src/graph/canvas_renderer.rs:148-177]

### Pattern 4: Canvas2D Dashed Edge Rendering

**What:** Draw dashed amber similarity edges using `set_line_dash()`.
**When to use:** Non-solid edge types.

```rust
// Source: resyn-app/src/graph/canvas_renderer.rs (AbcBridge pattern, lines 162-176)
// Adapt for Similarity edges:
if state.show_similarity {
    for edge in &state.edges {
        if edge.edge_type != EdgeType::Similarity { continue; }
        let from = match state.nodes.get(edge.from_idx) { Some(n) => n, None => continue };
        let to = match state.nodes.get(edge.to_idx) { Some(n) => n, None => continue };
        self.ctx.save();
        // Amber: #f0a030 (warm orange-amber, distinct from AbcBridge #d29922)
        self.ctx.set_stroke_style_str("#f0a030");
        // Thickness scaled by confidence: edge.confidence.unwrap_or(0.5)
        let thickness = 1.5 + edge.confidence.unwrap_or(0.5) * 2.5;
        self.ctx.set_line_width(thickness);
        let dash_array = Array::new();
        dash_array.push(&JsValue::from_f64(8.0));
        dash_array.push(&JsValue::from_f64(5.0));
        self.ctx.set_line_dash(&dash_array).unwrap();
        self.ctx.begin_path();
        self.ctx.move_to(from.x, from.y);
        self.ctx.line_to(to.x, to.y);
        self.ctx.stroke();
        self.ctx.restore();
    }
}
```

**Note:** `set_line_dash()` resets after `ctx.restore()` — no manual reset needed. [VERIFIED: MDN pattern, observed in canvas_renderer.rs AbcBridge draw pass]

### Pattern 5: Force Model Swap

**What:** When similarity-only mode is toggled, replace the edge list sent to the force worker with similarity-derived edges. Reheat alpha to trigger visible re-simulation (D-13).
**When to use:** On `force_mode` signal change in `graph.rs`.

```rust
// The force worker receives LayoutInput { nodes, edges: Vec<(usize, usize)>, alpha, ... }
// In graph.rs RAF loop, derive edges from GraphState based on active force mode:
let force_edges: Vec<(usize, usize)> = if graph_state.force_mode == ForceMode::Similarity {
    graph_state.edges.iter()
        .filter(|e| e.edge_type == EdgeType::Similarity)
        .map(|e| (e.from_idx, e.to_idx))
        .collect()
} else {
    graph_state.edges.iter()
        .filter(|e| e.edge_type == EdgeType::Regular)
        .map(|e| (e.from_idx, e.to_idx))
        .collect()
};
// On mode switch: graph_state.alpha = 0.5; (reheat for D-13 animation)
```

**Pattern for reheat:** Set `alpha` to 0.5 (mid-heat) when mode changes. This produces visible animation without full re-layout chaos. [ASSUMED — specific value is Claude's discretion per CONTEXT.md]

### Pattern 6: DrawerTab::Similar + Async Loading

**What:** A third drawer tab that fetches similar papers for the selected paper via a new server fn.
**When to use:** Following the Overview/Source tab pattern.

The existing `DrawerBody` component passes `initial_tab: DrawerTab` and maintains `let active_tab = RwSignal::new(initial_tab)`. The Similar tab body loads via a `Resource::new(|| paper_id.clone(), get_similar_papers)` call. When TF-IDF vectors don't exist, the server fn returns an empty list and the UI shows the "Waiting for TF-IDF analysis..." spinner (D-08). [VERIFIED: resyn-app/src/layout/drawer.rs]

### Pattern 7: Fingerprint-Guarded Recompute (SIM-04)

**What:** Similarity recomputation uses the same corpus fingerprint pattern as TF-IDF.
**When to use:** After the NLP stage completes in `start_analysis()`.

```rust
// Source pattern: resyn-app/src/server_fns/analysis.rs (NLP stage, lines 128-172)
// After NLP stage succeeds:
// --- Stage 2.5: Similarity computation ---
{
    use resyn_core::database::queries::{AnalysisRepository, SimilarityRepository};
    let analysis_repo = AnalysisRepository::new(&db);
    let sim_repo = SimilarityRepository::new(&db);
    let analyses = analysis_repo.get_all_analyses().await.unwrap_or_default();
    if !analyses.is_empty() {
        let fingerprint = analyses[0].corpus_fingerprint.clone();
        // Check if already computed for this fingerprint
        let already_done = sim_repo.get_similarity(&analyses[0].arxiv_id).await
            .ok().flatten()
            .map(|s| s.corpus_fingerprint == fingerprint)
            .unwrap_or(false);
        if !already_done {
            let similarities = compute_top_neighbors(&analyses, 10);
            for sim in &similarities {
                let _ = sim_repo.upsert_similarity(sim).await;
            }
        }
    }
}
```

**Critical:** This runs inline in the `tokio::spawn` background task (no new spawn needed). Silent — no progress event sent (D-07). [VERIFIED: resyn-app/src/server_fns/analysis.rs:44-334]

### Anti-Patterns to Avoid

- **Don't load TF-IDF vectors in the Similar tab directly.** Compute and store top-10 neighbors server-side. The tab only fetches pre-computed results — never does O(N²) work on a server fn call.
- **Don't add `shared_terms` computation to the server fn.** Shared terms require both papers' full TF-IDF vectors. Instead: either store them alongside neighbors at compute time, or re-derive from stored vectors in the server fn for the Similar tab display. Storing is simpler (aligns with persisted neighbor pattern).
- **Don't skip the fingerprint guard.** Re-running full pairwise similarity on every analysis pipeline run is wasteful. The fingerprint pattern (already used for TF-IDF and gap analysis) prevents this.
- **Don't add a new `DrawerOpenRequest` field for the Similar tab.** `initial_tab: DrawerTab` already handles initial tab selection. [VERIFIED: resyn-app/src/app.rs:26-30]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cosine similarity | Custom dot-product + magnitude | `cosine_similarity()` in `gap_analysis/similarity.rs` | Already implemented and tested with 4 unit tests |
| Shared keyword extraction | Custom set intersection | `shared_high_weight_terms()` same file | Already implemented and tested |
| Force simulation | New physics engine | Existing `forces::simulation_tick()` + worker bridge | Force model swap only changes edges input, not physics |
| Database persistence boilerplate | Custom query builder | `SurrealValue` derive + `UPSERT type::record(...)` pattern | Proven pattern across 9 existing tables |
| WASM dashed-line rendering | Custom line renderer | `ctx.set_line_dash()` Canvas2D API | Already used for AbcBridge edges |

## Common Pitfalls

### Pitfall 1: FLEXIBLE type SERDE in SurrealDB
**What goes wrong:** Using `Vec<(String, f32)>` as a `SurrealValue` field type fails because tuples don't derive `SurrealValue`. The compiler error is cryptic.
**Why it happens:** SurrealDB's `SurrealValue` derive only handles primitives and structs, not `Vec<(T, U)>`.
**How to avoid:** Store `neighbors` as `serde_json::Value` (same as `tfidf_vector` in `AnalysisRecord`). Serialize to JSON on write, deserialize on read.
**Warning signs:** Compile error on `SurrealValue` derive mentioning tuple types. [VERIFIED: resyn-core/src/database/queries.rs:417-456]

### Pitfall 2: WebGL2 Dashed Line Rendering
**What goes wrong:** WebGL2 has no native `set_line_dash()`. The Canvas2D dashed-line approach does not transfer to `webgl_renderer.rs`.
**Why it happens:** WebGL renders lines as solid primitives by default.
**How to avoid:** In the WebGL2 renderer, implement dash simulation via fragment shader: use `gl_FragCoord` or pass a `v_length` varying to discard alternating fragments. Alternatively, render dashed similarity edges using the existing Canvas2D overlay (label canvas) rather than WebGL2.
**Best approach for this phase:** The graph page already has a separate `label_canvas` (2D context) rendered on top of the WebGL2 canvas. Drawing similarity edges on the label canvas is the lowest-risk path that avoids shader complexity. [VERIFIED: resyn-app/src/pages/graph.rs:140-151]
**Warning signs:** Similarity edges appear solid in WebGL2 mode but dashed in Canvas2D mode.

### Pitfall 3: O(N²) pairwise computation blocking the analysis pipeline
**What goes wrong:** `compute_top_neighbors()` is called inline in the `tokio::spawn` background task. For large corpora, this could noticeably delay `analysis_complete` event.
**Why it happens:** N=500 papers → 250,000 comparisons. With sparse HashMaps, each comparison iterates only overlapping terms — fast in practice, but untested at scale.
**How to avoid:** Keep the fingerprint guard so recomputes don't happen on every analysis run. This is the dominant protection. No optimization needed for v1.4.
**Warning signs:** `analysis_complete` SSE event is delayed by several seconds after TF-IDF stage.

### Pitfall 4: `EdgeType::Similarity` breaks `match` exhaustiveness
**What goes wrong:** Adding `EdgeType::Similarity` without updating all `match edge.edge_type` sites causes compile errors.
**Why it happens:** Rust exhaustiveness checking. Match sites exist in: `canvas_renderer.rs` (arrowhead visibility check line ~192), `webgl_renderer.rs` (similar match), `layout_state.rs` (edge-building), and potentially `graph.rs`.
**How to avoid:** Use `cargo check` immediately after adding the variant. Add `EdgeType::Similarity => false` (no arrowhead for similarity edges) to the arrowhead pass.
**Warning signs:** `cargo check` fails with "non-exhaustive patterns" after adding the variant.

### Pitfall 5: Similar tab data fetches on every drawer open
**What goes wrong:** `Resource::new(|| paper_id.clone(), get_similar_papers)` re-fetches every time the paper_id changes, even if the drawer was just closed and reopened for the same paper.
**Why it happens:** Leptos `Resource` keyed on paper_id doesn't cache across drawer close/open cycles.
**How to avoid:** Accept this behavior for v1.4 — it is consistent with how `get_paper_detail` works in the existing Overview/Source tabs. The server fn cost is just a SurrealDB point lookup.

### Pitfall 6: Force model swap without alpha reheat
**What goes wrong:** Toggling similarity-only mode changes the edge list but the simulation may already be converged (alpha < ALPHA_MIN). The layout doesn't update visibly.
**Why it happens:** `check_alpha_convergence()` sets `simulation_running = false` when alpha drops below 0.001. A new edge list has no effect if the simulation is paused.
**How to avoid:** On force mode toggle: set `graph_state.alpha = 0.5` AND `graph_state.simulation_running = true` before the next RAF tick. This produces the visible animation D-13 requires. [VERIFIED: resyn-app/src/graph/layout_state.rs:241-248]

## Code Examples

### SurrealDB record ID pattern for new table
```rust
// Source: resyn-core/src/database/queries.rs (pattern from analysis_record_id)
fn similarity_record_id(arxiv_id: &str) -> RecordId {
    RecordId::new("paper_similarity", strip_version_suffix(arxiv_id))
}
```

### Server fn for Similar tab data
```rust
// Source: resyn-app/src/server_fns/ (new file similarity.rs, following analysis.rs pattern)
#[server(GetSimilarPapers, "/api")]
pub async fn get_similar_papers(arxiv_id: String) -> Result<Vec<SimilarPaperEntry>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::{AnalysisRepository, SimilarityRepository, PaperRepository};
        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        // Returns empty vec (not error) when no similarity data exists yet — D-08
        let sim_repo = SimilarityRepository::new(&db);
        let sim = sim_repo.get_similarity(&arxiv_id).await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        match sim {
            None => Ok(vec![]),  // Triggers "Waiting for TF-IDF analysis..." spinner in UI
            Some(paper_sim) => {
                // Enrich with paper metadata and shared terms
                // ...
                Ok(entries)
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
```

### Drawer Similar tab spinner (D-08)
```rust
// When get_similar_papers returns empty vec:
view! {
    <div class="similar-waiting-state">
        <div class="spinner"></div>
        <p class="text-body text-muted">"Waiting for TF-IDF analysis..."</p>
        <p class="text-label text-muted">"Run analysis to compute paper similarity."</p>
    </div>
}
```

### Graph controls similarity toggle
```rust
// New button in graph_controls.rs, following show_contradictions pattern
<button
    class=move || if show_similarity.get() { "graph-control-btn active" } else { "graph-control-btn" }
    on:click=move |_| show_similarity.update(|v| *v = !*v)
    aria-pressed=move || show_similarity.get().to_string()
    aria-label="Toggle similarity edges"
>
    "Similarity"
</button>
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| On-demand cosine similarity at query time | Precomputed top-K neighbors stored in DB | Phase 22 design | Drawer tab fetch is O(1) point lookup |
| Single force model (citation topology) | Switchable citation / similarity force models | Phase 22 design | Two fundamentally different graph perspectives |

**Deprecated/outdated:**
- None for this phase.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | O(N²) pairwise similarity is fast enough for typical corpus sizes (<500 papers) | Architecture Patterns, Pattern 2 | Could slow analysis pipeline; fix by adding async chunking or limiting corpus size |
| A2 | Alpha reheat to 0.5 provides good force-swap animation (D-13) | Anti-Patterns, Pitfall 6 | Visual result may be too jarring or too subtle; tune in implementation |

## Open Questions (RESOLVED)

1. **Shared terms storage vs. on-demand computation**
   - What we know: `shared_high_weight_terms()` requires both papers' full `tfidf_vector` (HashMap<String, f32>). The Similar tab needs 2-3 shared terms per neighbor (D-01).
   - What's unclear: Compute+store shared terms at similarity-compute time (adds ~KB per paper to DB), or load target+neighbor vectors in server fn and compute on request?
   - Recommendation: Store top-3 shared terms alongside each neighbor in the `neighbors` JSON at compute time. Avoids N extra DB fetches per drawer open.
   - RESOLVED: Store top-3 shared terms at compute time.

2. **WebGL2 dashed similarity edges**
   - What we know: WebGL2 has no native line dash; label canvas (2D) is already rendered above WebGL canvas.
   - What's unclear: Should we draw similarity edges on the label canvas overlay (simpler) or implement dash simulation in the WebGL2 shader (correct but complex)?
   - Recommendation: Use the label canvas overlay for similarity edges in WebGL2 mode. This is the approach least likely to introduce rendering bugs and is consistent with how the label canvas is already used.
   - RESOLVED: Use label canvas overlay for WebGL2 mode.

## Environment Availability

Step 2.6: SKIPPED (no external dependencies beyond the existing Rust toolchain and SurrealDB embedded runtime, both already verified working by the existing 330-test suite passing).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | Cargo.toml (workspace) |
| Quick run command | `cargo test --lib similarity` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SIM-01 | `compute_top_neighbors()` returns top-10 sorted by descending score | unit | `cargo test -p resyn-core similarity::tests` | ❌ Wave 0 |
| SIM-01 | `SimilarityRepository::upsert_similarity` + `get_similarity` roundtrip | DB | `cargo test -p resyn-core similarity_db` | ❌ Wave 0 |
| SIM-01 | Fingerprint guard prevents recompute on unchanged corpus | unit | `cargo test -p resyn-core similarity_fingerprint` | ❌ Wave 0 |
| SIM-02 | `get_similar_papers` returns empty vec when no similarity data exists | unit | `cargo test -p resyn-core get_similar_no_data` | ❌ Wave 0 |
| SIM-04 | Similarity recompute triggered after NLP with new fingerprint | integration | `cargo test -p resyn-core analysis_pipeline_test` | ✅ (extend) |

### Sampling Rate
- **Per task commit:** `cargo test -p resyn-core`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `resyn-core/src/datamodels/similarity.rs` — `PaperSimilarity` struct (new datamodel)
- [ ] `resyn-core/src/gap_analysis/similarity.rs` — add `compute_top_neighbors()` function + tests
- [ ] `resyn-core/src/database/queries.rs` — add `SimilarityRepository` struct + tests (upsert/get/roundtrip)
- [ ] `resyn-core/src/database/schema.rs` — add `apply_migration_10` + `paper_similarity` table DDL

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | n/a — single-user tool |
| V3 Session Management | no | n/a |
| V4 Access Control | no | n/a |
| V5 Input Validation | yes | `arxiv_id` param in `get_similar_papers` server fn — validate/strip using `strip_version_suffix()` before DB lookup |
| V6 Cryptography | no | no new crypto; corpus fingerprint uses existing SHA-256 via `sha2` crate |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed `arxiv_id` in `get_similar_papers` server fn | Tampering | Apply `strip_version_suffix()` before DB query; SurrealDB parameterized queries already used |
| No new attack surface | — | All similarity data is read-only from server fn perspective; writes only from authenticated pipeline |

## Sources

### Primary (HIGH confidence)
- `resyn-core/src/gap_analysis/similarity.rs` — `cosine_similarity()`, `shared_high_weight_terms()` — directly read and verified
- `resyn-core/src/database/queries.rs` — `AnalysisRepository`, `SurrealValue` FLEXIBLE pattern — directly read and verified
- `resyn-core/src/database/schema.rs` — migration pattern, current schema at migration 9 — directly read and verified
- `resyn-app/src/layout/drawer.rs` — `DrawerTab` enum, tab strip pattern — directly read and verified
- `resyn-app/src/graph/canvas_renderer.rs` — dashed AbcBridge edge pattern — directly read and verified
- `resyn-app/src/server_fns/analysis.rs` — NLP stage, fingerprint-guarded computation, `tokio::spawn` pipeline — directly read and verified
- `resyn-app/src/graph/layout_state.rs` — `GraphState`, `check_alpha_convergence()`, force model implications — directly read and verified
- `resyn-worker/src/forces.rs` — `simulation_tick()` edge-list input, alpha decay/ALPHA_MIN — directly read and verified
- `resyn-app/src/pages/graph.rs` — label canvas overlay above WebGL canvas — directly read and verified

### Secondary (MEDIUM confidence)
- WebGL2 dashed line limitation — well-known WebGL behavior; fragment shader workaround is standard approach

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all existing
- Architecture: HIGH — all integration points verified by direct code reading
- Pitfalls: HIGH — derived from direct code reading (exhaustiveness checking, SurrealDB FLEXIBLE type pattern, Canvas2D vs WebGL2 dashes)

**Research date:** 2026-04-09
**Valid until:** 2026-05-09 (stable stack; 30-day estimate)
