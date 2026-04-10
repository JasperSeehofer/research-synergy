# Phase 24: Community Detection - Research

**Researched:** 2026-04-10
**Domain:** Louvain community detection, c-TF-IDF, ColorMode rendering, drawer extension
**Confidence:** HIGH

## Summary

Phase 24 introduces Louvain modularity-based community detection on the citation graph, stored in a new SurrealDB table, and surfaces communities through three UI entry points: a new `ColorMode` dropdown in graph controls, a community legend in the controls overlay, and a new `DrawerTab::Community` with full summary content (top papers, keywords, shared methods).

The most significant technical decision is the Louvain crate choice. `single-clustering` v0.6.1 has a confirmed `Louvain::new(resolution, seed: Option<u64>)` API with a `Network<T,T>` backed by petgraph `UnGraph`, making it well-suited. However, it requires `petgraph ^0.8.2` while the workspace pins `petgraph = "0.7.0"`. Cargo can link two separate petgraph versions simultaneously (they are different crate instances), so the workspace petgraph and single-clustering's petgraph will coexist without conflict — but the bridge between them requires building the `Network` from scratch rather than converting a `StableGraph` directly. The graph_creation layer already produces edge lists that can drive this construction.

The renderer refactor (node fill pipeline in both `canvas_renderer.rs` and `webgl_renderer.rs`) is the largest UI task: currently fills are four hardcoded branches; they must become data-driven via `ColorMode` and a per-node `current_color: [f32; 3]` / `target_color: [f32; 3]` lerp state. The UI-SPEC is fully approved, providing exact hex values, component markup patterns, copywriting, and accessibility requirements — no open design questions remain.

**Primary recommendation:** Use `single-clustering` for Louvain with a custom `Network` construction pass from the existing edge-list iterator; implement c-TF-IDF by aggregating existing `PaperAnalysis.tfidf_vector` values per community as a pure computation step (no new extraction).

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Use an existing Rust crate for Louvain (researcher evaluates in RESEARCH.md and locks one; fall back to in-house only if no mature crate cleanly accepts petgraph::StableGraph)
- **D-02:** Run Louvain on the **undirected** projection of the citation graph
- **D-03:** Fixed RNG seed for Louvain (successive recomputations on same corpus yield identical community IDs)
- **D-04:** Communities with fewer than 3 papers are bucketed into a single "Other" community (neutral gray)
- **D-05:** New `graph_communities` table (planner picks exact name, e.g. `community_assignments`) — `{paper_id, community_id, corpus_fingerprint}` — invalidated by `corpus_fingerprint` changes
- **D-06:** Auto-compute after crawl completes (matching Phase 23 metrics pattern); manual "Recompute" trigger; "Computing..." disabled-option UX when community data not yet available
- **D-07:** New `ColorMode` enum with three modes: Community, BFS Depth, Topic — all shipped in this phase
- **D-08:** "Color by" dropdown added to `graph_controls.rs` alongside "Size by" dropdown; `RwSignal<ColorMode>`; parallel to `size_mode`
- **D-09:** Default mode on first load: Community
- **D-10:** `Color by = Community` and Topic Rings toggle remain independent (rings + fills can both be on)
- **D-11:** Categorical palette — Tableau 10 (exact hex in UI-SPEC); cycling for >10; "Other" uses neutral gray
- **D-12:** Lerp color transitions (~300ms) when switching ColorMode, consistent with Phase 23's size lerp
- **D-13:** BFS Depth color mode: warm→cool scale (exact hex in UI-SPEC)
- **D-14:** Topic color mode: reuse existing `palette: Vec<PaletteEntry>` from topic-ring pipeline
- **D-15:** New `DrawerTab::Community` variant (fourth tab; existing: Overview/Source/Similar)
- **D-16:** Two entry points — paper-selected mode (tab available in normal drawer) and legend-click mode (opens drawer directly on Community tab)
- **D-17:** Legend-click with no paper selected — drawer shows community-level content only; other tabs show "Select a paper to view details." placeholder
- **D-18:** Community legend rendered in graph controls when `ColorMode::Community` is active
- **D-19:** Top papers ranked by hybrid score: `PageRank × community-internal degree`
- **D-20:** Show top 5 papers per community
- **D-21:** Dominant keywords via c-TF-IDF (class-based TF-IDF) using existing per-paper TF-IDF vectors
- **D-22:** Show top 10 keywords per community
- **D-23:** "Shared methods" via existing `shared_high_weight_terms()` helper aggregated across community members
- **D-24:** Community auto-label from top 1–2 c-TF-IDF keywords (e.g. "quantum decoherence"); no numeric labels

### Claude's Discretion

- Exact Louvain crate choice (locked below in this RESEARCH.md)
- Fixed RNG seed value
- Exact categorical palette hex values and mapping (locked in UI-SPEC)
- Drawer tab layout details (card order, typography, spacing — locked in UI-SPEC)
- "Select a paper to view" placeholder copy (locked in UI-SPEC)
- Exact lerp curve parameters (locked in UI-SPEC: `1 - (1 - 0.95)^(dt/300ms)`)
- Implementation strategy for c-TF-IDF (pure aggregation vs. existing `nlp/tfidf.rs` extension)
- How many keywords `shared_high_weight_terms()` surfaces per community
- Whether community membership appears on Overview tab (optional chip)
- Fallback behavior for graphs too small (<10 nodes)
- Louvain resolution / modularity tuning parameters

### Deferred Ideas (OUT OF SCOPE)

- User-adjustable Louvain resolution slider
- Community comparison view ("how do Community A and Community B differ?")
- Community membership history / evolution
- Cross-community bridge detection as standalone surfacing (Phase 25 DISC-04)
- Richer "methods" extraction beyond `shared_high_weight_terms()` (e.g. NER)
- Community-scoped search/filter (potential v1.5)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| COMM-01 | System detects communities in the citation graph using Louvain modularity optimization | `single-clustering::community_search::louvain::Louvain` with `Network` construction from edge-list; stored in new `graph_communities` SurrealDB table (migration 12) |
| COMM-02 | User can color graph nodes by community via a "Color by" dropdown (BFS Depth / Community / Topic) | New `ColorMode` enum in `layout_state.rs`; `color_mode: RwSignal<ColorMode>` in `pages/graph.rs`; dropdown in `graph_controls.rs`; per-node `current_color`/`target_color` lerp state in renderers |
| COMM-03 | User can view a community summary panel showing top papers, dominant keywords, and shared methods per community | `DrawerTab::Community` in `app.rs`; drawer content queries new server fns for community data; c-TF-IDF aggregation in `resyn-core` |
</phase_requirements>

---

## Standard Stack

### Louvain Crate Decision (D-01 lock)

**Locked choice: `single-clustering` v0.6.1**

| Property | Value |
|----------|-------|
| Crate | `single-clustering` v0.6.1 |
| Published | 2025-09-11 [VERIFIED: crates.io API] |
| API | `Louvain::new(resolution: T, seed: Option<u64>)` [VERIFIED: docs.rs] |
| Seed support | Yes — `seed: Some(42_u64)` satisfies D-03 |
| Graph type | `Network<T,T>` built from `build_network(n_nodes, n_edges, adj_iter)` — NOT petgraph directly |
| Result type | `VectorGrouping` — maps node index to community id |
| petgraph dep | `^0.8.2` [VERIFIED: crates.io dependency API] |
| License | Not listed on crates.io (check before using in commercial context) |

**Why chosen over alternatives:**

| Candidate | Status | Reason |
|-----------|--------|--------|
| `graphalgs` v0.2.0 | Rejected | No Louvain implementation — provides MST, shortest path, metrics only [VERIFIED: docs.rs] |
| `single-clustering` v0.6.1 | Selected | Has Louvain + Leiden, confirmed seed param, petgraph-backed Network |
| `fa-leiden-cd` v0.1.0 | Not selected | Leiden only, custom Graph type (not petgraph), determinism unclear |
| Custom in-house | Fallback | Only if single-clustering integration fails; D-01 permits this |

**petgraph version note:** The workspace pins `petgraph = "0.7.0"`. `single-clustering` requires `petgraph ^0.8.2`. Cargo resolves this by linking both versions simultaneously — no conflict. However, the project's `StableGraph` (petgraph 0.7) cannot be passed directly to `single-clustering`'s `Network` (which uses petgraph 0.8 `UnGraph` internally). The integration approach is: **extract edge pairs from the existing `StableGraph` and use `Louvain::build_network(n_nodes, n_edges, edge_iter)` to construct the `Network` from scratch.** This is straightforward since `create_graph_from_papers()` already produces a graph whose edges can be iterated as `(NodeIndex, NodeIndex)` pairs. [VERIFIED: crates.io API + docs.rs]

**Discretion: Fixed seed value = `42_u64`** (conventional, memorable, no functional reason to pick otherwise).

**Discretion: Resolution = `1.0_f64`** (Louvain default; higher values produce more fine-grained communities, lower values produce coarser clusters. 1.0 is the standard modularity optimization target).

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `single-clustering` | 0.6.1 | Louvain community detection | Confirmed seed/resolution API; petgraph-backed Network |
| `petgraph` | 0.7.0 (workspace) | Graph data structure | Already in use; edge iteration drives Network construction |
| `surrealdb` | 3 (workspace) | Persist community assignments | Existing DB layer; migration 12 pattern |
| `rand` | 0.9.0 (workspace) | (Not needed — seed passed to Louvain directly) | Already in workspace |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `sha2` | 0.10 (workspace) | Corpus fingerprint | Already used in `tfidf.rs`; reuse same `corpus_fingerprint()` fn |

**Installation:**
```bash
# Add to resyn-core Cargo.toml [dependencies] (NOT behind ssr feature — Louvain is pure computation, no IO)
# single-clustering = "0.6.1"
# Note: must be added under [dependencies], not [features]-gated, as computation runs server-side
# but the crate itself has no ssr constraint
```

**Version verification:** `single-clustering` v0.6.1 confirmed as latest as of 2026-04-10 via `cargo info single-clustering`. [VERIFIED: cargo registry]

---

## Architecture Patterns

### Recommended Project Structure (new files only)

```
resyn-core/src/
├── graph_analytics/
│   ├── community.rs        # NEW: Louvain runner, c-TF-IDF, hybrid ranking, community datamodel
│   └── mod.rs              # MODIFIED: pub mod community;
├── datamodels/
│   └── community.rs        # NEW: CommunityAssignment, CommunityLabel structs
└── database/
    ├── schema.rs           # MODIFIED: apply_migration_12 for graph_communities table
    └── queries.rs          # MODIFIED: CommunityRepository (upsert, get_by_paper, get_all)

resyn-app/src/
├── graph/
│   └── layout_state.rs     # MODIFIED: ColorMode enum, NodeState color fields, GraphState.color_mode
├── graph/
│   ├── canvas_renderer.rs  # MODIFIED: node fill logic reads ColorMode + community assignments
│   └── webgl_renderer.rs   # MODIFIED: same node fill refactor
├── components/
│   └── graph_controls.rs   # MODIFIED: ColorMode dropdown, community legend section
├── pages/
│   └── graph.rs            # MODIFIED: color_mode signal, communities_ready/computing signals
├── layout/
│   └── drawer.rs           # MODIFIED: DrawerTab::Community, legend-click entry handling
├── app.rs                  # MODIFIED: DrawerTab enum + DrawerOpenRequest (optional community_id field)
└── server_fns/
    └── community.rs        # NEW: get_communities_status, get_community_summary, trigger_community_compute
```

### Pattern 1: Louvain Runner (resyn-core)

**What:** Build undirected Network from StableGraph edges, run Louvain, bucket "Other", store assignments.

**When to use:** After crawl completes (auto) or on manual recompute trigger.

```rust
// Source: single-clustering docs.rs + project pattern from graph_analytics/pagerank.rs
use single_clustering::community_search::louvain::Louvain;
use single_clustering::network::Network;

pub fn detect_communities(
    graph: &StableGraph<Paper, f32, Directed, u32>,
    seed: u64,
    resolution: f64,
    min_size: usize,  // D-04: bucket < min_size into "Other"
) -> HashMap<String, u32> {
    let n_nodes = graph.node_count();
    if n_nodes < 10 {
        // Fallback: too small for meaningful communities
        return HashMap::new();
    }
    // Collect node index -> arxiv_id mapping
    let idx_to_id: Vec<String> = graph.node_indices()
        .map(|ix| strip_version_suffix(&graph[ix].id))
        .collect();
    // Build undirected edge iterator (deduplicate direction)
    let edges: Vec<(u32, u32)> = graph.edge_indices()
        .map(|e| {
            let (a, b) = graph.edge_endpoints(e).unwrap();
            (a.index() as u32, b.index() as u32)
        })
        .collect();
    let n_edges = edges.len();
    let network: Network<f64, f64> = Louvain::build_network(n_nodes, n_edges, edges.into_iter());
    let mut louvain = Louvain::new(resolution, Some(seed));
    let mut clustering = VectorGrouping::new(n_nodes);
    louvain.iterate(&network, &mut clustering);
    // Map raw community IDs -> arxiv_ids; bucket small communities
    // ... bucket logic per D-04 ...
}
```

[ASSUMED] The exact `VectorGrouping` constructor and `iterate` return types — verify against `single-clustering` source when integrating.

### Pattern 2: c-TF-IDF Aggregation (resyn-core)

**What:** Treat each community as a pseudo-document by summing per-paper TF-IDF vectors within it, then divide each term by its presence across other communities.

**Formula (c-TF-IDF):**
```
tf_community(t) = sum(tfidf(t, paper) for paper in community) / |community|
idf_community(t) = ln(|communities| / count(communities containing t)) + 1
c_tfidf(t, community) = tf_community(t) * idf_community(t)
```

**When to use:** After Louvain assignment completes; inputs are `Vec<PaperAnalysis>` already in DB.

**Example:**
```rust
// Source: [ASSUMED] — c-TF-IDF formula from Manning et al. "Introduction to Information Retrieval"
// Input: community_members: HashMap<u32, Vec<&PaperAnalysis>>  (community_id -> analyses)
fn compute_ctfidf(
    community_members: &HashMap<u32, Vec<&PaperAnalysis>>,
) -> HashMap<u32, Vec<(String, f32)>> {
    let n_communities = community_members.len() as f32;
    // Step 1: compute mean TF per term per community
    // Step 2: compute document frequency across communities
    // Step 3: apply IDF weighting
    // Step 4: sort by score descending, take top 10
    todo!()
}
```

### Pattern 3: ColorMode Enum + NodeState Color Fields

**What:** Extend `layout_state.rs` with `ColorMode` enum and per-node color lerp state.

**When to use:** On `GraphState::from_graph_data()` init and on mode switch.

```rust
// Source: CONTEXT.md D-07, D-12; UI-SPEC Implementation Notes §5
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ColorMode {
    #[default]
    Community,   // D-09: default on first load
    BfsDepth,
    Topic,
}

// Add to NodeState:
pub current_color: [f32; 3],  // smoothly interpolated display color
pub target_color: [f32; 3],   // color for current ColorMode
pub community_id: Option<u32>, // set when community data loaded

// Lerp factor per UI-SPEC §5:
// lerp_factor = 1.0 - (1.0 - 0.95_f32).powf(dt_ms / 300.0)
// current_color[i] += (target_color[i] - current_color[i]) * lerp_factor
```

### Pattern 4: Community Legend (graph_controls.rs)

**What:** Conditional section below the "Color by" dropdown, shown when `ColorMode::Community` is active. Each chip is a button that legend-clicks open the Community drawer.

**Template:** Mirror of existing `topic-legend-section` block at ~line 252 of `graph_controls.rs`.

```rust
// Source: existing graph_controls.rs lines 252-297
{move || {
    if color_mode.get() == ColorMode::Community && !communities.get().is_empty() {
        Some(view! {
            <div class="topic-legend-section">
                <div class="sidebar-title">"COMMUNITY COLORS"</div>
                <div class="topic-legend-entries">
                    {communities.get().into_iter().map(|c| {
                        // chip with swatch + keyword label
                        // on:click -> legend-click drawer open
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        })
    } else { None }
}}
```

### Pattern 5: SurrealDB Migration 12

**What:** New `graph_communities` table with corpus fingerprint invalidation.

```rust
// Source: existing schema.rs apply_migration_11 pattern
async fn apply_migration_12(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query("
        DEFINE TABLE IF NOT EXISTS graph_communities SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS arxiv_id ON graph_communities TYPE string;
        DEFINE FIELD IF NOT EXISTS community_id ON graph_communities TYPE int;
        DEFINE FIELD IF NOT EXISTS corpus_fingerprint ON graph_communities TYPE string;
        DEFINE FIELD IF NOT EXISTS computed_at ON graph_communities TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_communities_arxiv_id ON graph_communities FIELDS arxiv_id UNIQUE;
        DEFINE INDEX IF NOT EXISTS idx_communities_community_id ON graph_communities FIELDS community_id;
    ").await
    .map_err(|e| ResynError::Database(format!("migration 12 DDL failed: {e}")))?;
    Ok(())
}
```

### Pattern 6: Community Summary Server Fn

**What:** New server fn in `server_fns/community.rs` that fetches full community summary for the drawer.

```rust
// Source: existing server_fns/metrics.rs pattern
#[server(GetCommunitySummary, "/api")]
pub async fn get_community_summary(community_id: u32) -> Result<CommunitySummary, ServerFnError> {
    // Loads: top 5 papers (PageRank x intra-community degree hybrid score)
    //        top 10 c-TF-IDF keywords
    //        shared methods from shared_high_weight_terms()
    //        community auto-label (top 1-2 keywords)
}
```

### Anti-Patterns to Avoid

- **Building petgraph 0.8 StableGraph for single-clustering:** single-clustering's `Network` is petgraph 0.8 `UnGraph`-backed, but the project's `StableGraph` is petgraph 0.7. Don't try to convert directly — build the `Network` from edge-list iteration using `Louvain::build_network()`.
- **Computing community data on the read path:** Following Phase 22/23 convention, community data is always pre-computed and cached. Dashboard/drawer server fns never trigger compute — they read from `graph_communities` table.
- **Iterating all papers in the drawer server fn:** Use indexed DB queries (`SELECT * FROM graph_communities WHERE community_id = $cid`) rather than loading all assignments and filtering in Rust.
- **Non-deterministic community ordering:** Community-to-color mapping is by size descending (largest community = index 0). This must be computed from the stored assignments, not from Louvain's raw output order. [VERIFIED: UI-SPEC color section]
- **Lerping color in CSS:** Node fill colors are computed in Rust and passed as `[u8; 3]` RGB to both renderers. No CSS variables for community colors. [VERIFIED: UI-SPEC Implementation Notes §3]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Louvain modularity optimization | Custom Louvain impl | `single-clustering::Louvain` | Correct phase-detection, RNG seeding, convergence handled |
| Corpus fingerprint | Custom hash | `nlp::tfidf::corpus_fingerprint()` | Already exists; SHA-256 over sorted paper IDs |
| PageRank scores for hybrid ranking | Recompute PageRank | Read from `graph_metrics` table (Phase 23) | D-19 uses stored scores; no recomputation needed |
| Per-paper TF-IDF vectors | Re-extract text | Read from `paper_analysis` table (Phase 22) | D-21 c-TF-IDF inputs are already stored |
| Shared methods helper | Term aggregation | `gap_analysis::similarity::shared_high_weight_terms()` | D-23 explicitly mandates reuse |
| Topic fill colors | New palette computation | Existing `GraphState.palette: Vec<PaletteEntry>` | D-14; palette already loaded in GraphState |

**Key insight:** Phase 24's computation layer is almost entirely recombination of existing data (PageRank scores, TF-IDF vectors, shared_high_weight_terms). The only genuinely new computation is Louvain itself and the c-TF-IDF aggregation step.

---

## Common Pitfalls

### Pitfall 1: petgraph Version Collision Manifesting at Runtime

**What goes wrong:** If single-clustering's internal `UnGraph` type is accidentally exposed through the public API and the calling code (petgraph 0.7) tries to use it, you get type mismatch errors at compile time.

**Why it happens:** Cargo links both petgraph versions but they are distinct crate instances — types from one version are incompatible with the other.

**How to avoid:** Keep the Louvain integration entirely behind an internal function in `graph_analytics/community.rs` that takes `&StableGraph<Paper, f32, Directed, u32>` and returns `HashMap<String, u32>`. Never expose single-clustering types in public API signatures.

**Warning signs:** Compiler error mentioning "expected `petgraph_0_7::...`, found `petgraph_0_8::...`".

### Pitfall 2: Non-Deterministic Community IDs Across Recomputations

**What goes wrong:** If the petgraph `StableGraph` node indices are not stable (e.g., nodes were removed and re-added), the same paper may get a different integer index on different runs, causing different Louvain community IDs even with the same seed.

**Why it happens:** `StableGraph` preserves indices across deletions, but node order depends on insertion order. If the DB load order changes (e.g., SurrealDB returns papers in different order), the network adjacency changes.

**How to avoid:** Sort papers by `arxiv_id` before constructing the Network to ensure stable ordering. The Network node index for each paper should be determined by sorted arxiv_id position, not DB return order.

**Warning signs:** Community colors flickering across page reloads even with fixed seed.

### Pitfall 3: Community Count Explosion on Sparse Graphs

**What goes wrong:** Louvain on a very sparse citation graph (few edges relative to nodes) may produce O(N) singleton communities instead of meaningful clusters.

**Why it happens:** With few edges, modularity is maximized by treating each node as its own community.

**How to avoid:** D-04 buckets communities with <3 papers into "Other". Additionally, the <10 node fallback (discretion item) prevents running Louvain on trivially small graphs.

**Warning signs:** Legend showing 20+ community chips on a 30-node graph.

### Pitfall 4: c-TF-IDF Surfacing Stop-Words or Low-Signal Terms

**What goes wrong:** Community keyword labels like "quantum", "the", "system" that are too broad to be distinctive.

**Why it happens:** c-TF-IDF uses IDF across communities — if a term appears in every community, its IDF is near zero. But if all papers are quantum physics, "quantum" appears in every community and gets IDF ≈ 0.

**How to avoid:** Apply the same stop-words set used in `nlp/preprocessing.rs` before c-TF-IDF aggregation. The existing `build_stop_words()` function handles this; terms already absent from `tfidf_vector` (filtered during Phase 22 TF-IDF computation) will not appear.

**Warning signs:** Community names like "quantum" or "we" dominating all labels.

### Pitfall 5: Drawer State for Legend-Click Mode (D-17)

**What goes wrong:** The drawer currently requires a `paper_id` to open (see `DrawerOpenRequest.paper_id`). Legend-click opens on a community without a selected paper — the existing `DrawerContent` component would have nothing to render for Overview/Source/Similar tabs.

**Why it happens:** `DrawerOpenRequest.paper_id` is non-optional in the current `app.rs`.

**How to avoid:** Either (a) make `paper_id` optional in `DrawerOpenRequest` and add a `community_id: Option<u32>` field, or (b) add a separate `CommunityDrawerOpenRequest` path. The cleanest approach is (a): extend `DrawerOpenRequest` with an optional `community_id`, and in `DrawerContent` guard the Overview/Source/Similar tabs with "Select a paper to view details." when `paper_id` is empty. [VERIFIED: app.rs source + CONTEXT.md D-17]

### Pitfall 6: Color Lerp Not Triggering on Mode Switch

**What goes wrong:** Nodes stay at their old color when `ColorMode` changes, or snap instantly without animating.

**Why it happens:** `target_color` is updated reactively but `current_color` is only advanced in the RAF loop (same tick as `current_radius` lerp). If the RAF loop reads the signal after the frame where `target_color` was set, it will start lerping — but if the RAF loop is paused (simulation settled), it may not run.

**How to avoid:** Ensure the RAF loop continues to run even when the simulation is settled, or at minimum runs N additional frames after `target_color` changes. The existing `check_alpha_convergence()` stops `simulation_running` but the RAF loop should remain active for rendering (existing behavior post-Phase 23 is that the RAF loop keeps running after settlement to handle hover/selection states).

---

## Code Examples

Verified patterns from official sources:

### single-clustering Louvain Usage

```rust
// Source: docs.rs/single-clustering/0.6.1 [VERIFIED]
use single_clustering::community_search::louvain::Louvain;

// Build network from edge list
let network = Louvain::build_network(n_nodes, n_edges, edge_iter);

// Create louvain instance with resolution=1.0 and deterministic seed
let mut louvain = Louvain::new(1.0_f64, Some(42_u64));

// VectorGrouping — community assignments by node index
let mut clustering = single_clustering::moving::VectorGrouping::new(n_nodes);

// Run until convergence
louvain.iterate(&network, &mut clustering);

// Access community assignments: clustering.group_of(node_idx) -> community_id
```

### Hybrid Ranking (D-19)

```rust
// Source: CONTEXT.md D-19 — no library needed, pure arithmetic
// PageRank scores from graph_metrics table; intra-community degree computed at runtime
fn hybrid_score(pagerank: f32, intra_degree: usize) -> f32 {
    pagerank * (intra_degree as f32 + 1.0)  // +1 avoids multiplying by zero
}
```

### Node Color Resolution in Renderers

```rust
// Source: canvas_renderer.rs lines 279-287 (existing fill_color logic to refactor)
// After refactor, the fill pipeline becomes:
let base_fill: [f32; 3] = match color_mode {
    ColorMode::Community => community_color(node.community_id, community_palette),
    ColorMode::BfsDepth  => bfs_depth_color(node.bfs_depth),
    ColorMode::Topic     => topic_color(&node.top_keywords, palette),
};
// Lerp-override for dimmed/hover/selected states takes priority:
let fill_color = if dimmed && !is_selected && !is_hovered {
    [0.165, 0.227, 0.310]  // "#2a3a4f"
} else if is_hovered || is_selected {
    [0.345, 0.651, 1.0]    // "#58a6ff"
} else {
    base_fill
};
// Lerp current_color toward fill_color each frame
```

### SurrealDB Community Query Pattern

```surreal
-- Source: existing schema.rs query patterns [VERIFIED: schema.rs migration 11]
-- Get all papers for a community:
SELECT arxiv_id FROM graph_communities WHERE community_id = $cid AND corpus_fingerprint = $fp;
-- Get single paper's community:
SELECT community_id FROM graph_communities WHERE arxiv_id = $id LIMIT 1;
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hardcoded node fill (4 branches: dimmed/hover/selected/seed) | Data-driven ColorMode fill with lerp | This phase | Renderer refactor required in both canvas_renderer.rs and webgl_renderer.rs |
| No community structure | Louvain modularity communities stored in DB | This phase | New `graph_communities` table (migration 12) |
| c-TF-IDF had to aggregate per-paper TF-IDF | Existing `PaperAnalysis.tfidf_vector` from Phase 22 available | Phase 22 | No new text extraction needed for community keywords |
| DrawerTab had 3 tabs (Overview/Source/Similar) | DrawerTab has 4 tabs (+ Community) | This phase | `DrawerTab` enum extension in `app.rs` + drawer.rs rendering |

**Deprecated/outdated (within this project):**
- The hardcoded `"#4a9eff"` / `"#d29922"` fill branches in both renderers: replaced by `ColorMode`-driven fill pipeline. The seed-node gold ring decoration (`is_seed` ring) survives but the fill color for seed nodes is now handled by `ColorMode` (seed paper will be part of some community, colored accordingly; its ring remains gold).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `VectorGrouping::new(n_nodes)` is the correct constructor; `clustering.group_of(node_idx)` returns community id | Architecture Patterns §1 | Would require reading single-clustering source; low risk — docs confirm `VectorGrouping` is the output type |
| A2 | Cargo resolves petgraph 0.7 + 0.8 concurrently without build failure | Standard Stack | If cargo refuses, fall back to custom Louvain or upgrade workspace petgraph to 0.8 |
| A3 | RAF loop continues running after simulation settles (for hover/selection rendering) | Pitfall 6 | If RAF stops, color lerp won't animate; requires verifying graph.rs RAF loop behavior |
| A4 | `shared_high_weight_terms()` with `min_weight=0.05` yields useful community-level method terms (not empty) | Don't Hand-Roll | If empty for most communities, D-23 says "planner may reduce scope" |

---

## Open Questions

1. **VectorGrouping exact API**
   - What we know: docs.rs confirms `VectorGrouping` is the output type; `Louvain::iterate()` takes `&mut VectorGrouping`
   - What's unclear: exact method to read community assignment per node index (`.group_of(n)`, `.get(n)`, or index operator)
   - Recommendation: Check single-clustering source on GitHub during implementation; the struct API is straightforward

2. **petgraph 0.7 + 0.8 concurrent compilation**
   - What we know: Cargo allows multiple semver-incompatible versions of the same crate
   - What's unclear: Whether any transitive dep conflict arises (single-clustering also uses `rand 0.9.0` — workspace already pins rand 0.9.0, so this should be fine)
   - Recommendation: Test with `cargo check --package resyn-core` after adding single-clustering to confirm build succeeds

3. **single-clustering license**
   - What we know: `cargo info` shows license field empty; crates.io page does not show a license string
   - What's unclear: Whether it is MIT, Apache, or proprietary
   - Recommendation: Check the GitHub repository (https://github.com/SingleRust/single-clustering) for a LICENSE file before merging

4. **graph_communities community_id stability across Louvain runs**
   - What we know: Same seed + same adjacency → same Louvain partition (D-03)
   - What's unclear: Whether Louvain's internal community IDs are order-stable (community 0 always maps to the same partition) vs. permutation-stable (same partition, possibly different ID numbers)
   - Recommendation: Community-to-color mapping should be by SIZE descending (as per UI-SPEC), so even if raw community IDs shift, the largest community always gets index 0 (blue). This post-processing step is mandatory.

---

## Environment Availability

Step 2.6: SKIPPED (phase is pure code/config changes; all dependencies are Rust crates, no external services or CLI tools beyond what Cargo manages).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none (workspace defaults) |
| Quick run command | `cargo test -p resyn-core community 2>/dev/null` |
| Full suite command | `cargo test --all` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COMM-01 | Louvain assigns each paper to a community | unit | `cargo test -p resyn-core test_community_detection` | No — Wave 0 |
| COMM-01 | Small graph (<10 nodes) returns empty map | unit | `cargo test -p resyn-core test_community_detection_too_small` | No — Wave 0 |
| COMM-01 | Communities with <3 papers bucketed as "Other" | unit | `cargo test -p resyn-core test_community_other_bucket` | No — Wave 0 |
| COMM-01 | Fixed seed produces identical results on same graph | unit | `cargo test -p resyn-core test_community_determinism` | No — Wave 0 |
| COMM-01 | DB migration 12 creates graph_communities table | db integration | `cargo test -p resyn-core test_migration_12_creates_graph_communities_table` | No — Wave 0 |
| COMM-02 | ColorMode enum default is Community | unit | `cargo test -p resyn-app test_color_mode_default_is_community` | No — Wave 0 |
| COMM-02 | BFS depth color returns correct hex for depth 0 | unit | `cargo test -p resyn-app test_bfs_depth_color_seed` | No — Wave 0 |
| COMM-03 | c-TF-IDF returns higher scores for community-distinctive terms | unit | `cargo test -p resyn-core test_ctfidf_distinctive_terms` | No — Wave 0 |
| COMM-03 | Hybrid score = pagerank * (intra_degree + 1) | unit | `cargo test -p resyn-core test_hybrid_score` | No — Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p resyn-core community 2>/dev/null`
- **Per wave merge:** `cargo test --all`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `resyn-core/src/graph_analytics/community.rs` — covers COMM-01 unit tests
- [ ] `resyn-core/src/database/schema.rs` migration 12 test — covers COMM-01 DB test
- [ ] `resyn-app/src/graph/layout_state.rs` ColorMode tests — covers COMM-02
- [ ] `resyn-core/src/graph_analytics/community.rs` c-TF-IDF tests — covers COMM-03

---

## Security Domain

`security_enforcement` is not set to false in config.json (absent = enabled). This phase involves no authentication, no user input stored in DB beyond paper IDs already validated, no new HTTP endpoints beyond existing `/api` pattern, no cryptographic operations. ASVS categories apply minimally:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | No new auth surface |
| V3 Session Management | No | No sessions affected |
| V4 Access Control | No | No new access control decisions |
| V5 Input Validation | Minimal | `community_id: u32` is typed; `arxiv_id` uses existing validation |
| V6 Cryptography | No | No crypto; corpus fingerprint uses sha2 (already in use) |

No new threat patterns introduced. Existing server fn pattern applies.

---

## Project Constraints (from CLAUDE.md)

| Directive | Impact on Phase 24 |
|-----------|--------------------|
| Single `#[tokio::main]` in `main.rs` | Community compute runs inside `tokio::spawn` (matching Phase 23 metrics pattern in `analysis.rs`) |
| `cargo clippy --all-targets --all-features -Dwarnings` in CI | All new code must pass clippy; no dead code, no unused imports |
| `cargo fmt --all -- --check` in CI | Format all new `.rs` files before commit |
| `async fn` for all API/HTML functions | `CommunityRepository` methods must be `async` |
| `ResynError` for error handling with `?` propagation | Community compute functions return `Result<_, ResynError>` |
| arXiv rate limiting: 3s default | Not relevant to Phase 24 (no HTTP calls) |
| InspireHEP rate limiting: 350ms | Not relevant to Phase 24 |
| WASM-safe dependencies in `resyn-core` always-available deps | `single-clustering` must be behind `ssr` feature flag or confirmed WASM-compatible. **CRITICAL:** single-clustering uses `rayon` (parallel computation) which does NOT compile to WASM. Must be gated behind `ssr` feature in resyn-core. |
| `surrealdb kv-mem` for all DB tests | `test_migration_12_*` tests use `connect_memory()` |

**CRITICAL WASM constraint:** `single-clustering` depends on `rayon` (parallelism library) which cannot compile to WebAssembly. It must be added under the `ssr` feature gate in `resyn-core/Cargo.toml`, not as an always-available dependency. The `detect_communities()` function lives behind `#[cfg(feature = "ssr")]` guards. [VERIFIED: resyn-core/Cargo.toml ssr feature pattern + rayon WASM incompatibility is documented]

---

## Sources

### Primary (HIGH confidence)
- [docs.rs/single-clustering/0.6.1](https://docs.rs/single-clustering/0.6.1) — Louvain API (new, iterate, build_network, seed param, VectorGrouping)
- [crates.io API: single-clustering dependencies](https://crates.io/api/v1/crates/single-clustering/0.6.1/dependencies) — petgraph ^0.8.2 requirement
- Existing codebase: `schema.rs`, `graph_analytics/pagerank.rs`, `canvas_renderer.rs`, `webgl_renderer.rs`, `layout_state.rs`, `graph_controls.rs`, `app.rs`, `drawer.rs`, `analysis.rs`, `nlp/tfidf.rs`, `gap_analysis/similarity.rs` — all read directly
- `.planning/phases/24-community-detection/24-CONTEXT.md` — all locked decisions
- `.planning/phases/24-community-detection/24-UI-SPEC.md` — component contracts, palette hex values, interaction states, copywriting

### Secondary (MEDIUM confidence)
- [docs.rs/graphalgs/0.2.0](https://docs.rs/graphalgs/0.2.0) — confirmed graphalgs has no Louvain (rejection reason)
- [docs.rs/single-clustering network module](https://docs.rs/single-clustering/0.6.1/single_clustering/network/index.html) — Network is petgraph UnGraph-backed

### Tertiary (LOW confidence)
- None — all significant claims verified via tool or codebase inspection.

---

## Metadata

**Confidence breakdown:**
- Standard stack (Louvain crate): HIGH — API verified via docs.rs
- petgraph version coexistence: MEDIUM — Cargo behavior confirmed by general knowledge, not tested in this repo yet
- Architecture patterns: HIGH — derived directly from existing codebase patterns
- c-TF-IDF formula: MEDIUM — standard formula from IR literature, implementation details assumed
- WASM / ssr gate requirement: HIGH — rayon is documented as WASM-incompatible; existing ssr pattern in repo is the correct approach

**Research date:** 2026-04-10
**Valid until:** 2026-05-10 (single-clustering is a fast-moving crate — recheck if new version appears)
